# workflow-engine-nodes

Comprehensive library of built-in workflow nodes for AI-powered automation systems.

## Features

- **AI Agent Integration**: Native support for OpenAI, Anthropic, and AWS Bedrock with automatic token management
- **External MCP Connectivity**: Seamless integration with external MCP servers for tool access and service communication
- **Research & Analysis**: Specialized nodes for web research, data analysis, and content processing
- **Template Processing**: Advanced template rendering for content generation and AI prompt engineering
- **Utility & Transformation**: Common processing patterns like validation, routing, and data transformation
- **Production Ready**: Built-in error handling, retry policies, and performance monitoring

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
workflow-engine-nodes = "0.6.0"
workflow-engine-core = "0.6.0"
workflow-engine-mcp = "0.6.0"
tokio = { version = "1.0", features = ["full"] }
serde_json = "1.0"
```

### Basic AI Agent Usage

```rust
use workflow_engine_nodes::prelude::*;
use workflow_engine_core::prelude::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), WorkflowError> {
    // Create OpenAI agent node
    let openai_agent = OpenAIAgentNode::builder()
        .model("gpt-4-1106-preview")
        .system_prompt("You are a data analysis expert. Provide clear, actionable insights.")
        .temperature(0.3)
        .max_tokens(1000)
        .build()?;
    
    // Create workflow with AI processing
    let workflow = TypedWorkflowBuilder::new("data_analysis_workflow")
        .start_with_node(NodeId::new("data_validation"))
        .connect_to(NodeId::new("ai_analysis"))
        .connect_to(NodeId::new("format_results"))
        .build()?;
    
    // Register nodes
    workflow.register_node(NodeId::new("data_validation"), DataValidationNode::new());
    workflow.register_async_node(NodeId::new("ai_analysis"), openai_agent);
    workflow.register_node(NodeId::new("format_results"), ResultFormatterNode::new());
    
    // Execute workflow
    let result = workflow.run_async(json!({
        "dataset": [1, 2, 3, 4, 5, 10, 15, 20],
        "analysis_type": "statistical_summary",
        "include_visualization": true
    })).await?;
    
    println!("Analysis complete: {:?}", result.get_final_result());
    Ok(())
}
```

### External MCP Integration

```rust
use workflow_engine_nodes::external_mcp::*;
use workflow_engine_mcp::prelude::*;

#[tokio::main]
async fn main() -> Result<(), WorkflowError> {
    // Create Notion MCP client node
    let notion_node = ExternalMcpNode::builder()
        .service_name("notion")
        .transport(TransportType::Http {
            base_url: "http://localhost:8002".to_string(),
            pool_config: Default::default(),
        })
        .auth_token("your-notion-token")
        .build()?;
    
    // Create Slack notification node  
    let slack_node = ExternalMcpNode::builder()
        .service_name("slack")
        .transport(TransportType::WebSocket {
            endpoint: "ws://localhost:8003/mcp".to_string(),
            auto_reconnect: true,
        })
        .build()?;
    
    // Build knowledge base workflow
    let workflow = TypedWorkflowBuilder::new("knowledge_workflow")
        .start_with_node(NodeId::new("search_notion"))
        .connect_to(NodeId::new("process_results"))
        .connect_to(NodeId::new("notify_slack"))
        .build()?;
    
    workflow.register_async_node(NodeId::new("search_notion"), notion_node);
    workflow.register_node(NodeId::new("process_results"), ResultProcessorNode::new());
    workflow.register_async_node(NodeId::new("notify_slack"), slack_node);
    
    // Execute with search query
    let result = workflow.run_async(json!({
        "query": "customer onboarding process",
        "limit": 10,
        "notify_channel": "#team-updates"
    })).await?;
    
    Ok(())
}
```

### Template Processing Node

```rust
use workflow_engine_nodes::template::*;

#[tokio::main] 
async fn main() -> Result<(), WorkflowError> {
    // Create template node for email generation
    let email_template = TemplateNode::builder()
        .template_content(r#"
            Subject: {{subject}}
            
            Dear {{customer.name}},
            
            Thank you for contacting us about {{issue.category}}.
            
            {{#if issue.urgent}}
            This has been marked as urgent and will be prioritized.
            {{/if}}
            
            Issue Details:
            - ID: {{issue.id}}
            - Priority: {{issue.priority}}
            - Description: {{issue.description}}
            
            {{#each suggested_actions}}
            - {{this}}
            {{/each}}
            
            Best regards,
            {{agent.name}}
            Customer Support Team
        "#)
        .with_helper("format_date", |date: &str| {
            chrono::DateTime::parse_from_rfc3339(date)
                .map(|dt| dt.format("%B %d, %Y").to_string())
                .unwrap_or_else(|_| date.to_string())
        })
        .build()?;
    
    let workflow = TypedWorkflowBuilder::new("email_generation")
        .start_with_node(NodeId::new("generate_email"))
        .build()?;
    
    workflow.register_node(NodeId::new("generate_email"), email_template);
    
    let result = workflow.run_async(json!({
        "subject": "Re: Your Support Request #12345",
        "customer": {
            "name": "John Doe",
            "email": "john@example.com"
        },
        "issue": {
            "id": "12345",
            "category": "billing",
            "priority": "high",
            "urgent": true,
            "description": "Double charge on credit card"
        },
        "suggested_actions": [
            "Review your recent transactions",
            "Check for duplicate charges",
            "Contact your bank if needed"
        ],
        "agent": {
            "name": "Sarah Johnson"
        }
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