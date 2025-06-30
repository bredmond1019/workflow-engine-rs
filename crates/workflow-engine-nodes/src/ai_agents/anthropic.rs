use workflow_engine_core::{
    error::WorkflowError,
    nodes::{
        agent::{AgentConfig, BaseAgentNode},
    },
};

/// Anthropic-specific agent node implementation
/// 
/// This is a convenience wrapper around BaseAgentNode for Anthropic-specific use cases.
/// For most purposes, you can use BaseAgentNode directly with ModelProvider::Anthropic.
#[derive(Debug)]
pub struct AnthropicAgentNode {
    base_node: BaseAgentNode,
}

impl AnthropicAgentNode {
    pub fn new(config: AgentConfig) -> Result<Self, WorkflowError> {
        // BaseAgentNode::new now returns Result for validation
        Ok(Self {
            base_node: BaseAgentNode::new(config)?,
        })
    }

    pub fn with_mcp_client(mut self, mcp_client: Box<dyn std::any::Any + Send + Sync>) -> Self {
        self.base_node = self.base_node.with_mcp_client(mcp_client);
        self
    }

    pub fn set_mcp_client(&mut self, mcp_client: Box<dyn std::any::Any + Send + Sync>) {
        self.base_node.set_mcp_client(mcp_client);
    }

    pub fn has_mcp_client(&self) -> bool {
        self.base_node.has_mcp_client()
    }
}

// Delegate all trait implementations to the base node
impl workflow_engine_core::nodes::Node for AnthropicAgentNode {
    fn process(&self, task_context: workflow_engine_core::task::TaskContext) -> Result<workflow_engine_core::task::TaskContext, WorkflowError> {
        self.base_node.process(task_context)
    }
    
    fn node_name(&self) -> String {
        "AnthropicAgentNode".to_string()
    }
}

#[async_trait::async_trait]
impl workflow_engine_core::nodes::agent::AgentNode for AnthropicAgentNode {
    fn get_agent_config(&self) -> AgentConfig {
        self.base_node.get_agent_config()
    }

    async fn process_with_ai(
        &self,
        task_context: workflow_engine_core::task::TaskContext,
    ) -> Result<workflow_engine_core::task::TaskContext, WorkflowError> {
        self.base_node.process_with_ai(task_context).await
    }
}

#[cfg(test)]
mod tests {
    use workflow_engine_core::nodes::agent::{ModelProvider, AgentNode};

    use super::*;

    #[test]
    fn test_anthropic_agent_creation() {
        let config = AgentConfig {
            system_prompt: "You are Claude".to_string(),
            model_provider: ModelProvider::Anthropic,
            model_name: "claude-3-opus-20240229".to_string(),
            mcp_server_uri: None,
        };

        let agent = AnthropicAgentNode::new(config).unwrap();
        assert_eq!(agent.get_agent_config().model_provider, ModelProvider::Anthropic);
    }
}