//! # Retry Logic with Exponential Backoff
//!
//! This module provides configurable retry logic for handling transient failures
//! with exponential backoff and jitter to prevent thundering herd problems.

use super::{WorkflowError, ErrorCategory, ErrorMetadata};
use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;
use rand::Rng;
use serde::{Serialize, Deserialize};

/// Retry policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial retry delay
    pub initial_delay: Duration,
    /// Maximum retry delay
    pub max_delay: Duration,
    /// Exponential backoff multiplier
    pub multiplier: f64,
    /// Jitter factor (0.0 to 1.0)
    pub jitter_factor: f64,
    /// Whether to retry on specific error types only
    pub retry_on: Option<Vec<String>>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            jitter_factor: 0.1,
            retry_on: None,
        }
    }
}

impl RetryPolicy {
    /// Create a policy with fixed delay
    pub fn fixed(attempts: u32, delay: Duration) -> Self {
        Self {
            max_attempts: attempts,
            initial_delay: delay,
            max_delay: delay,
            multiplier: 1.0,
            jitter_factor: 0.0,
            retry_on: None,
        }
    }
    
    /// Create a policy with exponential backoff
    pub fn exponential(attempts: u32) -> Self {
        Self {
            max_attempts: attempts,
            ..Default::default()
        }
    }
    
    /// Create a policy with linear backoff
    pub fn linear(attempts: u32, increment: Duration) -> Self {
        Self {
            max_attempts: attempts,
            initial_delay: increment,
            max_delay: Duration::from_secs(60),
            multiplier: 1.0,
            jitter_factor: 0.0,
            retry_on: None,
        }
    }
    
    /// Calculate delay for a given attempt
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::ZERO;
        }
        
        let mut delay = self.initial_delay.as_millis() as f64;
        
        // Apply exponential backoff
        if self.multiplier > 1.0 {
            delay *= self.multiplier.powi(attempt as i32 - 1);
        }
        
        // Cap at max delay
        delay = delay.min(self.max_delay.as_millis() as f64);
        
        // Apply jitter
        if self.jitter_factor > 0.0 {
            let mut rng = rand::thread_rng();
            let jitter = rng.gen_range(-self.jitter_factor..=self.jitter_factor);
            delay *= 1.0 + jitter;
        }
        
        Duration::from_millis(delay as u64)
    }
    
    /// Check if an error should be retried
    pub fn should_retry(&self, error: &WorkflowError, attempt: u32) -> bool {
        if attempt >= self.max_attempts {
            return false;
        }
        
        // Check if error is retryable
        if !is_retryable_error(error) {
            return false;
        }
        
        // Check specific error types if configured
        if let Some(ref retry_on) = self.retry_on {
            let error_type = format!("{:?}", error);
            retry_on.iter().any(|pattern| error_type.contains(pattern))
        } else {
            true
        }
    }
}

/// Trait for retryable operations
pub trait RetryableError {
    /// Check if the error is retryable
    fn is_retryable(&self) -> bool;
    
    /// Get the error category
    fn category(&self) -> ErrorCategory;
}

impl RetryableError for WorkflowError {
    fn is_retryable(&self) -> bool {
        is_retryable_error(self)
    }
    
    fn category(&self) -> ErrorCategory {
        match self {
            // Transient errors - can be retried
            WorkflowError::MCPConnectionError { .. } |
            WorkflowError::MCPTransportError { .. } |
            WorkflowError::ApiError { .. } |
            WorkflowError::DatabaseError { .. } => ErrorCategory::Transient,
            
            // Permanent errors - should not be retried
            WorkflowError::CycleDetected |
            WorkflowError::UnreachableNodes { .. } |
            WorkflowError::InvalidRouter { .. } |
            WorkflowError::NodeNotFound { .. } |
            WorkflowError::ValidationError { .. } |
            WorkflowError::InvalidStepType(_) |
            WorkflowError::InvalidInput(_) |
            WorkflowError::ConfigurationError(_) => ErrorCategory::Permanent,
            
            // System errors - may be retryable
            WorkflowError::ProcessingError { .. } |
            WorkflowError::SerializationError { .. } |
            WorkflowError::DeserializationError { .. } |
            WorkflowError::RuntimeError { .. } => ErrorCategory::System,
            
            // Other errors - check message for clues
            _ => ErrorCategory::System,
        }
    }
}

/// Check if an error is retryable based on its type
fn is_retryable_error(error: &WorkflowError) -> bool {
    matches!(
        error,
        WorkflowError::MCPConnectionError { .. } |
        WorkflowError::MCPTransportError { .. } |
        WorkflowError::ApiError { .. } |
        WorkflowError::DatabaseError { .. }
    )
}

/// Retry an async operation with the given policy
pub async fn retry_with_policy<F, Fut, T>(
    policy: &RetryPolicy,
    mut operation: F,
) -> Result<T, WorkflowError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, WorkflowError>>,
{
    let mut attempt = 0;
    let mut last_error = None;
    
    loop {
        match operation().await {
            Ok(result) => {
                if attempt > 0 {
                    tracing::info!(
                        attempt = attempt,
                        "Operation succeeded after retry"
                    );
                }
                return Ok(result);
            }
            Err(error) => {
                if !policy.should_retry(&error, attempt) {
                    tracing::error!(
                        error = %error,
                        attempt = attempt,
                        "Operation failed, no more retries"
                    );
                    return Err(error);
                }
                
                let delay = policy.calculate_delay(attempt + 1);
                tracing::warn!(
                    error = %error,
                    attempt = attempt + 1,
                    max_attempts = policy.max_attempts,
                    delay_ms = delay.as_millis(),
                    "Operation failed, retrying"
                );
                
                last_error = Some(error);
                attempt += 1;
                sleep(delay).await;
            }
        }
    }
}

