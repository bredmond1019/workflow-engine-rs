//! Tutorial 1: Text Processing Workflow Example
//! 
//! This example demonstrates the concepts from Tutorial 1 with a working
//! text processing workflow that validates, analyzes, and reports on text input.

use backend::core::task::TaskContext;
use backend::core::nodes::Node;
use backend::core::error::WorkflowError;
use serde_json::json;

/// Node that validates incoming text data
#[derive(Debug)]
struct TextValidationNode;

impl Node for TextValidationNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("🔍 Step 1: Validating text input...");
        
        // Extract the input data
        let input: serde_json::Value = context.get_event_data()?;
        let text = input.get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        // Perform validation checks
        let mut validation_errors = Vec::new();
        
        if text.trim().is_empty() {
            validation_errors.push("Text cannot be empty");
        }
        
        if text.len() > 1000 {
            validation_errors.push("Text too long (max 1000 characters)");
        }
        
        if text.chars().all(|c| c.is_numeric()) {
            validation_errors.push("Text cannot be only numbers");
        }
        
        let is_valid = validation_errors.is_empty();
        
        // Store validation results
        context.update_node("validation", json!({
            "is_valid": is_valid,
            "errors": validation_errors,
            "text_length": text.len(),
            "word_count": text.split_whitespace().count(),
            "processed_at": chrono::Utc::now()
        }));
        
        if is_valid {
            println!("   ✅ Text is valid and ready for analysis");
        } else {
            println!("   ❌ Validation failed: {:?}", validation_errors);
        }
        
        Ok(context)
    }
}

/// Node that analyzes text content
#[derive(Debug)]
struct TextAnalysisNode;

impl Node for TextAnalysisNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("🧠 Step 2: Analyzing text...");
        
        // Check if validation passed
        let validation_data = context.get_node_data::<serde_json::Value>("validation")?
            .ok_or_else(|| WorkflowError::ProcessingError {
                message: "No validation data found".to_string()
            })?;
        
        let is_valid = validation_data.get("is_valid")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        if !is_valid {
            // Skip analysis for invalid text
            context.update_node("analysis", json!({
                "analyzed": false,
                "reason": "Skipped due to validation failure"
            }));
            return Ok(context);
        }
        
        // Get the original text
        let input: serde_json::Value = context.get_event_data()?;
        let text = input.get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        // Simple text analysis
        let sentences = text.split('.').filter(|s| !s.trim().is_empty()).count();
        let words = text.split_whitespace().count();
        let avg_word_length = if words > 0 {
            text.chars().filter(|c| c.is_alphabetic()).count() as f64 / words as f64
        } else {
            0.0
        };
        
        // Simple sentiment analysis
        let positive_words = ["good", "great", "excellent", "love", "amazing", "helpful", "thanks"];
        let negative_words = ["bad", "terrible", "awful", "hate", "disappointed", "frustrated", "problem"];
        
        let text_lower = text.to_lowercase();
        let positive_count = positive_words.iter()
            .filter(|word| text_lower.contains(*word))
            .count();
        
        let negative_count = negative_words.iter()
            .filter(|word| text_lower.contains(*word))
            .count();
        
        let sentiment = if positive_count > negative_count {
            "positive"
        } else if negative_count > positive_count {
            "negative"
        } else {
            "neutral"
        };
        
        // Store analysis results
        context.update_node("analysis", json!({
            "analyzed": true,
            "sentences": sentences,
            "words": words,
            "avg_word_length": avg_word_length,
            "sentiment": sentiment,
            "positive_signals": positive_count,
            "negative_signals": negative_count,
            "analyzed_at": chrono::Utc::now()
        }));
        
        println!("   📊 Analysis: {} words, {} sentences, {} sentiment", 
                 words, sentences, sentiment);
        
        Ok(context)
    }
}

/// Node that generates a summary report
#[derive(Debug)]
struct ReportGeneratorNode;

