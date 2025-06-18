//! # AI Architecture Core Module
//!
//! This module provides the fundamental building blocks for creating and executing
//! AI-powered workflows in the AI Architecture system. It contains the core traits,
//! types, and error handling mechanisms needed to build robust, scalable AI applications.
//!
//! ## Architecture Overview
//!
//! The core module is built around several key concepts:
//!
//! - **Workflows**: Directed graphs of processing nodes that define AI task execution
//! - **Nodes**: Individual processing units that perform specific AI or business logic
//! - **Task Context**: Data container that flows through workflow execution
//! - **MCP Integration**: Model Context Protocol support for external AI service communication
//! - **Error Handling**: Comprehensive error types for robust failure management
//!
//! ## Module Structure
//!
//! ### [`workflow`]
//! Core workflow execution engine and schema definition. Provides the main
//! [`Workflow`](workflow::Workflow) type for building and running AI task flows.
//!
//! ### [`nodes`]
//! Node types and registry for workflow components. Includes base [`Node`](nodes::Node)
//! trait and specialized node types for different processing patterns.
//!
//! ### [`task`]
//! Task context data structures for workflow state management. The
//! [`TaskContext`](task::TaskContext) flows through nodes during execution.
//!
//! ### [`ai_agents`]
//! AI service integrations for Anthropic, OpenAI, and other providers.
//! Provides standardized interfaces for AI model interactions.
//!
//! ### [`mcp`]
//! Model Context Protocol (MCP) implementation for external service communication.
//! Supports both client and server-side MCP operations.
//!
//! ### [`error`]
//! Comprehensive error types and handling for all core operations.
//! Uses [`WorkflowError`](error::WorkflowError) as the primary error type.
//!
//! ## Quick Start
//!
//! Here's a simple example of creating and running a workflow:
//!
//! ```rust
//! use ai_architecture_core::{
//!     workflow::{builder::WorkflowBuilder, Workflow},
//!     nodes::Node,
//!     task::TaskContext,
//!     error::WorkflowError,
//! };
//! use serde_json::json;
//!
//! // Define a simple processing node
//! #[derive(Debug)]
//! struct GreetingNode;
//!
//! impl Node for GreetingNode {
//!     fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
//!         let data = context.get_event_data::<serde_json::Value>()?;
//!         let name = data.get("name").and_then(|v| v.as_str()).unwrap_or("World");
//!         
//!         context.update_node("greeting", json!({
//!             "message": format!("Hello, {}!", name)
//!         }));
//!         
//!         Ok(context)
//!     }
//! }
//!
//! // Build and run workflow
//! async fn run_greeting_workflow() -> Result<(), WorkflowError> {
//!     let workflow = WorkflowBuilder::new("greeting_workflow")
//!         .start_with::<GreetingNode>()
//!         .build()?;
//!     
//!     workflow.register_node(GreetingNode);
//!     
//!     let result = workflow.run(json!({"name": "Alice"}))?;
//!     println!("Workflow result: {:?}", result);
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Advanced Features
//!
//! ### Parallel Execution
//!
//! Workflows support parallel node execution for improved performance:
//!
//! ```rust
//! use ai_architecture_core::workflow::builder::WorkflowBuilder;
//!
//! let workflow = WorkflowBuilder::new("parallel_workflow")
//!     .start_with::<StartNode>()
//!     .parallel(&[
//!         std::any::TypeId::of::<ProcessorA>(),
//!         std::any::TypeId::of::<ProcessorB>(),
//!         std::any::TypeId::of::<ProcessorC>(),
//!     ])
//!     .then::<CombinerNode>()
//!     .build()?;
//! ```
//!
//! ### Router Nodes
//!
//! Create conditional workflow paths with router nodes:
//!
//! ```rust
//! use ai_architecture_core::nodes::{Node, Router};
//!
//! #[derive(Debug)]
//! struct ConditionalRouter;
//!
//! impl Router for ConditionalRouter {
//!     fn route(&self, context: &TaskContext) -> Option<Box<dyn Node>> {
//!         let priority = context.get_event_data::<serde_json::Value>()
//!             .ok()?
//!             .get("priority")?
//!             .as_str()?;
//!             
//!         match priority {
//!             "high" => Some(Box::new(HighPriorityNode)),
//!             "normal" => Some(Box::new(NormalPriorityNode)),
//!             _ => Some(Box::new(DefaultNode)),
//!         }
//!     }
//! }
//! ```
//!
//! ### MCP Integration
//!
//! Expose workflows as MCP servers or connect to external MCP services:
//!
//! ```rust
//! use ai_architecture_core::{
//!     mcp::transport::TransportType,
//!     workflow::Workflow,
//! };
//!
//! async fn setup_mcp_integration(workflow: &Workflow) -> Result<(), WorkflowError> {
//!     // Expose workflow as MCP server
//!     let mcp_server = workflow
//!         .expose_as_mcp_server("ai-workflow", "1.0.0")
//!         .await?;
//!     
//!     // Register external MCP server
//!     workflow.register_mcp_server(
//!         "ws://localhost:8080/mcp",
//!         TransportType::WebSocket {
//!             url: "ws://localhost:8080/mcp".to_string(),
//!         },
//!     ).await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Error Handling
//!
//! All operations return [`Result`] types with [`WorkflowError`](error::WorkflowError):
//!
//! ```rust
//! use ai_architecture_core::error::WorkflowError;
//!
//! match workflow.run(event_data) {
//!     Ok(result) => println!("Success: {:?}", result),
//!     Err(WorkflowError::CycleDetected) => {
//!         eprintln!("Workflow contains a cycle");
//!     }
//!     Err(WorkflowError::NodeNotFound { node_type }) => {
//!         eprintln!("Missing node: {:?}", node_type);
//!     }
//!     Err(e) => eprintln!("Other error: {}", e),
//! }
//! ```
//!
//! ## Performance Considerations
//!
//! - Use parallel execution for independent processing steps
//! - Consider node registry caching for frequently used workflows
//! - Implement proper error recovery strategies for long-running workflows
//! - Monitor memory usage in workflows with large data contexts
//!
//! ## Thread Safety
//!
//! All core types are designed for concurrent use:
//! - [`Workflow`](workflow::Workflow) instances can be shared across threads
//! - [`NodeRegistry`](nodes::registry::NodeRegistry) uses `RwLock` for safe concurrent access
//! - [`TaskContext`](task::TaskContext) is `Clone` for parallel processing
//!
//! For more detailed information, see the documentation for individual modules.

pub mod ai;
pub mod ai_agents;
pub mod auth;
pub mod error;
pub mod mcp;
pub mod models;
pub mod nodes;
pub mod registry;
pub mod streaming;
pub mod task;
pub mod workflow;
