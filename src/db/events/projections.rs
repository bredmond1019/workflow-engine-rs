// File: src/db/events/projections.rs
//
// Event projections for creating read models from event streams

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::db::schema::event_projections;
use super::{EventError, EventResult, EventEnvelope, EventStore};

/// Projection state for tracking progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectionState {
    Building,
    Active,
    Failed,
    Rebuilding,
    Stopped,
    Ready,
}

/// Trait for event projections that build read models
#[async_trait]
pub trait Projection: Send + Sync {
    /// Get the projection name
    fn name(&self) -> &str;
    
    /// Get the event types this projection is interested in
    fn event_types(&self) -> Vec<String>;
    
    /// Process an event and update the read model
    async fn handle_event(&mut self, event: &EventEnvelope) -> EventResult<()>;
    
    /// Initialize the projection (create tables, etc.)
    async fn initialize(&mut self) -> EventResult<()>;
    
    /// Reset the projection (clear all data)
    async fn reset(&mut self) -> EventResult<()>;
    
    /// Get the current state of the projection
    fn state(&self) -> ProjectionState;
    
    /// Set the state of the projection
    async fn set_state(&mut self, state: ProjectionState) -> EventResult<()>;
    
    /// Get the last processed position
    fn last_position(&self) -> Option<i64>;
    
    /// Get the last updated timestamp
    fn last_updated(&self) -> Option<chrono::DateTime<chrono::Utc>>;
    
    /// Check if the projection should handle the given event
    fn should_handle(&self, event: &EventEnvelope) -> bool {
        self.event_types().contains(&event.event_type)
    }
}

/// Manager for event projections
pub struct ProjectionManager {
    projections: HashMap<String, Box<dyn Projection>>,
    event_store: Arc<dyn EventStore>,
    db_pool: Arc<Pool<ConnectionManager<PgConnection>>>,
    checkpoint_frequency: i64,
}

impl ProjectionManager {
    /// Create a new projection manager
    pub fn new(
        event_store: Arc<dyn EventStore>,
        db_pool: Arc<Pool<ConnectionManager<PgConnection>>>,
        checkpoint_frequency: i64,
    ) -> Self {
        Self {
            projections: HashMap::new(),
            event_store,
            db_pool,
            checkpoint_frequency,
        }
    }
    
    /// Register a projection
    pub async fn register_projection(&mut self, projection: Box<dyn Projection>) -> EventResult<()> {
        let name = projection.name().to_string();
        
        // Initialize the projection
        let mut proj = projection;
        proj.initialize().await?;
        
        // Create or update projection tracking record
        self.ensure_projection_record(&name).await?;
        
        self.projections.insert(name, proj);
        Ok(())
    }
    
    /// Remove a projection
    pub fn unregister_projection(&mut self, name: &str) {
        self.projections.remove(name);
    }
    
    /// Rebuild a projection from the beginning
    pub async fn rebuild_projection(&mut self, name: &str) -> EventResult<()> {
        if let Some(projection) = self.projections.get_mut(name) {
            // Reset the projection
            projection.reset().await?;
            
            // Reset the checkpoint
            self.update_projection_checkpoint(name, 0, None).await?;
            
            // Rebuild from events
            self.build_projection(name).await?;
        } else {
            return Err(EventError::ProjectionError {
                message: format!("Projection '{}' not found", name),
            });
        }
        
        Ok(())
    }
    
    /// Build or continue building a projection
    pub async fn build_projection(&mut self, name: &str) -> EventResult<()> {
        // Check if projection exists
        if !self.projections.contains_key(name) {
            return Err(EventError::ProjectionError {
                message: format!("Projection '{}' not found", name),
            });
        }
        
        // Get the checkpoint before borrowing the projection mutably
        let checkpoint = self.get_projection_checkpoint(name).await?;
        let mut current_position = checkpoint.last_processed_position;
        
        loop {
            // Get events from current position
            let events = self.event_store
                .get_events_from_position(current_position, 1000)
                .await?;
                
            if events.is_empty() {
                break;
            }
            
            // Process events through projection in a scoped block
            {
                let projection = self.projections.get_mut(name).unwrap();
                for event in &events {
                    if projection.should_handle(event) {
                        projection.handle_event(event).await?;
                    }
                }
            } // projection borrow ends here
            
            // Update position
            if let Some(last_event) = events.last() {
                current_position = last_event.recorded_at.timestamp();
                
                // Save checkpoint periodically
                if events.len() >= self.checkpoint_frequency as usize {
                    self.update_projection_checkpoint(
                        name,
                        current_position,
                        Some(last_event.event_id),
                    ).await?;
                }
            }
        }
        
        // Final checkpoint update
        self.update_projection_checkpoint(name, current_position, None).await?;
        
        Ok(())
    }
    
