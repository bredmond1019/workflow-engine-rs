//! Real-time Communication Layer
//! 
//! High-performance WebSocket service for real-time messaging and notifications
//! with support for 10,000+ concurrent connections.

pub mod server;
pub mod connection;
pub mod actor;
pub mod actors;
pub mod routing;
pub mod protection;
pub mod auth;
pub mod messaging;
pub mod session;
pub mod api;
pub mod persistence;
pub mod notifications;
pub mod presence;

pub use server::*;
pub use connection::*;
pub use actor::*;
pub use actors::*;
pub use routing::*;
pub use protection::*;
pub use auth::*;
pub use messaging::*;
pub use session::*;
pub use persistence::*;
pub use notifications::*;
pub use presence::*;