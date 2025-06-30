//! MCP Protocol Message Validation
//! 
//! This module provides comprehensive validation for MCP protocol messages to prevent:
//! - Security vulnerabilities (injection attacks, malicious content)
//! - Resource exhaustion (oversized messages, excessive nesting)
//! - Protocol violations (malformed JSON-RPC, invalid structure)
//! 
//! The validation approach follows security-first principles with conservative
//! acceptance criteria to ensure system safety and protocol compliance.

use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

/// Type alias for validation results
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Comprehensive error types for MCP protocol validation
#[derive(Debug, Error, Clone)]
pub enum ValidationError {
    #[error("Unsupported protocol version: {0}")]
    UnsupportedProtocolVersion(String),
    
    #[error("Invalid request ID: {0}")]
    InvalidRequestId(String),
    
    #[error("Duplicate request ID: {0}")]
    DuplicateRequestId(String),
    
    #[error("Malicious arguments detected")]
    MaliciousArguments,
    
    #[error("Arguments too large")]
    ArgumentsTooBig,
    
    #[error("Malformed JSON: {0}")]
    MalformedJson(String),
    
    #[error("Message too large")]
    MessageTooLarge,
    
    #[error("Nesting too deep")]
    NestingTooDeep,
    
    #[error("Array too long")]
    ArrayTooLong,
}

/// Configuration for MCP message validation limits
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// Maximum JSON nesting depth
    pub max_nesting_depth: usize,
    /// Maximum array length
    pub max_array_length: usize,
    /// Maximum request ID length
    pub max_request_id_length: usize,
    /// Maximum number of tool arguments
    pub max_tool_arguments: usize,
    /// Maximum size of all tool arguments combined
    pub max_tool_arguments_size: usize,
    /// Maximum method name length
    pub max_method_name_length: usize,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_message_size: 1024 * 1024, // 1MB
            max_nesting_depth: 100,
            max_array_length: 10_000,
            max_request_id_length: 1000,
            max_tool_arguments: 100,
            max_tool_arguments_size: 1024 * 1024, // 1MB
            max_method_name_length: 100,
        }
    }
}

/// Main validator for MCP protocol messages
#[derive(Debug)]
pub struct McpMessageValidator {
    config: ValidationConfig,
}

impl McpMessageValidator {
    /// Create a new validator with default configuration
    pub fn new() -> Self {
        Self {
            config: ValidationConfig::default(),
        }
    }
    
    /// Create a new validator with custom configuration
    pub fn with_config(config: ValidationConfig) -> Self {
        Self { config }
    }
    
    /// Validate message size limits
    pub fn validate_message_size(&self, message: &Value) -> ValidationResult<()> {
        let size = message.to_string().len();
        if size > self.config.max_message_size {
            Err(ValidationError::MessageTooLarge)
        } else {
            Ok(())
        }
    }
    
    /// Validate JSON nesting depth to prevent stack overflow
    pub fn validate_nesting_depth(&self, message: &Value) -> ValidationResult<()> {
        fn count_depth(value: &Value, current_depth: usize) -> usize {
            match value {
                Value::Object(map) => {
                    map.values()
                        .map(|v| count_depth(v, current_depth + 1))
                        .max()
                        .unwrap_or(current_depth)
                }
                Value::Array(arr) => {
                    arr.iter()
                        .map(|v| count_depth(v, current_depth + 1))
                        .max()
                        .unwrap_or(current_depth)
                }
                _ => current_depth,
            }
        }
        
        let depth = count_depth(message, 0);
        if depth > self.config.max_nesting_depth {
            Err(ValidationError::NestingTooDeep)
        } else {
            Ok(())
        }
    }
    
    /// Validate array sizes to prevent memory exhaustion
    pub fn validate_array_sizes(&self, message: &Value) -> ValidationResult<()> {
        fn check_arrays(value: &Value, max_length: usize) -> ValidationResult<()> {
            match value {
                Value::Array(arr) => {
                    if arr.len() > max_length {
                        return Err(ValidationError::ArrayTooLong);
                    }
                    for item in arr {
                        check_arrays(item, max_length)?;
                    }
                }
                Value::Object(map) => {
                    for val in map.values() {
                        check_arrays(val, max_length)?;
                    }
                }
                _ => {}
            }
            Ok(())
        }
        
        check_arrays(message, self.config.max_array_length)
    }
    
    /// Validate JSON-RPC format compliance
    pub fn validate_jsonrpc_format(&self, message: &Value) -> ValidationResult<()> {
        if !message.get("jsonrpc").is_some() {
            Err(ValidationError::MalformedJson("Missing jsonrpc field".to_string()))
        } else {
            Ok(())
        }
    }
    
    /// Validate JSON-RPC version
    pub fn validate_jsonrpc_version(&self, message: &Value) -> ValidationResult<()> {
        match message.get("jsonrpc").and_then(|v| v.as_str()) {
            Some("2.0") => Ok(()),
            Some(version) => Err(ValidationError::UnsupportedProtocolVersion(version.to_string())),
            None => Err(ValidationError::MalformedJson("Missing jsonrpc version".to_string())),
        }
    }
    
