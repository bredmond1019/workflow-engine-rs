# MCP Integration - Connecting to External Tools and Services

Welcome to MCP (Model Context Protocol) integration examples! This section demonstrates how to connect your workflows to external tools and services, expanding the capabilities of your AI-powered systems.

## 🎯 Learning Objectives

By completing these examples, you will:
- Understand the Model Context Protocol (MCP)
- Connect to external MCP servers
- Use external tools in workflows
- Build multi-source knowledge integration
- Handle MCP server failures and retries

## 📚 Examples in This Section

### 1. basic-mcp-client
**File**: `basic-mcp-client.rs`
**Concepts**: MCP protocol basics, client connections, tool discovery
**Time**: 25 minutes

Learn MCP fundamentals:
- Connect to MCP servers
- Discover available tools
- Make basic tool calls
- Handle MCP responses

```bash
cargo run --bin basic-mcp-client
```

### 2. external-tools
**File**: `external-tools.rs`
**Concepts**: Tool integration, external services, data enrichment
**Time**: 30 minutes

Integrate external tools:
- Use HelpScout for customer data
- Query Notion for knowledge base
- Integrate Slack for team information
- Combine multiple tool responses

```bash
cargo run --bin external-tools
```

### 3. multi-source-search
**File**: `multi-source-search.rs`
**Concepts**: Knowledge aggregation, search across services, result ranking
**Time**: 35 minutes

Build comprehensive search:
- Search across multiple knowledge sources
- Aggregate and rank results
- Handle service availability
- Create unified responses

```bash
cargo run --bin multi-source-search
```

### 4. custom-mcp-server
**File**: `custom-mcp-server.rs`
**Concepts**: Building MCP servers, custom tools, server implementation
**Time**: 40 minutes

Create custom MCP servers:
- Implement MCP protocol
- Define custom tools
- Handle client connections
- Deploy your own MCP server

```bash
cargo run --bin custom-mcp-server
```

### 5. mcp-ai-integration
**File**: `mcp-ai-integration.rs`
**Concepts**: AI + MCP, enhanced prompts, tool-augmented responses
**Time**: 30 minutes

Combine AI with external tools:
- Enhance AI prompts with tool data
- Use AI to determine which tools to call
- Create tool-augmented AI responses
- Handle complex multi-step workflows

```bash
cargo run --bin mcp-ai-integration
```

## 🛠 Setup

### 1. Start MCP Test Servers
The project includes Python MCP servers for testing:

```bash
# Start all test servers
./scripts/start_test_servers.sh

# Or start individual servers
cd mcp-servers
python customer_support_server.py &  # Port 8001
python notion_server.py &           # Port 8002  
python slack_server.py &            # Port 8003
```

### 2. Dependencies
Navigate to this directory and install dependencies:

```bash
cd examples/04-mcp-integration
cargo build
```

### 3. Verify MCP Servers
Check that servers are running:

```bash
curl http://localhost:8001/health  # HelpScout MCP
curl http://localhost:8002/health  # Notion MCP
curl http://localhost:8003/health  # Slack MCP
```

## 📖 Key Concepts

### Model Context Protocol (MCP)
MCP is a protocol for connecting AI systems to external tools and data sources:

```rust
use workflow_engine_mcp::clients::http::HttpMCPClient;

// Connect to MCP server
let client = HttpMCPClient::new("http://localhost:8001");

// Discover available tools
let tools = client.list_tools().await?;

// Call a tool
let result = client.call_tool("search_tickets", Some(args)).await?;
```

### MCP Transports
Different ways to connect to MCP servers:

1. **HTTP**: RESTful API connections
2. **WebSocket**: Real-time bidirectional communication
3. **Stdio**: Direct process communication

### Tool Discovery
MCP servers expose their capabilities through tool discovery:

```rust
// List all available tools
let tools = client.list_tools().await?;

for tool in tools {
    println!("Tool: {}", tool.name);
    println!("Description: {}", tool.description);
    println!("Parameters: {:?}", tool.input_schema);
}
```

### Error Handling
MCP operations can fail, so robust error handling is essential:

```rust
match client.call_tool("search", Some(args)).await {
    Ok(result) => {
        // Process successful response
    }
    Err(WorkflowError::MCPError { message }) => {
        // Handle MCP-specific errors
    }
    Err(e) => {
        // Handle other errors
    }
}
```

## 🎓 What You'll Learn

### After basic-mcp-client:
- MCP protocol fundamentals
- Client connection patterns
- Tool discovery and introspection
- Basic tool invocation

### After external-tools:
- Real-world MCP integrations
- Multi-service data aggregation
- Service-specific tool usage
- Response data handling

### After multi-source-search:
- Knowledge base integration patterns
- Search result aggregation
- Ranking and relevance algorithms
- Service availability handling

