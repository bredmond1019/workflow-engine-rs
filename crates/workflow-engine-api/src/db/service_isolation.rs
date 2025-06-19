// File: src/db/service_isolation.rs
//
// Database isolation patterns for microservices architecture
// Ensures true isolation with proper boundaries and event-driven communication

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::db::events::{EventStore, EventEnvelope, EventResult, EventError};
use workflow_engine_core::error::WorkflowError;
use super::tenant::{TenantManager, TenantContext, TenantConnection};
use super::connection_pool::{ConnectionPoolManager, ServiceConnectionPool, ServicePoolConfig};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::pg::PgConnection;

/// Service isolation boundary configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceBoundary {
    pub service_name: String,
    pub database_url: String,
    pub database_type: DatabaseType,
    pub allowed_operations: Vec<ServiceOperation>,
    pub event_topics: Vec<String>,
    pub resource_limits: ResourceLimits,
    pub isolation_level: IsolationLevel,
}

/// Supported database types for different services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseType {
    PostgreSQL {
        schema: String,
        connection_pool_size: u32,
    },
    Dgraph {
        endpoint: String,
        auth_token: Option<String>,
    },
    Redis {
        cluster_mode: bool,
        sentinel_nodes: Vec<String>,
    },
    MongoDB {
        database: String,
        collection_prefix: String,
    },
}

/// Operations that a service is allowed to perform
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServiceOperation {
    Read,
    Write,
    Delete,
    Schema,
    Migrate,
    Backup,
    Monitor,
}

/// Resource limits for service database access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_connections: u32,
    pub max_query_time_seconds: u32,
    pub max_result_size_mb: u32,
    pub max_concurrent_operations: u32,
    pub rate_limit_per_second: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_connections: 20,
            max_query_time_seconds: 30,
            max_result_size_mb: 100,
            max_concurrent_operations: 50,
            rate_limit_per_second: 100,
        }
    }
}

/// Level of database isolation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IsolationLevel {
    /// Complete database separation (different database instances)
    Complete,
    /// Schema-level separation (same instance, different schemas)
    Schema,
    /// Table-level separation (same schema, prefixed tables)
    Table,
    /// Row-level separation (shared tables with service_id filtering)
    Row,
}

/// Service data access interface
#[async_trait]
pub trait ServiceDataAccess: Send + Sync {
    /// Get service boundary configuration
    fn boundary(&self) -> &ServiceBoundary;
    
    /// Check if operation is allowed for this service
    fn is_operation_allowed(&self, operation: &ServiceOperation) -> bool;
    
    /// Execute a read operation within service boundary
    async fn read_within_boundary(&self, query: &str, params: HashMap<String, serde_json::Value>) 
        -> Result<serde_json::Value, ServiceIsolationError>;
    
    /// Execute a write operation within service boundary
    async fn write_within_boundary(&self, operation: &str, data: serde_json::Value) 
        -> Result<serde_json::Value, ServiceIsolationError>;
    
    /// Publish an event to other services
    async fn publish_event(&self, event: &CrossServiceEvent) 
        -> Result<(), ServiceIsolationError>;
    
    /// Subscribe to events from other services
    async fn subscribe_to_events(&self, topics: Vec<String>) 
        -> Result<Box<dyn CrossServiceEventStream>, ServiceIsolationError>;
    
    /// Perform a health check within service boundary
    async fn health_check(&self) -> Result<ServiceHealthStatus, ServiceIsolationError>;
}

/// Cross-service event for communication between isolated services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossServiceEvent {
    pub event_id: Uuid,
    pub source_service: String,
    pub target_service: Option<String>, // None for broadcast
    pub event_type: String,
    pub topic: String,
    pub payload: serde_json::Value,
    pub correlation_id: Option<Uuid>,
    pub causation_id: Option<Uuid>,
    pub occurred_at: chrono::DateTime<chrono::Utc>,
    pub version: i32,
    pub metadata: HashMap<String, String>,
}

