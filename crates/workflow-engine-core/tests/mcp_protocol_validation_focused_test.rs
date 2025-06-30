//! Test 3c: MCP Protocol Message Validation Tests (Focused)
//! 
//! This test suite validates MCP protocol message handling against malformed, malicious, 
//! and resource-exhausting inputs using TDD methodology.
//!
//! RED -> GREEN -> REFACTOR cycle for MCP protocol security and robustness.

use serde_json::{json, Value};
use std::collections::HashMap;
use workflow_engine_core::mcp::validation::{
    McpMessageValidator, ValidationError, RequestTracker, ValidationConfig
};

#[cfg(test)]
mod mcp_protocol_validation_tests {
    use super::*;

    // =============================================================================
    // Test 3c.1: Message Size Limit Validation Tests (RED Phase)
    // =============================================================================

    #[test]
    fn test_mcp_message_size_limit_validation_should_exist() {
        // REFACTOR: Use the new validation module
        let validator = McpMessageValidator::new();
        
        // Create an extremely large JSON payload that could cause OOM
        let large_data = "x".repeat(10 * 1024 * 1024); // 10MB string
        let large_json = json!({
            "method": "tools/call",
            "id": "test-1",
            "params": {
                "name": "test_tool",
                "arguments": {
                    "large_field": large_data
                }
            }
        });

        // Should fail with message size validation error
        let result = validator.validate_message_size(&large_json);
        
        // GREEN: Now properly validates and rejects oversized messages
        assert!(result.is_err(), "Expected message size validation to reject large messages");
    }

    #[test]
    fn test_mcp_nested_object_depth_limit() {
        // REFACTOR: Use the new validation module
        let validator = McpMessageValidator::new();
        
        // Create deeply nested JSON that could cause stack overflow
        let mut nested_obj = json!("leaf");
        for _ in 0..1000 {
            nested_obj = json!({ "nested": nested_obj });
        }

        let deep_request = json!({
            "method": "tools/call",
            "id": "test-deep",
            "params": {
                "name": "test_tool", 
                "arguments": {
                    "deep_data": nested_obj
                }
            }
        });

        // Should fail with depth validation error
        let result = validator.validate_nesting_depth(&deep_request);
        
        // GREEN: Now properly validates and rejects deeply nested objects
        assert!(result.is_err(), "Expected deeply nested object validation to fail");
    }

    #[test]
    fn test_mcp_array_length_limit() {
        // Create extremely large array that could consume excessive memory
        let large_array: Vec<String> = (0..100_000).map(|i| format!("item_{}", i)).collect();
        
        let array_request = json!({
            "method": "tools/call",
            "id": "test-array",
            "params": {
                "name": "test_tool",
                "arguments": {
                    "large_array": large_array
                }
            }
        });

        let validator = McpMessageValidator::new();
        let result = validator.validate_array_sizes(&array_request);
        
        // GREEN: Should fail with array size validation error
        assert!(result.is_err(), "Expected large array validation to fail");
    }

    // =============================================================================
    // Test 3c.2: JSON-RPC Protocol Validation Tests (RED Phase)
    // =============================================================================

    #[test]
    fn test_mcp_missing_required_jsonrpc_fields() {
        // Missing "jsonrpc" version field
        let invalid_request = json!({
            "method": "initialize",
            "id": "test-1"
            // Missing "jsonrpc": "2.0" field
        });

        let validator = McpMessageValidator::new();
        let result = validator.validate_jsonrpc_format(&invalid_request);
        
        // GREEN: Should fail with missing jsonrpc field validation
        assert!(result.is_err(), "Expected missing jsonrpc field validation to fail");
    }

    #[test]
    fn test_mcp_invalid_jsonrpc_version() {
        let invalid_version_request = json!({
            "jsonrpc": "1.0", // Invalid version - should be "2.0"
            "method": "initialize",
            "id": "test-1",
            "params": {
                "protocol_version": "2024-11-05",
                "capabilities": {},
                "client_info": {
                    "name": "test",
                    "version": "1.0"
                }
            }
        });

        let validator = McpMessageValidator::new();
        let result = validator.validate_jsonrpc_version(&invalid_version_request);
        
        // GREEN: Should fail with invalid jsonrpc version validation
        assert!(result.is_err(), "Expected invalid jsonrpc version validation to fail");
    }

