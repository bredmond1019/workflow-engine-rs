//! Tutorial 4: MCP Integration Example
//! 
//! This example demonstrates MCP (Model Context Protocol) integration concepts
//! with a knowledge search workflow that queries various knowledge sources.

use workflow_engine_core::task::TaskContext;
use workflow_engine_core::nodes::Node;
use workflow_engine_core::error::WorkflowError;
use serde_json::json;

/// Node that simulates MCP search for a knowledge source
#[derive(Debug)]
struct CustomMcpSearchNode {
    service_name: String,
}

impl CustomMcpSearchNode {
    fn new(service_name: String) -> Self {
        Self { service_name }
    }
    
    fn search_knowledge_base(&self, query: &str) -> Result<Vec<serde_json::Value>, WorkflowError> {
        // This would use an actual MCP client
        match self.service_name.as_str() {
            "documentation" => Ok(vec![
                json!({
                    "title": "SSL Configuration Guide",
                    "url": "https://docs.example.com/ssl-config", 
                    "snippet": format!("Guide for configuring SSL with query: {}", query),
                    "relevance": 0.95,
                    "source": "documentation"
                }),
                json!({
                    "title": "Security Best Practices",
                    "url": "https://docs.example.com/security-practices",
                    "snippet": "Security guidelines and best practices",
                    "relevance": 0.87,
                    "source": "documentation"
                })
            ]),
            "knowledge_base" => Ok(vec![
                json!({
                    "title": "Troubleshooting SSL Issues",
                    "url": "https://kb.example.com/ssl-troubleshoot",
                    "snippet": format!("Common SSL problems for: {}", query),
                    "relevance": 0.78,
                    "source": "knowledge_base"
                })
            ]),
            "discussions" => Ok(vec![
                json!({
                    "title": "Engineering Discussion",
                    "content": format!("Team discussion about {}", query),
                    "timestamp": "2024-01-15T10:30:00Z",
                    "author": "john.doe",
                    "relevance": 0.65,
                    "source": "discussions"
                })
            ]),
            _ => Ok(vec![])
        }
    }
}

impl Node for CustomMcpSearchNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("🔍 Searching {} via MCP...", self.service_name);
        
        // Get the search query from context
        let input: serde_json::Value = context.get_event_data()?;
        let query = input.get("user_query")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        if query.is_empty() {
            return Err(WorkflowError::ValidationError {
                message: "No search query provided".to_string(),
            });
        }
        
        // Perform search based on service type
        let search_results = self.search_knowledge_base(query)?;
        
        println!("  📊 Found {} results from {}", search_results.len(), self.service_name);
        
        // Store results in context with service-specific key
        let results_key = format!("{}_search_results", self.service_name);
        context.set_data(&results_key, json!({
            "service": self.service_name,
            "query": query,
            "results_found": search_results.len(),
            "results": search_results
        }))?;
        
        // Also add to global results accumulator
        let mut all_results = context.get_data::<Vec<serde_json::Value>>("all_search_results")
            .unwrap_or_default()
            .unwrap_or_default();
        
        all_results.extend(search_results);
        context.set_data("all_search_results", json!(all_results))?;
        
        println!("  ✅ {} search completed", self.service_name);
        Ok(context)
    }
}

/// Node that aggregates and ranks search results from multiple MCP sources
#[derive(Debug)]
struct KnowledgeAggregatorNode;

impl Node for KnowledgeAggregatorNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("📊 Aggregating knowledge from all MCP sources...");
        
        // Collect results from all search nodes
        let all_results = context.get_data::<Vec<serde_json::Value>>("all_search_results")
            .unwrap_or_default()
            .unwrap_or_default();
        
        if all_results.is_empty() {
            return Err(WorkflowError::ProcessingError {
                message: "No search results found from any MCP source".to_string(),
            });
        }
        
        // Sort results by relevance
        let mut sorted_results = all_results;
        sorted_results.sort_by(|a, b| {
            let relevance_a = a.get("relevance").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let relevance_b = b.get("relevance").and_then(|v| v.as_f64()).unwrap_or(0.0);
            relevance_b.partial_cmp(&relevance_a).unwrap()
        });
        
        // Calculate statistics
        let total_results = sorted_results.len();
        let high_relevance_count = sorted_results.iter()
            .filter(|r| r.get("relevance").and_then(|v| v.as_f64()).unwrap_or(0.0) >= 0.8)
            .count();
        
        let aggregated_knowledge = json!({
            "total_results": total_results,
            "high_relevance_results": high_relevance_count,
            "top_results": sorted_results.into_iter().take(5).collect::<Vec<_>>(),
            "aggregation_timestamp": chrono::Utc::now().to_rfc3339(),
            "sources_queried": ["documentation", "knowledge_base", "discussions"]
        });
        
        context.set_data("aggregated_knowledge", aggregated_knowledge)?;
        
        println!("  📈 Aggregated {} results, {} high-relevance", total_results, high_relevance_count);
        println!("  ✅ Knowledge aggregation completed");
        
        Ok(context)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Tutorial 4: MCP Integration Example");
    println!("=" .repeat(60));
    
    // Create initial context with search query
    let mut context = TaskContext::new(
        "mcp_knowledge_search".to_string(),
        json!({
            "user_query": "SSL configuration best practices",
            "search_scope": "comprehensive",
            "max_results_per_source": 5
        })
    );
    
    println!("\n📋 Search Query: SSL configuration best practices");
    println!("🎯 Scope: Comprehensive search across multiple knowledge sources");
    
    // Create search nodes for different knowledge sources
    let search_nodes = vec![
        CustomMcpSearchNode::new("documentation".to_string()),
        CustomMcpSearchNode::new("knowledge_base".to_string()),
        CustomMcpSearchNode::new("discussions".to_string()),
    ];
    
    // Execute search across all MCP sources
    println!("\n🔄 Executing parallel MCP searches...");
    for node in search_nodes {
        context = node.process(context)?;
    }
    
    // Aggregate all results
    println!("\n🔄 Aggregating knowledge...");
    let aggregator = KnowledgeAggregatorNode;
    context = aggregator.process(context)?;
    
    // Display final results
    println!("\n📊 FINAL KNOWLEDGE SEARCH RESULTS");
    println!("=" .repeat(60));
    
    if let Ok(Some(aggregated)) = context.get_data::<serde_json::Value>("aggregated_knowledge") {
        let total_results = aggregated.get("total_results").and_then(|v| v.as_u64()).unwrap_or(0);
        let high_relevance = aggregated.get("high_relevance_results").and_then(|v| v.as_u64()).unwrap_or(0);
        
        println!("📈 Total Results: {}", total_results);
        println!("⭐ High Relevance Results: {}", high_relevance);
        
        if let Some(top_results) = aggregated.get("top_results").and_then(|v| v.as_array()) {
            println!("\n🏆 Top Results:");
            for (i, result) in top_results.iter().enumerate() {
                let title = result.get("title").and_then(|v| v.as_str()).unwrap_or("Unknown");
                let source = result.get("source").and_then(|v| v.as_str()).unwrap_or("unknown");
                let relevance = result.get("relevance").and_then(|v| v.as_f64()).unwrap_or(0.0);
                
                println!("  {}. {} (from {}) - Relevance: {:.2}", 
                    i + 1, title, source, relevance);
            }
        }
    }
    
    println!("\n✅ MCP Integration Tutorial Complete!");
    println!("\n💡 This example demonstrates:");
    println!("   • MCP client integration patterns");
    println!("   • Parallel knowledge source querying");
    println!("   • Result aggregation and ranking");
    println!("   • Multi-source knowledge synthesis");
    
    Ok(())
}