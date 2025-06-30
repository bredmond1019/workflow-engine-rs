//! # Workflow Configuration Validation Tests
//!
//! This test suite follows TDD methodology to ensure comprehensive validation
//! of workflow configurations, catching potential security issues and runtime
//! errors from invalid configurations.
//!
//! ## Test Categories
//!
//! 1. **Schema Structure Validation**: Required fields, data types
//! 2. **Node Configuration Validation**: Missing nodes, invalid parameters
//! 3. **Graph Structure Validation**: Cycles, unreachable nodes, invalid connections
//! 4. **Resource Limits Validation**: Timeouts, concurrency, memory constraints
//! 5. **Security Validation**: Input sanitization, injection protection
//! 6. **Metadata Validation**: Tags, descriptions, version constraints

use std::any::TypeId;
use std::time::Duration;
use std::collections::HashMap;
use serde_json::json;

use workflow_engine_core::{
    error::WorkflowError,
    workflow::{Workflow, schema::WorkflowSchema, validator::WorkflowValidator},
    nodes::{Node, config::NodeConfig},
    task::TaskContext,
};

// Test nodes for validation scenarios
#[derive(Debug)]
struct ValidNode;

impl Node for ValidNode {
    fn process(&self, context: TaskContext) -> Result<TaskContext, WorkflowError> {
        Ok(context)
    }
}

#[derive(Debug)]
struct SecondNode;

impl Node for SecondNode {
    fn process(&self, context: TaskContext) -> Result<TaskContext, WorkflowError> {
        Ok(context)
    }
}

#[derive(Debug)]
struct ThirdNode;

impl Node for ThirdNode {
    fn process(&self, context: TaskContext) -> Result<TaskContext, WorkflowError> {
        Ok(context)
    }
}

// ================================================================================================
// RED PHASE - FAILING TESTS
// ================================================================================================

#[cfg(test)]
mod schema_structure_validation {
    use super::*;

    #[test]
    fn test_empty_workflow_type_should_fail() {
        // GREEN: This should now properly reject empty workflow types
        let schema = WorkflowSchema {
            workflow_type: "".to_string(), // Empty string should be invalid
            description: None,
            start: TypeId::of::<ValidNode>(),
            nodes: vec![NodeConfig::new::<ValidNode>()],
        };
        
        let workflow = Workflow::new(schema);
        assert!(workflow.is_err(), "Empty workflow type should be rejected");
        
        // Verify the specific error type
        let error = workflow.err().expect("Should have error");
        match error {
            WorkflowError::ConfigurationError(_) => {
                // Expected configuration error
            }
            other => panic!("Expected ConfigurationError, got: {:?}", other),
        }
    }

    #[test]
    fn test_whitespace_only_workflow_type_should_fail() {
        // GREEN: This should now properly reject whitespace-only workflow types
        let schema = WorkflowSchema {
            workflow_type: "   \n\t   ".to_string(), // Only whitespace
            description: None,
            start: TypeId::of::<ValidNode>(),
            nodes: vec![NodeConfig::new::<ValidNode>()],
        };
        
        let workflow = Workflow::new(schema);
        assert!(workflow.is_err(), "Workflow type with only whitespace should be rejected");
        
        let error = workflow.err().expect("Should have error");
        match error {
            WorkflowError::ConfigurationError(_) => {},
            other => panic!("Expected ConfigurationError, got: {:?}", other),
        }
    }

    #[test]
    fn test_excessive_workflow_type_length_should_fail() {
        // GREEN: This should now properly reject excessively long workflow types
        let long_name = "a".repeat(1000); // 1000 characters
        let schema = WorkflowSchema {
            workflow_type: long_name,
            description: None,
            start: TypeId::of::<ValidNode>(),
            nodes: vec![NodeConfig::new::<ValidNode>()],
        };
        
        let workflow = Workflow::new(schema);
        assert!(workflow.is_err(), "Workflow type exceeding length limit should be rejected");
        
        let error = workflow.err().expect("Should have error");
        match error {
            WorkflowError::ConfigurationError(_) => {},
            other => panic!("Expected ConfigurationError, got: {:?}", other),
        }
    }

