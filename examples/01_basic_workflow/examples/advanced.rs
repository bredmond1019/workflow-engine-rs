//! Advanced workflow features demonstration
//!
//! This example shows more complex workflow patterns including:
//! - Custom node configurations
//! - Multiple processing modes
//! - Detailed metadata collection
//! - Performance monitoring

use workflow_engine_core::workflow::Workflow;
use workflow_engine_core::error::WorkflowError;
use workflow_engine_core::task::TaskContext;
use serde_json::json;
use std::time::Instant;

use basic_workflow_example::{WorkflowInput, TextProcessingConfig, WorkflowOutput};
use basic_workflow_example::nodes::{
    TextInputNode, TextProcessorNode, TextOutputNode,
    text_input::TextInputConfig,
    text_processor::{TextProcessorNodeConfig, TransformMode},
    text_output::{TextOutputConfig, OutputFormat}
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Advanced Workflow Example");
    println!("=" .repeat(40));

    // Demonstrate multiple workflow configurations
    run_advanced_text_processing().await?;
    run_performance_analysis().await?;
    run_custom_configurations().await?;

    println!("\n✅ Advanced examples completed successfully!");
    Ok(())
}

async fn run_advanced_text_processing() -> Result<(), WorkflowError> {
    println!("\n📝 Advanced Text Processing Demo");
    println!("-" .repeat(30));

    let mut workflow = Workflow::new("advanced_text_processing")?;

    // Configure nodes with custom settings
    let input_config = TextInputConfig {
        min_length: 5,
        max_length: 500,
        sanitize: true,
        trim: true,
        allowed_chars: Some(vec![
            "alphabetic".to_string(),
            "numeric".to_string(),
            "whitespace".to_string(),
            "punctuation".to_string()
        ]),
    };

    let processor_config = TextProcessorNodeConfig {
        default_mode: TransformMode::TitleCase,
        preserve_whitespace: true,
        max_processing_time_ms: Some(1000),
    };

    let output_config = TextOutputConfig {
        include_metadata: true,
        include_steps: true,
        include_original: true,
        format: OutputFormat::Summary,
        pretty_json: true,
    };

    // Register configured nodes
    workflow.register_node("input", Box::new(TextInputNode::with_config(input_config)))?;
    workflow.register_node("processor", Box::new(TextProcessorNode::with_config(processor_config)))?;
    workflow.register_node("output", Box::new(TextOutputNode::with_config(output_config)))?;

    // Connect nodes
    workflow.connect("input", "processor")?;
    workflow.connect("processor", "output")?;
    workflow.validate()?;

    // Test with various inputs
    let test_cases = vec![
        ("hello world from rust", "title_case"),
        ("CONVERTING FROM UPPERCASE", "lowercase"),
        ("testing reverse functionality", "reverse"),
        ("analyze this text for detailed metrics", "analyze"),
    ];

    for (text, mode) in test_cases {
        println!("\n🔄 Processing: \"{}\" (mode: {})", text, mode);
        
        let input = WorkflowInput::new(text)
            .with_config(TextProcessingConfig {
                mode: mode.to_string(),
                preserve_case: false,
                add_prefix: None,
                add_suffix: None,
                max_length: Some(1000),
            });

        let mut context = TaskContext::new();
        context.set_event_data(json!(input))?;

        let start_time = Instant::now();
        let result_context = workflow.execute(context).await?;
        let execution_time = start_time.elapsed();

        let formatted_output: String = result_context.get_data("formatted_output")?;
        println!("📤 Result ({}ms):", execution_time.as_millis());
        println!("{}", formatted_output);
    }

    Ok(())
}

async fn run_performance_analysis() -> Result<(), WorkflowError> {
    println!("\n⚡ Performance Analysis Demo");
    println!("-" .repeat(30));

    let mut workflow = Workflow::new("performance_test")?;

    // Register standard nodes
    workflow.register_node("input", Box::new(TextInputNode::new()))?;
    workflow.register_node("processor", Box::new(TextProcessorNode::new()))?;
    workflow.register_node("output", Box::new(TextOutputNode::new()))?;

    workflow.connect("input", "processor")?;
    workflow.connect("processor", "output")?;
    workflow.validate()?;

    // Test different text sizes
    let test_sizes = vec![10, 100, 1000, 5000];
    
    for size in test_sizes {
        let test_text = "performance test ".repeat(size / 16);
        println!("\n📊 Testing with {} characters", test_text.len());

        let input = WorkflowInput::new(test_text);
        let mut context = TaskContext::new();
        context.set_event_data(json!(input))?;

        let start_time = Instant::now();
        let result_context = workflow.execute(context).await?;
        let execution_time = start_time.elapsed();

        let final_output: WorkflowOutput = result_context.get_data("final_output")?;
        
        println!("   ⏱️  Execution time: {}ms", execution_time.as_millis());
        println!("   📈 Throughput: {:.2} chars/ms", 
                test_text.len() as f64 / execution_time.as_millis() as f64);
        println!("   🔢 Nodes processed: {}", final_output.metadata.nodes_processed);
    }

    Ok(())
}

async fn run_custom_configurations() -> Result<(), WorkflowError> {
    println!("\n⚙️  Custom Configuration Demo");
    println!("-" .repeat(30));

    // Demonstrate different output formats
    let formats = vec![
        ("JSON", OutputFormat::Json),
        ("Text", OutputFormat::Text),
        ("Summary", OutputFormat::Summary),
    ];

    for (name, format) in formats {
        println!("\n📋 Testing {} output format", name);
        
        let mut workflow = Workflow::new(&format!("format_test_{}", name.to_lowercase()))?;

        let output_config = TextOutputConfig {
            format,
            include_metadata: true,
            include_steps: name == "Summary",
            include_original: name != "Text",
            pretty_json: true,
        };

        workflow.register_node("input", Box::new(TextInputNode::new()))?;
        workflow.register_node("processor", Box::new(TextProcessorNode::new()))?;
        workflow.register_node("output", Box::new(TextOutputNode::with_config(output_config)))?;

        workflow.connect("input", "processor")?;
        workflow.connect("processor", "output")?;
        workflow.validate()?;

        let input = WorkflowInput::new("Testing different output formats");
        let mut context = TaskContext::new();
        context.set_event_data(json!(input))?;

        let result_context = workflow.execute(context).await?;
        let formatted_output: String = result_context.get_data("formatted_output")?;

        println!("📤 {} Output:", name);
        if formatted_output.len() > 200 {
            println!("{}...", &formatted_output[..200]);
        } else {
            println!("{}", formatted_output);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_advanced_text_processing() {
        let result = run_advanced_text_processing().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_performance_analysis() {
        let result = run_performance_analysis().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_custom_configurations() {
        let result = run_custom_configurations().await;
        assert!(result.is_ok());
    }
}