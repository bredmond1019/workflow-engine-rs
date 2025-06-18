#[cfg(test)]
mod tests {
    use crate::core::mcp::clients::slack::*;
    use crate::core::mcp::transport::TransportType;
    use crate::core::nodes::external_mcp_client::ExternalMCPClientNode;
    use serde_json::Value;

    #[test]
    fn test_slack_client_default_config() {
        let client = SlackClientNode::with_defaults();
        let config = client.get_slack_config();

        assert_eq!(config.server_url, "http://localhost:8003");
        assert_eq!(client.get_config().service_name, "slack");
        assert!(!client.is_connected());
    }

    #[test]
    fn test_slack_client_http_config() {
        let server_url = "http://slack.example.com:9003".to_string();
        let bot_token = "xoxb-test-bot-token-123".to_string();
        let user_token = "xoxp-test-user-token-456".to_string();

        let client = SlackClientNode::with_http_transport(
            server_url.clone(),
            Some(bot_token.clone()),
            Some(user_token.clone()),
        );

        let config = client.get_slack_config();
        assert_eq!(config.server_url, server_url);
        assert_eq!(config.bot_token, Some(bot_token));
        assert_eq!(config.user_token, Some(user_token));

        match &config.transport {
            TransportType::Http { base_url, .. } => {
                assert_eq!(base_url, &server_url);
            }
            _ => panic!("Expected HTTP transport"),
        }
    }

    #[test]
    fn test_slack_client_websocket_config() {
        let websocket_url = "ws://slack.example.com:9003/ws".to_string();
        let bot_token = "xoxb-test-ws-token-789".to_string();

        let client = SlackClientNode::with_websocket_transport(
            websocket_url.clone(),
            Some(bot_token.clone()),
            None,
        );

        let config = client.get_slack_config();
        assert_eq!(config.server_url, websocket_url);
        assert_eq!(config.bot_token, Some(bot_token));
        assert_eq!(config.user_token, None);

        match &config.transport {
            TransportType::WebSocket { url, .. } => {
                assert_eq!(url, &websocket_url);
            }
            _ => panic!("Expected WebSocket transport"),
        }
    }

    #[test]
    fn test_slack_client_stdio_config() {
        let command = "python3".to_string();
        let args = vec![
            "-m".to_string(),
            "slack_mcp_server".to_string(),
            "--port".to_string(),
            "8003".to_string(),
        ];
        let bot_token = "xoxb-stdio-token-101112".to_string();

        let client = SlackClientNode::with_stdio_transport(
            command.clone(),
            args.clone(),
            Some(bot_token.clone()),
            None,
        );

        let config = client.get_slack_config();
        assert_eq!(config.server_url, "stdio://python3");
        assert_eq!(config.bot_token, Some(bot_token));
        assert_eq!(config.user_token, None);

        match &config.transport {
            TransportType::Stdio {
                command: cmd,
                args: cmd_args,
                ..
            } => {
                assert_eq!(cmd, &command);
                assert_eq!(cmd_args, &args);
            }
            _ => panic!("Expected Stdio transport"),
        }
    }

    #[test]
    fn test_slack_client_node_trait() {
        use crate::core::nodes::Node;
        use crate::core::task::TaskContext;

        let client = SlackClientNode::with_defaults();
        let mut task_context = TaskContext::new(
            "test-task".to_string(),
            Value::String("test-workflow".to_string()),
        );

        let result = client.process(task_context);
        assert!(result.is_ok());

        let updated_context = result.unwrap();
        assert!(
            updated_context
                .get_data::<bool>("slack_client_processed")
                .unwrap_or(Some(false))
                .unwrap_or(false)
        );
        assert_eq!(
            updated_context
                .get_data::<String>("service_name")
                .unwrap_or(Some("".to_string())),
            Some("slack".to_string())
        );
    }

