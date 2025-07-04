#[cfg(test)]
mod tests {
    use crate::protocol::{
        ToolDefinition, CallToolResult, ListToolsResult, McpRequest, McpResponse, ResponseResult, ToolContent,
    };
    use crate::clients::{McpClient, McpConnection, StdioMcpClient, WebSocketMcpClient};
    use crate::transport::{McpTransport, TransportError};
    use async_trait::async_trait;

    struct MockTransport {
        responses: Vec<McpResponse>,
        response_index: usize,
        connected: bool,
    }

    impl MockTransport {
        fn new(responses: Vec<McpResponse>) -> Self {
            Self {
                responses,
                response_index: 0,
                connected: false,
            }
        }
    }

    #[async_trait]
    impl McpTransport for MockTransport {
        async fn connect(&mut self) -> Result<(), TransportError> {
            self.connected = true;
            Ok(())
        }

        async fn send(&mut self, _message: McpRequest) -> Result<(), TransportError> {
            if !self.connected {
                return Err(TransportError::ConnectionError {
                    message: "Not connected".to_string(),
                    endpoint: None,
                    transport_type: None,
                    retry_count: None,
                });
            }
            Ok(())
        }

        async fn receive(&mut self) -> Result<McpResponse, TransportError> {
            if !self.connected {
                return Err(TransportError::ConnectionError {
                    message: "Not connected".to_string(),
                    endpoint: None,
                    transport_type: None,
                    retry_count: None,
                });
            }

            if self.response_index >= self.responses.len() {
                return Err(TransportError::ConnectionError {
                    message: "No more responses".to_string(),
                    endpoint: None,
                    transport_type: None,
                    retry_count: None,
                });
            }

            let response = self.responses[self.response_index].clone();
            self.response_index += 1;
            Ok(response)
        }

        async fn disconnect(&mut self) -> Result<(), TransportError> {
            self.connected = false;
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_mcp_connection_creation() {
        let transport = Box::new(MockTransport::new(vec![]));
        let connection = McpConnection::new(transport);

        assert!(!connection.is_connected);
        assert!(!connection.is_initialized);
    }

    #[tokio::test]
    async fn test_stdio_mcp_client_creation() {
        let client = StdioMcpClient::new("echo".to_string(), vec!["hello".to_string()]);
        assert!(!client.is_connected());
    }

    #[tokio::test]
    async fn test_websocket_mcp_client_creation() {
        let client = WebSocketMcpClient::new("ws://localhost:8080".to_string());
        assert!(!client.is_connected());
    }

    #[tokio::test]
    async fn test_mcp_client_trait_methods() {
        // Test that we can create and use MCP clients through the trait
        let client: Box<dyn McpClient> = Box::new(StdioMcpClient::new(
            "echo".to_string(),
            vec!["hello".to_string()],
        ));

        // Initially not connected
        assert!(!client.is_connected());
    }

    #[tokio::test]
    async fn test_mcp_connection_with_mock_transport() {
        use workflow_engine_core::mcp::protocol::{InitializeResult, ServerCapabilities, ServerInfo};

        let init_response = McpResponse::Result {
            id: "test-id".to_string(),
            result: ResponseResult::Initialize(InitializeResult {
                protocol_version: "2024-11-05".to_string(),
                capabilities: ServerCapabilities {
                    logging: None,
                    prompts: None,
                    resources: None,
                    tools: None,
                },
                server_info: ServerInfo {
                    name: "test-server".to_string(),
                    version: "1.0.0".to_string(),
                },
            }),
        };

        let transport = Box::new(MockTransport::new(vec![init_response]));
        let mut connection = McpConnection::new(transport);

        // Test connection
        connection.transport.connect().await.unwrap();
        assert!(
            connection
                .transport
                .send(McpRequest::Initialized)
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn test_tool_definitions() {
        let tool_def = ToolDefinition {
            name: "test_tool".to_string(),
            description: Some("A test tool".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "input": {"type": "string"}
                }
            }),
        };

        assert_eq!(tool_def.name, "test_tool");
        assert!(tool_def.description.is_some());
    }

    #[tokio::test]
    async fn test_tool_call_result() {
        let result = CallToolResult {
            content: vec![ToolContent::Text {
                text: "Hello, World!".to_string(),
            }],
            is_error: Some(false),
        };

        assert_eq!(result.content.len(), 1);
        match &result.content[0] {
            ToolContent::Text { text } => assert_eq!(text, "Hello, World!"),
            _ => panic!("Expected text content"),
        }
    }

    #[tokio::test]
    async fn test_mcp_request_id_extraction() {
        let request = McpRequest::ListTools {
            id: "test-123".to_string(),
        };

        assert_eq!(request.get_id(), Some("test-123"));

        let notification = McpRequest::Initialized;
        assert_eq!(notification.get_id(), None);
    }

    #[tokio::test]
    async fn test_mcp_response_id_extraction() {
        let response = McpResponse::Result {
            id: "response-456".to_string(),
            result: ResponseResult::ListTools(ListToolsResult { tools: vec![] }),
        };

        assert_eq!(response.get_id(), "response-456");
    }
}
