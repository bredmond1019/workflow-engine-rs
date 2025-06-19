// File: src/db/events/mod.rs
//
// Event sourcing module for the AI Workflow Orchestration System
// Provides comprehensive event sourcing capabilities including:
// - Event store management
// - Event type definitions and serialization
// - Event dispatching and handling
// - Aggregate root pattern
// - Event projections
// - Real-time event streaming
// - Performance optimizations (caching, partitioning, indexing)
// - Event replay and projection rebuilding
// - Enhanced snapshot management with compression
// - Event versioning and migration system

pub mod store;
pub mod types;
pub mod dispatcher;
pub mod projections;
pub mod handlers;
pub mod aggregate;
pub mod streaming;
pub mod dead_letter_queue;
pub mod enhanced_dead_letter_queue;
pub mod error_integration;
pub mod cross_service_routing;
pub mod saga;
pub mod ordering;
pub mod replay;
pub mod projection_rebuild;
pub mod snapshots;
pub mod snapshot_triggers;
pub mod versioning;
pub mod migrations;
pub mod caching;
pub mod performance;

#[cfg(test)]
pub mod tests;

pub use store::{EventStore, EventStreaming, PostgreSQLEventStore, EventStoreConfig, AggregateSnapshot, EventStoreStatistics};
pub use types::{
    Event, AggregateEvent, EventMetadata,
    WorkflowEvent, AIInteractionEvent, ServiceCallEvent, SystemEvent
};
pub use dispatcher::{EventDispatcher, EventHandler, EventSubscription};
pub use projections::{ProjectionManager, Projection, ProjectionState};
pub use handlers::{WorkflowEventHandler, AIEventHandler, ServiceEventHandler};
pub use aggregate::{AggregateRoot, AggregateVersion};
pub use streaming::{EventStream, EventStreamConfig, StreamPosition};
pub use dead_letter_queue::{DeadLetterQueue, PostgreSQLDeadLetterQueue, DeadLetterConfig as DLQConfig, DeadLetterProcessor};
pub use enhanced_dead_letter_queue::{
    EnhancedDeadLetterQueue, EnhancedDLQConfig, CircuitBreaker, CircuitBreakerState,
    EnhancedDLQMetrics, EnhancedDLQStatistics, ProcessingResult, CleanupResult
};
pub use error_integration::{ResilientEventStore, EventSourcingRecovery};
pub use replay::{EventReplayEngine, ReplayHandler, ReplayConfig, ReplayPosition, BatchReplayProcessor};
pub use projection_rebuild::{ProjectionRebuildManager, ProjectionRebuildConfig, RebuildMetadata, RebuildStatistics, BatchProjectionRebuilder};
pub use snapshots::{EnhancedSnapshotManager, SnapshotConfig, EnhancedSnapshot, CompressionType, SnapshotStatistics, SnapshotCompressor, GzipCompressor, Lz4Compressor};
pub use snapshot_triggers::{SnapshotTriggerManager, SnapshotTriggerConfig, SnapshotTrigger, Snapshottable, TriggerEvent, TriggerStatistics, SnapshotScheduler};
pub use versioning::{EventVersionManager, VersioningConfig, SchemaVersion, EventMigrator, VersioningStatistics, MigratingReplayHandler};
pub use migrations::{MigrationRegistry, WorkflowStartedV1ToV2Migration, WorkflowCompletedV1ToV2Migration, PromptSentV1ToV2Migration, ResponseReceivedV1ToV2Migration, FieldRenameMigration, FieldRemovalMigration};
pub use caching::{CachedEventStore, CacheConfig, CacheStatistics, MultiTierCache};
pub use performance::{EventStorePerformanceOptimizer, PerformanceConfig, PerformanceStatistics, PartitionInfo, IndexInfo};
pub use cross_service_routing::{
    CrossServiceEventRouter, ServiceRoutingConfig, RoutingMetadata,
    RoutingPriority, RoutedEvent, CrossServiceEventHandler
};
pub use saga::{
    SagaOrchestrator, SagaDefinition, SagaStep, SagaExecution, SagaState,
    SagaStepStatus, SagaStepResult, SagaStepExecutor, SagaStore, InMemorySagaStore,
    RetryPolicy, CompensationStrategy
};
pub use ordering::{
    EventOrderingProcessor, EventOrderingManager, OrderedEvent, OrderingConfig,
    OrderingStrategy, DeduplicationStrategy, EventPriority, ProcessingStatistics
};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Result type for event sourcing operations
pub type EventResult<T> = Result<T, EventError>;

/// Error types for event sourcing operations
#[derive(Debug, thiserror::Error, Clone)]
pub enum EventError {
    #[error("Database error: {message}")]
    DatabaseError { message: String },
    
