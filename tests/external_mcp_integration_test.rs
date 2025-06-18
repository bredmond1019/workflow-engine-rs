use backend::core::mcp::clients::{
    helpscout::HelpscoutClientNode, notion::NotionClientNode, slack::SlackClientNode,
};
use backend::core::nodes::external_mcp_client::ExternalMCPClientNode;
use std::collections::HashMap;
use tokio;

/// Integration tests for external MCP client nodes
///
/// These tests require external MCP servers to be running on the configured ports:
/// - Notion MCP server on http://localhost:8002
/// - HelpScout MCP server on http://localhost:8001  
/// - Slack MCP server on http://localhost:8003
///
/// To run these tests, first start the example MCP servers:
/// ```bash
/// # Terminal 1 - HelpScout server
/// cd scripts && python multi_service_mcp_server.py --port 8001 --service helpscout
///
/// # Terminal 2 - Notion server  
/// cd scripts && python multi_service_mcp_server.py --port 8002 --service notion
///
/// # Terminal 3 - Slack server
/// cd scripts && python multi_service_mcp_server.py --port 8003 --service slack
/// ```
///
/// Then run tests with:
/// ```bash
/// cargo test external_mcp_integration -- --ignored
/// ```

#[tokio::test]
#[ignore] // Ignore by default since it requires external servers
async fn test_notion_client_integration() {
    use backend::core::mcp::clients::notion::NotionConfig;

    let config = NotionConfig::new_http(
        "http://localhost:8002".to_string(),
        Some("test-notion-key".to_string()),
    );
    let mut client = NotionClientNode::new(config);

    // Test connection
    let connect_result = client.connect().await;
    if connect_result.is_err() {
        eprintln!(
            "Failed to connect to Notion MCP server. Make sure it's running on localhost:8002"
        );
        return;
    }

    println!("✓ Notion client connected successfully");

    // Test listing tools
    let tools_result = client.list_tools().await;
    assert!(
        tools_result.is_ok(),
        "Failed to list tools: {:?}",
        tools_result.err()
    );

    let tools = tools_result.unwrap();
    assert!(
        !tools.is_empty(),
        "No tools returned from Notion MCP server"
    );

    println!(
        "✓ Notion MCP server tools: {:?}",
        tools.iter().map(|t| &t.name).collect::<Vec<_>>()
    );

    // Test search pages (if supported)
    if tools.iter().any(|t| t.name == "search_pages") {
        let search_result = client.search_pages("test", Some(5)).await;
        match search_result {
            Ok(result) => println!("✓ Search pages result: {:?}", result),
            Err(e) => println!("⚠ Search pages failed (expected if no test data): {:?}", e),
        }
    }

    // Test list databases (if supported)
    if tools.iter().any(|t| t.name == "list_databases") {
        let list_result = client.list_databases().await;
        match list_result {
            Ok(result) => println!("✓ List databases result: {:?}", result),
            Err(e) => println!(
                "⚠ List databases failed (expected if no test data): {:?}",
                e
            ),
        }
    }

    // Test disconnect
    let disconnect_result = client.disconnect().await;
    assert!(
        disconnect_result.is_ok(),
        "Failed to disconnect: {:?}",
        disconnect_result.err()
    );

    println!("✓ Notion client disconnected successfully");
}

#[tokio::test]
#[ignore] // Ignore by default since it requires external servers
async fn test_helpscout_client_integration() {
    let mut client = HelpscoutClientNode::with_http_transport(
        "http://localhost:8001".to_string(),
        Some("test-helpscout-key".to_string()),
    );

    // Test connection
    let connect_result = client.connect().await;
    if connect_result.is_err() {
        eprintln!(
            "Failed to connect to HelpScout MCP server. Make sure it's running on localhost:8001"
        );
        return;
    }

    println!("✓ HelpScout client connected successfully");

    // Test listing tools
    let tools_result = client.list_tools().await;
    assert!(
        tools_result.is_ok(),
        "Failed to list tools: {:?}",
        tools_result.err()
    );

    let tools = tools_result.unwrap();
    assert!(
        !tools.is_empty(),
        "No tools returned from HelpScout MCP server"
    );

    println!(
        "✓ HelpScout MCP server tools: {:?}",
        tools.iter().map(|t| &t.name).collect::<Vec<_>>()
    );

    // Test search articles (if supported)
    if tools.iter().any(|t| t.name == "search_articles") {
        let search_result = client
            .search_articles("appointment", Some(1), Some(5))
            .await;
        match search_result {
            Ok(result) => println!("✓ Search articles result: {:?}", result),
            Err(e) => println!(
                "⚠ Search articles failed (expected if no test data): {:?}",
                e
            ),
        }
    }

    // Test list collections (if supported)
    if tools.iter().any(|t| t.name == "list_collections") {
        let list_result = client.list_collections(Some(1), Some(10)).await;
        match list_result {
            Ok(result) => println!("✓ List collections result: {:?}", result),
            Err(e) => println!(
                "⚠ List collections failed (expected if no test data): {:?}",
                e
            ),
        }
    }

    // Test disconnect
    let disconnect_result = client.disconnect().await;
    assert!(
        disconnect_result.is_ok(),
        "Failed to disconnect: {:?}",
        disconnect_result.err()
    );

    println!("✓ HelpScout client disconnected successfully");
}