impl Node for ReportGeneratorNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("📋 Step 3: Generating report...");
        
        // Get analysis results
        let analysis_data = context.get_node_data::<serde_json::Value>("analysis")?
            .ok_or_else(|| WorkflowError::ProcessingError {
                message: "No analysis data found".to_string()
            })?;
        
        let analyzed = analysis_data.get("analyzed")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        if !analyzed {
            context.update_node("report", json!({
                "generated": false,
                "reason": "Analysis was skipped"
            }));
            return Ok(context);
        }
        
        // Get analysis details
        let words = analysis_data.get("words").and_then(|v| v.as_u64()).unwrap_or(0);
        let sentences = analysis_data.get("sentences").and_then(|v| v.as_u64()).unwrap_or(0);
        let sentiment = analysis_data.get("sentiment").and_then(|v| v.as_str()).unwrap_or("neutral");
        let avg_word_length = analysis_data.get("avg_word_length").and_then(|v| v.as_f64()).unwrap_or(0.0);
        
        // Get original text for reference
        let input: serde_json::Value = context.get_event_data()?;
        let original_text = input.get("text").and_then(|v| v.as_str()).unwrap_or("");
        
        // Generate report based on analysis
        let complexity = if avg_word_length > 6.0 {
            "complex"
        } else if avg_word_length > 4.0 {
            "moderate"
        } else {
            "simple"
        };
        
        let report_text = format!(
            "Text Analysis Report:\n\
             - Length: {} words, {} sentences\n\
             - Complexity: {} (avg word length: {:.1})\n\
             - Sentiment: {}\n\
             - Reading level: {}",
            words, 
            sentences, 
            complexity, 
            avg_word_length, 
            sentiment,
            if words > 100 { "intermediate" } else { "basic" }
        );
        
        // Store the generated report
        context.update_node("report", json!({
            "generated": true,
            "report_text": report_text,
            "summary": {
                "word_count": words,
                "sentence_count": sentences,
                "complexity": complexity,
                "sentiment": sentiment,
                "estimated_reading_time_seconds": words * 60 / 250  // ~250 WPM average
            },
            "generated_at": chrono::Utc::now()
        }));
        
        println!("   📊 Report generated: {} words, {} complexity, {} sentiment", 
                 words, complexity, sentiment);
        
        Ok(context)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Text Processing Workflow");
    println!("===========================\n");
    
    // Create our processing nodes
    let validation_node = TextValidationNode;
    let analysis_node = TextAnalysisNode;
    let report_node = ReportGeneratorNode;
    
    // Test data - different types of text
    let test_cases = vec![
        json!({
            "text": "This is a wonderful example of positive text! I love how clear and helpful this explanation is. Great work on making this easy to understand."
        }),
        json!({
            "text": "This system is terrible and frustrating. Nothing works properly and the documentation is awful. I hate dealing with this problem."
        }),
        json!({
            "text": "The implementation demonstrates sophisticated algorithmic approaches to natural language processing. This methodology utilizes advanced computational linguistics techniques."
        }),
        json!({
            "text": "Hello world. This is short."
        }),
    ];
    
    // Process each test case
    for (index, test_data) in test_cases.iter().enumerate() {
        println!("🔄 Processing text sample #{}", index + 1);
        println!("{}", "─".repeat(50));
        
        let original_text = test_data.get("text").and_then(|v| v.as_str()).unwrap_or("");
        println!("📝 Original text: \"{}\"", 
                if original_text.len() > 60 { 
                    format!("{}...", &original_text[..60]) 
                } else { 
                    original_text.to_string() 
                });
        
        // Create task context
        let mut context = TaskContext::new(
            "text_processing".to_string(),
            test_data.clone()
        );
        
        // Execute the workflow pipeline
        context = validation_node.process(context)?;
        context = analysis_node.process(context)?;
        context = report_node.process(context)?;
        
        // Display results
        if let Some(report_data) = context.get_node_data::<serde_json::Value>("report")? {
            if let Some(report_text) = report_data.get("report_text").and_then(|v| v.as_str()) {
                println!("\n📊 Analysis Report:");
                for line in report_text.lines() {
                    println!("   {}", line);
                }
            }
            
            if let Some(summary) = report_data.get("summary") {
                if let Some(reading_time) = summary.get("estimated_reading_time_seconds").and_then(|v| v.as_u64()) {
                    println!("   ⏱️  Estimated reading time: {} seconds", reading_time);
                }
            }
        }
        
        println!("\n");
    }
    
    println!("✨ Workflow demonstration completed!");
    println!("\n🎯 What you learned:");
    println!("   - How to create nodes that implement the Node trait");
    println!("   - How TaskContext carries data between nodes");
    println!("   - How to store and retrieve results from nodes"); 
    println!("   - How to chain nodes together for processing");
    
    Ok(())
}