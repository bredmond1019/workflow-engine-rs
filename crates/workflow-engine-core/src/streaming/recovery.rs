use futures_util::{Stream, StreamExt, stream};
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::{sleep, timeout};

use crate::error::WorkflowError;
use super::types::{StreamChunk, StreamConfig, StreamingError, StreamingProvider, StreamResponse};

/// Configuration for streaming error recovery
#[derive(Debug, Clone)]
pub struct StreamingRecoveryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial retry delay in milliseconds
    pub initial_retry_delay_ms: u64,
    /// Maximum retry delay in milliseconds (for exponential backoff)
    pub max_retry_delay_ms: u64,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Timeout for individual streaming operations in milliseconds
    pub operation_timeout_ms: u64,
    /// Whether to use jitter in retry delays
    pub use_jitter: bool,
    /// Circuit breaker failure threshold
    pub circuit_breaker_threshold: u32,
    /// Circuit breaker reset timeout in seconds
    pub circuit_breaker_reset_timeout_secs: u64,
}

impl Default for StreamingRecoveryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_retry_delay_ms: 1000,
            max_retry_delay_ms: 30000,
            backoff_multiplier: 2.0,
            operation_timeout_ms: 60000,
            use_jitter: true,
            circuit_breaker_threshold: 5,
            circuit_breaker_reset_timeout_secs: 60,
        }
    }
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

/// Circuit breaker for streaming operations
#[derive(Debug)]
pub struct StreamingCircuitBreaker {
    state: CircuitBreakerState,
    failure_count: u32,
    last_failure_time: Option<Instant>,
    config: StreamingRecoveryConfig,
}

impl StreamingCircuitBreaker {
    pub fn new(config: StreamingRecoveryConfig) -> Self {
        Self {
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            last_failure_time: None,
            config,
        }
    }

    /// Check if the circuit breaker allows the operation
    pub fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed().as_secs() >= self.config.circuit_breaker_reset_timeout_secs {
                        self.state = CircuitBreakerState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }

    /// Record a successful operation
    pub fn record_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitBreakerState::Closed;
        self.last_failure_time = None;
    }

    /// Record a failed operation
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());

        if self.failure_count >= self.config.circuit_breaker_threshold {
            self.state = CircuitBreakerState::Open;
        }
    }

    /// Get current state
    pub fn state(&self) -> &CircuitBreakerState {
        &self.state
    }
}

/// Recovery-enabled streaming provider wrapper
pub struct RecoveryStreamingProvider {
    inner: Arc<dyn StreamingProvider + Send + Sync>,
    circuit_breaker: Arc<tokio::sync::Mutex<StreamingCircuitBreaker>>,
    config: StreamingRecoveryConfig,
}

impl RecoveryStreamingProvider {
    pub fn new(
        provider: Arc<dyn StreamingProvider + Send + Sync>,
        config: StreamingRecoveryConfig,
    ) -> Self {
        let circuit_breaker = Arc::new(tokio::sync::Mutex::new(
            StreamingCircuitBreaker::new(config.clone())
        ));

        Self {
            inner: provider,
            circuit_breaker,
            config,
        }
    }

