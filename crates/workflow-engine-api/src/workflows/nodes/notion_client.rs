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
use workflow_engine_mcp::clients::notion::{NotionClientNode as MCPNotionClient, NotionConfig};
use crate::workflows::executor::{StepExecutor, WorkflowContext};
use crate::workflows::schema::StepDefinition;

/// Workflow-compatible Notion client node
pub struct WorkflowNotionClientNode {
    client: MCPNotionClient,
}

impl WorkflowNotionClientNode {
    pub fn new(config: NotionConfig) -> Self {
        Self {
            client: MCPNotionClient::new(config),
        }
    }
    
    /// Create a notion client from environment variables
    pub fn from_env() -> Result<Self, WorkflowError> {
        let base_url = std::env::var("NOTION_MCP_URL")
            .unwrap_or_else(|_| "http://localhost:3001".to_string());
        let api_key = std::env::var("NOTION_API_KEY").ok();
        
        let config = NotionConfig::new_http(base_url, api_key);
        Ok(Self::new(config))
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
        
        // Create a mutable copy of the client for async operations
        let mut client = MCPNotionClient::new(self.client.config.clone());
        
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
        
        // Handle different content types
        let result = if let Some(content_def) = input_obj.get("content") {
            match content_def {
                // Simple string content
                Value::String(content) => {
                    let parent_id = input_obj.get("parent_id")
                        .and_then(|v| v.as_str());
                    
                    client.create_page(title, content, parent_id).await?
                }
                
                // Structured content object
                Value::Object(content_obj) => {
                    if content_obj.get("type").and_then(|v| v.as_str()) == Some("template") {
                        // Handle templated research content
                        let template_name = content_obj.get("template")
                            .and_then(|v| v.as_str())
                            .unwrap_or("default");
                        
                        let default_data = Value::Object(Default::default());
                        let data = content_obj.get("data")
                            .unwrap_or(&default_data);
                        
                        match template_name {
                            "notion_research_page" => {
                                self.create_research_page(&mut client, title, data, input_obj).await?
                            }
                            _ => {
                                // Fallback to simple page creation
                                let content = serde_json::to_string_pretty(data)
                                    .unwrap_or_else(|_| "No content available".to_string());
                                let parent_id = input_obj.get("parent_id")
                                    .and_then(|v| v.as_str());
                                
                                client.create_page(title, &content, parent_id).await?
                            }
                        }
                    } else {
                        // Regular content object - serialize as JSON
                        let content = serde_json::to_string_pretty(content_def)
                            .unwrap_or_else(|_| "Invalid content".to_string());
                        let parent_id = input_obj.get("parent_id")
                            .and_then(|v| v.as_str());
                        
                        client.create_page(title, &content, parent_id).await?
                    }
                }
                
                _ => {
                    return Err(WorkflowError::InvalidInput(
                        "Content must be a string or object".to_string()
                    ));
                }
            }
        } else {
            return Err(WorkflowError::InvalidInput(
                "Missing required field: content".to_string()
            ));
        };
        
        // No need to disconnect - client handles this automatically
        
        log::info!("Notion step {} completed successfully", step.id);
        
        Ok(result)
    }
}

impl WorkflowNotionClientNode {
    /// Create a research documentation page using the specialized method
    async fn create_research_page(
        &self,
        client: &mut MCPNotionClient,
        title: &str,
        data: &Value,
        input_obj: &serde_json::Map<String, Value>,
    ) -> Result<Value, WorkflowError> {
        let data_obj = data.as_object()
            .ok_or_else(|| WorkflowError::InvalidInput("Template data must be an object".to_string()))?;
        
        // Extract research data
        let summary = data_obj.get("summary")
            .and_then(|v| v.as_str())
            .unwrap_or("No summary available");
        
        let key_points = if let Some(points) = data_obj.get("key_points") {
            match points {
                Value::Array(arr) => {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect()
                }
                Value::String(s) => vec![s.clone()],
                _ => vec![]
            }
        } else {
            vec![]
        };
        
        let sources = if let Some(sources) = data_obj.get("sources") {
            match sources {
                Value::Array(arr) => arr.clone(),
                _ => vec![sources.clone()]
            }
        } else {
            vec![]
        };
        
        let parent_id = input_obj.get("parent_id")
            .and_then(|v| v.as_str());
        
        // Extract properties if provided
        let properties = input_obj.get("properties")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect::<HashMap<String, Value>>()
            });
        
        client.create_research_page(
            title,
            summary,
            &key_points,
            &sources,
            parent_id,
            properties,
        ).await
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