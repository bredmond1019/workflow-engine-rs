//! Enhanced workflow builder with type-safe configuration
//!
//! This module provides an improved workflow builder that ensures
//! type safety and provides a fluent interface for workflow creation.

use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;

use crate::{
    error::WorkflowError,
    nodes::{Node, config::NodeConfig},
    workflow::{Workflow, schema::WorkflowSchema},
};

/// Type-safe workflow builder with compile-time guarantees
pub struct TypedWorkflowBuilder<StartNode: Node + 'static> {
    schema: WorkflowSchema,
    node_configs: HashMap<TypeId, NodeConfig>,
    validation_rules: Vec<Box<dyn Fn(&WorkflowSchema) -> Result<(), WorkflowError>>>,
    metadata: HashMap<String, serde_json::Value>,
    _phantom: PhantomData<StartNode>,
}

impl<StartNode: Node + 'static> TypedWorkflowBuilder<StartNode> {
    /// Create a new typed workflow builder
    pub fn new(workflow_type: impl Into<String>) -> Self {
        Self {
            schema: WorkflowSchema::new(workflow_type.into(), TypeId::of::<StartNode>()),
            node_configs: HashMap::new(),
            validation_rules: Vec::new(),
            metadata: HashMap::new(),
            _phantom: PhantomData,
        }
    }

    /// Set workflow description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.schema.description = Some(description.into());
        self
    }

    /// Add metadata to the workflow
    pub fn metadata(mut self, key: impl Into<String>, value: impl serde::Serialize) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.metadata.insert(key.into(), json_value);
        }
        self
    }

    /// Add a node with type safety
    pub fn add_node<N: Node + 'static>(mut self, config: NodeConfig) -> NodeBuilder<StartNode, N> {
        self.node_configs.insert(TypeId::of::<N>(), config.clone());
        self.schema.nodes.push(config);
        
        NodeBuilder {
            workflow_builder: self,
            current_node: PhantomData,
        }
    }

    /// Add a validation rule
    pub fn validate<F>(mut self, rule: F) -> Self
    where
        F: Fn(&WorkflowSchema) -> Result<(), WorkflowError> + 'static,
    {
        self.validation_rules.push(Box::new(rule));
        self
    }

    /// Build the workflow with comprehensive validation
    pub fn build(self) -> Result<Workflow, WorkflowError> {
        // Run custom validation rules
        for rule in &self.validation_rules {
            rule(&self.schema)?;
        }

        // Validate workflow structure
        self.validate_structure()?;
        
        // Create workflow
        let mut workflow = Workflow::new(self.schema)?;
        
        // Apply metadata
        // Note: This would require extending Workflow to support metadata
        
        Ok(workflow)
    }

    /// Validate workflow structure
    fn validate_structure(&self) -> Result<(), WorkflowError> {
        // Ensure start node exists
        if !self.node_configs.contains_key(&self.schema.start) {
            return Err(WorkflowError::NodeNotFound {
                node_type: self.schema.start,
            });
        }

        // Validate all connections point to existing nodes
        for config in &self.schema.nodes {
            for &connection in &config.connections {
                if !self.node_configs.contains_key(&connection) {
                    return Err(WorkflowError::NodeNotFound {
                        node_type: connection,
                    });
                }
            }
        }

        // Check for unreachable nodes
        let reachable = self.find_reachable_nodes();
        let all_nodes: Vec<_> = self.node_configs.keys().copied().collect();
        let unreachable: Vec<_> = all_nodes
            .into_iter()
            .filter(|node| !reachable.contains(node))
            .collect();

        if !unreachable.is_empty() {
            return Err(WorkflowError::UnreachableNodes {
                nodes: unreachable.into_iter()
                    .map(|_| "Unknown".to_string()) // TypeId doesn't have a good string repr
                    .collect(),
            });
        }

        Ok(())
    }

    /// Find all reachable nodes from start
    fn find_reachable_nodes(&self) -> Vec<TypeId> {
        let mut reachable = Vec::new();
        let mut to_visit = vec![self.schema.start];

        while let Some(current) = to_visit.pop() {
            if reachable.contains(&current) {
                continue;
            }
            reachable.push(current);

            // Add connected nodes
            if let Some(config) = self.schema.nodes.iter().find(|c| c.node_type == current) {
                for &connection in &config.connections {
                    if !reachable.contains(&connection) {
                        to_visit.push(connection);
                    }
                }
            }
        }

        reachable
    }
}

/// Builder state for adding node connections
pub struct NodeBuilder<StartNode: Node + 'static, CurrentNode: Node + 'static> {
    workflow_builder: TypedWorkflowBuilder<StartNode>,
    current_node: PhantomData<CurrentNode>,
}

impl<StartNode: Node + 'static, CurrentNode: Node + 'static> NodeBuilder<StartNode, CurrentNode> {
    /// Connect current node to another node
    pub fn connect_to<NextNode: Node + 'static>(mut self) -> Self {
        // Find current node config and add connection
        if let Some(config) = self.workflow_builder.schema.nodes
            .iter_mut()
            .find(|c| c.node_type == TypeId::of::<CurrentNode>()) {
            config.connections.push(TypeId::of::<NextNode>());
        }
        self
    }

    /// Mark current node as a router
    pub fn as_router(mut self) -> Self {
        if let Some(config) = self.workflow_builder.schema.nodes
            .iter_mut()
            .find(|c| c.node_type == TypeId::of::<CurrentNode>()) {
            config.is_router = true;
        }
        self
    }

    /// Add a parallel node
    pub fn parallel_with<ParallelNode: Node + 'static>(mut self) -> Self {
        if let Some(config) = self.workflow_builder.schema.nodes
            .iter_mut()
            .find(|c| c.node_type == TypeId::of::<CurrentNode>()) {
            config.parallel_nodes.push(TypeId::of::<ParallelNode>());
        }
        self
    }

    /// Continue building the workflow
    pub fn then(self) -> TypedWorkflowBuilder<StartNode> {
        self.workflow_builder
    }

    /// Add another node
    pub fn add_node<N: Node + 'static>(self, config: NodeConfig) -> NodeBuilder<StartNode, N> {
        self.workflow_builder.add_node(config)
    }

    /// Build the workflow
    pub fn build(self) -> Result<Workflow, WorkflowError> {
        self.workflow_builder.build()
    }
}

