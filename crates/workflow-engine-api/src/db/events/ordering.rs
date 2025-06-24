// File: src/db/events/ordering.rs
//
// Event ordering and deduplication system for reliable event processing

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{EventEnvelope, EventError, EventResult};

/// Event ordering strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderingStrategy {
    /// Events are processed in the order they arrive (FIFO)
    FirstInFirstOut,
    /// Events are ordered by timestamp
    Timestamp,
    /// Events are ordered by sequence number
    SequenceNumber,
    /// Events are ordered by priority and then timestamp
    PriorityTimestamp,
    /// Custom ordering based on partition key
    PartitionBased { partition_field: String },
}

/// Event deduplication strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeduplicationStrategy {
    /// No deduplication
    None,
    /// Deduplicate by event ID
    EventId,
    /// Deduplicate by content hash
    ContentHash,
    /// Deduplicate by custom key
    CustomKey { key_fields: Vec<String> },
    /// Deduplicate within a time window
    TimeWindow { window_seconds: u64 },
}

/// Configuration for event ordering and deduplication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderingConfig {
    pub ordering_strategy: OrderingStrategy,
    pub deduplication_strategy: DeduplicationStrategy,
    pub buffer_size: usize,
    pub max_out_of_order_delay_ms: u64,
    pub enable_strict_ordering: bool,
    pub partition_count: Option<usize>,
}

impl Default for OrderingConfig {
    fn default() -> Self {
        Self {
            ordering_strategy: OrderingStrategy::Timestamp,
            deduplication_strategy: DeduplicationStrategy::EventId,
            buffer_size: 1000,
            max_out_of_order_delay_ms: 5000,
            enable_strict_ordering: true,
            partition_count: Some(10),
        }
    }
}

/// Event with ordering metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderedEvent {
    pub event: EventEnvelope,
    pub sequence_number: i64,
    pub partition_key: String,
    pub priority: EventPriority,
    pub processing_deadline: DateTime<Utc>,
    pub deduplication_key: String,
}

/// Event priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Event ordering and deduplication processor
pub struct EventOrderingProcessor {
    config: OrderingConfig,
    sequence_generator: Arc<RwLock<i64>>,
    deduplication_cache: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
    partition_buffers: Arc<RwLock<HashMap<String, VecDeque<OrderedEvent>>>>,
    processing_stats: Arc<RwLock<ProcessingStatistics>>,
}

/// Processing statistics
#[derive(Debug, Clone, Default)]
pub struct ProcessingStatistics {
    pub total_events_processed: u64,
    pub duplicates_detected: u64,
    pub out_of_order_events: u64,
    pub events_buffered: u64,
    pub events_dropped: u64,
    pub average_processing_delay_ms: f64,
}

impl EventOrderingProcessor {
    /// Create a new event ordering processor
    pub fn new(config: OrderingConfig) -> Self {
        Self {
            config,
            sequence_generator: Arc::new(RwLock::new(0)),
            deduplication_cache: Arc::new(RwLock::new(HashMap::new())),
            partition_buffers: Arc::new(RwLock::new(HashMap::new())),
            processing_stats: Arc::new(RwLock::new(ProcessingStatistics::default())),
        }
    }
    
    /// Process an event through ordering and deduplication
    pub async fn process_event(&self, event: EventEnvelope) -> EventResult<Option<Vec<OrderedEvent>>> {
        let mut stats = self.processing_stats.write().await;
        stats.total_events_processed += 1;
        drop(stats);
        
        // Generate sequence number
        let sequence_number = self.get_next_sequence().await;
        
        // Determine partition key
        let partition_key = self.determine_partition_key(&event);
        
        // Determine priority
        let priority = self.determine_priority(&event);
        
        // Calculate processing deadline
        let processing_deadline = Utc::now() 
            + chrono::Duration::milliseconds(self.config.max_out_of_order_delay_ms as i64);
        
        // Generate deduplication key
        let deduplication_key = self.generate_deduplication_key(&event)?;
        
        // Check for duplicates
        if self.is_duplicate(&deduplication_key).await? {
            let mut stats = self.processing_stats.write().await;
            stats.duplicates_detected += 1;
            return Ok(None);
        }
        
        // Mark as processed for deduplication
        self.mark_as_processed(&deduplication_key).await?;
        
        let ordered_event = OrderedEvent {
            event,
            sequence_number,
            partition_key: partition_key.clone(),
            priority,
            processing_deadline,
            deduplication_key,
        };
        
        // Add to partition buffer
        self.add_to_partition_buffer(&partition_key, ordered_event).await?;
        
        // Try to flush ready events
        self.flush_ready_events(&partition_key).await
    }
    
    /// Get next sequence number
    async fn get_next_sequence(&self) -> i64 {
        let mut seq = self.sequence_generator.write().await;
        *seq += 1;
        *seq
    }
    
