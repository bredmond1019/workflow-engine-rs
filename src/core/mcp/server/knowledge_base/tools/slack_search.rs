/// Slack Search Node - Searches Slack messages and conversations
/// 
/// This node integrates with Slack via the SlackClientNode to search through
/// team conversations and messages for relevant information and past solutions.

use serde_json::Value;

use crate::core::{
    error::WorkflowError,
    mcp::clients::slack::SlackClientNode,
    nodes::Node,
    task::TaskContext,
};

/// Searches Slack conversations and messages for query matches
/// 
/// This node can optionally integrate with a SlackClientNode for real searches,
/// but provides mock results by default for testing and development purposes.
#[derive(Debug)]
pub struct SlackSearchNode {
    pub slack_client: Option<SlackClientNode>,
}

impl SlackSearchNode {
    /// Creates a new SlackSearchNode without a client (mock mode)
    pub fn new() -> Self {
        Self { slack_client: None }
    }

    /// Creates a SlackSearchNode with a configured Slack client
    pub fn with_client(mut self, client: SlackClientNode) -> Self {
        self.slack_client = Some(client);
        self
    }
}

impl Node for SlackSearchNode {
    fn node_name(&self) -> String {
        "SlackSearchNode".to_string()
    }

    fn process(&self, mut task_context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let user_query = task_context
            .get_data::<String>("user_query")
            .unwrap_or(None)
            .unwrap_or_default();

        // Mock search results for messages and conversations
        let mock_results = Value::Object(
            [
                ("source".to_string(), Value::String("slack".to_string())),
                ("query".to_string(), Value::String(user_query.to_string())),
                ("results_found".to_string(), Value::Number(4.into())),
                (
                    "messages".to_string(),
                    Value::Array(vec![
                        Value::Object(
                            [
                                ("channel".to_string(), Value::String("#general".to_string())),
                                ("user".to_string(), Value::String("john.doe".to_string())),
                                (
                                    "text".to_string(),
                                    Value::String("I had a similar question...".to_string()),
                                ),
                                (
                                    "timestamp".to_string(),
                                    Value::String("2024-01-15T10:30:00Z".to_string()),
                                ),
                                ("relevance".to_string(), Value::Number(82.into())),
                            ]
                            .into_iter()
                            .collect(),
                        ),
                        Value::Object(
                            [
                                ("channel".to_string(), Value::String("#support".to_string())),
                                ("user".to_string(), Value::String("jane.smith".to_string())),
                                (
                                    "text".to_string(),
                                    Value::String("Here's how I solved it...".to_string()),
                                ),
                                (
                                    "timestamp".to_string(),
                                    Value::String("2024-01-14T14:22:00Z".to_string()),
                                ),
                                ("relevance".to_string(), Value::Number(75.into())),
                            ]
                            .into_iter()
                            .collect(),
                        ),
                    ]),
                ),
            ]
            .into_iter()
            .collect(),
        );

        task_context.set_data("slack_search_results", mock_results)?;
        task_context.set_data("slack_search_completed", Value::Bool(true))?;

        Ok(task_context)
    }
}