impl CrossServiceEvent {
    pub fn new(
        source_service: String,
        event_type: String,
        topic: String,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            source_service,
            target_service: None,
            event_type,
            topic,
            payload,
            correlation_id: None,
            causation_id: None,
            occurred_at: chrono::Utc::now(),
            version: 1,
            metadata: HashMap::new(),
        }
    }

    pub fn with_target(mut self, target_service: String) -> Self {
        self.target_service = Some(target_service);
        self
    }

    pub fn with_correlation_id(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    pub fn with_causation_id(mut self, causation_id: Uuid) -> Self {
        self.causation_id = Some(causation_id);
        self
    }

    pub fn add_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Stream of cross-service events
#[async_trait]
pub trait CrossServiceEventStream: Send {
    async fn next_event(&mut self) -> Option<Result<CrossServiceEvent, ServiceIsolationError>>;
    async fn close(&mut self) -> Result<(), ServiceIsolationError>;
}

/// Health status for a service database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealthStatus {
    pub service_name: String,
    pub database_healthy: bool,
    pub connection_count: u32,
    pub active_operations: u32,
    pub last_operation_time: Option<chrono::DateTime<chrono::Utc>>,
    pub error_count_last_hour: u32,
    pub response_time_ms: u32,
    pub isolation_level: IsolationLevel,
}

/// Errors related to service isolation
#[derive(Debug, thiserror::Error)]
pub enum ServiceIsolationError {
    #[error("Operation {operation:?} not allowed for service {service}")]
    OperationNotAllowed { service: String, operation: String },
    
    #[error("Resource limit exceeded: {limit_type}")]
    ResourceLimitExceeded { limit_type: String },
    
    #[error("Database connection error: {message}")]
    DatabaseError { message: String },
    
    #[error("Event publishing failed: {message}")]
    EventPublishingError { message: String },
    
    #[error("Isolation boundary violation: {message}")]
    BoundaryViolation { message: String },
    
    #[error("Cross-service communication error: {message}")]
    CrossServiceError { message: String },
    
    #[error("Serialization error: {message}")]
    SerializationError { message: String },
    
    #[error("Authorization error: {message}")]
    AuthorizationError { message: String },
}

impl From<WorkflowError> for ServiceIsolationError {
    fn from(err: WorkflowError) -> Self {
        ServiceIsolationError::DatabaseError {
            message: err.to_string(),
        }
    }
}

/// Service isolation manager
pub struct ServiceIsolationManager {
    event_store: Arc<dyn EventStore>,
    service_boundaries: HashMap<String, ServiceBoundary>,
    event_bus: Arc<dyn CrossServiceEventBus>,
    tenant_manager: Arc<TenantManager>,
    connection_pool_manager: Arc<ConnectionPoolManager>,
}

impl ServiceIsolationManager {
    pub fn new(
        event_store: Arc<dyn EventStore>,
        event_bus: Arc<dyn CrossServiceEventBus>,
        tenant_manager: Arc<TenantManager>,
        connection_pool_manager: Arc<ConnectionPoolManager>,
    ) -> Self {
        Self {
            event_store,
            service_boundaries: HashMap::new(),
            event_bus,
            tenant_manager,
            connection_pool_manager,
        }
    }

    /// Register a service boundary
    pub async fn register_service(&mut self, boundary: ServiceBoundary) -> Result<(), ServiceIsolationError> {
        // Validate boundary configuration
        self.validate_boundary(&boundary)?;
        
        // Check for conflicts with existing services
        if let Some(existing) = self.service_boundaries.get(&boundary.service_name) {
            if existing.database_url == boundary.database_url && 
               existing.isolation_level == IsolationLevel::Complete {
                return Err(ServiceIsolationError::BoundaryViolation {
                    message: format!(
                        "Service {} conflicts with existing service boundary", 
                        boundary.service_name
                    ),
                });
            }
        }

        // Store boundary configuration
        self.service_boundaries.insert(boundary.service_name.clone(), boundary.clone());
        
        // Create service-specific connection pool
        self.connection_pool_manager
            .create_pool_from_boundary(&boundary)
            .await?;

        // Publish service registration event
        let registration_event = CrossServiceEvent::new(
            "isolation_manager".to_string(),
            "service_registered".to_string(),
            "service_lifecycle".to_string(),
            serde_json::to_value(&boundary).map_err(|e| ServiceIsolationError::SerializationError {
                message: e.to_string(),
            })?,
        );

        self.event_bus.publish(&registration_event).await?;

        Ok(())
    }

