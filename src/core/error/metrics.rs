//! # Error Metrics and Monitoring
//!
//! This module provides error metrics collection and monitoring integration
//! for tracking error rates, patterns, and system health.

use super::{WorkflowError, ErrorCategory, ErrorSeverity};
use prometheus::{Counter, CounterVec, Histogram, HistogramVec, Registry};
use std::sync::Arc;
use lazy_static::lazy_static;

lazy_static! {
    /// Global error metrics
    static ref ERROR_METRICS: ErrorMetrics = ErrorMetrics::new();
}

/// Error metrics collector
pub struct ErrorMetrics {
    /// Total error counter by category
    pub errors_by_category: CounterVec,
    /// Total error counter by severity
    pub errors_by_severity: CounterVec,
    /// Error counter by specific error type
    pub errors_by_type: CounterVec,
    /// Retry attempts counter
    pub retry_attempts: Counter,
    /// Successful retries counter
    pub retry_successes: Counter,
    /// Circuit breaker state changes
    pub circuit_breaker_transitions: CounterVec,
    /// Recovery attempts by strategy
    pub recovery_attempts: CounterVec,
    /// Error handling duration
    pub error_handling_duration: Histogram,
    /// Error rate by time window
    pub error_rate_window: HistogramVec,
}

impl ErrorMetrics {
    /// Create new error metrics
    fn new() -> Self {
        let errors_by_category = CounterVec::new(
            prometheus::Opts::new(
                "workflow_errors_by_category_total",
                "Total number of errors by category"
            ),
            &["category"]
        ).expect("Failed to create errors_by_category metric");
        
        let errors_by_severity = CounterVec::new(
            prometheus::Opts::new(
                "workflow_errors_by_severity_total",
                "Total number of errors by severity"
            ),
            &["severity"]
        ).expect("Failed to create errors_by_severity metric");
        
        let errors_by_type = CounterVec::new(
            prometheus::Opts::new(
                "workflow_errors_by_type_total",
                "Total number of errors by specific type"
            ),
            &["error_type", "error_code"]
        ).expect("Failed to create errors_by_type metric");
        
        let retry_attempts = Counter::new(
            "workflow_retry_attempts_total",
            "Total number of retry attempts"
        ).expect("Failed to create retry_attempts metric");
        
        let retry_successes = Counter::new(
            "workflow_retry_successes_total",
            "Total number of successful retries"
        ).expect("Failed to create retry_successes metric");
        
        let circuit_breaker_transitions = CounterVec::new(
            prometheus::Opts::new(
                "workflow_circuit_breaker_transitions_total",
                "Circuit breaker state transitions"
            ),
            &["from_state", "to_state", "service"]
        ).expect("Failed to create circuit_breaker_transitions metric");
        
        let recovery_attempts = CounterVec::new(
            prometheus::Opts::new(
                "workflow_recovery_attempts_total",
                "Recovery attempts by strategy"
            ),
            &["strategy", "success"]
        ).expect("Failed to create recovery_attempts metric");
        
        let error_handling_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "workflow_error_handling_duration_seconds",
                "Time spent handling errors"
            )
            .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0])
        ).expect("Failed to create error_handling_duration metric");
        
        let error_rate_window = HistogramVec::new(
            prometheus::HistogramOpts::new(
                "workflow_error_rate",
                "Error rate in sliding time windows"
            )
            .buckets(vec![0.0, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0]),
            &["window", "category"]
        ).expect("Failed to create error_rate_window metric");
        
        Self {
            errors_by_category,
            errors_by_severity,
            errors_by_type,
            retry_attempts,
            retry_successes,
            circuit_breaker_transitions,
            recovery_attempts,
            error_handling_duration,
            error_rate_window,
        }
    }
    
    /// Register all metrics with Prometheus
    pub fn register(&self, registry: &Registry) -> Result<(), prometheus::Error> {
        registry.register(Box::new(self.errors_by_category.clone()))?;
        registry.register(Box::new(self.errors_by_severity.clone()))?;
        registry.register(Box::new(self.errors_by_type.clone()))?;
        registry.register(Box::new(self.retry_attempts.clone()))?;
        registry.register(Box::new(self.retry_successes.clone()))?;
        registry.register(Box::new(self.circuit_breaker_transitions.clone()))?;
        registry.register(Box::new(self.recovery_attempts.clone()))?;
        registry.register(Box::new(self.error_handling_duration.clone()))?;
        registry.register(Box::new(self.error_rate_window.clone()))?;
        Ok(())
    }
}

