//! Example demonstrating the comprehensive error handling framework
//! This shows how other agents (especially Agent 2 working on AI) can use the error handling

use workflow_engine_core::error::{
    WorkflowError, context::ErrorContextExt, 
    retry::RetryBuilder, 
    circuit_breaker::{CircuitBreaker, CircuitBreakerConfig},
    recovery::{with_fallback, CacheRecovery},
};
use std::sync::Arc;
use std::time::Duration;

/// Example AI service that uses the error handling framework
struct AIService {
    circuit_breaker: Arc<CircuitBreaker>,
    cache: Arc<CacheRecovery<String>>,
}

impl AIService {
    fn new() -> Self {
        let circuit_breaker = Arc::new(CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            timeout: Duration::from_secs(30),
            ..Default::default()
        }));
        
        let cache = Arc::new(CacheRecovery::new(Duration::from_secs(300))); // 5 min cache
        
        Self {
            circuit_breaker,
            cache,
        }
    }
    
    /// Call an AI model with comprehensive error handling
    async fn call_ai_model(&self, prompt: &str, user_id: &str) -> Result<String, WorkflowError> {
        let cache_key = format!("ai_response:{}:{}", user_id, prompt);
        
        // Use circuit breaker to protect against cascade failures
        let result = self.circuit_breaker.call(|| async {
            // Try with retries for transient failures
            RetryBuilder::new()
                .max_attempts(3)
                .initial_delay(Duration::from_millis(100))
                .multiplier(2.0)
                .jitter(0.1)
                .execute(|| async {
                    // Simulate AI API call
                    self.make_api_call(prompt).await
                        .map_err(|e| e
                            .context("prompt", prompt)
                            .with_context("user_id", user_id)
                            .with_correlation_id(generate_request_id())
                            .error
                        )
                })
                .await
        })
        .await;

        match result {
            Ok(value) => Ok(value),
            Err(error) => {
                // Try cache on circuit breaker open or API failure
                self.cache.execute(&cache_key, || async {
                    Err(error)
                }).await
            }
        }
    }
    
    /// Simulate API call to AI service
    async fn make_api_call(&self, prompt: &str) -> Result<String, WorkflowError> {
        // Simulate various error conditions
        let random = rand::random::<f32>();
        
        if random < 0.1 {
            // 10% chance of connection error (transient)
            Err(WorkflowError::MCPConnectionError {
                message: "Failed to connect to AI service".to_string(),
            })
        } else if random < 0.2 {
            // 10% chance of API error (transient)
            Err(WorkflowError::ApiError {
                message: "AI service temporarily unavailable".to_string(),
            })
        } else if random < 0.25 {
            // 5% chance of validation error (permanent)
            Err(WorkflowError::ValidationError {
                message: "Invalid prompt format".to_string(),
            })
        } else {
            // Success
            Ok(format!("AI response to: {}", prompt))
        }
    }
}

/// Example workflow node that uses error handling
struct AIProcessingNode {
    ai_service: Arc<AIService>,
}

impl AIProcessingNode {
    async fn process(&self, input: &str, user_id: &str) -> Result<String, WorkflowError> {
        // Process with fallback to simpler model
        let result = with_fallback(
            || async {
                self.ai_service.call_ai_model(input, user_id).await
            },
            format!("Fallback response for: {}", input),
        ).await;
        Ok(result)
    }
}

/// Example of error handling in database operations
async fn save_with_retry(data: &str) -> Result<(), WorkflowError> {
    RetryBuilder::new()
        .max_attempts(5)
        .initial_delay(Duration::from_millis(200))
        .retry_on(vec!["DatabaseError".to_string()])
        .execute(|| async {
            // Simulate database save
            if rand::random::<f32>() < 0.3 {
                Err(WorkflowError::DatabaseError {
                    message: "Connection pool exhausted".to_string(),
                })
            } else {
                println!("Data saved: {}", data);
                Ok(())
            }
        })
        .await
}

/// Generate a request ID for correlation
fn generate_request_id() -> String {
    use uuid::Uuid;
    format!("req-{}", Uuid::new_v4())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the AI service with error handling
    let ai_service = Arc::new(AIService::new());
    let processing_node = AIProcessingNode {
        ai_service: ai_service.clone(),
    };
    
    // Example 1: Process with comprehensive error handling
    println!("Example 1: AI Processing with Error Handling");
    match processing_node.process("Explain quantum computing", "user123").await {
        Ok(response) => println!("Success: {}", response),
        Err(e) => println!("Error (handled gracefully): {}", e),
    }
    
    // Example 2: Database operation with retry
    println!("\nExample 2: Database Save with Retry");
    match save_with_retry("Important workflow state").await {
        Ok(_) => println!("Database save successful"),
        Err(e) => println!("Database save failed after retries: {}", e),
    }
    
    // Example 3: Simulate circuit breaker behavior
    println!("\nExample 3: Circuit Breaker Protection");
    for i in 0..5 {
        println!("Request {}: ", i + 1);
        match ai_service.call_ai_model("Test prompt", "user456").await {
            Ok(response) => println!("  Success: {}", response),
            Err(e) => println!("  Error: {}", e),
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    // Show circuit breaker metrics
    let metrics = ai_service.circuit_breaker.metrics();
    println!("\nCircuit Breaker Metrics:");
    println!("  Total calls: {}", metrics.total_calls);
    println!("  Total failures: {}", metrics.total_failures);
    println!("  Total successes: {}", metrics.total_successes);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_ai_service_error_handling() {
        let service = AIService::new();
        
        // Make multiple calls to test various scenarios
        let mut results = vec![];
        for _ in 0..10 {
            let result = service.call_ai_model("test prompt", "test_user").await;
            results.push(result.is_ok());
        }
        
        // Should have some successes and failures
        let successes = results.iter().filter(|&&x| x).count();
        assert!(successes > 0);
        assert!(successes < 10);
    }
}