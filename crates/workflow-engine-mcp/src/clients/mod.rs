use async_trait::async_trait;
use std::collections::HashMap;

pub mod connection;
pub mod http;
pub mod stdio;
pub mod websocket;

pub use connection::MCPConnection;
pub use http::HttpMCPClient;
pub use stdio::StdioMCPClient;
pub use websocket::WebSocketMCPClient;

use workflow_engine_core::error::WorkflowError;
use crate::protocol::{CallToolResult, ToolDefinition};

#[async_trait]
pub trait MCPClient: Send + Sync + std::fmt::Debug {
    async fn connect(&mut self) -> Result<(), WorkflowError>;
    async fn initialize(
        &mut self,
        client_name: &str,
        client_version: &str,
    ) -> Result<(), WorkflowError>;
    async fn list_tools(&mut self) -> Result<Vec<ToolDefinition>, WorkflowError>;
    async fn call_tool(
        &mut self,
        name: &str,
        arguments: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<CallToolResult, WorkflowError>;
    async fn disconnect(&mut self) -> Result<(), WorkflowError>;
    fn is_connected(&self) -> bool;
}
