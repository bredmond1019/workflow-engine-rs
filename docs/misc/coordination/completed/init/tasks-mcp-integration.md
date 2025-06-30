## Relevant Files

- `src/core/mcp/mod.rs` - Main MCP module definition and exports
- `src/core/mcp/client.rs` - MCP client implementation for connecting to MCP servers
- `src/core/mcp/server.rs` - MCP server wrapper to expose nodes as MCP tools
- `src/core/mcp/transport.rs` - Transport layer abstractions (stdio, websocket, http)
- `src/core/mcp/protocol.rs` - MCP protocol message definitions and serialization
- `src/core/nodes/agent.rs` - Modified to include MCP client support
- `src/core/agents/anthropic.rs` - Updated to use MCP client when available
- `src/core/agents/openai.rs` - Updated to use MCP client when available
- `src/core/workflow/mod.rs` - Extended to support MCP server registration
- `tests/mcp_integration_test.rs` - Integration tests for MCP functionality

### Notes

- Unit tests should typically be placed alongside the code files they are testing (e.g., `client.rs` and `client_test.rs` in the same directory).
- Use `cargo test` to run all tests or `cargo test --test mcp_integration_test` for specific integration tests.

## Tasks

- [x] 1.0 Set up MCP module structure and core protocol definitions
  - [x] 1.1 Create the `src/core/mcp/mod.rs` file with module exports
  - [x] 1.2 Define MCP protocol messages in `src/core/mcp/protocol.rs` (initialize, list_tools, call_tool, etc.)
  - [x] 1.3 Create transport abstraction in `src/core/mcp/transport.rs` with enum for Stdio, WebSocket, and HTTP
  - [x] 1.4 Add MCP-specific error types to `src/core/error.rs`
  - [x] 1.5 Update `Cargo.toml` with necessary dependencies (serde_json, tokio-tungstenite for WebSocket, etc.)
  
- [x] 2.0 Implement MCP client functionality for AgentNode
  - [x] 2.1 Create `MCPClient` trait in `src/core/mcp/client.rs` with connect, list_tools, and call_tool methods
  - [x] 2.2 Implement `StdioMCPClient` for stdio-based MCP communication
  - [x] 2.3 Implement `WebSocketMCPClient` for WebSocket-based MCP communication
  - [x] 2.4 Add `MCPConnection` struct to manage active MCP sessions
  - [x] 2.5 Modify `BaseAgentNode` in `src/core/nodes/agent.rs` to include optional MCP client field
  - [x] 2.6 Create unit tests for MCP client implementations
  
- [x] 3.0 Create MCP server wrapper to expose existing nodes as tools
  - [x] 3.1 Define `MCPToolServer` struct in `src/core/mcp/server.rs`
  - [x] 3.2 Implement `register_node_as_tool` method to convert Node interface to MCP tool
  - [x] 3.3 Create tool description generator that extracts metadata from nodes
  - [x] 3.4 Implement MCP server message handling (respond to list_tools, call_tool requests)
  - [x] 3.5 Add `expose_as_mcp_server` method to Workflow
  - [x] 3.6 Create example MCP server that exposes customer support tools
  
- [x] 4.0 Integrate MCP support into existing agent implementations
  - [x] 4.1 Add `process_with_mcp` method to `AnthropicAgentNode` in `src/core/agents/anthropic.rs`
  - [x] 4.2 Add `process_with_mcp` method to `OpenAIAgentNode` in `src/core/agents/openai.rs`
  - [x] 4.3 Modify existing `process` methods to check for MCP client and fallback to direct API
  - [x] 4.4 Update `AgentConfig` to include MCP server URI configuration
  - [x] 4.5 Add MCP connection pooling and retry logic
  - [x] 4.6 Create integration tests for agent MCP interactions
  
- [x] 5.0 Add MCP configuration and testing infrastructure
  - [x] 5.1 Add MCP configuration options to environment variables (.env file)
  - [x] 5.2 Create `register_mcp_server` method in `Workflow` for connecting to external MCP servers
  - [x] 5.3 Write integration tests in `tests/mcp_integration_test.rs`
  - [x] 5.4 Create example MCP server script for testing
  - [x] 5.5 Update demo workflow to showcase MCP integration
  - [x] 5.6 Add documentation for MCP setup and usage