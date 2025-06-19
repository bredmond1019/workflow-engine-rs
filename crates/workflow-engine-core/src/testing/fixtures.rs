/// Common test fixtures for workflow testing

use uuid::Uuid;
use crate::workflow::schema::WorkflowSchema;
use crate::nodes::config::NodeConfig;
use crate::task::TaskContext;
use std::collections::HashMap;
use std::any::TypeId;
use serde_json::json;

/// Creates a simple test workflow definition
pub fn create_test_workflow(name: &str) -> WorkflowDefinition {
    WorkflowDefinition {
        id: Uuid::new_v4(),
        name: name.to_string(),
        description: Some(format!("Test workflow: {}", name)),
        version: "1.0.0".to_string(),
        tags: vec!["test".to_string()],
        metadata: HashMap::new(),
        nodes: vec![
            NodeDefinition {
                id: "start".to_string(),
                name: "Start Node".to_string(),
                node_type: "start".to_string(),
                configuration: NodeConfiguration {
                    parameters: HashMap::new(),
                    inputs: HashMap::new(),
                    outputs: HashMap::new(),
                },
                description: None,
            },
            NodeDefinition {
                id: "process".to_string(), 
                name: "Process Node".to_string(),
                node_type: "process".to_string(),
                configuration: NodeConfiguration {
                    parameters: HashMap::new(),
                    inputs: HashMap::new(),
                    outputs: HashMap::new(),
                },
                description: None,
            },
            NodeDefinition {
                id: "end".to_string(),
                name: "End Node".to_string(),
                node_type: "end".to_string(),
                configuration: NodeConfiguration {
                    parameters: HashMap::new(),
                    inputs: HashMap::new(),
                    outputs: HashMap::new(),
                },
                description: None,
            },
        ],
        edges: vec![
            crate::workflow::EdgeDefinition {
                id: "edge1".to_string(),
                source_node: "start".to_string(),
                target_node: "process".to_string(),
                condition: None,
            },
            crate::workflow::EdgeDefinition {
                id: "edge2".to_string(),
                source_node: "process".to_string(),
                target_node: "end".to_string(),
                condition: None,
            },
        ],
    }
}

/// Creates a test workflow context
pub fn create_test_context() -> WorkflowContext {
    WorkflowContext {
        workflow_id: Uuid::new_v4(),
        execution_id: Uuid::new_v4(),
        user_id: Some("test-user".to_string()),
        correlation_id: Some(Uuid::new_v4()),
        metadata: HashMap::new(),
        variables: HashMap::from([
            ("test_var".to_string(), json!("test_value")),
        ]),
    }
}

/// Creates a test workflow with conditional branching
pub fn create_branching_workflow() -> WorkflowDefinition {
    WorkflowDefinition {
        id: Uuid::new_v4(),
        name: "branching-workflow".to_string(),
        description: Some("Test workflow with conditional branching".to_string()),
        version: "1.0.0".to_string(),
        tags: vec!["test".to_string(), "branching".to_string()],
        metadata: HashMap::new(),
        nodes: vec![
            NodeDefinition {
                id: "start".to_string(),
                name: "Start Node".to_string(),
                node_type: "start".to_string(),
                configuration: NodeConfiguration {
                    parameters: HashMap::new(),
                    inputs: HashMap::new(),
                    outputs: HashMap::new(),
                },
                description: None,
            },
            NodeDefinition {
                id: "condition".to_string(),
                name: "Condition Node".to_string(),
                node_type: "condition".to_string(),
                configuration: NodeConfiguration {
                    parameters: HashMap::from([
                        ("condition".to_string(), json!("{{test_var}} == 'true'")),
                    ]),
                    inputs: HashMap::new(),
                    outputs: HashMap::new(),
                },
                description: Some("Evaluates a condition".to_string()),
            },
            NodeDefinition {
                id: "true_branch".to_string(),
                name: "True Branch".to_string(),
                node_type: "process".to_string(),
                configuration: NodeConfiguration {
                    parameters: HashMap::new(),
                    inputs: HashMap::new(),
                    outputs: HashMap::new(),
                },
                description: None,
            },
            NodeDefinition {
                id: "false_branch".to_string(),
                name: "False Branch".to_string(),
                node_type: "process".to_string(),
                configuration: NodeConfiguration {
                    parameters: HashMap::new(),
                    inputs: HashMap::new(),
                    outputs: HashMap::new(),
                },
                description: None,
            },
            NodeDefinition {
                id: "end".to_string(),
                name: "End Node".to_string(),
                node_type: "end".to_string(),
                configuration: NodeConfiguration {
                    parameters: HashMap::new(),
                    inputs: HashMap::new(),
                    outputs: HashMap::new(),
                },
                description: None,
            },
        ],
        edges: vec![
            crate::workflow::EdgeDefinition {
                id: "edge1".to_string(),
                source_node: "start".to_string(),
                target_node: "condition".to_string(),
                condition: None,
            },
            crate::workflow::EdgeDefinition {
                id: "edge_true".to_string(),
                source_node: "condition".to_string(),
                target_node: "true_branch".to_string(),
                condition: Some("true".to_string()),
            },
            crate::workflow::EdgeDefinition {
                id: "edge_false".to_string(),
                source_node: "condition".to_string(),
                target_node: "false_branch".to_string(),
                condition: Some("false".to_string()),
            },
            crate::workflow::EdgeDefinition {
                id: "edge_end_true".to_string(),
                source_node: "true_branch".to_string(),
                target_node: "end".to_string(),
                condition: None,
            },
            crate::workflow::EdgeDefinition {
                id: "edge_end_false".to_string(),
                source_node: "false_branch".to_string(),
                target_node: "end".to_string(),
                condition: None,
            },
        ],
    }
}