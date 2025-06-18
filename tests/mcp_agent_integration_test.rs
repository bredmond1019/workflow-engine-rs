use std::collections::HashMap;
use std::sync::Arc;
use tokio;

use backend::core::{
    ai_agents::{anthropic::AnthropicAgentNode, openai::OpenAIAgentNode},
    error::WorkflowError,
    mcp::{
        clients::MCPClient,
        connection_pool::{ConnectionConfig, MCPConnectionPool},
        protocol::{
            CallToolResult, MCPRequest, MCPResponse, ResponseResult, ToolContent,
            ToolDefinition,
        },
        server::MCPToolServer,
        transport::TransportType,
    },
    nodes::agent::{AgentConfig, ModelProvider},
    task::TaskContext,
};

// Mock MCP Client for testing
#[derive(Debug)]
struct MockMCPClient {
    tools: Vec<ToolDefinition>,
    connected: bool,
}

impl MockMCPClient {
    fn new_with_customer_support_tools() -> Self {
        let tools = vec![
            ToolDefinition {
                name: "validate_ticket".to_string(),
                description: Some("Validates customer support ticket".to_string()),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "context_data": {"type": "object"}
                    }
                }),
            },
            ToolDefinition {
                name: "analyze_ticket".to_string(),
                description: Some("Analyzes customer support ticket".to_string()),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "context_data": {"type": "object"}
                    }
                }),
            },
        ];

        Self {
            tools,
            connected: false,
        }
    }
}

#[async_trait::async_trait]
impl MCPClient for MockMCPClient {
    async fn connect(&mut self) -> Result<(), WorkflowError> {
        self.connected = true;
        Ok(())
    }

    async fn initialize(
        &mut self,
        _client_name: &str,
        _client_version: &str,
    ) -> Result<(), WorkflowError> {
        Ok(())
    }

    async fn list_tools(&mut self) -> Result<Vec<ToolDefinition>, WorkflowError> {
        Ok(self.tools.clone())
    }

