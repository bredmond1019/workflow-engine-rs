# CLAUDE.md - workflow-engine-core

This file provides guidance for Claude Code when working with the workflow-engine-core crate.

## Crate Overview

The workflow-engine-core crate is the heart of the AI workflow orchestration system. It provides the fundamental building blocks for creating, executing, and managing AI-powered workflows. This crate contains zero external service dependencies and focuses on core workflow engine logic, AI integration utilities, and shared types used throughout the system.

## Purpose and Role

This crate serves as:
- **Workflow Engine Foundation**: Core execution engine and node system
- **Type Definitions**: Shared types and traits used across all crates
- **AI Integration Layer**: Template engine and token management for AI services
- **Error Handling Framework**: Comprehensive error types with retry and recovery
- **Core Abstractions**: Node traits, task context, and workflow builder patterns

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
```rust
impl Node for MyNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Extract input data
        let input: MyInput = context.get_event_data()?;
        
        // Process data
        let result = self.process_input(input)?;
        
        // Store results
        context.update_node("my_result", result);
        
        Ok(context)
    }
}
```

### 2. Type-Safe Workflow Construction
```rust
let workflow = TypedWorkflowBuilder::new("my_workflow")
    .start_with_node(NodeId::new("input"))
    .connect_to(NodeId::new("process"))
    .connect_to(NodeId::new("output"))
    .build()?;
```

### 3. Error Handling with Context
```rust
// Detailed error creation
WorkflowError::ProcessingError {
    message: "Failed to process data".to_string(),
}.with_context("node_id", "processor")
 .with_context("attempt", 3)
```

### 4. Template-Based AI Integration
```rust
let template = Template::new("prompt_template")
    .with_content("Process this {{input}} using {{method}}")
    .with_variable("input", VariableType::String)
    .with_variable("method", VariableType::String);
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