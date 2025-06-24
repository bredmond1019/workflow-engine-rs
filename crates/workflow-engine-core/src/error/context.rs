//! # Error Context Utilities
//!
//! This module provides utilities for adding rich context to errors,
//! including correlation IDs, structured metadata, and error chaining.

use super::{WorkflowError, ErrorCategory, ErrorSeverity, ErrorMetadata};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_json::Value;

/// Error with additional context
#[derive(Debug)]
pub struct ErrorContext {
    /// The underlying error
    pub error: WorkflowError,
    /// Error metadata
    pub metadata: ErrorMetadata,
    /// Error chain (causes)
    pub chain: Vec<String>,
}

impl ErrorContext {
    /// Create new error context
    pub fn new(error: WorkflowError) -> Self {
        let (category, severity, code) = categorize_error(&error);
        Self {
            error,
            metadata: ErrorMetadata::new(category, severity, code),
            chain: Vec::new(),
        }
    }
    
    /// Add context value
    pub fn with_context(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.metadata.context.insert(key.into(), json_value);
        }
        self
    }
    
    /// Set correlation ID
    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.metadata.correlation_id = Some(id.into());
        self
    }
    
    /// Add to error chain
    pub fn with_cause(mut self, cause: impl Into<String>) -> Self {
        self.chain.push(cause.into());
        self
    }
    
    /// Convert to JSON for logging
    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "error": self.error.to_string(),
            "category": self.metadata.category,
            "severity": self.metadata.severity,
            "code": self.metadata.error_code,
            "correlation_id": self.metadata.correlation_id,
            "context": self.metadata.context,
            "chain": self.chain,
            "timestamp": self.metadata.timestamp,
            "retry_count": self.metadata.retry_count,
        })
    }
}

/// Extension trait for adding context to errors
pub trait ErrorContextExt: Sized {
    /// Add context to the error
    fn context(self, key: impl Into<String>, value: impl Serialize) -> ErrorContext;
    
    /// Add correlation ID
    fn with_correlation_id(self, id: impl Into<String>) -> ErrorContext;
    
    /// Add multiple context values
    fn with_contexts(self, contexts: HashMap<String, Value>) -> ErrorContext;
}

impl ErrorContextExt for WorkflowError {
    fn context(self, key: impl Into<String>, value: impl Serialize) -> ErrorContext {
        ErrorContext::new(self).with_context(key, value)
    }
    
    fn with_correlation_id(self, id: impl Into<String>) -> ErrorContext {
        ErrorContext::new(self).with_correlation_id(id)
    }
    
    fn with_contexts(self, contexts: HashMap<String, Value>) -> ErrorContext {
        let mut error_context = ErrorContext::new(self);
        for (key, value) in contexts {
            error_context.metadata.context.insert(key, value);
        }
        error_context
    }
}

/// Categorize error for proper handling
pub fn categorize_error(error: &WorkflowError) -> (ErrorCategory, ErrorSeverity, String) {
    match error {
        // Infrastructure errors - usually transient
        WorkflowError::MCPConnectionError { .. } => (
            ErrorCategory::Transient,
            ErrorSeverity::Error,
            "MCP_CONN_001".to_string()
        ),
        WorkflowError::MCPTransportError { .. } => (
            ErrorCategory::Transient,
            ErrorSeverity::Error,
            "MCP_TRANS_001".to_string()
        ),
        WorkflowError::ApiError { .. } => (
            ErrorCategory::Transient,
            ErrorSeverity::Warning,
            "API_001".to_string()
        ),
        WorkflowError::DatabaseError { .. } => (
            ErrorCategory::Transient,
            ErrorSeverity::Error,
            "DB_001".to_string()
        ),
        
        // Workflow structure errors - permanent
        WorkflowError::CycleDetected => (
            ErrorCategory::Permanent,
            ErrorSeverity::Critical,
            "WF_CYCLE_001".to_string()
        ),
        WorkflowError::UnreachableNodes { .. } => (
            ErrorCategory::Permanent,
            ErrorSeverity::Error,
            "WF_UNREACH_001".to_string()
        ),
        WorkflowError::InvalidRouter { .. } => (
            ErrorCategory::Permanent,
            ErrorSeverity::Error,
            "WF_ROUTER_001".to_string()
        ),
        
        // User errors
        WorkflowError::ValidationError { .. } => (
            ErrorCategory::User,
            ErrorSeverity::Warning,
            "VAL_001".to_string()
        ),
        WorkflowError::InvalidInput { .. } => (
            ErrorCategory::User,
            ErrorSeverity::Warning,
            "INPUT_001".to_string()
        ),
        
        // System errors
        WorkflowError::NodeNotFound { .. } => (
            ErrorCategory::System,
            ErrorSeverity::Error,
            "NODE_404".to_string()
        ),
        WorkflowError::ProcessingError { .. } => (
            ErrorCategory::System,
            ErrorSeverity::Error,
            "PROC_001".to_string()
        ),
        WorkflowError::SerializationError { .. } => (
            ErrorCategory::System,
            ErrorSeverity::Error,
            "SER_001".to_string()
        ),
        WorkflowError::DeserializationError { .. } => (
            ErrorCategory::System,
            ErrorSeverity::Error,
            "DESER_001".to_string()
        ),
        
        // Default for other errors
        _ => (
            ErrorCategory::System,
            ErrorSeverity::Error,
            "UNKNOWN_001".to_string()
        ),
    }
}

