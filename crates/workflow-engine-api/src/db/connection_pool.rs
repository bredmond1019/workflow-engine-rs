// File: src/db/connection_pool.rs
//
// Service-specific database connection pooling

use async_trait::async_trait;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection, Builder};
use diesel::pg::PgConnection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;

use workflow_engine_core::error::WorkflowError;
use super::tenant::{TenantContext, TenantConnection, TenantManager};
use super::service_isolation::{ServiceBoundary, IsolationLevel, ResourceLimits};

/// Service-specific connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePoolConfig {
    pub service_name: String,
    pub database_url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout: Duration,
    pub idle_timeout: Option<Duration>,
    pub max_lifetime: Option<Duration>,
    pub test_on_checkout: bool,
}

impl ServicePoolConfig {
    pub fn new(service_name: String, database_url: String) -> Self {
        Self {
            service_name,
            database_url,
            max_connections: 20,
            min_connections: 5,
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Some(Duration::from_secs(600)),
            max_lifetime: Some(Duration::from_secs(1800)),
            test_on_checkout: true,
        }
    }

    pub fn with_pool_size(mut self, min: u32, max: u32) -> Self {
        self.min_connections = min;
        self.max_connections = max;
        self
    }

    pub fn with_timeouts(
        mut self,
        connection_timeout: Duration,
        idle_timeout: Option<Duration>,
        max_lifetime: Option<Duration>,
    ) -> Self {
        self.connection_timeout = connection_timeout;
        self.idle_timeout = idle_timeout;
        self.max_lifetime = max_lifetime;
        self
    }
}

/// Service-specific connection pool
pub struct ServiceConnectionPool {
    pool: Arc<Pool<ConnectionManager<PgConnection>>>,
    config: ServicePoolConfig,
    metrics: Arc<RwLock<PoolMetrics>>,
}

/// Connection pool metrics
#[derive(Debug, Default, Clone)]
pub struct PoolMetrics {
    pub total_connections: u32,
    pub idle_connections: u32,
    pub active_connections: u32,
    pub wait_count: u64,
    pub wait_time_ms: u64,
    pub timeout_count: u64,
    pub error_count: u64,
}

impl ServiceConnectionPool {
    /// Create a new service-specific connection pool
    pub fn new(config: ServicePoolConfig) -> Result<Self, WorkflowError> {
        let manager = ConnectionManager::<PgConnection>::new(&config.database_url);
        
        let mut builder = Builder::new()
            .max_size(config.max_connections)
            .min_idle(Some(config.min_connections))
            .connection_timeout(config.connection_timeout)
            .test_on_check_out(config.test_on_checkout);
        
        if let Some(idle_timeout) = config.idle_timeout {
            builder = builder.idle_timeout(Some(idle_timeout));
        }
        
        if let Some(max_lifetime) = config.max_lifetime {
            builder = builder.max_lifetime(Some(max_lifetime));
        }
        
        let pool = builder
            .build(manager)
            .map_err(|e| WorkflowError::DatabaseError { message: format!("Failed to create pool: {}", e) })?;
        
        Ok(Self {
            pool: Arc::new(pool),
            config,
            metrics: Arc::new(RwLock::new(PoolMetrics::default())),
        })
    }

    /// Get a connection from the pool
    pub async fn get_connection(&self) -> Result<PooledConnection<ConnectionManager<PgConnection>>, WorkflowError> {
        let start = std::time::Instant::now();
        
        match self.pool.get() {
            Ok(conn) => {
                let elapsed = start.elapsed().as_millis() as u64;
                
                let mut metrics = self.metrics.write().await;
                metrics.wait_count += 1;
                metrics.wait_time_ms += elapsed;
                
                Ok(conn)
            }
            Err(e) => {
                let mut metrics = self.metrics.write().await;
                
                if e.to_string().contains("timeout") {
                    metrics.timeout_count += 1;
                } else {
                    metrics.error_count += 1;
                }
                
                Err(WorkflowError::DatabaseError { message: format!(
                    "Failed to get connection for service '{}': {}",
                    self.config.service_name, e
                ) })
            }
        }
    }