    /// Determine partition key for an event
    fn determine_partition_key(&self, event: &EventEnvelope) -> String {
        match &self.config.ordering_strategy {
            OrderingStrategy::PartitionBased { partition_field } => {
                // Extract partition key from event data
                if let Some(value) = event.event_data.get(partition_field) {
                    value.as_str().unwrap_or(&event.aggregate_id.to_string()).to_string()
                } else {
                    event.aggregate_id.to_string()
                }
            }
            _ => {
                // Use aggregate ID as default partition key
                event.aggregate_id.to_string()
            }
        }
    }
    
    /// Determine event priority
    fn determine_priority(&self, event: &EventEnvelope) -> EventPriority {
        // Priority based on event type
        if event.event_type.contains("critical") || event.event_type.contains("error") {
            EventPriority::Critical
        } else if event.event_type.contains("important") || event.event_type.contains("urgent") {
            EventPriority::High
        } else if event.event_type.contains("system") {
            EventPriority::High
        } else {
            EventPriority::Normal
        }
    }
    
    /// Generate deduplication key
    fn generate_deduplication_key(&self, event: &EventEnvelope) -> EventResult<String> {
        match &self.config.deduplication_strategy {
            DeduplicationStrategy::None => Ok(format!("no-dedup-{}", event.event_id)),
            DeduplicationStrategy::EventId => Ok(event.event_id.to_string()),
            DeduplicationStrategy::ContentHash => {
                let content = serde_json::to_string(&event.event_data)
                    .map_err(|e| EventError::SerializationError {
                        message: format!("Failed to serialize event data: {}", e),
                    })?;
                Ok(format!("{:x}", md5::compute(content)))
            }
            DeduplicationStrategy::CustomKey { key_fields } => {
                let mut key_parts = Vec::new();
                for field in key_fields {
                    if let Some(value) = event.event_data.get(field) {
                        key_parts.push(value.to_string());
                    }
                }
                if key_parts.is_empty() {
                    key_parts.push(event.event_id.to_string());
                }
                Ok(key_parts.join(":"))
            }
            DeduplicationStrategy::TimeWindow { .. } => {
                // Combine event type and aggregate ID for time-window based deduplication
                Ok(format!("{}:{}", event.event_type, event.aggregate_id))
            }
        }
    }
    
    /// Check if event is a duplicate
    async fn is_duplicate(&self, deduplication_key: &str) -> EventResult<bool> {
        let cache = self.deduplication_cache.read().await;
        
        if let Some(processed_at) = cache.get(deduplication_key) {
            match &self.config.deduplication_strategy {
                DeduplicationStrategy::TimeWindow { window_seconds } => {
                    let window_duration = chrono::Duration::seconds(*window_seconds as i64);
                    let is_within_window = Utc::now() - *processed_at < window_duration;
                    Ok(is_within_window)
                }
                _ => Ok(true),
            }
        } else {
            Ok(false)
        }
    }
    
    /// Mark event as processed for deduplication
    async fn mark_as_processed(&self, deduplication_key: &str) -> EventResult<()> {
        let mut cache = self.deduplication_cache.write().await;
        cache.insert(deduplication_key.to_string(), Utc::now());
        
        // Clean up old entries to prevent memory leak
        if cache.len() > 10000 {
            let cutoff = Utc::now() - chrono::Duration::hours(24);
            cache.retain(|_, timestamp| *timestamp > cutoff);
        }
        
        Ok(())
    }
    
    /// Add event to partition buffer
    async fn add_to_partition_buffer(
        &self,
        partition_key: &str,
        ordered_event: OrderedEvent,
    ) -> EventResult<()> {
        let mut buffers = self.partition_buffers.write().await;
        let buffer = buffers.entry(partition_key.to_string()).or_insert_with(VecDeque::new);
        
        // Insert in order based on strategy
        let insert_position = match &self.config.ordering_strategy {
            OrderingStrategy::FirstInFirstOut => buffer.len(),
            OrderingStrategy::Timestamp => {
                buffer.iter().position(|e| e.event.occurred_at > ordered_event.event.occurred_at)
                    .unwrap_or(buffer.len())
            }
            OrderingStrategy::SequenceNumber => {
                buffer.iter().position(|e| e.sequence_number > ordered_event.sequence_number)
                    .unwrap_or(buffer.len())
            }
            OrderingStrategy::PriorityTimestamp => {
                buffer.iter().position(|e| {
                    e.priority < ordered_event.priority ||
                    (e.priority == ordered_event.priority && e.event.occurred_at > ordered_event.event.occurred_at)
                }).unwrap_or(buffer.len())
            }
            OrderingStrategy::PartitionBased { .. } => {
                // Within partition, use timestamp
                buffer.iter().position(|e| e.event.occurred_at > ordered_event.event.occurred_at)
                    .unwrap_or(buffer.len())
            }
        };
        
        buffer.insert(insert_position, ordered_event);
        
        // Limit buffer size
        if buffer.len() > self.config.buffer_size {
            buffer.pop_front();
            let mut stats = self.processing_stats.write().await;
            stats.events_dropped += 1;
        } else {
            let mut stats = self.processing_stats.write().await;
            stats.events_buffered += 1;
        }
        
        Ok(())
    }
    
