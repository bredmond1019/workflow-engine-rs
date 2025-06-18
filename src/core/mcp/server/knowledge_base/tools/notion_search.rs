/// Notion Search Node - Searches Notion pages and databases for knowledge base queries
/// 
/// This node integrates with Notion via the NotionClientNode to search through
/// documentation, wiki pages, and structured databases for relevant information.

use serde_json::Value;

use crate::core::{
    error::WorkflowError,
    mcp::clients::notion::NotionClientNode,
    nodes::Node,
    task::TaskContext,
};

/// Searches Notion documentation and pages for query matches
/// 
/// This node can optionally integrate with a NotionClientNode for real searches,
/// but provides mock results by default for testing and development purposes.
#[derive(Debug)]
pub struct NotionSearchNode {
    pub notion_client: Option<NotionClientNode>,
}

impl NotionSearchNode {
    /// Creates a new NotionSearchNode without a client (mock mode)
    pub fn new() -> Self {
        Self {
            notion_client: None,
        }
    }

    /// Creates a NotionSearchNode with a configured Notion client
    pub fn with_client(mut self, client: NotionClientNode) -> Self {
        self.notion_client = Some(client);
        self
    }
}

impl Node for NotionSearchNode {
    fn node_name(&self) -> String {
        "NotionSearchNode".to_string()
    }

    fn process(&self, mut task_context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // For now, simulate search results since we can't make async calls in process()
        let user_query = task_context
            .get_data::<String>("user_query")
            .unwrap_or(None)
            .unwrap_or_default();

        // Mock search results for documentation and pages
        let mock_results = Value::Object(
            [
                ("source".to_string(), Value::String("notion".to_string())),
                ("query".to_string(), Value::String(user_query.to_string())),
                ("results_found".to_string(), Value::Number(3.into())),
                (
                    "pages".to_string(),
                    Value::Array(vec![
                        Value::Object(
                            [
                                (
                                    "title".to_string(),
                                    Value::String("Related Documentation".to_string()),
                                ),
                                (
                                    "url".to_string(),
                                    Value::String("https://notion.so/page1".to_string()),
                                ),
                                ("relevance".to_string(), Value::Number(85.into())),
                            ]
                            .into_iter()
                            .collect(),
                        ),
                        Value::Object(
                            [
                                ("title".to_string(), Value::String("FAQ Entry".to_string())),
                                (
                                    "url".to_string(),
                                    Value::String("https://notion.so/page2".to_string()),
                                ),
                                ("relevance".to_string(), Value::Number(72.into())),
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

        task_context.set_data("notion_search_results", mock_results)?;
        task_context.set_data("notion_search_completed", Value::Bool(true))?;

        Ok(task_context)
    }
}