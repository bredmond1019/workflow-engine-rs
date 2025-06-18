//! Comprehensive error handling for the Knowledge Graph service
//! 
//! Provides custom error types with rich context, graceful degradation,
//! circuit breaker patterns, and retry logic.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Main error type for the Knowledge Graph service
#[derive(Error, Debug, Clone)]
pub enum KnowledgeGraphError {
    /// Parsing errors with detailed context
    #[error("Parsing failed: {message}")]
    ParseError {
        message: String,
        field: Option<String>,
        raw_data: Option<String>,
        source_error: Option<String>,
    },

    /// Network-related errors
    #[error("Network error: {message}")]
    NetworkError {
        message: String,
        endpoint: String,
        retry_count: usize,
        source_error: Option<String>,
    },

    /// Timeout errors with operation context
    #[error("Operation timed out after {duration:?}: {operation}")]
    TimeoutError {
        operation: String,
        duration: Duration,
        endpoint: String,
    },

    /// Connection pool errors
    #[error("Connection pool error: {message}")]
    ConnectionPoolError {
        message: String,
        available_connections: usize,
        max_connections: usize,
        source_error: Option<String>,
    },

    /// Circuit breaker open error
    #[error("Circuit breaker is open for {service}. Last failure: {last_failure_message}")]
    CircuitBreakerOpen {
        service: String,
        failure_count: usize,
        last_failure_message: String,
        next_retry_at: Instant,
    },

    /// GraphQL-specific errors
    #[error("GraphQL error: {message}")]
    GraphQLError {
        message: String,
        errors: Vec<GraphQLErrorDetail>,
        query: Option<String>,
    },

    /// Database transaction errors
    #[error("Transaction failed: {message}")]
    TransactionError {
        message: String,
        operation_type: String,
        affected_ids: Vec<Uuid>,
        source_error: Option<String>,
    },

    /// Validation errors
    #[error("Validation failed: {message}")]
    ValidationError {
        message: String,
        field: String,
        value: Option<String>,
        constraints: Vec<String>,
    },

    /// Partial result error (for graceful degradation)
    #[error("Partial results available: {message}")]
    PartialResultError {
        message: String,
        successful_operations: usize,
        failed_operations: usize,
        partial_data: Option<serde_json::Value>,
        failures: Vec<OperationFailure>,
    },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    ConfigurationError {
        message: String,
        parameter: String,
        expected: String,
        actual: Option<String>,
    },

    /// Generic internal error
    #[error("Internal error: {message}")]
    InternalError {
        message: String,
        source_error: Option<String>,
    },
}

/// Detailed GraphQL error information
#[derive(Debug, Clone)]
pub struct GraphQLErrorDetail {
    pub message: String,
    pub path: Option<Vec<String>>,
    pub extensions: Option<HashMap<String, serde_json::Value>>,
}

/// Information about a failed operation in partial results
#[derive(Debug, Clone)]
pub struct OperationFailure {
    pub operation_id: String,
    pub error_message: String,
    pub error_type: String,
    pub timestamp: Instant,
}

/// Result type alias for Knowledge Graph operations
pub type Result<T> = std::result::Result<T, KnowledgeGraphError>;

/// Error context builder for adding rich context to errors
#[derive(Debug, Clone)]
pub struct ErrorContext {
    operation: String,
    endpoint: Option<String>,
    metadata: HashMap<String, String>,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            endpoint: None,
            metadata: HashMap::new(),
        }
    }

    /// Add endpoint information
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Convert an error with context
    pub fn wrap_err<E: Into<KnowledgeGraphError>>(self, error: E) -> KnowledgeGraphError {
        let mut err = error.into();
        // Add context to the error based on its type
        match &mut err {
            KnowledgeGraphError::NetworkError { endpoint, .. } => {
                if let Some(ep) = self.endpoint {
                    *endpoint = ep;
                }
            }
            KnowledgeGraphError::TimeoutError { operation, endpoint, .. } => {
                *operation = self.operation.clone();
                if let Some(ep) = self.endpoint {
                    *endpoint = ep;
                }
            }
            _ => {}
        }
        err
    }
}

/// Circuit breaker for handling repeated failures
pub struct CircuitBreaker {
    name: String,
    failure_threshold: usize,
    success_threshold: usize,
    timeout: Duration,
    state: Arc<RwLock<CircuitBreakerState>>,
}

