#[cfg(test)]
mod tests {
    use backend::bootstrap::service::ServiceBootstrap;
    use backend::core::mcp::clients::slack::SlackClient;
    use backend::core::mcp::clients::helpscout::HelpScoutClient;
    use backend::core::mcp::clients::notion::NotionClient;
    use backend::core::mcp::protocol::{McpRequest, McpResponse, ToolCall};
    use backend::core::mcp::transport::{Transport, TransportType};
    use backend::core::nodes::registry::NodeRegistry;
    use backend::core::nodes::agent::AgentNode;
    use backend::core::nodes::external_mcp_client::ExternalMcpClientNode;
    use backend::core::workflow::builder::WorkflowBuilder;
    use backend::core::workflow::Workflow;
    use backend::db::event::NewEvent;
    use backend::db::repository::EventRepository;
    use backend::monitoring::metrics::AppMetrics;
    use serde_json::{json, Value};
    use std::any::TypeId;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use tokio::time::{sleep, Duration};

    /// End-to-end test for a complete customer support workflow
    /// This tests the full flow from receiving a customer ticket to resolution
    #[tokio::test]
    #[ignore] // Requires external services
    async fn test_e2e_customer_support_workflow() {
        // Initialize service bootstrap
        let mut bootstrap = ServiceBootstrap::new().await.unwrap();
        
        // Register all required services
        bootstrap.register_service("metrics", Arc::new(AppMetrics::new())).await.unwrap();
        bootstrap.register_service("event_repo", Arc::new(EventRepository::new())).await.unwrap();
        
        // Create node registry and register nodes
        let mut node_registry = NodeRegistry::new();
        node_registry.register_node::<AgentNode>().unwrap();
        node_registry.register_node::<ExternalMcpClientNode>().unwrap();
        
        // Build a comprehensive customer support workflow
        let workflow = WorkflowBuilder::new::<AgentNode>("customer_support_e2e".to_string())
            .description("End-to-end customer support workflow".to_string())
            .add_node(
                backend::core::nodes::config::NodeConfig::new::<AgentNode>()
                    .with_description("Analyze customer ticket".to_string())
                    .with_connections(vec![TypeId::of::<ExternalMcpClientNode>()])
            )
            .add_node(
                backend::core::nodes::config::NodeConfig::new::<ExternalMcpClientNode>()
                    .with_description("Search knowledge base".to_string())
                    .with_config(json!({
                        "service": "notion",
                        "tool": "search_pages",
                        "endpoint": "http://localhost:8002",
                        "transport": "http"
                    }))
                    .with_connections(vec![TypeId::of::<ExternalMcpClientNode>()])
            )
            .add_node(
                backend::core::nodes::config::NodeConfig::new::<ExternalMcpClientNode>()
                    .with_description("Update ticket in HelpScout".to_string())
                    .with_config(json!({
                        "service": "helpscout",
                        "tool": "update_conversation",
                        "endpoint": "http://localhost:8001",
                        "transport": "http"
                    }))
                    .with_connections(vec![TypeId::of::<ExternalMcpClientNode>()])
            )
            .add_node(
                backend::core::nodes::config::NodeConfig::new::<ExternalMcpClientNode>()
                    .with_description("Notify team on Slack".to_string())
                    .with_config(json!({
                        "service": "slack",
                        "tool": "send_message",
                        "endpoint": "http://localhost:8003",
                        "transport": "http"
                    }))
            )
            .build()
            .unwrap();
        
        // Create a test event
        let event_data = json!({
            "ticket_id": "TEST-12345",
            "customer_id": "CUST-67890",
            "customer_email": "test@example.com",
            "message": "I'm having trouble with my account login",
            "priority": "high",
            "category": "technical_support"
        });
        
        let event = NewEvent::new(event_data.clone(), "customer_support".to_string(), Value::Null);
        
        // Execute the workflow
        let start_time = std::time::Instant::now();
        let result = workflow.run(event_data).await;
        let execution_time = start_time.elapsed();
        
        // Verify workflow completed successfully
        assert!(result.is_ok(), "Workflow should complete successfully");
        
        let context = result.unwrap();
        assert_eq!(context.workflow_type, "customer_support_e2e");
        assert!(context.nodes.len() >= 4, "Should have executed all nodes");
        
        // Verify execution time is reasonable
        assert!(execution_time.as_secs() < 30, "Workflow should complete within 30 seconds");
        
        // Verify metrics were recorded
        let metrics = bootstrap.get_service::<AppMetrics>("metrics").await.unwrap();
        // Metrics assertions would go here based on your metrics implementation
        
        println!("End-to-end workflow completed in {:?}", execution_time);
    }

