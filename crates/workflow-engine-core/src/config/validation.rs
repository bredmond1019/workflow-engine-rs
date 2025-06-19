//! Configuration validation utilities
//!
//! This module provides validation helpers for configuration values.

use crate::config::{ConfigError, ConfigResult};
use std::time::Duration;

/// Validate that a duration is within acceptable range
pub fn validate_duration(value: Duration, min: Duration, max: Duration, field_name: &str) -> ConfigResult<()> {
    if value < min {
        return Err(ConfigError::ValidationFailed(
            format!("{} must be at least {} seconds", field_name, min.as_secs())
        ));
    }
    
    if value > max {
        return Err(ConfigError::ValidationFailed(
            format!("{} must be at most {} seconds", field_name, max.as_secs())
        ));
    }
    
    Ok(())
}

/// Validate that a string is not empty
pub fn validate_non_empty_string(value: &str, field_name: &str) -> ConfigResult<()> {
    if value.trim().is_empty() {
        return Err(ConfigError::ValidationFailed(
            format!("{} cannot be empty", field_name)
        ));
    }
    
    Ok(())
}

/// Validate that a number is within range
pub fn validate_range<T: PartialOrd + std::fmt::Display>(value: T, min: T, max: T, field_name: &str) -> ConfigResult<()> {
    if value < min {
        return Err(ConfigError::ValidationFailed(
            format!("{} must be at least {}", field_name, min)
        ));
    }
    
    if value > max {
        return Err(ConfigError::ValidationFailed(
            format!("{} must be at most {}", field_name, max)
        ));
    }
    
    Ok(())
}

/// Validate URL format
pub fn validate_url(url: &str, field_name: &str) -> ConfigResult<()> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(ConfigError::ValidationFailed(
            format!("{} must be a valid HTTP(S) URL", field_name)
        ));
    }
    
    Ok(())
}

/// Validate email address format (basic validation)
pub fn validate_email(email: &str) -> ConfigResult<()> {
    if !email.contains('@') || email.split('@').count() != 2 {
        return Err(ConfigError::ValidationFailed(
            format!("Invalid email address: {}", email)
        ));
    }
    
    let parts: Vec<&str> = email.split('@').collect();
    if parts[0].is_empty() || parts[1].is_empty() {
        return Err(ConfigError::ValidationFailed(
            format!("Invalid email address: {}", email)
        ));
    }
    
    if !parts[1].contains('.') {
        return Err(ConfigError::ValidationFailed(
            format!("Invalid email domain: {}", parts[1])
        ));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_duration_validation() {
        let min = Duration::from_secs(1);
        let max = Duration::from_secs(100);
        
        assert!(validate_duration(Duration::from_secs(50), min, max, "test").is_ok());
        assert!(validate_duration(Duration::from_secs(0), min, max, "test").is_err());
        assert!(validate_duration(Duration::from_secs(101), min, max, "test").is_err());
    }
    
    #[test]
    fn test_string_validation() {
        assert!(validate_non_empty_string("test", "field").is_ok());
        assert!(validate_non_empty_string("", "field").is_err());
        assert!(validate_non_empty_string("   ", "field").is_err());
    }
    
    #[test]
    fn test_range_validation() {
        assert!(validate_range(5, 1, 10, "test").is_ok());
        assert!(validate_range(0, 1, 10, "test").is_err());
        assert!(validate_range(11, 1, 10, "test").is_err());
    }
    
    #[test]
    fn test_url_validation() {
        assert!(validate_url("https://example.com", "url").is_ok());
        assert!(validate_url("http://example.com", "url").is_ok());
        assert!(validate_url("ftp://example.com", "url").is_err());
        assert!(validate_url("example.com", "url").is_err());
    }
    
    #[test]
    fn test_email_validation() {
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("user@sub.domain.com").is_ok());
        assert!(validate_email("test").is_err());
        assert!(validate_email("test@").is_err());
        assert!(validate_email("@example.com").is_err());
        assert!(validate_email("test@example").is_err());
    }
}