    /// Execute streaming with recovery logic
    pub async fn stream_with_recovery(
        &self,
        prompt: &str,
        stream_config: &StreamConfig,
    ) -> Pin<Box<dyn Stream<Item = Result<StreamChunk, WorkflowError>> + Send>> {
        let provider = self.inner.clone();
        let circuit_breaker = self.circuit_breaker.clone();
        let recovery_config = self.config.clone();
        let prompt = prompt.to_string();
        let stream_config = stream_config.clone();

        let stream = async_stream::stream! {
            let mut retry_count = 0;
            let mut retry_delay = recovery_config.initial_retry_delay_ms;

            loop {
                // Check circuit breaker
                {
                    let mut cb = circuit_breaker.lock().await;
                    if !cb.can_execute() {
                        yield Err(WorkflowError::ProcessingError {
                            message: "Circuit breaker is open - streaming service unavailable".to_string(),
                        });
                        return;
                    }
                }

                // Attempt streaming operation with timeout
                let operation_result = timeout(
                    Duration::from_millis(recovery_config.operation_timeout_ms),
                    Self::attempt_streaming(provider.clone(), &prompt, &stream_config)
                ).await;

                match operation_result {
                    Ok(stream_result) => {
                        match stream_result {
                            Ok(mut chunk_stream) => {
                                // Mark success in circuit breaker
                                {
                                    let mut cb = circuit_breaker.lock().await;
                                    cb.record_success();
                                }

                                // Stream all chunks
                                let mut chunk_count = 0;
                                let mut has_error = false;

                                while let Some(chunk_result) = chunk_stream.next().await {
                                    match chunk_result {
                                        Ok(chunk) => {
                                            chunk_count += 1;
                                            let is_final = chunk.is_final;
                                            yield Ok(chunk);
                                            
                                            if is_final {
                                                break;
                                            }
                                        }
                                        Err(e) => {
                                            has_error = true;
                                            tracing::warn!(
                                                error = %e,
                                                chunk_count = chunk_count,
                                                "Streaming error occurred during chunk processing"
                                            );
                                            
                                            // For mid-stream errors, try to recover gracefully
                                            if chunk_count > 0 {
                                                // Send an error chunk to indicate the issue
                                                yield Ok(StreamChunk::new(
                                                    format!("\n[Stream interrupted: {}]", e),
                                                    true
                                                ));
                                                return;
                                            } else {
                                                yield Err(e);
                                                has_error = true;
                                                break;
                                            }
                                        }
                                    }
                                }

                                if !has_error {
                                    return; // Successful completion
                                }
                            }
                            Err(e) => {
                                // Record failure in circuit breaker
                                {
                                    let mut cb = circuit_breaker.lock().await;
                                    cb.record_failure();
                                }

                                tracing::warn!(
                                    error = %e,
                                    retry_count = retry_count,
                                    max_retries = recovery_config.max_retries,
                                    "Streaming operation failed"
                                );

                                if retry_count >= recovery_config.max_retries {
                                    yield Err(WorkflowError::ProcessingError {
                                        message: format!("Streaming failed after {} retries: {}", retry_count, e),
                                    });
                                    return;
                                }

                                retry_count += 1;
                            }
                        }
                    }
                    Err(_timeout_err) => {
                        // Record timeout failure
                        {
                            let mut cb = circuit_breaker.lock().await;
                            cb.record_failure();
                        }

                        tracing::warn!(
                            timeout_ms = recovery_config.operation_timeout_ms,
                            retry_count = retry_count,
                            "Streaming operation timed out"
                        );

                        if retry_count >= recovery_config.max_retries {
                            yield Err(WorkflowError::ProcessingError {
                                message: format!("Streaming timed out after {} retries", retry_count),
                            });
                            return;
                        }

                        retry_count += 1;
                    }
                }

                // Calculate delay for next retry
                let delay_ms = if recovery_config.use_jitter {
                    Self::calculate_jittered_delay(retry_delay, 0.1)
                } else {
                    retry_delay
                };

                tracing::info!(
                    delay_ms = delay_ms,
                    retry_count = retry_count,
                    "Retrying streaming operation after delay"
                );

                sleep(Duration::from_millis(delay_ms)).await;

                // Update retry delay for exponential backoff
                retry_delay = std::cmp::min(
                    (retry_delay as f64 * recovery_config.backoff_multiplier) as u64,
                    recovery_config.max_retry_delay_ms,
                );
            }
        };

        Box::pin(stream)
    }

    /// Attempt a single streaming operation
    async fn attempt_streaming(
        provider: Arc<dyn StreamingProvider + Send + Sync>,
        prompt: &str,
        config: &StreamConfig,
    ) -> Result<StreamResponse, WorkflowError> {
        // For now, directly call the provider's stream_response
        // In a more sophisticated implementation, we might add additional
        // connection validation, health checks, etc.
        let mut stream = provider.stream_response(prompt, config);
        
        // Test the stream by trying to get the first item
        // If it immediately fails, return an error instead of the stream
        if let Some(first_result) = stream.next().await {
            match first_result {
                Ok(first_chunk) => {
                    // Prepend the first chunk back to the stream
                    let first_stream = stream::once(async move { Ok(first_chunk) });
                    let combined_stream = first_stream.chain(stream);
                    Ok(Box::pin(combined_stream))
                }
                Err(e) => {
                    // Return the error immediately so retry logic can handle it
                    Err(e)
                }
            }
        } else {
            // Empty stream
            Ok(stream)
        }
    }