/// Get global error metrics
pub fn metrics() -> &'static ErrorMetrics {
    &ERROR_METRICS
}

/// Record an error occurrence
pub fn record_error(error: &WorkflowError, category: ErrorCategory, severity: ErrorSeverity) {
    let error_type = match error {
        WorkflowError::CycleDetected => "CycleDetected",
        WorkflowError::UnreachableNodes { .. } => "UnreachableNodes",
        WorkflowError::InvalidRouter { .. } => "InvalidRouter",
        WorkflowError::ProcessingError { .. } => "ProcessingError",
        WorkflowError::NodeNotFound { .. } => "NodeNotFound",
        WorkflowError::SerializationError { .. } => "SerializationError",
        WorkflowError::DeserializationError { .. } => "DeserializationError",
        WorkflowError::DatabaseError { .. } => "DatabaseError",
        WorkflowError::WorkflowTypeMismatch { .. } => "WorkflowTypeMismatch",
        WorkflowError::ApiError { .. } => "ApiError",
        WorkflowError::RuntimeError { .. } => "RuntimeError",
        WorkflowError::MCPError { .. } => "MCPError",
        WorkflowError::MCPConnectionError { .. } => "MCPConnectionError",
        WorkflowError::MCPProtocolError { .. } => "MCPProtocolError",
        WorkflowError::MCPTransportError { .. } => "MCPTransportError",
        WorkflowError::ValidationError { .. } => "ValidationError",
        WorkflowError::RegistryError { .. } => "RegistryError",
        WorkflowError::InvalidStepType(_) => "InvalidStepType",
        WorkflowError::InvalidInput(_) => "InvalidInput",
        WorkflowError::CrossSystemError(_) => "CrossSystemError",
        WorkflowError::ConfigurationError(_) => "ConfigurationError",
    };
    
    let error_code = get_error_code(error);
    
    metrics().errors_by_category
        .with_label_values(&[&format!("{:?}", category)])
        .inc();
    
    metrics().errors_by_severity
        .with_label_values(&[&format!("{:?}", severity)])
        .inc();
    
    metrics().errors_by_type
        .with_label_values(&[error_type, &error_code])
        .inc();
}

/// Record retry attempt
pub fn record_retry_attempt() {
    metrics().retry_attempts.inc();
}

/// Record successful retry
pub fn record_retry_success() {
    metrics().retry_successes.inc();
}

/// Record circuit breaker state transition
pub fn record_circuit_breaker_transition(
    from_state: &str,
    to_state: &str,
    service: &str,
) {
    metrics().circuit_breaker_transitions
        .with_label_values(&[from_state, to_state, service])
        .inc();
}

/// Record recovery attempt
pub fn record_recovery_attempt(strategy: &str, success: bool) {
    metrics().recovery_attempts
        .with_label_values(&[strategy, if success { "true" } else { "false" }])
        .inc();
}

/// Record error handling duration
pub fn record_error_handling_duration(duration: std::time::Duration) {
    metrics().error_handling_duration
        .observe(duration.as_secs_f64());
}

/// Get error code for specific error
fn get_error_code(error: &WorkflowError) -> String {
    match error {
        WorkflowError::MCPConnectionError { .. } => "MCP_CONN_001",
        WorkflowError::MCPTransportError { .. } => "MCP_TRANS_001",
        WorkflowError::ApiError { .. } => "API_001",
        WorkflowError::DatabaseError { .. } => "DB_001",
        WorkflowError::CycleDetected => "WF_CYCLE_001",
        WorkflowError::UnreachableNodes { .. } => "WF_UNREACH_001",
        WorkflowError::InvalidRouter { .. } => "WF_ROUTER_001",
        WorkflowError::ValidationError { .. } => "VAL_001",
        WorkflowError::InvalidInput(_) => "INPUT_001",
        WorkflowError::NodeNotFound { .. } => "NODE_404",
        WorkflowError::ProcessingError { .. } => "PROC_001",
        WorkflowError::SerializationError { .. } => "SER_001",
        WorkflowError::DeserializationError { .. } => "DESER_001",
        _ => "UNKNOWN_001",
    }.to_string()
}

