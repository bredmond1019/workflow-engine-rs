//! Token counting implementations for different AI providers

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use tiktoken_rs::{cl100k_base, o200k_base, CoreBPE};
use unicode_segmentation::UnicodeSegmentation;
use crate::ai::tokens::{Model, Provider, TokenUsage, TokenError, TokenResult};

/// Trait for token counting implementations
#[async_trait]
pub trait TokenCounter: Send + Sync {
    /// Count tokens in input text
    async fn count_input_tokens(&self, text: &str, model: &Model) -> TokenResult<u32>;
    
    /// Count tokens in output text
    async fn count_output_tokens(&self, text: &str, model: &Model) -> TokenResult<u32>;
    
    /// Count tokens for both input and output
    async fn count_tokens(&self, input: &str, output: &str, model: &Model) -> TokenResult<TokenUsage>;
    
    /// Estimate tokens for a message before sending (useful for prompt optimization)
    async fn estimate_tokens(&self, text: &str, model: &Model) -> TokenResult<u32>;
}

/// OpenAI token counter using tiktoken
pub struct OpenAITokenCounter {
    cl100k_encoder: Arc<CoreBPE>,
    o200k_encoder: Arc<CoreBPE>,
}

impl OpenAITokenCounter {
    pub fn new() -> TokenResult<Self> {
        let cl100k_encoder = cl100k_base()
            .map_err(|e| TokenError::ConfigurationError(format!("Failed to load cl100k encoder: {}", e)))?;
        
        let o200k_encoder = o200k_base()
            .map_err(|e| TokenError::ConfigurationError(format!("Failed to load o200k encoder: {}", e)))?;

        Ok(Self {
            cl100k_encoder: Arc::new(cl100k_encoder),
            o200k_encoder: Arc::new(o200k_encoder),
        })
    }

    fn get_encoder(&self, model: &Model) -> TokenResult<&CoreBPE> {
        match model {
            Model::Gpt4 | Model::Gpt35Turbo | Model::TextEmbeddingAda002 => Ok(&self.cl100k_encoder),
            Model::Gpt4Turbo => Ok(&self.o200k_encoder),
            _ => Err(TokenError::UnsupportedModel(model.as_str().to_string())),
        }
    }

    fn count_tokens_internal(&self, text: &str, model: &Model) -> TokenResult<u32> {
        let encoder = self.get_encoder(model)?;
        let tokens = encoder.encode_with_special_tokens(text);
        Ok(tokens.len() as u32)
    }
}

#[async_trait]
impl TokenCounter for OpenAITokenCounter {
    async fn count_input_tokens(&self, text: &str, model: &Model) -> TokenResult<u32> {
        self.count_tokens_internal(text, model)
    }

    async fn count_output_tokens(&self, text: &str, model: &Model) -> TokenResult<u32> {
        self.count_tokens_internal(text, model)
    }

    async fn count_tokens(&self, input: &str, output: &str, model: &Model) -> TokenResult<TokenUsage> {
        let input_tokens = self.count_input_tokens(input, model).await?;
        let output_tokens = self.count_output_tokens(output, model).await?;
        Ok(TokenUsage::new(input_tokens, output_tokens))
    }

    async fn estimate_tokens(&self, text: &str, model: &Model) -> TokenResult<u32> {
        self.count_tokens_internal(text, model)
    }
}

/// Anthropic token counter (approximate implementation)
pub struct AnthropicTokenCounter {
    char_to_token_ratio: f64,
}

impl AnthropicTokenCounter {
    pub fn new() -> Self {
        Self {
            // Approximate ratio for Claude models (about 4 characters per token on average)
            char_to_token_ratio: 4.0,
        }
    }