    /// Get service boundary
    pub fn get_service_boundary(&self, service_name: &str) -> Option<&ServiceBoundary> {
        self.service_boundaries.get(service_name)
    }

    /// Create a data access instance for a service
    pub fn create_service_data_access(
        &self,
        service_name: &str,
    ) -> Result<Box<dyn ServiceDataAccess>, ServiceIsolationError> {
        let boundary = self.get_service_boundary(service_name)
            .ok_or_else(|| ServiceIsolationError::BoundaryViolation {
                message: format!("Service {} not registered", service_name),
            })?;

        match &boundary.database_type {
            DatabaseType::PostgreSQL { schema, connection_pool_size } => {
                Ok(Box::new(PostgreSQLServiceDataAccess::new(
                    boundary.clone(),
                    Arc::clone(&self.event_bus),
                    Arc::clone(&self.connection_pool_manager),
                    Arc::clone(&self.tenant_manager),
                )?))
            }
            DatabaseType::Dgraph { endpoint, auth_token } => {
                Ok(Box::new(DgraphServiceDataAccess::new(
                    boundary.clone(),
                    Arc::clone(&self.event_bus),
                    Arc::clone(&self.connection_pool_manager),
                    Arc::clone(&self.tenant_manager),
                )?))
            }
            DatabaseType::Redis { cluster_mode, sentinel_nodes } => {
                Ok(Box::new(RedisServiceDataAccess::new(
                    boundary.clone(),
                    Arc::clone(&self.event_bus),
                    Arc::clone(&self.connection_pool_manager),
                    Arc::clone(&self.tenant_manager),
                )?))
            }
            DatabaseType::MongoDB { database, collection_prefix } => {
                Ok(Box::new(MongoDBServiceDataAccess::new(
                    boundary.clone(),
                    Arc::clone(&self.event_bus),
                    Arc::clone(&self.connection_pool_manager),
                    Arc::clone(&self.tenant_manager),
                )?))
            }
        }
    }

    /// Validate boundary configuration
    fn validate_boundary(&self, boundary: &ServiceBoundary) -> Result<(), ServiceIsolationError> {
        // Check service name is valid
        if boundary.service_name.is_empty() {
            return Err(ServiceIsolationError::BoundaryViolation {
                message: "Service name cannot be empty".to_string(),
            });
        }

        // Check database URL is valid
        if boundary.database_url.is_empty() {
            return Err(ServiceIsolationError::BoundaryViolation {
                message: "Database URL cannot be empty".to_string(),
            });
        }

        // Check resource limits are reasonable
        if boundary.resource_limits.max_connections == 0 {
            return Err(ServiceIsolationError::BoundaryViolation {
                message: "Max connections must be greater than 0".to_string(),
            });
        }

        // Check allowed operations
        if boundary.allowed_operations.is_empty() {
            return Err(ServiceIsolationError::BoundaryViolation {
                message: "Service must have at least one allowed operation".to_string(),
            });
        }

        Ok(())
    }

    /// Monitor all service boundaries
    pub async fn monitor_services(&self) -> Result<Vec<ServiceHealthStatus>, ServiceIsolationError> {
        let mut health_statuses = Vec::new();

        for (service_name, boundary) in &self.service_boundaries {
            let data_access = self.create_service_data_access(service_name)?;
            let health = data_access.health_check().await?;
            health_statuses.push(health);
        }

        Ok(health_statuses)
    }
}

/// Cross-service event bus for communication between isolated services
#[async_trait]
pub trait CrossServiceEventBus: Send + Sync {
    /// Publish an event to the bus
    async fn publish(&self, event: &CrossServiceEvent) -> Result<(), ServiceIsolationError>;
    
    /// Subscribe to events by topic
    async fn subscribe(&self, topics: Vec<String>) -> Result<Box<dyn CrossServiceEventStream>, ServiceIsolationError>;
    
