# MCP Integration Example

This example demonstrates comprehensive Model Context Protocol (MCP) integration patterns with the AI Workflow System. It shows how to connect to external MCP servers, handle different transport protocols, and implement robust error handling for external service integration.

## What You'll Learn

- ✅ MCP protocol implementation with multi-transport support
- ✅ Connection pooling and load balancing for MCP clients
- ✅ Error handling for external service integration
- ✅ Workflow nodes that utilize MCP servers
- ✅ Python MCP server communication patterns
- ✅ Timeout handling and circuit breaker patterns
- ✅ Performance monitoring for external integrations

## Architecture Overview

```
Workflow Nodes → MCP Client Pool → Transport Layer → External MCP Servers
     ↓              ↓                    ↓               ↓
[MCP Node]    [Connection Pool]     [HTTP/WebSocket]   [HelpScout]
[HelpScout]   [Load Balancer]      [stdio/TCP]        [Notion] 
[Search]      [Circuit Breaker]    [Retry Logic]      [Slack]
```

This example creates workflows that integrate with:
1. **HelpScout MCP Server** (port 8001) - Customer support ticket management
2. **Notion MCP Server** (port 8002) - Knowledge base integration
3. **Slack MCP Server** (port 8003) - Team communication

## Files Overview

```
02_mcp_integration/
├── README.md              # This file
├── Cargo.toml            # Dependencies with MCP features
├── src/
│   ├── main.rs           # MCP integration demonstrations
│   ├── lib.rs            # MCP client utilities and types
│   └── nodes/            # MCP-enabled workflow nodes
│       ├── mod.rs
│       ├── helpscout_node.rs    # HelpScout integration node
│       ├── notion_node.rs       # Notion integration node
│       ├── slack_node.rs        # Slack integration node
│       └── mcp_search_node.rs   # Multi-source search node
├── examples/
│   ├── mcp_client.rs        # Basic MCP client usage
│   ├── multi_transport.rs   # Different transport protocols
│   ├── connection_pooling.rs # Connection management
│   └── error_resilience.rs  # Error handling patterns
├── python/                 # Python client examples
│   ├── mcp_client.py       # Python MCP client
│   └── requirements.txt
└── tests/
    └── integration_test.rs  # MCP integration tests
```

## Prerequisites

### 1. System Running
```bash
# Start the main system
cd ../../..
cargo run --bin workflow-engine          # Main API (8080)
cargo run --bin graphql-gateway          # Federation Gateway (4000)
```

### 2. MCP Test Servers
```bash
# Start Python MCP servers for testing
./scripts/start_test_servers.sh

# This starts:
# - HelpScout MCP Server on port 8001
# - Notion MCP Server on port 8002  
# - Slack MCP Server on port 8003
```

### 3. Environment Setup
```bash
export JWT_SECRET="your-secure-jwt-secret"
export DATABASE_URL="postgresql://user:pass@localhost/ai_workflow_db"

# Optional: MCP server endpoints (defaults shown)
export HELPSCOUT_MCP_URL="http://localhost:8001"
export NOTION_MCP_URL="http://localhost:8002"
export SLACK_MCP_URL="http://localhost:8003"
```

## Running the Examples

### Basic MCP Client Usage

```bash
# Navigate to this directory
cd examples/02_mcp_integration

# Run the main MCP demonstration
cargo run

# Run specific examples
cargo run --example mcp_client
cargo run --example multi_transport
cargo run --example connection_pooling
cargo run --example error_resilience
```

### Expected Output

