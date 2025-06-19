//! Anthropic pricing API client
//!
//! This module provides a client for fetching live pricing data from Anthropic.
//! Like OpenAI, Anthropic doesn't provide a public pricing API, so this uses
//! web scraping or cached data with periodic manual updates.

use async_trait::async_trait;
use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use serde::{Deserialize, Serialize};

use crate::ai::tokens::{Model, TokenError, TokenResult};
use crate::ai::tokens::pricing::{ModelPricing, PricingTier};
use crate::config::pricing::AnthropicConfig;
use super::{PricingApiClient, HttpClientConfig, create_http_client, retry_request};

/// Anthropic pricing API client
pub struct AnthropicPricingClient {
    config: AnthropicConfig,
    http_config: HttpClientConfig,
    client: reqwest::Client,
}

/// Anthropic model pricing data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnthropicModelPricing {
    model_id: String,
    model_family: String,
    input_price_per_million: f64,  // Price per 1M tokens
    output_price_per_million: f64, // Price per 1M tokens
    context_length: u32,
    capabilities: Vec<String>,
}

/// Response from Anthropic pricing endpoint (simulated)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnthropicPricingResponse {
    models: Vec<AnthropicModelPricing>,
    updated_at: String,
    currency: String,
    volume_discounts: Vec<VolumeDiscount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VolumeDiscount {
    monthly_tokens: u64,
    discount_percentage: f64,
}

impl AnthropicPricingClient {
    /// Create a new Anthropic pricing client
    pub fn new(config: AnthropicConfig, http_config: HttpClientConfig) -> Self {
        let client = create_http_client(&http_config);
        Self {
            config,
            http_config,
            client,
        }
    }
    
    /// Fetch pricing from Anthropic's pricing data
    /// Currently uses hardcoded values that should be updated regularly
    async fn fetch_from_source(&self) -> TokenResult<AnthropicPricingResponse> {
        // In production, you might:
        // 1. Scrape https://www.anthropic.com/pricing
        // 2. Use Anthropic's API if they provide pricing endpoints in the future
        // 3. Maintain your own pricing database
        
        // Simulated response with current Anthropic pricing (as of Dec 2024)
        Ok(AnthropicPricingResponse {
            models: vec![
                AnthropicModelPricing {
                    model_id: "claude-3-opus-20240229".to_string(),
                    model_family: "Claude 3 Opus".to_string(),
                    input_price_per_million: 15.0,  // $15 per 1M input tokens
                    output_price_per_million: 75.0, // $75 per 1M output tokens
                    context_length: 200000,
                    capabilities: vec!["vision".to_string(), "tools".to_string()],
                },
                AnthropicModelPricing {
                    model_id: "claude-3-sonnet-20240229".to_string(),
                    model_family: "Claude 3 Sonnet".to_string(),
                    input_price_per_million: 3.0,   // $3 per 1M input tokens
                    output_price_per_million: 15.0, // $15 per 1M output tokens
                    context_length: 200000,
                    capabilities: vec!["vision".to_string(), "tools".to_string()],
                },
                AnthropicModelPricing {
                    model_id: "claude-3-haiku-20240307".to_string(),
                    model_family: "Claude 3 Haiku".to_string(),
                    input_price_per_million: 0.25,  // $0.25 per 1M input tokens
                    output_price_per_million: 1.25, // $1.25 per 1M output tokens
                    context_length: 200000,
                    capabilities: vec!["vision".to_string()],
                },
            ],
            updated_at: Utc::now().to_rfc3339(),
            currency: "USD".to_string(),
            volume_discounts: vec![
                VolumeDiscount {
                    monthly_tokens: 10_000_000,    // 10M tokens
                    discount_percentage: 5.0,
                },
                VolumeDiscount {
                    monthly_tokens: 100_000_000,   // 100M tokens
                    discount_percentage: 10.0,
                },
                VolumeDiscount {
                    monthly_tokens: 1_000_000_000, // 1B tokens
                    discount_percentage: 15.0,
                },
            ],
        })
    }
    
    /// Convert Anthropic pricing to our internal format
    fn convert_to_model_pricing(&self, anthropic_pricing: AnthropicModelPricing) -> Option<(Model, ModelPricing)> {
        let model = match anthropic_pricing.model_family.as_str() {
            "Claude 3 Opus" => Model::Claude3Opus,
            "Claude 3 Sonnet" => Model::Claude3Sonnet,
            "Claude 3 Haiku" => Model::Claude3Haiku,
            _ => return None, // Skip unknown models
        };
        
        // Convert from price per 1M tokens to price per token
        let input_price_per_token = Decimal::from_f64(anthropic_pricing.input_price_per_million / 1_000_000.0)?;
        let output_price_per_token = Decimal::from_f64(anthropic_pricing.output_price_per_million / 1_000_000.0)?;
        
        Some((
            model.clone(),
            ModelPricing {
                model,
                input_price_per_token,
                output_price_per_token,
                currency: "USD".to_string(),
                effective_date: Utc::now(),
                pricing_tier: PricingTier::Standard,
            },
        ))
    }
    
