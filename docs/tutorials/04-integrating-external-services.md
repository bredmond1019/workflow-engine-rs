# Tutorial 4: External Service Integration with MCP

Welcome to the world of external service integration! The Model Context Protocol (MCP) is your gateway to connecting AI workflows with external systems like Notion, HelpScout, Slack, and custom APIs. By the end of this tutorial, you'll be building workflows that seamlessly integrate with real-world services.

## Understanding MCP: The Universal Connector

Think of MCP as a universal translator for AI systems. Just like how USB provides a standard way to connect different devices to your computer, MCP provides a standard way to connect AI agents to any external service.

### Why MCP Matters

Without MCP, integrating with external services looks like this:
```rust
// Without MCP: Different patterns for each service
let notion_client = NotionAPI::new(api_key);
let slack_client = SlackAPI::new(token, channel);
let helpscout_client = HelpScoutAPI::new(credentials);

// Different methods, different error handling, different patterns
let notion_result = notion_client.search_pages(query).await?;
let slack_result = slack_client.send_message(text).await?;
let helpscout_result = helpscout_client.create_ticket(data).await?;
```

With MCP, it's consistent across all services:
```rust
// With MCP: Same pattern for every service
let notion_client = HttpMCPClient::new("http://localhost:8002".to_string());
let slack_client = HttpMCPClient::new("http://localhost:8003".to_string());
let helpscout_client = HttpMCPClient::new("http://localhost:8001".to_string());

// Same methods, same error handling, same patterns
let notion_result = notion_client.call_tool("search_pages", Some(args)).await?;
let slack_result = slack_client.call_tool("send_message", Some(args)).await?;
let helpscout_result = helpscout_client.call_tool("create_ticket", Some(args)).await?;
```

## The MCP Architecture in Our System

Our system implements MCP with these key components:

### 1. Transport Layer (How Messages Travel)
- **HTTP Transport**: For stateless request/response communication
- **WebSocket Transport**: For persistent connections with real-time updates  
- **Stdio Transport**: For local process communication

### 2. Protocol Layer (Message Format)
- **Initialization**: Establishing capabilities between client and server
- **Tool Discovery**: Finding out what tools are available
- **Tool Execution**: Calling tools with parameters and getting results

### 3. Client Layer (Your Interface)
- **MCPClient Trait**: Common interface for all transports
- **HttpMCPClient**: HTTP implementation for cross-service communication
- **Connection Pooling**: Efficient resource management

## Building Your First MCP Integration

Let's start by building a workflow that integrates with a Notion knowledge base to answer customer questions.

### Step 1: Understanding the MCP Client

```rust
use backend::core::mcp::clients::{HttpMCPClient, MCPClient};
use backend::core::mcp::protocol::{ToolDefinition, CallToolResult};
use backend::core::error::WorkflowError;
use std::collections::HashMap;
use serde_json::json;

#[derive(Debug)]
struct NotionIntegrationNode {
    mcp_client: HttpMCPClient,
    initialized: bool,
}

impl NotionIntegrationNode {
    fn new(notion_server_url: String) -> Self {
        Self {
            mcp_client: HttpMCPClient::new(notion_server_url),
            initialized: false,
        }
    }

    fn with_auth(notion_server_url: String, auth_token: String) -> Self {
        Self {
            mcp_client: HttpMCPClient::with_auth_token(notion_server_url, auth_token),
            initialized: false,
        }
    }

    async fn ensure_initialized(&mut self) -> Result<(), WorkflowError> {
        if !self.initialized {
            // Connect to the MCP server
            self.mcp_client.connect().await?;
            
            // Initialize with client information
            self.mcp_client.initialize("ai-workflow-system", "1.0.0").await?;
            
            self.initialized = true;
            println!("✅ Connected to Notion MCP server");
        }
        Ok(())
    }

    async fn search_knowledge_base(&mut self, query: &str) -> Result<serde_json::Value, WorkflowError> {
        self.ensure_initialized().await?;
        
        // Prepare arguments for the search tool
        let mut args = HashMap::new();
        args.insert("query".to_string(), json!(query));
        args.insert("limit".to_string(), json!(5));
        args.insert("include_content".to_string(), json!(true));
        
        // Call the Notion search tool
        let result = self.mcp_client.call_tool("search_pages", Some(args)).await?;
        
        // Extract the response content
        if let Some(content) = result.content.first() {
            match content {
                backend::core::mcp::protocol::ToolContent::Text { text } => {
                    // Parse the JSON response
                    serde_json::from_str(text).map_err(|e| WorkflowError::DeserializationError {
                        message: format!("Failed to parse Notion response: {}", e)
                    })
                },
                _ => Err(WorkflowError::ProcessingError {
                    message: "Unexpected response format from Notion".to_string()
                })
            }
        } else {
            Err(WorkflowError::ProcessingError {
                message: "Empty response from Notion".to_string()
            })
        }
    }

    async fn discover_available_tools(&mut self) -> Result<Vec<ToolDefinition>, WorkflowError> {
        self.ensure_initialized().await?;
        
        let tools = self.mcp_client.list_tools().await?;
        
        println!("📋 Available Notion tools:");
        for tool in &tools {
            println!("   • {} - {}", tool.name, tool.description.as_deref().unwrap_or("No description"));
        }
        
        Ok(tools)
    }
}
```

