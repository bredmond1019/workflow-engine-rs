//! AI integration module for content processing
//!
//! This module provides integration with the main system's AI agents
//! for enhanced content analysis capabilities

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::json;

use crate::models::*;

/// Configuration for AI providers
#[derive(Debug, Clone)]
pub struct AIConfig {
    pub provider: AIProvider,
    pub model_name: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

/// Supported AI providers
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AIProvider {
    OpenAI,
    Anthropic,
}

/// AI-powered content analyzer
pub struct AIContentAnalyzer {
    configs: HashMap<AIProvider, AIConfig>,
    client: Arc<reqwest::Client>,
}

impl AIContentAnalyzer {
    pub fn new() -> Self {
        let mut configs = HashMap::new();
        
        // Configure OpenAI if API key is available
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            configs.insert(AIProvider::OpenAI, AIConfig {
                provider: AIProvider::OpenAI,
                model_name: "gpt-3.5-turbo".to_string(),
                api_key: Some(api_key),
                base_url: Some("https://api.openai.com/v1".to_string()),
                max_tokens: Some(1000),
                temperature: Some(0.3),
            });
        }
        
        // Configure Anthropic if API key is available
        if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
            configs.insert(AIProvider::Anthropic, AIConfig {
                provider: AIProvider::Anthropic,
                model_name: "claude-3-haiku-20240307".to_string(),
                api_key: Some(api_key),
                base_url: Some("https://api.anthropic.com".to_string()),
                max_tokens: Some(1000),
                temperature: Some(0.3),
            });
        }
        
