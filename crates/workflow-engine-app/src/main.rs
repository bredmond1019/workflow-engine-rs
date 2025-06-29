use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware, web};
use dotenvy::dotenv;
use log::info;
use std::{env, sync::Arc};

use workflow_engine_api::db::session::DbPool;
use workflow_engine_api::api;
use workflow_engine_core::auth::JwtAuth;
use workflow_engine_api::api::middleware::auth::JwtMiddleware;
use workflow_engine_api::api::rate_limit::{RateLimitConfig, RateLimitMiddleware};

mod config;
use config::{AppConfig, ConfigError};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file first
    dotenv().ok();
    
    // Initialize process start time first
    workflow_engine_api::api::startup::init_startup_time();

    // Initialize logging
    env_logger::init();

    // Initialize application configuration with proper error handling
    let config = match AppConfig::new().await {
        Ok(config) => {
            info!("Application configuration loaded successfully");
            config
        }
        Err(e) => {
            eprintln!("Configuration error: {}", e);
            std::process::exit(1);
        }
    };

    let server_url = config.server_address();
    info!("Starting server at http://{}", server_url);

    let rate_limit_config = RateLimitConfig {
        requests_per_minute: config.rate_limit_per_minute,
        burst_size: config.rate_limit_burst,
    };

    // Optional: Run demo workflows on startup (disabled by default for production)
    // Uncomment the following lines to run demos on server startup:
    // info!("Starting Demo Workflows");
    // workflow_engine_api::workflows::demos::run_all_demos().await;

    // Start HTTP server
    let jwt_auth = config.jwt_auth.clone();
    let database_pool = config.database_pool.clone();
    
    HttpServer::new(move || {
        // Configure CORS
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            // Add database pool to app data
            .app_data(web::Data::new(database_pool.clone()))
            // Add JWT auth to app data
            .app_data(jwt_auth.clone())
            // Enable logger middleware
            .wrap(middleware::Logger::default())
            // Enable CORS
            .wrap(cors)
            // Configure routes
            .configure(api::init_routes)
    })
    .bind(server_url)?
    .run()
    .await
}
