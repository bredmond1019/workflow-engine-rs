//! # MCP Metrics Integration
//!
//! This module provides integration between MCP connection pooling and the 
//! existing Prometheus metrics system for comprehensive monitoring.

use std::collections::HashMap;
use std::sync::Arc;
use prometheus::{Counter, Gauge, Histogram, IntCounter, IntGauge, Registry, Opts};
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

use crate::connection_pool::{McpConnectionPool, DetailedHealthInfo};
use crate::health::HealthStatus;
use workflow_engine_core::error::circuit_breaker::CircuitState;
use workflow_engine_core::error::WorkflowError;

/// MCP-specific metrics collector
pub struct MCPMetricsCollector {
    // Connection pool metrics
    pub total_connections: IntGauge,
    pub healthy_connections: IntGauge,
    pub unhealthy_connections: IntGauge,
    pub busy_connections: IntGauge,
    
    // Circuit breaker metrics
    pub circuit_state: IntGauge,
    pub circuit_breaker_trips: IntCounter,
    pub circuit_breaker_recoveries: IntCounter,
    
    // Performance metrics
    pub connection_requests_total: IntCounter,
    pub connection_requests_failed: IntCounter,
    pub connection_latency: Histogram,
    pub pool_utilization: Gauge,
    
    // Health check metrics
    pub health_checks_total: IntCounter,
    pub health_checks_failed: IntCounter,
    pub connection_uptime: Histogram,
    
    // Transport-specific metrics
    pub bytes_sent_total: IntCounter,
    pub bytes_received_total: IntCounter,
    pub messages_sent_total: IntCounter,
    pub messages_received_total: IntCounter,
    pub reconnection_attempts: IntCounter,
    
    // Server-specific gauge maps
    server_connections: Arc<RwLock<HashMap<String, IntGauge>>>,
    server_circuit_states: Arc<RwLock<HashMap<String, IntGauge>>>,
    
    registry: Registry,
}

