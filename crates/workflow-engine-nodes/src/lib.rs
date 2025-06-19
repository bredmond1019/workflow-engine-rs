//! # Workflow Engine Nodes
//! 
//! Built-in workflow nodes for the AI workflow engine.
//! This crate provides ready-to-use node implementations:
//! 
//! - AI agent nodes (OpenAI, Anthropic, AWS Bedrock)
//! - External MCP client nodes 
//! - Research and analysis nodes
//! - Template processing nodes
//! 
//! ## Features
//! 
//! - `ai-agents` - AI service integration nodes (enabled by default)
//! - `external-mcp` - External MCP server integration (enabled by default)
//! - `research` - Research and analysis nodes
//! - `template` - Template processing and generation nodes
//! - `all` - All node types
//! 
//! ## Node Categories
//! 
//! - **AI Agents**: Integrate with AI services for intelligent processing
//! - **External MCP**: Connect to external MCP servers for tool access
//! - **Research**: Perform research and data analysis tasks
//! - **Template**: Process templates and generate content
//! 
//! ## Examples
//! 
//! ```rust
//! use workflow_engine_nodes::ai_agents::openai::OpenAiNode;
//! use workflow_engine_core::prelude::*;
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), WorkflowError> {
//!     let node = OpenAiNode::new("gpt-4", "You are a helpful assistant")?;
//!     
//!     let context = TaskContext::new(
//!         "test".to_string(),
//!         json!({"message": "Hello, world!"})
//!     );
//!     
//!     let result = node.process(context)?;
//!     println!("AI response: {:?}", result);
//!     Ok(())
//! }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]

// AI agent nodes
// TODO: Re-enable when AI agent implementations are fixed
// #[cfg(feature = "ai-agents")]
// #[cfg_attr(docsrs, doc(cfg(feature = "ai-agents")))]
// pub mod ai_agents;

// External MCP nodes  
#[cfg(feature = "external-mcp")]
#[cfg_attr(docsrs, doc(cfg(feature = "external-mcp")))]
pub mod external_mcp;

// Research nodes
#[cfg(feature = "research")]
#[cfg_attr(docsrs, doc(cfg(feature = "research")))]
pub mod research;

// Template nodes
#[cfg(feature = "template")]
#[cfg_attr(docsrs, doc(cfg(feature = "template")))]
pub mod template;

// Common node utilities
pub mod utils;

// Re-export commonly used types
// TODO: Re-enable when AI agent implementations are fixed
// #[cfg(feature = "ai-agents")]
// pub use ai_agents::{openai::OpenAiNode, anthropic::AnthropicNode};

/// Current version of the nodes library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Prelude module for common node imports
pub mod prelude {
    // TODO: Re-enable when AI agent implementations are fixed
    // #[cfg(feature = "ai-agents")]
    // pub use crate::ai_agents::*;
    
    #[cfg(feature = "external-mcp")]
    pub use crate::external_mcp::*;
    
    pub use workflow_engine_core::prelude::*;
    pub use workflow_engine_mcp::prelude::*;
}