### Step 2: Creating a Multi-Service Integration Node

Now let's build a node that can integrate with multiple services:

```rust
use backend::core::nodes::Node;
use backend::core::task::TaskContext;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct CustomerQuery {
    customer_id: String,
    question: String,
    category: Option<String>,
    urgency: Option<String>,
}

#[derive(Debug)]
struct CustomerSupportNode {
    notion_client: Option<NotionIntegrationNode>,
    helpscout_client: Option<HttpMCPClient>,
    slack_client: Option<HttpMCPClient>,
}

impl CustomerSupportNode {
    fn new() -> Self {
        Self {
            notion_client: None,
            helpscout_client: None,
            slack_client: None,
        }
    }

    fn with_notion(mut self, server_url: String) -> Self {
        self.notion_client = Some(NotionIntegrationNode::new(server_url));
        self
    }

    fn with_helpscout(mut self, server_url: String) -> Self {
        self.helpscout_client = Some(HttpMCPClient::new(server_url));
        self
    }

    fn with_slack(mut self, server_url: String) -> Self {
        self.slack_client = Some(HttpMCPClient::new(server_url));
        self
    }

    async fn search_knowledge_base(&mut self, query: &str) -> Result<Vec<serde_json::Value>, WorkflowError> {
        let mut results = Vec::new();

        // Search Notion if available
        if let Some(notion_client) = &mut self.notion_client {
            match notion_client.search_knowledge_base(query).await {
                Ok(notion_results) => {
                    results.push(json!({
                        "source": "notion",
                        "results": notion_results,
                        "success": true
                    }));
                    println!("📚 Found Notion results for: {}", query);
                },
                Err(e) => {
                    results.push(json!({
                        "source": "notion", 
                        "error": e.to_string(),
                        "success": false
                    }));
                    println!("⚠️ Notion search failed: {}", e);
                }
            }
        }

        // Search HelpScout if available
        if let Some(helpscout_client) = &mut self.helpscout_client {
            match self.search_helpscout(helpscout_client, query).await {
                Ok(helpscout_results) => {
                    results.push(json!({
                        "source": "helpscout",
                        "results": helpscout_results,
                        "success": true
                    }));
                    println!("🎫 Found HelpScout results for: {}", query);
                },
                Err(e) => {
                    results.push(json!({
                        "source": "helpscout",
                        "error": e.to_string(), 
                        "success": false
                    }));
                    println!("⚠️ HelpScout search failed: {}", e);
                }
            }
        }

        Ok(results)
    }

    async fn search_helpscout(&self, client: &mut HttpMCPClient, query: &str) -> Result<serde_json::Value, WorkflowError> {
        // Initialize if needed
        client.connect().await?;
        client.initialize("ai-workflow-system", "1.0.0").await?;
        
        // Prepare search arguments
        let mut args = HashMap::new();
        args.insert("query".to_string(), json!(query));
        args.insert("type".to_string(), json!("conversations"));
        args.insert("limit".to_string(), json!(10));
        
        // Call HelpScout search tool
        let result = client.call_tool("search_conversations", Some(args)).await?;
        
        if let Some(content) = result.content.first() {
            match content {
                backend::core::mcp::protocol::ToolContent::Text { text } => {
                    serde_json::from_str(text).map_err(|e| WorkflowError::DeserializationError {
                        message: format!("Failed to parse HelpScout response: {}", e)
                    })
                },
                _ => Err(WorkflowError::ProcessingError {
                    message: "Unexpected response format from HelpScout".to_string()
                })
            }
        } else {
            Ok(json!({"results": []}))
        }
    }

    async fn create_support_ticket(&mut self, customer_id: &str, question: &str, urgency: &str) -> Result<serde_json::Value, WorkflowError> {
        if let Some(helpscout_client) = &mut self.helpscout_client {
            // Initialize if needed
            helpscout_client.connect().await?;
            helpscout_client.initialize("ai-workflow-system", "1.0.0").await?;
            
            // Prepare ticket creation arguments
            let mut args = HashMap::new();
            args.insert("customer_id".to_string(), json!(customer_id));
            args.insert("subject".to_string(), json!(format!("Support Request: {}", 
                question.chars().take(50).collect::<String>())));
            args.insert("message".to_string(), json!(question));
            args.insert("priority".to_string(), json!(urgency));
            args.insert("type".to_string(), json!("email"));
            
            // Create the ticket
            let result = helpscout_client.call_tool("create_conversation", Some(args)).await?;
            
            if let Some(content) = result.content.first() {
                match content {
                    backend::core::mcp::protocol::ToolContent::Text { text } => {
                        serde_json::from_str(text).map_err(|e| WorkflowError::DeserializationError {
                            message: format!("Failed to parse ticket creation response: {}", e)
                        })
                    },
                    _ => Err(WorkflowError::ProcessingError {
                        message: "Unexpected response format".to_string()
                    })
                }
            } else {
                Err(WorkflowError::ProcessingError {
                    message: "Empty response from ticket creation".to_string()
                })
            }
        } else {
            Err(WorkflowError::ProcessingError {
                message: "HelpScout client not configured".to_string()
            })
        }
    }

    async fn notify_team(&mut self, message: &str, channel: &str) -> Result<(), WorkflowError> {
        if let Some(slack_client) = &mut self.slack_client {
            // Initialize if needed
            slack_client.connect().await?;
            slack_client.initialize("ai-workflow-system", "1.0.0").await?;
            
            // Prepare message arguments
            let mut args = HashMap::new();
            args.insert("channel".to_string(), json!(channel));
            args.insert("text".to_string(), json!(message));
            args.insert("username".to_string(), json!("AI Support Assistant"));
            
            // Send the message
            let _result = slack_client.call_tool("send_message", Some(args)).await?;
            
            println!("💬 Notified team in #{}: {}", channel, message);
            Ok(())
        } else {
            println!("⚠️ Slack client not configured - skipping team notification");
            Ok(())
        }
    }
}

impl Node for CustomerSupportNode {
    fn node_name(&self) -> String {
        "Customer Support Integration".to_string()
    }

    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Note: This simplified version shows the pattern. In practice, you'd use async
        let runtime = tokio::runtime::Runtime::new().unwrap();
        
        runtime.block_on(async {
            println!("🤝 Step: Integrating with external support services...");
            
            // Extract customer query
            let query: CustomerQuery = context.get_event_data()?;
            
            // Create a mutable copy for async operations
            let mut node_copy = CustomerSupportNode::new()
                .with_notion(std::env::var("NOTION_MCP_URL").unwrap_or_else(|_| "http://localhost:8002".to_string()))
                .with_helpscout(std::env::var("HELPSCOUT_MCP_URL").unwrap_or_else(|_| "http://localhost:8001".to_string()))
                .with_slack(std::env::var("SLACK_MCP_URL").unwrap_or_else(|_| "http://localhost:8003".to_string()));
            
            // Search knowledge base for relevant information
            let knowledge_results = node_copy.search_knowledge_base(&query.question).await?;
            
            // Determine if we have a good answer from knowledge base
            let has_answer = knowledge_results.iter().any(|result| {
                result.get("success").and_then(|v| v.as_bool()).unwrap_or(false) &&
                result.get("results").and_then(|r| r.as_array()).map(|arr| arr.len() > 0).unwrap_or(false)
            });
            
            let mut ticket_id = None;
            
            // If no answer found, create a support ticket
            if !has_answer || query.urgency.as_deref() == Some("high") {
                let urgency = query.urgency.as_deref().unwrap_or("normal");
                match node_copy.create_support_ticket(&query.customer_id, &query.question, urgency).await {
                    Ok(ticket_result) => {
                        ticket_id = ticket_result.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());
                        println!("🎫 Created support ticket: {:?}", ticket_id);
                        
                        // Notify team for high urgency tickets
                        if urgency == "high" {
                            let notification = format!(
                                "🚨 High urgency ticket created: {} - Customer: {} - Question: {}", 
                                ticket_id.as_deref().unwrap_or("Unknown"),
                                query.customer_id,
                                query.question.chars().take(100).collect::<String>()
                            );
                            let _ = node_copy.notify_team(&notification, "customer-support").await;
                        }
                    },
                    Err(e) => {
                        println!("❌ Failed to create ticket: {}", e);
                    }
                }
            }
            
            // Store comprehensive results
            context.update_node("external_integration", json!({
                "knowledge_search": {
                    "query": query.question,
                    "results": knowledge_results,
                    "has_answer": has_answer
                },
                "support_ticket": {
                    "created": ticket_id.is_some(),
                    "ticket_id": ticket_id,
                    "reason": if !has_answer { "no_knowledge_found" } else { "high_urgency" }
                },
                "customer_info": {
                    "customer_id": query.customer_id,
                    "category": query.category,
                    "urgency": query.urgency
                },
                "processing_timestamp": chrono::Utc::now(),
                "services_used": {
                    "notion": true,
                    "helpscout": ticket_id.is_some(),
                    "slack": ticket_id.is_some() && query.urgency.as_deref() == Some("high")
                }
            }));
            
            // Add metadata for the next nodes
            context.set_metadata("has_knowledge_answer", has_answer)?;
            context.set_metadata("ticket_created", ticket_id.is_some())?;
            context.set_metadata("services_contacted", 
                knowledge_results.iter().filter(|r| r.get("success").and_then(|v| v.as_bool()).unwrap_or(false)).count())?;
            
            println!("   ✅ External integration complete: {} services contacted, ticket: {}", 
                     knowledge_results.len(),
                     if ticket_id.is_some() { "created" } else { "not needed" });
            
            Ok(context)
        })
    }
}
```