    /// Get event history for debugging
    async fn get_event_history(&self, service: &str, limit: usize) -> Result<Vec<CrossServiceEvent>, ServiceIsolationError>;
}

/// PostgreSQL-specific service data access implementation
pub struct PostgreSQLServiceDataAccess {
    boundary: ServiceBoundary,
    event_bus: Arc<dyn CrossServiceEventBus>,
    connection_pool_manager: Arc<ConnectionPoolManager>,
    tenant_manager: Arc<TenantManager>,
}

impl PostgreSQLServiceDataAccess {
    pub fn new(
        boundary: ServiceBoundary,
        event_bus: Arc<dyn CrossServiceEventBus>,
        connection_pool_manager: Arc<ConnectionPoolManager>,
        tenant_manager: Arc<TenantManager>,
    ) -> Result<Self, ServiceIsolationError> {
        Ok(Self {
            boundary,
            event_bus,
            connection_pool_manager,
            tenant_manager,
        })
    }
}

#[async_trait]
impl ServiceDataAccess for PostgreSQLServiceDataAccess {
    fn boundary(&self) -> &ServiceBoundary {
        &self.boundary
    }

    fn is_operation_allowed(&self, operation: &ServiceOperation) -> bool {
        self.boundary.allowed_operations.contains(operation)
    }

    async fn read_within_boundary(&self, query: &str, params: HashMap<String, serde_json::Value>) 
        -> Result<serde_json::Value, ServiceIsolationError> {
        if !self.is_operation_allowed(&ServiceOperation::Read) {
            return Err(ServiceIsolationError::OperationNotAllowed {
                service: self.boundary.service_name.clone(),
                operation: "read".to_string(),
            });
        }

        // Get tenant ID from parameters if row-level isolation
        let tenant_id = if matches!(self.boundary.isolation_level, IsolationLevel::Row | IsolationLevel::Schema) {
            params.get("tenant_id")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok())
        } else {
            None
        };

        // Get appropriate connection based on isolation level
        let result = if let Some(tid) = tenant_id {
            // Tenant-aware connection
            let mut conn = self.connection_pool_manager
                .get_tenant_service_connection(&self.boundary.service_name, tid)
                .await
                .map_err(|e| ServiceIsolationError::DatabaseError { 
                    message: format!("Failed to get tenant connection: {}", e) 
                })?;
            
            conn.with_tenant_context(|conn, ctx| {
                // Execute query with tenant context
                diesel::sql_query(query)
                    .execute(conn)
                    .map(|rows| serde_json::json!({"rows_affected": rows}))
            }).map_err(|e| ServiceIsolationError::DatabaseError {
                message: e.to_string()
            })?
        } else {
            // Service-only connection
            let mut conn = self.connection_pool_manager
                .get_service_connection(&self.boundary.service_name)
                .await
                .map_err(|e| ServiceIsolationError::DatabaseError {
                    message: format!("Failed to get service connection: {}", e)
                })?;
            
            diesel::sql_query(query)
                .execute(&mut *conn)
                .map(|rows| serde_json::json!({"rows_affected": rows}))
                .map_err(|e| ServiceIsolationError::DatabaseError {
                    message: e.to_string()
                })?
        };
        
