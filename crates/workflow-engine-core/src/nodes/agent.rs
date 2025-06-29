use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::Arc;
use futures_util::stream::{Stream, StreamExt};
use std::pin::Pin;

use crate::error::WorkflowError;
// // use workflow_engine_mcp::clients::MCPClient;  // Removed to avoid circular dependency
use crate::nodes::Node;
use crate::task::TaskContext;

/// Supported model providers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelProvider {
    OpenAI,
    AzureOpenAI,
    Anthropic,
    Bedrock,
}

/// Configuration for an agent node
#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub system_prompt: String,
    pub model_provider: ModelProvider,
    pub model_name: String,
    pub mcp_server_uri: Option<String>,
}

/// Base trait for agent nodes that process tasks using AI models
#[async_trait]
pub trait AgentNode: Node {
    /// Get the configuration for this agent
    fn get_agent_config(&self) -> AgentConfig;

    /// Process the task context using the configured AI model
    async fn process_with_ai(
        &self,
        task_context: TaskContext,
    ) -> Result<TaskContext, WorkflowError>;
}

/// A basic implementation of an agent node
#[derive(Debug)]
pub struct BaseAgentNode {
    config: AgentConfig,
    client: Arc<reqwest::Client>,
    // mcp_client: Option<Arc<tokio::sync::Mutex<Box<dyn MCPClient>>>>,
}