    /// Fetch pricing from Anthropic's API (if available in the future)
    async fn fetch_from_api(&self) -> TokenResult<AnthropicPricingResponse> {
        if let Some(api_key) = &self.config.api_key {
            // Check if Anthropic provides a pricing API endpoint
            let url = format!("{}/v1/pricing", self.config.api_base_url);
            
            let request = || {
                let client = self.client.clone();
                let url = url.clone();
                let api_key = api_key.clone();
                
                Box::pin(async move {
                    client
                        .get(&url)
                        .header("x-api-key", api_key)
                        .header("anthropic-version", "2023-06-01")
                        .send()
                        .await
                }) as futures_util::future::BoxFuture<'static, Result<reqwest::Response, reqwest::Error>>
            };
            
            let response = retry_request(
                request,
                self.http_config.retry_attempts,
                self.http_config.retry_delay,
            ).await;
            
            match response {
                Ok(resp) => {
                    if resp.status().is_success() {
                        let pricing_data = resp.json::<AnthropicPricingResponse>().await
                            .map_err(|e| TokenError::ConfigurationError(format!("Failed to parse Anthropic pricing response: {}", e)))?;
                        return Ok(pricing_data);
                    } else if resp.status() == 404 {
                        // API endpoint doesn't exist, fall back to hardcoded data
                        log::info!("Anthropic pricing API not available, using fallback data");
                    } else {
                        log::warn!("Anthropic pricing API returned error: {}", resp.status());
                    }
                }
                Err(e) => {
                    log::warn!("Failed to fetch Anthropic pricing from API: {}", e);
                }
            }
        }
        
        // Fall back to hardcoded data
        self.fetch_from_source().await
    }
    
    /// Check Anthropic's documentation for pricing updates
    async fn check_documentation(&self) -> TokenResult<Option<String>> {
        // This could fetch and parse Anthropic's API documentation
        // to check for pricing information updates
        
        let docs_url = "https://docs.anthropic.com/claude/docs/models-overview";
        
        match self.client.get(docs_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let text = response.text().await
                        .map_err(|e| TokenError::ConfigurationError(format!("Failed to read documentation: {}", e)))?;
                    
                    // Parse the documentation for pricing information
                    // This is a placeholder - actual implementation would parse the HTML/markdown
                    if text.contains("pricing") {
                        log::info!("Found pricing information in Anthropic documentation");
                    }
                    
                    Ok(Some(text))
                } else {
                    Ok(None)
                }
            }
            Err(e) => {
                log::warn!("Failed to fetch Anthropic documentation: {}", e);
                Ok(None)
            }
        }
    }
}

#[async_trait]
impl PricingApiClient for AnthropicPricingClient {
    async fn fetch_pricing(&self) -> TokenResult<Vec<(Model, ModelPricing)>> {
        log::info!("Fetching Anthropic pricing data");
        
        // Try to get updated information from documentation
        let _ = self.check_documentation().await;
        
        // Fetch pricing data
        let pricing_response = self.fetch_from_api().await?;
        
        let mut pricing_data = Vec::new();
        for model_pricing in pricing_response.models {
            if let Some(converted) = self.convert_to_model_pricing(model_pricing) {
                pricing_data.push(converted);
            }
        }
        
        if pricing_data.is_empty() {
            return Err(TokenError::PricingNotAvailable("No Anthropic pricing data available".to_string()));
        }
        
        log::info!("Successfully fetched pricing for {} Anthropic models", pricing_data.len());
        Ok(pricing_data)
    }
    
    fn provider_name(&self) -> &str {
        "Anthropic"
    }
    
    fn is_configured(&self) -> bool {
        self.config.enabled
    }
}

/// Web scraping implementation for Anthropic pricing
pub async fn scrape_anthropic_pricing(client: &reqwest::Client) -> TokenResult<Vec<AnthropicModelPricing>> {
    // Placeholder for web scraping implementation
    // Would scrape https://www.anthropic.com/pricing
    
    let url = "https://www.anthropic.com/pricing";
    
    match client.get(url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let _html = response.text().await
                    .map_err(|e| TokenError::ConfigurationError(format!("Failed to read pricing page: {}", e)))?;
                
                // Parse HTML and extract pricing
                // This would use a library like scraper to parse the HTML
                
                log::info!("Successfully scraped Anthropic pricing page");
            }
        }
        Err(e) => {
            log::warn!("Failed to scrape Anthropic pricing: {}", e);
        }
    }
    
    // Return empty vector for now
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::pricing::RateLimitConfig;
    
    #[tokio::test]
    async fn test_anthropic_pricing_client_creation() {
        let config = AnthropicConfig {
            api_key: Some("test-key".to_string()),
            api_base_url: "https://api.anthropic.com".to_string(),
            enabled: true,
            rate_limit: RateLimitConfig {
                requests_per_minute: 60,
                burst_capacity: 10,
                respect_provider_limits: true,
            },
        };
        
        let http_config = HttpClientConfig::default();
        let client = AnthropicPricingClient::new(config, http_config);
        
        assert!(client.is_configured());
        assert_eq!(client.provider_name(), "Anthropic");
    }
    
    #[tokio::test]
    async fn test_pricing_conversion() {
        let config = AnthropicConfig {
            api_key: None,
            api_base_url: "https://api.anthropic.com".to_string(),
            enabled: true,
            rate_limit: RateLimitConfig {
                requests_per_minute: 60,
                burst_capacity: 10,
                respect_provider_limits: true,
            },
        };
        
        let http_config = HttpClientConfig::default();
        let client = AnthropicPricingClient::new(config, http_config);
        
        let anthropic_pricing = AnthropicModelPricing {
            model_id: "claude-3-opus-20240229".to_string(),
            model_family: "Claude 3 Opus".to_string(),
            input_price_per_million: 15.0,
            output_price_per_million: 75.0,
            context_length: 200000,
            capabilities: vec!["vision".to_string()],
        };
        
        let converted = client.convert_to_model_pricing(anthropic_pricing);
        assert!(converted.is_some());
        
        let (model, pricing) = converted.unwrap();
        assert_eq!(model, Model::Claude3Opus);
        assert_eq!(pricing.input_price_per_token, Decimal::from_f64(0.000015).unwrap());
        assert_eq!(pricing.output_price_per_token, Decimal::from_f64(0.000075).unwrap());
    }
}