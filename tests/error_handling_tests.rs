//! Comprehensive tests for the error handling framework

use backend::core::error::{
    WorkflowError, ErrorCategory, ErrorSeverity,
    context::ErrorContextExt,
    retry::{RetryPolicy, retry_with_policy, RetryBuilder},
    circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState},
    recovery::{with_fallback, with_fallback_fn, CacheRecovery},
    metrics::{record_error, record_retry_attempt, record_retry_success},
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_error_context_creation() {
    let error = WorkflowError::ProcessingError {
        message: "Test processing error".to_string(),
    };
    
    let context = error
        .context("user_id", "12345")
        .with_context("operation", "data_processing")
        .with_correlation_id("req-test-123");
    
    assert_eq!(context.metadata.correlation_id.as_deref(), Some("req-test-123"));
    assert!(context.metadata.context.contains_key("user_id"));
    assert!(context.metadata.context.contains_key("operation"));
}

#[tokio::test]
async fn test_retry_with_exponential_backoff() {
    use std::sync::atomic::{AtomicU32, Ordering};
    
    let attempts = Arc::new(AtomicU32::new(0));
    let start = std::time::Instant::now();
    
    let policy = RetryPolicy::exponential(3);
    let attempts_clone = attempts.clone();
    let result = retry_with_policy(&policy, move || {
        let attempts = attempts_clone.clone();
        async move {
            let attempt = attempts.fetch_add(1, Ordering::SeqCst) + 1;
            if attempt < 3 {
                Err(WorkflowError::ApiError {
                    message: "Service temporarily unavailable".to_string(),
                })
            } else {
                Ok("Success")
            }
        }
    })
    .await;
    
    assert_eq!(result.unwrap(), "Success");
    assert_eq!(attempts.load(Ordering::SeqCst), 3);
    
    // Should have taken at least some time due to backoff
    assert!(start.elapsed() > Duration::from_millis(100));
}

#[tokio::test]
async fn test_retry_builder_fluent_api() {
    let result = RetryBuilder::new()
        .max_attempts(2)
        .initial_delay(Duration::from_millis(50))
        .jitter(0.1)
        .execute(|| async {
            static mut COUNTER: u32 = 0;
            unsafe {
                COUNTER += 1;
                if COUNTER == 1 {
                    Err(WorkflowError::MCPConnectionError {
                        message: "Connection failed".to_string(),
                    })
                } else {
                    Ok("Connected")
                }
            }
        })
        .await;
    
    assert_eq!(result.unwrap(), "Connected");
}

#[tokio::test]
async fn test_circuit_breaker_state_transitions() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 1,
        timeout: Duration::from_millis(100),
        ..Default::default()
    };
    
    let circuit_breaker = Arc::new(CircuitBreaker::new(config));
    
    // Initially closed
    assert_eq!(circuit_breaker.state().await, CircuitState::Closed);
    
    // First failure
    let _ = circuit_breaker.call(|| async {
        Err::<(), _>(WorkflowError::ApiError {
            message: "Service error".to_string(),
        })
    }).await;
    assert_eq!(circuit_breaker.state().await, CircuitState::Closed);
    
    // Second failure - should open
    let _ = circuit_breaker.call(|| async {
        Err::<(), _>(WorkflowError::ApiError {
            message: "Service error".to_string(),
        })
    }).await;
    assert_eq!(circuit_breaker.state().await, CircuitState::Open);
    
    // Calls should be blocked when open
    let result = circuit_breaker.call(|| async {
        Ok::<_, WorkflowError>("Should not execute")
    }).await;
    assert!(matches!(
        result,
        Err(WorkflowError::RuntimeError { message }) if message.contains("Circuit breaker is open")
    ));
    
    // Wait for timeout to transition to half-open
    sleep(Duration::from_millis(150)).await;
    assert_eq!(circuit_breaker.state().await, CircuitState::HalfOpen);
    
    // Success in half-open should close the circuit
    let result = circuit_breaker.call(|| async {
        Ok::<_, WorkflowError>("Service recovered")
    }).await;
    assert_eq!(result.unwrap(), "Service recovered");
    assert_eq!(circuit_breaker.state().await, CircuitState::Closed);
}

#[tokio::test]
async fn test_fallback_mechanisms() {
    // Test simple fallback
    let result = with_fallback(
        || async {
            Err::<String, _>(WorkflowError::ApiError {
                message: "API unavailable".to_string(),
            })
        },
        "Fallback value".to_string(),
    ).await;
    
    assert_eq!(result, "Fallback value");
    
    // Test fallback function
    let result = with_fallback_fn(
        || async {
            Err::<i32, _>(WorkflowError::DatabaseError {
                message: "Connection lost".to_string(),
            })
        },
        |_error| Ok(42), // Default value on error
    ).await;
    
    assert_eq!(result.unwrap(), 42);
}

