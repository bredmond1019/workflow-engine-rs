//! # Error Handling Best Practices Example
//!
//! This example demonstrates comprehensive error handling strategies including:
//!
//! - Error categorization (transient vs permanent)
//! - Retry logic with exponential backoff
//! - Circuit breaker pattern for external services
//! - Fallback mechanisms and graceful degradation
//! - Error context and correlation tracking
//! - Recovery strategies and cleanup
//!
//! ## Usage
//!
//! ```bash
//! cargo run --example 04_error_handling_best_practices
//! ```

use workflow_engine_core::prelude::*;
use workflow_engine_core::error::{
    ErrorCategory, ErrorSeverity, RetryPolicy, CircuitBreaker, CircuitBreakerConfig,
    RecoveryStrategy, ErrorContext, ErrorContextExt, retry_with_policy, with_fallback
};
use serde_json::json;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

/// Mock external service that can fail
#[derive(Debug, Clone)]
struct ExternalService {
    name: String,
    failure_rate: f64, // 0.0 to 1.0
    current_failures: Arc<Mutex<u32>>,
}

impl ExternalService {
    fn new(name: impl Into<String>, failure_rate: f64) -> Self {
        Self {
            name: name.into(),
            failure_rate,
            current_failures: Arc::new(Mutex::new(0)),
        }
    }
    
    async fn call(&self, data: &str) -> Result<String, ServiceError> {
        // Simulate network delay
        sleep(Duration::from_millis(100)).await;
        
        // Simulate random failures
        let should_fail = rand::random::<f64>() < self.failure_rate;
        
        if should_fail {
            let mut failures = self.current_failures.lock().unwrap();
            *failures += 1;
            
            // Alternate between different error types
            if *failures % 3 == 0 {
                Err(ServiceError::Timeout {
                    service: self.name.clone(),
                    timeout_ms: 5000,
                })
            } else if *failures % 2 == 0 {
                Err(ServiceError::NetworkError {
                    service: self.name.clone(),
                    message: "Connection refused".to_string(),
                })
            } else {
                Err(ServiceError::RateLimited {
                    service: self.name.clone(),
                    retry_after_seconds: 5,
                })
            }
        } else {
            Ok(format!("Response from {}: processed '{}'", self.name, data))
        }
    }
}

/// Custom error types for external services
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
enum ServiceError {
    #[error("Service {service} timed out after {timeout_ms}ms")]
    Timeout { service: String, timeout_ms: u64 },
    
    #[error("Network error for service {service}: {message}")]
    NetworkError { service: String, message: String },
    
    #[error("Service {service} rate limited, retry after {retry_after_seconds}s")]
    RateLimited { service: String, retry_after_seconds: u32 },
    
    #[error("Service {service} returned invalid data: {details}")]
    InvalidData { service: String, details: String },
    
    #[error("Service {service} is unavailable")]
    ServiceUnavailable { service: String },
}

impl ServiceError {
    fn category(&self) -> ErrorCategory {
        match self {
            ServiceError::Timeout { .. } => ErrorCategory::Transient,
            ServiceError::NetworkError { .. } => ErrorCategory::Transient,
            ServiceError::RateLimited { .. } => ErrorCategory::Transient,
            ServiceError::InvalidData { .. } => ErrorCategory::Permanent,
            ServiceError::ServiceUnavailable { .. } => ErrorCategory::System,
        }
    }
    
    fn severity(&self) -> ErrorSeverity {
        match self {
            ServiceError::Timeout { .. } => ErrorSeverity::Warning,
            ServiceError::NetworkError { .. } => ErrorSeverity::Error,
            ServiceError::RateLimited { .. } => ErrorSeverity::Warning,
            ServiceError::InvalidData { .. } => ErrorSeverity::Error,
            ServiceError::ServiceUnavailable { .. } => ErrorSeverity::Critical,
        }
    }
    
    fn is_retryable(&self) -> bool {
        matches!(self.category(), ErrorCategory::Transient)
    }
}

/// Node that demonstrates retry logic and circuit breaker patterns
#[derive(Debug)]
struct ResilientExternalCallNode {
    service: ExternalService,
    circuit_breaker: Arc<Mutex<CircuitBreaker>>,
    retry_policy: RetryPolicy,
}

