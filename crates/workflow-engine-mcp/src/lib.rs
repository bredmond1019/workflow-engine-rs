//! # Workflow Engine MCP
//! 
//! Model Context Protocol (MCP) integration for the workflow engine.
//! This crate provides:
//! 
//! - MCP protocol implementation
//! - Multiple transport types (HTTP, WebSocket, stdio)
//! - Connection pooling and load balancing
//! - Health monitoring and metrics
//! - Built-in MCP server implementations
//! 
//! ## Features
//! 
//! - `http` - HTTP transport support (enabled by default)
//! - `websocket` - WebSocket transport support (enabled by default) 
//! - `stdio` - Standard I/O transport support
//! - `all` - All transport types
//! 
//! ## Core Concepts
//! 
//! - **Protocol**: Core MCP message types and protocol handling
//! - **Transports**: Different ways to communicate with MCP servers
//! - **Clients**: High-level MCP client implementations
//! - **Servers**: Built-in MCP server implementations
//! - **Connection Pool**: Managed connections with health monitoring
//! 
//! ## Examples
//! 
//! ```rust
//! use workflow_engine_mcp::{
//!     clients::http::HttpMcpClient,
//!     transport::TransportType,
//! };
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = HttpMcpClient::new("http://localhost:8080/mcp")?;
//!     let tools = client.list_tools().await?;
//!     println!("Available tools: {:?}", tools);
//!     Ok(())
//! }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]

// Core MCP modules
pub mod protocol;
pub mod transport;
pub mod clients;
pub mod config;
pub mod config_builder;
pub mod health;
pub mod metrics;
pub mod connection_pool;
pub mod load_balancer;

// MCP server implementations
pub mod server;

// Re-export commonly used types
pub use protocol::{McpMessage, McpRequest, McpResponse, ToolDefinition as McpTool, CallToolResult as McpToolResult};
pub use transport::{TransportType, McpTransport};
pub use clients::McpClient;
pub use config::McpConfig;
pub use connection_pool::{McpConnectionPool as ConnectionPool, PooledConnection};

/// Current version of the MCP integration
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Prelude module for common MCP imports
pub mod prelude {
    pub use crate::{
        McpMessage, McpRequest, McpResponse, 
        TransportType, McpTransport, McpClient,
        McpConfig, ConnectionPool,
        protocol::{ToolDefinition, CallToolResult},
    };
    pub use workflow_engine_core::prelude::*;
}

