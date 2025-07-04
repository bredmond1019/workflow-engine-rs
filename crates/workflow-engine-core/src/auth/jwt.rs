use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::env;
use thiserror::Error;

/// JWT secret key loaded from environment
static JWT_SECRET: Lazy<String> = Lazy::new(|| {
    env::var("JWT_SECRET").unwrap_or_else(|_| "dev_secret_change_in_production".to_string())
});

/// JWT validation errors
#[derive(Error, Debug)]
pub enum JwtError {
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    
    #[error("Token expired")]
    TokenExpired,
    
    #[error("Token validation failed: {0}")]
    ValidationFailed(#[from] jsonwebtoken::errors::Error),
}

/// Minimal claims structure for JWT tokens
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Expiration time (as UTC timestamp)
    pub exp: usize,
    /// Simple role: admin|developer|service
    pub role: String,
    /// Issued at time (as UTC timestamp)
    pub iat: usize,
}

impl Claims {
    /// Create new claims with default expiration (24 hours)
    pub fn new(sub: String, role: String) -> Self {
        let now = Utc::now();
        let exp = now + Duration::hours(24);
        
        Self {
            sub,
            role,
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
        }
    }
    
    /// Create new claims with custom expiration
    pub fn with_expiration(sub: String, role: String, expires_at: DateTime<Utc>) -> Self {
        let now = Utc::now();
        
        Self {
            sub,
            role,
            exp: expires_at.timestamp() as usize,
            iat: now.timestamp() as usize,
        }
    }
}

/// Simple JWT middleware for Actix-web
pub struct JwtAuth {
    secret: String,
}

impl JwtAuth {
    /// Create a new JwtAuth instance with the provided secret
    pub fn new(secret: String) -> Self {
        if secret.is_empty() {
            panic!("JWT secret cannot be empty");
        }
        Self { secret }
    }
    /// Validate a JWT token and return the claims
    pub fn validate_token(&self, token: &str) -> Result<Claims, JwtError> {
        // Start with symmetric key (HS256)
        // Move to RS256 with Auth0 in Phase 1.5
        let decoding_key = DecodingKey::from_secret(self.secret.as_ref());
        let validation = Validation::new(Algorithm::HS256);
        
        match decode::<Claims>(token, &decoding_key, &validation) {
            Ok(token_data) => Ok(token_data.claims),
            Err(e) => {
                match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                        Err(JwtError::TokenExpired)
                    },
                    _ => Err(JwtError::ValidationFailed(e))
                }
            }
        }
    }
    
    /// Generate a new JWT token for the given claims
    pub fn generate_token(&self, claims: &Claims) -> Result<String, JwtError> {
        let encoding_key = EncodingKey::from_secret(self.secret.as_ref());
        let header = Header::new(Algorithm::HS256);
        
        encode(&header, claims, &encoding_key).map_err(JwtError::ValidationFailed)
    }
    
    /// Extract bearer token from Authorization header
    pub fn extract_bearer_token(auth_header: &str) -> Option<&str> {
        if auth_header.starts_with("Bearer ") {
            Some(&auth_header[7..])
        } else {
            None
        }
    }
    
    /// Create a default instance with secret from environment
    pub fn default() -> Self {
        Self::new(JWT_SECRET.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_claims() {
        let claims = Claims::new("user123".to_string(), "developer".to_string());
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.role, "developer");
        assert!(claims.exp > claims.iat);
    }
    
    #[test]
    fn test_generate_and_validate_token() {
        let auth = JwtAuth::new("test_secret".to_string());
        let claims = Claims::new("user123".to_string(), "admin".to_string());
        
        // Generate token
        let token = auth.generate_token(&claims).expect("Failed to generate token");
        assert!(!token.is_empty());
        
        // Validate token
        let decoded_claims = auth.validate_token(&token).expect("Failed to validate token");
        assert_eq!(decoded_claims.sub, "user123");
        assert_eq!(decoded_claims.role, "admin");
    }
    
    #[test]
    fn test_expired_token() {
        let auth = JwtAuth::new("test_secret".to_string());
        let past_time = Utc::now() - Duration::hours(1);
        let claims = Claims::with_expiration(
            "user123".to_string(), 
            "developer".to_string(),
            past_time
        );
        
        let token = auth.generate_token(&claims).expect("Failed to generate token");
        let result = auth.validate_token(&token);
        
        assert!(matches!(result, Err(JwtError::TokenExpired)));
    }
    
    #[test]
    fn test_invalid_token() {
        let auth = JwtAuth::new("test_secret".to_string());
        let result = auth.validate_token("invalid.token.here");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_extract_bearer_token() {
        assert_eq!(
            JwtAuth::extract_bearer_token("Bearer eyJhbGciOiJIUzI1NiJ9"),
            Some("eyJhbGciOiJIUzI1NiJ9")
        );
        
        assert_eq!(
            JwtAuth::extract_bearer_token("eyJhbGciOiJIUzI1NiJ9"),
            None
        );
        
        assert_eq!(
            JwtAuth::extract_bearer_token("Basic dXNlcjpwYXNz"),
            None
        );
    }
    
    #[test]
    fn test_new_with_valid_secret() {
        let auth = JwtAuth::new("valid_secret".to_string());
        // Test that it can generate tokens
        let claims = Claims::new("user123".to_string(), "admin".to_string());
        let token = auth.generate_token(&claims);
        assert!(token.is_ok());
    }
    
    #[test]
    #[should_panic(expected = "JWT secret cannot be empty")]
    fn test_new_with_empty_secret_panics() {
        JwtAuth::new("".to_string());
    }
    
    #[test]
    fn test_default_constructor() {
        let auth = JwtAuth::default();
        // Test that it can generate tokens with default secret
        let claims = Claims::new("user123".to_string(), "admin".to_string());
        let token = auth.generate_token(&claims);
        assert!(token.is_ok());
    }
}