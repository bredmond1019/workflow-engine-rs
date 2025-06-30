//! Basic Workflow Example Library
//!
//! This library provides reusable components for building basic workflows
//! with the AI Workflow System. It demonstrates best practices for:
//!
//! - Node implementation with proper error handling
//! - Workflow construction and validation
//! - Event sourcing integration
//! - Type-safe operation patterns

pub mod nodes;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use workflow_engine_core::error::WorkflowError;

/// Configuration for text processing operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextProcessingConfig {
    /// Processing mode: "uppercase", "lowercase", "reverse", "title_case"
    pub mode: String,
    /// Whether to preserve original casing in certain modes
    pub preserve_case: bool,
    /// Optional prefix to add to processed text
    pub add_prefix: Option<String>,
    /// Optional suffix to add to processed text
    pub add_suffix: Option<String>,
    /// Maximum length for input text
    pub max_length: Option<usize>,
}

impl Default for TextProcessingConfig {
    fn default() -> Self {
        Self {
            mode: "uppercase".to_string(),
            preserve_case: false,
            add_prefix: None,
            add_suffix: None,
            max_length: Some(1000),
        }
    }
}

/// Input data for the workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInput {
    /// The text to process
    pub text: String,
    /// Processing configuration
    pub config: Option<TextProcessingConfig>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl WorkflowInput {
    /// Create new workflow input with text
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            config: None,
            metadata: HashMap::new(),
        }
    }

    /// Add configuration to the input
    pub fn with_config(mut self, config: TextProcessingConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Add metadata to the input
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Validate the input data
    pub fn validate(&self) -> Result<(), WorkflowError> {
        // Check if text is empty
        if self.text.is_empty() {
            return Err(WorkflowError::validation_error(
                "Input text cannot be empty",
                "text",
                "non-empty string required",
                "in workflow input validation"
            ));
        }

        // Check maximum length if configured
        if let Some(config) = &self.config {
            if let Some(max_length) = config.max_length {
                if self.text.len() > max_length {
                    return Err(WorkflowError::validation_error(
                        format!("Input text too long: {} > {}", self.text.len(), max_length),
                        "text",
                        format!("max length: {}", max_length),
                        "in workflow input validation"
                    ));
                }
            }
        }

        // Validate processing mode if provided
        if let Some(config) = &self.config {
            let valid_modes = ["uppercase", "lowercase", "reverse", "title_case"];
            if !valid_modes.contains(&config.mode.as_str()) {
                return Err(WorkflowError::validation_error(
                    format!("Invalid processing mode: {}", config.mode),
                    "config.mode",
                    format!("must be one of: {}", valid_modes.join(", ")),
                    "in workflow input validation"
                ));
            }
        }

        Ok(())
    }
}

/// Output data from the workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowOutput {
    /// The processed text result
    pub result: String,
    /// Original input text
    pub original: String,
    /// Processing metadata
    pub metadata: WorkflowMetadata,
}

/// Metadata about workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetadata {
    /// Unique workflow execution ID
    pub workflow_id: Uuid,
    /// Execution start time
    pub start_time: DateTime<Utc>,
    /// Execution end time
    pub end_time: Option<DateTime<Utc>>,
    /// Total execution time in milliseconds
    pub execution_time_ms: Option<u64>,
    /// Number of nodes processed
    pub nodes_processed: usize,
    /// Execution status
    pub status: WorkflowStatus,
    /// Processing steps taken
    pub steps: Vec<ProcessingStep>,
}

/// Workflow execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowStatus {
    /// Workflow is starting
    Starting,
    /// Workflow is currently running
    Running,
    /// Workflow completed successfully
    Completed,
    /// Workflow failed with error
    Failed,
    /// Workflow was cancelled
    Cancelled,
}

/// Individual processing step information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingStep {
    /// Step index in the workflow
    pub index: usize,
    /// Node ID that processed this step
    pub node_id: String,
    /// Node type name
    pub node_type: String,
    /// Step start time
    pub start_time: DateTime<Utc>,
    /// Step end time
    pub end_time: Option<DateTime<Utc>>,
    /// Step execution time in milliseconds
    pub duration_ms: Option<u64>,
    /// Whether the step completed successfully
    pub success: bool,
    /// Error message if step failed
    pub error: Option<String>,
}

impl ProcessingStep {
    /// Create a new processing step
    pub fn new(index: usize, node_id: impl Into<String>, node_type: impl Into<String>) -> Self {
        Self {
            index,
            node_id: node_id.into(),
            node_type: node_type.into(),
            start_time: Utc::now(),
            end_time: None,
            duration_ms: None,
            success: false,
            error: None,
        }
    }

