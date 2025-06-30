//! Error handling patterns demonstration
//!
//! This example demonstrates comprehensive error handling patterns
//! using the new boxed error system. It shows how to:
//! - Handle different error categories
//! - Implement proper error recovery
//! - Use error context for debugging
//! - Test error scenarios systematically

use workflow_engine_core::workflow::Workflow;
use workflow_engine_core::error::{WorkflowError, ErrorExt};
use workflow_engine_core::task::TaskContext;
use serde_json::json;
use std::collections::HashMap;

use basic_workflow_example::{WorkflowInput, TextProcessingConfig};
use basic_workflow_example::nodes::{
    TextInputNode, TextProcessorNode, TextOutputNode,
    text_input::TextInputConfig
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Error Handling Patterns Demo");
    println!("=" .repeat(40));

    // Test different error categories
    test_validation_errors().await?;
    test_processing_errors().await?;
    test_configuration_errors().await?;
    test_error_recovery().await?;
    test_error_categorization().await?;

    println!("\n✅ Error handling demonstration completed!");
    Ok(())
}

async fn test_validation_errors() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 Testing Validation Errors");
    println!("-" .repeat(25));

    let mut workflow = create_test_workflow("validation_test")?;

    let validation_test_cases = vec![
        ("Empty input", WorkflowInput::new("")),
        ("Too long input", WorkflowInput::new(&"x".repeat(15000)).with_config(
            TextProcessingConfig {
                max_length: Some(100),
                ..Default::default()
            }
        )),
        ("Invalid mode", WorkflowInput::new("test").with_config(
            TextProcessingConfig {
                mode: "invalid_mode".to_string(),
                ..Default::default()
            }
        )),
    ];

    for (test_name, input) in validation_test_cases {
        println!("\n🧪 Test: {}", test_name);
        
        match execute_workflow_with_error_handling(&workflow, input).await {
            Ok(_) => println!("   ⚠️  Unexpectedly succeeded"),
            Err(WorkflowError::ValidationError(details)) => {
                println!("   ✅ Caught validation error:");
                println!("      Field: {}", details.field);
                println!("      Message: {}", details.message);
                println!("      Constraint: {}", details.constraint);
                println!("      Context: {}", details.context);
            }
            Err(e) => println!("   ⚠️  Unexpected error type: {}", e),
        }
    }

    Ok(())
}

async fn test_processing_errors() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n⚙️  Testing Processing Errors");
    println!("-" .repeat(25));

    // Create workflow with restrictive input validation
    let mut workflow = Workflow::new("processing_test")?;
    
    let restrictive_config = TextInputConfig {
        min_length: 1,
        max_length: 100,
        sanitize: true,
        trim: true,
        allowed_chars: Some(vec!["alphabetic".to_string()]), // Only letters
    };

    workflow.register_node("input", Box::new(TextInputNode::with_config(restrictive_config)))?;
    workflow.register_node("processor", Box::new(TextProcessorNode::new()))?;
    workflow.register_node("output", Box::new(TextOutputNode::new()))?;

    workflow.connect("input", "processor")?;
    workflow.connect("processor", "output")?;
    workflow.validate()?;

    let processing_test_cases = vec![
        ("Numbers in text", WorkflowInput::new("hello123world")),
        ("Punctuation", WorkflowInput::new("hello, world!")),
        ("Special characters", WorkflowInput::new("hello@world")),
    ];

    for (test_name, input) in processing_test_cases {
        println!("\n🧪 Test: {}", test_name);
        
        match execute_workflow_with_error_handling(&workflow, input).await {
            Ok(_) => println!("   ⚠️  Unexpectedly succeeded"),
            Err(WorkflowError::ValidationError(details)) => {
                println!("   ✅ Caught character validation error:");
                println!("      Message: {}", details.message);
            }
            Err(WorkflowError::ProcessingError(details)) => {
                println!("   ✅ Caught processing error:");
                println!("      Node: {}", details.node_type);
                println!("      Message: {}", details.message);
                if let Some(node_id) = &details.node_id {
                    println!("      Node ID: {}", node_id);
                }
            }
            Err(e) => println!("   ⚠️  Unexpected error type: {}", e),
        }
    }

    Ok(())
}

async fn test_configuration_errors() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n⚙️  Testing Configuration Errors");
    println!("-" .repeat(25));

    // Test workflow creation with invalid names
    let invalid_workflow_names = vec![
        "",
        "invalid@name",
        "workflow with spaces",
        &"x".repeat(200),
    ];

    for name in invalid_workflow_names {
        println!("\n🧪 Test: Invalid workflow name '{}'", 
                if name.len() > 20 { &name[..20] } else { name });
        
        match Workflow::new(name) {
            Ok(_) => println!("   ⚠️  Unexpectedly succeeded"),
            Err(e) => {
                if let Some(config_error) = e.downcast_ref::<WorkflowError>() {
                    match config_error {
                        WorkflowError::ValidationError(details) => {
                            println!("   ✅ Caught configuration error:");
                            println!("      Message: {}", details.message);
                        }
                        _ => println!("   ⚠️  Unexpected error type: {}", config_error),
                    }
                } else {
                    println!("   ⚠️  Unexpected error: {}", e);
                }
            }
        }
    }

    Ok(())
}

