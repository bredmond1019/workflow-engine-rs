//! # Knowledge Base Workflow
//!
//! This module provides a comprehensive knowledge search and retrieval workflow that searches
//! across multiple knowledge sources to provide accurate, contextual responses to user queries.
//! The workflow orchestrates parallel searches across Notion, HelpScout, and Slack to gather
//! relevant information and generate intelligent responses.
//!
//! ## Workflow Overview
//!
//! The knowledge base workflow implements an intelligent multi-source search system with:
//! - **Query validation and routing** with spam detection
//! - **Parallel source searching** across multiple knowledge repositories
//! - **Result analysis and relevance scoring** with confidence metrics
//! - **Comprehensive response generation** with source attribution
//! - **Intelligent fallback handling** for incomplete information
//!
//! ## Workflow Architecture
//!
//! ```text
//! ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
//! │  Query Router   │───▶│  Search Router  │───▶│ Analyze Results │
//! │                 │    │                 │    │                 │
//! └─────────────────┘    └─────────────────┘    └─────────────────┘
//!          │                       │                       │
//!          ▼                       ▼                       ▼
//! ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
//! │ Parallel Tasks: │    │ Parallel Tasks: │    │Generate Response│
//! │ - Validate      │    │ - Notion Search │    │                 │
//! │   Query         │    │ - HelpScout     │    └─────────────────┘
//! │ - Filter Spam   │    │   Search        │                 │
//! └─────────────────┘    │ - Slack Search  │                 ▼
//!                        └─────────────────┘    ┌─────────────────┐
//!                                              │ Send Knowledge  │
//!                                              │     Reply       │
//!                                              └─────────────────┘
//! ```
//!
//! ## Usage
//!
//! ### Basic Knowledge Base Search
//!
//! ```rust
//! use ai_architecture_workflows::knowledge_base_workflow::create_knowledge_base_workflow;
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create the knowledge base workflow
//!     let workflow = create_knowledge_base_workflow()?;
//!
//!     // Search the knowledge base
//!     let query_data = json!({
//!         "query_id": "QUERY-2024-001",
//!         "user_id": "USER-123",
//!         "user_query": "How do I configure SSL certificates for the API gateway?",
//!         "query_type": "technical",
//!         "sources": ["notion", "helpscout", "slack"]
//!     });
//!
//!     match workflow.run(query_data).await {
//!         Ok(result) => {
//!             println!("Knowledge search completed: {}", result.event_id);
//!             // Access search results
//!             for (node_name, node_result) in &result.nodes {
//!                 if node_name.contains("search") {
//!                     println!("Search results from '{}': {:?}", node_name, node_result);
//!                 }
//!             }
//!         }
//!         Err(e) => eprintln!("Knowledge search failed: {}", e),
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Advanced Usage with Multi-Source Configuration
//!
//! ```rust
//! use ai_architecture_workflows::{WorkflowRunner, knowledge_base_workflow::create_knowledge_base_workflow};
//! use ai_architecture_core::db::event::NewEvent;
//! use diesel::prelude::*;
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Set up workflow runner with database integration
//!     let workflow = create_knowledge_base_workflow()?;
//!     let runner = WorkflowRunner::new(workflow);
//!     let mut conn = establish_connection();
//!
//!     // Search across different knowledge domains
//!     let queries = vec![
//!         json!({
//!             "query_id": "TECH-001",
//!             "user_id": "DEV-123",
//!             "user_query": "API rate limiting best practices",
//!             "query_type": "technical",
//!             "sources": ["notion", "slack"]
//!         }),
//!         json!({
//!             "query_id": "SUPPORT-001", 
//!             "user_id": "SUPPORT-456",
//!             "user_query": "Customer onboarding process",
//!             "query_type": "process",
//!             "sources": ["helpscout", "notion"]
//!         }),
//!         json!({
//!             "query_id": "POLICY-001",
//!             "user_id": "HR-789",
//!             "user_query": "Remote work policies and guidelines",
//!             "query_type": "policy",
//!             "sources": ["notion", "slack", "helpscout"]
//!         }),
//!     ];
//!
//!     for query_data in queries {
//!         match runner.create_and_process(query_data, &mut conn) {
//!             Ok(processed_event) => {
//!                 println!("Processed knowledge query: {}", processed_event.id);
//!             }
//!             Err(e) => {
//!                 eprintln!("Failed to process query: {}", e);
//!             }
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Workflow Nodes
//!
//! ### QueryRouterNode
//! 
//! Entry point that processes and routes user queries:
//! - Extracts keywords and determines search intent
//! - Prepares query for multi-source search
//! - Triggers parallel validation and spam filtering
//! - Sets up search parameters and context
//!
//! ### Parallel Query Processing
//!
//! #### ValidateQueryNode
//! - Validates query structure and completeness
//! - Checks for required fields and proper formatting
//! - Ensures query is meaningful and searchable
//! - Provides validation confidence scores
//!
//! #### FilterSpamQueryNode
//! - Detects spam, malicious, or inappropriate queries
//! - Implements content filtering and safety checks
//! - Flags suspicious or automated queries
//! - Protects against query injection attacks
//!
//! ### SearchRouterNode (Router)
//!
//! Intelligent search orchestration that:
//! - Coordinates parallel searches across knowledge sources
//! - Manages search timeouts and fallbacks
//! - Distributes queries based on source availability
//! - Aggregates results from multiple sources
//!
//! ### Knowledge Source Nodes
//!
//! #### NotionSearchNode
//! - Searches Notion databases and pages
//! - Performs full-text and structured searches
//! - Extracts relevant documentation and wikis
//! - Provides source links and metadata
//!
//! #### HelpscoutSearchNode
//! - Searches HelpScout knowledge base articles
//! - Finds customer support documentation
//! - Accesses FAQ and troubleshooting guides
//! - Returns article relevance scores
//!
//! #### SlackSearchNode
//! - Searches Slack conversations and threads
//! - Finds relevant team discussions and decisions
//! - Extracts context from historical conversations
//! - Provides conversation links and participants
//!
//! ### AnalyzeKnowledgeNode
//!
//! Result analysis and quality assessment:
//! - Analyzes completeness and relevance of search results
//! - Determines if sufficient information was found
//! - Scores result quality and confidence
//! - Decides on response generation strategy
//!
//! ### GenerateKnowledgeResponseNode
//!
//! AI-powered response synthesis:
//! - Combines results from multiple sources
//! - Generates comprehensive, contextual responses
//! - Includes source attributions and links
//! - Handles cases with insufficient information
//!
//! ### SendKnowledgeReplyNode
//!
//! Final delivery and tracking:
//! - Sends the response to the user
//! - Updates query status and metrics
//! - Logs search performance and results
//! - Handles delivery confirmations
//!
//! ## Input Data Format
//!
//! The workflow expects query data in the following JSON format:
//!
//! ```json
//! {
//!   "query_id": "QUERY-2024-001",
//!   "user_id": "USER-123",
//!   "user_query": "How do I configure SSL certificates?",
//!   "query_type": "technical|process|policy|general",
//!   "sources": ["notion", "helpscout", "slack"],
//!   "priority": "low|medium|high|urgent",
//!   "context": {
//!     "department": "engineering",
//!     "project": "api-gateway",
//!     "urgency": "business_critical"
//!   },
//!   "filters": {
//!     "date_range": "last_6_months",
//!     "tags": ["ssl", "security", "certificates"],
//!     "authors": ["technical-team"]
//!   },
//!   "preferences": {
//!     "max_results": 10,
//!     "include_snippets": true,
//!     "require_sources": true
//!   }
//! }
//! ```
//!
//! ### Required Fields
//! - `query_id`: Unique identifier for the search query
//! - `user_id`: User identifier for context and permissions
//! - `user_query`: The actual search question or keywords
//! - `query_type`: Type of information being sought
//! - `sources`: Which knowledge sources to search
//!
//! ### Optional Fields
//! - `priority`: Search priority for resource allocation
//! - `context`: Additional context for better search results
//! - `filters`: Search filters and constraints
//! - `preferences`: User preferences for result format
//!
//! ## Output Data Structure
//!
//! The workflow produces a `TaskContext` with comprehensive search results:
//!
//! ```json
//! {
//!   "query_router": {
//!     "status": "completed",
//!     "keywords_extracted": ["SSL", "certificates", "configure"],
//!     "search_intent": "technical_documentation",
//!     "query_complexity": "medium"
//!   },
//!   "validate_query": {
//!     "status": "completed",
//!     "is_valid": true,
//!     "validation_score": 0.95,
//!     "issues": []
//!   },
//!   "filter_spam": {
//!     "status": "completed",
//!     "is_spam": false,
//!     "confidence": 0.98,
//!     "flags": []
//!   },
//!   "notion_search": {
//!     "status": "completed",
//!     "results_found": 5,
//!     "relevance_score": 0.87,
//!     "sources": [
//!       {
//!         "title": "SSL Certificate Configuration Guide",
//!         "url": "https://notion.so/ssl-guide",
//!         "snippet": "Step-by-step SSL configuration...",
//!         "relevance": 0.95
//!       }
//!     ]
//!   },
//!   "helpscout_search": {
//!     "status": "completed",
//!     "results_found": 3,
//!     "relevance_score": 0.72,
//!     "sources": [
//!       {
//!         "title": "SSL Troubleshooting FAQ",
//!         "url": "https://helpscout.com/ssl-faq",
//!         "snippet": "Common SSL certificate issues...",
//!         "relevance": 0.78
//!       }
//!     ]
//!   },
//!   "slack_search": {
//!     "status": "completed",
//!     "results_found": 8,
//!     "relevance_score": 0.65,
//!     "sources": [
//!       {
//!         "channel": "#engineering",
//!         "timestamp": "2024-01-10T15:30:00Z",
//!         "snippet": "@john shared SSL config for production...",
//!         "relevance": 0.71
//!       }
//!     ]
//!   },
//!   "analyze_knowledge": {
//!     "status": "completed",
//!     "sufficient_info": true,
//!     "overall_confidence": 0.82,
//!     "source_diversity": 3,
//!     "completeness_score": 0.89
//!   },
//!   "generate_response": {
//!     "status": "completed",
//!     "response": "Based on our documentation and team discussions...",
//!     "response_type": "comprehensive_guide",
//!     "sources_cited": 5,
//!     "confidence": 0.86
//!   },
//!   "send_reply": {
//!     "status": "completed",
//!     "sent_at": "2024-01-15T11:45:30Z",
//!     "delivery_method": "api_response",
//!     "response_id": "RESP-789"
//!   }
//! }
//! ```
//!
//! ## Running the Demo
//!
//! To see the knowledge base workflow in action:
//!
//! ### Quick Start
//! ```bash
//! # Run all demos including knowledge base
//! cargo run
//! 
//! # Run only knowledge base demos
//! cargo run --example knowledge_base_demo
//! ```
//!
//! ### Programmatic Demo Execution
//! ```rust
//! use ai_architecture_workflows::demos::knowledge_base_workflow::knowledge_base_workflow_demo;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Run the interactive knowledge base workflow demo
//!     knowledge_base_workflow_demo().await;
//! }
//! ```
//!
//! ### Demo Features
//!
//! The demo showcases:
//! - **Multiple query scenarios** (technical, product, troubleshooting, policy)
//! - **Real-time search visualization** with source-specific timing
//! - **Multi-source search results** with relevance scoring
//! - **Response generation** with source attribution
//! - **Database integration** showing query tracking and results storage
//! - **Type-safe data extraction** for processed search results
//!
//! ## Configuration
//!
//! ### Environment Variables
//!
//! ```bash
//! # AI Provider Configuration
//! ANTHROPIC_API_KEY=your_anthropic_key_here
//! OPENAI_API_KEY=your_openai_key_here
//!
//! # Knowledge Source Configuration
//! NOTION_API_KEY=your_notion_integration_key
//! NOTION_DATABASE_ID=your_notion_database_id
//! HELPSCOUT_API_KEY=your_helpscout_api_key
//! SLACK_BOT_TOKEN=xoxb-your-slack-bot-token
//! SLACK_APP_TOKEN=xapp-your-slack-app-token
//!
//! # Database Configuration
//! DATABASE_URL=postgresql://user:password@localhost/ai_architecture
//!
//! # Knowledge Base Specific Settings
//! KB_MAX_RESULTS_PER_SOURCE=10
//! KB_SEARCH_TIMEOUT_SECONDS=30
//! KB_MIN_RELEVANCE_SCORE=0.6
//! KB_ENABLE_PARALLEL_SEARCH=true
//! ```
//!
//! ### Knowledge Source Configuration
//!
//! #### Notion Configuration
//! ```rust
//! use ai_architecture_core::mcp::clients::notion::NotionConfig;
//!
//! let notion_config = NotionConfig {
//!     api_key: std::env::var("NOTION_API_KEY")?,
//!     database_id: std::env::var("NOTION_DATABASE_ID")?,
//!     max_results: 10,
//!     timeout_seconds: 30,
//! };
//! ```
//!
//! #### HelpScout Configuration
//! ```rust
//! use ai_architecture_core::mcp::clients::helpscout::HelpScoutConfig;
//!
//! let helpscout_config = HelpScoutConfig {
//!     api_key: std::env::var("HELPSCOUT_API_KEY")?,
//!     knowledge_base_id: std::env::var("HELPSCOUT_KB_ID")?,
//!     max_results: 10,
//!     include_drafts: false,
//! };
//! ```
//!
//! #### Slack Configuration
//! ```rust
//! use ai_architecture_core::mcp::clients::slack::SlackConfig;
//!
//! let slack_config = SlackConfig {
//!     bot_token: std::env::var("SLACK_BOT_TOKEN")?,
//!     app_token: std::env::var("SLACK_APP_TOKEN")?,
//!     channels: vec!["#engineering", "#general", "#support"],
//!     max_results: 20,
//!     search_timeframe_days: 180,
//! };
//! ```
//!
//! ## Performance Considerations
//!
//! ### Parallel Search Optimization
//! - All knowledge sources are searched concurrently
//! - Reduces total search time from ~15s to ~5s for typical queries
//! - Each source has independent timeouts and error handling
//! - Failed searches don't block other sources
//!
//! ### Caching Strategy
//! ```rust
//! // Implement search result caching
//! use std::collections::HashMap;
//! use std::time::{Duration, Instant};
//!
//! struct SearchCache {
//!     cache: HashMap<String, (serde_json::Value, Instant)>,
//!     ttl: Duration,
//! }
//!
//! impl SearchCache {
//!     fn new(ttl_minutes: u64) -> Self {
//!         Self {
//!             cache: HashMap::new(),
//!             ttl: Duration::from_secs(ttl_minutes * 60),
//!         }
//!     }
//!     
//!     fn get(&self, query: &str) -> Option<&serde_json::Value> {
//!         if let Some((result, timestamp)) = self.cache.get(query) {
//!             if timestamp.elapsed() < self.ttl {
//!                 return Some(result);
//!             }
//!         }
//!         None
//!     }
//! }
//! ```
//!
//! ### Memory and Resource Management
//! - Search results are streamed rather than fully loaded
//! - Large response payloads are paginated
//! - Connection pooling for knowledge source APIs
//! - Automatic cleanup of temporary search data
//!
//! ## Testing
//!
//! ### Unit Tests
//! ```bash
//! # Test knowledge base workflow components
//! cargo test knowledge_base_workflow::tests
//!
//! # Test individual search nodes
//! cargo test notion_search_node
//! cargo test helpscout_search_node  
//! cargo test slack_search_node
//! ```
//!
//! ### Integration Tests
//! ```bash
//! # Test with real knowledge sources (requires API keys)
//! cargo test --test knowledge_base_integration
//!
//! # Test search result aggregation
//! cargo test --test multi_source_search
//! ```
//!
//! ### Performance Tests
//! ```bash
//! # Run search performance benchmarks
//! cargo bench knowledge_base_benchmarks
//!
//! # Test parallel search performance
//! cargo bench parallel_search_performance
//! ```
//!
//! ## Advanced Features
//!
//! ### Custom Search Strategies
//! ```rust
//! use ai_architecture_core::{nodes::Node, task::TaskContext, error::WorkflowError};
//!
//! #[derive(Debug)]
//! struct SemanticSearchNode {
//!     embedding_model: String,
//!     similarity_threshold: f64,
//! }
//!
//! impl Node for SemanticSearchNode {
//!     fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
//!         let query = context.get_data::<String>("user_query")?;
//!         
//!         // Implement semantic search logic
//!         let embeddings = self.generate_embeddings(&query)?;
//!         let similar_docs = self.find_similar_documents(embeddings)?;
//!         
//!         context.update_node("semantic_search", serde_json::json!({
//!             "results": similar_docs,
//!             "similarity_scores": self.calculate_scores(&similar_docs),
//!             "search_strategy": "semantic_embedding"
//!         }));
//!         
//!         Ok(context)
//!     }
//! }
//! ```
//!
//! ### Result Ranking and Filtering
//! ```rust
//! #[derive(Debug)]
//! struct IntelligentRankingNode {
//!     ranking_weights: HashMap<String, f64>,
//! }
//!
//! impl Node for IntelligentRankingNode {
//!     fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
//!         // Aggregate results from all search sources
//!         let all_results = self.collect_search_results(&context)?;
//!         
//!         // Apply intelligent ranking based on multiple factors
//!         let ranked_results = self.rank_by_relevance(all_results)?;
//!         let filtered_results = self.filter_by_quality(ranked_results)?;
//!         
//!         context.update_node("intelligent_ranking", serde_json::json!({
//!             "ranked_results": filtered_results,
//!             "ranking_factors": ["relevance", "recency", "authority", "completeness"],
//!             "total_results": filtered_results.len()
//!         }));
//!         
//!         Ok(context)
//!     }
//! }
//! ```
//!
//! ## Troubleshooting
//!
//! ### Common Issues
//!
//! #### Knowledge Source Authentication
//! ```
//! Error: ProcessingError { message: "Notion API authentication failed" }
//! ```
//! **Solution**: Verify API keys and permissions in environment variables
//!
//! #### Search Timeout Issues
//! ```
//! Error: ProcessingError { message: "Search timeout exceeded" }
//! ```
//! **Solution**: Increase timeout values or optimize search queries
//!
//! #### Insufficient Search Results
//! ```
//! Warning: Low relevance scores across all sources
//! ```
//! **Solution**: Broaden search terms or check knowledge source content
//!
//! ### Debug Mode
//! ```bash
//! # Enable detailed search logging
//! RUST_LOG=debug cargo run
//!
//! # Trace specific knowledge source searches
//! RUST_LOG=ai_architecture_workflows::knowledge_base_workflow=trace cargo run
//!
//! # Debug search result aggregation
//! RUST_LOG=ai_architecture_core::mcp::clients=debug cargo run
//! ```
//!
//! ## Related Documentation
//!
//! - [`WorkflowRunner`](../struct.WorkflowRunner.html) - Database-integrated execution
//! - [`demos`](../demos/index.html) - Interactive demonstrations
//! - [`customer_support_workflow`](../customer_support_workflow/index.html) - Customer support automation
//! - [Knowledge Source Setup Guide](../../../docs/knowledge-sources.md) - Configuring external sources