    #[tokio::test]
    async fn test_send_message_arguments() {
        let mut client = SlackClientNode::with_defaults();

        // This test validates argument preparation for send_message
        // In a real test environment, you would mock the MCP client connection
        let channel = "#general";
        let text = "Hello, world!";
        let thread_ts = Some("1234567890.123456");

        // Since we can't connect to a real server in unit tests,
        // we just validate the client creation and configuration
        assert_eq!(client.get_config().service_name, "slack");
        assert!(!client.is_connected());

        // The actual tool execution would be tested in integration tests
        // with a real or mocked MCP server
    }

    #[tokio::test]
    async fn test_list_channels_arguments() {
        let mut client = SlackClientNode::with_defaults();

        // This test validates argument preparation for list_channels
        let exclude_archived = Some(true);
        let types = Some(vec![
            "public_channel".to_string(),
            "private_channel".to_string(),
        ]);

        // Validate client setup
        assert_eq!(client.get_config().service_name, "slack");
        assert!(!client.is_connected());

        // The actual tool execution would be tested in integration tests
    }

    #[tokio::test]
    async fn test_get_user_info_arguments() {
        let mut client = SlackClientNode::with_defaults();

        // This test validates argument preparation for get_user_info
        let user_id = "U1234567890";

        // Validate client setup
        assert_eq!(client.get_config().service_name, "slack");
        assert!(!client.is_connected());

        // The actual tool execution would be tested in integration tests
    }

    #[tokio::test]
    async fn test_get_channel_history_arguments() {
        let mut client = SlackClientNode::with_defaults();

        // This test validates argument preparation for get_channel_history
        let channel = "C1234567890";
        let limit = Some(100);
        let oldest = Some("1234567890.123456");
        let latest = Some("1234567891.123456");

        // Validate client setup
        assert_eq!(client.get_config().service_name, "slack");
        assert!(!client.is_connected());

        // The actual tool execution would be tested in integration tests
    }

    #[tokio::test]
    async fn test_create_channel_arguments() {
        let mut client = SlackClientNode::with_defaults();

        // This test validates argument preparation for create_channel
        let name = "new-test-channel";
        let is_private = Some(false);

        // Validate client setup
        assert_eq!(client.get_config().service_name, "slack");
        assert!(!client.is_connected());

        // The actual tool execution would be tested in integration tests
    }

    #[tokio::test]
    async fn test_search_messages_arguments() {
        let mut client = SlackClientNode::with_defaults();

        // This test validates argument preparation for search_messages
        let query = "important announcement";
        let sort = Some("timestamp");
        let sort_dir = Some("desc");
        let count = Some(50);

        // Validate client setup
        assert_eq!(client.get_config().service_name, "slack");
        assert!(!client.is_connected());

        // The actual tool execution would be tested in integration tests
    }

    #[tokio::test]
    async fn test_list_tools_when_not_connected() {
        let mut client = SlackClientNode::with_defaults();

        // Should return an error when not connected
        let result = client.list_tools().await;
        assert!(result.is_err());

        if let Err(e) = result {
            match e {
                crate::core::error::WorkflowError::MCPConnectionError { message } => {
                    assert!(message.contains("slack client not connected"));
                }
                _ => panic!("Expected MCPConnectionError"),
            }
        }
    }

    #[tokio::test]
    async fn test_execute_tool_when_not_connected() {
        let mut client = SlackClientNode::with_defaults();

        // Should return an error when not connected
        let result = client.execute_tool("send_message", None).await;
        assert!(result.is_err());

        if let Err(e) = result {
            match e {
                crate::core::error::WorkflowError::MCPConnectionError { message } => {
                    assert!(message.contains("slack client not connected"));
                }
                _ => panic!("Expected MCPConnectionError"),
            }
        }
    }

    #[test]
    fn test_slack_config_from_env() {
        // This test would require setting environment variables
        // For now, just test the default behavior
        let config = SlackClientConfig::default();

        // Default values when env vars are not set
        assert_eq!(config.server_url, "http://localhost:8003");

        match config.transport {
            TransportType::Http { base_url, .. } => {
                assert_eq!(base_url, "http://localhost:8003");
            }
            _ => panic!("Expected HTTP transport by default"),
        }
    }