    /// Flush events that are ready for processing
    async fn flush_ready_events(&self, partition_key: &str) -> EventResult<Option<Vec<OrderedEvent>>> {
        let mut buffers = self.partition_buffers.write().await;
        let buffer = match buffers.get_mut(partition_key) {
            Some(b) => b,
            None => return Ok(None),
        };
        
        let mut ready_events = Vec::new();
        let now = Utc::now();
        
        // Find events ready for processing
        while let Some(event) = buffer.front() {
            let should_process = if self.config.enable_strict_ordering {
                // In strict ordering, only process if it's the next in sequence
                self.is_next_in_sequence(partition_key, event).await
            } else {
                // In relaxed ordering, process if deadline passed or if it's the next in sequence
                now >= event.processing_deadline || self.is_next_in_sequence(partition_key, event).await
            };
            
            if should_process {
                if let Some(event) = buffer.pop_front() {
                    ready_events.push(event);
                }
            } else {
                break;
            }
        }
        
        if ready_events.is_empty() {
            Ok(None)
        } else {
            let mut stats = self.processing_stats.write().await;
            stats.events_buffered = stats.events_buffered.saturating_sub(ready_events.len() as u64);
            Ok(Some(ready_events))
        }
    }
    
    /// Check if event is the next in sequence for the partition
    async fn is_next_in_sequence(&self, _partition_key: &str, _event: &OrderedEvent) -> bool {
        // For now, assume it's always ready
        // In a real implementation, you'd track the last processed sequence number per partition
        true
    }
    
    /// Get processing statistics
    pub async fn get_statistics(&self) -> ProcessingStatistics {
        let stats = self.processing_stats.read().await;
        stats.clone()
    }
    
    /// Clean up expired entries and buffers
    pub async fn cleanup(&self) -> EventResult<()> {
        let now = Utc::now();
        
        // Clean up deduplication cache
        let mut cache = self.deduplication_cache.write().await;
        let cutoff = match &self.config.deduplication_strategy {
            DeduplicationStrategy::TimeWindow { window_seconds } => {
                now - chrono::Duration::seconds(*window_seconds as i64 * 2)
            }
            _ => now - chrono::Duration::hours(24),
        };
        
        let initial_size = cache.len();
        cache.retain(|_, timestamp| *timestamp > cutoff);
        let cleaned = initial_size - cache.len();
        
        tracing::debug!("Cleaned {} expired deduplication entries", cleaned);
        
        // Clean up expired events from buffers
        let mut buffers = self.partition_buffers.write().await;
        let mut total_expired = 0;
        
        for buffer in buffers.values_mut() {
            let initial_len = buffer.len();
            buffer.retain(|event| now < event.processing_deadline);
            total_expired += initial_len - buffer.len();
        }
        
        if total_expired > 0 {
            tracing::warn!("Dropped {} expired events from buffers", total_expired);
            let mut stats = self.processing_stats.write().await;
            stats.events_dropped += total_expired as u64;
        }
        
        Ok(())
    }
    
    /// Force flush all buffered events (for shutdown)
    pub async fn flush_all(&self) -> EventResult<Vec<OrderedEvent>> {
        let mut buffers = self.partition_buffers.write().await;
        let mut all_events = Vec::new();
        
        for buffer in buffers.values_mut() {
            while let Some(event) = buffer.pop_front() {
                all_events.push(event);
            }
        }
        
        // Sort by sequence number for final output
        all_events.sort_by_key(|e| e.sequence_number);
        
        Ok(all_events)
    }
}

/// Event ordering manager that coordinates multiple processors
pub struct EventOrderingManager {
    processors: Arc<RwLock<HashMap<String, Arc<EventOrderingProcessor>>>>,
    default_config: OrderingConfig,
}

impl EventOrderingManager {
    /// Create a new event ordering manager
    pub fn new(default_config: OrderingConfig) -> Self {
        Self {
            processors: Arc::new(RwLock::new(HashMap::new())),
            default_config,
        }
    }
    
