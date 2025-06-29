use workflow_engine_core::{
    error::WorkflowError,
    nodes::{Node, registry::NodeRegistry},
    task::TaskContext,
    workflow::{Workflow, schema::{WorkflowSchema, NodeConfig}},
};
use std::any::TypeId;
use serde_json::json;

// Test nodes for dependency testing
#[derive(Debug)]
struct StartNode;

impl Node for StartNode {
    fn process(&self, mut ctx: TaskContext) -> Result<TaskContext, WorkflowError> {
        ctx.set_data("started", true)?;
        Ok(ctx)
    }
}

#[derive(Debug)]
struct MissingNode;

impl Node for MissingNode {
    fn process(&self, ctx: TaskContext) -> Result<TaskContext, WorkflowError> {
        Ok(ctx)
    }
}

#[derive(Debug)]
struct AINode {
    api_key: Option<String>,
}

impl Node for AINode {
    fn process(&self, mut ctx: TaskContext) -> Result<TaskContext, WorkflowError> {
        if self.api_key.is_none() {
            return Err(WorkflowError::ConfigurationError(
                "AI API key not configured".to_string()
            ));
        }
        ctx.set_data("ai_processed", true)?;
        Ok(ctx)
    }
}

#[derive(Debug)]
struct ExternalServiceNode {
    service_url: String,
    connected: bool,
}

impl Node for ExternalServiceNode {
    fn process(&self, mut ctx: TaskContext) -> Result<TaskContext, WorkflowError> {
        if !self.connected {
            return Err(WorkflowError::ProcessingError {
                message: format!("Failed to connect to external service: {}", self.service_url),
            });
        }
        ctx.set_data("external_processed", true)?;
        Ok(ctx)
    }
}

#[test]
fn test_workflow_with_missing_node_type() {
    // Create a workflow schema that references a node that won't be registered
    let schema = WorkflowSchema {
        workflow_type: "missing_node_workflow".to_string(),
        start: TypeId::of::<StartNode>(),
        nodes: vec![
            NodeConfig {
                node_type: TypeId::of::<StartNode>(),
                connections: vec![TypeId::of::<MissingNode>()],
                parallel_nodes: vec![],
                is_router: false,
            },
            NodeConfig {
                node_type: TypeId::of::<MissingNode>(),
                connections: vec![],
                parallel_nodes: vec![],
                is_router: false,
            },
        ],
        description: None,
    };

    let workflow = Workflow::new(schema).expect("Workflow creation should succeed");
    
    // Register only the start node, not the missing node
    workflow.register_node(StartNode);
    
    // Try to run the workflow - should fail with NodeNotFound error
    let result = workflow.run(json!({"test": "data"}));
    
    match result {
        Err(WorkflowError::NodeNotFound { node_type }) => {
            assert_eq!(node_type, TypeId::of::<MissingNode>());
        }
        Ok(_) => panic!("Expected workflow to fail with missing node"),
        Err(e) => panic!("Expected NodeNotFound error, got: {:?}", e),
    }
}

#[test]
fn test_workflow_with_missing_ai_provider_key() {
    let schema = WorkflowSchema {
        workflow_type: "ai_workflow".to_string(),
        start: TypeId::of::<StartNode>(),
        nodes: vec![
            NodeConfig {
                node_type: TypeId::of::<StartNode>(),
                connections: vec![TypeId::of::<AINode>()],
                parallel_nodes: vec![],
                is_router: false,
            },
            NodeConfig {
                node_type: TypeId::of::<AINode>(),
                connections: vec![],
                parallel_nodes: vec![],
                is_router: false,
            },
        ],
        description: None,
    };

    let workflow = Workflow::new(schema).expect("Workflow creation should succeed");
    
    // Register nodes
    workflow.register_node(StartNode);
    workflow.register_node(AINode { api_key: None }); // Missing API key
    
    // Try to run the workflow - should fail with ConfigurationError
    let result = workflow.run(json!({"prompt": "Hello AI"}));
    
    match result {
        Err(WorkflowError::ConfigurationError(msg)) => {
            assert!(msg.contains("API key not configured"));
        }
        Ok(_) => panic!("Expected workflow to fail with missing API key"),
        Err(e) => panic!("Expected ConfigurationError, got: {:?}", e),
    }
}

#[test]
fn test_workflow_with_failed_external_service_connection() {
    let schema = WorkflowSchema {
        workflow_type: "external_service_workflow".to_string(),
        start: TypeId::of::<StartNode>(),
        nodes: vec![
            NodeConfig {
                node_type: TypeId::of::<StartNode>(),
                connections: vec![TypeId::of::<ExternalServiceNode>()],
                parallel_nodes: vec![],
                is_router: false,
            },
            NodeConfig {
                node_type: TypeId::of::<ExternalServiceNode>(),
                connections: vec![],
                parallel_nodes: vec![],
                is_router: false,
            },
        ],
        description: None,
    };

    let workflow = Workflow::new(schema).expect("Workflow creation should succeed");
    
    // Register nodes
    workflow.register_node(StartNode);
    workflow.register_node(ExternalServiceNode {
        service_url: "http://unavailable-service.local".to_string(),
        connected: false, // Simulate connection failure
    });
    
    // Try to run the workflow - should fail with ProcessingError
    let result = workflow.run(json!({"data": "test"}));
    
    match result {
        Err(WorkflowError::ProcessingError { message }) => {
            assert!(message.contains("Failed to connect to external service"));
        }
        Ok(_) => panic!("Expected workflow to fail with external service connection error"),
        Err(e) => panic!("Expected ProcessingError, got: {:?}", e),
    }
}

