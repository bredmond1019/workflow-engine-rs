//! Text Input Node Implementation
//!
//! This node handles input validation, sanitization, and preprocessing
//! for text-based workflows. It demonstrates proper error handling
//! with the new boxed error system.

use workflow_engine_core::nodes::Node;
use workflow_engine_core::task::TaskContext;
use workflow_engine_core::error::WorkflowError;
use serde::{Deserialize, Serialize};
use tracing::{info, debug, warn};
use crate::{WorkflowInput, utils};

/// Configuration for text input validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextInputConfig {
    /// Minimum allowed text length
    pub min_length: usize,
    /// Maximum allowed text length
    pub max_length: usize,
    /// Whether to perform input sanitization
    pub sanitize: bool,
    /// Whether to trim whitespace
    pub trim: bool,
    /// Allowed character sets
    pub allowed_chars: Option<Vec<String>>,
}

impl Default for TextInputConfig {
    fn default() -> Self {
        Self {
            min_length: 1,
            max_length: 10_000,
            sanitize: true,
            trim: true,
            allowed_chars: None,
        }
    }
}

/// Node for handling text input validation and preprocessing
#[derive(Debug)]
pub struct TextInputNode {
    config: TextInputConfig,
}

impl TextInputNode {
    /// Create a new text input node with default configuration
    pub fn new() -> Self {
        Self {
            config: TextInputConfig::default(),
        }
    }

    /// Create a new text input node with custom configuration
    pub fn with_config(config: TextInputConfig) -> Self {
        Self { config }
    }

    /// Validate the input text according to configuration
    fn validate_input(&self, text: &str) -> Result<(), WorkflowError> {
        debug!("Validating input text: length={}, config={:?}", text.len(), self.config);

        // Check minimum length
        if text.len() < self.config.min_length {
            return Err(WorkflowError::validation_error(
                format!("Input text too short: {} < {}", text.len(), self.config.min_length),
                "text",
                format!("minimum length: {}", self.config.min_length),
                "in text input validation"
            ));
        }

        // Check maximum length
        if text.len() > self.config.max_length {
            return Err(WorkflowError::validation_error(
                format!("Input text too long: {} > {}", text.len(), self.config.max_length),
                "text",
                format!("maximum length: {}", self.config.max_length),
                "in text input validation"
            ));
        }

        // Validate character sets if specified
        if let Some(allowed_chars) = &self.config.allowed_chars {
            for ch in text.chars() {
                let char_type = if ch.is_alphabetic() {
                    "alphabetic"
                } else if ch.is_numeric() {
                    "numeric"
                } else if ch.is_whitespace() {
                    "whitespace"
                } else if ch.is_ascii_punctuation() {
                    "punctuation"
                } else {
                    "other"
                };

                if !allowed_chars.contains(&char_type.to_string()) {
                    return Err(WorkflowError::validation_error(
                        format!("Invalid character '{}' of type '{}'", ch, char_type),
                        "text",
                        format!("allowed character types: {}", allowed_chars.join(", ")),
                        "in text input validation"
                    ));
                }
            }
        }

        Ok(())
    }

    /// Preprocess the input text according to configuration
    fn preprocess_input(&self, text: String) -> Result<String, WorkflowError> {
        let mut processed = text;

        // Trim whitespace if configured
        if self.config.trim {
            processed = processed.trim().to_string();
            debug!("Trimmed whitespace, new length: {}", processed.len());
        }

        // Sanitize input if configured
        if self.config.sanitize {
            processed = utils::sanitize_input(&processed)
                .map_err(|e| WorkflowError::processing_error(
                    format!("Input sanitization failed: {}", e),
                    "TextInputNode"
                ))?;
            debug!("Sanitized input, new length: {}", processed.len());
        }

        // Final validation after preprocessing
        if processed.is_empty() {
            return Err(WorkflowError::validation_error(
                "Input text is empty after preprocessing",
                "text",
                "non-empty text required",
                "in text input preprocessing"
            ));
        }

        Ok(processed)
    }
}