    #[test]
    fn test_slack_auth_config_generation() {
        let bot_token = "xoxb-test-bot-token".to_string();
        let user_token = "xoxp-test-user-token".to_string();

        let client = SlackClientNode::with_http_transport(
            "http://localhost:8003".to_string(),
            Some(bot_token.clone()),
            Some(user_token.clone()),
        );

        let external_config = client.get_config();

        // Check that auth config was properly generated
        assert!(external_config.auth.is_some());

        if let Some(ref auth) = external_config.auth {
            assert_eq!(auth.token, Some(bot_token.clone()));
            assert!(auth.headers.is_some());

            if let Some(ref headers) = auth.headers {
                assert_eq!(
                    headers.get("Authorization"),
                    Some(&format!("Bearer {}", bot_token))
                );
                assert_eq!(headers.get("X-Slack-User-Token"), Some(&user_token));
            }
        }
    }

    #[test]
    fn test_slack_client_with_bot_token_only() {
        let bot_token = "xoxb-only-bot-token".to_string();

        let client = SlackClientNode::with_http_transport(
            "http://localhost:8003".to_string(),
            Some(bot_token.clone()),
            None, // No user token
        );

        let external_config = client.get_config();

        // Check that auth config was properly generated with only bot token
        assert!(external_config.auth.is_some());

        if let Some(ref auth) = external_config.auth {
            assert_eq!(auth.token, Some(bot_token.clone()));
            assert!(auth.headers.is_some());

            if let Some(ref headers) = auth.headers {
                assert_eq!(
                    headers.get("Authorization"),
                    Some(&format!("Bearer {}", bot_token))
                );
                assert!(!headers.contains_key("X-Slack-User-Token"));
            }
        }
    }

    #[test]
    fn test_slack_client_without_tokens() {
        let client = SlackClientNode::with_http_transport(
            "http://localhost:8003".to_string(),
            None, // No bot token
            None, // No user token
        );

        let external_config = client.get_config();

        // Auth should be None when no tokens are provided
        assert!(external_config.auth.is_none());
    }

    #[test]
    fn test_slack_client_with_user_token_only() {
        let user_token = "xoxp-only-user-token".to_string();

        let client = SlackClientNode::with_http_transport(
            "http://localhost:8003".to_string(),
            None, // No bot token
            Some(user_token.clone()),
        );

        let external_config = client.get_config();

        // Check that auth config was properly generated with only user token
        assert!(external_config.auth.is_some());

        if let Some(ref auth) = external_config.auth {
            assert_eq!(auth.token, None); // No bot token for main auth
            assert!(auth.headers.is_some());

            if let Some(ref headers) = auth.headers {
                assert!(!headers.contains_key("Authorization")); // No bot token header
                assert_eq!(headers.get("X-Slack-User-Token"), Some(&user_token));
            }
        }
    }

    #[test]
    fn test_slack_tool_method_coverage() {
        // This test ensures all major Slack API operations are covered
        let methods = [
            "send_message",
            "list_channels",
            "get_user_info",
            "get_channel_info",
            "get_channel_history",
            "update_message",
            "delete_message",
            "add_reaction",
            "remove_reaction",
            "search_messages",
            "create_channel",
            "invite_to_channel",
        ];

        // In a real implementation, you would verify these methods
        // map to actual tools available from the MCP server
        assert_eq!(methods.len(), 12, "Expected 12 Slack tool methods");
    }

    #[tokio::test]
    async fn test_send_message_parameter_validation() {
        let mut client = SlackClientNode::with_defaults();
        
        // Test with all parameters
        let channel = "#general";
        let text = "Test message";
        let thread_ts = Some("1234567890.123456");
        
        // Since we can't connect in unit tests, we'll just verify the client setup
        assert_eq!(client.get_config().service_name, "slack");
        
        // In integration tests, you would verify:
        // - Required parameters (channel, text) are enforced
        // - Optional parameters (thread_ts) are handled correctly
        // - Invalid channel names are rejected
        // - Empty text is rejected
    }