impl BaseAgentNode {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config,
            client: Arc::new(reqwest::Client::new()),
            // mcp_client: None,
        }
    }

    // MCP integration stub implementations - circular dependency prevents full implementation
    // These methods provide API compatibility until dependency architecture is refactored
    pub fn with_mcp_client(self, _mcp_client: Box<dyn std::any::Any + Send + Sync>) -> Self {
        // Currently a no-op until MCP integration is fixed
        self
    }

    pub fn set_mcp_client(&mut self, _mcp_client: Box<dyn std::any::Any + Send + Sync>) {
        // Currently a no-op until MCP integration is fixed
    }

    pub fn has_mcp_client(&self) -> bool {
        // Always return false until MCP integration is fixed
        false
    }

    /*
    // MCP functionality temporarily commented out to avoid circular dependency
    pub async fn get_mcp_tools(
        &self,
    ) -> Result<Vec<crate::models::ToolDefinition>, WorkflowError> {
        if let Some(ref client) = self.mcp_client {
            let mut client_guard = client.lock().await;
            client_guard.list_tools().await
        } else {
            Ok(vec![])
        }
    }

    pub async fn call_mcp_tool(
        &self,
        name: &str,
        arguments: Option<std::collections::HashMap<String, serde_json::Value>>,
    ) -> Result<crate::models::CallToolResult, WorkflowError> {
        if let Some(ref client) = self.mcp_client {
            let mut client_guard = client.lock().await;
            client_guard.call_tool(name, arguments).await
        } else {
            Err(WorkflowError::mcp_error_simple("No MCP client configured"))
        }
    }
    */

    async fn get_model_instance(&self) -> Result<Box<dyn ModelInstance>, WorkflowError> {
        match self.config.model_provider {
            ModelProvider::OpenAI => {
                let instance = OpenAIModelInstance {
                    client: self.client.clone(),
                    model_name: self.config.model_name.clone(),
                    system_prompt: self.config.system_prompt.clone(),
                };
                Ok(Box::new(instance))
            }
            ModelProvider::AzureOpenAI => {
                // For now, Azure OpenAI uses the same implementation as OpenAI
                // In production, you might want to handle Azure-specific endpoints
                let instance = OpenAIModelInstance {
                    client: self.client.clone(),
                    model_name: self.config.model_name.clone(),
                    system_prompt: self.config.system_prompt.clone(),
                };
                Ok(Box::new(instance))
            }
            ModelProvider::Anthropic => {
                let instance = AnthropicModelInstance {
                    client: self.client.clone(),
                    model_name: self.config.model_name.clone(),
                    system_prompt: self.config.system_prompt.clone(),
                };
                Ok(Box::new(instance))
            }
            #[cfg(feature = "aws")]
            ModelProvider::Bedrock => {
                let instance = BedrockModelInstance {
                    model_name: self.config.model_name.clone(),
                    system_prompt: self.config.system_prompt.clone(),
                };
                Ok(Box::new(instance))
            }
            #[cfg(not(feature = "aws"))]
            ModelProvider::Bedrock => {
                Err(WorkflowError::configuration_error(
                    "Bedrock provider requires 'aws' feature to be enabled",
                    "model_provider",
                    "environment",
                    "bedrock feature flag",
                    Some("bedrock".to_string())
                ))
            }
        }
    }
    
    /// Extract prompt from the task context
    fn extract_prompt_from_context(&self, task_context: &TaskContext) -> Result<String, WorkflowError> {
        // Try various common fields for the prompt
        if let Ok(Some(prompt)) = task_context.get_data::<serde_json::Value>("prompt") {
            if let Some(prompt_str) = prompt.as_str() {
                return Ok(prompt_str.to_string());
            }
        }
        
        if let Ok(Some(message)) = task_context.get_data::<serde_json::Value>("message") {
            if let Some(message_str) = message.as_str() {
                return Ok(message_str.to_string());
            }
        }
        
        if let Ok(Some(query)) = task_context.get_data::<serde_json::Value>("query") {
            if let Some(query_str) = query.as_str() {
                return Ok(query_str.to_string());
            }
        }
        
        // Fallback to the entire event data as a string
        Ok(serde_json::to_string_pretty(&task_context.event_data)
            .unwrap_or_else(|_| task_context.event_data.to_string()))
    }
    
    /// Enhance prompt with MCP tool results if available (stub)
    /// Full MCP functionality available in workflow-engine-mcp crate
    async fn enhance_prompt_with_mcp(
        &self,
        original_prompt: &str,
        _task_context: &TaskContext,
    ) -> Result<String, WorkflowError> {
        // Stub implementation - actual MCP tool integration is in workflow-engine-mcp crate
        Ok(original_prompt.to_string())
    }
    
    fn select_relevant_tools(
        &self,
        tools: &[crate::models::ToolDefinition],
        prompt: &str,
    ) -> Result<Vec<crate::models::ToolDefinition>, WorkflowError> {
        // Simple keyword-based relevance matching
        let prompt_lower = prompt.to_lowercase();
        let relevant_tools: Vec<_> = tools
            .iter()
            .filter(|tool| {
                let tool_name_lower = tool.name.to_lowercase();
                let description_lower = tool.description
                    .as_ref()
                    .map(|d| d.to_lowercase())
                    .unwrap_or_default();
                
                // Check if tool name or description is mentioned in prompt
                prompt_lower.contains(&tool_name_lower) ||
                (!description_lower.is_empty() && 
                 description_lower.split_whitespace()
                    .any(|word| prompt_lower.contains(word)))
            })
            .cloned()
            .collect();
        
        Ok(relevant_tools)
    }
    
    fn prepare_tool_arguments(
        &self,
        task_context: &TaskContext,
        tool: &crate::models::ToolDefinition,
    ) -> Result<std::collections::HashMap<String, serde_json::Value>, WorkflowError> {
        let mut args = std::collections::HashMap::new();
        
        // Add context data
        args.insert("context_data".to_string(), 
                   serde_json::to_value(task_context.get_all_data())?);
        
        // Add metadata if available
        let metadata = task_context.get_all_metadata();
        if !metadata.is_empty() {
            args.insert("metadata".to_string(), 
                       serde_json::to_value(metadata)?);
        }
        
        Ok(args)
    }
    
    fn enhance_prompt_with_tool_results(
        &self,
        original_prompt: &str,
        tool_results: &[(String, crate::models::CallToolResult)],
    ) -> Result<String, WorkflowError> {
        if tool_results.is_empty() {
            return Ok(original_prompt.to_string());
        }
        
        let mut enhanced = format!("User request: {}\n\nAdditional context from tools:\n", original_prompt);
        
        for (tool_name, result) in tool_results {
            enhanced.push_str(&format!("\n[{}]:\n", tool_name));
            for content in &result.content {
                match content {
                    crate::models::ToolContent::Text { text } => {
                        enhanced.push_str(text);
                        enhanced.push('\n');
                    }
                    _ => {
                        enhanced.push_str("(non-text content)\n");
                    }
                }
            }
        }
        
        enhanced.push_str("\nPlease provide a response based on the user request and the additional context above.");
        
        Ok(enhanced)
    }
}