impl ResilientExternalCallNode {
    fn new(service: ExternalService) -> Self {
        let circuit_config = CircuitBreakerConfig {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(30),
            success_threshold: 3,
        };
        
        let retry_policy = RetryPolicy {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            multiplier: 2.0,
            jitter: true,
        };
        
        Self {
            service,
            circuit_breaker: Arc::new(Mutex::new(CircuitBreaker::new(circuit_config))),
            retry_policy,
        }
    }
    
    async fn call_with_resilience(&self, data: &str) -> Result<String, WorkflowError> {
        // Check circuit breaker first
        {
            let mut cb = self.circuit_breaker.lock().unwrap();
            if !cb.can_execute() {
                return Err(WorkflowError::CircuitBreakerOpen {
                    service: self.service.name.clone(),
                });
            }
        }
        
        // Attempt call with retry logic
        let result = retry_with_policy(&self.retry_policy, || async {
            match self.service.call(data).await {
                Ok(response) => {
                    // Record success in circuit breaker
                    self.circuit_breaker.lock().unwrap().record_success();
                    Ok(response)
                }
                Err(e) if e.is_retryable() => {
                    println!("⚠️  Retryable error: {}", e);
                    self.circuit_breaker.lock().unwrap().record_failure();
                    Err(WorkflowError::ServiceError {
                        service: self.service.name.clone(),
                        error: e.to_string(),
                    })
                }
                Err(e) => {
                    println!("❌ Permanent error: {}", e);
                    self.circuit_breaker.lock().unwrap().record_failure();
                    Err(WorkflowError::ServiceError {
                        service: self.service.name.clone(),
                        error: e.to_string(),
                    })
                }
            }
        }).await;
        
        result
    }
}

#[async_trait]
impl AsyncNode for ResilientExternalCallNode {
    async fn process_async(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let start_time = std::time::Instant::now();
        
        // Extract input data
        let input: serde_json::Value = context.get_event_data()?;
        let data = input.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("default message");
        
        println!("🔄 Calling external service: {}", self.service.name);
        
        // Attempt call with full resilience patterns
        let result = with_fallback(
            self.call_with_resilience(data),
            || async {
                println!("🛡️  Using fallback response");
                Ok(format!("Fallback response for: {}", data))
            }
        ).await;
        
        let processing_time = start_time.elapsed();
        
        match result {
            Ok(response) => {
                context.update_node("service_response", json!({
                    "response": response,
                    "service": self.service.name,
                    "success": true,
                    "processing_time_ms": processing_time.as_millis(),
                    "used_fallback": response.contains("Fallback")
                }));
                
                context.set_metadata("call_status", "success")?;
                context.set_metadata("processing_time_ms", processing_time.as_millis())?;
                
                println!("✅ Service call succeeded in {:?}", processing_time);
            }
            Err(e) => {
                // Add rich error context
                let error_context = ErrorContext::new()
                    .with_context("service_name", &self.service.name)
                    .with_context("input_data", data)
                    .with_context("processing_time_ms", processing_time.as_millis())
                    .with_context("error_type", "service_call_failed");
                
                context.update_node("service_error", json!({
                    "error": e.to_string(),
                    "service": self.service.name,
                    "success": false,
                    "processing_time_ms": processing_time.as_millis(),
                    "error_context": error_context
                }));
                
                context.set_metadata("call_status", "failed")?;
                context.set_metadata("error_message", e.to_string())?;
                
                println!("❌ Service call failed: {}", e);
                
                // Don't fail the workflow - store error and continue
                // This demonstrates graceful degradation
            }
        }
        
        Ok(context)
    }
}

/// Node that demonstrates validation with detailed error reporting
#[derive(Debug)]
struct ValidationNode {
    strict_mode: bool,
}

impl ValidationNode {
    fn new(strict_mode: bool) -> Self {
        Self { strict_mode }
    }
    
    fn validate_input(&self, data: &serde_json::Value) -> Result<Vec<String>, Vec<ValidationError>> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        
        // Required field validation
        if data.get("id").is_none() {
            errors.push(ValidationError {
                field: "id".to_string(),
                error_type: "required".to_string(),
                message: "ID field is required".to_string(),
                severity: ErrorSeverity::Error,
            });
        }
        