#[derive(Debug, Clone)]
struct CircuitBreakerState {
    failure_count: usize,
    success_count: usize,
    last_failure: Option<(Instant, String)>,
    state: BreakerState,
}

#[derive(Debug, Clone, PartialEq)]
enum BreakerState {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(
        name: impl Into<String>,
        failure_threshold: usize,
        success_threshold: usize,
        timeout: Duration,
    ) -> Self {
        Self {
            name: name.into(),
            failure_threshold,
            success_threshold,
            timeout,
            state: Arc::new(RwLock::new(CircuitBreakerState {
                failure_count: 0,
                success_count: 0,
                last_failure: None,
                state: BreakerState::Closed,
            })),
        }
    }

    /// Check if the circuit breaker allows an operation
    pub async fn check(&self) -> Result<()> {
        let mut state = self.state.write().await;
        
        match state.state {
            BreakerState::Open => {
                if let Some((last_failure_time, last_message)) = &state.last_failure {
                    if last_failure_time.elapsed() >= self.timeout {
                        // Transition to half-open
                        state.state = BreakerState::HalfOpen;
                        state.success_count = 0;
                        state.failure_count = 0;
                        Ok(())
                    } else {
                        Err(KnowledgeGraphError::CircuitBreakerOpen {
                            service: self.name.clone(),
                            failure_count: state.failure_count,
                            last_failure_message: last_message.clone(),
                            next_retry_at: *last_failure_time + self.timeout,
                        })
                    }
                } else {
                    // Should not happen, but handle gracefully
                    state.state = BreakerState::Closed;
                    Ok(())
                }
            }
            BreakerState::Closed | BreakerState::HalfOpen => Ok(()),
        }
    }

    /// Record a successful operation
    pub async fn record_success(&self) {
        let mut state = self.state.write().await;
        
        match state.state {
            BreakerState::HalfOpen => {
                state.success_count += 1;
                if state.success_count >= self.success_threshold {
                    state.state = BreakerState::Closed;
                    state.failure_count = 0;
                    state.success_count = 0;
                }
            }
            BreakerState::Closed => {
                state.failure_count = 0;
            }
            BreakerState::Open => {
                // Should not happen in normal flow
            }
        }
    }

    /// Record a failed operation
    pub async fn record_failure(&self, error_message: String) {
        let mut state = self.state.write().await;
        
        state.failure_count += 1;
        state.last_failure = Some((Instant::now(), error_message));
        
        match state.state {
            BreakerState::Closed => {
                if state.failure_count >= self.failure_threshold {
                    state.state = BreakerState::Open;
                }
            }
            BreakerState::HalfOpen => {
                // Single failure in half-open state reopens the circuit
                state.state = BreakerState::Open;
                state.success_count = 0;
            }
            BreakerState::Open => {
                // Already open, just update the failure count
            }
        }
    }

    /// Get current circuit breaker statistics
    pub async fn stats(&self) -> CircuitBreakerStats {
        let state = self.state.read().await;
        CircuitBreakerStats {
            name: self.name.clone(),
            state: format!("{:?}", state.state),
            failure_count: state.failure_count,
            success_count: state.success_count,
            last_failure_ago: state.last_failure.as_ref().map(|(t, _)| t.elapsed()),
        }
    }
}

/// Circuit breaker statistics
#[derive(Debug, Clone)]
pub struct CircuitBreakerStats {
    pub name: String,
    pub state: String,
    pub failure_count: usize,
    pub success_count: usize,
    pub last_failure_ago: Option<Duration>,
}

/// Retry policy configuration
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: usize,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub exponential_base: f64,
    pub jitter: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            exponential_base: 2.0,
            jitter: true,
        }
    }
}

impl RetryPolicy {
    /// Calculate delay for a given attempt number
    pub fn calculate_delay(&self, attempt: usize) -> Duration {
        let exponential_delay = self.initial_delay.as_millis() as f64
            * self.exponential_base.powi(attempt as i32 - 1);
        
        let capped_delay = exponential_delay.min(self.max_delay.as_millis() as f64);
        
        let final_delay = if self.jitter {
            // Add jitter: random value between 0.5 and 1.5 of the delay
            let jitter_factor = 0.5 + rand::random::<f64>();
            capped_delay * jitter_factor
        } else {
            capped_delay
        };
        
        Duration::from_millis(final_delay as u64)
    }

