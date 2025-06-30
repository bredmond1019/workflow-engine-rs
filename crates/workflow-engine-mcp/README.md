# workflow-engine-mcp

Production-ready Model Context Protocol (MCP) implementation for connecting to external AI tools and services.

## Features

- **Complete MCP Implementation**: Full support for the Model Context Protocol specification with all message types
- **Multiple Transport Types**: HTTP, WebSocket, and stdio transports for maximum compatibility
- **Advanced Connection Management**: Pooling, health monitoring, automatic reconnection, and load balancing
- **Built-in Server Framework**: Expose workflow nodes as MCP tools with automatic metadata generation
- **High-Level Client Libraries**: Type-safe client implementations for seamless external service integration
- **Production Features**: Circuit breakers, retry policies, metrics, and comprehensive error handling
- **Performance Optimized**: Connection pooling, message batching, and efficient serialization

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
workflow-engine-mcp = "0.6.0"
workflow-engine-core = "0.6.0"
tokio = { version = "1.0", features = ["full"] }
serde_json = "1.0"
```

### Basic HTTP Client

```rust
use workflow_engine_mcp::prelude::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create HTTP MCP client
    let mut client = HttpMcpClient::new("http://localhost:8002/mcp")?;
    
    // Initialize connection
    client.initialize().await?;
    
    // Discover available tools
    let tools = client.list_tools().await?;
    println!("Available tools: {:?}", tools);
    
    // Call a tool with parameters
    let result = client.call_tool("search_knowledge_base", json!({
        "query": "How to setup authentication",
        "limit": 5
    })).await?;
    
    println!("Search results: {:?}", result);
    Ok(())
}
```

### WebSocket Client with Auto-Reconnection

```rust
use workflow_engine_mcp::clients::websocket::WebSocketMcpClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create WebSocket client with auto-reconnection
    let mut client = WebSocketMcpClient::builder()
        .endpoint("ws://localhost:8003/mcp")
        .auto_reconnect(true)
        .reconnect_interval(Duration::from_secs(5))
        .heartbeat_interval(Duration::from_secs(30))
        .build()
        .await?;
    
    // Send real-time notification
    let response = client.call_tool("send_slack_message", json!({
        "channel": "#alerts",
        "message": "Workflow completed successfully!",
        "priority": "normal"
    })).await?;
    
    println!("Message sent: {:?}", response);
    Ok(())
}
```

### Stdio Client for Python MCP Servers

```rust
use workflow_engine_mcp::clients::stdio::StdioMcpClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Launch Python MCP server as subprocess
    let mut client = StdioMcpClient::builder()
        .command("python")
        .args(&["-m", "notion_mcp_server"])
        .working_directory("./mcp-servers/")
        .environment("NOTION_API_TOKEN", "your-notion-token")
        .auto_restart(true)
        .build()
        .await?;
    
    // Use Notion integration
    let pages = client.call_tool("search_pages", json!({
        "query": "project documentation",
        "filter": {"object": "page"}
    })).await?;
    
    println!("Found pages: {:?}", pages);
    Ok(())
}
```

## Feature Flags

- `http` - HTTP transport for MCP clients (default)
- `websocket` - WebSocket transport for MCP clients (default)
- `stdio` - Standard I/O transport for MCP clients
- `all` - All transport types

## Transports

### HTTP Transport
```rust
let client = HttpMcpClient::new("http://localhost:8080/mcp")?;
```

### WebSocket Transport
```rust
let client = WebSocketMcpClient::new("ws://localhost:8080/mcp").await?;
```

### Stdio Transport
```rust
let client = StdioMcpClient::new("/path/to/mcp/server", &["--arg1", "--arg2"]).await?;
```

## Documentation

For comprehensive documentation, visit [docs.rs/workflow-engine-mcp](https://docs.rs/workflow-engine-mcp).

## License

Licensed under the MIT License. See [LICENSE](../../LICENSE) for details.