async fn test_error_recovery() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 Testing Error Recovery Patterns");
    println!("-" .repeat(30));

    let mut workflow = create_test_workflow("recovery_test")?;

    // Demonstrate retry with backoff pattern
    let problematic_inputs = vec![
        WorkflowInput::new(""), // Will fail validation
        WorkflowInput::new("valid input"), // Will succeed
        WorkflowInput::new("test").with_config(TextProcessingConfig {
            mode: "invalid".to_string(),
            ..Default::default()
        }), // Will fail validation
    ];

    for (i, input) in problematic_inputs.into_iter().enumerate() {
        println!("\n🧪 Recovery Test {}: {:?}", i + 1, 
                if input.text.is_empty() { "empty" } else { &input.text });
        
        // Implement retry with exponential backoff
        let mut retry_count = 0;
        let max_retries = 3;
        
        loop {
            match execute_workflow_with_error_handling(&workflow, input.clone()).await {
                Ok(result) => {
                    println!("   ✅ Succeeded on attempt {}", retry_count + 1);
                    break;
                }
                Err(e) => {
                    retry_count += 1;
                    
                    match e.category() {
                        workflow_engine_core::error::ErrorCategory::Transient if retry_count < max_retries => {
                            println!("   🔄 Transient error, retrying... (attempt {})", retry_count);
                            tokio::time::sleep(tokio::time::Duration::from_millis(100 * retry_count as u64)).await;
                            continue;
                        }
                        _ => {
                            println!("   ❌ Permanent failure after {} attempts: {}", retry_count, e);
                            break;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

async fn test_error_categorization() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📊 Testing Error Categorization");
    println!("-" .repeat(30));

    let test_errors = vec![
        WorkflowError::validation_error("Test validation", "field", "constraint", "context"),
        WorkflowError::processing_error("Test processing", "TestNode"),
        WorkflowError::CycleDetected,
        WorkflowError::api_error_simple("Test API error"),
        WorkflowError::mcp_connection_error_simple("Test MCP connection"),
    ];

    for error in test_errors {
        println!("\n🏷️  Error: {}", error);
        println!("   Category: {:?}", error.category());
        println!("   Severity: {:?}", error.severity());
        println!("   Code: {}", error.error_code());
        
        // Demonstrate error handling based on category
        match error.category() {
            workflow_engine_core::error::ErrorCategory::User => {
                println!("   💡 Suggestion: Fix input validation");
            }
            workflow_engine_core::error::ErrorCategory::Transient => {
                println!("   💡 Suggestion: Retry with backoff");
            }
            workflow_engine_core::error::ErrorCategory::Permanent => {
                println!("   💡 Suggestion: Fix configuration");
            }
            workflow_engine_core::error::ErrorCategory::System => {
                println!("   💡 Suggestion: Check system resources");
            }
            workflow_engine_core::error::ErrorCategory::Business => {
                println!("   💡 Suggestion: Review business logic");
            }
        }
    }

    Ok(())
}

fn create_test_workflow(name: &str) -> Result<Workflow, Box<dyn std::error::Error>> {
    let mut workflow = Workflow::new(name)?;
    
    workflow.register_node("input", Box::new(TextInputNode::new()))?;
    workflow.register_node("processor", Box::new(TextProcessorNode::new()))?;
    workflow.register_node("output", Box::new(TextOutputNode::new()))?;

    workflow.connect("input", "processor")?;
    workflow.connect("processor", "output")?;
    workflow.validate()?;

    Ok(workflow)
}

async fn execute_workflow_with_error_handling(
    workflow: &Workflow,
    input: WorkflowInput,
) -> Result<String, WorkflowError> {
    // Validate input first
    input.validate()?;

    // Create context and execute
    let mut context = TaskContext::new();
    context.set_event_data(json!(input))
        .map_err(|e| WorkflowError::serialization_error(
            format!("Failed to serialize input: {}", e),
            "WorkflowInput",
            "in error handling test"
        ))?;

    let result_context = workflow.execute(context).await?;

    // Extract result
    let formatted_output: String = result_context.get_data("formatted_output")
        .map_err(|e| WorkflowError::processing_error(
            format!("Failed to extract output: {}", e),
            "ErrorHandlingTest"
        ))?;

    Ok(formatted_output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validation_error_handling() {
        let result = test_validation_errors().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_processing_error_handling() {
        let result = test_processing_errors().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_error_categorization_logic() {
        let result = test_error_categorization().await;
        assert!(result.is_ok());
    }
}