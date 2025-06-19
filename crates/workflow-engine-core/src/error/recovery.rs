//! # Error Recovery and Fallback Mechanisms
//!
//! This module provides strategies for graceful degradation and error recovery,
//! including fallback values, alternative operations, and recovery patterns.

use super::{WorkflowError, ErrorCategory};
// use crate::db::events::EventError;  // Commented out - db moved to API crate
use std::future::Future;
use serde::{Serialize, Deserialize};
use serde_json::Value;

/// Recovery strategy for handling errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    /// Return a fallback value
    Fallback(Value),
    /// Retry with modified parameters
    RetryWithModification,
    /// Use cached result
    UseCache,
    /// Degrade to simpler operation
    Degrade,
    /// Fail fast
    FailFast,
    /// Custom recovery logic
    Custom(String),
}

impl RecoveryStrategy {
    /*
    /// Apply recovery strategy to an error
    // Commented out to avoid EventError dependency (db moved to API crate)
    pub fn recover_from_error<T>(&self, _error: EventError) -> Result<T, WorkflowError> 
    where 
        T: Default,
    {
        match self {
            RecoveryStrategy::Fallback(_) => Ok(T::default()),
            RecoveryStrategy::RetryWithModification => Ok(T::default()),
            RecoveryStrategy::UseCache => Ok(T::default()),
            RecoveryStrategy::Degrade => Ok(T::default()),
            RecoveryStrategy::FailFast => Err(WorkflowError::ProcessingError {
                message: "Recovery strategy failed fast".to_string(),
            }),
            RecoveryStrategy::Custom(_) => Ok(T::default()),
        }
    }
    */
}

/// Trait for types that can provide fallback values
pub trait FallbackValue {
    /// Get the fallback value
    fn fallback() -> Self;
}

impl FallbackValue for String {
    fn fallback() -> Self {
        String::new()
    }
}

impl FallbackValue for Value {
    fn fallback() -> Self {
        Value::Null
    }
}

impl<T: Default> FallbackValue for Vec<T> {
    fn fallback() -> Self {
        Vec::new()
    }
}

impl<T: FallbackValue> FallbackValue for Option<T> {
    fn fallback() -> Self {
        None
    }
}

/// Recovery context for error handling
#[derive(Debug)]
pub struct RecoveryContext {
    /// Original error
    pub error: WorkflowError,
    /// Number of recovery attempts
    pub attempt: u32,
    /// Recovery strategy to use
    pub strategy: RecoveryStrategy,
    /// Additional context
    pub context: std::collections::HashMap<String, Value>,
}

impl RecoveryContext {
    /// Create new recovery context
    pub fn new(error: WorkflowError) -> Self {
        Self {
            error,
            attempt: 0,
            strategy: RecoveryStrategy::FailFast,
            context: std::collections::HashMap::new(),
        }
    }
    
    /// Set recovery strategy
    pub fn with_strategy(mut self, strategy: RecoveryStrategy) -> Self {
        self.strategy = strategy;
        self
    }
    
    /// Add context
    pub fn with_context(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.context.insert(key.into(), json_value);
        }
        self
    }
}

/// Execute operation with fallback
pub async fn with_fallback<F, Fut, T>(
    operation: F,
    fallback: T,
) -> T
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, WorkflowError>>,
{
    match operation().await {
        Ok(value) => value,
        Err(error) => {
            tracing::warn!(
                error = %error,
                "Operation failed, using fallback value"
            );
            fallback
        }
    }
}

/// Execute operation with fallback function
pub async fn with_fallback_fn<F, Fut, G, T>(
    operation: F,
    fallback_fn: G,
) -> Result<T, WorkflowError>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, WorkflowError>>,
    G: FnOnce(WorkflowError) -> Result<T, WorkflowError>,
{
    match operation().await {
        Ok(value) => Ok(value),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "Operation failed, attempting fallback"
            );
            fallback_fn(error)
        }
    }
}

/// Recovery handler trait
pub trait RecoveryHandler: Send + Sync {
    /// Handle recovery for an error
    fn handle_recovery(
        &self,
        context: &RecoveryContext,
    ) -> RecoveryStrategy;
    
    /// Check if recovery should be attempted
    fn should_recover(&self, error: &WorkflowError) -> bool;
}

/// Default recovery handler
pub struct DefaultRecoveryHandler;

impl RecoveryHandler for DefaultRecoveryHandler {
    fn handle_recovery(&self, context: &RecoveryContext) -> RecoveryStrategy {
        match context.error {
            WorkflowError::ApiError { .. } |
            WorkflowError::MCPConnectionError { .. } => {
                if context.attempt < 3 {
                    RecoveryStrategy::RetryWithModification
                } else {
                    RecoveryStrategy::UseCache
                }
            }
            WorkflowError::ValidationError { .. } => RecoveryStrategy::FailFast,
            _ => RecoveryStrategy::Degrade,
        }
    }
    
    fn should_recover(&self, error: &WorkflowError) -> bool {
        !matches!(
            error,
            WorkflowError::CycleDetected |
            WorkflowError::UnreachableNodes { .. } |
            WorkflowError::InvalidRouter { .. }
        )
    }
}

/// Graceful degradation builder
pub struct DegradationBuilder<T> {
    operations: Vec<std::pin::Pin<Box<dyn Future<Output = Result<T, WorkflowError>> + Send>>>,
}