    /// Build all projections
    pub async fn build_all_projections(&mut self) -> EventResult<()> {
        let projection_names: Vec<String> = self.projections.keys().cloned().collect();
        
        for name in projection_names {
            self.build_projection(&name).await?;
        }
        
        Ok(())
    }
    
    /// Process a new event through all relevant projections
    pub async fn process_event(&mut self, event: &EventEnvelope) -> EventResult<()> {
        for projection in self.projections.values_mut() {
            if projection.should_handle(event) {
                projection.handle_event(event).await?;
            }
        }
        
        Ok(())
    }
    
    /// Get projection checkpoint from database
    async fn get_projection_checkpoint(&self, name: &str) -> EventResult<ProjectionCheckpoint> {
        let mut conn = self.db_pool.get().map_err(|e| EventError::DatabaseError {
            message: format!("Failed to get database connection: {}", e),
        })?;
        
        let record: Option<ProjectionRecord> = event_projections::table
            .filter(event_projections::projection_name.eq(name))
            .first(&mut conn)
            .optional()
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to get projection checkpoint: {}", e),
            })?;
        
        if let Some(record) = record {
            Ok(ProjectionCheckpoint {
                projection_name: record.projection_name,
                last_processed_event_id: record.last_processed_event_id,
                last_processed_position: record.last_processed_position.unwrap_or(0),
                status: record.status,
                updated_at: record.updated_at,
            })
        } else {
            // Create new checkpoint
            self.ensure_projection_record(name).await?;
            Ok(ProjectionCheckpoint {
                projection_name: name.to_string(),
                last_processed_event_id: None,
                last_processed_position: 0,
                status: "building".to_string(),
                updated_at: Utc::now(),
            })
        }
    }
    
    /// Update projection checkpoint
    async fn update_projection_checkpoint(
        &self,
        name: &str,
        position: i64,
        event_id: Option<Uuid>,
    ) -> EventResult<()> {
        let mut conn = self.db_pool.get().map_err(|e| EventError::DatabaseError {
            message: format!("Failed to get database connection: {}", e),
        })?;
        
        diesel::update(event_projections::table)
            .filter(event_projections::projection_name.eq(name))
            .set((
                event_projections::last_processed_position.eq(Some(position)),
                event_projections::last_processed_event_id.eq(event_id),
                event_projections::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to update projection checkpoint: {}", e),
            })?;
        
        Ok(())
    }
    
    /// Ensure projection record exists
    async fn ensure_projection_record(&self, name: &str) -> EventResult<()> {
        let mut conn = self.db_pool.get().map_err(|e| EventError::DatabaseError {
            message: format!("Failed to get database connection: {}", e),
        })?;
        
        let new_record = NewProjectionRecord {
            projection_name: name.to_string(),
            last_processed_event_id: None,
            last_processed_position: Some(0),
            status: "building".to_string(),
        };
        
        diesel::insert_into(event_projections::table)
            .values(&new_record)
            .on_conflict(event_projections::projection_name)
            .do_nothing()
            .execute(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to create projection record: {}", e),
            })?;
        
        Ok(())
    }
}

/// Projection checkpoint information
#[derive(Debug, Clone)]
struct ProjectionCheckpoint {
    projection_name: String,
    last_processed_event_id: Option<Uuid>,
    last_processed_position: i64,
    status: String,
    updated_at: DateTime<Utc>,
}