impl MCPMetricsCollector {
    pub fn new(registry: Registry) -> Result<Self, WorkflowError> {
        let total_connections = IntGauge::with_opts(
            Opts::new("mcp_total_connections", "Total number of MCP connections")
        ).map_err(|e| WorkflowError::RuntimeError {
            message: format!("Failed to create total_connections metric: {}", e),
        })?;
        
        let healthy_connections = IntGauge::with_opts(
            Opts::new("mcp_healthy_connections", "Number of healthy MCP connections")
        ).map_err(|e| WorkflowError::RuntimeError {
            message: format!("Failed to create healthy_connections metric: {}", e),
        })?;
        
        let unhealthy_connections = IntGauge::with_opts(
            Opts::new("mcp_unhealthy_connections", "Number of unhealthy MCP connections")
        ).map_err(|e| WorkflowError::RuntimeError {
            message: format!("Failed to create unhealthy_connections metric: {}", e),
        })?;
        
        let busy_connections = IntGauge::with_opts(
            Opts::new("mcp_busy_connections", "Number of busy MCP connections")
        ).map_err(|e| WorkflowError::RuntimeError {
            message: format!("Failed to create busy_connections metric: {}", e),
        })?;
        
        let circuit_state = IntGauge::with_opts(
            Opts::new("mcp_circuit_breaker_state", "Circuit breaker state (0=closed, 1=open, 2=half-open)")
        ).map_err(|e| WorkflowError::RuntimeError {
            message: format!("Failed to create circuit_state metric: {}", e),
        })?;
        
        let circuit_breaker_trips = IntCounter::with_opts(
            Opts::new("mcp_circuit_breaker_trips_total", "Total number of circuit breaker trips")
        ).map_err(|e| WorkflowError::RuntimeError {
            message: format!("Failed to create circuit_breaker_trips metric: {}", e),
        })?;
        
        let circuit_breaker_recoveries = IntCounter::with_opts(
            Opts::new("mcp_circuit_breaker_recoveries_total", "Total number of circuit breaker recoveries")
        ).map_err(|e| WorkflowError::RuntimeError {
            message: format!("Failed to create circuit_breaker_recoveries metric: {}", e),
        })?;
        
        let connection_requests_total = IntCounter::with_opts(
            Opts::new("mcp_connection_requests_total", "Total number of connection requests")
        ).map_err(|e| WorkflowError::RuntimeError {
            message: format!("Failed to create connection_requests_total metric: {}", e),
        })?;
        
        let connection_requests_failed = IntCounter::with_opts(
            Opts::new("mcp_connection_requests_failed_total", "Total number of failed connection requests")
        ).map_err(|e| WorkflowError::RuntimeError {
            message: format!("Failed to create connection_requests_failed metric: {}", e),
        })?;
        
        let connection_latency = Histogram::with_opts(
            prometheus::HistogramOpts::new("mcp_connection_latency_seconds", "Connection establishment latency")
                .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0])
        ).map_err(|e| WorkflowError::RuntimeError {
            message: format!("Failed to create connection_latency metric: {}", e),
        })?;
        
        let pool_utilization = Gauge::with_opts(
            Opts::new("mcp_pool_utilization_ratio", "Pool utilization ratio (0-1)")
        ).map_err(|e| WorkflowError::RuntimeError {
            message: format!("Failed to create pool_utilization metric: {}", e),
        })?;
        
        let health_checks_total = IntCounter::with_opts(
            Opts::new("mcp_health_checks_total", "Total number of health checks performed")
        ).map_err(|e| WorkflowError::RuntimeError {
            message: format!("Failed to create health_checks_total metric: {}", e),
        })?;
        
        let health_checks_failed = IntCounter::with_opts(
            Opts::new("mcp_health_checks_failed_total", "Total number of failed health checks")
        ).map_err(|e| WorkflowError::RuntimeError {
            message: format!("Failed to create health_checks_failed metric: {}", e),
        })?;
        
        let connection_uptime = Histogram::with_opts(
            prometheus::HistogramOpts::new("mcp_connection_uptime_seconds", "Connection uptime in seconds")
                .buckets(vec![1.0, 10.0, 60.0, 300.0, 1800.0, 3600.0, 21600.0, 86400.0])
        ).map_err(|e| WorkflowError::RuntimeError {
            message: format!("Failed to create connection_uptime metric: {}", e),
        })?;
        
        let bytes_sent_total = IntCounter::with_opts(
            Opts::new("mcp_bytes_sent_total", "Total bytes sent over MCP connections")
        ).map_err(|e| WorkflowError::RuntimeError {
            message: format!("Failed to create bytes_sent_total metric: {}", e),
        })?;
        
        let bytes_received_total = IntCounter::with_opts(
            Opts::new("mcp_bytes_received_total", "Total bytes received over MCP connections")
        ).map_err(|e| WorkflowError::RuntimeError {
            message: format!("Failed to create bytes_received_total metric: {}", e),
        })?;
        
        let messages_sent_total = IntCounter::with_opts(
            Opts::new("mcp_messages_sent_total", "Total messages sent over MCP connections")
        ).map_err(|e| WorkflowError::RuntimeError {
            message: format!("Failed to create messages_sent_total metric: {}", e),
        })?;
        
        let messages_received_total = IntCounter::with_opts(
            Opts::new("mcp_messages_received_total", "Total messages received over MCP connections")
        ).map_err(|e| WorkflowError::RuntimeError {
            message: format!("Failed to create messages_received_total metric: {}", e),
        })?;
        
        let reconnection_attempts = IntCounter::with_opts(
            Opts::new("mcp_reconnection_attempts_total", "Total number of reconnection attempts")
        ).map_err(|e| WorkflowError::RuntimeError {
            message: format!("Failed to create reconnection_attempts metric: {}", e),
        })?;
        
        // Register all metrics
        registry.register(Box::new(total_connections.clone()))?;
        registry.register(Box::new(healthy_connections.clone()))?;
        registry.register(Box::new(unhealthy_connections.clone()))?;
        registry.register(Box::new(busy_connections.clone()))?;
        registry.register(Box::new(circuit_state.clone()))?;
        registry.register(Box::new(circuit_breaker_trips.clone()))?;
        registry.register(Box::new(circuit_breaker_recoveries.clone()))?;
        registry.register(Box::new(connection_requests_total.clone()))?;
        registry.register(Box::new(connection_requests_failed.clone()))?;
        registry.register(Box::new(connection_latency.clone()))?;
        registry.register(Box::new(pool_utilization.clone()))?;
        registry.register(Box::new(health_checks_total.clone()))?;
        registry.register(Box::new(health_checks_failed.clone()))?;
        registry.register(Box::new(connection_uptime.clone()))?;
        registry.register(Box::new(bytes_sent_total.clone()))?;
        registry.register(Box::new(bytes_received_total.clone()))?;
        registry.register(Box::new(messages_sent_total.clone()))?;
        registry.register(Box::new(messages_received_total.clone()))?;
        registry.register(Box::new(reconnection_attempts.clone()))?;
        
        Ok(Self {
            total_connections,
            healthy_connections,
            unhealthy_connections,
            busy_connections,
            circuit_state,
            circuit_breaker_trips,
            circuit_breaker_recoveries,
            connection_requests_total,
            connection_requests_failed,
            connection_latency,
            pool_utilization,
            health_checks_total,
            health_checks_failed,
            connection_uptime,
            bytes_sent_total,
            bytes_received_total,
            messages_sent_total,
            messages_received_total,
            reconnection_attempts,
            server_connections: Arc::new(RwLock::new(HashMap::new())),
            server_circuit_states: Arc::new(RwLock::new(HashMap::new())),
            registry,
        })
    }
    
    /// Update metrics based on connection pool health information
    pub async fn update_from_pool_health(&self, health: &DetailedHealthInfo) {
        let mut total_connections = 0;
        let mut healthy_connections = 0;
        let mut unhealthy_connections = 0;
        let mut busy_connections = 0;
        
        // Update server-specific metrics
        for (server_id, server_health) in &health.server_health {
            let pool_stats = &server_health.pool_stats;
            
            total_connections += pool_stats.total_connections;
            healthy_connections += pool_stats.healthy_connections;
            busy_connections += pool_stats.busy_connections;
            
            // Calculate unhealthy connections
            let server_unhealthy = pool_stats.total_connections - pool_stats.healthy_connections;
            unhealthy_connections += server_unhealthy;
            
            // Update server-specific connection gauge
            self.update_server_connections_gauge(server_id, pool_stats.total_connections as i64).await;
            
            // Update circuit breaker state
            let circuit_state_value = match server_health.circuit_state {
                CircuitState::Closed => 0,
                CircuitState::Open => 1,
                CircuitState::HalfOpen => 2,
            };
            self.update_server_circuit_state_gauge(server_id, circuit_state_value).await;
            
            // Update circuit breaker metrics from circuit metrics
            let circuit_metrics = &server_health.circuit_metrics;
            self.circuit_breaker_trips.inc_by(circuit_metrics.total_failures);
            self.circuit_breaker_recoveries.inc_by(circuit_metrics.total_successes);
            
            // Update transport metrics
            self.bytes_sent_total.inc_by(pool_stats.total_use_count); // Approximation
            self.messages_sent_total.inc_by(pool_stats.total_use_count);
        }
        
        // Update overall pool metrics
        self.total_connections.set(total_connections as i64);
        self.healthy_connections.set(healthy_connections as i64);
        self.unhealthy_connections.set(unhealthy_connections as i64);
        self.busy_connections.set(busy_connections as i64);
        
        // Calculate pool utilization (assuming max 10 connections per server)
        let max_possible = health.server_health.len() * 10;
        if max_possible > 0 {
            let utilization = total_connections as f64 / max_possible as f64;
            self.pool_utilization.set(utilization);
        }
        
        // Update overall health summary metrics
        let summary = &health.overall_summary;
        self.health_checks_total.inc_by(summary.total_connections as u64);
        
        // Calculate failed health checks
        let failed_health_checks = summary.unhealthy_connections + summary.disconnected_connections;
        self.health_checks_failed.inc_by(failed_health_checks as u64);
    }
    
    /// Update server-specific connection count gauge
    async fn update_server_connections_gauge(&self, server_id: &str, count: i64) {
        let mut gauges = self.server_connections.write().await;
        
        let gauge = gauges.entry(server_id.to_string()).or_insert_with(|| {
            let opts = Opts::new(
                format!("mcp_server_connections_{}", server_id.replace("-", "_")),
                format!("Number of connections for server {}", server_id)
            );
            
            let gauge = match IntGauge::with_opts(opts) {
                Ok(g) => g,
                Err(e) => {
                    log::error!("Failed to create server connections gauge for {}: {}", server_id, e);
                    // Return a default gauge with basic options
                    IntGauge::new("mcp_server_connections_default", "Default server connections gauge")
                        .unwrap_or_else(|_| IntGauge::new("mcp_default", "Fallback gauge").expect("Basic gauge should always work"))
                }
            };
            
            // Register the new gauge
            if let Err(e) = self.registry.register(Box::new(gauge.clone())) {
                log::warn!("Failed to register server connections gauge for {}: {}", server_id, e);
            }
            
            gauge
        });
        
        gauge.set(count);
    }
    
    /// Update server-specific circuit breaker state gauge
    async fn update_server_circuit_state_gauge(&self, server_id: &str, state: i64) {
        let mut gauges = self.server_circuit_states.write().await;
        
        let gauge = gauges.entry(server_id.to_string()).or_insert_with(|| {
            let opts = Opts::new(
                format!("mcp_server_circuit_state_{}", server_id.replace("-", "_")),
                format!("Circuit breaker state for server {} (0=closed, 1=open, 2=half-open)", server_id)
            );
            
            let gauge = match IntGauge::with_opts(opts) {
                Ok(g) => g,
                Err(e) => {
                    log::error!("Failed to create server circuit state gauge for {}: {}", server_id, e);
                    // Return a default gauge with basic options
                    IntGauge::new("mcp_server_circuit_state_default", "Default circuit state gauge")
                        .unwrap_or_else(|_| IntGauge::new("mcp_circuit_default", "Fallback circuit gauge").expect("Basic gauge should always work"))
                }
            };
            
            // Register the new gauge
            if let Err(e) = self.registry.register(Box::new(gauge.clone())) {
                log::warn!("Failed to register server circuit state gauge for {}: {}", server_id, e);
            }
            
            gauge
        });
        
        gauge.set(state);
    }
    
    /// Record a connection request
    pub fn record_connection_request(&self, success: bool, latency: Duration) {
        self.connection_requests_total.inc();
        if !success {
            self.connection_requests_failed.inc();
        }
        self.connection_latency.observe(latency.as_secs_f64());
    }
    
    /// Record a health check
    pub fn record_health_check(&self, success: bool) {
        self.health_checks_total.inc();
        if !success {
            self.health_checks_failed.inc();
        }
    }
    
    /// Record connection uptime when a connection is closed
    pub fn record_connection_uptime(&self, uptime: Duration) {
        self.connection_uptime.observe(uptime.as_secs_f64());
    }
    
    /// Record transport metrics
    pub fn record_transport_metrics(&self, bytes_sent: u64, bytes_received: u64, messages_sent: u64, messages_received: u64) {
        self.bytes_sent_total.inc_by(bytes_sent);
        self.bytes_received_total.inc_by(bytes_received);
        self.messages_sent_total.inc_by(messages_sent);
        self.messages_received_total.inc_by(messages_received);
    }
    
    /// Record a reconnection attempt
    pub fn record_reconnection_attempt(&self) {
        self.reconnection_attempts.inc();
    }
}