/// Error context builder for fluent API
pub struct ErrorContextBuilder {
    error: WorkflowError,
    context: HashMap<String, Value>,
    correlation_id: Option<String>,
    causes: Vec<String>,
}

impl ErrorContextBuilder {
    /// Create new builder
    pub fn new(error: WorkflowError) -> Self {
        Self {
            error,
            context: HashMap::new(),
            correlation_id: None,
            causes: Vec::new(),
        }
    }
    
    /// Add context value
    pub fn context(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.context.insert(key.into(), json_value);
        }
        self
    }
    
    /// Set correlation ID
    pub fn correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }
    
    /// Add cause
    pub fn cause(mut self, cause: impl Into<String>) -> Self {
        self.causes.push(cause.into());
        self
    }
    
    /// Build error context
    pub fn build(self) -> ErrorContext {
        let mut error_context = ErrorContext::new(self.error);
        error_context.metadata.context = self.context;
        error_context.metadata.correlation_id = self.correlation_id;
        error_context.chain = self.causes;
        error_context
    }
}

/// Correlation ID generator
pub struct CorrelationIdGenerator;

impl CorrelationIdGenerator {
    /// Generate a new correlation ID
    pub fn generate() -> String {
        use uuid::Uuid;
        format!("req-{}", Uuid::new_v4())
    }
    
    /// Generate with prefix
    pub fn generate_with_prefix(prefix: &str) -> String {
        use uuid::Uuid;
        format!("{}-{}", prefix, Uuid::new_v4())
    }
}

/// Context provider trait for extracting context from various sources
pub trait ContextProvider {
    /// Extract context into a HashMap
    fn extract_context(&self) -> HashMap<String, Value>;
}

/// Request context for HTTP requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    pub request_id: String,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub path: String,
    pub method: String,
}

impl ContextProvider for RequestContext {
    fn extract_context(&self) -> HashMap<String, Value> {
        let mut context = HashMap::new();
        context.insert("request_id".to_string(), json!(self.request_id));
        context.insert("path".to_string(), json!(self.path));
        context.insert("method".to_string(), json!(self.method));
        
        if let Some(ref user_id) = self.user_id {
            context.insert("user_id".to_string(), json!(user_id));
        }
        if let Some(ref session_id) = self.session_id {
            context.insert("session_id".to_string(), json!(session_id));
        }
        if let Some(ref ip) = self.ip_address {
            context.insert("ip_address".to_string(), json!(ip));
        }
        if let Some(ref ua) = self.user_agent {
            context.insert("user_agent".to_string(), json!(ua));
        }
        
        context
    }
}

/// Macro for adding context to errors easily
#[macro_export]
macro_rules! error_context {
    ($error:expr, $($key:expr => $value:expr),* $(,)?) => {{
        use $crate::core::error::ErrorContextExt;
        let mut ctx = $crate::core::error::ErrorContext::new($error);
        $(
            ctx = ctx.with_context($key, $value);
        )*
        ctx
    }};
}

/// Macro for creating errors with context
#[macro_export]
macro_rules! workflow_error {
    ($variant:ident { $($field:ident: $value:expr),* $(,)? }) => {
        $crate::core::error::WorkflowError::$variant {
            $($field: $value),*
        }
    };
    
    ($variant:ident { $($field:ident: $value:expr),* $(,)? }, context: { $($key:expr => $ctx_value:expr),* $(,)? }) => {{
        use $crate::core::error::ErrorContextExt;
        let error = $crate::core::error::WorkflowError::$variant {
            $($field: $value),*
        };
        let mut ctx = $crate::core::error::ErrorContext::new(error);
        $(
            ctx = ctx.with_context($key, $ctx_value);
        )*
        ctx
    }};
}

// Re-import for macro use
use serde_json::json;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_context_builder() {
        let error = WorkflowError::ProcessingError {
            message: "Test error".to_string(),
        };
        
        let context = ErrorContextBuilder::new(error)
            .context("user_id", "12345")
            .context("operation", "process_data")
            .correlation_id("req-123")
            .cause("Network timeout")
            .build();
        
        assert_eq!(context.metadata.correlation_id.as_deref(), Some("req-123"));
        assert_eq!(context.chain.len(), 1);
        assert_eq!(context.metadata.context.get("user_id"), Some(&json!("12345")));
    }
    
    #[test]
    fn test_error_categorization() {
        let transient_error = WorkflowError::ApiError {
            message: "Service unavailable".to_string(),
        };
        let (category, _, _) = categorize_error(&transient_error);
        assert_eq!(category, ErrorCategory::Transient);
        
        let permanent_error = WorkflowError::CycleDetected;
        let (category, severity, _) = categorize_error(&permanent_error);
        assert_eq!(category, ErrorCategory::Permanent);
        assert_eq!(severity, ErrorSeverity::Critical);
    }
    
    #[test]
    fn test_request_context_provider() {
        let request_ctx = RequestContext {
            request_id: "req-123".to_string(),
            user_id: Some("user-456".to_string()),
            session_id: None,
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: None,
            path: "/api/workflow".to_string(),
            method: "POST".to_string(),
        };
        
        let context = request_ctx.extract_context();
        assert_eq!(context.get("request_id"), Some(&json!("req-123")));
        assert_eq!(context.get("user_id"), Some(&json!("user-456")));
        assert_eq!(context.get("path"), Some(&json!("/api/workflow")));
        assert!(context.get("session_id").is_none());
    }
}