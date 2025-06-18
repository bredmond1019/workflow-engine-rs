use crate::core::mcp::server::ToolMetadata;
use crate::core::{error::WorkflowError, nodes::Node, task::TaskContext};
use crate::core::ai_agents::anthropic::AnthropicAgentNode;
use crate::core::nodes::agent::{AgentConfig, ModelProvider};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::{super::server::CustomerSupportMCPServer, CustomerCareEventData};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustomerIntent {
    GeneralQuestion,
    ProductQuestion,
    BillingInvoice,
    RefundRequest,
}

impl CustomerIntent {
    pub fn requires_escalation(&self) -> bool {
        matches!(self, CustomerIntent::RefundRequest)
    }
}

#[derive(Debug)]
pub struct DetermineTicketIntentNode;

impl DetermineTicketIntentNode {
    pub fn new() -> Self {
        Self
    }

    pub async fn register(server: &mut CustomerSupportMCPServer) -> Result<(), WorkflowError> {
        let node = Arc::new(Self::new());
        let metadata = ToolMetadata::new(
            "determine_intent".to_string(),
            "Analyzes customer message to determine the intent and categorize the request"
                .to_string(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "context_data": {
                        "type": "object",
                        "properties": {
                            "message": {"type": "string", "description": "Customer message to analyze"},
                            "ticket_history": {"type": "array", "description": "Previous ticket interactions (optional)"}
                        },
                        "required": ["message"]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentAnalysis {
    pub reasoning: String,
    pub intent: CustomerIntent,
    pub confidence: f32,
    pub escalate: bool,
}

impl Node for DetermineTicketIntentNode {
    fn process(&self, mut task_context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let event_data: CustomerCareEventData = task_context.get_event_data()?;

        let analysis = self.determine_intent(event_data)?;

        task_context.update_node(&self.node_name(), analysis);
        Ok(task_context)
    }
}

impl DetermineTicketIntentNode {
    fn determine_intent(
        &self,
        customer_event: CustomerCareEventData,
    ) -> Result<IntentAnalysis, WorkflowError> {
        // Create AI agent configuration for intent determination
        let agent_config = AgentConfig {
            system_prompt: "You are an expert at analyzing customer support messages to determine their intent. Classify customer intents accurately and provide reasoning for your decisions. Respond with valid JSON only.".to_string(),
            model_provider: ModelProvider::Anthropic,
            model_name: "claude-3-sonnet-20240229".to_string(),
            mcp_server_uri: None,
        };

        // Create the AI agent
        let ai_agent = AnthropicAgentNode::new(agent_config);

        // Prepare the intent analysis prompt
        let intent_prompt = format!(
            "Analyze this customer message and determine the intent. Respond with a JSON object containing:\n            - reasoning: explanation of your decision (string)\n            - intent: one of [\"GeneralQuestion\", \"ProductQuestion\", \"BillingInvoice\", \"RefundRequest\"]\n            - confidence: confidence score between 0.0 and 1.0 (number)\n            - escalate: whether this requires escalation (boolean)\n            \n            Customer Information:\n            - Ticket ID: {}\n            - Customer ID: {}\n            - Priority: {}\n            - Message: {}\n            \n            Consider these guidelines:\n            - BillingInvoice: payment issues, billing questions, invoice problems\n            - RefundRequest: explicit refund requests (requires escalation)\n            - ProductQuestion: how-to questions, feature inquiries, technical issues\n            - GeneralQuestion: general inquiries, compliments, other topics\n            \n            Respond only with valid JSON.",
            customer_event.ticket_id,
            customer_event.customer_id,
            customer_event.priority,
            customer_event.message
        );

        // Create a task context for the AI agent
        let mut ai_task_context = TaskContext::new(
            "intent_determination".to_string(),
            serde_json::json!({
                "prompt": intent_prompt,
                "ticket_data": customer_event
            })
        );

        // Process with AI agent
        ai_task_context = ai_agent.process(ai_task_context)?;

        // Extract AI response
        let ai_response = ai_task_context.get_data::<serde_json::Value>("ai_response")?;
        let intent_text = ai_response
            .and_then(|v| v["response"].as_str().map(|s| s.to_string()))
            .ok_or_else(|| WorkflowError::ProcessingError {
                message: "Failed to extract intent from AI response".to_string(),
            })?;

        // Parse the AI response as JSON
        let mut analysis: IntentAnalysis = serde_json::from_str(&intent_text)
            .map_err(|e| WorkflowError::DeserializationError {
                message: format!("Failed to parse AI intent analysis as JSON: {}. Raw response: {}", e, intent_text),
            })?;

        // Intent is already parsed and validated during deserialization
        
        // Set escalation based on intent requirements
        analysis.escalate = analysis.intent.requires_escalation() || analysis.escalate;
        
        // Ensure confidence is within valid range
        analysis.confidence = analysis.confidence.clamp(0.0, 1.0);

        Ok(analysis)
    }

    fn validate_intent(&self, intent_str: &str) -> Result<CustomerIntent, WorkflowError> {
        match intent_str {
            "GeneralQuestion" => Ok(CustomerIntent::GeneralQuestion),
            "ProductQuestion" => Ok(CustomerIntent::ProductQuestion),
            "BillingInvoice" => Ok(CustomerIntent::BillingInvoice),
            "RefundRequest" => Ok(CustomerIntent::RefundRequest),
            _ => {
                log::warn!("Unknown intent: {}, defaulting to GeneralQuestion", intent_str);
                Ok(CustomerIntent::GeneralQuestion)
            }
        }
    }
}
