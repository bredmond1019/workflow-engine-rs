//! Connection pool for DGraph clients
//! 
//! Provides efficient connection pooling with health checking,
//! automatic retry, and connection lifecycle management.

use crate::client::connection::DgraphConnection;
use crate::error::{KnowledgeGraphError, Result, ErrorContext, ResultExt};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

/// Configuration for the connection pool
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Minimum number of connections to maintain
    pub min_connections: usize,
    /// Maximum number of connections allowed
    pub max_connections: usize,
    /// Timeout for acquiring a connection from the pool
    pub acquire_timeout: Duration,
    /// Timeout for establishing a new connection
    pub connect_timeout: Duration,
    /// Maximum idle time before closing a connection
    pub max_idle_time: Duration,
    /// Interval for health checks
    pub health_check_interval: Duration,
    /// Maximum retry attempts for failed operations
    pub max_retry_attempts: usize,
    /// Initial retry delay
    pub initial_retry_delay: Duration,
    /// Maximum retry delay
    pub max_retry_delay: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_connections: 5,
            max_connections: 50,
            acquire_timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            max_idle_time: Duration::from_secs(300), // 5 minutes
            health_check_interval: Duration::from_secs(60), // 1 minute
            max_retry_attempts: 3,
            initial_retry_delay: Duration::from_millis(100),
            max_retry_delay: Duration::from_secs(5),
        }
    }
}

/// Statistics about the connection pool
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub idle_connections: usize,
    pub pending_requests: usize,
    pub successful_acquires: u64,
    pub failed_acquires: u64,
    pub total_created: u64,
    pub total_closed: u64,
}

/// A pooled connection wrapper
pub struct PooledConnection {
    connection: DgraphConnection,
    created_at: Instant,
    last_used: Instant,
    pool: Arc<ConnectionPoolInner>,
}

impl PooledConnection {
    /// Get a reference to the underlying connection
    pub fn connection(&self) -> &DgraphConnection {
        &self.connection
    }

    /// Check if the connection is still healthy
    pub async fn is_healthy(&self) -> bool {
        self.connection.health_check().await.unwrap_or(false)
    }

    /// Check if the connection has been idle for too long
    pub fn is_idle_timeout(&self, max_idle_time: Duration) -> bool {
        self.last_used.elapsed() > max_idle_time
    }

    /// Update the last used timestamp
    fn touch(&mut self) {
        self.last_used = Instant::now();
    }
}

impl Drop for PooledConnection {
    fn drop(&mut self) {
        // Return connection to pool
        let pool = Arc::clone(&self.pool);
        let connection = std::mem::replace(
            &mut self.connection,
            DgraphConnection::placeholder(), // Temporary placeholder
        );
        
        tokio::spawn(async move {
            pool.return_connection(connection).await;
        });
    }
}

/// Internal pool state
#[derive(Debug)]
struct ConnectionPoolInner {
    config: PoolConfig,
    available: Mutex<VecDeque<DgraphConnection>>,
    semaphore: Semaphore,
    active_count: AtomicUsize,
    stats: Mutex<PoolStats>,
    dgraph_endpoint: String,
}

/// Connection pool for DGraph clients
pub struct ConnectionPool {
    inner: Arc<ConnectionPoolInner>,
    _health_check_handle: tokio::task::JoinHandle<()>,
}

impl ConnectionPool {
    /// Create a new connection pool
    pub async fn new(dgraph_endpoint: String, config: PoolConfig) -> Result<Self> {
        info!(
            "Creating DGraph connection pool with {} min, {} max connections",
            config.min_connections, config.max_connections
        );

        let inner = Arc::new(ConnectionPoolInner {
            available: Mutex::new(VecDeque::new()),
            semaphore: Semaphore::new(config.max_connections),
            active_count: AtomicUsize::new(0),
            stats: Mutex::new(PoolStats {
                total_connections: 0,
                active_connections: 0,
                idle_connections: 0,
                pending_requests: 0,
                successful_acquires: 0,
                failed_acquires: 0,
                total_created: 0,
                total_closed: 0,
            }),
            dgraph_endpoint: dgraph_endpoint.clone(),
            config: config.clone(),
        });

        // Initialize minimum connections
        for i in 0..config.min_connections {
            let connection = DgraphConnection::new(&dgraph_endpoint)
                .await
                .with_context(|| ErrorContext::new("create_initial_connection")
                    .with_endpoint(&dgraph_endpoint)
                    .with_metadata("connection_index", i.to_string()))?;
            
            inner.available.lock().await.push_back(connection);
            inner.stats.lock().await.total_created += 1;
        }

        // Start health check task
        let health_check_handle = {
            let inner_clone = Arc::clone(&inner);
            tokio::spawn(async move {
                Self::health_check_task(inner_clone).await;
            })
        };

        info!(
            "Successfully created DGraph connection pool with {} initial connections",
            config.min_connections
        );

        Ok(Self {
            inner,
            _health_check_handle: health_check_handle,
        })
    }

