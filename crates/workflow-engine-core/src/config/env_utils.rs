//! Environment configuration utilities
//!
//! This module provides utilities for loading and validating environment variables
//! with type conversion and default value handling.

use std::env;
use std::str::FromStr;
use crate::config::{ConfigError, ConfigResult};

/// Environment variable loader with type conversion and validation
pub struct EnvLoader;

impl EnvLoader {
    /// Load a required environment variable
    pub fn load_required<T>(key: &str) -> ConfigResult<T>
    where
        T: FromStr,
        T::Err: std::fmt::Display,
    {
        let value = env::var(key)
            .map_err(|_| ConfigError::env_var_not_found(key, None))?;
        
        value.parse()
            .map_err(|e| ConfigError::parse_error(
                format!("{}: {}", key, e),
                "environment variable",
                key
            ))
    }
    
    /// Load an optional environment variable with a default value
    pub fn load_with_default<T>(key: &str, default: T) -> ConfigResult<T>
    where
        T: FromStr + Clone,
        T::Err: std::fmt::Display,
    {
        match env::var(key) {
            Ok(value) => value.parse()
                .map_err(|e| ConfigError::parse_error(
                    format!("{}: {}", key, e),
                    "environment variable",
                    key
                )),
            Err(_) => Ok(default),
        }
    }
    
    /// Load an optional environment variable
    pub fn load_optional<T>(key: &str) -> ConfigResult<Option<T>>
    where
        T: FromStr,
        T::Err: std::fmt::Display,
    {
        match env::var(key) {
            Ok(value) => {
                let parsed = value.parse()
                    .map_err(|e| ConfigError::parse_error(
                        format!("{}: {}", key, e),
                        "environment variable",
                        key
                    ))?;
                Ok(Some(parsed))
            },
            Err(_) => Ok(None),
        }
    }
    
    /// Load a boolean environment variable with string variations
    pub fn load_bool(key: &str, default: bool) -> bool {
        match env::var(key).as_deref() {
            Ok("true" | "True" | "TRUE" | "1" | "yes" | "Yes" | "YES" | "on" | "On" | "ON") => true,
            Ok("false" | "False" | "FALSE" | "0" | "no" | "No" | "NO" | "off" | "Off" | "OFF") => false,
            _ => default,
        }
    }
    
    /// Load a comma-separated list of values
    pub fn load_list<T>(key: &str) -> ConfigResult<Vec<T>>
    where
        T: FromStr,
        T::Err: std::fmt::Display,
    {
        match env::var(key) {
            Ok(value) => {
                if value.trim().is_empty() {
                    return Ok(Vec::new());
                }
                
                value.split(',')
                    .map(|s| s.trim().parse()
                        .map_err(|e| ConfigError::parse_error(
                            format!("{}: {}", key, e),
                            "environment variable",
                            key
                        )))
                    .collect()
            },
            Err(_) => Ok(Vec::new()),
        }
    }
    
    /// Load a duration in seconds
    pub fn load_duration_seconds(key: &str, default_seconds: u64) -> ConfigResult<std::time::Duration> {
        let seconds = Self::load_with_default(key, default_seconds)?;
        Ok(std::time::Duration::from_secs(seconds))
    }
    
    /// Validate that a required environment variable is set (without parsing)
    pub fn validate_present(key: &str) -> ConfigResult<()> {
        env::var(key)
            .map(|_| ())
            .map_err(|_| ConfigError::env_var_not_found(key, None))
    }
    
    /// Validate an environment variable against a set of allowed values
    pub fn validate_enum(key: &str, allowed_values: &[&str]) -> ConfigResult<()> {
        match env::var(key) {
            Ok(value) => {
                if allowed_values.contains(&value.as_str()) {
                    Ok(())
                } else {
                    Err(ConfigError::invalid_value(
                        key,
                        &value,
                        &format!("one of: {}", allowed_values.join(", ")),
                        "environment variable"
                    ))
                }
            },
            Err(_) => Ok(()), // Optional validation - OK if not present
        }
    }
    
    /// Load an environment variable with validation
    pub fn load_with_validation<T, F>(
        key: &str,
        default: T,
        validator: F,
    ) -> ConfigResult<T>
    where
        T: FromStr + Clone,
        T::Err: std::fmt::Display,
        F: Fn(&T) -> bool,
    {
        let value = Self::load_with_default(key, default)?;
        
        if validator(&value) {
            Ok(value)
        } else {
            Err(ConfigError::validation_failed(
                format!("Value for {} failed validation", key),
                "environment variable",
                "Ensure the value meets the required validation constraints",
                vec![(key.to_string(), "validation failed".to_string())]
            ))
        }
    }
}