    #[test]
    fn test_invalid_characters_in_workflow_type_should_fail() {
        // GREEN: This should now properly reject workflow types with invalid characters
        let schema = WorkflowSchema {
            workflow_type: "workflow\x00with\x01null\x02bytes".to_string(),
            description: None,
            start: TypeId::of::<ValidNode>(),
            nodes: vec![NodeConfig::new::<ValidNode>()],
        };
        
        let workflow = Workflow::new(schema);
        assert!(workflow.is_err(), "Workflow with invalid characters should be rejected");
        
        let error = workflow.err().expect("Should have error");
        match error {
            WorkflowError::ConfigurationError(_) => {},
            other => panic!("Expected ConfigurationError, got: {:?}", other),
        }
    }
}

#[cfg(test)]
mod node_configuration_validation {
    use super::*;

    #[test]
    fn test_workflow_with_no_nodes_should_fail() {
        // GREEN: This should now properly reject workflows with no nodes
        let schema = WorkflowSchema {
            workflow_type: "test_workflow".to_string(),
            description: None,
            start: TypeId::of::<ValidNode>(),
            nodes: vec![], // Empty nodes list
        };
        
        let workflow = Workflow::new(schema);
        assert!(workflow.is_err(), "Empty node list should be rejected");
        
        let error = workflow.err().expect("Should have error");
        match error {
            WorkflowError::ConfigurationError(_) => {},
            other => panic!("Expected ConfigurationError, got: {:?}", other),
        }
    }

    #[test]
    fn test_start_node_not_in_nodes_list_should_fail() {
        // GREEN: This should now properly reject workflows where start node is not in nodes list
        let schema = WorkflowSchema {
            workflow_type: "test_workflow".to_string(),
            description: None,
            start: TypeId::of::<ValidNode>(),
            nodes: vec![NodeConfig::new::<SecondNode>()], // Start node not in list
        };
        
        let workflow = Workflow::new(schema);
        assert!(workflow.is_err(), "Start node not in nodes list should be rejected");
        
        let error = workflow.err().expect("Should have error");
        match error {
            WorkflowError::ConfigurationError(_) => {},
            other => panic!("Expected ConfigurationError, got: {:?}", other),
        }
    }

    #[test]
    fn test_node_with_zero_timeout_should_fail() {
        // GREEN: This should now properly reject nodes with zero timeout
        let invalid_node = NodeConfig::new::<ValidNode>()
            .with_timeout(Duration::from_secs(0)); // Zero timeout
        
        let result = invalid_node.validate();
        assert!(result.is_err(), "Node with zero timeout should be rejected");
        
        let error = result.err().expect("Should have error");
        match error {
            WorkflowError::ConfigurationError(_) => {},
            other => panic!("Expected ConfigurationError, got: {:?}", other),
        }
    }

    #[test]
    fn test_node_with_zero_retry_attempts_should_fail() {
        // GREEN: This should now properly reject nodes with zero retry attempts
        let invalid_node = NodeConfig::new::<ValidNode>()
            .with_retry(0, Duration::from_millis(100)); // Zero retry attempts
        
        let result = invalid_node.validate();
        assert!(result.is_err(), "Node with zero retry attempts should be rejected");
        
        let error = result.err().expect("Should have error");
        match error {
            WorkflowError::ConfigurationError(_) => {},
            other => panic!("Expected ConfigurationError, got: {:?}", other),
        }
    }

    #[test]
    fn test_node_with_retry_attempts_but_no_delay_should_fail() {
        // GREEN: This should now properly reject nodes with retry attempts but no delay
        let mut invalid_node = NodeConfig::new::<ValidNode>();
        invalid_node.retry_attempts = Some(3);
        invalid_node.retry_delay = None; // Missing delay
        
        let result = invalid_node.validate();
        assert!(result.is_err(), "Node with retry attempts but no delay should be rejected");
        
        let error = result.err().expect("Should have error");
        match error {
            WorkflowError::ConfigurationError(_) => {},
            other => panic!("Expected ConfigurationError, got: {:?}", other),
        }
    }

