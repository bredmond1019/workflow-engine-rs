//! # Multi-Model AI Integration Example
//!
//! This example demonstrates how to use multiple AI models together for comparison,
//! consensus building, and fallback strategies. You'll learn to orchestrate different
//! AI providers to get the best results for various tasks.
//!
//! ## What You'll Learn
//! - Using multiple AI models in parallel
//! - Comparing responses from different providers
//! - Implementing model fallback strategies
//! - Building consensus from multiple AI opinions
//! - Choosing the best model for specific tasks
//!
//! ## Prerequisites
//! Set both API keys:
//! ```bash
//! export OPENAI_API_KEY="sk-your-openai-api-key"
//! export ANTHROPIC_API_KEY="your-anthropic-api-key"
//! ```
//!
//! ## Usage
//! ```bash
//! cargo run --bin multi-model
//! ```

use workflow_engine_core::{
    prelude::*,
    nodes::agent::{AgentConfig, ModelProvider},
};
use workflow_engine_nodes::ai_agents::{OpenAIAgentNode, AnthropicAgentNode};
use serde_json::json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Request for multi-model analysis
#[derive(Debug, Deserialize, Serialize)]
struct MultiModelRequest {
    task_type: String, // "creative", "analytical", "technical", "decision_making"
    prompt: String,
    context: Option<String>,
    models_to_use: Vec<String>, // ["gpt-4", "claude-3-opus", "gpt-3.5-turbo"]
    comparison_criteria: Vec<String>, // ["accuracy", "creativity", "clarity", "depth"]
}

/// Task router that distributes work to appropriate models
#[derive(Debug)]
struct ModelRouterNode;

impl Node for ModelRouterNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("🎯 ModelRouterNode: Routing task to appropriate models...");
        
        let request: MultiModelRequest = context.get_event_data()?;
        
        // Analyze task requirements and recommend optimal models
        let task_analysis = analyze_task_requirements(&request);
        let model_recommendations = recommend_models(&request.task_type, &request.models_to_use);
        
        // Prepare prompts for different model types
        let enhanced_prompt = enhance_prompt_for_multi_model(&request);
        
        context.update_node("routing_analysis", json!({
            "task_analysis": task_analysis,
            "model_recommendations": model_recommendations,
            "enhanced_prompt": enhanced_prompt,
            "original_request": request,
            "routing_timestamp": chrono::Utc::now().to_rfc3339()
        }));
        
        // Set the enhanced prompt for AI models
        context.set_data("prompt", enhanced_prompt)?;
        
        context.set_metadata("task_type", &request.task_type)?;
        context.set_metadata("models_requested", request.models_to_use.len())?;
        
        println!("   📊 Task type: {}", request.task_type);
        println!("   🤖 Models to use: {}", request.models_to_use.join(", "));
        println!("   ✅ Routing analysis completed");
        
        Ok(context)
    }
}

/// Runs multiple AI models in parallel and collects responses
#[derive(Debug)]
struct ParallelModelExecutorNode {
    openai_agent: OpenAIAgentNode,
    anthropic_agent: AnthropicAgentNode,
}

impl ParallelModelExecutorNode {
    fn new() -> Result<Self, WorkflowError> {
        let openai_config = AgentConfig {
            system_prompt: "You are an expert AI assistant. Provide thoughtful, accurate, and well-reasoned responses. Structure your answer clearly and support your points with evidence when appropriate.".to_string(),
            model_provider: ModelProvider::OpenAI,
            model_name: "gpt-4".to_string(),
            mcp_server_uri: None,
        };
        
        let anthropic_config = AgentConfig {
            system_prompt: "You are Claude, an expert AI assistant. Provide comprehensive, nuanced analysis with careful reasoning. Consider multiple perspectives and provide balanced, thoughtful responses with clear explanations.".to_string(),
            model_provider: ModelProvider::Anthropic,
            model_name: "claude-3-opus-20240229".to_string(),
            mcp_server_uri: None,
        };
        
        Ok(Self {
            openai_agent: OpenAIAgentNode::new(openai_config)?,
            anthropic_agent: AnthropicAgentNode::new(anthropic_config),
        })
    }
}

