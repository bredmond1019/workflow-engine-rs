//! # AI Agent Integrations
//!
//! This module provides concrete implementations of AI agent nodes that integrate
//! with external AI service providers. These agents can be used within workflows
//! to add AI-powered processing capabilities.
//!
//! ## Supported Providers
//!
//! ### Anthropic Claude
//! The [`anthropic::AnthropicAgentNode`] provides integration with Anthropic's
//! Claude models, supporting:
//! - Direct API calls to Claude models
//! - MCP (Model Context Protocol) tool integration
//! - Configurable system prompts and model selection
//! - Asynchronous and synchronous processing modes
//!
//! ### OpenAI GPT
//! The [`openai::OpenAIAgentNode`] provides integration with OpenAI's GPT models,
//! featuring:
//! - Chat completions API integration
//! - Built-in runtime management for async operations
//! - MCP tool support for enhanced capabilities
//! - Flexible prompt extraction from task context
//!
//! ## Usage Examples
//!
//! ### Basic Agent Setup
//!
//! ```rust
//! use ai_architecture_core::{
//!     ai_agents::{anthropic::AnthropicAgentNode, openai::OpenAIAgentNode},
//!     nodes::agent::{AgentConfig, ModelProvider},
//!     workflow::{builder::WorkflowBuilder, Workflow},
//! };
//!
//! // Configure Anthropic agent
//! let anthropic_config = AgentConfig {
//!     system_prompt: "You are Claude, an AI assistant specialized in customer support.".to_string(),
//!     model_provider: ModelProvider::Anthropic,
//!     model_name: "claude-3-opus-20240229".to_string(),
//!     mcp_server_uri: None,
//! };
//!
//! let anthropic_agent = AnthropicAgentNode::new(anthropic_config);
//!
//! // Configure OpenAI agent
//! let openai_config = AgentConfig {
//!     system_prompt: "You are GPT-4, a helpful AI assistant.".to_string(),
//!     model_provider: ModelProvider::OpenAI,
//!     model_name: "gpt-4".to_string(),
//!     mcp_server_uri: None,
//! };
//!
//! let openai_agent = OpenAIAgentNode::new(openai_config)?;
//! ```
//!
//! ### Workflow Integration
//!
//! ```rust
//! use ai_architecture_core::{
//!     ai_agents::anthropic::AnthropicAgentNode,
//!     nodes::agent::AgentConfig,
//!     workflow::builder::WorkflowBuilder,
//! };
//! use serde_json::json;
//!
//! // Build workflow with AI agent
//! let workflow = WorkflowBuilder::new("ai_processing_workflow")
//!     .start_with::<DataValidationNode>()
//!     .then::<AnthropicAgentNode>()
//!     .then::<ResponseFormatterNode>()
//!     .build()?;
//!
//! // Register the agent with configuration
//! let agent_config = AgentConfig {
//!     system_prompt: "Analyze the provided data and extract key insights.".to_string(),
//!     model_provider: ModelProvider::Anthropic,
//!     model_name: "claude-3-sonnet-20240229".to_string(),
//!     mcp_server_uri: None,
//! };
//!
//! workflow.register_node(AnthropicAgentNode::new(agent_config));
//!
//! // Process data through AI workflow
//! let result = workflow.run(json!({
//!     "prompt": "Analyze customer feedback trends",
//!     "data": {
//!         "feedback_items": ["Great service", "Could be faster", "Love the quality"]
//!     }
//! }))?;
//! ```
//!
//! ### MCP Tool Integration
//!
//! ```rust
//! use ai_architecture_core::{
//!     ai_agents::openai::OpenAIAgentNode,
//!     mcp::clients::websocket::WebSocketMCPClient,
//!     nodes::agent::AgentConfig,
//! };
//!
//! // Create agent with MCP capabilities
//! let config = AgentConfig {
//!     system_prompt: "You are an AI assistant with access to specialized tools.".to_string(),
//!     model_provider: ModelProvider::OpenAI,
//!     model_name: "gpt-4".to_string(),
//!     mcp_server_uri: Some("ws://localhost:8080/mcp".to_string()),
//! };
//!
//! let mut agent = OpenAIAgentNode::new(config)?;
//!
//! // Add MCP client for tool access
//! let mcp_client = WebSocketMCPClient::new("ws://localhost:8080/mcp".to_string());
//! agent.set_mcp_client(Box::new(mcp_client));
//!
//! // Agent can now use MCP tools during processing
//! ```
//!
//! ### Custom Prompt Extraction
//!
//! Both agents support flexible prompt extraction from task context:
//!
//! ```rust
//! use ai_architecture_core::{
//!     ai_agents::anthropic::AnthropicAgentNode,
//!     task::TaskContext,
//! };
//! use serde_json::json;
//!
//! // The agent will automatically extract prompts from:
//! // 1. "prompt" field in task context data
//! // 2. "message" field in task context data  
//! // 3. Serialized event data as fallback
//!
//! let context = TaskContext::new(
//!     "text_analysis".to_string(),
//!     json!({
//!         "prompt": "Analyze the sentiment of this text",
//!         "text": "I really enjoyed using this product!",
//!         "additional_context": "Customer review analysis"
//!     })
//! );
//!
//! // Agent will use "Analyze the sentiment of this text" as the prompt
//! ```
//!
//! ### Tool Selection and Enhancement
//!
//! Both agents support intelligent tool selection when MCP is available:
//!
//! ```rust
//! // Agents use keyword matching to select relevant tools:
//! // - "validate" tools for validation requests
//! // - "analyze" tools for analysis requests  
//! // - "generate" tools for generation requests
//! // - "process" tools for processing requests
//!
//! let context = TaskContext::new(
//!     "content_validation".to_string(),
//!     json!({
//!         "prompt": "Please validate this customer message for spam",
//!         "message": "Check out this amazing deal!"
//!     })
//! );
//!
//! // Agent will automatically select and use validation-related MCP tools
//! ```
//!
//! ## Error Handling
//!
//! AI agents handle various error conditions gracefully:
//!
//! ```rust
//! use ai_architecture_core::{error::WorkflowError, ai_agents::openai::OpenAIAgentNode};
//!
//! match agent.process(context) {
//!     Ok(result) => {
//!         println!("AI processing completed: {:?}", result);
//!     }
//!     Err(WorkflowError::ApiError { message }) => {
//!         eprintln!("AI API error: {}", message);
//!         // Handle API failures (rate limits, auth errors, etc.)
//!     }
//!     Err(WorkflowError::MCPError { message }) => {
//!         eprintln!("MCP tool error: {}", message);
//!         // Handle MCP tool failures
//!     }
//!     Err(WorkflowError::RuntimeError { message }) => {
//!         eprintln!("Runtime error: {}", message);
//!         // Handle async runtime issues
//!     }
//!     Err(e) => eprintln!("Other error: {}", e),
//! }
//! ```
//!
//! ## Configuration Options
//!
//! Both agents support comprehensive configuration:
//!
//! ```rust
//! use ai_architecture_core::nodes::agent::{AgentConfig, ModelProvider};
//!
//! let config = AgentConfig {
//!     // System prompt for consistent AI behavior
//!     system_prompt: "You are a specialized AI assistant for technical documentation.".to_string(),
//!     
//!     // Provider selection (Anthropic or OpenAI)
//!     model_provider: ModelProvider::Anthropic,
//!     
//!     // Specific model version
//!     model_name: "claude-3-opus-20240229".to_string(),
//!     
//!     // Optional MCP server for tool access
//!     mcp_server_uri: Some("ws://localhost:8080/tools".to_string()),
//! };
//! ```
//!
//! ## Async vs Sync Processing
//!
//! ### Anthropic Agent
//! - Uses channel-based communication for sync interface
//! - Spawns background threads for async operations
//! - Fallback mechanisms for runtime creation failures
//!
//! ### OpenAI Agent  
//! - Maintains internal Tokio runtime for blocking operations
//! - Direct async-to-sync conversion using `block_on`
//! - More efficient for sustained async operations
//!
//! ## Best Practices
//!
//! 1. **Environment Variables**: Set `ANTHROPIC_API_KEY` and `OPENAI_API_KEY`
//! 2. **Error Handling**: Always handle API errors and rate limiting
//! 3. **Model Selection**: Choose appropriate models for your use case
//! 4. **System Prompts**: Use specific, clear system prompts for consistent behavior
//! 5. **MCP Integration**: Leverage MCP tools for enhanced AI capabilities
//! 6. **Context Management**: Structure task context data for optimal prompt extraction
//!
//! ## Performance Considerations
//!
//! - AI API calls have latency - consider parallel processing for multiple agents
//! - MCP tool calls add overhead but provide enhanced capabilities
//! - Runtime creation (OpenAI) vs thread spawning (Anthropic) have different costs
//! - Consider caching for repeated similar requests
//!
//! ## Thread Safety
//!
//! Both agent implementations are designed for safe concurrent use:
//! - Internal HTTP clients use `Arc` for shared access
//! - Task contexts are cloned for parallel processing
//! - MCP clients handle concurrent tool calls safely

pub mod anthropic;
pub mod openai;