        // Type validation
        if let Some(age) = data.get("age") {
            if !age.is_number() {
                errors.push(ValidationError {
                    field: "age".to_string(),
                    error_type: "type".to_string(),
                    message: "Age must be a number".to_string(),
                    severity: ErrorSeverity::Error,
                });
            } else if let Some(age_val) = age.as_f64() {
                if age_val < 0.0 || age_val > 150.0 {
                    warnings.push("Age seems unusual".to_string());
                }
            }
        }
        
        // Email validation
        if let Some(email) = data.get("email") {
            if let Some(email_str) = email.as_str() {
                if !email_str.contains('@') {
                    errors.push(ValidationError {
                        field: "email".to_string(),
                        error_type: "format".to_string(),
                        message: "Invalid email format".to_string(),
                        severity: ErrorSeverity::Warning,
                    });
                }
            }
        }
        
        // In strict mode, warnings become errors
        if self.strict_mode && !warnings.is_empty() {
            for warning in warnings {
                errors.push(ValidationError {
                    field: "general".to_string(),
                    error_type: "strict_validation".to_string(),
                    message: warning,
                    severity: ErrorSeverity::Error,
                });
            }
            warnings.clear();
        }
        
        if errors.is_empty() {
            Ok(warnings)
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ValidationError {
    field: String,
    error_type: String,
    message: String,
    severity: ErrorSeverity,
}

impl Node for ValidationNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let input: serde_json::Value = context.get_event_data()?;
        
        println!("🔍 Validating input data (strict_mode: {})", self.strict_mode);
        
        match self.validate_input(&input) {
            Ok(warnings) => {
                context.update_node("validation_result", json!({
                    "status": "passed",
                    "warnings": warnings,
                    "strict_mode": self.strict_mode
                }));
                
                context.set_metadata("validation_status", "passed")?;
                
                if !warnings.is_empty() {
                    println!("⚠️  Validation passed with {} warnings", warnings.len());
                    for warning in &warnings {
                        println!("    ⚠️  {}", warning);
                    }
                } else {
                    println!("✅ Validation passed without warnings");
                }
            }
            Err(errors) => {
                let error_summary = json!({
                    "status": "failed",
                    "errors": errors,
                    "error_count": errors.len(),
                    "strict_mode": self.strict_mode
                });
                
                context.update_node("validation_result", error_summary);
                context.set_metadata("validation_status", "failed")?;
                context.set_metadata("error_count", errors.len())?;
                
                println!("❌ Validation failed with {} errors:", errors.len());
                for error in &errors {
                    println!("    ❌ [{}] {}: {}", error.field, error.error_type, error.message);
                }
                
                // In this example, we continue processing even with validation errors
                // In production, you might want to fail the workflow depending on severity
                if self.strict_mode {
                    return Err(WorkflowError::ValidationError {
                        message: format!("Strict validation failed with {} errors", errors.len())
                    });
                }
            }
        }
        
        Ok(context)
    }
}

/// Node that demonstrates cleanup and recovery operations
#[derive(Debug)]
struct CleanupNode;

impl Node for CleanupNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("🧹 Performing cleanup operations...");
        
        let mut cleanup_actions = Vec::new();
        
        // Check if there were any service errors
        if let Ok(Some(service_error)) = context.get_node_data::<serde_json::Value>("service_error") {
            cleanup_actions.push("Log service error for monitoring".to_string());
            cleanup_actions.push("Reset circuit breaker if needed".to_string());
            
            // Simulate cleanup delay
            std::thread::sleep(Duration::from_millis(50));
        }
        
        // Check validation results
        if let Ok(Some(validation)) = context.get_node_data::<serde_json::Value>("validation_result") {
            if validation["status"] == "failed" {
                cleanup_actions.push("Archive invalid data for review".to_string());
                cleanup_actions.push("Send notification to data team".to_string());
            }
        }
        
        // Always perform basic cleanup
        cleanup_actions.push("Clear temporary resources".to_string());
        cleanup_actions.push("Update processing metrics".to_string());
        
        context.update_node("cleanup_summary", json!({
            "actions_performed": cleanup_actions,
            "cleanup_timestamp": chrono::Utc::now(),
            "cleanup_success": true
        }));
        
        context.set_metadata("cleanup_performed", true)?;
        
        println!("✅ Cleanup completed: {} actions performed", cleanup_actions.len());
        for action in &cleanup_actions {
            println!("    🧹 {}", action);
        }
        
        Ok(context)
    }
}

