/// Search Router Node - Initiates parallel searches across knowledge sources
/// 
/// This node acts as a router that validates query processing status and
/// initiates parallel searches across multiple knowledge sources including
/// Notion documentation, HelpScout articles, and Slack conversations.

use serde_json::Value;

use crate::core::{
    error::WorkflowError,
    nodes::Node,
    task::TaskContext,
};

/// Routes validated queries to parallel search operations
/// 
/// Responsibilities:
/// - Validates that the query passed validation and spam checks
/// - Sets up search context for parallel execution
/// - Prepares search readiness flags for downstream search nodes
#[derive(Debug, Clone)]
pub struct SearchRouterNode;

impl Node for SearchRouterNode {
    fn node_name(&self) -> String {
        "SearchRouterNode".to_string()
    }

    fn process(&self, mut task_context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Check validation results
        let is_valid = task_context
            .get_data::<bool>("query_valid")?
            .unwrap_or(false);

        let is_spam = task_context.get_data::<bool>("is_spam")?.unwrap_or(true);

        if !is_valid || is_spam {
            return Err(WorkflowError::ValidationError {
                message: "Query failed validation or was detected as spam".to_string(),
            });
        }

        // Prepare search context for parallel search execution
        task_context.set_data("search_initiated", Value::Bool(true))?;
        task_context.set_data("notion_search_ready", Value::Bool(true))?;
        task_context.set_data("helpscout_search_ready", Value::Bool(true))?;
        task_context.set_data("slack_search_ready", Value::Bool(true))?;

        Ok(task_context)
    }
}