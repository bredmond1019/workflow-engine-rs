# CLAUDE.md - workflow-engine-core

This file provides guidance for Claude Code when working with the workflow-engine-core crate.

## Crate Overview

The workflow-engine-core crate is the heart of the AI workflow orchestration system. It provides the fundamental building blocks for creating, executing, and managing AI-powered workflows. This crate contains zero external service dependencies and focuses on core workflow engine logic, AI integration utilities, and shared types used throughout the system.

## Purpose and Role

This crate serves as the foundational layer for the entire workflow orchestration system:
- **Workflow Engine Foundation**: Core execution engine with type-safe node system
- **Type Definitions**: Shared types, traits, and abstractions used across all workspace crates
- **AI Integration Layer**: Comprehensive template engine and token management for AI services
- **Error Handling Framework**: Advanced error types with circuit breakers, retry policies, and recovery strategies
- **Core Abstractions**: Node traits, task context, workflow builder patterns, and async support
- **Authentication Utilities**: JWT handling and security primitives
- **Configuration Management**: Environment-based configuration with validation
- **Streaming Support**: Real-time data streaming with backpressure and recovery

### Crate Relationships

This crate provides the foundation for:
- **workflow-engine-api**: Uses core types, error handling, auth utilities, and workflow engine
- **workflow-engine-mcp**: Extends core error types and integrates with workflow nodes
- **workflow-engine-nodes**: Implements the Node trait and uses AI integration utilities
- **workflow-engine-app**: Depends on all features for complete functionality

Note: This crate has no dependencies on other workspace crates to prevent circular dependencies.

## Key Components

### 1. Node System (`src/nodes/`)
The foundation of workflow processing:
- **Node Trait** (`mod.rs`): Core trait all workflow components implement
- **Type-Safe Nodes** (`type_safe.rs`): Compile-time checked node system with `NodeId`, `TypedWorkflowBuilder`
- **Agent Nodes** (`agent.rs`, `template_agent.rs`): AI-powered processing nodes
- **Research Node** (`research.rs`): Specialized node for research workflows
- **Node Registry** (`registry.rs`): Runtime node registration and discovery
- **Config Builder** (`config_builder.rs`): Builder pattern for node configuration

### 2. Task Context (`src/task.rs`)
The data carrier through workflow execution:
- Stores event data, node results, and metadata
- Provides type-safe data access methods
- Tracks workflow execution state and timing
- Immutable design with explicit mutation methods

### 3. Workflow Engine (`src/workflow/`)
Core workflow execution logic:
- **Builder** (`builder.rs`, `workflow_builder.rs`): Construct workflows programmatically
- **Schema** (`schema.rs`): Workflow structure and validation
- **Validator** (`validator.rs`): Ensure workflow correctness (no cycles, reachability)

### 4. AI Integration (`src/ai/`)
Tools for AI service integration:

#### Templates (`src/ai/templates/`)
- **Engine** (`engine.rs`): Handlebars-based template rendering
- **Parser** (`parser.rs`): Template parsing and validation
- **Registry** (`registry.rs`): Template storage and management
- **Types** (`types.rs`): Template data structures
- **Storage** (`storage.rs`): Template persistence
- **Validator** (`validator.rs`): Template validation rules

#### Token Management (`src/ai/tokens/`)
- **Counter** (`counter.rs`): Track token usage across requests
- **Analytics** (`analytics.rs`): Token usage analytics and reporting
- **Budget** (`budget.rs`): Token budget management and limits
- **Pricing** (`pricing.rs`): Cost calculation for different AI providers
- **API Clients** (`api_clients/`): Provider-specific implementations (OpenAI, Anthropic, AWS)

### 5. Error Handling (`src/error/`)
Comprehensive error management:
- **Types** (`types.rs`): Main `WorkflowError` enum with all error variants
- **Enhanced Types** (`enhanced_types.rs`): Extended error information
- **Circuit Breaker** (`circuit_breaker.rs`): Fault tolerance patterns
- **Retry** (`retry.rs`): Configurable retry policies
- **Recovery** (`recovery.rs`): Error recovery strategies
- **Context** (`context.rs`): Error context and debugging info
- **Metrics** (`metrics.rs`): Error tracking and monitoring

### 6. Configuration (`src/config/`)
Configuration management:
- **Environment Utils** (`env_utils.rs`): Environment variable handling
- **Validation** (`validation.rs`): Configuration validation
- **Pricing** (`pricing.rs`): AI provider pricing configuration
- **Error** (`error.rs`): Configuration-specific errors

