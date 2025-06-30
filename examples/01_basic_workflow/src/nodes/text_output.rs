//! Text Output Node Implementation
//!
//! This node handles result formatting, metadata collection, and final
//! output preparation. It demonstrates how to aggregate data from multiple
//! processing steps and format comprehensive results.

use workflow_engine_core::nodes::Node;
use workflow_engine_core::task::TaskContext;
use workflow_engine_core::error::WorkflowError;
use serde::{Deserialize, Serialize};
use tracing::{info, debug};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::{WorkflowOutput, WorkflowMetadata, WorkflowStatus, ProcessingStep};

/// Configuration for output formatting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextOutputConfig {
    /// Whether to include detailed metadata in output
    pub include_metadata: bool,
    /// Whether to include processing steps
    pub include_steps: bool,
    /// Whether to include original text in output
    pub include_original: bool,
    /// Output format preference
    pub format: OutputFormat,
    /// Whether to prettify JSON output
    pub pretty_json: bool,
}

impl Default for TextOutputConfig {
    fn default() -> Self {
        Self {
            include_metadata: true,
            include_steps: true,
            include_original: true,
            format: OutputFormat::Json,
            pretty_json: true,
        }
    }
}

/// Supported output formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    /// JSON output with full metadata
    Json,
    /// Plain text output (result only)
    Text,
    /// Structured summary with key metrics
    Summary,
}

/// Node for formatting final workflow output
#[derive(Debug)]
pub struct TextOutputNode {
    config: TextOutputConfig,
}

impl TextOutputNode {
    /// Create a new text output node with default configuration
    pub fn new() -> Self {
        Self {
            config: TextOutputConfig::default(),
        }
    }

    /// Create a new text output node with custom configuration
    pub fn with_config(config: TextOutputConfig) -> Self {
        Self { config }
    }

    /// Collect all processing steps from the context
    fn collect_processing_steps(&self, context: &TaskContext) -> Vec<ProcessingStep> {
        let mut steps = Vec::new();

        // Step 1: Text Input
        if let (Ok(validation_metadata), Ok(_)) = (
            context.get_data::<serde_json::Value>("validation_metadata"),
            context.get_data::<String>("validated_text")
        ) {
            let mut step = ProcessingStep::new(0, "text_input", "TextInputNode");
            step.complete();
            if let Some(duration) = validation_metadata.get("processing_duration_ms") {
                if let Some(duration_num) = duration.as_u64() {
                    step.duration_ms = Some(duration_num);
                }
            }
            steps.push(step);
        }

        // Step 2: Text Processing
        if let (Ok(processing_metadata), Ok(_)) = (
            context.get_data::<serde_json::Value>("processing_metadata"),
            context.get_data::<String>("processed_text")
        ) {
            let mut step = ProcessingStep::new(1, "text_processor", "TextProcessorNode");
            step.complete();
            if let Some(duration) = processing_metadata.get("processing_duration_ms") {
                if let Some(duration_num) = duration.as_u64() {
                    step.duration_ms = Some(duration_num);
                }
            }
            steps.push(step);
        }

        // Step 3: Text Output (this step)
        let mut output_step = ProcessingStep::new(2, "text_output", "TextOutputNode");
        output_step.complete();
        steps.push(output_step);

        steps
    }

    /// Create comprehensive workflow metadata
    fn create_workflow_metadata(&self, context: &TaskContext, workflow_id: Uuid, start_time: DateTime<Utc>) -> WorkflowMetadata {
        let steps = if self.config.include_steps {
            self.collect_processing_steps(context)
        } else {
            Vec::new()
        };

        let end_time = Utc::now();
        let execution_time_ms = (end_time - start_time).num_milliseconds() as u64;

        WorkflowMetadata {
            workflow_id,
            start_time,
            end_time: Some(end_time),
            execution_time_ms: Some(execution_time_ms),
            nodes_processed: steps.len(),
            status: WorkflowStatus::Completed,
            steps,
        }
    }

