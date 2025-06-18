/// Knowledge base tools module - contains all workflow nodes for knowledge base operations
/// 
/// This module provides a complete set of nodes for processing knowledge base queries:
/// - Query routing and validation
/// - Search across multiple sources (Notion, HelpScout, Slack)
/// - Analysis and response generation
/// - Final response delivery

mod query_router;
mod validate_query;
mod filter_spam_query;
mod search_router;
mod notion_search;
mod helpscout_search;
mod slack_search;
mod analyze_knowledge;
mod generate_knowledge_response;
mod send_knowledge_reply;

pub use query_router::QueryRouterNode;
pub use validate_query::ValidateQueryNode;
pub use filter_spam_query::FilterSpamQueryNode;
pub use search_router::SearchRouterNode;
pub use notion_search::NotionSearchNode;
pub use helpscout_search::HelpscoutSearchNode;
pub use slack_search::SlackSearchNode;
pub use analyze_knowledge::AnalyzeKnowledgeNode;
pub use generate_knowledge_response::GenerateKnowledgeResponseNode;
pub use send_knowledge_reply::SendKnowledgeReplyNode;

/// Helper function to extract keywords from a query string
/// 
/// Removes common stop words and short words, returning meaningful keywords
/// for better search results across knowledge sources.
pub fn extract_keywords(query: &str) -> Vec<String> {
    // Simple keyword extraction - split on whitespace and remove common words
    let stop_words = [
        "the", "is", "at", "which", "on", "a", "an", "and", "or", "but", "in", "with", "to", "for",
        "of", "as", "by",
    ];

    query
        .split_whitespace()
        .map(|word| {
            word.trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|word| !word.is_empty() && word.len() > 2 && !stop_words.contains(&word.as_str()))
        .collect()
}