    #[test]
    fn test_mcp_malformed_method_names() {
        let long_method = "a".repeat(1000);
        let malformed_methods = vec![
            "", // Empty method
            "invalid method with spaces",
            "method/with/too/many/slashes",
            "method\nwith\nnewlines",
            "method\0with\0nulls",
            &long_method, // Extremely long method name
        ];

        for method in malformed_methods {
            let invalid_request = json!({
                "jsonrpc": "2.0",
                "method": method,
                "id": "test-1"
            });

            let validator = McpMessageValidator::new();
            let result = validator.validate_method_name(&invalid_request);
            
            // GREEN: Should fail with method name validation
            assert!(result.is_err(), "Expected malformed method '{}' validation to fail", method);
        }
    }

    // =============================================================================
    // Test 3c.3: Protocol Version Compatibility Tests (RED Phase)
    // =============================================================================

    #[test] 
    fn test_mcp_unsupported_protocol_versions() {
        let unsupported_versions = vec![
            "1.0.0",      // Too old
            "2025-01-01", // Future version
            "invalid",    // Malformed version
            "",           // Empty version
            "2024-11-05-beta", // Unsupported format
        ];

        for version in unsupported_versions {
            // GREEN: Should fail with unsupported protocol version validation
            let validator = McpMessageValidator::new();
            let validation_result = validator.validate_protocol_version(&version);
            assert!(validation_result.is_err(), 
                "Expected protocol version '{}' validation to fail", version);
        }
    }

    // =============================================================================
    // Test 3c.4: Request ID Validation Tests (RED Phase) 
    // =============================================================================

    #[test]
    fn test_mcp_malicious_request_ids() {
        let long_id = "a".repeat(10000);
        let malicious_ids = vec![
            "\0\0\0\0",           // Null bytes
            "id\nwith\nnewlines", // Control characters
            "id\twith\ttabs",     // Tab characters
            "id\"with'quotes",    // Quote injection
            "../../../etc/passwd", // Path traversal attempt
            "javascript:alert(1)", // Script injection attempt
            &long_id,             // Extremely long ID
        ];

        for id in malicious_ids {
            // GREEN: Should fail with malicious ID validation
            let validator = McpMessageValidator::new();
            let validation_result = validator.validate_request_id(id);
            assert!(validation_result.is_err(),
                "Expected malicious request ID '{}' validation to fail", id);
        }
    }

    #[test]
    fn test_mcp_duplicate_request_id_handling() {
        // REFACTOR: Use the new validation module's RequestTracker
        let duplicate_id = "duplicate-id-123".to_string();
        
        // Use the refactored RequestTracker
        let mut request_tracker = RequestTracker::new();
        
        // First request should succeed
        let result1 = request_tracker.track_request(&duplicate_id);
        assert!(result1.is_ok(), "First request ID should be accepted");
        assert_eq!(request_tracker.active_count(), 1);
        
        // Second request with same ID should fail
        let result2 = request_tracker.track_request(&duplicate_id);
        assert!(result2.is_err(), "Duplicate request ID should be rejected");
        
        // Test the complete_request functionality
        request_tracker.complete_request(&duplicate_id);
        assert_eq!(request_tracker.active_count(), 0);
        
        // Now the ID can be reused
        let result3 = request_tracker.track_request(&duplicate_id);
        assert!(result3.is_ok(), "Request ID should be reusable after completion");
    }

    // =============================================================================
    // Test 3c.5: Tool Arguments Security Validation Tests (RED Phase)
    // =============================================================================

    #[test]
    fn test_mcp_tool_arguments_injection_protection() {
        let malicious_arguments = HashMap::from([
            ("sql_injection".to_string(), json!("'; DROP TABLE users; --")),
            ("command_injection".to_string(), json!("$(rm -rf /)")),
            ("script_injection".to_string(), json!("<script>alert('xss')</script>")),
            ("path_traversal".to_string(), json!("../../../etc/passwd")),
            ("null_bytes".to_string(), json!("test\0\0\0")),
        ]);

        // GREEN: Should fail with argument security validation
        let validator = McpMessageValidator::new();
        let validation_result = validator.validate_tool_arguments_security(&malicious_arguments);
        assert!(validation_result.is_err(),
            "Expected malicious tool arguments validation to fail");
    }

