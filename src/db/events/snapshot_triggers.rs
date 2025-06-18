// File: src/db/events/snapshot_triggers.rs
//
// Automatic snapshot triggering and lifecycle management
// Provides intelligent snapshot creation based on various triggers

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{info, warn, error, debug};
use uuid::Uuid;

use super::{
    EventStore, EventEnvelope, EventError, EventResult,
    snapshots::{EnhancedSnapshotManager, SnapshotConfig, EnhancedSnapshot},
    replay::{ReplayHandler, EventReplayEngine, ReplayConfig},
};

/// Types of snapshot triggers
#[derive(Debug, Clone, PartialEq)]
pub enum SnapshotTrigger {
    /// Trigger based on event count since last snapshot
    EventCount(i64),
    /// Trigger based on time elapsed since last snapshot
    TimeElapsed(chrono::Duration),
    /// Trigger based on memory usage threshold
    MemoryThreshold(f64),
    /// Trigger based on aggregate size
    AggregateSize(usize),
    /// Manual trigger request
    Manual,
}

/// Configuration for snapshot triggers
#[derive(Debug, Clone)]
pub struct SnapshotTriggerConfig {
    /// Event count trigger (snapshot every N events)
    pub event_count_threshold: i64,
    /// Time trigger (snapshot every N hours)
    pub time_threshold_hours: i64,
    /// Memory usage trigger (snapshot when usage > N%)
    pub memory_threshold_percent: f64,
    /// Aggregate size trigger (snapshot when JSON size > N bytes)
    pub aggregate_size_threshold: usize,
    /// Whether to enable automatic triggers
    pub auto_triggers_enabled: bool,
    /// Minimum time between snapshots (prevents too frequent snapshots)
    pub min_snapshot_interval_minutes: i64,
}

impl Default for SnapshotTriggerConfig {
    fn default() -> Self {
        Self {
            event_count_threshold: 100,
            time_threshold_hours: 24,
            memory_threshold_percent: 80.0,
            aggregate_size_threshold: 1024 * 1024, // 1MB
            auto_triggers_enabled: true,
            min_snapshot_interval_minutes: 30,
        }
    }
}

/// Metadata about snapshot trigger events
#[derive(Debug, Clone)]
pub struct TriggerEvent {
    pub trigger_id: Uuid,
    pub aggregate_id: Uuid,
    pub trigger_type: SnapshotTrigger,
    pub triggered_at: DateTime<Utc>,
    pub snapshot_created: bool,
    pub snapshot_id: Option<Uuid>,
    pub error: Option<String>,
    pub metrics: HashMap<String, f64>,
}

/// Statistics about trigger activity
#[derive(Debug, Clone)]
pub struct TriggerStatistics {
    pub total_triggers: u64,
    pub successful_snapshots: u64,
    pub failed_snapshots: u64,
    pub triggers_by_type: HashMap<String, u64>,
    pub average_trigger_response_time_ms: f64,
    pub last_trigger_time: Option<DateTime<Utc>>,
}

/// Trait for objects that can be snapshotted
#[async_trait]
pub trait Snapshottable: Send + Sync {
    /// Get the aggregate ID
    fn aggregate_id(&self) -> Uuid;
    
    /// Get the aggregate type
    fn aggregate_type(&self) -> &str;
    
    /// Get the current version
    fn current_version(&self) -> i64;
    
    /// Create snapshot data from current state
    async fn create_snapshot_data(&self) -> EventResult<serde_json::Value>;
    
    /// Get the estimated size of the aggregate in bytes
    fn estimated_size_bytes(&self) -> usize;
    
    /// Get the time when this aggregate was last updated
    fn last_updated(&self) -> DateTime<Utc>;
}

/// Automatic snapshot trigger manager
pub struct SnapshotTriggerManager {
    snapshot_manager: EnhancedSnapshotManager,
    config: SnapshotTriggerConfig,
    trigger_history: Arc<RwLock<Vec<TriggerEvent>>>,
    aggregate_last_snapshot: Arc<RwLock<HashMap<Uuid, DateTime<Utc>>>>,
    statistics: Arc<RwLock<TriggerStatistics>>,
}

