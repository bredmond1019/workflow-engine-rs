# workflow-engine-nodes

Built-in workflow nodes for the AI workflow engine with production-ready integrations.

[![Crates.io](https://img.shields.io/crates/v/workflow-engine-nodes.svg)](https://crates.io/crates/workflow-engine-nodes)
[![Documentation](https://docs.rs/workflow-engine-nodes/badge.svg)](https://docs.rs/workflow-engine-nodes)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

## Features

- **AI Agent Nodes**: Integration with OpenAI, Anthropic, and AWS Bedrock
- **External MCP Nodes**: Connect to external MCP servers with retry and pooling
- **Research Nodes**: Web research and data analysis capabilities
- **Template Nodes**: Advanced template processing with Handlebars
- **Security First**: Input validation, parameter sanitization, secure connections
- **Type Safety**: Compile-time parameter validation for all nodes
- **Production Ready**: Retry logic, circuit breakers, connection pooling

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
workflow-engine-nodes = "0.6.0"
workflow-engine-core = "0.6.0"
tokio = { version = "1.0", features = ["full"] }
```

Use AI agent nodes:

```rust
use workflow_engine_nodes::prelude::*;
use workflow_engine_core::prelude::*;

#[tokio::main]
async fn main() -> Result<(), WorkflowError> {
    // Create an OpenAI node
    let openai_node = OpenAiNode::new(
        "gpt-4",
        "You are a helpful assistant that analyzes data."
    )?;
    
    // Create workflow
    let workflow = TypedWorkflowBuilder::new("ai_workflow")
        .start_with_node(NodeId::new("ai_analysis"))
        .build()?;
    
    workflow.register_async_node(NodeId::new("ai_analysis"), openai_node);
    
    let result = workflow.run_async(json!({
        "prompt": "Analyze this data: [1, 2, 3, 4, 5]"
    })).await?;
    
    Ok(())
}
```

## Advanced Examples

### External MCP Integration

```rust
use workflow_engine_nodes::external_mcp::{ExternalMcpNode, ExternalMcpConfig};
use workflow_engine_mcp::transport::TransportType;

// Configure external MCP connection
let config = ExternalMcpConfig {
    service_name: "notion".to_string(),
    transport: TransportType::Http {
        base_url: "http://localhost:8002".to_string(),
        pool_config: Default::default(),
    },
    auth: Some(AuthConfig {
        token: Some(std::env::var("NOTION_API_KEY")?),
        headers: None,
    }),
    retry_config: RetryConfig {
        max_attempts: 3,
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(10),
        exponential_base: 2.0,
    },
};

let notion_node = ExternalMcpNode::new(config);

// Use in workflow
let workflow = TypedWorkflowBuilder::new("knowledge_workflow")
    .start_with_node(NodeId::new("search_notion"))
    .then_node(NodeId::new("process_results"))
    .build()?;

workflow.register_node(NodeId::new("search_notion"), notion_node);
```

### Custom AI Agent with Tools

```rust
use workflow_engine_nodes::ai_agents::openai::OpenAIAgentNode;
use workflow_engine_core::nodes::agent::{AgentConfig, ModelProvider};

// Configure agent with MCP tools
let mut config = AgentConfig {
    system_prompt: "You are an AI assistant with access to external tools".to_string(),
    model_provider: ModelProvider::OpenAI,
    model_name: "gpt-4".to_string(),
    mcp_server_uri: Some("http://localhost:8001/mcp".to_string()),
};

let mut agent = OpenAIAgentNode::new(config)?;

// The agent can now use tools from the MCP server
let context = TaskContext::new(json!({
    "prompt": "Search for recent customer feedback and summarize the main issues"
}));

let result = agent.process(context)?;
```

### Template Processing with Context

```rust
use workflow_engine_nodes::template::TemplateNode;

let template_config = TemplateConfig {
    template_string: "Hello {{name}}, your analysis of {{data}} is ready!".to_string(),
    strict_mode: true,
};

let template_node = TemplateNode::new(template_config);

let context = TaskContext::new(json!({
    "name": "Alice",
    "data": "Q4 sales figures"
}));

let result = template_node.process(context)?;
```

## Built-in Nodes

### AI Agents
- `OpenAiNode` - OpenAI GPT integration with function calling
- `AnthropicNode` - Anthropic Claude integration with tools
- `BedrockNode` - AWS Bedrock integration (requires `aws` feature)

### External MCP
- `ExternalMcpNode` - Generic external MCP server client
- `NotionMcpNode` - Notion-specific MCP integration (planned)
- `SlackMcpNode` - Slack-specific MCP integration (planned)

### Research & Analysis
- `ResearchNode` - Web research with source tracking
- `AnalysisNode` - Statistical and data analysis
- `SummaryNode` - Multi-document summarization

### Templates
- `TemplateNode` - Handlebars template processing
- `MarkdownNode` - Markdown generation with frontmatter
- `JsonTemplateNode` - JSON template with schema validation

## Feature Flags

- `default = ["ai-agents", "external-mcp"]` - Common nodes
- `ai-agents` - AI service integration nodes
- `external-mcp` - External MCP server integration
- `research` - Research and analysis nodes
- `template` - Template processing nodes
- `aws` - AWS Bedrock support
- `all` - All node types

## Testing

```bash
# Unit tests
cargo test -p workflow-engine-nodes

# Integration tests (requires services)
./scripts/start_test_servers.sh
cargo test -p workflow-engine-nodes -- --ignored

# Specific node tests
cargo test -p workflow-engine-nodes openai
cargo test -p workflow-engine-nodes external_mcp
```

## Environment Variables

Required for AI nodes:
- `OPENAI_API_KEY` - OpenAI API key
- `ANTHROPIC_API_KEY` - Anthropic API key
- `AWS_REGION` - AWS region for Bedrock
- `AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY` - AWS credentials

Optional:
- `MCP_SERVER_URL` - Default MCP server endpoint
- `NODE_RETRY_ATTEMPTS` - Default retry attempts (default: 3)
- `NODE_TIMEOUT_SECONDS` - Default timeout (default: 30)

## Documentation

For comprehensive documentation, visit [docs.rs/workflow-engine-nodes](https://docs.rs/workflow-engine-nodes).

## Examples

See the [examples directory](../../examples/) for:
- AI agent workflows
- External MCP integration
- Template processing
- Error handling patterns
- Custom node implementation

## Dependencies

This crate depends on:
- `workflow-engine-core` - Core types and traits
- `workflow-engine-mcp` - MCP protocol support

## Contributing

Contributions are welcome! Please read our [Contributing Guide](../../CONTRIBUTING.md) for details.

## License

Licensed under the MIT License. See [LICENSE](../../LICENSE) for details.