    /// Get current pool metrics
    pub async fn get_metrics(&self) -> PoolMetrics {
        let state = self.pool.state();
        let mut metrics = self.metrics.write().await;
        
        metrics.total_connections = state.connections;
        metrics.idle_connections = state.idle_connections;
        metrics.active_connections = state.connections - state.idle_connections;
        
        metrics.clone()
    }

    /// Get pool configuration
    pub fn config(&self) -> &ServicePoolConfig {
        &self.config
    }

    /// Test pool connectivity
    pub async fn test_connectivity(&self) -> Result<(), WorkflowError> {
        let mut conn = self.get_connection().await?;
        
        // Run a simple query to test the connection
        diesel::sql_query("SELECT 1")
            .execute(&mut conn)
            .map_err(|e| WorkflowError::DatabaseError { message: format!("Connectivity test failed: {}", e) })?;
        
        Ok(())
    }
}

/// Multi-service connection pool manager
pub struct ConnectionPoolManager {
    service_pools: Arc<RwLock<HashMap<String, Arc<ServiceConnectionPool>>>>,
    tenant_manager: Arc<TenantManager>,
    default_config: ServicePoolConfig,
}

impl ConnectionPoolManager {
    pub fn new(
        tenant_manager: Arc<TenantManager>,
        default_database_url: String,
    ) -> Self {
        let default_config = ServicePoolConfig::new(
            "default".to_string(),
            default_database_url,
        );
        
        Self {
            service_pools: Arc::new(RwLock::new(HashMap::new())),
            tenant_manager,
            default_config,
        }
    }

    /// Register a service-specific connection pool
    pub async fn register_service_pool(
        &self,
        config: ServicePoolConfig,
    ) -> Result<(), WorkflowError> {
        let pool = ServiceConnectionPool::new(config.clone())?;
        
        // Test connectivity before registering
        pool.test_connectivity().await?;
        
        self.service_pools
            .write()
            .await
            .insert(config.service_name.clone(), Arc::new(pool));
        
        Ok(())
    }

    /// Get a connection for a specific service
    pub async fn get_service_connection(
        &self,
        service_name: &str,
    ) -> Result<PooledConnection<ConnectionManager<PgConnection>>, WorkflowError> {
        let pools = self.service_pools.read().await;
        
        if let Some(pool) = pools.get(service_name) {
            pool.get_connection().await
        } else {
            Err(WorkflowError::ValidationError { message: format!(
                "No connection pool registered for service '{}'",
                service_name
            ) })
        }
    }

    /// Get a tenant-aware connection for a service
    pub async fn get_tenant_service_connection(
        &self,
        service_name: &str,
        tenant_id: Uuid,
    ) -> Result<TenantConnection, WorkflowError> {
        let conn = self.get_service_connection(service_name).await?;
        let tenant = self.tenant_manager.get_tenant(tenant_id).await?;
        
        let isolation_mode = match tenant.isolation_mode.as_str() {
            "schema" => super::tenant::TenantIsolationMode::Schema,
            "row_level" => super::tenant::TenantIsolationMode::RowLevel,
            "hybrid" => super::tenant::TenantIsolationMode::Hybrid,
            _ => return Err(WorkflowError::ValidationError { message:
                format!("Invalid isolation mode: {}", tenant.isolation_mode)
            }),
        };
        
        let context = TenantContext {
            tenant_id: tenant.id,
            tenant_name: tenant.name,
            database_schema: tenant.database_schema,
            isolation_mode,
        };
        
        Ok(TenantConnection::new(conn, context))
    }

    /// Create a pool from service boundary configuration
    pub async fn create_pool_from_boundary(
        &self,
        boundary: &ServiceBoundary,
    ) -> Result<(), WorkflowError> {
        let config = ServicePoolConfig::new(
            boundary.service_name.clone(),
            boundary.database_url.clone(),
        )
        .with_pool_size(
            boundary.resource_limits.max_connections / 4,
            boundary.resource_limits.max_connections,
        )
        .with_timeouts(
            Duration::from_secs(30),
            Some(Duration::from_secs(600)),
            Some(Duration::from_secs(1800)),
        );
        
        self.register_service_pool(config).await
    }

