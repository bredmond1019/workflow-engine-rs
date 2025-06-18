#[cfg(test)]
mod tests {
    use backend::core::mcp::protocol::{
        McpRequest, McpResponse, ToolCall, ToolCallResult,
        ProtocolVersion, Capability, InitializeRequest, InitializeResponse
    };
    use backend::core::mcp::transport::{Transport, TransportType, HttpTransport, WebSocketTransport};
    use backend::core::mcp::connection_pool::{ConnectionPool, ConnectionPoolConfig};
    use backend::core::mcp::clients::slack::SlackClient;
    use backend::core::mcp::clients::helpscout::HelpScoutClient;
    use backend::core::mcp::clients::notion::NotionClient;
    use serde_json::{json, Value};
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::time::{sleep, timeout};

    /// Test basic MCP client to server handshake and initialization
    #[tokio::test]
    #[ignore] // Requires MCP servers running
    async fn test_mcp_client_server_handshake() {
        // Test with each transport type
        let endpoints = vec![
            ("http://localhost:8001", TransportType::Http),
            ("ws://localhost:8001/ws", TransportType::WebSocket),
        ];
        
        for (endpoint, transport_type) in endpoints {
            println!("Testing handshake with {} transport", transport_type);
            
            let transport: Box<dyn Transport> = match transport_type {
                TransportType::Http => Box::new(HttpTransport::new(endpoint.to_string())),
                TransportType::WebSocket => Box::new(WebSocketTransport::new(endpoint.to_string()).await.unwrap()),
                _ => panic!("Unsupported transport type for test"),
            };
            
            // Send initialization request
            let init_request = McpRequest::Initialize(InitializeRequest {
                protocol_version: ProtocolVersion { major: 1, minor: 0 },
                capabilities: vec![
                    Capability::ToolExecution,
                    Capability::ResourceSubscription,
                ],
                client_info: json!({
                    "name": "test_client",
                    "version": "1.0.0"
                }),
            });
            
            let response = transport.send_request(init_request).await;
            assert!(response.is_ok(), "Handshake should succeed");
            
            match response.unwrap() {
                McpResponse::Initialize(init_resp) => {
                    assert_eq!(init_resp.protocol_version.major, 1);
                    assert!(!init_resp.capabilities.is_empty());
                    assert!(init_resp.server_info.get("name").is_some());
                }
                _ => panic!("Expected Initialize response"),
            }
        }
    }

    /// Test connection pooling with multiple concurrent requests
    #[tokio::test]
    #[ignore] // Requires MCP servers
    async fn test_mcp_connection_pool_concurrent_requests() {
        let config = ConnectionPoolConfig {
            max_connections: 5,
            min_idle: 2,
            connection_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(60),
            retry_attempts: 3,
            retry_delay: Duration::from_millis(100),
        };
        
        let pool = Arc::new(ConnectionPool::new(config));
        
        // Spawn multiple concurrent requests
        let mut handles = vec![];
        
        for i in 0..10 {
            let pool_clone = pool.clone();
            let handle = tokio::spawn(async move {
                let transport = pool_clone
                    .get_connection("http://localhost:8001", TransportType::Http)
                    .await
                    .unwrap();
                
                let request = McpRequest::ToolCall(ToolCall {
                    id: format!("test_{}", i),
                    name: "list_conversations".to_string(),
                    arguments: json!({
                        "status": "active",
                        "limit": 5
                    }),
                });
                
                let result = transport.send_request(request).await;
                assert!(result.is_ok(), "Request {} should succeed", i);
                
                result
            });
            
            handles.push(handle);
        }
        
        // Wait for all requests to complete
        let results = futures::future::join_all(handles).await;
        
        // Verify all requests succeeded
        for (i, result) in results.iter().enumerate() {
            assert!(result.is_ok(), "Task {} should complete", i);
            let response = result.as_ref().unwrap();
            assert!(response.is_ok(), "Request {} should get valid response", i);
        }
        
        // Check pool statistics
        let stats = pool.get_stats().await;
        assert!(stats.total_connections <= 5, "Should respect max connections");
        assert!(stats.idle_connections >= 2, "Should maintain min idle connections");
        
        println!("Connection pool handled {} concurrent requests", results.len());
    }

    /// Test request/response correlation across multiple services
    #[tokio::test]
    #[ignore] // Requires MCP servers
    async fn test_mcp_request_correlation() {
        let services = vec![
            ("helpscout", "http://localhost:8001"),
            ("notion", "http://localhost:8002"),
            ("slack", "http://localhost:8003"),
        ];
        
        for (service_name, endpoint) in services {
            let transport = HttpTransport::new(endpoint.to_string());
            
            // Send multiple requests with unique IDs
            let request_ids = vec!["req_001", "req_002", "req_003"];
            let mut responses = vec![];
            
            for req_id in &request_ids {
                let request = McpRequest::ToolCall(ToolCall {
                    id: req_id.to_string(),
                    name: match service_name {
                        "helpscout" => "list_conversations",
                        "notion" => "search_pages",
                        "slack" => "list_channels",
                        _ => "unknown",
                    }.to_string(),
                    arguments: json!({}),
                });
                
                let response = transport.send_request(request).await.unwrap();
                responses.push((req_id, response));
            }
            
            // Verify responses are correlated correctly
            for (req_id, response) in responses {
                match response {
                    McpResponse::ToolCall(result) => {
                        assert_eq!(result.id, *req_id, "Response ID should match request ID");
                    }
                    _ => panic!("Expected ToolCall response"),
                }
            }
            
            println!("Request correlation verified for {}", service_name);
        }
    }

