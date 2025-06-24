# CLAUDE.md - workflow-engine-mcp

This file provides guidance to Claude Code when working with the Model Context Protocol (MCP) implementation in this crate.

## Crate Overview

The `workflow-engine-mcp` crate is the core implementation of the Model Context Protocol (MCP) framework for the workflow engine. It provides a complete, production-ready MCP implementation with multi-transport support, connection pooling, and built-in server implementations.

### Purpose and Role

- **Protocol Implementation**: Complete MCP specification compliance with all message types
- **Transport Abstraction**: Unified interface for HTTP, WebSocket, and stdio transports
- **Client Framework**: High-level MCP client implementations for external services
- **Server Framework**: Tools for building MCP servers from workflow nodes
- **Connection Management**: Advanced pooling, health monitoring, and load balancing
- **Production Features**: Circuit breakers, retries, metrics, and error recovery

## Architecture

### Core Components

1. **Protocol Layer** (`src/protocol.rs`)
   - MCP message types: Request, Response, Tool definitions
   - Serialization/deserialization with proper error handling
   - Protocol version negotiation support

2. **Transport Layer** (`src/transport.rs`)
   - Abstract `McpTransport` trait for all transport types
   - HTTP transport for request/response patterns
   - WebSocket transport for persistent connections
   - Stdio transport for process-based communication
   - Transport health monitoring and metrics

3. **Client Implementations** (`src/clients/`)
   - `McpClient` trait for unified client interface
   - `HttpMcpClient` for HTTP-based MCP servers
   - `WebSocketMcpClient` for WebSocket connections
   - `StdioMcpClient` for subprocess communication
   - Connection management and initialization

4. **Connection Pool** (`src/connection_pool.rs`)
   - Advanced connection pooling with per-server limits
   - Health-based connection selection
   - Automatic reconnection and recovery
   - Circuit breaker integration
   - Load balancing strategies

5. **Server Framework** (`src/server/`)
   - `McpToolServer` for exposing nodes as MCP tools
   - Built-in implementations for customer support and knowledge base
   - Automatic tool metadata generation

## MCP Protocol Implementation

### Message Types

```rust
// Request types
McpRequest::Initialize { id, params }      // Protocol handshake
McpRequest::ListTools { id }               // Tool discovery
McpRequest::CallTool { id, params }        // Tool execution
McpRequest::Initialized                    // Notification

// Response types
McpResponse::Result { id, result }         // Success response
McpResponse::Error { id, error }           // Error response
```

### Protocol Flow

1. **Connection**: Transport establishes connection
2. **Initialize**: Client sends initialization request
3. **Capabilities**: Server responds with capabilities
4. **Tool Discovery**: Client lists available tools
5. **Tool Execution**: Client calls tools with arguments
6. **Cleanup**: Graceful disconnection

## Transport Mechanisms

### HTTP Transport

- Stateless request/response model
- Built on `reqwest` with connection pooling
- Authentication via Bearer tokens
- Suitable for REST-style MCP servers

### WebSocket Transport

- Persistent bidirectional connections
- Automatic reconnection with exponential backoff
- Heartbeat/ping support for keep-alive
- Message framing and buffering

### Stdio Transport

- Process-based communication via stdin/stdout
- Automatic process restart on failure
- Line-based JSON message framing
- Ideal for Python MCP servers

## Connection Pooling and Load Balancing

### Pool Features

- **Per-server connection limits**: Configurable max connections
- **Health monitoring**: Continuous health checks
- **Idle timeout**: Automatic cleanup of unused connections
- **Circuit breakers**: Failure protection per server
- **Metrics collection**: Performance and health metrics

### Load Balancing Strategies

1. **Round-robin**: Equal distribution
2. **Random**: Random selection
3. **Least connections**: Prefer less loaded connections
4. **Health-based**: Weighted by connection health

### Advanced Features

- Client affinity for session persistence
- Server priorities for failover scenarios
- Automatic weight adjustment based on performance
- Detailed health reporting and diagnostics

## Testing Approach

### Unit Tests

Each module includes comprehensive unit tests:

```bash
# Run unit tests for the MCP crate
cargo test -p workflow-engine-mcp

# Run with verbose output
cargo test -p workflow-engine-mcp -- --nocapture
```

### Integration Tests

MCP integration tests require external servers:

```bash
# Start Python MCP test servers
./scripts/start_test_servers.sh

# Run MCP integration tests
cargo test mcp_integration -- --ignored
cargo test mcp_communication -- --ignored
cargo test mcp_connection -- --ignored
```

### Test Categories

1. **Protocol tests**: Message serialization, version negotiation
2. **Transport tests**: Connection lifecycle, error handling
3. **Pool tests**: Connection management, health checks
4. **Client tests**: End-to-end client functionality
5. **Server tests**: Tool registration and execution

## Common Development Tasks

### Adding a New Transport Type

1. Implement the `McpTransport` trait in `src/transport.rs`
2. Add transport-specific client in `src/clients/`
3. Update `TransportType` enum
4. Add transport tests
5. Update connection pool to support new transport

### Creating an MCP Client

