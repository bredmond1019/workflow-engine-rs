//! Graceful degradation utilities for the Knowledge Graph service
//! 
//! Provides utilities for handling partial failures and degraded service
//! scenarios with fallback strategies and partial result handling.

use crate::error::{Result, OperationFailure, partial_result};
#[cfg(test)]
use crate::error::KnowledgeGraphError;
use crate::graph::Concept;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Instant;
use tracing::{warn, info, debug};

/// Strategy for handling degraded service scenarios
#[derive(Debug, Clone)]
pub enum DegradationStrategy {
    /// Return partial results when available
    PartialResults {
        min_success_ratio: f64,
        include_error_details: bool,
    },
    /// Use cached/fallback data
    FallbackData {
        max_age: std::time::Duration,
        warn_on_use: bool,
    },
    /// Return empty results but don't fail
    EmptyResults {
        log_level: DegradationLogLevel,
    },
    /// Fail fast - propagate errors immediately
    FailFast,
}

/// Log level for degradation events
#[derive(Debug, Clone)]
pub enum DegradationLogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl Default for DegradationStrategy {
    fn default() -> Self {
        Self::PartialResults {
            min_success_ratio: 0.5,
            include_error_details: true,
        }
    }
}

/// Graceful degradation handler
pub struct GracefulDegradationHandler {
    strategy: DegradationStrategy,
    fallback_cache: HashMap<String, (Value, Instant)>,
}

impl GracefulDegradationHandler {
    /// Create a new degradation handler
    pub fn new(strategy: DegradationStrategy) -> Self {
        Self {
            strategy,
            fallback_cache: HashMap::new(),
        }
    }

    /// Handle a batch operation with graceful degradation
    pub fn handle_batch_operation<T, F>(
        &mut self,
        items: Vec<T>,
        operation: F,
        operation_name: &str,
    ) -> Result<Vec<T>>
    where
        F: Fn(&T) -> Result<T>,
        T: Clone + std::fmt::Debug + serde::Serialize + serde::de::DeserializeOwned,
    {
        let mut successful_results = Vec::new();
        let mut failures = Vec::new();
        
        for (index, item) in items.iter().enumerate() {
            match operation(item) {
                Ok(result) => successful_results.push(result),
                Err(e) => {
                    warn!("Operation {} failed for item {}: {}", operation_name, index, e);
                    failures.push(OperationFailure {
                        operation_id: format!("{}_{}", operation_name, index),
                        error_message: e.to_string(),
                        error_type: "BatchOperationError".to_string(),
                        timestamp: Instant::now(),
                    });
                }
            }
        }

        self.apply_degradation_strategy(
            successful_results,
            failures,
            items.len(),
            operation_name,
        )
    }

    /// Handle concept parsing with graceful degradation
    pub fn handle_concept_parsing(
        &mut self,
        concepts_data: &[Value],
        parser: impl Fn(&Value) -> Result<Concept>,
    ) -> Result<Vec<Concept>> {
        let mut successful_concepts = Vec::new();
        let mut failures = Vec::new();

        for (index, concept_data) in concepts_data.iter().enumerate() {
            match parser(concept_data) {
                Ok(concept) => successful_concepts.push(concept),
                Err(e) => {
                    debug!("Failed to parse concept at index {}: {}", index, e);
                    failures.push(OperationFailure {
                        operation_id: format!("parse_concept_{}", index),
                        error_message: e.to_string(),
                        error_type: "ConceptParseError".to_string(),
                        timestamp: Instant::now(),
                    });
                }
            }
        }

        self.apply_degradation_strategy(
            successful_concepts,
            failures,
            concepts_data.len(),
            "concept_parsing",
        )
    }

