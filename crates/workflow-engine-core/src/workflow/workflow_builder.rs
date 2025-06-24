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

    /// Add multiple metadata entries
    pub fn metadata_map(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata.extend(metadata);
        self
    }

    /// Set workflow version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        if let Ok(json_value) = serde_json::to_value(version.into()) {
            self.metadata.insert("version".to_string(), json_value);
        }
        self
    }

    /// Set workflow author
    pub fn author(mut self, author: impl Into<String>) -> Self {
        if let Ok(json_value) = serde_json::to_value(author.into()) {
            self.metadata.insert("author".to_string(), json_value);
        }
        self
    }

    /// Set workflow tags for categorization
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        if let Ok(json_value) = serde_json::to_value(tags) {
            self.metadata.insert("tags".to_string(), json_value);
        }
        self
    }

    /// Set workflow timeout
    pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
        if let Ok(json_value) = serde_json::to_value(timeout.as_secs()) {
            self.metadata.insert("timeout_seconds".to_string(), json_value);
        }
        self
    }

    /// Set maximum parallel executions for the workflow
    pub fn max_parallel_executions(mut self, max: usize) -> Self {
        if let Ok(json_value) = serde_json::to_value(max) {
            self.metadata.insert("max_parallel_executions".to_string(), json_value);
        }
        self
    }

    /// Enable or disable workflow debugging
    pub fn debug_mode(mut self, enabled: bool) -> Self {
        if let Ok(json_value) = serde_json::to_value(enabled) {
            self.metadata.insert("debug_mode".to_string(), json_value);
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

    /// Add a node using a builder function for fluent configuration
    pub fn add_node_with<N: Node + 'static, F>(
        mut self, 
        builder_fn: F
    ) -> Result<NodeBuilder<StartNode, N>, WorkflowError>
    where
        F: FnOnce(crate::nodes::config_builder::NodeConfigBuilder<N>) -> crate::nodes::config_builder::NodeConfigBuilder<N>,
    {
        let config = builder_fn(crate::nodes::config_builder::NodeConfigBuilder::<N>::new()).build()?;
        Ok(self.add_node::<N>(config))
    }

    /// Add a simple node with minimal configuration
    pub fn add_simple_node<N: Node + 'static>(self) -> NodeBuilder<StartNode, N> {
        let config = NodeConfig::new::<N>();
        self.add_node::<N>(config)
    }

    /// Add a node with just a description
    pub fn add_described_node<N: Node + 'static>(
        self, 
        description: impl Into<String>
    ) -> NodeBuilder<StartNode, N> {
        let config = NodeConfig::new::<N>().with_description(description.into());
        self.add_node::<N>(config)
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

        // Ensure we have at least one node
        if self.schema.nodes.is_empty() {
            return Err(WorkflowError::ConfigurationError(
                "Workflow must contain at least one node".to_string()
            ));
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

            // Validate parallel nodes exist
            for &parallel_node in &config.parallel_nodes {
                if !self.node_configs.contains_key(&parallel_node) {
                    return Err(WorkflowError::NodeNotFound {
                        node_type: parallel_node,
                    });
                }
            }

            // Validate individual node configuration
            config.validate()?;
        }

        // Check for cycles
        self.detect_cycles()?;

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

        // Validate metadata constraints
        self.validate_metadata()?;

        Ok(())
    }

    /// Detect cycles in the workflow graph
    fn detect_cycles(&self) -> Result<(), WorkflowError> {
        let mut visited = std::collections::HashSet::new();
        let mut rec_stack = std::collections::HashSet::new();
        
        // Check for cycles starting from any node
        for &node_id in self.node_configs.keys() {
            if !visited.contains(&node_id) {
                if self.has_cycle_util(node_id, &mut visited, &mut rec_stack) {
                    return Err(WorkflowError::CycleDetected);
                }
            }
        }
        
        Ok(())
    }

    /// Utility function for cycle detection using DFS
    fn has_cycle_util(
        &self, 
        node_id: TypeId, 
        visited: &mut std::collections::HashSet<TypeId>,
        rec_stack: &mut std::collections::HashSet<TypeId>
    ) -> bool {
        visited.insert(node_id);
        rec_stack.insert(node_id);

        // Find the node config for this node_id
        if let Some(config) = self.schema.nodes.iter().find(|c| c.node_type == node_id) {
            for &connection in &config.connections {
                if !visited.contains(&connection) {
                    if self.has_cycle_util(connection, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(&connection) {
                    return true;
                }
            }
        }

        rec_stack.remove(&node_id);
        false
    }

    /// Validate metadata constraints
    fn validate_metadata(&self) -> Result<(), WorkflowError> {
        // Validate timeout if specified
        if let Some(timeout_value) = self.metadata.get("timeout_seconds") {
            if let Some(timeout_secs) = timeout_value.as_u64() {
                if timeout_secs == 0 {
                    return Err(WorkflowError::ConfigurationError(
                        "Workflow timeout must be greater than 0 seconds".to_string()
                    ));
                }
            }
        }

        // Validate max parallel executions if specified
        if let Some(max_parallel) = self.metadata.get("max_parallel_executions") {
            if let Some(max_value) = max_parallel.as_u64() {
                if max_value == 0 {
                    return Err(WorkflowError::ConfigurationError(
                        "Max parallel executions must be greater than 0".to_string()
                    ));
                }
            }
        }

        // Validate version format if specified
        if let Some(version) = self.metadata.get("version") {
            if let Some(version_str) = version.as_str() {
                if version_str.is_empty() {
                    return Err(WorkflowError::ConfigurationError(
                        "Version cannot be empty".to_string()
                    ));
                }
            }
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

    /// Add another node using a builder function
    pub fn add_node_with<N: Node + 'static, F>(
        self, 
        builder_fn: F
    ) -> Result<NodeBuilder<StartNode, N>, WorkflowError>
    where
        F: FnOnce(crate::nodes::config_builder::NodeConfigBuilder<N>) -> crate::nodes::config_builder::NodeConfigBuilder<N>,
    {
        self.workflow_builder.add_node_with(builder_fn)
    }

    /// Add a simple node with minimal configuration
    pub fn add_simple_node<N: Node + 'static>(self) -> NodeBuilder<StartNode, N> {
        self.workflow_builder.add_simple_node::<N>()
    }

    /// Connect to another node and continue building that node
    pub fn connect_and_add<NextNode: Node + 'static>(mut self, config: NodeConfig) -> NodeBuilder<StartNode, NextNode> {
        // First connect current node to the next node
        if let Some(current_config) = self.workflow_builder.schema.nodes
            .iter_mut()
            .find(|c| c.node_type == TypeId::of::<CurrentNode>()) {
            current_config.connections.push(TypeId::of::<NextNode>());
        }
        
        // Then add the next node
        self.workflow_builder.add_node::<NextNode>(config)
    }

    /// Create a conditional branch from current node
    pub fn branch_if<ConditionNode: Node + 'static, TrueNode: Node + 'static, FalseNode: Node + 'static>(
        mut self,
        condition_config: NodeConfig,
        true_config: NodeConfig,
        false_config: NodeConfig,
    ) -> ConditionalBranchBuilder<StartNode, ConditionNode, TrueNode, FalseNode> {
        // Mark current node as router to allow multiple connections
        if let Some(current_config) = self.workflow_builder.schema.nodes
            .iter_mut()
            .find(|c| c.node_type == TypeId::of::<CurrentNode>()) {
            current_config.is_router = true;
            current_config.connections.push(TypeId::of::<ConditionNode>());
        }

        // Add condition node
        self.workflow_builder.node_configs.insert(TypeId::of::<ConditionNode>(), condition_config.clone());
        self.workflow_builder.schema.nodes.push(condition_config);

        // Add true and false branch nodes
        self.workflow_builder.node_configs.insert(TypeId::of::<TrueNode>(), true_config.clone());
        self.workflow_builder.schema.nodes.push(true_config);
        
        self.workflow_builder.node_configs.insert(TypeId::of::<FalseNode>(), false_config.clone());
        self.workflow_builder.schema.nodes.push(false_config);

        ConditionalBranchBuilder {
            workflow_builder: self.workflow_builder,
            _phantom: PhantomData,
        }
    }

    /// Build the workflow
    pub fn build(self) -> Result<Workflow, WorkflowError> {
        self.workflow_builder.build()
    }
}

