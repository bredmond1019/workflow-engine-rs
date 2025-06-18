//! Individual DGraph connection wrapper
//! 
//! Provides a high-level interface for individual DGraph connections
//! with health checking, timeout handling, and error recovery.

use crate::error::{KnowledgeGraphError, Result, ErrorContext, ResultExt};
use dgraph_tonic::{Client, Query};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{debug, error, warn};

/// Connection statistics
#[derive(Debug, Clone, Default)]
pub struct ConnectionStats {
    pub queries_executed: u64,
    pub mutations_executed: u64,
    pub errors: u64,
    pub avg_response_time_ms: f64,
    pub last_error: Option<String>,
    pub created_at_ms: Option<u64>,
    pub last_used_at_ms: Option<u64>,
}

/// Configuration for individual connections
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    /// Query timeout duration
    pub query_timeout: Duration,
    /// Mutation timeout duration
    pub mutation_timeout: Duration,
    /// Health check timeout duration
    pub health_check_timeout: Duration,
    /// Maximum number of concurrent operations
    pub max_concurrent_ops: usize,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            query_timeout: Duration::from_secs(30),
            mutation_timeout: Duration::from_secs(60),
            health_check_timeout: Duration::from_secs(5),
            max_concurrent_ops: 100,
        }
    }
}

/// A wrapper around DGraph client with enhanced functionality
#[derive(Debug)]
pub struct DgraphConnection {
    client: Arc<Client>,
    config: ConnectionConfig,
    stats: Arc<tokio::sync::RwLock<ConnectionStats>>,
    operation_count: Arc<AtomicU64>,
    endpoint: String,
}

impl DgraphConnection {
    /// Create a new DGraph connection
    pub async fn new(endpoint: &str) -> Result<Self> {
        debug!("Creating new DGraph connection to {}", endpoint);
        
        let client = Client::new(endpoint)
            .map_err(|e| KnowledgeGraphError::NetworkError {
                message: format!("Failed to create DGraph client: {}", e),
                endpoint: endpoint.to_string(),
                retry_count: 0,
                source_error: Some(e.to_string()),
            })?;
        
        let connection = Self {
            client: Arc::new(client),
            config: ConnectionConfig::default(),
            stats: Arc::new(tokio::sync::RwLock::new(ConnectionStats {
                created_at_ms: Some(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64
                ),
                ..Default::default()
            })),
            operation_count: Arc::new(AtomicU64::new(0)),
            endpoint: endpoint.to_string(),
        };
        
        // Test the connection
        connection.health_check().await
            .with_context(|| ErrorContext::new("initial_health_check")
                .with_endpoint(endpoint))?;
            
        debug!("Successfully created DGraph connection to {}", endpoint);
        Ok(connection)
    }

    /// Create a new connection with custom configuration
    pub async fn with_config(endpoint: &str, config: ConnectionConfig) -> Result<Self> {
        let mut connection = Self::new(endpoint).await?;
        connection.config = config;
        Ok(connection)
    }

    /// Create a placeholder connection (used internally by the pool)
    pub fn placeholder() -> Self {
        // This should never be used for actual operations
        Self {
            client: Arc::new(Client::new("placeholder:9080").unwrap()),
            config: ConnectionConfig::default(),
            stats: Arc::new(tokio::sync::RwLock::new(ConnectionStats::default())),
            operation_count: Arc::new(AtomicU64::new(0)),
            endpoint: "placeholder:9080".to_string(),
        }
    }

    /// Execute a read-only query
    pub async fn query(&self, query: &str) -> Result<Value> {
        self.query_with_vars(query, HashMap::new()).await
    }

    /// Execute a query with variables
    pub async fn query_with_vars(
        &self,
        query: &str,
        vars: HashMap<String, String>,
    ) -> Result<Value> {
        let start = Instant::now();
        let op_id = self.operation_count.fetch_add(1, Ordering::Relaxed);
        
        debug!("Executing query {} on {}: {}", op_id, self.endpoint, query);
        
        let result = self.execute_query(query, vars).await;
        
        let duration = start.elapsed();
        self.update_stats(duration, false, result.is_ok()).await;
        
        match result {
            Ok(response) => {
                debug!("Query {} completed in {:?}", op_id, duration);
                Ok(response)
            }
            Err(e) => {
                error!("Query {} failed after {:?}: {}", op_id, duration, e);
                Err(e)
            }
        }
    }

