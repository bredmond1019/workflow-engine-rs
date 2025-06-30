use std::any::TypeId;
use std::collections::HashSet;

use crate::{nodes::config::NodeConfig, error::WorkflowError};

// Configuration limits for validation - adjustable for different use cases
const MAX_WORKFLOW_TYPE_LENGTH: usize = 255;   // Reasonable length for workflow names
const MAX_WORKFLOW_NODES: usize = 1000;        // Prevent memory exhaustion
const MAX_WORKFLOW_DEPTH: usize = 100;         // Prevent stack overflow
const MAX_PARALLEL_NODES: usize = 50;          // Reasonable concurrency limit

#[derive(Debug)]
pub struct WorkflowSchema {
    pub workflow_type: String,
    pub description: Option<String>,
    pub start: TypeId,
    pub nodes: Vec<NodeConfig>,
}

impl WorkflowSchema {
    pub fn new(workflow_type: String, start: TypeId) -> Self {
        Self {
            workflow_type,
            description: None,
            start,
            nodes: Vec::new(),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_nodes(mut self, nodes: Vec<NodeConfig>) -> Self {
        self.nodes = nodes;
        self
    }

    /// Validate the workflow schema before creating a workflow
    pub fn validate(&self) -> Result<(), WorkflowError> {
        self.validate_workflow_type()?;
        self.validate_nodes_list()?;
        self.validate_start_node()?;
        self.validate_node_configurations()?;
        self.validate_workflow_limits()?;
        Ok(())
    }

    fn validate_workflow_type(&self) -> Result<(), WorkflowError> {
        // Check if workflow type is empty or only whitespace
        let trimmed = self.workflow_type.trim();
        if trimmed.is_empty() {
            return Err(WorkflowError::configuration_error(
                "Workflow type cannot be empty or only whitespace",
                "workflow_type",
                "workflow_schema",
                "non-empty string",
                Some(self.workflow_type.clone()),
            ));
        }

        // Check length limits
        if self.workflow_type.len() > MAX_WORKFLOW_TYPE_LENGTH {
            return Err(WorkflowError::configuration_error(
                format!("Workflow type exceeds maximum length of {}", MAX_WORKFLOW_TYPE_LENGTH),
                "workflow_type",
                "workflow_schema", 
                format!("string with length <= {}", MAX_WORKFLOW_TYPE_LENGTH),
                Some(format!("length: {}", self.workflow_type.len())),
            ));
        }

        // Check for invalid characters (control characters, null bytes)
        if self.workflow_type.chars().any(|c| c.is_control()) {
            return Err(WorkflowError::configuration_error(
                "Workflow type contains invalid control characters",
                "workflow_type",
                "workflow_schema",
                "string without control characters",
                Some("contains_control_chars".to_string()),
            ));
        }

        Ok(())
    }

    fn validate_nodes_list(&self) -> Result<(), WorkflowError> {
        // Check if nodes list is empty
        if self.nodes.is_empty() {
            return Err(WorkflowError::configuration_error(
                "Workflow must contain at least one node",
                "nodes",
                "workflow_schema",
                "non-empty list of nodes",
                Some("empty_list".to_string()),
            ));
        }

        // Check for excessive nodes count
        if self.nodes.len() > MAX_WORKFLOW_NODES {
            return Err(WorkflowError::configuration_error(
                format!("Workflow exceeds maximum node count of {}", MAX_WORKFLOW_NODES),
                "nodes",
                "workflow_schema",
                format!("node count <= {}", MAX_WORKFLOW_NODES),
                Some(self.nodes.len().to_string()),
            ));
        }

        Ok(())
    }

    fn validate_start_node(&self) -> Result<(), WorkflowError> {
        // Check if start node exists in the nodes list
        let has_start_node = self.nodes.iter().any(|node| node.node_type == self.start);
        if !has_start_node {
            return Err(WorkflowError::configuration_error(
                "Start node is not present in the nodes list",
                "start",
                "workflow_schema",
                "node type that exists in nodes list",
                Some(format!("{:?}", self.start)),
            ));
        }

        Ok(())
    }

    fn validate_node_configurations(&self) -> Result<(), WorkflowError> {
        // Validate each node configuration
        for node in &self.nodes {
            node.validate()?;
        }

        // Check for excessive parallel nodes in any single node
        for node in &self.nodes {
            if node.parallel_nodes.len() > MAX_PARALLEL_NODES {
                return Err(WorkflowError::configuration_error(
                    format!("Node has too many parallel nodes (max: {})", MAX_PARALLEL_NODES),
                    "parallel_nodes",
                    "node_configuration",
                    format!("parallel nodes count <= {}", MAX_PARALLEL_NODES),
                    Some(node.parallel_nodes.len().to_string()),
                ));
            }
        }

        Ok(())
    }

    fn validate_workflow_limits(&self) -> Result<(), WorkflowError> {
        // Check for excessive workflow depth (could cause stack overflow)
        let depth = self.calculate_max_depth();
        if depth > MAX_WORKFLOW_DEPTH {
            return Err(WorkflowError::configuration_error(
                format!("Workflow depth exceeds maximum of {}", MAX_WORKFLOW_DEPTH),
                "workflow_structure", 
                "workflow_schema",
                format!("depth <= {}", MAX_WORKFLOW_DEPTH),
                Some(depth.to_string()),
            ));
        }

        Ok(())
    }

    fn calculate_max_depth(&self) -> usize {
        let mut visited = HashSet::new();
        self.dfs_depth(self.start, &mut visited, 0)
    }

    fn dfs_depth(&self, node_type: TypeId, visited: &mut HashSet<TypeId>, current_depth: usize) -> usize {
        if visited.contains(&node_type) {
            return current_depth; // Avoid infinite recursion on cycles
        }

        visited.insert(node_type);

        let node_config = self.nodes.iter().find(|n| n.node_type == node_type);
        if let Some(config) = node_config {
            let mut max_depth = current_depth;
            for &connected_node in &config.connections {
                let depth = self.dfs_depth(connected_node, visited, current_depth + 1);
                max_depth = max_depth.max(depth);
            }
            visited.remove(&node_type);
            max_depth
        } else {
            visited.remove(&node_type);
            current_depth
        }
    }
}
