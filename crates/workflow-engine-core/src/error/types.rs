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
    /// - `node_id` - Identifier of the node that failed
    /// - `node_type` - Type name of the failed node
    /// - `source` - Underlying error that caused the failure
    #[error("Node processing error in {node_type}{}: {message}", node_id.as_ref().map(|id| format!(" (ID: {})", id)).unwrap_or_default())]
    ProcessingError { 
        /// Detailed description of the processing failure
        message: String,
        /// Node identifier for debugging
        node_id: Option<String>,
        /// Node type name for context
        node_type: String,
        /// Underlying error that caused this failure
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
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
    /// - `type_name` - Name of the type being serialized
    /// - `context` - Context where serialization failed
    /// - `source` - Underlying serde error
    #[error("Serialization error for type '{type_name}' {context}: {message}")]
    SerializationError { 
        /// Details about the serialization failure
        message: String,
        /// Type name being serialized
        type_name: String,
        /// Context information (e.g., "during workflow save", "in API response")
        context: String,
        /// Underlying serde_json::Error
        #[source]
        source: Option<serde_json::Error>,
    },

    /// Failed to deserialize JSON data to expected type.
    ///
    /// This error occurs when converting JSON values back to
    /// strongly-typed Rust structures.
    ///
    /// # Fields
    /// - `message` - Deserialization error details
    /// - `expected_type` - The type we tried to deserialize to
    /// - `context` - Context where deserialization failed
    /// - `raw_data` - Raw JSON data that failed to deserialize (truncated for large data)
    /// - `source` - Underlying serde error
    #[error("Deserialization error to type '{expected_type}' {context}: {message}")]
    DeserializationError { 
        /// Details about the deserialization failure
        message: String,
        /// Expected target type name
        expected_type: String,
        /// Context information (e.g., "from API response", "from database")
        context: String,
        /// Raw JSON data (truncated if too long)
        raw_data: Option<String>,
        /// Underlying serde_json::Error
        #[source]
        source: Option<serde_json::Error>,
    },

    /// Database operation failure.
    ///
    /// This error represents failures in database operations such as
    /// storing events, querying workflow state, or connection issues.
    ///
    /// # Fields
    /// - `message` - Database error details
    /// - `operation` - The database operation that failed
    /// - `table` - Database table involved (if applicable)
    /// - `source` - Underlying database error
    #[error("Database error during {operation}{}: {message}", table.as_ref().map(|t| format!(" on table '{}'", t)).unwrap_or_default())]
    DatabaseError { 
        /// Details about the database operation failure
        message: String,
        /// Database operation type (e.g., "SELECT", "INSERT", "connection")
        operation: String,
        /// Table name if applicable
        table: Option<String>,
        /// Underlying database error (Diesel, SQLx, etc.)
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
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
    /// - `service` - Name of the external service
    /// - `endpoint` - API endpoint that failed
    /// - `status_code` - HTTP status code if available
    /// - `retry_count` - Number of retries attempted
    /// - `source` - Underlying HTTP client error
    #[error("API error from {service} at {endpoint}{}: {message}", status_code.map(|c| format!(" (status {})", c)).unwrap_or_default())]
    ApiError { 
        /// Details about the API call failure
        message: String,
        /// Service name (e.g., "OpenAI", "Anthropic", "HelpScout")
        service: String,
        /// API endpoint path
        endpoint: String,
        /// HTTP status code if available
        status_code: Option<u16>,
        /// Number of retry attempts made
        retry_count: u32,
        /// Underlying HTTP error (reqwest, etc.)
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
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
    /// - `server_name` - Name of the MCP server
    /// - `operation` - MCP operation that failed
    /// - `source` - Underlying MCP error
    #[error("MCP error from server '{server_name}' during {operation}: {message}")]
    MCPError { 
        /// Details about the MCP operation failure
        message: String,
        /// Name of the MCP server
        server_name: String,
        /// MCP operation (e.g., "tool_call", "list_tools", "connect")
        operation: String,
        /// Underlying MCP-specific error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// MCP connection establishment failure.
    ///
    /// This error occurs when failing to establish or maintain
    /// connections to MCP servers.
    ///
    /// # Fields
    /// - `message` - Connection error details
    /// - `server_name` - Name of the MCP server
    /// - `transport_type` - Type of transport used (WebSocket, stdio, etc.)
    /// - `endpoint` - Connection endpoint or command
    /// - `retry_count` - Number of connection retries attempted
    /// - `source` - Underlying connection error
    #[error("MCP connection error to server '{server_name}' via {transport_type} at '{endpoint}' (retries: {retry_count}): {message}")]
    MCPConnectionError { 
        /// Details about the connection failure
        message: String,
        /// Name of the MCP server
        server_name: String,
        /// Transport type (WebSocket, stdio, HTTP)
        transport_type: String,
        /// Connection endpoint URL or command
        endpoint: String,
        /// Number of retry attempts made
        retry_count: u32,
        /// Underlying transport error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// MCP protocol violation or communication error.
    ///
    /// This error occurs when MCP communication doesn't follow
    /// the expected protocol format or sequence.
    ///
    /// # Fields
    /// - `message` - Protocol error details
    /// - `server_name` - Name of the MCP server
    /// - `expected` - What was expected in the protocol
    /// - `received` - What was actually received
    /// - `message_type` - Type of MCP message being processed
    /// - `source` - Underlying protocol error
    #[error("MCP protocol error from server '{server_name}' for {message_type}: {message}. Expected: {expected}, Received: {received}")]
    MCPProtocolError { 
        /// Details about the protocol violation
        message: String,
        /// Name of the MCP server
        server_name: String,
        /// What was expected according to protocol
        expected: String,
        /// What was actually received
        received: String,
        /// MCP message type (request, response, notification)
        message_type: String,
        /// Underlying protocol parsing error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// MCP transport layer failure.
    ///
    /// This error occurs in the underlying transport mechanism
    /// for MCP communication (WebSocket, stdio, etc.).
    ///
    /// # Fields
    /// - `message` - Transport error details
    /// - `server_name` - Name of the MCP server
    /// - `transport_type` - Type of transport that failed
    /// - `operation` - Transport operation that failed
    /// - `source` - Underlying transport error
    #[error("MCP transport error ({transport_type}) from server '{server_name}' during {operation}: {message}")]
    MCPTransportError { 
        /// Details about the transport layer failure
        message: String,
        /// Name of the MCP server
        server_name: String,
        /// Transport type (WebSocket, stdio, HTTP)
        transport_type: String,
        /// Transport operation (send, receive, connect, disconnect)
        operation: String,
        /// Underlying transport layer error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Input validation failure.
    ///
    /// This error occurs when input data doesn't meet validation
    /// requirements or business rules.
    ///
    /// # Fields
    /// - `message` - Validation error details
    /// - `field` - Field name that failed validation
    /// - `value` - Value that failed validation (sanitized)
    /// - `constraint` - Validation constraint that was violated
    /// - `context` - Context where validation occurred
    #[error("Validation error for field '{field}' {context}: {message}. Constraint: {constraint}")]
    ValidationError { 
        /// Details about what validation rule was violated
        message: String,
        /// Field name that failed validation
        field: String,
        /// Value that failed validation (potentially sanitized for security)
        value: Option<String>,
        /// Validation constraint description
        constraint: String,
        /// Context where validation occurred (e.g., "in workflow input", "during node processing")
        context: String,
    },

    /// Agent registry operation failure.
    ///
    /// This error occurs during agent registration, discovery, or
    /// registry management operations.
    ///
    /// # Fields
    /// - `message` - Registry error details
    /// - `operation` - Registry operation that failed
    /// - `resource_type` - Type of resource (agent, node, workflow)
    /// - `resource_id` - ID of the resource if applicable
    /// - `source` - Underlying registry error
    #[error("Registry error during {operation} for {resource_type}{}: {message}", resource_id.as_ref().map(|id| format!(" '{}'", id)).unwrap_or_default())]
    RegistryError { 
        /// Details about the registry operation failure
        message: String,
        /// Registry operation (register, unregister, lookup, list)
        operation: String,
        /// Type of resource being operated on
        resource_type: String,
        /// Resource identifier if applicable
        resource_id: Option<String>,
        /// Underlying registry-specific error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Invalid workflow step type.
    ///
    /// This error occurs when a workflow step references an unsupported
    /// or incorrectly configured step type.
    ///
    /// # Fields
    /// - `step_type` - The invalid step type name
    /// - `workflow_id` - ID of the workflow containing the invalid step
    /// - `step_index` - Index of the step in the workflow
    /// - `available_types` - List of available step types
    #[error("Invalid step type '{step_type}' at index {step_index} in workflow '{workflow_id}'. Available types: {}", available_types.join(", "))]
    InvalidStepType {
        /// Invalid step type name
        step_type: String,
        /// Workflow identifier
        workflow_id: String,
        /// Step index in workflow
        step_index: usize,
        /// Available step types for reference
        available_types: Vec<String>,
    },

    /// Invalid workflow input.
    ///
    /// This error occurs when workflow input data doesn't match the
    /// expected schema or is missing required fields.
    ///
    /// # Fields
    /// - `message` - Input validation error details
    /// - `workflow_id` - ID of the workflow receiving invalid input
    /// - `expected_schema` - Expected input schema description
    /// - `received_fields` - Fields that were actually received
    /// - `missing_fields` - Required fields that are missing
    #[error("Invalid input for workflow '{workflow_id}': {message}. Expected schema: {expected_schema}. Missing fields: {}", missing_fields.join(", "))]
    InvalidInput {
        /// Details about the invalid input
        message: String,
        /// Workflow identifier
        workflow_id: String,
        /// Description of expected schema
        expected_schema: String,
        /// Fields that were received
        received_fields: Vec<String>,
        /// Required fields that are missing
        missing_fields: Vec<String>,
    },

    /// Cross-system communication error.
    ///
    /// This error occurs when cross-system calls fail due to service
    /// discovery issues, network problems, or protocol errors.
    ///
    /// # Fields
    /// - `message` - Cross-system error details
    /// - `source_service` - Service making the call
    /// - `target_service` - Service being called
    /// - `operation` - Operation being performed
    /// - `retry_count` - Number of retries attempted
    /// - `source` - Underlying communication error
    #[error("Cross-system error from {source_service} to {target_service} during {operation} (retries: {retry_count}): {message}")]
    CrossSystemError {
        /// Details about the cross-system failure
        message: String,
        /// Service making the call
        source_service: String,
        /// Service being called
        target_service: String,
        /// Operation being performed
        operation: String,
        /// Number of retry attempts made
        retry_count: u32,
        /// Underlying communication error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Configuration error.
    ///
    /// This error occurs when workflow configuration is invalid or
    /// incompatible settings are provided.
    ///
    /// # Fields
    /// - `message` - Configuration error details
    /// - `config_key` - Configuration key that has the issue
    /// - `config_source` - Source of the configuration (file, env, default)
    /// - `expected_format` - Expected format or value range
    /// - `received_value` - Value that was received (sanitized)
    /// - `source` - Underlying configuration parsing error
    #[error("Configuration error for key '{config_key}' from {config_source}: {message}. Expected: {expected_format}, Received: {}", received_value.as_deref().unwrap_or("<none>"))]
    ConfigurationError {
        /// Details about the configuration issue
        message: String,
        /// Configuration key that has the issue
        config_key: String,
        /// Source of the configuration (environment, file, command-line, default)
        config_source: String,
        /// Expected format or value description
        expected_format: String,
        /// Value that was received (potentially sanitized for security)
        received_value: Option<String>,
        /// Underlying configuration parsing error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

#[cfg(feature = "database")]
impl From<diesel::result::Error> for WorkflowError {
    fn from(error: diesel::result::Error) -> Self {
        let (operation, table) = match &error {
            diesel::result::Error::NotFound => ("SELECT".to_string(), None),
            diesel::result::Error::InvalidCString(_) => ("connection".to_string(), None),
            _ => ("unknown".to_string(), None),
        };
        
        WorkflowError::DatabaseError {
            message: format!("Diesel error: {}", error),
            operation,
            table,
            source: Some(Box::new(error)),
        }
    }
}

// Add error variants to WorkflowError
impl From<reqwest::Error> for WorkflowError {
    fn from(error: reqwest::Error) -> Self {
        let status_code = error.status().map(|s| s.as_u16());
        let endpoint = error.url().map(|u| u.to_string()).unwrap_or_else(|| "unknown".to_string());
        let service = "external_api".to_string(); // Default service name
        
        WorkflowError::ApiError {
            message: format!("HTTP request failed: {}", error),
            service,
            endpoint,
            status_code,
            retry_count: 0,
            source: Some(Box::new(error)),
        }
    }
}

// MCP transport error conversion moved to workflow-engine-mcp crate to avoid circular dependency

impl From<serde_json::Error> for WorkflowError {
    fn from(error: serde_json::Error) -> Self {
        // Determine if this is serialization or deserialization based on error message
        let error_msg = error.to_string();
        let is_deserialization = error_msg.contains("missing field") || 
                               error_msg.contains("invalid type") ||
                               error_msg.contains("expected");
        
        if is_deserialization {
            WorkflowError::DeserializationError {
                message: format!("JSON deserialization failed: {}", error),
                expected_type: "unknown".to_string(),
                context: "during JSON parsing".to_string(),
                raw_data: None, // Raw data not available from serde_json::Error
                source: Some(error),
            }
        } else {
            WorkflowError::SerializationError {
                message: format!("JSON serialization failed: {}", error),
                type_name: "unknown".to_string(),
                context: "during JSON encoding".to_string(),
                source: Some(error),
            }
        }
    }
}

#[cfg(feature = "monitoring")]
impl From<prometheus::Error> for WorkflowError {
    fn from(error: prometheus::Error) -> Self {
        WorkflowError::ProcessingError {
            message: format!("Prometheus metrics error: {}", error),
            node_id: None,
            node_type: "metrics_collector".to_string(),
            source: Some(Box::new(error)),
        }
    }
}

impl WorkflowError {
    /// Create a processing error with basic information
    pub fn processing_error(message: impl Into<String>, node_type: impl Into<String>) -> Self {
        Self::ProcessingError {
            message: message.into(),
            node_id: None,
            node_type: node_type.into(),
            source: None,
        }
    }

    /// Create a processing error with full context
    pub fn processing_error_with_context(
        message: impl Into<String>,
        node_type: impl Into<String>,
        node_id: Option<String>,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self::ProcessingError {
            message: message.into(),
            node_id,
            node_type: node_type.into(),
            source,
        }
    }

    /// Create a validation error with context
    pub fn validation_error(
        message: impl Into<String>,
        field: impl Into<String>,
        constraint: impl Into<String>,
        context: impl Into<String>,
    ) -> Self {
        Self::ValidationError {
            message: message.into(),
            field: field.into(),
            value: None,
            constraint: constraint.into(),
            context: context.into(),
        }
    }

    /// Create a validation error with the failing value
    pub fn validation_error_with_value(
        message: impl Into<String>,
        field: impl Into<String>,
        value: Option<String>,
        constraint: impl Into<String>,
        context: impl Into<String>,
    ) -> Self {
        Self::ValidationError {
            message: message.into(),
            field: field.into(),
            value,
            constraint: constraint.into(),
            context: context.into(),
        }
    }

    /// Create a database error with operation context
    pub fn database_error(
        message: impl Into<String>,
        operation: impl Into<String>,
        table: Option<String>,
    ) -> Self {
        Self::DatabaseError {
            message: message.into(),
            operation: operation.into(),
            table,
            source: None,
        }
    }

    /// Create an API error with full context
    pub fn api_error(
        message: impl Into<String>,
        service: impl Into<String>,
        endpoint: impl Into<String>,
        status_code: Option<u16>,
    ) -> Self {
        Self::ApiError {
            message: message.into(),
            service: service.into(),
            endpoint: endpoint.into(),
            status_code,
            retry_count: 0,
            source: None,
        }
    }

    /// Create an MCP connection error
    pub fn mcp_connection_error(
        message: impl Into<String>,
        server_name: impl Into<String>,
        transport_type: impl Into<String>,
        endpoint: impl Into<String>,
    ) -> Self {
        Self::MCPConnectionError {
            message: message.into(),
            server_name: server_name.into(),
            transport_type: transport_type.into(),
            endpoint: endpoint.into(),
            retry_count: 0,
            source: None,
        }
    }

    /// Create an MCP protocol error
    pub fn mcp_protocol_error(
        message: impl Into<String>,
        server_name: impl Into<String>,
        expected: impl Into<String>,
        received: impl Into<String>,
        message_type: impl Into<String>,
    ) -> Self {
        Self::MCPProtocolError {
            message: message.into(),
            server_name: server_name.into(),
            expected: expected.into(),
            received: received.into(),
            message_type: message_type.into(),
            source: None,
        }
    }

    /// Create a serialization error with type context
    pub fn serialization_error(
        message: impl Into<String>,
        type_name: impl Into<String>,
        context: impl Into<String>,
    ) -> Self {
        Self::SerializationError {
            message: message.into(),
            type_name: type_name.into(),
            context: context.into(),
            source: None,
        }
    }

    /// Create a deserialization error with type context
    pub fn deserialization_error(
        message: impl Into<String>,
        expected_type: impl Into<String>,
        context: impl Into<String>,
        raw_data: Option<String>,
    ) -> Self {
        Self::DeserializationError {
            message: message.into(),
            expected_type: expected_type.into(),
            context: context.into(),
            raw_data,
            source: None,
        }
    }

    /// Create a configuration error
    pub fn configuration_error(
        message: impl Into<String>,
        config_key: impl Into<String>,
        config_source: impl Into<String>,
        expected_format: impl Into<String>,
        received_value: Option<String>,
    ) -> Self {
        Self::ConfigurationError {
            message: message.into(),
            config_key: config_key.into(),
            config_source: config_source.into(),
            expected_format: expected_format.into(),
            received_value,
            source: None,
        }
    }

    /// Create a registry error
    pub fn registry_error(
        message: impl Into<String>,
        operation: impl Into<String>,
        resource_type: impl Into<String>,
        resource_id: Option<String>,
    ) -> Self {
        Self::RegistryError {
            message: message.into(),
            operation: operation.into(),
            resource_type: resource_type.into(),
            resource_id,
            source: None,
        }
    }

    /// Create a cross-system error
    pub fn cross_system_error(
        message: impl Into<String>,
        source_service: impl Into<String>,
        target_service: impl Into<String>,
        operation: impl Into<String>,
    ) -> Self {
        Self::CrossSystemError {
            message: message.into(),
            source_service: source_service.into(),
            target_service: target_service.into(),
            operation: operation.into(),
            retry_count: 0,
            source: None,
        }
    }
}

impl super::ErrorExt for WorkflowError {
    fn category(&self) -> super::ErrorCategory {
        use super::ErrorCategory;
        match self {
            // Transient errors that may succeed on retry
            Self::MCPConnectionError { .. } | 
            Self::MCPTransportError { .. } |
            Self::ApiError { .. } => {
                ErrorCategory::Transient
            }
            Self::DatabaseError { operation, .. } if operation.contains("connection") => {
                ErrorCategory::Transient
            }
            
            // Permanent errors that won't succeed on retry
            Self::CycleDetected |
            Self::UnreachableNodes { .. } |
            Self::InvalidRouter { .. } |
            Self::NodeNotFound { .. } |
            Self::WorkflowTypeMismatch { .. } |
            Self::InvalidStepType { .. } |
            Self::InvalidInput { .. } => {
                ErrorCategory::Permanent
            }
            
            // User errors (bad input, validation failures)
            Self::ValidationError { .. } |
            Self::DeserializationError { .. } |
            Self::ConfigurationError { .. } => {
                ErrorCategory::User
            }
            
            // System errors (infrastructure, dependencies)
            Self::DatabaseError { .. } |
            Self::SerializationError { .. } |
            Self::RuntimeError { .. } |
            Self::CrossSystemError { .. } => {
                ErrorCategory::System
            }
            
            // Business logic errors
            Self::ProcessingError { .. } |
            Self::RegistryError { .. } |
            Self::MCPError { .. } |
            Self::MCPProtocolError { .. } => {
                ErrorCategory::Business
            }
        }
    }
    
    fn severity(&self) -> super::ErrorSeverity {
        use super::ErrorSeverity;
        match self {
            // Critical - immediate action required
            Self::CycleDetected => {
                ErrorSeverity::Critical
            }
            Self::DatabaseError { operation, .. } if operation.contains("connection") => {
                ErrorSeverity::Critical
            }
            
            // Error - requires attention
            Self::ProcessingError { .. } |
            Self::NodeNotFound { .. } |
            Self::RegistryError { .. } |
            Self::MCPConnectionError { .. } |
            Self::CrossSystemError { .. } |
            Self::RuntimeError { .. } => {
                ErrorSeverity::Error
            }
            
            // Warning - should be investigated
            Self::UnreachableNodes { .. } |
            Self::InvalidRouter { .. } |
            Self::MCPError { .. } |
            Self::MCPProtocolError { .. } |
            Self::MCPTransportError { .. } |
            Self::ApiError { .. } |
            Self::SerializationError { .. } |
            Self::DatabaseError { .. } => {
                ErrorSeverity::Warning
            }
            
            // Info - validation and user input errors
            Self::ValidationError { .. } |
            Self::DeserializationError { .. } |
            Self::WorkflowTypeMismatch { .. } |
            Self::InvalidStepType { .. } |
            Self::InvalidInput { .. } |
            Self::ConfigurationError { .. } => {
                ErrorSeverity::Info
            }
        }
    }
    
    fn error_code(&self) -> &'static str {
        match self {
            Self::CycleDetected => "WF_CYCLE_DETECTED",
            Self::UnreachableNodes { .. } => "WF_UNREACHABLE_NODES",
            Self::InvalidRouter { .. } => "WF_INVALID_ROUTER",
            Self::ProcessingError { .. } => "WF_PROCESSING_ERROR",
            Self::NodeNotFound { .. } => "WF_NODE_NOT_FOUND",
            Self::SerializationError { .. } => "WF_SERIALIZATION_ERROR",
            Self::DeserializationError { .. } => "WF_DESERIALIZATION_ERROR",
            Self::DatabaseError { .. } => "WF_DATABASE_ERROR",
            Self::WorkflowTypeMismatch { .. } => "WF_TYPE_MISMATCH",
            Self::ApiError { .. } => "WF_API_ERROR",
            Self::RuntimeError { .. } => "WF_RUNTIME_ERROR",
            Self::MCPError { .. } => "WF_MCP_ERROR",
            Self::MCPConnectionError { .. } => "WF_MCP_CONNECTION_ERROR",
            Self::MCPProtocolError { .. } => "WF_MCP_PROTOCOL_ERROR",
            Self::MCPTransportError { .. } => "WF_MCP_TRANSPORT_ERROR",
            Self::ValidationError { .. } => "WF_VALIDATION_ERROR",
            Self::RegistryError { .. } => "WF_REGISTRY_ERROR",
            Self::InvalidStepType { .. } => "WF_INVALID_STEP_TYPE",
            Self::InvalidInput { .. } => "WF_INVALID_INPUT",
            Self::CrossSystemError { .. } => "WF_CROSS_SYSTEM_ERROR",
            Self::ConfigurationError { .. } => "WF_CONFIGURATION_ERROR",
        }
    }
}
