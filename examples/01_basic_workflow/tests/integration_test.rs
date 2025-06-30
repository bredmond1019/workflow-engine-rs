//! Integration tests for the basic workflow example
//!
//! These tests verify that the workflow integrates correctly with the
//! broader AI Workflow System, including event sourcing, API endpoints,
//! and error handling across system boundaries.

use workflow_engine_core::workflow::Workflow;
use workflow_engine_core::error::WorkflowError;
use workflow_engine_core::task::TaskContext;
use serde_json::json;
use tokio::time::{timeout, Duration};

use basic_workflow_example::{WorkflowInput, TextProcessingConfig, WorkflowOutput};
use basic_workflow_example::nodes::{TextInputNode, TextProcessorNode, TextOutputNode};

/// Test workflow creation and basic execution
#[tokio::test]
async fn test_workflow_end_to_end() {
    let workflow = create_test_workflow().await.unwrap();
    
    let input = WorkflowInput::new("Integration test input");
    let result = execute_workflow(&workflow, input).await;
    
    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output.result, "INTEGRATION TEST INPUT");
    assert!(output.metadata.execution_time_ms.is_some());
    assert_eq!(output.metadata.nodes_processed, 3);
}

/// Test workflow with various text processing modes
#[tokio::test]
async fn test_all_processing_modes() {
    let workflow = create_test_workflow().await.unwrap();
    
    let test_cases = vec![
        ("uppercase", "hello world", "HELLO WORLD"),
        ("lowercase", "HELLO WORLD", "hello world"),
        ("title_case", "hello world", "Hello World"),
        ("reverse", "hello", "olleh"),
    ];
    
    for (mode, input_text, expected) in test_cases {
        let input = WorkflowInput::new(input_text)
            .with_config(TextProcessingConfig {
                mode: mode.to_string(),
                ..Default::default()
            });
            
        let result = execute_workflow(&workflow, input).await;
        assert!(result.is_ok(), "Failed for mode: {}", mode);
        
        let output = result.unwrap();
        assert_eq!(output.result, expected, "Wrong result for mode: {}", mode);
    }
}

/// Test workflow performance under load
#[tokio::test]
async fn test_workflow_performance() {
    let workflow = create_test_workflow().await.unwrap();
    
    // Test with increasing text sizes
    let sizes = vec![100, 1000, 5000];
    
    for size in sizes {
        let large_text = "performance test ".repeat(size / 16);
        let input = WorkflowInput::new(large_text.clone());
        
        let start_time = std::time::Instant::now();
        let result = execute_workflow(&workflow, input).await;
        let execution_time = start_time.elapsed();
        
        assert!(result.is_ok(), "Failed for size: {}", size);
        assert!(execution_time.as_millis() < 5000, 
               "Too slow for size {}: {}ms", size, execution_time.as_millis());
    }
}

/// Test concurrent workflow executions
#[tokio::test]
async fn test_concurrent_executions() {
    let workflow = create_test_workflow().await.unwrap();
    
    // Create multiple concurrent executions
    let mut handles = vec![];
    
    for i in 0..10 {
        let workflow_clone = workflow.clone();
        let input = WorkflowInput::new(format!("Concurrent test {}", i));
        
        let handle = tokio::spawn(async move {
            execute_workflow(&workflow_clone, input).await
        });
        
        handles.push(handle);
    }
    
    // Wait for all executions to complete
    let results = futures::future::join_all(handles).await;
    
    // Verify all succeeded
    for (i, result) in results.into_iter().enumerate() {
        let workflow_result = result.unwrap();
        assert!(workflow_result.is_ok(), "Concurrent execution {} failed", i);
        
        let output = workflow_result.unwrap();
        assert_eq!(output.result, format!("CONCURRENT TEST {}", i));
    }
}

/// Test error propagation across workflow boundaries
#[tokio::test]
async fn test_error_propagation() {
    let workflow = create_test_workflow().await.unwrap();
    
    // Test various error conditions
    let error_cases = vec![
        ("empty_input", WorkflowInput::new("")),
        ("invalid_mode", WorkflowInput::new("test").with_config(
            TextProcessingConfig {
                mode: "invalid_mode".to_string(),
                ..Default::default()
            }
        )),
        ("too_long", WorkflowInput::new(&"x".repeat(20000)).with_config(
            TextProcessingConfig {
                max_length: Some(100),
                ..Default::default()
            }
        )),
    ];
    
    for (test_name, input) in error_cases {
        let result = execute_workflow(&workflow, input).await;
        assert!(result.is_err(), "Expected error for test: {}", test_name);
        
        match result.unwrap_err() {
            WorkflowError::ValidationError(_) => {
                // Expected validation error
            }
            e => panic!("Unexpected error type for {}: {}", test_name, e),
        }
    }
}

/// Test workflow timeout handling
#[tokio::test]
async fn test_workflow_timeout() {
    let workflow = create_test_workflow().await.unwrap();
    let input = WorkflowInput::new("Timeout test");
    
    // Execute with a very short timeout
    let result = timeout(
        Duration::from_millis(1), // 1ms timeout - should fail
        execute_workflow(&workflow, input)
    ).await;
    
    // Should timeout (this test verifies timeout infrastructure works)
    assert!(result.is_err(), "Expected timeout");
}