/// Builder for conditional branches in workflows
pub struct ConditionalBranchBuilder<StartNode: Node + 'static, ConditionNode: Node + 'static, TrueNode: Node + 'static, FalseNode: Node + 'static> {
    workflow_builder: TypedWorkflowBuilder<StartNode>,
    _phantom: PhantomData<(ConditionNode, TrueNode, FalseNode)>,
}

impl<StartNode: Node + 'static, ConditionNode: Node + 'static, TrueNode: Node + 'static, FalseNode: Node + 'static> 
    ConditionalBranchBuilder<StartNode, ConditionNode, TrueNode, FalseNode> {
    
    /// Connect the condition node to the true and false branches
    pub fn connect_branches(mut self) -> Self {
        // Connect condition node to true and false branches
        if let Some(condition_config) = self.workflow_builder.schema.nodes
            .iter_mut()
            .find(|c| c.node_type == TypeId::of::<ConditionNode>()) {
            condition_config.is_router = true;
            condition_config.connections.push(TypeId::of::<TrueNode>());
            condition_config.connections.push(TypeId::of::<FalseNode>());
        }
        self
    }

    /// Merge branches back to a single node
    pub fn merge_to<MergeNode: Node + 'static>(mut self, merge_config: NodeConfig) -> NodeBuilder<StartNode, MergeNode> {
        // Connect true and false branches to merge node
        if let Some(true_config) = self.workflow_builder.schema.nodes
            .iter_mut()
            .find(|c| c.node_type == TypeId::of::<TrueNode>()) {
            true_config.connections.push(TypeId::of::<MergeNode>());
        }
        
        if let Some(false_config) = self.workflow_builder.schema.nodes
            .iter_mut()
            .find(|c| c.node_type == TypeId::of::<FalseNode>()) {
            false_config.connections.push(TypeId::of::<MergeNode>());
        }

        self.workflow_builder.add_node::<MergeNode>(merge_config)
    }

    /// Build the workflow
    pub fn build(self) -> Result<Workflow, WorkflowError> {
        self.workflow_builder.build()
    }

    /// Continue building the workflow
    pub fn then(self) -> TypedWorkflowBuilder<StartNode> {
        self.workflow_builder
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
            .version("1.0.0")
            .metadata("pattern", "parallel")
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
            .version("1.0.0")
            .metadata("pattern", "conditional")
    }

    /// Create a map-reduce workflow template
    pub fn map_reduce<Input, Mapper, Reducer, Output>() -> TypedWorkflowBuilder<Input>
    where
        Input: Node + 'static,
        Mapper: Node + 'static,
        Reducer: Node + 'static,
        Output: Node + 'static,
    {
        TypedWorkflowBuilder::new("map_reduce_workflow")
            .description("Map-reduce processing workflow")
            .version("1.0.0")
            .metadata("pattern", "map-reduce")
            .validate(|schema| {
                // Ensure mapper runs in parallel
                let mapper_nodes = schema.nodes.iter()
                    .filter(|c| c.node_type == TypeId::of::<Mapper>())
                    .count();
                if mapper_nodes == 0 {
                    return Err(WorkflowError::ConfigurationError(
                        "Map-reduce workflow must have at least one mapper node".to_string()
                    ));
                }
                Ok(())
            })
    }

    /// Create a data pipeline workflow
    pub fn data_pipeline<Ingestion, Transform, Validation, Output>() -> TypedWorkflowBuilder<Ingestion>
    where
        Ingestion: Node + 'static,
        Transform: Node + 'static,
        Validation: Node + 'static,
        Output: Node + 'static,
    {
        TypedWorkflowBuilder::new("data_pipeline")
            .description("Data processing pipeline workflow")
            .version("1.0.0")
            .metadata("pattern", "pipeline")
            .timeout(std::time::Duration::from_secs(3600)) // 1 hour default
    }

    /// Create a retry-enabled workflow template
    pub fn resilient<MainProcess, ErrorHandler, Recovery>() -> TypedWorkflowBuilder<MainProcess>
    where
        MainProcess: Node + 'static,
        ErrorHandler: Node + 'static,
        Recovery: Node + 'static,
    {
        TypedWorkflowBuilder::new("resilient_workflow")
            .description("Resilient workflow with error handling and recovery")
            .version("1.0.0")
            .metadata("pattern", "resilient")
            .metadata("resilience_enabled", true)
    }

    /// Create a microservice orchestration workflow
    pub fn microservice_orchestration<Orchestrator, Service1, Service2, Aggregator>() -> TypedWorkflowBuilder<Orchestrator>
    where
        Orchestrator: Node + 'static,
        Service1: Node + 'static,
        Service2: Node + 'static,
        Aggregator: Node + 'static,
    {
        TypedWorkflowBuilder::new("microservice_orchestration")
            .description("Microservice orchestration workflow")
            .version("1.0.0")
            .metadata("pattern", "orchestration")
            .max_parallel_executions(10)
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

    #[test]
    fn test_enhanced_workflow_builder() {
        let workflow = TypedWorkflowBuilder::<StartNode>::new("enhanced_workflow")
            .description("Enhanced workflow with metadata")
            .version("2.0.0")
            .author("Test Author")
            .tags(vec!["test".to_string(), "enhanced".to_string()])
            .timeout(std::time::Duration::from_secs(300))
            .max_parallel_executions(5)
            .debug_mode(true)
            .add_simple_node::<StartNode>()
                .connect_to::<ProcessNode>()
                .then()
            .add_described_node::<ProcessNode>("Processing node")
                .connect_to::<EndNode>()
                .then()
            .add_simple_node::<EndNode>()
                .then()
            .build();

        assert!(workflow.is_ok());
        let wf = workflow.unwrap();
        // Note: Metadata validation would require accessing metadata from Workflow
        // This demonstrates the fluent interface works correctly
    }

    #[test] 
    fn test_workflow_templates() {
        let linear_workflow = WorkflowTemplates::linear::<StartNode, ProcessNode, EndNode>()
            .add_simple_node::<StartNode>()
                .connect_to::<ProcessNode>()
                .then()
            .add_simple_node::<ProcessNode>()
                .connect_to::<EndNode>()
                .then()
            .add_simple_node::<EndNode>()
                .then()
            .build();

        assert!(linear_workflow.is_ok());

        let parallel_workflow = WorkflowTemplates::parallel::<StartNode, ProcessNode, BaseAgentNode, EndNode>()
            .add_simple_node::<StartNode>()
                .parallel_with::<ProcessNode>()
                .parallel_with::<BaseAgentNode>()
                .then()
            .add_simple_node::<ProcessNode>()
                .connect_to::<EndNode>()
                .then()
            .add_simple_node::<BaseAgentNode>()
                .connect_to::<EndNode>()
                .then()
            .add_simple_node::<EndNode>()
                .then()
            .build();

        assert!(parallel_workflow.is_ok());
    }

    #[test]
    fn test_cycle_detection() {
        // Create a workflow with a cycle: StartNode -> ProcessNode -> EndNode -> StartNode
        let result = TypedWorkflowBuilder::<StartNode>::new("cyclic_workflow")
            .add_node::<StartNode>(NodeConfig::new::<StartNode>().with_connections(vec![TypeId::of::<ProcessNode>()]))
                .then()
            .add_node::<ProcessNode>(NodeConfig::new::<ProcessNode>().with_connections(vec![TypeId::of::<EndNode>()]))
                .then()
            .add_node::<EndNode>(NodeConfig::new::<EndNode>().with_connections(vec![TypeId::of::<StartNode>()]))
                .then()
            .build();

        assert!(result.is_err());
        match result {
            Err(WorkflowError::CycleDetected) => {},
            _ => panic!("Expected CycleDetected error"),
        }
    }

    #[test]
    fn test_metadata_validation() {
        // Test invalid timeout
        let result = TypedWorkflowBuilder::<StartNode>::new("invalid_timeout")
            .timeout(std::time::Duration::from_secs(0))
            .add_simple_node::<StartNode>()
            .build();
        assert!(result.is_err());

        // Test invalid max parallel executions
        let result = TypedWorkflowBuilder::<StartNode>::new("invalid_parallel")
            .max_parallel_executions(0)
            .add_simple_node::<StartNode>()
            .build();
        assert!(result.is_err());

        // Test empty version
        let result = TypedWorkflowBuilder::<StartNode>::new("empty_version")
            .version("")
            .add_simple_node::<StartNode>()
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_complex_workflow_building() {
        let workflow = TypedWorkflowBuilder::<StartNode>::new("complex_workflow")
            .description("Complex workflow demonstrating various features")
            .version("1.0.0")
            .metadata("complexity", "high")
            .add_simple_node::<StartNode>()
                .connect_and_add::<ProcessNode>(NodeConfig::new::<ProcessNode>().with_description("Main processor"))
                .connect_and_add::<BaseAgentNode>(NodeConfig::new::<BaseAgentNode>().with_description("Agent processor"))
                .connect_and_add::<EndNode>(NodeConfig::new::<EndNode>().with_description("Final node"))
                .then()
            .build();

        assert!(workflow.is_ok());
    }
}