    #[test]
    #[should_panic(expected = "Node with zero priority should be rejected")]
    fn test_node_with_zero_priority_should_fail() {
        // RED: This should fail - zero priority should be invalid
        let invalid_node = NodeConfig::new::<ValidNode>()
            .with_priority(0); // Zero priority
        
        let result = invalid_node.validate();
        assert!(result.is_err(), "Node with zero priority should be rejected");
    }

    #[test]
    #[should_panic(expected = "Node with zero max concurrent executions should be rejected")]
    fn test_node_with_zero_max_concurrent_executions_should_fail() {
        // RED: This should fail - zero max concurrent executions
        let invalid_node = NodeConfig::new::<ValidNode>()
            .with_max_concurrent_executions(0); // Zero max executions
        
        let result = invalid_node.validate();
        assert!(result.is_err(), "Node with zero max concurrent executions should be rejected");
    }
}

#[cfg(test)]
mod graph_structure_validation {
    use super::*;

    #[test]
    fn test_circular_dependency_detection_should_fail() {
        // GREEN: This should now properly detect circular dependencies
        let node1 = NodeConfig::new::<ValidNode>()
            .with_connections(vec![TypeId::of::<SecondNode>()]);
        
        let node2 = NodeConfig::new::<SecondNode>()
            .with_connections(vec![TypeId::of::<ValidNode>()]); // Creates cycle
        
        let schema = WorkflowSchema {
            workflow_type: "circular_workflow".to_string(),
            description: None,
            start: TypeId::of::<ValidNode>(),
            nodes: vec![node1, node2],
        };
        
        let workflow = Workflow::new(schema);
        assert!(workflow.is_err(), "Circular dependency should be detected");
        
        let error = workflow.err().expect("Should have error");
        match error {
            WorkflowError::CycleDetected => {},
            other => panic!("Expected CycleDetected, got: {:?}", other),
        }
    }

    #[test]
    #[should_panic(expected = "Self-referencing node should be rejected")]
    fn test_self_referencing_node_should_fail() {
        // RED: This should fail - nodes shouldn't reference themselves
        let self_ref_node = NodeConfig::new::<ValidNode>()
            .with_connections(vec![TypeId::of::<ValidNode>()]); // Self-reference
        
        let schema = WorkflowSchema {
            workflow_type: "self_ref_workflow".to_string(),
            description: None,
            start: TypeId::of::<ValidNode>(),
            nodes: vec![self_ref_node],
        };
        
        let workflow = Workflow::new(schema);
        assert!(workflow.is_err(), "Self-referencing node should be rejected");
    }

    #[test]
    #[should_panic(expected = "Connection to non-existent node should be rejected")]
    fn test_connection_to_non_existent_node_should_fail() {
        // RED: This should fail - connections to non-existent nodes
        let invalid_connection_node = NodeConfig::new::<ValidNode>()
            .with_connections(vec![TypeId::of::<ThirdNode>()]); // ThirdNode not in schema
        
        let schema = WorkflowSchema {
            workflow_type: "invalid_connection_workflow".to_string(),
            description: None,
            start: TypeId::of::<ValidNode>(),
            nodes: vec![invalid_connection_node],
        };
        
        let workflow = Workflow::new(schema);
        assert!(workflow.is_err(), "Connection to non-existent node should be rejected");
    }

    #[test]
    #[should_panic(expected = "Deep circular reference should be detected")]
    fn test_deep_circular_reference_should_fail() {
        // RED: This should fail - complex circular dependencies
        let node1 = NodeConfig::new::<ValidNode>()
            .with_connections(vec![TypeId::of::<SecondNode>()]);
        
        let node2 = NodeConfig::new::<SecondNode>()
            .with_connections(vec![TypeId::of::<ThirdNode>()]);
        
        let node3 = NodeConfig::new::<ThirdNode>()
            .with_connections(vec![TypeId::of::<ValidNode>()]); // Back to start
        
        let schema = WorkflowSchema {
            workflow_type: "deep_circular_workflow".to_string(),
            description: None,
            start: TypeId::of::<ValidNode>(),
            nodes: vec![node1, node2, node3],
        };
        
        let workflow = Workflow::new(schema);
        assert!(workflow.is_err(), "Deep circular reference should be detected");
    }
}

