// File: src/db/events/store.rs
//
// PostgreSQL-backed event store implementation for comprehensive event sourcing

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::db::schema::{event_store, event_snapshots, event_dead_letter_queue, event_projections};
use super::{
    EventError, EventResult, EventEnvelope, EventMetadata, EventSourcingConfig,
    Event as EventTrait, EventSerializable
};

/// Configuration for the event store
#[derive(Debug, Clone)]
pub struct EventStoreConfig {
    pub database_url: String,
    pub connection_pool_size: u32,
    pub batch_size: usize,
    pub snapshot_frequency: i64,
    pub enable_checksums: bool,
}

/// Statistics about the event store
#[derive(Debug, Clone)]
pub struct EventStoreStatistics {
    pub total_events: u64,
    pub total_aggregates: u64,
    pub total_snapshots: u64,
}

impl Default for EventStoreConfig {
    fn default() -> Self {
        Self {
            database_url: "postgresql://localhost/ai_workflow_db".to_string(),
            connection_pool_size: 10,
            batch_size: 1000,
            snapshot_frequency: 100,
            enable_checksums: true,
        }
    }
}

/// Event store interface for persisting and retrieving events
#[async_trait]
pub trait EventStore: Send + Sync {
    /// Append a single event to the store
    async fn append_event(&self, event: &EventEnvelope) -> EventResult<()>;
    
    /// Append multiple events atomically
    async fn append_events(&self, events: &[EventEnvelope]) -> EventResult<()>;
    
    /// Get all events for a specific aggregate
    async fn get_events(&self, aggregate_id: Uuid) -> EventResult<Vec<EventEnvelope>>;
    
    /// Get events for an aggregate starting from a specific version
    async fn get_events_from_version(
        &self,
        aggregate_id: Uuid,
        from_version: i64,
    ) -> EventResult<Vec<EventEnvelope>>;
    
    /// Get events by type within a time range
    async fn get_events_by_type(
        &self,
        event_type: &str,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        limit: Option<usize>,
    ) -> EventResult<Vec<EventEnvelope>>;
    
    /// Get events by correlation ID
    async fn get_events_by_correlation_id(&self, correlation_id: Uuid) -> EventResult<Vec<EventEnvelope>>;
    
    /// Get the current version of an aggregate
    async fn get_aggregate_version(&self, aggregate_id: Uuid) -> EventResult<i64>;
    
    /// Check if an aggregate exists
    async fn aggregate_exists(&self, aggregate_id: Uuid) -> EventResult<bool>;
    
    /// Save a snapshot of an aggregate
    async fn save_snapshot(&self, snapshot: &AggregateSnapshot) -> EventResult<()>;
    
    /// Get the latest snapshot for an aggregate
    async fn get_snapshot(&self, aggregate_id: Uuid) -> EventResult<Option<AggregateSnapshot>>;
    
    /// Get all events from a global position for streaming
    async fn get_events_from_position(&self, position: i64, limit: usize) -> EventResult<Vec<EventEnvelope>>;
    
    /// Get the current global position (latest event position)
    async fn get_current_position(&self) -> EventResult<i64>;
    
    /// Replay events to rebuild projections
    async fn replay_events(
        &self,
        from_position: i64,
        event_types: Option<Vec<String>>,
        batch_size: usize,
    ) -> EventResult<Vec<EventEnvelope>>;
    
    /// Get events for multiple aggregates in one query
    async fn get_events_for_aggregates(&self, aggregate_ids: &[Uuid]) -> EventResult<Vec<EventEnvelope>>;
    
    /// Delete old snapshots (keep only the latest)
    async fn cleanup_old_snapshots(&self, keep_latest: usize) -> EventResult<usize>;
    
    /// Get aggregate IDs by type with pagination
    async fn get_aggregate_ids_by_type(
        &self,
        aggregate_type: &str,
        offset: i64,
        limit: usize,
    ) -> EventResult<Vec<Uuid>>;
    
    /// Optimize storage by vacuuming (PostgreSQL specific)
    async fn optimize_storage(&self) -> EventResult<()>;
}

