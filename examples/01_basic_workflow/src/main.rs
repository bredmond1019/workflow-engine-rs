//! Basic Workflow Example - Main Demonstration
//!
//! This example demonstrates the fundamental concepts of creating and executing
//! workflows in the AI Workflow System. It showcases:
//!
//! - Workflow creation and validation
//! - Node registration and connection
//! - Proper error handling with boxed errors
//! - Event sourcing integration
//! - Comprehensive result formatting

use workflow_engine_core::workflow::Workflow;
use workflow_engine_core::error::WorkflowError;
use workflow_engine_core::task::TaskContext;
use tracing::{info, warn, error, Level};
use tracing_subscriber;
use serde_json::json;
use std::time::Instant;

// Import our custom nodes and types
use basic_workflow_example::{WorkflowInput, TextProcessingConfig, utils};
use basic_workflow_example::nodes::{TextInputNode, TextProcessorNode, TextOutputNode};

/// Main demonstration function
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for better debugging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    println!("🚀 AI Workflow System - Basic Workflow Example");
    println!("=" .repeat(50));

    // Run the complete demonstration
    match run_basic_workflow_demo().await {
        Ok(_) => {
            println!("\n✅ Basic workflow demonstration completed successfully!");
            Ok(())
        }
        Err(e) => {
            error!("❌ Demo failed: {}", e);
            Err(e)
        }
    }
}

/// Run the complete basic workflow demonstration
async fn run_basic_workflow_demo() -> Result<(), WorkflowError> {
    println!("\n📋 Step 1: Creating and configuring workflow");
    let mut workflow = create_workflow()?;
    
    println!("\n🔗 Step 2: Registering and connecting nodes");
    register_nodes(&mut workflow)?;
    connect_nodes(&mut workflow)?;
    
    println!("\n🔍 Step 3: Validating workflow structure");
    validate_workflow(&workflow)?;
    
    println!("\n🚀 Step 4: Executing workflow with sample data");
    execute_workflow_examples(&workflow).await?;
    
    println!("\n📊 Step 5: Demonstrating error handling");
    demonstrate_error_handling(&workflow).await?;
    
    Ok(())
}

/// Create and configure a new workflow
fn create_workflow() -> Result<Workflow, WorkflowError> {
    println!("   🏗️  Creating workflow: 'simple_text_processor'");
    
    // Validate workflow name before creation
    utils::validate_workflow_id("simple_text_processor")?;
    
    let workflow = Workflow::new("simple_text_processor")
        .map_err(|e| WorkflowError::validation_error(
            format!("Failed to create workflow: {}", e),
            "workflow_name",
            "valid workflow identifier",
            "in workflow creation"
        ))?;
    
    println!("   ✅ Workflow created successfully");
    Ok(workflow)
}

/// Register all workflow nodes
fn register_nodes(workflow: &mut Workflow) -> Result<(), WorkflowError> {
    println!("   📝 Registering workflow nodes...");
    
    // Register input node
    workflow.register_node("text_input", Box::new(TextInputNode::new()))
        .map_err(|e| WorkflowError::registry_error(
            format!("Failed to register text input node: {}", e),
            "register_node",
            "TextInputNode",
            Some("text_input".to_string())
        ))?;
    println!("      ✅ TextInputNode registered (id: text_input)");
    
    // Register processor node
    workflow.register_node("text_processor", Box::new(TextProcessorNode::new()))
        .map_err(|e| WorkflowError::registry_error(
            format!("Failed to register text processor node: {}", e),
            "register_node",
            "TextProcessorNode",
            Some("text_processor".to_string())
        ))?;
    println!("      ✅ TextProcessorNode registered (id: text_processor)");
    
    // Register output node
    workflow.register_node("text_output", Box::new(TextOutputNode::new()))
        .map_err(|e| WorkflowError::registry_error(
            format!("Failed to register text output node: {}", e),
            "register_node", 
            "TextOutputNode",
            Some("text_output".to_string())
        ))?;
    println!("      ✅ TextOutputNode registered (id: text_output)");
    
    Ok(())
}