    /// Check if we should retry based on the error type
    pub fn should_retry(error: &KnowledgeGraphError) -> bool {
        matches!(
            error,
            KnowledgeGraphError::NetworkError { .. }
                | KnowledgeGraphError::TimeoutError { .. }
                | KnowledgeGraphError::ConnectionPoolError { .. }
        )
    }
}

/// Retry executor with exponential backoff
pub struct RetryExecutor {
    policy: RetryPolicy,
    circuit_breaker: Option<CircuitBreaker>,
}

impl RetryExecutor {
    /// Create a new retry executor
    pub fn new(policy: RetryPolicy) -> Self {
        Self {
            policy,
            circuit_breaker: None,
        }
    }

    /// Add a circuit breaker
    pub fn with_circuit_breaker(mut self, circuit_breaker: CircuitBreaker) -> Self {
        self.circuit_breaker = Some(circuit_breaker);
        self
    }

    /// Execute an operation with retry logic
    pub async fn execute<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut last_error = None;
        
        for attempt in 1..=self.policy.max_attempts {
            // Check circuit breaker if configured
            if let Some(ref cb) = self.circuit_breaker {
                cb.check().await?;
            }
            
            match operation().await {
                Ok(result) => {
                    // Record success in circuit breaker
                    if let Some(ref cb) = self.circuit_breaker {
                        cb.record_success().await;
                    }
                    return Ok(result);
                }
                Err(e) => {
                    last_error = Some(e);
                    
                    if let Some(ref error) = last_error {
                        // Record failure in circuit breaker
                        if let Some(ref cb) = self.circuit_breaker {
                            cb.record_failure(error.to_string()).await;
                        }
                        
                        // Check if we should retry this error type
                        if !RetryPolicy::should_retry(error) {
                            return Err(error.clone());
                        }
                        
                        // Don't sleep after the last attempt
                        if attempt < self.policy.max_attempts {
                            let delay = self.policy.calculate_delay(attempt);
                            tokio::time::sleep(delay).await;
                        }
                    }
                }
            }
        }
        
        // All attempts failed
        match last_error {
            Some(mut e) => {
                // Add retry context to the error
                if let KnowledgeGraphError::NetworkError { retry_count, .. } = &mut e {
                    *retry_count = self.policy.max_attempts;
                }
                Err(e)
            }
            None => Err(KnowledgeGraphError::InternalError {
                message: "Retry failed without capturing error".to_string(),
                source_error: None,
            }),
        }
    }
}

/// Extension trait for Result types to add context
pub trait ResultExt<T> {
    /// Add context to an error
    fn context(self, context: &str) -> Result<T>;
    
    /// Add detailed context with operation and endpoint
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> ErrorContext;
}

impl<T, E> ResultExt<T> for std::result::Result<T, E>
where
    E: Into<KnowledgeGraphError>,
{
    fn context(self, context: &str) -> Result<T> {
        self.map_err(|e| {
            let base_error = e.into();
            match base_error {
                KnowledgeGraphError::InternalError { message, source_error } => {
                    KnowledgeGraphError::InternalError {
                        message: format!("{}: {}", context, message),
                        source_error,
                    }
                }
                other => KnowledgeGraphError::InternalError {
                    message: context.to_string(),
                    source_error: Some(other.to_string()),
                },
            }
        })
    }
    
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> ErrorContext,
    {
        self.map_err(|e| f().wrap_err(e.into()))
    }
}

/// Helper function to create a partial result error
pub fn partial_result<T>(
    message: impl Into<String>,
    successful: usize,
    failed: usize,
    partial_data: Option<T>,
    failures: Vec<OperationFailure>,
) -> KnowledgeGraphError
where
    T: serde::Serialize,
{
    KnowledgeGraphError::PartialResultError {
        message: message.into(),
        successful_operations: successful,
        failed_operations: failed,
        partial_data: partial_data.and_then(|d| serde_json::to_value(d).ok()),
        failures,
    }
}

