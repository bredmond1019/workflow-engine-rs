// File: src/db/events/dispatcher.rs
//
// Event dispatcher and handler system for real-time event processing

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use super::{EventError, EventResult, EventEnvelope, EventStore};
use super::dead_letter_queue::{DeadLetterQueue, PostgreSQLDeadLetterQueue, DeadLetterConfig};
use super::cross_service_routing::{CrossServiceEventRouter, ServiceRoutingConfig, RoutingMetadata};

/// Event handler trait for processing events
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Handle a single event
    async fn handle(&self, event: &EventEnvelope) -> EventResult<()>;
    
    /// Get the event types this handler is interested in
    fn event_types(&self) -> Vec<String>;
    
    /// Get the handler name for identification
    fn name(&self) -> &str;
    
    /// Check if this handler should process the given event
    fn should_handle(&self, event: &EventEnvelope) -> bool {
        self.event_types().contains(&event.event_type)
    }
}

/// Event subscription configuration
#[derive(Debug, Clone)]
pub struct EventSubscription {
    pub subscription_id: Uuid,
    pub name: String,
    pub event_types: Vec<String>,
    pub filter_criteria: HashMap<String, serde_json::Value>,
    pub max_retry_count: i32,
    pub dead_letter_enabled: bool,
}

impl EventSubscription {
    pub fn new(name: String, event_types: Vec<String>) -> Self {
        Self {
            subscription_id: Uuid::new_v4(),
            name,
            event_types,
            filter_criteria: HashMap::new(),
            max_retry_count: 3,
            dead_letter_enabled: true,
        }
    }
    
    pub fn with_filter(mut self, key: String, value: serde_json::Value) -> Self {
        self.filter_criteria.insert(key, value);
        self
    }
    
    pub fn with_retry_config(mut self, max_retries: i32, dead_letter: bool) -> Self {
        self.max_retry_count = max_retries;
        self.dead_letter_enabled = dead_letter;
        self
    }
    
