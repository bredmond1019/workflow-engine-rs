//! Configuration validation utilities
//!
//! This module provides validation helpers for configuration values.

use crate::config::{ConfigError, ConfigResult};
use std::time::Duration;

/// Validate that a duration is within acceptable range
pub fn validate_duration(value: Duration, min: Duration, max: Duration, field_name: &str) -> ConfigResult<()> {
    if value < min {
        return Err(ConfigError::validation_failed(
            format!("{} must be at least {} seconds", field_name, min.as_secs()),
            "duration validation",
            "Increase the duration value",
            vec![(field_name.to_string(), format!("{} seconds", value.as_secs()))]
        ));
    }
    
    if value > max {
        return Err(ConfigError::validation_failed(
            format!("{} must be at most {} seconds", field_name, max.as_secs()),
            "duration validation",
            "Decrease the duration value",
            vec![(field_name.to_string(), format!("{} seconds", value.as_secs()))]
        ));
    }
    
    Ok(())
}

/// Validate that a string is not empty
pub fn validate_non_empty_string(value: &str, field_name: &str) -> ConfigResult<()> {
    if value.trim().is_empty() {
        return Err(ConfigError::validation_failed(
            format!("{} cannot be empty", field_name),
            "string validation",
            "Provide a non-empty value",
            vec![(field_name.to_string(), "empty".to_string())]
        ));
    }
    
    Ok(())
}

/// Validate that a number is within range
pub fn validate_range<T: PartialOrd + std::fmt::Display>(value: T, min: T, max: T, field_name: &str) -> ConfigResult<()> {
    if value < min {
        return Err(ConfigError::validation_failed(
            format!("{} must be at least {}", field_name, min),
            "range validation",
            "Increase the value",
            vec![(field_name.to_string(), value.to_string())]
        ));
    }
    
    if value > max {
        return Err(ConfigError::validation_failed(
            format!("{} must be at most {}", field_name, max),
            "range validation",
            "Decrease the value",
            vec![(field_name.to_string(), value.to_string())]
        ));
    }
    
    Ok(())
}

/// Validate URL format
pub fn validate_url(url: &str, field_name: &str) -> ConfigResult<()> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(ConfigError::validation_failed(
            format!("{} must be a valid HTTP(S) URL", field_name),
            "URL validation",
            "Ensure the URL starts with http:// or https://",
            vec![(field_name.to_string(), url.to_string())]
        ));
    }
    
    Ok(())
}

/// Validate email address format (basic validation)
pub fn validate_email(email: &str) -> ConfigResult<()> {
    if !email.contains('@') || email.split('@').count() != 2 {
        return Err(ConfigError::validation_failed(
            format!("Invalid email address: {}", email),
            "email validation",
            "Provide a valid email with @ symbol and proper format",
            vec![("email".to_string(), email.to_string())]
        ));
    }
    
    let parts: Vec<&str> = email.split('@').collect();
    if parts[0].is_empty() || parts[1].is_empty() {
        return Err(ConfigError::validation_failed(
            format!("Invalid email address: {}", email),
            "email validation",
            "Ensure both local and domain parts are present",
            vec![("email".to_string(), email.to_string())]
        ));
    }
    
    if !parts[1].contains('.') {
        return Err(ConfigError::validation_failed(
            format!("Invalid email domain: {}", parts[1]),
            "email validation",
            "Ensure the domain part contains a dot",
            vec![("domain".to_string(), parts[1].to_string())]
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