// Implement From conversions for common error types
impl From<anyhow::Error> for KnowledgeGraphError {
    fn from(err: anyhow::Error) -> Self {
        KnowledgeGraphError::InternalError {
            message: err.to_string(),
            source_error: None,
        }
    }
}

// Note: From<KnowledgeGraphError> for anyhow::Error is automatically provided by anyhow

impl From<serde_json::Error> for KnowledgeGraphError {
    fn from(err: serde_json::Error) -> Self {
        KnowledgeGraphError::ParseError {
            message: format!("JSON parsing error: {}", err),
            field: None,
            raw_data: None,
            source_error: Some(err.to_string()),
        }
    }
}

impl From<reqwest::Error> for KnowledgeGraphError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            KnowledgeGraphError::TimeoutError {
                operation: "HTTP request".to_string(),
                duration: Duration::from_secs(30), // Default timeout
                endpoint: err.url().map(|u| u.to_string()).unwrap_or_default(),
            }
        } else if err.is_connect() {
            KnowledgeGraphError::NetworkError {
                message: format!("Connection failed: {}", err),
                endpoint: err.url().map(|u| u.to_string()).unwrap_or_default(),
                retry_count: 0,
                source_error: Some(err.to_string()),
            }
        } else {
            KnowledgeGraphError::NetworkError {
                message: err.to_string(),
                endpoint: err.url().map(|u| u.to_string()).unwrap_or_default(),
                retry_count: 0,
                source_error: Some(err.to_string()),
            }
        }
    }
}

impl From<tokio::time::error::Elapsed> for KnowledgeGraphError {
    fn from(_: tokio::time::error::Elapsed) -> Self {
        KnowledgeGraphError::TimeoutError {
            operation: "Unknown operation".to_string(),
            duration: Duration::from_secs(30),
            endpoint: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_context_builder() {
        let error = ErrorContext::new("test_operation")
            .with_endpoint("http://localhost:8080")
            .with_metadata("key", "value")
            .wrap_err(KnowledgeGraphError::NetworkError {
                message: "Test error".to_string(),
                endpoint: String::new(),
                retry_count: 0,
                source_error: None,
            });

        match error {
            KnowledgeGraphError::NetworkError { endpoint, .. } => {
                assert_eq!(endpoint, "http://localhost:8080");
            }
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_retry_policy_delay_calculation() {
        let policy = RetryPolicy {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(1),
            exponential_base: 2.0,
            jitter: false,
            ..Default::default()
        };

        assert_eq!(policy.calculate_delay(1), Duration::from_millis(100));
        assert_eq!(policy.calculate_delay(2), Duration::from_millis(200));
        assert_eq!(policy.calculate_delay(3), Duration::from_millis(400));
        assert_eq!(policy.calculate_delay(4), Duration::from_millis(800));
        assert_eq!(policy.calculate_delay(5), Duration::from_millis(1000)); // Capped at max
    }

    #[tokio::test]
    async fn test_circuit_breaker_state_transitions() {
        let cb = CircuitBreaker::new("test", 2, 2, Duration::from_millis(100));

        // Initial state should be closed
        assert!(cb.check().await.is_ok());

        // First failure
        cb.record_failure("Error 1".to_string()).await;
        assert!(cb.check().await.is_ok());

        // Second failure - should open
        cb.record_failure("Error 2".to_string()).await;
        assert!(cb.check().await.is_err());

        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should be half-open now
        assert!(cb.check().await.is_ok());

        // Success in half-open
        cb.record_success().await;
        assert!(cb.check().await.is_ok());

        // Second success - should close
        cb.record_success().await;
        assert!(cb.check().await.is_ok());
    }

    #[test]
    fn test_partial_result_error() {
        let failures = vec![
            OperationFailure {
                operation_id: "op1".to_string(),
                error_message: "Failed".to_string(),
                error_type: "NetworkError".to_string(),
                timestamp: Instant::now(),
            },
        ];

        let error = partial_result(
            "Some operations failed",
            5,
            1,
            Some(vec!["result1", "result2"]),
            failures,
        );

        match error {
            KnowledgeGraphError::PartialResultError {
                successful_operations,
                failed_operations,
                ..
            } => {
                assert_eq!(successful_operations, 5);
                assert_eq!(failed_operations, 1);
            }
            _ => panic!("Unexpected error type"),
        }
    }
}