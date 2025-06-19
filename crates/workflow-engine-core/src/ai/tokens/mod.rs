//! # Token Counting and Cost Estimation Module
//!
//! This module provides comprehensive token counting, cost estimation, and usage
//! analytics for all supported AI providers including OpenAI, Anthropic, and AWS Bedrock.

pub mod counter;
pub mod pricing;
pub mod api_clients;
pub mod analytics;
pub mod budget;
pub mod limits;
pub mod tests;
pub mod integration_example;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

/// Supported AI providers for token counting
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Provider {
    OpenAI,
    Anthropic,
    Bedrock,
}

/// AI model variants for each provider
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Model {
    // OpenAI models
    #[serde(rename = "gpt-4")]
    Gpt4,
    #[serde(rename = "gpt-4-turbo")]
    Gpt4Turbo,
    #[serde(rename = "gpt-3.5-turbo")]
    Gpt35Turbo,
    #[serde(rename = "text-embedding-ada-002")]
    TextEmbeddingAda002,
    
    // Anthropic models
    #[serde(rename = "claude-3-opus")]
    Claude3Opus,
    #[serde(rename = "claude-3-sonnet")]
    Claude3Sonnet,
    #[serde(rename = "claude-3-haiku")]
    Claude3Haiku,
    
    // Bedrock models
    #[serde(rename = "anthropic.claude-3-opus-20240229-v1:0")]
    BedrockClaude3Opus,
    #[serde(rename = "anthropic.claude-3-sonnet-20240229-v1:0")]
    BedrockClaude3Sonnet,
    #[serde(rename = "anthropic.claude-3-haiku-20240307-v1:0")]
    BedrockClaude3Haiku,
    #[serde(rename = "amazon.titan-text-express-v1")]
    TitanTextExpress,
}

impl Model {
    /// Get the provider for this model
    pub fn provider(&self) -> Provider {
        match self {
            Model::Gpt4 | Model::Gpt4Turbo | Model::Gpt35Turbo | Model::TextEmbeddingAda002 => Provider::OpenAI,
            Model::Claude3Opus | Model::Claude3Sonnet | Model::Claude3Haiku => Provider::Anthropic,
            Model::BedrockClaude3Opus | Model::BedrockClaude3Sonnet | Model::BedrockClaude3Haiku | Model::TitanTextExpress => Provider::Bedrock,
        }
    }

    /// Get the model identifier string
    pub fn as_str(&self) -> &'static str {
        match self {
            Model::Gpt4 => "gpt-4",
            Model::Gpt4Turbo => "gpt-4-turbo",
            Model::Gpt35Turbo => "gpt-3.5-turbo",
            Model::TextEmbeddingAda002 => "text-embedding-ada-002",
            Model::Claude3Opus => "claude-3-opus",
            Model::Claude3Sonnet => "claude-3-sonnet",
            Model::Claude3Haiku => "claude-3-haiku",
            Model::BedrockClaude3Opus => "anthropic.claude-3-opus-20240229-v1:0",
            Model::BedrockClaude3Sonnet => "anthropic.claude-3-sonnet-20240229-v1:0",
            Model::BedrockClaude3Haiku => "anthropic.claude-3-haiku-20240307-v1:0",
            Model::TitanTextExpress => "amazon.titan-text-express-v1",
        }
    }
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
}

impl TokenUsage {
    pub fn new(input_tokens: u32, output_tokens: u32) -> Self {
        Self {
            input_tokens,
            output_tokens,
            total_tokens: input_tokens + output_tokens,
        }
    }

    pub fn add(&mut self, other: &TokenUsage) {
        self.input_tokens += other.input_tokens;
        self.output_tokens += other.output_tokens;
        self.total_tokens = self.input_tokens + self.output_tokens;
    }
}

/// Cost breakdown for a request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdown {
    pub input_cost: Decimal,
    pub output_cost: Decimal,
    pub total_cost: Decimal,
    pub currency: String,
}

impl CostBreakdown {
    pub fn new(input_cost: Decimal, output_cost: Decimal) -> Self {
        Self {
            input_cost,
            output_cost,
            total_cost: input_cost + output_cost,
            currency: "USD".to_string(),
        }
    }
}

/// Complete usage record for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub provider: Provider,
    pub model: Model,
    pub token_usage: TokenUsage,
    pub cost_breakdown: CostBreakdown,
    pub user_id: Option<String>,
    pub workflow_id: Option<Uuid>,
    pub session_id: Option<Uuid>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl UsageRecord {
    pub fn new(
        provider: Provider,
        model: Model,
        token_usage: TokenUsage,
        cost_breakdown: CostBreakdown,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            provider,
            model,
            token_usage,
            cost_breakdown,
            user_id: None,
            workflow_id: None,
            session_id: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_workflow_id(mut self, workflow_id: Uuid) -> Self {
        self.workflow_id = Some(workflow_id);
        self
    }

    pub fn with_session_id(mut self, session_id: Uuid) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Errors that can occur during token counting and cost estimation
#[derive(Debug, thiserror::Error)]
pub enum TokenError {
    #[error("Unsupported model: {0}")]
    UnsupportedModel(String),
    
    #[error("Token counting failed: {0}")]
    CountingFailed(String),
    
    #[error("Pricing not available for model: {0}")]
    PricingNotAvailable(String),
    
    #[error("Budget limit exceeded: current {current}, limit {limit}")]
    BudgetLimitExceeded { current: Decimal, limit: Decimal },
    
    #[error("Rate limit exceeded for model: {0}")]
    RateLimitExceeded(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Analytics error: {0}")]
    AnalyticsError(String),
}

/// Result type for token operations
pub type TokenResult<T> = Result<T, TokenError>;

// Re-export commonly used types
pub use counter::{TokenCounter, TokenCounterBuilder};
pub use pricing::{PricingEngine, VolumeTier};
pub use crate::config::pricing::PricingEngineConfig as PricingConfig;
pub use analytics::{UsageAnalytics, AnalyticsConfig};
pub use budget::{BudgetTracker, BudgetConfig, BudgetStatus, BudgetScope, BudgetPeriod};
pub use limits::{BudgetLimits, LimitConfig};