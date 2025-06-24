# CLAUDE.md - workflow-engine-nodes

This file provides guidance to Claude Code (claude.ai/code) when working with the workflow-engine-nodes crate.

## Crate Overview

The workflow-engine-nodes crate provides built-in workflow node implementations for the AI workflow orchestration system. It contains ready-to-use nodes for AI agent integrations, external MCP connections, research capabilities, and template processing.

## Purpose and Role

This crate serves as the node library for the workflow engine, providing:
- Pre-built node implementations for common workflow tasks
- AI service integrations (OpenAI, Anthropic, AWS Bedrock)
- External MCP server connectivity for tool access
- Research and analysis capabilities
- Template processing for content generation

## Available Node Types

### AI Agent Nodes (Feature: `ai-agents`)
Located in `src/ai_agents/`:
- **OpenAIAgentNode**: Integration with OpenAI GPT models
- **AnthropicAgentNode**: Integration with Anthropic Claude models
- **BedrockNode** (planned): AWS Bedrock integration

These nodes wrap the `BaseAgentNode` from workflow-engine-core and provide model-specific configurations.

### External MCP Nodes (Feature: `external-mcp`)
Located in `src/external_mcp*.rs`:
- **BaseExternalMcpClient**: Foundation for connecting to external MCP servers
- **ExternalMcpNode**: Generic external MCP server client
- **NotionMcpNode** (planned): Notion-specific MCP integration
- **SlackMcpNode** (planned): Slack-specific MCP integration

### Research Nodes (Feature: `research`)
Located in `src/research.rs`:
- **ResearchNode**: Web research and data gathering
- **AnalysisNode**: Data analysis and insights extraction
- **SummaryNode**: Text summarization capabilities

### Template Nodes (Feature: `template`)
Located in `src/template.rs`:
- **TemplateNode**: Handlebars template processing
- **MarkdownNode**: Markdown generation
- **JsonTemplateNode**: JSON template processing

## AI Agent Implementations

### OpenAI Integration
```rust
use workflow_engine_nodes::ai_agents::openai::OpenAIAgentNode;
use workflow_engine_core::nodes::agent::{AgentConfig, ModelProvider};

let config = AgentConfig {
    system_prompt: "You are a helpful assistant".to_string(),
    model_provider: ModelProvider::OpenAI,
    model_name: "gpt-4".to_string(),
    mcp_server_uri: None, // Optional MCP server for tools
};

let agent = OpenAIAgentNode::new(config)?;
```

### Anthropic Integration
```rust
use workflow_engine_nodes::ai_agents::anthropic::AnthropicAgentNode;
use workflow_engine_core::nodes::agent::{AgentConfig, ModelProvider};

let config = AgentConfig {
    system_prompt: "You are Claude".to_string(),
    model_provider: ModelProvider::Anthropic,
    model_name: "claude-3-opus-20240229".to_string(),
    mcp_server_uri: None,
};

let agent = AnthropicAgentNode::new(config);
```

Both agents support:
- MCP tool integration via `set_mcp_client()`
- Flexible prompt extraction from task context
- Automatic tool selection based on keywords
- Async processing with sync Node interface

## External MCP Client Integration

The external MCP client system provides a framework for connecting to external MCP servers:

### Configuration
```rust
use workflow_engine_nodes::external_mcp::{ExternalMcpConfig, AuthConfig, RetryConfig};
use workflow_engine_mcp::transport::TransportType;

let config = ExternalMcpConfig {
    service_name: "notion".to_string(),
    transport: TransportType::Http {
        base_url: "http://localhost:8002".to_string(),
        pool_config: Default::default(),
    },
    auth: Some(AuthConfig {
        token: Some("api_key".to_string()),
        headers: None,
    }),
    retry_config: RetryConfig::default(), // 3 retries with exponential backoff
};
```

### Transport Support
- **HTTP**: REST-based MCP servers with authentication
- **WebSocket**: Real-time bidirectional communication
- **Stdio**: Process-based MCP servers (Python scripts)

### Features
- Automatic retry with exponential backoff
- Connection pooling for efficiency
- Multiple transport types
- Authentication support (tokens, headers)
- Tool discovery and execution

## Node Registration and Configuration

### Workflow Integration
```rust
use workflow_engine_core::workflow::builder::WorkflowBuilder;
use workflow_engine_nodes::prelude::*;

let workflow = WorkflowBuilder::new("ai_workflow")
    .start_with_node(NodeId::new("validation"))
    .then_node(NodeId::new("ai_processing"))
    .then_node(NodeId::new("formatting"))
    .build()?;

// Register nodes
workflow.register_node(NodeId::new("ai_processing"), OpenAIAgentNode::new(config)?);
```

### Dynamic Node Discovery
Nodes can be registered with the workflow engine's node registry for dynamic discovery:
```rust
use workflow_engine_core::nodes::registry::NodeRegistry;

let mut registry = NodeRegistry::new();
registry.register("openai_agent", || Box::new(OpenAIAgentNode::new(default_config())?));
registry.register("external_mcp", || Box::new(BaseExternalMcpClient::new(mcp_config)));
```

