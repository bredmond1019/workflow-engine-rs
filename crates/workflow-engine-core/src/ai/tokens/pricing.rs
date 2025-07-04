//! Pricing engine for AI model cost estimation

use std::collections::HashMap;
use std::sync::{RwLock, Arc};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use crate::ai::tokens::{Model, Provider, TokenUsage, CostBreakdown, TokenError, TokenResult};
use crate::config::pricing::PricingEngineConfig;
use crate::ai::tokens::api_clients::{PricingApiClient, HttpClientConfig};
use crate::ai::tokens::api_clients::openai::OpenAIPricingClient;
use crate::ai::tokens::api_clients::anthropic::AnthropicPricingClient;
use crate::ai::tokens::api_clients::aws::AWSBedrockPricingClient;
#[cfg(feature = "streaming")]
use crate::streaming::types::StreamMetadata;

/// Pricing information for a specific model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    pub model: Model,
    pub input_price_per_token: Decimal,
    pub output_price_per_token: Decimal,
    pub currency: String,
    pub effective_date: DateTime<Utc>,
    pub pricing_tier: PricingTier,
}

/// Different pricing tiers (some providers offer volume discounts)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PricingTier {
    Standard,
    Volume,
    Enterprise,
}

/// Volume tiers for discount calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VolumeTier {
    Standard,    // No discount
    High,        // 5% discount
    Enterprise,  // 10% discount
}

/// Pricing data freshness indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PricingFreshness {
    VeryFresh,   // < 1 hour old
    Fresh,       // < 24 hours old
    Moderate,    // < 72 hours old
    Stale,       // > 72 hours old
}

/// Main pricing engine
pub struct PricingEngine {
    pricing_table: RwLock<HashMap<Model, ModelPricing>>,
    config: PricingEngineConfig,
    last_update: RwLock<DateTime<Utc>>,
    openai_client: Option<OpenAIPricingClient>,
    anthropic_client: Option<AnthropicPricingClient>,
    aws_client: Option<AWSBedrockPricingClient>,
}

impl PricingEngine {
    /// Helper function to create decimal prices from f64 values
    fn decimal_price(value: f64) -> Decimal {
        Decimal::from_f64(value).unwrap_or_else(|| {
            // Fallback to parsing string representation if direct conversion fails
            value.to_string().parse::<Decimal>().unwrap_or(Decimal::ZERO)
        })
    }

    /// Create a new pricing engine with configuration
    pub fn new(config: PricingEngineConfig) -> Self {
        let mut pricing_table = HashMap::new();
        
        // Initialize with current market rates (as of December 2024)
        Self::populate_default_pricing(&mut pricing_table);
        
        // Create HTTP client configuration
        let http_config = HttpClientConfig {
            timeout: config.api_timeout(),
            retry_attempts: config.retry_attempts,
            retry_delay: config.retry_delay(),
            user_agent: "workflow-engine-pricing/0.5.0".to_string(),
        };
        
        // Initialize API clients based on configuration
        let openai_client = if config.openai.enabled {
            Some(OpenAIPricingClient::new(config.openai.clone(), http_config.clone()))
        } else {
            None
        };
        
        let anthropic_client = if config.anthropic.enabled {
            Some(AnthropicPricingClient::new(config.anthropic.clone(), http_config.clone()))
        } else {
            None
        };
        
        let aws_client = if config.aws.enabled {
            Some(AWSBedrockPricingClient::new(config.aws.clone(), http_config))
        } else {
            None
        };
        
        Self {
            pricing_table: RwLock::new(pricing_table),
            config,
            last_update: RwLock::new(Utc::now()),
            openai_client,
            anthropic_client,
            aws_client,
        }
    }

