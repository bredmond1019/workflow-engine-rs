//! Enhanced Error Handling Demo
//!
//! This example demonstrates the improved error types and chaining
//! implemented in Task 3.3.

use workflow_engine_core::error::{WorkflowError, ErrorExt, ErrorCategory, ErrorSeverity};
use workflow_engine_mcp::transport::TransportError;
use std::io;

fn main() {
    println!("Enhanced Error Handling Demo");
    println!("============================");

    // Demonstrate structured error types with rich context
    let processing_error = WorkflowError::processing_error_with_context(
        "Failed to parse user input",
        "InputValidationNode",
        Some("node_123".to_string()),
        None,
    );

    println!("\n1. Processing Error with Context:");
    println!("   Error: {}", processing_error);
    println!("   Category: {:?}", processing_error.category());
    println!("   Severity: {:?}", processing_error.severity());
    println!("   Code: {}", processing_error.error_code());

    // Demonstrate validation error with detailed context
    let validation_error = WorkflowError::validation_error_with_value(
        "Value must be between 1 and 100",
        "max_retries",
        Some("150".to_string()),
        "must be in range [1, 100]",
        "in workflow configuration",
    );

    println!("\n2. Validation Error with Value Context:");
    println!("   Error: {}", validation_error);
    println!("   Category: {:?}", validation_error.category());
    println!("   Severity: {:?}", validation_error.severity());
    println!("   Code: {}", validation_error.error_code());
    println!("   Is Retryable: {}", validation_error.is_retryable());

    // Demonstrate API error with service context
    let api_error = WorkflowError::api_error(
        "Rate limit exceeded",
        "OpenAI",
        "/v1/chat/completions",
        Some(429),
    );

    println!("\n3. API Error with Service Context:");
    println!("   Error: {}", api_error);
    println!("   Category: {:?}", api_error.category());
    println!("   Severity: {:?}", api_error.severity());
    println!("   Code: {}", api_error.error_code());
    println!("   Is Retryable: {}", api_error.is_retryable());

    // Demonstrate MCP connection error
    let mcp_error = WorkflowError::mcp_connection_error(
        "WebSocket connection closed unexpectedly",
        "customer-support-server",
        "WebSocket",
        "ws://localhost:8001/mcp",
    );

    println!("\n4. MCP Connection Error:");
    println!("   Error: {}", mcp_error);
    println!("   Category: {:?}", mcp_error.category());
    println!("   Severity: {:?}", mcp_error.severity());
    println!("   Code: {}", mcp_error.error_code());
    println!("   Is Retryable: {}", mcp_error.is_retryable());

    // Demonstrate MCP protocol error with detailed context
    let protocol_error = WorkflowError::mcp_protocol_error(
        "Invalid JSON-RPC response format",
        "knowledge-base-server",
        "valid JSON-RPC 2.0 response",
        "malformed JSON",
        "response",
    );

    println!("\n5. MCP Protocol Error:");
    println!("   Error: {}", protocol_error);
    println!("   Category: {:?}", protocol_error.category());
    println!("   Severity: {:?}", protocol_error.severity());
    println!("   Code: {}", protocol_error.error_code());

    // Demonstrate database error with operation context
    let db_error = WorkflowError::database_error(
        "Connection pool exhausted",
        "connection",
        Some("events".to_string()),
    );

    println!("\n6. Database Error with Operation Context:");
    println!("   Error: {}", db_error);
    println!("   Category: {:?}", db_error.category());
    println!("   Severity: {:?}", db_error.severity());
    println!("   Code: {}", db_error.error_code());
    println!("   Is Retryable: {}", db_error.is_retryable());

    // Demonstrate transport error with error chaining
    let io_error = io::Error::new(io::ErrorKind::ConnectionRefused, "Connection refused");
    let transport_error = TransportError::io_error(
        "Failed to establish WebSocket connection",
        "connect",
        io_error,
    );

    println!("\n7. Transport Error with Error Chaining:");
    println!("   Error: {}", transport_error);
    println!("   Category: {:?}", transport_error.category());
    println!("   Severity: {:?}", transport_error.severity());
    println!("   Code: {}", transport_error.error_code());

    // Demonstrate error conversion
    let workflow_error: WorkflowError = transport_error.into();
    println!("\n8. Transport Error Converted to WorkflowError:");
    println!("   Error: {}", workflow_error);
    println!("   Category: {:?}", workflow_error.category());
    println!("   Severity: {:?}", workflow_error.severity());

    // Demonstrate error categorization for retry logic
    println!("\n9. Error Categorization for Retry Logic:");
    let errors = vec![
        WorkflowError::CycleDetected,
        WorkflowError::mcp_connection_error("timeout", "server", "ws", "localhost"),
        WorkflowError::validation_error("invalid", "field", "constraint", "context"),
    ];

    for (i, error) in errors.iter().enumerate() {
        println!("   Error {}: {}", i + 1, error);
        println!("     - Category: {:?}", error.category());
        println!("     - Should Retry: {}", error.is_retryable());
        println!("     - Severity: {:?}", error.severity());
    }

    println!("\n✓ Enhanced error handling demo completed successfully!");
    println!("  All error types now provide rich context, proper error chaining,");
    println!("  and structured information for better debugging and monitoring.");
}