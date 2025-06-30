use crate::server::ToolMetadata;
use workflow_engine_core::{error::WorkflowError, nodes::Node, task::TaskContext};
// Note: Uses template-based response generation instead of AI agent
// AI agent integration can be added later when AnthropicAgentNode is available
use workflow_engine_core::nodes::agent::{AgentConfig, ModelProvider};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::any::TypeId;

use super::{super::server::CustomerSupportMcpServer, CustomerCareEventData};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerResponse {
    pub response: String,
    pub tone: String,
    pub key_points: Vec<String>,
    pub generated_at: String,
}

/// Generate a customer support response based on ticket analysis
#[derive(Debug)]
pub struct GenerateCustomerResponseNode {
    pub metadata: ToolMetadata,
}

impl Default for GenerateCustomerResponseNode {
    fn default() -> Self {
        Self::new()
    }
}

impl GenerateCustomerResponseNode {
    pub fn new() -> Self {
        Self {
            metadata: ToolMetadata {
                name: "generate_customer_response".to_string(),
                description: "Generates a customer support response based on ticket analysis and intent".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "required": ["customer_event", "intent", "reasoning"],
                    "properties": {
                        "customer_event": {
                            "type": "object",
                            "description": "Original customer support event data"
                        },
                        "intent": {
                            "type": "string",
                            "description": "Determined intent of the ticket"
                        },
                        "reasoning": {
                            "type": "string",
                            "description": "Reasoning behind the intent determination"
                        }
                    }
                }),
                node_type: TypeId::of::<GenerateCustomerResponseNode>(),
            },
        }
    }

    pub fn node_name(&self) -> String {
        self.metadata.name.clone()
    }

    pub fn as_tool_metadata(&self) -> ToolMetadata {
        self.metadata.clone()
    }
}

impl Node for GenerateCustomerResponseNode {
    fn process(&self, mut task_context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let customer_event = task_context
            .get_data::<CustomerCareEventData>("customer_event")?
            .ok_or_else(|| WorkflowError::validation_error("Missing customer_event data", "customer_event", "required field", "in generate_response node"))?;

        let intent = task_context
            .get_data::<String>("intent")?
            .ok_or_else(|| WorkflowError::validation_error("Missing intent data", "intent", "required field", "in generate_response node"))?;

        let reasoning = task_context
            .get_data::<String>("reasoning")?
            .ok_or_else(|| WorkflowError::validation_error("Missing reasoning data", "reasoning", "required field", "in generate_response node"))?;

        let response = self.generate_response(customer_event, &intent, &reasoning)?;

        task_context.update_node(&self.node_name(), response);
        Ok(task_context)
    }
}

impl GenerateCustomerResponseNode {
    fn generate_response(
        &self,
        customer_event: CustomerCareEventData,
        intent: &str,
        reasoning: &str,
    ) -> Result<CustomerResponse, WorkflowError> {
        // Provides template-based response generation as a functional fallback
        // Can be enhanced with AI agent integration when AnthropicAgentNode becomes available
        let (response, tone, key_points) = match intent {
            "RefundRequest" => (
                format!(
                    "Dear Customer,\n\nThank you for contacting us regarding your refund request (Ticket #{}).\n\nI understand your concern and have escalated your request to our billing department for immediate review. They will process your refund request within 2-3 business days.\n\nYou will receive a confirmation email once the refund has been initiated. The funds should appear in your account within 5-7 business days after processing.\n\nIs there anything else I can help you with today?\n\nBest regards,\nCustomer Support Team",
                    customer_event.ticket_id
                ),
                "empathetic and professional",
                vec![
                    "Acknowledged refund request".to_string(),
                    "Escalated to billing department".to_string(),
                    "Provided timeline for processing".to_string(),
                ]
            ),
            "BillingInvoice" => (
                format!(
                    "Dear Customer,\n\nThank you for reaching out about your billing inquiry (Ticket #{}).\n\nI'd be happy to help clarify your billing concerns. Based on your message, I can see you have questions about your invoice.\n\nI've reviewed your account and can provide the following information:\n- Your current billing cycle ends on the last day of each month\n- Invoices are generated on the 1st of each month\n- Payment is automatically processed using your preferred payment method\n\nIf you need specific details about charges or would like to update your billing information, please let me know.\n\nBest regards,\nCustomer Support Team",
                    customer_event.ticket_id
                ),
                "helpful and informative",
                vec![
                    "Addressed billing inquiry".to_string(),
                    "Provided billing cycle information".to_string(),
                    "Offered further assistance".to_string(),
                ]
            ),
            "ProductQuestion" => (
                format!(
                    "Dear Customer,\n\nThank you for your question (Ticket #{}).\n\nI'm happy to help you with our product. Based on your inquiry, here's the information you need:\n\nOur product offers several features that might address your needs. I recommend checking our knowledge base for detailed guides and tutorials. You can also explore our video tutorials for step-by-step instructions.\n\nIf you need more specific guidance or encounter any issues, please don't hesitate to provide more details about what you're trying to achieve.\n\nBest regards,\nCustomer Support Team",
                    customer_event.ticket_id
                ),
                "friendly and supportive",
                vec![
                    "Acknowledged product question".to_string(),
                    "Directed to knowledge base".to_string(),
                    "Offered additional support".to_string(),
                ]
            ),
            _ => (
                format!(
                    "Dear Customer,\n\nThank you for contacting us (Ticket #{}).\n\nI've received your message and I'm here to help. Your inquiry is important to us, and I want to ensure I provide you with the most accurate information.\n\nCould you please provide a bit more detail about your specific needs? This will help me give you the best possible assistance.\n\nI'm looking forward to helping you resolve this matter.\n\nBest regards,\nCustomer Support Team",
                    customer_event.ticket_id
                ),
                "professional and attentive",
                vec![
                    "Acknowledged inquiry".to_string(),
                    "Requested additional information".to_string(),
                    "Expressed willingness to help".to_string(),
                ]
            ),
        };

        Ok(CustomerResponse {
            response: response.to_string(),
            tone: tone.to_string(),
            key_points,
            generated_at: Utc::now().to_rfc3339(),
        })
    }
}

/// Register this node as a tool in the MCP server
pub fn register_generate_response_tool(server: &CustomerSupportMcpServer) -> Result<(), WorkflowError> {
    let node = Arc::new(GenerateCustomerResponseNode::new());
    let metadata = node.as_tool_metadata();
    
    tokio::runtime::Handle::current()
        .block_on(server.register_node_as_tool(node, metadata))
}