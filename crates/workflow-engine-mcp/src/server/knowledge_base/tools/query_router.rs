/// Query Router Node - Prepares and routes user queries for knowledge base search
/// 
/// This node acts as the entry point for knowledge base queries, performing initial
/// processing and keyword extraction to prepare the query for downstream search operations.

use serde_json::Value;

use workflow_engine_core::{
    error::WorkflowError,
    nodes::Node,
    task::TaskContext,
    // Import from local module
};
use crate::server::knowledge_base::KnowledgeBaseEventData;

use super::extract_keywords;

/// Routes and prepares user queries for knowledge base processing
/// 
/// Responsibilities:
/// - Validates that a user query is present
/// - Extracts meaningful keywords for better search results
/// - Sets up the task context for downstream processing
#[derive(Debug, Clone)]
pub struct QueryRouterNode;

impl Node for QueryRouterNode {
    fn node_name(&self) -> String {
        "QueryRouterNode".to_string()
    }

    fn process(&self, mut task_context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let event_data: KnowledgeBaseEventData = task_context.get_event_data()?;

        if event_data.user_query.trim().is_empty() {
            return Err(WorkflowError::ValidationError {
                message: "User query cannot be empty".to_string(),
                field: "user_query".to_string(),
                value: Some(event_data.user_query.clone()),
                constraint: "non-empty string".to_string(),
                context: "in query_router node".to_string(),
            });
        }

        // Extract keywords for better search
        let keywords = extract_keywords(&event_data.user_query);
        task_context.set_data(
            "search_keywords",
            Value::Array(keywords.into_iter().map(Value::String).collect()),
        )?;

        // Set query as ready for processing
        task_context.set_data("query_processed", Value::Bool(true))?;
        task_context.set_data("ready_for_search", Value::Bool(true))?;

        Ok(task_context)
    }
}