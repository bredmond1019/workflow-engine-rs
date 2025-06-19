//! # Error Handling for AI Architecture Core
//!
//! This module provides comprehensive error types for all operations in the AI Architecture
//! system. All errors are variants of the main [`WorkflowError`] enum, which provides
//! detailed context for different failure modes.
//!
//! ## Error Categories
//!
//! ### Workflow Structure Errors
//! - [`WorkflowError::CycleDetected`] - Workflow contains circular dependencies
//! - [`WorkflowError::UnreachableNodes`] - Some nodes cannot be reached during execution
//! - [`WorkflowError::InvalidRouter`] - Node has multiple connections but isn't marked as router
//!
//! ### Node Processing Errors
//! - [`WorkflowError::NodeNotFound`] - Referenced node type not registered
//! - [`WorkflowError::ProcessingError`] - General node processing failure
//! - [`WorkflowError::ValidationError`] - Input validation failure
//!
//! ### Data Serialization Errors
//! - [`WorkflowError::SerializationError`] - Failed to serialize data to JSON
//! - [`WorkflowError::DeserializationError`] - Failed to deserialize JSON to type
//!
//! ### External System Errors
//! - [`WorkflowError::DatabaseError`] - Database operation failure
//! - [`WorkflowError::ApiError`] - External API call failure
//! - [`WorkflowError::MCPError`] - Model Context Protocol errors
//!
//! ## Usage Examples
//!
//! ### Basic Error Handling
//!
//! ```rust
//! use ai_architecture_core::{workflow::Workflow, error::WorkflowError};
//! use serde_json::json;
//!
//! fn handle_workflow_execution(workflow: &Workflow) {
//!     match workflow.run(json!({"input": "data"})) {
//!         Ok(result) => {
//!             println!("Workflow completed successfully: {:?}", result);
//!         }
//!         Err(WorkflowError::CycleDetected) => {
//!             eprintln!("Error: Workflow contains a cycle - check node connections");
//!         }
//!         Err(WorkflowError::NodeNotFound { node_type }) => {
//!             eprintln!("Error: Node type {:?} not registered in workflow", node_type);
//!         }
//!         Err(WorkflowError::ProcessingError { message }) => {
//!             eprintln!("Processing error: {}", message);
//!         }
//!         Err(e) => {
//!             eprintln!("Unexpected error: {}", e);
//!         }
//!     }
//! }
//! ```
//!
//! ### Error Propagation
//!
//! ```rust
//! use ai_architecture_core::{
//!     nodes::Node,
//!     task::TaskContext,
//!     error::WorkflowError,
//! };
//!
//! #[derive(Debug)]
//! struct ValidatedProcessingNode;
//!
//! impl Node for ValidatedProcessingNode {
//!     fn process(&self, context: TaskContext) -> Result<TaskContext, WorkflowError> {
//!         // Extract and validate input data
//!         let input: MyInputType = context.get_event_data()
//!             .map_err(|e| WorkflowError::ValidationError {
//!                 message: format!("Invalid input format: {}", e)
//!             })?;
//!
//!         // Validate business rules
//!         if input.value < 0 {
//!             return Err(WorkflowError::ValidationError {
//!                 message: "Value must be non-negative".to_string()
//!             });
//!         }
//!
//!         // Process and return updated context
//!         let mut updated_context = context;
//!         updated_context.update_node("processed_value", input.value * 2);
//!         Ok(updated_context)
//!     }
//! }
//! ```
//!
//! ### MCP Error Handling
//!
//! ```rust
//! use ai_architecture_core::{
//!     mcp::clients::MCPClient,
//!     error::WorkflowError,
//! };
//!
//! async fn safe_mcp_call(client: &MCPClient, tool_name: &str) -> Result<serde_json::Value, WorkflowError> {
//!     client.call_tool(tool_name, serde_json::json!({}))
//!         .await
//!         .map_err(|e| match e {
//!             // Specific MCP error handling
//!             mcp_error if e.to_string().contains("connection") => {
//!                 WorkflowError::MCPConnectionError {
//!                     message: format!("MCP connection failed: {}", e)
//!                 }
//!             }
//!             mcp_error if e.to_string().contains("protocol") => {
//!                 WorkflowError::MCPProtocolError {
//!                     message: format!("MCP protocol error: {}", e)
//!                 }
//!             }
//!             _ => WorkflowError::MCPError {
//!                 message: e.to_string()
//!             }
//!         })
//! }
//! ```
//!
//! ### Database Error Handling
//!
//! ```rust
//! use ai_architecture_core::{
//!     db::event::Event,
//!     error::WorkflowError,
//! };
//! use diesel::prelude::*;
//!
//! fn save_workflow_result(
//!     event: &Event,
//!     conn: &mut PgConnection
//! ) -> Result<(), WorkflowError> {
//!     event.store(conn)
//!         .map_err(|diesel_error| WorkflowError::DatabaseError {
//!             message: format!("Failed to save event: {}", diesel_error)
//!         })?;
//!     Ok(())
//! }
//! ```
//!
//! ## Error Recovery Strategies
//!
//! ### Retry with Backoff
//!
//! ```rust
//! use ai_architecture_core::error::WorkflowError;
//! use tokio::time::{sleep, Duration};
//!
//! async fn retry_with_backoff<F, T>(
//!     mut operation: F,
//!     max_retries: usize,
//! ) -> Result<T, WorkflowError>
//! where
//!     F: FnMut() -> Result<T, WorkflowError>,
//! {
//!     let mut attempt = 0;
//!     loop {
//!         match operation() {
//!             Ok(result) => return Ok(result),
//!             Err(WorkflowError::MCPConnectionError { .. }) | 
//!             Err(WorkflowError::ApiError { .. }) if attempt < max_retries => {
//!                 attempt += 1;
//!                 let delay = Duration::from_millis(100 * (1 << attempt));
//!                 sleep(delay).await;
//!                 continue;
//!             }
//!             Err(e) => return Err(e),
//!         }
//!     }
//! }
//! ```
//!
//! ### Graceful Degradation
//!
//! ```rust
//! use ai_architecture_core::{
//!     nodes::Node,
//!     task::TaskContext,
//!     error::WorkflowError,
//! };
//!
//! #[derive(Debug)]
//! struct ResilientProcessingNode;
//!
//! impl Node for ResilientProcessingNode {
//!     fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
//!         // Try primary processing
//!         match self.primary_processing(&context) {
//!             Ok(result) => {
//!                 context.update_node("result", result);
//!                 context.set_metadata("processing_mode", "primary")?;
//!             }
//!             Err(WorkflowError::ApiError { .. }) => {
//!                 // Fall back to offline processing
//!                 let fallback_result = self.fallback_processing(&context)?;
//!                 context.update_node("result", fallback_result);
//!                 context.set_metadata("processing_mode", "fallback")?;
//!             }
//!             Err(e) => return Err(e), // Don't handle other error types
//!         }
//!         Ok(context)
//!     }
//! }
//! ```
//!
//! ## Best Practices
//!
//! 1. **Use Specific Error Types**: Choose the most specific error variant for better debugging
//! 2. **Include Context**: Provide detailed error messages with relevant context
//! 3. **Handle Recoverable Errors**: Implement retry logic for transient failures
//! 4. **Log Appropriately**: Log errors at appropriate levels (error, warn, debug)
//! 5. **Fail Fast**: For unrecoverable errors, fail quickly rather than continuing
//!
//! ## Error Conversion
//!
//! The [`WorkflowError`] type implements [`From`] for common error types:
//!
//! - `diesel::result::Error` → `WorkflowError::DatabaseError`
//! - `reqwest::Error` → `WorkflowError::ApiError`
//! - `serde_json::Error` → `WorkflowError::SerializationError`
//! - `TransportError` → `WorkflowError::MCPTransportError`

