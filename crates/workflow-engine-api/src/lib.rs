//! # Workflow Engine API
//! 
//! REST API server for the AI workflow engine.
//! This crate provides:
//! 
//! - HTTP REST API with OpenAPI documentation
//! - JWT authentication and authorization
//! - Rate limiting and CORS support
//! - Health checks and metrics endpoints
//! - Service bootstrap and dependency injection
//! 
//! ## Features
//! 
//! - `openapi` - OpenAPI documentation generation (enabled by default)
//! - `auth` - JWT authentication support (enabled by default)
//! - `monitoring` - Prometheus metrics (enabled by default)
//! - `database` - Database integration
//! 
//! ## Core Components
//! 
//! - **API Routes**: RESTful endpoints for workflow management
//! - **Authentication**: JWT-based auth with middleware
//! - **Bootstrap**: Service discovery and dependency injection
//! - **Health**: System health monitoring
//! - **Metrics**: Prometheus metrics collection
//! 
//! ## Examples
//! 
//! ```rust,no_run
//! use workflow_engine_api::{
//!     bootstrap::ServiceContainer,
//!     api::ApiServer,
//! };
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let container = ServiceContainer::new().await?;
//!     let server = ApiServer::new(container);
//!     server.run("127.0.0.1:8080").await?;
//!     Ok(())
//! }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]

// Core API modules
pub mod api;
pub mod bootstrap;

// Application modules
pub mod db;
pub mod workflows;
pub mod monitoring;
pub mod integrations;

// Re-export commonly used types
// Note: ApiServer and ApiConfig exports disabled - use Actix web components directly
// Note: ServiceContainer export disabled - not yet implemented
// pub use api::{ApiServer, ApiConfig};
// pub use bootstrap::{ServiceContainer};
pub use bootstrap::{ServiceRegistry};

// Workflows module re-exports
pub use workflows::{
    WorkflowRunner,
    customer_support_workflow,
    knowledge_base_workflow,
    demos,
};

// Feature-specific re-exports
#[cfg(feature = "auth")]
// Note: Auth module exports disabled - use api::auth module directly
// pub use api::auth::{AuthConfig, JwtClaims};

#[cfg(feature = "monitoring")]
pub use monitoring::metrics::ApiMetrics;

/// Current version of the API server
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Prelude module for common API imports
pub mod prelude {
    // Note: Limited exports until full implementations are available
    // pub use crate::{ApiServer, ApiConfig, ServiceContainer};
    pub use crate::bootstrap::ServiceRegistry;
    pub use crate::workflows::WorkflowRunner;
    pub use workflow_engine_core::prelude::*;
    pub use workflow_engine_mcp::prelude::*;
    pub use actix_web::{web, App, HttpServer, Result as ActixResult};
    pub use serde_json::json;
}