// TDD Test 1c: MCP timeout handling tests
#[cfg(test)]
mod mcp_timeout_tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::timeout;
    use workflow_engine_core::error::WorkflowError;

    /// Test that connection timeouts are handled gracefully without panics
    #[tokio::test]
    async fn test_connection_timeout_no_panic() {
        let config = connection_pool::ConnectionConfig {
            connection_timeout: Duration::from_millis(100), // Very short timeout
            retry_attempts: 1, // Single attempt to avoid long test duration
            retry_delay: Duration::from_millis(10),
            ..Default::default()
        };

        let pool = connection_pool::McpConnectionPool::new(config);

        // Register server with invalid URL that will timeout
        pool.register_server(
            "timeout-server".to_string(),
            TransportType::WebSocket {
                url: "ws://192.0.2.1:1234/nonexistent".to_string(), // RFC 5737 test address
                heartbeat_interval: None,
                reconnect_config: Default::default(),
            },
            "test-client".to_string(),
            "1.0.0".to_string(),
        ).await;

        // This should fail gracefully with timeout error, not panic with unwrap()
        let result = pool.get_connection("timeout-server").await;
        
        assert!(result.is_err(), "Connection should fail due to timeout");
        
        // Verify it's the right type of error
        match result {
            Err(WorkflowError::MCPConnectionError(details)) => {
                assert!(details.message.contains("timeout") || details.message.contains("Failed to create connection"), 
                    "Error should indicate timeout or connection failure, got: {}", details.message);
            }
            Err(WorkflowError::MCPError(details)) => {
                assert!(details.message.contains("timeout") || details.message.contains("connection"), 
                    "Error should indicate timeout or connection failure, got: {}", details.message);
            }
            Err(other) => panic!("Unexpected error type: {:?}", other),
            Ok(_) => panic!("Connection should not succeed"),
        }
    }

    /// Test cleanup operations handle errors gracefully
    #[tokio::test]
    async fn test_cleanup_expired_connections_error_handling() {
        let pool = connection_pool::McpConnectionPool::new(connection_pool::ConnectionConfig::default());
        
        // This currently uses .unwrap() in tests - should return Result instead
        let result = pool.cleanup_expired_connections().await;
        
        // Should always return Ok for cleanup operations, even if there are issues
        assert!(result.is_ok(), "Cleanup should handle errors gracefully");
        
        let cleaned_count = result.expect("Already verified is_ok");
        assert_eq!(cleaned_count, 0, "No connections to clean initially");
    }

    /// Test invalid server URLs are handled gracefully
    #[tokio::test]
    async fn test_invalid_server_url_handling() {
        let pool = connection_pool::McpConnectionPool::new(connection_pool::ConnectionConfig {
            connection_timeout: Duration::from_millis(100),
            retry_attempts: 1,
            ..Default::default()
        });

        // Test completely invalid URL
        pool.register_server(
            "invalid-url-server".to_string(),
            TransportType::WebSocket {
                url: "not-a-valid-url".to_string(),
                heartbeat_interval: None,
                reconnect_config: Default::default(),
            },
            "test-client".to_string(),
            "1.0.0".to_string(),
        ).await;

        let result = pool.get_connection("invalid-url-server").await;
        assert!(result.is_err(), "Invalid URL should cause connection failure");
    }

    /// Test server unavailable scenarios
    #[tokio::test]
    async fn test_server_unavailable_handling() {
        let pool = connection_pool::McpConnectionPool::new(connection_pool::ConnectionConfig {
            connection_timeout: Duration::from_millis(100),
            retry_attempts: 2,
            retry_delay: Duration::from_millis(10),
            ..Default::default()
        });

        // Use localhost with a port that's unlikely to be in use
        pool.register_server(
            "unavailable-server".to_string(),
            TransportType::WebSocket {
                url: "ws://localhost:65432/mcp".to_string(), // High port unlikely to be used
                heartbeat_interval: None,
                reconnect_config: Default::default(),
            },
            "test-client".to_string(),
            "1.0.0".to_string(),
        ).await;

        let result = pool.get_connection("unavailable-server").await;
        assert!(result.is_err(), "Unavailable server should cause connection failure");
        
        // Verify error type
        match result {
            Err(WorkflowError::MCPConnectionError { .. }) => {
                // Expected error type
            }
            Err(WorkflowError::MCPError { .. }) => {
                // Also acceptable
            }
            Err(other) => panic!("Unexpected error type: {:?}", other),
            Ok(_) => panic!("Connection should not succeed to unavailable server"),
        }
    }

    /// Test MCP protocol message failures
    #[tokio::test]
    async fn test_mcp_protocol_error_handling() {
        let pool = connection_pool::McpConnectionPool::new(connection_pool::ConnectionConfig::default());

        // Test with unregistered server (should fail gracefully)
        let result = pool.get_connection("nonexistent-server").await;
        
        assert!(result.is_err(), "Unregistered server should cause error");
        
        match result {
            Err(WorkflowError::MCPError(details)) => {
                assert_eq!(details.server_name, "nonexistent-server");
                assert_eq!(details.operation, "get_connection");
                assert!(details.message.contains("not registered"));
            }
            Err(other) => panic!("Expected MCPError, got: {:?}", other),
            Ok(_) => panic!("Should not succeed for unregistered server"),
        }
    }

    /// Test timeout with external health checks
    #[tokio::test]
    async fn test_health_check_timeout_handling() {
        let pool = connection_pool::McpConnectionPool::new(connection_pool::ConnectionConfig {
            connection_timeout: Duration::from_millis(50),
            health_check_interval: Duration::from_millis(100),
            ..Default::default()
        });

        // Register a server that will timeout
        pool.register_server(
            "health-timeout-server".to_string(),
            TransportType::WebSocket {
                url: "ws://192.0.2.2:9999/mcp".to_string(), // Another RFC 5737 test address
                heartbeat_interval: Some(Duration::from_millis(30)),
                reconnect_config: Default::default(),
            },
            "test-client".to_string(),
            "1.0.0".to_string(),
        ).await;

        // Health check should handle timeouts gracefully
        let health_result = pool.health_check().await;
        
        assert!(health_result.is_ok(), "Health check should not panic on timeouts");
        
        let health_status = health_result.expect("Already verified is_ok");
        
        // Health check only returns servers that have actual connections in the pool
        // If no connection has been established yet, the server won't appear in health results
        // This tests that health_check handles empty pools gracefully
        assert_eq!(health_status.len(), 0, "Health check should return empty for servers with no connections");
        
        // Try to establish a connection (which should fail) and then check health
        let connection_result = pool.get_connection("health-timeout-server").await;
        assert!(connection_result.is_err(), "Connection should fail due to timeout");
        
        // Now check health again - the server should appear in health status
        let health_result2 = pool.health_check().await;
        assert!(health_result2.is_ok(), "Health check should handle connection failures gracefully");
    }

    /// Test force reconnect with timeout scenarios
    #[tokio::test]
    async fn test_force_reconnect_timeout_handling() {
        let pool = connection_pool::McpConnectionPool::new(connection_pool::ConnectionConfig::default());

        pool.register_server(
            "reconnect-timeout-server".to_string(),
            TransportType::WebSocket {
                url: "ws://192.0.2.3:8080/mcp".to_string(),
                heartbeat_interval: None,
                reconnect_config: Default::default(),
            },
            "test-client".to_string(),
            "1.0.0".to_string(),
        ).await;

        // Force reconnect should handle errors gracefully
        let result = pool.force_reconnect("reconnect-timeout-server").await;
        
        // Force reconnect should succeed in cleanup even if server is unreachable
        assert!(result.is_ok(), "Force reconnect should handle timeouts gracefully");
    }

    /// Test connection pool operations with timeout global wrapper
    #[tokio::test]
    async fn test_connection_operations_with_global_timeout() {
        let pool = connection_pool::McpConnectionPool::new(connection_pool::ConnectionConfig {
            connection_timeout: Duration::from_millis(100), // Short timeout to fail quickly
            retry_attempts: 1, // Single attempt to avoid long retries
            retry_delay: Duration::from_millis(10), // Short retry delay
            ..Default::default()
        });

        pool.register_server(
            "global-timeout-server".to_string(),
            TransportType::WebSocket {
                url: "ws://192.0.2.4:12345/mcp".to_string(),
                heartbeat_interval: None,
                reconnect_config: Default::default(),
            },
            "test-client".to_string(),
            "1.0.0".to_string(),
        ).await;

        // Wrap connection attempt in timeout to test that operations complete within reasonable time
        // We expect the operation to complete quickly due to short timeout and single retry
        let operation_result = timeout(
            Duration::from_millis(500), // Reasonable timeout accounting for connection + retry
            pool.get_connection("global-timeout-server")
        ).await;

        assert!(operation_result.is_ok(), "Operation should complete within global timeout");
        
        let connection_result = operation_result.expect("Already verified timeout didn't occur");
        assert!(connection_result.is_err(), "Connection should fail due to unreachable server");
    }
}