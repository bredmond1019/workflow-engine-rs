// =============================================================================
// Workflow Builder - Ergonomic API for defining workflows
// =============================================================================

use std::any::TypeId;

use crate::core::{
    error::WorkflowError,
    mcp::{
        clients::{
            helpscout::HelpscoutClientNode, 
            notion::NotionClientNode,
            slack::SlackClientNode
        },
        transport::TransportType,
    },
    nodes::{
        Node,
        config::NodeConfig,
        external_config::{ExternalMCPServerConfig, ExternalConfigBuilder},
        external_mcp_client::{ExternalMCPClientNode, ExternalMCPConfig},
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

    /// Add a Notion client node to the workflow
    pub fn add_notion_client(
        mut self,
        server_url: Option<String>,
        transport: Option<TransportType>,
    ) -> Self {
        let url = server_url.unwrap_or_else(|| {
            std::env::var("NOTION_MCP_URL").unwrap_or_else(|_| "http://localhost:8002".to_string())
        });
        let transport_type = transport.unwrap_or(TransportType::Http {
            base_url: url.clone(),
            pool_config: crate::core::mcp::transport::HttpPoolConfig::default(),
        });

        // let config = NodeConfig::new::<NotionClientNode>()
        //     .with_description(format!("Notion MCP client connected to {}", url));

        // self.schema.nodes.push(config);
        self
    }

    /// Add a HelpScout client node to the workflow
    pub fn add_helpscout_client(
        mut self,
        server_url: Option<String>,
        transport: Option<TransportType>,
    ) -> Self {
        let url = server_url.unwrap_or_else(|| {
            std::env::var("HELPSCOUT_MCP_URL")
                .unwrap_or_else(|_| "http://localhost:8001".to_string())
        });
        let transport_type = transport.unwrap_or(TransportType::Http {
            base_url: url.clone(),
            pool_config: crate::core::mcp::transport::HttpPoolConfig::default(),
        });

        let config = NodeConfig::new::<HelpscoutClientNode>()
            .with_description(format!("HelpScout MCP client connected to {}", url));

        self.schema.nodes.push(config);
        self
    }

    /// Add a Slack client node to the workflow
    pub fn add_slack_client(
        mut self,
        server_url: Option<String>,
        transport: Option<TransportType>,
    ) -> Self {
        let url = server_url.unwrap_or_else(|| {
            std::env::var("SLACK_MCP_URL").unwrap_or_else(|_| "http://localhost:8003".to_string())
        });
        let transport_type = transport.unwrap_or(TransportType::Http {
            base_url: url.clone(),
            pool_config: crate::core::mcp::transport::HttpPoolConfig::default(),
        });

        let config = NodeConfig::new::<SlackClientNode>()
            .with_description(format!("Slack MCP client connected to {}", url));

        self.schema.nodes.push(config);
        self
    }

    /// Add an external MCP client node with custom configuration
    pub fn add_external_mcp_client<T: ExternalMCPClientNode + Node + 'static>(
        mut self,
        node_type: std::marker::PhantomData<T>,
        config: ExternalMCPConfig,
    ) -> Self {
        let node_config = NodeConfig::new::<T>()
            .with_description(format!("External MCP client for {}", config.service_name));

        self.schema.nodes.push(node_config);
        self
    }

    /// Create a multi-service workflow with all three external MCP clients
    pub fn add_all_mcp_clients(mut self) -> Self {
        self.add_notion_client(None, None)
            .add_helpscout_client(None, None)
            .add_slack_client(None, None)
    }

    /// Add external MCP clients for customer support workflow
    pub fn add_customer_support_clients(mut self) -> Self {
        self.add_helpscout_client(None, None)
            .add_slack_client(None, None)
    }

    /// Add external MCP clients for content management workflow
    pub fn add_content_management_clients(mut self) -> Self {
        self.add_notion_client(None, None)
            .add_slack_client(None, None)
    }

    /// Add external MCP clients from configuration
    pub fn add_mcp_clients_from_config(mut self, config: ExternalMCPServerConfig) -> Self {
        // Add Notion client if configured and enabled
        if let Some(notion_config) = config.notion {
            if notion_config.enabled {
                // NotionClientNode uses a different configuration approach
                // For now, we'll add the configuration to the schema
                // The actual client creation would happen at runtime
                let node_config = NodeConfig::new::<NotionClientNode>()
                    .with_description(format!("Notion MCP client connected to {}", notion_config.url));
                self.schema.nodes.push(node_config);
            }
        }

        // Add HelpScout client if configured and enabled
        if let Some(helpscout_config) = config.helpscout {
            if helpscout_config.enabled {
                let helpscout_client = HelpscoutClientNode::with_http_transport(
                    helpscout_config.url,
                    helpscout_config.api_key
                );
                
                let node_config = NodeConfig::new::<HelpscoutClientNode>()
                    .with_description("HelpScout MCP client from configuration".to_string());
                self.schema.nodes.push(node_config);
            }
        }

        // Add Slack client if configured and enabled
        if let Some(slack_config) = config.slack {
            if slack_config.enabled {
                let slack_client = SlackClientNode::with_http_transport(
                    slack_config.url,
                    slack_config.bot_token,
                    slack_config.user_token
                );
                
                let node_config = NodeConfig::new::<SlackClientNode>()
                    .with_description("Slack MCP client from configuration".to_string());
                self.schema.nodes.push(node_config);
            }
        }

        self
    }

    /// Add external MCP clients from environment variables
    pub fn add_mcp_clients_from_env(self) -> Self {
        use crate::core::nodes::external_config::EnvConfigLoader;
        let config = EnvConfigLoader::load();
        self.add_mcp_clients_from_config(config)
    }

    /// Create a workflow with external MCP clients using custom configuration builder
    pub fn add_mcp_clients_with_builder<F>(self, config_builder: F) -> Self 
    where
        F: FnOnce(ExternalConfigBuilder) -> ExternalConfigBuilder,
    {
        let builder = ExternalConfigBuilder::new();
        let config = config_builder(builder).build();
        self.add_mcp_clients_from_config(config)
    }

    pub fn build(self) -> Result<Workflow, WorkflowError> {
        Workflow::new(self.schema)
    }
}

/// Builder extensions for creating common workflow patterns with external MCP clients
impl WorkflowBuilder {
    /// Create a customer support workflow with external MCP integrations
    pub fn customer_support_workflow() -> Self {
        Self::new::<crate::core::ai_agents::openai::OpenAIAgentNode>(
            "customer_support_with_external_mcp".to_string(),
        )
        .description(
            "Customer support workflow with external MCP integrations for HelpScout and Slack"
                .to_string(),
        )
        .add_customer_support_clients()
    }

    /// Create a content management workflow with external MCP integrations
    pub fn content_management_workflow() -> Self {
        Self::new::<crate::core::ai_agents::openai::OpenAIAgentNode>(
            "content_management_with_external_mcp".to_string(),
        )
        .description(
            "Content management workflow with external MCP integrations for Notion and Slack"
                .to_string(),
        )
        .add_content_management_clients()
    }

    /// Create a comprehensive workflow with all external MCP integrations
    pub fn comprehensive_external_mcp_workflow() -> Self {
        Self::new::<crate::core::ai_agents::openai::OpenAIAgentNode>(
            "comprehensive_external_mcp".to_string(),
        )
        .description(
            "Comprehensive workflow with all external MCP integrations (Notion, HelpScout, Slack)"
                .to_string(),
        )
        .add_all_mcp_clients()
    }
}
