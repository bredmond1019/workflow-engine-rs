use serde_json;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::core::error::WorkflowError;
use crate::core::mcp::protocol::{
    CallToolResult, InitializeResult, ListToolsResult, MCPError, MCPRequest, MCPResponse,
    ResponseResult, ServerCapabilities, ServerInfo, ToolContent, ToolDefinition,
};
use crate::core::nodes::Node;
use crate::core::task::TaskContext;

pub mod customer_support;
pub mod knowledge_base;

#[derive(Debug, Clone)]
pub struct ToolMetadata {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub node_type: TypeId,
}

impl ToolMetadata {
    pub fn new(
        name: String,
        description: String,
        input_schema: serde_json::Value,
        node_type: TypeId,
    ) -> Self {
        Self {
            name,
            description,
            input_schema,
            node_type,
        }
    }

    pub fn to_tool_definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name.clone(),
            description: Some(self.description.clone()),
            input_schema: self.input_schema.clone(),
        }
    }
}

pub struct MCPToolServer {
    server_name: String,
    server_version: String,
    tools: Arc<RwLock<HashMap<String, (ToolMetadata, Arc<dyn Node>)>>>,
    capabilities: ServerCapabilities,
}

impl MCPToolServer {
    pub fn new(server_name: String, server_version: String) -> Self {
        Self {
            server_name,
            server_version,
            tools: Arc::new(RwLock::new(HashMap::new())),
            capabilities: ServerCapabilities {
                logging: None,
                prompts: None,
                resources: None,
                tools: Some(crate::core::mcp::protocol::ToolsCapability {
                    list_changed: Some(true),
                }),
            },
        }
    }

    pub async fn register_node_as_tool<T>(
        &self,
        node: Arc<T>,
        metadata: ToolMetadata,
    ) -> Result<(), WorkflowError>
    where
        T: Node + 'static,
    {
        let mut tools = self.tools.write().await;
        let node_arc: Arc<dyn Node> = node;
        tools.insert(metadata.name.clone(), (metadata, node_arc));
        Ok(())
    }

    pub async fn register_node_with_auto_metadata<T>(
        &self,
        node: Arc<T>,
    ) -> Result<(), WorkflowError>
    where
        T: Node + 'static,
    {
        let node_name = node.node_name();
        let metadata = self.generate_tool_metadata(&node_name)?;
        self.register_node_as_tool(node, metadata).await
    }

