//! # Basic Nodes - Understanding Different Node Patterns
//!
//! This example demonstrates various node implementation patterns and common
//! node types you'll encounter when building workflows.
//!
//! ## What You'll Learn
//! - Different node design patterns
//! - Data validation techniques
//! - Text processing nodes
//! - Metadata usage for debugging
//! - Error handling in nodes
//!
//! ## Node Types Covered
//! - Validation nodes
//! - Transformation nodes  
//! - Filter nodes
//! - Aggregation nodes
//!
//! ## Usage
//! ```bash
//! cargo run --bin basic-nodes
//! ```

use workflow_engine_core::prelude::*;
use serde_json::json;
use serde::{Deserialize, Serialize};

/// Input data structure for our workflow
#[derive(Debug, Deserialize, Serialize)]
struct TextProcessingInput {
    text: String,
    author: Option<String>,
    category: Option<String>,
    min_length: Option<usize>,
}

/// A validation node that checks input data meets requirements
/// 
/// Validation nodes are essential for ensuring data quality and
/// preventing errors in downstream processing.
#[derive(Debug)]
struct ValidationNode {
    min_text_length: usize,
    required_fields: Vec<String>,
}

impl ValidationNode {
    fn new(min_text_length: usize, required_fields: Vec<String>) -> Self {
        Self {
            min_text_length,
            required_fields,
        }
    }
}

impl Node for ValidationNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("🔍 ValidationNode: Checking input data...");
        
        // Extract and validate the input data
        let input: TextProcessingInput = context.get_event_data()?;
        
        let mut validation_results = Vec::new();
        let mut is_valid = true;
        
        // Check text length
        if input.text.len() < self.min_text_length {
            let error = format!(
                "Text too short: {} characters (minimum: {})",
                input.text.len(),
                self.min_text_length
            );
            validation_results.push(error);
            is_valid = false;
        } else {
            validation_results.push("Text length: ✅ Valid".to_string());
        }
        
        // Check required fields (simplified example)
        for field in &self.required_fields {
            match field.as_str() {
                "author" => {
                    if input.author.is_some() && !input.author.as_ref().unwrap().is_empty() {
                        validation_results.push("Author field: ✅ Valid".to_string());
                    } else {
                        validation_results.push("Author field: ❌ Missing or empty".to_string());
                        is_valid = false;
                    }
                },
                "category" => {
                    if input.category.is_some() && !input.category.as_ref().unwrap().is_empty() {
                        validation_results.push("Category field: ✅ Valid".to_string());
                    } else {
                        validation_results.push("Category field: ❌ Missing or empty".to_string());
                        is_valid = false;
                    }
                },
                _ => {
                    validation_results.push(format!("Unknown field '{}': ⚠️ Skipped", field));
                }
            }
        }
        
        // Store validation results
        context.update_node("validation", json!({
            "is_valid": is_valid,
            "results": validation_results,
            "text_length": input.text.len(),
            "checked_fields": self.required_fields,
            "validated_at": chrono::Utc::now().to_rfc3339()
        }));
        
        // Add metadata
        context.set_metadata("validation_passed", is_valid)?;
        context.set_metadata("validator_version", "1.0")?;
        
        if is_valid {
            println!("   ✅ Validation passed");
        } else {
            println!("   ❌ Validation failed");
            // In a real workflow, you might want to stop processing here
            // For this example, we'll continue to show the full pipeline
        }
        
        Ok(context)
    }
}

/// A text transformation node that performs various text operations
/// 
/// Transformation nodes modify data according to specified rules.
/// This node demonstrates multiple transformation types.
#[derive(Debug)]
struct TextTransformationNode {
    operations: Vec<String>,
}

impl TextTransformationNode {
    fn new(operations: Vec<String>) -> Self {
        Self { operations }
    }
    
    fn apply_transformation(&self, text: &str, operation: &str) -> String {
        match operation {
            "uppercase" => text.to_uppercase(),
            "lowercase" => text.to_lowercase(),
            "trim" => text.trim().to_string(),
            "remove_spaces" => text.replace(' ', ""),
            "word_count" => format!("Word count: {}", text.split_whitespace().count()),
            "reverse" => text.chars().rev().collect(),
            "capitalize" => {
                let mut chars = text.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars.as_str().to_lowercase().chars()).collect(),
                }
            },
            _ => text.to_string(),
        }
    }
}

