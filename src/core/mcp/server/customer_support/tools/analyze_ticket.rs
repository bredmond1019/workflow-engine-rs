use crate::core::mcp::server::ToolMetadata;
use crate::core::{error::WorkflowError, nodes::Node, task::TaskContext};
use crate::core::ai_agents::anthropic::AnthropicAgentNode;
use crate::core::nodes::agent::{AgentConfig, ModelProvider};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

use super::{super::server::CustomerSupportMCPServer, CustomerCareEventData};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketAnalysis {
    pub sentiment: String,
    pub urgency: String,
    pub category: String,
    pub complexity: String,
    pub customer_emotion: String,
    pub key_issues: Vec<String>,
    pub suggested_actions: Vec<String>,
}

#[derive(Debug)]
pub struct AnalyzeTicketNode;

impl AnalyzeTicketNode {
    pub fn new() -> Self {
        Self
    }

    pub async fn register(server: &mut CustomerSupportMCPServer) -> Result<(), WorkflowError> {
        let node = Arc::new(Self::new());
        let metadata = ToolMetadata::new(
            "analyze_ticket".to_string(),
            "Performs comprehensive analysis of customer support ticket".to_string(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "context_data": {
                        "type": "object",
                        "properties": {
                            "ticket_id": {"type": "string", "description": "Ticket identifier"},
                            "message": {"type": "string", "description": "Customer message"},
                            "priority": {"type": "string", "description": "Ticket priority"},
                            "customer_data": {"type": "object", "description": "Customer information"}
                        },
                        "required": ["ticket_id", "message"]
                    }
                },
                "required": ["context_data"]
            }),
            std::any::TypeId::of::<Self>(),
        );
        server
            .get_server()
            .register_node_as_tool(node, metadata)
            .await?;
        Ok(())
    }
}

impl Node for AnalyzeTicketNode {
    fn process(&self, mut task_context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let event_data: CustomerCareEventData = task_context.get_event_data()?;

        // Create AI agent configuration for ticket analysis
        let agent_config = AgentConfig {
            system_prompt: "You are an expert customer support analyst. Analyze customer support tickets to extract sentiment, urgency, category, and other relevant metrics. Respond with a JSON object containing your analysis.".to_string(),
            model_provider: ModelProvider::Anthropic,
            model_name: "claude-3-sonnet-20240229".to_string(),
            mcp_server_uri: None,
        };

        // Create the AI agent
        let ai_agent = AnthropicAgentNode::new(agent_config);

        // Prepare the analysis prompt
        let analysis_prompt = format!(
            "Please analyze this customer support ticket and provide a JSON response with the following fields:
            - sentiment: (positive, negative, neutral, frustrated, confused)
            - urgency: (low, medium, high, critical)
            - category: (billing, technical, product_inquiry, refund_request, complaint, compliment, general_inquiry)
            - complexity: (simple, moderate, complex)
            - customer_emotion: brief description
            - key_issues: array of main issues mentioned
            - suggested_actions: array of recommended next steps
            
            Ticket Details:
            - Ticket ID: {}
            - Customer ID: {}
            - Priority: {}
            - Message: {}
            
            Respond only with valid JSON.",
            event_data.ticket_id,
            event_data.customer_id,
            event_data.priority,
            event_data.message
        );

        // Create a task context for the AI agent
        let mut ai_task_context = TaskContext::new(
            "ticket_analysis".to_string(),
            serde_json::json!({
                "prompt": analysis_prompt,
                "ticket_data": event_data
            })
        );

        // Process with AI agent
        ai_task_context = ai_agent.process(ai_task_context)?;

        // Extract AI response
        let ai_response = ai_task_context.get_data::<serde_json::Value>("ai_response")?;
        let analysis_text = ai_response
            .and_then(|v| v["response"].as_str().map(|s| s.to_string()))
            .ok_or_else(|| WorkflowError::ProcessingError {
                message: "Failed to extract analysis from AI response".to_string(),
            })?;

        // Parse the AI response as JSON
        let analysis: TicketAnalysis = serde_json::from_str(&analysis_text)
            .map_err(|e| WorkflowError::DeserializationError {
                message: format!("Failed to parse AI analysis as JSON: {}. Raw response: {}", e, analysis_text),
            })?;

        // Store the analysis in the task context
        task_context.update_node(&self.node_name(), serde_json::json!({
            "analysis": analysis,
            "ticket_id": event_data.ticket_id,
            "customer_id": event_data.customer_id,
            "analyzed_at": chrono::Utc::now(),
            "ai_model": "claude-3-sonnet-20240229"
        }));

        Ok(task_context)
    }
}