    /// Check if an event matches this subscription
    pub fn matches(&self, event: &EventEnvelope) -> bool {
        // Check event type
        if !self.event_types.contains(&event.event_type) {
            return false;
        }
        
        // Check filter criteria
        for (key, expected_value) in &self.filter_criteria {
            if let Some(actual_value) = event.metadata.custom.get(key) {
                if actual_value != expected_value {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        true
    }
}

/// Event dispatcher for broadcasting events to handlers
pub struct EventDispatcher {
    handlers: Arc<RwLock<HashMap<String, Arc<dyn EventHandler>>>>,
    subscriptions: Arc<RwLock<HashMap<Uuid, EventSubscription>>>,
    event_sender: broadcast::Sender<EventEnvelope>,
    event_store: Arc<dyn EventStore>,
    dead_letter_queue: Option<Arc<dyn DeadLetterQueue>>,
    cross_service_router: Option<Arc<CrossServiceEventRouter>>,
    service_name: String,
}

impl EventDispatcher {
    /// Create a new event dispatcher
    pub fn new(event_store: Arc<dyn EventStore>) -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
            event_store,
            dead_letter_queue: None,
            cross_service_router: None,
            service_name: "default".to_string(),
        }
    }
    
    /// Create a new event dispatcher with dead letter queue
    pub fn with_dead_letter_queue(
        event_store: Arc<dyn EventStore>,
        dead_letter_queue: Arc<dyn DeadLetterQueue>,
    ) -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
            event_store,
            dead_letter_queue: Some(dead_letter_queue),
            cross_service_router: None,
            service_name: "default".to_string(),
        }
    }
    
    /// Create a new event dispatcher with cross-service routing
    pub fn with_cross_service_routing(
        event_store: Arc<dyn EventStore>,
        dead_letter_queue: Arc<dyn DeadLetterQueue>,
        cross_service_router: Arc<CrossServiceEventRouter>,
        service_name: String,
    ) -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
            event_store,
            dead_letter_queue: Some(dead_letter_queue),
            cross_service_router: Some(cross_service_router),
            service_name,
        }
    }
    
    /// Register an event handler
    pub async fn register_handler(&self, handler: Arc<dyn EventHandler>) -> EventResult<()> {
        let mut handlers = self.handlers.write().await;
        handlers.insert(handler.name().to_string(), handler);
        Ok(())
    }
    
    /// Unregister an event handler
    pub async fn unregister_handler(&self, handler_name: &str) -> EventResult<()> {
        let mut handlers = self.handlers.write().await;
        handlers.remove(handler_name);
        Ok(())
    }
    
    /// Create a subscription for event filtering
    pub async fn create_subscription(&self, subscription: EventSubscription) -> EventResult<Uuid> {
        let subscription_id = subscription.subscription_id;
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.insert(subscription_id, subscription);
        Ok(subscription_id)
    }
    
    /// Remove a subscription
    pub async fn remove_subscription(&self, subscription_id: Uuid) -> EventResult<()> {
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.remove(&subscription_id);
        Ok(())
    }
    
    /// Dispatch a single event to all matching handlers
    pub async fn dispatch(&self, event: &EventEnvelope) -> EventResult<()> {
        // First, persist the event to the store
        self.event_store.append_event(event).await?;
        
        // Route to other services if cross-service routing is enabled
        if let Some(ref router) = self.cross_service_router {
            match router.route_event(event, &self.service_name).await {
                Ok(routed_services) => {
                    if !routed_services.is_empty() {
                        tracing::debug!(
                            "Event {} routed to services: {:?}",
                            event.event_id,
                            routed_services
                        );
                    }
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to route event {} across services: {}",
                        event.event_id,
                        e
                    );
                }
            }
        }
        
        // Then dispatch to handlers
        let handlers = self.handlers.read().await;
        let subscriptions = self.subscriptions.read().await;
        
        // Send event to broadcast channel for real-time streaming
        if let Err(e) = self.event_sender.send(event.clone()) {
            tracing::warn!("Failed to broadcast event: {}", e);
        }
        
        // Process event through handlers
        for handler in handlers.values() {
            if handler.should_handle(event) {
                // Check if event matches any subscriptions for this handler
                let should_process = subscriptions.values().any(|sub| sub.matches(event));
                
                if should_process || subscriptions.is_empty() {
                    if let Err(e) = handler.handle(event).await {
                        tracing::error!(
                            "Handler '{}' failed to process event {}: {}",
                            handler.name(),
                            event.event_id,
                            e
                        );
                        
                        // Handle the error based on subscription configuration
                        self.handle_processing_error(event, handler.name(), e).await?;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Dispatch multiple events in batch
    pub async fn dispatch_batch(&self, events: &[EventEnvelope]) -> EventResult<()> {
        // First, persist all events to the store atomically
        self.event_store.append_events(events).await?;
        
        // Then dispatch each event to handlers
        for event in events {
            // Send event to broadcast channel
            if let Err(e) = self.event_sender.send(event.clone()) {
                tracing::warn!("Failed to broadcast event in batch: {}", e);
            }
            
            // Process through handlers (without re-persisting)
            self.dispatch_to_handlers(event).await?;
        }
        
        Ok(())
    }
    
    /// Dispatch event to handlers without persisting to store
    async fn dispatch_to_handlers(&self, event: &EventEnvelope) -> EventResult<()> {
        let handlers = self.handlers.read().await;
        let subscriptions = self.subscriptions.read().await;
        
        for handler in handlers.values() {
            if handler.should_handle(event) {
                let should_process = subscriptions.values().any(|sub| sub.matches(event));
                
                if should_process || subscriptions.is_empty() {
                    if let Err(e) = handler.handle(event).await {
                        tracing::error!(
                            "Handler '{}' failed to process event {}: {}",
                            handler.name(),
                            event.event_id,
                            e
                        );
                        
                        self.handle_processing_error(event, handler.name(), e).await?;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Handle event processing errors
    async fn handle_processing_error(
        &self,
        event: &EventEnvelope,
        handler_name: &str,
        error: EventError,
    ) -> EventResult<()> {
        // For now, just log the error
        // In a full implementation, this would:
        // 1. Check retry configuration
        // 2. Increment retry count
        // 3. Send to dead letter queue if max retries exceeded
        // 4. Schedule retry if within limits
        
        tracing::error!(
            "Event processing error in handler '{}' for event {}: {}",
            handler_name,
            event.event_id,
            error
        );
        
        // Add to dead letter queue if configured
        if let Some(ref dlq) = self.dead_letter_queue {
            if let Err(dlq_error) = dlq.add_failed_event(
                event,
                error.to_string(),
                serde_json::json!({
                    "handler": handler_name,
                    "error_type": "processing_error",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "original_error": error.to_string()
                }),
            ).await {
                tracing::error!(
                    "Failed to add event {} to dead letter queue: {}",
                    event.event_id,
                    dlq_error
                );
            }
        }
        
        Ok(())
    }
    
    /// Get a subscriber for real-time event streaming
    pub fn subscribe(&self) -> broadcast::Receiver<EventEnvelope> {
        self.event_sender.subscribe()
    }
    
    /// Get all registered handlers
    pub async fn get_handlers(&self) -> Vec<String> {
        let handlers = self.handlers.read().await;
        handlers.keys().cloned().collect()
    }
    
    /// Get all active subscriptions
    pub async fn get_subscriptions(&self) -> Vec<EventSubscription> {
        let subscriptions = self.subscriptions.read().await;
        subscriptions.values().cloned().collect()
    }
    
    /// Replay events from a specific position to rebuild projections
    pub async fn replay_events(&self, from_position: i64, batch_size: usize) -> EventResult<()> {
        let mut current_position = from_position;
        
        loop {
            let events = self.event_store
                .get_events_from_position(current_position, batch_size)
                .await?;
            
            if events.is_empty() {
                break;
            }
            
            // Process events through handlers without re-persisting
            for event in &events {
                self.dispatch_to_handlers(event).await?;
            }
            
            // Update position for next batch
            current_position = events.last().unwrap().recorded_at.timestamp();
        }
        
        Ok(())
    }
}

/// Simple logging event handler for debugging
pub struct LoggingEventHandler {
    name: String,
    event_types: Vec<String>,
}

impl LoggingEventHandler {
    pub fn new(name: String, event_types: Vec<String>) -> Self {
        Self { name, event_types }
    }
}

#[async_trait]
impl EventHandler for LoggingEventHandler {
    async fn handle(&self, event: &EventEnvelope) -> EventResult<()> {
        tracing::info!(
            "Handler '{}' processing event: {} (type: {}, aggregate: {})",
            self.name,
            event.event_id,
            event.event_type,
            event.aggregate_id
        );
        Ok(())
    }
    
    fn event_types(&self) -> Vec<String> {
        self.event_types.clone()
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// Metrics collection event handler
pub struct MetricsEventHandler {
    name: String,
}

impl MetricsEventHandler {
    pub fn new() -> Self {
        Self {
            name: "metrics_handler".to_string(),
        }
    }
}

#[async_trait]
impl EventHandler for MetricsEventHandler {
    async fn handle(&self, event: &EventEnvelope) -> EventResult<()> {
        // Increment event counters by type
        // This would integrate with your monitoring system
        tracing::debug!(
            "Metrics: Event {} of type '{}' processed",
            event.event_id,
            event.event_type
        );
        
        // Integrate with actual metrics collection system
        use crate::monitoring::metrics::{WORKFLOWS_TRIGGERED_TOTAL, AI_REQUESTS_TOTAL, CROSS_SYSTEM_CALLS_TOTAL};
        
        // Increment event counters by type
        match event.event_type.as_str() {
            "workflow_event" => {
                // Extract workflow name from metadata or use default
                let workflow_name = event.metadata.custom
                    .get("workflow_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let status = event.metadata.custom
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("processed");
                
                WORKFLOWS_TRIGGERED_TOTAL
                    .with_label_values(&[workflow_name, status])
                    .inc();
            },
            "ai_interaction_event" => {
                // Extract AI provider and model from metadata
                let provider = event.metadata.custom
                    .get("provider")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let model = event.metadata.custom
                    .get("model")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let status = event.metadata.custom
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("processed");
                
                AI_REQUESTS_TOTAL
                    .with_label_values(&[provider, model, status])
                    .inc();
            },
            "service_call_event" => {
                // Extract target system and operation from metadata
                let target_system = event.metadata.custom
                    .get("target_system")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let operation = event.metadata.custom
                    .get("operation")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let status = event.metadata.custom
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("processed");
                
                CROSS_SYSTEM_CALLS_TOTAL
                    .with_label_values(&[target_system, operation, status])
                    .inc();
            },
            _ => {
                // Generic event counter for other event types
                tracing::debug!("Processing generic event type: {}", event.event_type);
            }
        }
        
        Ok(())
    }
    
    fn event_types(&self) -> Vec<String> {
        // Process all event types for metrics
        vec!["*".to_string()]
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn should_handle(&self, _event: &EventEnvelope) -> bool {
        // Always handle all events for metrics
        true
    }
}