//! Builder pattern for McpConfig
//!
//! This module provides a fluent builder interface for creating
//! McpConfig instances with validation and sensible defaults.

use std::collections::HashMap;
use std::time::Duration;

use crate::config::{McpConfig, McpServerConfig};
use crate::connection_pool::{ConnectionConfig, LoadBalancingStrategy, BackoffConfig};
use crate::transport::{TransportType, ReconnectConfig, HttpPoolConfig};
use crate::health::HealthConfig;
use workflow_engine_core::error::{WorkflowError, circuit_breaker::CircuitBreakerConfig};

/// Builder for creating McpConfig with fluent interface
pub struct McpConfigBuilder {
    enabled: bool,
    client_name: String,
    client_version: String,
    connection_pool: ConnectionConfigBuilder,
    servers: HashMap<String, McpServerConfig>,
}

impl McpConfigBuilder {
    /// Create a new McpConfig builder
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
    pub fn add_server(mut self, name: impl Into<String>, config: McpServerConfig) -> Self {
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
        let server_config = McpServerConfig {
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

    /// Add a WebSocket server with custom configuration
    pub fn add_websocket_server_with_config(
        mut self,
        name: impl Into<String>,
        url: impl Into<String>,
        heartbeat_interval: Option<Duration>,
        reconnect_config: ReconnectConfig,
    ) -> Self {
        let server_name = name.into();
        let server_config = McpServerConfig {
            name: server_name.clone(),
            enabled: true,
            transport: TransportType::WebSocket {
                url: url.into(),
                heartbeat_interval,
                reconnect_config,
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
        let server_config = McpServerConfig {
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
        let server_config = McpServerConfig {
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

    /// Add a stdio server with custom configuration
    pub fn add_stdio_server_with_config(
        mut self,
        name: impl Into<String>,
        command: impl Into<String>,
        args: Vec<String>,
        auto_restart: bool,
        max_restarts: u32,
    ) -> Self {
        let server_name = name.into();
        let server_config = McpServerConfig {
            name: server_name.clone(),
            enabled: true,
            transport: TransportType::Stdio {
                command: command.into(),
                args,
                auto_restart,
                max_restarts,
            },
            auto_connect: true,
            retry_on_failure: true,
        };
        self.servers.insert(server_name, server_config);
        self
    }

    /// Add server with custom enablement and connection settings
    pub fn add_server_with_settings(
        mut self,
        name: impl Into<String>,
        config: McpServerConfig,
        enabled: bool,
        auto_connect: bool,
        retry_on_failure: bool,
    ) -> Self {
        let server_name = name.into();
        let mut server_config = config;
        server_config.name = server_name.clone();
        server_config.enabled = enabled;
        server_config.auto_connect = auto_connect;
        server_config.retry_on_failure = retry_on_failure;
        self.servers.insert(server_name, server_config);
        self
    }

    /// Remove a server configuration
    pub fn remove_server(mut self, name: impl Into<String>) -> Self {
        self.servers.remove(&name.into());
        self
    }

    /// Disable a server
    pub fn disable_server(mut self, name: impl Into<String>) -> Self {
        let server_name = name.into();
        if let Some(config) = self.servers.get_mut(&server_name) {
            config.enabled = false;
        }
        self
    }

    /// Enable a server
    pub fn enable_server(mut self, name: impl Into<String>) -> Self {
        let server_name = name.into();
        if let Some(config) = self.servers.get_mut(&server_name) {
            config.enabled = true;
        }
        self
    }

    /// Set auto-connect for a server
    pub fn set_server_auto_connect(mut self, name: impl Into<String>, auto_connect: bool) -> Self {
        let server_name = name.into();
        if let Some(config) = self.servers.get_mut(&server_name) {
            config.auto_connect = auto_connect;
        }
        self
    }

    /// Build the McpConfig with validation
    pub fn build(self) -> Result<McpConfig, WorkflowError> {
        // Validate client name
        if self.client_name.is_empty() {
            return Err(WorkflowError::configuration_error(
                "Client name cannot be empty",
                "client_name",
                "builder",
                "non-empty string",
                Some(self.client_name.clone())
            ));
        }

        // Validate client version
        if self.client_version.is_empty() {
            return Err(WorkflowError::configuration_error(
                "Client version cannot be empty",
                "client_version",
                "builder",
                "non-empty string",
                Some(self.client_version.clone())
            ));
        }

        // Validate client name format (basic validation)
        if !self.client_name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(WorkflowError::configuration_error(
                "Client name can only contain alphanumeric characters, hyphens, and underscores",
                "client_name",
                "builder",
                "alphanumeric characters, hyphens, and underscores only",
                Some(self.client_name.clone())
            ));
        }

        // Validate servers if MCP is enabled
        if self.enabled {
            for (name, server_config) in &self.servers {
                // Validate server name matches config name
                if name != &server_config.name {
                    return Err(WorkflowError::configuration_error(
                        format!("Server name mismatch: key '{}' vs config name '{}'", name, server_config.name),
                        format!("servers.{}", name),
                        "builder",
                        format!("name matching key '{}'", name),
                        Some(server_config.name.clone())
                    ));
                }

                // Validate transport configurations
                match &server_config.transport {
                    TransportType::WebSocket { url, .. } => {
                        if url.is_empty() {
                            return Err(WorkflowError::configuration_error(
                                format!("WebSocket URL cannot be empty for server '{}'", name),
                                format!("servers.{}.transport.url", name),
                                "builder",
                                "non-empty WebSocket URL",
                                Some(url.clone())
                            ));
                        }
                        if !url.starts_with("ws://") && !url.starts_with("wss://") {
                            return Err(WorkflowError::configuration_error(
                                format!("WebSocket URL must start with 'ws://' or 'wss://' for server '{}'", name),
                                format!("servers.{}.transport.url", name),
                                "builder",
                                "URL starting with ws:// or wss://",
                                Some(url.clone())
                            ));
                        }
                    }
                    TransportType::Http { base_url, .. } => {
                        if base_url.is_empty() {
                            return Err(WorkflowError::configuration_error(
                                format!("HTTP base URL cannot be empty for server '{}'", name),
                                format!("servers.{}.transport.base_url", name),
                                "builder",
                                "non-empty HTTP URL",
                                Some(base_url.clone())
                            ));
                        }
                        if !base_url.starts_with("http://") && !base_url.starts_with("https://") {
                            return Err(WorkflowError::configuration_error(
                                format!("HTTP base URL must start with 'http://' or 'https://' for server '{}'", name),
                                format!("servers.{}.transport.base_url", name),
                                "builder",
                                "URL starting with http:// or https://",
                                Some(base_url.clone())
                            ));
                        }
                    }
                    TransportType::Stdio { command, .. } => {
                        if command.is_empty() {
                            return Err(WorkflowError::configuration_error(
                                format!("Stdio command cannot be empty for server '{}'", name),
                                format!("servers.{}.transport.command", name),
                                "builder",
                                "non-empty command",
                                Some(command.clone())
                            ));
                        }
                    }
                }
            }

            // Ensure at least one enabled server if MCP is enabled
            let enabled_servers: Vec<_> = self.servers.values()
                .filter(|config| config.enabled)
                .collect();
            
            if enabled_servers.is_empty() {
                return Err(WorkflowError::configuration_error(
                "At least one server must be enabled when MCP is enabled",
                "servers",
                "builder",
                "at least one enabled server",
                Some("no enabled servers".to_string())
            ));
            }
        }

        // Build connection config
        let connection_pool = self.connection_pool.build()?;

        Ok(McpConfig {
            enabled: self.enabled,
            client_name: self.client_name,
            client_version: self.client_version,
            connection_pool,
            servers: self.servers,
        })
    }
}

impl Default for McpConfigBuilder {
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
            return Err(WorkflowError::configuration_error(
                "Maximum connections per server must be greater than 0",
                "max_connections_per_server",
                "builder",
                "positive integer",
                Some(self.max_connections_per_server.to_string())
            ));
        }

        if self.retry_attempts > 0 && self.retry_delay.as_millis() == 0 {
            return Err(WorkflowError::configuration_error(
                "Retry delay must be greater than 0 when retry is enabled",
                "retry_delay",
                "builder",
                "positive duration",
                Some(format!("{:?}", self.retry_delay))
            ));
        }

        Ok(ConnectionConfig {
            max_connections_per_server: self.max_connections_per_server,
            connection_timeout: self.connection_timeout,
            idle_timeout: self.idle_timeout,
            retry_attempts: self.retry_attempts as usize,
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

/// Extension trait for McpConfig to provide builder
pub trait McpConfigExt {
    /// Create a builder for McpConfig
    fn builder() -> McpConfigBuilder;
}

impl McpConfigExt for McpConfig {
    fn builder() -> McpConfigBuilder {
        McpConfigBuilder::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_mcp_config() {
        let config = McpConfigBuilder::new()
            .enabled(false)  // Disable MCP when no servers are configured
            .client_name("test-client")
            .client_version("2.0.0")
            .build()
            .unwrap();

        assert!(!config.enabled);
        assert_eq!(config.client_name, "test-client");
        assert_eq!(config.client_version, "2.0.0");
        assert!(config.servers.is_empty());
    }

    #[test]
    fn test_mcp_config_with_servers() {
        let config = McpConfigBuilder::new()
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
        let config = McpConfigBuilder::new()
            .enabled(false)  // Disable MCP when no servers are configured
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
        let result = McpConfigBuilder::new()
            .client_name("")
            .build();
        assert!(result.is_err());

        // Invalid connection pool config
        let result = McpConfigBuilder::new()
            .connection_pool(|pool| pool.max_connections_per_server(0))
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_disabled_mcp() {
        let config = McpConfigBuilder::new()
            .enabled(false)
            .add_websocket_server("test", "ws://localhost:8080")
            .build()
            .unwrap();

        assert!(!config.enabled);
        assert_eq!(config.servers.len(), 1); // Server is added but MCP is disabled
    }

    #[test]
    fn test_enhanced_fluent_interface() {
        let config = McpConfigBuilder::new()
            .client_name("enhanced-client")
            .client_version("2.1.0")
            .add_websocket_server_with_config(
                "ws-server",
                "wss://secure.example.com/mcp",
                Some(Duration::from_secs(60)),
                ReconnectConfig::default()
            )
            .add_stdio_server_with_config(
                "stdio-server",
                "python3",
                vec!["server.py".to_string(), "--port".to_string(), "8080".to_string()],
                true,
                5
            )
            .disable_server("stdio-server")
            .set_server_auto_connect("ws-server", false)
            .connection_pool(|pool| {
                pool.max_connections_per_server(15)
                    .health_check_interval(Duration::from_secs(45))
                    .auto_reconnect(true)
            })
            .build()
            .unwrap();

        assert_eq!(config.client_name, "enhanced-client");
        assert_eq!(config.client_version, "2.1.0");
        assert_eq!(config.servers.len(), 2);
        
        let ws_server = &config.servers["ws-server"];
        assert!(ws_server.enabled);
        assert!(!ws_server.auto_connect);
        
        let stdio_server = &config.servers["stdio-server"];
        assert!(!stdio_server.enabled);
        
        assert_eq!(config.connection_pool.max_connections_per_server, 15);
    }

    #[test]
    fn test_server_management() {
        let config = McpConfigBuilder::new()
            .add_websocket_server("server1", "ws://localhost:8080")
            .add_http_server("server2", "http://localhost:8081")
            .remove_server("server1")
            .enable_server("server2")
            .build()
            .unwrap();

        assert_eq!(config.servers.len(), 1);
        assert!(config.servers.contains_key("server2"));
        assert!(!config.servers.contains_key("server1"));
    }

    #[test]
    fn test_enhanced_validation_errors() {
        // Invalid client name format
        let result = McpConfigBuilder::new()
            .client_name("invalid client name!")
            .build();
        assert!(result.is_err());

        // Invalid WebSocket URL
        let result = McpConfigBuilder::new()
            .add_websocket_server("test", "invalid-url")
            .build();
        assert!(result.is_err());

        // Invalid HTTP URL
        let result = McpConfigBuilder::new()
            .add_http_server("test", "invalid-url")
            .build();
        assert!(result.is_err());

        // Empty command for stdio
        let result = McpConfigBuilder::new()
            .add_stdio_server("test", "", vec![])
            .build();
        assert!(result.is_err());

        // No enabled servers when MCP is enabled
        let result = McpConfigBuilder::new()
            .enabled(true)
            .add_websocket_server("test", "ws://localhost:8080")
            .disable_server("test")
            .build();
        assert!(result.is_err());
    }
}