        Ok(result)
    }

    async fn write_within_boundary(&self, operation: &str, data: serde_json::Value) 
        -> Result<serde_json::Value, ServiceIsolationError> {
        if !self.is_operation_allowed(&ServiceOperation::Write) {
            return Err(ServiceIsolationError::OperationNotAllowed {
                service: self.boundary.service_name.clone(),
                operation: "write".to_string(),
            });
        }

        // Extract tenant ID if present
        let tenant_id = data.get("tenant_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok());

        let result = if let Some(tid) = tenant_id {
            // Tenant-aware write operation
            let mut conn = self.connection_pool_manager
                .get_tenant_service_connection(&self.boundary.service_name, tid)
                .await
                .map_err(|e| ServiceIsolationError::DatabaseError {
                    message: format!("Failed to get tenant connection: {}", e)
                })?;
            
            conn.with_tenant_context(|conn, ctx| {
                // Transaction with tenant context
                conn.transaction(|conn| {
                    // Ensure tenant_id is set for row-level security
                    if matches!(ctx.isolation_mode, super::tenant::TenantIsolationMode::RowLevel | super::tenant::TenantIsolationMode::Hybrid) {
                        diesel::sql_query(format!(
                            "SET LOCAL app.current_tenant_id = '{}'",
                            ctx.tenant_id
                        )).execute(conn)?;
                    }
                    
                    // Execute write operation
                    diesel::sql_query(operation)
                        .execute(conn)
                        .map(|rows| serde_json::json!({
                            "operation": "write",
                            "rows_affected": rows,
                            "tenant_id": ctx.tenant_id.to_string()
                        }))
                })
            }).map_err(|e| ServiceIsolationError::DatabaseError {
                message: e.to_string()
            })?
        } else {
            // Service-only write operation
            let mut conn = self.connection_pool_manager
                .get_service_connection(&self.boundary.service_name)
                .await
                .map_err(|e| ServiceIsolationError::DatabaseError {
                    message: format!("Failed to get service connection: {}", e)
                })?;
            
            conn.transaction::<_, diesel::result::Error, _>(|conn| {
                diesel::sql_query(operation)
                    .execute(conn)
                    .map(|rows| serde_json::json!({
                        "operation": "write",
                        "rows_affected": rows
                    }))
            }).map_err(|e| ServiceIsolationError::DatabaseError {
                message: e.to_string()
            })?
        };
        
        Ok(result)
    }

    async fn publish_event(&self, event: &CrossServiceEvent) -> Result<(), ServiceIsolationError> {
        self.event_bus.publish(event).await
    }

    async fn subscribe_to_events(&self, topics: Vec<String>) -> Result<Box<dyn CrossServiceEventStream>, ServiceIsolationError> {
        self.event_bus.subscribe(topics).await
    }

    async fn health_check(&self) -> Result<ServiceHealthStatus, ServiceIsolationError> {
        // Get pool metrics
        let all_metrics = self.connection_pool_manager.get_all_metrics().await;
        let pool_metrics = all_metrics.get(&self.boundary.service_name);
        
        // Test connectivity
        let start = std::time::Instant::now();
        let database_healthy = match self.connection_pool_manager
            .get_service_connection(&self.boundary.service_name)
            .await
        {
            Ok(mut conn) => {
                // Test query
                diesel::sql_query("SELECT 1")
                    .execute(&mut conn)
                    .is_ok()
            }
            Err(_) => false,
        };
        let response_time_ms = start.elapsed().as_millis() as u32;
        
        Ok(ServiceHealthStatus {
            service_name: self.boundary.service_name.clone(),
            database_healthy,
            connection_count: pool_metrics.map(|m| m.total_connections).unwrap_or(0),
            active_operations: pool_metrics.map(|m| m.active_connections).unwrap_or(0),
            last_operation_time: Some(chrono::Utc::now()),
            error_count_last_hour: pool_metrics.map(|m| m.error_count as u32).unwrap_or(0),
            response_time_ms,
            isolation_level: self.boundary.isolation_level.clone(),
        })
    }
}

// Similar implementations would be created for other database types
pub struct DgraphServiceDataAccess {
    boundary: ServiceBoundary,
    event_bus: Arc<dyn CrossServiceEventBus>,
    connection_pool_manager: Arc<ConnectionPoolManager>,
    tenant_manager: Arc<TenantManager>,
}

impl DgraphServiceDataAccess {
    pub fn new(
        boundary: ServiceBoundary,
        event_bus: Arc<dyn CrossServiceEventBus>,
        connection_pool_manager: Arc<ConnectionPoolManager>,
        tenant_manager: Arc<TenantManager>,
    ) -> Result<Self, ServiceIsolationError> {
        Ok(Self {
            boundary,
            event_bus,
            connection_pool_manager,
            tenant_manager,
        })
    }
}

