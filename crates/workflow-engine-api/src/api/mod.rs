//! API module handling HTTP endpoints and request processing.
//!
//! This module contains the API endpoints for the backend service,
//! including event handling and processing endpoints.
//!
//! # Examples
//!
//! The API endpoints can be mounted in an Actix web application:
//!
//! ```rust,no_run
//! use actix_web::{App, HttpServer, web};
//! use backend::api::endpoint::create_event;
//!
//! async fn start_server() -> std::io::Result<()> {
//!     HttpServer::new(|| {
//!         App::new()
//!             .service(create_event)
//!     })
//!     .bind("127.0.0.1:8080")?
//!     .run()
//!     .await
//! }
//! ```

use actix_web::{HttpResponse, Responder, get, web};

pub mod auth;
pub mod events;
pub mod health;
pub mod login;
pub mod metrics;
pub mod middleware;
pub mod openapi;
pub mod openapi_types;
pub mod rate_limit;
pub mod routes;
pub mod startup;
pub mod uptime;
// Workflow API endpoints for triggering and monitoring workflows
pub mod workflows;

#[cfg(feature = "graphql")]
pub mod graphql;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(events::create_event);
    
    // Configure health routes
    cfg.service(
        web::scope("/api/v1")
            .configure(health::config)
    );
    
    // Configure auth routes
    cfg.service(
        web::scope("/api/v1/auth")
            .configure(login::config)
    );
    
    routes::configure(cfg);
    // Workflow API routes for triggering and monitoring workflows
    workflows::configure_routes(cfg);
    metrics::configure_routes(cfg);
    
    // Configure streaming routes
    // Streaming functionality not implemented - requires core::streaming module
    // crate::core::streaming::handlers::configure_streaming_routes(cfg);
    
    // Configure WebSocket endpoint  
    // WebSocket streaming not implemented - requires core::streaming::websocket module
    // cfg.route("/ws/stream", web::get().to(crate::core::streaming::websocket::websocket_streaming_handler));
    
    // Configure OpenAPI/Swagger documentation
    openapi::configure(cfg);
    
    // Configure GraphQL endpoint
    #[cfg(feature = "graphql")]
    {
        use actix_web::web;
        let schema = graphql::create_schema();
        cfg.app_data(web::Data::new(schema))
            .route("/api/v1/graphql", web::post().to(graphql::graphql_handler))
            .route("/api/v1/graphql", web::get().to(graphql::graphql_playground));
    }
}