/// Predefined workflow templates
pub struct WorkflowTemplates;

impl WorkflowTemplates {
    /// Create a linear processing workflow
    pub fn linear<N1, N2, N3>() -> TypedWorkflowBuilder<N1>
    where
        N1: Node + 'static,
        N2: Node + 'static,
        N3: Node + 'static,
    {
        TypedWorkflowBuilder::new("linear_workflow")
            .description("Linear processing workflow")
            .validate(|schema| {
                // Ensure linear flow (no branches)
                for config in &schema.nodes {
                    if config.connections.len() > 1 && !config.is_router {
                        return Err(WorkflowError::InvalidRouter {
                            node: "Unknown".to_string(),
                        });
                    }
                }
                Ok(())
            })
    }

    /// Create a parallel processing workflow
    pub fn parallel<Start, P1, P2, End>() -> TypedWorkflowBuilder<Start>
    where
        Start: Node + 'static,
        P1: Node + 'static,
        P2: Node + 'static,
        End: Node + 'static,
    {
        TypedWorkflowBuilder::new("parallel_workflow")
            .description("Parallel processing workflow")
    }

    /// Create a conditional workflow
    pub fn conditional<Start, Router, Branch1, Branch2, End>() -> TypedWorkflowBuilder<Start>
    where
        Start: Node + 'static,
        Router: Node + 'static,
        Branch1: Node + 'static,
        Branch2: Node + 'static,
        End: Node + 'static,
    {
        TypedWorkflowBuilder::new("conditional_workflow")
            .description("Conditional branching workflow")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::agent::BaseAgentNode;

    #[derive(Debug)]
    struct StartNode;
    impl Node for StartNode {
        fn process(&self, context: crate::task::TaskContext) -> Result<crate::task::TaskContext, WorkflowError> {
            Ok(context)
        }
    }

    #[derive(Debug)]
    struct ProcessNode;
    impl Node for ProcessNode {
        fn process(&self, context: crate::task::TaskContext) -> Result<crate::task::TaskContext, WorkflowError> {
            Ok(context)
        }
    }

    #[derive(Debug)]
    struct EndNode;
    impl Node for EndNode {
        fn process(&self, context: crate::task::TaskContext) -> Result<crate::task::TaskContext, WorkflowError> {
            Ok(context)
        }
    }

    #[test]
    fn test_typed_workflow_builder() {
        let workflow = TypedWorkflowBuilder::<StartNode>::new("test_workflow")
            .description("Test workflow")
            .metadata("version", "1.0.0")
            .metadata("author", "test")
            .add_node::<StartNode>(NodeConfig::new::<StartNode>())
                .connect_to::<ProcessNode>()
                .then()
            .add_node::<ProcessNode>(NodeConfig::new::<ProcessNode>())
                .connect_to::<EndNode>()
                .then()
            .add_node::<EndNode>(NodeConfig::new::<EndNode>())
                .then()
            .build();

        assert!(workflow.is_ok());
    }

    #[test]
    fn test_parallel_workflow() {
        let workflow = TypedWorkflowBuilder::<StartNode>::new("parallel_workflow")
            .add_node::<StartNode>(NodeConfig::new::<StartNode>())
                .parallel_with::<ProcessNode>()
                .parallel_with::<BaseAgentNode>()
                .then()
            .add_node::<ProcessNode>(NodeConfig::new::<ProcessNode>())
                .connect_to::<EndNode>()
                .then()
            .add_node::<BaseAgentNode>(NodeConfig::new::<BaseAgentNode>())
                .connect_to::<EndNode>()
                .then()
            .add_node::<EndNode>(NodeConfig::new::<EndNode>())
                .then()
            .build();

        assert!(workflow.is_ok());
    }

    #[test]
    fn test_unreachable_node_detection() {
        let result = TypedWorkflowBuilder::<StartNode>::new("broken_workflow")
            .add_node::<StartNode>(NodeConfig::new::<StartNode>())
                .connect_to::<ProcessNode>()
                .then()
            .add_node::<ProcessNode>(NodeConfig::new::<ProcessNode>())
                .then()
            .add_node::<EndNode>(NodeConfig::new::<EndNode>()) // Not connected!
                .then()
            .build();

        assert!(result.is_err());
        match result {
            Err(WorkflowError::UnreachableNodes { .. }) => {},
            _ => panic!("Expected UnreachableNodes error"),
        }
    }

    #[test]
    fn test_custom_validation() {
        let result = TypedWorkflowBuilder::<StartNode>::new("validated_workflow")
            .validate(|schema| {
                if schema.nodes.len() > 3 {
                    Err(WorkflowError::ConfigurationError(
                        "Workflow too complex".to_string()
                    ))
                } else {
                    Ok(())
                }
            })
            .add_node::<StartNode>(NodeConfig::new::<StartNode>())
            .add_node::<ProcessNode>(NodeConfig::new::<ProcessNode>())
            .add_node::<BaseAgentNode>(NodeConfig::new::<BaseAgentNode>())
            .add_node::<EndNode>(NodeConfig::new::<EndNode>()) // 4th node - should fail
            .build();

        assert!(result.is_err());
    }
}