impl SnapshotTriggerManager {
    pub fn new(
        snapshot_manager: EnhancedSnapshotManager,
        config: SnapshotTriggerConfig,
    ) -> Self {
        let statistics = TriggerStatistics {
            total_triggers: 0,
            successful_snapshots: 0,
            failed_snapshots: 0,
            triggers_by_type: HashMap::new(),
            average_trigger_response_time_ms: 0.0,
            last_trigger_time: None,
        };
        
        Self {
            snapshot_manager,
            config,
            trigger_history: Arc::new(RwLock::new(Vec::new())),
            aggregate_last_snapshot: Arc::new(RwLock::new(HashMap::new())),
            statistics: Arc::new(RwLock::new(statistics)),
        }
    }
    
    /// Check if any triggers should fire for the given aggregate
    pub async fn check_triggers<T: Snapshottable + ?Sized>(
        &self,
        aggregate: &T,
    ) -> EventResult<Vec<SnapshotTrigger>> {
        if !self.config.auto_triggers_enabled {
            return Ok(Vec::new());
        }
        
        let mut triggers = Vec::new();
        let aggregate_id = aggregate.aggregate_id();
        
        // Check if minimum interval has passed
        let last_snapshot_time = {
            let last_snapshots = self.aggregate_last_snapshot.read().await;
            last_snapshots.get(&aggregate_id).copied()
        };
        
        if let Some(last_time) = last_snapshot_time {
            let minutes_since_last = (Utc::now() - last_time).num_minutes();
            if minutes_since_last < self.config.min_snapshot_interval_minutes {
                debug!(
                    "Skipping triggers for aggregate {}: only {} minutes since last snapshot",
                    aggregate_id, minutes_since_last
                );
                return Ok(Vec::new());
            }
        }
        
        // Check event count trigger
        if let Some(last_snapshot) = self.snapshot_manager.restore_snapshot(aggregate_id).await? {
            let events_since_snapshot = aggregate.current_version() - last_snapshot.aggregate_version;
            if events_since_snapshot >= self.config.event_count_threshold {
                triggers.push(SnapshotTrigger::EventCount(events_since_snapshot));
            }
        } else if aggregate.current_version() >= self.config.event_count_threshold {
            // No snapshot exists and we have enough events
            triggers.push(SnapshotTrigger::EventCount(aggregate.current_version()));
        }
        
        // Check time trigger
        let hours_since_update = (Utc::now() - aggregate.last_updated()).num_hours();
        if hours_since_update >= self.config.time_threshold_hours {
            triggers.push(SnapshotTrigger::TimeElapsed(
                chrono::Duration::hours(hours_since_update)
            ));
        }
        
        // Check aggregate size trigger
        if aggregate.estimated_size_bytes() >= self.config.aggregate_size_threshold {
            triggers.push(SnapshotTrigger::AggregateSize(aggregate.estimated_size_bytes()));
        }
        
        // Check memory usage trigger (simplified implementation)
        if let Ok(memory_usage) = self.get_memory_usage_percent().await {
            if memory_usage >= self.config.memory_threshold_percent {
                triggers.push(SnapshotTrigger::MemoryThreshold(memory_usage));
            }
        }
        
        Ok(triggers)
    }
    
    /// Execute snapshot creation for triggered conditions
    pub async fn execute_triggers<T: Snapshottable + ?Sized>(
        &self,
        aggregate: &T,
        triggers: Vec<SnapshotTrigger>,
    ) -> EventResult<Vec<TriggerEvent>> {
        let mut trigger_events = Vec::new();
        
        for trigger in triggers {
            let trigger_event = self.execute_single_trigger(aggregate, trigger).await?;
            trigger_events.push(trigger_event);
        }
        
        Ok(trigger_events)
    }
    