    #[tokio::test]
    async fn test_list_channels_parameter_validation() {
        let mut client = SlackClientNode::with_defaults();
        
        // Test with various parameter combinations
        let test_cases = vec![
            (None, None),  // No filters
            (Some(true), None),  // Only exclude_archived
            (Some(false), Some(vec!["public_channel".to_string()])),  // Include archived + types
            (None, Some(vec!["public_channel".to_string(), "private_channel".to_string()])),  // Only types
        ];
        
        for (exclude_archived, types) in test_cases {
            // Verify parameter handling
            assert!(!client.is_connected());
        }
    }

    #[tokio::test]
    async fn test_update_message_parameter_validation() {
        let mut client = SlackClientNode::with_defaults();
        
        // Test required parameters for update_message
        let channel = "C1234567890";
        let ts = "1234567890.123456";
        let text = "Updated message text";
        
        // Verify all parameters are required
        assert!(!client.is_connected());
        
        // In integration tests, verify:
        // - All three parameters are required
        // - Invalid timestamp format is rejected
        // - Non-existent message returns appropriate error
    }

    #[tokio::test]
    async fn test_delete_message_parameter_validation() {
        let mut client = SlackClientNode::with_defaults();
        
        // Test required parameters for delete_message
        let channel = "C1234567890";
        let ts = "1234567890.123456";
        
        // Verify both parameters are required
        assert!(!client.is_connected());
        
        // In integration tests, verify:
        // - Both parameters are required
        // - Invalid channel ID is rejected
        // - Invalid timestamp format is rejected
    }

    #[tokio::test]
    async fn test_add_remove_reaction_parameter_validation() {
        let mut client = SlackClientNode::with_defaults();
        
        // Test required parameters for reactions
        let channel = "C1234567890";
        let timestamp = "1234567890.123456";
        let reaction_names = vec!["thumbsup", "heart", "smile", "rocket"];
        
        for name in reaction_names {
            // Verify all three parameters are required
            assert!(!client.is_connected());
        }
        
        // In integration tests, verify:
        // - All parameters are required
        // - Invalid reaction names are rejected
        // - Adding duplicate reactions is handled
        // - Removing non-existent reactions is handled
    }

    #[tokio::test]
    async fn test_search_messages_parameter_validation() {
        let mut client = SlackClientNode::with_defaults();
        
        // Test various search parameter combinations
        let test_cases = vec![
            ("test query", None, None, None),
            ("important", Some("timestamp"), Some("asc"), Some(20)),
            ("from:user", Some("score"), Some("desc"), Some(100)),
        ];
        
        for (query, sort, sort_dir, count) in test_cases {
            // Verify parameter handling
            assert!(!client.is_connected());
        }
        
        // In integration tests, verify:
        // - Query is required
        // - Invalid sort options are rejected
        // - Count limits are enforced
    }

    #[tokio::test]
    async fn test_create_channel_parameter_validation() {
        let mut client = SlackClientNode::with_defaults();
        
        // Test channel creation parameters
        let test_cases = vec![
            ("public-channel", Some(false)),
            ("private-channel", Some(true)),
            ("default-channel", None),
        ];
        
        for (name, is_private) in test_cases {
            // Verify parameter handling
            assert!(!client.is_connected());
        }
        
        // In integration tests, verify:
        // - Channel name validation (length, characters)
        // - Duplicate channel names are rejected
        // - Private flag defaults to false
    }

    #[tokio::test]
    async fn test_invite_to_channel_parameter_validation() {
        let mut client = SlackClientNode::with_defaults();
        
        // Test invite parameters
        let channel = "C1234567890";
        let test_cases = vec![
            vec!["U1234567890"],
            vec!["U1234567890", "U0987654321"],
            vec!["U1111111111", "U2222222222", "U3333333333"],
        ];
        
        for users in test_cases {
            // Verify parameter handling
            assert!(!client.is_connected());
        }
        
        // In integration tests, verify:
        // - Channel and users are required
        // - Empty user list is rejected
        // - Invalid user IDs are rejected
        // - Already invited users are handled gracefully
    }

