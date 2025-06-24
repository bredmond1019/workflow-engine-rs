//! Pricing engine configuration module
//!
//! This module provides comprehensive configuration for the pricing engine,
//! including API keys, update intervals, caching, and fallback strategies.

use std::env;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use crate::config::{ConfigError, ConfigResult};

/// Comprehensive pricing engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingEngineConfig {
    /// Whether to enable automatic pricing updates
    pub auto_update: bool,
    
    /// How often to update pricing data (in hours)
    pub update_interval_hours: u64,
    
    /// How long to cache pricing data before considering it stale (in hours)
    pub cache_duration_hours: u64,
    
    /// Whether to enable fallback to cached/hardcoded pricing
    pub fallback_enabled: bool,
    
    /// API timeout for pricing requests (in seconds)
    pub api_timeout_seconds: u64,
    
    /// Number of retry attempts for failed API calls
    pub retry_attempts: u32,
    
    /// Delay between retry attempts (in seconds)
    pub retry_delay_seconds: u64,
    
    /// Provider-specific configurations
    pub openai: OpenAIConfig,
    pub anthropic: AnthropicConfig,
    pub aws: AWSConfig,
    
    /// Cache configuration
    pub cache: CacheConfig,
    
    /// Monitoring and alerting configuration
    pub monitoring: PricingMonitoringConfig,
}

/// OpenAI API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIConfig {
    /// OpenAI API key
    pub api_key: Option<String>,
    
    /// Custom API base URL (for testing or proxies)
    pub api_base_url: String,
    
    /// Whether to enable OpenAI pricing updates
    pub enabled: bool,
    
    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,
}

/// Anthropic API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicConfig {
    /// Anthropic API key
    pub api_key: Option<String>,
    
    /// Custom API base URL
    pub api_base_url: String,
    
    /// Whether to enable Anthropic pricing updates
    pub enabled: bool,
    
    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,
}

/// AWS configuration for Bedrock pricing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AWSConfig {
    /// AWS access key ID
    pub access_key_id: Option<String>,
    
    /// AWS secret access key
    pub secret_access_key: Option<String>,
    
    /// AWS region
    pub region: String,
    
    /// Whether to enable AWS Bedrock pricing updates
    pub enabled: bool,
    
    /// Specific Bedrock configuration
    pub bedrock: BedrockConfig,
}

/// AWS Bedrock specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BedrockConfig {
    /// Whether to fetch pricing for all available regions
    pub multi_region: bool,
    
    /// Specific regions to fetch pricing for (if multi_region is false)
    pub regions: Vec<String>,
    
    /// Whether to include on-demand pricing
    pub include_on_demand: bool,
    
    /// Whether to include provisioned throughput pricing
    pub include_provisioned: bool,
}

/// Rate limiting configuration for API calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum requests per minute
    pub requests_per_minute: u32,
    
    /// Burst capacity for rate limiting
    pub burst_capacity: u32,
    
    /// Whether to respect provider rate limit headers
    pub respect_provider_limits: bool,
}

/// Cache configuration for pricing data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Cache backend type
    pub backend: CacheBackend,
    
    /// Maximum cache size in MB
    pub max_size_mb: u64,
    
    /// Cache TTL for pricing data (in hours)
    pub ttl_hours: u64,
    
    /// Whether to persist cache to disk
    pub persist_to_disk: bool,
    
    /// Cache file path (if persist_to_disk is true)
    pub cache_file_path: Option<String>,
}

/// Cache backend options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheBackend {
    Memory,
    Redis(RedisConfig),
    File(String), // file path
}

/// Redis cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub password: Option<String>,
    pub database: u8,
    pub key_prefix: String,
}

/// Monitoring configuration for pricing engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingMonitoringConfig {
    /// Whether to enable pricing update alerts
    pub enable_alerts: bool,
    
    /// Webhook URL for pricing update notifications
    pub webhook_url: Option<String>,
    
    /// Email addresses for pricing alerts
    pub alert_emails: Vec<String>,
    
    /// Metrics collection configuration
    pub metrics: MetricsConfig,
}