impl Node for ParallelModelExecutorNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("🔄 ParallelModelExecutorNode: Running multiple AI models...");
        
        let routing_data = context
            .get_node_data::<serde_json::Value>("routing_analysis")?
            .ok_or_else(|| WorkflowError::validation_error("Missing routing analysis"))?;
        
        let original_request: MultiModelRequest = serde_json::from_value(
            routing_data["original_request"].clone()
        )?;
        
        let mut model_responses = HashMap::new();
        let mut execution_metadata = HashMap::new();
        
        // Execute models based on the request
        for model_name in &original_request.models_to_use {
            let start_time = std::time::Instant::now();
            
            match model_name.as_str() {
                "gpt-4" | "gpt-3.5-turbo" => {
                    if std::env::var("OPENAI_API_KEY").is_ok() {
                        println!("   🤖 Executing {}...", model_name);
                        
                        // Create a clone of context for this model
                        let model_context = context.clone();
                        
                        match self.openai_agent.process(model_context) {
                            Ok(result_context) => {
                                if let Ok(Some(response)) = result_context.get_node_data::<serde_json::Value>("ai_response") {
                                    model_responses.insert(model_name.clone(), response);
                                    execution_metadata.insert(model_name.clone(), json!({
                                        "execution_time_ms": start_time.elapsed().as_millis(),
                                        "status": "success",
                                        "provider": "OpenAI"
                                    }));
                                    println!("      ✅ {} completed in {}ms", model_name, start_time.elapsed().as_millis());
                                }
                            }
                            Err(e) => {
                                println!("      ❌ {} failed: {}", model_name, e);
                                execution_metadata.insert(model_name.clone(), json!({
                                    "execution_time_ms": start_time.elapsed().as_millis(),
                                    "status": "error",
                                    "error": e.to_string(),
                                    "provider": "OpenAI"
                                }));
                            }
                        }
                    } else {
                        println!("      ⚠️  Skipping {} - OPENAI_API_KEY not set", model_name);
                    }
                }
                model_name if model_name.starts_with("claude") => {
                    if std::env::var("ANTHROPIC_API_KEY").is_ok() {
                        println!("   🤖 Executing {}...", model_name);
                        
                        // Create a clone of context for this model
                        let model_context = context.clone();
                        
                        match self.anthropic_agent.process(model_context) {
                            Ok(result_context) => {
                                if let Ok(Some(response)) = result_context.get_node_data::<serde_json::Value>("ai_response") {
                                    model_responses.insert(model_name.clone(), response);
                                    execution_metadata.insert(model_name.clone(), json!({
                                        "execution_time_ms": start_time.elapsed().as_millis(),
                                        "status": "success",
                                        "provider": "Anthropic"
                                    }));
                                    println!("      ✅ {} completed in {}ms", model_name, start_time.elapsed().as_millis());
                                }
                            }
                            Err(e) => {
                                println!("      ❌ {} failed: {}", model_name, e);
                                execution_metadata.insert(model_name.clone(), json!({
                                    "execution_time_ms": start_time.elapsed().as_millis(),
                                    "status": "error",
                                    "error": e.to_string(),
                                    "provider": "Anthropic"
                                }));
                            }
                        }
                    } else {
                        println!("      ⚠️  Skipping {} - ANTHROPIC_API_KEY not set", model_name);
                    }
                }
                _ => {
                    println!("      ❌ Unknown model: {}", model_name);
                }
            }
        }
        
        // Store all model responses
        context.update_node("model_responses", json!({
            "responses": model_responses,
            "execution_metadata": execution_metadata,
            "successful_models": model_responses.len(),
            "requested_models": original_request.models_to_use.len(),
            "execution_timestamp": chrono::Utc::now().to_rfc3339()
        }));
        
        context.set_metadata("successful_executions", model_responses.len())?;
        context.set_metadata("total_requested", original_request.models_to_use.len())?;
        
        println!("   📊 Executed {}/{} models successfully", 
            model_responses.len(), original_request.models_to_use.len());
        
        Ok(context)
    }
}