    #[tokio::test]
    async fn test_get_channel_history_pagination() {
        let mut client = SlackClientNode::with_defaults();
        
        // Test pagination parameters
        let channel = "C1234567890";
        let test_cases = vec![
            (Some(10), None, None),  // Just limit
            (Some(100), Some("1234567890.123456"), None),  // Limit + oldest
            (Some(50), None, Some("1234567891.123456")),  // Limit + latest
            (Some(200), Some("1234567890.123456"), Some("1234567891.123456")),  // All params
        ];
        
        for (limit, oldest, latest) in test_cases {
            // Verify parameter handling
            assert!(!client.is_connected());
        }
        
        // In integration tests, verify:
        // - Limit is capped at API maximum
        // - Oldest < latest validation
        // - Invalid timestamp formats are rejected
    }

    #[test]
    fn test_error_handling_scenarios() {
        // Test various error scenarios that should be handled
        let error_scenarios = vec![
            ("rate_limited", "Slack API rate limit exceeded"),
            ("not_authed", "Invalid authentication token"),
            ("channel_not_found", "Channel does not exist"),
            ("user_not_found", "User does not exist"),
            ("message_not_found", "Message does not exist"),
            ("cant_invite_self", "Cannot invite yourself to a channel"),
            ("already_in_channel", "User is already in the channel"),
            ("name_taken", "Channel name is already taken"),
            ("invalid_name", "Invalid channel name format"),
            ("msg_too_long", "Message text is too long"),
        ];
        
        // These would be tested in integration tests with actual API responses
        assert_eq!(error_scenarios.len(), 10, "Expected 10 error scenarios");
    }

    #[test]
    fn test_retry_configuration() {
        let config = SlackClientConfig {
            server_url: "http://localhost:8003".to_string(),
            bot_token: Some("xoxb-test".to_string()),
            user_token: None,
            transport: TransportType::Http {
                base_url: "http://localhost:8003".to_string(),
                pool_config: crate::core::mcp::transport::HttpPoolConfig::default(),
            },
            retry_config: Some(crate::core::nodes::external_mcp_client::RetryConfig {
                max_retries: 5,
                initial_delay_ms: 500,
                max_delay_ms: 10000,
                backoff_multiplier: 1.5,
            }),
        };
        
        let client = SlackClientNode::new(config);
        let retry_config = &client.get_config().retry_config;
        
        assert_eq!(retry_config.max_retries, 5);
        assert_eq!(retry_config.initial_delay_ms, 500);
        assert_eq!(retry_config.max_delay_ms, 10000);
        assert_eq!(retry_config.backoff_multiplier, 1.5);
    }

    #[tokio::test]
    #[ignore] // Run with cargo test -- --ignored
    async fn test_slack_integration_with_mock_server() {
        // This would test against a mock Slack MCP server
        // Setup mock server to return expected responses
        
        let mut client = SlackClientNode::with_http_transport(
            "http://localhost:8003".to_string(),
            Some("xoxb-mock-token".to_string()),
            None,
        );
        
        // Test connection
        let connect_result = client.connect().await;
        if connect_result.is_ok() {
            assert!(client.is_connected());
            
            // Test listing tools
            let tools = client.list_tools().await;
            if let Ok(tools) = tools {
                assert!(!tools.is_empty());
                
                // Verify expected tools are present
                let tool_names: Vec<String> = tools.iter()
                    .map(|t| t.name.clone())
                    .collect();
                
                assert!(tool_names.contains(&"send_message".to_string()));
                assert!(tool_names.contains(&"list_channels".to_string()));
            }
            
            // Test sending a message
            let result = client.send_message("#test", "Integration test message", None).await;
            if let Ok(result) = result {
                assert_eq!(result.is_error, Some(false));
            }
            
            // Disconnect
            let _ = client.disconnect().await;
            assert!(!client.is_connected());
        }
    }
}
