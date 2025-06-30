//! # OpenAI Agent Integration Example
//!
//! This example demonstrates how to integrate OpenAI's GPT models into your workflows
//! using the AI agent system. You'll learn how to configure agents, handle responses,
//! and build AI-powered processing pipelines.
//!
//! ## What You'll Learn
//! - Setting up OpenAI agent nodes
//! - Configuring different GPT models
//! - Writing effective prompts
//! - Handling AI responses and errors
//! - Building AI-powered workflows
//!
//! ## Prerequisites
//! Set your OpenAI API key:
//! ```bash
//! export OPENAI_API_KEY="sk-your-openai-api-key"
//! ```
//!
//! ## Usage
//! ```bash
//! cargo run --bin openai-agent
//! ```

use workflow_engine_core::{
    prelude::*,
    nodes::agent::{AgentConfig, ModelProvider},
};
use workflow_engine_nodes::ai_agents::OpenAIAgentNode;
use serde_json::json;
use serde::{Deserialize, Serialize};

/// Input structure for content analysis requests
#[derive(Debug, Deserialize, Serialize)]
struct ContentAnalysisRequest {
    text: String,
    analysis_type: String, // "sentiment", "summary", "keywords", "classification"
    target_audience: Option<String>,
    max_length: Option<usize>,
}

/// A preprocessing node that prepares data for AI analysis
#[derive(Debug)]
struct ContentPreprocessorNode;

impl Node for ContentPreprocessorNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("📝 ContentPreprocessorNode: Preparing content for AI analysis...");
        
        let request: ContentAnalysisRequest = context.get_event_data()?;
        
        // Validate and clean the text
        let cleaned_text = request.text
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<&str>>()
            .join(" ");
        
        if cleaned_text.is_empty() {
            return Err(WorkflowError::validation_error("Content text cannot be empty"));
        }
        
        // Create AI-specific prompts based on analysis type
        let prompt = match request.analysis_type.as_str() {
            "sentiment" => {
                format!(
                    "Analyze the sentiment of the following text. Provide a sentiment score from -1 (very negative) to 1 (very positive) and explain your reasoning.\n\nText: {}",
                    cleaned_text
                )
            }
            "summary" => {
                let max_length = request.max_length.unwrap_or(100);
                format!(
                    "Summarize the following text in no more than {} words. Focus on the key points and main ideas.\n\nText: {}",
                    max_length, cleaned_text
                )
            }
            "keywords" => {
                format!(
                    "Extract the most important keywords and key phrases from the following text. List them in order of importance.\n\nText: {}",
                    cleaned_text
                )
            }
            "classification" => {
                let audience = request.target_audience.as_deref().unwrap_or("general");
                format!(
                    "Classify the following text by topic and determine if it's appropriate for a {} audience. Provide the main topic category and an appropriateness rating.\n\nText: {}",
                    audience, cleaned_text
                )
            }
            _ => {
                format!(
                    "Analyze the following text and provide insights about its content, style, and meaning.\n\nText: {}",
                    cleaned_text
                )
            }
        };
        
        // Store the processed data for the AI agent
        context.update_node("preprocessed_content", json!({
            "prompt": prompt,
            "original_request": request,
            "cleaned_text": cleaned_text,
            "text_length": cleaned_text.len(),
            "word_count": cleaned_text.split_whitespace().count(),
            "analysis_type": request.analysis_type,
            "prepared_at": chrono::Utc::now().to_rfc3339()
        }));
        
        // Set the prompt field that the AI agent will look for
        context.set_data("prompt", prompt)?;
        
        context.set_metadata("preprocessing_complete", true)?;
        context.set_metadata("text_word_count", cleaned_text.split_whitespace().count())?;
        
        println!("   ✅ Content preprocessed: {} words, {} analysis", 
            cleaned_text.split_whitespace().count(), request.analysis_type);
        
        Ok(context)
    }
}

/// A post-processing node that structures AI responses
#[derive(Debug)]
struct ResponseFormatterNode;