    /// Acquire a connection from the pool
    pub async fn acquire(&self) -> Result<PooledConnection> {
        self.acquire_with_retry().await
    }

    /// Acquire a connection with retry logic
    async fn acquire_with_retry(&self) -> Result<PooledConnection> {
        let mut attempts = 0;
        let mut delay = self.inner.config.initial_retry_delay;

        loop {
            match self.try_acquire().await {
                Ok(conn) => {
                    self.inner.stats.lock().await.successful_acquires += 1;
                    return Ok(conn);
                }
                Err(e) if attempts < self.inner.config.max_retry_attempts => {
                    attempts += 1;
                    warn!(
                        "Failed to acquire connection (attempt {}/{}): {}",
                        attempts, self.inner.config.max_retry_attempts, e
                    );
                    
                    tokio::time::sleep(delay).await;
                    delay = std::cmp::min(delay * 2, self.inner.config.max_retry_delay);
                }
                Err(e) => {
                    self.inner.stats.lock().await.failed_acquires += 1;
                    let stats = self.inner.stats.lock().await.clone();
                    return Err(KnowledgeGraphError::ConnectionPoolError {
                        message: format!("Failed to acquire connection after {} retries", attempts),
                        available_connections: stats.idle_connections,
                        max_connections: self.inner.config.max_connections,
                        source_error: Some(e.to_string()),
                    });
                }
            }
        }
    }

    /// Try to acquire a connection (single attempt)
    async fn try_acquire(&self) -> Result<PooledConnection> {
        // Wait for available slot
        let _permit = timeout(
            self.inner.config.acquire_timeout,
            self.inner.semaphore.acquire(),
        )
        .await
        .map_err(|_| KnowledgeGraphError::TimeoutError {
            operation: "acquire_connection_slot".to_string(),
            duration: self.inner.config.acquire_timeout,
            endpoint: self.inner.dgraph_endpoint.clone(),
        })?
        .map_err(|e| {
            let available_connections = 0; // Approximate since we can't call async here
            KnowledgeGraphError::ConnectionPoolError {
                message: "Failed to acquire semaphore permit".to_string(),
                available_connections,
                max_connections: self.inner.config.max_connections,
                source_error: Some(e.to_string()),
            }
        })?;

        // Try to get an existing connection
        if let Some(connection) = self.inner.available.lock().await.pop_front() {
            // Check if connection is still healthy
            if connection.health_check().await.unwrap_or(false) {
                self.inner.active_count.fetch_add(1, Ordering::Relaxed);
                
                return Ok(PooledConnection {
                    connection,
                    created_at: Instant::now(),
                    last_used: Instant::now(),
                    pool: Arc::clone(&self.inner),
                });
            } else {
                // Connection is unhealthy, create a new one
                debug!("Discarding unhealthy connection");
                self.inner.stats.lock().await.total_closed += 1;
            }
        }

        // Create new connection
        let connection = timeout(
            self.inner.config.connect_timeout,
            DgraphConnection::new(&self.inner.dgraph_endpoint),
        )
        .await
        .map_err(|_| KnowledgeGraphError::TimeoutError {
            operation: "create_new_connection".to_string(),
            duration: self.inner.config.connect_timeout,
            endpoint: self.inner.dgraph_endpoint.clone(),
        })?
        .with_context(|| ErrorContext::new("create_connection")
            .with_endpoint(&self.inner.dgraph_endpoint))?;

        self.inner.active_count.fetch_add(1, Ordering::Relaxed);
        self.inner.stats.lock().await.total_created += 1;

        Ok(PooledConnection {
            connection,
            created_at: Instant::now(),
            last_used: Instant::now(),
            pool: Arc::clone(&self.inner),
        })
    }