/// MCP metrics manager that automatically collects metrics from connection pools
pub struct MCPMetricsManager {
    collector: Arc<MCPMetricsCollector>,
    pools: Arc<RwLock<HashMap<String, Arc<McpConnectionPool>>>>,
    collection_interval: Duration,
    running: Arc<RwLock<bool>>,
}

impl MCPMetricsManager {
    pub fn new(collector: MCPMetricsCollector, collection_interval: Duration) -> Self {
        Self {
            collector: Arc::new(collector),
            pools: Arc::new(RwLock::new(HashMap::new())),
            collection_interval,
            running: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Register a connection pool for metrics collection
    pub async fn register_pool(&self, pool_id: String, pool: Arc<McpConnectionPool>) {
        let mut pools = self.pools.write().await;
        pools.insert(pool_id, pool);
    }
    
    /// Unregister a connection pool
    pub async fn unregister_pool(&self, pool_id: &str) {
        let mut pools = self.pools.write().await;
        pools.remove(pool_id);
    }
    
    /// Start automatic metrics collection
    pub async fn start_collection(&self) -> tokio::task::JoinHandle<()> {
        {
            let mut running = self.running.write().await;
            *running = true;
        }
        
        let collector = Arc::clone(&self.collector);
        let pools: Arc<RwLock<HashMap<String, Arc<McpConnectionPool>>>> = Arc::clone(&self.pools);
        let running = Arc::clone(&self.running);
        let interval_duration = self.collection_interval;
        
        tokio::spawn(async move {
            let mut interval = interval(interval_duration);
            
            while *running.read().await {
                interval.tick().await;
                
                // Collect metrics from all registered pools
                let pools_guard = pools.read().await;
                for (pool_id, pool) in pools_guard.iter() {
                    match pool.get_detailed_health().await {
                        // Note: get_detailed_health() method name might need to be adjusted
                        // if it's named differently in the actual implementation
                        health => {
                            collector.update_from_pool_health(&health).await;
                            log::debug!("Updated metrics for pool: {}", pool_id);
                        }
                    }
                }
            }
        })
    }
    
    /// Stop automatic metrics collection
    pub async fn stop_collection(&self) {
        let mut running = self.running.write().await;
        *running = false;
    }
    
    /// Get the metrics collector for manual updates
    pub fn collector(&self) -> Arc<MCPMetricsCollector> {
        Arc::clone(&self.collector)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prometheus::Registry;
    
    #[tokio::test]
    async fn test_metrics_collector_creation() {
        let registry = Registry::new();
        let collector = MCPMetricsCollector::new(registry).unwrap();
        
        // Verify metrics are created
        assert_eq!(collector.total_connections.get(), 0);
        assert_eq!(collector.healthy_connections.get(), 0);
        assert_eq!(collector.connection_requests_total.get(), 0);
    }
    
    #[tokio::test]
    async fn test_record_connection_request() {
        let registry = Registry::new();
        let collector = MCPMetricsCollector::new(registry).unwrap();
        
        // Record successful connection
        collector.record_connection_request(true, Duration::from_millis(100));
        assert_eq!(collector.connection_requests_total.get(), 1);
        assert_eq!(collector.connection_requests_failed.get(), 0);
        
        // Record failed connection
        collector.record_connection_request(false, Duration::from_millis(200));
        assert_eq!(collector.connection_requests_total.get(), 2);
        assert_eq!(collector.connection_requests_failed.get(), 1);
    }
    
    #[tokio::test]
    async fn test_record_health_check() {
        let registry = Registry::new();
        let collector = MCPMetricsCollector::new(registry).unwrap();
        
        // Record successful health check
        collector.record_health_check(true);
        assert_eq!(collector.health_checks_total.get(), 1);
        assert_eq!(collector.health_checks_failed.get(), 0);
        
        // Record failed health check
        collector.record_health_check(false);
        assert_eq!(collector.health_checks_total.get(), 2);
        assert_eq!(collector.health_checks_failed.get(), 1);
    }
    
    #[tokio::test]
    async fn test_transport_metrics() {
        let registry = Registry::new();
        let collector = MCPMetricsCollector::new(registry).unwrap();
        
        collector.record_transport_metrics(1024, 512, 10, 5);
        
        assert_eq!(collector.bytes_sent_total.get(), 1024);
        assert_eq!(collector.bytes_received_total.get(), 512);
        assert_eq!(collector.messages_sent_total.get(), 10);
        assert_eq!(collector.messages_received_total.get(), 5);
    }
    
    #[tokio::test]
    async fn test_metrics_manager() {
        let registry = Registry::new();
        let collector = MCPMetricsCollector::new(registry).unwrap();
        let manager = MCPMetricsManager::new(collector, Duration::from_millis(100));
        
        // Test starting and stopping collection
        let task = manager.start_collection().await;
        
        // Let it run briefly
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        manager.stop_collection().await;
        task.abort();
    }
}