impl Node for ResponseFormatterNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("📋 ResponseFormatterNode: Formatting AI response...");
        
        // Get the AI response
        let ai_response = context
            .get_node_data::<serde_json::Value>("ai_response")?
            .ok_or_else(|| WorkflowError::validation_error("Missing AI response"))?;
        
        // Get the original request data
        let preprocessed = context
            .get_node_data::<serde_json::Value>("preprocessed_content")?
            .ok_or_else(|| WorkflowError::validation_error("Missing preprocessed content"))?;
        
        let analysis_type = preprocessed["analysis_type"]
            .as_str()
            .unwrap_or("unknown");
        
        let ai_text_response = ai_response["response"]
            .as_str()
            .unwrap_or("No response available");
        
        // Structure the response based on analysis type
        let structured_response = match analysis_type {
            "sentiment" => {
                // Try to extract sentiment score from response
                let sentiment_score = extract_sentiment_score(ai_text_response);
                json!({
                    "analysis_type": "sentiment",
                    "sentiment_score": sentiment_score,
                    "sentiment_label": classify_sentiment(sentiment_score),
                    "explanation": ai_text_response,
                    "confidence": calculate_confidence(ai_text_response)
                })
            }
            "summary" => {
                json!({
                    "analysis_type": "summary",
                    "summary": ai_text_response,
                    "original_word_count": preprocessed["word_count"],
                    "summary_word_count": ai_text_response.split_whitespace().count(),
                    "compression_ratio": calculate_compression_ratio(
                        preprocessed["word_count"].as_u64().unwrap_or(0),
                        ai_text_response.split_whitespace().count() as u64
                    )
                })
            }
            "keywords" => {
                let keywords = extract_keywords_from_response(ai_text_response);
                json!({
                    "analysis_type": "keywords",
                    "keywords": keywords,
                    "keyword_count": keywords.len(),
                    "full_response": ai_text_response
                })
            }
            "classification" => {
                let (topic, appropriateness) = extract_classification_from_response(ai_text_response);
                json!({
                    "analysis_type": "classification",
                    "topic": topic,
                    "appropriateness": appropriateness,
                    "full_analysis": ai_text_response
                })
            }
            _ => {
                json!({
                    "analysis_type": analysis_type,
                    "analysis": ai_text_response,
                    "processed_at": chrono::Utc::now().to_rfc3339()
                })
            }
        };
        
        // Create comprehensive final result
        let final_result = json!({
            "request_id": context.event_id,
            "analysis": structured_response,
            "model_info": {
                "provider": ai_response.get("provider").unwrap_or(&json!("OpenAI")),
                "model": ai_response.get("model").unwrap_or(&json!("unknown")),
                "timestamp": ai_response.get("timestamp")
            },
            "processing_metadata": {
                "input_word_count": preprocessed["word_count"],
                "analysis_type": analysis_type,
                "processing_time_estimate": "2-5 seconds"
            }
        });
        
        context.update_node("final_result", final_result);
        context.set_metadata("formatting_complete", true)?;
        context.set_metadata("response_type", analysis_type)?;
        
        println!("   ✅ Response formatted for {} analysis", analysis_type);
        
        Ok(context)
    }
}

// Helper functions for response parsing
fn extract_sentiment_score(response: &str) -> f64 {
    // Simple regex-based extraction (in production, use more robust parsing)
    let response_lower = response.to_lowercase();
    
    // Look for explicit score mentions
    if let Some(start) = response_lower.find("score") {
        let substr = &response[start..];
        if let Some(captures) = regex::Regex::new(r"-?\d+\.?\d*")
            .ok()
            .and_then(|re| re.find(substr)) 
        {
            if let Ok(score) = captures.as_str().parse::<f64>() {
                return score.max(-1.0).min(1.0);
            }
        }
    }
    
    // Fallback: simple keyword-based scoring
    if response_lower.contains("very positive") || response_lower.contains("extremely positive") {
        0.9
    } else if response_lower.contains("positive") {
        0.6
    } else if response_lower.contains("very negative") || response_lower.contains("extremely negative") {
        -0.9
    } else if response_lower.contains("negative") {
        -0.6
    } else if response_lower.contains("neutral") {
        0.0
    } else {
        0.0 // Default neutral
    }
}

fn classify_sentiment(score: f64) -> &'static str {
    if score > 0.5 {
        "positive"
    } else if score < -0.5 {
        "negative"
    } else {
        "neutral"
    }
}

