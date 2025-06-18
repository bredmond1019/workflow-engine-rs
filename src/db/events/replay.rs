// File: src/db/events/replay.rs
//
// Event replay functionality for rebuilding projections and state
// Provides efficient batch processing with position tracking

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{info, warn, error};
use uuid::Uuid;

use super::{
    EventStore, EventEnvelope, EventError, EventResult, EventMetadata,
    PostgreSQLEventStore, AggregateSnapshot
};

/// Configuration for event replay operations
#[derive(Debug, Clone)]
pub struct ReplayConfig {
    /// Number of events to process in each batch
    pub batch_size: usize,
    /// Whether to use snapshots when available
    pub use_snapshots: bool,
    /// Maximum number of parallel replay tasks
    pub parallelism: usize,
    /// Checkpoint frequency (save position every N events)
    pub checkpoint_frequency: usize,
    /// Timeout for processing a batch (in seconds)
    pub batch_timeout_seconds: u64,
}

impl Default for ReplayConfig {
    fn default() -> Self {
        Self {
            batch_size: 1000,
            use_snapshots: true,
            parallelism: 4,
            checkpoint_frequency: 100,
            batch_timeout_seconds: 300,
        }
    }
}

/// Position tracking for event replay
#[derive(Debug, Clone)]
pub struct ReplayPosition {
    /// The consumer/projection name
    pub consumer_name: String,
    /// Last processed event timestamp (milliseconds)
    pub position: i64,
    /// Last processed event ID
    pub last_event_id: Option<Uuid>,
    /// Number of events processed
    pub events_processed: u64,
    /// Last checkpoint time
    pub last_checkpoint: DateTime<Utc>,
}

impl ReplayPosition {
    pub fn new(consumer_name: String) -> Self {
        Self {
            consumer_name,
            position: 0,
            last_event_id: None,
            events_processed: 0,
            last_checkpoint: Utc::now(),
        }
    }
}

/// Trait for handling replayed events
#[async_trait]
pub trait ReplayHandler: Send + Sync {
    /// Handle a batch of events
    async fn handle_events(&mut self, events: &[EventEnvelope]) -> EventResult<()>;
    
    /// Called when replay starts
    async fn on_replay_start(&mut self, from_position: i64) -> EventResult<()> {
        Ok(())
    }
    
    /// Called when replay completes
    async fn on_replay_complete(&mut self, events_processed: u64) -> EventResult<()> {
        Ok(())
    }
    
    /// Get the consumer name for position tracking
    fn consumer_name(&self) -> &str;
}

/// Event replay engine for batch processing with position tracking
pub struct EventReplayEngine {
    event_store: Arc<dyn EventStore>,
    config: ReplayConfig,
    positions: Arc<RwLock<HashMap<String, ReplayPosition>>>,
    checkpoints: Arc<Mutex<HashMap<String, DateTime<Utc>>>>,
}

