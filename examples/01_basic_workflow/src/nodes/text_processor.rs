//! Text Processor Node Implementation
//!
//! This node performs the core text processing operations with configurable
//! transformation modes. It demonstrates proper error handling and configuration
//! management within the workflow system.

use workflow_engine_core::nodes::Node;
use workflow_engine_core::task::TaskContext;
use workflow_engine_core::error::WorkflowError;
use serde::{Deserialize, Serialize};
use tracing::{info, debug, warn};
use crate::TextProcessingConfig;

/// Text transformation operations supported by the processor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransformMode {
    /// Convert text to uppercase
    Uppercase,
    /// Convert text to lowercase
    Lowercase,
    /// Reverse the character order
    Reverse,
    /// Convert to title case (first letter of each word capitalized)
    TitleCase,
    /// Add custom prefix and/or suffix
    Wrap { prefix: String, suffix: String },
    /// Replace specific patterns
    Replace { from: String, to: String },
    /// Count words and characters
    Analyze,
}

impl TransformMode {
    /// Parse a transform mode from a string
    pub fn from_string(mode: &str) -> Result<Self, WorkflowError> {
        match mode.to_lowercase().as_str() {
            "uppercase" => Ok(TransformMode::Uppercase),
            "lowercase" => Ok(TransformMode::Lowercase),
            "reverse" => Ok(TransformMode::Reverse),
            "title_case" | "titlecase" => Ok(TransformMode::TitleCase),
            "analyze" => Ok(TransformMode::Analyze),
            _ => Err(WorkflowError::validation_error(
                format!("Unknown transform mode: {}", mode),
                "transform_mode",
                "uppercase, lowercase, reverse, title_case, analyze",
                "in transform mode parsing"
            )),
        }
    }
}

/// Configuration for the text processor node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextProcessorNodeConfig {
    /// Default transformation mode
    pub default_mode: TransformMode,
    /// Whether to preserve whitespace patterns
    pub preserve_whitespace: bool,
    /// Maximum processing time in milliseconds
    pub max_processing_time_ms: Option<u64>,
}

impl Default for TextProcessorNodeConfig {
    fn default() -> Self {
        Self {
            default_mode: TransformMode::Uppercase,
            preserve_whitespace: true,
            max_processing_time_ms: Some(5000), // 5 second timeout
        }
    }
}

/// Node for performing text transformations
#[derive(Debug)]
pub struct TextProcessorNode {
    config: TextProcessorNodeConfig,
}

impl TextProcessorNode {
    /// Create a new text processor node with default configuration
    pub fn new() -> Self {
        Self {
            config: TextProcessorNodeConfig::default(),
        }
    }

    /// Create a new text processor node with custom configuration
    pub fn with_config(config: TextProcessorNodeConfig) -> Self {
        Self { config }
    }

    /// Determine the transformation mode from context or use default
    fn determine_transform_mode(&self, context: &TaskContext) -> Result<TransformMode, WorkflowError> {
        // Try to get processing config from context (set by input node)
        if let Ok(processing_config) = context.get_data::<TextProcessingConfig>("processing_config") {
            return TransformMode::from_string(&processing_config.mode);
        }

        // Fall back to default mode
        Ok(self.config.default_mode.clone())
    }

