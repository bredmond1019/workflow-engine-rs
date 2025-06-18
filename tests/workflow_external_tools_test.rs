#[cfg(test)]
mod tests {
    use backend::core::workflow::builder::WorkflowBuilder;
    use backend::core::nodes::registry::NodeRegistry;
    use backend::core::nodes::agent::AgentNode;
    use backend::core::nodes::external_mcp_client::ExternalMcpClientNode;
    use backend::core::nodes::config::NodeConfig;
    use backend::core::mcp::clients::slack::SlackClient;
    use backend::core::mcp::clients::helpscout::HelpScoutClient;
    use backend::core::mcp::clients::notion::NotionClient;
    use backend::core::task::TaskContext;
    use backend::db::event::NewEvent;
    use serde_json::{json, Value};
    use std::any::TypeId;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    /// Test workflow execution with Slack integration
    #[tokio::test]
    #[ignore] // Requires Slack MCP server
    async fn test_workflow_with_slack_tools() {
        let mut registry = NodeRegistry::new();
        registry.register_node::<AgentNode>().unwrap();
        registry.register_node::<ExternalMcpClientNode>().unwrap();
        
        // Build workflow with Slack tools
        let workflow = WorkflowBuilder::new::<AgentNode>("slack_workflow".to_string())
            .description("Workflow using Slack tools".to_string())
            .add_node(
                NodeConfig::new::<AgentNode>()
                    .with_description("Prepare Slack message".to_string())
                    .with_connections(vec![TypeId::of::<ExternalMcpClientNode>()])
            )
            .add_node(
                NodeConfig::new::<ExternalMcpClientNode>()
                    .with_description("Send Slack message".to_string())
                    .with_config(json!({
                        "service": "slack",
                        "tool": "send_message",
                        "endpoint": "http://localhost:8003",
                        "transport": "http"
                    }))
                    .with_connections(vec![TypeId::of::<ExternalMcpClientNode>()])
            )
            .add_node(
                NodeConfig::new::<ExternalMcpClientNode>()
                    .with_description("Create Slack channel".to_string())
                    .with_config(json!({
                        "service": "slack",
                        "tool": "create_channel",
                        "endpoint": "http://localhost:8003",
                        "transport": "http"
                    }))
                    .with_connections(vec![TypeId::of::<ExternalMcpClientNode>()])
            )
            .add_node(
                NodeConfig::new::<ExternalMcpClientNode>()
                    .with_description("Add users to channel".to_string())
                    .with_config(json!({
                        "service": "slack",
                        "tool": "add_users_to_channel",
                        "endpoint": "http://localhost:8003",
                        "transport": "http"
                    }))
            )
            .build()
            .unwrap();
        
        let context = json!({
            "channel_name": "test-workflow-channel",
            "message": "Workflow test message",
            "users": ["user1@example.com", "user2@example.com"]
        });
        
        let result = workflow.run(context).await;
        
        assert!(result.is_ok(), "Slack workflow should complete successfully");
        let workflow_context = result.unwrap();
        
        // Verify all Slack operations completed
        assert_eq!(workflow_context.nodes.len(), 4, "All nodes should execute");
        
        // Check for any errors in node execution
        for (node_id, node_result) in &workflow_context.nodes {
            if let Some(error) = node_result.get("error") {
                panic!("Node {} failed: {}", node_id, error);
            }
        }
        
        println!("Slack workflow completed successfully");
    }

    /// Test workflow execution with HelpScout integration
    #[tokio::test]
    #[ignore] // Requires HelpScout MCP server
    async fn test_workflow_with_helpscout_tools() {
        let mut registry = NodeRegistry::new();
        registry.register_node::<AgentNode>().unwrap();
        registry.register_node::<ExternalMcpClientNode>().unwrap();
        
        let workflow = WorkflowBuilder::new::<AgentNode>("helpscout_workflow".to_string())
            .description("Customer support workflow with HelpScout".to_string())
            .add_node(
                NodeConfig::new::<AgentNode>()
                    .with_description("Analyze support ticket".to_string())
                    .with_connections(vec![TypeId::of::<ExternalMcpClientNode>()])
            )
            .add_node(
                NodeConfig::new::<ExternalMcpClientNode>()
                    .with_description("Search existing conversations".to_string())
                    .with_config(json!({
                        "service": "helpscout",
                        "tool": "list_conversations",
                        "endpoint": "http://localhost:8001",
                        "transport": "http"
                    }))
                    .with_connections(vec![TypeId::of::<ExternalMcpClientNode>()])
            )
            .add_node(
                NodeConfig::new::<ExternalMcpClientNode>()
                    .with_description("Create new conversation".to_string())
                    .with_config(json!({
                        "service": "helpscout",
                        "tool": "create_conversation",
                        "endpoint": "http://localhost:8001",
                        "transport": "http"
                    }))
                    .with_connections(vec![TypeId::of::<ExternalMcpClientNode>()])
            )
            .add_node(
                NodeConfig::new::<ExternalMcpClientNode>()
                    .with_description("Add internal note".to_string())
                    .with_config(json!({
                        "service": "helpscout",
                        "tool": "add_note",
                        "endpoint": "http://localhost:8001",
                        "transport": "http"
                    }))
            )
            .build()
            .unwrap();
        
        let context = json!({
            "customer_email": "customer@example.com",
            "subject": "Technical issue with login",
            "message": "I cannot log into my account",
            "priority": "high",
            "tags": ["login", "technical", "urgent"]
        });
        
        let result = workflow.run(context).await;
        
        assert!(result.is_ok(), "HelpScout workflow should complete");
        
        let workflow_context = result.unwrap();
        
        // Verify conversation was created
        let create_node = workflow_context.nodes.iter()
            .find(|(_, result)| result.get("tool") == Some(&json!("create_conversation")))
            .expect("Create conversation node should exist");
        
        assert!(create_node.1.get("conversation_id").is_some(), 
                "Should have created conversation ID");
        
        println!("HelpScout workflow completed successfully");
    }

