//! Stub types for MCP functionality to avoid circular dependencies
//! These are minimal implementations to allow the core crate to compile
//! independently. Full MCP types are in the workflow-engine-mcp crate.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Minimal tool definition for core functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: Option<Value>,
}

/// Minimal tool result for core functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolResult {
    pub content: Vec<ToolContent>,
    pub is_error: Option<bool>,
}

/// Tool content variants
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { 
        data: String,
        mime_type: String,
    },
    #[serde(rename = "resource")]
    Resource {
        resource: ResourceContent,
    },
}

/// Resource content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    pub uri: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

impl ToolDefinition {
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            input_schema: None,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_schema(mut self, schema: Value) -> Self {
        self.input_schema = Some(schema);
        self
    }
}

impl CallToolResult {
    pub fn success(text: String) -> Self {
        Self {
            content: vec![ToolContent::Text { text }],
            is_error: Some(false),
        }
    }

    pub fn error(error_text: String) -> Self {
        Self {
            content: vec![ToolContent::Text { text: error_text }],
            is_error: Some(true),
        }
    }
}