### After custom-mcp-server:
- MCP server implementation
- Protocol compliance
- Custom tool development
- Server deployment strategies

### After mcp-ai-integration:
- AI + tool integration patterns
- Enhanced prompt generation
- Tool selection algorithms
- Complex workflow orchestration

## 🔧 Available MCP Servers

### HelpScout MCP Server (Port 8001)
Customer support integration:
- `search_tickets` - Search support tickets
- `get_customer` - Get customer information
- `create_ticket` - Create new support tickets
- `update_ticket` - Update ticket status

### Notion MCP Server (Port 8002)
Knowledge base integration:
- `search_pages` - Search Notion pages
- `get_page` - Get specific page content
- `create_page` - Create new pages
- `update_page` - Update existing pages

### Slack MCP Server (Port 8003)
Team communication integration:
- `search_messages` - Search Slack messages
- `get_channel` - Get channel information
- `send_message` - Send messages to channels
- `get_user_info` - Get user information

## 💡 Best Practices

### Connection Management
1. **Connection Pooling**: Reuse connections when possible
2. **Health Checks**: Monitor server availability
3. **Timeouts**: Set appropriate timeout values
4. **Retry Logic**: Implement exponential backoff

### Tool Usage
1. **Discovery First**: Always discover tools before using them
2. **Parameter Validation**: Validate parameters before tool calls
3. **Response Handling**: Handle all possible response types
4. **Caching**: Cache tool responses when appropriate

### Error Handling
1. **Graceful Degradation**: Continue workflow when tools fail
2. **Fallback Strategies**: Have backup plans for critical tools
3. **User Feedback**: Inform users about tool failures
4. **Monitoring**: Track tool usage and failure rates

## 🧪 Testing

### Unit Tests
Test MCP integrations with mock servers:

```bash
# Test individual MCP clients
cargo test basic_mcp_tests
cargo test external_tools_tests

# Test all MCP integration examples
cargo test --package mcp-integration
```

### Integration Tests
Test with real MCP servers (requires running servers):

```bash
# Start test servers first
./scripts/start_test_servers.sh

# Run integration tests
cargo test -- --ignored

# Test specific MCP server
cargo test helpscout_integration -- --ignored
cargo test notion_integration -- --ignored
cargo test slack_integration -- --ignored
```

## 🐛 Troubleshooting

### Common Issues

1. **Connection Refused**
   - Verify MCP servers are running
   - Check port numbers and URLs
   - Ensure no firewall blocking

2. **Tool Not Found**
   - Use `list_tools()` to verify available tools
   - Check tool name spelling
   - Verify server capabilities

3. **Authentication Errors**
   - Check API keys and credentials
   - Verify permission scopes
   - Ensure proper authentication method

4. **Timeout Errors**
   - Increase timeout values
   - Check server responsiveness
   - Implement retry logic

### Debugging

1. **Enable Debug Logging**:
   ```bash
   RUST_LOG=debug cargo run --bin basic-mcp-client
   ```

2. **Test Server Connectivity**:
   ```bash
   curl -v http://localhost:8001/health
   ```

3. **Monitor Network Traffic**:
   Use tools like Wireshark or browser dev tools

### Getting Help

1. **Check Server Logs**: Look at MCP server output
2. **Review MCP Specification**: Understand protocol details
3. **Test with Simple Tools**: Start with basic operations
4. **Check Example Code**: Review working examples

## 🔗 Additional Resources

- **[MCP Specification](https://spec.modelcontextprotocol.io/)**
- **[MCP Python SDK](https://github.com/modelcontextprotocol/python-sdk)**
- **[MCP Server Examples](https://github.com/modelcontextprotocol/servers)**
- **[Anthropic MCP Guide](https://docs.anthropic.com/en/docs/mcp)**

## 📊 MCP Server Health

Monitor your MCP servers:

```bash
# Check all servers
./scripts/check_mcp_health.sh

# Individual server status
curl http://localhost:8001/health | jq .
curl http://localhost:8002/health | jq .
curl http://localhost:8003/health | jq .
```

## 🚀 Performance Tips

### Optimization Strategies
1. **Parallel Tool Calls**: Use `tokio::join!` for concurrent operations
2. **Caching**: Cache expensive tool responses
3. **Connection Pooling**: Reuse HTTP connections
4. **Batch Operations**: Group related tool calls

### Monitoring
1. **Response Times**: Track tool call latencies
2. **Success Rates**: Monitor tool call success/failure
3. **Usage Patterns**: Understand which tools are used most
4. **Resource Usage**: Monitor server resource consumption

---

**Ready to connect to external tools?** Start with the [basic MCP client example](basic-mcp-client.rs) and discover the power of tool-augmented workflows!