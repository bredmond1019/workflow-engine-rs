use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::time::Duration;

use workflow_engine_core::error::WorkflowError;
use crate::connection_pool::ConnectionConfig;
use crate::transport::TransportType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    pub enabled: bool,
    pub client_name: String,
    pub client_version: String,
    pub connection_pool: ConnectionConfig,
    pub servers: HashMap<String, McpServerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub enabled: bool,
    pub transport: TransportType,
    pub auto_connect: bool,
    pub retry_on_failure: bool,
}

impl McpConfig {
    pub fn from_env() -> Result<Self, WorkflowError> {
        let enabled = Self::get_enabled_from_env();
        let client_name = Self::get_client_name_from_env();
        let client_version = Self::get_client_version_from_env();
        let connection_pool = Self::get_connection_pool_from_env();
        let servers = Self::load_servers_from_env()?;

        Ok(McpConfig {
            enabled,
            client_name,
            client_version,
            connection_pool,
            servers,
        })
    }

    fn get_enabled_from_env() -> bool {
        env::var("MCP_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false)
    }

    fn get_client_name_from_env() -> String {
        env::var("MCP_CLIENT_NAME").unwrap_or_else(|_| "ai-workflow-system".to_string())
    }

    fn get_client_version_from_env() -> String {
        env::var("MCP_CLIENT_VERSION").unwrap_or_else(|_| "1.0.0".to_string())
    }

    fn get_connection_pool_from_env() -> ConnectionConfig {
        ConnectionConfig {
            max_connections_per_server: Self::get_env_var_or_default(
                "MCP_MAX_CONNECTIONS_PER_SERVER",
                "5",
                5,
            ),
            connection_timeout: Duration::from_secs(Self::get_env_var_or_default(
                "MCP_CONNECTION_TIMEOUT_SECONDS",
                "30",
                30,
            )),
            idle_timeout: Duration::from_secs(Self::get_env_var_or_default(
                "MCP_IDLE_TIMEOUT_SECONDS",
                "300",
                300,
            )),
            retry_attempts: Self::get_env_var_or_default("MCP_RETRY_ATTEMPTS", "3", 3),
            retry_delay: Duration::from_millis(Self::get_env_var_or_default(
                "MCP_RETRY_DELAY_MS",
                "1000",
                1000,
            )),
            health_check_interval: Duration::from_secs(Self::get_env_var_or_default(
                "MCP_HEALTH_CHECK_INTERVAL_SECONDS",
                "60",
                60,
            )),
            enable_load_balancing: Self::get_env_var_or_default("MCP_ENABLE_LOAD_BALANCING", "true", true),
            load_balancing_strategy: crate::connection_pool::LoadBalancingStrategy::HealthBased,
            circuit_breaker: workflow_engine_core::error::circuit_breaker::CircuitBreakerConfig::default(),
            health_monitoring: crate::health::HealthConfig::default(),
            enable_auto_reconnect: Self::get_env_var_or_default("MCP_ENABLE_AUTO_RECONNECT", "true", true),
            backoff_config: crate::connection_pool::BackoffConfig::default(),
        }
    }

    fn get_env_var_or_default<T: std::str::FromStr + std::default::Default>(
        key: &str,
        default: &str,
        fallback: T,
    ) -> T
    where
        T::Err: std::fmt::Debug,
    {
        env::var(key)
            .unwrap_or_else(|_| default.to_string())
            .parse()
            .unwrap_or(fallback)
    }

    fn load_servers_from_env() -> Result<HashMap<String, McpServerConfig>, WorkflowError> {
        let mut servers = HashMap::new();

        Self::load_customer_support_server(&mut servers)?;
        Self::load_external_servers(&mut servers)?;

        Ok(servers)
    }

    fn load_customer_support_server(
        servers: &mut HashMap<String, McpServerConfig>,
    ) -> Result<(), WorkflowError> {
        if !Self::get_env_var_or_default("MCP_CUSTOMER_SUPPORT_ENABLED", "false", false) {
            return Ok(());
        }

        let transport = Self::create_customer_support_transport()?;

        servers.insert(
            "customer-support".to_string(),
            McpServerConfig {
                name: "customer-support".to_string(),
                enabled: true,
                transport,
                auto_connect: true,
                retry_on_failure: true,
            },
        );

        Ok(())
    }

    fn create_customer_support_transport() -> Result<TransportType, WorkflowError> {
        let transport_type =
            env::var("MCP_CUSTOMER_SUPPORT_TRANSPORT").unwrap_or_else(|_| "stdio".to_string());

        match transport_type.as_str() {
            "stdio" => {
                let command = env::var("MCP_CUSTOMER_SUPPORT_COMMAND")
                    .unwrap_or_else(|_| "python".to_string());
                let args_str = env::var("MCP_CUSTOMER_SUPPORT_ARGS")
                    .unwrap_or_else(|_| "scripts/customer_support_server.py".to_string());
                let args: Vec<String> =
                    args_str.split_whitespace().map(|s| s.to_string()).collect();

                Ok(TransportType::Stdio { 
                    command, 
                    args,
                    auto_restart: true,
                    max_restarts: 3,
                })
            }
            "websocket" => {
                let url = env::var("MCP_CUSTOMER_SUPPORT_URI")
                    .unwrap_or_else(|_| "ws://localhost:8080/mcp".to_string());
                Ok(TransportType::WebSocket { 
                    url,
                    heartbeat_interval: Some(std::time::Duration::from_secs(30)),
                    reconnect_config: crate::transport::ReconnectConfig::default(),
                })
            }
            _ => Err(WorkflowError::MCPError {
                message: "Invalid transport type for customer support server".to_string(),
                server_name: "customer-support".to_string(),
                operation: "load_transport".to_string(),
                source: None,
            }),
        }
    }

    fn load_external_servers(
        servers: &mut HashMap<String, McpServerConfig>,
    ) -> Result<(), WorkflowError> {
        let mut server_index = 1;

        while let Ok(name) = env::var(format!("MCP_EXTERNAL_SERVER_{}_NAME", server_index)) {
            if Self::get_env_var_or_default(
                &format!("MCP_EXTERNAL_SERVER_{}_ENABLED", server_index),
                "false",
                false,
            ) {
                let server_config = Self::create_external_server_config(server_index, &name)?;
                servers.insert(name, server_config);
            }
            server_index += 1;
        }

        Ok(())
    }

    fn create_external_server_config(
        server_index: u32,
        name: &str,
    ) -> Result<McpServerConfig, WorkflowError> {
        let uri_key = format!("MCP_EXTERNAL_SERVER_{}_URI", server_index);
        let transport_key = format!("MCP_EXTERNAL_SERVER_{}_TRANSPORT", server_index);

        let uri = env::var(&uri_key).map_err(|_| WorkflowError::MCPError {
            message: format!("Missing URI for external server {}", name),
            server_name: name.to_string(),
            operation: "load_external_server".to_string(),
            source: None,
        })?;

        let transport_str = env::var(&transport_key).unwrap_or_else(|_| "websocket".to_string());
        let transport =
            Self::create_transport_for_external_server(server_index, &transport_str, uri)?;

        Ok(McpServerConfig {
            name: name.to_string(),
            enabled: true,
            transport,
            auto_connect: true,
            retry_on_failure: true,
        })
    }

    fn create_transport_for_external_server(
        server_index: u32,
        transport_str: &str,
        uri: String,
    ) -> Result<TransportType, WorkflowError> {
        match transport_str {
            "websocket" => Ok(TransportType::WebSocket { 
                url: uri,
                heartbeat_interval: Some(std::time::Duration::from_secs(30)),
                reconnect_config: crate::transport::ReconnectConfig::default(),
            }),
            "stdio" => {
                let command_key = format!("MCP_EXTERNAL_SERVER_{}_COMMAND", server_index);
                let args_key = format!("MCP_EXTERNAL_SERVER_{}_ARGS", server_index);

                let command = env::var(&command_key).unwrap_or_else(|_| "node".to_string());
                let args_str = env::var(&args_key).unwrap_or_else(|_| "".to_string());
                let args: Vec<String> = if args_str.is_empty() {
                    vec![]
                } else {
                    args_str.split_whitespace().map(|s| s.to_string()).collect()
                };

                Ok(TransportType::Stdio { 
                    command, 
                    args,
                    auto_restart: true,
                    max_restarts: 3,
                })
            }
            "http" => Ok(TransportType::Http { 
                base_url: uri,
                pool_config: crate::transport::HttpPoolConfig::default(),
            }),
            _ => Err(WorkflowError::MCPError {
                message: format!(
                    "Invalid transport type '{}' for server {}",
                    transport_str, server_index
                ),
                server_name: format!("external-server-{}", server_index),
                operation: "create_transport".to_string(),
                source: None,
            }),
        }
    }

    pub fn get_server_config(&self, server_name: &str) -> Option<&McpServerConfig> {
        self.servers.get(server_name)
    }

    pub fn get_enabled_servers(&self) -> Vec<&McpServerConfig> {
        self.servers
            .values()
            .filter(|config| config.enabled)
            .collect()
    }

    pub fn is_server_enabled(&self, server_name: &str) -> bool {
        self.servers
            .get(server_name)
            .map(|config| config.enabled)
            .unwrap_or(false)
    }
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            client_name: "ai-workflow-system".to_string(),
            client_version: "1.0.0".to_string(),
            connection_pool: ConnectionConfig::default(),
            servers: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    
    // Helper functions for safe environment variable manipulation in tests
    fn with_env_vars<F, R>(vars: Vec<(&str, &str)>, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        // Save original values
        let mut original_values = Vec::new();
        for (key, _) in &vars {
            original_values.push((key.to_string(), std::env::var(key).ok()));
        }
        
        // Set new values
        for (key, value) in &vars {
            std::env::set_var(key, value);
        }
        
        // Run the test function
        let result = f();
        
        // Restore original values
        for (key, original) in original_values {
            match original {
                Some(value) => std::env::set_var(&key, value),
                None => std::env::remove_var(&key),
            }
        }
        
        result
    }
    
    fn without_env_vars<F, R>(vars: Vec<&str>, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        // Save original values
        let mut original_values = Vec::new();
        for key in &vars {
            original_values.push((key.to_string(), std::env::var(key).ok()));
        }
        
        // Remove variables
        for key in &vars {
            std::env::remove_var(key);
        }
        
        // Run the test function
        let result = f();
        
        // Restore original values
        for (key, original) in original_values {
            if let Some(value) = original {
                std::env::set_var(&key, value);
            }
        }
        
        result
    }

    #[test]
    #[serial]
    fn test_mcp_config_default() {
        let config = McpConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.client_name, "ai-workflow-system");
        assert_eq!(config.client_version, "1.0.0");
        assert!(config.servers.is_empty());
    }

