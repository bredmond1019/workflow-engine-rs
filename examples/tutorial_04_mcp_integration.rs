//! Tutorial 4: MCP Integration Example
//! 
//! This example demonstrates MCP (Model Context Protocol) integration concepts
//! with a knowledge search workflow that queries multiple external services.

use backend::core::task::TaskContext;
use backend::core::nodes::Node;
use backend::core::error::WorkflowError;
use serde_json::json;

/// Node that simulates MCP search for a specific service
#[derive(Debug)]
struct CustomMcpSearchNode {
    service_name: String,
}

impl CustomMcpSearchNode {
    fn new(service_name: String) -> Self {
        Self { service_name }
    }
    
    fn search_notion(&self, query: &str) -> Result<Vec<serde_json::Value>, WorkflowError> {
        // This would use the actual Notion MCP client
        Ok(vec![
            json!({
                "title": "SSL Configuration Guide",
                "url": "https://notion.so/ssl-config", 
                "snippet": format!("Guide for configuring SSL with query: {}", query),
                "relevance": 0.95,
                "source": "notion"
            }),
            json!({
                "title": "Security Best Practices",
                "url": "https://notion.so/security-practices",
                "snippet": "Security guidelines and best practices",
                "relevance": 0.87,
                "source": "notion"
            })
        ])
    }
    
    fn search_helpscout(&self, query: &str) -> Result<Vec<serde_json::Value>, WorkflowError> {
        // This would use the actual HelpScout MCP client
        Ok(vec![
            json!({
                "title": "Troubleshooting SSL Issues",
                "url": "https://helpscout.com/ssl-troubleshoot",
                "snippet": format!("Common SSL problems for: {}", query),
                "relevance": 0.78,
                "source": "helpscout"
            })
        ])
    }
    
    fn search_slack(&self, query: &str) -> Result<Vec<serde_json::Value>, WorkflowError> {
        // This would use the actual Slack MCP client
        Ok(vec![
            json!({
                "channel": "#engineering",
                "message": format!("Discussion about {}", query),
                "timestamp": "2024-01-15T10:30:00Z",
                "user": "john.doe",
                "relevance": 0.65,
                "source": "slack"
            })
        ])
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
            context.update_node("mcp_search_error", json!({
                "error": "No query provided for MCP search"
            }));
            return Ok(context);
        }
        
        // Simulate MCP tool call (in reality, this would use the actual MCP client)
        let search_results = match self.service_name.as_str() {
            "notion" => self.search_notion(query)?,
            "helpscout" => self.search_helpscout(query)?,
            "slack" => self.search_slack(query)?,
            _ => vec![]
        };
        
        // Store results in context
        context.update_node(&format!("{}_search_results", self.service_name), json!({
            "query": query,
            "results": search_results,
            "result_count": search_results.len(),
            "service": self.service_name,
            "searched_at": chrono::Utc::now()
        }));
        
        println!("   ✅ Found {} results from {}", search_results.len(), self.service_name);
        
        Ok(context)
    }
}

/// Node that aggregates results from multiple MCP services
#[derive(Debug)]
struct McpAggregatorNode;