### 7. Authentication (`src/auth/`)
Core authentication utilities:
- **JWT** (`jwt.rs`): JWT token creation and validation
- Shared authentication types and traits

### 8. Models (`src/models/`)
Core data models:
- **Unified** (`unified.rs`): Shared model definitions
- **MCP Stub** (`mcp_stub.rs`): Stub types to avoid circular dependencies

### 9. Testing Utilities (`src/testing/`)
Test infrastructure:
- **Fixtures** (`fixtures.rs`): Common test data and scenarios
- **Mocks** (`mocks.rs`): Mock implementations of core traits
- **Test Config** (`test_config.rs`): Test-specific configuration

### 10. Streaming Support (`src/streaming/`) [Feature-gated]
Real-time data streaming:
- **Types** (`types.rs`): Streaming data structures
- **Handlers** (`handlers.rs`): Stream processing handlers
- **WebSocket** (`websocket.rs`): WebSocket streaming support
- **SSE** (`sse.rs`): Server-sent events support
- **Backpressure** (`backpressure.rs`): Flow control mechanisms
- **Recovery** (`recovery.rs`): Stream recovery strategies

## Important Files and Their Functions

### Core Entry Points
- `lib.rs`: Crate root with public API exports and feature flags
- `mod.rs`: Module organization and re-exports

### Critical Implementations
- `nodes/mod.rs`: Node trait definition - the foundation of all processing
- `task.rs`: TaskContext implementation - the data flow mechanism
- `error/types.rs`: WorkflowError enum - all possible error states
- `workflow/builder.rs`: WorkflowBuilder - programmatic workflow construction
- `ai/templates/engine.rs`: Template rendering engine for AI prompts

## Core Abstractions and Patterns

### 1. Node Processing Pattern

#### Basic Node Implementation
```rust
use workflow_engine_core::prelude::*;

#[derive(Debug)]
struct DataProcessorNode {
    config: ProcessorConfig,
}

impl Node for DataProcessorNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Extract typed input data
        let input: ProcessingRequest = context.get_event_data()
            .map_err(|e| WorkflowError::InvalidInput { 
                message: format!("Missing processing request: {}", e),
                field: "processing_request".to_string(),
                expected: "ProcessingRequest".to_string(),
                received: "None".to_string(),
            })?;
        
        // Process with error handling
        let result = self.process_data(&input.data)
            .map_err(|e| WorkflowError::ProcessingError {
                message: format!("Data processing failed: {}", e),
            }.with_context("node_id", self.node_name())
             .with_context("input_size", input.data.len()))?;
        
        // Store results with metadata
        context.update_node("processed_data", result);
        context.update_node("processing_metadata", json!({
            "processed_at": chrono::Utc::now(),
            "input_size": input.data.len(),
            "output_size": result.len(),
        }));
        
        Ok(context)
    }
    
    fn node_name(&self) -> String {
        "DataProcessorNode".to_string()
    }
}
```

#### Async Node Implementation
```rust
use async_trait::async_trait;

#[derive(Debug)]
struct ApiCallNode {
    client: reqwest::Client,
    endpoint: String,
}

#[async_trait]
impl AsyncNode for ApiCallNode {
    async fn process_async(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let request_data: ApiRequest = context.get_event_data()?;
        
        // Make async API call with timeout
        let response = tokio::time::timeout(
            Duration::from_secs(30),
            self.client.post(&self.endpoint).json(&request_data).send()
        ).await
        .map_err(|_| WorkflowError::TimeoutError {
            operation: "API call".to_string(),
            timeout_duration: Duration::from_secs(30),
        })?
        .map_err(|e| WorkflowError::ExternalServiceError {
            service: "API".to_string(),
            operation: "POST request".to_string(),
            error: e.to_string(),
        })?;
        
        let result: ApiResponse = response.json().await
            .map_err(|e| WorkflowError::DeserializationError {
                message: format!("Failed to parse API response: {}", e),
                data_type: "ApiResponse".to_string(),
            })?;
        
        context.update_node("api_response", result);
        Ok(context)
    }
}
```

### 2. Type-Safe Workflow Construction

#### Simple Linear Workflow
```rust
use workflow_engine_core::nodes::type_safe::*;

let workflow = TypedWorkflowBuilder::new("data_processing_workflow")
    .start_with_node(NodeId::new("input_validation"))
    .connect_to(NodeId::new("data_processing"))
    .connect_to(NodeId::new("output_formatting"))
    .build()?;

// Register nodes with type safety
workflow.register_node(NodeId::new("input_validation"), ValidationNode::new());
workflow.register_node(NodeId::new("data_processing"), DataProcessorNode::new(config));
workflow.register_node(NodeId::new("output_formatting"), FormatterNode::new());
```

