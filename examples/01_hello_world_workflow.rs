//! # Hello World Workflow Example
//!
//! This example demonstrates the most basic workflow using the workflow engine.
//! It shows how to:
//!
//! - Create a simple node that processes data
//! - Build a workflow with type-safe construction
//! - Execute the workflow with input data
//! - Extract results from the task context
//!
//! ## Usage
//!
//! ```bash
//! cargo run --example 01_hello_world_workflow
//! ```

use workflow_engine_core::prelude::*;
use serde_json::json;

/// A simple greeting node that transforms names into greetings
#[derive(Debug)]
struct GreetingNode {
    greeting_prefix: String,
}

impl GreetingNode {
    /// Create a new greeting node with the specified prefix
    fn new(prefix: impl Into<String>) -> Self {
        Self {
            greeting_prefix: prefix.into(),
        }
    }
}

impl Node for GreetingNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Extract the input data from the context
        let input: serde_json::Value = context.get_event_data()?;
        
        // Get the name from input, with a default value
        let name = input
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("World");
        
        // Create the greeting message
        let greeting = format!("{} {}!", self.greeting_prefix, name);
        
        // Store the result in the context
        context.update_node("greeting", json!({
            "message": greeting,
            "processed_at": chrono::Utc::now(),
            "input_name": name
        }));
        
        // Add some metadata about the processing
        context.set_metadata("node_type", "greeting")?;
        context.set_metadata("processing_success", true)?;
        
        println!("✅ Greeting processed: {}", greeting);
        
        Ok(context)
    }
}

/// A response formatting node that creates a structured response
#[derive(Debug)]
struct ResponseFormatterNode;

impl Node for ResponseFormatterNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Extract the greeting result from the previous node
        let greeting_data: serde_json::Value = context
            .get_node_data("greeting")?
            .unwrap_or_else(|| json!({"message": "No greeting available"}));
        
        // Create a formatted response
        let response = json!({
            "status": "success",
            "workflow": "hello_world",
            "data": greeting_data,
            "formatted_at": chrono::Utc::now()
        });
        
        // Store the final response
        context.update_node("response", response);
        
        // Update metadata
        context.set_metadata("final_node", "response_formatter")?;
        
        println!("✅ Response formatted successfully");
        
        Ok(context)
    }
}

#[tokio::main]
async fn main() -> Result<(), WorkflowError> {
    println!("🚀 Starting Hello World Workflow Example");
    println!("==========================================");
    
    // Build a simple workflow using the type-safe builder
    let workflow = TypedWorkflowBuilder::new("hello_world_workflow")
        .description("A simple greeting workflow demonstration")
        .start_with_node(NodeId::new("greeting"))
        .then_node(NodeId::new("response_formatter"))
        .build()?;
    
    // Register our nodes with the workflow
    workflow.register_node(NodeId::new("greeting"), GreetingNode::new("Hello"));
    workflow.register_node(NodeId::new("response_formatter"), ResponseFormatterNode);
    
    println!("📋 Workflow built with 2 nodes:");
    println!("   1. GreetingNode - Creates greeting message");
    println!("   2. ResponseFormatterNode - Formats final response");
    println!();
    
    // Test data for different scenarios
    let test_cases = vec![
        json!({"name": "Alice"}),
        json!({"name": "Bob"}),
        json!({"name": "Charlie"}),
        json!({}), // Test default case
    ];
    
    // Execute the workflow with different inputs
    for (i, input_data) in test_cases.into_iter().enumerate() {
        println!("🔄 Test Case {} - Input: {}", i + 1, input_data);
        
        // Run the workflow
        let result = workflow.run(input_data).await?;
        
        // Extract and display the final response
        if let Some(response) = result.get_node_data::<serde_json::Value>("response")? {
            println!("   📤 Result: {}", serde_json::to_string_pretty(&response)?);
        }
        
        // Show metadata
        println!("   📊 Metadata:");
        for (key, value) in result.get_all_metadata() {
            println!("      {} = {}", key, value);
        }
        
        println!("   ✅ Test case completed successfully");
        println!();
    }
    
    println!("🎉 Hello World Workflow Example completed!");
    println!("==========================================");
    println!();
    println!("Key takeaways from this example:");
    println!("• Nodes implement the Node trait with a process() method");
    println!("• Workflows are built using TypedWorkflowBuilder for type safety");
    println!("• Data flows through TaskContext between nodes");
    println!("• Results are stored with update_node() and retrieved with get_node_data()");
    println!("• Metadata can be added for debugging and monitoring");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_greeting_node() {
        let node = GreetingNode::new("Hi");
        let context = TaskContext::new(
            "test".to_string(),
            json!({"name": "Test User"})
        );
        
        let result = node.process(context).unwrap();
        let greeting: serde_json::Value = result.get_node_data("greeting").unwrap().unwrap();
        
        assert_eq!(greeting["message"], "Hi Test User!");
        assert_eq!(greeting["input_name"], "Test User");
    }
    
    #[test]
    fn test_greeting_node_default_name() {
        let node = GreetingNode::new("Hello");
        let context = TaskContext::new(
            "test".to_string(),
            json!({}) // No name provided
        );
        
        let result = node.process(context).unwrap();
        let greeting: serde_json::Value = result.get_node_data("greeting").unwrap().unwrap();
        
        assert_eq!(greeting["message"], "Hello World!");
        assert_eq!(greeting["input_name"], "World");
    }
    
    #[test]
    fn test_response_formatter_node() {
        let node = ResponseFormatterNode;
        let mut context = TaskContext::new(
            "test".to_string(),
            json!({})
        );
        
        // Add greeting data as if it came from a previous node
        context.update_node("greeting", json!({
            "message": "Hello Test!",
            "processed_at": chrono::Utc::now()
        }));
        
        let result = node.process(context).unwrap();
        let response: serde_json::Value = result.get_node_data("response").unwrap().unwrap();
        
        assert_eq!(response["status"], "success");
        assert_eq!(response["workflow"], "hello_world");
        assert!(response["data"]["message"].as_str().unwrap().contains("Hello Test!"));
    }
}