impl Node for TextTransformationNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("🔄 TextTransformationNode: Applying transformations...");
        
        let input: TextProcessingInput = context.get_event_data()?;
        let mut transformations = Vec::new();
        let mut current_text = input.text.clone();
        
        // Apply each transformation in sequence
        for operation in &self.operations {
            let original = current_text.clone();
            current_text = self.apply_transformation(&current_text, operation);
            
            transformations.push(json!({
                "operation": operation,
                "before": original,
                "after": current_text.clone(),
                "length_change": current_text.len() as i32 - original.len() as i32
            }));
            
            println!("   🔧 Applied '{}': {} chars -> {} chars", 
                operation, original.len(), current_text.len());
        }
        
        // Store transformation results
        context.update_node("transformation", json!({
            "original_text": input.text,
            "final_text": current_text,
            "operations_applied": self.operations,
            "transformations": transformations,
            "transformed_at": chrono::Utc::now().to_rfc3339()
        }));
        
        context.set_metadata("transformations_count", self.operations.len())?;
        context.set_metadata("final_text_length", current_text.len())?;
        
        println!("   ✅ Applied {} transformations", self.operations.len());
        
        Ok(context)
    }
}

/// A filter node that determines if content should continue processing
/// 
/// Filter nodes implement conditional logic to determine workflow paths.
/// This example filters based on content characteristics.
#[derive(Debug)]
struct ContentFilterNode {
    min_quality_score: f64,
    blocked_words: Vec<String>,
}

impl ContentFilterNode {
    fn new(min_quality_score: f64, blocked_words: Vec<String>) -> Self {
        Self {
            min_quality_score,
            blocked_words,
        }
    }
    
    fn calculate_quality_score(&self, text: &str) -> f64 {
        let mut score = 0.5; // Base score
        
        // Length bonus
        if text.len() > 100 {
            score += 0.2;
        }
        
        // Variety bonus (unique words vs total words)
        let words: Vec<&str> = text.split_whitespace().collect();
        if !words.is_empty() {
            let unique_words: std::collections::HashSet<_> = words.iter().collect();
            let variety_ratio = unique_words.len() as f64 / words.len() as f64;
            score += variety_ratio * 0.3;
        }
        
        // Penalty for blocked words
        for blocked_word in &self.blocked_words {
            if text.to_lowercase().contains(&blocked_word.to_lowercase()) {
                score -= 0.3;
            }
        }
        
        // Ensure score is between 0 and 1
        score.max(0.0).min(1.0)
    }
}

impl Node for ContentFilterNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("🔍 ContentFilterNode: Analyzing content quality...");
        
        // Get the transformed text from the previous node, or original if not available
        let transformed_text = if let Ok(Some(transformation_data)) = 
            context.get_node_data::<serde_json::Value>("transformation") {
            transformation_data["final_text"].as_str().unwrap_or("").to_string()
        } else {
            let input: TextProcessingInput = context.get_event_data()?;
            input.text
        };
        
        // Calculate quality score
        let quality_score = self.calculate_quality_score(&transformed_text);
        let passes_filter = quality_score >= self.min_quality_score;
        
        // Check for blocked words
        let mut blocked_words_found = Vec::new();
        for blocked_word in &self.blocked_words {
            if transformed_text.to_lowercase().contains(&blocked_word.to_lowercase()) {
                blocked_words_found.push(blocked_word.clone());
            }
        }
        
        let has_blocked_content = !blocked_words_found.is_empty();
        let final_decision = passes_filter && !has_blocked_content;
        
        // Store filter results
        context.update_node("filter", json!({
            "quality_score": quality_score,
            "min_required_score": self.min_quality_score,
            "passes_quality_check": passes_filter,
            "blocked_words_found": blocked_words_found,
            "has_blocked_content": has_blocked_content,
            "final_decision": final_decision,
            "filtered_at": chrono::Utc::now().to_rfc3339(),
            "text_analyzed": transformed_text
        }));
        
        context.set_metadata("content_approved", final_decision)?;
        context.set_metadata("quality_score", quality_score)?;
        
        if final_decision {
            println!("   ✅ Content approved (score: {:.2})", quality_score);
        } else {
            println!("   ❌ Content rejected (score: {:.2}, blocked words: {})", 
                quality_score, blocked_words_found.len());
        }
        
        Ok(context)
    }
}

/// An aggregation node that combines results from all previous nodes
/// 
/// Aggregation nodes collect and summarize data from multiple sources
/// to create comprehensive reports or final results.
#[derive(Debug)]
struct ResultAggregatorNode;

