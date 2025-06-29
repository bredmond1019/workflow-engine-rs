//! Boxed error types for reducing WorkflowError size
//! 
//! This module contains boxed versions of large error variants to keep
//! the main WorkflowError enum size small and avoid clippy warnings.

use std::error::Error;
use std::fmt;

/// Boxed details for MCP errors to reduce enum size
#[derive(Debug)]
pub struct MCPErrorDetails {
    pub message: String,
    pub server_name: String,
    pub operation: String,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl fmt::Display for MCPErrorDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MCP error from server '{}' during {}: {}", 
            self.server_name, self.operation, self.message)
    }
}

/// Boxed details for MCP connection errors
#[derive(Debug)]
pub struct MCPConnectionErrorDetails {
    pub message: String,
    pub server_name: String,
    pub transport_type: String,
    pub endpoint: String,
    pub retry_count: u32,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl fmt::Display for MCPConnectionErrorDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MCP connection error to server '{}' via {} at '{}' (retries: {}): {}", 
            self.server_name, self.transport_type, self.endpoint, self.retry_count, self.message)
    }
}

/// Boxed details for MCP protocol errors
#[derive(Debug)]
pub struct MCPProtocolErrorDetails {
    pub message: String,
    pub server_name: String,
    pub expected: String,
    pub received: String,
    pub message_type: String,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl fmt::Display for MCPProtocolErrorDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MCP protocol error from server '{}' for {}: {}. Expected: {}, Received: {}", 
            self.server_name, self.message_type, self.message, self.expected, self.received)
    }
}

/// Boxed details for MCP transport errors
#[derive(Debug)]
pub struct MCPTransportErrorDetails {
    pub message: String,
    pub server_name: String,
    pub transport_type: String,
    pub operation: String,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl fmt::Display for MCPTransportErrorDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MCP transport error ({}) from server '{}' during {}: {}", 
            self.transport_type, self.server_name, self.operation, self.message)
    }
}

/// Boxed details for validation errors
#[derive(Debug)]
pub struct ValidationErrorDetails {
    pub message: String,
    pub field: String,
    pub value: Option<String>,
    pub constraint: String,
    pub context: String,
}

impl fmt::Display for ValidationErrorDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation error for field '{}' {}: {}. Constraint: {}", 
            self.field, self.context, self.message, self.constraint)
    }
}

/// Boxed details for registry errors
#[derive(Debug)]
pub struct RegistryErrorDetails {
    pub message: String,
    pub operation: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl fmt::Display for RegistryErrorDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Registry error during {} for {}{}: {}", 
            self.operation, 
            self.resource_type,
            self.resource_id.as_ref().map(|id| format!(" '{}'", id)).unwrap_or_default(),
            self.message)
    }
}

/// Boxed details for invalid step type errors
#[derive(Debug)]
pub struct InvalidStepTypeDetails {
    pub step_type: String,
    pub workflow_id: String,
    pub step_index: usize,
    pub available_types: Vec<String>,
}

impl fmt::Display for InvalidStepTypeDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid step type '{}' at index {} in workflow '{}'. Available types: {}", 
            self.step_type, self.step_index, self.workflow_id, self.available_types.join(", "))
    }
}

/// Boxed details for invalid input errors
#[derive(Debug)]
pub struct InvalidInputDetails {
    pub message: String,
    pub workflow_id: String,
    pub expected_schema: String,
    pub received_fields: Vec<String>,
    pub missing_fields: Vec<String>,
}

impl fmt::Display for InvalidInputDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid input for workflow '{}': {}. Expected schema: {}. Missing fields: {}", 
            self.workflow_id, self.message, self.expected_schema, self.missing_fields.join(", "))
    }
}

/// Boxed details for cross-system errors
#[derive(Debug)]
pub struct CrossSystemErrorDetails {
    pub message: String,
    pub source_service: String,
    pub target_service: String,
    pub operation: String,
    pub retry_count: u32,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl fmt::Display for CrossSystemErrorDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cross-system error from {} to {} during {} (retries: {}): {}", 
            self.source_service, self.target_service, self.operation, self.retry_count, self.message)
    }
}

/// Boxed details for configuration errors
#[derive(Debug)]
pub struct ConfigurationErrorDetails {
    pub message: String,
    pub config_key: String,
    pub config_source: String,
    pub expected_format: String,
    pub received_value: Option<String>,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl fmt::Display for ConfigurationErrorDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Configuration error for key '{}' from {}: {}. Expected: {}, Received: {}", 
            self.config_key, 
            self.config_source, 
            self.message, 
            self.expected_format,
            self.received_value.as_deref().unwrap_or("<none>"))
    }
}

/// Boxed details for database errors
#[derive(Debug)]
pub struct DatabaseErrorDetails {
    pub message: String,
    pub operation: String,
    pub table: Option<String>,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl fmt::Display for DatabaseErrorDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Database error during {} operation{}: {}", 
            self.operation,
            self.table.as_ref().map(|t| format!(" on table '{}'", t)).unwrap_or_default(),
            self.message)
    }
}

/// Boxed details for API errors
#[derive(Debug)]
pub struct ApiErrorDetails {
    pub message: String,
    pub status_code: Option<u16>,
    pub endpoint: String,
    pub service: String,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl fmt::Display for ApiErrorDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "API error from service '{}' at endpoint '{}'{}: {}", 
            self.service,
            self.endpoint,
            self.status_code.map(|c| format!(" (status: {})", c)).unwrap_or_default(),
            self.message)
    }
}

/// Boxed details for processing errors
#[derive(Debug)]
pub struct ProcessingErrorDetails {
    pub message: String,
    pub node_id: Option<String>,
    pub node_type: String,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl fmt::Display for ProcessingErrorDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Node processing error in {}{}: {}", 
            self.node_type,
            self.node_id.as_ref().map(|id| format!(" (ID: {})", id)).unwrap_or_default(),
            self.message)
    }
}

/// Boxed details for serialization errors
#[derive(Debug)]
pub struct SerializationErrorDetails {
    pub message: String,
    pub type_name: String,
    pub context: String,
    pub source: Option<serde_json::Error>,
}

impl fmt::Display for SerializationErrorDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Serialization error for type '{}' {}: {}", 
            self.type_name, self.context, self.message)
    }
}

/// Boxed details for deserialization errors
#[derive(Debug)]
pub struct DeserializationErrorDetails {
    pub message: String,
    pub expected_type: String,
    pub context: String,
    pub raw_data: Option<String>,
    pub source: Option<serde_json::Error>,
}

impl fmt::Display for DeserializationErrorDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Deserialization error to type '{}' {}: {}", 
            self.expected_type, self.context, self.message)
    }
}