### Step 3: Building a Robust MCP Connection Manager

For production use, you'll want proper connection management:

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

#[derive(Debug)]
pub struct MCPConnectionManager {
    connections: Arc<RwLock<HashMap<String, Box<dyn MCPClient>>>>,
    connection_config: HashMap<String, MCPConnectionConfig>,
}

#[derive(Debug, Clone)]
pub struct MCPConnectionConfig {
    pub name: String,
    pub url: String,
    pub transport_type: String, // "http", "websocket", "stdio"
    pub auth_token: Option<String>,
    pub timeout_ms: Option<u64>,
    pub retry_attempts: Option<u32>,
}

impl MCPConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            connection_config: HashMap::new(),
        }
    }

    pub fn add_service(&mut self, config: MCPConnectionConfig) {
        self.connection_config.insert(config.name.clone(), config);
    }

    pub async fn get_client(&self, service_name: &str) -> Result<Box<dyn MCPClient>, WorkflowError> {
        // Check if we already have a connection
        {
            let connections = self.connections.read().await;
            if let Some(client) = connections.get(service_name) {
                if client.is_connected() {
                    // Return a new instance with the same configuration
                    // Note: This is simplified - in practice you'd implement proper cloning
                }
            }
        }

        // Create new connection
        let config = self.connection_config.get(service_name)
            .ok_or_else(|| WorkflowError::ConfigurationError {
                message: format!("No configuration found for service: {}", service_name)
            })?;

        let mut client: Box<dyn MCPClient> = match config.transport_type.as_str() {
            "http" => {
                if let Some(auth_token) = &config.auth_token {
                    Box::new(HttpMCPClient::with_auth_token(config.url.clone(), auth_token.clone()))
                } else {
                    Box::new(HttpMCPClient::new(config.url.clone()))
                }
            },
            "websocket" => {
                Box::new(WebSocketMCPClient::new(config.url.clone()))
            },
            _ => return Err(WorkflowError::ConfigurationError {
                message: format!("Unsupported transport type: {}", config.transport_type)
            })
        };

        // Initialize the connection
        client.connect().await?;
        client.initialize("ai-workflow-system", "1.0.0").await?;

        // Store the connection
        {
            let mut connections = self.connections.write().await;
            connections.insert(service_name.to_string(), client);
        }

        // Return the client
        let connections = self.connections.read().await;
        let client = connections.get(service_name).unwrap();
        // Note: In practice, you'd implement proper client retrieval
        Err(WorkflowError::ProcessingError {
            message: "Client retrieval not fully implemented in this example".to_string()
        })
    }

    pub async fn call_service_tool(
        &self, 
        service_name: &str, 
        tool_name: &str, 
        arguments: Option<HashMap<String, serde_json::Value>>
    ) -> Result<CallToolResult, WorkflowError> {
        let mut client = self.get_client(service_name).await?;
        client.call_tool(tool_name, arguments).await
    }

    pub async fn discover_service_tools(&self, service_name: &str) -> Result<Vec<ToolDefinition>, WorkflowError> {
        let mut client = self.get_client(service_name).await?;
        client.list_tools().await
    }

    pub async fn close_all_connections(&self) -> Result<(), WorkflowError> {
        let mut connections = self.connections.write().await;
        
        for (service_name, client) in connections.iter_mut() {
            match client.disconnect().await {
                Ok(_) => println!("✅ Disconnected from {}", service_name),
                Err(e) => println!("⚠️ Error disconnecting from {}: {}", service_name, e),
            }
        }
        
        connections.clear();
        Ok(())
    }
}
```

### Step 4: Creating a Complete Multi-Service Workflow

Let's put everything together in a comprehensive example:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 Multi-Service Integration Workflow");
    println!("====================================\n");
    
    // Test scenarios with different types of customer queries
    let test_queries = vec![
        json!({
            "customer_id": "CUST-001",
            "question": "How do I reset my password?",
            "category": "account",
            "urgency": "normal"
        }),
        json!({
            "customer_id": "CUST-002",
            "question": "My payment failed and I can't access my account. This is urgent!",
            "category": "billing",
            "urgency": "high"
        }),
        json!({
            "customer_id": "CUST-003",
            "question": "What are the system requirements for your software?",
            "category": "technical",
            "urgency": "low"
        }),
    ];
    
    // Create the integration node
    let support_node = CustomerSupportNode::new()
        .with_notion("http://localhost:8002".to_string())
        .with_helpscout("http://localhost:8001".to_string())
        .with_slack("http://localhost:8003".to_string());
    
    // Process each query
    for (index, query_data) in test_queries.iter().enumerate() {
        println!("🔄 Processing Query #{}: {}", 
                 index + 1, 
                 query_data.get("question").and_then(|v| v.as_str()).unwrap_or("Unknown"));
        println!("─".repeat(60));
        
        // Create task context
        let mut context = TaskContext::new(
            "customer_support_integration".to_string(),
            query_data.clone()
        );
        
        // Process through the integration node
        context = support_node.process(context)?;
        
        // Display results
        if let Some(integration_results) = context.get_node_data::<serde_json::Value>("external_integration")? {
            println!("\n📊 Integration Results:");
            println!("─".repeat(40));
            
            // Show knowledge search results
            if let Some(knowledge) = integration_results.get("knowledge_search") {
                println!("🔍 Knowledge Search:");
                if let Some(has_answer) = knowledge.get("has_answer").and_then(|v| v.as_bool()) {
                    if has_answer {
                        println!("   ✅ Found relevant information in knowledge base");
                        if let Some(results) = knowledge.get("results").and_then(|v| v.as_array()) {
                            for result in results {
                                if let Some(source) = result.get("source").and_then(|v| v.as_str()) {
                                    let success = result.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
                                    println!("   • {}: {}", source, if success { "✅" } else { "❌" });
                                }
                            }
                        }
                    } else {
                        println!("   ❌ No relevant information found");
                    }
                }
            }
            
            // Show ticket creation results
            if let Some(ticket) = integration_results.get("support_ticket") {
                if let Some(created) = ticket.get("created").and_then(|v| v.as_bool()) {
                    if created {
                        let ticket_id = ticket.get("ticket_id").and_then(|v| v.as_str()).unwrap_or("Unknown");
                        let reason = ticket.get("reason").and_then(|v| v.as_str()).unwrap_or("Unknown");
                        println!("🎫 Support Ticket:");
                        println!("   Created: {} (reason: {})", ticket_id, reason);
                    } else {
                        println!("🎫 Support Ticket: Not needed");
                    }
                }
            }
            
            // Show services used
            if let Some(services) = integration_results.get("services_used") {
                println!("🔧 Services Contacted:");
                if let Some(notion) = services.get("notion").and_then(|v| v.as_bool()) {
                    println!("   • Notion: {}", if notion { "✅" } else { "❌" });
                }
                if let Some(helpscout) = services.get("helpscout").and_then(|v| v.as_bool()) {
                    println!("   • HelpScout: {}", if helpscout { "✅" } else { "❌" });
                }
                if let Some(slack) = services.get("slack").and_then(|v| v.as_bool()) {
                    println!("   • Slack: {}", if slack { "✅" } else { "❌" });
                }
            }
        }
        
        println!("\n");
    }
    
    println!("✨ Multi-service integration demonstration completed!");
    Ok(())
}
```

