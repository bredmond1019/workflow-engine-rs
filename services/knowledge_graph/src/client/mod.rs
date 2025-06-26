//! DGraph client module
//! 
//! Provides high-level client functionality with connection pooling,
//! health checking, and automatic retry capabilities.

pub mod connection;
pub mod dgraph;
pub mod pool;

pub use connection::{ConnectionConfig, ConnectionStats, DgraphConnection};
pub use dgraph::{DgraphResponseParser, MutationResult, MutationOperationType, ConflictInfo};
pub use pool::{ConnectionPool, PoolConfig, PoolStats, PooledConnection};

use crate::error::{Result, ErrorContext, RetryExecutor, RetryPolicy, ResultExt};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

/// High-level DGraph client with connection pooling
pub struct DgraphClient {
    pool: Arc<ConnectionPool>,
}

impl DgraphClient {
    /// Create a new DGraph client with connection pooling
    pub async fn new(endpoint: String) -> Result<Self> {
        let pool_config = PoolConfig::default();
        Self::with_config(endpoint, pool_config).await
    }

    /// Create a new DGraph client with custom pool configuration
    pub async fn with_config(endpoint: String, pool_config: PoolConfig) -> Result<Self> {
        info!("Initializing DGraph client with endpoint: {}", endpoint);
        
        let pool = ConnectionPool::new(endpoint.clone(), pool_config)
            .await
            .with_context(|| ErrorContext::new("create_connection_pool")
                .with_endpoint(&endpoint))?;
        
        Ok(Self {
            pool: Arc::new(pool),
        })
    }

    /// Execute a read-only query with retry logic
    pub async fn query(&self, query: &str) -> Result<Value> {
        let retry_executor = RetryExecutor::new(RetryPolicy::default());
        
        retry_executor.execute(|| async {
            let conn = self.pool.acquire().await
                .with_context(|| ErrorContext::new("query")
                    .with_metadata("operation", "acquire_connection"))?;
            
            conn.connection().query(query).await
                .with_context(|| ErrorContext::new("query")
                    .with_metadata("query_type", "read_only"))
        }).await
    }

    /// Execute a query with variables
    pub async fn query_with_vars(
        &self,
        query: &str,
        vars: HashMap<String, String>,
    ) -> Result<Value> {
        let retry_executor = RetryExecutor::new(RetryPolicy::default());
        
        retry_executor.execute(|| async {
            let conn = self.pool.acquire().await
                .with_context(|| ErrorContext::new("query_with_vars")
                    .with_metadata("operation", "acquire_connection"))?;
            
            conn.connection().query_with_vars(query, vars.clone()).await
                .with_context(|| ErrorContext::new("query_with_vars")
                    .with_metadata("query_type", "parameterized"))
        }).await
    }

    /// Execute a mutation with retry logic
    pub async fn mutate(&self, mutation: &str) -> Result<Value> {
        let retry_executor = RetryExecutor::new(RetryPolicy {
            max_attempts: 2, // Fewer retries for mutations
            ..Default::default()
        });
        
        retry_executor.execute(|| async {
            let conn = self.pool.acquire().await
                .with_context(|| ErrorContext::new("mutate")
                    .with_metadata("operation", "acquire_connection"))?;
            
            conn.connection().mutate(mutation).await
                .with_context(|| ErrorContext::new("mutate")
                    .with_metadata("operation_type", "mutation"))
        }).await
    }

    /// Execute a transaction
    pub async fn transaction<F, T>(&self, operations: F) -> Result<T>
    where
        F: FnOnce(&dgraph_tonic::Client) -> Result<T> + Send,
        T: Send,
    {
        let conn = self.pool.acquire().await
            .context("Failed to acquire connection from pool")?;
        
        conn.connection().transaction(operations).await
    }

    /// Get pool statistics
    pub async fn pool_stats(&self) -> PoolStats {
        self.pool.stats().await
    }

    /// Check overall client health
    pub async fn health_check(&self) -> Result<bool> {
        let conn = self.pool.acquire().await
            .with_context(|| ErrorContext::new("health_check")
                .with_metadata("operation", "acquire_connection"))?;
        
        conn.connection().health_check().await
            .with_context(|| ErrorContext::new("health_check")
                .with_metadata("check_type", "connection_health"))
    }
}

impl Clone for DgraphClient {
    fn clone(&self) -> Self {
        Self {
            pool: Arc::clone(&self.pool),
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    #[ignore] // Requires DGraph instance
    async fn test_client_creation() {
        let client = DgraphClient::new("localhost:9080".to_string())
            .await
            .expect("Failed to create client");
        
        assert!(client.health_check().await.unwrap());
    }

    #[tokio::test]
    #[ignore] // Requires DGraph instance
    async fn test_client_query() {
        let client = DgraphClient::new("localhost:9080".to_string())
            .await
            .expect("Failed to create client");
        
        let query = r#"
            {
                q(func: has(dgraph.type)) {
                    count(uid)
                }
            }
        "#;
        
        let result = client.query(query).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires DGraph instance
    async fn test_client_pool_stats() {
        let client = DgraphClient::new("localhost:9080".to_string())
            .await
            .expect("Failed to create client");
        
        let stats = client.pool_stats().await;
        assert!(stats.total_connections > 0);
    }
}