/// Compares and analyzes responses from multiple models
#[derive(Debug)]
struct ResponseComparatorNode;

impl Node for ResponseComparatorNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("📊 ResponseComparatorNode: Comparing model responses...");
        
        let routing_data = context
            .get_node_data::<serde_json::Value>("routing_analysis")?
            .ok_or_else(|| WorkflowError::validation_error("Missing routing analysis"))?;
        
        let model_responses_data = context
            .get_node_data::<serde_json::Value>("model_responses")?
            .ok_or_else(|| WorkflowError::validation_error("Missing model responses"))?;
        
        let original_request: MultiModelRequest = serde_json::from_value(
            routing_data["original_request"].clone()
        )?;
        
        let responses = &model_responses_data["responses"];
        let execution_metadata = &model_responses_data["execution_metadata"];
        
        // Compare responses across different criteria
        let mut comparison_results = HashMap::new();
        
        for criterion in &original_request.comparison_criteria {
            let criterion_scores = evaluate_responses_by_criterion(responses, criterion);
            comparison_results.insert(criterion.clone(), criterion_scores);
        }
        
        // Find consensus and disagreements
        let consensus_analysis = analyze_consensus(responses);
        let performance_analysis = analyze_performance(execution_metadata);
        
        // Determine best response for different use cases
        let recommendations = generate_model_recommendations(
            responses, 
            &comparison_results, 
            &original_request.task_type
        );
        
        let comparison_result = json!({
            "comparison_results": comparison_results,
            "consensus_analysis": consensus_analysis,
            "performance_analysis": performance_analysis,
            "recommendations": recommendations,
            "criteria_evaluated": original_request.comparison_criteria,
            "task_type": original_request.task_type,
            "comparison_timestamp": chrono::Utc::now().to_rfc3339()
        });
        
        context.update_node("comparison_analysis", comparison_result);
        context.set_metadata("comparison_complete", true)?;
        
        println!("   📈 Evaluated {} criteria across {} models", 
            original_request.comparison_criteria.len(),
            responses.as_object().map(|o| o.len()).unwrap_or(0)
        );
        println!("   ✅ Response comparison completed");
        
        Ok(context)
    }
}

// Helper functions for analysis

fn analyze_task_requirements(request: &MultiModelRequest) -> serde_json::Value {
    let complexity = estimate_task_complexity(&request.prompt);
    let creativity_required = requires_creativity(&request.task_type, &request.prompt);
    let analytical_depth = requires_analysis(&request.task_type, &request.prompt);
    
    json!({
        "complexity": complexity,
        "creativity_required": creativity_required,
        "analytical_depth": analytical_depth,
        "prompt_length": request.prompt.len(),
        "has_context": request.context.is_some()
    })
}

fn recommend_models(task_type: &str, available_models: &[String]) -> Vec<serde_json::Value> {
    let mut recommendations = Vec::new();
    
    for model in available_models {
        let recommendation = match (task_type, model.as_str()) {
            ("creative", "gpt-4") => json!({
                "model": model,
                "suitability": "high",
                "reason": "Excellent creative capabilities and nuanced understanding"
            }),
            ("creative", model_name) if model_name.starts_with("claude") => json!({
                "model": model,
                "suitability": "very_high",
                "reason": "Outstanding creative writing and literary analysis"
            }),
            ("analytical", "gpt-4") => json!({
                "model": model,
                "suitability": "very_high",
                "reason": "Strong analytical reasoning and problem-solving"
            }),
            ("analytical", model_name) if model_name.starts_with("claude") => json!({
                "model": model,
                "suitability": "very_high",
                "reason": "Excellent analytical depth and comprehensive reasoning"
            }),
            ("technical", "gpt-4") => json!({
                "model": model,
                "suitability": "very_high",
                "reason": "Strong technical knowledge and coding capabilities"
            }),
            ("technical", "gpt-3.5-turbo") => json!({
                "model": model,
                "suitability": "high",
                "reason": "Good technical capabilities, faster and more cost-effective"
            }),
            ("decision_making", model_name) if model_name.starts_with("claude") => json!({
                "model": model,
                "suitability": "very_high",
                "reason": "Excellent at weighing multiple factors and providing balanced analysis"
            }),
            _ => json!({
                "model": model,
                "suitability": "medium",
                "reason": "General-purpose capability for this task type"
            })
        };
        recommendations.push(recommendation);
    }
    
    recommendations
}

