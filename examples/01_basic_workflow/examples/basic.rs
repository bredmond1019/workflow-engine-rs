//! Basic workflow execution example
//!
//! This example demonstrates the simplest possible workflow execution
//! with minimal configuration and error handling.

use workflow_engine_core::workflow::Workflow;
use workflow_engine_core::error::WorkflowError;
use workflow_engine_core::task::TaskContext;
use serde_json::json;

use basic_workflow_example::{WorkflowInput, WorkflowOutput};
use basic_workflow_example::nodes::{TextInputNode, TextProcessorNode, TextOutputNode};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Basic Workflow Example - Simple Execution");
    println!("=" .repeat(40));

    // Create a simple workflow
    let mut workflow = Workflow::new("basic_example")?;

    // Register nodes
    workflow.register_node("input", Box::new(TextInputNode::new()))?;
    workflow.register_node("processor", Box::new(TextProcessorNode::new()))?;
    workflow.register_node("output", Box::new(TextOutputNode::new()))?;

    // Connect nodes
    workflow.connect("input", "processor")?;
    workflow.connect("processor", "output")?;

    // Validate workflow
    workflow.validate()?;
    println!("✅ Workflow validated successfully");

    // Create input
    let input = WorkflowInput::new("Hello, Simple Workflow!");
    println!("📝 Input: \"{}\"", input.text);

    // Execute workflow
    let mut context = TaskContext::new();
    context.set_event_data(json!(input))?;

    let result_context = workflow.execute(context).await?;

    // Extract result
    let final_output: WorkflowOutput = result_context.get_data("final_output")?;
    
    println!("📤 Result: \"{}\"", final_output.result);
    println!("⏱️  Execution time: {}ms", 
             final_output.metadata.execution_time_ms.unwrap_or(0));
    println!("🔢 Nodes processed: {}", final_output.metadata.nodes_processed);

    println!("\n✅ Basic example completed successfully!");
    Ok(())
}