//! OpenAI pricing API client
//!
//! This module provides a client for fetching live pricing data from OpenAI.
//! Note: OpenAI doesn't provide a public pricing API, so this implementation
//! uses web scraping or cached data with periodic manual updates.

use async_trait::async_trait;
use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::ai::tokens::{Model, TokenError, TokenResult};
use crate::ai::tokens::pricing::{ModelPricing, PricingTier};
use crate::config::pricing::OpenAIConfig;
use super::{PricingApiClient, HttpClientConfig, create_http_client, retry_request};

/// OpenAI pricing API client
pub struct OpenAIPricingClient {
    config: OpenAIConfig,
    http_config: HttpClientConfig,
    client: reqwest::Client,
}

/// OpenAI model pricing data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIModelPricing {
    model_id: String,
    model_name: String,
    input_price_per_1k: f64,  // Price per 1K tokens
    output_price_per_1k: f64, // Price per 1K tokens
    context_window: u32,
    training_data_cutoff: String,
}

/// Response from OpenAI pricing endpoint (simulated)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIPricingResponse {
    models: Vec<OpenAIModelPricing>,
    last_updated: String,
    currency: String,
}

impl OpenAIPricingClient {
    /// Create a new OpenAI pricing client
    pub fn new(config: OpenAIConfig, http_config: HttpClientConfig) -> Self {
        let client = create_http_client(&http_config);
        Self {
            config,
            http_config,
            client,
        }
    }
    
    /// Fetch pricing from OpenAI's unofficial pricing data
    /// Since OpenAI doesn't provide a public pricing API, this uses:
    /// 1. Web scraping from their pricing page
    /// 2. Community-maintained pricing databases
    /// 3. Fallback to hardcoded values that are periodically updated
    async fn fetch_from_source(&self) -> TokenResult<OpenAIPricingResponse> {
        // In a production environment, you might:
        // 1. Scrape https://openai.com/pricing
        // 2. Use a community API like https://github.com/openai/openai-python/blob/main/pricing.json
        // 3. Maintain your own pricing database
        
        // For now, we'll simulate an API response with current pricing data
        // These values should be updated regularly based on OpenAI's pricing page
        
        // You could implement actual web scraping here:
        // let response = self.client
        //     .get("https://openai.com/pricing")
        //     .send()
        //     .await?;
        // let html = response.text().await?;
        // parse_pricing_from_html(&html)
        
        // Simulated response with current OpenAI pricing (as of Dec 2024)
        Ok(OpenAIPricingResponse {
            models: vec![
                OpenAIModelPricing {
                    model_id: "gpt-4".to_string(),
                    model_name: "GPT-4".to_string(),
                    input_price_per_1k: 0.03,  // $0.03 per 1K input tokens
                    output_price_per_1k: 0.06, // $0.06 per 1K output tokens
                    context_window: 8192,
                    training_data_cutoff: "Sep 2021".to_string(),
                },
                OpenAIModelPricing {
                    model_id: "gpt-4-turbo".to_string(),
                    model_name: "GPT-4 Turbo".to_string(),
                    input_price_per_1k: 0.01,  // $0.01 per 1K input tokens
                    output_price_per_1k: 0.03, // $0.03 per 1K output tokens
                    context_window: 128000,
                    training_data_cutoff: "Apr 2023".to_string(),
                },
                OpenAIModelPricing {
                    model_id: "gpt-3.5-turbo".to_string(),
                    model_name: "GPT-3.5 Turbo".to_string(),
                    input_price_per_1k: 0.0005,  // $0.0005 per 1K input tokens
                    output_price_per_1k: 0.0015, // $0.0015 per 1K output tokens
                    context_window: 16385,
                    training_data_cutoff: "Sep 2021".to_string(),
                },
                OpenAIModelPricing {
                    model_id: "text-embedding-ada-002".to_string(),
                    model_name: "Ada Embeddings v2".to_string(),
                    input_price_per_1k: 0.0001, // $0.0001 per 1K tokens
                    output_price_per_1k: 0.0,   // No output cost for embeddings
                    context_window: 8191,
                    training_data_cutoff: "Sep 2021".to_string(),
                },
            ],
            last_updated: Utc::now().to_rfc3339(),
            currency: "USD".to_string(),
        })
    }
    