fn enhance_prompt_for_multi_model(request: &MultiModelRequest) -> String {
    let mut enhanced = String::new();
    
    // Add task context
    enhanced.push_str(&format!("Task Type: {}\n\n", request.task_type));
    
    if let Some(context) = &request.context {
        enhanced.push_str(&format!("Context: {}\n\n", context));
    }
    
    // Add the main prompt
    enhanced.push_str(&request.prompt);
    
    // Add task-specific instructions
    match request.task_type.as_str() {
        "creative" => {
            enhanced.push_str("\n\nPlease provide a creative, original response that demonstrates imagination and artistic flair.");
        }
        "analytical" => {
            enhanced.push_str("\n\nPlease provide a thorough analytical response with clear reasoning, evidence, and logical conclusions.");
        }
        "technical" => {
            enhanced.push_str("\n\nPlease provide a technically accurate response with specific details, examples, and practical considerations.");
        }
        "decision_making" => {
            enhanced.push_str("\n\nPlease provide a balanced analysis of options, considering pros and cons, and offer clear recommendations.");
        }
        _ => {}
    }
    
    enhanced
}

fn estimate_task_complexity(prompt: &str) -> String {
    let word_count = prompt.split_whitespace().count();
    let has_multiple_questions = prompt.matches('?').count() > 1;
    let has_complex_language = prompt.to_lowercase().contains("analyze") || 
                               prompt.to_lowercase().contains("compare") ||
                               prompt.to_lowercase().contains("evaluate");
    
    if word_count > 100 || has_multiple_questions || has_complex_language {
        "high"
    } else if word_count > 50 {
        "medium"
    } else {
        "low"
    }.to_string()
}

fn requires_creativity(task_type: &str, prompt: &str) -> bool {
    task_type == "creative" || 
    prompt.to_lowercase().contains("creative") ||
    prompt.to_lowercase().contains("story") ||
    prompt.to_lowercase().contains("imagine")
}

fn requires_analysis(task_type: &str, prompt: &str) -> bool {
    task_type == "analytical" || 
    prompt.to_lowercase().contains("analyze") ||
    prompt.to_lowercase().contains("compare") ||
    prompt.to_lowercase().contains("evaluate")
}

fn evaluate_responses_by_criterion(responses: &serde_json::Value, criterion: &str) -> HashMap<String, f64> {
    let mut scores = HashMap::new();
    
    if let Some(responses_obj) = responses.as_object() {
        for (model, response_data) in responses_obj {
            if let Some(response_text) = response_data["response"].as_str() {
                let score = match criterion {
                    "accuracy" => evaluate_accuracy(response_text),
                    "creativity" => evaluate_creativity(response_text),
                    "clarity" => evaluate_clarity(response_text),
                    "depth" => evaluate_depth(response_text),
                    _ => 0.5,
                };
                scores.insert(model.clone(), score);
            }
        }
    }
    
    scores
}

fn evaluate_accuracy(response: &str) -> f64 {
    // Simple heuristics for accuracy assessment
    let has_qualifiers = response.to_lowercase().contains("likely") || 
                        response.to_lowercase().contains("generally") ||
                        response.to_lowercase().contains("typically");
    let has_specific_details = response.matches(char::is_numeric).count() > 3;
    let hedging_language = response.to_lowercase().contains("might") ||
                          response.to_lowercase().contains("could");
    
    let mut score = 0.5;
    if has_specific_details { score += 0.2; }
    if has_qualifiers { score += 0.1; }
    if hedging_language { score -= 0.1; }
    
    score.max(0.0).min(1.0)
}

