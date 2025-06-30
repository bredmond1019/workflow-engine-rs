# workflow-engine-core

Foundation library for building AI-powered workflow automation systems with type-safe node composition.

## Features

- **Type-Safe Workflow Engine**: Compile-time verified workflow construction with `TypedWorkflowBuilder`
- **Flexible Node System**: Both sync `Node` and async `AsyncNode` traits for composable processing units
- **Rich Task Context**: Comprehensive data container with metadata, error handling, and type safety
- **Advanced Error Handling**: Circuit breakers, retry policies, contextual errors, and recovery strategies
- **AI Integration Suite**: Template engine, token management, and multi-provider AI support
- **Authentication Utilities**: JWT handling and security primitives
- **Configuration Management**: Environment-based configuration with validation
- **Streaming Support**: Real-time data processing with backpressure handling

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
workflow-engine-core = "0.6.0"
tokio = { version = "1.0", features = ["full"] }
serde_json = "1.0"
async-trait = "0.1"
```

### Basic Node Implementation

```rust
use workflow_engine_core::prelude::*;
use serde_json::json;

#[derive(Debug)]
struct GreetingNode {
    template: String,
}

impl GreetingNode {
    fn new(template: &str) -> Self {
        Self {
            template: template.to_string(),
        }
    }
}

impl Node for GreetingNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Extract input with validation
        let input: serde_json::Value = context.get_event_data()
            .map_err(|e| WorkflowError::InvalidInput {
                message: format!("Missing input data: {}", e),
                field: "input".to_string(),
                expected: "JSON object".to_string(),
                received: "None".to_string(),
            })?;
        
        let name = input.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("World");
        
        // Process with the template
        let message = self.template.replace("{name}", name);
        
        // Store result with metadata
        context.update_node("greeting", json!({
            "message": message,
            "processed_at": chrono::Utc::now(),
            "template_used": self.template,
        }));
        
        Ok(context)
    }
    
    fn node_name(&self) -> String {
        "GreetingNode".to_string()
    }
}

#[tokio::main]
async fn main() -> Result<(), WorkflowError> {
    // Create a type-safe workflow
    let workflow = TypedWorkflowBuilder::new("greeting_workflow")
        .start_with_node(NodeId::new("greeting"))
        .build()?;
    
    // Register nodes
    workflow.register_node(
        NodeId::new("greeting"), 
        GreetingNode::new("Hello, {name}! Welcome to the workflow engine.")
    );
    
    // Execute workflow
    let result = workflow.run(json!({
        "name": "Alice",
        "source": "api"
    })).await?;
    
    // Access results
    if let Some(greeting) = result.get_node_data::<serde_json::Value>("greeting")? {
        println!("{}", greeting["message"]);
        println!("Processed at: {}", greeting["processed_at"]);
    }
    
    Ok(())
}
```

### Async Node with External API

```rust
use async_trait::async_trait;
use reqwest::Client;

#[derive(Debug)]
struct WeatherNode {
    client: Client,
    api_key: String,
}

#[async_trait]
impl AsyncNode for WeatherNode {
    async fn process_async(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let location: String = context.get_data("location")?;
        
        // Make async API call with timeout and error handling
        let weather_data = tokio::time::timeout(
            Duration::from_secs(10),
            self.client.get(&format!(
                "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}",
                location, self.api_key
            )).send()
        ).await
        .map_err(|_| WorkflowError::TimeoutError {
            operation: "Weather API call".to_string(),
            timeout_duration: Duration::from_secs(10),
        })?
        .map_err(|e| WorkflowError::ExternalServiceError {
            service: "OpenWeatherMap".to_string(),
            operation: "GET weather".to_string(),
            error: e.to_string(),
        })?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| WorkflowError::DeserializationError {
            message: format!("Failed to parse weather data: {}", e),
            data_type: "WeatherResponse".to_string(),
        })?;
        
        context.update_node("weather", weather_data);
        Ok(context)
    }
}
```

### Complex Workflow with Branching

```rust
use workflow_engine_core::nodes::type_safe::*;

#[tokio::main]
async fn main() -> Result<(), WorkflowError> {
    let workflow = TypedWorkflowBuilder::new("customer_service_workflow")
        .start_with_node(NodeId::new("input_validation"))
        .connect_to(NodeId::new("sentiment_analysis"))
        .connect_conditional(
            NodeId::new("sentiment_analysis"),
            vec![
                (NodeId::new("escalate_to_human"), "sentiment == 'negative'"),
                (NodeId::new("auto_response"), "sentiment == 'positive'"),
                (NodeId::new("detailed_analysis"), "sentiment == 'neutral'"),
            ]
        )
        .connect_to(NodeId::new("generate_response"))
        .connect_to(NodeId::new("send_email"))
        .build()?;
    
    // Register all nodes
    workflow.register_node(NodeId::new("input_validation"), InputValidationNode::new());
    workflow.register_async_node(NodeId::new("sentiment_analysis"), SentimentAnalysisNode::new());
    workflow.register_node(NodeId::new("escalate_to_human"), EscalationNode::new());
    workflow.register_node(NodeId::new("auto_response"), AutoResponseNode::new());
    workflow.register_async_node(NodeId::new("detailed_analysis"), DetailedAnalysisNode::new());
    workflow.register_async_node(NodeId::new("generate_response"), ResponseGeneratorNode::new());
    workflow.register_async_node(NodeId::new("send_email"), EmailSenderNode::new());
    
    // Execute with customer input
    let result = workflow.run(json!({
        "customer_id": "12345",
        "message": "I'm having trouble with my order",
        "priority": "normal"
    })).await?;
    
    println!("Workflow completed: {:?}", result.get_final_result());
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