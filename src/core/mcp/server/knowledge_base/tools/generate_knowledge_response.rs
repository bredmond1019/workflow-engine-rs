/// Generate Knowledge Response Node - Creates synthesized responses from search results
/// 
/// This node takes the analyzed search results from all knowledge sources and generates
/// a comprehensive, well-formatted response that includes relevant information and source links.

use serde_json::Value;

use crate::core::{
    error::WorkflowError,
    nodes::Node,
    task::TaskContext,
};

/// Generates comprehensive responses from knowledge base search results
/// 
/// Creates formatted responses that include:
/// - Summary of findings from all sources
/// - Organized sections for different source types
/// - Direct links to relevant documentation and conversations
/// - Fallback responses when insufficient information is found
#[derive(Debug, Clone)]
pub struct GenerateKnowledgeResponseNode;

impl Node for GenerateKnowledgeResponseNode {
    fn node_name(&self) -> String {
        "GenerateKnowledgeResponseNode".to_string()
    }

    fn process(&self, mut task_context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let sufficient_info = task_context
            .get_data::<bool>("sufficient_information")?
            .unwrap_or(false);

        if !sufficient_info {
            let response = "I apologize, but I couldn't find enough relevant information to provide a comprehensive answer to your question. You might want to try rephrasing your question or contacting support directly.".to_string();

            task_context.set_data("generated_response", Value::String(response))?;
            task_context.set_data(
                "response_type",
                Value::String("insufficient_info".to_string()),
            )?;
            return Ok(task_context);
        }

        let user_query = task_context
            .get_data::<String>("user_query")?
            .unwrap_or_else(|| "your question".to_string());

        // Collect all relevant information
        let mut response_parts = Vec::new();
        let mut sources = Vec::new();

        response_parts.push(format!(
            "Based on my search across our knowledge base, here's what I found regarding {}:",
            user_query
        ));

        // Process Notion results
        if let Some(notion) = task_context
            .get_data::<Value>("notion_search_results")
            .unwrap_or(None)
        {
            if let Some(pages) = notion.get("pages").and_then(|v| v.as_array()) {
                if !pages.is_empty() {
                    response_parts.push("\n**From Documentation:**".to_string());
                    for page in pages {
                        if let (Some(title), Some(url)) = (
                            page.get("title").and_then(|v| v.as_str()),
                            page.get("url").and_then(|v| v.as_str()),
                        ) {
                            response_parts.push(format!("- [{}]({})", title, url));
                            sources.push(Value::Object(
                                [
                                    (
                                        "type".to_string(),
                                        Value::String("documentation".to_string()),
                                    ),
                                    ("title".to_string(), Value::String(title.to_string())),
                                    ("url".to_string(), Value::String(url.to_string())),
                                ]
                                .into_iter()
                                .collect(),
                            ));
                        }
                    }
                }
            }
        }

        // Process HelpScout results
        if let Some(helpscout) = task_context
            .get_data::<Value>("helpscout_search_results")
            .unwrap_or(None)
        {
            if let Some(articles) = helpscout.get("articles").and_then(|v| v.as_array()) {
                if !articles.is_empty() {
                    response_parts.push("\n**From Knowledge Base:**".to_string());
                    for article in articles {
                        if let (Some(title), Some(url)) = (
                            article.get("title").and_then(|v| v.as_str()),
                            article.get("url").and_then(|v| v.as_str()),
                        ) {
                            response_parts.push(format!("- [{}]({})", title, url));
                            sources.push(Value::Object(
                                [
                                    (
                                        "type".to_string(),
                                        Value::String("knowledge_base".to_string()),
                                    ),
                                    ("title".to_string(), Value::String(title.to_string())),
                                    ("url".to_string(), Value::String(url.to_string())),
                                ]
                                .into_iter()
                                .collect(),
                            ));
                        }
                    }
                }
            }
        }

        // Process Slack results
        if let Some(slack) = task_context
            .get_data::<Value>("slack_search_results")
            .unwrap_or(None)
        {
            if let Some(messages) = slack.get("messages").and_then(|v| v.as_array()) {
                if !messages.is_empty() {
                    response_parts.push("\n**From Team Discussions:**".to_string());
                    for message in messages.iter().take(2) {
                        // Limit to most relevant
                        if let (Some(channel), Some(user), Some(text)) = (
                            message.get("channel").and_then(|v| v.as_str()),
                            message.get("user").and_then(|v| v.as_str()),
                            message.get("text").and_then(|v| v.as_str()),
                        ) {
                            response_parts.push(format!("- **{}** in {}: {}", user, channel, text));
                        }
                    }
                }
            }
        }

        response_parts.push("\n\n**Most relevant source:** Based on relevance scores, I recommend starting with the highest-ranked result above.".to_string());

        let final_response = response_parts.join("\n");

        task_context.set_data("generated_response", Value::String(final_response))?;
        task_context.set_data("response_type", Value::String("comprehensive".to_string()))?;
        task_context.set_data("response_sources", Value::Array(sources))?;
        task_context.set_data("response_generated", Value::Bool(true))?;

        Ok(task_context)
    }
}