impl Node for TextInputNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        info!("Processing input in TextInputNode");

        // Extract input data from context
        let input_data: WorkflowInput = context.get_event_data()
            .map_err(|e| WorkflowError::deserialization_error(
                format!("Failed to extract workflow input: {}", e),
                "WorkflowInput",
                "in text input node",
                None
            ))?;

        debug!("Extracted input data: text_length={}", input_data.text.len());

        // Validate the input data structure
        input_data.validate()
            .map_err(|e| WorkflowError::validation_error(
                format!("Workflow input validation failed: {}", e),
                "input_data",
                "valid WorkflowInput structure",
                "in text input node"
            ))?;

        // Validate the text content
        self.validate_input(&input_data.text)?;

        // Preprocess the input text
        let processed_text = self.preprocess_input(input_data.text.clone())?;

        // Store processed text in context for next node
        context.update_node("validated_text", processed_text.clone());
        context.update_node("original_text", input_data.text);
        
        // Store configuration if provided
        if let Some(config) = input_data.config {
            context.update_node("processing_config", config);
        }

        // Store metadata
        context.update_node("metadata", input_data.metadata);

        // Add validation metadata
        let validation_metadata = serde_json::json!({
            "validation_passed": true,
            "original_length": input_data.text.len(),
            "processed_length": processed_text.len(),
            "sanitized": self.config.sanitize,
            "trimmed": self.config.trim,
            "node_type": "TextInputNode"
        });
        context.update_node("validation_metadata", validation_metadata);

        info!("Text input validation completed successfully");
        Ok(context)
    }
}

impl Default for TextInputNode {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use workflow_engine_core::task::TaskContext;
    use serde_json::json;

    #[test]
    fn test_text_input_validation() {
        let node = TextInputNode::new();

        // Valid input should pass
        assert!(node.validate_input("Hello, World!").is_ok());

        // Empty input should fail (below min length)
        assert!(node.validate_input("").is_err());

        // Very long input should fail
        let long_text = "x".repeat(20_000);
        assert!(node.validate_input(&long_text).is_err());
    }

    #[test]
    fn test_text_input_preprocessing() {
        let node = TextInputNode::new();

        // Trimming should work
        let result = node.preprocess_input("  hello world  ".to_string()).unwrap();
        assert_eq!(result, "hello world");

        // Sanitization should remove control characters
        let result = node.preprocess_input("hello\x00world".to_string()).unwrap();
        assert!(!result.contains('\x00'));
    }

    #[test]
    fn test_character_set_validation() {
        let config = TextInputConfig {
            allowed_chars: Some(vec!["alphabetic".to_string(), "whitespace".to_string()]),
            ..Default::default()
        };
        let node = TextInputNode::with_config(config);

        // Only letters and spaces should pass
        assert!(node.validate_input("hello world").is_ok());

        // Numbers should fail
        assert!(node.validate_input("hello123").is_err());

        // Punctuation should fail
        assert!(node.validate_input("hello!").is_err());
    }

    #[test] 
    fn test_node_processing() {
        let node = TextInputNode::new();
        let mut context = TaskContext::new();

        // Create test input
        let input = WorkflowInput::new("  Hello, World!  ");
        context.set_event_data(json!(input)).unwrap();

        // Process should succeed
        let result = node.process(context);
        assert!(result.is_ok());

        let result_context = result.unwrap();
        
        // Check that validated text was stored
        let validated_text: String = result_context.get_data("validated_text").unwrap();
        assert_eq!(validated_text, "Hello, World!");

        // Check that original text was preserved
        let original_text: String = result_context.get_data("original_text").unwrap();
        assert_eq!(original_text, "  Hello, World!  ");

        // Check that validation metadata was added
        let metadata: serde_json::Value = result_context.get_data("validation_metadata").unwrap();
        assert_eq!(metadata["validation_passed"], true);
        assert_eq!(metadata["node_type"], "TextInputNode");
    }
}