//! # Template-Enhanced Agent Node
//!
//! This module provides an agent node that uses the template system
//! for dynamic prompt generation and management.

use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

use crate::core::{
    ai::templates::{Template, TemplateManager, TemplateVariables, OutputFormat},
    error::WorkflowError,
    mcp::clients::MCPClient,
    nodes::{agent::{AgentConfig, BaseAgentNode, AgentNode}, Node},
    task::TaskContext,
};

/// Configuration for template-based agent
#[derive(Debug, Clone)]
pub struct TemplateAgentConfig {
    /// Base agent configuration
    pub agent_config: AgentConfig,
    /// Template ID for system prompt
    pub system_prompt_template: Option<String>,
    /// Template ID for user message formatting
    pub user_message_template: Option<String>,
    /// Additional template variables
    pub template_vars: HashMap<String, serde_json::Value>,
    /// Whether to use template-based tool selection
    pub template_based_tools: bool,
}

/// Agent node that uses templates for prompt generation
#[derive(Debug)]
pub struct TemplateAgentNode {
    base_node: BaseAgentNode,
    template_manager: Arc<TemplateManager>,
    config: TemplateAgentConfig,
}

impl TemplateAgentNode {
    /// Create new template agent node
    pub fn new(
        config: TemplateAgentConfig,
        template_manager: Arc<TemplateManager>,
    ) -> Result<Self, WorkflowError> {
        let base_config = config.agent_config.clone();
        let base_node = BaseAgentNode::new(base_config);
        
        Ok(Self {
            base_node,
            template_manager,
            config,
        })
    }
    
    /// Set MCP client
    pub fn with_mcp_client(mut self, mcp_client: Box<dyn MCPClient>) -> Self {
        self.base_node = self.base_node.with_mcp_client(mcp_client);
        self
    }
    
    /// Generate system prompt from template
    fn generate_system_prompt(&self, context: &TaskContext) -> Result<String, WorkflowError> {
        if let Some(template_id) = &self.config.system_prompt_template {
            // Prepare variables
            let mut vars = self.config.template_vars.clone();
            
            // Add context-specific variables
            vars.insert("agent_name".to_string(), json!(self.node_name()));
            vars.insert("model".to_string(), json!(self.config.agent_config.model_name));
            
            // Add any context metadata  
            let all_data = context.get_all_data().clone();
            vars.insert("context_metadata".to_string(), json!(all_data));
            
            // Render template
            self.template_manager
                .render(template_id, &vars)
                .map_err(|e| WorkflowError::ProcessingError {
                    message: format!("Failed to render system prompt template: {}", e),
                })
        } else {
            // Use static system prompt
            Ok(self.config.agent_config.system_prompt.clone())
        }
    }
    
    /// Generate user message from template
    fn generate_user_message(
        &self,
        prompt: &str,
        context: &TaskContext,
    ) -> Result<String, WorkflowError> {
        if let Some(template_id) = &self.config.user_message_template {
            // Prepare variables
            let mut vars = self.config.template_vars.clone();
            
            // Add prompt and context data
            vars.insert("prompt".to_string(), json!(prompt));
            vars.insert("timestamp".to_string(), json!(chrono::Utc::now().to_rfc3339()));
            
            // Add event data if available
            if let Ok(event_data) = context.get_event_data::<serde_json::Value>() {
                vars.insert("data".to_string(), event_data);
            }
            
            // Add any additional context
            let all_data = context.get_all_data().clone();
            vars.insert("workflow_context".to_string(), json!(all_data));
            
            // Render template
            self.template_manager
                .render(template_id, &vars)
                .map_err(|e| WorkflowError::ProcessingError {
                    message: format!("Failed to render user message template: {}", e),
                })
        } else {
            // Return prompt as-is
            Ok(prompt.to_string())
        }
    }
    
