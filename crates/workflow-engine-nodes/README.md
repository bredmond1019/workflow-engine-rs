# workflow-engine-nodes

Built-in workflow nodes for the AI workflow engine.

## Features

- **AI Agent Nodes**: Integration with OpenAI, Anthropic, and AWS Bedrock
- **External MCP Nodes**: Connect to external MCP servers for tool access  
- **Research Nodes**: Perform research and data analysis tasks
- **Template Nodes**: Process templates and generate content
- **Utility Nodes**: Common processing and transformation operations

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

## Built-in Nodes

### AI Agents
- `OpenAiNode` - OpenAI GPT integration
- `AnthropicNode` - Anthropic Claude integration  
- `BedrockNode` - AWS Bedrock integration

### External MCP
- `ExternalMcpNode` - Generic external MCP server client
- `NotionMcpNode` - Notion-specific MCP integration
- `SlackMcpNode` - Slack-specific MCP integration

### Research & Analysis
- `ResearchNode` - Web research and data gathering
- `AnalysisNode` - Data analysis and insights
- `SummaryNode` - Text summarization

### Templates
- `TemplateNode` - Handlebars template processing
- `MarkdownNode` - Markdown generation
- `JsonTemplateNode` - JSON template processing

## Feature Flags

- `ai-agents` - AI service integration nodes (default)
- `external-mcp` - External MCP server integration (default)
- `research` - Research and analysis nodes
- `template` - Template processing nodes
- `all` - All node types

## Documentation

For comprehensive documentation, visit [docs.rs/workflow-engine-nodes](https://docs.rs/workflow-engine-nodes).

## Examples

See the [examples directory](../../examples/) for complete examples of using built-in nodes.

## License

Licensed under the MIT License. See [LICENSE](../../LICENSE) for details.