    /// Calculate jittered delay to avoid thundering herd
    fn calculate_jittered_delay(base_delay_ms: u64, jitter_factor: f64) -> u64 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let jitter = rng.gen_range(-jitter_factor..=jitter_factor);
        let jittered_delay = base_delay_ms as f64 * (1.0 + jitter);
        std::cmp::max(1, jittered_delay as u64)
    }

    /// Get circuit breaker state
    pub async fn circuit_breaker_state(&self) -> CircuitBreakerState {
        let cb = self.circuit_breaker.lock().await;
        cb.state().clone()
    }

    /// Reset circuit breaker (for testing/admin purposes)
    pub async fn reset_circuit_breaker(&self) {
        let mut cb = self.circuit_breaker.lock().await;
        cb.state = CircuitBreakerState::Closed;
        cb.failure_count = 0;
        cb.last_failure_time = None;
    }
}

/// Stream reconnection manager for WebSocket connections
pub struct StreamReconnectionManager {
    config: StreamingRecoveryConfig,
    connection_attempts: u32,
    last_connection_time: Option<Instant>,
}

impl StreamReconnectionManager {
    pub fn new(config: StreamingRecoveryConfig) -> Self {
        Self {
            config,
            connection_attempts: 0,
            last_connection_time: None,
        }
    }

    /// Check if reconnection should be attempted
    pub fn should_reconnect(&self) -> bool {
        self.connection_attempts < self.config.max_retries
    }

    /// Get delay before next reconnection attempt
    pub fn get_reconnection_delay(&self) -> Duration {
        let base_delay = self.config.initial_retry_delay_ms;
        let exponential_delay = base_delay * (self.config.backoff_multiplier.powi(self.connection_attempts as i32)) as u64;
        let capped_delay = std::cmp::min(exponential_delay, self.config.max_retry_delay_ms);
        
        if self.config.use_jitter {
            Duration::from_millis(Self::calculate_jittered_delay(capped_delay, 0.1))
        } else {
            Duration::from_millis(capped_delay)
        }
    }

    /// Record connection attempt
    pub fn record_attempt(&mut self) {
        self.connection_attempts += 1;
        self.last_connection_time = Some(Instant::now());
    }

    /// Record successful connection
    pub fn record_success(&mut self) {
        self.connection_attempts = 0;
        self.last_connection_time = Some(Instant::now());
    }

    /// Reset connection state
    pub fn reset(&mut self) {
        self.connection_attempts = 0;
        self.last_connection_time = None;
    }

    fn calculate_jittered_delay(base_delay_ms: u64, jitter_factor: f64) -> u64 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let jitter = rng.gen_range(-jitter_factor..=jitter_factor);
        let jittered_delay = base_delay_ms as f64 * (1.0 + jitter);
        std::cmp::max(1, jittered_delay as u64)
    }
}

/// Recovery statistics
#[derive(Debug, Clone, Default)]
pub struct StreamingRecoveryStats {
    pub total_attempts: u64,
    pub successful_streams: u64,
    pub failed_streams: u64,
    pub retries_performed: u64,
    pub circuit_breaker_trips: u64,
    pub timeouts: u64,
    pub mid_stream_errors: u64,
}

impl StreamingRecoveryStats {
    pub fn success_rate(&self) -> f64 {
        if self.total_attempts == 0 {
            0.0
        } else {
            self.successful_streams as f64 / self.total_attempts as f64
        }
    }

    pub fn average_retries_per_attempt(&self) -> f64 {
        if self.total_attempts == 0 {
            0.0
        } else {
            self.retries_performed as f64 / self.total_attempts as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming::types::{StreamMetadata, StreamingProvider};
    use futures_util::stream;

    struct MockStreamingProvider {
        should_fail: bool,
        fail_count: std::sync::Arc<std::sync::atomic::AtomicU32>,
    }

    impl MockStreamingProvider {
        fn new(should_fail: bool) -> Self {
            Self {
                should_fail,
                fail_count: std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0)),
            }
        }
    }

