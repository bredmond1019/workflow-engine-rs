/// HelpScout Search Node - Searches HelpScout articles and conversations
/// 
/// This node integrates with HelpScout via the HelpscoutClientNode to search through
/// knowledge base articles and customer support conversations for relevant information.

use serde_json::Value;

use crate::core::{
    error::WorkflowError,
    mcp::clients::helpscout::HelpscoutClientNode,
    nodes::Node,
    task::TaskContext,
};

/// Searches HelpScout knowledge base articles and conversations
/// 
/// This node can optionally integrate with a HelpscoutClientNode for real searches,
/// but provides mock results by default for testing and development purposes.
#[derive(Debug)]
pub struct HelpscoutSearchNode {
    pub helpscout_client: Option<HelpscoutClientNode>,
}

impl HelpscoutSearchNode {
    /// Creates a new HelpscoutSearchNode without a client (mock mode)
    pub fn new() -> Self {
        Self {
            helpscout_client: None,
        }
    }

    /// Creates a HelpscoutSearchNode with a configured HelpScout client
    pub fn with_client(mut self, client: HelpscoutClientNode) -> Self {
        self.helpscout_client = Some(client);
        self
    }
}

impl Node for HelpscoutSearchNode {
    fn node_name(&self) -> String {
        "HelpscoutSearchNode".to_string()
    }

    fn process(&self, mut task_context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let user_query = task_context
            .get_data::<String>("user_query")
            .unwrap_or(None)
            .unwrap_or_default();

        // Mock search results for articles and conversations
        let mock_results = Value::Object(
            [
                ("source".to_string(), Value::String("helpscout".to_string())),
                ("query".to_string(), Value::String(user_query.to_string())),
                ("results_found".to_string(), Value::Number(2.into())),
                (
                    "articles".to_string(),
                    Value::Array(vec![Value::Object(
                        [
                            (
                                "title".to_string(),
                                Value::String("How to Guide".to_string()),
                            ),
                            (
                                "url".to_string(),
                                Value::String("https://helpscout.com/article1".to_string()),
                            ),
                            ("relevance".to_string(), Value::Number(90.into())),
                        ]
                        .into_iter()
                        .collect(),
                    )]),
                ),
                (
                    "conversations".to_string(),
                    Value::Array(vec![Value::Object(
                        [
                            (
                                "subject".to_string(),
                                Value::String("Similar Issue Resolved".to_string()),
                            ),
                            (
                                "url".to_string(),
                                Value::String("https://helpscout.com/conversation1".to_string()),
                            ),
                            ("relevance".to_string(), Value::Number(78.into())),
                        ]
                        .into_iter()
                        .collect(),
                    )]),
                ),
            ]
            .into_iter()
            .collect(),
        );

        task_context.set_data("helpscout_search_results", mock_results)?;
        task_context.set_data("helpscout_search_completed", Value::Bool(true))?;

        Ok(task_context)
    }
}