    fn count_tokens_internal(&self, text: &str, model: &Model) -> TokenResult<u32> {
        match model {
            Model::Claude3Opus | Model::Claude3Sonnet | Model::Claude3Haiku => {
                // More sophisticated approximation considering:
                // - Unicode characters
                // - Common word boundaries
                // - Special tokens
                
                let graphemes = text.graphemes(true).count();
                let words = text.split_whitespace().count();
                let lines = text.lines().count();
                
                // Approximate formula based on Claude tokenization patterns
                let estimated_tokens = (graphemes as f64 / self.char_to_token_ratio)
                    + (words as f64 * 0.1) // Word boundary tokens
                    + (lines as f64 * 0.05); // Line break consideration
                
                Ok(estimated_tokens.ceil() as u32)
            }
            _ => Err(TokenError::UnsupportedModel(model.as_str().to_string())),
        }
    }
}

#[async_trait]
impl TokenCounter for AnthropicTokenCounter {
    async fn count_input_tokens(&self, text: &str, model: &Model) -> TokenResult<u32> {
        self.count_tokens_internal(text, model)
    }

    async fn count_output_tokens(&self, text: &str, model: &Model) -> TokenResult<u32> {
        self.count_tokens_internal(text, model)
    }

    async fn count_tokens(&self, input: &str, output: &str, model: &Model) -> TokenResult<TokenUsage> {
        let input_tokens = self.count_input_tokens(input, model).await?;
        let output_tokens = self.count_output_tokens(output, model).await?;
        Ok(TokenUsage::new(input_tokens, output_tokens))
    }

    async fn estimate_tokens(&self, text: &str, model: &Model) -> TokenResult<u32> {
        self.count_tokens_internal(text, model)
    }
}

/// Bedrock token counter
pub struct BedrockTokenCounter {
    anthropic_counter: AnthropicTokenCounter,
}

impl BedrockTokenCounter {
    pub fn new() -> Self {
        Self {
            anthropic_counter: AnthropicTokenCounter::new(),
        }
    }

    fn count_tokens_internal(&self, text: &str, model: &Model) -> TokenResult<u32> {
        match model {
            Model::BedrockClaude3Opus | Model::BedrockClaude3Sonnet | Model::BedrockClaude3Haiku => {
                // Use Anthropic counting for Claude models on Bedrock
                let anthropic_model = match model {
                    Model::BedrockClaude3Opus => Model::Claude3Opus,
                    Model::BedrockClaude3Sonnet => Model::Claude3Sonnet,
                    Model::BedrockClaude3Haiku => Model::Claude3Haiku,
                    _ => return Err(TokenError::UnsupportedModel(format!("Invalid Bedrock Claude model: {:?}", model))),
                };
                self.anthropic_counter.count_tokens_internal(text, &anthropic_model)
            }
            Model::TitanTextExpress => {
                // Titan models use different tokenization
                // Approximate based on character count (roughly 3.5 chars per token)
                let char_count = text.chars().count();
                Ok((char_count as f64 / 3.5).ceil() as u32)
            }
            _ => Err(TokenError::UnsupportedModel(model.as_str().to_string())),
        }
    }
}

#[async_trait]
impl TokenCounter for BedrockTokenCounter {
    async fn count_input_tokens(&self, text: &str, model: &Model) -> TokenResult<u32> {
        self.count_tokens_internal(text, model)
    }

    async fn count_output_tokens(&self, text: &str, model: &Model) -> TokenResult<u32> {
        self.count_tokens_internal(text, model)
    }

    async fn count_tokens(&self, input: &str, output: &str, model: &Model) -> TokenResult<TokenUsage> {
        let input_tokens = self.count_input_tokens(input, model).await?;
        let output_tokens = self.count_output_tokens(output, model).await?;
        Ok(TokenUsage::new(input_tokens, output_tokens))
    }

    async fn estimate_tokens(&self, text: &str, model: &Model) -> TokenResult<u32> {
        self.count_tokens_internal(text, model)
    }
}

/// Unified token counter that delegates to provider-specific implementations
pub struct UnifiedTokenCounter {
    openai_counter: OpenAITokenCounter,
    anthropic_counter: AnthropicTokenCounter,
    bedrock_counter: BedrockTokenCounter,
}

impl UnifiedTokenCounter {
    pub fn new() -> TokenResult<Self> {
        Ok(Self {
            openai_counter: OpenAITokenCounter::new()?,
            anthropic_counter: AnthropicTokenCounter::new(),
            bedrock_counter: BedrockTokenCounter::new(),
        })
    }