    /// Apply the specified transformation to the input text
    fn apply_transformation(&self, text: &str, mode: &TransformMode) -> Result<serde_json::Value, WorkflowError> {
        debug!("Applying transformation {:?} to text of length {}", mode, text.len());

        let result = match mode {
            TransformMode::Uppercase => {
                serde_json::json!({
                    "transformed_text": text.to_uppercase(),
                    "operation": "uppercase",
                    "original_length": text.len(),
                    "transformed_length": text.len()
                })
            }
            
            TransformMode::Lowercase => {
                serde_json::json!({
                    "transformed_text": text.to_lowercase(),
                    "operation": "lowercase", 
                    "original_length": text.len(),
                    "transformed_length": text.len()
                })
            }
            
            TransformMode::Reverse => {
                let reversed: String = if self.config.preserve_whitespace {
                    // Reverse character order but preserve word boundaries
                    text.split_whitespace()
                        .map(|word| word.chars().rev().collect::<String>())
                        .collect::<Vec<_>>()
                        .join(" ")
                } else {
                    // Simple character reversal
                    text.chars().rev().collect()
                };
                
                serde_json::json!({
                    "transformed_text": reversed,
                    "operation": "reverse",
                    "original_length": text.len(),
                    "transformed_length": reversed.len(),
                    "preserve_whitespace": self.config.preserve_whitespace
                })
            }
            
            TransformMode::TitleCase => {
                let title_case = text
                    .split_whitespace()
                    .map(|word| {
                        let mut chars = word.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                
                serde_json::json!({
                    "transformed_text": title_case,
                    "operation": "title_case",
                    "original_length": text.len(),
                    "transformed_length": title_case.len()
                })
            }
            
            TransformMode::Wrap { prefix, suffix } => {
                let wrapped = format!("{}{}{}", prefix, text, suffix);
                serde_json::json!({
                    "transformed_text": wrapped,
                    "operation": "wrap",
                    "prefix": prefix,
                    "suffix": suffix,
                    "original_length": text.len(),
                    "transformed_length": wrapped.len()
                })
            }
            
            TransformMode::Replace { from, to } => {
                let replaced = text.replace(from, to);
                let replacement_count = (text.len() - replaced.len() + to.len()) / from.len();
                serde_json::json!({
                    "transformed_text": replaced,
                    "operation": "replace",
                    "from": from,
                    "to": to,
                    "replacement_count": replacement_count,
                    "original_length": text.len(),
                    "transformed_length": replaced.len()
                })
            }
            
            TransformMode::Analyze => {
                let word_count = text.split_whitespace().count();
                let char_count = text.chars().count();
                let char_count_no_spaces = text.chars().filter(|&c| !c.is_whitespace()).count();
                let line_count = text.lines().count();
                
                // Character frequency analysis
                let mut char_freq = std::collections::HashMap::new();
                for ch in text.chars() {
                    *char_freq.entry(ch).or_insert(0) += 1;
                }
                
                let most_common_char = char_freq
                    .iter()
                    .max_by_key(|(_, &count)| count)
                    .map(|(&ch, &count)| (ch, count));
                
                serde_json::json!({
                    "transformed_text": text, // Analysis doesn't change the text
                    "operation": "analyze",
                    "analysis": {
                        "word_count": word_count,
                        "character_count": char_count,
                        "character_count_no_spaces": char_count_no_spaces,
                        "line_count": line_count,
                        "most_common_character": most_common_char.map(|(ch, count)| {
                            json!({
                                "character": ch.to_string(),
                                "count": count
                            })
                        }),
                        "unique_characters": char_freq.len()
                    }
                })
            }
        };

        Ok(result)
    }

    /// Extract processing configuration from workflow input
    fn extract_processing_config(&self, context: &TaskContext) -> Option<TextProcessingConfig> {
        context.get_data::<TextProcessingConfig>("processing_config").ok()
    }
}

impl Node for TextProcessorNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        info!("Processing text in TextProcessorNode");

        // Extract the validated text from previous node
        let validated_text: String = context.get_data("validated_text")
            .map_err(|_| WorkflowError::processing_error(
                "Missing validated_text from previous node",
                "TextProcessorNode"
            ))?;

        debug!("Processing text: length={}", validated_text.len());

        // Determine transformation mode
        let transform_mode = self.determine_transform_mode(&context)?;
        debug!("Using transform mode: {:?}", transform_mode);

        // Apply transformation with timeout protection
        let start_time = std::time::Instant::now();
        let transformation_result = self.apply_transformation(&validated_text, &transform_mode)?;
        let processing_duration = start_time.elapsed();

        // Check for timeout
        if let Some(max_time) = self.config.max_processing_time_ms {
            if processing_duration.as_millis() > max_time as u128 {
                warn!("Text processing took {}ms, exceeding limit of {}ms", 
                     processing_duration.as_millis(), max_time);
                return Err(WorkflowError::processing_error(
                    format!("Processing timeout: {}ms > {}ms", 
                           processing_duration.as_millis(), max_time),
                    "TextProcessorNode"
                ));
            }
        }

        // Extract transformed text from result
        let transformed_text = transformation_result["transformed_text"]
            .as_str()
            .ok_or_else(|| WorkflowError::processing_error(
                "Failed to extract transformed text from result",
                "TextProcessorNode"
            ))?
            .to_string();

        // Store results in context
        context.update_node("processed_text", transformed_text);
        context.update_node("transformation_result", transformation_result.clone());
        
        // Add processing metadata
        let processing_metadata = serde_json::json!({
            "node_type": "TextProcessorNode",
            "transform_mode": format!("{:?}", transform_mode),
            "processing_duration_ms": processing_duration.as_millis(),
            "processing_config": self.extract_processing_config(&context),
            "preserved_whitespace": self.config.preserve_whitespace
        });
        context.update_node("processing_metadata", processing_metadata);

        info!("Text processing completed in {}ms", processing_duration.as_millis());
        Ok(context)
    }
}

