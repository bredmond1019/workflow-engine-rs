use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use workflow_engine_core::auth::{Claims, JwtAuth, JwtError};

/// Request body for token generation
#[derive(Debug, Deserialize, Serialize)]
pub struct TokenRequest {
    /// User ID or service identifier
    pub sub: String,
    /// Role: admin|developer|service
    pub role: String,
}

/// Response body for token generation
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    /// JWT access token
    pub access_token: String,
    /// Token type (always "Bearer")
    pub token_type: String,
    /// Expiration time in seconds
    pub expires_in: u64,
}

/// Request body for token verification
#[derive(Debug, Deserialize, Serialize)]
pub struct VerifyRequest {
    /// JWT token to verify
    pub token: String,
}

/// Response body for token verification
#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyResponse {
    /// Whether the token is valid
    pub valid: bool,
    /// Claims if token is valid
    pub claims: Option<Claims>,
    /// Error message if token is invalid
    pub error: Option<String>,
}

/// Generate a development JWT token
/// 
/// This endpoint is for development purposes only.
/// In production, tokens should be issued by Auth0 or similar.
pub async fn generate_token(
    req: web::Json<TokenRequest>,
) -> Result<HttpResponse> {
    let claims = Claims::new(req.sub.clone(), req.role.clone());
    
    let jwt_auth = JwtAuth::default();
    match jwt_auth.generate_token(&claims) {
        Ok(token) => {
            let response = TokenResponse {
                access_token: token,
                token_type: "Bearer".to_string(),
                expires_in: 86400, // 24 hours
            };
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "token_generation_failed",
                "message": e.to_string()
            })))
        }
    }
}

/// Verify token validity
pub async fn verify_token(
    req: web::Json<VerifyRequest>,
) -> Result<HttpResponse> {
    let jwt_auth = JwtAuth::default();
    match jwt_auth.validate_token(&req.token) {
        Ok(claims) => {
            let response = VerifyResponse {
                valid: true,
                claims: Some(claims),
                error: None,
            };
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            let response = VerifyResponse {
                valid: false,
                claims: None,
                error: Some(e.to_string()),
            };
            Ok(HttpResponse::Ok().json(response))
        }
    }
}

/// Configure auth routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/token", web::post().to(generate_token))
            .route("/verify", web::get().to(verify_token))
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_generate_token_endpoint() {
        let app = test::init_service(
            App::new().configure(configure)
        ).await;

        let req = test::TestRequest::post()
            .uri("/auth/token")
            .set_json(&TokenRequest {
                sub: "test_user".to_string(),
                role: "developer".to_string(),
            })
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: TokenResponse = test::read_body_json(resp).await;
        assert!(!body.access_token.is_empty());
        assert_eq!(body.token_type, "Bearer");
        assert_eq!(body.expires_in, 86400);
    }

    #[actix_web::test]
    async fn test_verify_token_endpoint() {
        let app = test::init_service(
            App::new().configure(configure)
        ).await;

        // First generate a token
        let token_req = test::TestRequest::post()
            .uri("/auth/token")
            .set_json(&TokenRequest {
                sub: "test_user".to_string(),
                role: "admin".to_string(),
            })
            .to_request();

        let token_resp = test::call_service(&app, token_req).await;
        let token_body: TokenResponse = test::read_body_json(token_resp).await;

        // Then verify it
        let verify_req = test::TestRequest::get()
            .uri("/auth/verify")
            .set_json(&VerifyRequest {
                token: token_body.access_token,
            })
            .to_request();

        let verify_resp = test::call_service(&app, verify_req).await;
        assert!(verify_resp.status().is_success());

        let verify_body: VerifyResponse = test::read_body_json(verify_resp).await;
        assert!(verify_body.valid);
        assert!(verify_body.claims.is_some());
        assert_eq!(verify_body.claims.unwrap().sub, "test_user");
    }

    #[actix_web::test]
    async fn test_verify_invalid_token() {
        let app = test::init_service(
            App::new().configure(configure)
        ).await;

        let req = test::TestRequest::get()
            .uri("/auth/verify")
            .set_json(&VerifyRequest {
                token: "invalid.token.here".to_string(),
            })
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: VerifyResponse = test::read_body_json(resp).await;
        assert!(!body.valid);
        assert!(body.claims.is_none());
        assert!(body.error.is_some());
    }
}