impl Node for BaseAgentNode {
    fn process(&self, task_context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Create a runtime to execute async code
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| WorkflowError::RuntimeError {
                message: format!("Failed to create runtime: {}", e),
            })?;
        
        // Block on the async process_with_ai method
        runtime.block_on(self.process_with_ai(task_context))
    }
}

#[async_trait]
impl AgentNode for BaseAgentNode {
    fn get_agent_config(&self) -> AgentConfig {
        self.config.clone()
    }

    async fn process_with_ai(
        &self,
        mut task_context: TaskContext,
    ) -> Result<TaskContext, WorkflowError> {
        let model = self.get_model_instance().await?;
        
        // Extract prompt from context
        let prompt = self.extract_prompt_from_context(&task_context)?;
        
        // MCP enhancement is handled in workflow-engine-mcp crate
        let enhanced_prompt = prompt;
        
        // Process the request with the model
        let response = model.process_request(&enhanced_prompt).await?;
        
        // Store the response in the task context
        task_context.update_node("ai_response", serde_json::json!({
            "response": response,
            "model": self.config.model_name.clone(),
            "provider": format!("{:?}", self.config.model_provider),
            "timestamp": chrono::Utc::now()
        }));
        
        Ok(task_context)
    }
}

/// Trait for model instances that can process requests
#[async_trait]
pub trait ModelInstance: Send + Sync + Debug {
    /// Process a request and return the complete response
    async fn process_request(&self, prompt: &str) -> Result<String, WorkflowError>;
    
    /// Process a request and return a stream of response chunks
    async fn process_request_stream(
        &self,
        prompt: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, WorkflowError>> + Send>>, WorkflowError> {
        // Default implementation: return the complete response as a single-item stream
        let response = self.process_request(prompt).await?;
        let stream = futures_util::stream::once(async move { Ok(response) });
        Ok(Box::pin(stream))
    }
}

/// Response chunk for streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub content: String,
    pub is_final: bool,
}

/// Configuration for streaming responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamConfig {
    pub enabled: bool,
    pub chunk_size: Option<usize>,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            chunk_size: None,
        }
    }
}

/// OpenAI model instance implementation
#[derive(Debug)]
struct OpenAIModelInstance {
    client: Arc<reqwest::Client>,
    model_name: String,
    system_prompt: String,
}

#[async_trait]
impl ModelInstance for OpenAIModelInstance {
    async fn process_request(&self, prompt: &str) -> Result<String, WorkflowError> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| WorkflowError::configuration_error_simple("OPENAI_API_KEY not set"))?;
        
        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&serde_json::json!({
                "model": &self.model_name,
                "messages": [
                    {
                        "role": "system",
                        "content": &self.system_prompt
                    },
                    {
                        "role": "user",
                        "content": prompt
                    }
                ],
                "stream": false
            }))
            .send()
            .await
            .map_err(|e| WorkflowError::api_error(
                format!("OpenAI API request failed: {}", e),
                "OpenAI",
                "https://api.openai.com/v1/chat/completions",
                e.status().map(|s| s.as_u16()),
            ))?;
        
        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            return Err(WorkflowError::api_error(
                format!("OpenAI API error: {} - {}", status, error_body),
                "OpenAI",
                "https://api.openai.com/v1/chat/completions",
                Some(status.as_u16()),
            ));
        }
        
        let result = response
            .json::<serde_json::Value>()
            .await
            .map_err(|e| WorkflowError::api_error(
                format!("Failed to parse OpenAI response: {}", e),
                "OpenAI",
                "https://api.openai.com/v1/chat/completions",
                None,
            ))?;
        
        result["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| WorkflowError::api_error(
                "Invalid response structure from OpenAI",
                "OpenAI",
                "https://api.openai.com/v1/chat/completions",
                None,
            ))
            .map(|s| s.to_string())
    }
    
    // Real streaming support using the streaming module
    #[cfg(feature = "streaming")]
    async fn process_request_stream(
        &self,
        prompt: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, WorkflowError>> + Send>>, WorkflowError> {
        use crate::streaming::providers::OpenAIStreamingProvider;
        use crate::streaming::types::{StreamConfig, StreamingProvider};
        
        let provider = OpenAIStreamingProvider::new(
            self.client.clone(),
            self.model_name.clone(),
            self.system_prompt.clone(),
        );
        
        let config = StreamConfig::default();
        let chunk_stream = provider.stream_response(prompt, &config);
        
        // Convert StreamChunk stream to String stream
        let string_stream = chunk_stream.map(|chunk_result| {
            chunk_result.map(|chunk| chunk.content)
        });
        
        Ok(Box::pin(string_stream))
    }
}