    /// Get metrics for all service pools
    pub async fn get_all_metrics(&self) -> HashMap<String, PoolMetrics> {
        let mut all_metrics = HashMap::new();
        let pools = self.service_pools.read().await;
        
        for (service_name, pool) in pools.iter() {
            all_metrics.insert(
                service_name.clone(),
                pool.get_metrics().await,
            );
        }
        
        all_metrics
    }

    /// Remove a service pool
    pub async fn remove_service_pool(&self, service_name: &str) -> Result<(), WorkflowError> {
        let removed = self.service_pools.write().await.remove(service_name);
        
        if removed.is_some() {
            Ok(())
        } else {
            Err(WorkflowError::ValidationError { message: format!(
                "No pool found for service '{}'",
                service_name
            ) })
        }
    }

    /// Test connectivity for all pools
    pub async fn test_all_pools(&self) -> HashMap<String, Result<(), String>> {
        let mut results = HashMap::new();
        let pools = self.service_pools.read().await;
        
        for (service_name, pool) in pools.iter() {
            let result = match pool.test_connectivity().await {
                Ok(()) => Ok(()),
                Err(e) => Err(e.to_string()),
            };
            results.insert(service_name.clone(), result);
        }
        
        results
    }
}

/// Extension trait for connection pools with isolation
#[async_trait]
pub trait IsolatedConnectionPool {
    /// Get a connection with isolation applied
    async fn get_isolated_connection(
        &self,
        isolation_level: &IsolationLevel,
        tenant_id: Option<Uuid>,
    ) -> Result<PooledConnection<ConnectionManager<PgConnection>>, WorkflowError>;
}

#[async_trait]
impl IsolatedConnectionPool for ServiceConnectionPool {
    async fn get_isolated_connection(
        &self,
        isolation_level: &IsolationLevel,
        tenant_id: Option<Uuid>,
    ) -> Result<PooledConnection<ConnectionManager<PgConnection>>, WorkflowError> {
        let mut conn = self.get_connection().await?;
        
        // Apply isolation based on level
        match isolation_level {
            IsolationLevel::Complete => {
                // Complete isolation - no additional setup needed
            }
            IsolationLevel::Schema => {
                if let Some(tenant_id) = tenant_id {
                    let schema = format!("tenant_{}", tenant_id.to_string().replace("-", "_"));
                    diesel::sql_query(format!("SET search_path TO {}, public", schema))
                        .execute(&mut *conn)
                        .map_err(|e| WorkflowError::DatabaseError { message: e.to_string() })?;
                }
            }
            IsolationLevel::Table => {
                // Table-level isolation is handled by prefixed table names
            }
            IsolationLevel::Row => {
                if let Some(tenant_id) = tenant_id {
                    diesel::sql_query(format!(
                        "SET LOCAL app.current_tenant_id = '{}'",
                        tenant_id
                    ))
                    .execute(&mut *conn)
                    .map_err(|e| WorkflowError::DatabaseError { message: e.to_string() })?;
                }
            }
        }
        
        Ok(conn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_pool_config() {
        let config = ServicePoolConfig::new(
            "test_service".to_string(),
            "postgresql://localhost/test".to_string(),
        )
        .with_pool_size(10, 50)
        .with_timeouts(
            Duration::from_secs(20),
            Some(Duration::from_secs(300)),
            None,
        );

        assert_eq!(config.service_name, "test_service");
        assert_eq!(config.max_connections, 50);
        assert_eq!(config.min_connections, 10);
        assert_eq!(config.connection_timeout, Duration::from_secs(20));
        assert_eq!(config.idle_timeout, Some(Duration::from_secs(300)));
        assert_eq!(config.max_lifetime, None);
    }

    #[tokio::test]
    async fn test_pool_metrics() {
        let metrics = PoolMetrics {
            total_connections: 10,
            idle_connections: 7,
            active_connections: 3,
            wait_count: 100,
            wait_time_ms: 5000,
            timeout_count: 2,
            error_count: 1,
        };

        assert_eq!(metrics.total_connections, 10);
        assert_eq!(metrics.active_connections, 3);
        assert_eq!(metrics.idle_connections, 7);
        assert_eq!(metrics.wait_count, 100);
        assert_eq!(metrics.timeout_count, 2);
    }
}