// File: src/db/events/streaming.rs
//
// Real-time event streaming capabilities for live event processing

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};
use tokio::time::interval;
use uuid::Uuid;

use super::{EventError, EventResult, EventEnvelope, EventStore, EventHandler};

/// Position in the event stream
pub type StreamPosition = i64;

/// Configuration for event streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStreamConfig {
    /// Name of the stream
    pub stream_name: String,
    
    /// Event types to include in the stream
    pub event_types: Vec<String>,
    
    /// Filter criteria for events
    pub filters: std::collections::HashMap<String, serde_json::Value>,
    
    /// Batch size for polling
    pub batch_size: usize,
    
    /// Polling interval in milliseconds
    pub poll_interval_ms: u64,
    
    /// Starting position (0 for beginning, -1 for end)
    pub start_position: StreamPosition,
    
    /// Whether to include existing events or only new ones
    pub include_existing: bool,
}

impl Default for EventStreamConfig {
    fn default() -> Self {
        Self {
            stream_name: "default_stream".to_string(),
            event_types: vec!["*".to_string()],
            filters: std::collections::HashMap::new(),
            batch_size: 100,
            poll_interval_ms: 1000,
            start_position: -1, // Start from end
            include_existing: false,
        }
    }
}

impl EventStreamConfig {
    /// Create a new stream config with name
    pub fn new(stream_name: String) -> Self {
        Self {
            stream_name,
            ..Default::default()
        }
    }
    
    /// Set event types to filter
    pub fn with_event_types(mut self, event_types: Vec<String>) -> Self {
        self.event_types = event_types;
        self
    }
    
    /// Add a filter condition
    pub fn with_filter(mut self, key: String, value: serde_json::Value) -> Self {
        self.filters.insert(key, value);
        self
    }
    