#[tokio::test]
async fn test_cache_recovery() {
    let cache = Arc::new(CacheRecovery::new(Duration::from_secs(60)));
    
    // First call succeeds and populates cache
    let result = cache.execute("user_data", || async {
        Ok::<_, WorkflowError>("Fresh data".to_string())
    }).await;
    assert_eq!(result.unwrap(), "Fresh data");
    
    // Second call fails but uses cached value
    let result = cache.execute("user_data", || async {
        Err::<String, _>(WorkflowError::ApiError {
            message: "Service down".to_string(),
        })
    }).await;
    assert_eq!(result.unwrap(), "Fresh data");
    
    // Different key should fail
    let result = cache.execute("other_data", || async {
        Err::<String, _>(WorkflowError::ApiError {
            message: "Service down".to_string(),
        })
    }).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_error_categorization() {
    let transient_errors = vec![
        WorkflowError::MCPConnectionError { message: "Connection failed".to_string() },
        WorkflowError::ApiError { message: "Service unavailable".to_string() },
        WorkflowError::DatabaseError { message: "Timeout".to_string() },
    ];
    
    let permanent_errors = vec![
        WorkflowError::CycleDetected,
        WorkflowError::ValidationError { message: "Invalid input".to_string() },
        WorkflowError::InvalidInput("Bad data".to_string()),
    ];
    
    // Test transient errors are retryable
    for error in transient_errors {
        let policy = RetryPolicy::default();
        assert!(policy.should_retry(&error, 0));
    }
    
    // Test permanent errors are not retryable
    for error in permanent_errors {
        let policy = RetryPolicy::default();
        assert!(!policy.should_retry(&error, 0));
    }
}

#[tokio::test]
async fn test_concurrent_circuit_breaker() {
    let circuit_breaker = Arc::new(CircuitBreaker::new(CircuitBreakerConfig {
        failure_threshold: 5,
        ..Default::default()
    }));
    
    // Spawn multiple concurrent failures
    let mut handles = vec![];
    for i in 0..10 {
        let cb = circuit_breaker.clone();
        handles.push(tokio::spawn(async move {
            cb.call(|| async {
                if i < 7 {
                    Err::<(), _>(WorkflowError::ApiError {
                        message: format!("Error {}", i),
                    })
                } else {
                    Ok(())
                }
            }).await
        }));
    }
    
    // Wait for all to complete
    for handle in handles {
        let _ = handle.await;
    }
    
    // Circuit should be open after threshold failures
    assert_eq!(circuit_breaker.state().await, CircuitState::Open);
    
    let metrics = circuit_breaker.metrics();
    assert_eq!(metrics.total_calls, 10);
    assert!(metrics.total_failures >= 5);
}

#[test]
fn test_error_metrics_recording() {
    // Record various error types
    let error = WorkflowError::ApiError { message: "Test".to_string() };
    record_error(&error, ErrorCategory::Transient, ErrorSeverity::Warning);
    
    record_retry_attempt();
    record_retry_success();
    
    // Metrics should be recorded (actual verification would require access to metrics registry)
    // In production, these would be exposed via Prometheus endpoint
}

#[tokio::test]
async fn test_complex_error_recovery_scenario() {
    // Simulate a complex scenario with retries, circuit breaker, and fallback
    let circuit_breaker = Arc::new(CircuitBreaker::new(CircuitBreakerConfig {
        failure_threshold: 3,
        timeout: Duration::from_millis(200),
        ..Default::default()
    }));
    
    let cache = Arc::new(CacheRecovery::new(Duration::from_secs(60)));
    
    // Helper function that combines all error handling strategies
    async fn resilient_operation(
        cb: Arc<CircuitBreaker>,
        cache: Arc<CacheRecovery<String>>,
        key: &str,
    ) -> Result<String, WorkflowError> {
        // Try with circuit breaker
        match cb.call(|| async {
            // Simulate API call with retries
            RetryBuilder::new()
                .max_attempts(2)
                .initial_delay(Duration::from_millis(50))
                .execute(|| async {
                    static mut CALL_COUNT: u32 = 0;
                    unsafe {
                        CALL_COUNT += 1;
                        if CALL_COUNT < 4 {
                            Err(WorkflowError::ApiError {
                                message: "API error".to_string(),
                            })
                        } else {
                            Ok("API Success".to_string())
                        }
                    }
                })
                .await
        }).await {
            Ok(result) => {
                // Update cache on success
                let _ = cache.execute(key, || async { Ok(result.clone()) }).await;
                Ok(result)
            }
            Err(_) => {
                // Try cache fallback
                cache.execute(key, || async {
                    Err(WorkflowError::RuntimeError {
                        message: "All strategies failed".to_string(),
                    })
                }).await
            }
        }
    }
    
    // Prepopulate cache
    let _ = cache.execute("data", || async {
        Ok::<_, WorkflowError>("Cached value".to_string())
    }).await;
    
    // First calls will fail and open circuit breaker
    for _ in 0..3 {
        let _ = resilient_operation(circuit_breaker.clone(), cache.clone(), "data").await;
    }
    
    // Circuit should be open, but we have cache
    let result = resilient_operation(circuit_breaker.clone(), cache.clone(), "data").await;
    assert_eq!(result.unwrap(), "Cached value");
    
    // Wait for circuit to transition to half-open
    sleep(Duration::from_millis(250)).await;
    
    // Next call should succeed and close circuit
    let result = resilient_operation(circuit_breaker.clone(), cache.clone(), "data").await;
    assert_eq!(result.unwrap(), "API Success");
}

#[test]
fn test_error_code_generation() {
    use backend::core::error::context::categorize_error;
    
    let test_cases = vec![
        (WorkflowError::MCPConnectionError { message: "Test".to_string() }, "MCP_CONN_001"),
        (WorkflowError::ApiError { message: "Test".to_string() }, "API_001"),
        (WorkflowError::DatabaseError { message: "Test".to_string() }, "DB_001"),
        (WorkflowError::CycleDetected, "WF_CYCLE_001"),
        (WorkflowError::ValidationError { message: "Test".to_string() }, "VAL_001"),
    ];
    
    for (error, expected_code) in test_cases {
        let (_, _, code) = categorize_error(&error);
        assert_eq!(code, expected_code);
    }
}