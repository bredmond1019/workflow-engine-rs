/// Analyze Knowledge Node - Determines if sufficient information was found
/// 
/// This node analyzes the search results from knowledge sources
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
/// - Total number of results found from available sources
/// - Number of high-relevance results (relevance score >= 80)
/// - Quality of search results across different knowledge sources
#[derive(Debug, Clone)]
pub struct AnalyzeKnowledgeNode;

impl Node for AnalyzeKnowledgeNode {
    fn node_name(&self) -> String {
        "AnalyzeKnowledgeNode".to_string()
    }

    fn process(&self, mut task_context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Collect search results from available sources
        let search_results = task_context
            .get_data::<Value>("search_results")
            .unwrap_or(None);

        let mut total_results = 0;
        let mut high_relevance_count = 0;
        let mut sources_found = Vec::new();

        // Analyze search results
        if let Some(results) = search_results {
            if let Some(count) = results.get("total_results").and_then(|v| v.as_u64()) {
                total_results = count;
            }
            
            if let Some(items) = results.get("items").and_then(|v| v.as_array()) {
                for item in items {
                    if let Some(relevance) = item.get("relevance").and_then(|v| v.as_u64()) {
                        if relevance >= 80 {
                            high_relevance_count += 1;
                        }
                    }
                    
                    if let Some(source) = item.get("source").and_then(|v| v.as_str()) {
                        if !sources_found.contains(&source.to_string()) {
                            sources_found.push(source.to_string());
                        }
                    }
                }
            }
        }

        // Determine if we have enough information (at least 1 result with sufficient relevance)
        let sufficient_info = total_results >= 1 && high_relevance_count >= 1;

        task_context.set_data("total_results_found", Value::Number(total_results.into()))?;
        task_context.set_data(
            "high_relevance_count",
            Value::Number(high_relevance_count.into()),
        )?;
        task_context.set_data("sufficient_information", Value::Bool(sufficient_info))?;
        task_context.set_data(
            "sources_searched",
            Value::Array(
                sources_found
                    .into_iter()
                    .map(|s| Value::String(s))
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