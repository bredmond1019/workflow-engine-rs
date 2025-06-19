// =============================================================================
// Workflow Builder - Ergonomic API for defining workflows
// =============================================================================

use std::any::TypeId;

use crate::{
    error::WorkflowError,
    /*
    mcp::{
        clients::{
            helpscout::HelpscoutClientNode, 
            notion::NotionClientNode,
            slack::SlackClientNode
        },
        transport::TransportType,
    },
    */
    nodes::{
        Node,
        config::NodeConfig,
        /*
        external_config::{ExternalMCPServerConfig, ExternalConfigBuilder},
        external_mcp_client::{ExternalMCPClientNode, ExternalMCPConfig},
        */
    },
    workflow::{Workflow, schema::WorkflowSchema},
};

pub struct WorkflowBuilder {
    schema: WorkflowSchema,
}

impl WorkflowBuilder {
    pub fn new<T: Node + 'static>(workflow_type: String) -> Self {
        Self {
            schema: WorkflowSchema::new(workflow_type, TypeId::of::<T>()),
        }
    }

    pub fn description(mut self, description: String) -> Self {
        self.schema.description = Some(description);
        self
    }

    pub fn add_node(mut self, config: NodeConfig) -> Self {
        self.schema.nodes.push(config);
        self
    }

    // MCP client methods removed - use workflow-engine-mcp crate directly for MCP integration










    pub fn build(self) -> Result<Workflow, WorkflowError> {
        Workflow::new(self.schema)
    }
}

/// Builder extensions for creating common workflow patterns
impl WorkflowBuilder {
    /// Create a customer support workflow template (stub)
    /// Full implementation with agent nodes available in workflow-engine-nodes crate
    pub fn customer_support_workflow() -> Self {
        WorkflowBuilder::new::<crate::nodes::agent::BaseAgentNode>(
            "customer_support_workflow".to_string(),
        )
        .description(
            "Customer support workflow template - add specific agent nodes for full functionality"
                .to_string(),
        )
    }

    /// Create a content management workflow template (stub)
    /// Full implementation with agent nodes available in workflow-engine-nodes crate
    pub fn content_management_workflow() -> Self {
        WorkflowBuilder::new::<crate::nodes::agent::BaseAgentNode>(
            "content_management_workflow".to_string(),
        )
        .description(
            "Content management workflow template - add specific agent nodes for full functionality"
                .to_string(),
        )
    }

    /// Create a comprehensive workflow template (stub)
    /// Full implementation with agent nodes available in workflow-engine-nodes crate
    pub fn comprehensive_workflow() -> Self {
        WorkflowBuilder::new::<crate::nodes::agent::BaseAgentNode>(
            "comprehensive_workflow".to_string(),
        )
        .description(
            "Comprehensive workflow template - add specific nodes for full functionality"
                .to_string(),
        )
    }
}