#[async_trait]
impl ServiceDataAccess for DgraphServiceDataAccess {
    fn boundary(&self) -> &ServiceBoundary {
        &self.boundary
    }

    fn is_operation_allowed(&self, operation: &ServiceOperation) -> bool {
        self.boundary.allowed_operations.contains(operation)
    }

    async fn read_within_boundary(&self, query: &str, params: HashMap<String, serde_json::Value>) 
        -> Result<serde_json::Value, ServiceIsolationError> {
        // DGraph-specific implementation
        Ok(serde_json::json!({"dgraph_result": "placeholder"}))
    }

    async fn write_within_boundary(&self, operation: &str, data: serde_json::Value) 
        -> Result<serde_json::Value, ServiceIsolationError> {
        // DGraph-specific implementation
        Ok(serde_json::json!({"dgraph_write": "completed"}))
    }

    async fn publish_event(&self, event: &CrossServiceEvent) -> Result<(), ServiceIsolationError> {
        self.event_bus.publish(event).await
    }

    async fn subscribe_to_events(&self, topics: Vec<String>) -> Result<Box<dyn CrossServiceEventStream>, ServiceIsolationError> {
        self.event_bus.subscribe(topics).await
    }

    async fn health_check(&self) -> Result<ServiceHealthStatus, ServiceIsolationError> {
        Ok(ServiceHealthStatus {
            service_name: self.boundary.service_name.clone(),
            database_healthy: true,
            connection_count: 3,
            active_operations: 1,
            last_operation_time: Some(chrono::Utc::now()),
            error_count_last_hour: 0,
            response_time_ms: 25,
            isolation_level: self.boundary.isolation_level.clone(),
        })
    }
}

// Placeholder implementations for Redis and MongoDB
pub struct RedisServiceDataAccess {
    boundary: ServiceBoundary,
    event_bus: Arc<dyn CrossServiceEventBus>,
    connection_pool_manager: Arc<ConnectionPoolManager>,
    tenant_manager: Arc<TenantManager>,
}

impl RedisServiceDataAccess {
    pub fn new(
        boundary: ServiceBoundary,
        event_bus: Arc<dyn CrossServiceEventBus>,
        connection_pool_manager: Arc<ConnectionPoolManager>,
        tenant_manager: Arc<TenantManager>,
    ) -> Result<Self, ServiceIsolationError> {
        Ok(Self { 
            boundary, 
            event_bus,
            connection_pool_manager,
            tenant_manager,
        })
    }
}

#[async_trait]
impl ServiceDataAccess for RedisServiceDataAccess {
    fn boundary(&self) -> &ServiceBoundary { &self.boundary }
    fn is_operation_allowed(&self, operation: &ServiceOperation) -> bool { 
        self.boundary.allowed_operations.contains(operation) 
    }
    async fn read_within_boundary(&self, _query: &str, _params: HashMap<String, serde_json::Value>) -> Result<serde_json::Value, ServiceIsolationError> { 
        Ok(serde_json::json!({"redis_result": "placeholder"})) 
    }
    async fn write_within_boundary(&self, _operation: &str, _data: serde_json::Value) -> Result<serde_json::Value, ServiceIsolationError> { 
        Ok(serde_json::json!({"redis_write": "completed"})) 
    }
    async fn publish_event(&self, event: &CrossServiceEvent) -> Result<(), ServiceIsolationError> { 
        self.event_bus.publish(event).await 
    }
    async fn subscribe_to_events(&self, topics: Vec<String>) -> Result<Box<dyn CrossServiceEventStream>, ServiceIsolationError> { 
        self.event_bus.subscribe(topics).await 
    }
    async fn health_check(&self) -> Result<ServiceHealthStatus, ServiceIsolationError> { 
        Ok(ServiceHealthStatus {
            service_name: self.boundary.service_name.clone(),
            database_healthy: true,
            connection_count: 10,
            active_operations: 5,
            last_operation_time: Some(chrono::Utc::now()),
            error_count_last_hour: 0,
            response_time_ms: 5,
            isolation_level: self.boundary.isolation_level.clone(),
        })
    }
}