/// Error rate calculator for monitoring
pub struct ErrorRateCalculator {
    window_size: std::time::Duration,
    errors: Arc<tokio::sync::Mutex<Vec<(std::time::Instant, ErrorCategory)>>>,
}

impl ErrorRateCalculator {
    /// Create new error rate calculator
    pub fn new(window_size: std::time::Duration) -> Self {
        Self {
            window_size,
            errors: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }
    
    /// Record an error
    pub async fn record_error(&self, category: ErrorCategory) {
        let mut errors = self.errors.lock().await;
        let now = std::time::Instant::now();
        
        // Remove old errors outside window
        errors.retain(|(timestamp, _)| now.duration_since(*timestamp) < self.window_size);
        
        // Add new error
        errors.push((now, category));
    }
    
    /// Calculate error rate
    pub async fn error_rate(&self) -> f64 {
        let errors = self.errors.lock().await;
        let now = std::time::Instant::now();
        
        // Count errors in window
        let error_count = errors.iter()
            .filter(|(timestamp, _)| now.duration_since(*timestamp) < self.window_size)
            .count();
        
        // Calculate rate (errors per second)
        error_count as f64 / self.window_size.as_secs_f64()
    }
    
    /// Get error rate by category
    pub async fn error_rate_by_category(&self) -> std::collections::HashMap<ErrorCategory, f64> {
        let errors = self.errors.lock().await;
        let now = std::time::Instant::now();
        
        let mut category_counts = std::collections::HashMap::new();
        
        for (timestamp, category) in errors.iter() {
            if now.duration_since(*timestamp) < self.window_size {
                *category_counts.entry(*category).or_insert(0) += 1;
            }
        }
        
        category_counts.into_iter()
            .map(|(category, count)| {
                (category, count as f64 / self.window_size.as_secs_f64())
            })
            .collect()
    }
}

/// Error pattern detector for anomaly detection
pub struct ErrorPatternDetector {
    threshold: f64,
    calculator: ErrorRateCalculator,
}

impl ErrorPatternDetector {
    /// Create new pattern detector
    pub fn new(threshold: f64, window_size: std::time::Duration) -> Self {
        Self {
            threshold,
            calculator: ErrorRateCalculator::new(window_size),
        }
    }
    
    /// Check if error rate is anomalous
    pub async fn is_anomalous(&self) -> bool {
        self.calculator.error_rate().await > self.threshold
    }
    
    /// Get anomalous categories
    pub async fn anomalous_categories(&self) -> Vec<ErrorCategory> {
        self.calculator.error_rate_by_category().await
            .into_iter()
            .filter(|(_, rate)| *rate > self.threshold)
            .map(|(category, _)| category)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_metrics_creation() {
        let metrics = ErrorMetrics::new();
        
        // Test that metrics can be created without panic
        metrics.errors_by_category
            .with_label_values(&["Transient"])
            .inc();
        
        assert_eq!(
            metrics.errors_by_category
                .with_label_values(&["Transient"])
                .get(),
            1.0
        );
    }
    
    #[tokio::test]
    async fn test_error_rate_calculator() {
        let calculator = ErrorRateCalculator::new(std::time::Duration::from_secs(10));
        
        // Record some errors
        calculator.record_error(ErrorCategory::Transient).await;
        calculator.record_error(ErrorCategory::Permanent).await;
        calculator.record_error(ErrorCategory::Transient).await;
        
        // Check rate
        let rate = calculator.error_rate().await;
        assert!(rate > 0.0);
        
        // Check by category
        let rates = calculator.error_rate_by_category().await;
        assert_eq!(rates.len(), 2);
        assert!(rates.contains_key(&ErrorCategory::Transient));
        assert!(rates.contains_key(&ErrorCategory::Permanent));
    }
    
    #[tokio::test]
    async fn test_error_pattern_detector() {
        let detector = ErrorPatternDetector::new(1.0, std::time::Duration::from_secs(1));
        
        // Initially should not be anomalous
        assert!(!detector.is_anomalous().await);
        
        // Record many errors quickly
        for _ in 0..10 {
            detector.calculator.record_error(ErrorCategory::Transient).await;
        }
        
        // Should now be anomalous
        assert!(detector.is_anomalous().await);
        
        let anomalous = detector.anomalous_categories().await;
        assert!(anomalous.contains(&ErrorCategory::Transient));
    }
}