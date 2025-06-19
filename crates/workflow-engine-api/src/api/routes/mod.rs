pub mod auth;
pub mod health;
pub mod registry;

use actix_web::web;

/// Configure all API routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    // Health routes (no authentication required)
    health::configure_health_routes(cfg);
    
    // Authenticated routes
    auth::configure(cfg);
    registry::configure(cfg);
}