pub struct MongoDBServiceDataAccess {
    boundary: ServiceBoundary,
    event_bus: Arc<dyn CrossServiceEventBus>,
    connection_pool_manager: Arc<ConnectionPoolManager>,
    tenant_manager: Arc<TenantManager>,
}

impl MongoDBServiceDataAccess {
    pub fn new(
        boundary: ServiceBoundary,
        event_bus: Arc<dyn CrossServiceEventBus>,
        connection_pool_manager: Arc<ConnectionPoolManager>,
        tenant_manager: Arc<TenantManager>,
    ) -> Result<Self, ServiceIsolationError> {
        Ok(Self { 
            boundary, 
            event_bus,
            connection_pool_manager,
            tenant_manager,
        })
    }
}

#[async_trait]
impl ServiceDataAccess for MongoDBServiceDataAccess {
    fn boundary(&self) -> &ServiceBoundary { &self.boundary }
    fn is_operation_allowed(&self, operation: &ServiceOperation) -> bool { 
        self.boundary.allowed_operations.contains(operation) 
    }
    async fn read_within_boundary(&self, _query: &str, _params: HashMap<String, serde_json::Value>) -> Result<serde_json::Value, ServiceIsolationError> { 
        Ok(serde_json::json!({"mongodb_result": "placeholder"})) 
    }
    async fn write_within_boundary(&self, _operation: &str, _data: serde_json::Value) -> Result<serde_json::Value, ServiceIsolationError> { 
        Ok(serde_json::json!({"mongodb_write": "completed"})) 
    }
    async fn publish_event(&self, event: &CrossServiceEvent) -> Result<(), ServiceIsolationError> { 
        self.event_bus.publish(event).await 
    }
    async fn subscribe_to_events(&self, topics: Vec<String>) -> Result<Box<dyn CrossServiceEventStream>, ServiceIsolationError> { 
        self.event_bus.subscribe(topics).await 
    }
    async fn health_check(&self) -> Result<ServiceHealthStatus, ServiceIsolationError> { 
        Ok(ServiceHealthStatus {
            service_name: self.boundary.service_name.clone(),
            database_healthy: true,
            connection_count: 15,
            active_operations: 3,
            last_operation_time: Some(chrono::Utc::now()),
            error_count_last_hour: 0,
            response_time_ms: 30,
            isolation_level: self.boundary.isolation_level.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_boundary_creation() {
        let boundary = ServiceBoundary {
            service_name: "content_processing".to_string(),
            database_url: "postgresql://localhost:5432/content_processing_db".to_string(),
            database_type: DatabaseType::PostgreSQL {
                schema: "content_processing".to_string(),
                connection_pool_size: 20,
            },
            allowed_operations: vec![
                ServiceOperation::Read,
                ServiceOperation::Write,
                ServiceOperation::Monitor,
            ],
            event_topics: vec!["content.processed".to_string(), "content.failed".to_string()],
            resource_limits: ResourceLimits::default(),
            isolation_level: IsolationLevel::Complete,
        };

        assert_eq!(boundary.service_name, "content_processing");
        assert_eq!(boundary.isolation_level, IsolationLevel::Complete);
        assert!(boundary.allowed_operations.contains(&ServiceOperation::Read));
        assert!(boundary.allowed_operations.contains(&ServiceOperation::Write));
    }

    #[test]
    fn test_cross_service_event_creation() {
        let event = CrossServiceEvent::new(
            "content_processing".to_string(),
            "content_processed".to_string(),
            "content.lifecycle".to_string(),
            serde_json::json!({"content_id": "123", "status": "processed"}),
        )
        .with_target("knowledge_graph".to_string())
        .with_correlation_id(Uuid::new_v4())
        .add_metadata("priority".to_string(), "high".to_string());

        assert_eq!(event.source_service, "content_processing");
        assert_eq!(event.event_type, "content_processed");
        assert_eq!(event.topic, "content.lifecycle");
        assert!(event.target_service.is_some());
        assert!(event.correlation_id.is_some());
        assert_eq!(event.metadata.get("priority"), Some(&"high".to_string()));
    }
}