    #[error("Serialization error: {message}")]
    SerializationError { message: String },
    
    #[error("Concurrency conflict: {message}")]
    ConcurrencyError { message: String },
    
    #[error("Event not found: {event_id}")]
    EventNotFound { event_id: Uuid },
    
    #[error("Aggregate not found: {aggregate_id}")]
    AggregateNotFound { aggregate_id: Uuid },
    
    #[error("Invalid event version: expected {expected}, got {actual}")]
    InvalidVersion { expected: i64, actual: i64 },
    
    #[error("Projection error: {message}")]
    ProjectionError { message: String },
    
    #[error("Handler error: {message}")]
    HandlerError { message: String },
    
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },
}

impl From<diesel::result::Error> for EventError {
    fn from(error: diesel::result::Error) -> Self {
        EventError::DatabaseError {
            message: error.to_string(),
        }
    }
}

impl From<serde_json::Error> for EventError {
    fn from(error: serde_json::Error) -> Self {
        EventError::SerializationError {
            message: error.to_string(),
        }
    }
}

impl From<diesel::r2d2::PoolError> for EventError {
    fn from(error: diesel::r2d2::PoolError) -> Self {
        EventError::DatabaseError {
            message: format!("Connection pool error: {}", error),
        }
    }
}

/// Configuration for event sourcing system with performance optimizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSourcingConfig {
    /// Database connection configuration
    pub database_url: String,
    
    /// Maximum number of events to load per batch
    pub batch_size: usize,
    
    /// Snapshot frequency (every N events)
    pub snapshot_frequency: i64,
    
    /// Enable real-time event streaming
    pub enable_streaming: bool,
    
    /// Dead letter queue configuration
    pub dlq_config: DLQConfig,
    
    /// Projection configuration
    pub projection_config: ProjectionConfig,
    
    /// Cache configuration for performance
    pub cache_config: CacheConfig,
    
    /// Performance optimization configuration
    pub performance_config: PerformanceConfig,
    
    /// Snapshot configuration with compression
    pub snapshot_config: SnapshotConfig,
    
    /// Event versioning configuration
    pub versioning_config: VersioningConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionConfig {
    pub auto_rebuild: bool,
    pub batch_size: usize,
    pub checkpoint_frequency: i64,
}

impl Default for EventSourcingConfig {
    fn default() -> Self {
        Self {
            database_url: "postgresql://localhost/ai_workflow_db".to_string(),
            batch_size: 1000,
            snapshot_frequency: 100,
            enable_streaming: true,
            dlq_config: DLQConfig {
                max_retries: 3,
                base_retry_delay_seconds: 60,
                backoff_multiplier: 2.0,
                max_retry_delay_seconds: 3600,
                enabled: true,
                processing_interval_seconds: 300,
            },
            projection_config: ProjectionConfig {
                auto_rebuild: false,
                batch_size: 500,
                checkpoint_frequency: 50,
            },
            cache_config: CacheConfig::default(),
            performance_config: PerformanceConfig::default(),
            snapshot_config: SnapshotConfig::default(),
            versioning_config: VersioningConfig::default(),
        }
    }
}

/// Trait for objects that can be reconstructed from events
#[async_trait]
pub trait EventSourcing {
    type Event;
    type Error;
    
    /// Apply an event to the current state
    fn apply_event(&mut self, event: &Self::Event) -> Result<(), Self::Error>;
    
    /// Get the current version of the aggregate
    fn version(&self) -> i64;
    
    /// Get the aggregate ID
    fn aggregate_id(&self) -> Uuid;
}

/// Trait for event serialization and versioning
pub trait EventSerializable {
    /// Serialize the event to JSON
    fn serialize(&self) -> EventResult<serde_json::Value>;
    
    /// Deserialize from JSON with version support
    fn deserialize(data: &serde_json::Value, version: i32) -> EventResult<Self>
    where
        Self: Sized;
    
    /// Get the schema version for this event type
    fn schema_version() -> i32;
    
    /// Get the event type name
    fn event_type() -> &'static str;
}

/// Event envelope that wraps all events with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event_id: Uuid,
    pub aggregate_id: Uuid,
    pub aggregate_type: String,
    pub event_type: String,
    pub aggregate_version: i64,
    pub event_data: serde_json::Value,
    pub metadata: EventMetadata,
    pub occurred_at: DateTime<Utc>,
    pub recorded_at: DateTime<Utc>,
    pub schema_version: i32,
    pub causation_id: Option<Uuid>,
    pub correlation_id: Option<Uuid>,
    pub checksum: Option<String>,
}

/// Comprehensive event sourcing system with all optimizations
pub struct OptimizedEventSourcingSystem {
    /// Core event store with caching
    pub event_store: Arc<CachedEventStore>,
    