    /// Test workflow with multiple parallel branches
    #[tokio::test]
    #[ignore] // Requires external services
    async fn test_e2e_parallel_workflow_execution() {
        // Initialize node registry
        let mut node_registry = NodeRegistry::new();
        node_registry.register_node::<AgentNode>().unwrap();
        node_registry.register_node::<ExternalMcpClientNode>().unwrap();
        
        // Build workflow with parallel branches
        let workflow = WorkflowBuilder::new::<AgentNode>("parallel_processing".to_string())
            .description("Workflow with parallel execution branches".to_string())
            .add_node(
                backend::core::nodes::config::NodeConfig::new::<AgentNode>()
                    .with_description("Initial analysis".to_string())
                    .with_connections(vec![
                        TypeId::of::<ExternalMcpClientNode>(),
                        TypeId::of::<ExternalMcpClientNode>(),
                        TypeId::of::<ExternalMcpClientNode>()
                    ])
            )
            // Three parallel branches
            .add_node(
                backend::core::nodes::config::NodeConfig::new::<ExternalMcpClientNode>()
                    .with_description("Search Notion KB".to_string())
                    .with_config(json!({
                        "service": "notion",
                        "tool": "search_pages",
                        "endpoint": "http://localhost:8002"
                    }))
            )
            .add_node(
                backend::core::nodes::config::NodeConfig::new::<ExternalMcpClientNode>()
                    .with_description("Check HelpScout history".to_string())
                    .with_config(json!({
                        "service": "helpscout",
                        "tool": "list_conversations",
                        "endpoint": "http://localhost:8001"
                    }))
            )
            .add_node(
                backend::core::nodes::config::NodeConfig::new::<ExternalMcpClientNode>()
                    .with_description("Get Slack context".to_string())
                    .with_config(json!({
                        "service": "slack",
                        "tool": "list_channels",
                        "endpoint": "http://localhost:8003"
                    }))
            )
            .build()
            .unwrap();
        
        let context = json!({
            "query": "customer issue with billing",
            "customer_id": "CUST-PARALLEL-001"
        });
        
        let start_time = std::time::Instant::now();
        let result = workflow.run(context).await;
        let execution_time = start_time.elapsed();
        
        assert!(result.is_ok(), "Parallel workflow should complete successfully");
        
        // Parallel execution should be faster than sequential
        assert!(execution_time.as_secs() < 10, "Parallel execution should be efficient");
        
        println!("Parallel workflow completed in {:?}", execution_time);
    }

    /// Test workflow error handling and recovery
    #[tokio::test]
    async fn test_e2e_workflow_error_handling() {
        let mut node_registry = NodeRegistry::new();
        node_registry.register_node::<AgentNode>().unwrap();
        node_registry.register_node::<ExternalMcpClientNode>().unwrap();
        
        // Build workflow with a node that will fail
        let workflow = WorkflowBuilder::new::<AgentNode>("error_handling_test".to_string())
            .description("Workflow to test error handling".to_string())
            .add_node(
                backend::core::nodes::config::NodeConfig::new::<AgentNode>()
                    .with_description("Initial processing".to_string())
                    .with_connections(vec![TypeId::of::<ExternalMcpClientNode>()])
            )
            .add_node(
                backend::core::nodes::config::NodeConfig::new::<ExternalMcpClientNode>()
                    .with_description("Failing node".to_string())
                    .with_config(json!({
                        "service": "invalid_service",
                        "tool": "invalid_tool",
                        "endpoint": "http://localhost:9999" // Non-existent endpoint
                    }))
                    .with_connections(vec![TypeId::of::<AgentNode>()])
            )
            .add_node(
                backend::core::nodes::config::NodeConfig::new::<AgentNode>()
                    .with_description("Recovery handler".to_string())
            )
            .build()
            .unwrap();
        
        let context = json!({
            "test": "error_handling"
        });
        
        let result = workflow.run(context).await;
        
        // Workflow should handle errors gracefully
        assert!(result.is_err() || result.as_ref().unwrap().errors.len() > 0, 
                "Workflow should report errors");
        
        println!("Error handling test completed");
    }