        Self {
            configs,
            client: Arc::new(reqwest::Client::new()),
        }
    }
    
    /// Generate a summary using AI
    pub async fn generate_summary(&self, text: &str, max_length: usize) -> crate::Result<String> {
        let prompt = format!(
            "Please provide a concise summary of the following text in approximately {} words:\n\n{}",
            max_length / 5, // Rough estimate: 5 chars per word
            text
        );
        
        if let Some(result) = self.call_ai(&prompt, AIProvider::OpenAI).await? {
            return Ok(result);
        }
        
        if let Some(result) = self.call_ai(&prompt, AIProvider::Anthropic).await? {
            return Ok(result);
        }
        
        // Fallback to simple truncation
        Ok(self.simple_summary(text, max_length))
    }
    
    /// Analyze sentiment using AI
    pub async fn analyze_sentiment(&self, text: &str) -> crate::Result<SentimentAnalysis> {
        let prompt = format!(
            "Analyze the sentiment of the following text and respond with ONLY a JSON object in this exact format: {{\"sentiment\": \"positive|negative|neutral\", \"confidence\": 0.85, \"reasoning\": \"brief explanation\"}}.\n\nText: {}",
            text
        );
        
        if let Some(response) = self.call_ai(&prompt, AIProvider::OpenAI).await? {
            if let Ok(analysis) = self.parse_sentiment_response(&response) {
                return Ok(analysis);
            }
        }
        
        if let Some(response) = self.call_ai(&prompt, AIProvider::Anthropic).await? {
            if let Ok(analysis) = self.parse_sentiment_response(&response) {
                return Ok(analysis);
            }
        }
        
        // Fallback to simple sentiment analysis
        Ok(self.simple_sentiment_analysis(text))
    }
    
    /// Extract entities using AI
    pub async fn extract_entities(&self, text: &str) -> crate::Result<Vec<Entity>> {
        let prompt = format!(
            "Extract named entities from the following text and respond with ONLY a JSON array in this format: [{{\"name\": \"entity name\", \"type\": \"Person|Organization|Location|Date|Money|Technology|Concept\", \"confidence\": 0.95}}].\n\nText: {}",
            text
        );
        
        if let Some(response) = self.call_ai(&prompt, AIProvider::OpenAI).await? {
            if let Ok(entities) = self.parse_entities_response(&response) {
                return Ok(entities);
            }
        }
        
        if let Some(response) = self.call_ai(&prompt, AIProvider::Anthropic).await? {
            if let Ok(entities) = self.parse_entities_response(&response) {
                return Ok(entities);
            }
        }
        
        // Fallback to simple entity extraction
        Ok(self.simple_entity_extraction(text))
    }
    
    /// Classify content using AI
    pub async fn classify_content(&self, text: &str) -> crate::Result<ContentClassification> {
        let prompt = format!(
            "Classify the following text and respond with ONLY a JSON object: {{\"category\": \"category_name\", \"subcategory\": \"subcategory_name\", \"confidence\": 0.90, \"topics\": [\"topic1\", \"topic2\"]}}.\n\nText: {}",
            text
        );
        
        if let Some(response) = self.call_ai(&prompt, AIProvider::OpenAI).await? {
            if let Ok(classification) = self.parse_classification_response(&response) {
                return Ok(classification);
            }
        }
        
        if let Some(response) = self.call_ai(&prompt, AIProvider::Anthropic).await? {
            if let Ok(classification) = self.parse_classification_response(&response) {
                return Ok(classification);
            }
        }
        
        // Fallback to simple classification
        Ok(self.simple_content_classification(text))
    }
    
    /// Call AI provider with fallback
    async fn call_ai(&self, prompt: &str, provider: AIProvider) -> crate::Result<Option<String>> {
        if let Some(config) = self.configs.get(&provider) {
            match provider {
                AIProvider::OpenAI => self.call_openai(prompt, config).await,
                AIProvider::Anthropic => self.call_anthropic(prompt, config).await,
            }
        } else {
            Ok(None)
        }
    }
    
    /// Call OpenAI API
    async fn call_openai(&self, prompt: &str, config: &AIConfig) -> crate::Result<Option<String>> {
        let api_key = config.api_key.as_ref().ok_or_else(|| ProcessingError::InternalError {
            message: "OpenAI API key not configured".to_string(),
            trace: None,
        })?;
        
        let request_body = json!({
            "model": config.model_name,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "max_tokens": config.max_tokens.unwrap_or(1000),
            "temperature": config.temperature.unwrap_or(0.3)
        });
        
        let response = self.client
            .post(&format!("{}/chat/completions", config.base_url.as_ref().unwrap()))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ProcessingError::NetworkError {
                url: Some("OpenAI API".to_string()),
                error_message: e.to_string(),
            })?;
        
        if !response.status().is_success() {
            return Err(ProcessingError::NetworkError {
                url: Some("OpenAI API".to_string()),
                error_message: format!("API request failed with status: {}", response.status()),
            });
        }
        
        let response_json: serde_json::Value = response.json().await.map_err(|e| ProcessingError::ParseError {
            message: format!("Failed to parse OpenAI response: {}", e),
            position: None,
        })?;
        
        if let Some(content) = response_json
            .get("choices")
            .and_then(|choices| choices.get(0))
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str()) {
            Ok(Some(content.trim().to_string()))
        } else {
            Err(ProcessingError::ParseError {
                message: "Invalid OpenAI response format".to_string(),
                position: None,
            })
        }
    }
    
    /// Call Anthropic API
    async fn call_anthropic(&self, prompt: &str, config: &AIConfig) -> crate::Result<Option<String>> {
        let api_key = config.api_key.as_ref().ok_or_else(|| ProcessingError::InternalError {
            message: "Anthropic API key not configured".to_string(),
            trace: None,
        })?;
        
        let request_body = json!({
            "model": config.model_name,
            "max_tokens": config.max_tokens.unwrap_or(1000),
            "messages": [
                {
                    "role": "user", 
                    "content": prompt
                }
            ]
        });
        
        let response = self.client
            .post(&format!("{}/v1/messages", config.base_url.as_ref().unwrap()))
            .header("x-api-key", api_key)
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ProcessingError::NetworkError {
                url: Some("Anthropic API".to_string()),
                error_message: e.to_string(),
            })?;
        
        if !response.status().is_success() {
            return Err(ProcessingError::NetworkError {
                url: Some("Anthropic API".to_string()),
                error_message: format!("API request failed with status: {}", response.status()),
            });
        }
        
        let response_json: serde_json::Value = response.json().await.map_err(|e| ProcessingError::ParseError {
            message: format!("Failed to parse Anthropic response: {}", e),
            position: None,
        })?;
        
        if let Some(content) = response_json
            .get("content")
            .and_then(|content| content.get(0))
            .and_then(|item| item.get("text"))
            .and_then(|text| text.as_str()) {
            Ok(Some(content.trim().to_string()))
        } else {
            Err(ProcessingError::ParseError {
                message: "Invalid Anthropic response format".to_string(),
                position: None,
            })
        }
    }
    
    /// Parse sentiment analysis response
    fn parse_sentiment_response(&self, response: &str) -> Result<SentimentAnalysis, ProcessingError> {
        // Try to extract JSON from response
        let json_str = if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                &response[start..=end]
            } else {
                response
            }
        } else {
            response
        };
        
        let parsed: serde_json::Value = serde_json::from_str(json_str).map_err(|e| ProcessingError::ParseError {
            message: format!("Failed to parse sentiment JSON: {}", e),
            position: None,
        })?;
        
        let sentiment = parsed.get("sentiment")
            .and_then(|s| s.as_str())
            .unwrap_or("neutral")
            .to_string();
            
        let confidence = parsed.get("confidence")
            .and_then(|c| c.as_f64())
            .unwrap_or(0.5) as f32;
            
        let reasoning = parsed.get("reasoning")
            .and_then(|r| r.as_str())
            .unwrap_or("AI analysis")
            .to_string();
        
        Ok(SentimentAnalysis {
            sentiment,
            confidence,
            reasoning,
        })
    }
    
    /// Parse entities response
    fn parse_entities_response(&self, response: &str) -> Result<Vec<Entity>, ProcessingError> {
        // Try to extract JSON array from response
        let json_str = if let Some(start) = response.find('[') {
            if let Some(end) = response.rfind(']') {
                &response[start..=end]
            } else {
                response
            }
        } else {
            response
        };
        
        let parsed: serde_json::Value = serde_json::from_str(json_str).map_err(|e| ProcessingError::ParseError {
            message: format!("Failed to parse entities JSON: {}", e),
            position: None,
        })?;
        
        let mut entities = Vec::new();
        
        if let Some(array) = parsed.as_array() {
            for item in array {
                if let (Some(name), Some(entity_type), Some(confidence)) = (
                    item.get("name").and_then(|n| n.as_str()),
                    item.get("type").and_then(|t| t.as_str()),
                    item.get("confidence").and_then(|c| c.as_f64())
                ) {
                    let entity_type = match entity_type.to_lowercase().as_str() {
                        "person" => EntityType::Person,
                        "organization" => EntityType::Organization,
                        "location" => EntityType::Location,
                        "date" => EntityType::Date,
                        "money" => EntityType::Money,
                        "technology" => EntityType::Technology,
                        "concept" => EntityType::Concept,
                        _ => EntityType::Other(entity_type.to_string()),
                    };
                    
                    entities.push(Entity {
                        name: name.to_string(),
                        entity_type,
                        confidence: confidence as f32,
                        mentions: Vec::new(), // Would need additional processing
                        linked_data_uri: None,
                    });
                }
            }
        }
        
        Ok(entities)
    }
    
    /// Parse classification response
    fn parse_classification_response(&self, response: &str) -> Result<ContentClassification, ProcessingError> {
        let json_str = if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                &response[start..=end]
            } else {
                response
            }
        } else {
            response
        };
        
        let parsed: serde_json::Value = serde_json::from_str(json_str).map_err(|e| ProcessingError::ParseError {
            message: format!("Failed to parse classification JSON: {}", e),
            position: None,
        })?;
        
        let category = parsed.get("category")
            .and_then(|c| c.as_str())
            .unwrap_or("general")
            .to_string();
            
        let subcategory = parsed.get("subcategory")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());
            
        let confidence = parsed.get("confidence")
            .and_then(|c| c.as_f64())
            .unwrap_or(0.5) as f32;
            
        let topics = parsed.get("topics")
            .and_then(|t| t.as_array())
            .map(|arr| arr.iter()
                .filter_map(|item| item.as_str())
                .map(|s| s.to_string())
                .collect())
            .unwrap_or_else(Vec::new);
        
        Ok(ContentClassification {
            category,
            subcategory,
            confidence,
            topics,
        })
    }
    
    /// Simple fallback summary
    fn simple_summary(&self, text: &str, max_length: usize) -> String {
        if text.len() <= max_length {
            return text.to_string();
        }
        
        // Take first few sentences up to max_length
        let mut summary = String::new();
        for sentence in text.split('.') {
            let sentence = sentence.trim();
            if !sentence.is_empty() {
                let candidate = if summary.is_empty() {
                    sentence.to_string()
                } else {
                    format!("{}. {}", summary, sentence)
                };
                
                if candidate.len() <= max_length {
                    summary = candidate;
                } else {
                    break;
                }
            }
        }
        
        if summary.is_empty() {
            text.chars().take(max_length).collect::<String>()
        } else {
            summary
        }
    }
    
    /// Simple fallback sentiment analysis
    fn simple_sentiment_analysis(&self, text: &str) -> SentimentAnalysis {
        let text_lower = text.to_lowercase();
        
        let positive_words = ["good", "great", "excellent", "amazing", "wonderful", "fantastic", "love", "like", "happy", "pleased"];
        let negative_words = ["bad", "terrible", "awful", "hate", "dislike", "sad", "angry", "disappointed", "frustrated"];
        
        let mut positive_count = 0;
        let mut negative_count = 0;
        
        for word in positive_words.iter() {
            if text_lower.contains(word) {
                positive_count += 1;
            }
        }
        
        for word in negative_words.iter() {
            if text_lower.contains(word) {
                negative_count += 1;
            }
        }
        
        let sentiment = if positive_count > negative_count {
            "positive"
        } else if negative_count > positive_count {
            "negative"
        } else {
            "neutral"
        };
        
        let confidence = if positive_count + negative_count == 0 {
            0.3
        } else {
            0.6
        };
        
        SentimentAnalysis {
            sentiment: sentiment.to_string(),
            confidence,
            reasoning: "Simple keyword-based analysis".to_string(),
        }
    }
    
    /// Simple fallback entity extraction
    fn simple_entity_extraction(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();
        
        // Simple pattern matching for common entities
        // This is very basic - in a real implementation you'd use more sophisticated NLP
        
        // Look for capitalized words that might be names/organizations
        for word in text.split_whitespace() {
            let cleaned = word.trim_matches(|c: char| !c.is_alphabetic());
            if cleaned.len() > 2 && cleaned.chars().next().unwrap().is_uppercase() {
                entities.push(Entity {
                    name: cleaned.to_string(),
                    entity_type: EntityType::Other("potential_name".to_string()),
                    confidence: 0.4,
                    mentions: Vec::new(),
                    linked_data_uri: None,
                });
            }
        }
        
        entities
    }
    
    /// Simple content classification
    fn simple_content_classification(&self, text: &str) -> ContentClassification {
        let text_lower = text.to_lowercase();
        
        let (category, topics) = if text_lower.contains("technical") || text_lower.contains("programming") || text_lower.contains("software") {
            ("technology", vec!["technical".to_string(), "software".to_string()])
        } else if text_lower.contains("business") || text_lower.contains("marketing") || text_lower.contains("sales") {
            ("business", vec!["business".to_string(), "marketing".to_string()])
        } else if text_lower.contains("science") || text_lower.contains("research") || text_lower.contains("study") {
            ("science", vec!["research".to_string(), "science".to_string()])
        } else {
            ("general", vec!["general".to_string()])
        };
        
        ContentClassification {
            category: category.to_string(),
            subcategory: None,
            confidence: 0.5,
            topics,
        }
    }
}

impl Default for AIContentAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Sentiment analysis result
#[derive(Debug, Clone)]
pub struct SentimentAnalysis {
    pub sentiment: String,
    pub confidence: f32,
    pub reasoning: String,
}

/// Content classification result
#[derive(Debug, Clone)]
pub struct ContentClassification {
    pub category: String,
    pub subcategory: Option<String>,
    pub confidence: f32,
    pub topics: Vec<String>,
}