# CLAUDE.md - workflow-engine-nodes

This file provides guidance to Claude Code (claude.ai/code) when working with the workflow-engine-nodes crate.

## Crate Overview

The workflow-engine-nodes crate provides built-in workflow node implementations for the AI workflow orchestration system. It contains ready-to-use nodes for AI agent integrations, external MCP connections, research capabilities, and template processing.

## Purpose and Role in the System

This crate serves as the comprehensive node library for the workflow engine, providing:
- **Pre-built Node Implementations**: Ready-to-use nodes for common workflow tasks and patterns
- **AI Service Integrations**: Native support for OpenAI, Anthropic, and AWS Bedrock with automatic token management
- **External MCP Connectivity**: Seamless integration with external MCP servers for tool access and service communication
- **Research and Analysis**: Specialized nodes for web research, data analysis, and content processing
- **Template Processing**: Advanced template rendering for content generation and AI prompt engineering
- **Utility Nodes**: Common processing patterns like validation, transformation, and routing

### Crate Relationships

This crate builds upon the foundation provided by other workspace crates:
- **workflow-engine-core** (v0.6.0): Implements the `Node` and `AsyncNode` traits, uses AI utilities and error handling
- **workflow-engine-mcp** (v0.6.0): Leverages MCP client framework for external service integration
- **workflow-engine-api**: Nodes are registered and executed via the API server's workflow engine

### External Integrations
- **AI Services**: OpenAI GPT models, Anthropic Claude, AWS Bedrock
- **MCP Servers**: Notion (port 8002), Slack (port 8003), HelpScout (port 8001)
- **External APIs**: Research services, data sources, notification systems

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

### OpenAI Integration with Advanced Features

```rust
use workflow_engine_nodes::ai_agents::openai::OpenAIAgentNode;
use workflow_engine_core::nodes::agent::{AgentConfig, ModelProvider};
use workflow_engine_core::ai::tokens::{TokenBudget, PricingConfig};

// Configure OpenAI agent with token management
let config = AgentConfig {
    system_prompt: "You are a customer support specialist. Analyze customer inquiries and provide helpful, accurate responses.".to_string(),
    model_provider: ModelProvider::OpenAI,
    model_name: "gpt-4-1106-preview".to_string(),
    temperature: 0.7,
    max_tokens: 1000,
    mcp_server_uri: Some("http://localhost:8002".to_string()), // Notion MCP for knowledge base
    token_budget: Some(TokenBudget {
        max_tokens_per_request: 8000,
        max_cost_per_request: 0.50,
        max_daily_cost: 50.0,
    }),
    pricing: Some(PricingConfig {
        input_cost_per_1k: 0.01,
        output_cost_per_1k: 0.03,
        currency: "USD".to_string(),
    }),
};

let agent = OpenAIAgentNode::new(config)?;

// Use in workflow
impl Node for CustomerSupportWorkflow {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Extract customer inquiry
        let inquiry: CustomerInquiry = context.get_event_data()?;
        
        // Prepare AI prompt with context
        context.update_node("ai_prompt", json!({
            "customer_message": inquiry.message,
            "customer_history": inquiry.history,
            "urgency": inquiry.priority,
            "category": inquiry.category
        }));
        
        // Process with OpenAI agent
        let ai_result = self.openai_agent.process(context)?;
        
        // Extract AI response
        let response = ai_result.get_node_data::<serde_json::Value>("ai_response")?;
        
        Ok(ai_result)
    }
}
```

### Anthropic Integration with Tool Usage

```rust
use workflow_engine_nodes::ai_agents::anthropic::AnthropicAgentNode;
use workflow_engine_mcp::prelude::*;

// Create Anthropic agent with external tools
let mut agent = AnthropicAgentNode::builder()
    .model("claude-3-opus-20240229")
    .system_prompt("You are an expert data analyst. Use available tools to gather and analyze information.")
    .temperature(0.3)
    .max_tokens(2000)
    .build()?;

// Add MCP client for external tools
let mcp_client = HttpMcpClient::new("http://localhost:8003")?; // Slack integration
agent.set_mcp_client(Box::new(mcp_client));

// Advanced usage with tool selection
impl AsyncNode for DataAnalysisNode {
    async fn process_async(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let analysis_request: AnalysisRequest = context.get_event_data()?;
        
        // Prepare context with tool availability
        context.update_node("available_tools", json!([
            "search_data_sources",
            "run_statistical_analysis", 
            "generate_visualization",
            "send_slack_notification"
        ]));
        
        context.update_node("analysis_prompt", json!({
            "data_sources": analysis_request.data_sources,
            "analysis_type": analysis_request.analysis_type,
            "output_format": analysis_request.output_format
        }));
        
        // Claude will automatically select and use appropriate tools
        let result = self.anthropic_agent.process_async(context).await?;
        
        Ok(result)
    }
}
```

### AWS Bedrock Integration (Future)

```rust
use workflow_engine_nodes::ai_agents::bedrock::BedrockAgentNode;

// Multi-model Bedrock configuration
let bedrock_agent = BedrockAgentNode::builder()
    .region("us-east-1")
    .model("anthropic.claude-v2")  // Or "amazon.titan-text-express-v1"
    .system_prompt("You are a compliance specialist reviewing documents.")
    .max_tokens(4000)
    .aws_profile("default")
    .build()?;
```

### Common AI Agent Features

All AI agent nodes support:

1. **MCP Tool Integration**: Automatic tool discovery and execution
2. **Token Management**: Budget tracking and cost optimization
3. **Prompt Engineering**: Template-based prompt construction
4. **Context Preservation**: Conversation history and state management
5. **Error Recovery**: Retry policies and fallback strategies
6. **Performance Monitoring**: Token usage analytics and latency tracking

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