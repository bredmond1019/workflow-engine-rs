use crate::core::mcp::server::ToolMetadata;
use crate::core::{error::WorkflowError, nodes::Node, task::TaskContext};
use crate::core::ai_agents::anthropic::AnthropicAgentNode;
use crate::core::nodes::agent::{AgentConfig, ModelProvider};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::{super::server::CustomerSupportMCPServer, CustomerCareEventData};

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneratedResponse {
    pub subject: String,
    pub body: String,
    pub tone: String,
    pub response_type: String,
    pub confidence: f32,
    pub estimated_resolution_time: String,
}

#[derive(Debug, Clone)]
pub struct GenerateResponseNode;

impl GenerateResponseNode {
    pub fn new() -> Self {
        Self
    }

    pub async fn register(server: &mut CustomerSupportMCPServer) -> Result<(), WorkflowError> {
        let node = Arc::new(Self::new());
        let metadata = ToolMetadata::new(
            "generate_response".to_string(),
            "Generates appropriate response to customer inquiry based on analysis".to_string(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "context_data": {
                        "type": "object",
                        "properties": {
                            "customer_message": {"type": "string", "description": "Original customer message"},
                            "intent": {"type": "string", "description": "Determined customer intent"},
                            "analysis": {"type": "object", "description": "Ticket analysis results"},
                            "tone": {"type": "string", "enum": ["formal", "friendly", "apologetic"], "description": "Response tone"}
                        },
                        "required": ["customer_message", "intent"]
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

impl Node for GenerateResponseNode {
    fn process(&self, mut task_context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let event_data: CustomerCareEventData = task_context.get_event_data()?;
        
        // Get previous analysis results from the task context
        let ticket_analysis: serde_json::Value = task_context.get_node_data("analyze_ticket")?
            .ok_or_else(|| WorkflowError::ProcessingError { message: "Missing ticket analysis data".to_string() })?;
        let intent_analysis: serde_json::Value = task_context.get_node_data("determine_intent")?
            .ok_or_else(|| WorkflowError::ProcessingError { message: "Missing intent analysis data".to_string() })?;
        let spam_check: serde_json::Value = task_context.get_node_data("filter_spam")?
            .ok_or_else(|| WorkflowError::ProcessingError { message: "Missing spam check data".to_string() })?;
        
        // Generate the response using AI
        let generated_response = self.generate_ai_response(
            &event_data,
            &ticket_analysis,
            &intent_analysis,
            &spam_check,
        )?;

        task_context.update_node(&self.node_name(), serde_json::json!({
            "response": generated_response,
            "generated_at": Utc::now(),
            "model_used": "claude-3-sonnet-20240229"
        }));
        
        Ok(task_context)
    }
}

impl GenerateResponseNode {
    /// Generate AI-powered response based on ticket analysis
    fn generate_ai_response(
        &self,
        event_data: &CustomerCareEventData,
        ticket_analysis: &serde_json::Value,
        intent_analysis: &serde_json::Value,
        spam_check: &serde_json::Value,
    ) -> Result<GeneratedResponse, WorkflowError> {
        // Create AI agent configuration for response generation
        let agent_config = AgentConfig {
            system_prompt: "You are an expert customer support representative. Generate helpful, professional, and empathetic responses to customer inquiries. Always maintain a friendly tone while being informative and solution-oriented. Respond with valid JSON only.".to_string(),
            model_provider: ModelProvider::Anthropic,
            model_name: "claude-3-sonnet-20240229".to_string(),
            mcp_server_uri: None,
        };

        // Create the AI agent
        let ai_agent = AnthropicAgentNode::new(agent_config);

        // Extract key information from analyses
        let sentiment = ticket_analysis
            .get("analysis")
            .and_then(|a| a.get("sentiment"))
            .and_then(|s| s.as_str())
            .unwrap_or("neutral");
            
        let urgency = ticket_analysis
            .get("analysis")
            .and_then(|a| a.get("urgency"))
            .and_then(|u| u.as_str())
            .unwrap_or("medium");
            
        let category = ticket_analysis
            .get("analysis")
            .and_then(|a| a.get("category"))
            .and_then(|c| c.as_str())
            .unwrap_or("general_inquiry");
            
        let intent = intent_analysis
            .get("intent")
            .and_then(|i| i.as_str())
            .unwrap_or("GeneralQuestion");
            
        let confidence = intent_analysis
            .get("confidence")
            .and_then(|c| c.as_f64())
            .unwrap_or(0.5) as f32;

        let is_spam = spam_check
            .get("is_spam")
            .and_then(|s| s.as_bool())
            .unwrap_or(false);

        // Don't generate responses for spam
        if is_spam {
            return Ok(GeneratedResponse {
                subject: "Re: Your Inquiry".to_string(),
                body: "Thank you for your message. We have received your inquiry and will review it according to our policies.".to_string(),
                tone: "formal".to_string(),
                response_type: "spam_response".to_string(),
                confidence: 1.0,
                estimated_resolution_time: "N/A".to_string(),
            });
        }

        // Prepare the response generation prompt
        let response_prompt = format!(
            "Generate a customer support response with the following JSON structure:
            {{
                \"subject\": \"Appropriate email subject line\",
                \"body\": \"Complete response body with helpful information\",
                \"tone\": \"friendly\", 
                \"response_type\": \"solution_provided|information_requested|escalation_needed|acknowledgment\",
                \"confidence\": 0.95,
                \"estimated_resolution_time\": \"immediate|2-4 hours|24-48 hours|3-5 business days\"
            }}
            
            Customer Information:
            - Ticket ID: {}
            - Customer ID: {}
            - Priority: {}
            - Original Message: {}
            
            Analysis Results:
            - Sentiment: {}
            - Urgency: {}
            - Category: {}
            - Intent: {}
            - Intent Confidence: {}
            
            Guidelines:
            - For billing issues: Provide clear next steps and relevant policy information
            - For technical issues: Offer troubleshooting steps and escalation if needed
            - For product questions: Provide comprehensive information and helpful resources
            - For refund requests: Acknowledge the request and explain the process
            - Match the tone to the customer's sentiment (empathetic if frustrated, professional if neutral)
            - Always end with an offer for further assistance
            
            Respond only with valid JSON.",
            event_data.ticket_id,
            event_data.customer_id,
            event_data.priority,
            event_data.message,
            sentiment,
            urgency,
            category,
            intent,
            confidence
        );

        // Create a task context for the AI agent
        let mut ai_task_context = TaskContext::new(
            "response_generation".to_string(),
            serde_json::json!({
                "prompt": response_prompt,
                "ticket_data": event_data,
                "analysis_data": {
                    "sentiment": sentiment,
                    "urgency": urgency,
                    "category": category,
                    "intent": intent
                }
            })
        );

        // Process with AI agent
        ai_task_context = ai_agent.process(ai_task_context)?;

        // Extract AI response
        let ai_response = ai_task_context.get_data::<serde_json::Value>("ai_response")?;
        let response_text = ai_response
            .and_then(|v| v["response"].as_str().map(|s| s.to_string()))
            .ok_or_else(|| WorkflowError::ProcessingError {
                message: "Failed to extract response from AI response".to_string(),
            })?;

        // Parse the AI response as JSON
        let generated_response: GeneratedResponse = serde_json::from_str(&response_text)
            .map_err(|e| WorkflowError::DeserializationError {
                message: format!("Failed to parse AI response as JSON: {}. Raw response: {}", e, response_text),
            })?;

        Ok(generated_response)
    }
}