/// Database record for projection tracking
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = event_projections)]
struct ProjectionRecord {
    id: Uuid,
    projection_name: String,
    last_processed_event_id: Option<Uuid>,
    last_processed_position: Option<i64>,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = event_projections)]
struct NewProjectionRecord {
    projection_name: String,
    last_processed_event_id: Option<Uuid>,
    last_processed_position: Option<i64>,
    status: String,
}

/// Example workflow statistics projection
pub struct WorkflowStatsProjection {
    name: String,
    state: ProjectionState,
    db_pool: Arc<Pool<ConnectionManager<PgConnection>>>,
    last_position: Option<i64>,
    last_updated: Option<chrono::DateTime<chrono::Utc>>,
}

impl WorkflowStatsProjection {
    pub fn new(db_pool: Arc<Pool<ConnectionManager<PgConnection>>>) -> Self {
        Self {
            name: "workflow_stats".to_string(),
            state: ProjectionState::Building,
            db_pool,
            last_position: None,
            last_updated: None,
        }
    }
    
    async fn update_workflow_stats(&self, event: &EventEnvelope) -> EventResult<()> {
        // Parse workflow event and update statistics
        // This would typically update summary tables, counters, etc.
        
        tracing::debug!(
            "Updating workflow stats for event: {} (type: {})",
            event.event_id,
            event.event_type
        );
        
        // Example: increment counters based on event type
        // In a real implementation, this would update actual database tables
        
        Ok(())
    }
}

#[async_trait]
impl Projection for WorkflowStatsProjection {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn event_types(&self) -> Vec<String> {
        vec![
            "workflow_event".to_string(),
        ]
    }
    
    async fn handle_event(&mut self, event: &EventEnvelope) -> EventResult<()> {
        if self.should_handle(event) {
            self.update_workflow_stats(event).await?;
        }
        Ok(())
    }
    
    async fn initialize(&mut self) -> EventResult<()> {
        // Create necessary tables for workflow statistics
        tracing::info!("Initializing workflow stats projection");
        self.state = ProjectionState::Building;
        
        let mut conn = self.db_pool.get().map_err(|e| EventError::DatabaseError {
            message: format!("Failed to get database connection: {}", e),
        })?;
        
        // Create workflow statistics tables
        let create_workflow_stats_sql = r#"
            CREATE TABLE IF NOT EXISTS workflow_stats (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                workflow_name VARCHAR NOT NULL,
                total_executions BIGINT NOT NULL DEFAULT 0,
                successful_executions BIGINT NOT NULL DEFAULT 0,
                failed_executions BIGINT NOT NULL DEFAULT 0,
                avg_execution_time_ms FLOAT,
                total_execution_time_ms BIGINT NOT NULL DEFAULT 0,
                last_execution_at TIMESTAMPTZ,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE(workflow_name)
            );
            
            CREATE INDEX IF NOT EXISTS idx_workflow_stats_name ON workflow_stats(workflow_name);
            CREATE INDEX IF NOT EXISTS idx_workflow_stats_updated_at ON workflow_stats(updated_at);
        "#;
        
        let create_workflow_execution_history_sql = r#"
            CREATE TABLE IF NOT EXISTS workflow_execution_history (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                workflow_name VARCHAR NOT NULL,
                execution_id UUID NOT NULL,
                status VARCHAR NOT NULL,
                execution_time_ms BIGINT,
                error_message TEXT,
                metadata JSONB,
                started_at TIMESTAMPTZ NOT NULL,
                completed_at TIMESTAMPTZ,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            
            CREATE INDEX IF NOT EXISTS idx_workflow_execution_history_name ON workflow_execution_history(workflow_name);
            CREATE INDEX IF NOT EXISTS idx_workflow_execution_history_status ON workflow_execution_history(status);
            CREATE INDEX IF NOT EXISTS idx_workflow_execution_history_started_at ON workflow_execution_history(started_at);
        "#;
        
        // Execute table creation
        diesel::sql_query(create_workflow_stats_sql)
            .execute(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to create workflow_stats table: {}", e),
            })?;
            
        diesel::sql_query(create_workflow_execution_history_sql)
            .execute(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to create workflow_execution_history table: {}", e),
            })?;
        