#[tokio::main]
async fn main() -> Result<(), WorkflowError> {
    println!("🚀 Starting Error Handling Best Practices Example");
    println!("==================================================");
    
    // Create services with different failure rates
    let reliable_service = ExternalService::new("ReliableAPI", 0.1); // 10% failure rate
    let unreliable_service = ExternalService::new("UnreliableAPI", 0.7); // 70% failure rate
    
    // Build workflow with error handling
    let workflow = TypedWorkflowBuilder::new("error_handling_workflow")
        .description("Demonstrates comprehensive error handling patterns")
        .start_with_node(NodeId::new("validation"))
        .parallel_nodes(vec![
            NodeId::new("reliable_call"),
            NodeId::new("unreliable_call"),
        ])
        .then_node(NodeId::new("cleanup"))
        .build()?;
    
    // Register nodes
    workflow.register_node(NodeId::new("validation"), ValidationNode::new(false));
    workflow.register_async_node(NodeId::new("reliable_call"), ResilientExternalCallNode::new(reliable_service));
    workflow.register_async_node(NodeId::new("unreliable_call"), ResilientExternalCallNode::new(unreliable_service));
    workflow.register_node(NodeId::new("cleanup"), CleanupNode);
    
    println!("📋 Error handling workflow built:");
    println!("   1. ValidationNode - Input validation with detailed errors");
    println!("   2. ResilientExternalCallNode (Reliable) - 10% failure rate");
    println!("   3. ResilientExternalCallNode (Unreliable) - 70% failure rate");
    println!("   4. CleanupNode - Cleanup and recovery operations");
    println!();
    
    // Test cases with different error scenarios
    let test_cases = vec![
        // Valid data
        json!({
            "id": "USER-001",
            "name": "Alice Smith",
            "email": "alice@example.com",
            "age": 28,
            "message": "Process this data"
        }),
        
        // Missing required field
        json!({
            "name": "Bob Johnson",
            "email": "bob@example.com",
            "message": "Missing ID field"
        }),
        
        // Invalid email format
        json!({
            "id": "USER-002",
            "name": "Charlie Brown",
            "email": "invalid-email-format",
            "age": 35,
            "message": "Bad email format"
        }),
        
        // Invalid age type
        json!({
            "id": "USER-003",
            "name": "Diana Prince",
            "email": "diana@example.com",
            "age": "thirty-two",
            "message": "Invalid age type"
        }),
        
        // Edge case - extreme age
        json!({
            "id": "USER-004",
            "name": "Eve Adams",
            "email": "eve@example.com",
            "age": 200,
            "message": "Unusual age value"
        }),
    ];
    
    for (i, input_data) in test_cases.into_iter().enumerate() {
        println!("🔄 Test Case {} - Error Handling Scenario", i + 1);
        println!("   Input: {}", serde_json::to_string(&input_data)?);
        
        let start_time = std::time::Instant::now();
        
        // Run the workflow - it should complete even with errors
        let result = workflow.run_async(input_data).await?;
        
        let total_time = start_time.elapsed();
        
        // Analyze results
        println!("   📊 Results Analysis:");
        
        // Validation results
        if let Some(validation) = result.get_node_data::<serde_json::Value>("validation_result")? {
            println!("      🔍 Validation: {}", validation["status"]);
            if let Some(errors) = validation.get("errors").and_then(|e| e.as_array()) {
                for error in errors {
                    println!("         ❌ {}", error["message"]);
                }
            }
            if let Some(warnings) = validation.get("warnings").and_then(|w| w.as_array()) {
                for warning in warnings {
                    println!("         ⚠️  {}", warning);
                }
            }
        }
        
        // Service call results
        if let Some(reliable_response) = result.get_node_data::<serde_json::Value>("service_response")? {
            let service_name = reliable_response.get("service").and_then(|s| s.as_str()).unwrap_or("Unknown");
            let success = reliable_response.get("success").and_then(|s| s.as_bool()).unwrap_or(false);
            println!("      📞 {}: {}", service_name, if success { "✅ Success" } else { "❌ Failed" });
        }
        
        if let Some(service_error) = result.get_node_data::<serde_json::Value>("service_error")? {
            let service_name = service_error.get("service").and_then(|s| s.as_str()).unwrap_or("Unknown");
            println!("      📞 {}: ❌ Failed", service_name);
        }
        
        // Cleanup results
        if let Some(cleanup) = result.get_node_data::<serde_json::Value>("cleanup_summary")? {
            let actions = cleanup.get("actions_performed").and_then(|a| a.as_array()).map(|a| a.len()).unwrap_or(0);
            println!("      🧹 Cleanup: {} actions performed", actions);
        }
        
        println!("   ⏱️  Total execution time: {:?}", total_time);
        println!("   ✅ Workflow completed gracefully despite errors");
        println!();
    }
    
    // Test strict validation mode
    println!("🔍 Testing Strict Validation Mode");
    println!("=================================");
    
    let strict_workflow = TypedWorkflowBuilder::new("strict_validation_workflow")
        .start_with_node(NodeId::new("strict_validation"))
        .build()?;
    
    strict_workflow.register_node(NodeId::new("strict_validation"), ValidationNode::new(true));
    
    let problematic_data = json!({
        "id": "USER-005",
        "name": "Test User",
        "email": "test@example.com",
        "age": 200  // This will trigger a warning that becomes an error in strict mode
    });
    
    match strict_workflow.run_async(problematic_data).await {
        Ok(_) => println!("   ❌ Expected strict validation to fail!"),
        Err(e) => println!("   ✅ Strict validation correctly failed: {}", e),
    }
    
    println!();
    println!("🎉 Error Handling Best Practices Example completed!");
    println!("===================================================");
    println!();
    println!("Error handling patterns demonstrated:");
    println!("• Retry logic with exponential backoff");
    println!("• Circuit breaker pattern for external services");
    println!("• Fallback mechanisms and graceful degradation");
    println!("• Detailed error categorization and context");
    println!("• Validation with configurable strictness");
    println!("• Cleanup and recovery operations");
    println!("• Workflow continuation despite individual failures");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_external_service_failure_handling() {
        let service = ExternalService::new("TestService", 1.0); // Always fails
        let node = ResilientExternalCallNode::new(service);
        
        let context = TaskContext::new(
            "test".to_string(),
            json!({"message": "test"})
        );
        
        // Should not panic, should handle gracefully
        let result = node.process_async(context).await;
        assert!(result.is_ok());
        
        let context = result.unwrap();
        assert!(context.get_node_data::<serde_json::Value>("service_response").unwrap().is_some() ||
                context.get_node_data::<serde_json::Value>("service_error").unwrap().is_some());
    }
    
    #[test]
    fn test_validation_node_strict_mode() {
        let strict_validator = ValidationNode::new(true);
        let lenient_validator = ValidationNode::new(false);
        
        let data_with_warnings = json!({
            "id": "TEST-001",
            "email": "test@example.com",
            "age": 200  // Unusual age - triggers warning
        });
        
        let context = TaskContext::new("test".to_string(), data_with_warnings);
        
        // Strict mode should fail
        let strict_result = strict_validator.process(context.clone());
        assert!(strict_result.is_err());
        
        // Lenient mode should pass
        let lenient_result = lenient_validator.process(context);
        assert!(lenient_result.is_ok());
    }
    
    #[test]
    fn test_service_error_categorization() {
        let timeout_error = ServiceError::Timeout {
            service: "test".to_string(),
            timeout_ms: 5000,
        };
        assert_eq!(timeout_error.category(), ErrorCategory::Transient);
        assert!(timeout_error.is_retryable());
        
        let invalid_data_error = ServiceError::InvalidData {
            service: "test".to_string(),
            details: "bad format".to_string(),
        };
        assert_eq!(invalid_data_error.category(), ErrorCategory::Permanent);
        assert!(!invalid_data_error.is_retryable());
    }
}