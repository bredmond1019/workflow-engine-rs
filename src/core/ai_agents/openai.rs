use crate::core::{
    error::WorkflowError,
    mcp::clients::MCPClient,
    nodes::{
        agent::{AgentConfig, BaseAgentNode},
    },
};

/// OpenAI-specific agent node implementation
/// 
/// This is a convenience wrapper around BaseAgentNode for OpenAI-specific use cases.
/// For most purposes, you can use BaseAgentNode directly with ModelProvider::OpenAI.
#[derive(Debug)]
pub struct OpenAIAgentNode {
    base_node: BaseAgentNode,
}

impl OpenAIAgentNode {
    pub fn new(config: AgentConfig) -> Result<Self, WorkflowError> {
        // Validate that the config is for OpenAI
        if config.model_provider != crate::core::nodes::agent::ModelProvider::OpenAI {
            return Err(WorkflowError::ConfigurationError(
                "OpenAIAgentNode requires ModelProvider::OpenAI".to_string()
            ));
        }
        
        Ok(Self {
            base_node: BaseAgentNode::new(config),
        })
    }

    pub fn with_mcp_client(mut self, mcp_client: Box<dyn MCPClient>) -> Self {
        self.base_node = self.base_node.with_mcp_client(mcp_client);
        self
    }

    pub fn set_mcp_client(&mut self, mcp_client: Box<dyn MCPClient>) {
        self.base_node.set_mcp_client(mcp_client);
    }

    pub fn has_mcp_client(&self) -> bool {
        self.base_node.has_mcp_client()
    }
}

// Delegate all trait implementations to the base node
impl crate::core::nodes::Node for OpenAIAgentNode {
    fn process(&self, task_context: crate::core::task::TaskContext) -> Result<crate::core::task::TaskContext, WorkflowError> {
        self.base_node.process(task_context)
    }
    
    fn node_name(&self) -> String {
        "OpenAIAgentNode".to_string()
    }
}

#[async_trait::async_trait]
impl crate::core::nodes::agent::AgentNode for OpenAIAgentNode {
    fn get_agent_config(&self) -> AgentConfig {
        self.base_node.get_agent_config()
    }

    async fn process_with_ai(
        &self,
        task_context: crate::core::task::TaskContext,
    ) -> Result<crate::core::task::TaskContext, WorkflowError> {
        self.base_node.process_with_ai(task_context).await
    }
}

#[cfg(test)]
mod tests {
    use crate::core::nodes::agent::{ModelProvider, AgentNode};

    use super::*;

    #[test]
    fn test_openai_agent_creation() {
        let config = AgentConfig {
            system_prompt: "You are a helpful assistant".to_string(),
            model_provider: ModelProvider::OpenAI,
            model_name: "gpt-4".to_string(),
            mcp_server_uri: None,
        };

        let agent = OpenAIAgentNode::new(config).unwrap();
        assert_eq!(agent.get_agent_config().model_provider, ModelProvider::OpenAI);
    }
}