use std::any::TypeId;

/// Primary error type for all AI Architecture Core operations.
///
/// This enum represents all possible error conditions that can occur during
/// workflow execution, node processing, and external system interactions.
/// Each variant provides specific context about the failure mode to aid
/// in debugging and error recovery.
///
/// # Error Categories
///
/// ## Workflow Structure Errors
/// These errors occur during workflow validation or execution:
/// - [`CycleDetected`] - The workflow graph contains a circular dependency
/// - [`UnreachableNodes`] - Some nodes cannot be reached from the start node
/// - [`InvalidRouter`] - A node has multiple connections but isn't marked as a router
///
/// ## Node Processing Errors  
/// These errors occur during individual node execution:
/// - [`NodeNotFound`] - A referenced node type is not registered
/// - [`ProcessingError`] - General processing failure within a node
/// - [`ValidationError`] - Input validation failure
///
/// ## Data Handling Errors
/// These errors occur during data serialization/deserialization:
/// - [`SerializationError`] - Failed to convert data to JSON
/// - [`DeserializationError`] - Failed to convert JSON to expected type
///
/// ## External System Errors
/// These errors occur when interacting with external systems:
/// - [`DatabaseError`] - Database operation failure
/// - [`ApiError`] - External API call failure
/// - [`MCPError`] - General Model Context Protocol error
/// - [`MCPConnectionError`] - MCP connection establishment failure
/// - [`MCPProtocolError`] - MCP protocol violation
/// - [`MCPTransportError`] - MCP transport layer failure
///
/// # Examples
///
/// ```rust
/// use ai_architecture_core::error::WorkflowError;
///
/// // Create custom validation error
/// let validation_error = WorkflowError::ValidationError {
///     message: "Input must be a positive number".to_string()
/// };
///
/// // Handle different error types
/// match some_operation() {
///     Ok(result) => println!("Success: {:?}", result),
///     Err(WorkflowError::CycleDetected) => {
///         eprintln!("Fix workflow structure - cycle detected");
///     }
///     Err(WorkflowError::NodeNotFound { node_type }) => {
///         eprintln!("Register missing node: {:?}", node_type);
///     }
///     Err(e) => eprintln!("Other error: {}", e),
/// }
/// ```
#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    /// Workflow contains a cycle in its node graph.
    ///
    /// This error occurs during workflow validation when the workflow
    /// contains circular dependencies that would cause infinite loops
    /// during execution.
    ///
    /// # Recovery
    /// - Review workflow schema for circular node connections
    /// - Use workflow visualization tools to identify cycles
    /// - Restructure workflow to remove circular dependencies
    #[error("Workflow contains a cycle")]
    CycleDetected,

    /// Some nodes in the workflow cannot be reached from the start node.
    ///
    /// This error indicates that certain nodes are isolated and will
    /// never be executed during workflow processing.
    ///
    /// # Fields
    /// - `nodes` - List of unreachable node names
    ///
    /// # Recovery
    /// - Add connections to reach isolated nodes
    /// - Remove unused nodes from the workflow
    /// - Check that all nodes have proper input connections
    #[error("Unreachable nodes: {nodes:?}")]
    UnreachableNodes { 
        /// Names of nodes that cannot be reached during execution
        nodes: Vec<String> 
    },

    /// Node has multiple outgoing connections but is not marked as a router.
    ///
    /// This error occurs when a node has multiple possible next nodes
    /// but doesn't implement routing logic to choose between them.
    ///
    /// # Fields
    /// - `node` - Name of the problematic node
    ///
    /// # Recovery
    /// - Mark the node as a router in the workflow schema
    /// - Implement routing logic for the node
    /// - Reduce connections to a single output path
    #[error("Node {node} has multiple connections but is not marked as router")]
    InvalidRouter { 
        /// Name of the node with invalid routing configuration
        node: String 
    },

    /// General node processing failure.
    ///
    /// This error represents failures that occur during node execution,
    /// such as business logic errors, invalid state, or processing failures.
    ///
    /// # Fields
    /// - `message` - Detailed error description
    #[error("Node processing error: {message}")]
    ProcessingError { 
        /// Detailed description of the processing failure
        message: String 
    },

    /// Referenced node type is not registered in the workflow.
    ///
    /// This error occurs when the workflow tries to execute a node
    /// that hasn't been registered in the node registry.
    ///
    /// # Fields
    /// - `node_type` - TypeId of the missing node
    ///
    /// # Recovery
    /// - Register the missing node type before workflow execution
    /// - Check that all required nodes are properly initialized
    #[error("Node not found: {node_type:?}")]
    NodeNotFound { 
        /// TypeId of the node that could not be found
        node_type: TypeId 
    },

    /// Failed to serialize data to JSON format.
    ///
    /// This error occurs when converting Rust types to JSON values
    /// for storage or transmission.
    ///
    /// # Fields
    /// - `message` - Serialization error details
    #[error("Serialization error: {message}")]
    SerializationError { 
        /// Details about the serialization failure
        message: String 
    },

    /// Failed to deserialize JSON data to expected type.
    ///
    /// This error occurs when converting JSON values back to
    /// strongly-typed Rust structures.
    ///
    /// # Fields
    /// - `message` - Deserialization error details
    #[error("Deserialization error: {message}")]
    DeserializationError { 
        /// Details about the deserialization failure
        message: String 
    },

    /// Database operation failure.
    ///
    /// This error represents failures in database operations such as
    /// storing events, querying workflow state, or connection issues.
    ///
    /// # Fields
    /// - `message` - Database error details
    #[error("Database error: {message}")]
    DatabaseError { 
        /// Details about the database operation failure
        message: String 
    },

    /// Workflow type mismatch between expected and actual.
    ///
    /// This error occurs when trying to run an event through a workflow
    /// that expects a different workflow type.
    ///
    /// # Fields
    /// - `expected` - Expected workflow type
    /// - `actual` - Actual workflow type provided
    #[error("Workflow type mismatch: expected {expected}, got {actual}")]
    WorkflowTypeMismatch { 
        /// The workflow type that was expected
        expected: String, 
        /// The workflow type that was actually provided
        actual: String 
    },

    /// External API call failure.
    ///
    /// This error represents failures when making HTTP requests to
    /// external services or APIs.
    ///
    /// # Fields
    /// - `message` - API error details
    #[error("API error: {message}")]
    ApiError { 
        /// Details about the API call failure
        message: String 
    },

    /// General runtime error.
    ///
    /// This error represents unexpected runtime failures that don't
    /// fit into other specific categories.
    ///
    /// # Fields
    /// - `message` - Runtime error details
    #[error("Runtime error: {message}")]
    RuntimeError { 
        /// Details about the runtime failure
        message: String 
    },

    /// General Model Context Protocol error.
    ///
    /// This error represents general MCP-related failures that don't
    /// fit into more specific MCP error categories.
    ///
    /// # Fields
    /// - `message` - MCP error details
    #[error("MCP error: {message}")]
    MCPError { 
        /// Details about the MCP operation failure
        message: String 
    },

    /// MCP connection establishment failure.
    ///
    /// This error occurs when failing to establish or maintain
    /// connections to MCP servers.
    ///
    /// # Fields
    /// - `message` - Connection error details
    #[error("MCP connection error: {message}")]
    MCPConnectionError { 
        /// Details about the connection failure
        message: String 
    },

    /// MCP protocol violation or communication error.
    ///
    /// This error occurs when MCP communication doesn't follow
    /// the expected protocol format or sequence.
    ///
    /// # Fields
    /// - `message` - Protocol error details
    #[error("MCP protocol error: {message}")]
    MCPProtocolError { 
        /// Details about the protocol violation
        message: String 
    },

    /// MCP transport layer failure.
    ///
    /// This error occurs in the underlying transport mechanism
    /// for MCP communication (WebSocket, stdio, etc.).
    ///
    /// # Fields
    /// - `message` - Transport error details
    #[error("MCP transport error: {message}")]
    MCPTransportError { 
        /// Details about the transport layer failure
        message: String 
    },

    /// Input validation failure.
    ///
    /// This error occurs when input data doesn't meet validation
    /// requirements or business rules.
    ///
    /// # Fields
    /// - `message` - Validation error details
    #[error("Validation error: {message}")]
    ValidationError { 
        /// Details about what validation rule was violated
        message: String 
    },

    /// Agent registry operation failure.
    ///
    /// This error occurs during agent registration, discovery, or
    /// registry management operations.
    ///
    /// # Fields
    /// - `message` - Registry error details
    #[error("Registry error: {message}")]
    RegistryError { 
        /// Details about the registry operation failure
        message: String 
    },

    /// Invalid workflow step type.
    ///
    /// This error occurs when a workflow step references an unsupported
    /// or incorrectly configured step type.
    ///
    /// # Fields
    /// - `message` - Step type error details
    #[error("Invalid step type: {0}")]
    InvalidStepType(
        /// Details about the invalid step type
        String
    ),

    /// Invalid workflow input.
    ///
    /// This error occurs when workflow input data doesn't match the
    /// expected schema or is missing required fields.
    ///
    /// # Fields
    /// - `message` - Input validation error details
    #[error("Invalid input: {0}")]
    InvalidInput(
        /// Details about the invalid input
        String
    ),

    /// Cross-system communication error.
    ///
    /// This error occurs when cross-system calls fail due to service
    /// discovery issues, network problems, or protocol errors.
    ///
    /// # Fields
    /// - `message` - Cross-system error details
    #[error("Cross-system error: {0}")]
    CrossSystemError(
        /// Details about the cross-system failure
        String
    ),

    /// Configuration error.
    ///
    /// This error occurs when workflow configuration is invalid or
    /// incompatible settings are provided.
    ///
    /// # Fields
    /// - `message` - Configuration error details
    #[error("Configuration error: {0}")]
    ConfigurationError(
        /// Details about the configuration issue
        String
    ),
}

#[cfg(feature = "database")]
impl From<diesel::result::Error> for WorkflowError {
    fn from(error: diesel::result::Error) -> Self {
        WorkflowError::DatabaseError {
            message: error.to_string(),
        }
    }
}

// Add error variants to WorkflowError
impl From<reqwest::Error> for WorkflowError {
    fn from(error: reqwest::Error) -> Self {
        WorkflowError::ApiError {
            message: error.to_string(),
        }
    }
}

// MCP transport error conversion moved to workflow-engine-mcp crate to avoid circular dependency

impl From<serde_json::Error> for WorkflowError {
    fn from(error: serde_json::Error) -> Self {
        WorkflowError::SerializationError {
            message: error.to_string(),
        }
    }
}

#[cfg(feature = "monitoring")]
impl From<prometheus::Error> for WorkflowError {
    fn from(error: prometheus::Error) -> Self {
        WorkflowError::ProcessingError {
            message: format!("Prometheus metrics error: {}", error),
        }
    }
}