## Advanced MCP Patterns

### 1. Error Handling and Retry Logic

```rust
use std::time::Duration;
use tokio::time::sleep;

pub struct ResilientMCPClient {
    client: HttpMCPClient,
    max_retries: u32,
    base_delay: Duration,
}

impl ResilientMCPClient {
    pub fn new(url: String, max_retries: u32) -> Self {
        Self {
            client: HttpMCPClient::new(url),
            max_retries,
            base_delay: Duration::from_millis(1000),
        }
    }

    pub async fn call_tool_with_retry(
        &mut self,
        tool_name: &str,
        arguments: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<CallToolResult, WorkflowError> {
        let mut last_error = None;
        
        for attempt in 0..self.max_retries {
            match self.client.call_tool(tool_name, arguments.clone()).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    
                    if attempt < self.max_retries - 1 {
                        // Exponential backoff
                        let delay = self.base_delay * 2u32.pow(attempt);
                        println!("⏳ Attempt {} failed, retrying in {:?}...", attempt + 1, delay);
                        sleep(delay).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| WorkflowError::ProcessingError {
            message: "All retry attempts failed".to_string()
        }))
    }
}
```

### 2. Connection Pooling for Performance

```rust
use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct MCPConnectionPool {
    connections: Arc<RwLock<Vec<HttpMCPClient>>>,
    semaphore: Arc<Semaphore>,
    base_url: String,
    pool_size: usize,
}

impl MCPConnectionPool {
    pub fn new(base_url: String, pool_size: usize) -> Self {
        Self {
            connections: Arc::new(RwLock::new(Vec::new())),
            semaphore: Arc::new(Semaphore::new(pool_size)),
            base_url,
            pool_size,
        }
    }

    pub async fn acquire(&self) -> Result<PooledConnection, WorkflowError> {
        // Acquire semaphore permit
        let permit = self.semaphore.acquire().await.map_err(|_| {
            WorkflowError::ProcessingError {
                message: "Failed to acquire connection permit".to_string()
            }
        })?;

        // Try to get existing connection
        {
            let mut connections = self.connections.write().await;
            if let Some(client) = connections.pop() {
                return Ok(PooledConnection {
                    client,
                    pool: self.connections.clone(),
                    _permit: permit,
                });
            }
        }

        // Create new connection
        let mut client = HttpMCPClient::new(self.base_url.clone());
        client.connect().await?;
        client.initialize("ai-workflow-system", "1.0.0").await?;

        Ok(PooledConnection {
            client,
            pool: self.connections.clone(),
            _permit: permit,
        })
    }
}

pub struct PooledConnection {
    client: HttpMCPClient,
    pool: Arc<RwLock<Vec<HttpMCPClient>>>,
    _permit: tokio::sync::SemaphorePermit<'_>,
}

impl Drop for PooledConnection {
    fn drop(&mut self) {
        // Return connection to pool
        let pool = self.pool.clone();
        let client = std::mem::replace(&mut self.client, HttpMCPClient::new("dummy".to_string()));
        
        tokio::spawn(async move {
            let mut connections = pool.write().await;
            connections.push(client);
        });
    }
}
```

