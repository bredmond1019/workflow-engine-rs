# Tutorial 4: MCP Integration - Connecting External Services

Welcome to the Model Context Protocol (MCP) integration tutorial! MCP is the system's powerful framework for connecting to external services like Notion, HelpScout, and Slack. By the end of this tutorial, you'll understand how to integrate external tools into your workflows and build sophisticated multi-service applications.

## What is MCP (Model Context Protocol)?

MCP is a standardized protocol that allows AI systems to interact with external tools and services. Think of it as a universal adapter that lets your AI workflows:

- Search through Notion databases
- Query HelpScout knowledge bases  
- Search Slack conversations
- Call any REST API or external service
- Maintain context across service calls

## Why Use MCP Integration?

Instead of building custom integrations for every service, MCP provides:

- **Standardized Interface**: One protocol to connect to multiple services
- **Type Safety**: Structured input/output with validation
- **Connection Management**: Automatic pooling, retries, and error handling
- **Transport Flexibility**: HTTP, WebSocket, and stdio transports
- **Built-in Tools**: Ready-to-use integrations for popular services

## Understanding the Current MCP Architecture

Let's examine how MCP works in this system:

### MCP Components

1. **MCP Clients**: Connect to external MCP servers
2. **MCP Servers**: Provide tools and resources (Notion, HelpScout, Slack)
3. **Transport Layer**: HTTP, WebSocket, or stdio communication
4. **Connection Pool**: Manages multiple client connections
5. **Tool Registry**: Registers available MCP tools

### Current MCP Structure

```rust
use backend::core::mcp::{
    clients::http::HttpMcpClient,
    transport::TransportType,
    connection_pool::McpConnectionPool,
};
```

## Building Your First MCP Integration: Knowledge Search

Let's create a practical workflow that demonstrates MCP integration by building a knowledge search system that queries multiple external services.

### Step 1: Understanding MCP Tool Nodes

The system already provides MCP-enabled nodes that you can use. Let's look at how they work:

```rust
use backend::core::mcp::server::knowledge_base::tools::{
    NotionSearchNode,
    HelpscoutSearchNode, 
    SlackSearchNode,
};
use backend::core::nodes::Node;
use backend::core::task::TaskContext;
use backend::core::error::WorkflowError;
use serde_json::json;

// Example of how the built-in MCP nodes work
#[derive(Debug)]
struct CustomMcpSearchNode {
    service_name: String,
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
```

### Step 2: Create an MCP Aggregator Node

Let's build a node that combines results from multiple MCP services:

```rust
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
```

### Step 3: Create an MCP Response Generator

Now let's create a node that generates intelligent responses using the MCP results:

```rust
#[derive(Debug)]
struct McpResponseGeneratorNode;

impl Node for McpResponseGeneratorNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("✍️ Generating response from MCP results...");
        
        // Get aggregated MCP results
        let aggregated_data = context.get_node_data::<serde_json::Value>("mcp_aggregated_results")?
            .ok_or_else(|| WorkflowError::ProcessingError {
                message: "No aggregated MCP results found".to_string()
            })?;
        
        let total_results = aggregated_data.get("total_results").and_then(|v| v.as_u64()).unwrap_or(0);
        let sources = aggregated_data.get("sources_searched").and_then(|v| v.as_array()).unwrap_or(&vec![]);
        let avg_relevance = aggregated_data.get("average_relevance").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let top_results = aggregated_data.get("top_results").and_then(|v| v.as_array()).unwrap_or(&vec![]);
        
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
```

### Step 4: Build the Complete MCP Workflow

Now let's put it all together in a runnable example:

```rust
use backend::core::task::TaskContext;
use serde_json::json;

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
```

## Understanding Real MCP Integration

The examples above demonstrate the concepts, but let's look at how to use the actual MCP system:

### Using Built-in MCP Nodes

The system provides ready-to-use MCP nodes:

```rust
use backend::workflows::knowledge_base_workflow::create_knowledge_base_workflow;
use backend::core::task::TaskContext;
use serde_json::json;

// Use the real knowledge base workflow with MCP integration
async fn use_real_mcp_workflow() -> Result<(), Box<dyn std::error::Error>> {
    // Create the knowledge base workflow (uses real MCP clients)
    let workflow = create_knowledge_base_workflow()?;
    
    // Create a knowledge search query
    let context = TaskContext::new(
        "knowledge_base".to_string(),
        json!({
            "query_id": "REAL-001",
            "user_id": "USER-123", 
            "user_query": "How do I set up SSL certificates?",
            "query_type": "technical",
            "sources": ["notion", "helpscout", "slack"]
        })
    );
    
    // Run the workflow (this will use real MCP connections)
    match workflow.run(context.event_data) {
        Ok(result) => {
            println!("✅ Real MCP search completed!");
            
            // Check actual results from MCP services
            if let Some(notion_results) = result.get_data::<serde_json::Value>("notion_search_results").ok().flatten() {
                println!("📚 Notion Results: {:?}", notion_results);
            }
        }
        Err(e) => println!("❌ MCP search failed: {}", e),
    }
    
    Ok(())
}
```

