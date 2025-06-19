//! Enhanced error types with proper chaining and context
//!
//! This module provides improved error types that follow Rust best practices
//! for error handling, including proper error chaining with #[source] attributes
//! and structured error information.

use std::any::TypeId;
use thiserror::Error;

/// Enhanced workflow error type with proper error chaining
#[derive(Debug, Error)]
pub enum EnhancedWorkflowError {
    /// Workflow contains a cycle in its node graph
    #[error("Workflow contains a cycle")]
    CycleDetected {
        /// Optional details about the cycle
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        /// Nodes involved in the cycle
        nodes_in_cycle: Vec<String>,
    },

    /// Some nodes cannot be reached from the start node
    #[error("Unreachable nodes detected: {}", nodes.join(", "))]
    UnreachableNodes {
        /// Names of unreachable nodes
        nodes: Vec<String>,
        /// Starting node used for reachability analysis
        start_node: String,
    },

    /// Node has invalid routing configuration
    #[error("Node '{node}' has {connection_count} connections but is not marked as router")]
    InvalidRouter {
        /// Name of the problematic node
        node: String,
        /// Number of connections
        connection_count: usize,
        /// List of connected nodes
        connected_nodes: Vec<String>,
    },

    /// Node processing failure with context
    #[error("Node '{node_id}' processing failed: {message}")]
    NodeProcessing {
        /// ID of the failing node
        node_id: String,
        /// Descriptive error message
        message: String,
        /// Underlying cause
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        /// Processing context (input data, state, etc.)
        context: std::collections::HashMap<String, serde_json::Value>,
    },

    /// Node type not found in registry
    #[error("Node type not found in registry")]
    NodeNotFound {
        /// TypeId of the missing node
        node_type: TypeId,
        /// Type name if available
        type_name: Option<String>,
        /// Available node types in registry
        available_types: Vec<String>,
    },

    /// Serialization failure with context
    #[error("Failed to serialize {data_type}: {reason}")]
    SerializationError {
        /// Type of data being serialized
        data_type: String,
        /// Reason for failure
        reason: String,
        /// Underlying JSON error
        #[source]
        source: serde_json::Error,
    },

    /// Deserialization failure with context
    #[error("Failed to deserialize {data_type}: {reason}")]
    DeserializationError {
        /// Expected data type
        data_type: String,
        /// Reason for failure
        reason: String,
        /// Underlying JSON error
        #[source]
        source: serde_json::Error,
        /// Sample of the data that failed to parse
        data_sample: Option<String>,
    },