    #[test]
    #[serial]
    fn test_mcp_config_from_env_disabled() {
        // Clear all MCP environment variables
        without_env_vars(vec![
            "MCP_ENABLED",
            "MCP_CLIENT_NAME",
            "MCP_CLIENT_VERSION",
            "MCP_CUSTOMER_SUPPORT_ENABLED",
            "MCP_EXTERNAL_SERVER_1_NAME",
            "MCP_EXTERNAL_SERVER_1_ENABLED",
            "MCP_EXTERNAL_SERVER_1_URI",
            "MCP_EXTERNAL_SERVER_1_TRANSPORT",
            "MCP_EXTERNAL_SERVER_2_NAME",
            "MCP_EXTERNAL_SERVER_2_ENABLED",
            "MCP_EXTERNAL_SERVER_2_URI",
            "MCP_EXTERNAL_SERVER_2_TRANSPORT",
            "MCP_EXTERNAL_SERVER_3_NAME",
            "MCP_EXTERNAL_SERVER_3_ENABLED",
            "MCP_EXTERNAL_SERVER_3_URI",
            "MCP_EXTERNAL_SERVER_3_TRANSPORT",
        ], || {
            let config = McpConfig::from_env().unwrap();
            assert!(!config.enabled);
        });
    }

    #[test]
    #[serial]
    fn test_mcp_config_from_env_enabled() {
        with_env_vars(vec![
            ("MCP_ENABLED", "true"),
            ("MCP_CLIENT_NAME", "test-client"),
            ("MCP_CLIENT_VERSION", "2.0.0"),
        ], || {
            let config = McpConfig::from_env().unwrap();
            assert!(config.enabled);
            assert_eq!(config.client_name, "test-client");
            assert_eq!(config.client_version, "2.0.0");
        });
    }