/// Metrics collection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Whether to collect pricing update metrics
    pub enabled: bool,
    
    /// Metric name prefix
    pub prefix: String,
    
    /// Whether to include detailed timing metrics
    pub include_timing: bool,
    
    /// Whether to include error rate metrics
    pub include_errors: bool,
}

impl PricingEngineConfig {
    /// Load pricing configuration from environment variables
    pub fn from_env() -> ConfigResult<Self> {
        Ok(Self {
            auto_update: env::var("PRICING_AUTO_UPDATE")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            
            update_interval_hours: env::var("PRICING_UPDATE_INTERVAL_HOURS")
                .unwrap_or_else(|_| "6".to_string())
                .parse()
                .map_err(|e| ConfigError::parse_error(
                    format!("PRICING_UPDATE_INTERVAL_HOURS: {}", e),
                    "environment variable",
                    "PRICING_UPDATE_INTERVAL_HOURS"
                ))?,
            
            cache_duration_hours: env::var("PRICING_CACHE_DURATION_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .map_err(|e| ConfigError::parse_error(
                    format!("PRICING_CACHE_DURATION_HOURS: {}", e),
                    "environment variable",
                    "PRICING_CACHE_DURATION_HOURS"
                ))?,
            
            fallback_enabled: env::var("PRICING_FALLBACK_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            
            api_timeout_seconds: env::var("PRICING_API_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .map_err(|e| ConfigError::parse_error(
                    format!("PRICING_API_TIMEOUT_SECONDS: {}", e),
                    "environment variable",
                    "PRICING_API_TIMEOUT_SECONDS"
                ))?,
            
            retry_attempts: env::var("PRICING_RETRY_ATTEMPTS")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .map_err(|e| ConfigError::parse_error(
                    format!("PRICING_RETRY_ATTEMPTS: {}", e),
                    "environment variable",
                    "PRICING_RETRY_ATTEMPTS"
                ))?,
            
            retry_delay_seconds: env::var("PRICING_RETRY_DELAY_SECONDS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .map_err(|e| ConfigError::parse_error(
                    format!("PRICING_RETRY_DELAY_SECONDS: {}", e),
                    "environment variable",
                    "PRICING_RETRY_DELAY_SECONDS"
                ))?,
            
            openai: OpenAIConfig::from_env()?,
            anthropic: AnthropicConfig::from_env()?,
            aws: AWSConfig::from_env()?,
            cache: CacheConfig::from_env()?,
            monitoring: PricingMonitoringConfig::from_env()?,
        })
    }
    
    /// Validate pricing configuration
    pub fn validate(&self) -> ConfigResult<()> {
        if self.update_interval_hours == 0 {
            return Err(ConfigError::validation_failed(
                "update_interval_hours must be greater than 0",
                "pricing config",
                "Set PRICING_UPDATE_INTERVAL_HOURS to a value greater than 0",
                vec![("update_interval_hours".to_string(), "0".to_string())]
            ));
        }
        
        if self.cache_duration_hours == 0 {
            return Err(ConfigError::validation_failed(
                "cache_duration_hours must be greater than 0",
                "pricing config",
                "Set PRICING_CACHE_DURATION_HOURS to a value greater than 0",
                vec![("cache_duration_hours".to_string(), "0".to_string())]
            ));
        }
        
        if self.api_timeout_seconds == 0 {
            return Err(ConfigError::validation_failed(
                "api_timeout_seconds must be greater than 0",
                "pricing config",
                "Set PRICING_API_TIMEOUT_SECONDS to a value greater than 0",
                vec![("api_timeout_seconds".to_string(), "0".to_string())]
            ));
        }
        
        if self.retry_delay_seconds == 0 {
            return Err(ConfigError::validation_failed(
                "retry_delay_seconds must be greater than 0",
                "pricing config",
                "Set PRICING_RETRY_DELAY_SECONDS to a value greater than 0",
                vec![("retry_delay_seconds".to_string(), "0".to_string())]
            ));
        }
        
        self.openai.validate()?;
        self.anthropic.validate()?;
        self.aws.validate()?;
        self.cache.validate()?;
        
        Ok(())
    }
    
    /// Get API timeout as Duration
    pub fn api_timeout(&self) -> Duration {
        Duration::from_secs(self.api_timeout_seconds)
    }
    
    /// Get retry delay as Duration
    pub fn retry_delay(&self) -> Duration {
        Duration::from_secs(self.retry_delay_seconds)
    }
    
    /// Get update interval as Duration
    pub fn update_interval(&self) -> Duration {
        Duration::from_secs(self.update_interval_hours * 3600)
    }
    
    /// Check if any provider is enabled
    pub fn has_enabled_providers(&self) -> bool {
        self.openai.enabled || self.anthropic.enabled || self.aws.enabled
    }
}

