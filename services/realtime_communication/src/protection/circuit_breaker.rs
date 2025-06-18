//! Circuit Breaker Implementation
//! 
//! Implements circuit breaker pattern for protecting downstream services
//! with configurable failure thresholds, timeouts, and backpressure mechanisms.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{warn, info};
use std::collections::HashMap;

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub enum CircuitState {
    Closed,      // Normal operation
    Open,        // Circuit is open, requests are failing fast
    HalfOpen,    // Testing if service has recovered
}

/// Circuit breaker configuration
#[derive(Debug, Clone, serde::Serialize)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub timeout: Duration,
    pub max_requests_in_half_open: u32,
    pub slow_call_threshold: Duration,
    pub slow_call_rate_threshold: f64,
    pub minimum_throughput: u32,
    pub sliding_window_size: u32,
    pub enabled: bool,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(60),
            max_requests_in_half_open: 10,
            slow_call_threshold: Duration::from_secs(5),
            slow_call_rate_threshold: 0.5,
            minimum_throughput: 10,
            sliding_window_size: 100,
            enabled: true,
        }
    }
}

/// Circuit breaker result
#[derive(Debug, Clone)]
pub enum CircuitResult<T> {
    Success(T),
    Failure(CircuitError),
    Rejected(String),
}

#[derive(Debug, Clone)]
pub enum CircuitError {
    Timeout,
    ServiceError(String),
    SlowCall(Duration),
    CircuitOpen,
}

/// Call result for tracking
#[derive(Debug, Clone)]
pub struct CallResult {
    pub success: bool,
    pub duration: Duration,
    pub timestamp: Instant,
    pub error: Option<String>,
}

/// Sliding window for tracking call results
#[derive(Debug, Clone)]
pub struct SlidingWindow {
    pub results: Vec<CallResult>,
    pub max_size: usize,
    pub current_index: usize,
}

impl SlidingWindow {
    pub fn new(size: usize) -> Self {
        Self {
            results: Vec::with_capacity(size),
            max_size: size,
            current_index: 0,
        }
    }

    pub fn add_result(&mut self, result: CallResult) {
        if self.results.len() < self.max_size {
            self.results.push(result);
        } else {
            self.results[self.current_index] = result;
            self.current_index = (self.current_index + 1) % self.max_size;
        }
    }

    pub fn failure_rate(&self) -> f64 {
        if self.results.is_empty() {
            return 0.0;
        }

        let failures = self.results.iter().filter(|r| !r.success).count();
        failures as f64 / self.results.len() as f64
    }

    pub fn slow_call_rate(&self, threshold: Duration) -> f64 {
        if self.results.is_empty() {
            return 0.0;
        }

        let slow_calls = self.results.iter()
            .filter(|r| r.duration > threshold)
            .count();
        slow_calls as f64 / self.results.len() as f64
    }

    pub fn total_calls(&self) -> usize {
        self.results.len()
    }

    pub fn recent_successes(&self, count: usize) -> usize {
        self.results.iter()
            .rev()
            .take(count)
            .filter(|r| r.success)
            .count()
    }

    pub fn recent_failures(&self, count: usize) -> usize {
        self.results.iter()
            .rev()
            .take(count)
            .filter(|r| !r.success)
            .count()
    }
}

/// Circuit breaker implementation
pub struct CircuitBreaker {
    pub name: String,
    pub state: Arc<RwLock<CircuitState>>,
    pub config: CircuitBreakerConfig,
    pub window: Arc<RwLock<SlidingWindow>>,
    pub last_failure_time: Arc<RwLock<Option<Instant>>>,
    pub half_open_requests: Arc<RwLock<u32>>,
    pub metrics: Arc<CircuitBreakerMetrics>,
}

/// Circuit breaker metrics
#[derive(Debug, Default)]
pub struct CircuitBreakerMetrics {
    pub total_requests: Arc<RwLock<u64>>,
    pub successful_requests: Arc<RwLock<u64>>,
    pub failed_requests: Arc<RwLock<u64>>,
    pub rejected_requests: Arc<RwLock<u64>>,
    pub slow_requests: Arc<RwLock<u64>>,
    pub state_transitions: Arc<RwLock<HashMap<String, u64>>>,
}

impl CircuitBreakerMetrics {
    pub async fn increment_total(&self) {
        *self.total_requests.write().await += 1;
    }

    pub async fn increment_successful(&self) {
        *self.successful_requests.write().await += 1;
    }

    pub async fn increment_failed(&self) {
        *self.failed_requests.write().await += 1;
    }

    pub async fn increment_rejected(&self) {
        *self.rejected_requests.write().await += 1;
    }

