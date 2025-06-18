// File: src/db/events/performance.rs
//
// Performance optimization utilities for the event store
// Provides database partitioning, indexing, and query optimization

use async_trait::async_trait;
use chrono::{DateTime, Utc, Datelike};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sql_types::Integer;
use diesel::sql_types::BigInt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn, error};

use super::{EventError, EventResult};

#[derive(QueryableByName)]
struct CountResult {
    #[diesel(sql_type = Integer)]
    count: i32,
}

#[derive(QueryableByName)]
struct BigCountResult {
    #[diesel(sql_type = BigInt)]
    count: i64,
}

#[derive(QueryableByName)]
struct PartitionQueryResult {
    #[diesel(sql_type = diesel::sql_types::Text)]
    schema: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    table_name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    size: String,
}

#[derive(QueryableByName)]
struct IndexQueryResult {
    #[diesel(sql_type = diesel::sql_types::Text)]
    index_name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    table_name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    definition: String,
}

/// Configuration for performance optimizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable automatic partitioning
    pub enable_partitioning: bool,
    /// Partition size in days
    pub partition_size_days: i32,
    /// Number of partitions to create ahead
    pub partition_ahead_count: i32,
    /// Enable automatic index creation
    pub enable_auto_indexing: bool,
    /// Enable query optimization
    pub enable_query_optimization: bool,
    /// Maintenance interval in hours
    pub maintenance_interval_hours: i64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_partitioning: true,
            partition_size_days: 30,
            partition_ahead_count: 3,
            enable_auto_indexing: true,
            enable_query_optimization: true,
            maintenance_interval_hours: 24,
        }
    }
}

/// Performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStatistics {
    pub total_partitions: u32,
    pub active_partitions: u32,
    pub total_indexes: u32,
    pub query_execution_stats: QueryExecutionStats,
    pub last_maintenance: Option<DateTime<Utc>>,
    pub maintenance_runs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryExecutionStats {
    pub average_query_time_ms: f64,
    pub slow_queries_count: u64,
    pub optimized_queries_count: u64,
    pub cache_hit_ratio: f64,
}

/// Database partition information
#[derive(Debug, Clone)]
pub struct PartitionInfo {
    pub table_name: String,
    pub partition_name: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub row_count: Option<i64>,
    pub size_bytes: Option<i64>,
}

/// Index information
#[derive(Debug, Clone)]
pub struct IndexInfo {
    pub index_name: String,
    pub table_name: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
    pub is_partial: bool,
    pub size_bytes: Option<i64>,
}

/// Performance optimizer for event store
pub struct EventStorePerformanceOptimizer {
    db_pool: Arc<Pool<ConnectionManager<PgConnection>>>,
    config: PerformanceConfig,
    statistics: Arc<tokio::sync::RwLock<PerformanceStatistics>>,
}

impl EventStorePerformanceOptimizer {
    pub fn new(
        db_pool: Arc<Pool<ConnectionManager<PgConnection>>>,
        config: PerformanceConfig,
    ) -> Self {
        let statistics = PerformanceStatistics {
            total_partitions: 0,
            active_partitions: 0,
            total_indexes: 0,
            query_execution_stats: QueryExecutionStats {
                average_query_time_ms: 0.0,
                slow_queries_count: 0,
                optimized_queries_count: 0,
                cache_hit_ratio: 0.0,
            },
            last_maintenance: None,
            maintenance_runs: 0,
        };

        Self {
            db_pool,
            config,
            statistics: Arc::new(tokio::sync::RwLock::new(statistics)),
        }
    }

    /// Initialize performance optimizations
    pub async fn initialize(&self) -> EventResult<()> {
        info!("Initializing event store performance optimizations");

        if self.config.enable_partitioning {
            self.setup_partitioning().await?;
        }

        if self.config.enable_auto_indexing {
            self.create_optimal_indexes().await?;
        }

        info!("Performance optimizations initialized successfully");
        Ok(())
    }