    /// Get pool statistics
    pub async fn stats(&self) -> PoolStats {
        let mut stats = self.inner.stats.lock().await.clone();
        stats.active_connections = self.inner.active_count.load(Ordering::Relaxed);
        stats.idle_connections = self.inner.available.lock().await.len();
        stats.total_connections = stats.active_connections + stats.idle_connections;
        stats.pending_requests = self.inner.config.max_connections - self.inner.semaphore.available_permits();
        stats
    }

    /// Health check task that runs periodically
    async fn health_check_task(inner: Arc<ConnectionPoolInner>) {
        let mut interval = tokio::time::interval(inner.config.health_check_interval);
        
        loop {
            interval.tick().await;
            
            let mut available = inner.available.lock().await;
            let mut healthy_connections = VecDeque::new();
            let mut closed_count = 0;
            
            // Check each available connection
            while let Some(connection) = available.pop_front() {
                if connection.health_check().await.unwrap_or(false) 
                    && !connection.is_idle_timeout(inner.config.max_idle_time) {
                    healthy_connections.push_back(connection);
                } else {
                    closed_count += 1;
                    debug!("Closed unhealthy or idle connection");
                }
            }
            
            *available = healthy_connections;
            
            if closed_count > 0 {
                inner.stats.lock().await.total_closed += closed_count;
                debug!("Health check closed {} unhealthy connections", closed_count);
            }
            
            // Ensure minimum connections
            let current_total = available.len() + inner.active_count.load(Ordering::Relaxed);
            if current_total < inner.config.min_connections {
                let needed = inner.config.min_connections - current_total;
                debug!("Creating {} connections to maintain minimum", needed);
                
                for _ in 0..needed {
                    match DgraphConnection::new(&inner.dgraph_endpoint).await {
                        Ok(connection) => {
                            available.push_back(connection);
                            inner.stats.lock().await.total_created += 1;
                        }
                        Err(e) => {
                            error!("Failed to create connection during health check: {}", e);
                            break;
                        }
                    }
                }
            }
        }
    }
}

impl ConnectionPoolInner {
    /// Return a connection to the pool
    async fn return_connection(&self, connection: DgraphConnection) {
        self.active_count.fetch_sub(1, Ordering::Relaxed);
        
        // Check if we should keep the connection
        let current_idle = self.available.lock().await.len();
        let max_idle = self.config.max_connections - self.config.min_connections;
        
        if current_idle < max_idle && connection.health_check().await.unwrap_or(false) {
            self.available.lock().await.push_back(connection);
        } else {
            // Too many idle connections or connection is unhealthy
            self.stats.lock().await.total_closed += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    #[ignore] // Requires DGraph instance
    async fn test_pool_creation() {
        let config = PoolConfig {
            min_connections: 2,
            max_connections: 5,
            ..Default::default()
        };
        
        let pool = ConnectionPool::new("localhost:9080".to_string(), config)
            .await
            .expect("Failed to create pool");
        
        let stats = pool.stats().await;
        assert_eq!(stats.idle_connections, 2);
    }

    #[tokio::test]
    #[ignore] // Requires DGraph instance
    async fn test_connection_acquire_and_return() {
        let config = PoolConfig {
            min_connections: 1,
            max_connections: 3,
            ..Default::default()
        };
        
        let pool = ConnectionPool::new("localhost:9080".to_string(), config)
            .await
            .expect("Failed to create pool");
        
        let conn = pool.acquire().await.expect("Failed to acquire connection");
        assert!(conn.is_healthy().await);
        
        drop(conn); // Should return to pool
        
        // Give time for the connection to be returned
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let stats = pool.stats().await;
        assert_eq!(stats.active_connections, 0);
    }

    #[tokio::test]
    async fn test_pool_config_defaults() {
        let config = PoolConfig::default();
        assert_eq!(config.min_connections, 5);
        assert_eq!(config.max_connections, 50);
        assert_eq!(config.max_retry_attempts, 3);
    }
}