    /// Register a processor for a specific event type or service
    pub async fn register_processor(
        &self,
        key: &str,
        config: Option<OrderingConfig>,
    ) -> Arc<EventOrderingProcessor> {
        let processor_config = config.unwrap_or_else(|| self.default_config.clone());
        let processor = Arc::new(EventOrderingProcessor::new(processor_config));
        
        let mut processors = self.processors.write().await;
        processors.insert(key.to_string(), Arc::clone(&processor));
        
        processor
    }
    
    /// Get processor for a key
    pub async fn get_processor(&self, key: &str) -> Option<Arc<EventOrderingProcessor>> {
        let processors = self.processors.read().await;
        processors.get(key).cloned()
    }
    
    /// Process event with appropriate processor
    pub async fn process_event(
        &self,
        event: EventEnvelope,
        processor_key: Option<&str>,
    ) -> EventResult<Option<Vec<OrderedEvent>>> {
        let key = processor_key.unwrap_or("default");
        
        // Get or create processor
        let processor = if let Some(p) = self.get_processor(key).await {
            p
        } else {
            self.register_processor(key, None).await
        };
        
        processor.process_event(event).await
    }
    
    /// Get combined statistics from all processors
    pub async fn get_combined_statistics(&self) -> HashMap<String, ProcessingStatistics> {
        let processors = self.processors.read().await;
        let mut stats = HashMap::new();
        
        for (key, processor) in processors.iter() {
            stats.insert(key.clone(), processor.get_statistics().await);
        }
        
        stats
    }
    
    /// Run cleanup on all processors
    pub async fn cleanup_all(&self) -> EventResult<()> {
        let processors = self.processors.read().await;
        
        for processor in processors.values() {
            processor.cleanup().await?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::events::EventMetadata;
    
    fn create_test_event(event_type: &str, aggregate_id: Uuid) -> EventEnvelope {
        EventEnvelope {
            event_id: Uuid::new_v4(),
            aggregate_id,
            aggregate_type: "test".to_string(),
            event_type: event_type.to_string(),
            aggregate_version: 1,
            event_data: serde_json::json!({"test": "data"}),
            metadata: EventMetadata {
                user_id: None,
                session_id: None,
                correlation_id: None,
                causation_id: None,
                source: None,
                tags: Default::default(),
                timestamp: Utc::now(),
                custom: Default::default(),
            },
            occurred_at: Utc::now(),
            recorded_at: Utc::now(),
            schema_version: 1,
            causation_id: None,
            correlation_id: None,
            checksum: None,
        }
    }
    
    #[tokio::test]
    async fn test_event_deduplication() {
        let config = OrderingConfig {
            deduplication_strategy: DeduplicationStrategy::EventId,
            ..OrderingConfig::default()
        };
        
        let processor = EventOrderingProcessor::new(config);
        let event = create_test_event("test_event", Uuid::new_v4());
        
        // First processing should succeed
        let result1 = processor.process_event(event.clone()).await.unwrap();
        assert!(result1.is_some());
        
        // Second processing should be deduplicated
        let result2 = processor.process_event(event.clone()).await.unwrap();
        assert!(result2.is_none());
        
        let stats = processor.get_statistics().await;
        assert_eq!(stats.duplicates_detected, 1);
    }
    
    #[tokio::test]
    async fn test_event_priority_ordering() {
        let config = OrderingConfig {
            ordering_strategy: OrderingStrategy::PriorityTimestamp,
            enable_strict_ordering: false,
            ..OrderingConfig::default()
        };
        
        let processor = EventOrderingProcessor::new(config);
        
        // Process normal priority event
        let normal_event = create_test_event("normal_event", Uuid::new_v4());
        processor.process_event(normal_event).await.unwrap();
        
        // Process critical priority event
        let critical_event = create_test_event("critical_error", Uuid::new_v4());
        let result = processor.process_event(critical_event).await.unwrap();
        
        // Critical event should be processed immediately
        assert!(result.is_some());
        let events = result.unwrap();
        assert!(events.iter().any(|e| e.priority == EventPriority::Critical));
    }
    
    #[tokio::test]
    async fn test_sequence_number_generation() {
        let processor = EventOrderingProcessor::new(OrderingConfig::default());
        
        let seq1 = processor.get_next_sequence().await;
        let seq2 = processor.get_next_sequence().await;
        
        assert_eq!(seq1, 1);
        assert_eq!(seq2, 2);
    }
    
    #[tokio::test]
    async fn test_ordering_manager() {
        let manager = EventOrderingManager::new(OrderingConfig::default());
        
        // Register processor for specific service
        let processor = manager.register_processor("service_a", None).await;
        assert!(Arc::ptr_eq(&processor, &manager.get_processor("service_a").await.unwrap()));
        
        // Process event through manager
        let event = create_test_event("test_event", Uuid::new_v4());
        let result = manager.process_event(event, Some("service_a")).await.unwrap();
        assert!(result.is_some());
    }
}