        tracing::info!("Workflow stats projection tables created successfully");
        self.state = ProjectionState::Active;
        Ok(())
    }
    
    async fn reset(&mut self) -> EventResult<()> {
        tracing::info!("Resetting workflow stats projection");
        
        let mut conn = self.db_pool.get().map_err(|e| EventError::DatabaseError {
            message: format!("Failed to get database connection: {}", e),
        })?;
        
        // Clear all data in the projection tables
        let clear_workflow_stats_sql = "TRUNCATE TABLE workflow_stats RESTART IDENTITY CASCADE;";
        let clear_workflow_execution_history_sql = "TRUNCATE TABLE workflow_execution_history RESTART IDENTITY CASCADE;";
        
        diesel::sql_query(clear_workflow_stats_sql)
            .execute(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to clear workflow_stats table: {}", e),
            })?;
            
        diesel::sql_query(clear_workflow_execution_history_sql)
            .execute(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to clear workflow_execution_history table: {}", e),
            })?;
        
        tracing::info!("Workflow stats projection data cleared successfully");
        self.state = ProjectionState::Building;
        Ok(())
    }
    
    fn state(&self) -> ProjectionState {
        self.state.clone()
    }
    
    async fn set_state(&mut self, state: ProjectionState) -> EventResult<()> {
        self.state = state;
        Ok(())
    }
    
    fn last_position(&self) -> Option<i64> {
        self.last_position
    }
    
    fn last_updated(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.last_updated
    }
}

/// AI metrics projection for token usage and cost tracking
pub struct AIMetricsProjection {
    name: String,
    state: ProjectionState,
    db_pool: Arc<Pool<ConnectionManager<PgConnection>>>,
    last_position: Option<i64>,
    last_updated: Option<chrono::DateTime<chrono::Utc>>,
}

impl AIMetricsProjection {
    pub fn new(db_pool: Arc<Pool<ConnectionManager<PgConnection>>>) -> Self {
        Self {
            name: "ai_metrics".to_string(),
            state: ProjectionState::Building,
            db_pool,
            last_position: None,
            last_updated: None,
        }
    }
    
    async fn update_ai_metrics(&self, event: &EventEnvelope) -> EventResult<()> {
        tracing::debug!(
            "Updating AI metrics for event: {} (type: {})",
            event.event_id,
            event.event_type
        );
        
        // Parse AI interaction events and update token/cost metrics
        // This would update tables like:
        // - ai_usage_by_model
        // - daily_token_usage
        // - cost_by_provider
        // etc.
        
        Ok(())
    }
}

#[async_trait]
impl Projection for AIMetricsProjection {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn event_types(&self) -> Vec<String> {
        vec![
            "ai_interaction_event".to_string(),
        ]
    }
    
    async fn handle_event(&mut self, event: &EventEnvelope) -> EventResult<()> {
        if self.should_handle(event) {
            self.update_ai_metrics(event).await?;
        }
        Ok(())
    }
    