    /// Populate the pricing table with current market rates
    fn populate_default_pricing(pricing_table: &mut HashMap<Model, ModelPricing>) {
        let now = Utc::now();
        
        // OpenAI Pricing (per 1M tokens as of Dec 2024)
        pricing_table.insert(Model::Gpt4, ModelPricing {
            model: Model::Gpt4,
            input_price_per_token: Self::decimal_price(0.00003), // $30 per 1M tokens
            output_price_per_token: Self::decimal_price(0.00006), // $60 per 1M tokens
            currency: "USD".to_string(),
            effective_date: now,
            pricing_tier: PricingTier::Standard,
        });

        pricing_table.insert(Model::Gpt4Turbo, ModelPricing {
            model: Model::Gpt4Turbo,
            input_price_per_token: Self::decimal_price(0.00001), // $10 per 1M tokens
            output_price_per_token: Self::decimal_price(0.00003), // $30 per 1M tokens
            currency: "USD".to_string(),
            effective_date: now,
            pricing_tier: PricingTier::Standard,
        });

        pricing_table.insert(Model::Gpt35Turbo, ModelPricing {
            model: Model::Gpt35Turbo,
            input_price_per_token: Self::decimal_price(0.0000005), // $0.50 per 1M tokens
            output_price_per_token: Self::decimal_price(0.0000015), // $1.50 per 1M tokens
            currency: "USD".to_string(),
            effective_date: now,
            pricing_tier: PricingTier::Standard,
        });

        pricing_table.insert(Model::TextEmbeddingAda002, ModelPricing {
            model: Model::TextEmbeddingAda002,
            input_price_per_token: Self::decimal_price(0.0000001), // $0.10 per 1M tokens
            output_price_per_token: Self::decimal_price(0.0), // No output cost for embeddings
            currency: "USD".to_string(),
            effective_date: now,
            pricing_tier: PricingTier::Standard,
        });

        // Anthropic Pricing (per 1M tokens as of Dec 2024)
        pricing_table.insert(Model::Claude3Opus, ModelPricing {
            model: Model::Claude3Opus,
            input_price_per_token: Self::decimal_price(0.000015), // $15 per 1M tokens
            output_price_per_token: Self::decimal_price(0.000075), // $75 per 1M tokens
            currency: "USD".to_string(),
            effective_date: now,
            pricing_tier: PricingTier::Standard,
        });

        pricing_table.insert(Model::Claude3Sonnet, ModelPricing {
            model: Model::Claude3Sonnet,
            input_price_per_token: Self::decimal_price(0.000003), // $3 per 1M tokens
            output_price_per_token: Self::decimal_price(0.000015), // $15 per 1M tokens
            currency: "USD".to_string(),
            effective_date: now,
            pricing_tier: PricingTier::Standard,
        });

        pricing_table.insert(Model::Claude3Haiku, ModelPricing {
            model: Model::Claude3Haiku,
            input_price_per_token: Self::decimal_price(0.00000025), // $0.25 per 1M tokens
            output_price_per_token: Self::decimal_price(0.00000125), // $1.25 per 1M tokens
            currency: "USD".to_string(),
            effective_date: now,
            pricing_tier: PricingTier::Standard,
        });

        // AWS Bedrock Pricing (generally similar to native providers but may vary)
        pricing_table.insert(Model::BedrockClaude3Opus, ModelPricing {
            model: Model::BedrockClaude3Opus,
            input_price_per_token: Self::decimal_price(0.000015),
            output_price_per_token: Self::decimal_price(0.000075),
            currency: "USD".to_string(),
            effective_date: now,
            pricing_tier: PricingTier::Standard,
        });

        pricing_table.insert(Model::BedrockClaude3Sonnet, ModelPricing {
            model: Model::BedrockClaude3Sonnet,
            input_price_per_token: Self::decimal_price(0.000003),
            output_price_per_token: Self::decimal_price(0.000015),
            currency: "USD".to_string(),
            effective_date: now,
            pricing_tier: PricingTier::Standard,
        });

        pricing_table.insert(Model::BedrockClaude3Haiku, ModelPricing {
            model: Model::BedrockClaude3Haiku,
            input_price_per_token: Self::decimal_price(0.00000025),
            output_price_per_token: Self::decimal_price(0.00000125),
            currency: "USD".to_string(),
            effective_date: now,
            pricing_tier: PricingTier::Standard,
        });

        pricing_table.insert(Model::TitanTextExpress, ModelPricing {
            model: Model::TitanTextExpress,
            input_price_per_token: Self::decimal_price(0.0000008), // $0.80 per 1M input tokens
            output_price_per_token: Self::decimal_price(0.0000016), // $1.60 per 1M output tokens
            currency: "USD".to_string(),
            effective_date: now,
            pricing_tier: PricingTier::Standard,
        });
    }