    /// Apply the configured degradation strategy
    fn apply_degradation_strategy<T>(
        &mut self,
        successful_results: Vec<T>,
        failures: Vec<OperationFailure>,
        total_items: usize,
        operation_name: &str,
    ) -> Result<Vec<T>>
    where
        T: Clone + std::fmt::Debug + serde::Serialize + serde::de::DeserializeOwned,
    {
        let success_count = successful_results.len();
        let failure_count = failures.len();
        let success_ratio = if total_items > 0 {
            success_count as f64 / total_items as f64
        } else {
            0.0
        };

        match &self.strategy {
            DegradationStrategy::PartialResults { min_success_ratio, include_error_details } => {
                if success_ratio >= *min_success_ratio {
                    if failure_count > 0 {
                        info!(
                            "Partial success for {}: {} succeeded, {} failed (ratio: {:.2})",
                            operation_name, success_count, failure_count, success_ratio
                        );
                    }
                    Ok(successful_results)
                } else if success_count > 0 {
                    // Return partial result error but include the successful data
                    Err(partial_result(
                        format!("Low success ratio for {}: {:.2} < {:.2}", 
                               operation_name, success_ratio, min_success_ratio),
                        success_count,
                        failure_count,
                        Some(successful_results),
                        if *include_error_details { failures } else { vec![] },
                    ))
                } else {
                    // Complete failure
                    Err(partial_result(
                        format!("Complete failure for {}", operation_name),
                        0,
                        failure_count,
                        None::<Vec<T>>,
                        if *include_error_details { failures } else { vec![] },
                    ))
                }
            }

            DegradationStrategy::FallbackData { max_age, warn_on_use } => {
                if success_count > 0 {
                    // Update cache with successful results
                    let cache_key = format!("fallback_{}", operation_name);
                    self.update_cache(&cache_key, &successful_results);
                    Ok(successful_results)
                } else {
                    // Try to use cached data
                    let cache_key = format!("fallback_{}", operation_name);
                    if let Some(cached_data) = self.get_cached_data::<Vec<T>>(&cache_key, *max_age) {
                        if *warn_on_use {
                            warn!("Using fallback data for {} due to complete failure", operation_name);
                        }
                        Ok(cached_data)
                    } else {
                        Err(partial_result(
                            format!("No fallback data available for {}", operation_name),
                            0,
                            failure_count,
                            None::<Vec<T>>,
                            failures,
                        ))
                    }
                }
            }

            DegradationStrategy::EmptyResults { log_level } => {
                if failure_count > 0 {
                    match log_level {
                        DegradationLogLevel::Debug => debug!("Returning empty results for {} due to failures", operation_name),
                        DegradationLogLevel::Info => info!("Returning empty results for {} due to failures", operation_name),
                        DegradationLogLevel::Warn => warn!("Returning empty results for {} due to failures", operation_name),
                        DegradationLogLevel::Error => tracing::error!("Returning empty results for {} due to failures", operation_name),
                    }
                }
                Ok(successful_results) // Return whatever succeeded, even if empty
            }

            DegradationStrategy::FailFast => {
                if failure_count > 0 {
                    Err(partial_result(
                        format!("Fail-fast triggered for {}", operation_name),
                        success_count,
                        failure_count,
                        Some(successful_results),
                        failures,
                    ))
                } else {
                    Ok(successful_results)
                }
            }
        }
    }

    /// Update the fallback cache
    fn update_cache<T>(&mut self, key: &str, data: &T)
    where
        T: serde::Serialize,
    {
        if let Ok(serialized) = serde_json::to_value(data) {
            self.fallback_cache.insert(key.to_string(), (serialized, Instant::now()));
        }
    }