    async fn initialize(&mut self) -> EventResult<()> {
        tracing::info!("Initializing AI metrics projection");
        self.state = ProjectionState::Building;
        
        let mut conn = self.db_pool.get().map_err(|e| EventError::DatabaseError {
            message: format!("Failed to get database connection: {}", e),
        })?;
        
        // Create AI metrics tables
        let create_ai_usage_stats_sql = r#"
            CREATE TABLE IF NOT EXISTS ai_usage_stats (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                provider VARCHAR NOT NULL,
                model VARCHAR NOT NULL,
                total_requests BIGINT NOT NULL DEFAULT 0,
                successful_requests BIGINT NOT NULL DEFAULT 0,
                failed_requests BIGINT NOT NULL DEFAULT 0,
                total_input_tokens BIGINT NOT NULL DEFAULT 0,
                total_output_tokens BIGINT NOT NULL DEFAULT 0,
                total_cost_usd DECIMAL(10,6) NOT NULL DEFAULT 0.0,
                avg_request_time_ms FLOAT,
                total_request_time_ms BIGINT NOT NULL DEFAULT 0,
                last_request_at TIMESTAMPTZ,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE(provider, model)
            );
            
            CREATE INDEX IF NOT EXISTS idx_ai_usage_stats_provider_model ON ai_usage_stats(provider, model);
            CREATE INDEX IF NOT EXISTS idx_ai_usage_stats_updated_at ON ai_usage_stats(updated_at);
        "#;
        
        let create_daily_ai_usage_sql = r#"
            CREATE TABLE IF NOT EXISTS daily_ai_usage (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                usage_date DATE NOT NULL,
                provider VARCHAR NOT NULL,
                model VARCHAR NOT NULL,
                request_count BIGINT NOT NULL DEFAULT 0,
                input_tokens BIGINT NOT NULL DEFAULT 0,
                output_tokens BIGINT NOT NULL DEFAULT 0,
                cost_usd DECIMAL(10,6) NOT NULL DEFAULT 0.0,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE(usage_date, provider, model)
            );
            
            CREATE INDEX IF NOT EXISTS idx_daily_ai_usage_date ON daily_ai_usage(usage_date);
            CREATE INDEX IF NOT EXISTS idx_daily_ai_usage_provider_model ON daily_ai_usage(provider, model);
        "#;
        
        let create_ai_request_history_sql = r#"
            CREATE TABLE IF NOT EXISTS ai_request_history (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                request_id UUID NOT NULL,
                provider VARCHAR NOT NULL,
                model VARCHAR NOT NULL,
                input_tokens BIGINT,
                output_tokens BIGINT,
                cost_usd DECIMAL(10,6),
                request_time_ms BIGINT,
                status VARCHAR NOT NULL,
                error_message TEXT,
                metadata JSONB,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            
            CREATE INDEX IF NOT EXISTS idx_ai_request_history_provider_model ON ai_request_history(provider, model);
            CREATE INDEX IF NOT EXISTS idx_ai_request_history_created_at ON ai_request_history(created_at);
            CREATE INDEX IF NOT EXISTS idx_ai_request_history_status ON ai_request_history(status);
        "#;
        
        // Execute table creation
        diesel::sql_query(create_ai_usage_stats_sql)
            .execute(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to create ai_usage_stats table: {}", e),
            })?;
            
        diesel::sql_query(create_daily_ai_usage_sql)
            .execute(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to create daily_ai_usage table: {}", e),
            })?;
            
        diesel::sql_query(create_ai_request_history_sql)
            .execute(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to create ai_request_history table: {}", e),
            })?;
        
        tracing::info!("AI metrics projection tables created successfully");
        self.state = ProjectionState::Active;
        Ok(())
    }
    
    async fn reset(&mut self) -> EventResult<()> {
        tracing::info!("Resetting AI metrics projection");
        
        let mut conn = self.db_pool.get().map_err(|e| EventError::DatabaseError {
            message: format!("Failed to get database connection: {}", e),
        })?;
        
        // Clear all data in the AI metrics tables
        let clear_ai_usage_stats_sql = "TRUNCATE TABLE ai_usage_stats RESTART IDENTITY CASCADE;";
        let clear_daily_ai_usage_sql = "TRUNCATE TABLE daily_ai_usage RESTART IDENTITY CASCADE;";
        let clear_ai_request_history_sql = "TRUNCATE TABLE ai_request_history RESTART IDENTITY CASCADE;";
        
        diesel::sql_query(clear_ai_usage_stats_sql)
            .execute(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to clear ai_usage_stats table: {}", e),
            })?;
            
        diesel::sql_query(clear_daily_ai_usage_sql)
            .execute(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to clear daily_ai_usage table: {}", e),
            })?;
            
        diesel::sql_query(clear_ai_request_history_sql)
            .execute(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to clear ai_request_history table: {}", e),
            })?;
        
        tracing::info!("AI metrics projection data cleared successfully");
        self.state = ProjectionState::Building;
        Ok(())
    }
    
    fn state(&self) -> ProjectionState {
        self.state.clone()
    }
    
    async fn set_state(&mut self, state: ProjectionState) -> EventResult<()> {
        self.state = state;
        Ok(())
    }
    
    fn last_position(&self) -> Option<i64> {
        self.last_position
    }
    
    fn last_updated(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.last_updated
    }
}