/// Environment variable validator
pub struct EnvValidator;

impl EnvValidator {
    /// Validate all required environment variables are present
    pub fn validate_required_vars(required_vars: &[&str]) -> ConfigResult<()> {
        for var in required_vars {
            EnvLoader::validate_present(var)?;
        }
        Ok(())
    }
    
    /// Validate pricing-specific environment variables
    pub fn validate_pricing_vars() -> ConfigResult<()> {
        // Validate log level if present
        EnvLoader::validate_enum("LOG_LEVEL", &["trace", "debug", "info", "warn", "error"])?;
        
        // Validate cache backend if present
        EnvLoader::validate_enum("PRICING_CACHE_BACKEND", &["memory", "redis", "file"])?;
        
        // Validate boolean environment variables by attempting to parse them
        Self::validate_bool_var("PRICING_AUTO_UPDATE")?;
        Self::validate_bool_var("PRICING_FALLBACK_ENABLED")?;
        Self::validate_bool_var("OPENAI_PRICING_ENABLED")?;
        Self::validate_bool_var("ANTHROPIC_PRICING_ENABLED")?;
        Self::validate_bool_var("AWS_PRICING_ENABLED")?;
        
        // Validate numeric variables
        Self::validate_numeric_var::<u64>("PRICING_UPDATE_INTERVAL_HOURS")?;
        Self::validate_numeric_var::<u64>("PRICING_CACHE_DURATION_HOURS")?;
        Self::validate_numeric_var::<u64>("PRICING_API_TIMEOUT_SECONDS")?;
        Self::validate_numeric_var::<u32>("PRICING_RETRY_ATTEMPTS")?;
        Self::validate_numeric_var::<u64>("PRICING_RETRY_DELAY_SECONDS")?;
        
        Ok(())
    }
    
    /// Validate a boolean environment variable
    fn validate_bool_var(key: &str) -> ConfigResult<()> {
        if let Ok(value) = env::var(key) {
            match value.to_lowercase().as_str() {
                "true" | "false" | "1" | "0" | "yes" | "no" | "on" | "off" => Ok(()),
                _ => Err(ConfigError::invalid_value(
                    key,
                    &value,
                    "true/false, 1/0, yes/no, or on/off",
                    "environment variable"
                )),
            }
        } else {
            Ok(()) // Optional variable - OK if not present
        }
    }
    
    /// Validate a numeric environment variable
    fn validate_numeric_var<T>(key: &str) -> ConfigResult<()>
    where
        T: FromStr,
        T::Err: std::fmt::Display,
    {
        if let Ok(value) = env::var(key) {
            value.parse::<T>()
                .map(|_| ())
                .map_err(|e| ConfigError::parse_error(
                    format!("{}: {}", key, e),
                    "environment variable",
                    key
                ))
        } else {
            Ok(()) // Optional variable - OK if not present
        }
    }
    
    /// Validate URL format for API endpoints
    pub fn validate_url_var(key: &str) -> ConfigResult<()> {
        if let Ok(url) = env::var(key) {
            if url.starts_with("http://") || url.starts_with("https://") {
                Ok(())
            } else {
                Err(ConfigError::invalid_value(
                    key,
                    &url,
                    "URL starting with http:// or https://",
                    "environment variable"
                ))
            }
        } else {
            Ok(()) // Optional variable - OK if not present
        }
    }
    
    /// Validate email format for alert emails
    pub fn validate_email_list(key: &str) -> ConfigResult<()> {
        if let Ok(emails) = env::var(key) {
            if emails.trim().is_empty() {
                return Ok(());
            }
            
            for email in emails.split(',') {
                let email = email.trim();
                if !email.contains('@') || !email.contains('.') {
                    return Err(ConfigError::invalid_value(
                        key,
                        &emails,
                        "comma-separated list of valid email addresses",
                        "environment variable"
                    ));
                }
            }
        }
        Ok(())
    }
}

/// Configuration preset for different environments
#[derive(Debug, Clone)]
pub enum ConfigPreset {
    Development,
    Testing,
    Staging,
    Production,
}

impl ConfigPreset {
    /// Get current preset from environment
    pub fn from_env() -> Self {
        match env::var("ENVIRONMENT").as_deref() {
            Ok("development") | Ok("dev") => Self::Development,
            Ok("testing") | Ok("test") => Self::Testing,
            Ok("staging") | Ok("stage") => Self::Staging,
            Ok("production") | Ok("prod") => Self::Production,
            _ => Self::Development, // Default to development
        }
    }
    