/// Extended trait for event streaming functionality (not object-safe)
#[async_trait]
pub trait EventStreaming: EventStore {
    /// Stream events with callback for processing
    async fn stream_events<F>(
        &self,
        from_position: i64,
        event_types: Option<Vec<String>>,
        batch_size: usize,
        callback: F,
    ) -> EventResult<i64>
    where
        F: Fn(&[EventEnvelope]) -> EventResult<bool> + Send + Sync;
}

/// PostgreSQL implementation of the event store
pub struct PostgreSQLEventStore {
    pool: Arc<Pool<ConnectionManager<PgConnection>>>,
    config: EventStoreConfig,
}

impl PostgreSQLEventStore {
    /// Create a new PostgreSQL event store
    pub fn new(config: EventStoreConfig) -> EventResult<Self> {
        let manager = ConnectionManager::<PgConnection>::new(&config.database_url);
        let pool = Pool::builder()
            .max_size(config.connection_pool_size)
            .build(manager)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to create connection pool: {}", e),
            })?;

        Ok(Self {
            pool: Arc::new(pool),
            config,
        })
    }
    
    /// Create indexes for performance optimization
    pub async fn create_indexes(&self) -> EventResult<()> {
        let mut conn = self.get_connection()?;
        
        // Create performance indexes if they don't exist
        let index_queries = vec![
            // Composite index for aggregate reconstruction
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_event_store_aggregate_version 
             ON event_store(aggregate_id, aggregate_version)",
            
            // Index for event type queries
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_event_store_event_type_time 
             ON event_store(event_type, occurred_at DESC)",
            
            // Index for correlation ID lookups
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_event_store_correlation 
             ON event_store(correlation_id) WHERE correlation_id IS NOT NULL",
            
            // BRIN index for time-based queries (efficient for large datasets)
            "CREATE INDEX IF NOT EXISTS idx_event_store_recorded_brin 
             ON event_store USING BRIN(recorded_at)",
            
            // GIN index for JSONB queries
            "CREATE INDEX IF NOT EXISTS idx_event_store_metadata_gin 
             ON event_store USING GIN(metadata)",
        ];
        
        for query in index_queries {
            diesel::sql_query(query)
                .execute(&mut conn)
                .map_err(|e| EventError::DatabaseError {
                    message: format!("Failed to create index: {}", e),
                })?;
        }
        
        Ok(())
    }
    
    /// Get database statistics for monitoring
    pub async fn get_statistics(&self) -> EventResult<EventStoreStatistics> {
        let mut conn = self.get_connection()?;
        
        #[derive(diesel::QueryableByName)]
        struct Stats {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            total_events: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            total_aggregates: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            total_snapshots: i64,
        }
        
        let stats: Stats = diesel::sql_query(
            r#"
            SELECT 
                (SELECT COUNT(*) FROM event_store) as total_events,
                (SELECT COUNT(DISTINCT aggregate_id) FROM event_store) as total_aggregates,
                (SELECT COUNT(*) FROM event_snapshots) as total_snapshots
            "#
        )
        .get_result(&mut conn)
        .map_err(|e| EventError::DatabaseError {
            message: format!("Failed to get statistics: {}", e),
        })?;
        
        Ok(EventStoreStatistics {
            total_events: stats.total_events as u64,
            total_aggregates: stats.total_aggregates as u64,
            total_snapshots: stats.total_snapshots as u64,
        })
    }

    /// Get a database connection from the pool
    fn get_connection(&self) -> EventResult<diesel::r2d2::PooledConnection<ConnectionManager<PgConnection>>> {
        self.pool.get().map_err(|e| EventError::DatabaseError {
            message: format!("Failed to get database connection: {}", e),
        })
    }

    /// Convert EventEnvelope to database model
    fn event_to_db_model(&self, event: &EventEnvelope) -> EventStoreRecord {
        EventStoreRecord {
            id: event.event_id,
            aggregate_id: event.aggregate_id,
            aggregate_type: event.aggregate_type.clone(),
            event_type: event.event_type.clone(),
            aggregate_version: event.aggregate_version,
            event_data: event.event_data.clone(),
            metadata: serde_json::to_value(&event.metadata).unwrap_or_default(),
            occurred_at: event.occurred_at,
            recorded_at: event.recorded_at,
            schema_version: event.schema_version,
            causation_id: event.causation_id,
            correlation_id: event.correlation_id,
            checksum: event.checksum.clone(),
        }
    }

    /// Convert database model to EventEnvelope
    fn db_model_to_event(&self, record: EventStoreRecord) -> EventResult<EventEnvelope> {
        let metadata: EventMetadata = serde_json::from_value(record.metadata)
            .map_err(|e| EventError::SerializationError {
                message: format!("Failed to deserialize event metadata: {}", e),
            })?;

        Ok(EventEnvelope {
            event_id: record.id,
            aggregate_id: record.aggregate_id,
            aggregate_type: record.aggregate_type,
            event_type: record.event_type,
            aggregate_version: record.aggregate_version,
            event_data: record.event_data,
            metadata,
            occurred_at: record.occurred_at,
            recorded_at: record.recorded_at,
            schema_version: record.schema_version,
            causation_id: record.causation_id,
            correlation_id: record.correlation_id,
            checksum: record.checksum,
        })
    }

    /// Calculate checksum for event data integrity
    fn calculate_checksum(&self, event_data: &Value, metadata: &Value) -> String {
        use sha2::{Sha256, Digest};
        let combined = format!("{}{}", event_data, metadata);
        let mut hasher = Sha256::new();
        hasher.update(combined.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[async_trait]
impl EventStore for PostgreSQLEventStore {
    async fn append_event(&self, event: &EventEnvelope) -> EventResult<()> {
        let mut conn = self.get_connection()?;
        
        // Check for concurrency conflicts
        let current_version = self.get_aggregate_version(event.aggregate_id).await.unwrap_or(0);
        if event.aggregate_version <= current_version {
            return Err(EventError::ConcurrencyError {
                message: format!(
                    "Concurrency conflict: expected version > {}, got {}",
                    current_version, event.aggregate_version
                ),
            });
        }

        let mut db_event = self.event_to_db_model(event);
        
        // Calculate checksum if enabled
        if self.config.enable_checksums {
            db_event.checksum = Some(self.calculate_checksum(&db_event.event_data, &db_event.metadata));
        }

        // Insert the event
        diesel::insert_into(event_store::table)
            .values(&db_event)
            .execute(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to insert event: {}", e),
            })?;

        Ok(())
    }

    async fn append_events(&self, events: &[EventEnvelope]) -> EventResult<()> {
        if events.is_empty() {
            return Ok(());
        }

        let mut conn = self.get_connection()?;
        
        // Start transaction
        conn.transaction::<_, EventError, _>(|conn| {
            for event in events {
                // Check for concurrency conflicts
                let current_version: i64 = event_store::table
                    .filter(event_store::aggregate_id.eq(event.aggregate_id))
                    .select(diesel::dsl::max(event_store::aggregate_version))
                    .first::<Option<i64>>(conn)
                    .map_err(|e| EventError::DatabaseError {
                        message: format!("Failed to get current version: {}", e),
                    })?
                    .unwrap_or(0);

                if event.aggregate_version <= current_version {
                    return Err(EventError::ConcurrencyError {
                        message: format!(
                            "Concurrency conflict in batch: expected version > {}, got {}",
                            current_version, event.aggregate_version
                        ),
                    });
                }

                let mut db_event = self.event_to_db_model(event);
                
                // Calculate checksum if enabled
                if self.config.enable_checksums {
                    db_event.checksum = Some(self.calculate_checksum(&db_event.event_data, &db_event.metadata));
                }

                // Insert the event
                diesel::insert_into(event_store::table)
                    .values(&db_event)
                    .execute(conn)
                    .map_err(|e| EventError::DatabaseError {
                        message: format!("Failed to insert event in batch: {}", e),
                    })?;
            }
            Ok(())
        }).map_err(|e| e)?;

        Ok(())
    }

    async fn get_events(&self, aggregate_id: Uuid) -> EventResult<Vec<EventEnvelope>> {
        let mut conn = self.get_connection()?;
        
        let records: Vec<EventStoreRecord> = event_store::table
            .filter(event_store::aggregate_id.eq(aggregate_id))
            .order(event_store::aggregate_version.asc())
            .load(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to load events: {}", e),
            })?;

        let events: Result<Vec<EventEnvelope>, EventError> = records
            .into_iter()
            .map(|record| self.db_model_to_event(record))
            .collect();

        events
    }

    async fn get_events_from_version(
        &self,
        aggregate_id: Uuid,
        from_version: i64,
    ) -> EventResult<Vec<EventEnvelope>> {
        let mut conn = self.get_connection()?;
        
        let records: Vec<EventStoreRecord> = event_store::table
            .filter(event_store::aggregate_id.eq(aggregate_id))
            .filter(event_store::aggregate_version.gt(from_version))
            .order(event_store::aggregate_version.asc())
            .load(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to load events from version: {}", e),
            })?;

        let events: Result<Vec<EventEnvelope>, EventError> = records
            .into_iter()
            .map(|record| self.db_model_to_event(record))
            .collect();

        events
    }

    async fn get_events_by_type(
        &self,
        event_type: &str,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        limit: Option<usize>,
    ) -> EventResult<Vec<EventEnvelope>> {
        let mut conn = self.get_connection()?;
        
        let mut query = event_store::table
            .filter(event_store::event_type.eq(event_type))
            .into_boxed();

        if let Some(from_date) = from {
            query = query.filter(event_store::occurred_at.ge(from_date));
        }

        if let Some(to_date) = to {
            query = query.filter(event_store::occurred_at.le(to_date));
        }

        query = query.order(event_store::occurred_at.asc());

        if let Some(limit_size) = limit {
            query = query.limit(limit_size as i64);
        }

        let records: Vec<EventStoreRecord> = query
            .load(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to load events by type: {}", e),
            })?;

        let events: Result<Vec<EventEnvelope>, EventError> = records
            .into_iter()
            .map(|record| self.db_model_to_event(record))
            .collect();

        events
    }

    async fn get_events_by_correlation_id(&self, correlation_id: Uuid) -> EventResult<Vec<EventEnvelope>> {
        let mut conn = self.get_connection()?;
        
        let records: Vec<EventStoreRecord> = event_store::table
            .filter(event_store::correlation_id.eq(correlation_id))
            .order(event_store::occurred_at.asc())
            .load(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to load events by correlation ID: {}", e),
            })?;

        let events: Result<Vec<EventEnvelope>, EventError> = records
            .into_iter()
            .map(|record| self.db_model_to_event(record))
            .collect();

        events
    }

    async fn get_aggregate_version(&self, aggregate_id: Uuid) -> EventResult<i64> {
        let mut conn = self.get_connection()?;
        
        let version: Option<i64> = event_store::table
            .filter(event_store::aggregate_id.eq(aggregate_id))
            .select(diesel::dsl::max(event_store::aggregate_version))
            .first(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to get aggregate version: {}", e),
            })?;

        Ok(version.unwrap_or(0))
    }

    async fn aggregate_exists(&self, aggregate_id: Uuid) -> EventResult<bool> {
        let mut conn = self.get_connection()?;
        
        let count: i64 = event_store::table
            .filter(event_store::aggregate_id.eq(aggregate_id))
            .count()
            .get_result(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to check aggregate existence: {}", e),
            })?;

        Ok(count > 0)
    }

    async fn save_snapshot(&self, snapshot: &AggregateSnapshot) -> EventResult<()> {
        let mut conn = self.get_connection()?;
        
        let db_snapshot = SnapshotRecord {
            id: snapshot.id,
            aggregate_id: snapshot.aggregate_id,
            aggregate_type: snapshot.aggregate_type.clone(),
            aggregate_version: snapshot.aggregate_version,
            snapshot_data: snapshot.snapshot_data.clone(),
            created_at: snapshot.created_at,
            metadata: serde_json::to_value(&snapshot.metadata).unwrap_or_default(),
        };

        // Upsert the snapshot (replace if exists)
        diesel::insert_into(event_snapshots::table)
            .values(&db_snapshot)
            .on_conflict(event_snapshots::aggregate_id)
            .do_update()
            .set((
                event_snapshots::aggregate_version.eq(&db_snapshot.aggregate_version),
                event_snapshots::snapshot_data.eq(&db_snapshot.snapshot_data),
                event_snapshots::created_at.eq(&db_snapshot.created_at),
                event_snapshots::metadata.eq(&db_snapshot.metadata),
            ))
            .execute(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to save snapshot: {}", e),
            })?;

        Ok(())
    }

    async fn get_snapshot(&self, aggregate_id: Uuid) -> EventResult<Option<AggregateSnapshot>> {
        let mut conn = self.get_connection()?;
        
        let record: Option<SnapshotRecord> = event_snapshots::table
            .filter(event_snapshots::aggregate_id.eq(aggregate_id))
            .first(&mut conn)
            .optional()
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to load snapshot: {}", e),
            })?;

        if let Some(record) = record {
            let metadata: HashMap<String, Value> = serde_json::from_value(record.metadata)
                .map_err(|e| EventError::SerializationError {
                    message: format!("Failed to deserialize snapshot metadata: {}", e),
                })?;

            Ok(Some(AggregateSnapshot {
                id: record.id,
                aggregate_id: record.aggregate_id,
                aggregate_type: record.aggregate_type,
                aggregate_version: record.aggregate_version,
                snapshot_data: record.snapshot_data,
                created_at: record.created_at,
                metadata,
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_events_from_position(&self, position: i64, limit: usize) -> EventResult<Vec<EventEnvelope>> {
        let mut conn = self.get_connection()?;
        
        // Use recorded_at timestamp as position for better portability
        let from_time = DateTime::from_timestamp_millis(position)
            .unwrap_or_else(|| DateTime::<Utc>::MIN_UTC);
        
        let records: Vec<EventStoreRecord> = event_store::table
            .filter(event_store::recorded_at.gt(from_time))
            .order(event_store::recorded_at.asc())
            .order(event_store::id.asc()) // Secondary sort for consistent ordering
            .limit(limit as i64)
            .load(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to load events from position: {}", e),
            })?;

        let events: Result<Vec<EventEnvelope>, EventError> = records
            .into_iter()
            .map(|record| self.db_model_to_event(record))
            .collect();

        events
    }

    async fn get_current_position(&self) -> EventResult<i64> {
        let mut conn = self.get_connection()?;
        
        // Get the highest recorded_at timestamp as a simple position marker
        let latest_time: Option<DateTime<Utc>> = event_store::table
            .select(diesel::dsl::max(event_store::recorded_at))
            .first(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to get current position: {}", e),
            })?;

        Ok(latest_time.map(|t| t.timestamp_millis()).unwrap_or(0))
    }

    async fn replay_events(
        &self,
        from_position: i64,
        event_types: Option<Vec<String>>,
        batch_size: usize,
    ) -> EventResult<Vec<EventEnvelope>> {
        let mut conn = self.get_connection()?;
        
        let from_time = DateTime::from_timestamp_millis(from_position)
            .unwrap_or_else(|| DateTime::<Utc>::MIN_UTC);
        
        let mut query = event_store::table
            .filter(event_store::recorded_at.ge(from_time))
            .into_boxed();

        if let Some(types) = event_types {
            query = query.filter(event_store::event_type.eq_any(types));
        }

        let records: Vec<EventStoreRecord> = query
            .order(event_store::recorded_at.asc())
            .order(event_store::id.asc()) // Secondary sort for consistent ordering
            .limit(batch_size as i64)
            .load(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to replay events: {}", e),
            })?;

        let events: Result<Vec<EventEnvelope>, EventError> = records
            .into_iter()
            .map(|record| self.db_model_to_event(record))
            .collect();

        events
    }
    
    async fn get_events_for_aggregates(&self, aggregate_ids: &[Uuid]) -> EventResult<Vec<EventEnvelope>> {
        if aggregate_ids.is_empty() {
            return Ok(vec![]);
        }
        
        let mut conn = self.get_connection()?;
        
        let records: Vec<EventStoreRecord> = event_store::table
            .filter(event_store::aggregate_id.eq_any(aggregate_ids))
            .order(event_store::aggregate_id.asc())
            .order(event_store::aggregate_version.asc())
            .load(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to load events for multiple aggregates: {}", e),
            })?;
        
        let events: Result<Vec<EventEnvelope>, EventError> = records
            .into_iter()
            .map(|record| self.db_model_to_event(record))
            .collect();
        
        events
    }
    
    async fn cleanup_old_snapshots(&self, keep_latest: usize) -> EventResult<usize> {
        let mut conn = self.get_connection()?;
        
        // Complex query to delete old snapshots keeping only the latest N per aggregate
        let delete_query = diesel::sql_query(
            r#"
            DELETE FROM event_snapshots
            WHERE id IN (
                SELECT id FROM (
                    SELECT id,
                           ROW_NUMBER() OVER (PARTITION BY aggregate_id ORDER BY created_at DESC) as rn
                    FROM event_snapshots
                ) t
                WHERE t.rn > $1
            )
            "#
        ).bind::<diesel::sql_types::Integer, _>(keep_latest as i32);
        
        let deleted_count = delete_query
            .execute(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to cleanup old snapshots: {}", e),
            })?;
        
        Ok(deleted_count)
    }
    
    async fn get_aggregate_ids_by_type(
        &self,
        aggregate_type: &str,
        offset: i64,
        limit: usize,
    ) -> EventResult<Vec<Uuid>> {
        let mut conn = self.get_connection()?;
        
        let aggregate_ids: Vec<Uuid> = event_store::table
            .select(event_store::aggregate_id)
            .filter(event_store::aggregate_type.eq(aggregate_type))
            .distinct()
            .offset(offset)
            .limit(limit as i64)
            .order(event_store::aggregate_id.asc())
            .load(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to get aggregate IDs by type: {}", e),
            })?;
        
        Ok(aggregate_ids)
    }
    
    async fn optimize_storage(&self) -> EventResult<()> {
        let mut conn = self.get_connection()?;
        
        // Run VACUUM ANALYZE on event tables for PostgreSQL optimization
        diesel::sql_query("VACUUM ANALYZE event_store")
            .execute(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to vacuum event_store table: {}", e),
            })?;
            
        diesel::sql_query("VACUUM ANALYZE event_snapshots")
            .execute(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to vacuum event_snapshots table: {}", e),
            })?;
            
        Ok(())
    }
}

