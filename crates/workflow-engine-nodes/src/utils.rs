//! Common utilities for workflow nodes
//! 
//! This module provides shared utilities and helpers
//! used across different node implementations.

use workflow_engine_core::prelude::*;

/// Create a mock task context for testing
pub fn mock_context() -> TaskContext {
    TaskContext::new(
        "test_workflow".to_string(),
        json!({"test": true})
    )
}

/// Assert that a node output contains expected data
pub fn assert_node_output(context: &TaskContext, key: &str, expected: &serde_json::Value) {
    if let Ok(data) = context.get_event_data::<serde_json::Value>() {
        if let Some(actual) = data.get(key) {
            assert_eq!(actual, expected, "Node output mismatch for key: {}", key);
        } else {
            panic!("Expected key '{}' not found in node output", key);
        }
    } else {
        panic!("Failed to get event data from context");
    }
}

/// Test fixture for common node scenarios
pub struct NodeTestFixture {
    pub input_data: serde_json::Value,
    pub expected_output: serde_json::Value,
}

impl NodeTestFixture {
    /// Create a new test fixture
    pub fn new(input: serde_json::Value, expected: serde_json::Value) -> Self {
        Self {
            input_data: input,
            expected_output: expected,
        }
    }
    
    /// Create a context with the fixture input data
    pub fn create_context(&self) -> TaskContext {
        TaskContext::new("test_fixture".to_string(), self.input_data.clone())
    }
    
    /// Validate the output matches expected results
    pub fn validate_output(&self, result: &TaskContext) {
        if let Ok(output) = result.get_event_data::<serde_json::Value>() {
            // Compare relevant fields
            let expected_obj = match self.expected_output.as_object() {
                Some(obj) => obj,
                None => {
                    panic!("Expected output is not a JSON object");
                }
            };
            
            for (key, expected_value) in expected_obj {
                if let Some(actual_value) = output.get(key) {
                    assert_eq!(actual_value, expected_value, 
                        "Output mismatch for key '{}': expected {:?}, got {:?}", 
                        key, expected_value, actual_value);
                }
            }
        }
    }
}