    /// Performance optimizer
    pub performance_optimizer: EventStorePerformanceOptimizer,
    
    /// Enhanced snapshot manager
    pub snapshot_manager: EnhancedSnapshotManager,
    
    /// Event version manager
    pub version_manager: EventVersionManager,
    
    /// Event replay engine
    pub replay_engine: EventReplayEngine,
    
    /// Projection rebuild manager
    pub projection_manager: ProjectionRebuildManager,
    
    /// Configuration
    pub config: EventSourcingConfig,
}

impl OptimizedEventSourcingSystem {
    /// Create a new optimized event sourcing system
    pub async fn new(
        db_pool: Arc<diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>>,
        config: EventSourcingConfig,
    ) -> EventResult<Self> {
        // Create base event store
        let base_store = PostgreSQLEventStore::new(EventStoreConfig::default())?;
        let base_store_arc = Arc::new(base_store);
        
        // Wrap with caching
        let cached_store = Arc::new(CachedEventStore::new(
            base_store_arc,
            config.cache_config.clone(),
        ));
        
        // Initialize performance optimizer
        let performance_optimizer = EventStorePerformanceOptimizer::new(
            db_pool.clone(),
            config.performance_config.clone(),
        );
        
        // Initialize performance optimizations
        performance_optimizer.initialize().await?;
        
        // Create enhanced snapshot manager
        let snapshot_manager = EnhancedSnapshotManager::new(
            cached_store.clone() as Arc<dyn EventStore>,
            config.snapshot_config.clone(),
        );
        
        // Create version manager
        let version_manager = EventVersionManager::new(config.versioning_config.clone());
        
        // Create replay engine
        let replay_engine = EventReplayEngine::new(
            cached_store.clone() as Arc<dyn EventStore>,
            ReplayConfig::default(),
        );
        
        // Create projection manager
        let projection_manager = ProjectionRebuildManager::new(
            cached_store.clone() as Arc<dyn EventStore>,
            ProjectionRebuildConfig::default(),
        );
        
        Ok(Self {
            event_store: cached_store,
            performance_optimizer,
            snapshot_manager,
            version_manager,
            replay_engine,
            projection_manager,
            config,
        })
    }
    
    /// Start background optimization tasks
    pub async fn start_background_tasks(&self) -> Vec<tokio::task::JoinHandle<()>> {
        let mut handles = Vec::new();
        
        // Start cache cleanup
        let cache_cleanup = self.event_store.start_cache_cleanup().await;
        handles.push(cache_cleanup);
        
        // Start performance maintenance
        let performance_maintenance = self.performance_optimizer.start_background_maintenance().await;
        handles.push(performance_maintenance);
        
        handles
    }
    
    /// Get comprehensive system statistics
    pub async fn get_system_statistics(&self) -> SystemStatistics {
        let cache_stats = self.event_store.get_cache_statistics().await;
        let performance_stats = self.performance_optimizer.get_statistics().await;
        let snapshot_stats = self.snapshot_manager.get_statistics().await;
        let versioning_stats = self.version_manager.get_statistics().await;
        let projection_stats = self.projection_manager.get_statistics().await;
        
        SystemStatistics {
            cache_statistics: cache_stats,
            performance_statistics: performance_stats,
            snapshot_statistics: snapshot_stats,
            versioning_statistics: versioning_stats,
            projection_statistics: projection_stats,
        }
    }
    
    /// Run comprehensive system maintenance
    pub async fn run_system_maintenance(&self) -> EventResult<MaintenanceResult> {
        let start_time = std::time::Instant::now();
        
        // Run performance maintenance
        self.performance_optimizer.run_maintenance().await?;
        
        // Clean up old snapshots
        let cleaned_snapshots = self.snapshot_manager.cleanup_old_snapshots().await?;
        
        // Clear cache if needed
        let cache_stats_before = self.event_store.get_cache_statistics().await;
        if cache_stats_before.hit_ratio < 0.3 {
            self.event_store.clear_cache().await;
        }
        
        let duration = start_time.elapsed();
        
        Ok(MaintenanceResult {
            duration_ms: duration.as_millis() as u64,
            cleaned_snapshots,
            cache_cleared: cache_stats_before.hit_ratio < 0.3,
        })
    }
}

/// System-wide statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatistics {
    pub cache_statistics: CacheStatistics,
    pub performance_statistics: PerformanceStatistics,
    pub snapshot_statistics: SnapshotStatistics,
    pub versioning_statistics: VersioningStatistics,
    pub projection_statistics: RebuildStatistics,
}

/// Maintenance operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceResult {
    pub duration_ms: u64,
    pub cleaned_snapshots: usize,
    pub cache_cleared: bool,
}