/// Anthropic model instance implementation
#[derive(Debug)]
struct AnthropicModelInstance {
    client: Arc<reqwest::Client>,
    model_name: String,
    system_prompt: String,
}

#[async_trait]
impl ModelInstance for AnthropicModelInstance {
    async fn process_request(&self, prompt: &str) -> Result<String, WorkflowError> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| WorkflowError::configuration_error_simple("ANTHROPIC_API_KEY not set"))?;
        
        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&serde_json::json!({
                "model": &self.model_name,
                "max_tokens": 4096,
                "messages": [
                    {
                        "role": "user",
                        "content": prompt
                    }
                ],
                "system": &self.system_prompt
            }))
            .send()
            .await
            .map_err(|e| WorkflowError::api_error(
                format!("Anthropic API request failed: {}", e),
                "Anthropic",
                "https://api.anthropic.com/v1/messages",
                e.status().map(|s| s.as_u16()),
            ))?;
        
        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            return Err(WorkflowError::api_error(
                format!("Anthropic API error: {} - {}", status, error_body),
                "Anthropic",
                "https://api.anthropic.com/v1/messages",
                Some(status.as_u16()),
            ));
        }
        
        let result = response
            .json::<serde_json::Value>()
            .await
            .map_err(|e| WorkflowError::api_error(
                format!("Failed to parse Anthropic response: {}", e),
                "Anthropic",
                "https://api.anthropic.com/v1/messages",
                None,
            ))?;
        
        result["content"][0]["text"]
            .as_str()
            .ok_or_else(|| WorkflowError::api_error(
                "Invalid response structure from Anthropic",
                "Anthropic",
                "https://api.anthropic.com/v1/messages",
                None,
            ))
            .map(|s| s.to_string())
    }
    
    // Real streaming support using the streaming module
    #[cfg(feature = "streaming")]
    async fn process_request_stream(
        &self,
        prompt: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, WorkflowError>> + Send>>, WorkflowError> {
        use crate::streaming::providers::AnthropicStreamingProvider;
        use crate::streaming::types::{StreamConfig, StreamingProvider};
        
        let provider = AnthropicStreamingProvider::new(
            self.client.clone(),
            self.model_name.clone(),
            self.system_prompt.clone(),
        );
        
        let config = StreamConfig::default();
        let chunk_stream = provider.stream_response(prompt, &config);
        
        // Convert StreamChunk stream to String stream
        let string_stream = chunk_stream.map(|chunk_result| {
            chunk_result.map(|chunk| chunk.content)
        });
        
        Ok(Box::pin(string_stream))
    }
}

/// AWS Bedrock model instance implementation
#[cfg(feature = "aws")]
#[derive(Debug)]
struct BedrockModelInstance {
    model_name: String,
    system_prompt: String,
}