    async fn call_tool(
        &mut self,
        name: &str,
        _arguments: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<CallToolResult, WorkflowError> {
        let content = match name {
            "validate_ticket" => vec![ToolContent::Text {
                text: "Ticket validation: VALID - All required fields present".to_string(),
            }],
            "analyze_ticket" => vec![ToolContent::Text {
                text: "Ticket analysis: Priority: HIGH, Category: BILLING, Sentiment: FRUSTRATED"
                    .to_string(),
            }],
            _ => vec![ToolContent::Text {
                text: format!("Tool '{}' executed successfully", name),
            }],
        };

        Ok(CallToolResult {
            content,
            is_error: Some(false),
        })
    }

    async fn disconnect(&mut self) -> Result<(), WorkflowError> {
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

#[tokio::test]
async fn test_anthropic_agent_with_mcp_client() {
    let config = AgentConfig {
        system_prompt: "You are Claude, a helpful AI assistant".to_string(),
        model_provider: ModelProvider::Anthropic,
        model_name: "claude-3-opus".to_string(),
        mcp_server_uri: Some("mock://customer-support".to_string()),
    };

    let mut agent = AnthropicAgentNode::new(config);
    let mut mock_client = MockMCPClient::new_with_customer_support_tools();
    mock_client.connect().await.unwrap();
    mock_client
        .initialize("test-client", "1.0.0")
        .await
        .unwrap();

    agent.set_mcp_client(Box::new(mock_client));
    assert!(agent.has_mcp_client());

    // Create task context for customer support scenario
    let mut task_context = TaskContext::new(
        "customer_support".to_string(), 
        serde_json::Value::Null
    );
    task_context.set_data(
        "prompt",
        serde_json::Value::String("Please validate and analyze this customer ticket".to_string()),
    ).unwrap();
    task_context.set_data(
        "ticket_id",
        serde_json::Value::String("TKT-123".to_string()),
    ).unwrap();
    task_context.set_data(
        "customer_message",
        serde_json::Value::String("I'm having trouble with my billing".to_string()),
    ).unwrap();

    // Note: We can't easily test the full MCP integration without mocking the Anthropic API
    // This test verifies the structure is in place
    assert!(agent.has_mcp_client());
}

#[tokio::test]
async fn test_openai_agent_with_mcp_client() {
    let config = AgentConfig {
        system_prompt: "You are a helpful assistant".to_string(),
        model_provider: ModelProvider::OpenAI,
        model_name: "gpt-4".to_string(),
        mcp_server_uri: Some("mock://customer-support".to_string()),
    };

    let mut agent = OpenAIAgentNode::new(config).unwrap();
    let mut mock_client = MockMCPClient::new_with_customer_support_tools();
    mock_client.connect().await.unwrap();
    mock_client
        .initialize("test-client", "1.0.0")
        .await
        .unwrap();

    agent.set_mcp_client(Box::new(mock_client));
    assert!(agent.has_mcp_client());

    // Create task context
    let mut task_context = TaskContext::new(
        "customer_support".to_string(), 
        serde_json::Value::Null
    );
    task_context.set_data(
        "prompt",
        serde_json::Value::String("Generate a response to validate this ticket".to_string()),
    ).unwrap();

    // Note: We can't easily test the full MCP integration without mocking the OpenAI API
    // This test verifies the structure is in place
    assert!(agent.has_mcp_client());
}

#[tokio::test]
async fn test_agent_tool_selection_logic() {
    let mut mock_client = MockMCPClient::new_with_customer_support_tools();
    mock_client.connect().await.unwrap();

    let tools = mock_client.list_tools().await.unwrap();
    assert_eq!(tools.len(), 2);
    assert!(tools.iter().any(|t| t.name == "validate_ticket"));
    assert!(tools.iter().any(|t| t.name == "analyze_ticket"));

    // Test tool calling
    let validate_result = mock_client
        .call_tool("validate_ticket", None)
        .await
        .unwrap();
    assert!(!validate_result.is_error.unwrap_or(true));
    assert!(!validate_result.content.is_empty());

    let analyze_result = mock_client.call_tool("analyze_ticket", None).await.unwrap();
    assert!(!analyze_result.is_error.unwrap_or(true));
    assert!(!analyze_result.content.is_empty());
}

#[tokio::test]
async fn test_mcp_connection_pool_integration() {
    let config = ConnectionConfig {
        max_connections_per_server: 2,
        connection_timeout: std::time::Duration::from_secs(5),
        idle_timeout: std::time::Duration::from_secs(30),
        retry_attempts: 2,
        retry_delay: std::time::Duration::from_millis(100),
        health_check_interval: std::time::Duration::from_secs(10),
        enable_load_balancing: true,
        load_balancing_strategy: backend::core::mcp::connection_pool::LoadBalancingStrategy::RoundRobin,
        circuit_breaker: backend::core::error::circuit_breaker::CircuitBreakerConfig::default(),
        health_monitoring: backend::core::mcp::health::HealthConfig::default(),
        enable_auto_reconnect: true,
        backoff_config: backend::core::mcp::connection_pool::BackoffConfig::default(),
    };

    let pool = MCPConnectionPool::new(config);

    // Register a mock server
    pool.register_server(
        "test-server".to_string(),
        TransportType::WebSocket {
            url: "ws://localhost:9999".to_string(), // Non-existent server for testing
            heartbeat_interval: Some(std::time::Duration::from_secs(30)),
            reconnect_config: backend::core::mcp::transport::ReconnectConfig::default(),
        },
        "test-client".to_string(),
        "1.0.0".to_string(),
    )
    .await;

    // Test connection failure handling
    let connection_result = pool.get_connection("test-server").await;
    assert!(connection_result.is_err()); // Should fail to connect to non-existent server

    // Test pool stats
    let stats = pool.get_pool_stats().await;
    assert_eq!(stats.len(), 0); // No successful connections

    // Test health check
    let health = pool.health_check().await.unwrap();
    assert_eq!(health.len(), 0); // No connections to check

    // Test cleanup
    let cleaned = pool.cleanup_expired_connections().await.unwrap();
    assert_eq!(cleaned, 0); // No connections to clean
}

#[tokio::test]
async fn test_mcp_tool_server_integration() {
    let server = MCPToolServer::new("test-server".to_string(), "1.0.0".to_string());
    
    // Test basic server functionality
    assert_eq!(server.get_tool_count().await, 0); // No tools registered yet
    
    // Test list tools request on empty server
    let list_request = MCPRequest::ListTools {
        id: "test-list".to_string(),
    };

    let response = server.handle_request(list_request).await.unwrap();
    match response {
        MCPResponse::Result {
            result: ResponseResult::ListTools(tools_result),
            ..
        } => {
            assert_eq!(tools_result.tools.len(), 0);
        }
        _ => panic!("Expected ListTools response"),
    }
}

#[tokio::test]
async fn test_mock_mcp_client_tool_calling() {
    let mut client = MockMCPClient::new_with_customer_support_tools();
    client.connect().await.unwrap();
    client.initialize("test", "1.0").await.unwrap();

    // Test with arguments
    let mut args = HashMap::new();
    args.insert(
        "context_data".to_string(),
        serde_json::json!({
            "ticket_id": "TKT-123",
            "message": "Test message"
        }),
    );

    let result = client
        .call_tool("validate_ticket", Some(args))
        .await
        .unwrap();
    assert!(!result.is_error.unwrap_or(true));

    match &result.content[0] {
        ToolContent::Text { text } => {
            assert!(text.contains("VALID"));
        }
        _ => panic!("Expected text content"),
    }

    client.disconnect().await.unwrap();
    assert!(!client.is_connected());
}

#[tokio::test]
async fn test_agent_config_with_mcp_uri() {
    let config = AgentConfig {
        system_prompt: "Test prompt".to_string(),
        model_provider: ModelProvider::Anthropic,
        model_name: "claude-3".to_string(),
        mcp_server_uri: Some("stdio://customer-support-server".to_string()),
    };

    assert!(config.mcp_server_uri.is_some());
    assert_eq!(
        config.mcp_server_uri.unwrap(),
        "stdio://customer-support-server"
    );
}

#[tokio::test]
async fn test_mcp_integration_error_handling() {
    let mut mock_client = MockMCPClient::new_with_customer_support_tools();

    // Test calling tool on disconnected client
    let result = mock_client.call_tool("validate_ticket", None).await;
    assert!(result.is_ok()); // Mock client doesn't enforce connection state

    // Test calling non-existent tool
    mock_client.connect().await.unwrap();
    let result = mock_client
        .call_tool("non_existent_tool", None)
        .await
        .unwrap();
    assert!(!result.is_error.unwrap_or(true)); // Mock returns success for any tool name

    // In a real implementation, this would return an error
    match &result.content[0] {
        ToolContent::Text { text } => {
            assert!(text.contains("non_existent_tool"));
        }
        _ => panic!("Expected text content"),
    }
}