fn evaluate_creativity(response: &str) -> f64 {
    let word_count = response.split_whitespace().count();
    let unique_words = response.split_whitespace()
        .map(|w| w.to_lowercase())
        .collect::<std::collections::HashSet<_>>()
        .len();
    
    let vocabulary_richness = if word_count > 0 {
        unique_words as f64 / word_count as f64
    } else { 0.0 };
    
    let creative_indicators = response.to_lowercase().contains("imagine") ||
                             response.to_lowercase().contains("creative") ||
                             response.to_lowercase().contains("novel");
    
    let mut score = vocabulary_richness * 0.7;
    if creative_indicators { score += 0.3; }
    
    score.max(0.0).min(1.0)
}

fn evaluate_clarity(response: &str) -> f64 {
    let sentences = response.split('.').count().saturating_sub(1);
    let words = response.split_whitespace().count();
    
    if sentences == 0 { return 0.0; }
    
    let avg_sentence_length = words as f64 / sentences as f64;
    let clarity_score = if avg_sentence_length <= 20.0 { 0.8 }
                       else if avg_sentence_length <= 30.0 { 0.6 }
                       else { 0.4 };
    
    let has_structure = response.contains('\n') || response.contains(':');
    if has_structure { clarity_score + 0.2 } else { clarity_score }.min(1.0)
}

fn evaluate_depth(response: &str) -> f64 {
    let word_count = response.split_whitespace().count();
    let depth_indicators = ["because", "therefore", "however", "furthermore", "analysis"];
    let indicator_count = depth_indicators.iter()
        .map(|&indicator| response.to_lowercase().matches(indicator).count())
        .sum::<usize>();
    
    let length_score = (word_count as f64 / 200.0).min(0.7);
    let depth_score = (indicator_count as f64 / 10.0).min(0.3);
    
    length_score + depth_score
}

fn analyze_consensus(responses: &serde_json::Value) -> serde_json::Value {
    if let Some(responses_obj) = responses.as_object() {
        let response_texts: Vec<&str> = responses_obj.values()
            .filter_map(|r| r["response"].as_str())
            .collect();
        
        if response_texts.len() < 2 {
            return json!({
                "consensus_level": "insufficient_data",
                "agreement_score": 0.0,
                "common_themes": []
            });
        }
        
        let agreement_score = calculate_agreement_score(&response_texts);
        let common_themes = extract_common_themes(&response_texts);
        
        json!({
            "consensus_level": classify_consensus_level(agreement_score),
            "agreement_score": agreement_score,
            "common_themes": common_themes,
            "responses_compared": response_texts.len()
        })
    } else {
        json!({
            "consensus_level": "no_data",
            "agreement_score": 0.0,
            "common_themes": []
        })
    }
}

fn calculate_agreement_score(responses: &[&str]) -> f64 {
    // Simple keyword overlap analysis
    let all_words: Vec<Vec<String>> = responses.iter()
        .map(|r| r.to_lowercase().split_whitespace()
            .filter(|w| w.len() > 3)
            .map(|w| w.to_string())
            .collect())
        .collect();
    
    if all_words.len() < 2 { return 0.0; }
    
    let mut total_overlap = 0.0;
    let mut comparisons = 0;
    
    for i in 0..all_words.len() {
        for j in (i+1)..all_words.len() {
            let set1: std::collections::HashSet<_> = all_words[i].iter().collect();
            let set2: std::collections::HashSet<_> = all_words[j].iter().collect();
            
            let intersection = set1.intersection(&set2).count();
            let union = set1.union(&set2).count();
            
            if union > 0 {
                total_overlap += intersection as f64 / union as f64;
            }
            comparisons += 1;
        }
    }
    
    if comparisons > 0 { total_overlap / comparisons as f64 } else { 0.0 }
}

fn extract_common_themes(responses: &[&str]) -> Vec<String> {
    let mut word_counts = HashMap::new();
    
    for response in responses {
        let words: std::collections::HashSet<String> = response.to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() > 4)
            .map(|w| w.to_string())
            .collect();
        
        for word in words {
            *word_counts.entry(word).or_insert(0) += 1;
        }
    }
    
    let threshold = (responses.len() as f64 * 0.7).ceil() as usize;
    word_counts.into_iter()
        .filter(|(_, count)| *count >= threshold)
        .map(|(word, _)| word)
        .take(10)
        .collect()
}

