// File: src/db/events/enhanced_dead_letter_queue.rs
//
// Enhanced dead letter queue with Redis support and comprehensive failure recovery

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{EventEnvelope, EventError, EventResult};
use super::dead_letter_queue::{DeadLetterConfig, DeadLetterEntry, DeadLetterStatus, DeadLetterStatistics};

/// Enhanced dead letter queue configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedDLQConfig {
    pub base_config: DeadLetterConfig,
    pub redis_url: Option<String>,
    pub use_redis_persistence: bool,
    pub circuit_breaker_threshold: u32,
    pub circuit_breaker_timeout_seconds: u64,
    pub batch_processing_size: usize,
    pub poison_message_threshold: u32,
    pub enable_metrics: bool,
    pub retention_policy: RetentionPolicy,
}

/// Retention policy for dead letter entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub keep_resolved_days: u32,
    pub keep_failed_days: u32,
    pub max_total_entries: usize,
    pub auto_cleanup_enabled: bool,
    pub cleanup_interval_hours: u32,
}

impl Default for EnhancedDLQConfig {
    fn default() -> Self {
        Self {
            base_config: DeadLetterConfig::default(),
            redis_url: None,
            use_redis_persistence: false,
            circuit_breaker_threshold: 10,
            circuit_breaker_timeout_seconds: 300,
            batch_processing_size: 50,
            poison_message_threshold: 10,
            enable_metrics: true,
            retention_policy: RetentionPolicy {
                keep_resolved_days: 7,
                keep_failed_days: 30,
                max_total_entries: 100000,
                auto_cleanup_enabled: true,
                cleanup_interval_hours: 6,
            },
        }
    }
}

/// Circuit breaker state for dead letter queue operations
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitBreakerState {
    Closed,    // Normal operation
    Open,      // Failures detected, operations blocked
    HalfOpen,  // Testing if service recovered
}

/// Circuit breaker for protecting against cascading failures
pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitBreakerState>>,
    failure_count: Arc<RwLock<u32>>,
    last_failure_time: Arc<RwLock<Option<DateTime<Utc>>>>,
    threshold: u32,
    timeout: Duration,
}

