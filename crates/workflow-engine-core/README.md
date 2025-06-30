# workflow-engine-core

Core workflow engine primitives and execution engine for building AI-powered automation systems.

[![Crates.io](https://img.shields.io/crates/v/workflow-engine-core.svg)](https://crates.io/crates/workflow-engine-core)
[![Documentation](https://docs.rs/workflow-engine-core/badge.svg)](https://docs.rs/workflow-engine-core)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

## Features

- **Node-based Architecture**: Composable processing units with the `Node` and `AsyncNode` traits
- **Type-safe Workflows**: Compile-time checked workflow construction with `TypedWorkflowBuilder`
- **Task Context**: Rich data container that flows through workflow execution
- **Optimized Error Handling**: Memory-efficient error types (16 bytes) with comprehensive context
- **AI Integration**: Template engine and token management for AI services
- **GraphQL Support**: Query validation and security features for GraphQL workflows
- **MCP Protocol**: Model Context Protocol validation and integration
- **Security First**: TDD-driven security validation across all components

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
workflow-engine-core = "0.6.0"
tokio = { version = "1.0", features = ["full"] }
serde_json = "1.0"
```

Create a simple workflow:

```rust
use workflow_engine_core::prelude::*;
use serde_json::json;

#[derive(Debug)]
struct GreetingNode;

impl Node for GreetingNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let input: serde_json::Value = context.get_event_data()?;
        let name = input.get("name").and_then(|v| v.as_str()).unwrap_or("World");
        
        context.update_node("greeting", json!({
            "message": format!("Hello, {}!", name)
        }));
        
        Ok(context)
    }
}

#[tokio::main]
async fn main() -> Result<(), WorkflowError> {
    let workflow = TypedWorkflowBuilder::new("hello_workflow")
        .start_with_node(NodeId::new("greeting"))
        .build()?;
    
    workflow.register_node(NodeId::new("greeting"), GreetingNode);
    
    let result = workflow.run(json!({"name": "Alice"})).await?;
    
    if let Some(greeting) = result.get_node_data::<serde_json::Value>("greeting")? {
        println!("{}", greeting["message"]); // "Hello, Alice!"
    }
    
    Ok(())
}
```

## Advanced Examples

### Creating an Async Node

```rust
use workflow_engine_core::prelude::*;
use async_trait::async_trait;

#[derive(Debug)]
struct ApiCallNode {
    endpoint: String,
}

#[async_trait]
impl AsyncNode for ApiCallNode {
    async fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let response = reqwest::get(&self.endpoint)
            .await
            .map_err(|e| WorkflowError::api_error(e.to_string()))?;
        
        let data = response.json::<serde_json::Value>()
            .await
            .map_err(|e| WorkflowError::deserialization_error(e.to_string()))?;
        
        context.update_node("api_response", data);
        Ok(context)
    }
}
```

### Error Handling with Retry

```rust
use workflow_engine_core::error::{WorkflowError, retry::RetryPolicy};

let policy = RetryPolicy::exponential()
    .with_max_attempts(3)
    .with_initial_delay(Duration::from_millis(100));

let result = policy.retry(|| {
    // Your operation that might fail
    risky_operation()
}).await?;
```

### Template-Based AI Integration

```rust
use workflow_engine_core::ai::templates::{Template, TemplateEngine};

let template = Template::builder()
    .name("summarizer")
    .content("Summarize the following text in {{style}} style: {{text}}")
    .variable("style", VariableType::String)
    .variable("text", VariableType::String)
    .build()?;

let engine = TemplateEngine::new();
engine.register_template(template)?;

let rendered = engine.render("summarizer", json!({
    "style": "concise",
    "text": "Long document content here..."
}))?;
```

## Feature Flags

- `default = ["monitoring"]` - Basic monitoring support
- `database` - Database integration with Diesel ORM
- `monitoring` - Prometheus metrics collection  
- `aws` - AWS Bedrock AI integration
- `streaming` - Real-time streaming capabilities
- `full` - Enables all optional features

## Breaking Changes (v0.6.0)

### Error Type Refactoring
Error variants now use boxed details to reduce memory usage:

```rust
// Before
match error {
    WorkflowError::ValidationError { message, field, .. } => {
        println!("Validation failed for {}: {}", field, message);
    }
}

// After
match error {
    WorkflowError::ValidationError(details) => {
        println!("Validation failed for {}: {}", details.field, details.message);
    }
}
```

## Documentation

For comprehensive documentation, visit [docs.rs/workflow-engine-core](https://docs.rs/workflow-engine-core).

## Examples

See the [examples directory](../../examples/) for complete examples including:
- Basic workflow construction
- Async nodes with external APIs
- Custom node implementations
- Error handling patterns
- GraphQL integration
- MCP protocol usage

## Dependencies

Core dependencies are kept minimal for maximum compatibility:
- `serde` - Serialization framework
- `serde_json` - JSON support
- `thiserror` - Error derivation
- `async-trait` - Async trait support
- `tokio` - Async runtime (optional)

## Contributing

Contributions are welcome! Please read our [Contributing Guide](../../CONTRIBUTING.md) for details.

## License

Licensed under the MIT License. See [LICENSE](../../LICENSE) for details.