fn classify_consensus_level(score: f64) -> String {
    if score > 0.7 { "high" }
    else if score > 0.4 { "medium" }
    else { "low" }.to_string()
}

fn analyze_performance(execution_metadata: &serde_json::Value) -> serde_json::Value {
    if let Some(metadata_obj) = execution_metadata.as_object() {
        let mut total_time = 0u64;
        let mut successful_count = 0;
        let mut fastest_model = String::new();
        let mut fastest_time = u64::MAX;
        
        for (model, data) in metadata_obj {
            if data["status"] == "success" {
                successful_count += 1;
                if let Some(time) = data["execution_time_ms"].as_u64() {
                    total_time += time;
                    if time < fastest_time {
                        fastest_time = time;
                        fastest_model = model.clone();
                    }
                }
            }
        }
        
        let avg_time = if successful_count > 0 { total_time / successful_count } else { 0 };
        
        json!({
            "average_execution_time_ms": avg_time,
            "fastest_model": fastest_model,
            "fastest_time_ms": if fastest_time == u64::MAX { 0 } else { fastest_time },
            "successful_executions": successful_count,
            "total_models": metadata_obj.len()
        })
    } else {
        json!({
            "average_execution_time_ms": 0,
            "fastest_model": "",
            "fastest_time_ms": 0,
            "successful_executions": 0,
            "total_models": 0
        })
    }
}

fn generate_model_recommendations(
    responses: &serde_json::Value,
    comparison_results: &HashMap<String, HashMap<String, f64>>,
    task_type: &str
) -> Vec<serde_json::Value> {
    let mut recommendations = Vec::new();
    
    if let Some(responses_obj) = responses.as_object() {
        for model in responses_obj.keys() {
            let mut total_score = 0.0;
            let mut criteria_count = 0;
            
            for (criterion, scores) in comparison_results {
                if let Some(score) = scores.get(model) {
                    total_score += score;
                    criteria_count += 1;
                }
            }
            
            let avg_score = if criteria_count > 0 { total_score / criteria_count as f64 } else { 0.0 };
            
            let use_case = determine_best_use_case(model, task_type, comparison_results);
            
            recommendations.push(json!({
                "model": model,
                "overall_score": avg_score,
                "best_use_case": use_case,
                "ranking": classify_performance(avg_score)
            }));
        }
    }
    
    // Sort by overall score
    recommendations.sort_by(|a, b| {
        let score_a = a["overall_score"].as_f64().unwrap_or(0.0);
        let score_b = b["overall_score"].as_f64().unwrap_or(0.0);
        score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
    });
    
    recommendations
}

fn determine_best_use_case(
    model: &str,
    task_type: &str,
    comparison_results: &HashMap<String, HashMap<String, f64>>
) -> String {
    let mut best_criterion = String::new();
    let mut best_score = 0.0;
    
    for (criterion, scores) in comparison_results {
        if let Some(score) = scores.get(model) {
            if *score > best_score {
                best_score = *score;
                best_criterion = criterion.clone();
            }
        }
    }
    
    if best_criterion.is_empty() {
        format!("General {} tasks", task_type)
    } else {
        format!("Tasks requiring {}", best_criterion)
    }
}

fn classify_performance(score: f64) -> String {
    if score > 0.8 { "Excellent" }
    else if score > 0.6 { "Good" }
    else if score > 0.4 { "Fair" }
    else { "Poor" }.to_string()
}