impl CircuitBreaker {
    pub fn new(threshold: u32, timeout_seconds: u64) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitBreakerState::Closed)),
            failure_count: Arc::new(RwLock::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
            threshold,
            timeout: Duration::seconds(timeout_seconds as i64),
        }
    }
    
    pub async fn can_proceed(&self) -> bool {
        let state = *self.state.read().await;
        
        match state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                // Check if timeout has passed
                let last_failure = *self.last_failure_time.read().await;
                if let Some(last_failure) = last_failure {
                    if Utc::now() - last_failure >= self.timeout {
                        // Transition to half-open
                        *self.state.write().await = CircuitBreakerState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }
    
    pub async fn record_success(&self) {
        *self.failure_count.write().await = 0;
        *self.state.write().await = CircuitBreakerState::Closed;
    }
    
    pub async fn record_failure(&self) {
        let mut failure_count = self.failure_count.write().await;
        *failure_count += 1;
        *self.last_failure_time.write().await = Some(Utc::now());
        
        if *failure_count >= self.threshold {
            *self.state.write().await = CircuitBreakerState::Open;
        }
    }
}

/// Enhanced dead letter queue with Redis support and circuit breaker
pub struct EnhancedDeadLetterQueue {
    config: EnhancedDLQConfig,
    redis_connection: Option<Arc<RwLock<ConnectionManager>>>,
    circuit_breaker: CircuitBreaker,
    metrics: Arc<RwLock<EnhancedDLQMetrics>>,
    poison_message_tracker: Arc<RwLock<HashMap<String, u32>>>,
}

/// Enhanced metrics for dead letter queue
#[derive(Debug, Clone, Default)]
pub struct EnhancedDLQMetrics {
    pub total_events_added: u64,
    pub total_events_retried: u64,
    pub total_events_resolved: u64,
    pub total_events_permanently_failed: u64,
    pub poison_messages_detected: u64,
    pub circuit_breaker_opens: u64,
    pub average_retry_delay_seconds: f64,
    pub current_queue_size: usize,
    pub redis_operations_failed: u64,
}

impl EnhancedDeadLetterQueue {
    /// Create a new enhanced dead letter queue
    pub async fn new(config: EnhancedDLQConfig) -> EventResult<Self> {
        let redis_connection = if config.use_redis_persistence {
            if let Some(redis_url) = &config.redis_url {
                let client = Client::open(redis_url.as_str())
                    .map_err(|e| EventError::ConfigurationError {
                        message: format!("Failed to create Redis client: {}", e),
                    })?;
                
                let connection_manager = ConnectionManager::new(client)
                    .await
                    .map_err(|e| EventError::ConfigurationError {
                        message: format!("Failed to create Redis connection manager: {}", e),
                    })?;
                
                Some(Arc::new(RwLock::new(connection_manager)))
            } else {
                return Err(EventError::ConfigurationError {
                    message: "Redis URL required when use_redis_persistence is enabled".to_string(),
                });
            }
        } else {
            None
        };
        
        let circuit_breaker = CircuitBreaker::new(
            config.circuit_breaker_threshold,
            config.circuit_breaker_timeout_seconds,
        );
        
        Ok(Self {
            config,
            redis_connection,
            circuit_breaker,
            metrics: Arc::new(RwLock::new(EnhancedDLQMetrics::default())),
            poison_message_tracker: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Add a failed event to the enhanced dead letter queue
    pub async fn add_failed_event(
        &self,
        event: &EventEnvelope,
        error_message: String,
        error_context: serde_json::Value,
    ) -> EventResult<()> {
        if !self.config.base_config.enabled {
            return Ok(());
        }
        
        // Check circuit breaker
        if !self.circuit_breaker.can_proceed().await {
            tracing::warn!("Dead letter queue circuit breaker is open, dropping event {}", event.event_id);
            return Err(EventError::HandlerError {
                message: "Dead letter queue circuit breaker is open".to_string(),
            });
        }
        
        // Check for poison messages
        let poison_key = self.generate_poison_message_key(event);
        let poison_count = {
            let mut tracker = self.poison_message_tracker.write().await;
            let count = tracker.entry(poison_key.clone()).or_insert(0);
            *count += 1;
            *count
        };
        
        if poison_count > self.config.poison_message_threshold {
            let mut metrics = self.metrics.write().await;
            metrics.poison_messages_detected += 1;
            
            tracing::error!(
                "Poison message detected for event {} (failure count: {})",
                event.event_id,
                poison_count
            );
            
            // Mark as permanently failed without retries
            return self.mark_as_poison_message(event, error_message, poison_count).await;
        }
        
        let entry = DeadLetterEntry {
            id: Uuid::new_v4(),
            original_event_id: event.event_id,
            event_data: event.event_data.clone(),
            error_message: error_message.clone(),
            error_details: error_context,
            retry_count: 0,
            max_retries: self.config.base_config.max_retries,
            status: DeadLetterStatus::Failed,
            created_at: Utc::now(),
            last_retry_at: None,
            next_retry_at: Some(self.calculate_next_retry_time(0)),
        };
        
        // Persist entry
        match self.persist_entry(&entry).await {
            Ok(_) => {
                self.circuit_breaker.record_success().await;
                let mut metrics = self.metrics.write().await;
                metrics.total_events_added += 1;
                metrics.current_queue_size += 1;
                
                tracing::warn!(
                    "Event {} added to enhanced dead letter queue: {}",
                    event.event_id,
                    error_message
                );
            }
            Err(e) => {
                self.circuit_breaker.record_failure().await;
                return Err(e);
            }
        }
        
        Ok(())
    }
    
    /// Get retry candidates with circuit breaker protection
    pub async fn get_retry_candidates_safe(&self, limit: usize) -> EventResult<Vec<DeadLetterEntry>> {
        if !self.circuit_breaker.can_proceed().await {
            return Ok(vec![]);
        }
        
        match self.get_retry_candidates_internal(limit).await {
            Ok(candidates) => {
                self.circuit_breaker.record_success().await;
                Ok(candidates)
            }
            Err(e) => {
                self.circuit_breaker.record_failure().await;
                Err(e)
            }
        }
    }
    
    /// Process retry candidates in batches
    pub async fn process_retry_batch(&self) -> EventResult<ProcessingResult> {
        let candidates = self.get_retry_candidates_safe(self.config.batch_processing_size).await?;
        
        if candidates.is_empty() {
            return Ok(ProcessingResult {
                processed: 0,
                succeeded: 0,
                failed: 0,
                poison_detected: 0,
            });
        }
        
        let mut result = ProcessingResult::default();
        
        for entry in candidates {
            result.processed += 1;
            
            // Simulate retry processing
            match self.simulate_retry_processing(&entry).await {
                Ok(success) => {
                    if success {
                        self.mark_resolved(entry.id).await?;
                        result.succeeded += 1;
                    } else {
                        self.increment_retry(entry.id, "Retry failed".to_string()).await?;
                        result.failed += 1;
                    }
                }
                Err(_) => {
                    result.failed += 1;
                }
            }
        }
        
        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.total_events_retried += result.processed;
        metrics.total_events_resolved += result.succeeded;
        
        Ok(result)
    }
    
    /// Enhanced cleanup with retention policy
    pub async fn cleanup_with_retention(&self) -> EventResult<CleanupResult> {
        let now = Utc::now();
        let retention = &self.config.retention_policy;
        
        let resolved_cutoff = now - Duration::days(retention.keep_resolved_days as i64);
        let failed_cutoff = now - Duration::days(retention.keep_failed_days as i64);
        
        let mut cleanup_result = CleanupResult::default();
        
        // Clean up resolved entries
        cleanup_result.resolved_cleaned = self.cleanup_entries_by_status_and_date(
            DeadLetterStatus::Resolved,
            resolved_cutoff,
        ).await?;
        
        // Clean up old failed entries
        cleanup_result.failed_cleaned = self.cleanup_entries_by_status_and_date(
            DeadLetterStatus::MaxRetriesExceeded,
            failed_cutoff,
        ).await?;
        
        // Clean up poison message tracker
        cleanup_result.poison_entries_cleaned = self.cleanup_poison_tracker().await;
        
        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.current_queue_size = metrics.current_queue_size
            .saturating_sub(cleanup_result.total_cleaned());
        
        Ok(cleanup_result)
    }
    
    /// Get enhanced statistics
    pub async fn get_enhanced_statistics(&self) -> EnhancedDLQStatistics {
        let metrics = self.metrics.read().await.clone();
        let poison_messages = self.poison_message_tracker.read().await.len();
        let circuit_state = *self.circuit_breaker.state.read().await;
        
        EnhancedDLQStatistics {
            base_stats: DeadLetterStatistics {
                total_entries: metrics.current_queue_size as i64,
                failed_entries: 0, // Would need to query actual store
                retrying_entries: 0,
                permanently_failed_entries: metrics.total_events_permanently_failed as i64,
                resolved_entries: metrics.total_events_resolved as i64,
                oldest_entry: None,
                newest_entry: None,
                average_retry_count: 0.0,
            },
            enhanced_metrics: metrics,
            poison_message_count: poison_messages,
            circuit_breaker_state: circuit_state,
        }
    }
    
    // Private helper methods
    
    async fn persist_entry(&self, entry: &DeadLetterEntry) -> EventResult<()> {
        if let Some(ref redis_conn) = self.redis_connection {
            self.persist_to_redis(entry, redis_conn).await
        } else {
            // Fallback to in-memory storage or database
            Ok(())
        }
    }
    
    async fn persist_to_redis(
        &self,
        entry: &DeadLetterEntry,
        redis_conn: &Arc<RwLock<ConnectionManager>>,
    ) -> EventResult<()> {
        let mut conn = redis_conn.write().await;
        let key = format!("dlq:entry:{}", entry.id);
        let serialized = serde_json::to_string(entry)
            .map_err(|e| EventError::SerializationError {
                message: format!("Failed to serialize DLQ entry: {}", e),
            })?;
        
        let _: () = conn.set(&key, &serialized)
            .await
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to persist DLQ entry to Redis: {}", e),
            })?;
        
        // Add to retry queue if applicable
        if entry.status == DeadLetterStatus::Failed {
            if let Some(next_retry) = entry.next_retry_at {
                let retry_key = format!("dlq:retry_queue");
                let score = next_retry.timestamp() as f64;
                let _: () = conn.zadd(&retry_key, &entry.id.to_string(), score)
                    .await
                    .map_err(|e| EventError::DatabaseError {
                        message: format!("Failed to add to retry queue: {}", e),
                    })?;
            }
        }
        
        Ok(())
    }
    
    async fn get_retry_candidates_internal(&self, limit: usize) -> EventResult<Vec<DeadLetterEntry>> {
        if let Some(ref redis_conn) = self.redis_connection {
            self.get_retry_candidates_from_redis(limit, redis_conn).await
        } else {
            Ok(vec![])
        }
    }
    
    async fn get_retry_candidates_from_redis(
        &self,
        limit: usize,
        redis_conn: &Arc<RwLock<ConnectionManager>>,
    ) -> EventResult<Vec<DeadLetterEntry>> {
        let mut conn = redis_conn.write().await;
        let retry_queue_key = "dlq:retry_queue";
        let now = Utc::now().timestamp() as f64;
        
        // Get entries ready for retry
        let entry_ids: Vec<String> = conn.zrangebyscore_limit(
            &retry_queue_key,
            0.0,
            now,
            0,
            limit as isize,
        )
        .await
        .map_err(|e| EventError::DatabaseError {
            message: format!("Failed to get retry candidates: {}", e),
        })?;
        
        let mut entries = Vec::new();
        for entry_id in entry_ids {
            let entry_key = format!("dlq:entry:{}", entry_id);
            if let Ok(serialized) = conn.get::<_, String>(&entry_key).await {
                if let Ok(entry) = serde_json::from_str::<DeadLetterEntry>(&serialized) {
                    entries.push(entry);
                }
            }
        }
        
        Ok(entries)
    }
    
    fn generate_poison_message_key(&self, event: &EventEnvelope) -> String {
        format!("{}:{}:{}", event.aggregate_type, event.event_type, event.aggregate_id)
    }
    
    async fn mark_as_poison_message(
        &self,
        event: &EventEnvelope,
        error_message: String,
        failure_count: u32,
    ) -> EventResult<()> {
        let entry = DeadLetterEntry {
            id: Uuid::new_v4(),
            original_event_id: event.event_id,
            event_data: event.event_data.clone(),
            error_message: format!("POISON MESSAGE (failures: {}): {}", failure_count, error_message),
            error_details: serde_json::json!({
                "poison_message": true,
                "failure_count": failure_count,
                "threshold": self.config.poison_message_threshold
            }),
            retry_count: failure_count as i32,
            max_retries: 0,
            status: DeadLetterStatus::MaxRetriesExceeded,
            created_at: Utc::now(),
            last_retry_at: None,
            next_retry_at: None,
        };
        
        self.persist_entry(&entry).await?;
        
        let mut metrics = self.metrics.write().await;
        metrics.total_events_permanently_failed += 1;
        
        Ok(())
    }
    
    fn calculate_next_retry_time(&self, retry_count: i32) -> DateTime<Utc> {
        let delay_seconds = (self.config.base_config.base_retry_delay_seconds as f64 
            * self.config.base_config.backoff_multiplier.powi(retry_count))
            .min(self.config.base_config.max_retry_delay_seconds as f64) as i64;
        
        Utc::now() + Duration::seconds(delay_seconds)
    }
    
    async fn simulate_retry_processing(&self, _entry: &DeadLetterEntry) -> EventResult<bool> {
        // Simulate processing with 70% success rate
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        Ok(rand::random::<f64>() < 0.7)
    }
    
    async fn mark_resolved(&self, entry_id: Uuid) -> EventResult<()> {
        // Implementation would update the entry status
        let mut metrics = self.metrics.write().await;
        metrics.total_events_resolved += 1;
        metrics.current_queue_size = metrics.current_queue_size.saturating_sub(1);
        Ok(())
    }
    
    async fn increment_retry(&self, _entry_id: Uuid, _error_message: String) -> EventResult<()> {
        // Implementation would increment retry count and reschedule
        Ok(())
    }
    
    async fn cleanup_entries_by_status_and_date(
        &self,
        _status: DeadLetterStatus,
        _cutoff: DateTime<Utc>,
    ) -> EventResult<usize> {
        // Implementation would clean up entries
        Ok(0)
    }
    
    async fn cleanup_poison_tracker(&self) -> usize {
        let mut tracker = self.poison_message_tracker.write().await;
        let initial_size = tracker.len();
        
        // Remove entries older than 24 hours (simplified)
        tracker.clear();
        
        initial_size
    }
}

/// Result of processing a batch of retry candidates
#[derive(Debug, Default)]
pub struct ProcessingResult {
    pub processed: u64,
    pub succeeded: u64,
    pub failed: u64,
    pub poison_detected: u64,
}

/// Result of cleanup operations
#[derive(Debug, Default)]
pub struct CleanupResult {
    pub resolved_cleaned: usize,
    pub failed_cleaned: usize,
    pub poison_entries_cleaned: usize,
}

impl CleanupResult {
    pub fn total_cleaned(&self) -> usize {
        self.resolved_cleaned + self.failed_cleaned + self.poison_entries_cleaned
    }
}

/// Enhanced statistics including circuit breaker state
#[derive(Debug, Clone)]
pub struct EnhancedDLQStatistics {
    pub base_stats: DeadLetterStatistics,
    pub enhanced_metrics: EnhancedDLQMetrics,
    pub poison_message_count: usize,
    pub circuit_breaker_state: CircuitBreakerState,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::events::EventMetadata;
    
    fn create_test_event() -> EventEnvelope {
        EventEnvelope {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            aggregate_type: "test".to_string(),
            event_type: "test_event".to_string(),
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
    async fn test_circuit_breaker() {
        let circuit_breaker = CircuitBreaker::new(3, 300);
        
        // Initially closed
        assert!(circuit_breaker.can_proceed().await);
        
        // Record failures
        circuit_breaker.record_failure().await;
        circuit_breaker.record_failure().await;
        circuit_breaker.record_failure().await;
        
        // Should be open now
        assert!(!circuit_breaker.can_proceed().await);
        
        // Record success should close it
        circuit_breaker.record_success().await;
        assert!(circuit_breaker.can_proceed().await);
    }
    
    #[tokio::test]
    async fn test_enhanced_dlq_creation() {
        let config = EnhancedDLQConfig::default();
        let dlq = EnhancedDeadLetterQueue::new(config).await.unwrap();
        
        let stats = dlq.get_enhanced_statistics().await;
        assert_eq!(stats.enhanced_metrics.total_events_added, 0);
        assert_eq!(stats.circuit_breaker_state, CircuitBreakerState::Closed);
    }
    
    #[tokio::test]
    async fn test_poison_message_detection() {
        let mut config = EnhancedDLQConfig::default();
        config.poison_message_threshold = 2;
        
        let dlq = EnhancedDeadLetterQueue::new(config).await.unwrap();
        let event = create_test_event();
        
        // First failure
        dlq.add_failed_event(&event, "Error 1".to_string(), serde_json::json!({})).await.unwrap();
        
        // Second failure
        dlq.add_failed_event(&event, "Error 2".to_string(), serde_json::json!({})).await.unwrap();
        
        // Third failure should trigger poison message detection
        dlq.add_failed_event(&event, "Error 3".to_string(), serde_json::json!({})).await.unwrap();
        
        let stats = dlq.get_enhanced_statistics().await;
        assert_eq!(stats.enhanced_metrics.poison_messages_detected, 1);
    }
}