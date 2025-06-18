use crate::core::mcp::server::ToolMetadata;
use crate::core::{error::WorkflowError, nodes::Node, task::TaskContext};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::super::server::CustomerSupportMCPServer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyResult {
    pub ticket_id: String,
    pub message_id: String,
    pub channel: String,
    pub status: String,
    pub sent_at: String,
    pub delivery_status: String,
    pub retry_count: u32,
}

#[derive(Debug, Clone)]
pub struct SendReplyNode;

impl SendReplyNode {
    pub fn new() -> Self {
        Self
    }
    
    fn validate_message_content(&self, message: &str) -> Result<(), WorkflowError> {
        if message.trim().is_empty() {
            return Err(WorkflowError::ValidationError {
                message: "Reply message cannot be empty".to_string(),
            });
        }
        
        if message.len() > 5000 {
            return Err(WorkflowError::ValidationError {
                message: "Reply message exceeds maximum length of 5000 characters".to_string(),
            });
        }
        
        // Check for inappropriate content (basic validation)
        let message_lower = message.to_lowercase();
        let inappropriate_words = ["spam", "scam", "inappropriate"];
        
        for word in &inappropriate_words {
            if message_lower.contains(word) {
                log::warn!("Potentially inappropriate content detected in reply: {}", word);
            }
        }
        
        Ok(())
    }
    
    fn validate_channel(&self, channel: &str) -> Result<String, WorkflowError> {
        match channel.to_lowercase().as_str() {
            "email" => Ok("email".to_string()),
            "chat" | "live_chat" => Ok("chat".to_string()),
            "phone" | "call" => Ok("phone".to_string()),
            "sms" | "text" => Ok("sms".to_string()),
            _ => Err(WorkflowError::ValidationError {
                message: format!("Unsupported communication channel: {}", channel),
            }),
        }
    }
    
    fn send_reply_through_channel(
        &self,
        ticket_id: &str,
        message: &str,
        channel: &str,
        priority: &str,
    ) -> Result<ReplyResult, WorkflowError> {
        let message_id = self.generate_message_id(ticket_id, channel);
        let sent_at = chrono::Utc::now().to_rfc3339();
        
        // Simulate sending through different channels
        let delivery_status = match channel {
            "email" => self.send_email_reply(ticket_id, message, priority)?,
            "chat" => self.send_chat_reply(ticket_id, message, priority)?,
            "phone" => self.send_phone_reply(ticket_id, message, priority)?,
            "sms" => self.send_sms_reply(ticket_id, message, priority)?,
            _ => return Err(WorkflowError::ProcessingError {
                message: format!("Unknown channel: {}", channel),
            }),
        };
        
        Ok(ReplyResult {
            ticket_id: ticket_id.to_string(),
            message_id,
            channel: channel.to_string(),
            status: "sent".to_string(),
            sent_at,
            delivery_status,
            retry_count: 0,
        })
    }
    
    fn generate_message_id(&self, ticket_id: &str, channel: &str) -> String {
        let timestamp = chrono::Utc::now().timestamp();
        format!("msg_{}_{}__{}", channel, ticket_id, timestamp)
    }
    
    fn send_email_reply(
        &self,
        ticket_id: &str,
        message: &str,
        priority: &str,
    ) -> Result<String, WorkflowError> {
        // In a real implementation, this would integrate with an email service
        // For now, we'll simulate the process
        
        log::info!(
            "Sending email reply for ticket {} (Priority: {})",
            ticket_id,
            priority
        );
        
        // Simulate email delivery (always succeeds in simulation)
        Ok("delivered".to_string())
    }
    
    fn send_chat_reply(
        &self,
        ticket_id: &str,
        message: &str,
        priority: &str,
    ) -> Result<String, WorkflowError> {
        // In a real implementation, this would integrate with a chat service
        log::info!(
            "Sending chat reply for ticket {} (Priority: {})",
            ticket_id,
            priority
        );
        
        // Simulate instant delivery for chat
        Ok("delivered".to_string())
    }
    
    fn send_phone_reply(
        &self,
        ticket_id: &str,
        message: &str,
        priority: &str,
    ) -> Result<String, WorkflowError> {
        // Phone replies would typically be callback requests or voicemails
        log::info!(
            "Scheduling phone callback for ticket {} (Priority: {})",
            ticket_id,
            priority
        );
        
        // Simulate callback scheduling
        Ok("callback_scheduled".to_string())
    }
    
    fn send_sms_reply(
        &self,
        ticket_id: &str,
        message: &str,
        priority: &str,
    ) -> Result<String, WorkflowError> {
        // In a real implementation, this would integrate with an SMS service
        log::info!(
            "Sending SMS reply for ticket {} (Priority: {})",
            ticket_id,
            priority
        );
        
        // Simulate SMS delivery
        Ok("delivered".to_string())
    }

    pub async fn register(server: &mut CustomerSupportMCPServer) -> Result<(), WorkflowError> {
        let node = Arc::new(Self::new());
        let metadata = ToolMetadata::new(
            "send_reply".to_string(),
            "Sends reply to customer through appropriate communication channel".to_string(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "context_data": {
                        "type": "object",
                        "properties": {
                            "ticket_id": {"type": "string", "description": "Ticket to reply to"},
                            "message": {"type": "string", "description": "Reply message content"},
                            "channel": {"type": "string", "enum": ["email", "chat", "phone"], "description": "Communication channel"},
                            "priority": {"type": "string", "enum": ["low", "normal", "high", "urgent"], "description": "Reply priority"}
                        },
                        "required": ["ticket_id", "message", "channel"]
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

impl Node for SendReplyNode {
    fn process(&self, mut task_context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Extract reply parameters from context
        let context_data = task_context.get_data::<serde_json::Value>("context_data")?;
        
        let ticket_id = context_data
            .as_ref()
            .and_then(|v| v["ticket_id"].as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| WorkflowError::ValidationError {
                message: "Missing required field: ticket_id".to_string(),
            })?;
            
        let message = context_data
            .as_ref()
            .and_then(|v| v["message"].as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| WorkflowError::ValidationError {
                message: "Missing required field: message".to_string(),
            })?;
            
        let channel = context_data
            .as_ref()
            .and_then(|v| v["channel"].as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| WorkflowError::ValidationError {
                message: "Missing required field: channel".to_string(),
            })?;
        
        let priority = context_data
            .as_ref()
            .and_then(|v| v["priority"].as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "normal".to_string());

        // Validate message content
        self.validate_message_content(&message)?;
        
        // Validate channel
        let validated_channel = self.validate_channel(&channel)?;
        
        // Send the reply through the appropriate channel
        let reply_result = self.send_reply_through_channel(
            &ticket_id,
            &message,
            &validated_channel,
            &priority,
        )?;

        // Log the reply sending
        log::info!(
            "Sent reply for ticket {} via {} (Message ID: {})",
            ticket_id,
            reply_result.channel,
            reply_result.message_id
        );

        // Update task context with reply results
        task_context.update_node(&self.node_name(), serde_json::json!({
            "reply_result": reply_result,
            "sent_at": chrono::Utc::now()
        }));
        
        // Mark reply as sent for backward compatibility
        task_context.update_node("reply_sent", &true);
        
        Ok(task_context)
    }
}