    #[test]
    fn test_mcp_tool_arguments_resource_limits() {
        // Test resource exhaustion through tool arguments
        let mut large_args = HashMap::new();
        
        // Add many arguments to test limit
        for i in 0..1000 {
            large_args.insert(
                format!("arg_{}", i),
                json!(format!("value_{}", "x".repeat(1000)))
            );
        }

        // GREEN: Should fail with resource limit validation
        let validator = McpMessageValidator::new();
        let validation_result = validator.validate_tool_arguments_size(&large_args);
        assert!(validation_result.is_err(),
            "Expected large tool arguments validation to fail");
    }

    // =============================================================================
    // Test 3c.6: Transport Layer Validation Tests (RED Phase)
    // =============================================================================

    #[test]
    fn test_mcp_malformed_json_handling() {
        let malformed_json_messages = vec![
            "{",                    // Incomplete JSON
            "{ invalid json }",     // Malformed syntax
            "null",                 // Null message
            "{}",                   // Empty object
            "[]",                   // Array instead of object
            "\"string\"",           // String instead of object
            "{\"method\":}",        // Missing value
            "{\"method\":\"test\",}", // Trailing comma
        ];

        for json_str in malformed_json_messages {
            let validator = McpMessageValidator::new();
            let result = validator.validate_json_message(json_str);
            
            // GREEN: Should fail gracefully with proper error handling
            assert!(result.is_err(),
                "Expected JSON validation to fail for: {}", json_str);
        }
    }

    // =============================================================================
    // Test 3c.7: Edge Cases and Boundary Conditions (RED Phase)
    // =============================================================================

    #[test]
    fn test_mcp_unicode_handling() {
        // Test various Unicode edge cases that could cause issues
        let unicode_tests = vec![
            "🚀💥🔥",                    // Emojis
            "𝕿𝖊𝖘𝖙",                     // Mathematical symbols
            "\u{202E}reversed\u{202D}",  // Right-to-left override
            "\u{FEFF}bom_test",          // Byte order mark
            "test\u{0000}null",          // Embedded null
            "test\u{FFFF}invalid",       // Invalid Unicode
        ];

        for unicode_str in unicode_tests {
            // GREEN: Should validate Unicode safety
            let validator = McpMessageValidator::new();
            let validation_result = validator.validate_unicode_safety(unicode_str);
            assert!(validation_result.is_err(),
                "Expected unsafe Unicode '{}' validation to fail", unicode_str);
        }
    }

    // =============================================================================
    // REFACTOR: Helper functions are now in the validation module
    // These tests demonstrate the refactored validation architecture
    // =============================================================================
    
    #[test]
    fn test_validation_config_customization() {
        // REFACTOR: Test the configurable validation limits
        let custom_config = ValidationConfig {
            max_message_size: 512, // Very small limit
            max_nesting_depth: 5,
            max_array_length: 10,
            ..ValidationConfig::default()
        };
        
        let validator = McpMessageValidator::with_config(custom_config);
        
        // Test that small message passes
        let small_message = json!({"method": "test"});
        assert!(validator.validate_message_size(&small_message).is_ok());
        
        // Test that larger message fails with custom limit
        let larger_message = json!({"method": "test", "data": "x".repeat(600)});
        assert!(validator.validate_message_size(&larger_message).is_err());
    }
    
    #[test]
    fn test_comprehensive_message_validation() {
        // REFACTOR: Test the comprehensive validation method
        let validator = McpMessageValidator::new();
        
        // Valid MCP message should pass
        let valid_message = r#"{"jsonrpc": "2.0", "method": "tools/list", "id": "test-123"}"#;
        let result = validator.validate_complete_message(valid_message);
        assert!(result.is_ok(), "Valid MCP message should pass comprehensive validation");
        
        // Invalid message should fail
        let invalid_message = r#"{"invalid": "message"}"#;
        let result = validator.validate_complete_message(invalid_message);
        assert!(result.is_err(), "Invalid MCP message should fail comprehensive validation");
    }

    // =============================================================================
    // REFACTOR: All validation logic is now in the validation module
    // Tests now use the well-structured validation API
    // =============================================================================

    // =============================================================================
    // REFACTOR: Error types and RequestTracker are now in the validation module
    // This provides better encapsulation and reusability
    // =============================================================================
}