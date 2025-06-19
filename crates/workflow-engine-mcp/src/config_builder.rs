//! Builder pattern for MCPConfig
//!
//! This module provides a fluent builder interface for creating
//! MCPConfig instances with validation and sensible defaults.

use std::collections::HashMap;
use std::time::Duration;

use crate::config::{MCPConfig, MCPServerConfig};
use crate::connection_pool::{ConnectionConfig, LoadBalancingStrategy, BackoffConfig};
use crate::transport::{TransportType, ReconnectConfig, HttpPoolConfig};
use crate::health::HealthConfig;
use workflow_engine_core::error::{WorkflowError, circuit_breaker::CircuitBreakerConfig};

/// Builder for creating MCPConfig with fluent interface
pub struct MCPConfigBuilder {
    enabled: bool,
    client_name: String,
    client_version: String,
    connection_pool: ConnectionConfigBuilder,
    servers: HashMap<String, MCPServerConfig>,
}

impl MCPConfigBuilder {
    /// Create a new MCPConfig builder
    pub fn new() -> Self {
        Self {
            enabled: true,
            client_name: "ai-workflow-system".to_string(),
            client_version: "1.0.0".to_string(),
            connection_pool: ConnectionConfigBuilder::new(),
            servers: HashMap::new(),
        }
    }

    /// Set whether MCP is enabled
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set the client name
    pub fn client_name(mut self, name: impl Into<String>) -> Self {
        self.client_name = name.into();
        self
    }

    /// Set the client version
    pub fn client_version(mut self, version: impl Into<String>) -> Self {
        self.client_version = version.into();
        self
    }

    /// Configure the connection pool
    pub fn connection_pool<F>(mut self, f: F) -> Self
    where
        F: FnOnce(ConnectionConfigBuilder) -> ConnectionConfigBuilder,
    {
        self.connection_pool = f(self.connection_pool);
        self
    }

    /// Add a server configuration
    pub fn add_server(mut self, name: impl Into<String>, config: MCPServerConfig) -> Self {
        self.servers.insert(name.into(), config);
        self
    }

    /// Add a WebSocket server
    pub fn add_websocket_server(
        mut self,
        name: impl Into<String>,
        url: impl Into<String>,
    ) -> Self {
        let server_name = name.into();
        let server_config = MCPServerConfig {
            name: server_name.clone(),
            enabled: true,
            transport: TransportType::WebSocket {
                url: url.into(),
                heartbeat_interval: Some(Duration::from_secs(30)),
                reconnect_config: ReconnectConfig::default(),
            },
            auto_connect: true,
            retry_on_failure: true,
        };
        self.servers.insert(server_name, server_config);
        self
    }

    /// Add an HTTP server
    pub fn add_http_server(
        mut self,
        name: impl Into<String>,
        base_url: impl Into<String>,
    ) -> Self {
        let server_name = name.into();
        let server_config = MCPServerConfig {
            name: server_name.clone(),
            enabled: true,
            transport: TransportType::Http {
                base_url: base_url.into(),
                pool_config: HttpPoolConfig::default(),
            },
            auto_connect: true,
            retry_on_failure: true,
        };
        self.servers.insert(server_name, server_config);
        self
    }

    /// Add a stdio server
    pub fn add_stdio_server(
        mut self,
        name: impl Into<String>,
        command: impl Into<String>,
        args: Vec<String>,
    ) -> Self {
        let server_name = name.into();
        let server_config = MCPServerConfig {
            name: server_name.clone(),
            enabled: true,
            transport: TransportType::Stdio {
                command: command.into(),
                args,
                auto_restart: true,
                max_restarts: 3,
            },
            auto_connect: true,
            retry_on_failure: true,
        };
        self.servers.insert(server_name, server_config);
        self
    }

    /// Build the MCPConfig with validation
    pub fn build(self) -> Result<MCPConfig, WorkflowError> {
        // Validate client name
        if self.client_name.is_empty() {
            return Err(WorkflowError::ConfigurationError(
                "Client name cannot be empty".to_string()
            ));
        }

        // Validate client version
        if self.client_version.is_empty() {
            return Err(WorkflowError::ConfigurationError(
                "Client version cannot be empty".to_string()
            ));
        }

        // Build connection config
        let connection_pool = self.connection_pool.build()?;

        Ok(MCPConfig {
            enabled: self.enabled,
            client_name: self.client_name,
            client_version: self.client_version,
            connection_pool,
            servers: self.servers,
        })
    }
}

impl Default for MCPConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for ConnectionConfig
pub struct ConnectionConfigBuilder {
    max_connections_per_server: usize,
    connection_timeout: Duration,
    idle_timeout: Duration,
    retry_attempts: u32,
    retry_delay: Duration,
    health_check_interval: Duration,
    enable_load_balancing: bool,
    load_balancing_strategy: LoadBalancingStrategy,
    circuit_breaker: Option<CircuitBreakerConfig>,
    health_monitoring: Option<HealthConfig>,
    enable_auto_reconnect: bool,
    backoff_config: Option<BackoffConfig>,
}

impl ConnectionConfigBuilder {
    /// Create a new ConnectionConfig builder
    pub fn new() -> Self {
        Self {
            max_connections_per_server: 5,
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(300),
            retry_attempts: 3,
            retry_delay: Duration::from_millis(1000),
            health_check_interval: Duration::from_secs(60),
            enable_load_balancing: true,
            load_balancing_strategy: LoadBalancingStrategy::HealthBased,
            circuit_breaker: None,
            health_monitoring: None,
            enable_auto_reconnect: true,
            backoff_config: None,
        }
    }