#### Complex Branching Workflow
```rust
let workflow = TypedWorkflowBuilder::new("conditional_workflow")
    .start_with_node(NodeId::new("input"))
    .connect_to(NodeId::new("condition_check"))
    // Branch based on condition
    .connect_conditional(
        NodeId::new("condition_check"),
        vec![
            (NodeId::new("path_a"), "condition == 'A'"),
            (NodeId::new("path_b"), "condition == 'B'"),
            (NodeId::new("default_path"), "true"), // Default case
        ]
    )
    // Merge branches
    .connect_to(NodeId::new("merge_results"))
    .build()?;
```

#### Parallel Processing Workflow
```rust
let workflow = TypedWorkflowBuilder::new("parallel_workflow")
    .start_with_node(NodeId::new("input"))
    .parallel_execution(vec![
        NodeId::new("analysis_a"),
        NodeId::new("analysis_b"),
        NodeId::new("analysis_c"),
    ])
    .wait_for_all() // Wait for all parallel nodes to complete
    .connect_to(NodeId::new("combine_results"))
    .build()?;
```

### 3. Advanced Error Handling

#### Error Context and Recovery
```rust
// Create error with rich context
let error = WorkflowError::ProcessingError {
    message: "Failed to process customer data".to_string(),
}.with_context("customer_id", customer.id)
 .with_context("node_id", "customer_processor")
 .with_context("attempt", 3)
 .with_context("timestamp", Utc::now().to_rfc3339());

// Implement error recovery
impl Node for ResilientNode {
    fn process(&self, context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let mut attempts = 0;
        let max_attempts = 3;
        
        loop {
            attempts += 1;
            
            match self.try_process(&context) {
                Ok(result) => return Ok(result),
                Err(e) if attempts < max_attempts => {
                    log::warn!("Attempt {} failed, retrying: {}", attempts, e);
                    std::thread::sleep(Duration::from_millis(100 * attempts as u64));
                    continue;
                },
                Err(e) => return Err(e.with_context("total_attempts", attempts)),
            }
        }
    }
}
```

#### Circuit Breaker Pattern
```rust
use workflow_engine_core::error::circuit_breaker::CircuitBreaker;

#[derive(Debug)]
struct ProtectedNode {
    inner_node: Box<dyn Node>,
    circuit_breaker: CircuitBreaker,
}

impl Node for ProtectedNode {
    fn process(&self, context: TaskContext) -> Result<TaskContext, WorkflowError> {
        self.circuit_breaker.call(|| {
            self.inner_node.process(context.clone())
        })
    }
}
```

### 4. Template-Based AI Integration

#### Advanced Template Usage
```rust
use workflow_engine_core::ai::templates::*;

// Create template with validation
let template = TemplateBuilder::new("customer_support_prompt")
    .with_content(r#"
        You are a customer support assistant. 
        
        Customer: {{customer.name}} (ID: {{customer.id}})
        Issue Type: {{issue.category}}
        Priority: {{issue.priority}}
        
        Customer Message:
        {{issue.message}}
        
        Previous Interactions: {{#each history}}
        - {{date}}: {{summary}}
        {{/each}}
        
        Please provide a helpful response addressing their concern.
    "#)
    .with_variable("customer", VariableType::Object)
    .with_variable("issue", VariableType::Object)
    .with_variable("history", VariableType::Array)
    .with_helper("format_date", |date: &str| {
        // Custom helper function
        chrono::DateTime::parse_from_rfc3339(date)
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|_| date.to_string())
    })
    .validate()?
    .build()?;

// Use template in node
let rendered = template.render(&json!({
    "customer": {
        "name": "John Doe",
        "id": "12345"
    },
    "issue": {
        "category": "billing",
        "priority": "high",
        "message": "I was charged twice for my subscription"
    },
    "history": [
        {"date": "2024-01-15T10:00:00Z", "summary": "Initial inquiry about billing"},
        {"date": "2024-01-16T14:30:00Z", "summary": "Requested account review"}
    ]
}))?;
```

