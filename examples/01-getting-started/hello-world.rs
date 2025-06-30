//! # Hello World - Your First Workflow
//!
//! Welcome to your first AI Workflow Engine example! This demonstrates the absolute
//! basics of creating and running a workflow.
//!
//! ## What You'll Learn
//! - How to implement the Node trait
//! - Basic TaskContext operations  
//! - Simple workflow execution
//! - Data flow between nodes
//!
//! ## Key Concepts
//! - **Node**: A processing unit that transforms data
//! - **TaskContext**: The data container that flows through the workflow
//! - **Workflow**: A series of connected nodes that process data
//!
//! ## Usage
//! ```bash
//! cargo run --bin hello-world
//! ```

use workflow_engine_core::prelude::*;
use serde_json::json;

/// A simple greeting node that takes a name and creates a greeting message
/// 
/// This is your first Node implementation. Every node must:
/// 1. Implement the Node trait
/// 2. Be Send + Sync + Debug for thread safety
/// 3. Have a process() method that takes and returns TaskContext
#[derive(Debug)]
struct GreetingNode {
    /// The greeting prefix (e.g., "Hello", "Hi", "Welcome")
    prefix: String,
}

impl GreetingNode {
    /// Create a new greeting node with a custom prefix
    fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
        }
    }
}

/// Implementation of the Node trait for GreetingNode
/// 
/// This is where the actual processing happens. The node:
/// 1. Extracts input data from the TaskContext
/// 2. Performs its processing logic
/// 3. Stores results back in the TaskContext
/// 4. Returns the updated TaskContext
impl Node for GreetingNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("📝 GreetingNode: Processing input...");
        
        // Step 1: Extract input data from the context
        // get_event_data() deserializes the original input data
        let input: serde_json::Value = context.get_event_data()?;
        
        // Step 2: Get the name from the input, with a default fallback
        let name = input
            .get("name")                    // Try to get "name" field
            .and_then(|v| v.as_str())      // Convert to string if it exists
            .unwrap_or("World");           // Use "World" as default
        
        println!("   📥 Input name: {}", name);
        
        // Step 3: Create the greeting message
        let greeting_message = format!("{} {}!", self.prefix, name);
        
        println!("   ✨ Created greeting: {}", greeting_message);
        
        // Step 4: Store the result in the context
        // update_node() stores data that other nodes can access later
        context.update_node("greeting", json!({
            "message": greeting_message,
            "processed_at": chrono::Utc::now().to_rfc3339(),
            "input_name": name,
            "prefix_used": self.prefix
        }));
        
        // Step 5: Add some metadata for debugging/monitoring
        context.set_metadata("node_type", "greeting")?;
        context.set_metadata("processing_success", true)?;
        
        println!("   ✅ GreetingNode: Processing complete");
        
        // Step 6: Return the updated context
        Ok(context)
    }
}

/// A response formatter node that creates a structured final response
/// 
/// This node demonstrates:
/// - How to access data from previous nodes
/// - Creating structured output
/// - Finalizing workflow results
#[derive(Debug)]
struct ResponseFormatterNode;

impl Node for ResponseFormatterNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("📋 ResponseFormatterNode: Formatting response...");
        
        // Step 1: Get the greeting data from the previous node
        // get_node_data() retrieves data stored by a specific node
        let greeting_data = context
            .get_node_data::<serde_json::Value>("greeting")?
            .unwrap_or_else(|| json!({"message": "No greeting available"}));
        
        println!("   📥 Retrieved greeting data");
        
        // Step 2: Create a structured final response
        let final_response = json!({
            "status": "success",
            "workflow_type": "hello_world",
            "data": greeting_data,
            "formatted_at": chrono::Utc::now().to_rfc3339(),
            "version": "1.0"
        });
        
        // Step 3: Store the final response
        context.update_node("final_response", final_response);
        
        // Step 4: Update metadata
        context.set_metadata("final_node", "response_formatter")?;
        context.set_metadata("workflow_complete", true)?;
        
        println!("   ✅ ResponseFormatterNode: Formatting complete");
        
        Ok(context)
    }
}