    /// Get cached data if it exists and is not too old
    fn get_cached_data<T>(&self, key: &str, max_age: std::time::Duration) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.fallback_cache.get(key).and_then(|(data, timestamp)| {
            if timestamp.elapsed() <= max_age {
                serde_json::from_value(data.clone()).ok()
            } else {
                None
            }
        })
    }

    /// Clear old cache entries
    pub fn cleanup_cache(&mut self, max_age: std::time::Duration) {
        let now = Instant::now();
        self.fallback_cache.retain(|_, (_, timestamp)| {
            now.duration_since(*timestamp) <= max_age
        });
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        let now = Instant::now();
        let total_entries = self.fallback_cache.len();
        let mut fresh_entries = 0;
        let mut stale_entries = 0;
        let cache_max_age = std::time::Duration::from_secs(3600); // 1 hour default

        for (_, timestamp) in self.fallback_cache.values() {
            if now.duration_since(*timestamp) <= cache_max_age {
                fresh_entries += 1;
            } else {
                stale_entries += 1;
            }
        }

        CacheStats {
            total_entries,
            fresh_entries,
            stale_entries,
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub fresh_entries: usize,
    pub stale_entries: usize,
}

/// Utility function to create a degradation handler with common strategies
pub fn create_degradation_handler(strategy_type: &str) -> GracefulDegradationHandler {
    let strategy = match strategy_type {
        "partial" => DegradationStrategy::PartialResults {
            min_success_ratio: 0.5,
            include_error_details: true,
        },
        "fallback" => DegradationStrategy::FallbackData {
            max_age: std::time::Duration::from_secs(300), // 5 minutes
            warn_on_use: true,
        },
        "empty" => DegradationStrategy::EmptyResults {
            log_level: DegradationLogLevel::Warn,
        },
        "fail_fast" => DegradationStrategy::FailFast,
        _ => DegradationStrategy::default(),
    };

    GracefulDegradationHandler::new(strategy)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_operation_partial_success() {
        let mut handler = GracefulDegradationHandler::new(DegradationStrategy::PartialResults {
            min_success_ratio: 0.5,
            include_error_details: true,
        });

        let items = vec![1, 2, 3, 4, 5];
        let operation = |x: &i32| -> Result<i32> {
            if *x % 2 == 0 {
                Err(KnowledgeGraphError::InternalError {
                    message: "Even number".to_string(),
                    source_error: None,
                })
            } else {
                Ok(*x * 2)
            }
        };

        let result = handler.handle_batch_operation(items, operation, "test_operation");
        assert!(result.is_ok());
        
        let results = result.unwrap();
        assert_eq!(results, vec![2, 6, 10]); // 1*2, 3*2, 5*2
    }

    #[test]
    fn test_batch_operation_low_success_ratio() {
        let mut handler = GracefulDegradationHandler::new(DegradationStrategy::PartialResults {
            min_success_ratio: 0.8,
            include_error_details: false,
        });

        let items = vec![1, 2, 3, 4, 5];
        let operation = |x: &i32| -> Result<i32> {
            if *x <= 2 {
                Ok(*x * 2)
            } else {
                Err(KnowledgeGraphError::InternalError {
                    message: "Too large".to_string(),
                    source_error: None,
                })
            }
        };

        let result = handler.handle_batch_operation(items, operation, "test_operation");
        assert!(result.is_err());
        
        // Should be a partial result error
        match result.unwrap_err() {
            KnowledgeGraphError::PartialResultError { successful_operations, failed_operations, .. } => {
                assert_eq!(successful_operations, 2);
                assert_eq!(failed_operations, 3);
            }
            _ => panic!("Expected PartialResultError"),
        }
    }

    #[test]
    fn test_empty_results_strategy() {
        let mut handler = GracefulDegradationHandler::new(DegradationStrategy::EmptyResults {
            log_level: DegradationLogLevel::Warn,
        });

        let items = vec![1, 2, 3];
        let operation = |_: &i32| -> Result<i32> {
            Err(KnowledgeGraphError::InternalError {
                message: "Always fails".to_string(),
                source_error: None,
            })
        };

        let result = handler.handle_batch_operation(items, operation, "test_operation");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Vec::<i32>::new());
    }

    #[test]
    fn test_fail_fast_strategy() {
        let mut handler = GracefulDegradationHandler::new(DegradationStrategy::FailFast);

        let items = vec![1, 2, 3];
        let operation = |x: &i32| -> Result<i32> {
            if *x == 2 {
                Err(KnowledgeGraphError::InternalError {
                    message: "Failed on 2".to_string(),
                    source_error: None,
                })
            } else {
                Ok(*x * 2)
            }
        };

        let result = handler.handle_batch_operation(items, operation, "test_operation");
        assert!(result.is_err());
        
        match result.unwrap_err() {
            KnowledgeGraphError::PartialResultError { .. } => {},
            _ => panic!("Expected PartialResultError"),
        }
    }

    #[test]
    fn test_cache_operations() {
        let mut handler = GracefulDegradationHandler::new(DegradationStrategy::FallbackData {
            max_age: std::time::Duration::from_secs(60),
            warn_on_use: false,
        });

        // Test cache update
        let test_data = vec![1, 2, 3];
        handler.update_cache("test_key", &test_data);

        let stats = handler.cache_stats();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.fresh_entries, 1);

        // Test cache retrieval
        let cached: Option<Vec<i32>> = handler.get_cached_data("test_key", std::time::Duration::from_secs(60));
        assert!(cached.is_some());
        assert_eq!(cached.unwrap(), test_data);
    }

    #[test]
    fn test_create_degradation_handler() {
        let partial_handler = create_degradation_handler("partial");
        let fallback_handler = create_degradation_handler("fallback");
        let empty_handler = create_degradation_handler("empty");
        let fail_fast_handler = create_degradation_handler("fail_fast");
        let default_handler = create_degradation_handler("unknown");

        // Just ensure they're created without panicking
        assert_eq!(partial_handler.cache_stats().total_entries, 0);
        assert_eq!(fallback_handler.cache_stats().total_entries, 0);
        assert_eq!(empty_handler.cache_stats().total_entries, 0);
        assert_eq!(fail_fast_handler.cache_stats().total_entries, 0);
        assert_eq!(default_handler.cache_stats().total_entries, 0);
    }
}