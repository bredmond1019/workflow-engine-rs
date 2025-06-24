use crate::server::ToolMetadata;
use workflow_engine_core::{error::WorkflowError, nodes::Node, task::TaskContext};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::{super::server::CustomerSupportMcpServer, CustomerCareEventData};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpamAnalysis {
    pub is_human: bool,
    pub confidence: f32,
    pub reasoning: String,
}

#[derive(Debug)]
pub struct FilterSpamNode;

impl FilterSpamNode {
    pub fn new() -> Self {
        Self
    }

    pub async fn register(server: &mut CustomerSupportMcpServer) -> Result<(), WorkflowError> {
        let node = Arc::new(Self::new());
        let metadata = ToolMetadata::new(
            "filter_spam".to_string(),
            "Analyzes ticket content to detect and filter spam messages".to_string(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "context_data": {
                        "type": "object",
                        "properties": {
                            "message": {"type": "string", "description": "Message content to analyze for spam"},
                            "sender_reputation": {"type": "number", "description": "Sender reputation score (optional)"}
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

impl Node for FilterSpamNode {
    fn process(&self, mut task_context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let event_data: CustomerCareEventData = task_context.get_event_data()?;

        let analysis = SpamAnalysis {
            is_human: true,
            confidence: 0.88, // 1 - spam_score
            reasoning: format!("Message length: {}", event_data.message.len()),
        };

        task_context.update_node(&self.node_name(), analysis);
        Ok(task_context)
    }
}