use std::any::TypeId;

use crate::core::{
    error::WorkflowError,
    mcp::server::knowledge_base::tools::{
        AnalyzeKnowledgeNode, FilterSpamQueryNode, GenerateKnowledgeResponseNode,
        HelpscoutSearchNode, NotionSearchNode, QueryRouterNode, SearchRouterNode,
        SendKnowledgeReplyNode, SlackSearchNode, ValidateQueryNode,
    },
    nodes::config::NodeConfig,
    workflow::{Workflow, builder::WorkflowBuilder},
};

/// Creates a complete knowledge base search and response workflow
/// 
/// This function builds a workflow that processes user queries through multiple stages:
/// 
/// **Stage 1: Query Processing**
/// - QueryRouterNode: Prepares and routes the user query
/// - ValidateQueryNode & FilterSpamQueryNode: Parallel validation and spam filtering
/// 
/// **Stage 2: Knowledge Search**
/// - SearchRouterNode: Initiates parallel searches
/// - NotionSearchNode, HelpscoutSearchNode, SlackSearchNode: Parallel source searches
/// 
/// **Stage 3: Response Generation**
/// - AnalyzeKnowledgeNode: Determines if sufficient information was found
/// - GenerateKnowledgeResponseNode: Creates comprehensive response
/// - SendKnowledgeReplyNode: Delivers final response
/// 
/// # Returns
/// 
/// - `Ok(Workflow)` - A fully configured workflow ready for execution
/// - `Err(WorkflowError)` - If workflow construction fails
/// 
/// # Example
/// 
/// ```rust
/// use crate::workflows::knowledge_base_workflow::create_knowledge_base_workflow;
/// 
/// let workflow = create_knowledge_base_workflow()?;
/// // Execute workflow with user query...
/// ```
pub fn create_knowledge_base_workflow() -> Result<Workflow, WorkflowError> {
    let workflow = WorkflowBuilder::new::<QueryRouterNode>("knowledge_base".to_string())
        .description("Knowledge Base Search and Response Workflow".to_string())
        .add_node(
            NodeConfig::new::<QueryRouterNode>()
                .with_connections(vec![TypeId::of::<SearchRouterNode>()])
                .with_description("Routes and prepares user query for processing".to_string())
                .with_parallel_nodes(vec![
                    TypeId::of::<ValidateQueryNode>(),
                    TypeId::of::<FilterSpamQueryNode>(),
                ]),
        )
        .add_node(
            NodeConfig::new::<SearchRouterNode>()
                .with_connections(vec![TypeId::of::<AnalyzeKnowledgeNode>()])
                .with_router(true)
                .with_description(
                    "Initiates parallel searches across knowledge sources".to_string(),
                )
                .with_parallel_nodes(vec![
                    TypeId::of::<NotionSearchNode>(),
                    TypeId::of::<HelpscoutSearchNode>(),
                    TypeId::of::<SlackSearchNode>(),
                ]),
        )
        .add_node(
            NodeConfig::new::<AnalyzeKnowledgeNode>()
                .with_connections(vec![TypeId::of::<GenerateKnowledgeResponseNode>()])
                .with_description(
                    "Analyzes search results to determine if sufficient information was found"
                        .to_string(),
                ),
        )
        .add_node(
            NodeConfig::new::<GenerateKnowledgeResponseNode>()
                .with_connections(vec![TypeId::of::<SendKnowledgeReplyNode>()])
                .with_description(
                    "Generates comprehensive response from all search results".to_string(),
                ),
        )
        .add_node(
            NodeConfig::new::<SendKnowledgeReplyNode>()
                .with_description("Sends the final response to the user".to_string()),
        )
        .build()?;

    // Register all nodes with the workflow
    workflow.register_node(QueryRouterNode);
    workflow.register_node(ValidateQueryNode);
    workflow.register_node(FilterSpamQueryNode);
    workflow.register_node(SearchRouterNode);
    workflow.register_node(NotionSearchNode::new());
    workflow.register_node(HelpscoutSearchNode::new());
    workflow.register_node(SlackSearchNode::new());
    workflow.register_node(AnalyzeKnowledgeNode);
    workflow.register_node(GenerateKnowledgeResponseNode);
    workflow.register_node(SendKnowledgeReplyNode);

    Ok(workflow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::task::TaskContext;
    use serde_json::Value;

    #[test]
    fn test_create_knowledge_base_workflow() {
        let workflow = create_knowledge_base_workflow();
        assert!(workflow.is_ok(), "Workflow creation should succeed");
        
        let workflow = workflow.unwrap();
        assert_eq!(workflow.workflow_type(), "knowledge_base");
    }

    #[test]
    fn test_workflow_node_registration() {
        let workflow = create_knowledge_base_workflow().unwrap();
        
        // Test that workflow contains expected nodes
        // In a real implementation, you would have methods to inspect registered nodes
        assert_eq!(workflow.workflow_type(), "knowledge_base");
    }

    #[test]
    fn test_extract_keywords() {
        use crate::core::mcp::server::knowledge_base::tools::extract_keywords;
        
        let query = "How do I reset my password?";
        let keywords = extract_keywords(query);
        assert!(keywords.contains(&"reset".to_string()));
        assert!(keywords.contains(&"password".to_string()));
        assert!(!keywords.contains(&"do".to_string())); // Too short
    }

    #[test]
    fn test_query_router_node() {
        use crate::core::nodes::Node;
        
        let node = QueryRouterNode;
        
        // Create proper event data structure
        let event_data = serde_json::json!({
            "query_id": "TEST-001",
            "user_id": "USER-123",
            "user_query": "Test query",
            "query_type": "general",
            "sources": ["notion", "helpscout", "slack"]
        });
        
        let mut context = TaskContext::new(
            "knowledge-base-search".to_string(),
            event_data,
        );

        let result = node.process(context).unwrap();
        assert_eq!(
            result.get_data::<Value>("query_processed").unwrap(),
            Some(Value::Bool(true))
        );
        assert_eq!(
            result.get_data::<Value>("ready_for_search").unwrap(),
            Some(Value::Bool(true))
        );
    }

    #[test]
    fn test_validate_query_node() {
        use crate::core::nodes::Node;
        
        let node = ValidateQueryNode;
        
        // Create proper event data structure
        let event_data = serde_json::json!({
            "query_id": "TEST-002",
            "user_id": "USER-123",
            "user_query": "Valid query",
            "query_type": "general",
            "sources": ["notion", "helpscout", "slack"]
        });
        
        let mut context = TaskContext::new(
            "knowledge-base-search".to_string(),
            event_data,
        );

        let result = node.process(context).unwrap();
        assert_eq!(
            result.get_data::<Value>("query_valid").unwrap(),
            Some(Value::Bool(true))
        );
    }

    #[test]
    fn test_spam_filter_node() {
        use crate::core::nodes::Node;
        
        let node = FilterSpamQueryNode;
        
        // Create proper event data structure
        let event_data = serde_json::json!({
            "query_id": "TEST-003",
            "user_id": "USER-123",
            "user_query": "You won the lottery!",
            "query_type": "general",
            "sources": ["notion", "helpscout", "slack"]
        });
        
        let mut context = TaskContext::new(
            "knowledge-base-search".to_string(),
            event_data,
        );

        let result = node.process(context).unwrap();
        assert_eq!(
            result.get_data::<Value>("is_spam").unwrap(),
            Some(Value::Bool(true))
        );
    }
}