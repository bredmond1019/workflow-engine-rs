use std::any::TypeId;

use crate::nodes::config::NodeConfig;

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
}