/// Service health projection for monitoring service availability
pub struct ServiceHealthProjection {
    name: String,
    state: ProjectionState,
    db_pool: Arc<Pool<ConnectionManager<PgConnection>>>,
    last_position: Option<i64>,
    last_updated: Option<chrono::DateTime<chrono::Utc>>,
}

impl ServiceHealthProjection {
    pub fn new(db_pool: Arc<Pool<ConnectionManager<PgConnection>>>) -> Self {
        Self {
            name: "service_health".to_string(),
            state: ProjectionState::Building,
            db_pool,
            last_position: None,
            last_updated: None,
        }
    }
    
    async fn update_service_health(&self, event: &EventEnvelope) -> EventResult<()> {
        tracing::debug!(
            "Updating service health for event: {} (type: {})",
            event.event_id,
            event.event_type
        );
        
        // Parse service events and update health status
        // This would update tables like:
        // - service_availability
        // - service_performance_metrics
        // - service_error_rates
        // etc.
        
        Ok(())
    }
}

#[async_trait]
impl Projection for ServiceHealthProjection {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn event_types(&self) -> Vec<String> {
        vec![
            "service_call_event".to_string(),
        ]
    }
    
    async fn handle_event(&mut self, event: &EventEnvelope) -> EventResult<()> {
        if self.should_handle(event) {
            self.update_service_health(event).await?;
        }
        Ok(())
    }
    
    async fn initialize(&mut self) -> EventResult<()> {
        tracing::info!("Initializing service health projection");
        self.state = ProjectionState::Building;
        
        let mut conn = self.db_pool.get().map_err(|e| EventError::DatabaseError {
            message: format!("Failed to get database connection: {}", e),
        })?;
        
        // Create service health tracking tables
        let create_service_availability_sql = r#"
            CREATE TABLE IF NOT EXISTS service_availability (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                service_name VARCHAR NOT NULL,
                endpoint VARCHAR NOT NULL,
                total_calls BIGINT NOT NULL DEFAULT 0,
                successful_calls BIGINT NOT NULL DEFAULT 0,
                failed_calls BIGINT NOT NULL DEFAULT 0,
                avg_response_time_ms FLOAT,
                total_response_time_ms BIGINT NOT NULL DEFAULT 0,
                last_successful_call_at TIMESTAMPTZ,
                last_failed_call_at TIMESTAMPTZ,
                current_status VARCHAR NOT NULL DEFAULT 'unknown',
                uptime_percentage DECIMAL(5,2),
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE(service_name, endpoint)
            );
            
            CREATE INDEX IF NOT EXISTS idx_service_availability_service_name ON service_availability(service_name);
            CREATE INDEX IF NOT EXISTS idx_service_availability_status ON service_availability(current_status);
            CREATE INDEX IF NOT EXISTS idx_service_availability_updated_at ON service_availability(updated_at);
        "#;
        
        let create_service_performance_metrics_sql = r#"
            CREATE TABLE IF NOT EXISTS service_performance_metrics (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                service_name VARCHAR NOT NULL,
                endpoint VARCHAR NOT NULL,
                metric_date DATE NOT NULL,
                call_count BIGINT NOT NULL DEFAULT 0,
                success_count BIGINT NOT NULL DEFAULT 0,
                failure_count BIGINT NOT NULL DEFAULT 0,
                avg_response_time_ms FLOAT,
                min_response_time_ms BIGINT,
                max_response_time_ms BIGINT,
                p95_response_time_ms BIGINT,
                error_rate_percentage DECIMAL(5,2),
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE(service_name, endpoint, metric_date)
            );
            
            CREATE INDEX IF NOT EXISTS idx_service_performance_metrics_service_date ON service_performance_metrics(service_name, metric_date);
            CREATE INDEX IF NOT EXISTS idx_service_performance_metrics_date ON service_performance_metrics(metric_date);
        "#;
        
        let create_service_error_rates_sql = r#"
            CREATE TABLE IF NOT EXISTS service_error_rates (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                service_name VARCHAR NOT NULL,
                endpoint VARCHAR NOT NULL,
                error_type VARCHAR NOT NULL,
                error_count BIGINT NOT NULL DEFAULT 0,
                last_error_at TIMESTAMPTZ,
                last_error_message TEXT,
                error_rate_per_hour DECIMAL(8,2),
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE(service_name, endpoint, error_type)
            );
            
            CREATE INDEX IF NOT EXISTS idx_service_error_rates_service ON service_error_rates(service_name);
            CREATE INDEX IF NOT EXISTS idx_service_error_rates_error_type ON service_error_rates(error_type);
            CREATE INDEX IF NOT EXISTS idx_service_error_rates_last_error_at ON service_error_rates(last_error_at);
        "#;
        
        let create_service_call_history_sql = r#"
            CREATE TABLE IF NOT EXISTS service_call_history (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                service_name VARCHAR NOT NULL,
                endpoint VARCHAR NOT NULL,
                operation VARCHAR NOT NULL,
                status VARCHAR NOT NULL,
                response_time_ms BIGINT,
                error_message TEXT,
                error_type VARCHAR,
                metadata JSONB,
                called_at TIMESTAMPTZ NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            
            CREATE INDEX IF NOT EXISTS idx_service_call_history_service ON service_call_history(service_name);
            CREATE INDEX IF NOT EXISTS idx_service_call_history_called_at ON service_call_history(called_at);
            CREATE INDEX IF NOT EXISTS idx_service_call_history_status ON service_call_history(status);
        "#;
        
        // Execute table creation
        diesel::sql_query(create_service_availability_sql)
            .execute(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to create service_availability table: {}", e),
            })?;
            
        diesel::sql_query(create_service_performance_metrics_sql)
            .execute(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to create service_performance_metrics table: {}", e),
            })?;
            