## Testing Approach

### Unit Tests
Each node implementation includes unit tests alongside the source:
- AI agents: Test configuration, MCP client setup
- External MCP: Test connection, retry logic, transport types
- Mock MCP clients for isolated testing

### Integration Tests
For testing with real MCP servers:
```bash
# Start test MCP servers
./scripts/start_test_servers.sh

# Run integration tests
cargo test --package workflow-engine-nodes -- --ignored
```

### Test Patterns
```rust
// Mock external dependencies
use mockall::mock;

mock! {
    TestMcpClient {}
    
    #[async_trait]
    impl McpClient for TestMcpClient {
        async fn connect(&mut self) -> Result<(), WorkflowError>;
        async fn list_tools(&mut self) -> Result<Vec<ToolDefinition>, WorkflowError>;
        // ... other methods
    }
}

// Test with mocks
let mut mock_client = MockTestMcpClient::new();
mock_client.expect_connect().returning(|| Ok(()));
```

## Common Development Tasks

### Adding a New AI Agent

1. Create new file in `src/ai_agents/`:
```rust
// src/ai_agents/new_provider.rs
use workflow_engine_core::nodes::agent::{AgentConfig, BaseAgentNode};

pub struct NewProviderAgentNode {
    base_node: BaseAgentNode,
}

impl NewProviderAgentNode {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            base_node: BaseAgentNode::new(config),
        }
    }
}

// Implement Node and AgentNode traits...
```

2. Add to `src/ai_agents/mod.rs`:
```rust
pub mod new_provider;
```

3. Update feature flags if needed in `Cargo.toml`

### Creating an External MCP Integration

1. Implement specific MCP client:
```rust
// src/notion_mcp.rs
use crate::external_mcp_client::{BaseExternalMcpClient, ExternalMcpConfig};

pub struct NotionMcpNode {
    base_client: BaseExternalMcpClient,
}

impl NotionMcpNode {
    pub fn new() -> Self {
        let config = ExternalMcpConfig {
            service_name: "notion".to_string(),
            transport: TransportType::Http {
                base_url: "http://localhost:8002".to_string(),
                pool_config: Default::default(),
            },
            auth: None,
            retry_config: Default::default(),
        };
        
        Self {
            base_client: BaseExternalMcpClient::new(config),
        }
    }
    
    // Add Notion-specific methods...
}
```

2. Add specialized tool methods for the service

### Implementing a New Node Type

1. Create the node implementation:
```rust
use workflow_engine_core::nodes::Node;
use workflow_engine_core::task::TaskContext;
use workflow_engine_core::error::WorkflowError;

pub struct CustomNode {
    config: CustomConfig,
}

impl Node for CustomNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Process the task
        let data = context.get_data::<YourInputType>("input")?;
        
        // Perform operations
        let result = self.do_something(data)?;
        
        // Update context
        context.set_data("output", result)?;
        Ok(context)
    }
    
    fn node_name(&self) -> String {
        "CustomNode".to_string()
    }
}
```

2. Add tests for the node
3. Update documentation

### Testing External Integrations

1. Use mock MCP servers for unit tests
2. For integration tests with real servers:
```rust
#[tokio::test]
#[ignore] // Run with --ignored flag
async fn test_real_notion_integration() {
    let mut node = NotionMcpNode::new();
    node.connect().await.unwrap();
    
    let tools = node.list_tools().await.unwrap();
    assert!(!tools.is_empty());
    
    // Test specific tool execution
    let result = node.execute_tool("search_pages", Some(params)).await.unwrap();
}
```

## Best Practices

1. **Feature Flags**: Use feature flags for optional dependencies
2. **Error Handling**: Propagate errors appropriately, use WorkflowError types
3. **Async/Sync Bridge**: Handle async operations in sync Node interface carefully
4. **Configuration**: Make nodes configurable via builders or config structs
5. **Documentation**: Document node capabilities and requirements
6. **Testing**: Provide both unit tests with mocks and integration test examples
7. **Retry Logic**: Implement appropriate retry strategies for external services

## Performance Considerations

- AI API calls have inherent latency - consider timeouts
- MCP connections should be reused via connection pooling
- External service calls should have circuit breakers
- Use async operations where possible to avoid blocking

## Dependencies

This crate depends on:
- `workflow-engine-core`: Core workflow engine types and traits
- `workflow-engine-mcp`: MCP protocol implementation
- `tokio`: Async runtime
- `serde`/`serde_json`: Serialization
- `async-trait`: Async trait support
- `reqwest`: HTTP client for external services
- `aws-sdk-bedrockruntime` (optional): AWS Bedrock support

## Environment Variables

Required for AI agents:
- `OPENAI_API_KEY`: OpenAI API authentication
- `ANTHROPIC_API_KEY`: Anthropic API authentication
- `AWS_PROFILE` or AWS credentials: For Bedrock integration

## Debugging Tips

- Enable debug logging: `RUST_LOG=workflow_engine_nodes=debug`
- Check MCP server connectivity before running workflows
- Verify API keys are set in environment
- Use mock clients for testing without external dependencies
- Monitor retry attempts in logs for connection issues