    /// Select tools based on template analysis
    async fn select_tools_from_template(
        &self,
        prompt: &str,
    ) -> Result<Vec<String>, WorkflowError> {
        if !self.config.template_based_tools || !self.base_node.has_mcp_client() {
            return Ok(Vec::new());
        }
        
        // Use a template to analyze the prompt and suggest tools
        let analysis_vars = HashMap::from([
            ("prompt".to_string(), json!(prompt)),
            ("available_tools".to_string(), json!(self.base_node.get_mcp_tools().await?)),
        ]);
        
        let tool_analysis = self.template_manager
            .render_contextual("tool_selection", &analysis_vars)
            .unwrap_or_default();
        
        // Parse tool names from analysis
        // In a real implementation, this might use a more sophisticated parser
        let tool_names: Vec<String> = tool_analysis
            .lines()
            .filter(|line| line.starts_with("- "))
            .map(|line| line.trim_start_matches("- ").to_string())
            .collect();
        
        Ok(tool_names)
    }
}

#[async_trait]
impl Node for TemplateAgentNode {
    fn process(&self, context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Use tokio runtime for async operations
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| WorkflowError::RuntimeError {
                message: format!("Failed to create runtime: {}", e),
            })?;
        
        runtime.block_on(self.process_with_ai(context))
    }
    
    fn node_name(&self) -> String {
        format!("TemplateAgentNode[{:?}]", self.config.agent_config.model_provider)
    }
}

#[async_trait]
impl AgentNode for TemplateAgentNode {
    fn get_agent_config(&self) -> AgentConfig {
        self.config.agent_config.clone()
    }
    
    async fn process_with_ai(
        &self,
        mut context: TaskContext,
    ) -> Result<TaskContext, WorkflowError> {
        // Extract prompt from context
        let prompt = context
            .get_event_data::<serde_json::Value>()
            .ok()
            .and_then(|data| {
                data.get("prompt")
                    .or_else(|| data.get("message"))
                    .and_then(|v| v.as_str())
                    .map(String::from)
            })
            .unwrap_or_else(|| {
                context
                    .get_event_data::<serde_json::Value>()
                    .map(|v| serde_json::to_string(&v).unwrap_or_default())
                    .unwrap_or_default()
            });
        
        // Generate system prompt from template
        let system_prompt = self.generate_system_prompt(&context)?;
        
        // Generate user message from template
        let user_message = self.generate_user_message(&prompt, &context)?;
        
        // Select tools if template-based selection is enabled
        let selected_tools = self.select_tools_from_template(&prompt).await?;
        
        // Create a modified agent config with the generated prompts
        let mut modified_config = self.config.agent_config.clone();
        modified_config.system_prompt = system_prompt;
        
        // Create a temporary base node with the modified config
        let temp_node = BaseAgentNode::new(modified_config);
        
        // Process with the base node
        let mut result_context = temp_node.process_with_ai(context).await?;
        
        // Add template metadata to the result
        result_context.set_metadata("template_enhanced", json!(true))?;
        result_context.set_metadata("selected_tools", json!(selected_tools))?;
        
        Ok(result_context)
    }
}

/// Builder for template agent configuration
pub struct TemplateAgentBuilder {
    agent_config: AgentConfig,
    system_prompt_template: Option<String>,
    user_message_template: Option<String>,
    template_vars: HashMap<String, serde_json::Value>,
    template_based_tools: bool,
}

impl TemplateAgentBuilder {
    /// Create new builder
    pub fn new(agent_config: AgentConfig) -> Self {
        Self {
            agent_config,
            system_prompt_template: None,
            user_message_template: None,
            template_vars: HashMap::new(),
            template_based_tools: false,
        }
    }
    
    /// Set system prompt template
    pub fn with_system_prompt_template(mut self, template_id: impl Into<String>) -> Self {
        self.system_prompt_template = Some(template_id.into());
        self
    }
    
    /// Set user message template
    pub fn with_user_message_template(mut self, template_id: impl Into<String>) -> Self {
        self.user_message_template = Some(template_id.into());
        self
    }
    