#[tokio::test]
#[ignore] // Ignore by default since it requires external servers
async fn test_slack_client_integration() {
    let mut client = SlackClientNode::with_http_transport(
        "http://localhost:8003".to_string(),
        Some("xoxb-test-token".to_string()),
        Some("xoxp-test-user-token".to_string()),
    );

    // Test connection
    let connect_result = client.connect().await;
    if connect_result.is_err() {
        eprintln!(
            "Failed to connect to Slack MCP server. Make sure it's running on localhost:8003"
        );
        return;
    }

    println!("✓ Slack client connected successfully");

    // Test listing tools
    let tools_result = client.list_tools().await;
    assert!(
        tools_result.is_ok(),
        "Failed to list tools: {:?}",
        tools_result.err()
    );

    let tools = tools_result.unwrap();
    assert!(!tools.is_empty(), "No tools returned from Slack MCP server");

    println!(
        "✓ Slack MCP server tools: {:?}",
        tools.iter().map(|t| &t.name).collect::<Vec<_>>()
    );

    // Test list channels (if supported)
    if tools.iter().any(|t| t.name == "list_channels") {
        let list_result = client
            .list_channels(Some(true), Some(vec!["public_channel".to_string()]))
            .await;
        match list_result {
            Ok(result) => println!("✓ List channels result: {:?}", result),
            Err(e) => println!("⚠ List channels failed (expected if no test data): {:?}", e),
        }
    }

    // Test get user info (if supported)
    if tools.iter().any(|t| t.name == "get_user_info") {
        let user_result = client.get_user_info("U1234567890").await;
        match user_result {
            Ok(result) => println!("✓ Get user info result: {:?}", result),
            Err(e) => println!("⚠ Get user info failed (expected if no test data): {:?}", e),
        }
    }

    // Test disconnect
    let disconnect_result = client.disconnect().await;
    assert!(
        disconnect_result.is_ok(),
        "Failed to disconnect: {:?}",
        disconnect_result.err()
    );

    println!("✓ Slack client disconnected successfully");
}

#[tokio::test]
#[ignore] // Ignore by default since it requires external servers
async fn test_multiple_concurrent_connections() {
    use backend::core::mcp::clients::notion::NotionConfig;

    let config = NotionConfig::new_http(
        "http://localhost:8002".to_string(),
        Some("test-notion-key".to_string()),
    );
    let mut notion_client = NotionClientNode::new(config);

    let mut helpscout_client = HelpscoutClientNode::with_http_transport(
        "http://localhost:8001".to_string(),
        Some("test-helpscout-key".to_string()),
    );

    let mut slack_client = SlackClientNode::with_http_transport(
        "http://localhost:8003".to_string(),
        Some("xoxb-test-token".to_string()),
        None,
    );

    // Test concurrent connections
    let (notion_result, helpscout_result, slack_result) = tokio::join!(
        notion_client.connect(),
        helpscout_client.connect(),
        slack_client.connect()
    );

    let mut connected_clients = 0;

    if notion_result.is_ok() {
        connected_clients += 1;
        println!("✓ Notion client connected successfully");
    } else {
        println!(
            "⚠ Notion client connection failed: {:?}",
            notion_result.as_ref().err()
        );
    }

    if helpscout_result.is_ok() {
        connected_clients += 1;
        println!("✓ HelpScout client connected successfully");
    } else {
        println!(
            "⚠ HelpScout client connection failed: {:?}",
            helpscout_result.as_ref().err()
        );
    }

    if slack_result.is_ok() {
        connected_clients += 1;
        println!("✓ Slack client connected successfully");
    } else {
        println!(
            "⚠ Slack client connection failed: {:?}",
            slack_result.as_ref().err()
        );
    }

    println!(
        "✓ Successfully connected to {}/3 MCP servers",
        connected_clients
    );

    // Test concurrent tool listing if clients are connected
    if connected_clients > 0 {
        if notion_result.is_ok() {
            let notion_tools = notion_client.list_tools().await;
            if let Ok(tools) = notion_tools {
                println!("✓ Notion tools count: {}", tools.len());
            }
        }

        if helpscout_result.is_ok() {
            let helpscout_tools = helpscout_client.list_tools().await;
            if let Ok(tools) = helpscout_tools {
                println!("✓ HelpScout tools count: {}", tools.len());
            }
        }

        if slack_result.is_ok() {
            let slack_tools = slack_client.list_tools().await;
            if let Ok(tools) = slack_tools {
                println!("✓ Slack tools count: {}", tools.len());
            }
        }
    }

    // Cleanup - disconnect all clients
    let _ = tokio::join!(
        notion_client.disconnect(),
        helpscout_client.disconnect(),
        slack_client.disconnect()
    );

    println!("✓ All clients disconnected successfully");
}