    fn get_counter(&self, provider: &Provider) -> &dyn TokenCounter {
        match provider {
            Provider::OpenAI => &self.openai_counter,
            Provider::Anthropic => &self.anthropic_counter,
            Provider::Bedrock => &self.bedrock_counter,
        }
    }
}

#[async_trait]
impl TokenCounter for UnifiedTokenCounter {
    async fn count_input_tokens(&self, text: &str, model: &Model) -> TokenResult<u32> {
        let provider = model.provider();
        self.get_counter(&provider).count_input_tokens(text, model).await
    }

    async fn count_output_tokens(&self, text: &str, model: &Model) -> TokenResult<u32> {
        let provider = model.provider();
        self.get_counter(&provider).count_output_tokens(text, model).await
    }

    async fn count_tokens(&self, input: &str, output: &str, model: &Model) -> TokenResult<TokenUsage> {
        let provider = model.provider();
        self.get_counter(&provider).count_tokens(input, output, model).await
    }

    async fn estimate_tokens(&self, text: &str, model: &Model) -> TokenResult<u32> {
        let provider = model.provider();
        self.get_counter(&provider).estimate_tokens(text, model).await
    }
}

/// Builder for configuring token counters
pub struct TokenCounterBuilder {
    openai_config: Option<OpenAIConfig>,
    anthropic_config: Option<AnthropicConfig>,
    bedrock_config: Option<BedrockConfig>,
}

#[derive(Debug, Clone)]
pub struct OpenAIConfig {
    pub custom_char_ratio: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct AnthropicConfig {
    pub char_to_token_ratio: f64,
}

#[derive(Debug, Clone)]
pub struct BedrockConfig {
    pub titan_char_ratio: f64,
}

impl TokenCounterBuilder {
    pub fn new() -> Self {
        Self {
            openai_config: None,
            anthropic_config: None,
            bedrock_config: None,
        }
    }

    pub fn with_openai_config(mut self, config: OpenAIConfig) -> Self {
        self.openai_config = Some(config);
        self
    }

    pub fn with_anthropic_config(mut self, config: AnthropicConfig) -> Self {
        self.anthropic_config = Some(config);
        self
    }

    pub fn with_bedrock_config(mut self, config: BedrockConfig) -> Self {
        self.bedrock_config = Some(config);
        self
    }

    pub fn build(self) -> TokenResult<UnifiedTokenCounter> {
        UnifiedTokenCounter::new()
    }
}

impl Default for TokenCounterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AnthropicConfig {
    fn default() -> Self {
        Self {
            char_to_token_ratio: 4.0,
        }
    }
}

impl Default for BedrockConfig {
    fn default() -> Self {
        Self {
            titan_char_ratio: 3.5,
        }
    }
}

/// Utility functions for token counting
pub mod utils {
    use super::*;

    /// Quick token estimation without creating a full counter (less accurate)
    pub fn quick_estimate_tokens(text: &str, model: &Model) -> u32 {
        let char_count = text.chars().count();
        let ratio = match model.provider() {
            Provider::OpenAI => 4.0,
            Provider::Anthropic => 4.0,
            Provider::Bedrock => match model {
                Model::TitanTextExpress => 3.5,
                _ => 4.0,
            },
        };
        (char_count as f64 / ratio).ceil() as u32
    }

    /// Calculate token percentage difference between two counts
    pub fn token_diff_percentage(actual: u32, estimated: u32) -> f64 {
        if estimated == 0 {
            return if actual == 0 { 0.0 } else { 100.0 };
        }
        ((actual as f64 - estimated as f64) / estimated as f64 * 100.0).abs()
    }

    /// Batch token counting for multiple texts
    pub async fn batch_count_tokens(
        counter: &dyn TokenCounter,
        texts: &[String],
        model: &Model,
    ) -> TokenResult<Vec<u32>> {
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            let count = counter.estimate_tokens(text, model).await?;
            results.push(count);
        }
        Ok(results)
    }
}