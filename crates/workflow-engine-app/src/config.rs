//! Configuration module for the workflow engine application
//! 
//! This module handles application configuration including JWT setup,
//! database configuration, and server settings. It replaces unsafe
//! .expect() calls with proper error handling.

use std::env;
use std::sync::Arc;
use actix_web::web;
use workflow_engine_core::auth::JwtAuth;
use workflow_engine_api::db::session::DbPool;

/// Configuration errors that can occur during application startup
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {var_name}")]
    MissingEnvVar { var_name: String },
    
    #[error("Invalid environment variable value for {var_name}: {value}")]
    InvalidEnvVar { var_name: String, value: String },
    
    #[error("JWT secret is too weak (minimum 32 characters required)")]
    WeakJwtSecret,
    
    #[error("Database initialization failed: {source}")]
    DatabaseError { source: Box<dyn std::error::Error + Send + Sync> },
    
    #[error("Configuration validation failed: {message}")]
    ValidationError { message: String },
}

/// Application configuration
pub struct AppConfig {
    pub host: String,
    pub port: String,
    pub jwt_auth: web::Data<JwtAuth>,
    pub database_pool: Arc<DbPool>,
    pub rate_limit_per_minute: u32,
    pub rate_limit_burst: u32,
}

impl AppConfig {
    /// Initialize application configuration with proper error handling
    /// 
    /// This replaces the unsafe .expect() calls in main.rs with proper
    /// error handling that can be tested.
    pub async fn new() -> Result<Self, ConfigError> {
        // Load JWT secret with validation
        let jwt_secret = Self::load_jwt_secret()?;
        let jwt_auth = web::Data::new(JwtAuth::new(jwt_secret));
        
        // Load server configuration
        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
        
        // Load rate limiting configuration
        let rate_limit_per_minute = Self::parse_env_var("RATE_LIMIT_PER_MINUTE", 60)?;
        let rate_limit_burst = Self::parse_env_var("RATE_LIMIT_BURST", 10)?;
        
        // Initialize database pool
        let database_pool = Self::init_database_pool().await?;
        
        Ok(AppConfig {
            host,
            port,
            jwt_auth,
            database_pool,
            rate_limit_per_minute,
            rate_limit_burst,
        })
    }
    
    /// Load and validate JWT secret
    fn load_jwt_secret() -> Result<String, ConfigError> {
        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| ConfigError::MissingEnvVar { 
                var_name: "JWT_SECRET".to_string() 
            })?;
        
        // Validate JWT secret strength
        Self::validate_jwt_secret(&jwt_secret)?;
        
        Ok(jwt_secret)
    }
    
    /// Validate JWT secret meets security requirements
    fn validate_jwt_secret(secret: &str) -> Result<(), ConfigError> {
        if secret.is_empty() {
            return Err(ConfigError::InvalidEnvVar {
                var_name: "JWT_SECRET".to_string(),
                value: "(empty)".to_string(),
            });
        }
        
        if secret.len() < 32 {
            return Err(ConfigError::WeakJwtSecret);
        }
        
        Ok(())
    }
    
    /// Parse environment variable with default value and validation
    fn parse_env_var(var_name: &str, default_value: u32) -> Result<u32, ConfigError> {
        let value_str = env::var(var_name)
            .unwrap_or_else(|_| default_value.to_string());
        
        value_str.parse().map_err(|_| ConfigError::InvalidEnvVar {
            var_name: var_name.to_string(),
            value: value_str,
        })
    }
    
    /// Initialize database pool with proper error handling
    async fn init_database_pool() -> Result<Arc<DbPool>, ConfigError> {
        let pool = workflow_engine_api::db::session::init_pool()
            .map_err(|e| ConfigError::DatabaseError { 
                source: Box::new(e) 
            })?;
        
        Ok(Arc::new(pool))
    }
    
    /// Get server bind address
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    #[test]
    fn test_validate_jwt_secret_empty_fails() {
        // RED: Test that empty JWT secret is rejected
        let result = AppConfig::validate_jwt_secret("");
        assert!(result.is_err());
        
        if let Err(ConfigError::InvalidEnvVar { var_name, value }) = result {
            assert_eq!(var_name, "JWT_SECRET");
            assert_eq!(value, "(empty)");
        } else {
            panic!("Expected InvalidEnvVar error for empty JWT secret");
        }
    }
    
    #[test]
    fn test_validate_jwt_secret_too_short_fails() {
        // RED: Test that short JWT secret is rejected
        let result = AppConfig::validate_jwt_secret("short");
        assert!(result.is_err());
        
        if let Err(ConfigError::WeakJwtSecret) = result {
            // Expected
        } else {
            panic!("Expected WeakJwtSecret error for short JWT secret");
        }
    }
    
    #[test]
    fn test_validate_jwt_secret_valid_succeeds() {
        // Test that valid JWT secret is accepted
        let valid_secret = "this-is-a-valid-jwt-secret-with-32-plus-characters";
        let result = AppConfig::validate_jwt_secret(valid_secret);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_load_jwt_secret_missing_env_var() {
        // RED: Test that missing JWT_SECRET environment variable is handled
        env::remove_var("JWT_SECRET");
        
        let result = AppConfig::load_jwt_secret();
        assert!(result.is_err());
        
        if let Err(ConfigError::MissingEnvVar { var_name }) = result {
            assert_eq!(var_name, "JWT_SECRET");
        } else {
            panic!("Expected MissingEnvVar error when JWT_SECRET is not set");
        }
    }
    
    #[test]
    fn test_load_jwt_secret_valid_succeeds() {
        // Test that valid JWT_SECRET environment variable works
        let valid_secret = "this-is-a-valid-jwt-secret-with-32-plus-characters";
        env::set_var("JWT_SECRET", valid_secret);
        
        let result = AppConfig::load_jwt_secret();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), valid_secret);
        
        // Clean up
        env::remove_var("JWT_SECRET");
    }
    
    #[test]
    fn test_parse_env_var_with_default() {
        // Test parsing environment variable with default fallback
        let test_var = "TEST_VAR_DEFAULT";
        env::remove_var(test_var);
        
        let result = AppConfig::parse_env_var(test_var, 42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }
    
    #[test]
    fn test_parse_env_var_invalid_value() {
        // RED: Test that invalid environment variable values are handled
        // Use a unique variable name to avoid race conditions between tests
        let test_var = "TEST_VAR_INVALID";
        env::set_var(test_var, "not-a-number");
        
        let result = AppConfig::parse_env_var(test_var, 42);
        assert!(result.is_err());
        
        if let Err(ConfigError::InvalidEnvVar { var_name, value }) = result {
            assert_eq!(var_name, test_var);
            assert_eq!(value, "not-a-number");
        } else {
            panic!("Expected InvalidEnvVar error for invalid numeric value");
        }
        
        // Clean up
        env::remove_var(test_var);
    }
}