    /// Format output according to configuration
    fn format_output(&self, output: &WorkflowOutput) -> Result<String, WorkflowError> {
        match self.config.format {
            OutputFormat::Json => {
                if self.config.pretty_json {
                    serde_json::to_string_pretty(output)
                } else {
                    serde_json::to_string(output)
                }
                .map_err(|e| WorkflowError::serialization_error(
                    format!("Failed to serialize output to JSON: {}", e),
                    "WorkflowOutput",
                    "in text output formatting"
                ))
            }
            
            OutputFormat::Text => {
                Ok(output.result.clone())
            }
            
            OutputFormat::Summary => {
                let mut summary = String::new();
                summary.push_str(&format!("Result: {}\n", output.result));
                summary.push_str(&format!("Original: {}\n", output.original));
                summary.push_str(&format!("Workflow ID: {}\n", output.metadata.workflow_id));
                summary.push_str(&format!("Status: {:?}\n", output.metadata.status));
                summary.push_str(&format!("Nodes Processed: {}\n", output.metadata.nodes_processed));
                
                if let Some(execution_time) = output.metadata.execution_time_ms {
                    summary.push_str(&format!("Execution Time: {}ms\n", execution_time));
                }
                
                if self.config.include_steps && !output.metadata.steps.is_empty() {
                    summary.push_str("\nProcessing Steps:\n");
                    for step in &output.metadata.steps {
                        summary.push_str(&format!(
                            "  {}: {} ({}ms)\n",
                            step.index + 1,
                            step.node_type,
                            step.duration_ms.unwrap_or(0)
                        ));
                    }
                }
                
                Ok(summary)
            }
        }
    }

    /// Validate that all required data is present in context
    fn validate_context(&self, context: &TaskContext) -> Result<(), WorkflowError> {
        // Check for processed text
        if context.get_data::<String>("processed_text").is_err() {
            return Err(WorkflowError::validation_error(
                "Missing processed_text from previous processing node",
                "processed_text",
                "valid processed text result",
                "in text output validation"
            ));
        }

        // Check for original text
        if context.get_data::<String>("original_text").is_err() {
            return Err(WorkflowError::validation_error(
                "Missing original_text from input node",
                "original_text",
                "valid original text input",
                "in text output validation"
            ));
        }

        Ok(())
    }
}

impl Node for TextOutputNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        info!("Formatting output in TextOutputNode");

        // Validate context has required data
        self.validate_context(&context)?;

        // Extract all necessary data from context
        let processed_text: String = context.get_data("processed_text")
            .map_err(|e| WorkflowError::processing_error(
                format!("Failed to extract processed text: {}", e),
                "TextOutputNode"
            ))?;

        let original_text: String = context.get_data("original_text")
            .map_err(|e| WorkflowError::processing_error(
                format!("Failed to extract original text: {}", e),
                "TextOutputNode"
            ))?;

        debug!("Formatting output: processed_length={}, original_length={}", 
               processed_text.len(), original_text.len());

        // Generate workflow ID and timing
        let workflow_id = Uuid::new_v4();
        let start_time = Utc::now() - chrono::Duration::milliseconds(100); // Approximate start time

        // Create comprehensive metadata
        let metadata = if self.config.include_metadata {
            self.create_workflow_metadata(&context, workflow_id, start_time)
        } else {
            WorkflowMetadata {
                workflow_id,
                start_time,
                end_time: Some(Utc::now()),
                execution_time_ms: Some(100), // Minimal metadata
                nodes_processed: 3,
                status: WorkflowStatus::Completed,
                steps: Vec::new(),
            }
        };

        // Create final output structure
        let workflow_output = WorkflowOutput {
            result: processed_text.clone(),
            original: if self.config.include_original {
                original_text
            } else {
                String::new()
            },
            metadata,
        };

        // Format output according to configuration
        let formatted_output = self.format_output(&workflow_output)?;

        // Store results in context
        context.update_node("final_output", workflow_output.clone());
        context.update_node("formatted_output", formatted_output.clone());
        
        // Add output metadata
        let output_metadata = serde_json::json!({
            "node_type": "TextOutputNode",
            "output_format": format!("{:?}", self.config.format),
            "include_metadata": self.config.include_metadata,
            "include_steps": self.config.include_steps,
            "include_original": self.config.include_original,
            "formatted_length": formatted_output.len(),
            "workflow_id": workflow_output.metadata.workflow_id,
            "execution_time_ms": workflow_output.metadata.execution_time_ms
        });
        context.update_node("output_metadata", output_metadata);

        info!("Output formatting completed successfully");
        Ok(context)
    }
}