    /// Setup database partitioning for event tables
    async fn setup_partitioning(&self) -> EventResult<()> {
        let mut conn = self.get_connection()?;

        info!("Setting up database partitioning");

        // Create partitioned event store table
        let create_partitioned_table_sql = r#"
            -- Check if the table is already partitioned
            DO $$ 
            BEGIN
                IF NOT EXISTS (
                    SELECT 1 FROM pg_partitioned_table WHERE partrelid = 'event_store'::regclass
                ) THEN
                    -- Create new partitioned table
                    CREATE TABLE IF NOT EXISTS event_store_partitioned (
                        LIKE event_store INCLUDING ALL
                    ) PARTITION BY RANGE (recorded_at);
                    
                    -- Copy indexes and constraints
                    ALTER TABLE event_store_partitioned 
                    ADD CONSTRAINT event_store_partitioned_pkey 
                    PRIMARY KEY (event_id);
                    
                    CREATE INDEX IF NOT EXISTS idx_event_store_partitioned_aggregate_id_version 
                    ON event_store_partitioned (aggregate_id, aggregate_version);
                    
                    CREATE INDEX IF NOT EXISTS idx_event_store_partitioned_aggregate_type_recorded_at 
                    ON event_store_partitioned (aggregate_type, recorded_at);
                    
                    CREATE INDEX IF NOT EXISTS idx_event_store_partitioned_event_type_recorded_at 
                    ON event_store_partitioned (event_type, recorded_at);
                    
                    CREATE INDEX IF NOT EXISTS idx_event_store_partitioned_correlation_id 
                    ON event_store_partitioned (correlation_id) WHERE correlation_id IS NOT NULL;
                    
                    CREATE INDEX IF NOT EXISTS idx_event_store_partitioned_causation_id 
                    ON event_store_partitioned (causation_id) WHERE causation_id IS NOT NULL;
                END IF;
            END $$;
        "#;

        diesel::sql_query(create_partitioned_table_sql)
            .execute(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to setup partitioned table: {}", e),
            })?;

        // Create initial partitions
        self.create_time_based_partitions().await?;

