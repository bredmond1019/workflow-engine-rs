use crate::server::ToolMetadata;
use workflow_engine_core::{error::WorkflowError, nodes::Node, task::TaskContext};
// TODO: AnthropicAgentNode needs to be reimplemented or imported from correct location
// use workflow_engine_core::ai::anthropic::AnthropicAgentNode;
use workflow_engine_core::nodes::agent::{AgentConfig, ModelProvider};
use std::sync::Arc;
use std::any::TypeId;

use super::{super::server::CustomerSupportMCPServer, CustomerCareEventData};

/// Analyze a customer support ticket
#[derive(Debug)]
pub struct AnalyzeTicketNode {
    pub metadata: ToolMetadata,
}

impl Default for AnalyzeTicketNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalyzeTicketNode {
    pub fn new() -> Self {
        Self {
            metadata: ToolMetadata {
                name: "analyze_ticket".to_string(),
                description: "Analyzes customer support tickets to extract sentiment, urgency, and actionable insights".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "required": ["ticket_id", "customer_id", "message", "priority"],
                    "properties": {
                        "ticket_id": { "type": "string" },
                        "customer_id": { "type": "string" },
                        "message": { "type": "string" },
                        "priority": { "type": "string" }
                    }
                }),
                node_type: TypeId::of::<AnalyzeTicketNode>(),
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

impl AnalyzeTicketNode {
    fn analyze_ticket(&self, event_data: &CustomerCareEventData) -> serde_json::Value {
        // TODO: This is a stub implementation. The actual implementation requires AnthropicAgentNode
        // which is not currently available. This should be reimplemented once AI capabilities are restored.
        
        // For now, provide a simple rule-based analysis
        let message_lower = event_data.message.to_lowercase();
        
        // Determine sentiment
        let sentiment = if message_lower.contains("thank") || message_lower.contains("great") || message_lower.contains("excellent") {
            "positive"
        } else if message_lower.contains("angry") || message_lower.contains("frustrated") || message_lower.contains("terrible") {
            "frustrated"
        } else if message_lower.contains("confused") || message_lower.contains("don't understand") {
            "confused"
        } else if message_lower.contains("disappointed") || message_lower.contains("unhappy") {
            "negative"
        } else {
            "neutral"
        };
        
        // Determine urgency based on priority and keywords
        let urgency = match event_data.priority.as_str() {
            "high" => "high",
            "urgent" => "critical",
            _ => {
                if message_lower.contains("asap") || message_lower.contains("urgent") || message_lower.contains("immediately") {
                    "high"
                } else if message_lower.contains("soon") || message_lower.contains("quickly") {
                    "medium"
                } else {
                    "low"
                }
            }
        };
        
        // Determine category
        let category = if message_lower.contains("bill") || message_lower.contains("invoice") || message_lower.contains("payment") {
            "billing"
        } else if message_lower.contains("bug") || message_lower.contains("error") || message_lower.contains("broken") {
            "technical"
        } else if message_lower.contains("refund") {
            "refund_request"
        } else if message_lower.contains("how") || message_lower.contains("feature") {
            "product_inquiry"
        } else if sentiment == "frustrated" || sentiment == "negative" {
            "complaint"
        } else if sentiment == "positive" {
            "compliment"
        } else {
            "general_inquiry"
        };
        
        // Determine complexity
        let word_count = event_data.message.split_whitespace().count();
        let complexity = if word_count > 100 {
            "complex"
        } else if word_count > 50 {
            "moderate"
        } else {
            "simple"
        };
        
        // Extract key issues (simple keyword-based)
        let mut key_issues = Vec::new();
        if message_lower.contains("refund") { key_issues.push("Refund request"); }
        if message_lower.contains("not working") { key_issues.push("Feature not working"); }
        if message_lower.contains("charge") || message_lower.contains("bill") { key_issues.push("Billing concern"); }
        if message_lower.contains("slow") { key_issues.push("Performance issue"); }
        if message_lower.contains("can't") || message_lower.contains("cannot") { key_issues.push("Unable to complete action"); }
        
        if key_issues.is_empty() {
            key_issues.push("General inquiry");
        }
        
        // Suggest actions based on analysis
        let mut suggested_actions = Vec::new();
        match category {
            "refund_request" => {
                suggested_actions.push("Escalate to billing team");
                suggested_actions.push("Verify refund eligibility");
            }
            "technical" => {
                suggested_actions.push("Gather technical details");
                suggested_actions.push("Check known issues");
            }
            "billing" => {
                suggested_actions.push("Review account billing history");
                suggested_actions.push("Clarify charges");
            }
            _ => {
                suggested_actions.push("Provide detailed response");
                suggested_actions.push("Offer additional assistance");
            }
        }
        
        serde_json::json!({
            "sentiment": sentiment,
            "urgency": urgency,
            "category": category,
            "complexity": complexity,
            "customer_emotion": format!("Customer appears to be {}", sentiment),
            "key_issues": key_issues,
            "suggested_actions": suggested_actions
        })
    }
}

impl Node for AnalyzeTicketNode {
    fn process(&self, mut task_context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let event_data: CustomerCareEventData = task_context.get_event_data()?;
        
        // Perform analysis
        let analysis = self.analyze_ticket(&event_data);
        
        // Store analysis results in task context
        task_context.update_node(&self.node_name(), analysis);
        
        Ok(task_context)
    }
}

/// Register this node as a tool in the MCP server
pub fn register_analyze_ticket_tool(server: &CustomerSupportMCPServer) -> Result<(), WorkflowError> {
    let node = Arc::new(AnalyzeTicketNode::new());
    let metadata = node.as_tool_metadata();
    
    tokio::runtime::Handle::current()
        .block_on(server.register_node_as_tool(node, metadata))
}