    /// Calculate cost for given token usage
    pub fn calculate_cost(&self, token_usage: &TokenUsage, model: &Model) -> TokenResult<CostBreakdown> {
        let pricing_table = self.pricing_table.read()
            .map_err(|e| TokenError::ConfigurationError(format!("Failed to read pricing table: {}", e)))?;
        
        let pricing = pricing_table.get(model)
            .ok_or_else(|| TokenError::PricingNotAvailable(model.as_str().to_string()))?;

        let input_cost = pricing.input_price_per_token * Decimal::from(token_usage.input_tokens);
        let output_cost = pricing.output_price_per_token * Decimal::from(token_usage.output_tokens);

        Ok(CostBreakdown::new(input_cost, output_cost))
    }

    /// Get pricing information for a model
    pub fn get_pricing(&self, model: &Model) -> TokenResult<ModelPricing> {
        let pricing_table = self.pricing_table.read()
            .map_err(|e| TokenError::ConfigurationError(format!("Failed to read pricing table: {}", e)))?;
        
        pricing_table.get(model)
            .cloned()
            .ok_or_else(|| TokenError::PricingNotAvailable(model.as_str().to_string()))
    }

    /// Update pricing for a specific model
    pub fn update_model_pricing(&self, pricing: ModelPricing) -> TokenResult<()> {
        let mut pricing_table = self.pricing_table.write()
            .map_err(|e| TokenError::ConfigurationError(format!("Failed to write pricing table: {}", e)))?;
        
        pricing_table.insert(pricing.model.clone(), pricing);
        
        let mut last_update = self.last_update.write()
            .map_err(|e| TokenError::ConfigurationError(format!("Failed to update timestamp: {}", e)))?;
        *last_update = Utc::now();
        
        Ok(())
    }

    /// Bulk update pricing from all configured sources
    pub async fn update_pricing_from_all_sources(&self) -> TokenResult<()> {
        let mut total_updates = 0;
        let mut errors = Vec::new();
        
        // Update from each configured provider
        if self.openai_client.is_some() {
            match self.refresh_openai_pricing().await {
                Ok(()) => total_updates += 1,
                Err(e) => errors.push(format!("OpenAI: {}", e)),
            }
        }
        
        if self.anthropic_client.is_some() {
            match self.refresh_anthropic_pricing().await {
                Ok(()) => total_updates += 1,
                Err(e) => errors.push(format!("Anthropic: {}", e)),
            }
        }
        
        if self.aws_client.is_some() {
            match self.refresh_bedrock_pricing().await {
                Ok(()) => total_updates += 1,
                Err(e) => errors.push(format!("AWS Bedrock: {}", e)),
            }
        }
        
        if !errors.is_empty() && self.config.fallback_enabled {
            log::warn!("Some pricing updates failed, using fallback data. Errors: {:?}", errors);
            // Fallback data is already loaded in the constructor
        } else if !errors.is_empty() {
            return Err(TokenError::ConfigurationError(
                format!("Failed to update pricing from {} sources: {:?}", errors.len(), errors)
            ));
        }
        
        log::info!("Successfully updated pricing from {} sources", total_updates);
        Ok(())
    }

    /// Check if pricing data needs updating
    pub fn needs_update(&self) -> bool {
        if !self.config.auto_update {
            return false;
        }

        let last_update = match self.last_update.read() {
            Ok(lock) => lock,
            Err(_) => {
                log::error!("Failed to acquire read lock on last_update");
                return false;
            }
        };
        let hours_since_update = Utc::now()
            .signed_duration_since(*last_update)
            .num_hours() as u64;

        hours_since_update >= self.config.update_interval_hours
    }

    /// Get all available pricing information
    pub fn get_all_pricing(&self) -> TokenResult<HashMap<Model, ModelPricing>> {
        let pricing_table = self.pricing_table.read()
            .map_err(|e| TokenError::ConfigurationError(format!("Failed to read pricing table: {}", e)))?;
        
        Ok(pricing_table.clone())
    }