// Implement EventStreaming for PostgreSQLEventStore
#[async_trait]
impl EventStreaming for PostgreSQLEventStore {
    async fn stream_events<F>(
        &self,
        from_position: i64,
        event_types: Option<Vec<String>>,
        batch_size: usize,
        callback: F,
    ) -> EventResult<i64>
    where
        F: Fn(&[EventEnvelope]) -> EventResult<bool> + Send + Sync,
    {
        let mut current_position = from_position;
        let mut continue_streaming = true;
        
        while continue_streaming {
            let events = self.replay_events(
                current_position,
                event_types.clone(),
                batch_size,
            ).await?;
            
            if events.is_empty() {
                break;
            }
            
            // Update position to the last event's timestamp
            if let Some(last_event) = events.last() {
                current_position = last_event.recorded_at.timestamp_millis();
            }
            
            // Call the callback and check if we should continue
            continue_streaming = callback(&events)?;
        }
        
        Ok(current_position)
    }
}

// Database model structs
#[derive(Debug, Clone, Queryable, Insertable, Selectable)]
#[diesel(table_name = event_store)]
struct EventStoreRecord {
    pub id: Uuid,
    pub aggregate_id: Uuid,
    pub aggregate_type: String,
    pub event_type: String,
    pub aggregate_version: i64,
    pub event_data: Value,
    pub metadata: Value,
    pub occurred_at: DateTime<Utc>,
    pub recorded_at: DateTime<Utc>,
    pub schema_version: i32,
    pub causation_id: Option<Uuid>,
    pub correlation_id: Option<Uuid>,
    pub checksum: Option<String>,
}

#[derive(Debug, Clone, Queryable, Insertable, Selectable)]
#[diesel(table_name = event_snapshots)]
struct SnapshotRecord {
    pub id: Uuid,
    pub aggregate_id: Uuid,
    pub aggregate_type: String,
    pub aggregate_version: i64,
    pub snapshot_data: Value,
    pub created_at: DateTime<Utc>,
    pub metadata: Value,
}

/// Aggregate snapshot for performance optimization
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AggregateSnapshot {
    pub id: Uuid,
    pub aggregate_id: Uuid,
    pub aggregate_type: String,
    pub aggregate_version: i64,
    pub snapshot_data: Value,
    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, Value>,
}

impl AggregateSnapshot {
    pub fn new(
        aggregate_id: Uuid,
        aggregate_type: String,
        aggregate_version: i64,
        snapshot_data: Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            aggregate_id,
            aggregate_type,
            aggregate_version,
            snapshot_data,
            created_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }
}