/// Connect nodes in the workflow graph
fn connect_nodes(workflow: &mut Workflow) -> Result<(), WorkflowError> {
    println!("   🔗 Connecting workflow nodes...");
    
    // Connect input to processor
    workflow.connect("text_input", "text_processor")
        .map_err(|e| WorkflowError::validation_error(
            format!("Failed to connect text_input -> text_processor: {}", e),
            "node_connection",
            "valid node identifiers",
            "in workflow graph construction"
        ))?;
    println!("      ✅ Connected: text_input → text_processor");
    
    // Connect processor to output
    workflow.connect("text_processor", "text_output")
        .map_err(|e| WorkflowError::validation_error(
            format!("Failed to connect text_processor -> text_output: {}", e),
            "node_connection",
            "valid node identifiers", 
            "in workflow graph construction"
        ))?;
    println!("      ✅ Connected: text_processor → text_output");
    
    Ok(())
}

/// Validate the complete workflow structure
fn validate_workflow(workflow: &Workflow) -> Result<(), WorkflowError> {
    println!("   🔍 Validating workflow structure...");
    
    workflow.validate()
        .map_err(|e| match e {
            WorkflowError::CycleDetected => {
                WorkflowError::validation_error(
                    "Workflow contains circular dependencies",
                    "workflow_structure",
                    "directed acyclic graph (DAG)",
                    "in workflow validation"
                )
            }
            WorkflowError::UnreachableNodes { nodes } => {
                WorkflowError::validation_error(
                    format!("Unreachable nodes detected: {:?}", nodes),
                    "workflow_connectivity",
                    "all nodes must be reachable from start",
                    "in workflow validation"
                )
            }
            other => other,
        })?;
    
    println!("      ✅ No cycles detected");
    println!("      ✅ All nodes reachable");
    println!("      ✅ Workflow validation passed");
    
    Ok(())
}

/// Execute the workflow with various example inputs
async fn execute_workflow_examples(workflow: &Workflow) -> Result<(), WorkflowError> {
    // Example 1: Basic text processing
    println!("\n   📝 Example 1: Basic uppercase transformation");
    let input1 = WorkflowInput::new("Hello, Workflow System!")
        .with_config(TextProcessingConfig {
            mode: "uppercase".to_string(),
            ..Default::default()
        });
    execute_single_workflow(workflow, input1, "uppercase").await?;
    
    // Example 2: Lowercase transformation
    println!("\n   📝 Example 2: Lowercase transformation");
    let input2 = WorkflowInput::new("CONVERTING TO lowercase")
        .with_config(TextProcessingConfig {
            mode: "lowercase".to_string(),
            ..Default::default()
        });
    execute_single_workflow(workflow, input2, "lowercase").await?;
    
    // Example 3: Title case transformation
    println!("\n   📝 Example 3: Title case transformation");
    let input3 = WorkflowInput::new("this should become title case")
        .with_config(TextProcessingConfig {
            mode: "title_case".to_string(),
            ..Default::default()
        });
    execute_single_workflow(workflow, input3, "title_case").await?;
    
    // Example 4: Text analysis
    println!("\n   📝 Example 4: Text analysis");
    let input4 = WorkflowInput::new("Analyzing this text for various metrics and statistics")
        .with_config(TextProcessingConfig {
            mode: "analyze".to_string(),
            ..Default::default()
        });
    execute_single_workflow(workflow, input4, "analyze").await?;
    
    Ok(())
}

