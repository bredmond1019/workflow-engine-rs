//! API clients for fetching live pricing data

pub mod openai;
pub mod anthropic;
pub mod aws;

use serde::{Deserialize, Serialize};
use crate::ai::tokens::{Model, TokenError, TokenResult};
use crate::ai::tokens::pricing::ModelPricing;
use std::time::Duration;
use async_trait::async_trait;

/// Trait for pricing API clients
#[async_trait]
pub trait PricingApiClient: Send + Sync {
    /// Fetch current pricing data from the provider
    async fn fetch_pricing(&self) -> TokenResult<Vec<(Model, ModelPricing)>>;
    
    /// Get the provider name
    fn provider_name(&self) -> &str;
    
    /// Check if the client is properly configured
    fn is_configured(&self) -> bool;
}

/// Common HTTP client configuration
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    pub timeout: Duration,
    pub retry_attempts: u32,
    pub retry_delay: Duration,
    pub user_agent: String,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            retry_attempts: 3,
            retry_delay: Duration::from_secs(5),
            user_agent: "workflow-engine-pricing/0.5.0".to_string(),
        }
    }
}

/// Helper function to create HTTP client with common configuration
pub fn create_http_client(config: &HttpClientConfig) -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(config.timeout)
        .user_agent(&config.user_agent)
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

/// Helper function to perform retryable HTTP request
pub async fn retry_request(
    operation: impl Fn() -> futures_util::future::BoxFuture<'static, Result<reqwest::Response, reqwest::Error>>,
    retry_attempts: u32,
    retry_delay: Duration,
) -> TokenResult<reqwest::Response> {
    let mut last_error = None;
    
    for attempt in 0..retry_attempts {
        if attempt > 0 {
            tokio::time::sleep(retry_delay).await;
            log::info!("Retrying request, attempt {}/{}", attempt + 1, retry_attempts);
        }
        
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                log::warn!("Request failed on attempt {}: {}", attempt + 1, e);
                last_error = Some(e);
            }
        }
    }
    
    Err(TokenError::ConfigurationError(format!(
        "Failed after {} attempts: {}",
        retry_attempts,
        last_error.map(|e| e.to_string()).unwrap_or_else(|| "Unknown error".to_string())
    )))
}

/// Common pricing response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingResponse {
    pub model: String,
    pub input_price: f64,
    pub output_price: f64,
    pub currency: String,
    pub unit: String, // e.g., "per 1M tokens"
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_http_config_defaults() {
        let config = HttpClientConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.retry_attempts, 3);
        assert_eq!(config.retry_delay, Duration::from_secs(5));
    }
}