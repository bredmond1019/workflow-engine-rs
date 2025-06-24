use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use workflow_engine_core::error::WorkflowError;
use crate::protocol::{
    CallToolResult, ClientCapabilities, ClientInfo, InitializeParams, McpRequest, McpResponse,
    ResponseResult, ToolCallParams, ToolDefinition,
};
use crate::transport::{McpTransport, StdioTransport, WebSocketTransport};

pub struct McpConnection {
    pub transport: Box<dyn McpTransport>,
    pub is_connected: bool,
    pub is_initialized: bool,
    pending_requests: Arc<Mutex<HashMap<String, tokio::sync::oneshot::Sender<McpResponse>>>>,
}

impl std::fmt::Debug for McpConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpConnection")
            .field("is_connected", &self.is_connected)
            .field("is_initialized", &self.is_initialized)
            .finish()
    }
}

impl McpConnection {
    pub fn new(transport: Box<dyn McpTransport>) -> Self {
        Self {
            transport,
            is_connected: false,
            is_initialized: false,
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn send_request(
        &mut self,
        request: McpRequest,
    ) -> Result<McpResponse, WorkflowError> {
        let id = request
            .get_id()
            .map(|id| id.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let (tx, rx) = tokio::sync::oneshot::channel();
        {
            let mut pending = self.pending_requests.lock().await;
            pending.insert(id.clone(), tx);
        }

        self.transport.send(request).await?;

        match rx.await {
            Ok(response) => Ok(response),
            Err(_) => Err(WorkflowError::MCPError {
                message: "Request timeout or connection closed".to_string(),
            }),
        }
    }

    async fn receive_response(&mut self) -> Result<(), WorkflowError> {
        let response = self.transport.receive().await?;
        let id = response.get_id().to_string();

        let mut pending = self.pending_requests.lock().await;
        if let Some(tx) = pending.remove(&id) {
            let _ = tx.send(response);
        }

        Ok(())
    }
}