impl<T> DegradationBuilder<T> {
    /// Create new degradation builder
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }
    
    /// Add primary operation
    pub fn primary<F, Fut>(mut self, operation: F) -> Self
    where
        F: FnOnce() -> Fut + 'static,
        Fut: Future<Output = Result<T, WorkflowError>> + Send + 'static,
    {
        self.operations.insert(0, Box::pin(operation()));
        self
    }
    
    /// Add fallback operation
    pub fn fallback<F, Fut>(mut self, operation: F) -> Self
    where
        F: FnOnce() -> Fut + 'static,
        Fut: Future<Output = Result<T, WorkflowError>> + Send + 'static,
    {
        self.operations.push(Box::pin(operation()));
        self
    }
    
    /// Execute with graceful degradation
    pub async fn execute(self) -> Result<T, WorkflowError> {
        let mut last_error = None;
        
        for (i, operation) in self.operations.into_iter().enumerate() {
            match operation.await {
                Ok(result) => {
                    if i > 0 {
                        tracing::info!(
                            fallback_level = i,
                            "Operation succeeded with degradation"
                        );
                    }
                    return Ok(result);
                }
                Err(error) => {
                    tracing::warn!(
                        error = %error,
                        level = i,
                        "Operation failed, trying next level"
                    );
                    last_error = Some(error);
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| {
            WorkflowError::RuntimeError {
                message: "All operations failed".to_string(),
            }
        }))
    }
}

/// Cache-based recovery
pub struct CacheRecovery<T> {
    cache: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, (T, std::time::Instant)>>>,
    ttl: std::time::Duration,
}

impl<T: Clone> CacheRecovery<T> {
    /// Create new cache recovery
    pub fn new(ttl: std::time::Duration) -> Self {
        Self {
            cache: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            ttl,
        }
    }
    
    /// Execute with cache fallback
    pub async fn execute<F, Fut>(
        &self,
        key: &str,
        operation: F,
    ) -> Result<T, WorkflowError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, WorkflowError>>,
    {
        // Try the operation first
        match operation().await {
            Ok(value) => {
                // Update cache on success
                let mut cache = self.cache.write().await;
                cache.insert(key.to_string(), (value.clone(), std::time::Instant::now()));
                Ok(value)
            }
            Err(error) => {
                // Check cache on failure
                let cache = self.cache.read().await;
                if let Some((cached_value, timestamp)) = cache.get(key) {
                    if timestamp.elapsed() < self.ttl {
                        tracing::info!(
                            key = key,
                            age_seconds = timestamp.elapsed().as_secs(),
                            "Using cached value after error"
                        );
                        return Ok(cached_value.clone());
                    }
                }
                Err(error)
            }
        }
    }
    
    /// Invalidate cache entry
    pub async fn invalidate(&self, key: &str) {
        let mut cache = self.cache.write().await;
        cache.remove(key);
    }
    
    /// Clear all cache entries
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
}

/// Recovery metrics
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RecoveryMetrics {
    pub total_recoveries: u64,
    pub successful_recoveries: u64,
    pub failed_recoveries: u64,
    pub fallback_used: u64,
    pub cache_hits: u64,
    pub degradation_used: u64,
}

impl RecoveryMetrics {
    /// Record successful recovery
    pub fn record_success(&mut self, strategy: &RecoveryStrategy) {
        self.total_recoveries += 1;
        self.successful_recoveries += 1;
        
        match strategy {
            RecoveryStrategy::Fallback(_) => self.fallback_used += 1,
            RecoveryStrategy::UseCache => self.cache_hits += 1,
            RecoveryStrategy::Degrade => self.degradation_used += 1,
            _ => {}
        }
    }
    
    /// Record failed recovery
    pub fn record_failure(&mut self) {
        self.total_recoveries += 1;
        self.failed_recoveries += 1;
    }
    
    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_recoveries == 0 {
            0.0
        } else {
            self.successful_recoveries as f64 / self.total_recoveries as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_with_fallback() {
        let result = with_fallback(
            || async {
                Err::<String, _>(WorkflowError::ApiError {
                    message: "Service unavailable".to_string(),
                })
            },
            "fallback_value".to_string(),
        ).await;
        
        assert_eq!(result, "fallback_value");
    }
    
    #[tokio::test]
    async fn test_with_fallback_fn() {
        let result = with_fallback_fn(
            || async {
                Err::<String, _>(WorkflowError::ApiError {
                    message: "Service unavailable".to_string(),
                })
            },
            |_error| Ok("recovered_value".to_string()),
        ).await;
        
        assert_eq!(result.unwrap(), "recovered_value");
    }
    
    #[tokio::test]
    async fn test_cache_recovery() {
        let cache = CacheRecovery::new(std::time::Duration::from_secs(60));
        
        // First call succeeds and caches
        let result = cache.execute("test_key", || async {
            Ok::<_, WorkflowError>("cached_value".to_string())
        }).await;
        assert_eq!(result.unwrap(), "cached_value");
        
        // Second call fails but uses cache
        let result = cache.execute("test_key", || async {
            Err::<String, _>(WorkflowError::ApiError {
                message: "Service error".to_string(),
            })
        }).await;
        assert_eq!(result.unwrap(), "cached_value");
    }
    
    #[test]
    fn test_recovery_metrics() {
        let mut metrics = RecoveryMetrics::default();
        
        metrics.record_success(&RecoveryStrategy::Fallback(Value::Null));
        metrics.record_success(&RecoveryStrategy::UseCache);
        metrics.record_failure();
        
        assert_eq!(metrics.total_recoveries, 3);
        assert_eq!(metrics.successful_recoveries, 2);
        assert_eq!(metrics.failed_recoveries, 1);
        assert_eq!(metrics.fallback_used, 1);
        assert_eq!(metrics.cache_hits, 1);
        assert_eq!(metrics.success_rate(), 2.0 / 3.0);
    }
}