    /// Test error propagation from server to client
    #[tokio::test]
    #[ignore] // Requires MCP servers
    async fn test_mcp_error_propagation() {
        let transport = HttpTransport::new("http://localhost:8001".to_string());
        
        // Test various error scenarios
        let error_scenarios = vec![
            // Invalid tool name
            ToolCall {
                id: "error_1".to_string(),
                name: "invalid_tool_name".to_string(),
                arguments: json!({}),
            },
            // Missing required arguments
            ToolCall {
                id: "error_2".to_string(),
                name: "update_conversation".to_string(),
                arguments: json!({}), // Missing required 'id' field
            },
            // Invalid argument types
            ToolCall {
                id: "error_3".to_string(),
                name: "get_conversation".to_string(),
                arguments: json!({
                    "id": 12345 // Should be string
                }),
            },
        ];
        
        for scenario in error_scenarios {
            let request = McpRequest::ToolCall(scenario.clone());
            let response = transport.send_request(request).await;
            
            match response {
                Ok(McpResponse::ToolCall(result)) => {
                    assert!(result.error.is_some(), "Should return error for scenario {}", scenario.id);
                    println!("Error scenario {} handled correctly: {}", scenario.id, result.error.unwrap());
                }
                Ok(McpResponse::Error(err)) => {
                    println!("Error scenario {} returned error response: {}", scenario.id, err.message);
                }
                _ => panic!("Expected error response for scenario {}", scenario.id),
            }
        }
    }

    /// Test connection resilience and automatic retry
    #[tokio::test]
    async fn test_mcp_connection_resilience() {
        let config = ConnectionPoolConfig {
            max_connections: 3,
            min_idle: 1,
            connection_timeout: Duration::from_secs(2),
            idle_timeout: Duration::from_secs(30),
            retry_attempts: 3,
            retry_delay: Duration::from_millis(500),
        };
        
        let pool = ConnectionPool::new(config);
        
        // Test connection to non-existent server (should retry and fail gracefully)
        let result = timeout(
            Duration::from_secs(10),
            pool.get_connection("http://localhost:9999", TransportType::Http)
        ).await;
        
        assert!(result.is_ok(), "Should complete within timeout");
        assert!(result.unwrap().is_err(), "Should fail to connect to non-existent server");
        
        // Test connection recovery after transient failure
        // This would require a test server that can simulate failures
        println!("Connection resilience test completed");
    }

    /// Test streaming responses from MCP servers
    #[tokio::test]
    #[ignore] // Requires MCP servers with streaming support
    async fn test_mcp_streaming_responses() {
        let transport = HttpTransport::new("http://localhost:8002".to_string());
        
        // Request that triggers streaming response
        let request = McpRequest::ToolCall(ToolCall {
            id: "stream_test".to_string(),
            name: "search_pages".to_string(),
            arguments: json!({
                "query": "test",
                "stream": true,
                "limit": 100
            }),
        });
        
        let start_time = std::time::Instant::now();
        let response = transport.send_request(request).await;
        let elapsed = start_time.elapsed();
        
        assert!(response.is_ok(), "Streaming request should succeed");
        
        match response.unwrap() {
            McpResponse::ToolCall(result) => {
                assert!(result.error.is_none(), "Should not have errors");
                
                // Verify we received multiple results
                if let Some(results) = result.result.as_array() {
                    assert!(!results.is_empty(), "Should receive streamed results");
                    println!("Received {} streamed results in {:?}", results.len(), elapsed);
                }
            }
            _ => panic!("Expected ToolCall response"),
        }
    }

    /// Test protocol version negotiation
    #[tokio::test]
    #[ignore] // Requires MCP servers
    async fn test_mcp_protocol_version_negotiation() {
        let transport = HttpTransport::new("http://localhost:8001".to_string());
        
        // Test different protocol versions
        let versions = vec![
            ProtocolVersion { major: 1, minor: 0 },
            ProtocolVersion { major: 1, minor: 1 },
            ProtocolVersion { major: 2, minor: 0 }, // Future version
        ];
        
        for version in versions {
            let request = McpRequest::Initialize(InitializeRequest {
                protocol_version: version.clone(),
                capabilities: vec![Capability::ToolExecution],
                client_info: json!({
                    "name": "version_test_client",
                    "version": "1.0.0"
                }),
            });
            
            let response = transport.send_request(request).await;
            
            match response {
                Ok(McpResponse::Initialize(init_resp)) => {
                    // Server should negotiate to a compatible version
                    assert!(
                        init_resp.protocol_version.major <= version.major,
                        "Server should not exceed requested major version"
                    );
                    println!("Negotiated protocol version: {:?}", init_resp.protocol_version);
                }
                Ok(McpResponse::Error(err)) => {
                    // Incompatible version
                    println!("Version {:?} rejected: {}", version, err.message);
                }
                _ => panic!("Unexpected response for version negotiation"),
            }
        }
    }

    /// Test request timeout handling
    #[tokio::test]
    #[ignore] // Requires MCP servers
    async fn test_mcp_request_timeout() {
        let transport = HttpTransport::new("http://localhost:8001".to_string());
        
        // Create a request that might take long to process
        let request = McpRequest::ToolCall(ToolCall {
            id: "timeout_test".to_string(),
            name: "list_conversations".to_string(),
            arguments: json!({
                "limit": 10000, // Large limit to potentially slow down response
                "include_details": true
            }),
        });
        
        // Set a short timeout
        let result = timeout(
            Duration::from_millis(100), // Very short timeout
            transport.send_request(request)
        ).await;
        
        match result {
            Err(_) => {
                println!("Request timed out as expected");
            }
            Ok(Ok(_)) => {
                println!("Request completed within timeout");
            }
            Ok(Err(e)) => {
                println!("Request failed with error: {:?}", e);
            }
        }
    }
}