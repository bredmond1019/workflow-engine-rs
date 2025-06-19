use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware, web};
use dotenvy::dotenv;
use log::info;
use std::{env, sync::Arc, time::{SystemTime, UNIX_EPOCH}};

use workflow_engine_api::db::session::DbPool;
use workflow_engine_api::api;
use workflow_engine_core::auth::JwtAuth;
use workflow_engine_api::api::middleware::auth::JwtMiddleware;
use workflow_engine_api::api::rate_limit::{RateLimitConfig, RateLimitMiddleware};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Set backtrace and logging
    // Note: In production, set these via environment or command line
    if env::var("RUST_BACKTRACE").is_err() {
        std::env::set_var("RUST_BACKTRACE", "1");
    }
    if env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    
    // Set process start time for health checks
    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time before UNIX epoch")
        .as_secs();
    std::env::set_var("PROCESS_START_TIME", start_time.to_string());

    // Load environment variables from .env file
    dotenv().ok();

    // Initialize structured logging with correlation ID support
    workflow_engine_api::monitoring::logging::init_structured_logging();

    // Get host and port from environment variables or use defaults
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let server_url = format!("{}:{}", host, port);

    info!("Starting server at http://{}", server_url);

    // Initialize database pool
    let pool: DbPool = workflow_engine_api::db::session::init_pool()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to initialize database pool: {}", e)))?;
    let arc_pool = Arc::new(pool.clone());

    // Initialize JWT auth
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key".to_string());
    let jwt_auth = web::Data::new(JwtAuth::new(jwt_secret.clone()));

    // Configure rate limiting
    let requests_per_minute = env::var("RATE_LIMIT_PER_MINUTE")
        .unwrap_or_else(|_| "60".to_string())
        .parse()
        .unwrap_or_else(|e| {
            log::warn!("Invalid RATE_LIMIT_PER_MINUTE value, using default 60: {}", e);
            60
        });
    
    let burst_size = env::var("RATE_LIMIT_BURST")
        .unwrap_or_else(|_| "10".to_string())
        .parse()
        .unwrap_or_else(|e| {
            log::warn!("Invalid RATE_LIMIT_BURST value, using default 10: {}", e);
            10
        });
    
    let rate_limit_config = RateLimitConfig {
        requests_per_minute,
        burst_size,
    };

    // TODO: Re-enable when workflows module is fixed
    // info!("Starting Demo Workflows");
    // workflows::demos::run_all_demos().await;

    // Start HTTP server
    HttpServer::new(move || {
        // Configure CORS
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            // Add database pool to app data
            .app_data(web::Data::new(arc_pool.clone()))
            // Add JWT auth to app data
            .app_data(jwt_auth.clone())
            // Enable logger middleware
            .wrap(middleware::Logger::default())
            // Enable CORS
            .wrap(cors)
            // Enable rate limiting
            .wrap(RateLimitMiddleware::new(rate_limit_config.clone()))
            // Enable JWT authentication
            .wrap(JwtMiddleware::new(jwt_secret.clone()))
            // Configure routes
            .configure(api::init_routes)
    })
    .bind(server_url)?
    .run()
    .await
}
