use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::env;
use thiserror::Error;

/// Minimum length for a valid JWT token (approximate - a real JWT is much longer)
const MIN_JWT_TOKEN_LENGTH: usize = 10;

/// JWT secret key loaded from environment
static JWT_SECRET: Lazy<String> = Lazy::new(|| {
    env::var("JWT_SECRET").unwrap_or_else(|_| {
        // Only allow default in test mode
        if cfg!(test) {
            "test_secret_key_for_testing_only".to_string()
        } else {
            panic!("JWT_SECRET environment variable must be set")
        }
    })
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
    /// Returns None if:
    /// - Header doesn't start with "Bearer "
    /// - Token part is empty, only whitespace, or too short
    /// - Token contains characters that suggest it's not a valid JWT
    pub fn extract_bearer_token(auth_header: &str) -> Option<&str> {
        let token = auth_header.strip_prefix("Bearer ")?;
        
        // Reject empty or whitespace-only tokens
        let trimmed = token.trim();
        if trimmed.is_empty() {
            return None;
        }
        
        // Reject tokens that are too short to be valid JWTs
        if trimmed.len() < MIN_JWT_TOKEN_LENGTH {
            return None;
        }
        
        Some(token)
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

    // Test 3a: JWT Token Validation Edge Cases (TDD - RED phase)
    
    /// Test malformed JWT tokens - tokens with missing parts  
    #[test]
    fn test_malformed_jwt_missing_parts() {
        let auth = JwtAuth::new("test_secret".to_string());
        
        // Test cases that should fail with proper error messages
        let malformed_tokens = vec![
            "", // Empty token
            ".", // Single dot
            "..", // Two dots but no content
            "header", // Only one part
            "header.", // Header with dot but no payload
            "header.payload", // Missing signature
            ".payload.signature", // Missing header
            "header..signature", // Missing payload
            "....", // Multiple empty dots
        ];
        
        for token in malformed_tokens {
            let result = auth.validate_token(token);
            assert!(
                result.is_err(),
                "Token '{}' should be invalid but was accepted",
                token
            );
            
            // Should provide meaningful error message
            match result {
                Err(JwtError::ValidationFailed(_)) => {}, // Expected
                Err(JwtError::InvalidToken(_)) => {}, // Also acceptable
                _ => panic!("Expected ValidationFailed or InvalidToken error for malformed token: {}", token),
            }
        }
    }

    /// Test JWT tokens with invalid base64 encoding
    #[test]
    fn test_invalid_base64_encoding() {
        let auth = JwtAuth::new("test_secret".to_string());
        
        // Tokens with invalid base64 characters or structure
        let invalid_base64_tokens = vec![
            "invalid@base64.invalid@base64.invalid@base64", // Invalid chars
            "header!.payload!.signature!", // Invalid chars
            "header header.payload payload.signature signature", // Spaces
            "Zm9v.YmFy.YmF6", // Valid base64 but not valid JWT structure
            "!!!.???.***", // Special characters
        ];
        
        for token in invalid_base64_tokens {
            let result = auth.validate_token(token);
            assert!(
                result.is_err(),
                "Token with invalid base64 '{}' should be rejected",
                token
            );
        }
    }

    /// Test JWT tokens with tampered components
    #[test]
    fn test_tampered_jwt_components() {
        let auth = JwtAuth::new("test_secret".to_string());
        let claims = Claims::new("user123".to_string(), "admin".to_string());
        
        // Generate a valid token first
        let valid_token = auth.generate_token(&claims).expect("Failed to generate token");
        let parts: Vec<&str> = valid_token.split('.').collect();
        assert_eq!(parts.len(), 3, "Valid token should have 3 parts");
        
        // Test tampered header
        let tampered_header = format!("tamperedheader.{}.{}", parts[1], parts[2]);
        let result = auth.validate_token(&tampered_header);
        assert!(result.is_err(), "Tampered header should be rejected");
        
        // Test tampered payload 
        let tampered_payload = format!("{}.tamperedpayload.{}", parts[0], parts[2]);
        let result = auth.validate_token(&tampered_payload);
        assert!(result.is_err(), "Tampered payload should be rejected");
        
        // Test tampered signature
        let tampered_signature = format!("{}.{}.tamperedsignature", parts[0], parts[1]);
        let result = auth.validate_token(&tampered_signature);
        assert!(result.is_err(), "Tampered signature should be rejected");
    }

    /// Test expired tokens with various edge cases
    #[test]
    fn test_expired_token_edge_cases() {
        let auth = JwtAuth::new("test_secret".to_string());
        
        // Test token expired well in the past (1 hour ago - should definitely be rejected)
        let expired_1_hour = Utc::now() - Duration::hours(1);
        let claims = Claims::with_expiration("user123".to_string(), "admin".to_string(), expired_1_hour);
        let token = auth.generate_token(&claims).expect("Failed to generate token");
        let result = auth.validate_token(&token);
        assert!(matches!(result, Err(JwtError::TokenExpired)), "Token expired 1 hour ago should be rejected");
        
        // Test token that expires in the future (should be valid)  
        let expires_future = Utc::now() + Duration::hours(1);
        let claims = Claims::with_expiration("user123".to_string(), "admin".to_string(), expires_future);
        let token = auth.generate_token(&claims).expect("Failed to generate token");
        let result = auth.validate_token(&token);
        assert!(result.is_ok(), "Token expiring in 1 hour should be valid");
        
        // Test edge case: token expiring very soon (should still be valid until expired)
        let expires_very_soon = Utc::now() + Duration::milliseconds(100);
        let claims = Claims::with_expiration("user123".to_string(), "admin".to_string(), expires_very_soon);
        let token = auth.generate_token(&claims).expect("Failed to generate token");
        let result = auth.validate_token(&token);
        // This may pass or fail depending on execution timing - both are acceptable
        // The important thing is it doesn't panic or give wrong error types
        match result {
            Ok(_) => {}, // Still valid
            Err(JwtError::TokenExpired) => {}, // Just expired
            Err(JwtError::ValidationFailed(_)) => {}, // Also acceptable
            Err(e) => panic!("Unexpected error type for near-expired token: {:?}", e)
        }
    }

    /// Test bearer token extraction edge cases
    #[test]
    fn test_bearer_token_extraction_edge_cases() {
        let edge_cases = vec![
            ("", None), // Empty string
            ("Bearer", None), // Just "Bearer" without space or token
            ("Bearer ", None), // "Bearer " with space but no token
            ("Bearer  ", None), // "Bearer" with multiple spaces but no token
            ("bearer token", None), // Lowercase "bearer"
            ("BEARER token", None), // Uppercase "BEARER"  
            ("Basic dXNlcjpwYXNz", None), // Different auth scheme
            ("Bearer\ttoken", None), // Tab instead of space
            ("Bearer\ntoken", None), // Newline instead of space
            ("BearerToken", None), // No space between Bearer and token
            ("Bearer token extra", Some("token extra")), // Extra content after token
            ("Bearer ", None), // Bearer with just space
        ];
        
        for (input, expected) in edge_cases {
            let result = JwtAuth::extract_bearer_token(input);
            assert_eq!(
                result, expected,
                "Bearer extraction failed for input: '{}'",
                input.replace('\n', "\\n").replace('\t', "\\t")
            );
        }
    }

    /// Test null and edge case token values
    #[test]
    fn test_null_and_edge_case_tokens() {
        let auth = JwtAuth::new("test_secret".to_string());
        
        let edge_case_tokens = vec![
            "null", // String "null"
            "undefined", // String "undefined"
            "false", // String "false"
            "0", // String "0"
            " ", // Single space
            "\t", // Tab character
            "\n", // Newline
            "\r\n", // Windows newline
            "Bearer ", // Authorization header prefix without token
            "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9", // Valid header but incomplete
        ];
        
        for token in edge_case_tokens {
            let result = auth.validate_token(token);
            assert!(
                result.is_err(),
                "Edge case token '{}' should be rejected",
                token.replace('\n', "\\n").replace('\t', "\\t")
            );
        }
    }

    /// Test concurrent token validation (thread safety)
    #[test]
    fn test_concurrent_token_validation() {
        use std::sync::Arc;
        use std::thread;
        
        let auth = Arc::new(JwtAuth::new("test_secret".to_string()));
        let claims = Claims::new("user123".to_string(), "admin".to_string());
        let token = auth.generate_token(&claims).expect("Failed to generate token");
        
        let mut handles = vec![];
        
        // Spawn multiple threads to validate the same token
        for i in 0..5 {
            let auth_clone = Arc::clone(&auth);
            let token_clone = token.clone();
            
            let handle = thread::spawn(move || {
                for _ in 0..10 {
                    let result = auth_clone.validate_token(&token_clone);
                    assert!(
                        result.is_ok(),
                        "Token validation failed in thread {}: {:?}",
                        i,
                        result
                    );
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().expect("Thread panicked");
        }
    }

    /// Test memory safety with large tokens
    #[test]
    fn test_large_token_handling() {
        let auth = JwtAuth::new("test_secret".to_string());
        
        // Create a large token payload
        let large_sub = "a".repeat(1000); // 1KB subject
        let claims = Claims::new(large_sub, "admin".to_string());
        
        // Should handle large tokens gracefully
        let result = auth.generate_token(&claims);
        match result {
            Ok(token) => {
                // If generation succeeds, validation should also work
                let validation_result = auth.validate_token(&token);
                assert!(validation_result.is_ok(), "Large token validation should succeed");
            }
            Err(_) => {
                // It's acceptable to reject very large tokens
            }
        }
        
        // Test token that's unreasonably large
        let very_large_token = "a".repeat(10000);
        let result = auth.validate_token(&very_large_token);
        assert!(result.is_err(), "Extremely large token should be rejected");
    }
}