    /// Execute a single trigger
    async fn execute_single_trigger<T: Snapshottable + ?Sized>(
        &self,
        aggregate: &T,
        trigger: SnapshotTrigger,
    ) -> EventResult<TriggerEvent> {
        let start_time = std::time::Instant::now();
        let trigger_id = Uuid::new_v4();
        let aggregate_id = aggregate.aggregate_id();
        
        info!(
            "Executing snapshot trigger {:?} for aggregate {}",
            trigger, aggregate_id
        );
        
        let mut trigger_event = TriggerEvent {
            trigger_id,
            aggregate_id,
            trigger_type: trigger.clone(),
            triggered_at: Utc::now(),
            snapshot_created: false,
            snapshot_id: None,
            error: None,
            metrics: HashMap::new(),
        };
        
        // Add metrics based on trigger type
        match &trigger {
            SnapshotTrigger::EventCount(count) => {
                trigger_event.metrics.insert("event_count".to_string(), *count as f64);
            }
            SnapshotTrigger::TimeElapsed(duration) => {
                trigger_event.metrics.insert("hours_elapsed".to_string(), duration.num_hours() as f64);
            }
            SnapshotTrigger::MemoryThreshold(usage) => {
                trigger_event.metrics.insert("memory_usage_percent".to_string(), *usage);
            }
            SnapshotTrigger::AggregateSize(size) => {
                trigger_event.metrics.insert("aggregate_size_bytes".to_string(), *size as f64);
            }
            SnapshotTrigger::Manual => {
                trigger_event.metrics.insert("manual_trigger".to_string(), 1.0);
            }
        }
        
        // Create snapshot
        match aggregate.create_snapshot_data().await {
            Ok(snapshot_data) => {
                match self.snapshot_manager.create_snapshot(
                    aggregate_id,
                    aggregate.aggregate_type().to_string(),
                    aggregate.current_version(),
                    snapshot_data,
                ).await {
                    Ok(snapshot) => {
                        trigger_event.snapshot_created = true;
                        trigger_event.snapshot_id = Some(snapshot.id);
                        
                        // Update last snapshot time
                        let mut last_snapshots = self.aggregate_last_snapshot.write().await;
                        last_snapshots.insert(aggregate_id, Utc::now());
                        
                        info!(
                            "Successfully created snapshot {} for aggregate {} (trigger: {:?})",
                            snapshot.id, aggregate_id, trigger
                        );
                    }
                    Err(e) => {
                        trigger_event.error = Some(e.to_string());
                        error!(
                            "Failed to create snapshot for aggregate {} (trigger: {:?}): {}",
                            aggregate_id, trigger, e
                        );
                    }
                }
            }
            Err(e) => {
                trigger_event.error = Some(format!("Failed to create snapshot data: {}", e));
                error!(
                    "Failed to create snapshot data for aggregate {} (trigger: {:?}): {}",
                    aggregate_id, trigger, e
                );
            }
        }
        
        let response_time_ms = start_time.elapsed().as_millis() as f64;
        trigger_event.metrics.insert("response_time_ms".to_string(), response_time_ms);
        
        // Update statistics
        self.update_statistics(&trigger_event, response_time_ms).await;
        
        // Add to history
        {
            let mut history = self.trigger_history.write().await;
            history.push(trigger_event.clone());
            
            // Keep only last 1000 trigger events
            if history.len() > 1000 {
                history.remove(0);
            }
        }
        
        Ok(trigger_event)
    }
    
    /// Manually trigger a snapshot for an aggregate
    pub async fn manual_trigger<T: Snapshottable>(
        &self,
        aggregate: &T,
    ) -> EventResult<TriggerEvent> {
        info!(
            "Manual snapshot trigger for aggregate {}",
            aggregate.aggregate_id()
        );
        
        self.execute_single_trigger(aggregate, SnapshotTrigger::Manual).await
    }
    
    /// Get trigger statistics
    pub async fn get_statistics(&self) -> TriggerStatistics {
        self.statistics.read().await.clone()
    }
    
    /// Get trigger history
    pub async fn get_trigger_history(&self) -> Vec<TriggerEvent> {
        self.trigger_history.read().await.clone()
    }
    