### 3. Dynamic Tool Discovery and Routing

```rust
pub struct DynamicMCPRouter {
    services: HashMap<String, HttpMCPClient>,
    tool_catalog: HashMap<String, String>, // tool_name -> service_name
}

impl DynamicMCPRouter {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
            tool_catalog: HashMap::new(),
        }
    }

    pub async fn register_service(&mut self, name: String, url: String) -> Result<(), WorkflowError> {
        let mut client = HttpMCPClient::new(url);
        client.connect().await?;
        client.initialize("ai-workflow-system", "1.0.0").await?;
        
        // Discover tools from this service
        let tools = client.list_tools().await?;
        for tool in tools {
            self.tool_catalog.insert(tool.name.clone(), name.clone());
            println!("📋 Registered tool '{}' from service '{}'", tool.name, name);
        }
        
        self.services.insert(name, client);
        Ok(())
    }

    pub async fn call_any_tool(
        &mut self,
        tool_name: &str,
        arguments: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<CallToolResult, WorkflowError> {
        // Find which service provides this tool
        let service_name = self.tool_catalog.get(tool_name)
            .ok_or_else(|| WorkflowError::ProcessingError {
                message: format!("Tool '{}' not found in any registered service", tool_name)
            })?;

        // Get the service and call the tool
        let client = self.services.get_mut(service_name)
            .ok_or_else(|| WorkflowError::ProcessingError {
                message: format!("Service '{}' not found", service_name)
            })?;

        client.call_tool(tool_name, arguments).await
    }

    pub fn list_available_tools(&self) -> Vec<(String, String)> {
        // Returns (tool_name, service_name) pairs
        self.tool_catalog.iter().map(|(tool, service)| (tool.clone(), service.clone())).collect()
    }
}
```

