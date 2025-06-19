//! Configuration error types
//!
//! This module provides the error types for configuration management.

use thiserror::Error;

/// Configuration errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Environment variable not found: {0}")]
    EnvVarNotFound(String),
    
    #[error("Invalid configuration value for {key}: {value}")]
    InvalidValue { key: String, value: String },
    
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Parsing error: {0}")]
    ParseError(String),
    
    #[error("Required field missing: {0}")]
    RequiredFieldMissing(String),
}

/// Result type for configuration operations
pub type ConfigResult<T> = Result<T, ConfigError>;