/// Retry builder for fluent API
pub struct RetryBuilder {
    policy: RetryPolicy,
}

impl RetryBuilder {
    /// Create a new retry builder
    pub fn new() -> Self {
        Self {
            policy: RetryPolicy::default(),
        }
    }
    
    /// Set maximum attempts
    pub fn max_attempts(mut self, attempts: u32) -> Self {
        self.policy.max_attempts = attempts;
        self
    }
    
    /// Set initial delay
    pub fn initial_delay(mut self, delay: Duration) -> Self {
        self.policy.initial_delay = delay;
        self
    }
    
    /// Set maximum delay
    pub fn max_delay(mut self, delay: Duration) -> Self {
        self.policy.max_delay = delay;
        self
    }
    
    /// Set backoff multiplier
    pub fn multiplier(mut self, multiplier: f64) -> Self {
        self.policy.multiplier = multiplier;
        self
    }
    
    /// Set jitter factor
    pub fn jitter(mut self, factor: f64) -> Self {
        self.policy.jitter_factor = factor.clamp(0.0, 1.0);
        self
    }
    
    /// Set specific error types to retry on
    pub fn retry_on(mut self, errors: Vec<String>) -> Self {
        self.policy.retry_on = Some(errors);
        self
    }
    
    /// Build the retry policy
    pub fn build(self) -> RetryPolicy {
        self.policy
    }
    
    /// Execute an operation with the built policy
    pub async fn execute<F, Fut, T>(self, operation: F) -> Result<T, WorkflowError>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, WorkflowError>>,
    {
        retry_with_policy(&self.policy, operation).await
    }
}

/// Convenience function to retry with default policy
pub async fn retry<F, Fut, T>(operation: F) -> Result<T, WorkflowError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, WorkflowError>>,
{
    retry_with_policy(&RetryPolicy::default(), operation).await
}

/// Retry with custom attempts
pub async fn retry_with_attempts<F, Fut, T>(
    attempts: u32,
    operation: F,
) -> Result<T, WorkflowError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, WorkflowError>>,
{
    let policy = RetryPolicy {
        max_attempts: attempts,
        ..Default::default()
    };
    retry_with_policy(&policy, operation).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    
    #[test]
    fn test_retry_policy_delay_calculation() {
        let policy = RetryPolicy::default();
        
        assert_eq!(policy.calculate_delay(0), Duration::ZERO);
        
        // Test exponential backoff
        let delay1 = policy.calculate_delay(1);
        let delay2 = policy.calculate_delay(2);
        let delay3 = policy.calculate_delay(3);
        
        assert!(delay1 >= Duration::from_millis(90)); // With jitter
        assert!(delay1 <= Duration::from_millis(110));
        assert!(delay2 > delay1);
        assert!(delay3 > delay2);
    }
    
    #[test]
    fn test_fixed_retry_policy() {
        let policy = RetryPolicy::fixed(5, Duration::from_millis(500));
        
        assert_eq!(policy.calculate_delay(1), Duration::from_millis(500));
        assert_eq!(policy.calculate_delay(2), Duration::from_millis(500));
        assert_eq!(policy.calculate_delay(3), Duration::from_millis(500));
    }
    
    #[tokio::test]
    async fn test_retry_success() {
        use std::sync::atomic::{AtomicU32, Ordering};
        let attempt = Arc::new(AtomicU32::new(0));
        let attempt_clone = attempt.clone();
        
        let result = retry_with_attempts(3, move || {
            let attempt_clone = attempt_clone.clone();
            async move {
                let current = attempt_clone.fetch_add(1, Ordering::SeqCst) + 1;
                if current < 3 {
                    Err(WorkflowError::ApiError {
                        message: "Temporary failure".to_string(),
                    })
                } else {
                    Ok(42)
                }
            }
        })
        .await;
        
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempt.load(Ordering::SeqCst), 3);
    }
    
    #[tokio::test]
    async fn test_retry_permanent_failure() {
        use std::sync::atomic::{AtomicU32, Ordering};
        let attempt = Arc::new(AtomicU32::new(0));
        let attempt_clone = attempt.clone();
        
        let result: Result<(), WorkflowError> = retry_with_attempts(3, move || {
            let attempt_clone = attempt_clone.clone();
            async move {
                attempt_clone.fetch_add(1, Ordering::SeqCst);
                Err(WorkflowError::ValidationError {
                    message: "Invalid input".to_string(),
                })
            }
        })
        .await;
        
        assert!(result.is_err());
        assert_eq!(attempt.load(Ordering::SeqCst), 1); // Should not retry validation errors
    }
    
    #[test]
    fn test_retry_builder() {
        let policy = RetryBuilder::new()
            .max_attempts(5)
            .initial_delay(Duration::from_millis(200))
            .max_delay(Duration::from_secs(10))
            .multiplier(3.0)
            .jitter(0.2)
            .build();
        
        assert_eq!(policy.max_attempts, 5);
        assert_eq!(policy.initial_delay, Duration::from_millis(200));
        assert_eq!(policy.max_delay, Duration::from_secs(10));
        assert_eq!(policy.multiplier, 3.0);
        assert_eq!(policy.jitter_factor, 0.2);
    }
}