    /// Execute a mutation
    pub async fn mutate(&self, mutation: &str) -> Result<Value> {
        let start = Instant::now();
        let op_id = self.operation_count.fetch_add(1, Ordering::Relaxed);
        
        debug!("Executing mutation {} on {}: {}", op_id, self.endpoint, mutation);
        
        let result = self.execute_mutation(mutation).await;
        
        let duration = start.elapsed();
        self.update_stats(duration, true, result.is_ok()).await;
        
        match result {
            Ok(response) => {
                debug!("Mutation {} completed in {:?}", op_id, duration);
                Ok(response)
            }
            Err(e) => {
                error!("Mutation {} failed after {:?}: {}", op_id, duration, e);
                Err(e)
            }
        }
    }

    /// Execute a transaction
    pub async fn transaction<F, T>(&self, operations: F) -> Result<T>
    where
        F: FnOnce(&Client) -> Result<T> + Send,
        T: Send,
    {
        let start = Instant::now();
        let op_id = self.operation_count.fetch_add(1, Ordering::Relaxed);
        
        debug!("Executing transaction {} on {}", op_id, self.endpoint);
        
        let result = timeout(
            self.config.mutation_timeout,
            self.execute_transaction(operations),
        )
        .await
        .context("Transaction timeout")?;
        
        let duration = start.elapsed();
        self.update_stats(duration, true, result.is_ok()).await;
        
        match result {
            Ok(response) => {
                debug!("Transaction {} completed in {:?}", op_id, duration);
                Ok(response)
            }
            Err(e) => {
                error!("Transaction {} failed after {:?}: {}", op_id, duration, e);
                Err(e)
            }
        }
    }

    /// Check if the connection is healthy
    pub async fn health_check(&self) -> Result<bool> {
        debug!("Performing health check on {}", self.endpoint);
        
        let health_query = r#"
            {
                health() {
                    status
                    version
                }
            }
        "#;
        
        match timeout(
            self.config.health_check_timeout,
            self.client.new_read_only_txn().query(health_query),
        )
        .await
        {
            Ok(Ok(_)) => {
                debug!("Health check passed for {}", self.endpoint);
                Ok(true)
            }
            Ok(Err(e)) => {
                warn!("Health check failed for {}: {}", self.endpoint, e);
                Ok(false)
            }
            Err(_) => {
                warn!("Health check timeout for {}", self.endpoint);
                Ok(false)
            }
        }
    }

    /// Check if connection has been idle for too long
    pub fn is_idle_timeout(&self, _max_idle_time: Duration) -> bool {
        // This would be implemented by checking last_used_at from stats
        // For now, return false as we update stats on each operation
        false
    }

    /// Get connection statistics
    pub async fn stats(&self) -> ConnectionStats {
        self.stats.read().await.clone()
    }

    /// Get the underlying DGraph client
    pub fn client(&self) -> Arc<Client> {
        Arc::clone(&self.client)
    }

    /// Execute a query with timeout
    async fn execute_query(
        &self,
        query: &str,
        vars: HashMap<String, String>,
    ) -> Result<Value> {
        let mut txn = self.client.new_read_only_txn();
        
        let response = if vars.is_empty() {
            timeout(self.config.query_timeout, txn.query(query)).await
        } else {
            timeout(self.config.query_timeout, txn.query_with_vars(query, vars)).await
        }
        .map_err(|_| KnowledgeGraphError::TimeoutError {
            operation: "query".to_string(),
            duration: self.config.query_timeout,
            endpoint: self.endpoint.clone(),
        })?
        .map_err(|e| KnowledgeGraphError::NetworkError {
            message: format!("Query execution failed: {}", e),
            endpoint: self.endpoint.clone(),
            retry_count: 0,
            source_error: Some(e.to_string()),
        })?;
        
        let result: Value = serde_json::from_slice(&response.json)
            .map_err(|e| KnowledgeGraphError::ParseError {
                message: format!("Failed to parse query response: {}", e),
                field: None,
                raw_data: Some(String::from_utf8_lossy(&response.json).to_string()),
                source_error: Some(e.to_string()),
            })?;
        
        Ok(result)
    }