impl Node for ResultAggregatorNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("📊 ResultAggregatorNode: Aggregating workflow results...");
        
        // Collect data from all previous nodes
        let validation_data = context.get_node_data::<serde_json::Value>("validation")?;
        let transformation_data = context.get_node_data::<serde_json::Value>("transformation")?;
        let filter_data = context.get_node_data::<serde_json::Value>("filter")?;
        
        // Create a comprehensive summary
        let mut summary = json!({
            "workflow_id": context.event_id,
            "processed_at": chrono::Utc::now().to_rfc3339(),
            "processing_stages": ["validation", "transformation", "filter", "aggregation"]
        });
        
        // Add validation summary
        if let Some(validation) = validation_data {
            summary["validation_summary"] = json!({
                "passed": validation["is_valid"],
                "issues_found": validation["results"].as_array().map(|r| r.len()).unwrap_or(0),
                "text_length": validation["text_length"]
            });
        }
        
        // Add transformation summary
        if let Some(transformation) = transformation_data {
            summary["transformation_summary"] = json!({
                "operations_count": transformation["operations_applied"].as_array().map(|o| o.len()).unwrap_or(0),
                "text_changed": transformation["original_text"] != transformation["final_text"],
                "final_length": transformation["final_text"].as_str().map(|s| s.len()).unwrap_or(0)
            });
        }
        
        // Add filter summary
        if let Some(filter) = filter_data {
            summary["filter_summary"] = json!({
                "approved": filter["final_decision"],
                "quality_score": filter["quality_score"],
                "blocked_content": filter["has_blocked_content"]
            });
        }
        
        // Calculate overall workflow success
        let validation_passed = validation_data
            .as_ref()
            .and_then(|v| v["is_valid"].as_bool())
            .unwrap_or(false);
        let content_approved = filter_data
            .as_ref()
            .and_then(|f| f["final_decision"].as_bool())
            .unwrap_or(false);
        
        let overall_success = validation_passed && content_approved;
        
        summary["overall_result"] = json!({
            "success": overall_success,
            "validation_passed": validation_passed,
            "content_approved": content_approved,
            "ready_for_next_stage": overall_success
        });
        
        // Store the aggregated results
        context.update_node("aggregated_results", summary);
        
        // Update final metadata
        context.set_metadata("processing_complete", true)?;
        context.set_metadata("overall_success", overall_success)?;
        context.set_metadata("aggregation_version", "1.0")?;
        
        println!("   📋 Workflow Summary:");
        println!("      Validation: {}", if validation_passed { "✅ Passed" } else { "❌ Failed" });
        println!("      Content: {}", if content_approved { "✅ Approved" } else { "❌ Rejected" });
        println!("      Overall: {}", if overall_success { "✅ Success" } else { "❌ Failed" });
        
        Ok(context)
    }
}

