use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;

use workflow_engine_core::error::WorkflowError;
use crate::clients::McpClient;
use crate::clients::connection::McpConnection;
use crate::protocol::{
    CallToolResult, ClientCapabilities, ClientInfo, InitializeParams, McpRequest, McpResponse,
    ResponseResult, ToolCallParams, ToolDefinition,
};
use crate::transport::WebSocketTransport;

#[derive(Debug)]
pub struct WebSocketMcpClient {
    connection: Option<McpConnection>,
    url: String,
}

impl WebSocketMcpClient {
    pub fn new(url: String) -> Self {
        Self {
            connection: None,
            url,
        }
    }
}

#[async_trait]
impl McpClient for WebSocketMcpClient {
    async fn connect(&mut self) -> Result<(), WorkflowError> {
        let transport = Box::new(WebSocketTransport::new(self.url.clone()));
        let mut connection = McpConnection::new(transport);

        connection.transport.connect().await?;
        connection.is_connected = true;

        self.connection = Some(connection);
        Ok(())
    }

    async fn initialize(
        &mut self,
        client_name: &str,
        client_version: &str,
    ) -> Result<(), WorkflowError> {
        let connection =
            self.connection
                .as_mut()
                .ok_or_else(|| WorkflowError::mcp_connection_error(
                    "Not connected",
                    &self.url,
                    "websocket",
                    &self.url,
                ))?;

        let request = McpRequest::Initialize {
            id: Uuid::new_v4().to_string(),
            params: InitializeParams {
                protocol_version: "2024-11-05".to_string(),
                capabilities: ClientCapabilities {
                    roots: None,
                    sampling: None,
                },
                client_info: ClientInfo {
                    name: client_name.to_string(),
                    version: client_version.to_string(),
                },
            },
        };

        let response = connection.send_request(request).await?;
        match response {
            McpResponse::Result {
                result: ResponseResult::Initialize(_),
                ..
            } => {
                connection.is_initialized = true;

                // Send initialized notification
                let initialized = McpRequest::Initialized;
                connection.transport.send(initialized).await?;

                Ok(())
            }
            McpResponse::Error { error, .. } => Err(WorkflowError::mcp_error(
                &format!("Initialize failed: {}", error.message),
                &self.url,
                "initialize",
            )),
            _ => Err(WorkflowError::mcp_protocol_error(
                "Unexpected response to initialize",
                &self.url,
                "InitializeResult",
                "unknown response type",
                "response",
            )),
        }
    }

    async fn list_tools(&mut self) -> Result<Vec<ToolDefinition>, WorkflowError> {
        let connection =
            self.connection
                .as_mut()
                .ok_or_else(|| WorkflowError::mcp_connection_error(
                    "Not connected",
                    &self.url,
                    "websocket",
                    &self.url,
                ))?;

        if !connection.is_initialized {
            return Err(WorkflowError::mcp_error(
                "Client not initialized",
                &self.url,
                "list_tools",
            ));
        }

        let request = McpRequest::ListTools {
            id: Uuid::new_v4().to_string(),
        };

        let response = connection.send_request(request).await?;
        match response {
            McpResponse::Result {
                result: ResponseResult::ListTools(tools_result),
                ..
            } => Ok(tools_result.tools),
            McpResponse::Error { error, .. } => Err(WorkflowError::mcp_error(
                &format!("List tools failed: {}", error.message),
                &self.url,
                "list_tools",
            )),
            _ => Err(WorkflowError::mcp_protocol_error(
                "Unexpected response to list_tools",
                &self.url,
                "ListToolsResult",
                "unknown response type",
                "response",
            )),
        }
    }

    async fn call_tool(
        &mut self,
        name: &str,
        arguments: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<CallToolResult, WorkflowError> {
        let connection =
            self.connection
                .as_mut()
                .ok_or_else(|| WorkflowError::mcp_connection_error(
                    "Not connected",
                    &self.url,
                    "websocket",
                    &self.url,
                ))?;

        if !connection.is_initialized {
            return Err(WorkflowError::mcp_error(
                "Client not initialized",
                &self.url,
                "call_tool",
            ));
        }

        let request = McpRequest::CallTool {
            id: Uuid::new_v4().to_string(),
            params: ToolCallParams {
                name: name.to_string(),
                arguments,
            },
        };

        let response = connection.send_request(request).await?;
        match response {
            McpResponse::Result {
                result: ResponseResult::CallTool(call_result),
                ..
            } => Ok(call_result),
            McpResponse::Error { error, .. } => Err(WorkflowError::mcp_error(
                &format!("Tool call failed: {}", error.message),
                &self.url,
                &format!("call_tool:{}", name),
            )),
            _ => Err(WorkflowError::mcp_protocol_error(
                "Unexpected response to call_tool",
                &self.url,
                "CallToolResult",
                "unknown response type",
                "response",
            )),
        }
    }

    async fn disconnect(&mut self) -> Result<(), WorkflowError> {
        if let Some(mut connection) = self.connection.take() {
            connection.transport.disconnect().await?;
            connection.is_connected = false;
            connection.is_initialized = false;
        }
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connection
            .as_ref()
            .map(|c| c.is_connected)
            .unwrap_or(false)
    }
}