    /// Test workflow execution with Notion integration
    #[tokio::test]
    #[ignore] // Requires Notion MCP server
    async fn test_workflow_with_notion_tools() {
        let mut registry = NodeRegistry::new();
        registry.register_node::<AgentNode>().unwrap();
        registry.register_node::<ExternalMcpClientNode>().unwrap();
        
        let workflow = WorkflowBuilder::new::<AgentNode>("notion_workflow".to_string())
            .description("Knowledge management workflow with Notion".to_string())
            .add_node(
                NodeConfig::new::<AgentNode>()
                    .with_description("Prepare content for Notion".to_string())
                    .with_connections(vec![TypeId::of::<ExternalMcpClientNode>()])
            )
            .add_node(
                NodeConfig::new::<ExternalMcpClientNode>()
                    .with_description("Search existing pages".to_string())
                    .with_config(json!({
                        "service": "notion",
                        "tool": "search_pages",
                        "endpoint": "http://localhost:8002",
                        "transport": "http"
                    }))
                    .with_connections(vec![TypeId::of::<ExternalMcpClientNode>()])
            )
            .add_node(
                NodeConfig::new::<ExternalMcpClientNode>()
                    .with_description("Create knowledge base page".to_string())
                    .with_config(json!({
                        "service": "notion",
                        "tool": "create_page",
                        "endpoint": "http://localhost:8002",
                        "transport": "http"
                    }))
                    .with_connections(vec![TypeId::of::<ExternalMcpClientNode>()])
            )
            .add_node(
                NodeConfig::new::<ExternalMcpClientNode>()
                    .with_description("Add content blocks".to_string())
                    .with_config(json!({
                        "service": "notion",
                        "tool": "append_blocks",
                        "endpoint": "http://localhost:8002",
                        "transport": "http"
                    }))
            )
            .build()
            .unwrap();
        
        let context = json!({
            "title": "Workflow Test Documentation",
            "content": "This page was created by an automated workflow test",
            "tags": ["test", "workflow", "automation"],
            "parent_page": "knowledge_base_root"
        });
        
        let result = workflow.run(context).await;
        
        assert!(result.is_ok(), "Notion workflow should complete");
        
        let workflow_context = result.unwrap();
        
        // Verify page was created
        let create_node = workflow_context.nodes.iter()
            .find(|(_, result)| result.get("tool") == Some(&json!("create_page")))
            .expect("Create page node should exist");
        
        assert!(create_node.1.get("page_id").is_some(), 
                "Should have created page ID");
        
        println!("Notion workflow completed successfully");
    }