    pub async fn increment_slow(&self) {
        *self.slow_requests.write().await += 1;
    }

    pub async fn record_state_transition(&self, from: &CircuitState, to: &CircuitState) {
        let transition = format!("{:?}_to_{:?}", from, to);
        let mut transitions = self.state_transitions.write().await;
        *transitions.entry(transition).or_insert(0) += 1;
    }

    pub async fn get_stats(&self) -> CircuitBreakerStats {
        CircuitBreakerStats {
            total_requests: *self.total_requests.read().await,
            successful_requests: *self.successful_requests.read().await,
            failed_requests: *self.failed_requests.read().await,
            rejected_requests: *self.rejected_requests.read().await,
            slow_requests: *self.slow_requests.read().await,
            state_transitions: self.state_transitions.read().await.clone(),
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct CircuitBreakerStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub rejected_requests: u64,
    pub slow_requests: u64,
    pub state_transitions: HashMap<String, u64>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(name: String, config: CircuitBreakerConfig) -> Self {
        Self {
            name,
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            window: Arc::new(RwLock::new(SlidingWindow::new(config.sliding_window_size as usize))),
            last_failure_time: Arc::new(RwLock::new(None)),
            half_open_requests: Arc::new(RwLock::new(0)),
            config,
            metrics: Arc::new(CircuitBreakerMetrics::default()),
        }
    }

    /// Execute a function with circuit breaker protection
    pub async fn execute<F, Fut, T>(&self, operation: F) -> CircuitResult<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, String>>,
    {
        if !self.config.enabled {
            // Circuit breaker disabled, execute directly
            let _start = Instant::now();
            match operation().await {
                Ok(result) => CircuitResult::Success(result),
                Err(error) => CircuitResult::Failure(CircuitError::ServiceError(error)),
            }
        } else {
            self.execute_with_circuit_breaker(operation).await
        }
    }

    async fn execute_with_circuit_breaker<F, Fut, T>(&self, operation: F) -> CircuitResult<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, String>>,
    {
        self.metrics.increment_total().await;

        // Check if we can proceed based on current state
        let can_proceed = self.can_proceed().await;
        if !can_proceed {
            self.metrics.increment_rejected().await;
            return CircuitResult::Rejected(format!("Circuit breaker {} is open", self.name));
        }

        // Execute the operation
        let start = Instant::now();
        let result = tokio::time::timeout(self.config.timeout, operation()).await;
        let duration = start.elapsed();

        match result {
            Ok(Ok(value)) => {
                // Successful execution
                self.record_success(duration).await;
                CircuitResult::Success(value)
            }
            Ok(Err(error)) => {
                // Operation failed
                self.record_failure(duration, Some(error.clone())).await;
                CircuitResult::Failure(CircuitError::ServiceError(error))
            }
            Err(_) => {
                // Timeout
                self.record_failure(duration, Some("Timeout".to_string())).await;
                CircuitResult::Failure(CircuitError::Timeout)
            }
        }
    }

    /// Check if request can proceed based on circuit state
    async fn can_proceed(&self) -> bool {
        let state = *self.state.read().await;

        match state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if timeout has passed
                if let Some(last_failure) = *self.last_failure_time.read().await {
                    if last_failure.elapsed() >= self.config.timeout {
                        self.transition_to_half_open().await;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => {
                // Allow limited requests in half-open state
                let current_requests = *self.half_open_requests.read().await;
                current_requests < self.config.max_requests_in_half_open
            }
        }
    }

    /// Record successful operation
    async fn record_success(&self, duration: Duration) {
        self.metrics.increment_successful().await;

        // Check if it's a slow call
        if duration > self.config.slow_call_threshold {
            self.metrics.increment_slow().await;
        }

        let result = CallResult {
            success: true,
            duration,
            timestamp: Instant::now(),
            error: None,
        };

        self.window.write().await.add_result(result);

        let current_state = *self.state.read().await;
        match current_state {
            CircuitState::HalfOpen => {
                // Check if we have enough successes to close the circuit
                let window = self.window.read().await;
                let recent_successes = window.recent_successes(self.config.success_threshold as usize);
                
                if recent_successes >= self.config.success_threshold as usize {
                    self.transition_to_closed().await;
                }
            }
            _ => {
                // Check if we need to transition from closed to open due to slow calls
                if current_state == CircuitState::Closed {
                    self.check_health().await;
                }
            }
        }
    }

    /// Record failed operation
    async fn record_failure(&self, duration: Duration, error: Option<String>) {
        self.metrics.increment_failed().await;

        let result = CallResult {
            success: false,
            duration,
            timestamp: Instant::now(),
            error,
        };

        self.window.write().await.add_result(result);
        *self.last_failure_time.write().await = Some(Instant::now());

        let current_state = *self.state.read().await;
        match current_state {
            CircuitState::Closed => {
                self.check_health().await;
            }
            CircuitState::HalfOpen => {
                // Any failure in half-open state opens the circuit
                self.transition_to_open().await;
            }
            CircuitState::Open => {
                // Already open, just update the failure time
            }
        }
    }

    /// Check circuit health and transition if necessary
    async fn check_health(&self) {
        let window = self.window.read().await;
        
        // Only evaluate if we have minimum throughput
        if window.total_calls() < self.config.minimum_throughput as usize {
            return;
        }

        let failure_rate = window.failure_rate();
        let slow_call_rate = window.slow_call_rate(self.config.slow_call_threshold);

        // Check if we should open the circuit
        let should_open = failure_rate >= (self.config.failure_threshold as f64 / 100.0) ||
                         slow_call_rate >= self.config.slow_call_rate_threshold;

        if should_open {
            drop(window); // Release the lock before state transition
            self.transition_to_open().await;
        }
    }

    /// Transition to closed state
    async fn transition_to_closed(&self) {
        let old_state = {
            let mut state = self.state.write().await;
            let old = *state;
            *state = CircuitState::Closed;
            old
        };

        *self.half_open_requests.write().await = 0;
        *self.last_failure_time.write().await = None;

        self.metrics.record_state_transition(&old_state, &CircuitState::Closed).await;
        info!("Circuit breaker {} transitioned to CLOSED", self.name);
    }

    /// Transition to open state
    async fn transition_to_open(&self) {
        let old_state = {
            let mut state = self.state.write().await;
            let old = *state;
            *state = CircuitState::Open;
            old
        };

        *self.last_failure_time.write().await = Some(Instant::now());

        self.metrics.record_state_transition(&old_state, &CircuitState::Open).await;
        warn!("Circuit breaker {} transitioned to OPEN", self.name);
    }

    /// Transition to half-open state
    async fn transition_to_half_open(&self) {
        let old_state = {
            let mut state = self.state.write().await;
            let old = *state;
            *state = CircuitState::HalfOpen;
            old
        };

        *self.half_open_requests.write().await = 0;

        self.metrics.record_state_transition(&old_state, &CircuitState::HalfOpen).await;
        info!("Circuit breaker {} transitioned to HALF_OPEN", self.name);
    }

    /// Get current circuit breaker status
    pub async fn get_status(&self) -> CircuitBreakerStatus {
        let state = *self.state.read().await;
        let window = self.window.read().await;
        let half_open_requests = *self.half_open_requests.read().await;
        let last_failure_time = *self.last_failure_time.read().await;

        CircuitBreakerStatus {
            name: self.name.clone(),
            state,
            failure_rate: window.failure_rate(),
            slow_call_rate: window.slow_call_rate(self.config.slow_call_threshold),
            total_calls: window.total_calls(),
            half_open_requests,
            last_failure_time,
            config: self.config.clone(),
        }
    }

    /// Get circuit breaker metrics
    pub async fn get_metrics(&self) -> CircuitBreakerStats {
        self.metrics.get_stats().await
    }

    /// Reset circuit breaker state
    pub async fn reset(&self) {
        let old_state = {
            let mut state = self.state.write().await;
            let old = *state;
            *state = CircuitState::Closed;
            old
        };

        *self.window.write().await = SlidingWindow::new(self.config.sliding_window_size as usize);
        *self.half_open_requests.write().await = 0;
        *self.last_failure_time.write().await = None;

        self.metrics.record_state_transition(&old_state, &CircuitState::Closed).await;
        info!("Circuit breaker {} has been reset", self.name);
    }
}

/// Circuit breaker status for monitoring
#[derive(Debug, serde::Serialize)]
pub struct CircuitBreakerStatus {
    pub name: String,
    pub state: CircuitState,
    pub failure_rate: f64,
    pub slow_call_rate: f64,
    pub total_calls: usize,
    pub half_open_requests: u32,
    #[serde(skip)]
    pub last_failure_time: Option<Instant>,
    pub config: CircuitBreakerConfig,
}

/// Manager for multiple circuit breakers
pub struct CircuitBreakerManager {
    breakers: Arc<RwLock<HashMap<String, Arc<CircuitBreaker>>>>,
}

impl CircuitBreakerManager {
    pub fn new() -> Self {
        Self {
            breakers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add or update a circuit breaker
    pub async fn add_breaker(&self, name: String, config: CircuitBreakerConfig) {
        let breaker = Arc::new(CircuitBreaker::new(name.clone(), config));
        self.breakers.write().await.insert(name, breaker);
    }

    /// Get a circuit breaker by name
    pub async fn get_breaker(&self, name: &str) -> Option<Arc<CircuitBreaker>> {
        self.breakers.read().await.get(name).cloned()
    }

    /// Remove a circuit breaker
    pub async fn remove_breaker(&self, name: &str) -> bool {
        self.breakers.write().await.remove(name).is_some()
    }

    /// Get all circuit breaker statuses
    pub async fn get_all_statuses(&self) -> Vec<CircuitBreakerStatus> {
        let breakers = self.breakers.read().await;
        let mut statuses = Vec::new();

        for breaker in breakers.values() {
            statuses.push(breaker.get_status().await);
        }

        statuses
    }

    /// Reset all circuit breakers
    pub async fn reset_all(&self) {
        let breakers = self.breakers.read().await;
        for breaker in breakers.values() {
            breaker.reset().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_circuit_breaker_success() {
        let config = CircuitBreakerConfig::default();
        let breaker = CircuitBreaker::new("test".to_string(), config);

        let result = breaker.execute(|| async { Ok::<i32, String>(42) }).await;
        
        match result {
            CircuitResult::Success(value) => assert_eq!(value, 42),
            other => panic!("Expected success, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_circuit_breaker_failure() {
        let config = CircuitBreakerConfig::default();
        let breaker = CircuitBreaker::new("test".to_string(), config);

        let result = breaker.execute(|| async { Err::<i32, String>("test error".to_string()) }).await;
        
        match result {
            CircuitResult::Failure(CircuitError::ServiceError(error)) => {
                assert_eq!(error, "test error");
            }
            other => panic!("Expected failure, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_circuit_breaker_timeout() {
        let mut config = CircuitBreakerConfig::default();
        config.timeout = Duration::from_millis(100);
        let breaker = CircuitBreaker::new("test".to_string(), config);

        let result = breaker.execute(|| async {
            sleep(Duration::from_millis(200)).await;
            Ok::<i32, String>(42)
        }).await;
        
        match result {
            CircuitResult::Failure(CircuitError::Timeout) => {}
            other => panic!("Expected timeout, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_circuit_breaker_state_transitions() {
        let mut config = CircuitBreakerConfig::default();
        config.failure_threshold = 2;
        config.minimum_throughput = 2;
        config.sliding_window_size = 10;
        
        let breaker = CircuitBreaker::new("test".to_string(), config);

        // Initial state should be closed
        assert_eq!(*breaker.state.read().await, CircuitState::Closed);

        // Trigger failures to open circuit
        for _ in 0..3 {
            breaker.execute(|| async { Err::<i32, String>("error".to_string()) }).await;
        }

        // Circuit should now be open
        assert_eq!(*breaker.state.read().await, CircuitState::Open);

        // Next request should be rejected
        let result = breaker.execute(|| async { Ok::<i32, String>(42) }).await;
        match result {
            CircuitResult::Rejected(_) => {}
            other => panic!("Expected rejection, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_sliding_window() {
        let mut window = SlidingWindow::new(3);
        
        // Add some results
        window.add_result(CallResult {
            success: true,
            duration: Duration::from_millis(100),
            timestamp: Instant::now(),
            error: None,
        });
        
        window.add_result(CallResult {
            success: false,
            duration: Duration::from_millis(200),
            timestamp: Instant::now(),
            error: Some("error".to_string()),
        });
        
        window.add_result(CallResult {
            success: true,
            duration: Duration::from_millis(300),
            timestamp: Instant::now(),
            error: None,
        });

        assert_eq!(window.total_calls(), 3);
        assert_eq!(window.failure_rate(), 1.0 / 3.0);
        assert_eq!(window.slow_call_rate(Duration::from_millis(250)), 1.0 / 3.0);
    }

    #[tokio::test]
    async fn test_circuit_breaker_manager() {
        let manager = CircuitBreakerManager::new();
        let config = CircuitBreakerConfig::default();
        
        // Add a circuit breaker
        manager.add_breaker("test1".to_string(), config.clone()).await;
        
        // Should be able to retrieve it
        let breaker = manager.get_breaker("test1").await;
        assert!(breaker.is_some());
        
        // Add another one
        manager.add_breaker("test2".to_string(), config).await;
        
        // Should have two statuses
        let statuses = manager.get_all_statuses().await;
        assert_eq!(statuses.len(), 2);
        
        // Remove one
        assert!(manager.remove_breaker("test1").await);
        assert!(!manager.remove_breaker("nonexistent").await);
        
        // Should have one status now
        let statuses = manager.get_all_statuses().await;
        assert_eq!(statuses.len(), 1);
    }
}