/// Main function that demonstrates a complete workflow execution
#[tokio::main]
async fn main() -> Result<(), WorkflowError> {
    println!("🚀 Hello World Workflow Example");
    println!("=".repeat(50));
    println!("This example demonstrates the basics of workflow execution.\n");
    
    // Step 1: Create our nodes
    println!("📦 Creating workflow nodes...");
    let greeting_node = GreetingNode::new("Hello");
    let formatter_node = ResponseFormatterNode;
    println!("   ✅ Created GreetingNode and ResponseFormatterNode\n");
    
    // Step 2: Define test cases to demonstrate different scenarios
    let test_cases = vec![
        ("Alice", json!({"name": "Alice"})),
        ("Bob", json!({"name": "Bob"})),
        ("Charlie", json!({"name": "Charlie"})),
        ("Default case", json!({})), // Test with no name provided
    ];
    
    // Step 3: Execute the workflow for each test case
    for (test_name, input_data) in test_cases {
        println!("🔄 Test Case: {}", test_name);
        println!("   📥 Input: {}", input_data);
        
        // Step 3a: Create initial context with workflow type and input data
        let mut context = TaskContext::new(
            "hello_world_workflow".to_string(),
            input_data
        );
        
        // Step 3b: Execute the greeting node
        context = greeting_node.process(context)?;
        
        // Step 3c: Execute the formatter node
        context = formatter_node.process(context)?;
        
        // Step 3d: Extract and display the final result
        if let Some(response) = context.get_node_data::<serde_json::Value>("final_response")? {
            println!("   📤 Final Response:");
            println!("      {}", serde_json::to_string_pretty(&response)?);
            
            // Extract the greeting message for a friendly display
            if let Some(message) = response
                .get("data")
                .and_then(|d| d.get("message"))
                .and_then(|m| m.as_str()) 
            {
                println!("   💬 Greeting: {}", message);
            }
        }
        
        // Step 3e: Show workflow metadata
        println!("   📊 Workflow Metadata:");
        for (key, value) in context.get_all_metadata() {
            println!("      {} = {}", key, value);
        }
        
        println!("   ✅ Test case completed\n");
    }
    
    // Step 4: Demonstrate error handling
    println!("🔧 Demonstrating Error Handling");
    println!("-".repeat(30));
    
    // Try with invalid input data (this should still work due to our fallbacks)
    let invalid_input = json!({"not_a_name": "invalid"});
    println!("📥 Input with invalid field: {}", invalid_input);
    
    let mut context = TaskContext::new(
        "hello_world_workflow".to_string(),
        invalid_input
    );
    
    context = greeting_node.process(context)?;
    context = formatter_node.process(context)?;
    
    if let Some(response) = context.get_node_data::<serde_json::Value>("final_response")? {
        println!("📤 Response (graceful fallback): {}", 
            response.get("data")
                .and_then(|d| d.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("No message"));
    }
    
    println!("✅ Error handling test completed\n");
    
    // Summary and next steps
    println!("🎉 Hello World Workflow Example Complete!");
    println!("=".repeat(50));
    println!("🎓 What you learned:");
    println!("   • How to implement the Node trait");
    println!("   • Using TaskContext to pass data between nodes");
    println!("   • Basic workflow execution patterns");
    println!("   • Error handling with graceful fallbacks");
    println!("   • Storing and retrieving node data");
    println!("   • Adding metadata for debugging");
    println!();
    println!("➡️  Next steps:");
    println!("   • Try modifying the greeting prefix");
    println!("   • Add your own node to the workflow");
    println!("   • Explore the basic-nodes.rs example");
    println!("   • Check out the test cases below");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Test the GreetingNode with a specific name
    #[test]
    fn test_greeting_node_with_name() {
        let node = GreetingNode::new("Hi");
        let context = TaskContext::new(
            "test".to_string(),
            json!({"name": "Alice"})
        );
        
        let result = node.process(context).unwrap();
        let greeting: serde_json::Value = result
            .get_node_data("greeting")
            .unwrap()
            .unwrap();
        
        assert_eq!(greeting["message"], "Hi Alice!");
        assert_eq!(greeting["input_name"], "Alice");
        assert_eq!(greeting["prefix_used"], "Hi");
    }
    
    /// Test the GreetingNode with no name (default fallback)
    #[test]
    fn test_greeting_node_default_name() {
        let node = GreetingNode::new("Hello");
        let context = TaskContext::new(
            "test".to_string(),
            json!({}) // No name provided
        );
        
        let result = node.process(context).unwrap();
        let greeting: serde_json::Value = result
            .get_node_data("greeting")
            .unwrap()
            .unwrap();
        
        assert_eq!(greeting["message"], "Hello World!");
        assert_eq!(greeting["input_name"], "World");
    }
    
    /// Test the ResponseFormatterNode
    #[test]
    fn test_response_formatter_node() {
        let node = ResponseFormatterNode;
        let mut context = TaskContext::new(
            "test".to_string(),
            json!({})
        );
        
        // Simulate data from a previous greeting node
        context.update_node("greeting", json!({
            "message": "Hello Test!",
            "processed_at": chrono::Utc::now().to_rfc3339()
        }));
        
        let result = node.process(context).unwrap();
        let response: serde_json::Value = result
            .get_node_data("final_response")
            .unwrap()
            .unwrap();
        
        assert_eq!(response["status"], "success");
        assert_eq!(response["workflow_type"], "hello_world");
        assert!(response["data"]["message"].as_str().unwrap().contains("Hello Test!"));
    }
    
    /// Test the complete workflow pipeline
    #[test]
    fn test_complete_workflow() {
        let greeting_node = GreetingNode::new("Welcome");
        let formatter_node = ResponseFormatterNode;
        
        let mut context = TaskContext::new(
            "test_workflow".to_string(),
            json!({"name": "Tester"})
        );
        
        // Execute the complete pipeline
        context = greeting_node.process(context).unwrap();
        context = formatter_node.process(context).unwrap();
        
        // Verify the final result
        let response: serde_json::Value = context
            .get_node_data("final_response")
            .unwrap()
            .unwrap();
        
        assert_eq!(response["status"], "success");
        assert!(response["data"]["message"].as_str().unwrap().contains("Welcome Tester!"));
        
        // Verify metadata was set correctly
        assert_eq!(
            context.get_metadata::<bool>("workflow_complete").unwrap().unwrap(),
            true
        );
    }
}