#[test]
fn test_workflow_with_parallel_node_missing() {
    let schema = WorkflowSchema {
        workflow_type: "parallel_missing_workflow".to_string(),
        start: TypeId::of::<StartNode>(),
        nodes: vec![
            NodeConfig {
                node_type: TypeId::of::<StartNode>(),
                connections: vec![],
                parallel_nodes: vec![TypeId::of::<MissingNode>()], // Parallel node not registered
                is_router: false,
            },
        ],
        description: None,
    };

    let workflow = Workflow::new(schema).expect("Workflow creation should succeed");
    
    // Register only the start node
    workflow.register_node(StartNode);
    
    // Try to run the workflow - should fail when trying to execute parallel node
    let result = workflow.run(json!({"test": "parallel"}));
    
    match result {
        Err(WorkflowError::NodeNotFound { node_type }) => {
            assert_eq!(node_type, TypeId::of::<MissingNode>());
        }
        Ok(_) => panic!("Expected workflow to fail with missing parallel node"),
        Err(e) => panic!("Expected NodeNotFound error, got: {:?}", e),
    }
}

#[test]
fn test_workflow_graceful_recovery_from_missing_node() {
    // Test that we can handle missing nodes gracefully and provide helpful error messages
    let schema = WorkflowSchema {
        workflow_type: "recovery_workflow".to_string(),
        start: TypeId::of::<StartNode>(),
        nodes: vec![
            NodeConfig {
                node_type: TypeId::of::<StartNode>(),
                connections: vec![TypeId::of::<MissingNode>()],
                parallel_nodes: vec![],
                is_router: false,
            },
            NodeConfig {
                node_type: TypeId::of::<MissingNode>(),
                connections: vec![TypeId::of::<AINode>()],
                parallel_nodes: vec![],
                is_router: false,
            },
            NodeConfig {
                node_type: TypeId::of::<AINode>(),
                connections: vec![],
                parallel_nodes: vec![],
                is_router: false,
            },
        ],
        description: None,
    };

    let workflow = Workflow::new(schema).expect("Workflow creation should succeed");
    
    // Register some nodes but not all
    workflow.register_node(StartNode);
    workflow.register_node(AINode { api_key: Some("test-key".to_string()) });
    // MissingNode is not registered
    
    let result = workflow.run(json!({"test": "recovery"}));
    
    // Should fail at the missing node, not continue to AINode
    match result {
        Err(WorkflowError::NodeNotFound { node_type }) => {
            assert_eq!(node_type, TypeId::of::<MissingNode>());
        }
        _ => panic!("Expected NodeNotFound error for MissingNode"),
    }
}

#[test]
fn test_workflow_validation_with_missing_dependencies() {
    // Test that workflow validation catches missing dependencies early
    let schema = WorkflowSchema {
        workflow_type: "validation_test".to_string(),
        start: TypeId::of::<StartNode>(),
        nodes: vec![
            NodeConfig {
                node_type: TypeId::of::<StartNode>(),
                connections: vec![TypeId::of::<MissingNode>()],
                parallel_nodes: vec![],
                is_router: false,
            },
            // Note: MissingNode is not even defined in the schema
        ],
        description: None,
    };

    // This should pass validation (schema is structurally valid)
    let workflow = Workflow::new(schema);
    assert!(workflow.is_ok(), "Workflow creation should succeed even with references to undefined nodes");
    
    // But execution should fail when the node is not registered
    let workflow = workflow.unwrap();
    workflow.register_node(StartNode);
    
    let result = workflow.run(json!({}));
    assert!(result.is_err(), "Workflow execution should fail with missing node");
}

// Test for checking error propagation in complex workflows
#[test]
fn test_error_propagation_in_workflow_chain() {
    #[derive(Debug)]
    struct MiddleNode;
    
    impl Node for MiddleNode {
        fn process(&self, mut ctx: TaskContext) -> Result<TaskContext, WorkflowError> {
            ctx.set_data("middle_processed", true)?;
            Ok(ctx)
        }
    }

    let schema = WorkflowSchema {
        workflow_type: "error_propagation_workflow".to_string(),
        start: TypeId::of::<StartNode>(),
        nodes: vec![
            NodeConfig {
                node_type: TypeId::of::<StartNode>(),
                connections: vec![TypeId::of::<MiddleNode>()],
                parallel_nodes: vec![],
                is_router: false,
            },
            NodeConfig {
                node_type: TypeId::of::<MiddleNode>(),
                connections: vec![TypeId::of::<ExternalServiceNode>()],
                parallel_nodes: vec![],
                is_router: false,
            },
            NodeConfig {
                node_type: TypeId::of::<ExternalServiceNode>(),
                connections: vec![],
                parallel_nodes: vec![],
                is_router: false,
            },
        ],
        description: None,
    };

    let workflow = Workflow::new(schema).expect("Workflow creation should succeed");
    
    workflow.register_node(StartNode);
    workflow.register_node(MiddleNode);
    workflow.register_node(ExternalServiceNode {
        service_url: "http://failing-service.local".to_string(),
        connected: false,
    });
    
    let result = workflow.run(json!({"test": "error_chain"}));
    
    // Error should propagate from the external service node
    match result {
        Err(WorkflowError::ProcessingError { message }) => {
            assert!(message.contains("Failed to connect to external service"));
        }
        _ => panic!("Expected ProcessingError from external service"),
    }
}