    #[test]
    #[serial]
    fn test_customer_support_server_config() {
        // Clean up environment variables first
        without_env_vars(vec![
            "MCP_CUSTOMER_SUPPORT_ENABLED",
            "MCP_CUSTOMER_SUPPORT_TRANSPORT",
            "MCP_CUSTOMER_SUPPORT_COMMAND",
            "MCP_CUSTOMER_SUPPORT_ARGS",
            "MCP_EXTERNAL_SERVER_1_NAME",
            "MCP_EXTERNAL_SERVER_1_ENABLED",
            "MCP_EXTERNAL_SERVER_1_URI",
            "MCP_EXTERNAL_SERVER_1_TRANSPORT",
            "MCP_EXTERNAL_SERVER_2_NAME",
            "MCP_EXTERNAL_SERVER_2_ENABLED",
            "MCP_EXTERNAL_SERVER_2_URI",
            "MCP_EXTERNAL_SERVER_2_TRANSPORT",
        ], || {
            with_env_vars(vec![
                ("MCP_CUSTOMER_SUPPORT_ENABLED", "true"),
                ("MCP_CUSTOMER_SUPPORT_TRANSPORT", "stdio"),
                ("MCP_CUSTOMER_SUPPORT_COMMAND", "python3"),
                ("MCP_CUSTOMER_SUPPORT_ARGS", "scripts/server.py --port 8080"),
            ], || {
                let config = McpConfig::from_env().unwrap();

                assert!(config.is_server_enabled("customer-support"));
                let server_config = config.get_server_config("customer-support").unwrap();
                assert_eq!(server_config.name, "customer-support");
                assert!(server_config.enabled);

                match &server_config.transport {
                    TransportType::Stdio { command, args, .. } => {
                        assert_eq!(command, "python3");
                        assert_eq!(args, &vec!["scripts/server.py", "--port", "8080"]);
                    }
                    _ => panic!("Expected Stdio transport"),
                }
            });
        });
    }

