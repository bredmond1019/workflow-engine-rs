//! # Circuit Breaker Pattern Implementation
//!
//! This module provides a circuit breaker implementation to prevent cascade failures
//! in distributed systems by temporarily blocking calls to failing services.

use super::WorkflowError;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
use std::future::Future;

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitState {
    /// Circuit is closed - normal operation
    Closed,
    /// Circuit is open - calls are blocked
    Open,
    /// Circuit is half-open - testing if service recovered
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening circuit
    pub failure_threshold: u32,
    /// Number of successes in half-open state before closing
    pub success_threshold: u32,
    /// Duration to wait before transitioning from open to half-open
    pub timeout: Duration,
    /// Time window for counting failures
    pub window: Duration,
    /// Optional callback when state changes
    #[serde(skip)]
    pub on_state_change: Option<Arc<dyn Fn(CircuitState) + Send + Sync>>,
}

impl std::fmt::Debug for CircuitBreakerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CircuitBreakerConfig")
            .field("failure_threshold", &self.failure_threshold)
            .field("success_threshold", &self.success_threshold)
            .field("timeout", &self.timeout)
            .field("window", &self.window)
            .field("on_state_change", &self.on_state_change.is_some())
            .finish()
    }
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            timeout: Duration::from_secs(60),
            window: Duration::from_secs(60),
            on_state_change: None,
        }
    }
}

/// Circuit breaker implementation
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitState>>,
    failure_count: Arc<AtomicU32>,
    success_count: Arc<AtomicU32>,
    last_failure_time: Arc<Mutex<Option<Instant>>>,
    state_changed_at: Arc<Mutex<Instant>>,
    total_calls: Arc<AtomicU64>,
    total_failures: Arc<AtomicU64>,
    total_successes: Arc<AtomicU64>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(AtomicU32::new(0)),
            success_count: Arc::new(AtomicU32::new(0)),
            last_failure_time: Arc::new(Mutex::new(None)),
            state_changed_at: Arc::new(Mutex::new(Instant::now())),
            total_calls: Arc::new(AtomicU64::new(0)),
            total_failures: Arc::new(AtomicU64::new(0)),
            total_successes: Arc::new(AtomicU64::new(0)),
        }
    }
    
    /// Create a circuit breaker with default configuration
    pub fn default() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }
    
    /// Get current circuit state
    pub async fn state(&self) -> CircuitState {
        let state = *self.state.read().await;
        
        // Check if we should transition from Open to HalfOpen
        if state == CircuitState::Open {
            let state_changed_at = self.state_changed_at.lock().unwrap().clone();
            if state_changed_at.elapsed() >= self.config.timeout {
                self.transition_to(CircuitState::HalfOpen).await;
                return CircuitState::HalfOpen;
            }
        }
        
        state
    }
    
    /// Execute a function through the circuit breaker
    pub async fn call<F, Fut, T>(&self, f: F) -> Result<T, WorkflowError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, WorkflowError>>,
    {
        self.total_calls.fetch_add(1, Ordering::Relaxed);
        
        let current_state = self.state().await;
        
        match current_state {
            CircuitState::Open => {
                Err(WorkflowError::RuntimeError {
                    message: "Circuit breaker is open".to_string(),
                })
            }
            CircuitState::Closed | CircuitState::HalfOpen => {
                match f().await {
                    Ok(result) => {
                        self.on_success().await;
                        Ok(result)
                    }
                    Err(error) => {
                        self.on_failure().await;
                        Err(error)
                    }
                }
            }
        }
    }
    
    /// Record a successful call
    async fn on_success(&self) {
        self.total_successes.fetch_add(1, Ordering::Relaxed);
        
        let current_state = *self.state.read().await;
        
        match current_state {
            CircuitState::HalfOpen => {
                let count = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;
                if count >= self.config.success_threshold {
                    self.transition_to(CircuitState::Closed).await;
                }
            }
            CircuitState::Closed => {
                // Reset failure count on success in closed state
                self.failure_count.store(0, Ordering::SeqCst);
            }
            _ => {}
        }
    }
    
    /// Record a failed call
    async fn on_failure(&self) {
        self.total_failures.fetch_add(1, Ordering::Relaxed);
        
        let current_state = *self.state.read().await;
        
        match current_state {
            CircuitState::HalfOpen => {
                // Any failure in half-open state opens the circuit
                self.transition_to(CircuitState::Open).await;
            }
            CircuitState::Closed => {
                // Check if we're within the time window
                let now = Instant::now();
                let should_increment = {
                    let mut last_failure = self.last_failure_time.lock().unwrap();
                    if let Some(last_time) = *last_failure {
                        if now.duration_since(last_time) > self.config.window {
                            // Outside window, reset count
                            self.failure_count.store(1, Ordering::SeqCst);
                            *last_failure = Some(now);
                            false
                        } else {
                            true
                        }
                    } else {
                        *last_failure = Some(now);
                        true
                    }
                };
                
                if should_increment {
                    let count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
                    if count >= self.config.failure_threshold {
                        self.transition_to(CircuitState::Open).await;
                    }
                }
            }
            _ => {}
        }
    }
    
    /// Transition to a new state
    async fn transition_to(&self, new_state: CircuitState) {
        let mut state = self.state.write().await;
        let old_state = *state;
        
        if old_state != new_state {
            *state = new_state;
            *self.state_changed_at.lock().unwrap() = Instant::now();
            
            // Reset counters based on transition
            match new_state {
                CircuitState::Closed => {
                    self.failure_count.store(0, Ordering::SeqCst);
                    self.success_count.store(0, Ordering::SeqCst);
                }
                CircuitState::HalfOpen => {
                    self.success_count.store(0, Ordering::SeqCst);
                }
                CircuitState::Open => {
                    self.failure_count.store(0, Ordering::SeqCst);
                }
            }
            
            // Call state change callback if configured
            if let Some(ref callback) = self.config.on_state_change {
                callback(new_state);
            }
            
            tracing::info!(
                old_state = ?old_state,
                new_state = ?new_state,
                "Circuit breaker state changed"
            );
        }
    }
    
    /// Get circuit breaker metrics
    pub fn metrics(&self) -> CircuitBreakerMetrics {
        CircuitBreakerMetrics {
            total_calls: self.total_calls.load(Ordering::Relaxed),
            total_failures: self.total_failures.load(Ordering::Relaxed),
            total_successes: self.total_successes.load(Ordering::Relaxed),
            failure_count: self.failure_count.load(Ordering::Relaxed),
            success_count: self.success_count.load(Ordering::Relaxed),
        }
    }
    
    /// Reset the circuit breaker
    pub async fn reset(&self) {
        self.transition_to(CircuitState::Closed).await;
        self.failure_count.store(0, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        *self.last_failure_time.lock().unwrap() = None;
    }
}