impl OpenAIConfig {
    fn from_env() -> ConfigResult<Self> {
        Ok(Self {
            api_key: env::var("OPENAI_API_KEY").ok(),
            api_base_url: env::var("OPENAI_API_BASE_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
            enabled: env::var("OPENAI_PRICING_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            rate_limit: RateLimitConfig::openai_defaults(),
        })
    }
    
    fn validate(&self) -> ConfigResult<()> {
        if self.enabled && self.api_key.is_none() {
            return Err(ConfigError::validation_failed(
                "OpenAI pricing is enabled but OPENAI_API_KEY is not set",
                "OpenAI config",
                "Set OPENAI_API_KEY environment variable or disable with OPENAI_PRICING_ENABLED=false",
                vec![("api_key".to_string(), "missing".to_string())]
            ));
        }
        Ok(())
    }
}

impl AnthropicConfig {
    fn from_env() -> ConfigResult<Self> {
        Ok(Self {
            api_key: env::var("ANTHROPIC_API_KEY").ok(),
            api_base_url: env::var("ANTHROPIC_API_BASE_URL")
                .unwrap_or_else(|_| "https://api.anthropic.com".to_string()),
            enabled: env::var("ANTHROPIC_PRICING_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            rate_limit: RateLimitConfig::anthropic_defaults(),
        })
    }
    
    fn validate(&self) -> ConfigResult<()> {
        if self.enabled && self.api_key.is_none() {
            return Err(ConfigError::validation_failed(
                "Anthropic pricing is enabled but ANTHROPIC_API_KEY is not set",
                "Anthropic config",
                "Set ANTHROPIC_API_KEY environment variable or disable with ANTHROPIC_PRICING_ENABLED=false",
                vec![("api_key".to_string(), "missing".to_string())]
            ));
        }
        Ok(())
    }
}

impl AWSConfig {
    fn from_env() -> ConfigResult<Self> {
        Ok(Self {
            access_key_id: env::var("AWS_ACCESS_KEY_ID").ok(),
            secret_access_key: env::var("AWS_SECRET_ACCESS_KEY").ok(),
            region: env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            enabled: env::var("AWS_PRICING_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            bedrock: BedrockConfig::from_env()?,
        })
    }
    
    fn validate(&self) -> ConfigResult<()> {
        if self.enabled {
            if self.access_key_id.is_none() {
                return Err(ConfigError::validation_failed(
                    "AWS pricing is enabled but AWS_ACCESS_KEY_ID is not set",
                    "AWS config",
                    "Set AWS_ACCESS_KEY_ID environment variable or disable with AWS_PRICING_ENABLED=false",
                    vec![("access_key_id".to_string(), "missing".to_string())]
                ));
            }
            if self.secret_access_key.is_none() {
                return Err(ConfigError::validation_failed(
                    "AWS pricing is enabled but AWS_SECRET_ACCESS_KEY is not set",
                    "AWS config",
                    "Set AWS_SECRET_ACCESS_KEY environment variable or disable with AWS_PRICING_ENABLED=false",
                    vec![("secret_access_key".to_string(), "missing".to_string())]
                ));
            }
        }
        Ok(())
    }
}

impl BedrockConfig {
    fn from_env() -> ConfigResult<Self> {
        let regions_str = env::var("AWS_BEDROCK_REGIONS")
            .unwrap_or_else(|_| "us-east-1,us-west-2".to_string());
        let regions = regions_str
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        
        Ok(Self {
            multi_region: env::var("AWS_BEDROCK_MULTI_REGION")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            regions,
            include_on_demand: env::var("AWS_BEDROCK_INCLUDE_ON_DEMAND")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            include_provisioned: env::var("AWS_BEDROCK_INCLUDE_PROVISIONED")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
        })
    }
}

impl RateLimitConfig {
    fn openai_defaults() -> Self {
        Self {
            requests_per_minute: 500,
            burst_capacity: 10,
            respect_provider_limits: true,
        }
    }
    
    fn anthropic_defaults() -> Self {
        Self {
            requests_per_minute: 100,
            burst_capacity: 5,
            respect_provider_limits: true,
        }
    }
}

impl CacheConfig {
    fn from_env() -> ConfigResult<Self> {
        let backend = match env::var("PRICING_CACHE_BACKEND").as_deref() {
            Ok("redis") => {
                let redis_url = env::var("REDIS_URL")
                    .or_else(|_| env::var("PRICING_CACHE_REDIS_URL"))
                    .unwrap_or_else(|_| "redis://localhost:6379".to_string());
                
                CacheBackend::Redis(RedisConfig {
                    url: redis_url,
                    password: env::var("REDIS_PASSWORD").ok(),
                    database: env::var("PRICING_CACHE_REDIS_DB")
                        .unwrap_or_else(|_| "0".to_string())
                        .parse()
                        .unwrap_or(0),
                    key_prefix: env::var("PRICING_CACHE_KEY_PREFIX")
                        .unwrap_or_else(|_| "pricing:".to_string()),
                })
            },
            Ok("file") => {
                let file_path = env::var("PRICING_CACHE_FILE_PATH")
                    .unwrap_or_else(|_| "/tmp/pricing_cache.json".to_string());
                CacheBackend::File(file_path)
            },
            _ => CacheBackend::Memory,
        };
        
        Ok(Self {
            backend,
            max_size_mb: env::var("PRICING_CACHE_MAX_SIZE_MB")
                .unwrap_or_else(|_| "50".to_string())
                .parse()
                .map_err(|e| ConfigError::parse_error(
                    format!("PRICING_CACHE_MAX_SIZE_MB: {}", e),
                    "environment variable",
                    "PRICING_CACHE_MAX_SIZE_MB"
                ))?,
            ttl_hours: env::var("PRICING_CACHE_TTL_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .map_err(|e| ConfigError::parse_error(
                    format!("PRICING_CACHE_TTL_HOURS: {}", e),
                    "environment variable",
                    "PRICING_CACHE_TTL_HOURS"
                ))?,
            persist_to_disk: env::var("PRICING_CACHE_PERSIST")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            cache_file_path: env::var("PRICING_CACHE_FILE_PATH").ok(),
        })
    }
    
    fn validate(&self) -> ConfigResult<()> {
        if self.max_size_mb == 0 {
            return Err(ConfigError::validation_failed(
                "Cache max_size_mb must be greater than 0",
                "cache config",
                "Set PRICING_CACHE_MAX_SIZE_MB to a value greater than 0",
                vec![("max_size_mb".to_string(), "0".to_string())]
            ));
        }
        
        if self.ttl_hours == 0 {
            return Err(ConfigError::validation_failed(
                "Cache TTL must be greater than 0",
                "cache config",
                "Set PRICING_CACHE_TTL_HOURS to a value greater than 0",
                vec![("ttl_hours".to_string(), "0".to_string())]
            ));
        }
        
        Ok(())
    }
}

impl PricingMonitoringConfig {
    fn from_env() -> ConfigResult<Self> {
        let alert_emails_str = env::var("PRICING_ALERT_EMAILS").unwrap_or_default();
        let alert_emails = if alert_emails_str.is_empty() {
            Vec::new()
        } else {
            alert_emails_str
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        };
        
        Ok(Self {
            enable_alerts: env::var("PRICING_ENABLE_ALERTS")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            webhook_url: env::var("PRICING_WEBHOOK_URL").ok(),
            alert_emails,
            metrics: MetricsConfig::from_env()?,
        })
    }
}

impl MetricsConfig {
    fn from_env() -> ConfigResult<Self> {
        Ok(Self {
            enabled: env::var("PRICING_METRICS_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            prefix: env::var("PRICING_METRICS_PREFIX")
                .unwrap_or_else(|_| "pricing_engine".to_string()),
            include_timing: env::var("PRICING_METRICS_INCLUDE_TIMING")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            include_errors: env::var("PRICING_METRICS_INCLUDE_ERRORS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        })
    }
}

impl Default for PricingEngineConfig {
    fn default() -> Self {
        Self {
            auto_update: true,
            update_interval_hours: 6,
            cache_duration_hours: 24,
            fallback_enabled: true,
            api_timeout_seconds: 30,
            retry_attempts: 3,
            retry_delay_seconds: 5,
            openai: OpenAIConfig::default(),
            anthropic: AnthropicConfig::default(),
            aws: AWSConfig::default(),
            cache: CacheConfig::default(),
            monitoring: PricingMonitoringConfig::default(),
        }
    }
}

impl Default for OpenAIConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            api_base_url: "https://api.openai.com/v1".to_string(),
            enabled: false, // Disabled by default without API key
            rate_limit: RateLimitConfig::openai_defaults(),
        }
    }
}

impl Default for AnthropicConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            api_base_url: "https://api.anthropic.com".to_string(),
            enabled: false, // Disabled by default without API key
            rate_limit: RateLimitConfig::anthropic_defaults(),
        }
    }
}

impl Default for AWSConfig {
    fn default() -> Self {
        Self {
            access_key_id: None,
            secret_access_key: None,
            region: "us-east-1".to_string(),
            enabled: false, // Disabled by default without credentials
            bedrock: BedrockConfig::default(),
        }
    }
}

impl Default for BedrockConfig {
    fn default() -> Self {
        Self {
            multi_region: false,
            regions: vec!["us-east-1".to_string(), "us-west-2".to_string()],
            include_on_demand: true,
            include_provisioned: false,
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            backend: CacheBackend::Memory,
            max_size_mb: 50,
            ttl_hours: 24,
            persist_to_disk: true,
            cache_file_path: Some("/tmp/pricing_cache.json".to_string()),
        }
    }
}

impl Default for PricingMonitoringConfig {
    fn default() -> Self {
        Self {
            enable_alerts: false,
            webhook_url: None,
            alert_emails: Vec::new(),
            metrics: MetricsConfig::default(),
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            prefix: "pricing_engine".to_string(),
            include_timing: true,
            include_errors: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pricing_config_validation() {
        let mut config = PricingEngineConfig::default();
        assert!(config.validate().is_ok());
        
        // Test invalid update interval
        config.update_interval_hours = 0;
        assert!(config.validate().is_err());
        
        // Reset and test invalid cache duration
        config = PricingEngineConfig::default();
        config.cache_duration_hours = 0;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_openai_config_validation() {
        let mut config = OpenAIConfig::default();
        config.enabled = false;
        assert!(config.validate().is_ok());
        
        // Should fail when enabled without API key
        config.enabled = true;
        assert!(config.validate().is_err());
        
        // Should pass when enabled with API key
        config.api_key = Some("test-key".to_string());
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_cache_config_validation() {
        let mut config = CacheConfig::default();
        assert!(config.validate().is_ok());
        
        config.max_size_mb = 0;
        assert!(config.validate().is_err());
        
        config = CacheConfig::default();
        config.ttl_hours = 0;
        assert!(config.validate().is_err());
    }
}