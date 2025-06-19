use crate::server::ToolMetadata;
use workflow_engine_core::{error::WorkflowError, nodes::Node, task::TaskContext};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::{super::server::CustomerSupportMCPServer, CustomerCareEventData};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketValidation {
    pub is_valid: bool,
    pub missing_fields: Vec<String>,
    pub validation_message: String,
}

#[derive(Debug)]
pub struct ValidateTicketNode;

impl ValidateTicketNode {
    pub fn new() -> Self {
        Self
    }

    pub async fn register(server: &mut CustomerSupportMCPServer) -> Result<(), WorkflowError> {
        let node = Arc::new(Self::new());
        let metadata = ToolMetadata::new(
            "validate_ticket".to_string(),
            "Validates customer support ticket data for completeness and format".to_string(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "context_data": {
                        "type": "object",
                        "properties": {
                            "ticket_id": {"type": "string", "description": "Unique ticket identifier"},
                            "customer_id": {"type": "string", "description": "Customer identifier"},
                            "message": {"type": "string", "description": "Customer message content"},
                            "priority": {"type": "string", "enum": ["low", "medium", "high", "urgent"]}
                        },
                        "required": ["ticket_id", "customer_id", "message"]
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

impl Node for ValidateTicketNode {
    fn process(&self, mut task_context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let event_data: CustomerCareEventData = task_context.get_event_data()?;

        let mut missing_fields = Vec::new();
        if event_data.ticket_id.is_empty() {
            missing_fields.push("ticket_id".to_string());
        }
        if event_data.customer_id.is_empty() {
            missing_fields.push("customer_id".to_string());
        }

        let validation = TicketValidation {
            is_valid: missing_fields.is_empty(),
            missing_fields: missing_fields.clone(),
            validation_message: if missing_fields.is_empty() {
                "Ticket is valid".to_string()
            } else {
                format!("Missing required fields: {}", missing_fields.join(", "))
            },
        };

        task_context.update_node(&self.node_name(), validation);
        Ok(task_context)
    }
}
