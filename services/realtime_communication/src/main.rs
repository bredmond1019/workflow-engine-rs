//! Real-time Communication Service
//!
//! Main entry point for the real-time communication microservice

use actix_web::{web, App, HttpServer, middleware};
use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use realtime_communication::{
    api::graphql::schema::create_schema,
    server::{WebSocketServer, ServerConfig, ServerState},
    connection::ConnectionManager,
    actors::manager::ActorManager,
    persistence::MessageStore,
    presence::PresenceManager,
    protection::rate_limiter::RateLimiter,
    protection::circuit_breaker::CircuitBreaker,
    ServerMetrics,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Real-time Communication Service");

    // Get configuration from environment
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8002".to_string())
        .parse::<u16>()
        .expect("Invalid PORT");
    let redis_url = env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost/realtime_communication".to_string());

    // Create server configuration
    let config = ServerConfig {
        host: host.clone(),
        port,
        ..Default::default()
    };

    // Initialize components
    let connection_manager = std::sync::Arc::new(ConnectionManager::new(config.max_connections));
    let metrics = std::sync::Arc::new(ServerMetrics::new());
    
    // Create server state
    let server_state = ServerState {
        connection_manager: connection_manager.clone(),
        config: config.clone(),
        metrics: metrics.clone(),
    };

    // Create GraphQL schema
    let graphql_schema = create_schema();

    tracing::info!("Starting HTTP server on {}:{}", host, port);

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(server_state.clone()))
            .app_data(web::Data::new(graphql_schema.clone()))
            .wrap(middleware::Logger::default())
            .configure(|cfg| {
                // WebSocket endpoints
                cfg.route("/ws", web::get().to(realtime_communication::server::websocket_handler));
                cfg.route("/health", web::get().to(realtime_communication::server::health_handler));
                cfg.route("/metrics", web::get().to(realtime_communication::server::metrics_handler));
                
                // GraphQL endpoints
                cfg.route("/graphql", web::post().to(realtime_communication::api::graphql_handler));
                cfg.route("/graphql", web::get().to(realtime_communication::api::graphql_handler));
                cfg.route("/graphiql", web::get().to(realtime_communication::api::graphiql));
            })
    })
    .workers(num_cpus::get())
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}