fn calculate_confidence(response: &str) -> f64 {
    // Simple confidence estimation based on response characteristics
    let words = response.split_whitespace().count();
    let has_numbers = response.chars().any(|c| c.is_ascii_digit());
    let has_qualifiers = response.to_lowercase().contains("likely") || 
                        response.to_lowercase().contains("probably") ||
                        response.to_lowercase().contains("seems");
    
    let mut confidence = 0.5; // Base confidence
    
    if words > 50 { confidence += 0.2; } // Detailed responses are more confident
    if has_numbers { confidence += 0.2; } // Quantitative analysis
    if has_qualifiers { confidence -= 0.1; } // Hedging language reduces confidence
    
    confidence.max(0.0).min(1.0)
}

fn calculate_compression_ratio(original_words: u64, summary_words: u64) -> f64 {
    if original_words == 0 { return 0.0; }
    summary_words as f64 / original_words as f64
}

fn extract_keywords_from_response(response: &str) -> Vec<String> {
    // Simple keyword extraction (in production, use more sophisticated parsing)
    response
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with('-') || trimmed.starts_with('•') || trimmed.chars().next().map_or(false, |c| c.is_ascii_digit()) {
                Some(trimmed.trim_start_matches('-')
                          .trim_start_matches('•')
                          .trim_start_matches(char::is_numeric)
                          .trim_start_matches('.')
                          .trim()
                          .to_string())
            } else {
                None
            }
        })
        .filter(|s| !s.is_empty() && s.len() > 2)
        .take(10) // Limit to top 10 keywords
        .collect()
}

fn extract_classification_from_response(response: &str) -> (String, String) {
    let response_lower = response.to_lowercase();
    
    // Simple topic extraction
    let topic = if response_lower.contains("technology") || response_lower.contains("tech") {
        "Technology"
    } else if response_lower.contains("business") || response_lower.contains("finance") {
        "Business"
    } else if response_lower.contains("health") || response_lower.contains("medical") {
        "Health"
    } else if response_lower.contains("education") || response_lower.contains("learning") {
        "Education"
    } else {
        "General"
    }.to_string();
    
    // Simple appropriateness extraction
    let appropriateness = if response_lower.contains("appropriate") || response_lower.contains("suitable") {
        "Appropriate"
    } else if response_lower.contains("inappropriate") || response_lower.contains("not suitable") {
        "Inappropriate"
    } else {
        "Requires Review"
    }.to_string();
    
    (topic, appropriateness)
}

