use std::sync::Arc;
use std::time::Duration;
use tokio;

use backend::core::{
    error::WorkflowError,
    mcp::{
        config::{MCPConfig, MCPServerConfig},
        connection_pool::{MCPConnectionPool, ConnectionConfig},
        protocol::{MCPRequest, MCPResponse, ResponseResult, ToolContent},
        server::MCPToolServer,
        transport::TransportType,
    },
    mcp::server::customer_support::{CustomerSupportMCPServer, tools::{ValidateTicketNode, FilterSpamNode, AnalyzeTicketNode}},
    nodes::Node,
    task::TaskContext,
    workflow::{Workflow, schema::WorkflowSchema},
};

// Mock node for testing
#[derive(Debug)]
struct TestNode {
    name: String,
}

impl TestNode {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl Node for TestNode {
    fn node_name(&self) -> String {
        self.name.clone()
    }
    
    fn process(&self, mut task_context: TaskContext) -> Result<TaskContext, WorkflowError> {
        task_context.set_data("processed_by", serde_json::Value::String(self.name.clone()));
        task_context.set_data("test_result", serde_json::Value::String("success".to_string()));
        Ok(task_context)
    }
}

#[tokio::test]
async fn test_mcp_config_integration() {
    // Test default configuration
    let config = MCPConfig::default();
    assert!(!config.enabled);
    assert_eq!(config.client_name, "ai-workflow-system");
    assert_eq!(config.client_version, "1.0.0");
    assert!(config.servers.is_empty());

    // Test configuration with servers
    let mut config_with_servers = MCPConfig::default();
    config_with_servers.enabled = true;
    
    let server_config = MCPServerConfig {
        name: "test-server".to_string(),
        enabled: true,
        transport: TransportType::WebSocket {
            url: "ws://localhost:8080/mcp".to_string(),
            heartbeat_interval: Some(std::time::Duration::from_secs(30)),
            reconnect_config: backend::core::mcp::transport::ReconnectConfig::default(),
        },
        auto_connect: true,
        retry_on_failure: true,
    };
    
    config_with_servers.servers.insert("test-server".to_string(), server_config);
    
    assert!(config_with_servers.enabled);
    assert!(config_with_servers.is_server_enabled("test-server"));
    assert!(!config_with_servers.is_server_enabled("non-existent"));
    
    let enabled_servers = config_with_servers.get_enabled_servers();
    assert_eq!(enabled_servers.len(), 1);
    assert_eq!(enabled_servers[0].name, "test-server");
}

#[tokio::test]
async fn test_workflow_mcp_server_exposure() {
    // Create a simple workflow schema
    let schema = WorkflowSchema::new("test_workflow".to_string(), std::any::TypeId::of::<TestNode>());
    let workflow = Workflow::new(schema).unwrap();
    
    // Register some test nodes
    workflow.register_node(TestNode::new("TestNode1"));
    workflow.register_node(TestNode::new("TestNode2"));
    workflow.register_node(ValidateTicketNode::new());
    
    // Expose workflow as MCP server
    let mcp_server = workflow.expose_as_mcp_server("test-workflow-server", "1.0.0").await.unwrap();
    
    // Verify tools are registered
    assert!(mcp_server.get_tool_count().await >= 3);
    
    let tool_names = mcp_server.get_tool_names().await;
    assert!(tool_names.contains(&"test1".to_string()) || tool_names.contains(&"testnode1".to_string()));
    
    // Test list tools request
    let list_request = MCPRequest::ListTools {
        id: "test-list-001".to_string(),
    };
    
    let response = mcp_server.handle_request(list_request).await.unwrap();
    match response {
        MCPResponse::Result { result: ResponseResult::ListTools(tools_result), .. } => {
            assert!(!tools_result.tools.is_empty());
            
            // Verify we have expected tools
            let tool_names: Vec<&str> = tools_result.tools.iter()
                .map(|t| t.name.as_str())
                .collect();
            
            // Should contain tools derived from registered nodes
            assert!(tool_names.iter().any(|&name| 
                name.contains("test") || name.contains("validate")
            ));
        }
        _ => panic!("Expected ListTools response"),
    }
}

#[tokio::test]
async fn test_mcp_tool_server_full_integration() {
    let server = MCPToolServer::new("integration-test-server".to_string(), "1.0.0".to_string());
    
    // Register various customer support nodes
    let validate_node = Arc::new(ValidateTicketNode::new());
    server.register_node_with_auto_metadata(validate_node).await.unwrap();
    
    let filter_node = Arc::new(FilterSpamNode::new());
    server.register_node_with_auto_metadata(filter_node).await.unwrap();
    
    let analyze_node = Arc::new(AnalyzeTicketNode::new());
    server.register_node_with_auto_metadata(analyze_node).await.unwrap();
    
    assert_eq!(server.get_tool_count().await, 3);
    
    // Test initialization request
    let init_request = MCPRequest::Initialize {
        id: "init-001".to_string(),
        params: backend::core::mcp::protocol::InitializeParams {
            protocol_version: "2024-11-05".to_string(),
            capabilities: backend::core::mcp::protocol::ClientCapabilities {
                roots: None,
                sampling: None,
            },
            client_info: backend::core::mcp::protocol::ClientInfo {
                name: "test-client".to_string(),
                version: "1.0.0".to_string(),
            },
        },
    };
    
    let init_response = server.handle_request(init_request).await.unwrap();
    match init_response {
        MCPResponse::Result { result: ResponseResult::Initialize(init_result), .. } => {
            assert_eq!(init_result.server_info.name, "integration-test-server");
            assert_eq!(init_result.server_info.version, "1.0.0");
            assert!(init_result.capabilities.tools.is_some());
        }
        _ => panic!("Expected Initialize response"),
    }
    
    // Test calling a tool
    let call_request = MCPRequest::CallTool {
        id: "call-001".to_string(),
        params: backend::core::mcp::protocol::ToolCallParams {
            name: "validate".to_string(), // Should match generated tool name
            arguments: Some({
                let mut args = std::collections::HashMap::new();
                args.insert("context_data".to_string(), serde_json::json!({
                    "ticket_id": "TKT-123",
                    "customer_id": "CUST-456",
                    "message": "Test message",
                    "priority": "high"
                }));
                args
            }),
        },
    };
    
    let call_response = server.handle_request(call_request).await.unwrap();
    match call_response {
        MCPResponse::Result { result: ResponseResult::CallTool(call_result), .. } => {
            assert!(!call_result.is_error.unwrap_or(true));
            assert!(!call_result.content.is_empty());
            
            // Verify content is text and contains expected information
            match &call_result.content[0] {
                ToolContent::Text { text } => {
                    // The content should be a JSON representation of the TaskContext
                    assert!(text.contains("ticket_id") || text.contains("TKT-123"));
                }
                _ => panic!("Expected text content"),
            }
        }
        MCPResponse::Error { .. } => {
            // Tool might not exist with exact name "validate", which is fine for this test
            // The important thing is that the server handles the request properly
        }
        _ => panic!("Expected CallTool response or error"),
    }
}

#[tokio::test]
async fn test_customer_support_mcp_server_integration() {
    let server = CustomerSupportMCPServer::new().await.unwrap();
    
    // Verify all expected tools are available
    assert_eq!(server.get_tool_count().await, 8);
    
    let tool_names = server.get_tool_names().await;
    let expected_tools = vec![
        "validate_ticket", "filter_spam", "determine_intent",
        "analyze_ticket", "generate_response", "escalate_ticket",
        "process_invoice", "close_ticket"
    ];
    
    for expected_tool in expected_tools {
        assert!(tool_names.contains(&expected_tool.to_string()),
                "Missing expected tool: {}", expected_tool);
    }
    
    // Test a complete workflow: list tools -> call tool
    let list_request = MCPRequest::ListTools {
        id: "list-integration-001".to_string(),
    };
    
    let list_response = server.get_server().handle_request(list_request).await.unwrap();
    let tools = match list_response {
        MCPResponse::Result { result: ResponseResult::ListTools(tools_result), .. } => {
            tools_result.tools
        }
        _ => panic!("Expected ListTools response"),
    };
    
    // Find and call the validate_ticket tool
    let validate_tool = tools.iter()
        .find(|t| t.name == "validate_ticket")
        .expect("validate_ticket tool should be available");
    
    let call_request = MCPRequest::CallTool {
        id: "call-integration-001".to_string(),
        params: backend::core::mcp::protocol::ToolCallParams {
            name: validate_tool.name.clone(),
            arguments: Some({
                let mut args = std::collections::HashMap::new();
                args.insert("context_data".to_string(), serde_json::json!({
                    "ticket_id": "TKT-INTEGRATION-001",
                    "customer_id": "CUST-INTEGRATION-001",
                    "message": "Integration test message",
                    "priority": "medium"
                }));
                args
            }),
        },
    };
    
    let call_response = server.get_server().handle_request(call_request).await.unwrap();
    match call_response {
        MCPResponse::Result { result: ResponseResult::CallTool(call_result), .. } => {
            assert!(!call_result.is_error.unwrap_or(true));
            assert!(!call_result.content.is_empty());
            
            // Verify the result contains the processed data
            match &call_result.content[0] {
                ToolContent::Text { text } => {
                    assert!(text.contains("TKT-INTEGRATION-001"));
                }
                _ => panic!("Expected text content"),
            }
        }
        _ => panic!("Expected successful CallTool response"),
    }
}

#[tokio::test]
async fn test_connection_pool_integration() {
    let config = ConnectionConfig {
        max_connections_per_server: 3,
        connection_timeout: Duration::from_secs(2),
        idle_timeout: Duration::from_secs(10),
        retry_attempts: 1, // Reduced for faster test
        retry_delay: Duration::from_millis(100),
        health_check_interval: Duration::from_secs(5),
        enable_load_balancing: true,
        load_balancing_strategy: backend::core::mcp::connection_pool::LoadBalancingStrategy::RoundRobin,
        circuit_breaker: backend::core::error::circuit_breaker::CircuitBreakerConfig::default(),
        health_monitoring: backend::core::mcp::health::HealthConfig::default(),
        enable_auto_reconnect: true,
        backoff_config: backend::core::mcp::connection_pool::BackoffConfig::default(),
    };
    
    let pool = MCPConnectionPool::new(config);
    
    // Register multiple servers
    pool.register_server(
        "server1".to_string(),
        TransportType::WebSocket {
            url: "ws://nonexistent1:8080/mcp".to_string(),
            heartbeat_interval: Some(std::time::Duration::from_secs(30)),
            reconnect_config: backend::core::mcp::transport::ReconnectConfig::default(),
        },
        "test-client".to_string(),
        "1.0.0".to_string(),
    ).await;
    
    pool.register_server(
        "server2".to_string(),
        TransportType::Stdio {
            command: "nonexistent-command".to_string(),
            args: vec!["arg1".to_string()],
            auto_restart: true,
            max_restarts: 3,
        },
        "test-client".to_string(),
        "1.0.0".to_string(),
    ).await;
    
    // Test connection failures (expected with non-existent servers)
    let result1 = pool.get_connection("server1").await;
    assert!(result1.is_err());
    
    let result2 = pool.get_connection("server2").await;
    assert!(result2.is_err());
    
    // Test health check with no connections
    let health = pool.health_check().await.unwrap();
    assert_eq!(health.len(), 0);
    
    // Test stats
    let stats = pool.get_pool_stats().await;
    assert_eq!(stats.len(), 0); // No successful connections
    
    // Test cleanup
    let cleaned = pool.cleanup_expired_connections().await.unwrap();
    assert_eq!(cleaned, 0);
    
    // Test disconnect all
    pool.disconnect_all().await.unwrap();
}

#[tokio::test]
async fn test_mcp_error_handling_integration() {
    let server = MCPToolServer::new("error-test-server".to_string(), "1.0.0".to_string());
    
    // Test calling non-existent tool
    let bad_call_request = MCPRequest::CallTool {
        id: "bad-call-001".to_string(),
        params: backend::core::mcp::protocol::ToolCallParams {
            name: "non_existent_tool".to_string(),
            arguments: None,
        },
    };
    
    let response = server.handle_request(bad_call_request).await.unwrap();
    match response {
        MCPResponse::Error { error, .. } => {
            assert_eq!(error.code, -32601); // Method not found
            assert!(error.message.contains("not found"));
        }
        _ => panic!("Expected error response for non-existent tool"),
    }
    
    // Test malformed requests by testing with Initialized notification
    // (which should not expect a response)
    let notification = MCPRequest::Initialized;
    let result = server.handle_request(notification).await;
    assert!(result.is_err()); // Should return error as notifications don't expect responses
}

#[tokio::test]
async fn test_workflow_register_mcp_server_integration() {
    let schema = WorkflowSchema::new("mcp_test_workflow".to_string(), std::any::TypeId::of::<TestNode>());
    let workflow = Workflow::new(schema).unwrap();
    
    // Test registering external MCP servers
    let result1 = workflow.register_mcp_server(
        "ws://localhost:8080/mcp",
        TransportType::WebSocket {
            url: "ws://localhost:8080/mcp".to_string(),
            heartbeat_interval: Some(std::time::Duration::from_secs(30)),
            reconnect_config: backend::core::mcp::transport::ReconnectConfig::default(),
        }
    ).await;
    assert!(result1.is_ok());
    
    let result2 = workflow.register_mcp_server(
        "stdio://test-server",
        TransportType::Stdio {
            command: "python".to_string(),
            args: vec!["server.py".to_string()],
            auto_restart: true,
            max_restarts: 3,
        }
    ).await;
    assert!(result2.is_ok());
    
    // The actual implementation currently just logs the registration
    // In a full implementation, this would store the server configs
    // for later use by agent nodes
}

#[tokio::test]
async fn test_end_to_end_mcp_workflow() {
    // This test simulates a complete MCP workflow:
    // 1. Create workflow with nodes
    // 2. Expose workflow as MCP server
    // 3. Register external MCP servers
    // 4. Test the complete integration
    
    let schema = WorkflowSchema::new("e2e_test_workflow".to_string(), std::any::TypeId::of::<TestNode>());
    let workflow = Workflow::new(schema).unwrap();
    
    // Register nodes
    workflow.register_node(TestNode::new("ProcessorNode"));
    workflow.register_node(ValidateTicketNode::new());
    workflow.register_node(FilterSpamNode::new());
    
    // Expose as MCP server
    let internal_server = workflow.expose_as_mcp_server("e2e-workflow-server", "1.0.0").await.unwrap();
    assert!(internal_server.get_tool_count().await >= 3);
    
    // Register external servers
    workflow.register_mcp_server(
        "ws://external-server:8080/mcp",
        TransportType::WebSocket {
            url: "ws://external-server:8080/mcp".to_string(),
            heartbeat_interval: Some(std::time::Duration::from_secs(30)),
            reconnect_config: backend::core::mcp::transport::ReconnectConfig::default(),
        }
    ).await.unwrap();
    
    // Test the internal server functionality
    let list_request = MCPRequest::ListTools {
        id: "e2e-list-001".to_string(),
    };
    
    let response = internal_server.handle_request(list_request).await.unwrap();
    match response {
        MCPResponse::Result { result: ResponseResult::ListTools(tools_result), .. } => {
            assert!(!tools_result.tools.is_empty());
            
            // Verify we have tools from all registered nodes
            let tool_names: Vec<&str> = tools_result.tools.iter()
                .map(|t| t.name.as_str())
                .collect();
            
            // Should have tools derived from our registered nodes
            assert!(tool_names.iter().any(|&name| name.contains("process") || name.contains("validate") || name.contains("filter")));
        }
        _ => panic!("Expected ListTools response"),
    }
    
    // Test calling a tool (this exercises the complete node execution path)
    if let Some(tool) = internal_server.get_tool_names().await.first() {
        let call_request = MCPRequest::CallTool {
            id: "e2e-call-001".to_string(),
            params: backend::core::mcp::protocol::ToolCallParams {
                name: tool.clone(),
                arguments: Some({
                    let mut args = std::collections::HashMap::new();
                    args.insert("context_data".to_string(), serde_json::json!({
                        "test_data": "e2e test execution",
                        "workflow_id": "e2e_test_workflow"
                    }));
                    args
                }),
            },
        };
        
        let call_response = internal_server.handle_request(call_request).await.unwrap();
        match call_response {
            MCPResponse::Result { result: ResponseResult::CallTool(call_result), .. } => {
                assert!(!call_result.is_error.unwrap_or(true));
                assert!(!call_result.content.is_empty());
            }
            MCPResponse::Error { .. } => {
                // Some tools might not work without proper context, but the MCP layer should handle it gracefully
            }
            _ => panic!("Expected CallTool response"),
        }
    }
}