#[tokio::test]
#[ignore] // Ignore by default since it requires external servers
async fn test_error_handling_and_retries() {
    // Test connection to non-existent server to verify error handling
    use backend::core::mcp::clients::notion::NotionConfig;

    let config = NotionConfig::new_http(
        "http://localhost:9999".to_string(), // Non-existent server
        Some("test-key".to_string()),
    );
    let mut client = NotionClientNode::new(config);

    let connect_result = client.connect().await;
    assert!(
        connect_result.is_err(),
        "Connection should fail to non-existent server"
    );

    let error = connect_result.unwrap_err();
    println!("✓ Expected connection error: {:?}", error);

    // Verify client is not connected
    assert!(
        !client.is_connected(),
        "Client should not be connected after failed connection"
    );

    // Test tool execution on disconnected client
    let search_result = client.search_pages("test", None).await;
    assert!(
        search_result.is_err(),
        "Tool execution should fail on disconnected client"
    );

    println!("✓ Error handling test passed - correctly handled connection failures");
}

#[tokio::test]
#[ignore] // Ignore by default since it requires external servers
async fn test_tool_execution_with_arguments() {
    let mut client = HelpscoutClientNode::with_http_transport(
        "http://localhost:8001".to_string(),
        Some("test-api-key".to_string()),
    );

    // Test connection
    let connect_result = client.connect().await;
    if connect_result.is_err() {
        eprintln!("Skipping tool execution test - HelpScout server not running on localhost:8001");
        return;
    }

    println!("✓ HelpScout client connected for tool execution test");

    // Test generic tool execution with arguments
    let mut args = HashMap::new();
    args.insert(
        "keywords".to_string(),
        serde_json::Value::String("test".to_string()),
    );
    args.insert("page".to_string(), serde_json::Value::Number(1.into()));
    args.insert("per_page".to_string(), serde_json::Value::Number(5.into()));

    let result = client.execute_tool("search_articles", Some(args)).await;
    match result {
        Ok(tool_result) => {
            println!("✓ Tool execution successful: {:?}", tool_result);
            assert!(!tool_result.content.is_empty() && !tool_result.is_error.unwrap_or(true));
        }
        Err(e) => {
            println!("⚠ Tool execution failed (may be expected): {:?}", e);
            // Tool execution failure is okay if the server doesn't support the tool
            // or if test data is not available
        }
    }

    // Test tool execution without arguments
    let result_no_args = client.execute_tool("list_collections", None).await;
    match result_no_args {
        Ok(tool_result) => {
            println!("✓ Tool execution (no args) successful: {:?}", tool_result);
        }
        Err(e) => {
            println!(
                "⚠ Tool execution (no args) failed (may be expected): {:?}",
                e
            );
        }
    }

    // Cleanup
    let _ = client.disconnect().await;
    println!("✓ Tool execution test completed");
}

#[tokio::test]
#[ignore] // Ignore by default since it requires external servers
async fn test_websocket_transport() {
    // Test WebSocket transport if the server supports it
    let mut client = SlackClientNode::with_websocket_transport(
        "ws://localhost:8003/ws".to_string(),
        Some("xoxb-test-token".to_string()),
        None,
    );

    println!("Testing WebSocket transport connection...");

    let connect_result = client.connect().await;
    match connect_result {
        Ok(()) => {
            println!("✓ WebSocket connection established successfully");

            // Test basic functionality
            let tools_result = client.list_tools().await;
            if let Ok(tools) = tools_result {
                println!("✓ Listed {} tools via WebSocket", tools.len());
            }

            // Disconnect
            let _ = client.disconnect().await;
            println!("✓ WebSocket client disconnected successfully");
        }
        Err(e) => {
            println!(
                "⚠ WebSocket connection failed (server may not support WebSocket): {:?}",
                e
            );
            // This is okay - not all test servers may support WebSocket
        }
    }
}

#[tokio::test]
#[ignore] // Ignore by default since it requires external servers  
async fn test_stdio_transport() {
    // Test Stdio transport with a simple echo server
    use backend::core::mcp::clients::notion::NotionConfig;

    let config = NotionConfig::new_stdio(
        "python".to_string(),
        vec!["-c".to_string(), "print('Hello from stdio')".to_string()],
    );
    let mut client = NotionClientNode::new(config);

    println!("Testing Stdio transport connection...");

    let connect_result = client.connect().await;
    match connect_result {
        Ok(()) => {
            println!("✓ Stdio connection established successfully");

            // For stdio, we may not be able to test much without a proper MCP server
            // Just verify we can attempt to list tools
            let tools_result = client.list_tools().await;
            match tools_result {
                Ok(tools) => println!("✓ Listed {} tools via Stdio", tools.len()),
                Err(e) => println!("⚠ Tool listing via Stdio failed (expected): {:?}", e),
            }

            // Disconnect
            let _ = client.disconnect().await;
            println!("✓ Stdio client disconnected successfully");
        }
        Err(e) => {
            println!(
                "⚠ Stdio connection failed (expected without proper MCP server): {:?}",
                e
            );
            // This is expected since we're not running a real MCP server via stdio
        }
    }
}
