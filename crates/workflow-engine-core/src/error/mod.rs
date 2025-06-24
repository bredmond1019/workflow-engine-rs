//! # Comprehensive Error Handling Framework
//!
//! This module provides a production-ready error handling framework with:
//! - Structured error types with categorization
//! - Retry logic with exponential backoff
//! - Circuit breaker pattern for external services
//! - Error context and correlation tracking
//! - Recovery strategies and fallback mechanisms
//!
//! ## Architecture
//!
//! The error handling system is built around these core concepts:
//! 
//! 1. **Error Categorization**: Errors are classified as transient or permanent
//! 2. **Retry Policies**: Configurable retry logic for transient failures
//! 3. **Circuit Breakers**: Prevent cascade failures in distributed systems
//! 4. **Error Context**: Rich context for debugging and monitoring
//! 5. **Recovery Strategies**: Fallback mechanisms for graceful degradation

pub mod types;
pub mod retry;
pub mod circuit_breaker;
pub mod context;
pub mod recovery;

#[cfg(feature = "monitoring")]
pub mod metrics;

// Re-export core types
pub use types::WorkflowError;
pub use retry::{RetryPolicy, RetryableError, retry_with_policy, RetryBuilder};
pub use circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
pub use context::{ErrorContext, ErrorContextExt};
pub use recovery::{RecoveryStrategy, FallbackValue, with_fallback, with_fallback_fn, CacheRecovery};

#[cfg(feature = "monitoring")]
pub use metrics::ErrorMetrics;

use serde::{Serialize, Deserialize};

/// Error severity levels for monitoring and alerting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Informational - no action required
    Info,
    /// Warning - should be investigated
    Warning,
    /// Error - requires attention
    Error,
    /// Critical - immediate action required
    Critical,
}

/// Error categories for classification and handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// Transient errors that may succeed on retry
    Transient,
    /// Permanent errors that won't succeed on retry
    Permanent,
    /// User errors (bad input, validation failures)
    User,
    /// System errors (infrastructure, dependencies)
    System,
    /// Business logic errors
    Business,
}

/// Enhanced error trait with additional context
pub trait ErrorExt: std::error::Error {
    /// Get the error category
    fn category(&self) -> ErrorCategory;
    
    /// Get the error severity
    fn severity(&self) -> ErrorSeverity;
    
    /// Check if the error is retryable
    fn is_retryable(&self) -> bool {
        matches!(self.category(), ErrorCategory::Transient)
    }
    
    /// Get error code for structured logging
    fn error_code(&self) -> &'static str;
    
    /// Get correlation ID if available
    fn correlation_id(&self) -> Option<&str> {
        None
    }
}

/// Error metadata for enhanced tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetadata {
    /// Error category
    pub category: ErrorCategory,
    /// Error severity
    pub severity: ErrorSeverity,
    /// Unique error code
    pub error_code: String,
    /// Correlation ID for request tracing
    pub correlation_id: Option<String>,
    /// Additional context
    pub context: std::collections::HashMap<String, serde_json::Value>,
    /// Timestamp when error occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Number of retry attempts
    pub retry_count: u32,
}

impl ErrorMetadata {
    /// Create new error metadata
    pub fn new(category: ErrorCategory, severity: ErrorSeverity, error_code: String) -> Self {
        Self {
            category,
            severity,
            error_code,
            correlation_id: None,
            context: std::collections::HashMap::new(),
            timestamp: chrono::Utc::now(),
            retry_count: 0,
        }
    }
    
    /// Add context to the error
    pub fn with_context(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.context.insert(key.into(), json_value);
        }
        self
    }
    
    /// Set correlation ID
    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }
}

/// Result type with workflow error
pub type Result<T> = std::result::Result<T, WorkflowError>;

/// Error handler trait for custom error handling strategies
pub trait ErrorHandler: Send + Sync {
    /// Handle an error
    fn handle_error(&self, error: &WorkflowError, metadata: &ErrorMetadata);
    
    /// Check if error should be retried
    fn should_retry(&self, error: &WorkflowError, metadata: &ErrorMetadata) -> bool;
    
    /// Get retry delay for the error
    fn retry_delay(&self, error: &WorkflowError, metadata: &ErrorMetadata) -> Option<std::time::Duration>;
}

/// Default error handler implementation
pub struct DefaultErrorHandler;

impl ErrorHandler for DefaultErrorHandler {
    fn handle_error(&self, error: &WorkflowError, metadata: &ErrorMetadata) {
        tracing::error!(
            error = %error,
            category = ?metadata.category,
            severity = ?metadata.severity,
            error_code = %metadata.error_code,
            correlation_id = ?metadata.correlation_id,
            retry_count = metadata.retry_count,
            "Error occurred"
        );
    }
    
    fn should_retry(&self, _error: &WorkflowError, metadata: &ErrorMetadata) -> bool {
        metadata.category == ErrorCategory::Transient && metadata.retry_count < 3
    }
    
    fn retry_delay(&self, _error: &WorkflowError, metadata: &ErrorMetadata) -> Option<std::time::Duration> {
        if self.should_retry(_error, metadata) {
            // Exponential backoff with jitter
            let base_delay = 100u64 * (1 << metadata.retry_count);
            let jitter = {
                use rand::Rng;
                rand::thread_rng().gen_range(0..50)
            };
            Some(std::time::Duration::from_millis(base_delay + jitter))
        } else {
            None
        }
    }
}

/// Global error handler for the application
static ERROR_HANDLER: std::sync::OnceLock<Box<dyn ErrorHandler>> = std::sync::OnceLock::new();

/// Set the global error handler
pub fn set_error_handler(handler: Box<dyn ErrorHandler>) {
    ERROR_HANDLER.set(handler).ok();
}

/// Get the global error handler
pub fn error_handler() -> &'static dyn ErrorHandler {
    ERROR_HANDLER.get_or_init(|| Box::new(DefaultErrorHandler)).as_ref()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_metadata() {
        let metadata = ErrorMetadata::new(
            ErrorCategory::Transient,
            ErrorSeverity::Warning,
            "TEST_001".to_string()
        )
        .with_context("user_id", "12345")
        .with_correlation_id("req-123");
        
        assert_eq!(metadata.category, ErrorCategory::Transient);
        assert_eq!(metadata.severity, ErrorSeverity::Warning);
        assert_eq!(metadata.error_code, "TEST_001");
        assert_eq!(metadata.correlation_id.as_deref(), Some("req-123"));
        assert!(metadata.context.contains_key("user_id"));
    }
    
    #[test]
    fn test_default_error_handler() {
        let handler = DefaultErrorHandler;
        let error = WorkflowError::processing_error_simple("Test error");
        let metadata = ErrorMetadata::new(
            ErrorCategory::Transient,
            ErrorSeverity::Error,
            "TEST_002".to_string()
        );
        
        assert!(handler.should_retry(&error, &metadata));
        assert!(handler.retry_delay(&error, &metadata).is_some());
    }
}