impl Default for TextOutputNode {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use workflow_engine_core::task::TaskContext;
    use serde_json::json;

    fn create_test_context() -> TaskContext {
        let mut context = TaskContext::new();
        
        // Add required data that would come from previous nodes
        context.update_node("processed_text", "HELLO WORLD".to_string());
        context.update_node("original_text", "hello world".to_string());
        
        // Add metadata from previous nodes
        context.update_node("validation_metadata", json!({
            "validation_passed": true,
            "processing_duration_ms": 10
        }));
        
        context.update_node("processing_metadata", json!({
            "node_type": "TextProcessorNode",
            "processing_duration_ms": 25
        }));
        
        context
    }

    #[test]
    fn test_json_output_formatting() {
        let node = TextOutputNode::new();
        let context = create_test_context();

        let result = node.process(context);
        assert!(result.is_ok());

        let result_context = result.unwrap();
        let final_output: WorkflowOutput = result_context.get_data("final_output").unwrap();
        
        assert_eq!(final_output.result, "HELLO WORLD");
        assert_eq!(final_output.original, "hello world");
        assert_eq!(final_output.metadata.nodes_processed, 3);
        assert!(matches!(final_output.metadata.status, WorkflowStatus::Completed));
    }

    #[test]
    fn test_text_output_formatting() {
        let config = TextOutputConfig {
            format: OutputFormat::Text,
            ..Default::default()
        };
        let node = TextOutputNode::with_config(config);
        let context = create_test_context();

        let result = node.process(context);
        assert!(result.is_ok());

        let result_context = result.unwrap();
        let formatted_output: String = result_context.get_data("formatted_output").unwrap();
        
        assert_eq!(formatted_output, "HELLO WORLD");
    }

    #[test]
    fn test_summary_output_formatting() {
        let config = TextOutputConfig {
            format: OutputFormat::Summary,
            ..Default::default()
        };
        let node = TextOutputNode::with_config(config);
        let context = create_test_context();

        let result = node.process(context);
        assert!(result.is_ok());

        let result_context = result.unwrap();
        let formatted_output: String = result_context.get_data("formatted_output").unwrap();
        
        assert!(formatted_output.contains("Result: HELLO WORLD"));
        assert!(formatted_output.contains("Original: hello world"));
        assert!(formatted_output.contains("Nodes Processed: 3"));
        assert!(formatted_output.contains("Processing Steps:"));
    }

    #[test]
    fn test_processing_steps_collection() {
        let node = TextOutputNode::new();
        let context = create_test_context();

        let steps = node.collect_processing_steps(&context);
        
        assert_eq!(steps.len(), 3);
        assert_eq!(steps[0].node_type, "TextInputNode");
        assert_eq!(steps[1].node_type, "TextProcessorNode");
        assert_eq!(steps[2].node_type, "TextOutputNode");
    }

    #[test]
    fn test_missing_data_validation() {
        let node = TextOutputNode::new();
        let context = TaskContext::new(); // Empty context

        let result = node.process(context);
        assert!(result.is_err());
        
        if let Err(WorkflowError::ValidationError(details)) = result {
            assert!(details.message.contains("Missing processed_text"));
        } else {
            panic!("Expected ValidationError");
        }
    }

    #[test]
    fn test_minimal_metadata_config() {
        let config = TextOutputConfig {
            include_metadata: false,
            include_steps: false,
            include_original: false,
            ..Default::default()
        };
        let node = TextOutputNode::with_config(config);
        let context = create_test_context();

        let result = node.process(context);
        assert!(result.is_ok());

        let result_context = result.unwrap();
        let final_output: WorkflowOutput = result_context.get_data("final_output").unwrap();
        
        assert_eq!(final_output.result, "HELLO WORLD");
        assert_eq!(final_output.original, ""); // Not included
        assert!(final_output.metadata.steps.is_empty()); // Not included
    }
}