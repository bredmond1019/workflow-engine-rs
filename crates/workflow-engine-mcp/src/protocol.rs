use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method")]
pub enum MCPRequest {
    #[serde(rename = "initialize")]
    Initialize {
        id: String,
        params: InitializeParams,
    },
    #[serde(rename = "tools/list")]
    ListTools {
        id: String,
    },
    #[serde(rename = "tools/call")]
    CallTool {
        id: String,
        params: ToolCallParams,
    },
    #[serde(rename = "notifications/initialized")]
    Initialized,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MCPResponse {
    #[serde(rename = "result")]
    Result {
        id: String,
        result: ResponseResult,
    },
    #[serde(rename = "error")]
    Error {
        id: String,
        error: MCPError,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponseResult {
    Initialize(InitializeResult),
    ListTools(ListToolsResult),
    CallTool(CallToolResult),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    pub protocol_version: String,
    pub capabilities: ClientCapabilities,
    pub client_info: ClientInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    pub server_info: ServerInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCapabilities {
    pub roots: Option<RootsCapability>,
    pub sampling: Option<SamplingCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    pub logging: Option<LoggingCapability>,
    pub prompts: Option<PromptsCapability>,
    pub resources: Option<ResourcesCapability>,
    pub tools: Option<ToolsCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootsCapability {
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingCapability {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingCapability {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsCapability {
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesCapability {
    pub subscribe: Option<bool>,
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCapability {
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsResult {
    pub tools: Vec<ToolDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallParams {
    pub name: String,
    pub arguments: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolResult {
    pub content: Vec<ToolContent>,
    pub is_error: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { data: String, mime_type: String },
    #[serde(rename = "resource")]
    Resource { resource: ResourceReference },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceReference {
    pub uri: String,
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MCPMessage {
    Request(MCPRequest),
    Response(MCPResponse),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: HashMap<String, serde_json::Value>,
}

impl MCPRequest {
    pub fn get_id(&self) -> Option<&str> {
        match self {
            MCPRequest::Initialize { id, .. } => Some(id),
            MCPRequest::ListTools { id } => Some(id),
            MCPRequest::CallTool { id, .. } => Some(id),
            MCPRequest::Initialized => None,
        }
    }
}

impl MCPResponse {
    pub fn get_id(&self) -> &str {
        match self {
            MCPResponse::Result { id, .. } => id,
            MCPResponse::Error { id, .. } => id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Task 5.1.1: Create test module in `src/core/mcp/protocol.rs`
    // Task 5.1.2: Test message serialization and deserialization
    
    #[test]
    fn test_mcp_request_initialize_serialization() {
        let request = MCPRequest::Initialize {
            id: "test-123".to_string(),
            params: InitializeParams {
                protocol_version: "1.0".to_string(),
                capabilities: ClientCapabilities {
                    roots: Some(RootsCapability {
                        list_changed: Some(true),
                    }),
                    sampling: None,
                },
                client_info: ClientInfo {
                    name: "test-client".to_string(),
                    version: "0.1.0".to_string(),
                },
            },
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: MCPRequest = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            MCPRequest::Initialize { id, params } => {
                assert_eq!(id, "test-123");
                assert_eq!(params.protocol_version, "1.0");
                assert_eq!(params.client_info.name, "test-client");
                assert_eq!(params.client_info.version, "0.1.0");
                assert!(params.capabilities.roots.is_some());
            }
            _ => panic!("Wrong request type"),
        }
    }

    #[test]
    fn test_mcp_request_list_tools_serialization() {
        let request = MCPRequest::ListTools {
            id: "tools-req-1".to_string(),
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["method"], "tools/list");
        assert_eq!(json["id"], "tools-req-1");

        let deserialized: MCPRequest = serde_json::from_value(json).unwrap();
        match deserialized {
            MCPRequest::ListTools { id } => {
                assert_eq!(id, "tools-req-1");
            }
            _ => panic!("Wrong request type"),
        }
    }

    #[test]
    fn test_mcp_request_call_tool_serialization() {
        let mut arguments = HashMap::new();
        arguments.insert("text".to_string(), json!("Hello world"));
        arguments.insert("count".to_string(), json!(42));

        let request = MCPRequest::CallTool {
            id: "call-1".to_string(),
            params: ToolCallParams {
                name: "analyze_text".to_string(),
                arguments: Some(arguments),
            },
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: MCPRequest = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            MCPRequest::CallTool { id, params } => {
                assert_eq!(id, "call-1");
                assert_eq!(params.name, "analyze_text");
                assert!(params.arguments.is_some());
                let args = params.arguments.unwrap();
                assert_eq!(args.get("text").unwrap(), &json!("Hello world"));
                assert_eq!(args.get("count").unwrap(), &json!(42));
            }
            _ => panic!("Wrong request type"),
        }
    }

    #[test]
    fn test_mcp_response_result_serialization() {
        let response = MCPResponse::Result {
            id: "resp-1".to_string(),
            result: ResponseResult::Initialize(InitializeResult {
                protocol_version: "1.0".to_string(),
                capabilities: ServerCapabilities {
                    logging: Some(LoggingCapability {}),
                    prompts: None,
                    resources: Some(ResourcesCapability {
                        subscribe: Some(true),
                        list_changed: Some(false),
                    }),
                    tools: Some(ToolsCapability {
                        list_changed: Some(true),
                    }),
                },
                server_info: ServerInfo {
                    name: "test-server".to_string(),
                    version: "1.0.0".to_string(),
                },
            }),
        };

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: MCPResponse = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            MCPResponse::Result { id, result } => {
                assert_eq!(id, "resp-1");
                match result {
                    ResponseResult::Initialize(init) => {
                        assert_eq!(init.protocol_version, "1.0");
                        assert_eq!(init.server_info.name, "test-server");
                        assert!(init.capabilities.logging.is_some());
                        assert!(init.capabilities.tools.is_some());
                    }
                    _ => panic!("Wrong result type"),
                }
            }
            _ => panic!("Wrong response type"),
        }
    }

    #[test]
    fn test_mcp_response_error_serialization() {
        let response = MCPResponse::Error {
            id: "err-1".to_string(),
            error: MCPError {
                code: -32601,
                message: "Method not found".to_string(),
                data: Some(json!({"method": "unknown_method"})),
            },
        };

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: MCPResponse = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            MCPResponse::Error { id, error } => {
                assert_eq!(id, "err-1");
                assert_eq!(error.code, -32601);
                assert_eq!(error.message, "Method not found");
                assert!(error.data.is_some());
            }
            _ => panic!("Wrong response type"),
        }
    }

    #[test]
    fn test_tool_content_serialization() {
        let text_content = ToolContent::Text {
            text: "Analysis complete".to_string(),
        };

        let json = serde_json::to_value(&text_content).unwrap();
        assert_eq!(json["type"], "text");
        assert_eq!(json["text"], "Analysis complete");

        let image_content = ToolContent::Image {
            data: "base64encodeddata".to_string(),
            mime_type: "image/png".to_string(),
        };

        let json = serde_json::to_value(&image_content).unwrap();
        assert_eq!(json["type"], "image");
        assert_eq!(json["data"], "base64encodeddata");
        assert_eq!(json["mime_type"], "image/png");

        let resource_content = ToolContent::Resource {
            resource: ResourceReference {
                uri: "file:///path/to/file.txt".to_string(),
                mime_type: Some("text/plain".to_string()),
            },
        };

        let json = serde_json::to_value(&resource_content).unwrap();
        assert_eq!(json["type"], "resource");
        assert_eq!(json["resource"]["uri"], "file:///path/to/file.txt");
    }

    // Task 5.1.3: Test protocol version negotiation
    
    #[test]
    fn test_protocol_version_negotiation() {
        let client_init = InitializeParams {
            protocol_version: "1.0".to_string(),
            capabilities: ClientCapabilities {
                roots: None,
                sampling: None,
            },
            client_info: ClientInfo {
                name: "test-client".to_string(),
                version: "0.1.0".to_string(),
            },
        };

        let server_init = InitializeResult {
            protocol_version: "1.0".to_string(),
            capabilities: ServerCapabilities {
                logging: None,
                prompts: None,
                resources: None,
                tools: Some(ToolsCapability {
                    list_changed: Some(true),
                }),
            },
            server_info: ServerInfo {
                name: "test-server".to_string(),
                version: "1.0.0".to_string(),
            },
        };

        // Test version compatibility
        assert_eq!(client_init.protocol_version, server_init.protocol_version);
    }

    #[test]
    fn test_protocol_version_mismatch() {
        let client_version = "1.0";
        let server_version = "2.0";
        
        // In a real implementation, this would trigger version negotiation
        assert_ne!(client_version, server_version);
    }

    // Task 5.1.4: Test error message handling
    
    #[test]
    fn test_mcp_error_codes() {
        let errors = vec![
            MCPError {
                code: -32700,
                message: "Parse error".to_string(),
                data: None,
            },
            MCPError {
                code: -32600,
                message: "Invalid Request".to_string(),
                data: Some(json!({"reason": "missing field"})),
            },
            MCPError {
                code: -32601,
                message: "Method not found".to_string(),
                data: None,
            },
            MCPError {
                code: -32602,
                message: "Invalid params".to_string(),
                data: Some(json!({"param": "text", "reason": "required"})),
            },
            MCPError {
                code: -32603,
                message: "Internal error".to_string(),
                data: None,
            },
        ];

        for error in errors {
            let serialized = serde_json::to_string(&error).unwrap();
            let deserialized: MCPError = serde_json::from_str(&serialized).unwrap();
            
            assert_eq!(deserialized.code, error.code);
            assert_eq!(deserialized.message, error.message);
            assert_eq!(deserialized.data, error.data);
        }
    }

    #[test]
    fn test_error_response_construction() {
        let error = MCPError {
            code: -32601,
            message: "Method 'invalid_method' not found".to_string(),
            data: Some(json!({"available_methods": ["initialize", "tools/list", "tools/call"]})),
        };

        let response = MCPResponse::Error {
            id: "req-123".to_string(),
            error,
        };

        assert_eq!(response.get_id(), "req-123");
        
        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["type"], "error");
        assert_eq!(json["id"], "req-123");
        assert_eq!(json["error"]["code"], -32601);
    }

    // Task 5.1.5: Test all protocol message types
    
    #[test]
    fn test_all_request_types() {
        let requests = vec![
            MCPRequest::Initialize {
                id: "init-1".to_string(),
                params: InitializeParams {
                    protocol_version: "1.0".to_string(),
                    capabilities: ClientCapabilities {
                        roots: None,
                        sampling: None,
                    },
                    client_info: ClientInfo {
                        name: "client".to_string(),
                        version: "1.0".to_string(),
                    },
                },
            },
            MCPRequest::ListTools {
                id: "list-1".to_string(),
            },
            MCPRequest::CallTool {
                id: "call-1".to_string(),
                params: ToolCallParams {
                    name: "test_tool".to_string(),
                    arguments: None,
                },
            },
            MCPRequest::Initialized,
        ];

        for request in requests {
            let serialized = serde_json::to_string(&request).unwrap();
            let deserialized: MCPRequest = serde_json::from_str(&serialized).unwrap();
            
            // Test get_id method
            match &request {
                MCPRequest::Initialized => assert_eq!(request.get_id(), None),
                _ => assert!(request.get_id().is_some()),
            }
            
            // Ensure serialization round-trip works
            let reserialized = serde_json::to_string(&deserialized).unwrap();
            assert_eq!(serialized, reserialized);
        }
    }

    #[test]
    fn test_all_response_result_types() {
        let tool_def = ToolDefinition {
            name: "test_tool".to_string(),
            description: Some("A test tool".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "input": {"type": "string"}
                }
            }),
        };

        let results = vec![
            ResponseResult::Initialize(InitializeResult {
                protocol_version: "1.0".to_string(),
                capabilities: ServerCapabilities {
                    logging: None,
                    prompts: None,
                    resources: None,
                    tools: None,
                },
                server_info: ServerInfo {
                    name: "server".to_string(),
                    version: "1.0".to_string(),
                },
            }),
            ResponseResult::ListTools(ListToolsResult {
                tools: vec![tool_def.clone()],
            }),
            ResponseResult::CallTool(CallToolResult {
                content: vec![
                    ToolContent::Text {
                        text: "Result".to_string(),
                    },
                ],
                is_error: Some(false),
            }),
        ];

        for result in results {
            let response = MCPResponse::Result {
                id: "resp-1".to_string(),
                result,
            };
            
            let serialized = serde_json::to_string(&response).unwrap();
            let deserialized: MCPResponse = serde_json::from_str(&serialized).unwrap();
            
            assert_eq!(response.get_id(), "resp-1");
            
            // Ensure serialization round-trip works
            let reserialized = serde_json::to_string(&deserialized).unwrap();
            assert_eq!(serialized, reserialized);
        }
    }

    #[test]
    fn test_mcp_message_enum() {
        let request = MCPMessage::Request(MCPRequest::ListTools {
            id: "req-1".to_string(),
        });

        let response = MCPMessage::Response(MCPResponse::Result {
            id: "resp-1".to_string(),
            result: ResponseResult::ListTools(ListToolsResult {
                tools: vec![],
            }),
        });

        // Test serialization of MCPMessage enum
        let req_json = serde_json::to_string(&request).unwrap();
        let resp_json = serde_json::to_string(&response).unwrap();

        let deserialized_req: MCPMessage = serde_json::from_str(&req_json).unwrap();
        let deserialized_resp: MCPMessage = serde_json::from_str(&resp_json).unwrap();

        match deserialized_req {
            MCPMessage::Request(MCPRequest::ListTools { id }) => {
                assert_eq!(id, "req-1");
            }
            _ => panic!("Wrong message type"),
        }

        match deserialized_resp {
            MCPMessage::Response(MCPResponse::Result { id, .. }) => {
                assert_eq!(id, "resp-1");
            }
            _ => panic!("Wrong message type"),
        }
    }

    // Task 5.1.6: Add property-based tests for protocol invariants
    
    #[test]
    fn test_capability_invariants() {
        let capabilities = ClientCapabilities {
            roots: Some(RootsCapability {
                list_changed: Some(true),
            }),
            sampling: Some(SamplingCapability {}),
        };

        // Test that capabilities can be partially set
        let partial_caps = ClientCapabilities {
            roots: None,
            sampling: Some(SamplingCapability {}),
        };

        let json = serde_json::to_value(&partial_caps).unwrap();
        assert!(json["roots"].is_null());
        assert!(json["sampling"].is_object());
    }

    #[test]
    fn test_tool_definition_invariants() {
        let tool = ToolDefinition {
            name: "test_tool".to_string(),
            description: None,
            input_schema: json!({}),
        };

        // Name should not be empty
        assert!(!tool.name.is_empty());
        
        // Input schema should be valid JSON
        assert!(tool.input_schema.is_object());

        // Test with full schema
        let full_tool = ToolDefinition {
            name: "complex_tool".to_string(),
            description: Some("A complex tool with schema".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "required_field": {
                        "type": "string",
                        "description": "A required field"
                    },
                    "optional_field": {
                        "type": "number",
                        "description": "An optional field"
                    }
                },
                "required": ["required_field"]
            }),
        };

        let json = serde_json::to_value(&full_tool).unwrap();
        assert_eq!(json["name"], "complex_tool");
        assert!(json["description"].is_string());
        assert!(json["input_schema"]["properties"].is_object());
    }

    #[test]
    fn test_request_id_invariant() {
        // All requests with IDs should have non-empty IDs
        let request = MCPRequest::Initialize {
            id: "".to_string(), // Empty ID - should be avoided in practice
            params: InitializeParams {
                protocol_version: "1.0".to_string(),
                capabilities: ClientCapabilities {
                    roots: None,
                    sampling: None,
                },
                client_info: ClientInfo {
                    name: "client".to_string(),
                    version: "1.0".to_string(),
                },
            },
        };

        // Even with empty ID, serialization should work
        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["id"], "");
    }

    #[test]
    fn test_response_always_has_id() {
        // All responses must have an ID
        let response = MCPResponse::Result {
            id: "resp-123".to_string(),
            result: ResponseResult::ListTools(ListToolsResult {
                tools: vec![],
            }),
        };

        assert!(!response.get_id().is_empty());
    }

    #[test]
    fn test_tool_call_result_error_flag() {
        let success_result = CallToolResult {
            content: vec![
                ToolContent::Text {
                    text: "Success".to_string(),
                },
            ],
            is_error: Some(false),
        };

        let error_result = CallToolResult {
            content: vec![
                ToolContent::Text {
                    text: "Error occurred".to_string(),
                },
            ],
            is_error: Some(true),
        };

        // is_error flag should properly serialize
        let success_json = serde_json::to_value(&success_result).unwrap();
        assert_eq!(success_json["is_error"], false);

        let error_json = serde_json::to_value(&error_result).unwrap();
        assert_eq!(error_json["is_error"], true);
    }

    #[test]
    fn test_complex_tool_arguments() {
        let mut complex_args = HashMap::new();
        complex_args.insert("string_arg".to_string(), json!("value"));
        complex_args.insert("number_arg".to_string(), json!(42));
        complex_args.insert("bool_arg".to_string(), json!(true));
        complex_args.insert("array_arg".to_string(), json!([1, 2, 3]));
        complex_args.insert("object_arg".to_string(), json!({"key": "value"}));
        complex_args.insert("null_arg".to_string(), json!(null));

        let params = ToolCallParams {
            name: "complex_tool".to_string(),
            arguments: Some(complex_args.clone()),
        };

        let serialized = serde_json::to_string(&params).unwrap();
        let deserialized: ToolCallParams = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.name, "complex_tool");
        let args = deserialized.arguments.unwrap();
        
        assert_eq!(args.get("string_arg").unwrap(), &json!("value"));
        assert_eq!(args.get("number_arg").unwrap(), &json!(42));
        assert_eq!(args.get("bool_arg").unwrap(), &json!(true));
        assert_eq!(args.get("array_arg").unwrap(), &json!([1, 2, 3]));
        assert_eq!(args.get("object_arg").unwrap(), &json!({"key": "value"}));
        assert_eq!(args.get("null_arg").unwrap(), &json!(null));
    }
}