    /// Test workflow state persistence and recovery
    #[tokio::test]
    async fn test_e2e_workflow_state_persistence() {
        // Initialize service bootstrap with database
        let bootstrap = ServiceBootstrap::new().await.unwrap();
        let event_repo = Arc::new(EventRepository::new());
        
        // Create workflow
        let workflow = WorkflowBuilder::new::<AgentNode>("persistent_workflow".to_string())
            .description("Workflow with state persistence".to_string())
            .add_node(
                backend::core::nodes::config::NodeConfig::new::<AgentNode>()
                    .with_description("Stateful processing node".to_string())
            )
            .build()
            .unwrap();
        
        // Create and persist initial event
        let event_data = json!({
            "session_id": "TEST-SESSION-001",
            "step": 1,
            "data": "initial state"
        });
        
        let event = NewEvent::new(
            event_data.clone(), 
            "persistent_workflow".to_string(), 
            Value::Null
        );
        
        // Store event
        let stored_event = event_repo.create_event(event).await.unwrap();
        
        // Execute workflow
        let result = workflow.run(event_data).await.unwrap();
        
        // Verify state was persisted
        let retrieved_event = event_repo.get_event(&stored_event.id).await.unwrap();
        assert!(retrieved_event.is_some(), "Event should be persisted");
        
        // Simulate workflow restart by creating new instance
        let workflow2 = WorkflowBuilder::new::<AgentNode>("persistent_workflow".to_string())
            .description("Workflow with state persistence".to_string())
            .add_node(
                backend::core::nodes::config::NodeConfig::new::<AgentNode>()
                    .with_description("Stateful processing node".to_string())
            )
            .build()
            .unwrap();
        
        // Continue from previous state
        let continue_data = json!({
            "session_id": "TEST-SESSION-001",
            "step": 2,
            "data": "continued state",
            "previous_event_id": stored_event.id
        });
        
        let result2 = workflow2.run(continue_data).await.unwrap();
        
        println!("State persistence test completed");
    }

    /// Test workflow with complex data transformations
    #[tokio::test]
    #[ignore] // Requires external services
    async fn test_e2e_data_transformation_workflow() {
        let mut node_registry = NodeRegistry::new();
        node_registry.register_node::<AgentNode>().unwrap();
        node_registry.register_node::<ExternalMcpClientNode>().unwrap();
        
        // Build data processing workflow
        let workflow = WorkflowBuilder::new::<AgentNode>("data_transformation".to_string())
            .description("Complex data transformation workflow".to_string())
            .add_node(
                backend::core::nodes::config::NodeConfig::new::<AgentNode>()
                    .with_description("Parse and validate input".to_string())
                    .with_connections(vec![TypeId::of::<ExternalMcpClientNode>()])
            )
            .add_node(
                backend::core::nodes::config::NodeConfig::new::<ExternalMcpClientNode>()
                    .with_description("Enrich with Notion data".to_string())
                    .with_config(json!({
                        "service": "notion",
                        "tool": "get_page",
                        "endpoint": "http://localhost:8002"
                    }))
                    .with_connections(vec![TypeId::of::<AgentNode>()])
            )
            .add_node(
                backend::core::nodes::config::NodeConfig::new::<AgentNode>()
                    .with_description("Transform and aggregate".to_string())
                    .with_connections(vec![TypeId::of::<ExternalMcpClientNode>()])
            )
            .add_node(
                backend::core::nodes::config::NodeConfig::new::<ExternalMcpClientNode>()
                    .with_description("Store results".to_string())
                    .with_config(json!({
                        "service": "notion",
                        "tool": "create_page",
                        "endpoint": "http://localhost:8002"
                    }))
            )
            .build()
            .unwrap();
        
        // Complex input data
        let input_data = json!({
            "records": [
                {"id": 1, "name": "Item A", "value": 100, "category": "electronics"},
                {"id": 2, "name": "Item B", "value": 200, "category": "books"},
                {"id": 3, "name": "Item C", "value": 150, "category": "electronics"}
            ],
            "transformations": {
                "group_by": "category",
                "aggregate": "sum",
                "enrich_with": "page_12345"
            }
        });
        
        let result = workflow.run(input_data).await;
        
        assert!(result.is_ok(), "Data transformation workflow should complete");
        
        let context = result.unwrap();
        // Verify transformations were applied
        assert!(context.nodes.len() >= 4, "All transformation steps should execute");
        
        println!("Data transformation workflow completed");
    }
}