    /// Estimate cost for given token count before making request
    pub fn estimate_cost(&self, input_tokens: u32, output_tokens: u32, model: &Model) -> TokenResult<CostBreakdown> {
        let usage = TokenUsage::new(input_tokens, output_tokens);
        self.calculate_cost(&usage, model)
    }

    /// Calculate cost savings between two models for the same usage
    pub fn compare_model_costs(&self, token_usage: &TokenUsage, model_a: &Model, model_b: &Model) -> TokenResult<CostComparison> {
        let cost_a = self.calculate_cost(token_usage, model_a)?;
        let cost_b = self.calculate_cost(token_usage, model_b)?;
        
        let savings = cost_a.total_cost - cost_b.total_cost;
        let percentage_diff = if cost_a.total_cost != Decimal::ZERO {
            (savings / cost_a.total_cost * Decimal::from(100)).abs()
        } else {
            Decimal::ZERO
        };

        let cheaper_model = if cost_a.total_cost < cost_b.total_cost { model_a.clone() } else { model_b.clone() };
        
        Ok(CostComparison {
            model_a: model_a.clone(),
            model_b: model_b.clone(),
            cost_a,
            cost_b,
            savings,
            percentage_difference: percentage_diff,
            cheaper_model,
        })
    }

    /// Calculate cost for streaming chunk metadata
    #[cfg(feature = "streaming")]
    pub fn calculate_streaming_cost(&self, metadata: &StreamMetadata) -> TokenResult<CostBreakdown> {
        let model = self.parse_model_from_string(&metadata.model)?;
        
        // For streaming, we calculate cost based on token_count if available
        let token_usage = if let Some(token_count) = metadata.token_count {
            // Estimate input vs output tokens (streaming typically means output)
            TokenUsage::new(0, token_count)
        } else {
            // If no token count, we can't calculate cost accurately
            return Err(TokenError::CountingFailed("No token count in streaming metadata".to_string()));
        };

        self.calculate_cost(&token_usage, &model)
    }

    /// Calculate cumulative cost for a streaming session
    #[cfg(feature = "streaming")]
    pub fn calculate_cumulative_streaming_cost(&self, metadata: &StreamMetadata) -> TokenResult<CostBreakdown> {
        let model = self.parse_model_from_string(&metadata.model)?;
        
        if let Some(total_tokens) = metadata.total_tokens {
            // For cumulative cost, we assume it's all output tokens in streaming
            let token_usage = TokenUsage::new(0, total_tokens);
            self.calculate_cost(&token_usage, &model)
        } else {
            Err(TokenError::CountingFailed("No total token count in streaming metadata".to_string()))
        }
    }

    /// Calculate cost per token for quick estimates
    pub fn get_cost_per_token(&self, model: &Model, is_input: bool) -> TokenResult<Decimal> {
        let pricing_table = self.pricing_table.read()
            .map_err(|e| TokenError::ConfigurationError(format!("Failed to read pricing table: {}", e)))?;
        
        let pricing = pricing_table.get(model)
            .ok_or_else(|| TokenError::PricingNotAvailable(model.as_str().to_string()))?;

        Ok(if is_input {
            pricing.input_price_per_token
        } else {
            pricing.output_price_per_token
        })
    }

    /// Calculate cost with volume discounts for high usage
    pub fn calculate_cost_with_volume_discount(&self, token_usage: &TokenUsage, model: &Model, volume_tier: &VolumeTier) -> TokenResult<CostBreakdown> {
        let mut base_cost = self.calculate_cost(token_usage, model)?;
        
        let discount_multiplier = match volume_tier {
            VolumeTier::Standard => Decimal::from(1),
            VolumeTier::High => Self::decimal_price(0.95), // 5% discount
            VolumeTier::Enterprise => Self::decimal_price(0.90), // 10% discount
        };

        base_cost.input_cost *= discount_multiplier;
        base_cost.output_cost *= discount_multiplier;
        base_cost.total_cost = base_cost.input_cost + base_cost.output_cost;

        Ok(base_cost)
    }