impl EventReplayEngine {
    pub fn new(event_store: Arc<dyn EventStore>, config: ReplayConfig) -> Self {
        Self {
            event_store,
            config,
            positions: Arc::new(RwLock::new(HashMap::new())),
            checkpoints: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Replay events for a specific handler
    pub async fn replay_for_handler<H: ReplayHandler>(
        &self,
        handler: Arc<Mutex<H>>,
        event_types: Option<Vec<String>>,
    ) -> EventResult<ReplayPosition> {
        let consumer_name = {
            let h = handler.lock().await;
            h.consumer_name().to_string()
        };
        
        // Get or create position
        let mut position = self.get_or_create_position(&consumer_name).await?;
        
        // Notify handler of replay start
        {
            let mut h = handler.lock().await;
            h.on_replay_start(position.position).await?;
        }
        
        info!(
            "Starting event replay for consumer '{}' from position {}",
            consumer_name, position.position
        );
        
        let start_time = Utc::now();
        let mut total_events = 0u64;
        let mut last_checkpoint = Utc::now();
        
        loop {
            // Fetch next batch
            let events = self.event_store
                .replay_events(
                    position.position,
                    event_types.clone(),
                    self.config.batch_size,
                )
                .await?;
            
            if events.is_empty() {
                break;
            }
            
            let batch_size = events.len();
            
            // Update position to last event in batch
            if let Some(last_event) = events.last() {
                position.position = last_event.recorded_at.timestamp_millis();
                position.last_event_id = Some(last_event.event_id);
            }
            
            // Process batch with timeout
            let process_result = tokio::time::timeout(
                std::time::Duration::from_secs(self.config.batch_timeout_seconds),
                handler.lock().await.handle_events(&events)
            ).await;
            
            match process_result {
                Ok(Ok(())) => {
                    total_events += batch_size as u64;
                    position.events_processed += batch_size as u64;
                    
                    // Checkpoint if needed
                    if total_events % self.config.checkpoint_frequency as u64 == 0 {
                        self.save_checkpoint(&mut position).await?;
                        last_checkpoint = Utc::now();
                    }
                }
                Ok(Err(e)) => {
                    error!(
                        "Handler error processing batch for consumer '{}': {}",
                        consumer_name, e
                    );
                    return Err(e);
                }
                Err(_) => {
                    error!(
                        "Timeout processing batch for consumer '{}' after {} seconds",
                        consumer_name, self.config.batch_timeout_seconds
                    );
                    return Err(EventError::HandlerError {
                        message: format!("Batch processing timeout after {} seconds", 
                                       self.config.batch_timeout_seconds),
                    });
                }
            }
            
            // Log progress
            if total_events % 10_000 == 0 {
                let elapsed = (Utc::now() - start_time).num_seconds();
                let rate = if elapsed > 0 { total_events / elapsed as u64 } else { 0 };
                info!(
                    "Replay progress for '{}': {} events processed ({} events/sec)",
                    consumer_name, total_events, rate
                );
            }
        }
        
        // Final checkpoint
        self.save_checkpoint(&mut position).await?;
        
        // Notify handler of completion
        {
            let mut h = handler.lock().await;
            h.on_replay_complete(total_events).await?;
        }
        
        let elapsed = (Utc::now() - start_time).num_seconds();
        info!(
            "Completed replay for consumer '{}': {} events in {} seconds",
            consumer_name, total_events, elapsed
        );
        
        Ok(position)
    }
    
    /// Replay events for multiple handlers in parallel
    pub async fn replay_parallel<H: ReplayHandler + 'static>(
        &self,
        handlers: Vec<Arc<Mutex<H>>>,
        event_types: Option<Vec<String>>,
    ) -> EventResult<Vec<ReplayPosition>> {
        let mut tasks = Vec::new();
        
        for handler in handlers {
            let engine = self.clone();
            let types = event_types.clone();
            
            let task = tokio::spawn(async move {
                engine.replay_for_handler(handler, types).await
            });
            
            tasks.push(task);
        }
        
        let mut results = Vec::new();
        for task in tasks {
            match task.await {
                Ok(Ok(position)) => results.push(position),
                Ok(Err(e)) => return Err(e),
                Err(e) => return Err(EventError::HandlerError {
                    message: format!("Task join error: {}", e),
                }),
            }
        }
        
        Ok(results)
    }
    
    /// Replay events for a specific aggregate with snapshot support
    pub async fn replay_aggregate<F>(
        &self,
        aggregate_id: Uuid,
        mut from_version: i64,
        callback: F,
    ) -> EventResult<i64>
    where
        F: Fn(&EventEnvelope) -> EventResult<()>,
    {
        let mut last_version = from_version;
        
        // Try to load from snapshot if enabled
        if self.config.use_snapshots && from_version == 0 {
            if let Some(snapshot) = self.event_store.get_snapshot(aggregate_id).await? {
                info!(
                    "Loading aggregate {} from snapshot at version {}",
                    aggregate_id, snapshot.aggregate_version
                );
                from_version = snapshot.aggregate_version;
                last_version = snapshot.aggregate_version;
            }
        }
        
        // Load events after snapshot
        let events = if from_version > 0 {
            self.event_store.get_events_from_version(aggregate_id, from_version).await?
        } else {
            self.event_store.get_events(aggregate_id).await?
        };
        
        // Process events
        for event in &events {
            callback(event)?;
            last_version = event.aggregate_version;
        }
        
        info!(
            "Replayed {} events for aggregate {} (version {} -> {})",
            events.len(),
            aggregate_id,
            from_version,
            last_version
        );
        
        Ok(last_version)
    }
    
    /// Get or create a replay position
    async fn get_or_create_position(&self, consumer_name: &str) -> EventResult<ReplayPosition> {
        let positions = self.positions.read().await;
        
        if let Some(position) = positions.get(consumer_name) {
            Ok(position.clone())
        } else {
            drop(positions);
            
            let position = ReplayPosition::new(consumer_name.to_string());
            let mut positions = self.positions.write().await;
            positions.insert(consumer_name.to_string(), position.clone());
            
            Ok(position)
        }
    }
    
    /// Save checkpoint for a consumer
    async fn save_checkpoint(&self, position: &mut ReplayPosition) -> EventResult<()> {
        position.last_checkpoint = Utc::now();
        
        let mut positions = self.positions.write().await;
        positions.insert(position.consumer_name.clone(), position.clone());
        
        let mut checkpoints = self.checkpoints.lock().await;
        checkpoints.insert(position.consumer_name.clone(), position.last_checkpoint);
        
        info!(
            "Saved checkpoint for consumer '{}' at position {} ({} events)",
            position.consumer_name, position.position, position.events_processed
        );
        
        Ok(())
    }
    
    /// Get current replay status for all consumers
    pub async fn get_replay_status(&self) -> HashMap<String, ReplayPosition> {
        self.positions.read().await.clone()
    }
    
    /// Reset replay position for a consumer
    pub async fn reset_position(&self, consumer_name: &str) -> EventResult<()> {
        let mut positions = self.positions.write().await;
        positions.remove(consumer_name);
        
        let mut checkpoints = self.checkpoints.lock().await;
        checkpoints.remove(consumer_name);
        
        info!("Reset replay position for consumer '{}'", consumer_name);
        
        Ok(())
    }
}

impl Clone for EventReplayEngine {
    fn clone(&self) -> Self {
        Self {
            event_store: self.event_store.clone(),
            config: self.config.clone(),
            positions: self.positions.clone(),
            checkpoints: self.checkpoints.clone(),
        }
    }
}

/// Batch replay processor for high-throughput scenarios
pub struct BatchReplayProcessor {
    engine: EventReplayEngine,
    buffer_size: usize,
    flush_interval: std::time::Duration,
}

impl BatchReplayProcessor {
    pub fn new(engine: EventReplayEngine, buffer_size: usize, flush_interval_seconds: u64) -> Self {
        Self {
            engine,
            buffer_size,
            flush_interval: std::time::Duration::from_secs(flush_interval_seconds),
        }
    }
    
