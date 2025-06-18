/// Query Validation Node - Validates user queries for knowledge base processing
/// 
/// This node performs basic validation checks on user queries to ensure they meet
/// minimum requirements for processing through the knowledge base workflow.

use serde_json::Value;

use crate::core::{
    error::WorkflowError,
    nodes::Node,
    task::TaskContext,
    mcp::server::knowledge_base::KnowledgeBaseEventData,
};

/// Validates user queries before processing
/// 
/// Performs validation checks including:
/// - Query length validation (3-1000 characters)
/// - Non-empty query validation
/// - Sets validation status in the task context
#[derive(Debug, Clone)]
pub struct ValidateQueryNode;

impl Node for ValidateQueryNode {
    fn node_name(&self) -> String {
        "ValidateQueryNode".to_string()
    }

    fn process(&self, mut task_context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let event_data: KnowledgeBaseEventData = task_context.get_event_data()?;

        // Basic validation checks
        let is_valid =
            !event_data.user_query.trim().is_empty() 
            && event_data.user_query.len() >= 3 
            && event_data.user_query.len() <= 1000;

        task_context.set_data("query_valid", Value::Bool(is_valid))?;

        if !is_valid {
            task_context.set_data(
                "validation_error",
                Value::String("Query must be between 3 and 1000 characters".to_string()),
            )?;
        }

        Ok(task_context)
    }
}