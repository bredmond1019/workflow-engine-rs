//! Configuration management for the workflow engine
//!
//! This module provides comprehensive configuration management for all aspects
//! of the workflow engine, including pricing, authentication, and API settings.

pub mod error;
pub mod pricing;
pub mod env_utils;
pub mod validation;

// Re-export commonly used types
pub use error::{ConfigError, ConfigResult};
pub use pricing::PricingEngineConfig;

use std::env;
use serde::{Deserialize, Serialize};

/// Main configuration structure for the workflow engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    pub pricing: pricing::PricingEngineConfig,
    pub api: ApiConfig,
    pub monitoring: MonitoringConfig,
}

/// API configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub jwt_secret: String,
    pub rate_limit_per_minute: u32,
    pub rate_limit_burst: u32,
}

/// Monitoring and observability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub prometheus_enabled: bool,
    pub prometheus_port: u16,
    pub log_level: String,
    pub jaeger_endpoint: Option<String>,
}


impl WorkflowConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> ConfigResult<Self> {
        Ok(Self {
            pricing: pricing::PricingEngineConfig::from_env()?,
            api: ApiConfig::from_env()?,
            monitoring: MonitoringConfig::from_env()?,
        })
    }
    
    /// Validate the complete configuration
    pub fn validate(&self) -> ConfigResult<()> {
        self.pricing.validate()?;
        self.api.validate()?;
        self.monitoring.validate()?;
        Ok(())
    }
}

impl ApiConfig {
    fn from_env() -> ConfigResult<Self> {
        Ok(Self {
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .map_err(|e| ConfigError::ParseError(format!("PORT: {}", e)))?,
            jwt_secret: env::var("JWT_SECRET")
                .map_err(|_| ConfigError::EnvVarNotFound("JWT_SECRET".to_string()))?,
            rate_limit_per_minute: env::var("RATE_LIMIT_PER_MINUTE")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .map_err(|e| ConfigError::ParseError(format!("RATE_LIMIT_PER_MINUTE: {}", e)))?,
            rate_limit_burst: env::var("RATE_LIMIT_BURST")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .map_err(|e| ConfigError::parse_error(
                    format!("RATE_LIMIT_BURST: {}", e),
                    "environment",
                    "RATE_LIMIT_BURST"
                ))?,
        })
    }
    
    fn validate(&self) -> ConfigResult<()> {
        if self.jwt_secret.len() < 32 {
            return Err(ConfigError::validation_failed(
                "JWT_SECRET must be at least 32 characters long",
                "security",
                "Use a longer secret key",
                vec![("jwt_secret".to_string(), format!("length: {}", self.jwt_secret.len()))]
            ));
        }
        
        if self.port == 0 {
            return Err(ConfigError::validation_failed(
                "PORT must be greater than 0",
                "server",
                "Use a valid port number",
                vec![("port".to_string(), self.port.to_string())]
            ));
        }
        
        Ok(())
    }
}

impl MonitoringConfig {
    fn from_env() -> ConfigResult<Self> {
        Ok(Self {
            prometheus_enabled: env::var("PROMETHEUS_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            prometheus_port: env::var("PROMETHEUS_PORT")
                .unwrap_or_else(|_| "9090".to_string())
                .parse()
                .map_err(|e| ConfigError::parse_error(
                    format!("PROMETHEUS_PORT: {}", e),
                    "environment",
                    "PROMETHEUS_PORT"
                ))?,
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            jaeger_endpoint: env::var("JAEGER_ENDPOINT").ok(),
        })
    }
    
    fn validate(&self) -> ConfigResult<()> {
        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(ConfigError::validation_failed(
                format!("Invalid log level: {}. Must be one of: {}", 
                       self.log_level, 
                       valid_log_levels.join(", ")),
                "logging",
                "Use one of the supported log levels",
                vec![("log_level".to_string(), self.log_level.clone())]
            ));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    #[test]
    fn test_api_config_validation() {
        let valid_config = ApiConfig {
            host: "localhost".to_string(),
            port: 8080,
            jwt_secret: "a".repeat(32), // 32 characters
            rate_limit_per_minute: 60,
            rate_limit_burst: 10,
        };
        
        assert!(valid_config.validate().is_ok());
        
        let invalid_config = ApiConfig {
            host: "localhost".to_string(),
            port: 8080,
            jwt_secret: "short".to_string(), // Too short
            rate_limit_per_minute: 60,
            rate_limit_burst: 10,
        };
        
        assert!(invalid_config.validate().is_err());
    }
    
    #[test]
    fn test_monitoring_config_validation() {
        let valid_config = MonitoringConfig {
            prometheus_enabled: true,
            prometheus_port: 9090,
            log_level: "info".to_string(),
            jaeger_endpoint: None,
        };
        
        assert!(valid_config.validate().is_ok());
        
        let invalid_config = MonitoringConfig {
            prometheus_enabled: true,
            prometheus_port: 9090,
            log_level: "invalid".to_string(),
            jaeger_endpoint: None,
        };
        
        assert!(invalid_config.validate().is_err());
    }
}