    /// Process events in batches with automatic flushing
    pub async fn process_with_buffer<F>(
        &self,
        from_position: i64,
        event_types: Option<Vec<String>>,
        mut processor: F,
    ) -> EventResult<i64>
    where
        F: FnMut(&[EventEnvelope]) -> EventResult<()>,
    {
        let mut buffer = Vec::with_capacity(self.buffer_size);
        let mut current_position = from_position;
        let mut last_flush = std::time::Instant::now();
        
        loop {
            let events = self.engine.event_store
                .replay_events(
                    current_position,
                    event_types.clone(),
                    self.engine.config.batch_size,
                )
                .await?;
            
            if events.is_empty() {
                // Flush remaining buffer
                if !buffer.is_empty() {
                    processor(&buffer)?;
                }
                break;
            }
            
            // Update position
            if let Some(last_event) = events.last() {
                current_position = last_event.recorded_at.timestamp_millis();
            }
            
            // Add to buffer
            buffer.extend(events);
            
            // Flush if buffer is full or timeout reached
            if buffer.len() >= self.buffer_size || last_flush.elapsed() >= self.flush_interval {
                processor(&buffer)?;
                buffer.clear();
                last_flush = std::time::Instant::now();
            }
        }
        
        Ok(current_position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    
    struct TestHandler {
        name: String,
        events_processed: Arc<AtomicU64>,
    }
    
    #[async_trait]
    impl ReplayHandler for TestHandler {
        async fn handle_events(&mut self, events: &[EventEnvelope]) -> EventResult<()> {
            self.events_processed.fetch_add(events.len() as u64, Ordering::SeqCst);
            Ok(())
        }
        
        fn consumer_name(&self) -> &str {
            &self.name
        }
    }
    
    #[tokio::test]
    async fn test_replay_position_tracking() {
        let position = ReplayPosition::new("test_consumer".to_string());
        assert_eq!(position.consumer_name, "test_consumer");
        assert_eq!(position.position, 0);
        assert_eq!(position.events_processed, 0);
        assert!(position.last_event_id.is_none());
    }
    
    #[test]
    fn test_replay_config_defaults() {
        let config = ReplayConfig::default();
        assert_eq!(config.batch_size, 1000);
        assert!(config.use_snapshots);
        assert_eq!(config.parallelism, 4);
        assert_eq!(config.checkpoint_frequency, 100);
        assert_eq!(config.batch_timeout_seconds, 300);
    }
}