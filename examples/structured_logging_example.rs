/*!
Example demonstrating structured logging with correlation IDs.

This example shows how to use the structured logging system with
correlation ID propagation in the AI Workflow System.
*/

use backend::{
    monitoring::{
        correlation::{set_correlation_id, get_correlation_id},
        logging::{log_http_request, log_workflow_event},
    },
    info_with_correlation,
    debug_with_correlation,
};
use std::collections::HashMap;
use tracing::{info, warn, error, instrument};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    // Initialize structured logging
    backend::monitoring::logging::init_structured_logging();
    
    // Example 1: Basic logging with correlation ID
    let correlation_id = Uuid::new_v4().to_string();
    set_correlation_id(Some(correlation_id.clone()));
    
    info_with_correlation!("Starting example application");
    debug_with_correlation!("Debug information");
    
    // Example 2: Logging HTTP requests
    simulate_http_request().await;
    
    // Example 3: Logging workflow events
    simulate_workflow_execution().await;
    
    // Example 4: Structured logging with custom fields
    demonstrate_structured_logging().await;
    
    // Example 5: Error logging with context
    demonstrate_error_logging().await;
    
    info_with_correlation!("Example application completed");
}

#[instrument]
async fn simulate_http_request() {
    let correlation_id = get_correlation_id();
    
    // Simulate request processing
    let start = std::time::Instant::now();
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    let duration = start.elapsed().as_millis() as u64;
    
    // Log the HTTP request
    log_http_request(
        "POST",
        "/api/v1/workflows/trigger",
        200,
        duration,
        correlation_id.as_deref(),
    );
}

#[instrument]
async fn simulate_workflow_execution() {
    let workflow_id = Uuid::new_v4().to_string();
    let correlation_id = get_correlation_id();
    
    // Log workflow start
    let mut details = HashMap::new();
    details.insert("workflow_type".to_string(), "research_to_docs".to_string());
    details.insert("user_id".to_string(), "user123".to_string());
    
    log_workflow_event(
        &workflow_id,
        "started",
        None,
        details.clone(),
        correlation_id.as_deref(),
    );
    
    // Simulate workflow steps
    let nodes = ["research_node", "analysis_node", "documentation_node"];
    
    for node in &nodes {
        // Log node execution
        details.insert("duration_ms".to_string(), "150".to_string());
        log_workflow_event(
            &workflow_id,
            "node_completed",
            Some(node),
            details.clone(),
            correlation_id.as_deref(),
        );
        
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    // Log workflow completion
    details.insert("total_duration_ms".to_string(), "500".to_string());
    details.insert("status".to_string(), "success".to_string());
    
    log_workflow_event(
        &workflow_id,
        "completed",
        None,
        details,
        correlation_id.as_deref(),
    );
}

#[instrument]
async fn demonstrate_structured_logging() {
    // Structured logging with multiple fields
    info!(
        correlation_id = ?get_correlation_id(),
        user_id = "user456",
        action = "create_workflow",
        workflow_name = "content_pipeline",
        metadata = ?serde_json::json!({
            "priority": "high",
            "estimated_duration": 300,
            "tags": ["content", "automated"]
        }),
        "User created a new workflow"
    );
    
    // Logging with numeric values
    info!(
        correlation_id = ?get_correlation_id(),
        queue_depth = 42,
        active_workers = 8,
        cpu_usage = 65.5,
        memory_mb = 1024,
        "System metrics update"
    );
    
    // Warning with structured context
    warn!(
        correlation_id = ?get_correlation_id(),
        threshold = 80,
        current_value = 85,
        resource = "memory",
        action_required = "scale_up",
        "Resource usage above threshold"
    );
}

#[instrument]
async fn demonstrate_error_logging() {
    // Simulate an error scenario
    let result = simulate_operation_that_might_fail().await;
    
    match result {
        Ok(_) => {
            info_with_correlation!("Operation completed successfully");
        }
        Err(e) => {
            // Log error with full context
            error!(
                correlation_id = ?get_correlation_id(),
                error_type = "DatabaseConnectionError",
                error_code = "DB_CONN_001",
                retry_count = 3,
                last_retry_at = ?chrono::Utc::now(),
                connection_string = "postgres://***:***@localhost/db",
                error_details = %e,
                stack_trace = ?e,
                "Failed to connect to database after multiple retries"
            );
            
            // Log related warning
            warn!(
                correlation_id = ?get_correlation_id(),
                fallback_mode = "read_only",
                cache_enabled = true,
                "Switching to fallback mode due to database error"
            );
        }
    }
}

async fn simulate_operation_that_might_fail() -> Result<(), Box<dyn std::error::Error>> {
    // Simulate a failure
    Err("Connection timeout".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_structured_logging() {
        // Initialize logging for tests
        backend::monitoring::logging::init_structured_logging();
        
        // Set correlation ID
        let test_correlation_id = "test-correlation-123";
        set_correlation_id(Some(test_correlation_id.to_string()));
        
        // Test logging
        info_with_correlation!("Test log entry");
        
        // Verify correlation ID is set
        assert_eq!(get_correlation_id(), Some(test_correlation_id.to_string()));
    }
}