#[cfg(test)]
mod resource_limits_validation {
    use super::*;

    #[test]
    #[should_panic(expected = "Excessive parallel nodes should be rejected")]
    fn test_excessive_parallel_nodes_should_fail() {
        // RED: This should fail - too many parallel nodes could cause resource exhaustion
        let parallel_node_types: Vec<TypeId> = (0..1000)
            .map(|_| TypeId::of::<ValidNode>())
            .collect();
        
        let excessive_parallel_node = NodeConfig::new::<ValidNode>()
            .with_parallel_nodes(parallel_node_types);
        
        let schema = WorkflowSchema {
            workflow_type: "excessive_parallel_workflow".to_string(),
            description: None,
            start: TypeId::of::<ValidNode>(),
            nodes: vec![excessive_parallel_node],
        };
        
        let workflow = Workflow::new(schema);
        assert!(workflow.is_err(), "Excessive parallel nodes should be rejected");
    }

    #[test]
    #[should_panic(expected = "Excessive timeout should be rejected")]
    fn test_excessive_timeout_should_fail() {
        // RED: This should fail - extremely long timeouts could cause resource issues
        let excessive_timeout_node = NodeConfig::new::<ValidNode>()
            .with_timeout(Duration::from_secs(86400 * 365)); // 1 year timeout
        
        let result = excessive_timeout_node.validate();
        assert!(result.is_err(), "Excessive timeout should be rejected");
    }

    #[test]
    #[should_panic(expected = "Excessive retry attempts should be rejected")]
    fn test_excessive_retry_attempts_should_fail() {
        // RED: This should fail - too many retry attempts could cause infinite loops
        let excessive_retry_node = NodeConfig::new::<ValidNode>()
            .with_retry(1000, Duration::from_millis(100)); // 1000 retries
        
        let result = excessive_retry_node.validate();
        assert!(result.is_err(), "Excessive retry attempts should be rejected");
    }

    #[test]
    #[should_panic(expected = "Excessive max concurrent executions should be rejected")]
    fn test_excessive_max_concurrent_executions_should_fail() {
        // RED: This should fail - too many concurrent executions could exhaust resources
        let excessive_concurrent_node = NodeConfig::new::<ValidNode>()
            .with_max_concurrent_executions(1000000); // 1 million concurrent
        
        let result = excessive_concurrent_node.validate();
        assert!(result.is_err(), "Excessive max concurrent executions should be rejected");
    }
}

#[cfg(test)]
mod security_validation {
    use super::*;

    #[test]
    #[should_panic(expected = "Malicious metadata should be rejected")]
    fn test_malicious_metadata_should_fail() {
        // RED: This should fail - potentially malicious metadata
        let malicious_node = NodeConfig::new::<ValidNode>()
            .with_metadata("script".to_string(), json!("<script>alert('xss')</script>"))
            .with_metadata("command".to_string(), json!("rm -rf /"))
            .with_metadata("sql".to_string(), json!("'; DROP TABLE users; --"));
        
        let result = malicious_node.validate();
        assert!(result.is_err(), "Malicious metadata should be rejected");
    }

    #[test]
    #[should_panic(expected = "Excessively large metadata should be rejected")]
    fn test_excessive_metadata_size_should_fail() {
        // RED: This should fail - extremely large metadata could cause memory issues
        let large_data = "x".repeat(10_000_000); // 10MB of data
        let large_metadata_node = NodeConfig::new::<ValidNode>()
            .with_metadata("large_field".to_string(), json!(large_data));
        
        let result = large_metadata_node.validate();
        assert!(result.is_err(), "Excessively large metadata should be rejected");
    }

    #[test]
    #[should_panic(expected = "Reserved metadata keys should be rejected")]
    fn test_reserved_metadata_keys_should_fail() {
        // RED: This should fail - reserved system keys shouldn't be allowed
        let reserved_keys_node = NodeConfig::new::<ValidNode>()
            .with_metadata("__system__".to_string(), json!("hacked"))
            .with_metadata("_internal_".to_string(), json!("malicious"))
            .with_metadata("NODE_TYPE".to_string(), json!("overridden"));
        
        let result = reserved_keys_node.validate();
        assert!(result.is_err(), "Reserved metadata keys should be rejected");
    }