    /// Parse model from string (for streaming integration)
    fn parse_model_from_string(&self, model_str: &str) -> TokenResult<Model> {
        match model_str {
            "gpt-4" => Ok(Model::Gpt4),
            "gpt-4-turbo" => Ok(Model::Gpt4Turbo),
            "gpt-3.5-turbo" => Ok(Model::Gpt35Turbo),
            "text-embedding-ada-002" => Ok(Model::TextEmbeddingAda002),
            "claude-3-opus" => Ok(Model::Claude3Opus),
            "claude-3-sonnet" => Ok(Model::Claude3Sonnet),
            "claude-3-haiku" => Ok(Model::Claude3Haiku),
            "anthropic.claude-3-opus-20240229-v1:0" => Ok(Model::BedrockClaude3Opus),
            "anthropic.claude-3-sonnet-20240229-v1:0" => Ok(Model::BedrockClaude3Sonnet),
            "anthropic.claude-3-haiku-20240307-v1:0" => Ok(Model::BedrockClaude3Haiku),
            "amazon.titan-text-express-v1" => Ok(Model::TitanTextExpress),
            _ => Err(TokenError::UnsupportedModel(model_str.to_string())),
        }
    }

    /// Update pricing with external API integration
    pub async fn refresh_pricing_from_provider(&self, provider: &Provider) -> TokenResult<()> {
        match provider {
            Provider::OpenAI => self.refresh_openai_pricing().await,
            Provider::Anthropic => self.refresh_anthropic_pricing().await,
            Provider::Bedrock => self.refresh_bedrock_pricing().await,
        }
    }

    async fn refresh_openai_pricing(&self) -> TokenResult<()> {
        log::info!("Refreshing OpenAI pricing");
        
        if let Some(client) = &self.openai_client {
            if !client.is_configured() {
                return Err(TokenError::ConfigurationError("OpenAI client not configured".to_string()));
            }
            
            let pricing_updates = client.fetch_pricing().await?;
            let update_count = pricing_updates.len();
            
            let mut pricing_table = self.pricing_table.write()
                .map_err(|e| TokenError::ConfigurationError(format!("Failed to write pricing table: {}", e)))?;
            
            for (model, pricing) in pricing_updates {
                pricing_table.insert(model, pricing);
            }
            
            // Update last update timestamp
            let mut last_update = self.last_update.write()
                .map_err(|e| TokenError::ConfigurationError(format!("Failed to update timestamp: {}", e)))?;
            *last_update = Utc::now();
            
            log::info!("Successfully updated OpenAI pricing for {} models", update_count);
            Ok(())
        } else {
            Err(TokenError::ConfigurationError("OpenAI pricing client not initialized".to_string()))
        }
    }

    async fn refresh_anthropic_pricing(&self) -> TokenResult<()> {
        log::info!("Refreshing Anthropic pricing");
        
        if let Some(client) = &self.anthropic_client {
            if !client.is_configured() {
                return Err(TokenError::ConfigurationError("Anthropic client not configured".to_string()));
            }
            
            let pricing_updates = client.fetch_pricing().await?;
            let update_count = pricing_updates.len();
            
            let mut pricing_table = self.pricing_table.write()
                .map_err(|e| TokenError::ConfigurationError(format!("Failed to write pricing table: {}", e)))?;
            
            for (model, pricing) in pricing_updates {
                pricing_table.insert(model, pricing);
            }
            
            // Update last update timestamp
            let mut last_update = self.last_update.write()
                .map_err(|e| TokenError::ConfigurationError(format!("Failed to update timestamp: {}", e)))?;
            *last_update = Utc::now();
            
            log::info!("Successfully updated Anthropic pricing for {} models", update_count);
            Ok(())
        } else {
            Err(TokenError::ConfigurationError("Anthropic pricing client not initialized".to_string()))
        }
    }

    async fn refresh_bedrock_pricing(&self) -> TokenResult<()> {
        log::info!("Refreshing AWS Bedrock pricing");
        
        if let Some(client) = &self.aws_client {
            if !client.is_configured() {
                return Err(TokenError::ConfigurationError("AWS Bedrock client not configured".to_string()));
            }
            
            let pricing_updates = client.fetch_pricing().await?;
            let update_count = pricing_updates.len();
            
            let mut pricing_table = self.pricing_table.write()
                .map_err(|e| TokenError::ConfigurationError(format!("Failed to write pricing table: {}", e)))?;
            
            for (model, pricing) in pricing_updates {
                pricing_table.insert(model, pricing);
            }
            
            // Update last update timestamp
            let mut last_update = self.last_update.write()
                .map_err(|e| TokenError::ConfigurationError(format!("Failed to update timestamp: {}", e)))?;
            *last_update = Utc::now();
            
            log::info!("Successfully updated Bedrock pricing for {} models", update_count);
            Ok(())
        } else {
            Err(TokenError::ConfigurationError("AWS Bedrock pricing client not initialized".to_string()))
        }
    }


