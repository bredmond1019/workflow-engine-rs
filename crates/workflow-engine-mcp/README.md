# workflow-engine-mcp

Model Context Protocol (MCP) integration for the workflow engine.

## Features

- **Complete MCP Implementation**: Full support for the Model Context Protocol specification
- **Multiple Transports**: HTTP, WebSocket, and stdio transport types
- **Connection Pooling**: Managed connections with health monitoring and load balancing  
- **Built-in Servers**: Ready-to-use MCP server implementations
- **Client Libraries**: High-level client implementations for external MCP servers

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