    /// Test workflow with mixed external tool usage
    #[tokio::test]
    #[ignore] // Requires all MCP servers
    async fn test_workflow_with_mixed_external_tools() {
        let mut registry = NodeRegistry::new();
        registry.register_node::<AgentNode>().unwrap();
        registry.register_node::<ExternalMcpClientNode>().unwrap();
        
        let workflow = WorkflowBuilder::new::<AgentNode>("mixed_tools_workflow".to_string())
            .description("Workflow using multiple external services".to_string())
            .add_node(
                NodeConfig::new::<AgentNode>()
                    .with_description("Initial analysis".to_string())
                    .with_connections(vec![
                        TypeId::of::<ExternalMcpClientNode>(),
                        TypeId::of::<ExternalMcpClientNode>(),
                        TypeId::of::<ExternalMcpClientNode>()
                    ])
            )
            // Parallel execution of different tools
            .add_node(
                NodeConfig::new::<ExternalMcpClientNode>()
                    .with_description("Search Notion KB".to_string())
                    .with_config(json!({
                        "service": "notion",
                        "tool": "search_pages",
                        "endpoint": "http://localhost:8002"
                    }))
            )
            .add_node(
                NodeConfig::new::<ExternalMcpClientNode>()
                    .with_description("Check HelpScout tickets".to_string())
                    .with_config(json!({
                        "service": "helpscout",
                        "tool": "list_conversations",
                        "endpoint": "http://localhost:8001"
                    }))
            )
            .add_node(
                NodeConfig::new::<ExternalMcpClientNode>()
                    .with_description("Notify on Slack".to_string())
                    .with_config(json!({
                        "service": "slack",
                        "tool": "send_message",
                        "endpoint": "http://localhost:8003"
                    }))
            )
            .build()
            .unwrap();
        
        let context = json!({
            "query": "system performance issues",
            "customer_id": "CUST-MIXED-001",
            "notify_channel": "#support-alerts"
        });
        
        let start_time = std::time::Instant::now();
        let result = workflow.run(context).await;
        let execution_time = start_time.elapsed();
        
        assert!(result.is_ok(), "Mixed tools workflow should complete");
        
        let workflow_context = result.unwrap();
        assert_eq!(workflow_context.nodes.len(), 4, "All nodes should execute");
        
        // Verify parallel execution was efficient
        assert!(execution_time.as_secs() < 10, 
                "Parallel execution should be faster than sequential");
        
        println!("Mixed tools workflow completed in {:?}", execution_time);
    }

    /// Test workflow error handling with external tools
    #[tokio::test]
    async fn test_workflow_external_tool_error_handling() {
        let mut registry = NodeRegistry::new();
        registry.register_node::<AgentNode>().unwrap();
        registry.register_node::<ExternalMcpClientNode>().unwrap();
        
        // Create workflow with intentional failure points
        let workflow = WorkflowBuilder::new::<AgentNode>("error_handling_workflow".to_string())
            .description("Test error handling with external tools".to_string())
            .add_node(
                NodeConfig::new::<AgentNode>()
                    .with_description("Initial processing".to_string())
                    .with_connections(vec![TypeId::of::<ExternalMcpClientNode>()])
            )
            .add_node(
                NodeConfig::new::<ExternalMcpClientNode>()
                    .with_description("Failing external call".to_string())
                    .with_config(json!({
                        "service": "invalid_service",
                        "tool": "invalid_tool",
                        "endpoint": "http://localhost:9999", // Non-existent
                        "transport": "http"
                    }))
                    .with_connections(vec![TypeId::of::<AgentNode>()])
            )
            .add_node(
                NodeConfig::new::<AgentNode>()
                    .with_description("Error recovery handler".to_string())
            )
            .build()
            .unwrap();
        
        let context = json!({
            "test": "error_handling"
        });
        
        let result = workflow.run(context).await;
        
        // The workflow should handle the error gracefully
        // Depending on implementation, it might return an error or continue with error logged
        if let Ok(workflow_context) = result {
            // Check if error was captured in the workflow context
            let failed_node = workflow_context.nodes.iter()
                .find(|(_, result)| result.get("error").is_some());
            
            assert!(failed_node.is_some(), "Should capture node failure");
            
            println!("Error handling verified: {:?}", 
                     failed_node.unwrap().1.get("error"));
        } else {
            println!("Workflow failed as expected: {:?}", result.err());
        }
    }

    /// Test workflow with conditional external tool execution
    #[tokio::test]
    #[ignore] // Requires MCP servers
    async fn test_workflow_conditional_tool_execution() {
        let mut registry = NodeRegistry::new();
        registry.register_node::<AgentNode>().unwrap();
        registry.register_node::<ExternalMcpClientNode>().unwrap();
        
        let workflow = WorkflowBuilder::new::<AgentNode>("conditional_workflow".to_string())
            .description("Workflow with conditional tool execution".to_string())
            .add_node(
                NodeConfig::new::<AgentNode>()
                    .with_description("Evaluate conditions".to_string())
                    .with_connections(vec![TypeId::of::<ExternalMcpClientNode>()])
            )
            .add_node(
                NodeConfig::new::<ExternalMcpClientNode>()
                    .with_description("Conditional Slack notification".to_string())
                    .with_config(json!({
                        "service": "slack",
                        "tool": "send_message",
                        "endpoint": "http://localhost:8003",
                        "condition": {
                            "field": "priority",
                            "operator": "equals",
                            "value": "high"
                        }
                    }))
            )
            .build()
            .unwrap();
        
        // Test with high priority (should execute Slack tool)
        let high_priority_context = json!({
            "priority": "high",
            "message": "High priority alert"
        });
        
        let result1 = workflow.run(high_priority_context).await;
        assert!(result1.is_ok(), "High priority workflow should complete");
        
        // Test with low priority (should skip Slack tool)
        let low_priority_context = json!({
            "priority": "low",
            "message": "Low priority message"
        });
        
        let result2 = workflow.run(low_priority_context).await;
        assert!(result2.is_ok(), "Low priority workflow should complete");
        
        println!("Conditional tool execution tested successfully");
    }
}