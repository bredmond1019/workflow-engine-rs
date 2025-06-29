//! Test to verify WorkflowError size optimization
//! This test ensures that the WorkflowError enum is properly optimized
//! to avoid clippy's large_enum_variant warning

use workflow_engine_core::error::WorkflowError;

#[test]
fn test_workflow_error_size() {
    // Check the size of WorkflowError
    let error_size = std::mem::size_of::<WorkflowError>();
    
    // The error should be reasonably sized after optimization
    // Typically, a well-optimized error enum should be under 128 bytes
    assert!(
        error_size <= 128,
        "WorkflowError size is {} bytes, which is too large. Consider boxing large variants.",
        error_size
    );
    
    // Print the actual size for debugging
    println!("WorkflowError size: {} bytes", error_size);
}

#[test]
fn test_result_size() {
    // Check the size of Result<T, WorkflowError> for common return types
    let result_unit_size = std::mem::size_of::<Result<(), WorkflowError>>();
    let result_string_size = std::mem::size_of::<Result<String, WorkflowError>>();
    let result_value_size = std::mem::size_of::<Result<serde_json::Value, WorkflowError>>();
    
    println!("Result<(), WorkflowError> size: {} bytes", result_unit_size);
    println!("Result<String, WorkflowError> size: {} bytes", result_string_size);
    println!("Result<Value, WorkflowError> size: {} bytes", result_value_size);
    
    // Results should be reasonably sized
    assert!(result_unit_size <= 136, "Result<(), WorkflowError> is too large");
    assert!(result_string_size <= 160, "Result<String, WorkflowError> is too large");
    assert!(result_value_size <= 160, "Result<Value, WorkflowError> is too large");
}