    /// Add template variable
    pub fn with_template_var(
        mut self,
        name: impl Into<String>,
        value: serde_json::Value,
    ) -> Self {
        self.template_vars.insert(name.into(), value);
        self
    }
    
    /// Enable template-based tool selection
    pub fn with_template_based_tools(mut self) -> Self {
        self.template_based_tools = true;
        self
    }
    
    /// Build the configuration
    pub fn build(self) -> TemplateAgentConfig {
        TemplateAgentConfig {
            agent_config: self.agent_config,
            system_prompt_template: self.system_prompt_template,
            user_message_template: self.user_message_template,
            template_vars: self.template_vars,
            template_based_tools: self.template_based_tools,
        }
    }
}

/// Pre-configured templates for common agent scenarios
pub fn register_default_agent_templates(
    template_manager: &mut TemplateManager,
) -> Result<(), WorkflowError> {
    // System prompt template with role and capabilities
    let system_prompt = Template::new(
        "agent_system_prompt",
        r#"You are {{agent_name}}, an AI assistant powered by {{model}}.

{{#if capabilities}}
Your capabilities include:
{{#each capabilities}}
- {{this}}
{{/each}}
{{/if}}

{{#if instructions}}
Special instructions:
{{instructions}}
{{/if}}

{{#if context_metadata}}
Current context:
{{json context_metadata}}
{{/if}}

Always be helpful, accurate, and concise in your responses."#
    ).map_err(|e| WorkflowError::ProcessingError {
        message: format!("Failed to create system prompt template: {}", e),
    })?;
    
    template_manager.register(system_prompt)?;
    
    // User message template with context
    let user_message = Template::new(
        "agent_user_message",
        r#"{{#if workflow_context}}
Previous context:
{{json workflow_context}}

{{/if}}
{{#if data}}
Current data:
{{json data}}

{{/if}}
User request: {{prompt}}

Timestamp: {{timestamp}}"#
    ).map_err(|e| WorkflowError::ProcessingError {
        message: format!("Failed to create user message template: {}", e),
    })?;
    
    template_manager.register(user_message)?;
    
    // Tool selection template
    let tool_selection = Template::new(
        "tool_selection",
        r#"Analyze the following user request and suggest which tools to use:

Request: {{prompt}}

Available tools:
{{#each available_tools}}
- {{this.name}}: {{this.description}}
{{/each}}

List the tool names that would be most helpful for this request, one per line with a dash prefix:"#
    ).map_err(|e| WorkflowError::ProcessingError {
        message: format!("Failed to create tool selection template: {}", e),
    })?
    .with_context("tool_selection");
    
    template_manager.register(tool_selection)?;
    
    // Error response template
    let error_response = Template::new(
        "agent_error_response",
        r#"I encountered an error while processing your request.

Error Type: {{error_type}}
Error Message: {{error_message}}

{{#if suggestions}}
Suggestions:
{{#each suggestions}}
- {{this}}
{{/each}}
{{/if}}

Please try again or contact support if the issue persists."#
    ).map_err(|e| WorkflowError::ProcessingError {
        message: format!("Failed to create error response template: {}", e),
    })?
    .with_context("error");
    
    template_manager.register(error_response)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nodes::agent::ModelProvider;
    
    #[test]
    fn test_template_agent_builder() {
        let agent_config = AgentConfig {
            system_prompt: "Default prompt".to_string(),
            model_provider: ModelProvider::OpenAI,
            model_name: "gpt-4".to_string(),
            mcp_server_uri: None,
        };
        
        let config = TemplateAgentBuilder::new(agent_config)
            .with_system_prompt_template("agent_system_prompt")
            .with_user_message_template("agent_user_message")
            .with_template_var("department", json!("Engineering"))
            .with_template_based_tools()
            .build();
        
        assert_eq!(config.system_prompt_template, Some("agent_system_prompt".to_string()));
        assert_eq!(config.user_message_template, Some("agent_user_message".to_string()));
        assert!(config.template_based_tools);
        assert_eq!(config.template_vars.get("department"), Some(&json!("Engineering")));
    }
}