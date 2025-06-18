//! JWT Authentication Implementation
//! 
//! Provides JWT token validation, claims extraction, and token refresh
//! mechanisms for WebSocket authentication.

use super::{AuthError, AuthResult, SystemRole, UserContext};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::{debug, error, warn};
use uuid::Uuid;

/// JWT Claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    // Standard claims
    pub sub: String,    // Subject (user ID)
    pub iat: i64,       // Issued at
    pub exp: i64,       // Expiration time
    pub nbf: Option<i64>, // Not before
    pub iss: Option<String>, // Issuer
    pub aud: Option<String>, // Audience
    
    // Custom claims
    pub user_id: String,
    pub session_id: Option<String>,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub refresh_threshold: Option<i64>, // Time when refresh is recommended
}

impl Claims {
    /// Create new claims for a user
    pub fn new(
        user_id: String,
        roles: Vec<String>,
        permissions: Vec<String>,
        session_id: Option<String>,
        expires_in: Duration,
    ) -> Self {
        let now = Utc::now();
        let exp = now + expires_in;
        let refresh_threshold = now + (expires_in * 3 / 4); // Recommend refresh at 75% of lifetime

        Self {
            sub: user_id.clone(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
            nbf: Some(now.timestamp()),
            iss: Some("ai-system-rust".to_string()),
            aud: Some("realtime-communication".to_string()),
            user_id,
            session_id,
            roles,
            permissions,
            refresh_threshold: Some(refresh_threshold.timestamp()),
        }
    }

    /// Check if the token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }

    /// Check if the token should be refreshed
    pub fn should_refresh(&self) -> bool {
        if let Some(threshold) = self.refresh_threshold {
            Utc::now().timestamp() > threshold
        } else {
            false
        }
    }

    /// Convert to UserContext
    pub fn to_user_context(&self, connection_id: Uuid) -> UserContext {
        UserContext {
            user_id: self.user_id.clone(),
            connection_id,
            roles: self.roles.iter().cloned().collect(),
            permissions: self.permissions.iter().cloned().collect(),
            session_id: self.session_id.clone(),
            authenticated_at: DateTime::from_timestamp(self.iat, 0)
                .unwrap_or_else(|| Utc::now()),
            expires_at: Some(DateTime::from_timestamp(self.exp, 0)
                .unwrap_or_else(|| Utc::now())),
        }
    }
}

/// JWT Token validator and generator
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
    algorithm: Algorithm,
}

impl JwtService {
    /// Create a new JWT service with a secret key
    pub fn new(secret: &[u8]) -> Result<Self, AuthError> {
        let algorithm = Algorithm::HS256;
        let encoding_key = EncodingKey::from_secret(secret);
        let decoding_key = DecodingKey::from_secret(secret);
        
        let mut validation = Validation::new(algorithm);
        validation.set_issuer(&["ai-system-rust"]);
        validation.set_audience(&["realtime-communication"]);
        validation.validate_nbf = true;

        Ok(Self {
            encoding_key,
            decoding_key,
            validation,
            algorithm,
        })
    }

    /// Generate a JWT token for a user
    pub fn generate_token(
        &self,
        user_id: String,
        roles: Vec<SystemRole>,
        session_id: Option<String>,
        expires_in: Option<Duration>,
    ) -> Result<String, AuthError> {
        let duration = expires_in.unwrap_or_else(|| Duration::hours(24));
        
        // Convert roles to strings and get permissions
        let role_strings: Vec<String> = roles.iter().map(|r| r.as_str().to_string()).collect();
        let mut permissions = HashSet::new();
        
        for role in &roles {
            permissions.extend(role.default_permissions());
        }
        
        let claims = Claims::new(
            user_id,
            role_strings,
            permissions.into_iter().collect(),
            session_id,
            duration,
        );

        let header = Header::new(self.algorithm);
        
        encode(&header, &claims, &self.encoding_key)
            .map_err(|e| {
                error!("Failed to encode JWT token: {}", e);
                AuthError::InvalidToken(format!("Encoding failed: {}", e))
            })
    }