    /// Set batch size
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }
    
    /// Set polling interval
    pub fn with_poll_interval(mut self, interval_ms: u64) -> Self {
        self.poll_interval_ms = interval_ms;
        self
    }
    
    /// Set starting position
    pub fn from_position(mut self, position: StreamPosition) -> Self {
        self.start_position = position;
        self.include_existing = position >= 0;
        self
    }
    
    /// Start from the beginning
    pub fn from_beginning(mut self) -> Self {
        self.start_position = 0;
        self.include_existing = true;
        self
    }
    
    /// Start from the end (only new events)
    pub fn from_end(mut self) -> Self {
        self.start_position = -1;
        self.include_existing = false;
        self
    }
    
    /// Check if an event matches this stream's criteria
    pub fn matches(&self, event: &EventEnvelope) -> bool {
        // Check event types
        if !self.event_types.contains(&"*".to_string()) && 
           !self.event_types.contains(&event.event_type) {
            return false;
        }
        
        // Check filters
        for (key, expected_value) in &self.filters {
            match key.as_str() {
                "aggregate_type" => {
                    if let Some(value) = expected_value.as_str() {
                        if event.aggregate_type != value {
                            return false;
                        }
                    }
                }
                "aggregate_id" => {
                    if let Some(value) = expected_value.as_str() {
                        if let Ok(uuid) = Uuid::parse_str(value) {
                            if event.aggregate_id != uuid {
                                return false;
                            }
                        }
                    }
                }
                _ => {
                    // Check in metadata
                    if let Some(actual_value) = event.metadata.custom.get(key) {
                        if actual_value != expected_value {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
            }
        }
        
        true
    }
}

/// Event stream subscriber trait
#[async_trait]
pub trait EventStreamSubscriber: Send + Sync {
    /// Handle a batch of events from the stream
    async fn handle_events(&mut self, events: Vec<EventEnvelope>) -> EventResult<()>;
    
    /// Handle stream errors
    async fn handle_error(&mut self, error: EventError) -> EventResult<()>;
    
    /// Get subscriber name for identification
    fn name(&self) -> &str;
}

/// Event stream for real-time event processing
pub struct EventStream {
    config: EventStreamConfig,
    event_store: Arc<dyn EventStore>,
    current_position: Arc<RwLock<StreamPosition>>,
    subscribers: Arc<RwLock<Vec<Box<dyn EventStreamSubscriber>>>>,
    is_running: Arc<RwLock<bool>>,
    stop_sender: Option<broadcast::Sender<()>>,
}

impl EventStream {
    /// Create a new event stream
    pub fn new(config: EventStreamConfig, event_store: Arc<dyn EventStore>) -> Self {
        let initial_position = if config.start_position >= 0 {
            config.start_position
        } else {
            // Start from current position (end of stream)
            chrono::Utc::now().timestamp()
        };
        
        Self {
            config,
            event_store,
            current_position: Arc::new(RwLock::new(initial_position)),
            subscribers: Arc::new(RwLock::new(Vec::new())),
            is_running: Arc::new(RwLock::new(false)),
            stop_sender: None,
        }
    }
    
    /// Add a subscriber to the stream
    pub async fn subscribe(&mut self, subscriber: Box<dyn EventStreamSubscriber>) {
        let mut subscribers = self.subscribers.write().await;
        subscribers.push(subscriber);
    }
    
    /// Remove a subscriber from the stream
    pub async fn unsubscribe(&mut self, subscriber_name: &str) {
        let mut subscribers = self.subscribers.write().await;
        subscribers.retain(|s| s.name() != subscriber_name);
    }
    
    /// Start the event stream
    pub async fn start(&mut self) -> EventResult<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Ok(()); // Already running
        }
        
        *is_running = true;
        drop(is_running);
        
        let (stop_sender, mut stop_receiver) = broadcast::channel(1);
        self.stop_sender = Some(stop_sender);
        
        // Clone necessary data for the background task
        let config = self.config.clone();
        let event_store = Arc::clone(&self.event_store);
        let current_position = Arc::clone(&self.current_position);
        let subscribers = Arc::clone(&self.subscribers);
        let is_running = Arc::clone(&self.is_running);
        
        // Spawn background task for event polling
        tokio::spawn(async move {
            let mut poll_interval = interval(Duration::from_millis(config.poll_interval_ms));
            
            loop {
                // Check for stop signal
                if stop_receiver.try_recv().is_ok() {
                    break;
                }
                
                // Wait for next poll interval
                poll_interval.tick().await;
                
                // Poll for new events
                match Self::poll_events(
                    &config,
                    &event_store,
                    &current_position,
                    &subscribers,
                ).await {
                    Ok(processed_count) => {
                        if processed_count > 0 {
                            tracing::debug!(
                                "Stream '{}' processed {} events",
                                config.stream_name,
                                processed_count
                            );
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            "Error in stream '{}': {}",
                            config.stream_name,
                            e
                        );
                        
                        // Notify subscribers of error
                        let mut subs = subscribers.write().await;
                        for subscriber in subs.iter_mut() {
                            if let Err(handle_err) = subscriber.handle_error(e.clone()).await {
                                tracing::error!(
                                    "Subscriber '{}' failed to handle error: {}",
                                    subscriber.name(),
                                    handle_err
                                );
                            }
                        }
                    }
                }
                
                // Check if we should continue running
                let running = *is_running.read().await;
                if !running {
                    break;
                }
            }
            
            tracing::info!("Event stream '{}' stopped", config.stream_name);
        });
        
        tracing::info!("Event stream '{}' started", self.config.stream_name);
        Ok(())
    }
    
    /// Stop the event stream
    pub async fn stop(&mut self) -> EventResult<()> {
        let mut is_running = self.is_running.write().await;
        if !*is_running {
            return Ok(()); // Already stopped
        }
        
        *is_running = false;
        
        // Send stop signal
        if let Some(sender) = &self.stop_sender {
            let _ = sender.send(());
        }
        
        tracing::info!("Event stream '{}' stopping", self.config.stream_name);
        Ok(())
    }
    
    /// Poll for new events and process them
    async fn poll_events(
        config: &EventStreamConfig,
        event_store: &Arc<dyn EventStore>,
        current_position: &Arc<RwLock<StreamPosition>>,
        subscribers: &Arc<RwLock<Vec<Box<dyn EventStreamSubscriber>>>>,
    ) -> EventResult<usize> {
        let position = *current_position.read().await;
        
        // Get events from current position
        let all_events = event_store
            .get_events_from_position(position, config.batch_size)
            .await?;
        
        if all_events.is_empty() {
            return Ok(0);
        }
        
        // Filter events based on stream configuration
        let filtered_events: Vec<EventEnvelope> = all_events
            .into_iter()
            .filter(|event| config.matches(event))
            .collect();
        
        if filtered_events.is_empty() {
            // Update position even if no events matched
            if let Some(last_event) = filtered_events.last() {
                let mut pos = current_position.write().await;
                *pos = last_event.recorded_at.timestamp();
            }
            return Ok(0);
        }
        
        // Process events through subscribers
        let mut subs = subscribers.write().await;
        for subscriber in subs.iter_mut() {
            if let Err(e) = subscriber.handle_events(filtered_events.clone()).await {
                tracing::error!(
                    "Subscriber '{}' failed to handle events: {}",
                    subscriber.name(),
                    e
                );
            }
        }
        
        // Update current position
        if let Some(last_event) = filtered_events.last() {
            let mut pos = current_position.write().await;
            *pos = last_event.recorded_at.timestamp();
        }
        
        Ok(filtered_events.len())
    }
    
    /// Get current stream position
    pub async fn get_position(&self) -> StreamPosition {
        *self.current_position.read().await
    }
    
    /// Set stream position
    pub async fn set_position(&self, position: StreamPosition) {
        let mut pos = self.current_position.write().await;
        *pos = position;
    }
    
    /// Check if stream is running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }
    
    /// Get stream configuration
    pub fn config(&self) -> &EventStreamConfig {
        &self.config
    }
}