#[tokio::main]
async fn main() -> Result<(), WorkflowError> {
    println!("🚀 Multi-Model AI Integration Example");
    println!("=".repeat(50));
    println!("This example demonstrates orchestrating multiple AI models for comparison and consensus.\n");
    
    // Check for API keys
    let has_openai = std::env::var("OPENAI_API_KEY").is_ok();
    let has_anthropic = std::env::var("ANTHROPIC_API_KEY").is_ok();
    
    if !has_openai && !has_anthropic {
        println!("❌ Error: No API keys found");
        println!("   Please set at least one API key:");
        println!("   export OPENAI_API_KEY=\"sk-your-openai-api-key\"");
        println!("   export ANTHROPIC_API_KEY=\"your-anthropic-api-key\"");
        return Ok(());
    }
    
    println!("🔑 Available AI providers:");
    if has_openai { println!("   ✅ OpenAI"); }
    if has_anthropic { println!("   ✅ Anthropic"); }
    println!();
    
    // Create workflow nodes
    println!("📦 Creating multi-model workflow...");
    let router = ModelRouterNode;
    let executor = ParallelModelExecutorNode::new()?;
    let comparator = ResponseComparatorNode;
    
    println!("   ✅ Created routing, execution, and comparison nodes\n");
    
    // Test cases for different types of tasks
    let test_cases = vec![
        MultiModelRequest {
            task_type: "creative".to_string(),
            prompt: "Write a short story about a time traveler who discovers that every time they change the past, they create a new version of themselves in the present. Focus on the emotional impact of meeting these alternate selves.".to_string(),
            context: Some("This is for a science fiction anthology focused on character-driven stories rather than hard science.".to_string()),
            models_to_use: vec!["gpt-4".to_string(), "claude-3-opus-20240229".to_string()],
            comparison_criteria: vec!["creativity".to_string(), "clarity".to_string(), "depth".to_string()],
        },
        MultiModelRequest {
            task_type: "analytical".to_string(),
            prompt: "Analyze the potential economic impacts of widespread adoption of autonomous vehicles. Consider effects on employment, urban planning, insurance, and supply chains. Provide specific examples and quantifiable predictions where possible.".to_string(),
            context: Some("This analysis is for a policy brief aimed at government decision-makers.".to_string()),
            models_to_use: vec!["gpt-4".to_string(), "claude-3-opus-20240229".to_string(), "gpt-3.5-turbo".to_string()],
            comparison_criteria: vec!["accuracy".to_string(), "depth".to_string(), "clarity".to_string()],
        },
        MultiModelRequest {
            task_type: "technical".to_string(),
            prompt: "Explain the differences between microservices and monolithic architecture. Include pros and cons, suitable use cases, and migration strategies. Provide code examples where helpful.".to_string(),
            context: None,
            models_to_use: vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
            comparison_criteria: vec!["accuracy".to_string(), "clarity".to_string()],
        },
        MultiModelRequest {
            task_type: "decision_making".to_string(),
            prompt: "A startup has $500K in funding and needs to decide between three growth strategies: 1) Heavy marketing spend to acquire customers quickly, 2) Product development to add new features, or 3) Geographic expansion to new markets. Analyze each option and recommend the best approach.".to_string(),
            context: Some("The startup is in the B2B SaaS space with 100 current customers and $50K monthly recurring revenue.".to_string()),
            models_to_use: vec!["gpt-4".to_string(), "claude-3-opus-20240229".to_string()],
            comparison_criteria: vec!["accuracy".to_string(), "depth".to_string()],
        },
    ];
    
    // Process each test case
    for (i, test_case) in test_cases.into_iter().enumerate() {
        println!("🔄 Test Case {} - {} Task", i + 1, test_case.task_type.to_uppercase());
        println!("   📝 Prompt: {}...", 
            test_case.prompt.chars().take(100).collect::<String>());
        println!("   🤖 Models: {}", test_case.models_to_use.join(", "));
        println!("   📊 Criteria: {}", test_case.comparison_criteria.join(", "));
        
        // Execute the workflow
        let mut context = TaskContext::new(
            "multi_model_analysis".to_string(),
            serde_json::to_value(&test_case)?
        );
        
        context = router.process(context)?;
        context = executor.process(context)?;
        context = comparator.process(context)?;
        
        // Display results
        if let Some(comparison) = context.get_node_data::<serde_json::Value>("comparison_analysis")? {
            println!("   📊 COMPARISON RESULTS:");
            display_comparison_results(&comparison, "      ");
        }
        
        println!("   ✅ Test case {} completed\n", i + 1);
    }
    
    println!("🎉 Multi-Model AI Integration Example Complete!");
    println!("=".repeat(50));
    println!("🎓 What you learned:");
    println!("   • Orchestrating multiple AI models in parallel");
    println!("   • Comparing responses across different criteria");
    println!("   • Building consensus from multiple AI opinions");
    println!("   • Implementing model fallback strategies");
    println!("   • Choosing optimal models for specific tasks");
    println!("   • Performance analysis and recommendation systems");
    println!();
    println!("💡 Key insights:");
    println!("   • Different models excel at different types of tasks");
    println!("   • Comparison helps identify model strengths and weaknesses");
    println!("   • Multiple perspectives can improve overall quality");
    println!("   • Performance varies significantly across providers");
    println!();
    println!("➡️  Next steps:");
    println!("   • Implement weighted scoring based on task requirements");
    println!("   • Add more sophisticated consensus algorithms");
    println!("   • Try streaming responses for real-time comparison");
    println!("   • Explore the prompt-engineering.rs example");
    
    Ok(())
}