#[tokio::main]
async fn main() -> Result<(), WorkflowError> {
    println!("🚀 OpenAI Agent Integration Example");
    println!("=".repeat(55));
    println!("This example demonstrates AI-powered content analysis using OpenAI's GPT models.\n");
    
    // Check for API key
    if std::env::var("OPENAI_API_KEY").is_err() {
        println!("❌ Error: OPENAI_API_KEY environment variable not set");
        println!("   Please set your OpenAI API key:");
        println!("   export OPENAI_API_KEY=\"sk-your-openai-api-key\"");
        return Ok(());
    }
    
    // Create different OpenAI agent configurations for different models
    println!("📦 Creating OpenAI agent configurations...");
    
    let gpt4_config = AgentConfig {
        system_prompt: "You are an expert content analyst. Provide detailed, accurate analysis with specific insights and actionable recommendations. Use clear, professional language and support your conclusions with evidence from the text.".to_string(),
        model_provider: ModelProvider::OpenAI,
        model_name: "gpt-4".to_string(),
        mcp_server_uri: None,
    };
    
    let gpt35_config = AgentConfig {
        system_prompt: "You are a helpful content analysis assistant. Provide clear, concise analysis focused on the key insights and main points.".to_string(),
        model_provider: ModelProvider::OpenAI,
        model_name: "gpt-3.5-turbo".to_string(),
        mcp_server_uri: None,
    };
    
    // Create the workflow nodes
    let preprocessor = ContentPreprocessorNode;
    let gpt4_agent = OpenAIAgentNode::new(gpt4_config)?;
    let gpt35_agent = OpenAIAgentNode::new(gpt35_config)?;
    let formatter = ResponseFormatterNode;
    
    println!("   ✅ Created GPT-4 and GPT-3.5-turbo agents");
    println!("   ✅ Created preprocessing and formatting nodes\n");
    
    // Test cases with different types of content and analysis
    let test_cases = vec![
        ContentAnalysisRequest {
            text: "I absolutely love this new smartphone! The camera quality is incredible, the battery lasts all day, and the user interface is so intuitive. It's definitely worth the investment. The only minor issue is that it can get a bit warm during heavy gaming, but that's barely noticeable. Overall, I'm extremely satisfied with this purchase and would recommend it to anyone looking for a premium device.".to_string(),
            analysis_type: "sentiment".to_string(),
            target_audience: Some("general".to_string()),
            max_length: None,
        },
        ContentAnalysisRequest {
            text: "Artificial intelligence is rapidly transforming industries across the globe. Machine learning algorithms are being deployed in healthcare to diagnose diseases, in finance to detect fraud, and in transportation to enable autonomous vehicles. Natural language processing has made chatbots and virtual assistants more sophisticated than ever before. Computer vision technology is revolutionizing manufacturing quality control and retail customer experiences. Deep learning models are pushing the boundaries of what's possible in scientific research, from drug discovery to climate modeling. However, these advances also raise important questions about job displacement, privacy, and the ethical implications of AI decision-making. As we continue to integrate AI into our daily lives, it's crucial that we develop robust frameworks for responsible AI development and deployment.".to_string(),
            analysis_type: "summary".to_string(),
            target_audience: Some("business professionals".to_string()),
            max_length: Some(75),
        },
        ContentAnalysisRequest {
            text: "The renewable energy sector has experienced unprecedented growth in recent years. Solar panel efficiency has improved dramatically while costs have plummeted. Wind turbine technology has advanced to capture energy from lower wind speeds. Energy storage solutions, particularly lithium-ion batteries, have become more affordable and reliable. Government policies and incentives have accelerated adoption across residential and commercial sectors. Smart grid infrastructure is enabling better integration of renewable sources. Electric vehicle adoption is driving demand for clean energy. Sustainable finance and green bonds are providing capital for renewable projects.".to_string(),
            analysis_type: "keywords".to_string(),
            target_audience: None,
            max_length: None,
        },
        ContentAnalysisRequest {
            text: "This article discusses advanced quantum computing principles and their applications in cryptography. The content covers quantum entanglement, superposition, and quantum algorithms like Shor's algorithm. Technical depth includes mathematical formulations and implementation considerations for quantum circuits.".to_string(),
            analysis_type: "classification".to_string(),
            target_audience: Some("high school students".to_string()),
            max_length: None,
        },
    ];
    
    // Process each test case with both GPT-4 and GPT-3.5
    for (i, test_case) in test_cases.into_iter().enumerate() {
        println!("🔄 Test Case {} - {} Analysis", i + 1, test_case.analysis_type.to_uppercase());
        println!("   📝 Content: {}...", 
            test_case.text.chars().take(100).collect::<String>());
        
        // Test with GPT-4
        println!("\n   🧠 Processing with GPT-4...");
        let mut context = TaskContext::new(
            "openai_content_analysis".to_string(),
            serde_json::to_value(&test_case)?
        );
        
        // Execute the workflow
        context = preprocessor.process(context)?;
        context = gpt4_agent.process(context)?;
        context = formatter.process(context)?;
        
        // Display GPT-4 results
        if let Some(result) = context.get_node_data::<serde_json::Value>("final_result")? {
            println!("      📊 GPT-4 Results:");
            display_analysis_results(&result, "      ");
        }
        
        // Test with GPT-3.5 for comparison
        println!("\n   🧠 Processing with GPT-3.5-turbo...");
        let mut context2 = TaskContext::new(
            "openai_content_analysis_gpt35".to_string(),
            serde_json::to_value(&test_case)?
        );
        
        // Execute the workflow with GPT-3.5
        context2 = preprocessor.process(context2)?;
        context2 = gpt35_agent.process(context2)?;
        context2 = formatter.process(context2)?;
        
        // Display GPT-3.5 results
        if let Some(result) = context2.get_node_data::<serde_json::Value>("final_result")? {
            println!("      📊 GPT-3.5-turbo Results:");
            display_analysis_results(&result, "      ");
        }
        
        println!("   ✅ Test case {} completed\n", i + 1);
    }
    
    println!("🎉 OpenAI Agent Integration Example Complete!");
    println!("=".repeat(55));
    println!("🎓 What you learned:");
    println!("   • Setting up OpenAI agent nodes with different models");
    println!("   • Configuring system prompts for specific tasks");
    println!("   • Building AI-powered content analysis workflows");
    println!("   • Preprocessing data for AI consumption");
    println!("   • Post-processing AI responses for structured output");
    println!("   • Comparing different model capabilities (GPT-4 vs GPT-3.5)");
    println!();
    println!("💡 Key insights:");
    println!("   • GPT-4 typically provides more detailed and nuanced analysis");
    println!("   • GPT-3.5-turbo is faster and more cost-effective for simpler tasks");
    println!("   • System prompts significantly influence response quality");
    println!("   • Preprocessing and post-processing are crucial for production use");
    println!();
    println!("➡️  Next steps:");
    println!("   • Try different system prompts and compare results");
    println!("   • Experiment with other GPT models (GPT-4-turbo)");
    println!("   • Add error handling and retry logic");
    println!("   • Explore the anthropic-agent.rs example");
    
    Ok(())
}