    /// Database operation failure
    #[error("Database operation failed: {operation}")]
    DatabaseError {
        /// Operation that failed (e.g., "insert", "query", "update")
        operation: String,
        /// Table or collection name
        table: Option<String>,
        /// Underlying database error
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Workflow type mismatch
    #[error("Workflow type mismatch: expected '{expected}', got '{actual}'")]
    WorkflowTypeMismatch {
        /// Expected workflow type
        expected: String,
        /// Actual workflow type
        actual: String,
        /// Workflow ID if available
        workflow_id: Option<String>,
    },

    /// External API call failure
    #[error("API call to {service} failed: {message}")]
    ApiError {
        /// Service name (e.g., "OpenAI", "Anthropic")
        service: String,
        /// Error message
        message: String,
        /// HTTP status code if applicable
        status_code: Option<u16>,
        /// Underlying HTTP error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Runtime error with context
    #[error("Runtime error in {component}: {message}")]
    RuntimeError {
        /// Component where error occurred
        component: String,
        /// Error message
        message: String,
        /// Underlying cause
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// MCP communication error
    #[error("MCP error with {server}: {message}")]
    MCPError {
        /// MCP server identifier
        server: String,
        /// Error message
        message: String,
        /// Error category
        category: MCPErrorCategory,
        /// Underlying cause
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Input validation failure
    #[error("Validation failed for {field}: {reason}")]
    ValidationError {
        /// Field that failed validation
        field: String,
        /// Validation failure reason
        reason: String,
        /// Provided value (as string)
        provided_value: Option<String>,
        /// Expected format or constraints
        expected: Option<String>,
    },

    /// Registry operation failure
    #[error("Registry operation '{operation}' failed: {message}")]
    RegistryError {
        /// Operation that failed
        operation: String,
        /// Error message
        message: String,
        /// Registry type (e.g., "node", "workflow", "agent")
        registry_type: String,
        /// Underlying cause
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Invalid workflow step configuration
    #[error("Invalid step type '{step_type}' in workflow '{workflow}'")]
    InvalidStepType {
        /// Invalid step type
        step_type: String,
        /// Workflow name
        workflow: String,
        /// Valid step types
        valid_types: Vec<String>,
    },

    /// Invalid workflow input
    #[error("Invalid input for workflow '{workflow}': {reason}")]
    InvalidInput {
        /// Workflow name
        workflow: String,
        /// Reason for invalidity
        reason: String,
        /// Input schema if available
        expected_schema: Option<serde_json::Value>,
    },

    /// Cross-system communication error
    #[error("Cross-system communication failed between {source_system} and {target_system}")]
    CrossSystemError {
        /// Source system
        source_system: String,
        /// Target system
        target_system: String,
        /// Operation being performed
        operation: String,
        /// Underlying error
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Configuration error
    #[error("Configuration error in {component}: {message}")]
    ConfigurationError {
        /// Component with configuration error
        component: String,
        /// Error message
        message: String,
        /// Configuration key if applicable
        config_key: Option<String>,
        /// Underlying cause
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

/// MCP error categories for better error handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MCPErrorCategory {
    /// Connection establishment or maintenance issues
    Connection,
    /// Protocol-level errors (invalid messages, sequencing)
    Protocol,
    /// Transport layer failures
    Transport,
    /// Authentication or authorization failures
    Auth,
    /// Server-side errors
    Server,
    /// Client-side errors
    Client,
    /// Timeout errors
    Timeout,
}

impl EnhancedWorkflowError {
    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::MCPError { category, .. } => matches!(
                category,
                MCPErrorCategory::Connection | MCPErrorCategory::Timeout | MCPErrorCategory::Transport
            ),
            Self::ApiError { status_code, .. } => {
                // Retry on 5xx errors and specific 4xx errors
                match status_code {
                    Some(code) => *code >= 500 || *code == 429 || *code == 408,
                    None => true, // Network errors without status codes are often retryable
                }
            }
            Self::DatabaseError { .. } => true, // Most DB errors are transient
            Self::RuntimeError { .. } => false, // Runtime errors usually aren't retryable
            _ => false,
        }
    }

    /// Get a unique error code for this error type
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::CycleDetected { .. } => "WF001",
            Self::UnreachableNodes { .. } => "WF002",
            Self::InvalidRouter { .. } => "WF003",
            Self::NodeProcessing { .. } => "ND001",
            Self::NodeNotFound { .. } => "ND002",
            Self::SerializationError { .. } => "SR001",
            Self::DeserializationError { .. } => "SR002",
            Self::DatabaseError { .. } => "DB001",
            Self::WorkflowTypeMismatch { .. } => "WF004",
            Self::ApiError { .. } => "API001",
            Self::RuntimeError { .. } => "RT001",
            Self::MCPError { .. } => "MCP001",
            Self::ValidationError { .. } => "VAL001",
            Self::RegistryError { .. } => "REG001",
            Self::InvalidStepType { .. } => "WF005",
            Self::InvalidInput { .. } => "IN001",
            Self::CrossSystemError { .. } => "XS001",
            Self::ConfigurationError { .. } => "CFG001",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_chaining() {
        let json_err = serde_json::from_str::<String>("invalid").unwrap_err();
        let err = EnhancedWorkflowError::DeserializationError {
            data_type: "UserInput".to_string(),
            reason: "Invalid JSON format".to_string(),
            source: json_err,
            data_sample: Some("invalid".to_string()),
        };

        // Check that error source is properly set
        assert!(err.source().is_some());
        assert_eq!(err.error_code(), "SR002");
    }

    #[test]
    fn test_retryable_errors() {
        let mcp_err = EnhancedWorkflowError::MCPError {
            server: "test-server".to_string(),
            message: "Connection timeout".to_string(),
            category: MCPErrorCategory::Timeout,
            source: None,
        };
        assert!(mcp_err.is_retryable());

        let api_err = EnhancedWorkflowError::ApiError {
            service: "OpenAI".to_string(),
            message: "Service unavailable".to_string(),
            status_code: Some(503),
            source: None,
        };
        assert!(api_err.is_retryable());

        let validation_err = EnhancedWorkflowError::ValidationError {
            field: "email".to_string(),
            reason: "Invalid format".to_string(),
            provided_value: Some("not-an-email".to_string()),
            expected: Some("valid email address".to_string()),
        };
        assert!(!validation_err.is_retryable());
    }
}