    /// Get recent trigger events for a specific aggregate
    pub async fn get_aggregate_trigger_history(&self, aggregate_id: Uuid) -> Vec<TriggerEvent> {
        let history = self.trigger_history.read().await;
        history.iter()
            .filter(|event| event.aggregate_id == aggregate_id)
            .cloned()
            .collect()
    }
    
    /// Update trigger statistics
    async fn update_statistics(&self, trigger_event: &TriggerEvent, response_time_ms: f64) {
        let mut stats = self.statistics.write().await;
        
        stats.total_triggers += 1;
        stats.last_trigger_time = Some(trigger_event.triggered_at);
        
        if trigger_event.snapshot_created {
            stats.successful_snapshots += 1;
        } else {
            stats.failed_snapshots += 1;
        }
        
        let trigger_type_name = match &trigger_event.trigger_type {
            SnapshotTrigger::EventCount(_) => "event_count".to_string(),
            SnapshotTrigger::TimeElapsed(_) => "time_elapsed".to_string(),
            SnapshotTrigger::MemoryThreshold(_) => "memory_threshold".to_string(),
            SnapshotTrigger::AggregateSize(_) => "aggregate_size".to_string(),
            SnapshotTrigger::Manual => "manual".to_string(),
        };
        
        *stats.triggers_by_type.entry(trigger_type_name).or_insert(0) += 1;
        
        // Update average response time
        let total_response_time = stats.average_trigger_response_time_ms * (stats.total_triggers - 1) as f64;
        stats.average_trigger_response_time_ms = (total_response_time + response_time_ms) / stats.total_triggers as f64;
    }
    
    /// Get current memory usage percentage (simplified implementation)
    async fn get_memory_usage_percent(&self) -> EventResult<f64> {
        // In a real implementation, this would check actual system memory usage
        // For now, return a mock value
        Ok(50.0)
    }
}

/// Automated snapshot scheduler that runs in the background
pub struct SnapshotScheduler {
    trigger_manager: SnapshotTriggerManager,
    registered_aggregates: Arc<RwLock<HashMap<Uuid, Arc<dyn Snapshottable>>>>,
    check_interval_seconds: u64,
    is_running: Arc<RwLock<bool>>,
}

