// Tests for external MCP client
use backend::core::error::WorkflowError;
use backend::core::mcp::protocol::{CallToolResult, ToolContent, ToolDefinition};
use backend::core::mcp::transport::{TransportType, HttpPoolConfig, ReconnectConfig};
use backend::core::nodes::external_mcp_client::{
    AuthConfig, BaseExternalMCPClient, ExternalMCPClientNode, ExternalMCPConfig, RetryConfig,
};
use backend::core::nodes::Node;
use backend::core::task::TaskContext;
use std::collections::HashMap;

fn create_test_config(service_name: &str) -> ExternalMCPConfig {
    ExternalMCPConfig {
        service_name: service_name.to_string(),
        transport: TransportType::Http {
            base_url: "http://localhost:8080".to_string(),
            pool_config: HttpPoolConfig::default(),
        },
        auth: None,
        retry_config: RetryConfig::default(),
    }
}

#[test]
fn test_base_external_mcp_client_creation() {
    let config = create_test_config("test_service");
    let client = BaseExternalMCPClient::new(config.clone());
    
    assert_eq!(client.get_config().service_name, "test_service");
    assert!(!client.is_connected());
}

#[test]
fn test_retry_config_default() {
    let config = RetryConfig::default();
    
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.initial_delay_ms, 1000);
    assert_eq!(config.max_delay_ms, 30000);
    assert_eq!(config.backoff_multiplier, 2.0);
}

#[test]
fn test_node_process() {
    let config = create_test_config("test_service");
    let client = BaseExternalMCPClient::new(config);
    
    let task_context = TaskContext::new("test_workflow".to_string(), serde_json::json!({}));
    let result = client.process(task_context);
    
    assert!(result.is_ok());
    let updated_context = result.unwrap();
    let data = updated_context.get_data::<bool>("external_mcp_client_processed");
    assert!(data.is_ok());
    assert_eq!(data.unwrap(), Some(true));
}

#[test]
fn test_auth_config_creation() {
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), "Bearer token123".to_string());
    
    let auth = AuthConfig {
        token: Some("token123".to_string()),
        headers: Some(headers.clone()),
    };
    
    assert_eq!(auth.token, Some("token123".to_string()));
    assert_eq!(auth.headers.unwrap().get("Authorization"), Some(&"Bearer token123".to_string()));
}

#[tokio::test]
async fn test_execute_tool_not_connected() {
    let config = create_test_config("test_service");
    let mut client = BaseExternalMCPClient::new(config);
    
    let result = client.execute_tool("test_tool", None).await;
    
    assert!(result.is_err());
    match result {
        Err(WorkflowError::MCPConnectionError { message }) => {
            assert!(message.contains("not connected"));
        }
        _ => panic!("Expected MCPConnectionError"),
    }
}

#[tokio::test]
async fn test_list_tools_not_connected() {
    let config = create_test_config("test_service");
    let mut client = BaseExternalMCPClient::new(config);
    
    let result = client.list_tools().await;
    
    assert!(result.is_err());
    match result {
        Err(WorkflowError::MCPConnectionError { message }) => {
            assert!(message.contains("not connected"));
        }
        _ => panic!("Expected MCPConnectionError"),
    }
}

#[tokio::test]
async fn test_disconnect_when_not_connected() {
    let config = create_test_config("test_service");
    let mut client = BaseExternalMCPClient::new(config);
    
    let result = client.disconnect().await;
    
    assert!(result.is_ok());
    assert!(!client.is_connected());
}

#[test]
fn test_transport_type_creation() {
    // Test HTTP transport
    let http_transport = TransportType::Http {
        base_url: "http://localhost:8080".to_string(),
        pool_config: HttpPoolConfig::default(),
    };
    
    match http_transport {
        TransportType::Http { base_url, .. } => {
            assert_eq!(base_url, "http://localhost:8080");
        }
        _ => panic!("Expected HTTP transport"),
    }
    
    // Test WebSocket transport
    let ws_transport = TransportType::WebSocket {
        url: "ws://localhost:8080".to_string(),
        heartbeat_interval: Some(std::time::Duration::from_secs(30)),
        reconnect_config: ReconnectConfig::default(),
    };
    
    match ws_transport {
        TransportType::WebSocket { url, .. } => {
            assert_eq!(url, "ws://localhost:8080");
        }
        _ => panic!("Expected WebSocket transport"),
    }
    
    // Test stdio transport
    let stdio_transport = TransportType::Stdio {
        command: "python".to_string(),
        args: vec!["-m".to_string(), "mcp_server".to_string()],
        auto_restart: true,
        max_restarts: 3,
    };
    
    match stdio_transport {
        TransportType::Stdio { command, args, .. } => {
            assert_eq!(command, "python");
            assert_eq!(args, vec!["-m", "mcp_server"]);
        }
        _ => panic!("Expected stdio transport"),
    }
}