/// Test workflow state persistence (if event sourcing is enabled)
#[tokio::test]
async fn test_workflow_state_persistence() {
    let workflow = create_test_workflow().await.unwrap();
    let input = WorkflowInput::new("State persistence test");
    
    let result = execute_workflow(&workflow, input).await;
    assert!(result.is_ok());
    
    let output = result.unwrap();
    
    // Verify metadata contains execution information
    assert!(output.metadata.workflow_id.to_string().len() > 0);
    assert!(output.metadata.start_time <= output.metadata.end_time.unwrap());
    assert!(output.metadata.steps.len() > 0);
    
    // Verify processing steps are recorded
    assert_eq!(output.metadata.steps.len(), 3);
    assert_eq!(output.metadata.steps[0].node_type, "TextInputNode");
    assert_eq!(output.metadata.steps[1].node_type, "TextProcessorNode");
    assert_eq!(output.metadata.steps[2].node_type, "TextOutputNode");
}

/// Test workflow validation edge cases
#[tokio::test]
async fn test_workflow_validation_edge_cases() {
    // Test empty workflow
    let empty_workflow = Workflow::new("empty_test").unwrap();
    assert!(empty_workflow.validate().is_err());
    
    // Test disconnected workflow
    let mut disconnected_workflow = Workflow::new("disconnected_test").unwrap();
    disconnected_workflow.register_node("node1", Box::new(TextInputNode::new())).unwrap();
    disconnected_workflow.register_node("node2", Box::new(TextProcessorNode::new())).unwrap();
    // Don't connect the nodes
    assert!(disconnected_workflow.validate().is_err());
    
    // Test circular workflow
    let mut circular_workflow = Workflow::new("circular_test").unwrap();
    circular_workflow.register_node("node1", Box::new(TextInputNode::new())).unwrap();
    circular_workflow.register_node("node2", Box::new(TextProcessorNode::new())).unwrap();
    circular_workflow.connect("node1", "node2").unwrap();
    circular_workflow.connect("node2", "node1").unwrap(); // Creates cycle
    assert!(matches!(circular_workflow.validate(), Err(WorkflowError::CycleDetected)));
}

/// Test input sanitization and validation
#[tokio::test]
async fn test_input_sanitization() {
    let workflow = create_test_workflow().await.unwrap();
    
    // Test various problematic inputs
    let sanitization_tests = vec![
        ("null_bytes", "hello\x00world", "helloworld"),
        ("control_chars", "hello\x01\x02world", "helloworld"),
        ("mixed_whitespace", "  hello\t\nworld  ", "hello\t\nworld"), // Trim but preserve internal whitespace
    ];
    
    for (test_name, input_text, expected_clean) in sanitization_tests {
        let input = WorkflowInput::new(input_text);
        let result = execute_workflow(&workflow, input).await;
        
        // Should succeed with sanitized input
        assert!(result.is_ok(), "Sanitization failed for: {}", test_name);
        
        let output = result.unwrap();
        // Result should be based on sanitized input
        assert_eq!(output.result.to_lowercase(), expected_clean.to_uppercase().to_lowercase());
    }
}

/// Test workflow metadata collection
#[tokio::test]
async fn test_metadata_collection() {
    let workflow = create_test_workflow().await.unwrap();
    let input = WorkflowInput::new("Metadata test");
    
    let result = execute_workflow(&workflow, input).await;
    assert!(result.is_ok());
    
    let output = result.unwrap();
    let metadata = &output.metadata;
    
    // Verify all metadata fields are populated
    assert!(metadata.execution_time_ms.is_some());
    assert!(metadata.execution_time_ms.unwrap() > 0);
    assert_eq!(metadata.nodes_processed, 3);
    assert!(matches!(metadata.status, basic_workflow_example::WorkflowStatus::Completed));
    
    // Verify processing steps
    assert_eq!(metadata.steps.len(), 3);
    for (i, step) in metadata.steps.iter().enumerate() {
        assert_eq!(step.index, i);
        assert!(step.success);
        assert!(step.duration_ms.is_some());
        assert!(step.error.is_none());
    }
}

// Helper functions

async fn create_test_workflow() -> Result<Workflow, WorkflowError> {
    let mut workflow = Workflow::new("integration_test_workflow")?;
    
    workflow.register_node("input", Box::new(TextInputNode::new()))?;
    workflow.register_node("processor", Box::new(TextProcessorNode::new()))?;
    workflow.register_node("output", Box::new(TextOutputNode::new()))?;
    
    workflow.connect("input", "processor")?;
    workflow.connect("processor", "output")?;
    
    workflow.validate()?;
    Ok(workflow)
}

async fn execute_workflow(
    workflow: &Workflow,
    input: WorkflowInput,
) -> Result<WorkflowOutput, WorkflowError> {
    let mut context = TaskContext::new();
    context.set_event_data(json!(input))?;
    
    let result_context = workflow.execute(context).await?;
    let output: WorkflowOutput = result_context.get_data("final_output")?;
    
    Ok(output)
}