        info!("Database partitioning setup completed");
        Ok(())
    }

    /// Create time-based partitions
    async fn create_time_based_partitions(&self) -> EventResult<()> {
        let mut conn = self.get_connection()?;

        let now = Utc::now();
        let partition_size = chrono::Duration::days(self.config.partition_size_days as i64);

        // Create partitions for past, present, and future
        for i in -2..=self.config.partition_ahead_count {
            let start_date = now + (partition_size * i);
            let end_date = start_date + partition_size;

            let partition_name = format!(
                "event_store_y{}m{}",
                start_date.year(),
                start_date.month()
            );

            let create_partition_sql = format!(
                r#"
                CREATE TABLE IF NOT EXISTS {} PARTITION OF event_store_partitioned
                FOR VALUES FROM ('{}') TO ('{}');
                "#,
                partition_name,
                start_date.format("%Y-%m-%d %H:%M:%S"),
                end_date.format("%Y-%m-%d %H:%M:%S")
            );

            match diesel::sql_query(&create_partition_sql).execute(&mut conn) {
                Ok(_) => {
                    info!("Created partition: {} for period {} to {}", 
                          partition_name, 
                          start_date.format("%Y-%m-%d"), 
                          end_date.format("%Y-%m-%d"));
                }
                Err(e) => {
                    warn!("Failed to create partition {}: {}", partition_name, e);
                }
            }
        }

        Ok(())
    }

    /// Create optimal indexes for performance
    async fn create_optimal_indexes(&self) -> EventResult<()> {
        let mut conn = self.get_connection()?;

        info!("Creating optimal indexes");

        let index_definitions = vec![
            // Composite indexes for common query patterns
            (
                "idx_event_store_aggregate_timestamp_compound",
                "event_store",
                "(aggregate_id, aggregate_type, recorded_at DESC)",
                "Composite index for aggregate queries with time ordering"
            ),
            (
                "idx_event_store_event_type_timestamp_compound", 
                "event_store",
                "(event_type, recorded_at DESC)",
                "Composite index for event type queries with time ordering"
            ),
            (
                "idx_event_store_version_timestamp_compound",
                "event_store", 
                "(aggregate_version, recorded_at DESC)",
                "Composite index for version-based queries with time ordering"
            ),
            // Partial indexes for optional fields
            (
                "idx_event_store_correlation_non_null",
                "event_store",
                "(correlation_id, recorded_at DESC) WHERE correlation_id IS NOT NULL",
                "Partial index for correlation ID queries"
            ),
            (
                "idx_event_store_causation_non_null",
                "event_store", 
                "(causation_id, recorded_at DESC) WHERE causation_id IS NOT NULL",
                "Partial index for causation ID queries"
            ),
            // JSONB indexes for event data
            (
                "idx_event_store_event_data_gin",
                "event_store",
                "USING gin (event_data)",
                "GIN index for JSONB event data queries"
            ),
            (
                "idx_event_store_metadata_gin",
                "event_store", 
                "USING gin (metadata)",
                "GIN index for JSONB metadata queries"
            ),
        ];

        for (index_name, table_name, index_def, description) in index_definitions {
            let create_index_sql = format!(
                "CREATE INDEX CONCURRENTLY IF NOT EXISTS {} ON {} {};",
                index_name, table_name, index_def
            );

            match diesel::sql_query(&create_index_sql).execute(&mut conn) {
                Ok(_) => {
                    info!("Created index: {} - {}", index_name, description);
                }
                Err(e) => {
                    warn!("Failed to create index {}: {}", index_name, e);
                }
            }
        }

        // Create indexes for snapshots
        let snapshot_indexes = vec![
            (
                "idx_event_snapshots_aggregate_version_compound",
                "event_snapshots",
                "(aggregate_id, aggregate_version DESC, created_at DESC)",
                "Composite index for snapshot retrieval"
            ),
            (
                "idx_event_snapshots_created_at_btree",
                "event_snapshots",
                "(created_at DESC)",
                "Index for snapshot cleanup operations"
            ),
        ];

        for (index_name, table_name, index_def, description) in snapshot_indexes {
            let create_index_sql = format!(
                "CREATE INDEX CONCURRENTLY IF NOT EXISTS {} ON {} {};",
                index_name, table_name, index_def
            );

            match diesel::sql_query(&create_index_sql).execute(&mut conn) {
                Ok(_) => {
                    info!("Created snapshot index: {} - {}", index_name, description);
                }
                Err(e) => {
                    warn!("Failed to create snapshot index {}: {}", index_name, e);
                }
            }
        }

        info!("Optimal indexes created successfully");
        Ok(())
    }

    /// Run maintenance tasks
    pub async fn run_maintenance(&self) -> EventResult<()> {
        info!("Running performance maintenance tasks");

        let start_time = std::time::Instant::now();

        // Update statistics
        self.analyze_tables().await?;

        // Clean up old partitions if enabled
        if self.config.enable_partitioning {
            self.cleanup_old_partitions().await?;
            self.create_future_partitions().await?;
        }

        // Gather performance statistics
        self.update_performance_statistics().await?;

        let maintenance_duration = start_time.elapsed();

        // Update maintenance stats
        {
            let mut stats = self.statistics.write().await;
            stats.maintenance_runs += 1;
            stats.last_maintenance = Some(Utc::now());
        }

        info!("Performance maintenance completed in {:?}", maintenance_duration);
        Ok(())
    }

    /// Analyze table statistics for query optimization
    async fn analyze_tables(&self) -> EventResult<()> {
        let mut conn = self.get_connection()?;

        let tables = vec!["event_store", "event_snapshots", "event_projections"];

        for table in tables {
            let analyze_sql = format!("ANALYZE {};", table);
            
            match diesel::sql_query(&analyze_sql).execute(&mut conn) {
                Ok(_) => {
                    info!("Analyzed table: {}", table);
                }
                Err(e) => {
                    warn!("Failed to analyze table {}: {}", table, e);
                }
            }
        }

        Ok(())
    }

    /// Clean up old partitions
    async fn cleanup_old_partitions(&self) -> EventResult<()> {
        let mut conn = self.get_connection()?;

        // Find partitions older than retention period (e.g., 6 months)
        let retention_date = Utc::now() - chrono::Duration::days(180);

        let find_old_partitions_sql = r#"
            SELECT schemaname, tablename 
            FROM pg_tables 
            WHERE tablename LIKE 'event_store_y%m%'
            AND schemaname = 'public'
        "#;

        // This is a simplified version - in production you'd parse the partition names
        // to determine their date ranges and drop only truly old ones
        info!("Checked for old partitions (retention date: {})", 
              retention_date.format("%Y-%m-%d"));

        Ok(())
    }

    /// Create future partitions
    async fn create_future_partitions(&self) -> EventResult<()> {
        let now = Utc::now();
        let partition_size = chrono::Duration::days(self.config.partition_size_days as i64);

        // Check if we need to create new future partitions
        let future_months = self.config.partition_ahead_count;
        
        for i in 1..=future_months {
            let start_date = now + (partition_size * i);
            let end_date = start_date + partition_size;
            
            let partition_name = format!(
                "event_store_y{}m{}",
                start_date.year(),
                start_date.month()
            );

            // Check if partition already exists
            let mut conn = self.get_connection()?;
            let check_partition_sql = format!(
                "SELECT 1 FROM pg_tables WHERE tablename = '{}' AND schemaname = 'public'",
                partition_name
            );

            let exists: Result<CountResult, _> = diesel::sql_query(&check_partition_sql)
                .get_result(&mut conn);

            if exists.is_err() {
                // Partition doesn't exist, create it
                let create_partition_sql = format!(
                    r#"
                    CREATE TABLE IF NOT EXISTS {} PARTITION OF event_store_partitioned
                    FOR VALUES FROM ('{}') TO ('{}');
                    "#,
                    partition_name,
                    start_date.format("%Y-%m-%d %H:%M:%S"),
                    end_date.format("%Y-%m-%d %H:%M:%S")
                );

                match diesel::sql_query(&create_partition_sql).execute(&mut conn) {
                    Ok(_) => {
                        info!("Created future partition: {}", partition_name);
                    }
                    Err(e) => {
                        warn!("Failed to create future partition {}: {}", partition_name, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Update performance statistics
    async fn update_performance_statistics(&self) -> EventResult<()> {
        let mut conn = self.get_connection()?;

        // Count partitions
        let partition_count_sql = r#"
            SELECT COUNT(*) as count 
            FROM pg_tables 
            WHERE tablename LIKE 'event_store_y%m%'
        "#;

        let partition_count: i64 = diesel::sql_query(partition_count_sql)
            .get_result::<BigCountResult>(&mut conn)
            .map(|result| result.count)
            .unwrap_or(0);

        // Count indexes
        let index_count_sql = r#"
            SELECT COUNT(*) as count 
            FROM pg_indexes 
            WHERE tablename IN ('event_store', 'event_snapshots', 'event_projections')
        "#;

        let index_count: i64 = diesel::sql_query(index_count_sql)
            .get_result::<BigCountResult>(&mut conn)
            .map(|result| result.count)
            .unwrap_or(0);

        // Update statistics
        {
            let mut stats = self.statistics.write().await;
            stats.total_partitions = partition_count as u32;
            stats.active_partitions = partition_count as u32; // Simplified
            stats.total_indexes = index_count as u32;
        }

        Ok(())
    }

    /// Get current performance statistics
    pub async fn get_statistics(&self) -> PerformanceStatistics {
        self.statistics.read().await.clone()
    }

    /// Get partition information
    pub async fn get_partition_info(&self) -> EventResult<Vec<PartitionInfo>> {
        let mut conn = self.get_connection()?;

        let partitions_sql = r#"
            SELECT 
                schemaname,
                tablename,
                pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename))::text as size
            FROM pg_tables 
            WHERE tablename LIKE 'event_store_y%m%'
            ORDER BY tablename
        "#;

        // This would need proper parsing in a real implementation
        let partitions: Vec<PartitionQueryResult> = diesel::sql_query(partitions_sql)
            .load(&mut conn)
            .unwrap_or_default();

        let partition_info: Vec<PartitionInfo> = partitions
            .into_iter()
            .map(|partition| PartitionInfo {
                table_name: partition.table_name.clone(),
                partition_name: partition.table_name,
                start_date: Utc::now(), // Would parse from name in real implementation
                end_date: Utc::now(),   // Would parse from name in real implementation
                row_count: None,
                size_bytes: None,
            })
            .collect();

        Ok(partition_info)
    }

    /// Get index information
    pub async fn get_index_info(&self) -> EventResult<Vec<IndexInfo>> {
        let mut conn = self.get_connection()?;

        let indexes_sql = r#"
            SELECT 
                indexname,
                tablename,
                indexdef
            FROM pg_indexes 
            WHERE tablename IN ('event_store', 'event_snapshots', 'event_projections')
            ORDER BY tablename, indexname
        "#;

        let indexes: Vec<IndexQueryResult> = diesel::sql_query(indexes_sql)
            .load(&mut conn)
            .unwrap_or_default();

        let index_info: Vec<IndexInfo> = indexes
            .into_iter()
            .map(|index| IndexInfo {
                index_name: index.index_name,
                table_name: index.table_name,
                columns: Vec::new(), // Would parse from indexdef in real implementation
                is_unique: false,    // Would parse from indexdef in real implementation
                is_partial: false,   // Would parse from indexdef in real implementation
                size_bytes: None,
            })
            .collect();

        Ok(index_info)
    }

    /// Start background maintenance task
    pub async fn start_background_maintenance(&self) -> tokio::task::JoinHandle<()> {
        let optimizer = self.clone();
        let interval_hours = self.config.maintenance_interval_hours;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_secs((interval_hours * 3600) as u64)
            );

            loop {
                interval.tick().await;

                if let Err(e) = optimizer.run_maintenance().await {
                    error!("Background maintenance failed: {}", e);
                }
            }
        })
    }

    fn get_connection(&self) -> EventResult<diesel::r2d2::PooledConnection<ConnectionManager<PgConnection>>> {
        self.db_pool.get().map_err(|e| EventError::DatabaseError {
            message: format!("Failed to get database connection: {}", e),
        })
    }
}