    /// Apply preset-specific defaults
    pub fn apply_defaults(&self) {
        match self {
            Self::Development => {
                env::set_var("PRICING_AUTO_UPDATE", "false");
                env::set_var("PRICING_UPDATE_INTERVAL_HOURS", "24");
                env::set_var("LOG_LEVEL", "debug");
            },
            Self::Testing => {
                env::set_var("PRICING_AUTO_UPDATE", "false");
                env::set_var("PRICING_FALLBACK_ENABLED", "true");
                env::set_var("LOG_LEVEL", "warn");
            },
            Self::Staging => {
                env::set_var("PRICING_AUTO_UPDATE", "true");
                env::set_var("PRICING_UPDATE_INTERVAL_HOURS", "12");
                env::set_var("LOG_LEVEL", "info");
            },
            Self::Production => {
                env::set_var("PRICING_AUTO_UPDATE", "true");
                env::set_var("PRICING_UPDATE_INTERVAL_HOURS", "6");
                env::set_var("PRICING_ENABLE_ALERTS", "true");
                env::set_var("LOG_LEVEL", "info");
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    #[test]
    fn test_env_loader_required() {
        env::set_var("TEST_REQUIRED", "42");
        let result: ConfigResult<i32> = EnvLoader::load_required("TEST_REQUIRED");
        assert_eq!(result.unwrap(), 42);
        
        env::remove_var("TEST_REQUIRED");
        let result: ConfigResult<i32> = EnvLoader::load_required("TEST_REQUIRED");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_env_loader_with_default() {
        env::set_var("TEST_DEFAULT", "100");
        let result: ConfigResult<i32> = EnvLoader::load_with_default("TEST_DEFAULT", 50);
        assert_eq!(result.unwrap(), 100);
        
        env::remove_var("TEST_DEFAULT");
        let result: ConfigResult<i32> = EnvLoader::load_with_default("TEST_DEFAULT", 50);
        assert_eq!(result.unwrap(), 50);
    }
    
    #[test]
    fn test_env_loader_bool() {
        env::set_var("TEST_BOOL_TRUE", "true");
        assert_eq!(EnvLoader::load_bool("TEST_BOOL_TRUE", false), true);
        
        env::set_var("TEST_BOOL_FALSE", "false");
        assert_eq!(EnvLoader::load_bool("TEST_BOOL_FALSE", true), false);
        
        env::set_var("TEST_BOOL_ONE", "1");
        assert_eq!(EnvLoader::load_bool("TEST_BOOL_ONE", false), true);
        
        env::remove_var("TEST_BOOL_MISSING");
        assert_eq!(EnvLoader::load_bool("TEST_BOOL_MISSING", true), true);
    }
    
    #[test]
    fn test_env_loader_list() {
        env::set_var("TEST_LIST", "a,b,c");
        let result: ConfigResult<Vec<String>> = EnvLoader::load_list("TEST_LIST");
        assert_eq!(result.unwrap(), vec!["a", "b", "c"]);
        
        env::set_var("TEST_LIST_EMPTY", "");
        let result: ConfigResult<Vec<String>> = EnvLoader::load_list("TEST_LIST_EMPTY");
        assert!(result.unwrap().is_empty());
        
        env::remove_var("TEST_LIST_MISSING");
        let result: ConfigResult<Vec<String>> = EnvLoader::load_list("TEST_LIST_MISSING");
        assert!(result.unwrap().is_empty());
    }
    
    #[test]
    fn test_env_validator_enum() {
        env::set_var("TEST_ENUM", "valid");
        assert!(EnvValidator::validate_enum("TEST_ENUM", &["valid", "also_valid"]).is_ok());
        
        env::set_var("TEST_ENUM", "invalid");
        assert!(EnvValidator::validate_enum("TEST_ENUM", &["valid", "also_valid"]).is_err());
        
        env::remove_var("TEST_ENUM");
        assert!(EnvValidator::validate_enum("TEST_ENUM", &["valid", "also_valid"]).is_ok());
    }
    
    #[test]
    fn test_config_preset() {
        env::set_var("ENVIRONMENT", "production");
        let preset = ConfigPreset::from_env();
        matches!(preset, ConfigPreset::Production);
        
        env::set_var("ENVIRONMENT", "dev");
        let preset = ConfigPreset::from_env();
        matches!(preset, ConfigPreset::Development);
        
        env::remove_var("ENVIRONMENT");
        let preset = ConfigPreset::from_env();
        matches!(preset, ConfigPreset::Development);
    }
}