#[tokio::main]
async fn main() -> Result<(), WorkflowError> {
    println!("🚀 Basic Nodes Example - Understanding Node Patterns");
    println!("=".repeat(60));
    println!("This example demonstrates various node types and patterns.\n");
    
    // Create our processing nodes
    println!("📦 Creating processing nodes...");
    let validation_node = ValidationNode::new(10, vec!["author".to_string()]);
    let transformation_node = TextTransformationNode::new(vec![
        "trim".to_string(),
        "capitalize".to_string(),
    ]);
    let filter_node = ContentFilterNode::new(0.6, vec!["spam".to_string(), "bad".to_string()]);
    let aggregator_node = ResultAggregatorNode;
    println!("   ✅ Created all processing nodes\n");
    
    // Test cases with different characteristics
    let test_cases = vec![
        (
            "Valid Content",
            TextProcessingInput {
                text: "  this is a well-written article about rust programming  ".to_string(),
                author: Some("Alice Developer".to_string()),
                category: Some("programming".to_string()),
                min_length: Some(20),
            }
        ),
        (
            "Short Content",
            TextProcessingInput {
                text: "too short".to_string(),
                author: Some("Bob Writer".to_string()),
                category: Some("general".to_string()),
                min_length: Some(5),
            }
        ),
        (
            "Missing Author",
            TextProcessingInput {
                text: "This is a longer piece of content that should pass length validation".to_string(),
                author: None,
                category: Some("tech".to_string()),
                min_length: Some(30),
            }
        ),
        (
            "Blocked Content",
            TextProcessingInput {
                text: "This content contains spam and bad words that should be filtered".to_string(),
                author: Some("Spammer".to_string()),
                category: Some("unwanted".to_string()),
                min_length: Some(20),
            }
        ),
    ];
    
    // Process each test case through the complete pipeline
    for (test_name, input) in test_cases {
        println!("🔄 Processing Test Case: {}", test_name);
        println!("   📥 Input: {} characters, author: {}", 
            input.text.len(),
            input.author.as_deref().unwrap_or("None")
        );
        
        // Create context with the input data
        let mut context = TaskContext::new(
            "basic_nodes_workflow".to_string(),
            serde_json::to_value(input)?
        );
        
        // Execute the complete processing pipeline
        context = validation_node.process(context)?;
        context = transformation_node.process(context)?;
        context = filter_node.process(context)?;
        context = aggregator_node.process(context)?;
        
        // Display final results
        if let Some(results) = context.get_node_data::<serde_json::Value>("aggregated_results")? {
            let overall_success = results["overall_result"]["success"].as_bool().unwrap_or(false);
            println!("   📊 Final Result: {}", if overall_success { "✅ SUCCESS" } else { "❌ FAILED" });
            
            if let Some(validation_summary) = results.get("validation_summary") {
                println!("      Validation: {}", 
                    if validation_summary["passed"].as_bool().unwrap_or(false) { 
                        "✅ Passed" 
                    } else { 
                        "❌ Failed" 
                    }
                );
            }
            
            if let Some(filter_summary) = results.get("filter_summary") {
                println!("      Quality Score: {:.2}", 
                    filter_summary["quality_score"].as_f64().unwrap_or(0.0)
                );
                println!("      Content Approved: {}", 
                    if filter_summary["approved"].as_bool().unwrap_or(false) { 
                        "✅ Yes" 
                    } else { 
                        "❌ No" 
                    }
                );
            }
        }
        
        println!("   ✅ Test case completed\n");
    }
    
    println!("🎉 Basic Nodes Example Complete!");
    println!("=".repeat(60));
    println!("🎓 What you learned:");
    println!("   • Validation node patterns for data quality");
    println!("   • Transformation nodes for data processing");
    println!("   • Filter nodes for conditional logic");
    println!("   • Aggregation nodes for result compilation");
    println!("   • Using metadata for workflow tracking");
    println!("   • Handling different data types and structures");
    println!();
    println!("➡️  Next steps:");
    println!("   • Try creating your own validation rules");
    println!("   • Add new transformation operations");
    println!("   • Experiment with different filter criteria");
    println!("   • Move on to the data-flow.rs example");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validation_node_success() {
        let node = ValidationNode::new(5, vec!["author".to_string()]);
        let input = TextProcessingInput {
            text: "This is a valid text".to_string(),
            author: Some("Test Author".to_string()),
            category: None,
            min_length: None,
        };
        
        let context = TaskContext::new(
            "test".to_string(),
            serde_json::to_value(input).unwrap()
        );
        
        let result = node.process(context).unwrap();
        let validation: serde_json::Value = result.get_node_data("validation").unwrap().unwrap();
        
        assert_eq!(validation["is_valid"], true);
    }
    
    #[test]
    fn test_validation_node_failure() {
        let node = ValidationNode::new(100, vec!["author".to_string()]);
        let input = TextProcessingInput {
            text: "Short".to_string(),
            author: None,
            category: None,
            min_length: None,
        };
        
        let context = TaskContext::new(
            "test".to_string(),
            serde_json::to_value(input).unwrap()
        );
        
        let result = node.process(context).unwrap();
        let validation: serde_json::Value = result.get_node_data("validation").unwrap().unwrap();
        
        assert_eq!(validation["is_valid"], false);
    }
    
    #[test]
    fn test_text_transformation_node() {
        let node = TextTransformationNode::new(vec!["uppercase".to_string(), "trim".to_string()]);
        let input = TextProcessingInput {
            text: "  hello world  ".to_string(),
            author: None,
            category: None,
            min_length: None,
        };
        
        let context = TaskContext::new(
            "test".to_string(),
            serde_json::to_value(input).unwrap()
        );
        
        let result = node.process(context).unwrap();
        let transformation: serde_json::Value = result.get_node_data("transformation").unwrap().unwrap();
        
        assert_eq!(transformation["final_text"], "HELLO WORLD");
    }
    
    #[test]
    fn test_content_filter_node() {
        let node = ContentFilterNode::new(0.5, vec!["spam".to_string()]);
        let input = TextProcessingInput {
            text: "This is spam content that should be blocked".to_string(),
            author: None,
            category: None,
            min_length: None,
        };
        
        let context = TaskContext::new(
            "test".to_string(),
            serde_json::to_value(input).unwrap()
        );
        
        let result = node.process(context).unwrap();
        let filter: serde_json::Value = result.get_node_data("filter").unwrap().unwrap();
        
        assert_eq!(filter["has_blocked_content"], true);
        assert_eq!(filter["final_decision"], false);
    }
}