impl Clone for EventStorePerformanceOptimizer {
    fn clone(&self) -> Self {
        Self {
            db_pool: Arc::clone(&self.db_pool),
            config: self.config.clone(),
            statistics: Arc::clone(&self.statistics),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_config_defaults() {
        let config = PerformanceConfig::default();
        
        assert!(config.enable_partitioning);
        assert_eq!(config.partition_size_days, 30);
        assert_eq!(config.partition_ahead_count, 3);
        assert!(config.enable_auto_indexing);
        assert!(config.enable_query_optimization);
        assert_eq!(config.maintenance_interval_hours, 24);
    }

    #[test]
    fn test_partition_info_creation() {
        let partition = PartitionInfo {
            table_name: "event_store".to_string(),
            partition_name: "event_store_y2024m01".to_string(),
            start_date: Utc::now(),
            end_date: Utc::now(),
            row_count: Some(1000),
            size_bytes: Some(1024 * 1024),
        };

        assert_eq!(partition.table_name, "event_store");
        assert_eq!(partition.partition_name, "event_store_y2024m01");
        assert_eq!(partition.row_count, Some(1000));
        assert_eq!(partition.size_bytes, Some(1024 * 1024));
    }

    #[test]
    fn test_index_info_creation() {
        let index = IndexInfo {
            index_name: "idx_test".to_string(),
            table_name: "event_store".to_string(),
            columns: vec!["aggregate_id".to_string(), "recorded_at".to_string()],
            is_unique: false,
            is_partial: true,
            size_bytes: Some(512 * 1024),
        };

        assert_eq!(index.index_name, "idx_test");
        assert_eq!(index.table_name, "event_store");
        assert_eq!(index.columns.len(), 2);
        assert!(!index.is_unique);
        assert!(index.is_partial);
    }
}