/// Analyze Knowledge Node - Determines if sufficient information was found
/// 
/// This node analyzes the search results from all knowledge sources (Notion, HelpScout, Slack)
/// to determine if enough relevant information was found to provide a comprehensive answer.

use serde_json::Value;

use crate::core::{
    error::WorkflowError,
    nodes::Node,
    task::TaskContext,
};

/// Analyzes search results to determine if sufficient information was found
/// 
/// Evaluates search results based on:
/// - Total number of results found across all sources
/// - Number of high-relevance results (relevance score >= 80)
/// - Coverage across different knowledge sources
#[derive(Debug, Clone)]
pub struct AnalyzeKnowledgeNode;

impl Node for AnalyzeKnowledgeNode {
    fn node_name(&self) -> String {
        "AnalyzeKnowledgeNode".to_string()
    }

    fn process(&self, mut task_context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Collect results from all search nodes
        let notion_results = task_context
            .get_data::<Value>("notion_search_results")
            .unwrap_or(None);
        let helpscout_results = task_context
            .get_data::<Value>("helpscout_search_results")
            .unwrap_or(None);
        let slack_results = task_context
            .get_data::<Value>("slack_search_results")
            .unwrap_or(None);

        let mut total_results = 0;
        let mut high_relevance_count = 0;
        let mut all_sources = Vec::new();

        // Analyze Notion results
        if let Some(notion) = notion_results {
            if let Some(count) = notion.get("results_found").and_then(|v| v.as_u64()) {
                total_results += count;
            }
            if let Some(pages) = notion.get("pages").and_then(|v| v.as_array()) {
                for page in pages {
                    if let Some(relevance) = page.get("relevance").and_then(|v| v.as_u64()) {
                        if relevance >= 80 {
                            high_relevance_count += 1;
                        }
                    }
                }
            }
            all_sources.push("notion");
        }

        // Analyze HelpScout results
        if let Some(helpscout) = helpscout_results {
            if let Some(count) = helpscout.get("results_found").and_then(|v| v.as_u64()) {
                total_results += count;
            }
            // Check both articles and conversations for high relevance
            for result_type in ["articles", "conversations"] {
                if let Some(items) = helpscout.get(result_type).and_then(|v| v.as_array()) {
                    for item in items {
                        if let Some(relevance) = item.get("relevance").and_then(|v| v.as_u64()) {
                            if relevance >= 80 {
                                high_relevance_count += 1;
                            }
                        }
                    }
                }
            }
            all_sources.push("helpscout");
        }

        // Analyze Slack results
        if let Some(slack) = slack_results {
            if let Some(count) = slack.get("results_found").and_then(|v| v.as_u64()) {
                total_results += count;
            }
            if let Some(messages) = slack.get("messages").and_then(|v| v.as_array()) {
                for message in messages {
                    if let Some(relevance) = message.get("relevance").and_then(|v| v.as_u64()) {
                        if relevance >= 80 {
                            high_relevance_count += 1;
                        }
                    }
                }
            }
            all_sources.push("slack");
        }

        // Determine if we have enough information (at least 2 results with 1+ high relevance)
        let sufficient_info = total_results >= 2 && high_relevance_count >= 1;

        task_context.set_data("total_results_found", Value::Number(total_results.into()))?;
        task_context.set_data(
            "high_relevance_count",
            Value::Number(high_relevance_count.into()),
        )?;
        task_context.set_data("sufficient_information", Value::Bool(sufficient_info))?;
        task_context.set_data(
            "sources_searched",
            Value::Array(
                all_sources
                    .into_iter()
                    .map(|s| Value::String(s.to_string()))
                    .collect(),
            ),
        )?;
        task_context.set_data("analysis_completed", Value::Bool(true))?;

        if !sufficient_info {
            task_context.set_data(
                "analysis_message",
                Value::String(
                    "Insufficient information found to provide a comprehensive answer".to_string(),
                ),
            )?;
        }

        Ok(task_context)
    }
}