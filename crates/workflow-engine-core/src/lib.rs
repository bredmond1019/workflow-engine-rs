//! # Workflow Engine Core
//! 
//! This crate provides the core workflow engine primitives including:
//! - Node trait definitions and execution framework
//! - Task context and workflow state management
//! - Error handling and recovery mechanisms
//! - Template engine and AI integration utilities
//! - Workflow builder and validation
//! 
//! ## Features
//! 
//! - `database` - Enables database integration with Diesel ORM
//! - `monitoring` - Enables Prometheus metrics collection  
//! - `aws` - Enables AWS Bedrock AI integration
//! - `full` - Enables all optional features
//! 
//! ## Core Concepts
//! 
//! The workflow engine is built around these key abstractions:
//! 
//! - **Nodes**: Processing units that implement the [`Node`] trait
//! - **TaskContext**: Carries workflow state and data between nodes
//! - **WorkflowBuilder**: Type-safe workflow construction
//! - **AsyncNode**: Async node execution (coming in next version)
//! 
//! ## Examples
//! 
//! ```rust
//! use workflow_engine_core::{
//!     nodes::Node,
//!     task::TaskContext,
//!     error::WorkflowError,
//! };
//! use serde_json::json;
//! 
//! #[derive(Debug)]
//! struct ExampleNode;
//! 
//! impl Node for ExampleNode {
//!     fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
//!         let input: serde_json::Value = context.get_event_data()?;
//!         context.update_node("result", json!({"processed": true}));
//!         Ok(context)
//!     }
//! }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]

// Core modules - always available
pub mod error;
pub mod task;
pub mod nodes;
pub mod workflow;
pub mod ai;
pub mod auth;
pub mod models;
pub mod config;
pub mod mcp;
#[cfg(feature = "streaming")]
#[cfg_attr(docsrs, doc(cfg(feature = "streaming")))]
pub mod streaming;

// Feature-gated modules
#[cfg(feature = "database")]
#[cfg_attr(docsrs, doc(cfg(feature = "database")))]
pub mod registry;


// Monitoring is now in the API crate

// Re-export commonly used types
pub use error::{WorkflowError, Result, ErrorCategory, ErrorSeverity};
pub use task::TaskContext;
pub use nodes::{
    Node, Router, ParallelNode, AsyncNode, AsyncNodeAdapter,
    type_safe::{NodeId, TypedNodeConfig, TypedWorkflowBuilder, TypedWorkflow}
};
pub use workflow::builder::WorkflowBuilder;

// Feature-specific re-exports are now in respective crates

/// Current version of the workflow engine core
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Prelude module for common imports
pub mod prelude {
    pub use crate::{
        Node, Router, ParallelNode, AsyncNode, AsyncNodeAdapter,
        NodeId, TypedNodeConfig, TypedWorkflowBuilder, TypedWorkflow,
        TaskContext, WorkflowError, Result, WorkflowBuilder,
    };
    pub use async_trait::async_trait;
    pub use serde_json::{json, Value};
    pub use uuid::Uuid;
}