/// Event stream manager for handling multiple streams
pub struct EventStreamManager {
    streams: std::collections::HashMap<String, EventStream>,
    event_store: Arc<dyn EventStore>,
}

impl EventStreamManager {
    /// Create a new stream manager
    pub fn new(event_store: Arc<dyn EventStore>) -> Self {
        Self {
            streams: std::collections::HashMap::new(),
            event_store,
        }
    }
    
    /// Create and register a new stream
    pub fn create_stream(&mut self, config: EventStreamConfig) -> EventResult<()> {
        let stream_name = config.stream_name.clone();
        
        if self.streams.contains_key(&stream_name) {
            return Err(EventError::ConfigurationError {
                message: format!("Stream '{}' already exists", stream_name),
            });
        }
        
        let stream = EventStream::new(config, Arc::clone(&self.event_store));
        self.streams.insert(stream_name, stream);
        
        Ok(())
    }
    
    /// Get a mutable reference to a stream
    pub fn get_stream_mut(&mut self, stream_name: &str) -> Option<&mut EventStream> {
        self.streams.get_mut(stream_name)
    }
    
    /// Start a stream
    pub async fn start_stream(&mut self, stream_name: &str) -> EventResult<()> {
        if let Some(stream) = self.streams.get_mut(stream_name) {
            stream.start().await
        } else {
            Err(EventError::ConfigurationError {
                message: format!("Stream '{}' not found", stream_name),
            })
        }
    }
    
    /// Stop a stream
    pub async fn stop_stream(&mut self, stream_name: &str) -> EventResult<()> {
        if let Some(stream) = self.streams.get_mut(stream_name) {
            stream.stop().await
        } else {
            Err(EventError::ConfigurationError {
                message: format!("Stream '{}' not found", stream_name),
            })
        }
    }
    
    /// Remove a stream
    pub async fn remove_stream(&mut self, stream_name: &str) -> EventResult<()> {
        if let Some(mut stream) = self.streams.remove(stream_name) {
            stream.stop().await?;
        }
        Ok(())
    }
    
    /// Start all streams
    pub async fn start_all(&mut self) -> EventResult<()> {
        let stream_names: Vec<String> = self.streams.keys().cloned().collect();
        
        for stream_name in stream_names {
            self.start_stream(&stream_name).await?;
        }
        
        Ok(())
    }
    
    /// Stop all streams
    pub async fn stop_all(&mut self) -> EventResult<()> {
        let stream_names: Vec<String> = self.streams.keys().cloned().collect();
        
        for stream_name in stream_names {
            self.stop_stream(&stream_name).await?;
        }
        
        Ok(())
    }
    
    /// Get list of all stream names
    pub fn list_streams(&self) -> Vec<String> {
        self.streams.keys().cloned().collect()
    }
}

/// Simple logging subscriber for debugging
pub struct LoggingSubscriber {
    name: String,
}

impl LoggingSubscriber {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[async_trait]
impl EventStreamSubscriber for LoggingSubscriber {
    async fn handle_events(&mut self, events: Vec<EventEnvelope>) -> EventResult<()> {
        for event in events {
            tracing::info!(
                "Subscriber '{}' received event: {} (type: {}, aggregate: {})",
                self.name,
                event.event_id,
                event.event_type,
                event.aggregate_id
            );
        }
        Ok(())
    }
    
    async fn handle_error(&mut self, error: EventError) -> EventResult<()> {
        tracing::error!("Subscriber '{}' received error: {}", self.name, error);
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// Event handler adapter for using EventHandler as a stream subscriber
pub struct EventHandlerSubscriber {
    name: String,
    handler: Arc<dyn EventHandler>,
}

impl EventHandlerSubscriber {
    pub fn new(name: String, handler: Arc<dyn EventHandler>) -> Self {
        Self { name, handler }
    }
}

#[async_trait]
impl EventStreamSubscriber for EventHandlerSubscriber {
    async fn handle_events(&mut self, events: Vec<EventEnvelope>) -> EventResult<()> {
        for event in events {
            if self.handler.should_handle(&event) {
                self.handler.handle(&event).await?;
            }
        }
        Ok(())
    }
    
    async fn handle_error(&mut self, error: EventError) -> EventResult<()> {
        tracing::error!("Handler subscriber '{}' received error: {}", self.name, error);
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}