    /// Validate and decode a JWT token
    pub fn validate_token(&self, token: &str) -> AuthResult {
        match decode::<Claims>(token, &self.decoding_key, &self.validation) {
            Ok(token_data) => {
                let claims = token_data.claims;
                
                // Check if token is expired
                if claims.is_expired() {
                    warn!("Token expired for user: {}", claims.user_id);
                    return AuthResult::Unauthenticated(AuthError::ExpiredToken);
                }

                // Check if token should be refreshed
                if claims.should_refresh() {
                    debug!("Token should be refreshed for user: {}", claims.user_id);
                    return AuthResult::RequiresRefresh(claims.user_id.clone());
                }

                debug!("Token validated successfully for user: {}", claims.user_id);
                
                // Create temporary UUID for validation - will be replaced with actual connection ID
                let temp_connection_id = Uuid::new_v4();
                let user_context = claims.to_user_context(temp_connection_id);
                
                AuthResult::Authenticated(user_context)
            }
            Err(e) => {
                error!("Token validation failed: {}", e);
                
                let auth_error = match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::ExpiredToken,
                    jsonwebtoken::errors::ErrorKind::InvalidSignature => AuthError::SignatureValidationFailed,
                    jsonwebtoken::errors::ErrorKind::InvalidToken => {
                        AuthError::InvalidToken("Malformed token".to_string())
                    }
                    _ => AuthError::InvalidToken(format!("Validation error: {}", e)),
                };
                
                AuthResult::Unauthenticated(auth_error)
            }
        }
    }

    /// Extract token from Authorization header
    pub fn extract_token_from_header(&self, auth_header: &str) -> Result<String, AuthError> {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            Ok(token.to_string())
        } else {
            Err(AuthError::InvalidToken(
                "Authorization header must start with 'Bearer '".to_string(),
            ))
        }
    }

    /// Extract token from query parameters
    pub fn extract_token_from_query(&self, query: &str) -> Result<String, AuthError> {
        let params: std::collections::HashMap<String, String> = url::form_urlencoded::parse(query.as_bytes())
            .into_owned()
            .collect();

        params
            .get("token")
            .or_else(|| params.get("access_token"))
            .cloned()
            .ok_or(AuthError::MissingToken)
    }

    /// Generate a refresh token (longer-lived, used only for token refresh)
    pub fn generate_refresh_token(
        &self,
        user_id: String,
        session_id: String,
    ) -> Result<String, AuthError> {
        let duration = Duration::days(30); // Refresh tokens last longer
        
        let claims = Claims::new(
            user_id,
            vec!["refresh".to_string()], // Special role for refresh tokens
            vec!["token_refresh".to_string()],
            Some(session_id),
            duration,
        );

        let header = Header::new(self.algorithm);
        
        encode(&header, &claims, &self.encoding_key)
            .map_err(|e| {
                error!("Failed to encode refresh token: {}", e);
                AuthError::InvalidToken(format!("Refresh token encoding failed: {}", e))
            })
    }

    /// Validate a refresh token and extract user info
    pub fn validate_refresh_token(&self, token: &str) -> Result<(String, Option<String>), AuthError> {
        match decode::<Claims>(token, &self.decoding_key, &self.validation) {
            Ok(token_data) => {
                let claims = token_data.claims;
                
                if claims.is_expired() {
                    return Err(AuthError::ExpiredToken);
                }

                // Check if this is actually a refresh token
                if !claims.roles.contains(&"refresh".to_string()) {
                    return Err(AuthError::InvalidToken(
                        "Not a refresh token".to_string(),
                    ));
                }

                Ok((claims.user_id, claims.session_id))
            }
            Err(e) => {
                error!("Refresh token validation failed: {}", e);
                Err(AuthError::InvalidToken(format!("Refresh validation error: {}", e)))
            }
        }
    }

    /// Create claims for a service account
    pub fn create_service_claims(
        &self,
        service_name: String,
        permissions: Vec<String>,
    ) -> Claims {
        Claims::new(
            format!("service:{}", service_name),
            vec![SystemRole::Service.as_str().to_string()],
            permissions,
            None,
            Duration::hours(1), // Service tokens are short-lived
        )
    }
}

/// JWT Token extractor for various sources
pub struct TokenExtractor;

impl TokenExtractor {
    /// Extract token from multiple sources in priority order:
    /// 1. Authorization header
    /// 2. Query parameters
    /// 3. WebSocket subprotocol (if available)
    pub fn extract_token(
        headers: &actix_web::http::header::HeaderMap,
        query_string: Option<&str>,
    ) -> Result<String, AuthError> {
        // Try Authorization header first
        if let Some(auth_header) = headers.get("Authorization") {
            if let Ok(header_str) = auth_header.to_str() {
                if let Some(token) = header_str.strip_prefix("Bearer ") {
                    return Ok(token.to_string());
                }
            }
        }

        // Try query parameters
        if let Some(query) = query_string {
            let params: std::collections::HashMap<String, String> = 
                url::form_urlencoded::parse(query.as_bytes())
                    .into_owned()
                    .collect();

            if let Some(token) = params.get("token").or_else(|| params.get("access_token")) {
                return Ok(token.clone());
            }
        }

        Err(AuthError::MissingToken)
    }