```
$ cargo run

=== MCP Integration Example ===
🔗 Initializing MCP client connections...

📋 Available MCP Servers:
✅ HelpScout (localhost:8001) - stdio transport
✅ Notion (localhost:8002) - stdio transport  
✅ Slack (localhost:8003) - stdio transport

🔧 Testing individual server connections...

=== HelpScout MCP Server ===
🔗 Connecting via stdio transport...
✅ Connection established in 245ms

📋 Available tools:
- search_tickets: Search support tickets by query
- create_ticket: Create new support ticket
- update_ticket: Update existing ticket status
- get_ticket_stats: Get ticket statistics

🔧 Testing tool: search_tickets
Parameters: {"query": "login issues", "status": "open"}

📊 Results:
Found 3 tickets matching "login issues":
- Ticket #12345: User cannot access dashboard (Priority: high)
- Ticket #12346: Login form validation error (Priority: medium)
- Ticket #12347: SSO integration failing (Priority: high)
✅ Tool call completed in 156ms

=== Notion MCP Server ===
🔗 Connecting via stdio transport...
✅ Connection established in 198ms

📋 Available tools:
- search_pages: Search Notion pages
- create_page: Create new page
- update_page: Update page content
- get_page_content: Retrieve page content

🔧 Testing tool: search_pages
Parameters: {"query": "API documentation", "limit": 5}

📊 Results:
Found 2 pages matching "API documentation":
- "REST API Guide" (Last modified: 2024-12-15)
- "GraphQL API Reference" (Last modified: 2024-12-17)
✅ Tool call completed in 134ms

=== Multi-Source Search Workflow ===
🚀 Creating workflow with MCP search node...
✅ Workflow created: mcp_multi_search

🔍 Searching across all sources for: "authentication setup"
📊 Query results:
HelpScout: 2 tickets found
Notion: 3 pages found  
Slack: 5 messages found

🤖 Aggregating and ranking results...
Top results:
1. "JWT Authentication Guide" (Notion) - Relevance: 0.95
2. "Auth troubleshooting ticket" (HelpScout) - Relevance: 0.87
3. "Auth implementation discussion" (Slack) - Relevance: 0.82

✅ MCP integration demonstration completed!
```

## Key Concepts Demonstrated

### 1. MCP Client Configuration

```rust
use workflow_engine_mcp::config::McpClientConfig;
use workflow_engine_mcp::clients::McpClient;
use workflow_engine_core::error::WorkflowError;

// Configure MCP client with connection pooling
let mcp_config = McpClientConfig::builder()
    .server_name("helpscout")
    .transport_type("stdio")
    .command("python")
    .args(vec![
        "./scripts/customer_support_server.py".to_string(),
        "--port".to_string(),
        "8001".to_string()
    ])
    .max_connections(5)
    .connection_timeout_ms(5000)
    .request_timeout_ms(30000)
    .retry_attempts(3)
    .build()
    .map_err(|e| WorkflowError::mcp_connection_error(
        format!("Failed to build MCP config: {}", e),
        "helpscout",
        "stdio",
        "config_builder"
    ))?;

// Create client with error handling
let mut mcp_client = McpClient::new(mcp_config)
    .await
    .map_err(|e| WorkflowError::mcp_connection_error(
        format!("Failed to create MCP client: {}", e),
        "helpscout",
        "stdio",
        "client_creation"
    ))?;
```

### 2. MCP Tool Invocation with Error Handling

```rust
use serde_json::json;

// Call MCP tool with proper error handling
async fn call_mcp_tool(
    client: &mut McpClient,
    tool_name: &str,
    parameters: serde_json::Value,
) -> Result<serde_json::Value, WorkflowError> {
    client.call_tool(tool_name, Some(parameters))
        .await
        .map_err(|e| {
            // Categorize MCP errors for proper handling
            let error_msg = e.to_string();
            if error_msg.contains("connection") || error_msg.contains("timeout") {
                WorkflowError::mcp_connection_error(
                    format!("MCP connection failed: {}", e),
                    client.server_name(),
                    client.transport_type(),
                    tool_name
                )
            } else if error_msg.contains("protocol") || error_msg.contains("invalid") {
                WorkflowError::mcp_protocol_error(
                    format!("MCP protocol error: {}", e),
                    client.server_name(),
                    "valid_tool_call",
                    error_msg,
                    "tool_invocation"
                )
            } else {
                WorkflowError::mcp_error(
                    format!("MCP tool call failed: {}", e),
                    client.server_name(),
                    tool_name
                )
            }
        })
}

// Example usage
let search_params = json!({
    "query": "login issues",
    "status": "open",
    "limit": 10
});

let results = call_mcp_tool(&mut helpscout_client, "search_tickets", search_params).await?;
```

### 3. MCP-Enabled Workflow Node

```rust
use workflow_engine_core::nodes::Node;
use workflow_engine_core::task::TaskContext;
use workflow_engine_core::error::WorkflowError;
use workflow_engine_mcp::clients::McpClient;

#[derive(Debug)]
pub struct HelpScoutSearchNode {
    client: McpClient,
    config: SearchConfig,
}

impl Node for HelpScoutSearchNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Extract search query from context
        let query: String = context.get_data("search_query")
            .map_err(|e| WorkflowError::validation_error(
                "Missing search query",
                "search_query",
                "non-empty string",
                "in HelpScout search node"
            ))?;

        // Prepare MCP tool parameters
        let search_params = json!({
            "query": query,
            "status": self.config.ticket_status,
            "limit": self.config.max_results,
            "priority": self.config.priority_filter
        });

        // Call MCP tool with timeout and retry
        let search_results = tokio::time::timeout(
            std::time::Duration::from_millis(self.config.timeout_ms),
            call_mcp_tool(&mut self.client, "search_tickets", search_params)
        )
        .await
        .map_err(|_| WorkflowError::mcp_connection_error(
            "MCP call timeout",
            "helpscout",
            "stdio",
            "search_tickets"
        ))??;

        // Process and store results
        let processed_results = self.process_search_results(search_results)?;
        context.update_node("helpscout_results", processed_results);

        Ok(context)
    }
}
```