impl SnapshotScheduler {
    pub fn new(
        trigger_manager: SnapshotTriggerManager,
        check_interval_seconds: u64,
    ) -> Self {
        Self {
            trigger_manager,
            registered_aggregates: Arc::new(RwLock::new(HashMap::new())),
            check_interval_seconds,
            is_running: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Register an aggregate for automatic snapshot monitoring
    pub async fn register_aggregate<T: Snapshottable + 'static>(
        &self,
        aggregate: Arc<T>,
    ) -> EventResult<()> {
        let aggregate_id = aggregate.aggregate_id();
        let mut aggregates = self.registered_aggregates.write().await;
        aggregates.insert(aggregate_id, aggregate);
        
        info!("Registered aggregate {} for automatic snapshot monitoring", aggregate_id);
        Ok(())
    }
    
    /// Unregister an aggregate from monitoring
    pub async fn unregister_aggregate(&self, aggregate_id: Uuid) -> EventResult<()> {
        let mut aggregates = self.registered_aggregates.write().await;
        aggregates.remove(&aggregate_id);
        
        info!("Unregistered aggregate {} from snapshot monitoring", aggregate_id);
        Ok(())
    }
    
    /// Start the automated scheduler
    pub async fn start(&self) -> EventResult<()> {
        {
            let mut running = self.is_running.write().await;
            if *running {
                return Err(EventError::ConfigurationError {
                    message: "Snapshot scheduler is already running".to_string(),
                });
            }
            *running = true;
        }
        
        info!("Starting automatic snapshot scheduler (interval: {} seconds)", self.check_interval_seconds);
        
        let trigger_manager = self.trigger_manager.clone();
        let aggregates = self.registered_aggregates.clone();
        let is_running = self.is_running.clone();
        let check_interval = self.check_interval_seconds;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                std::time::Duration::from_secs(check_interval)
            );
            
            while *is_running.read().await {
                interval.tick().await;
                
                let aggregates_snapshot = aggregates.read().await;
                
                for (aggregate_id, aggregate) in aggregates_snapshot.iter() {
                    match trigger_manager.check_triggers(&**aggregate).await {
                        Ok(triggers) => {
                            if !triggers.is_empty() {
                                debug!(
                                    "Found {} triggers for aggregate {}",
                                    triggers.len(), aggregate_id
                                );
                                
                                if let Err(e) = trigger_manager.execute_triggers(
                                    &**aggregate,
                                    triggers,
                                ).await {
                                    error!(
                                        "Failed to execute triggers for aggregate {}: {}",
                                        aggregate_id, e
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            error!(
                                "Failed to check triggers for aggregate {}: {}",
                                aggregate_id, e
                            );
                        }
                    }
                }
            }
            
            info!("Snapshot scheduler stopped");
        });
        
        Ok(())
    }
    
    /// Stop the automated scheduler
    pub async fn stop(&self) -> EventResult<()> {
        let mut running = self.is_running.write().await;
        *running = false;
        
        info!("Stopping automatic snapshot scheduler");
        Ok(())
    }
    
    /// Check if the scheduler is running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }
}

impl Clone for SnapshotTriggerManager {
    fn clone(&self) -> Self {
        Self {
            snapshot_manager: self.snapshot_manager.clone(),
            config: self.config.clone(),
            trigger_history: self.trigger_history.clone(),
            aggregate_last_snapshot: self.aggregate_last_snapshot.clone(),
            statistics: self.statistics.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    struct TestAggregate {
        id: Uuid,
        version: i64,
        size_bytes: usize,
        last_updated: DateTime<Utc>,
    }
    
    #[async_trait]
    impl Snapshottable for TestAggregate {
        fn aggregate_id(&self) -> Uuid {
            self.id
        }
        
        fn aggregate_type(&self) -> &str {
            "test_aggregate"
        }
        
        fn current_version(&self) -> i64 {
            self.version
        }
        
        async fn create_snapshot_data(&self) -> EventResult<serde_json::Value> {
            Ok(json!({
                "id": self.id,
                "version": self.version,
                "state": "test_state",
                "timestamp": Utc::now()
            }))
        }
        
        fn estimated_size_bytes(&self) -> usize {
            self.size_bytes
        }
        
        fn last_updated(&self) -> DateTime<Utc> {
            self.last_updated
        }
    }
    
    #[test]
    fn test_trigger_config_defaults() {
        let config = SnapshotTriggerConfig::default();
        assert_eq!(config.event_count_threshold, 100);
        assert_eq!(config.time_threshold_hours, 24);
        assert_eq!(config.memory_threshold_percent, 80.0);
        assert_eq!(config.aggregate_size_threshold, 1024 * 1024);
        assert!(config.auto_triggers_enabled);
        assert_eq!(config.min_snapshot_interval_minutes, 30);
    }
    
    #[test]
    fn test_trigger_event_creation() {
        let trigger_event = TriggerEvent {
            trigger_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            trigger_type: SnapshotTrigger::EventCount(150),
            triggered_at: Utc::now(),
            snapshot_created: true,
            snapshot_id: Some(Uuid::new_v4()),
            error: None,
            metrics: HashMap::new(),
        };
        
        assert!(trigger_event.snapshot_created);
        assert!(trigger_event.snapshot_id.is_some());
        assert!(trigger_event.error.is_none());
    }
    
    #[tokio::test]
    async fn test_snapshottable_implementation() {
        let aggregate = TestAggregate {
            id: Uuid::new_v4(),
            version: 42,
            size_bytes: 2048,
            last_updated: Utc::now(),
        };
        
        assert_eq!(aggregate.current_version(), 42);
        assert_eq!(aggregate.estimated_size_bytes(), 2048);
        assert_eq!(aggregate.aggregate_type(), "test_aggregate");
        
        let snapshot_data = aggregate.create_snapshot_data().await.unwrap();
        assert_eq!(snapshot_data["version"], 42);
        assert_eq!(snapshot_data["state"], "test_state");
    }
}