    /// Execute a mutation with timeout
    async fn execute_mutation(&self, mutation: &str) -> Result<Value> {
        // For GraphQL mutations, we use the HTTP endpoint
        let endpoint = format!("http://{}/graphql", self.endpoint.replace(":9080", ":8080"));
        let client = reqwest::Client::new();
        
        let response = timeout(
            self.config.mutation_timeout,
            client.post(&endpoint)
                .header("Content-Type", "application/json")
                .json(&serde_json::json!({
                    "query": mutation
                }))
                .send()
        )
        .await
        .map_err(|_| KnowledgeGraphError::TimeoutError {
            operation: "mutation".to_string(),
            duration: self.config.mutation_timeout,
            endpoint: self.endpoint.clone(),
        })?
        .map_err(|e| KnowledgeGraphError::NetworkError {
            message: format!("Failed to send mutation request: {}", e),
            endpoint: self.endpoint.clone(),
            retry_count: 0,
            source_error: Some(e.to_string()),
        })?;

        let result: Value = response.json().await
            .map_err(|e| KnowledgeGraphError::ParseError {
                message: format!("Failed to parse mutation response: {}", e),
                field: None,
                raw_data: None,
                source_error: Some(e.to_string()),
            })?;
        
        Ok(result)
    }

    /// Execute a transaction with timeout
    async fn execute_transaction<F, T>(&self, operations: F) -> Result<T>
    where
        F: FnOnce(&Client) -> Result<T> + Send,
        T: Send,
    {
        operations(self.client.as_ref())
    }

    /// Update connection statistics
    async fn update_stats(&self, duration: Duration, is_mutation: bool, success: bool) {
        let mut stats = self.stats.write().await;
        
        stats.last_used_at_ms = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64
        );
        
        if is_mutation {
            stats.mutations_executed += 1;
        } else {
            stats.queries_executed += 1;
        }
        
        if !success {
            stats.errors += 1;
        }
        
        // Update running average of response time
        let total_ops = stats.queries_executed + stats.mutations_executed;
        let duration_ms = duration.as_millis() as f64;
        
        if total_ops == 1 {
            stats.avg_response_time_ms = duration_ms;
        } else {
            stats.avg_response_time_ms = 
                (stats.avg_response_time_ms * (total_ops - 1) as f64 + duration_ms) / total_ops as f64;
        }
    }
}

impl Clone for DgraphConnection {
    fn clone(&self) -> Self {
        Self {
            client: Arc::clone(&self.client),
            config: self.config.clone(),
            stats: Arc::clone(&self.stats),
            operation_count: Arc::clone(&self.operation_count),
            endpoint: self.endpoint.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    #[ignore] // Requires DGraph instance
    async fn test_connection_creation() {
        let conn = DgraphConnection::new("localhost:9080")
            .await
            .expect("Failed to create connection");
        
        assert!(conn.health_check().await.unwrap());
    }

    #[tokio::test]
    #[ignore] // Requires DGraph instance
    async fn test_simple_query() {
        let conn = DgraphConnection::new("localhost:9080")
            .await
            .expect("Failed to create connection");
        
        let query = r#"
            {
                q(func: has(dgraph.type)) {
                    count(uid)
                }
            }
        "#;
        
        let result = conn.query(query).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires DGraph instance
    async fn test_connection_stats() {
        let conn = DgraphConnection::new("localhost:9080")
            .await
            .expect("Failed to create connection");
        
        let initial_stats = conn.stats().await;
        assert_eq!(initial_stats.queries_executed, 0);
        
        // Execute a query
        let query = "{ health() }";
        let _ = conn.query(query).await;
        
        let updated_stats = conn.stats().await;
        assert_eq!(updated_stats.queries_executed, 1);
    }

    #[tokio::test]
    async fn test_connection_config_defaults() {
        let config = ConnectionConfig::default();
        assert_eq!(config.query_timeout, Duration::from_secs(30));
        assert_eq!(config.mutation_timeout, Duration::from_secs(60));
        assert_eq!(config.max_concurrent_ops, 100);
    }
}