#[tokio::test]
async fn test_retry_config_backoff() {
    let config = create_test_config("test_service");
    let retry_config = config.retry_config.clone();
    
    let mut delay = retry_config.initial_delay_ms;
    
    // Test exponential backoff calculation
    for _ in 0..3 {
        delay = (delay as f64 * retry_config.backoff_multiplier) as u64;
        delay = delay.min(retry_config.max_delay_ms);
    }
    
    // After 3 iterations with 2.0 multiplier: 1000 -> 2000 -> 4000 -> 8000
    assert_eq!(delay, 8000);
}

#[test]
fn test_external_mcp_config_with_auth() {
    let mut headers = HashMap::new();
    headers.insert("X-API-Key".to_string(), "secret_key".to_string());
    
    let auth = Some(AuthConfig {
        token: Some("secret_key".to_string()),
        headers: Some(headers),
    });
    
    let config = ExternalMCPConfig {
        service_name: "auth_service".to_string(),
        transport: TransportType::Http {
            base_url: "https://api.example.com".to_string(),
            pool_config: HttpPoolConfig::default(),
        },
        auth: auth.clone(),
        retry_config: RetryConfig::default(),
    };
    
    assert_eq!(config.service_name, "auth_service");
    assert!(config.auth.is_some());
    
    let auth_config = config.auth.unwrap();
    assert_eq!(auth_config.token, Some("secret_key".to_string()));
    assert!(auth_config.headers.is_some());
}

#[test]
fn test_exponential_backoff_calculation() {
    let retry_config = RetryConfig {
        max_retries: 5,
        initial_delay_ms: 100,
        max_delay_ms: 5000,
        backoff_multiplier: 2.5,
    };
    
    let mut delay = retry_config.initial_delay_ms;
    let mut delays = vec![delay];
    
    for _ in 0..4 {
        delay = (delay as f64 * retry_config.backoff_multiplier) as u64;
        delay = delay.min(retry_config.max_delay_ms);
        delays.push(delay);
    }
    
    // Expected: 100, 250, 625, 1562, 3905
    assert_eq!(delays[0], 100);
    assert_eq!(delays[1], 250);
    assert_eq!(delays[2], 625);
    assert_eq!(delays[3], 1562);
    assert_eq!(delays[4], 3905);
}

#[test]
fn test_create_client_with_different_transports() {
    let config_http = ExternalMCPConfig {
        service_name: "http_service".to_string(),
        transport: TransportType::Http {
            base_url: "http://example.com".to_string(),
            pool_config: HttpPoolConfig::default(),
        },
        auth: None,
        retry_config: RetryConfig::default(),
    };
    
    let config_ws = ExternalMCPConfig {
        service_name: "ws_service".to_string(),
        transport: TransportType::WebSocket {
            url: "ws://example.com".to_string(),
            heartbeat_interval: None,
            reconnect_config: ReconnectConfig::default(),
        },
        auth: None,
        retry_config: RetryConfig::default(),
    };
    
    let config_stdio = ExternalMCPConfig {
        service_name: "stdio_service".to_string(),
        transport: TransportType::Stdio {
            command: "python".to_string(),
            args: vec![],
            auto_restart: false,
            max_restarts: 0,
        },
        auth: None,
        retry_config: RetryConfig::default(),
    };
    
    let client_http = BaseExternalMCPClient::new(config_http);
    let client_ws = BaseExternalMCPClient::new(config_ws);
    let client_stdio = BaseExternalMCPClient::new(config_stdio);
    
    assert_eq!(client_http.get_config().service_name, "http_service");
    assert_eq!(client_ws.get_config().service_name, "ws_service");
    assert_eq!(client_stdio.get_config().service_name, "stdio_service");
}

#[tokio::test]
#[ignore] // Run with cargo test -- --ignored
async fn test_integration_with_real_mcp_server() {
    // This test would connect to a real MCP server
    // For now, it's a placeholder showing the structure
    
    let config = ExternalMCPConfig {
        service_name: "test_integration".to_string(),
        transport: TransportType::Http {
            base_url: "http://localhost:8001".to_string(), // HelpScout test server
            pool_config: HttpPoolConfig::default(),
        },
        auth: None,
        retry_config: RetryConfig::default(),
    };
    
    let mut client = BaseExternalMCPClient::new(config);
    
    // Would test real connection
    let connect_result = client.connect().await;
    if connect_result.is_ok() {
        assert!(client.is_connected());
        
        let tools = client.list_tools().await;
        assert!(tools.is_ok());
        
        let _ = client.disconnect().await;
    }
}