/// Circuit breaker metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerMetrics {
    pub total_calls: u64,
    pub total_failures: u64,
    pub total_successes: u64,
    pub failure_count: u32,
    pub success_count: u32,
}

/// Circuit breaker builder
pub struct CircuitBreakerBuilder {
    config: CircuitBreakerConfig,
}

impl CircuitBreakerBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: CircuitBreakerConfig::default(),
        }
    }
    
    /// Set failure threshold
    pub fn failure_threshold(mut self, threshold: u32) -> Self {
        self.config.failure_threshold = threshold;
        self
    }
    
    /// Set success threshold
    pub fn success_threshold(mut self, threshold: u32) -> Self {
        self.config.success_threshold = threshold;
        self
    }
    
    /// Set timeout duration
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }
    
    /// Set time window
    pub fn window(mut self, window: Duration) -> Self {
        self.config.window = window;
        self
    }
    
    /// Set state change callback
    pub fn on_state_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(CircuitState) + Send + Sync + 'static,
    {
        self.config.on_state_change = Some(Arc::new(callback));
        self
    }
    
    /// Build the circuit breaker
    pub fn build(self) -> CircuitBreaker {
        CircuitBreaker::new(self.config)
    }
}

/// Collection of circuit breakers for different services
pub struct CircuitBreakerRegistry {
    breakers: Arc<RwLock<std::collections::HashMap<String, Arc<CircuitBreaker>>>>,
    default_config: CircuitBreakerConfig,
}

impl CircuitBreakerRegistry {
    /// Create a new registry
    pub fn new(default_config: CircuitBreakerConfig) -> Self {
        Self {
            breakers: Arc::new(RwLock::new(std::collections::HashMap::new())),
            default_config,
        }
    }
    
    /// Get or create a circuit breaker for a service
    pub async fn get(&self, service: &str) -> Arc<CircuitBreaker> {
        let breakers = self.breakers.read().await;
        if let Some(breaker) = breakers.get(service) {
            return breaker.clone();
        }
        drop(breakers);
        
        let mut breakers = self.breakers.write().await;
        breakers.entry(service.to_string())
            .or_insert_with(|| Arc::new(CircuitBreaker::new(self.default_config.clone())))
            .clone()
    }
    
    /// Get all circuit breakers
    pub async fn all(&self) -> Vec<(String, Arc<CircuitBreaker>)> {
        self.breakers.read().await
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_circuit_breaker_state_transitions() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 1,
            timeout: Duration::from_millis(100),
            window: Duration::from_secs(60),
            on_state_change: None,
        });
        
        assert_eq!(cb.state().await, CircuitState::Closed);
        
        // First failure
        let _ = cb.call(|| async {
            Err::<(), _>(WorkflowError::api_error_simple("Test error"))
        }).await;
        assert_eq!(cb.state().await, CircuitState::Closed);
        
        // Second failure - should open
        let _ = cb.call(|| async {
            Err::<(), _>(WorkflowError::api_error_simple("Test error"))
        }).await;
        assert_eq!(cb.state().await, CircuitState::Open);
        
        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert_eq!(cb.state().await, CircuitState::HalfOpen);
        
        // Success in half-open - should close
        let _ = cb.call(|| async { Ok::<_, WorkflowError>(()) }).await;
        assert_eq!(cb.state().await, CircuitState::Closed);
    }
    
    #[tokio::test]
    async fn test_circuit_breaker_blocks_when_open() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 1,
            ..Default::default()
        });
        
        // Open the circuit
        let _ = cb.call(|| async {
            Err::<(), _>(WorkflowError::api_error_simple("Test error"))
        }).await;
        
        // Should block calls
        let result = cb.call(|| async { Ok::<_, WorkflowError>(42) }).await;
        assert!(matches!(
            result,
            Err(WorkflowError::RuntimeError { message }) if message.contains("Circuit breaker is open")
        ));
    }
    
    #[tokio::test]
    async fn test_circuit_breaker_metrics() {
        let cb = CircuitBreaker::default();
        
        // Some successful calls
        for _ in 0..3 {
            let _ = cb.call(|| async { Ok::<_, WorkflowError>(()) }).await;
        }
        
        // Some failed calls
        for _ in 0..2 {
            let _ = cb.call(|| async {
                Err::<(), _>(WorkflowError::api_error_simple("Test error"))
            }).await;
        }
        
        let metrics = cb.metrics();
        assert_eq!(metrics.total_calls, 5);
        assert_eq!(metrics.total_successes, 3);
        assert_eq!(metrics.total_failures, 2);
    }
}