impl Node for McpAggregatorNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("📊 Aggregating MCP search results...");
        
        // Collect results from all MCP services
        let mut all_results = Vec::new();
        let mut sources_searched = Vec::new();
        
        // Check for Notion results
        if let Some(notion_data) = context.get_node_data::<serde_json::Value>("notion_search_results")? {
            if let Some(results) = notion_data.get("results").and_then(|v| v.as_array()) {
                all_results.extend(results.clone());
                sources_searched.push("notion");
            }
        }
        
        // Check for HelpScout results  
        if let Some(helpscout_data) = context.get_node_data::<serde_json::Value>("helpscout_search_results")? {
            if let Some(results) = helpscout_data.get("results").and_then(|v| v.as_array()) {
                all_results.extend(results.clone());
                sources_searched.push("helpscout");
            }
        }
        
        // Check for Slack results
        if let Some(slack_data) = context.get_node_data::<serde_json::Value>("slack_search_results")? {
            if let Some(results) = slack_data.get("results").and_then(|v| v.as_array()) {
                all_results.extend(results.clone());
                sources_searched.push("slack");
            }
        }
        
        // Sort results by relevance
        all_results.sort_by(|a, b| {
            let relevance_a = a.get("relevance").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let relevance_b = b.get("relevance").and_then(|v| v.as_f64()).unwrap_or(0.0);
            relevance_b.partial_cmp(&relevance_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Calculate statistics
        let total_results = all_results.len();
        let avg_relevance = if total_results > 0 {
            let sum: f64 = all_results.iter()
                .filter_map(|r| r.get("relevance").and_then(|v| v.as_f64()))
                .sum();
            sum / total_results as f64
        } else {
            0.0
        };
        
        // Store aggregated results
        context.update_node("mcp_aggregated_results", json!({
            "total_results": total_results,
            "sources_searched": sources_searched,
            "average_relevance": avg_relevance,
            "top_results": all_results.iter().take(10).collect::<Vec<_>>(),
            "all_results": all_results,
            "aggregated_at": chrono::Utc::now()
        }));
        
        println!("   📈 Aggregated {} results from {} sources (avg relevance: {:.2})", 
                 total_results, sources_searched.len(), avg_relevance);
        
        Ok(context)
    }
}

/// Node that generates intelligent responses using MCP results
#[derive(Debug)]
struct McpResponseGeneratorNode;

impl McpResponseGeneratorNode {
    fn format_sources(sources: &[serde_json::Value]) -> String {
        let source_names: Vec<String> = sources.iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        
        match source_names.len() {
            0 => "our knowledge sources".to_string(),
            1 => source_names[0].clone(),
            2 => format!("{} and {}", source_names[0], source_names[1]),
            _ => format!("{}, and {}", source_names[..source_names.len()-1].join(", "), source_names.last().unwrap())
        }
    }
    
    fn format_top_results(results: &[serde_json::Value], max_results: usize) -> String {
        let mut formatted = String::new();
        
        for (i, result) in results.iter().take(max_results).enumerate() {
            let title = result.get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Untitled");
            let snippet = result.get("snippet")
                .and_then(|v| v.as_str())
                .unwrap_or("No description available");
            let source = result.get("source")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let relevance = result.get("relevance")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            
            formatted.push_str(&format!(
                "{}. **{}** (from {}, relevance: {:.0}%)\n   {}\n\n",
                i + 1,
                title,
                source,
                relevance * 100.0,
                snippet
            ));
        }
        
        formatted
    }
}

impl Node for McpResponseGeneratorNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("✍️ Generating response from MCP results...");
        
        // Get aggregated MCP results
        let aggregated_data = context.get_node_data::<serde_json::Value>("mcp_aggregated_results")?
            .ok_or_else(|| WorkflowError::ProcessingError {
                message: "No aggregated MCP results found".to_string()
            })?;
        
        let total_results = aggregated_data.get("total_results").and_then(|v| v.as_u64()).unwrap_or(0);
        let empty_vec = vec![];
        let sources = aggregated_data.get("sources_searched").and_then(|v| v.as_array()).unwrap_or(&empty_vec);
        let avg_relevance = aggregated_data.get("average_relevance").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let empty_vec2 = vec![];
        let top_results = aggregated_data.get("top_results").and_then(|v| v.as_array()).unwrap_or(&empty_vec2);
        
        // Get original query for context
        let input: serde_json::Value = context.get_event_data()?;
        let original_query = input.get("user_query").and_then(|v| v.as_str()).unwrap_or("");
        
        // Generate response based on results quality
        let response = if total_results == 0 {
            format!(
                "I searched {} for information about '{}', but unfortunately didn't find any relevant results. \
                You might want to try rephrasing your question or checking if the information exists in these sources.",
                Self::format_sources(&sources),
                original_query
            )
        } else if avg_relevance > 0.8 {
            format!(
                "I found {} highly relevant results about '{}' from {}. Here are the top findings:\n\n{}",
                total_results,
                original_query,
                Self::format_sources(&sources),
                Self::format_top_results(&top_results, 3)
            )
        } else if avg_relevance > 0.5 {
            format!(
                "I found {} moderately relevant results about '{}' from {}. The best matches are:\n\n{}",
                total_results,
                original_query,
                Self::format_sources(&sources),
                Self::format_top_results(&top_results, 2)
            )
        } else {
            format!(
                "I found {} results about '{}' from {}, but they have low relevance scores. \
                You might want to refine your search terms. Here's what I found:\n\n{}",
                total_results,
                original_query,
                Self::format_sources(&sources),
                Self::format_top_results(&top_results, 1)
            )
        };
        
        // Determine response quality
        let response_quality = if avg_relevance > 0.8 {
            "excellent"
        } else if avg_relevance > 0.6 {
            "good"  
        } else if avg_relevance > 0.4 {
            "fair"
        } else {
            "poor"
        };
        
        // Store the generated response
        context.update_node("mcp_response", json!({
            "response_text": response,
            "response_quality": response_quality,
            "sources_used": sources,
            "total_results_referenced": total_results,
            "confidence_score": avg_relevance,
            "generated_at": chrono::Utc::now()
        }));
        
        println!("   💬 Generated {} quality response using {} sources", 
                 response_quality, sources.len());
        
        Ok(context)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 MCP Knowledge Search Workflow");
    println!("=================================\n");
    
    // Create our MCP-enabled nodes
    let notion_search = CustomMcpSearchNode::new("notion".to_string());
    let helpscout_search = CustomMcpSearchNode::new("helpscout".to_string());
    let slack_search = CustomMcpSearchNode::new("slack".to_string());
    let aggregator = McpAggregatorNode;
    let response_generator = McpResponseGeneratorNode;
    
    // Test different types of queries
    let test_queries = vec![
        "How do I configure SSL certificates for production?",
        "What are the best practices for API rate limiting?",
        "How to troubleshoot database connection issues?",
        "What is the onboarding process for new customers?",
    ];
    
    // Process each query
    for (index, query) in test_queries.iter().enumerate() {
        println!("🔄 Processing Query #{}: \"{}\"", index + 1, query);
        println!("{}", "─".repeat(60));
        
        // Create task context with query
        let mut context = TaskContext::new(
            "mcp_knowledge_search".to_string(),
            json!({
                "query_id": format!("QUERY-{:03}", index + 1),
                "user_id": "USER-123",
                "user_query": query,
                "query_type": "knowledge_search",
                "sources": ["notion", "helpscout", "slack"]
            })
        );
        
        // Execute the MCP workflow pipeline
        context = notion_search.process(context)?;
        context = helpscout_search.process(context)?;
        context = slack_search.process(context)?;
        context = aggregator.process(context)?;
        context = response_generator.process(context)?;
        
        // Display results
        if let Some(response_data) = context.get_node_data::<serde_json::Value>("mcp_response")? {
            if let Some(response_text) = response_data.get("response_text").and_then(|v| v.as_str()) {
                println!("\n📝 Generated Response:");
                println!("{}", response_text);
                
                if let Some(quality) = response_data.get("response_quality").and_then(|v| v.as_str()) {
                    if let Some(confidence) = response_data.get("confidence_score").and_then(|v| v.as_f64()) {
                        println!("📊 Response Quality: {} (confidence: {:.0}%)", 
                                quality, confidence * 100.0);
                    }
                }
                
                if let Some(sources) = response_data.get("sources_used").and_then(|v| v.as_array()) {
                    let source_names: Vec<String> = sources.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                    println!("🔗 Sources Used: {}", source_names.join(", "));
                }
            }
        }
        
        println!("\n");
    }
    
    println!("✨ MCP workflow demonstration completed!");
    println!("\n🎯 What you learned:");
    println!("   - How to create MCP-enabled nodes for external service integration");
    println!("   - How to aggregate results from multiple MCP sources");
    println!("   - How to generate intelligent responses from MCP data");
    println!("   - How to handle MCP errors and missing data gracefully");
    
    Ok(())
}