#[cfg(feature = "aws")]
#[async_trait]
impl ModelInstance for BedrockModelInstance {
    async fn process_request(&self, prompt: &str) -> Result<String, WorkflowError> {
        use aws_sdk_bedrockruntime::{primitives::Blob, Client};
        
        // Initialize AWS SDK
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let client = Client::new(&config);
        
        // Prepare the request body based on the model
        let request_body = if self.model_name.starts_with("anthropic.claude") {
            serde_json::json!({
                "anthropic_version": "bedrock-2023-05-31",
                "max_tokens": 4096,
                "messages": [{
                    "role": "user",
                    "content": prompt
                }],
                "system": &self.system_prompt
            })
        } else if self.model_name.starts_with("amazon.titan") {
            serde_json::json!({
                "inputText": format!("{}\n\n{}", self.system_prompt, prompt),
                "textGenerationConfig": {
                    "maxTokenCount": 4096,
                    "temperature": 0.7,
                    "topP": 0.9
                }
            })
        } else {
            return Err(WorkflowError::configuration_error_simple(
                format!("Unsupported Bedrock model: {}", self.model_name)
            ));
        };
        
        let body = Blob::new(serde_json::to_vec(&request_body).map_err(|e| {
            WorkflowError::serialization_error(
                format!("Failed to serialize request body: {}", e),
                "BedrockRequestBody",
                "during API request preparation",
            )
        })?);
        
        let response = client
            .invoke_model()
            .model_id(&self.model_name)
            .content_type("application/json")
            .accept("application/json")
            .body(body)
            .send()
            .await
            .map_err(|e| WorkflowError::api_error(
                format!("Bedrock API request failed: {}", e),
                "AWS Bedrock",
                &format!("model/{}", self.model_name),
                None,
            ))?;
        
        let response_body = response.body().as_ref();
        let response_json: serde_json::Value = serde_json::from_slice(response_body)
            .map_err(|e| WorkflowError::deserialization_error(
                format!("Failed to parse Bedrock response: {}", e),
                "serde_json::Value",
                "from Bedrock API response",
                Some(String::from_utf8_lossy(response_body).to_string()),
            ))?;
        
        // Extract text based on model response format
        if self.model_name.starts_with("anthropic.claude") {
            response_json["content"][0]["text"]
                .as_str()
                .ok_or_else(|| WorkflowError::api_error(
                    "Invalid response structure from Bedrock Claude",
                    "AWS Bedrock",
                    &format!("model/{}", self.model_name),
                    None,
                ))
                .map(|s| s.to_string())
        } else if self.model_name.starts_with("amazon.titan") {
            response_json["results"][0]["outputText"]
                .as_str()
                .ok_or_else(|| WorkflowError::api_error(
                    "Invalid response structure from Bedrock Titan",
                    "AWS Bedrock",
                    &format!("model/{}", self.model_name),
                    None,
                ))
                .map(|s| s.to_string())
        } else {
            Err(WorkflowError::api_error(
                "Unknown response format",
                "AWS Bedrock",
                &format!("model/{}", self.model_name),
                None,
            ))
        }
    }
    
    // Real streaming support using the streaming module
    #[cfg(feature = "streaming")]
    async fn process_request_stream(
        &self,
        prompt: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, WorkflowError>> + Send>>, WorkflowError> {
        use crate::streaming::providers::BedrockStreamingProvider;
        use crate::streaming::types::{StreamConfig, StreamingProvider};
        
        let provider = BedrockStreamingProvider::new(
            self.model_name.clone(),
            self.system_prompt.clone(),
        );
        
        let config = StreamConfig::default();
        let chunk_stream = provider.stream_response(prompt, &config);
        
        // Convert StreamChunk stream to String stream
        let string_stream = chunk_stream.map(|chunk_result| {
            chunk_result.map(|chunk| chunk.content)
        });
        
        Ok(Box::pin(string_stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_config_creation() {
        let config = AgentConfig {
            system_prompt: "Test prompt".to_string(),
            model_provider: ModelProvider::OpenAI,
            model_name: "gpt-4".to_string(),
            mcp_server_uri: None,
        };

        assert_eq!(config.model_provider, ModelProvider::OpenAI);
        assert_eq!(config.model_name, "gpt-4");
    }

    #[test]
    fn test_base_agent_node_creation() {
        let config = AgentConfig {
            system_prompt: "Test prompt".to_string(),
            model_provider: ModelProvider::OpenAI,
            model_name: "gpt-4".to_string(),
            mcp_server_uri: None,
        };

        let agent = BaseAgentNode::new(config);
        assert_eq!(
            agent.get_agent_config().model_provider,
            ModelProvider::OpenAI
        );
    }
}
