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
use crate::transport::StdioTransport;

#[derive(Debug)]
pub struct StdioMcpClient {
    connection: Option<McpConnection>,
    command: String,
    args: Vec<String>,
}

impl StdioMcpClient {
    pub fn new(command: String, args: Vec<String>) -> Self {
        Self {
            connection: None,
            command,
            args,
        }
    }
}

#[async_trait]
impl McpClient for StdioMcpClient {
    async fn connect(&mut self) -> Result<(), WorkflowError> {
        let transport = Box::new(StdioTransport::new(self.command.clone(), self.args.clone()));
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
                .ok_or_else(|| WorkflowError::MCPConnectionError {
                    message: "Not connected".to_string(),
                    server_name: self.command.clone(),
                    transport_type: "stdio".to_string(),
                    endpoint: self.command.clone(),
                    retry_count: 0,
                    source: None,
                })?;

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
            McpResponse::Error { error, .. } => Err(WorkflowError::MCPError {
                message: format!("Initialize failed: {}", error.message),
                server_name: self.command.clone(),
                operation: "initialize".to_string(),
                source: None,
            }),
            _ => Err(WorkflowError::MCPProtocolError {
                message: "Unexpected response to initialize".to_string(),
                server_name: self.command.clone(),
                expected: "InitializeResult".to_string(),
                received: "unknown response type".to_string(),
                message_type: "response".to_string(),
                source: None,
            }),
        }
    }

    async fn list_tools(&mut self) -> Result<Vec<ToolDefinition>, WorkflowError> {
        let connection =
            self.connection
                .as_mut()
                .ok_or_else(|| WorkflowError::MCPConnectionError {
                    message: "Not connected".to_string(),
                    server_name: self.command.clone(),
                    transport_type: "stdio".to_string(),
                    endpoint: self.command.clone(),
                    retry_count: 0,
                    source: None,
                })?;

        if !connection.is_initialized {
            return Err(WorkflowError::MCPError {
                message: "Client not initialized".to_string(),
                server_name: self.command.clone(),
                operation: "list_tools".to_string(),
                source: None,
            });
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
            McpResponse::Error { error, .. } => Err(WorkflowError::MCPError {
                message: format!("List tools failed: {}", error.message),
                server_name: self.command.clone(),
                operation: "list_tools".to_string(),
                source: None,
            }),
            _ => Err(WorkflowError::MCPProtocolError {
                message: "Unexpected response to list_tools".to_string(),
                server_name: self.command.clone(),
                expected: "ListToolsResult".to_string(),
                received: "unknown response type".to_string(),
                message_type: "response".to_string(),
                source: None,
            }),
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
                .ok_or_else(|| WorkflowError::MCPConnectionError {
                    message: "Not connected".to_string(),
                    server_name: self.command.clone(),
                    transport_type: "stdio".to_string(),
                    endpoint: self.command.clone(),
                    retry_count: 0,
                    source: None,
                })?;

        if !connection.is_initialized {
            return Err(WorkflowError::MCPError {
                message: "Client not initialized".to_string(),
                server_name: self.command.clone(),
                operation: "list_tools".to_string(),
                source: None,
            });
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
            McpResponse::Error { error, .. } => Err(WorkflowError::MCPError {
                message: format!("Tool call failed: {}", error.message),
                server_name: self.command.clone(),
                operation: format!("call_tool:{}", name),
                source: None,
            }),
            _ => Err(WorkflowError::MCPProtocolError {
                message: "Unexpected response to call_tool".to_string(),
                server_name: self.command.clone(),
                expected: "CallToolResult".to_string(),
                received: "unknown response type".to_string(),
                message_type: "response".to_string(),
                source: None,
            }),
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