fn display_comparison_results(comparison: &serde_json::Value, indent: &str) {
    // Display consensus analysis
    if let Some(consensus) = comparison.get("consensus_analysis") {
        if let (Some(level), Some(score)) = (
            consensus.get("consensus_level").and_then(|v| v.as_str()),
            consensus.get("agreement_score").and_then(|v| v.as_f64())
        ) {
            println!("{}Consensus: {} (score: {:.2})", indent, level, score);
        }
        
        if let Some(themes) = consensus.get("common_themes").and_then(|v| v.as_array()) {
            if !themes.is_empty() {
                println!("{}Common themes: {}", indent, 
                    themes.iter()
                        .filter_map(|t| t.as_str())
                        .take(3)
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }
    }
    
    // Display performance analysis
    if let Some(performance) = comparison.get("performance_analysis") {
        if let (Some(fastest), Some(avg_time)) = (
            performance.get("fastest_model").and_then(|v| v.as_str()),
            performance.get("average_execution_time_ms").and_then(|v| v.as_u64())
        ) {
            println!("{}Performance: {} fastest (avg: {}ms)", indent, fastest, avg_time);
        }
    }
    
    // Display recommendations
    if let Some(recommendations) = comparison.get("recommendations").and_then(|v| v.as_array()) {
        println!("{}Model Rankings:", indent);
        for (i, rec) in recommendations.iter().take(3).enumerate() {
            if let (Some(model), Some(score), Some(ranking)) = (
                rec.get("model").and_then(|v| v.as_str()),
                rec.get("overall_score").and_then(|v| v.as_f64()),
                rec.get("ranking").and_then(|v| v.as_str())
            ) {
                println!("{}  {}. {} - {} (score: {:.2})", 
                    indent, i + 1, model, ranking, score);
            }
        }
    }
    
    // Display criteria scores
    if let Some(criteria) = comparison.get("comparison_results").and_then(|v| v.as_object()) {
        println!("{}Criteria Scores:", indent);
        for (criterion, scores) in criteria {
            println!("{}  {}:", indent, criterion);
            if let Some(scores_obj) = scores.as_object() {
                for (model, score) in scores_obj {
                    if let Some(score_val) = score.as_f64() {
                        println!("{}    {}: {:.2}", indent, model, score_val);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_task_complexity_estimation() {
        assert_eq!(estimate_task_complexity("Simple task"), "low");
        assert_eq!(estimate_task_complexity("This is a more complex task that requires analysis and evaluation of multiple factors"), "high");
    }
    
    #[test]
    fn test_creativity_evaluation() {
        let creative_text = "Imagine a world where dreams become reality through magical storytelling and creative expression";
        let score = evaluate_creativity(creative_text);
        assert!(score > 0.5);
    }
    
    #[test]
    fn test_consensus_calculation() {
        let responses = vec!["This is a good analysis", "This is also good analysis"];
        let score = calculate_agreement_score(&responses);
        assert!(score > 0.0);
    }
}