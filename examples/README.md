# AI Workflow Engine - Examples & Tutorials

Welcome to the comprehensive examples collection for the AI Workflow Engine! This directory provides a structured learning path from basic concepts to advanced production-ready applications.

## 📚 Learning Path

### Phase 1: Foundation (Start Here!)
- **[01-getting-started](01-getting-started/)** - Basic workflow concepts and first steps
- **[02-core-concepts](02-core-concepts/)** - Essential patterns and node development

### Phase 2: Integration
- **[03-ai-integration](03-ai-integration/)** - OpenAI, Anthropic, and AI agent patterns
- **[04-mcp-integration](04-mcp-integration/)** - Model Context Protocol and external tools

### Phase 3: Advanced Patterns
- **[05-advanced-patterns](05-advanced-patterns/)** - Parallel processing, error handling, and optimization
- **[06-microservices](06-microservices/)** - Multi-service architectures and integration

### Phase 4: Production
- **[07-production-ready](07-production-ready/)** - Monitoring, scaling, and deployment
- **[08-real-world-applications](08-real-world-applications/)** - Complete end-to-end solutions

## 🚀 Quick Start

1. **Prerequisites**: Ensure you have Rust installed and the main server running:
   ```bash
   # In the project root
   cargo run --bin workflow-engine
   ```

2. **Start with Hello World**:
   ```bash
   cd examples/01-getting-started
   cargo run --bin hello-world
   ```

3. **Follow the progression**: Each directory has its own README with specific instructions.

## 📖 Example Categories

### 01-getting-started/
- **hello-world** - Your first workflow
- **basic-nodes** - Understanding the Node trait
- **data-flow** - How data moves through workflows
- **simple-pipeline** - Chaining nodes together

### 02-core-concepts/
- **async-nodes** - Asynchronous processing patterns
- **error-handling** - Robust error management
- **routing** - Conditional workflow paths
- **parallel-processing** - Concurrent node execution
- **state-management** - Working with TaskContext

### 03-ai-integration/
- **openai-agent** - OpenAI API integration
- **anthropic-agent** - Claude API integration
- **multi-model** - Using multiple AI providers
- **prompt-engineering** - Effective prompt patterns
- **token-management** - Cost optimization

### 04-mcp-integration/
- **basic-mcp-client** - Simple MCP server connection
- **external-tools** - Using external MCP tools
- **custom-mcp-server** - Building your own MCP server
- **multi-source-search** - Querying multiple knowledge bases

### 05-advanced-patterns/
- **circuit-breakers** - Resilience patterns
- **caching** - Performance optimization
- **streaming** - Real-time data processing
- **batch-processing** - Bulk operations
- **workflow-composition** - Dynamic workflow building

### 06-microservices/
- **content-processing** - Document analysis service
- **knowledge-graph** - Graph database integration  
- **realtime-communication** - WebSocket messaging
- **service-orchestration** - Coordinating multiple services

### 07-production-ready/
- **monitoring** - Metrics and observability
- **logging** - Structured logging patterns
- **deployment** - Docker and Kubernetes
- **scaling** - Performance optimization
- **security** - Authentication and authorization

### 08-real-world-applications/
- **customer-support** - Complete support automation
- **content-pipeline** - Content creation workflow
- **knowledge-assistant** - AI-powered search and Q&A
- **document-processing** - End-to-end document analysis

## 🛠 Development Setup

### Running Examples Locally

1. **Start Dependencies**:
   ```bash
   # Start PostgreSQL (if using database features)
   docker-compose up -d postgres
   
   # Start MCP test servers (for MCP examples)
   ./scripts/start_test_servers.sh
   ```

2. **Environment Variables**:
   ```bash
   export DATABASE_URL="postgresql://user:pass@localhost/workflow_db"
   export OPENAI_API_KEY="your-openai-key"
   export ANTHROPIC_API_KEY="your-anthropic-key"
   ```

3. **Run Examples**:
   ```bash
   # Individual examples
   cargo run --bin hello-world
   
   # With specific features
   cargo run --bin ai-agent --features="ai-integration"
   
   # Integration tests
   cargo test --test mcp_integration -- --ignored
   ```

### Creating New Examples

1. **Choose the Right Directory**: Place examples in the appropriate learning phase
2. **Follow Naming Conventions**: Use descriptive, kebab-case names
3. **Include Documentation**: Add comprehensive README files
4. **Add Tests**: Include unit and integration tests
5. **Update Main README**: Add your example to the appropriate section

### Example Template

```rust
//! # Example Title
//!
//! Brief description of what this example demonstrates.
//!
//! ## Features Demonstrated
//! - Feature 1
//! - Feature 2
//!
//! ## Usage
//! ```bash
//! cargo run --bin example-name
//! ```

use workflow_engine_core::prelude::*;
use serde_json::json;

#[derive(Debug)]
struct ExampleNode;

impl Node for ExampleNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Implementation here
        Ok(context)
    }
}

#[tokio::main]
async fn main() -> Result<(), WorkflowError> {
    println!("🚀 Starting Example");
    
    // Example implementation
    
    println!("✅ Example completed!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_example_node() {
        // Test implementation
    }
}
```

## 🧪 Testing Examples

### Unit Tests
```bash
# Test individual examples
cargo test --bin hello-world

# Test all examples
cargo test --workspace
```

### Integration Tests
```bash
# Requires external services
cargo test --test integration_examples -- --ignored

# Specific integration test
cargo test --test mcp_integration -- --ignored
```

### End-to-End Tests
```bash
# Full system tests
cargo test --test e2e_examples -- --ignored
```

## 📊 Performance Benchmarks

Some examples include performance benchmarks:

```bash
# Run benchmarks
cargo bench

# Specific benchmark
cargo bench --bench workflow_performance
```

## 🐛 Troubleshooting

### Common Issues

1. **Connection Errors**: Ensure all required services are running
2. **Missing API Keys**: Check environment variables are set
3. **Compilation Errors**: Verify feature flags are correct
4. **Integration Test Failures**: Start test servers with `./scripts/start_test_servers.sh`

### Getting Help

1. **Check Documentation**: Each example has detailed README files
2. **Review Error Messages**: Examples include comprehensive error handling
3. **Check Logs**: Enable debug logging with `RUST_LOG=debug`
4. **Ask for Help**: Open an issue on GitHub

## 🔗 Additional Resources

- **[Main Documentation](../docs/)** - Core system documentation
- **[API Reference](../docs/api/)** - Detailed API documentation
- **[Contributing Guide](../CONTRIBUTING.md)** - How to contribute examples
- **[Security Guidelines](../SECURITY.md)** - Security best practices

## 📝 License

All examples are provided under the same license as the main project. See [LICENSE](../LICENSE) for details.

---

**Happy Learning!** 🎉

Start with the [hello-world example](01-getting-started/hello-world/) and work your way through the learning path. Each example builds on the previous ones, providing a comprehensive understanding of the AI Workflow Engine.