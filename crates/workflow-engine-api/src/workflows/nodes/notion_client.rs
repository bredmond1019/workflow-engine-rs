/*!
# Notion Client Workflow Node

This module implements a workflow-compatible NotionClientNode that can
be used in declarative workflows to create documentation pages.

Task 2.3: Implement NotionClientNode for creating documentation pages
*/

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

use workflow_engine_core::error::WorkflowError;
use workflow_engine_mcp::clients::{McpClient, HttpMcpClient};
use crate::workflows::executor::{StepExecutor, WorkflowContext};
use crate::workflows::schema::StepDefinition;

/// Workflow-compatible Notion client node
pub struct WorkflowNotionClientNode {
    base_url: String,
}

impl WorkflowNotionClientNode {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
        }
    }
    
    /// Create a notion client from environment variables
    pub fn from_env() -> Result<Self, WorkflowError> {
        let base_url = std::env::var("NOTION_MCP_URL")
            .unwrap_or_else(|_| "http://localhost:8002".to_string());
        
        Ok(Self::new(base_url))
    }
}

#[async_trait]
impl StepExecutor for WorkflowNotionClientNode {
    async fn execute(
        &self,
        step: &StepDefinition,
        context: &WorkflowContext,
    ) -> Result<Value, WorkflowError> {
        log::info!("Executing Notion workflow step: {}", step.id);
        
        // Create a mutable client for async operations
        let mut client = HttpMcpClient::new(self.base_url.clone());
        
        // Render step input with current context
        let template_engine = context.create_template_engine();
        let rendered_input = template_engine.render_json(&step.input)?;
        
        log::debug!("Rendered input for Notion step {}: {}", step.id, rendered_input);
        
        // Extract parameters from rendered input
        let input_obj = rendered_input.as_object()
            .ok_or_else(|| WorkflowError::InvalidInput("Input must be a JSON object".to_string()))?;
        
        let title = input_obj.get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| WorkflowError::InvalidInput("Missing required field: title".to_string()))?;
        
        // Connect and initialize the client
        client.connect().await?;
        client.initialize("workflow-engine", "1.0").await?;
        
        // Prepare arguments for the MCP tool call
        let mut arguments = HashMap::new();
        arguments.insert("title".to_string(), Value::String(title.to_string()));
        
        if let Some(content) = input_obj.get("content") {
            arguments.insert("content".to_string(), content.clone());
        } else {
            return Err(WorkflowError::InvalidInput(
                "Missing required field: content".to_string()
            ));
        }
        
        if let Some(parent_id) = input_obj.get("parent_id") {
            arguments.insert("parent_id".to_string(), parent_id.clone());
        }
        
        if let Some(properties) = input_obj.get("properties") {
            arguments.insert("properties".to_string(), properties.clone());
        }
        
        // Call the create_page tool
        let result = client.call_tool("create_page", Some(arguments)).await?;
        
        // Disconnect the client
        client.disconnect().await.ok(); // Ignore disconnect errors
        
        log::info!("Notion step {} completed successfully", step.id);
        
        // Convert the result to JSON
        serde_json::to_value(result).map_err(|e| WorkflowError::SerializationError {
            message: format!("Failed to serialize MCP result: {}", e),
        })
    }
}


/// Factory function for creating NotionClientNode executors
pub fn create_notion_executor() -> Result<Box<dyn StepExecutor>, WorkflowError> {
    let node = WorkflowNotionClientNode::from_env()?;
    Ok(Box::new(node))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflows::schema::{StepDefinition, StepType, StepConfig};
    use crate::workflows::executor::{WorkflowContext, TemplateEngine};
    use crate::workflows::schema::{WorkflowInstance, WorkflowDefinition, WorkflowStatus};
    use std::collections::HashMap;
    
    fn create_test_workflow_context() -> WorkflowContext {
        let workflow = WorkflowDefinition {
            name: "test_workflow".to_string(),
            description: "Test workflow".to_string(),
            version: "1.0".to_string(),
            inputs: HashMap::new(),
            steps: vec![],
            outputs: HashMap::new(),
            config: Default::default(),
        };
        
        let instance = WorkflowInstance {
            id: uuid::Uuid::new_v4(),
            workflow,
            status: WorkflowStatus::Running,
            inputs: serde_json::json!({"topic": "test topic"}),
            steps: HashMap::new(),
            outputs: None,
            created_at: chrono::Utc::now(),
            started_at: Some(chrono::Utc::now()),
            completed_at: None,
            error: None,
        };
        
        WorkflowContext::new(instance)
    }
    
    #[test]
    fn test_template_engine_integration() {
        let mut engine = TemplateEngine::new();
        engine = engine.with_context("topic".to_string(), Value::String("machine learning".to_string()));
        
        let result = engine.render("Research: {{ topic }}").unwrap();
        assert_eq!(result, "Research: machine learning");
    }
    
    #[tokio::test]
    async fn test_notion_step_structure() {
        // This test validates the step structure without actually calling Notion
        let step = StepDefinition {
            id: "create_notion_page".to_string(),
            name: Some("Create Documentation Page".to_string()),
            step_type: StepType::Node {
                node: "NotionClientNode".to_string(),
            },
            input: serde_json::json!({
                "title": "Test Research Page",
                "content": {
                    "type": "template",
                    "template": "notion_research_page",
                    "data": {
                        "topic": "machine learning",
                        "summary": "A comprehensive overview of machine learning concepts",
                        "key_points": ["Supervised learning", "Unsupervised learning"],
                        "sources": [
                            {"title": "ML Basics", "url": "https://example.com/ml"},
                            {"title": "Advanced ML", "url": "https://example.com/advanced"}
                        ]
                    }
                },
                "parent_id": "test-parent-id",
                "properties": {
                    "Topic": "machine learning",
                    "Difficulty": "intermediate"
                }
            }),
            depends_on: vec![],
            parallel: false,
            config: StepConfig::default(),
        };
        
        let context = create_test_workflow_context();
        
        // Validate step structure
        assert_eq!(step.id, "create_notion_page");
        assert!(matches!(step.step_type, StepType::Node { .. }));
        
        // Validate input structure
        let input_obj = step.input.as_object().unwrap();
        assert!(input_obj.contains_key("title"));
        assert!(input_obj.contains_key("content"));
        
        // Validate content structure
        let content = input_obj.get("content").unwrap().as_object().unwrap();
        assert_eq!(content.get("type").unwrap().as_str().unwrap(), "template");
        assert_eq!(content.get("template").unwrap().as_str().unwrap(), "notion_research_page");
        
        // Test template rendering
        let template_engine = context.create_template_engine();
        let rendered = template_engine.render_json(&step.input).unwrap();
        
        // Should preserve the structure
        assert!(rendered.as_object().unwrap().contains_key("title"));
    }
}