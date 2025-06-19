/// Spam Filter Node - Filters out spam queries from knowledge base processing
/// 
/// This node implements basic spam detection to prevent malicious or irrelevant
/// queries from consuming knowledge base resources.

use serde_json::Value;

use workflow_engine_core::{
    error::WorkflowError,
    nodes::Node,
    task::TaskContext,
    // Import from local module
};
use crate::server::knowledge_base::KnowledgeBaseEventData;

/// Filters spam queries before knowledge base processing
/// 
/// Uses a simple keyword-based approach to detect common spam indicators.
/// Queries identified as spam are marked and can be rejected by downstream nodes.
#[derive(Debug, Clone)]
pub struct FilterSpamQueryNode;

impl Node for FilterSpamQueryNode {
    fn node_name(&self) -> String {
        "FilterSpamQueryNode".to_string()
    }

    fn process(&self, mut task_context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let event_data: KnowledgeBaseEventData = task_context.get_event_data()?;

        // Simple spam detection using common spam indicators
        let spam_indicators = [
            "viagra",
            "lottery",
            "winner",
            "congratulations",
            "click here",
            "free money",
        ];
        let is_spam = spam_indicators
            .iter()
            .any(|&indicator| event_data.user_query.to_lowercase().contains(indicator));

        task_context.set_data("is_spam", Value::Bool(is_spam))?;
        task_context.set_data("spam_check_completed", Value::Bool(true))?;

        Ok(task_context)
    }
}