### 4. Connection Pooling and Load Balancing

```rust
use workflow_engine_mcp::connection_pool::McpConnectionPool;
use workflow_engine_mcp::load_balancer::McpLoadBalancer;

// Create connection pool for multiple MCP servers
let mut pool = McpConnectionPool::new();

// Add HelpScout servers with load balancing
pool.add_server_config(McpClientConfig::builder()
    .server_name("helpscout_primary")
    .transport_type("stdio")
    .command("python")
    .args(vec!["./scripts/customer_support_server.py".to_string()])
    .build()?
).await?;

pool.add_server_config(McpClientConfig::builder()
    .server_name("helpscout_backup")
    .transport_type("http")
    .endpoint("http://backup.helpscout.local:8001")
    .build()?
).await?;

// Configure load balancer
let load_balancer = McpLoadBalancer::new()
    .with_strategy(LoadBalancingStrategy::RoundRobin)
    .with_health_checks(true)
    .with_failover(true);

// Use pooled connection with automatic failover
let client = pool.get_client("helpscout")
    .await
    .map_err(|e| WorkflowError::mcp_connection_error(
        format!("Failed to get pooled client: {}", e),
        "helpscout",
        "pooled",
        "connection_acquisition"
    ))?;
```

### 5. Circuit Breaker Pattern

```rust
use workflow_engine_core::error::{WorkflowError, ErrorCategory};

pub struct McpCircuitBreaker {
    failure_count: u32,
    failure_threshold: u32,
    reset_timeout: std::time::Duration,
    last_failure: Option<std::time::Instant>,
    state: CircuitBreakerState,
}

#[derive(Debug, Clone)]
enum CircuitBreakerState {
    Closed,    // Normal operation
    Open,      // Failing fast
    HalfOpen,  // Testing recovery
}

impl McpCircuitBreaker {
    pub async fn call_with_circuit_breaker<F, T>(
        &mut self,
        operation: F,
    ) -> Result<T, WorkflowError>
    where
        F: Future<Output = Result<T, WorkflowError>>,
    {
        match self.state {
            CircuitBreakerState::Open => {
                if self.should_attempt_reset() {
                    self.state = CircuitBreakerState::HalfOpen;
                } else {
                    return Err(WorkflowError::mcp_connection_error(
                        "Circuit breaker is open",
                        "unknown",
                        "circuit_breaker",
                        "circuit_protection"
                    ));
                }
            }
            _ => {}
        }

        match operation.await {
            Ok(result) => {
                self.on_success();
                Ok(result)
            }
            Err(e) => {
                // Only count certain errors as failures
                if matches!(e.category(), ErrorCategory::Transient) {
                    self.on_failure();
                }
                Err(e)
            }
        }
    }

    fn on_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitBreakerState::Closed;
    }

    fn on_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure = Some(std::time::Instant::now());
        
        if self.failure_count >= self.failure_threshold {
            self.state = CircuitBreakerState::Open;
        }
    }
}
```

## Advanced Features

### Multi-Transport Support

The example demonstrates connecting to MCP servers using different transport protocols:

```rust
// stdio transport (for local Python servers)
let stdio_config = McpClientConfig::builder()
    .transport_type("stdio")
    .command("python")
    .args(vec!["./mcp_server.py".to_string()])
    .build()?;

// HTTP transport (for remote services)
let http_config = McpClientConfig::builder()
    .transport_type("http")
    .endpoint("https://api.external-service.com/mcp")
    .headers(vec![("Authorization".to_string(), "Bearer token".to_string())])
    .build()?;

// WebSocket transport (for real-time services)
let ws_config = McpClientConfig::builder()
    .transport_type("websocket")
    .endpoint("wss://realtime.service.com/mcp")
    .build()?;
```

### Performance Monitoring