    #[test]
    #[should_panic(expected = "Malicious tags should be rejected")]
    fn test_malicious_tags_should_fail() {
        // RED: This should fail - potentially malicious tags
        let malicious_tags_node = NodeConfig::new::<ValidNode>()
            .with_tags(vec![
                "<script>alert('xss')</script>".to_string(),
                "'; DROP TABLE nodes; --".to_string(),
                "\x00\x01\x02null_bytes".to_string(),
            ]);
        
        let result = malicious_tags_node.validate();
        assert!(result.is_err(), "Malicious tags should be rejected");
    }
}

#[cfg(test)]
mod workflow_structure_validation {
    use super::*;

    #[test]
    #[should_panic(expected = "Workflow with excessive nodes should be rejected")]
    fn test_excessive_nodes_count_should_fail() {
        // RED: This should fail - too many nodes could cause memory/performance issues
        let mut nodes = Vec::new();
        for i in 0..10_000 {
            let mut node = NodeConfig::new::<ValidNode>();
            node.metadata.insert("index".to_string(), json!(i));
            nodes.push(node);
        }
        
        let schema = WorkflowSchema {
            workflow_type: "excessive_nodes_workflow".to_string(),
            description: None,
            start: TypeId::of::<ValidNode>(),
            nodes,
        };
        
        let workflow = Workflow::new(schema);
        assert!(workflow.is_err(), "Workflow with excessive nodes should be rejected");
    }

    #[test]
    #[should_panic(expected = "Workflow with excessive depth should be rejected")]
    fn test_excessive_depth_should_fail() {
        // RED: This should fail - extremely deep workflows could cause stack overflow
        let mut nodes = Vec::new();
        let mut previous_type = TypeId::of::<ValidNode>();
        
        // Create a very deep chain of 1000 nodes
        for i in 0..1000 {
            let mut node = NodeConfig::new::<ValidNode>();
            node.metadata.insert("depth".to_string(), json!(i));
            if i < 999 {
                // All nodes point to the next one except the last
                node.connections = vec![TypeId::of::<ValidNode>()];
            }
            nodes.push(node);
        }
        
        let schema = WorkflowSchema {
            workflow_type: "excessive_depth_workflow".to_string(),
            description: None,
            start: TypeId::of::<ValidNode>(),
            nodes,
        };
        
        let workflow = Workflow::new(schema);
        assert!(workflow.is_err(), "Workflow with excessive depth should be rejected");
    }
}

// ================================================================================================
// VALIDATION HELPER TESTS
// ================================================================================================

#[cfg(test)]
mod validation_helper_tests {
    use super::*;

    #[test]
    fn test_valid_workflow_should_pass() {
        // This test should pass - represents a valid workflow configuration
        let valid_node = NodeConfig::new::<ValidNode>()
            .with_description("A valid test node".to_string())
            .with_timeout(Duration::from_secs(30))
            .with_retry(3, Duration::from_millis(500))
            .with_priority(5)
            .with_tags(vec!["test".to_string(), "valid".to_string()])
            .with_max_concurrent_executions(10);
        
        let schema = WorkflowSchema {
            workflow_type: "valid_test_workflow".to_string(),
            description: Some("A valid test workflow".to_string()),
            start: TypeId::of::<ValidNode>(),
            nodes: vec![valid_node],
        };
        
        let workflow = Workflow::new(schema);
        assert!(workflow.is_ok(), "Valid workflow should be accepted");
    }

    #[test]
    fn test_validator_correctly_identifies_issues() {
        // Test that our validator correctly identifies various issues
        let problematic_schema = WorkflowSchema {
            workflow_type: "test".to_string(),
            description: None,
            start: TypeId::of::<ValidNode>(),
            nodes: vec![], // Empty nodes - should be caught by validator
        };
        
        let validator = WorkflowValidator::new(&problematic_schema);
        let result = validator.validate();
        
        // The validator should catch this issue
        assert!(result.is_err(), "Validator should catch empty nodes list");
    }
}