    /// Schedule automatic pricing updates
    pub async fn start_automatic_pricing_updates(self: Arc<Self>) -> TokenResult<()> {
        if !self.config.auto_update {
            return Ok(());
        }

        log::info!("Starting automatic pricing updates every {} hours", self.config.update_interval_hours);
        
        let engine = Arc::clone(&self);
        let update_interval = self.config.update_interval_hours;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_secs(update_interval * 3600)
            );
            
            loop {
                interval.tick().await;
                
                // Update pricing from all configured sources
                if let Err(e) = engine.update_pricing_from_all_sources().await {
                    log::error!("Failed to update pricing: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Get pricing freshness information
    pub fn get_pricing_freshness(&self) -> TokenResult<PricingFreshness> {
        let last_update = self.last_update.read()
            .map_err(|e| TokenError::ConfigurationError(format!("Failed to read last update: {}", e)))?;
        
        let hours_since_update = Utc::now()
            .signed_duration_since(*last_update)
            .num_hours() as u64;

        let freshness = if hours_since_update < 1 {
            PricingFreshness::VeryFresh
        } else if hours_since_update < 24 {
            PricingFreshness::Fresh
        } else if hours_since_update < 72 {
            PricingFreshness::Moderate
        } else {
            PricingFreshness::Stale
        };

        Ok(freshness)
    }
}

/// Cost comparison between two models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostComparison {
    pub model_a: Model,
    pub model_b: Model,
    pub cost_a: CostBreakdown,
    pub cost_b: CostBreakdown,
    pub savings: Decimal,
    pub percentage_difference: Decimal,
    pub cheaper_model: Model,
}


/// Utility functions for pricing operations
pub mod pricing_utils {
    use super::*;

    /// Calculate the cost per request for a given model and average token usage
    pub fn cost_per_request(pricing: &ModelPricing, avg_input_tokens: u32, avg_output_tokens: u32) -> Decimal {
        let input_cost = pricing.input_price_per_token * Decimal::from(avg_input_tokens);
        let output_cost = pricing.output_price_per_token * Decimal::from(avg_output_tokens);
        input_cost + output_cost
    }

    /// Calculate monthly cost estimate based on requests per day
    pub fn monthly_cost_estimate(
        pricing: &ModelPricing,
        avg_input_tokens: u32,
        avg_output_tokens: u32,
        requests_per_day: u32,
    ) -> Decimal {
        let cost_per_req = cost_per_request(pricing, avg_input_tokens, avg_output_tokens);
        cost_per_req * Decimal::from(requests_per_day) * Decimal::from(30) // 30 days
    }

    /// Find the most cost-effective model for given usage pattern
    pub fn find_cheapest_model(
        pricing_engine: &PricingEngine,
        models: &[Model],
        avg_input_tokens: u32,
        avg_output_tokens: u32,
    ) -> TokenResult<(Model, Decimal)> {
        let mut cheapest_model = None;
        let mut lowest_cost = Decimal::MAX;

        for model in models {
            let pricing = pricing_engine.get_pricing(model)?;
            let cost = cost_per_request(&pricing, avg_input_tokens, avg_output_tokens);
            
            if cost < lowest_cost {
                lowest_cost = cost;
                cheapest_model = Some(model.clone());
            }
        }

        cheapest_model
            .map(|model| (model, lowest_cost))
            .ok_or_else(|| TokenError::ConfigurationError("No models provided".to_string()))
    }

    /// Format cost as a human-readable string
    pub fn format_cost(cost: &Decimal, currency: &str) -> String {
        format!("{}{:.6}", 
            match currency {
                "USD" => "$",
                "EUR" => "€",
                "GBP" => "£",
                _ => "",
            },
            cost
        )
    }
}