    /// Extract token from WebSocket subprotocol if present
    pub fn extract_from_subprotocol(protocols: &[String]) -> Option<String> {
        for protocol in protocols {
            if let Some(token) = protocol.strip_prefix("access_token.") {
                return Some(token.to_string());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_jwt_service() -> JwtService {
        let secret = b"test_secret_key_for_jwt_validation";
        JwtService::new(secret).unwrap()
    }

    #[test]
    fn test_jwt_generation_and_validation() {
        let jwt_service = create_test_jwt_service();
        
        let user_id = "test_user".to_string();
        let roles = vec![SystemRole::User, SystemRole::Agent];
        let session_id = Some("session_123".to_string());
        
        // Generate token
        let token = jwt_service
            .generate_token(user_id.clone(), roles, session_id.clone(), None)
            .unwrap();
        
        // Validate token
        match jwt_service.validate_token(&token) {
            AuthResult::Authenticated(context) => {
                assert_eq!(context.user_id, user_id);
                assert_eq!(context.session_id, session_id);
                assert!(context.has_role("user"));
                assert!(context.has_role("agent"));
            }
            _ => panic!("Token validation should succeed"),
        }
    }

    #[test]
    fn test_expired_token() {
        let jwt_service = create_test_jwt_service();
        
        // Create token that expires immediately
        let token = jwt_service
            .generate_token(
                "test_user".to_string(),
                vec![SystemRole::User],
                None,
                Some(Duration::seconds(-1)), // Already expired
            )
            .unwrap();
        
        // Should fail validation
        match jwt_service.validate_token(&token) {
            AuthResult::Unauthenticated(AuthError::ExpiredToken) => {}
            other => panic!("Expected expired token error, got {:?}", other),
        }
    }

    #[test]
    fn test_refresh_token() {
        let jwt_service = create_test_jwt_service();
        
        let user_id = "test_user".to_string();
        let session_id = "session_123".to_string();
        
        // Generate refresh token
        let refresh_token = jwt_service
            .generate_refresh_token(user_id.clone(), session_id.clone())
            .unwrap();
        
        // Validate refresh token
        let (extracted_user_id, extracted_session_id) = jwt_service
            .validate_refresh_token(&refresh_token)
            .unwrap();
        
        assert_eq!(extracted_user_id, user_id);
        assert_eq!(extracted_session_id, Some(session_id));
    }

    #[test]
    fn test_token_extraction() {
        use actix_web::http::header::{HeaderMap, HeaderName, HeaderValue};
        
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("authorization"),
            HeaderValue::from_static("Bearer test_token_123"),
        );
        
        let token = TokenExtractor::extract_token(&headers, None).unwrap();
        assert_eq!(token, "test_token_123");
    }

    #[test]
    fn test_token_extraction_from_query() {
        use actix_web::http::header::HeaderMap;
        
        let query = "token=query_token_456&other_param=value";
        let token = TokenExtractor::extract_token(&HeaderMap::new(), Some(query)).unwrap();
        assert_eq!(token, "query_token_456");
    }

    #[test]
    fn test_missing_token() {
        use actix_web::http::header::HeaderMap;
        
        let result = TokenExtractor::extract_token(&HeaderMap::new(), None);
        assert!(matches!(result, Err(AuthError::MissingToken)));
    }

    #[test]
    fn test_user_context_permissions() {
        let mut roles = HashSet::new();
        roles.insert("admin".to_string());
        
        let mut permissions = HashSet::new();
        permissions.insert("read".to_string());
        permissions.insert("write".to_string());
        
        let context = UserContext {
            user_id: "test_user".to_string(),
            connection_id: Uuid::new_v4(),
            roles,
            permissions,
            session_id: None,
            authenticated_at: Utc::now(),
            expires_at: Some(Utc::now() + Duration::hours(1)),
        };
        
        assert!(context.has_role("admin"));
        assert!(!context.has_role("user"));
        assert!(context.has_permission("read"));
        assert!(context.has_permission("write"));
        assert!(!context.has_permission("delete"));
        assert!(context.is_valid());
    }
}