fn display_analysis_results(result: &serde_json::Value, indent: &str) {
    if let Some(analysis) = result.get("analysis") {
        let analysis_type = analysis.get("analysis_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
            
        match analysis_type {
            "sentiment" => {
                if let (Some(score), Some(label)) = (
                    analysis.get("sentiment_score").and_then(|v| v.as_f64()),
                    analysis.get("sentiment_label").and_then(|v| v.as_str())
                ) {
                    println!("{}Sentiment: {} (score: {:.2})", indent, label, score);
                    if let Some(confidence) = analysis.get("confidence").and_then(|v| v.as_f64()) {
                        println!("{}Confidence: {:.1}%", indent, confidence * 100.0);
                    }
                }
            }
            "summary" => {
                if let Some(summary) = analysis.get("summary").and_then(|v| v.as_str()) {
                    println!("{}Summary: {}", indent, summary);
                    if let Some(ratio) = analysis.get("compression_ratio").and_then(|v| v.as_f64()) {
                        println!("{}Compression: {:.1}%", indent, ratio * 100.0);
                    }
                }
            }
            "keywords" => {
                if let Some(keywords) = analysis.get("keywords").and_then(|v| v.as_array()) {
                    println!("{}Keywords:", indent);
                    for (i, keyword) in keywords.iter().take(5).enumerate() {
                        if let Some(kw) = keyword.as_str() {
                            println!("{}  {}. {}", indent, i + 1, kw);
                        }
                    }
                }
            }
            "classification" => {
                if let (Some(topic), Some(appropriateness)) = (
                    analysis.get("topic").and_then(|v| v.as_str()),
                    analysis.get("appropriateness").and_then(|v| v.as_str())
                ) {
                    println!("{}Topic: {}", indent, topic);
                    println!("{}Appropriateness: {}", indent, appropriateness);
                }
            }
            _ => {
                if let Some(analysis_text) = analysis.get("analysis").and_then(|v| v.as_str()) {
                    println!("{}Analysis: {}...", indent, 
                        analysis_text.chars().take(100).collect::<String>());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_content_preprocessor() {
        let request = ContentAnalysisRequest {
            text: "  This is a test  \n\n  with multiple lines  ".to_string(),
            analysis_type: "sentiment".to_string(),
            target_audience: None,
            max_length: None,
        };
        
        let context = TaskContext::new(
            "test".to_string(),
            serde_json::to_value(request).unwrap()
        );
        
        let node = ContentPreprocessorNode;
        let result = node.process(context).unwrap();
        
        let preprocessed: serde_json::Value = result
            .get_node_data("preprocessed_content")
            .unwrap()
            .unwrap();
        
        assert!(preprocessed.get("prompt").is_some());
        assert_eq!(preprocessed["analysis_type"], "sentiment");
    }
    
    #[test]
    fn test_sentiment_score_extraction() {
        assert_eq!(extract_sentiment_score("The score is 0.8"), 0.8);
        assert_eq!(extract_sentiment_score("Very positive sentiment"), 0.9);
        assert_eq!(extract_sentiment_score("Neutral content"), 0.0);
        assert_eq!(extract_sentiment_score("Very negative tone"), -0.9);
    }
    
    #[test]
    fn test_keyword_extraction() {
        let response = "Key points:\n- Machine learning\n- Artificial intelligence\n- Data science";
        let keywords = extract_keywords_from_response(response);
        
        assert!(keywords.len() > 0);
        assert!(keywords.iter().any(|k| k.contains("Machine learning")));
    }
}