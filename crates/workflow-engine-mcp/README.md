# workflow-engine-mcp

Model Context Protocol (MCP) integration for the workflow engine with production-ready features.

[![Crates.io](https://img.shields.io/crates/v/workflow-engine-mcp.svg)](https://crates.io/crates/workflow-engine-mcp)
[![Documentation](https://docs.rs/workflow-engine-mcp/badge.svg)](https://docs.rs/workflow-engine-mcp)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

## Features

- **Complete MCP Implementation**: Full support for the Model Context Protocol specification
- **Multiple Transports**: HTTP, WebSocket, and stdio transport types with security
- **Advanced Connection Pooling**: Health monitoring, circuit breakers, and load balancing  
- **Built-in Servers**: Production-ready MCP server implementations
- **Client Libraries**: High-level client implementations for external MCP servers
- **Security First**: TDD-driven validation, TLS support, and parameter sanitization
- **Error Recovery**: Automatic reconnection, exponential backoff, and circuit breakers

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
workflow-engine-mcp = "0.6.0"
tokio = { version = "1.0", features = ["full"] }
```

Create an MCP client:

```rust
use workflow_engine_mcp::prelude::*;

#[tokio::main]
async fn main() -> Result<(), McpClientError> {
    // HTTP transport
    let client = HttpMcpClient::new("http://localhost:8080/mcp")?;
    let tools = client.list_tools().await?;
    println!("Available tools: {:?}", tools);
    
    // WebSocket transport
    let ws_client = WebSocketMcpClient::new("ws://localhost:8080/mcp").await?;
    let result = ws_client.call_tool("analyze_data", json!({"data": "sample"})).await?;
    
    Ok(())
}
```

## Feature Flags

- `http` - HTTP transport for MCP clients (default)
- `websocket` - WebSocket transport for MCP clients (default)
- `stdio` - Standard I/O transport for MCP clients
- `all` - All transport types

## Advanced Usage

### Connection Pooling

```rust
use workflow_engine_mcp::{
    connection_pool::{McpConnectionPool, ConnectionConfig},
    load_balancer::LoadBalancingStrategy,
};

let config = ConnectionConfig {
    max_connections_per_server: 10,
    connection_timeout: Duration::from_secs(30),
    idle_timeout: Duration::from_secs(300),
    enable_load_balancing: true,
    load_balancing_strategy: LoadBalancingStrategy::HealthBased,
    enable_circuit_breaker: true,
    circuit_breaker_threshold: 5,
    circuit_breaker_timeout: Duration::from_secs(60),
    ..Default::default()
};

let pool = McpConnectionPool::new(config);

// Add servers to the pool
pool.add_server("helpscout", "http://localhost:8001", TransportType::Http).await?;
pool.add_server("notion", "ws://localhost:8002", TransportType::WebSocket).await?;

// Get a healthy connection
let mut conn = pool.get_connection("helpscout").await?;
let tools = conn.list_tools().await?;
```

### Creating an MCP Server

```rust
use workflow_engine_mcp::server::{McpToolServer, McpTool};
use workflow_engine_core::nodes::Node;

// Convert a workflow node to an MCP tool
#[derive(Debug)]
struct AnalyzeDataNode;

impl Node for AnalyzeDataNode {
    fn process(&self, context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Node implementation
        Ok(context)
    }
}

// Create MCP server
let server = McpToolServer::new("analytics-server", "1.0.0");

// Register the node as a tool
let node = Arc::new(AnalyzeDataNode);
server.register_node_with_auto_metadata("analyze_data", node).await?;

// Handle MCP requests
let response = server.handle_request(mcp_request).await?;
```

### Error Handling and Recovery

```rust
use workflow_engine_mcp::prelude::*;
use std::time::Duration;

// Configure retry policy
let retry_config = RetryConfig {
    max_attempts: 3,
    initial_delay: Duration::from_millis(100),
    max_delay: Duration::from_secs(10),
    exponential_base: 2.0,
    jitter: true,
};

// Client with automatic retry
let client = HttpMcpClient::builder()
    .url("http://localhost:8080/mcp")
    .retry_config(retry_config)
    .timeout(Duration::from_secs(30))
    .build()?;

// Calls will automatically retry on transient failures
match client.call_tool("process_data", params).await {
    Ok(result) => println!("Success: {:?}", result),
    Err(WorkflowError::MCPConnectionError(details)) => {
        eprintln!("Connection failed after {} retries: {}", 
            details.retry_count, details.message);
    }
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Transports

### HTTP Transport
```rust
let client = HttpMcpClient::new("http://localhost:8080/mcp")?;
// Supports: Bearer auth, custom headers, connection pooling
```

### WebSocket Transport
```rust
let client = WebSocketMcpClient::new("ws://localhost:8080/mcp").await?;
// Supports: Auto-reconnect, heartbeat, message buffering
```

### Stdio Transport
```rust
let client = StdioMcpClient::new("/path/to/mcp/server", &["--arg1", "--arg2"]).await?;
// Supports: Process management, restart on failure, line-based JSON
```

## Testing

```bash
# Unit tests
cargo test -p workflow-engine-mcp

# Integration tests (requires MCP servers)
./scripts/start_test_servers.sh
cargo test mcp_ -- --ignored --test-threads=1

# Specific transport tests
cargo test -p workflow-engine-mcp http_transport
cargo test -p workflow-engine-mcp websocket_transport
cargo test -p workflow-engine-mcp stdio_transport
```

## Documentation

For comprehensive documentation, visit [docs.rs/workflow-engine-mcp](https://docs.rs/workflow-engine-mcp).

## Examples

See the [examples directory](../../examples/) for:
- Basic MCP client usage
- Connection pool configuration
- Custom MCP server implementation
- Error handling patterns
- Transport-specific examples

## Dependencies

This crate depends on `workflow-engine-core` for base types and error handling.

Key dependencies:
- `tokio` - Async runtime
- `reqwest` - HTTP client
- `tokio-tungstenite` - WebSocket client  
- `serde` - Serialization
- `async-trait` - Async traits

## Contributing

Contributions are welcome! Please read our [Contributing Guide](../../CONTRIBUTING.md) for details.

## License

Licensed under the MIT License. See [LICENSE](../../LICENSE) for details.