    pub fn generate_tool_metadata(&self, node_name: &str) -> Result<ToolMetadata, WorkflowError> {
        let tool_name = node_name.to_lowercase().replace("node", "");
        let description = format!("Tool generated from {} node", node_name);

        // Generate a basic input schema based on TaskContext
        let input_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "context_data": {
                    "type": "object",
                    "description": "Task context data to process"
                },
                "metadata": {
                    "type": "object",
                    "description": "Additional metadata for the task"
                }
            },
            "required": ["context_data"]
        });

        Ok(ToolMetadata::new(
            tool_name,
            description,
            input_schema,
            TypeId::of::<TaskContext>(),
        ))
    }

    pub async fn handle_request(&self, request: MCPRequest) -> Result<MCPResponse, WorkflowError> {
        match request {
            MCPRequest::Initialize { id, params } => {
                let result = InitializeResult {
                    protocol_version: params.protocol_version,
                    capabilities: self.capabilities.clone(),
                    server_info: ServerInfo {
                        name: self.server_name.clone(),
                        version: self.server_version.clone(),
                    },
                };

                Ok(MCPResponse::Result {
                    id,
                    result: ResponseResult::Initialize(result),
                })
            }
            MCPRequest::ListTools { id } => {
                let tools = self.tools.read().await;
                let tool_definitions: Vec<ToolDefinition> = tools
                    .values()
                    .map(|(metadata, _)| metadata.to_tool_definition())
                    .collect();

                Ok(MCPResponse::Result {
                    id,
                    result: ResponseResult::ListTools(ListToolsResult {
                        tools: tool_definitions,
                    }),
                })
            }
            MCPRequest::CallTool { id, params } => {
                let tools = self.tools.read().await;

                if let Some((metadata, node)) = tools.get(&params.name) {
                    // Convert MCP arguments to TaskContext
                    let task_context = self.arguments_to_task_context(params.arguments)?;

                    // Execute the node
                    match node.process(task_context) {
                        Ok(result_context) => {
                            let content = self.task_context_to_content(result_context)?;

                            Ok(MCPResponse::Result {
                                id,
                                result: ResponseResult::CallTool(CallToolResult {
                                    content,
                                    is_error: Some(false),
                                }),
                            })
                        }
                        Err(error) => {
                            let content = vec![ToolContent::Text {
                                text: format!("Error executing tool: {}", error),
                            }];

                            Ok(MCPResponse::Result {
                                id,
                                result: ResponseResult::CallTool(CallToolResult {
                                    content,
                                    is_error: Some(true),
                                }),
                            })
                        }
                    }
                } else {
                    Ok(MCPResponse::Error {
                        id,
                        error: MCPError {
                            code: -32601,
                            message: format!("Tool '{}' not found", params.name),
                            data: None,
                        },
                    })
                }
            }
            MCPRequest::Initialized => {
                // Notification - no response needed
                Err(WorkflowError::MCPProtocolError {
                    message: "Initialized notification should not expect a response".to_string(),
                })
            }
        }
    }

    fn arguments_to_task_context(
        &self,
        arguments: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<TaskContext, WorkflowError> {
        let mut task_context = TaskContext::new("mcp_tool".to_string(), serde_json::Value::Null);

        if let Some(args) = arguments {
            if let Some(context_data) = args.get("context_data") {
                if let Ok(data_map) = serde_json::from_value::<HashMap<String, serde_json::Value>>(
                    context_data.clone(),
                ) {
                    for (key, value) in data_map {
                        task_context.set_data(&key, value);
                    }
                }
            }

            if let Some(metadata) = args.get("metadata") {
                if let Ok(meta_map) =
                    serde_json::from_value::<HashMap<String, serde_json::Value>>(metadata.clone())
                {
                    for (key, value) in meta_map {
                        task_context.set_metadata(&key, value);
                    }
                }
            }
        }

        Ok(task_context)
    }

    fn task_context_to_content(
        &self,
        task_context: TaskContext,
    ) -> Result<Vec<ToolContent>, WorkflowError> {
        let context_json = serde_json::to_string_pretty(&task_context).map_err(|e| {
            WorkflowError::SerializationError {
                message: format!("Failed to serialize task context: {}", e),
            }
        })?;

        Ok(vec![ToolContent::Text { text: context_json }])
    }

    pub async fn get_tool_count(&self) -> usize {
        let tools = self.tools.read().await;
        tools.len()
    }

    pub async fn get_tool_names(&self) -> Vec<String> {
        let tools = self.tools.read().await;
        tools.keys().cloned().collect()
    }

    pub async fn remove_tool(&self, tool_name: &str) -> Result<bool, WorkflowError> {
        let mut tools = self.tools.write().await;
        Ok(tools.remove(tool_name).is_some())
    }

    pub async fn has_tool(&self, tool_name: &str) -> bool {
        let tools = self.tools.read().await;
        tools.contains_key(tool_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nodes::Node;

    #[derive(Debug)]
    struct TestNode {
        name: String,
    }

    impl TestNode {
        fn new(name: String) -> Self {
            Self { name }
        }
    }

    impl Node for TestNode {
        fn node_name(&self) -> String {
            self.name.clone()
        }

        fn process(&self, mut task_context: TaskContext) -> Result<TaskContext, WorkflowError> {
            task_context.set_data("processed_by", serde_json::Value::String(self.name.clone()));
            Ok(task_context)
        }
    }

    #[tokio::test]
    async fn test_mcp_tool_server_creation() {
        let server = MCPToolServer::new("test-server".to_string(), "1.0.0".to_string());
        assert_eq!(server.server_name, "test-server");
        assert_eq!(server.server_version, "1.0.0");
        assert_eq!(server.get_tool_count().await, 0);
    }

    #[tokio::test]
    async fn test_register_node_as_tool() {
        let server = MCPToolServer::new("test-server".to_string(), "1.0.0".to_string());
        let node = Arc::new(TestNode::new("TestNode".to_string()));

        let metadata = ToolMetadata::new(
            "test_tool".to_string(),
            "A test tool".to_string(),
            serde_json::json!({"type": "object"}),
            TypeId::of::<TestNode>(),
        );

        server.register_node_as_tool(node, metadata).await.unwrap();
        assert_eq!(server.get_tool_count().await, 1);
        assert!(server.has_tool("test_tool").await);
    }

    #[tokio::test]
    async fn test_tool_metadata_generation() {
        let server = MCPToolServer::new("test-server".to_string(), "1.0.0".to_string());
        let metadata = server.generate_tool_metadata("TestNode").unwrap();

        assert_eq!(metadata.name, "test");
        assert!(metadata.description.contains("TestNode"));
    }

    #[tokio::test]
    async fn test_handle_list_tools_request() {
        let server = MCPToolServer::new("test-server".to_string(), "1.0.0".to_string());
        let node = Arc::new(TestNode::new("TestNode".to_string()));
        server.register_node_with_auto_metadata(node).await.unwrap();

        let request = MCPRequest::ListTools {
            id: "test-123".to_string(),
        };

        let response = server.handle_request(request).await.unwrap();
        match response {
            MCPResponse::Result {
                result: ResponseResult::ListTools(tools_result),
                ..
            } => {
                assert_eq!(tools_result.tools.len(), 1);
                assert_eq!(tools_result.tools[0].name, "test");
            }
            _ => panic!("Expected ListTools response"),
        }
    }
}
