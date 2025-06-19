use crate::server::ToolMetadata;
use workflow_engine_core::{error::WorkflowError, nodes::Node, task::TaskContext};
// TODO: AnthropicAgentNode needs to be reimplemented or imported from correct location
// use workflow_engine_core::ai::anthropic::AnthropicAgentNode;
use workflow_engine_core::nodes::agent::{AgentConfig, ModelProvider};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::any::TypeId;

use super::{super::server::CustomerSupportMCPServer, CustomerCareEventData};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentAnalysis {
    pub intent: String,
    pub confidence: f64,
    pub reasoning: String,
    pub escalate: bool,
}

/// Determine the intent of a customer support ticket
#[derive(Debug)]
pub struct DetermineTicketIntentNode {
    pub metadata: ToolMetadata,
}

impl Default for DetermineTicketIntentNode {
    fn default() -> Self {
        Self::new()
    }
}

impl DetermineTicketIntentNode {
    pub fn new() -> Self {
        Self {
            metadata: ToolMetadata {
                name: "determine_ticket_intent".to_string(),
                description: "Analyzes customer support tickets to determine their intent category (GeneralQuestion, ProductQuestion, BillingInvoice, or RefundRequest)".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "required": ["customer_event"],
                    "properties": {
                        "customer_event": {
                            "type": "object",
                            "description": "Customer support event data",
                            "required": ["ticket_id", "customer_id", "message", "priority"],
                            "properties": {
                                "ticket_id": { "type": "string" },
                                "customer_id": { "type": "string" },
                                "message": { "type": "string" },
                                "priority": { "type": "string" }
                            }
                        }
                    }
                }),
                node_type: TypeId::of::<DetermineTicketIntentNode>(),
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

impl Node for DetermineTicketIntentNode {
    fn process(&self, mut task_context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let event_data = task_context
            .get_data::<CustomerCareEventData>("customer_event")?
            .ok_or_else(|| WorkflowError::ValidationError {
                message: "Missing customer_event data".to_string(),
            })?;

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
        // TODO: This is a stub implementation. The actual implementation requires AnthropicAgentNode
        // which is not currently available. This should be reimplemented once AI capabilities are restored.
        
        // For now, provide a simple rule-based intent detection
        let message_lower = customer_event.message.to_lowercase();
        
        let (intent, confidence, reasoning, escalate) = if message_lower.contains("refund") {
            ("RefundRequest", 0.9, "Message contains 'refund' keyword", true)
        } else if message_lower.contains("billing") || message_lower.contains("invoice") || message_lower.contains("payment") {
            ("BillingInvoice", 0.85, "Message contains billing-related keywords", false)
        } else if message_lower.contains("how to") || message_lower.contains("feature") || message_lower.contains("technical") {
            ("ProductQuestion", 0.8, "Message contains product-related keywords", false)
        } else {
            ("GeneralQuestion", 0.7, "No specific keywords detected", false)
        };
        
        Ok(IntentAnalysis {
            intent: intent.to_string(),
            confidence,
            reasoning: reasoning.to_string(),
            escalate,
        })
    }
}

/// Register this node as a tool in the MCP server
pub fn register_determine_intent_tool(server: &CustomerSupportMCPServer) -> Result<(), WorkflowError> {
    let node = Arc::new(DetermineTicketIntentNode::new());
    let metadata = node.as_tool_metadata();
    
    tokio::runtime::Handle::current()
        .block_on(server.register_node_as_tool(node, metadata))
}