## Testing Your MCP Integrations

### Unit Testing with Mock MCP Servers

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    // Create a mock MCP server for testing
    pub struct MockMCPServer {
        port: u16,
        responses: HashMap<String, serde_json::Value>,
    }
    
    impl MockMCPServer {
        pub fn new(port: u16) -> Self {
            Self {
                port,
                responses: HashMap::new(),
            }
        }
        
        pub fn set_tool_response(&mut self, tool_name: &str, response: serde_json::Value) {
            self.responses.insert(tool_name.to_string(), response);
        }
        
        pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
            // Start a simple HTTP server that responds to MCP requests
            // This is simplified - in practice you'd use a proper HTTP server
            Ok(())
        }
    }
    
    #[tokio::test]
    async fn test_notion_integration() {
        // Start mock server
        let mut mock_server = MockMCPServer::new(8999);
        mock_server.set_tool_response("search_pages", json!({
            "pages": [
                {"title": "Password Reset Guide", "content": "To reset your password..."}
            ]
        }));
        mock_server.start().await.unwrap();
        
        // Test the integration
        let mut node = NotionIntegrationNode::new("http://localhost:8999".to_string());
        let result = node.search_knowledge_base("password reset").await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.get("pages").is_some());
    }
    
    #[tokio::test]
    async fn test_customer_support_workflow() {
        let context = TaskContext::new(
            "test".to_string(),
            json!({
                "customer_id": "TEST-001",
                "question": "How do I reset my password?",
                "urgency": "normal"
            })
        );
        
        let support_node = CustomerSupportNode::new()
            .with_notion("http://localhost:8999".to_string());
        
        let result = support_node.process(context);
        assert!(result.is_ok());
        
        let context = result.unwrap();
        let integration_data = context.get_node_data::<serde_json::Value>("external_integration").unwrap().unwrap();
        assert!(integration_data.get("knowledge_search").is_some());
    }
}
```

### Integration Testing with Real Services

```rust
#[tokio::test]
#[ignore] // Only run with --ignored flag
async fn test_real_notion_integration() {
    // This test requires a real Notion MCP server running
    let mut node = NotionIntegrationNode::new("http://localhost:8002".to_string());
    
    // Test tool discovery
    let tools = node.discover_available_tools().await.unwrap();
    assert!(!tools.is_empty(), "Should discover at least one tool");
    
    // Test actual search
    let result = node.search_knowledge_base("test query").await;
    // Don't assert success since the query might not return results
    // Just verify it doesn't crash
    println!("Search result: {:?}", result);
}
```

## Production Best Practices

### 1. Configuration Management

```yaml
# config/mcp_services.yml
services:
  notion:
    url: ${NOTION_MCP_URL:-http://localhost:8002}
    transport: http
    auth_token: ${NOTION_AUTH_TOKEN}
    timeout_ms: 30000
    retry_attempts: 3
    
  helpscout:
    url: ${HELPSCOUT_MCP_URL:-http://localhost:8001}
    transport: http
    auth_token: ${HELPSCOUT_AUTH_TOKEN}
    timeout_ms: 15000
    retry_attempts: 2
    
  slack:
    url: ${SLACK_MCP_URL:-http://localhost:8003}
    transport: websocket
    auth_token: ${SLACK_BOT_TOKEN}
    timeout_ms: 10000
    retry_attempts: 1
```

### 2. Monitoring and Observability

```rust
use prometheus::{Counter, Histogram, Gauge};

pub struct MCPMetrics {
    requests_total: Counter,
    request_duration: Histogram,
    active_connections: Gauge,
    tool_calls_total: Counter,
}

impl MCPMetrics {
    pub fn record_tool_call(&self, service: &str, tool: &str, duration: Duration, success: bool) {
        self.requests_total.with_label_values(&[service, tool, if success { "success" } else { "error" }]).inc();
        self.request_duration.with_label_values(&[service, tool]).observe(duration.as_secs_f64());
        self.tool_calls_total.with_label_values(&[service, tool]).inc();
    }

    pub fn update_active_connections(&self, service: &str, count: i64) {
        self.active_connections.with_label_values(&[service]).set(count as f64);
    }
}
```

### 3. Security Considerations

```rust
// Always use environment variables for sensitive data
let auth_token = std::env::var("NOTION_AUTH_TOKEN")
    .map_err(|_| WorkflowError::ConfigurationError {
        message: "NOTION_AUTH_TOKEN environment variable not set".to_string()
    })?;

// Implement request signing for enhanced security
pub struct SecureMCPClient {
    client: HttpMCPClient,
    api_key: String,
    secret_key: String,
}

impl SecureMCPClient {
    pub fn sign_request(&self, request: &MCPRequest) -> String {
        // Implement HMAC signing or similar
        // This is a placeholder implementation
        format!("signature_for_request")
    }
}
```

## Key Takeaways

✅ **MCP Standardization**: One interface for all external services

✅ **Transport Flexibility**: HTTP, WebSocket, or stdio based on your needs

✅ **Tool Discovery**: Dynamically find out what services can do

✅ **Error Resilience**: Proper retry logic and connection management

✅ **Testing Strategy**: Both unit and integration testing approaches

✅ **Production Readiness**: Configuration, monitoring, and security

## Next Steps

Now that you understand MCP integration:

1. **Practice**: Build integrations with your own services
2. **Expand**: Try WebSocket transport for real-time updates
3. **Scale**: Implement connection pooling for high-throughput workflows
4. **Monitor**: Add comprehensive observability to your integrations
5. **Continue**: Move to [Tutorial 5: Event Sourcing and State Management](./05-event-sourcing-tutorial.md)

## Quick Reference

```rust
// Basic MCP client setup
let mut client = HttpMCPClient::new("http://localhost:8002".to_string());
client.connect().await?;
client.initialize("my-app", "1.0.0").await?;

// Discover available tools
let tools = client.list_tools().await?;

// Call a tool
let mut args = HashMap::new();
args.insert("query".to_string(), json!("search term"));
let result = client.call_tool("search_pages", Some(args)).await?;

// Handle response
match result.content.first() {
    Some(ToolContent::Text { text }) => {
        let data: serde_json::Value = serde_json::from_str(text)?;
        // Process the data
    },
    _ => {
        // Handle other content types or errors
    }
}

// Always clean up
client.disconnect().await?;
```

You now have the power to integrate any external service into your AI workflows! The MCP framework makes it consistent, reliable, and scalable.