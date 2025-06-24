//! # Workflow Engine MCP
//! 
//! Model Context Protocol (MCP) integration for the workflow engine.
//! This crate provides:
//! 
//! - MCP protocol implementation
//! - Multiple transport types (HTTP, WebSocket, stdio)
//! - Connection pooling and load balancing
//! - Health monitoring and metrics
//! - Built-in MCP server implementations
//! 
//! ## Features
//! 
//! - `http` - HTTP transport support (enabled by default)
//! - `websocket` - WebSocket transport support (enabled by default) 
//! - `stdio` - Standard I/O transport support
//! - `all` - All transport types
//! 
//! ## Core Concepts
//! 
//! - **Protocol**: Core MCP message types and protocol handling
//! - **Transports**: Different ways to communicate with MCP servers
//! - **Clients**: High-level MCP client implementations
//! - **Servers**: Built-in MCP server implementations
//! - **Connection Pool**: Managed connections with health monitoring
//! 
//! ## Examples
//! 
//! ```rust
//! use workflow_engine_mcp::{
//!     clients::http::HttpMcpClient,
//!     transport::TransportType,
//! };
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = HttpMcpClient::new("http://localhost:8080/mcp")?;
//!     let tools = client.list_tools().await?;
//!     println!("Available tools: {:?}", tools);
//!     Ok(())
//! }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]

// Core MCP modules
pub mod protocol;
pub mod transport;
pub mod clients;
pub mod config;
pub mod config_builder;
pub mod health;
pub mod metrics;
pub mod connection_pool;
pub mod load_balancer;

// MCP server implementations
pub mod server;

// Re-export commonly used types
pub use protocol::{McpMessage, McpRequest, McpResponse, ToolDefinition as McpTool, CallToolResult as McpToolResult};
pub use transport::{TransportType, McpTransport};
pub use clients::McpClient;
pub use config::McpConfig;
pub use connection_pool::{McpConnectionPool as ConnectionPool, PooledConnection};

/// Current version of the MCP integration
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Prelude module for common MCP imports
pub mod prelude {
    pub use crate::{
        McpMessage, McpRequest, McpResponse, 
        TransportType, McpTransport, McpClient,
        McpConfig, ConnectionPool,
        protocol::{ToolDefinition, CallToolResult},
    };
    pub use workflow_engine_core::prelude::*;
}