/// Execute a single workflow with the given input
async fn execute_single_workflow(
    workflow: &Workflow, 
    input: WorkflowInput,
    example_name: &str
) -> Result<(), WorkflowError> {
    let start_time = Instant::now();
    
    // Validate input before processing
    input.validate()
        .map_err(|e| WorkflowError::validation_error(
            format!("Input validation failed for {}: {}", example_name, e),
            "workflow_input",
            "valid WorkflowInput structure",
            "in workflow execution"
        ))?;
    
    println!("      Input: \"{}\"", input.text);
    
    // Create task context and set input data
    let mut context = TaskContext::new();
    context.set_event_data(json!(input))
        .map_err(|e| WorkflowError::serialization_error(
            format!("Failed to serialize input data: {}", e),
            "WorkflowInput",
            "in workflow execution setup"
        ))?;
    
    // Execute the workflow
    println!("      🔄 Executing workflow...");
    let result_context = workflow.execute(context)
        .await
        .map_err(|e| WorkflowError::processing_error(
            format!("Workflow execution failed for {}: {}", example_name, e),
            "WorkflowExecutor"
        ))?;
    
    let execution_time = start_time.elapsed();
    
    // Extract and display results
    let formatted_output: String = result_context.get_data("formatted_output")
        .map_err(|e| WorkflowError::processing_error(
            format!("Failed to extract formatted output: {}", e),
            "WorkflowExecutor"
        ))?;
    
    println!("      ✅ Completed in {}ms", execution_time.as_millis());
    
    // Display result based on format
    if formatted_output.starts_with('{') {
        // JSON output - parse and display nicely
        let output_value: serde_json::Value = serde_json::from_str(&formatted_output)
            .map_err(|e| WorkflowError::deserialization_error(
                format!("Failed to parse output JSON: {}", e),
                "serde_json::Value",
                "in result formatting",
                Some(formatted_output.clone())
            ))?;
        
        if let Some(result) = output_value.get("result") {
            println!("      📤 Result: {}", result.as_str().unwrap_or("N/A"));
        }
        
        if let Some(metadata) = output_value.get("metadata") {
            if let Some(exec_time) = metadata.get("execution_time_ms") {
                println!("      ⏱️  Execution time: {}ms", exec_time.as_u64().unwrap_or(0));
            }
            if let Some(nodes) = metadata.get("nodes_processed") {
                println!("      🔢 Nodes processed: {}", nodes.as_u64().unwrap_or(0));
            }
        }
    } else {
        // Text output
        println!("      📤 Result: {}", formatted_output);
    }
    
    Ok(())
}

/// Demonstrate various error handling scenarios
async fn demonstrate_error_handling(workflow: &Workflow) -> Result<(), WorkflowError> {
    println!("\n   ⚠️  Testing error handling scenarios...");
    
    // Test 1: Empty input validation
    println!("\n      🧪 Test 1: Empty input validation");
    let empty_input = WorkflowInput::new("");
    match execute_single_workflow(workflow, empty_input, "empty_test").await {
        Ok(_) => warn!("      ⚠️ Empty input unexpectedly succeeded"),
        Err(WorkflowError::ValidationError(details)) => {
            println!("      ✅ Correctly caught validation error: {}", details.message);
        }
        Err(e) => {
            warn!("      ⚠️ Unexpected error type: {}", e);
        }
    }
    
    // Test 2: Invalid processing mode
    println!("\n      🧪 Test 2: Invalid processing mode");
    let invalid_mode_input = WorkflowInput::new("test text")
        .with_config(TextProcessingConfig {
            mode: "invalid_mode".to_string(),
            ..Default::default()
        });
    match execute_single_workflow(workflow, invalid_mode_input, "invalid_mode_test").await {
        Ok(_) => warn!("      ⚠️ Invalid mode unexpectedly succeeded"),
        Err(WorkflowError::ValidationError(details)) => {
            println!("      ✅ Correctly caught validation error: {}", details.message);
        }
        Err(e) => {
            warn!("      ⚠️ Unexpected error type: {}", e);
        }
    }
    
    // Test 3: Text too long
    println!("\n      🧪 Test 3: Text length validation");
    let long_input = WorkflowInput::new(&"x".repeat(15000))
        .with_config(TextProcessingConfig {
            max_length: Some(1000),
            ..Default::default()
        });
    match execute_single_workflow(workflow, long_input, "length_test").await {
        Ok(_) => warn!("      ⚠️ Long input unexpectedly succeeded"),
        Err(WorkflowError::ValidationError(details)) => {
            println!("      ✅ Correctly caught validation error: {}", details.message);
        }
        Err(e) => {
            warn!("      ⚠️ Unexpected error type: {}", e);
        }
    }
    
    println!("\n      ✅ Error handling demonstration completed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_workflow_creation() {
        let result = create_workflow();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_node_registration() {
        let mut workflow = create_workflow().unwrap();
        let result = register_nodes(&mut workflow);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_workflow_validation() {
        let mut workflow = create_workflow().unwrap();
        register_nodes(&mut workflow).unwrap();
        connect_nodes(&mut workflow).unwrap();
        
        let result = validate_workflow(&workflow);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_basic_execution() {
        let mut workflow = create_workflow().unwrap();
        register_nodes(&mut workflow).unwrap();
        connect_nodes(&mut workflow).unwrap();
        
        let input = WorkflowInput::new("test input");
        let result = execute_single_workflow(&workflow, input, "test").await;
        assert!(result.is_ok());
    }
}