```rust
use std::time::Instant;
use tracing::{info, warn};

pub struct McpPerformanceMonitor {
    request_count: u64,
    total_response_time: std::time::Duration,
    error_count: u64,
}

impl McpPerformanceMonitor {
    pub async fn monitor_mcp_call<F, T>(
        &mut self,
        server_name: &str,
        tool_name: &str,
        operation: F,
    ) -> Result<T, WorkflowError>
    where
        F: Future<Output = Result<T, WorkflowError>>,
    {
        let start_time = Instant::now();
        self.request_count += 1;

        match operation.await {
            Ok(result) => {
                let response_time = start_time.elapsed();
                self.total_response_time += response_time;
                
                info!(
                    "MCP call successful: server={}, tool={}, duration={}ms",
                    server_name,
                    tool_name,
                    response_time.as_millis()
                );

                // Alert on slow responses
                if response_time > std::time::Duration::from_secs(5) {
                    warn!(
                        "Slow MCP response: server={}, tool={}, duration={}ms",
                        server_name,
                        tool_name,
                        response_time.as_millis()
                    );
                }

                Ok(result)
            }
            Err(e) => {
                self.error_count += 1;
                warn!(
                    "MCP call failed: server={}, tool={}, error={}",
                    server_name,
                    tool_name,
                    e
                );
                Err(e)
            }
        }
    }

    pub fn get_metrics(&self) -> McpMetrics {
        McpMetrics {
            request_count: self.request_count,
            error_count: self.error_count,
            error_rate: if self.request_count > 0 {
                self.error_count as f64 / self.request_count as f64
            } else {
                0.0
            },
            average_response_time: if self.request_count > 0 {
                self.total_response_time / self.request_count as u32
            } else {
                std::time::Duration::default()
            },
        }
    }
}
```

## Testing

### Unit Tests
```bash
# Test MCP client functionality
cargo test mcp_client

# Test connection pooling
cargo test connection_pool

# Test error handling
cargo test mcp_error_handling
```

### Integration Tests
```bash
# Start MCP test servers first
./scripts/start_test_servers.sh

# Run integration tests
cargo test --test integration_test -- --ignored

# Test individual MCP servers
cargo test helpscout_integration -- --ignored
cargo test notion_integration -- --ignored
cargo test slack_integration -- --ignored
```

### Python Examples
```bash
# Install Python dependencies
cd python
pip install -r requirements.txt

# Run Python MCP client example
python mcp_client.py
```

## Troubleshooting

### Common MCP Integration Issues

1. **Connection Failures**
   ```
   ❌ MCP connection error: Failed to start stdio transport
   ```
   **Solution**: Ensure Python MCP servers are running and accessible
   ```bash
   ./scripts/start_test_servers.sh
   ps aux | grep customer_support_server.py
   ```

2. **Protocol Errors**
   ```
   ❌ MCP protocol error: Invalid tool name 'unknown_tool'
   ```
   **Solution**: Check available tools using introspection
   ```rust
   let tools = client.list_tools().await?;
   println!("Available tools: {:?}", tools);
   ```

3. **Timeout Issues**
   ```
   ❌ MCP call timeout after 5000ms
   ```
   **Solution**: Increase timeout or check server performance
   ```rust
   let config = McpClientConfig::builder()
       .request_timeout_ms(10000)  // Increase to 10 seconds
       .build()?;
   ```

4. **Transport Issues**
   ```
   ❌ stdio transport failed: Process exited with code 1
   ```
   **Solution**: Check MCP server logs and dependencies
   ```bash
   python ./scripts/customer_support_server.py --debug
   ```

### Debug Commands

```bash
# Check MCP server status
curl http://localhost:8001/health
curl http://localhost:8002/health
curl http://localhost:8003/health

# Test MCP servers manually
python ./scripts/test_mcp_server.py --server helpscout --port 8001

# View MCP client logs
RUST_LOG=debug cargo run --example mcp_client

# Monitor connection pool
RUST_LOG=workflow_engine_mcp::connection_pool=debug cargo run
```

## Next Steps

1. **Try Advanced Examples**: Run `cargo run --example connection_pooling` to see connection management
2. **Custom MCP Integration**: Build your own MCP-enabled workflow nodes
3. **Performance Optimization**: Implement connection pooling and circuit breakers
4. **Multi-Source Workflows**: Combine data from multiple MCP servers
5. **Move to Federation**: Continue with [GraphQL Federation example](../03_graphql_federation/) to see cross-service integration

## Further Reading

- **[MCP Protocol Specification](../../crates/workflow-engine-mcp/README.md)**
- **[Connection Pooling Guide](../../docs/MCP_POOLING.md)**
- **[Circuit Breaker Patterns](../../docs/RESILIENCE.md)**
- **[Python MCP Server Development](../../mcp-servers/README.md)**