    /// Set maximum connections per server
    pub fn max_connections_per_server(mut self, max: usize) -> Self {
        self.max_connections_per_server = max;
        self
    }

    /// Set connection timeout
    pub fn connection_timeout(mut self, timeout: Duration) -> Self {
        self.connection_timeout = timeout;
        self
    }

    /// Set idle timeout
    pub fn idle_timeout(mut self, timeout: Duration) -> Self {
        self.idle_timeout = timeout;
        self
    }

    /// Set retry configuration
    pub fn retry(mut self, attempts: u32, delay: Duration) -> Self {
        self.retry_attempts = attempts;
        self.retry_delay = delay;
        self
    }

    /// Set health check interval
    pub fn health_check_interval(mut self, interval: Duration) -> Self {
        self.health_check_interval = interval;
        self
    }

    /// Enable or disable load balancing
    pub fn load_balancing(mut self, enabled: bool, strategy: LoadBalancingStrategy) -> Self {
        self.enable_load_balancing = enabled;
        self.load_balancing_strategy = strategy;
        self
    }

    /// Set circuit breaker configuration
    pub fn circuit_breaker(mut self, config: CircuitBreakerConfig) -> Self {
        self.circuit_breaker = Some(config);
        self
    }

    /// Enable auto-reconnect
    pub fn auto_reconnect(mut self, enabled: bool) -> Self {
        self.enable_auto_reconnect = enabled;
        self
    }

    /// Build the ConnectionConfig with validation
    fn build(self) -> Result<ConnectionConfig, WorkflowError> {
        // Validate connection settings
        if self.max_connections_per_server == 0 {
            return Err(WorkflowError::ConfigurationError(
                "Maximum connections per server must be greater than 0".to_string()
            ));
        }

        if self.retry_attempts > 0 && self.retry_delay.as_millis() == 0 {
            return Err(WorkflowError::ConfigurationError(
                "Retry delay must be greater than 0 when retry is enabled".to_string()
            ));
        }

        Ok(ConnectionConfig {
            max_connections_per_server: self.max_connections_per_server,
            connection_timeout: self.connection_timeout,
            idle_timeout: self.idle_timeout,
            retry_attempts: self.retry_attempts,
            retry_delay: self.retry_delay,
            health_check_interval: self.health_check_interval,
            enable_load_balancing: self.enable_load_balancing,
            load_balancing_strategy: self.load_balancing_strategy,
            circuit_breaker: self.circuit_breaker.unwrap_or_default(),
            health_monitoring: self.health_monitoring.unwrap_or_default(),
            enable_auto_reconnect: self.enable_auto_reconnect,
            backoff_config: self.backoff_config.unwrap_or_default(),
        })
    }
}

impl Default for ConnectionConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait for MCPConfig to provide builder
pub trait MCPConfigExt {
    /// Create a builder for MCPConfig
    fn builder() -> MCPConfigBuilder;
}

impl MCPConfigExt for MCPConfig {
    fn builder() -> MCPConfigBuilder {
        MCPConfigBuilder::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_mcp_config() {
        let config = MCPConfigBuilder::new()
            .client_name("test-client")
            .client_version("2.0.0")
            .build()
            .unwrap();

        assert!(config.enabled);
        assert_eq!(config.client_name, "test-client");
        assert_eq!(config.client_version, "2.0.0");
        assert!(config.servers.is_empty());
    }

    #[test]
    fn test_mcp_config_with_servers() {
        let config = MCPConfigBuilder::new()
            .add_websocket_server("test-ws", "ws://localhost:8080")
            .add_http_server("test-http", "http://localhost:8081")
            .add_stdio_server("test-stdio", "python", vec!["server.py".to_string()])
            .build()
            .unwrap();

        assert_eq!(config.servers.len(), 3);
        assert!(config.servers.contains_key("test-ws"));
        assert!(config.servers.contains_key("test-http"));
        assert!(config.servers.contains_key("test-stdio"));
    }

    #[test]
    fn test_connection_pool_config() {
        let config = MCPConfigBuilder::new()
            .connection_pool(|pool| {
                pool.max_connections_per_server(10)
                    .connection_timeout(Duration::from_secs(60))
                    .retry(5, Duration::from_millis(500))
                    .load_balancing(true, LoadBalancingStrategy::RoundRobin)
            })
            .build()
            .unwrap();

        assert_eq!(config.connection_pool.max_connections_per_server, 10);
        assert_eq!(config.connection_pool.connection_timeout, Duration::from_secs(60));
        assert_eq!(config.connection_pool.retry_attempts, 5);
    }

    #[test]
    fn test_validation_errors() {
        // Empty client name
        let result = MCPConfigBuilder::new()
            .client_name("")
            .build();
        assert!(result.is_err());

        // Invalid connection pool config
        let result = MCPConfigBuilder::new()
            .connection_pool(|pool| pool.max_connections_per_server(0))
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_disabled_mcp() {
        let config = MCPConfigBuilder::new()
            .enabled(false)
            .add_websocket_server("test", "ws://localhost:8080")
            .build()
            .unwrap();

        assert!(!config.enabled);
        assert_eq!(config.servers.len(), 1); // Server is added but MCP is disabled
    }
}