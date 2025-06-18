#[cfg(test)]
mod tests {
    use super::super::server::CustomerSupportMCPServer;
    use crate::core::mcp::protocol::MCPRequest;

    #[tokio::test]
    async fn test_customer_support_server_creation() {
        let server = CustomerSupportMCPServer::new().await.unwrap();
        assert_eq!(server.get_tool_count().await, 8); // All customer support tools
    }

    #[tokio::test]
    async fn test_server_has_expected_tools() {
        let server = CustomerSupportMCPServer::new().await.unwrap();
        let tool_names = server.get_tool_names().await;
        
        let expected_tools = vec![
            "validate_ticket",
            "filter_spam", 
            "determine_intent",
            "analyze_ticket",
            "generate_response",
            "escalate_ticket",
            "process_invoice",
            "close_ticket"
        ];

        for tool in expected_tools {
            assert!(tool_names.contains(&tool.to_string()), "Missing tool: {}", tool);
        }
    }

    #[tokio::test]
    async fn test_list_tools_request() {
        let server = CustomerSupportMCPServer::new().await.unwrap();
        
        let request = MCPRequest::ListTools {
            id: "test-123".to_string(),
        };

        let response = server.get_server().handle_request(request).await.unwrap();
        
        match response {
            crate::core::mcp::protocol::MCPResponse::Result { 
                result: crate::core::mcp::protocol::ResponseResult::ListTools(tools_result), 
                .. 
            } => {
                assert_eq!(tools_result.tools.len(), 8);
                assert!(tools_result.tools.iter().any(|t| t.name == "validate_ticket"));
                assert!(tools_result.tools.iter().any(|t| t.name == "filter_spam"));
            }
            _ => panic!("Expected ListTools response"),
        }
    }
} 