        diesel::sql_query(create_service_error_rates_sql)
            .execute(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to create service_error_rates table: {}", e),
            })?;
            
        diesel::sql_query(create_service_call_history_sql)
            .execute(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to create service_call_history table: {}", e),
            })?;
        
        tracing::info!("Service health projection tables created successfully");
        self.state = ProjectionState::Active;
        Ok(())
    }
    
    async fn reset(&mut self) -> EventResult<()> {
        tracing::info!("Resetting service health projection");
        
        let mut conn = self.db_pool.get().map_err(|e| EventError::DatabaseError {
            message: format!("Failed to get database connection: {}", e),
        })?;
        
        // Clear all data in the service health tables
        let clear_service_availability_sql = "TRUNCATE TABLE service_availability RESTART IDENTITY CASCADE;";
        let clear_service_performance_metrics_sql = "TRUNCATE TABLE service_performance_metrics RESTART IDENTITY CASCADE;";
        let clear_service_error_rates_sql = "TRUNCATE TABLE service_error_rates RESTART IDENTITY CASCADE;";
        let clear_service_call_history_sql = "TRUNCATE TABLE service_call_history RESTART IDENTITY CASCADE;";
        
        diesel::sql_query(clear_service_availability_sql)
            .execute(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to clear service_availability table: {}", e),
            })?;
            
        diesel::sql_query(clear_service_performance_metrics_sql)
            .execute(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to clear service_performance_metrics table: {}", e),
            })?;
            
        diesel::sql_query(clear_service_error_rates_sql)
            .execute(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to clear service_error_rates table: {}", e),
            })?;
            
        diesel::sql_query(clear_service_call_history_sql)
            .execute(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to clear service_call_history table: {}", e),
            })?;
        
        tracing::info!("Service health projection data cleared successfully");
        self.state = ProjectionState::Building;
        Ok(())
    }
    
    fn state(&self) -> ProjectionState {
        self.state.clone()
    }
    
    async fn set_state(&mut self, state: ProjectionState) -> EventResult<()> {
        self.state = state;
        Ok(())
    }
    
    fn last_position(&self) -> Option<i64> {
        self.last_position
    }
    
    fn last_updated(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.last_updated
    }
}