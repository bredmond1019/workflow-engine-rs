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

    /// Add a Notion client node to the workflow (stub - full implementation in workflow-engine-mcp)
    pub fn add_notion_client(
        self,
        _server_url: Option<String>,
        _transport: Option<String>, // Using String instead of TransportType to avoid dependency
    ) -> Self {
        // Stub implementation - actual MCP client integration is in workflow-engine-mcp crate
        self
    }

    /// Add a HelpScout client node to the workflow (stub - full implementation in workflow-engine-mcp)
    pub fn add_helpscout_client(
        self,
        _server_url: Option<String>,
        _transport: Option<String>, // Using String instead of TransportType to avoid dependency
    ) -> Self {
        // Stub implementation - actual MCP client integration is in workflow-engine-mcp crate
        self
    }

    /// Add a Slack client node to the workflow (stub - full implementation in workflow-engine-mcp)
    pub fn add_slack_client(
        self,
        _server_url: Option<String>,
        _transport: Option<String>, // Using String instead of TransportType to avoid dependency
    ) -> Self {
        // Stub implementation - actual MCP client integration is in workflow-engine-mcp crate
        self
    }

    /// Add an external MCP client node with custom configuration (stub)
    /// Full implementation available in workflow-engine-mcp crate
    pub fn add_external_mcp_client(self, _service_name: String) -> Self {
        // Stub implementation - actual MCP client integration is in workflow-engine-mcp crate
        self
    }

    /// Create a multi-service workflow with all three external MCP clients
    pub fn add_all_mcp_clients(self) -> Self {
        self.add_notion_client(None, None)
            .add_helpscout_client(None, None)
            .add_slack_client(None, None)
    }

    /// Add external MCP clients for customer support workflow
    pub fn add_customer_support_clients(self) -> Self {
        self.add_helpscout_client(None, None)
            .add_slack_client(None, None)
    }

    /// Add external MCP clients for content management workflow
    pub fn add_content_management_clients(self) -> Self {
        self.add_notion_client(None, None)
            .add_slack_client(None, None)
    }

    /// Add external MCP clients from configuration (stub)
    /// Full implementation available in workflow-engine-mcp crate
    pub fn add_mcp_clients_from_config(self, _config_json: String) -> Self {
        // Stub implementation - actual MCP client integration is in workflow-engine-mcp crate
        self
    }

    /// Add external MCP clients from environment variables
    pub fn add_mcp_clients_from_env(self) -> Self {
        // use crate::nodes::external_config::EnvConfigLoader;
        // let config = EnvConfigLoader::load();
        // self.add_mcp_clients_from_config(config)
        self  // Return self unchanged for now
    }

    /// Create a workflow with external MCP clients using custom configuration builder (stub)
    /// Full implementation available in workflow-engine-mcp crate
    pub fn add_mcp_clients_with_builder(self, _config_json: String) -> Self {
        // Stub implementation - actual MCP client integration is in workflow-engine-mcp crate
        self
    }

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