impl Default for TextProcessorNode {
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
    fn test_transform_mode_parsing() {
        assert!(matches!(
            TransformMode::from_string("uppercase").unwrap(),
            TransformMode::Uppercase
        ));
        assert!(matches!(
            TransformMode::from_string("LOWERCASE").unwrap(),
            TransformMode::Lowercase
        ));
        assert!(TransformMode::from_string("invalid").is_err());
    }

    #[test]
    fn test_uppercase_transformation() {
        let node = TextProcessorNode::new();
        let result = node.apply_transformation("hello world", &TransformMode::Uppercase).unwrap();
        
        assert_eq!(result["transformed_text"], "HELLO WORLD");
        assert_eq!(result["operation"], "uppercase");
    }

    #[test]
    fn test_reverse_transformation() {
        let node = TextProcessorNode::new();
        let result = node.apply_transformation("hello world", &TransformMode::Reverse).unwrap();
        
        // With preserve_whitespace=true, should reverse words individually
        assert_eq!(result["transformed_text"], "olleh dlrow");
        assert_eq!(result["operation"], "reverse");
    }

    #[test]
    fn test_title_case_transformation() {
        let node = TextProcessorNode::new();
        let result = node.apply_transformation("hello world test", &TransformMode::TitleCase).unwrap();
        
        assert_eq!(result["transformed_text"], "Hello World Test");
        assert_eq!(result["operation"], "title_case");
    }

    #[test]
    fn test_analyze_transformation() {
        let node = TextProcessorNode::new();
        let result = node.apply_transformation("hello world", &TransformMode::Analyze).unwrap();
        
        assert_eq!(result["transformed_text"], "hello world");
        assert_eq!(result["operation"], "analyze");
        assert_eq!(result["analysis"]["word_count"], 2);
        assert_eq!(result["analysis"]["character_count"], 11);
    }

    #[test]
    fn test_node_processing() {
        let node = TextProcessorNode::new();
        let mut context = TaskContext::new();

        // Set up context with validated text
        context.update_node("validated_text", "hello world".to_string());

        // Process should succeed
        let result = node.process(context);
        assert!(result.is_ok());

        let result_context = result.unwrap();
        
        // Check that processed text was stored
        let processed_text: String = result_context.get_data("processed_text").unwrap();
        assert_eq!(processed_text, "HELLO WORLD"); // Default is uppercase

        // Check that metadata was added
        let metadata: serde_json::Value = result_context.get_data("processing_metadata").unwrap();
        assert_eq!(metadata["node_type"], "TextProcessorNode");
        assert!(metadata["processing_duration_ms"].as_u64().unwrap() >= 0);
    }

    #[test]
    fn test_missing_input_error() {
        let node = TextProcessorNode::new();
        let context = TaskContext::new(); // No validated_text

        // Process should fail with appropriate error
        let result = node.process(context);
        assert!(result.is_err());
        
        if let Err(WorkflowError::ProcessingError(details)) = result {
            assert!(details.message.contains("Missing validated_text"));
            assert_eq!(details.node_type, "TextProcessorNode");
        } else {
            panic!("Expected ProcessingError");
        }
    }
}