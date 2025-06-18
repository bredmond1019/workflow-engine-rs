/// Send Knowledge Reply Node - Sends the final response to the user
/// 
/// This node handles the final step of delivering the generated knowledge base response
/// to the user through the appropriate communication channel.

use serde_json::Value;

use crate::core::{
    error::WorkflowError,
    nodes::Node,
    task::TaskContext,
};

/// Sends the final knowledge base response to the user
/// 
/// Responsibilities:
/// - Validates that a response was generated
/// - Records delivery metadata (timestamp, content, type)
/// - Handles logging for monitoring and analytics
/// - In a real implementation, would integrate with communication channels
#[derive(Debug, Clone)]
pub struct SendKnowledgeReplyNode;

impl Node for SendKnowledgeReplyNode {
    fn node_name(&self) -> String {
        "SendKnowledgeReplyNode".to_string()
    }

    fn process(&self, mut task_context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let response = task_context
            .get_data::<String>("generated_response")?
            .ok_or_else(|| WorkflowError::ValidationError {
                message: "No generated response found".to_string(),
            })?;

        let response_type = task_context
            .get_data::<String>("response_type")?
            .unwrap_or_else(|| "unknown".to_string());

        // In a real implementation, this would send the response via email, Slack, etc.
        task_context.set_data("reply_sent", Value::Bool(true))?;
        task_context.set_data("reply_content", Value::String(response.to_string()))?;
        task_context.set_data(
            "reply_timestamp",
            Value::String(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
                    .to_string(),
            ),
        )?;

        // Log the response for monitoring
        println!("Knowledge Base Response ({}): {}", response_type, response);

        Ok(task_context)
    }
}