    #[test]
    #[serial]
    fn test_external_server_config() {
        // Cleanup any existing environment variables first
        without_env_vars(vec![
            "MCP_EXTERNAL_SERVER_1_NAME",
            "MCP_EXTERNAL_SERVER_1_ENABLED",
            "MCP_EXTERNAL_SERVER_1_URI",
            "MCP_EXTERNAL_SERVER_1_TRANSPORT",
            "MCP_CUSTOMER_SUPPORT_ENABLED",
        ], || {
            with_env_vars(vec![
                ("MCP_EXTERNAL_SERVER_1_NAME", "test-server"),
                ("MCP_EXTERNAL_SERVER_1_ENABLED", "true"),
                ("MCP_EXTERNAL_SERVER_1_URI", "ws://localhost:9090/mcp"),
                ("MCP_EXTERNAL_SERVER_1_TRANSPORT", "websocket"),
            ], || {
                let config = McpConfig::from_env().unwrap();

                assert!(config.is_server_enabled("test-server"));
                let server_config = config.get_server_config("test-server").unwrap();
                assert_eq!(server_config.name, "test-server");

                match &server_config.transport {
                    TransportType::WebSocket { url, .. } => {
                        assert_eq!(url, "ws://localhost:9090/mcp");
                    }
                    _ => panic!("Expected WebSocket transport"),
                }
            });
        });
    }

    #[test]
    #[serial]
    fn test_get_enabled_servers() {
        // Cleanup any existing environment variables first
        without_env_vars(vec![
            "MCP_EXTERNAL_SERVER_1_NAME",
            "MCP_EXTERNAL_SERVER_1_ENABLED",
            "MCP_EXTERNAL_SERVER_1_URI",
            "MCP_EXTERNAL_SERVER_1_TRANSPORT",
            "MCP_EXTERNAL_SERVER_2_NAME",
            "MCP_EXTERNAL_SERVER_2_ENABLED",
            "MCP_EXTERNAL_SERVER_2_URI",
            "MCP_EXTERNAL_SERVER_2_TRANSPORT",
            "MCP_CUSTOMER_SUPPORT_ENABLED",
        ], || {
            with_env_vars(vec![
                ("MCP_EXTERNAL_SERVER_1_NAME", "server1"),
                ("MCP_EXTERNAL_SERVER_1_ENABLED", "true"),
                ("MCP_EXTERNAL_SERVER_1_URI", "ws://localhost:8080"),
                ("MCP_EXTERNAL_SERVER_1_TRANSPORT", "websocket"),
                ("MCP_EXTERNAL_SERVER_2_NAME", "server2"),
                ("MCP_EXTERNAL_SERVER_2_ENABLED", "false"),
                ("MCP_EXTERNAL_SERVER_2_URI", "ws://localhost:8081"),
                ("MCP_EXTERNAL_SERVER_2_TRANSPORT", "websocket"),
            ], || {
                let config = McpConfig::from_env().unwrap();
                let enabled_servers = config.get_enabled_servers();

                assert_eq!(enabled_servers.len(), 1);
                assert_eq!(enabled_servers[0].name, "server1");
            });
        });
    }
}