    /// Mark the step as completed successfully
    pub fn complete(&mut self) {
        let end_time = Utc::now();
        self.end_time = Some(end_time);
        self.duration_ms = Some(
            (end_time - self.start_time).num_milliseconds() as u64
        );
        self.success = true;
    }

    /// Mark the step as failed with error
    pub fn fail(&mut self, error: impl Into<String>) {
        let end_time = Utc::now();
        self.end_time = Some(end_time);
        self.duration_ms = Some(
            (end_time - self.start_time).num_milliseconds() as u64
        );
        self.success = false;
        self.error = Some(error.into());
    }
}

/// Utility functions for workflow operations
pub mod utils {
    use super::*;

    /// Validate that a string is a valid workflow identifier
    pub fn validate_workflow_id(id: &str) -> Result<(), WorkflowError> {
        if id.is_empty() {
            return Err(WorkflowError::validation_error(
                "Workflow ID cannot be empty",
                "workflow_id",
                "non-empty string required",
                "in workflow ID validation"
            ));
        }

        if !id.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(WorkflowError::validation_error(
                "Workflow ID contains invalid characters",
                "workflow_id",
                "alphanumeric, underscore, and hyphen only",
                "in workflow ID validation"
            ));
        }

        if id.len() > 100 {
            return Err(WorkflowError::validation_error(
                format!("Workflow ID too long: {} > 100", id.len()),
                "workflow_id",
                "max length: 100 characters",
                "in workflow ID validation"
            ));
        }

        Ok(())
    }

    /// Create a new workflow metadata instance
    pub fn create_workflow_metadata(workflow_id: Uuid) -> WorkflowMetadata {
        WorkflowMetadata {
            workflow_id,
            start_time: Utc::now(),
            end_time: None,
            execution_time_ms: None,
            nodes_processed: 0,
            status: WorkflowStatus::Starting,
            steps: Vec::new(),
        }
    }

    /// Sanitize input text by removing dangerous characters
    pub fn sanitize_input(text: &str) -> Result<String, WorkflowError> {
        // Remove null bytes and control characters except common whitespace
        let sanitized: String = text
            .chars()
            .filter(|&c| {
                !c.is_control() || c == '\n' || c == '\r' || c == '\t'
            })
            .collect();

        if sanitized.is_empty() && !text.is_empty() {
            return Err(WorkflowError::validation_error(
                "Input text contains only invalid characters",
                "text",
                "valid text characters required",
                "in input sanitization"
            ));
        }

        Ok(sanitized)
    }

    /// Format execution time in a human-readable way
    pub fn format_duration(duration_ms: u64) -> String {
        if duration_ms < 1000 {
            format!("{}ms", duration_ms)
        } else if duration_ms < 60_000 {
            format!("{:.1}s", duration_ms as f64 / 1000.0)
        } else {
            let minutes = duration_ms / 60_000;
            let seconds = (duration_ms % 60_000) as f64 / 1000.0;
            format!("{}m {:.1}s", minutes, seconds)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_input_validation() {
        // Valid input should pass
        let valid_input = WorkflowInput::new("Hello, World!");
        assert!(valid_input.validate().is_ok());

        // Empty input should fail
        let empty_input = WorkflowInput::new("");
        assert!(empty_input.validate().is_err());

        // Too long input should fail
        let long_input = WorkflowInput::new("x".repeat(2000))
            .with_config(TextProcessingConfig {
                max_length: Some(100),
                ..Default::default()
            });
        assert!(long_input.validate().is_err());

        // Invalid mode should fail
        let invalid_mode = WorkflowInput::new("test")
            .with_config(TextProcessingConfig {
                mode: "invalid_mode".to_string(),
                ..Default::default()
            });
        assert!(invalid_mode.validate().is_err());
    }

    #[test]
    fn test_workflow_id_validation() {
        assert!(utils::validate_workflow_id("valid_workflow_id").is_ok());
        assert!(utils::validate_workflow_id("workflow-123").is_ok());
        assert!(utils::validate_workflow_id("").is_err());
        assert!(utils::validate_workflow_id("invalid@workflow").is_err());
        assert!(utils::validate_workflow_id(&"x".repeat(200)).is_err());
    }

    #[test]
    fn test_input_sanitization() {
        assert_eq!(utils::sanitize_input("hello").unwrap(), "hello");
        assert_eq!(utils::sanitize_input("hello\nworld").unwrap(), "hello\nworld");
        assert!(utils::sanitize_input("hello\x00world").unwrap().contains("helloworld"));
    }

    #[test]
    fn test_duration_formatting() {
        assert_eq!(utils::format_duration(500), "500ms");
        assert_eq!(utils::format_duration(1500), "1.5s");
        assert_eq!(utils::format_duration(65000), "1m 5.0s");
    }
}