    /// Validate method names according to MCP protocol rules
    pub fn validate_method_name(&self, message: &Value) -> ValidationResult<()> {
        match message.get("method").and_then(|v| v.as_str()) {
            Some(method) if method.is_empty() => {
                Err(ValidationError::InvalidRequestId("Empty method".to_string()))
            }
            Some(method) if method.len() > self.config.max_method_name_length => {
                Err(ValidationError::InvalidRequestId("Method too long".to_string()))
            }
            Some(method) if method.contains('\n') || method.contains('\0') || method.contains('\t') => {
                Err(ValidationError::InvalidRequestId("Invalid characters in method".to_string()))
            }
            Some(method) if method.contains(' ') => {
                Err(ValidationError::InvalidRequestId("Method cannot contain spaces".to_string()))
            }
            Some(method) if method.matches('/').count() > 1 => {
                Err(ValidationError::InvalidRequestId("Too many slashes in method".to_string()))
            }
            Some(_) => Ok(()),
            None => Err(ValidationError::MalformedJson("Missing method field".to_string())),
        }
    }
    
    /// Validate protocol version compatibility
    pub fn validate_protocol_version(&self, version: &str) -> ValidationResult<()> {
        if version == "2024-11-05" {
            Ok(())
        } else {
            Err(ValidationError::UnsupportedProtocolVersion(version.to_string()))
        }
    }
    
    /// Validate request IDs for security and compliance
    pub fn validate_request_id(&self, id: &str) -> ValidationResult<()> {
        if id.is_empty() {
            return Err(ValidationError::InvalidRequestId("Empty request ID".to_string()));
        }
        
        if id.len() > self.config.max_request_id_length {
            return Err(ValidationError::InvalidRequestId("Request ID too long".to_string()));
        }
        
        // Check for control characters
        if id.contains('\0') || id.contains('\n') || id.contains('\t') || id.contains('\r') {
            return Err(ValidationError::InvalidRequestId("Request ID contains control characters".to_string()));
        }
        
        // Check for potential injection patterns
        if id.contains("../") || id.contains("javascript:") || id.contains("<script") {
            return Err(ValidationError::InvalidRequestId("Request ID contains suspicious patterns".to_string()));
        }
        
        // Check for quote characters that could cause injection
        if id.contains('"') || id.contains('\'') {
            return Err(ValidationError::InvalidRequestId("Request ID contains quote characters".to_string()));
        }
        
        Ok(())
    }
    
    /// Validate tool arguments for security threats
    pub fn validate_tool_arguments_security(&self, args: &HashMap<String, Value>) -> ValidationResult<()> {
        for (_key, value) in args {
            if let Some(string_val) = value.as_str() {
                if string_val.contains("DROP TABLE") || 
                   string_val.contains("rm -rf") ||
                   string_val.contains("<script>") ||
                   string_val.contains("../") ||
                   string_val.contains('\0') {
                    return Err(ValidationError::MaliciousArguments);
                }
            }
        }
        Ok(())
    }
    
    /// Validate tool arguments size limits
    pub fn validate_tool_arguments_size(&self, args: &HashMap<String, Value>) -> ValidationResult<()> {
        let total_size: usize = args.iter()
            .map(|(k, v)| k.len() + v.to_string().len())
            .sum();
        
        if total_size > self.config.max_tool_arguments_size || args.len() > self.config.max_tool_arguments {
            Err(ValidationError::ArgumentsTooBig)
        } else {
            Ok(())
        }
    }
    
    /// Validate JSON message structure and required fields
    pub fn validate_json_message(&self, json_str: &str) -> ValidationResult<()> {
        match serde_json::from_str::<Value>(json_str) {
            Ok(Value::Object(obj)) => {
                // Valid JSON object - check if it has required MCP fields
                if obj.is_empty() {
                    return Err(ValidationError::MalformedJson("Empty object is not a valid MCP message".to_string()));
                }
                
                // MCP messages should have either "method" (for requests) or "result"/"error" (for responses)
                let has_method = obj.contains_key("method");
                let has_result = obj.contains_key("result");
                let has_error = obj.contains_key("error");
                
                if !(has_method || has_result || has_error) {
                    return Err(ValidationError::MalformedJson("MCP message must have method, result, or error field".to_string()));
                }
                
                Ok(())
            }
            Ok(Value::Null) => {
                Err(ValidationError::MalformedJson("Message cannot be null".to_string()))
            }
            Ok(Value::Array(_)) => {
                Err(ValidationError::MalformedJson("Message must be an object, not an array".to_string()))
            }
            Ok(Value::String(_)) | Ok(Value::Number(_)) | Ok(Value::Bool(_)) => {
                Err(ValidationError::MalformedJson("Message must be an object".to_string()))
            }
            Err(_) => {
                Err(ValidationError::MalformedJson("Invalid JSON syntax".to_string()))
            }
        }
    }
    
