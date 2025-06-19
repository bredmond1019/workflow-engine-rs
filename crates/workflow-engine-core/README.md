# workflow-engine-core

Core workflow engine primitives and execution engine for building AI-powered automation systems.

## Features

- **Node-based Architecture**: Composable processing units with the `Node` and `AsyncNode` traits
- **Type-safe Workflows**: Compile-time checked workflow construction with `TypedWorkflowBuilder`
- **Task Context**: Rich data container that flows through workflow execution
- **Error Handling**: Comprehensive error types with retry logic and circuit breakers
- **AI Integration**: Template engine and token management for AI services

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

## Feature Flags

- `database` - Database integration with Diesel ORM
- `monitoring` - Prometheus metrics collection  
- `aws` - AWS Bedrock AI integration
- `full` - Enables all optional features

## Documentation

For comprehensive documentation, visit [docs.rs/workflow-engine-core](https://docs.rs/workflow-engine-core).

## Examples

See the [examples directory](../../examples/) for complete examples including:
- Basic workflow construction
- Async nodes with external APIs
- Custom node implementations
- Error handling patterns

## License

Licensed under the MIT License. See [LICENSE](../../LICENSE) for details.