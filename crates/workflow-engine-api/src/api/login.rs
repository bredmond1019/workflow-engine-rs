use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::auth::JwtAuth;
use crate::db::{session::DbPool, user::UserRepository};

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: String,
    pub username: String,
    pub role: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "Authentication",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 400, description = "Invalid credentials"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn login(
    auth: web::Data<JwtAuth>,
    db_pool: web::Data<DbPool>,
    body: web::Json<LoginRequest>,
) -> Result<HttpResponse> {
    // Validate input
    if body.username.trim().is_empty() || body.password.trim().is_empty() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Username and password are required"
        })));
    }

    // Get database connection from pool
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            log::error!("Failed to get database connection: {}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database connection error"
            })));
        }
    };

    // Create user repository and validate credentials
    let mut user_repo = UserRepository::new(&mut conn);
    
    match user_repo.validate_credentials(&body.username, &body.password) {
        Ok(Some(user)) => {
            // Valid credentials - generate token
            match auth.generate_token(&user.username) {
                Ok(token) => {
                    log::info!("User {} logged in successfully", user.username);
                    Ok(HttpResponse::Ok().json(LoginResponse {
                        token,
                        user_id: user.id.to_string(),
                        username: user.username,
                        role: user.role,
                    }))
                },
                Err(e) => {
                    log::error!("Failed to generate token for user {}: {}", user.username, e);
                    Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Failed to generate authentication token"
                    })))
                }
            }
        },
        Ok(None) => {
            // Invalid credentials
            log::warn!("Failed login attempt for username: {}", body.username);
            Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid username or password"
            })))
        },
        Err(e) => {
            // Database error
            log::error!("Database error during login for {}: {}", body.username, e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Authentication service temporarily unavailable"
            })))
        }
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/login").route(web::post().to(login)));
}