    impl StreamingProvider for MockStreamingProvider {
        fn stream_response(&self, _prompt: &str, _config: &StreamConfig) -> StreamResponse {
            if self.should_fail {
                let count = self.fail_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if count < 2 {
                    // Fail first two attempts
                    let error_stream = stream::once(async {
                        Err(WorkflowError::ApiError {
                            message: "Mock failure".to_string(),
                        })
                    });
                    return Box::pin(error_stream);
                }
            }

            // Success case
            let chunks = vec![
                Ok(StreamChunk::with_metadata(
                    "Hello".to_string(),
                    false,
                    StreamMetadata::new("mock-model".to_string(), "mock".to_string()),
                )),
                Ok(StreamChunk::with_metadata(
                    " world!".to_string(),
                    true,
                    StreamMetadata::new("mock-model".to_string(), "mock".to_string()),
                )),
            ];
            let success_stream = stream::iter(chunks);
            Box::pin(success_stream)
        }

        fn provider_name(&self) -> &str {
            "mock"
        }

        fn supports_streaming(&self) -> bool {
            true
        }
    }

    #[tokio::test]
    async fn test_circuit_breaker() {
        let config = StreamingRecoveryConfig {
            circuit_breaker_threshold: 2,
            ..Default::default()
        };
        let mut cb = StreamingCircuitBreaker::new(config);

        // Initially closed
        assert_eq!(cb.state(), &CircuitBreakerState::Closed);
        assert!(cb.can_execute());

        // Record failures
        cb.record_failure();
        assert_eq!(cb.state(), &CircuitBreakerState::Closed);
        assert!(cb.can_execute());

        cb.record_failure();
        assert_eq!(cb.state(), &CircuitBreakerState::Open);
        assert!(!cb.can_execute());

        // Record success should reset
        cb.record_success();
        assert_eq!(cb.state(), &CircuitBreakerState::Closed);
        assert!(cb.can_execute());
    }

    #[tokio::test]
    async fn test_recovery_provider_success() {
        let mock_provider = Arc::new(MockStreamingProvider::new(false));
        let config = StreamingRecoveryConfig::default();
        let recovery_provider = RecoveryStreamingProvider::new(mock_provider, config);

        let stream_config = StreamConfig::default();
        let mut chunk_stream = recovery_provider
            .stream_with_recovery("test prompt", &stream_config)
            .await;

        let mut chunks = Vec::new();
        while let Some(chunk_result) = chunk_stream.next().await {
            chunks.push(chunk_result.unwrap());
        }

        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].content, "Hello");
        assert_eq!(chunks[1].content, " world!");
        assert!(chunks[1].is_final);
    }

    #[tokio::test]
    async fn test_recovery_provider_with_retries() {
        let mock_provider = Arc::new(MockStreamingProvider::new(true));
        let config = StreamingRecoveryConfig {
            max_retries: 3,
            initial_retry_delay_ms: 10, // Short delay for testing
            ..Default::default()
        };
        let recovery_provider = RecoveryStreamingProvider::new(mock_provider, config);

        let stream_config = StreamConfig::default();
        let mut chunk_stream = recovery_provider
            .stream_with_recovery("test prompt", &stream_config)
            .await;

        let mut chunks = Vec::new();
        while let Some(chunk_result) = chunk_stream.next().await {
            chunks.push(chunk_result.unwrap());
        }

        // Should succeed after retries
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].content, "Hello");
        assert_eq!(chunks[1].content, " world!");
    }

    #[test]
    fn test_reconnection_manager() {
        let config = StreamingRecoveryConfig {
            max_retries: 3,
            initial_retry_delay_ms: 1000,
            backoff_multiplier: 2.0,
            ..Default::default()
        };
        let mut manager = StreamReconnectionManager::new(config);

        // Should allow reconnection initially
        assert!(manager.should_reconnect());

        // Record attempts
        manager.record_attempt();
        assert!(manager.should_reconnect());
        assert_eq!(manager.connection_attempts, 1);

        // Check delay increases
        let delay1 = manager.get_reconnection_delay();
        manager.record_attempt();
        let delay2 = manager.get_reconnection_delay();
        assert!(delay2 > delay1);

        // Reset should clear state
        manager.reset();
        assert_eq!(manager.connection_attempts, 0);
    }
}