```rust
use workflow_engine_mcp::prelude::*;

// HTTP client
let client = HttpMcpClient::new("http://localhost:8080/mcp")?;

// WebSocket client  
let ws_client = WebSocketMcpClient::new("ws://localhost:8080/mcp").await?;

// Stdio client
let stdio_client = StdioMcpClient::new("python", &["-m", "mcp_server"]).await?;
```

### Building an MCP Server

```rust
use workflow_engine_mcp::server::McpToolServer;

// Create server
let server = McpToolServer::new("my-server".to_string(), "1.0.0".to_string());

// Register workflow nodes as tools
let node = Arc::new(MyNode::new());
server.register_node_with_auto_metadata(node).await?;

// Handle requests
let response = server.handle_request(request).await?;
```

### Configuring Connection Pool

```rust
use workflow_engine_mcp::connection_pool::{McpConnectionPool, ConnectionConfig};

let config = ConnectionConfig {
    max_connections_per_server: 10,
    connection_timeout: Duration::from_secs(30),
    idle_timeout: Duration::from_secs(300),
    enable_load_balancing: true,
    load_balancing_strategy: LoadBalancingStrategy::HealthBased,
    ..Default::default()
};

let pool = McpConnectionPool::new(config);
```

### Implementing Custom Health Checks

```rust
use workflow_engine_mcp::health::{HealthChecker, HealthStatus};

impl HealthChecker for MyHealthChecker {
    async fn check_health(&self, client: &mut dyn McpClient) -> Result<HealthStatus, WorkflowError> {
        // Custom health check logic
        match client.list_tools().await {
            Ok(tools) if !tools.is_empty() => Ok(HealthStatus::Healthy),
            Ok(_) => Ok(HealthStatus::Degraded),
            Err(_) => Ok(HealthStatus::Unhealthy),
        }
    }
}
```

## Error Handling

### Transport Errors

All transport errors implement rich error context:

```rust
TransportError::IoError { message, operation, source }
TransportError::WebSocketError { message, endpoint, operation, source }
TransportError::ConnectionError { message, endpoint, transport_type, retry_count }
TransportError::ProtocolError { message, operation, expected, received }
```

### Error Recovery

- Automatic reconnection for transient failures
- Circuit breakers for cascading failure prevention
- Exponential backoff with jitter
- Health-based connection selection

## Performance Considerations

### Connection Pool Tuning

- Adjust `max_connections_per_server` based on load
- Configure appropriate `idle_timeout` for connection reuse
- Enable connection metrics for monitoring
- Use health-based load balancing for reliability

### Transport Selection

- **HTTP**: Best for occasional requests, simple integration
- **WebSocket**: Best for high-frequency communication, real-time updates
- **Stdio**: Best for local processes, Python MCP servers

### Monitoring and Metrics

- Connection pool statistics via `get_pool_stats()`
- Circuit breaker metrics via `get_circuit_breaker_metrics()`
- Transport-level metrics for latency and throughput
- Health monitoring dashboard integration

## Debugging Tips

### Connection Issues

```rust
// Check pool health
let health = pool.get_detailed_health().await;
println!("{:#?}", health);

// Force reconnection
pool.force_reconnect("problematic-server").await?;

// Get connection metrics
let stats = pool.get_pool_stats().await;
```

### Protocol Debugging

```rust
// Enable debug logging
env_logger::Builder::from_env(Env::default().default_filter_or("workflow_engine_mcp=debug")).init();

// Inspect raw messages
let transport = HttpTransport::new(url);
// Messages are logged at debug level
```

### Common Issues

1. **Connection timeouts**: Increase `connection_timeout` in config
2. **Idle disconnections**: Adjust `idle_timeout` or enable heartbeats
3. **Circuit breaker trips**: Check server health, adjust thresholds
4. **Protocol mismatches**: Verify server supports MCP version

## Feature Flags

```toml
[dependencies]
workflow-engine-mcp = { version = "0.6.0", features = ["all"] }

# Individual features
features = ["http", "websocket", "stdio"]
```

- `http`: HTTP transport support (default)
- `websocket`: WebSocket transport support (default)
- `stdio`: Standard I/O transport support
- `all`: All transport types

## Security Considerations

1. **Authentication**: Use Bearer tokens for HTTP, connection auth for WebSocket
2. **TLS**: Always use HTTPS/WSS in production
3. **Process isolation**: Stdio processes run with limited permissions
4. **Input validation**: All MCP messages are validated against schema
5. **Rate limiting**: Implement at transport or pool level

## Integration with Workflow Engine

The MCP crate integrates seamlessly with the workflow engine:

1. **Node execution**: MCP tools can be workflow nodes
2. **External services**: Connect to Python MCP servers
3. **Service discovery**: Automatic tool discovery
4. **Error propagation**: MCP errors convert to WorkflowError
5. **Context passing**: TaskContext maps to MCP arguments

## Best Practices

1. **Always use connection pooling** for production deployments
2. **Implement health checks** for critical MCP servers
3. **Configure circuit breakers** to prevent cascading failures
4. **Monitor connection metrics** for performance insights
5. **Use appropriate transports** based on communication patterns
6. **Handle disconnections gracefully** with retry logic
7. **Validate tool schemas** before registration
8. **Log protocol interactions** for debugging
9. **Test with external servers** using integration tests
10. **Document tool capabilities** in metadata