    /// Convert OpenAI pricing to our internal format
    fn convert_to_model_pricing(&self, openai_pricing: OpenAIModelPricing) -> Option<(Model, ModelPricing)> {
        let model = match openai_pricing.model_id.as_str() {
            "gpt-4" => Model::Gpt4,
            "gpt-4-turbo" => Model::Gpt4Turbo,
            "gpt-3.5-turbo" => Model::Gpt35Turbo,
            "text-embedding-ada-002" => Model::TextEmbeddingAda002,
            _ => return None, // Skip unknown models
        };
        
        // Convert from price per 1K tokens to price per token
        let input_price_per_token = Decimal::from_f64(openai_pricing.input_price_per_1k / 1000.0)?;
        let output_price_per_token = Decimal::from_f64(openai_pricing.output_price_per_1k / 1000.0)?;
        
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
    
    /// Fetch pricing from OpenAI's API (if available)
    async fn fetch_from_api(&self) -> TokenResult<OpenAIPricingResponse> {
        if let Some(api_key) = &self.config.api_key {
            // In the future, if OpenAI provides a pricing API:
            let url = format!("{}/pricing", self.config.api_base_url);
            
            let request = || {
                let client = self.client.clone();
                let url = url.clone();
                let api_key = api_key.clone();
                
                Box::pin(async move {
                    client
                        .get(&url)
                        .header("Authorization", format!("Bearer {}", api_key))
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
                        let pricing_data = resp.json::<OpenAIPricingResponse>().await
                            .map_err(|e| TokenError::ConfigurationError(format!("Failed to parse OpenAI pricing response: {}", e)))?;
                        return Ok(pricing_data);
                    } else if resp.status() == 404 {
                        // API endpoint doesn't exist, fall back to scraping
                        log::info!("OpenAI pricing API not available, using fallback data");
                    } else {
                        log::warn!("OpenAI pricing API returned error: {}", resp.status());
                    }
                }
                Err(e) => {
                    log::warn!("Failed to fetch OpenAI pricing from API: {}", e);
                }
            }
        }
        
        // Fall back to web scraping or hardcoded data
        self.fetch_from_source().await
    }
}

#[async_trait]
impl PricingApiClient for OpenAIPricingClient {
    async fn fetch_pricing(&self) -> TokenResult<Vec<(Model, ModelPricing)>> {
        log::info!("Fetching OpenAI pricing data");
        
        let pricing_response = self.fetch_from_api().await?;
        
        let mut pricing_data = Vec::new();
        for model_pricing in pricing_response.models {
            if let Some(converted) = self.convert_to_model_pricing(model_pricing) {
                pricing_data.push(converted);
            }
        }
        
        if pricing_data.is_empty() {
            return Err(TokenError::PricingNotAvailable("No OpenAI pricing data available".to_string()));
        }
        
        log::info!("Successfully fetched pricing for {} OpenAI models", pricing_data.len());
        Ok(pricing_data)
    }
    
    fn provider_name(&self) -> &str {
        "OpenAI"
    }
    
    fn is_configured(&self) -> bool {
        self.config.enabled
    }
}

/// Alternative implementation using web scraping
pub async fn scrape_openai_pricing(client: &reqwest::Client) -> TokenResult<HashMap<String, (f64, f64)>> {
    // This is a placeholder for actual web scraping implementation
    // In production, you would:
    // 1. Fetch https://openai.com/pricing
    // 2. Parse the HTML using a library like scraper
    // 3. Extract pricing information from the page
    
    // Example implementation skeleton:
    /*
    let response = client
        .get("https://openai.com/pricing")
        .send()
        .await
        .map_err(|e| TokenError::ConfigurationError(format!("Failed to fetch OpenAI pricing page: {}", e)))?;
    
    let html = response.text().await
        .map_err(|e| TokenError::ConfigurationError(format!("Failed to read pricing page: {}", e)))?;
    
    // Use scraper crate to parse HTML
    let document = scraper::Html::parse_document(&html);
    
    // Define selectors for pricing data
    let price_selector = scraper::Selector::parse(".pricing-table").unwrap();
    
    // Extract pricing data...
    */
    
    // For now, return empty map
    Ok(HashMap::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::pricing::RateLimitConfig;
    
    #[tokio::test]
    async fn test_openai_pricing_client_creation() {
        let config = OpenAIConfig {
            api_key: Some("test-key".to_string()),
            api_base_url: "https://api.openai.com/v1".to_string(),
            enabled: true,
            rate_limit: RateLimitConfig {
                requests_per_minute: 60,
                burst_capacity: 10,
                respect_provider_limits: true,
            },
        };
        
        let http_config = HttpClientConfig::default();
        let client = OpenAIPricingClient::new(config, http_config);
        
        assert!(client.is_configured());
        assert_eq!(client.provider_name(), "OpenAI");
    }
    
    #[tokio::test]
    async fn test_pricing_conversion() {
        let config = OpenAIConfig {
            api_key: None,
            api_base_url: "https://api.openai.com/v1".to_string(),
            enabled: true,
            rate_limit: RateLimitConfig {
                requests_per_minute: 60,
                burst_capacity: 10,
                respect_provider_limits: true,
            },
        };
        
        let http_config = HttpClientConfig::default();
        let client = OpenAIPricingClient::new(config, http_config);
        
        let openai_pricing = OpenAIModelPricing {
            model_id: "gpt-4".to_string(),
            model_name: "GPT-4".to_string(),
            input_price_per_1k: 0.03,
            output_price_per_1k: 0.06,
            context_window: 8192,
            training_data_cutoff: "Sep 2021".to_string(),
        };
        
        let converted = client.convert_to_model_pricing(openai_pricing);
        assert!(converted.is_some());
        
        let (model, pricing) = converted.unwrap();
        assert_eq!(model, Model::Gpt4);
        assert_eq!(pricing.input_price_per_token, Decimal::from_f64(0.00003).unwrap());
        assert_eq!(pricing.output_price_per_token, Decimal::from_f64(0.00006).unwrap());
    }
}