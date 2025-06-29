//! TDD Test 1c: MCP server connection timeouts
//! 
//! This test file demonstrates Test-Driven Development for MCP connection timeout scenarios.
//! It follows the RED-GREEN-REFACTOR cycle to replace .unwrap()/.expect() calls with proper error handling.

use std::time::Duration;
use tokio::time::timeout;

use workflow_engine_core::error::WorkflowError;
use workflow_engine_mcp::connection_pool::{ConnectionConfig, McpConnectionPool};
use workflow_engine_mcp::transport::TransportType;

/// Test that connection timeouts are handled gracefully without panics
#[tokio::test]
async fn test_connection_timeout_no_panic() {
    let config = ConnectionConfig {
        connection_timeout: Duration::from_millis(100), // Very short timeout
        retry_attempts: 1, // Single attempt to avoid long test duration
        retry_delay: Duration::from_millis(10),
        ..Default::default()
    };

    let pool = McpConnectionPool::new(config);

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
        Err(WorkflowError::MCPConnectionError { message }) => {
            assert!(message.contains("timeout") || message.contains("Failed to create connection"), 
                "Error should indicate timeout or connection failure, got: {}", message);
        }
        Err(WorkflowError::MCPError { message, .. }) => {
            assert!(message.contains("timeout") || message.contains("connection"), 
                "Error should indicate timeout or connection failure, got: {}", message);
        }
        Err(other) => panic!("Unexpected error type: {:?}", other),
        Ok(_) => panic!("Connection should not succeed"),
    }
}

/// Test cleanup operations handle errors gracefully
#[tokio::test]
async fn test_cleanup_expired_connections_error_handling() {
    let pool = McpConnectionPool::new(ConnectionConfig::default());
    
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
    let pool = McpConnectionPool::new(ConnectionConfig {
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
    let pool = McpConnectionPool::new(ConnectionConfig {
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
    let pool = McpConnectionPool::new(ConnectionConfig::default());

    // Test with unregistered server (should fail gracefully)
    let result = pool.get_connection("nonexistent-server").await;
    
    assert!(result.is_err(), "Unregistered server should cause error");
    
    match result {
        Err(WorkflowError::MCPError { message, operation, server_name }) => {
            assert_eq!(server_name, "nonexistent-server");
            assert_eq!(operation, "get_connection");
            assert!(message.contains("not registered"));
        }
        Err(other) => panic!("Expected MCPError, got: {:?}", other),
        Ok(_) => panic!("Should not succeed for unregistered server"),
    }
}

/// Test timeout with external health checks
#[tokio::test]
async fn test_health_check_timeout_handling() {
    let pool = McpConnectionPool::new(ConnectionConfig {
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
    assert!(health_status.contains_key("health-timeout-server"));
    
    // Server should be reported as unhealthy due to connection issues
    let server_healthy = health_status["health-timeout-server"];
    assert!(!server_healthy, "Server should be reported as unhealthy due to timeout");
}

/// Test force reconnect with timeout scenarios
#[tokio::test]
async fn test_force_reconnect_timeout_handling() {
    let pool = McpConnectionPool::new(ConnectionConfig::default());

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
    let pool = McpConnectionPool::new(ConnectionConfig {
        connection_timeout: Duration::from_millis(200),
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
    let operation_result = timeout(
        Duration::from_secs(2), // Global timeout for the test operation
        pool.get_connection("global-timeout-server")
    ).await;

    assert!(operation_result.is_ok(), "Operation should complete within global timeout");
    
    let connection_result = operation_result.expect("Already verified timeout didn't occur");
    assert!(connection_result.is_err(), "Connection should fail due to unreachable server");
}