    /// Validate Unicode safety for text content
    pub fn validate_unicode_safety(&self, text: &str) -> ValidationResult<()> {
        // Check for null bytes
        if text.contains('\0') {
            return Err(ValidationError::InvalidRequestId("Text contains null bytes".to_string()));
        }
        
        // Check for byte order mark (BOM)
        if text.contains('\u{FEFF}') {
            return Err(ValidationError::InvalidRequestId("Text contains byte order mark".to_string()));
        }
        
        // Check for right-to-left override (can be used for visual spoofing)
        if text.contains('\u{202E}') || text.contains('\u{202D}') {
            return Err(ValidationError::InvalidRequestId("Text contains bidirectional override characters".to_string()));
        }
        
        // Check for other potentially dangerous Unicode characters
        if text.contains('\u{FFFF}') || text.contains('\u{FFFE}') {
            return Err(ValidationError::InvalidRequestId("Text contains non-character Unicode points".to_string()));
        }
        
        // For MCP protocol safety, be very conservative about Unicode
        // Only allow ASCII alphanumeric, basic punctuation, and whitespace
        let has_only_safe_chars = text.chars().all(|c| {
            c.is_ascii_alphanumeric() || 
            c.is_ascii_whitespace() ||
            c.is_ascii_punctuation()
        });
        
        if !has_only_safe_chars {
            return Err(ValidationError::InvalidRequestId("Text contains non-ASCII Unicode characters".to_string()));
        }
        
        Ok(())
    }
    
    /// Comprehensive validation of an MCP message
    pub fn validate_complete_message(&self, json_str: &str) -> ValidationResult<Value> {
        // First validate JSON structure
        self.validate_json_message(json_str)?;
        
        // Parse the message for further validation
        let message: Value = serde_json::from_str(json_str)
            .map_err(|_| ValidationError::MalformedJson("Failed to parse JSON".to_string()))?;
        
        // Validate size limits
        self.validate_message_size(&message)?;
        
        // Validate structure limits
        self.validate_nesting_depth(&message)?;
        self.validate_array_sizes(&message)?;
        
        // Validate JSON-RPC compliance (if present)
        if message.get("jsonrpc").is_some() {
            self.validate_jsonrpc_version(&message)?;
        }
        
        // Validate method name (if present)
        if message.get("method").is_some() {
            self.validate_method_name(&message)?;
        }
        
        // Validate request ID (if present)
        if let Some(id) = message.get("id").and_then(|v| v.as_str()) {
            self.validate_request_id(id)?;
        }
        
        Ok(message)
    }
}

impl Default for McpMessageValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Request tracker for detecting duplicate request IDs
#[derive(Debug, Default)]
pub struct RequestTracker {
    active_requests: std::collections::HashSet<String>,
}

impl RequestTracker {
    /// Create a new request tracker
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Track a new request ID, returning error if duplicate
    pub fn track_request(&mut self, id: &str) -> ValidationResult<()> {
        if self.active_requests.contains(id) {
            Err(ValidationError::DuplicateRequestId(id.to_string()))
        } else {
            self.active_requests.insert(id.to_string());
            Ok(())
        }
    }
    
    /// Remove a request ID when the request is completed
    pub fn complete_request(&mut self, id: &str) {
        self.active_requests.remove(id);
    }
    
    /// Get the number of active requests
    pub fn active_count(&self) -> usize {
        self.active_requests.len()
    }
    
    /// Clear all active requests (useful for cleanup)
    pub fn clear(&mut self) {
        self.active_requests.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validator_creation() {
        let validator = McpMessageValidator::new();
        assert!(validator.config.max_message_size > 0);
        
        let custom_config = ValidationConfig {
            max_message_size: 512,
            ..ValidationConfig::default()
        };
        let custom_validator = McpMessageValidator::with_config(custom_config);
        assert_eq!(custom_validator.config.max_message_size, 512);
    }
    
    #[test]
    fn test_message_size_validation() {
        let validator = McpMessageValidator::new();
        
        let small_message = json!({"test": "small"});
        assert!(validator.validate_message_size(&small_message).is_ok());
        
        // Test with custom small limit
        let small_config = ValidationConfig {
            max_message_size: 10,
            ..ValidationConfig::default()
        };
        let small_validator = McpMessageValidator::with_config(small_config);
        assert!(small_validator.validate_message_size(&small_message).is_err());
    }
    
    #[test]
    fn test_request_tracker() {
        let mut tracker = RequestTracker::new();
        
        // First request should succeed
        assert!(tracker.track_request("req-1").is_ok());
        assert_eq!(tracker.active_count(), 1);
        
        // Duplicate should fail
        assert!(tracker.track_request("req-1").is_err());
        
        // Different request should succeed
        assert!(tracker.track_request("req-2").is_ok());
        assert_eq!(tracker.active_count(), 2);
        
        // Complete request
        tracker.complete_request("req-1");
        assert_eq!(tracker.active_count(), 1);
        
        // Now the ID can be reused
        assert!(tracker.track_request("req-1").is_ok());
        
        // Clear all
        tracker.clear();
        assert_eq!(tracker.active_count(), 0);
    }
}