### MCP Connection Configuration

To use real MCP services, you need to configure the connection:

```bash
# Environment variables for MCP services
export NOTION_API_KEY="your_notion_integration_key"
export HELPSCOUT_API_KEY="your_helpscout_api_key"
export SLACK_BOT_TOKEN="xoxb-your-slack-bot-token"

# MCP server endpoints (if using external MCP servers)
export MCP_NOTION_URL="http://localhost:8002"
export MCP_HELPSCOUT_URL="http://localhost:8001"  
export MCP_SLACK_URL="http://localhost:8003"
```

### Starting MCP Test Servers

The system includes test MCP servers you can use for development:

```bash
# Start all MCP test servers
./scripts/start_test_servers.sh

# Or start individual servers
cd scripts
python customer_support_server.py &  # HelpScout on port 8001
python multi_service_mcp_server.py & # Notion/Slack on ports 8002/8003
```

## Advanced MCP Patterns

### Custom MCP Client Node

You can create your own MCP client nodes for custom services:

```rust
use backend::core::mcp::clients::http::HttpMcpClient;
use backend::core::mcp::transport::TransportType;

#[derive(Debug)]
struct CustomApiMcpNode {
    client: HttpMcpClient,
    api_name: String,
}

impl CustomApiMcpNode {
    fn new(api_name: String, base_url: String) -> Self {
        let transport = TransportType::Http { url: base_url };
        let client = HttpMcpClient::new(transport);
        
        Self { client, api_name }
    }
}

impl Node for CustomApiMcpNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Use the MCP client to call external API
        let input: serde_json::Value = context.get_event_data()?;
        
        // This would make an actual MCP call
        let mcp_result = json!({
            "tool": "custom_search",
            "result": "Custom API search result",
            "api": self.api_name
        });
        
        context.update_node(&format!("{}_mcp_result", self.api_name), mcp_result);
        Ok(context)
    }
}
```

### MCP Error Handling

Handle MCP failures gracefully:

```rust
impl Node for RobustMcpNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Try MCP call with retry logic
        let mut retry_count = 0;
        const MAX_RETRIES: u32 = 3;
        
        while retry_count < MAX_RETRIES {
            match self.call_mcp_service(&context) {
                Ok(result) => {
                    context.update_node("mcp_result", result);
                    return Ok(context);
                }
                Err(e) => {
                    retry_count += 1;
                    println!("MCP call failed (attempt {}): {}", retry_count, e);
                    
                    if retry_count >= MAX_RETRIES {
                        // Store error information and continue with fallback
                        context.update_node("mcp_error", json!({
                            "error": format!("MCP failed after {} retries: {}", MAX_RETRIES, e),
                            "fallback_used": true
                        }));
                        return Ok(context);
                    }
                    
                    // Exponential backoff
                    std::thread::sleep(std::time::Duration::from_millis(100 * 2u64.pow(retry_count)));
                }
            }
        }
        
        Ok(context)
    }
    
    fn call_mcp_service(&self, context: &TaskContext) -> Result<serde_json::Value, WorkflowError> {
        // Implement actual MCP service call
        Ok(json!({"mock": "result"}))
    }
}
```

## Running the MCP Examples

To run the MCP examples:

1. **Start the MCP test servers**:
   ```bash
   ./scripts/start_test_servers.sh
   ```

2. **Run the basic MCP example**:
   ```bash
   cargo run --example basic-workflow
   ```

3. **Create and run your custom MCP example**:
   ```bash
   # Create examples/tutorial_04_mcp_integration.rs with the code above
   cargo run --example tutorial_04_mcp_integration
   ```

## Key MCP Concepts You've Learned

✅ **MCP Protocol**: Universal interface for external service integration

✅ **MCP Nodes**: How to create workflow nodes that use MCP services

✅ **Service Aggregation**: Combining results from multiple MCP sources

✅ **Error Handling**: Graceful degradation when MCP services fail

✅ **Real Integration**: Using the built-in knowledge base workflow with real MCP clients

✅ **Custom MCP Clients**: Building your own MCP integrations for custom services

## Best Practices for MCP Integration

### 1. Connection Management
- Use connection pooling for multiple MCP calls
- Implement proper timeouts and retries
- Monitor connection health and implement circuit breakers

### 2. Error Handling
- Always provide fallback behavior when MCP services fail
- Log MCP errors for debugging and monitoring
- Use graceful degradation rather than complete failure

### 3. Performance Optimization
- Run MCP calls in parallel when possible
- Cache MCP results for repeated queries
- Implement request batching for bulk operations

### 4. Security
- Validate all MCP inputs and outputs
- Use secure authentication for MCP connections
- Implement rate limiting to prevent abuse

## What's Next?

You now understand MCP integration! Continue your learning journey:

1. **[Tutorial 5: Event Sourcing and State Management](./05-event-sourcing.md)** - Learn persistent state management
2. **Experiment with Real MCP** - Set up actual Notion/HelpScout/Slack integrations
3. **Build Custom MCP Services** - Create your own MCP servers for custom tools

MCP opens up endless possibilities for integrating AI workflows with external services. Start building!