#### Token Management
```rust
use workflow_engine_core::ai::tokens::*;

// Initialize token counter with pricing
let token_counter = TokenCounter::new()
    .with_provider(ModelProvider::OpenAI)
    .with_pricing(PricingConfig {
        input_cost_per_1k: 0.01,
        output_cost_per_1k: 0.03,
        currency: "USD".to_string(),
    })
    .with_budget(TokenBudget {
        max_tokens_per_request: 8000,
        max_cost_per_request: 1.0,
        max_daily_cost: 100.0,
    });

// Count tokens before API call
let input_tokens = token_counter.count_tokens(&prompt_text)?;
token_counter.check_budget(input_tokens, estimated_output_tokens)?;

// Track actual usage
let usage = ApiUsage {
    input_tokens,
    output_tokens: actual_output_tokens,
    total_cost: calculated_cost,
};
token_counter.record_usage(usage)?;
```

## Error Handling Approach

The crate uses a comprehensive error handling strategy:

1. **Single Error Type**: `WorkflowError` enum covers all error cases
2. **Contextual Information**: Errors include relevant context for debugging
3. **Recovery Mechanisms**: Built-in retry policies and circuit breakers
4. **Error Categories**: Errors are categorized by severity and type
5. **Propagation**: Errors bubble up with additional context at each level

### Error Categories
- **Structural Errors**: Workflow construction issues (cycles, unreachable nodes)
- **Runtime Errors**: Processing failures, validation errors
- **External Errors**: API failures, database errors
- **Configuration Errors**: Invalid settings or missing configuration

## Testing Approach

### Unit Testing
- Each module has accompanying unit tests
- Mock traits using `mockall` for dependency injection
- Test both success and error paths
- Use fixtures for common test scenarios

### Integration Testing
- Tests requiring external dependencies use `#[ignore]`
- Template engine tests validate rendering behavior
- Token counter tests verify usage tracking
- Workflow execution tests cover full scenarios

### Test Organization
```bash
# Run all core tests
cargo test -p workflow-engine-core

# Run specific module tests
cargo test -p workflow-engine-core nodes::
cargo test -p workflow-engine-core ai::templates::

# Run with coverage
cargo tarpaulin -p workflow-engine-core
```

## Common Development Tasks

### 1. Adding a New Node Type
1. Create node struct in `src/nodes/` or as a separate module
2. Implement the `Node` trait
3. Add node to registry in workflow initialization
4. Write unit tests for the node
5. Update workflow examples

### 2. Adding Error Variants
1. Add variant to `WorkflowError` in `src/error/types.rs`
2. Implement `Display` formatting for the variant
3. Add error creation convenience methods if needed
4. Update error handling in affected code
5. Add tests for new error cases

### 3. Extending Task Context
1. Add new fields to `TaskContext` struct in `src/task.rs`
2. Add accessor/mutator methods
3. Update serialization if needed
4. Ensure backward compatibility
5. Update tests and documentation

### 4. Adding AI Provider Support
1. Create new module in `src/ai/tokens/api_clients/`
2. Implement provider-specific token counting
3. Add pricing information to configuration
4. Create provider-specific templates if needed
5. Add integration tests

### 5. Creating New Template Helpers
1. Implement helper function in `src/ai/templates/engine.rs`
2. Register helper in template engine
3. Add validation for helper usage
4. Document helper in template guide
5. Add tests for helper functionality

## Performance Considerations

1. **Task Context Cloning**: Context is cloned between nodes - keep data minimal
2. **Template Caching**: Templates are cached after compilation
3. **Token Counting**: Use batch counting for multiple texts
4. **Error Creation**: Avoid expensive operations in error paths
5. **Node Processing**: Keep node operations focused and fast

## Feature Flags

- `default = ["monitoring"]`: Basic monitoring support
- `database`: Diesel ORM integration
- `monitoring`: Prometheus metrics
- `aws`: AWS Bedrock AI support
- `streaming`: Real-time streaming capabilities
- `full`: All features enabled

## Best Practices

1. **Node Design**: Keep nodes focused on a single responsibility
2. **Error Handling**: Always provide context with errors
3. **Testing**: Write tests for both success and failure cases
4. **Documentation**: Use comprehensive doc comments with examples
5. **Type Safety**: Leverage Rust's type system for compile-time guarantees
6. **Async Support**: Use `AsyncNode` for I/O-bound operations
7. **Configuration**: Validate all configuration at startup

## Dependencies and Compatibility

- Requires Rust 1.70+ for async trait support
- Core dependencies are minimal for maximum compatibility
- Optional features add specific dependencies
- No circular dependencies with other crates
- Designed for use in both library and binary contexts