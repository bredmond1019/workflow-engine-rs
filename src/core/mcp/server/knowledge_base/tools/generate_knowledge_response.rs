/// Generate Knowledge Response Node - Creates synthesized responses from search results
/// 
/// This node takes the analyzed search results from knowledge sources and generates
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
/// - Summary of findings from available sources
/// - Organized sections for different result types
/// - Direct links to relevant documentation and resources
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

        // Collect search results
        let mut response_parts = Vec::new();
        let mut sources = Vec::new();

        response_parts.push(format!(
            "Based on my search across available knowledge sources, here's what I found regarding {}:",
            user_query
        ));

        // Process search results
        if let Some(search_results) = task_context
            .get_data::<Value>("search_results")
            .unwrap_or(None)
        {
            if let Some(items) = search_results.get("items").and_then(|v| v.as_array()) {
                if !items.is_empty() {
                    response_parts.push("\n**Search Results:**".to_string());
                    
                    for item in items.iter().take(5) { // Limit to top 5 results
                        if let (Some(title), Some(content)) = (
                            item.get("title").and_then(|v| v.as_str()),
                            item.get("content").and_then(|v| v.as_str()),
                        ) {
                            let snippet = if content.len() > 200 {
                                format!("{}...", &content[..200])
                            } else {
                                content.to_string()
                            };
                            
                            response_parts.push(format!("- **{}**: {}", title, snippet));
                            
                            let mut source_obj = serde_json::Map::new();
                            source_obj.insert("title".to_string(), Value::String(title.to_string()));
                            source_obj.insert("content".to_string(), Value::String(content.to_string()));
                            
                            if let Some(url) = item.get("url").and_then(|v| v.as_str()) {
                                source_obj.insert("url".to_string(), Value::String(url.to_string()));
                            }
                            
                            if let Some(source_type) = item.get("source").and_then(|v| v.as_str()) {
                                source_obj.insert("type".to_string(), Value::String(source_type.to_string()));
                            }
                            
                            sources.push(Value::Object(source_obj));
                        }
                    }
                }
            }
        }

        response_parts.push("\n\n**Recommendation:** I recommend reviewing the results above, starting with the most relevant items.".to_string());

        let final_response = response_parts.join("\n");

        task_context.set_data("generated_response", Value::String(final_response))?;
        task_context.set_data("response_type", Value::String("comprehensive".to_string()))?;
        task_context.set_data("response_sources", Value::Array(sources))?;
        task_context.set_data("response_generated", Value::Bool(true))?;

        Ok(task_context)
    }
}