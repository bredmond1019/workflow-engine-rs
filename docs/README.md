# AI Workflow Engine Documentation

Welcome to the AI Workflow Engine documentation! This directory contains comprehensive guides and references for using and contributing to the project.

## 📚 Documentation Structure

### Getting Started
- [Quick Start Guide](QUICK_START.md) - Get running in 5 minutes
- [Tutorial Index](tutorials/00-index.md) - Step-by-step tutorials
- [Examples](../examples/README.md) - Working code examples

### Architecture & Design
- [System Architecture](architecture/) - High-level system design and components
  - [Performance Architecture](architecture/performance.md) - Performance considerations
  - [Pricing Engine](architecture/PRICING_ENGINE_IMPLEMENTATION.md) - Token usage and pricing
  - [GraphQL Federation](architecture/GRAPHQL_FEDERATION.md) - GraphQL federation setup
  - [Service Documentation](architecture/services/) - Individual service architectures
- [Workflow Diagrams](workflows/) - Visual workflow representations
- [Security Guide](SECURITY.md) - Security implementation and best practices

### Development
- [Development Setup](development/DEVELOPMENT_SETUP.md) - Set up your development environment
- [Development Setup Guide](development/DEVELOPMENT_SETUP_GUIDE.md) - Detailed setup instructions
- [Testing Guide](development/TESTING.md) - How to run and write tests
- [Troubleshooting](development/TROUBLESHOOTING_ADDENDUM.md) - Common issues and solutions

### Deployment & Operations
- [Monitoring Guide](deployment/MONITORING.md) - Production monitoring setup
- [Scripts Documentation](deployment/SCRIPTS.md) - Deployment and utility scripts

### API Documentation
- **REST API**: The main HTTP API is documented with OpenAPI/Swagger
  - Run the server and visit `http://localhost:8080/swagger-ui/`
- **Service APIs**: Individual service documentation in `architecture/services/`

### Tutorials
1. [Getting Started](tutorials/01-getting-started.md) - Your first workflow
2. [Understanding Nodes](tutorials/02-understanding-nodes.md) - Node types and usage
3. [AI-Powered Automation](tutorials/03-ai-powered-automation.md) - Integrating AI services
4. [External Service Integration](tutorials/04-integrating-external-services.md) - MCP and external APIs
5. [MCP Integration Deep Dive](tutorials/04-mcp-integration.md) - Model Context Protocol
6. [Event Sourcing](tutorials/05-event-sourcing.md) - Event-driven architecture
7. [Scaling Workflows](tutorials/05-scaling-your-workflows.md) - Performance and scaling
8. [Debugging & Monitoring](tutorials/06-debugging-and-monitoring.md) - Observability
9. [Best Practices](tutorials/07-best-practices.md) - Production recommendations

## 🔍 Core Concepts

### Architecture Overview
1. **Workflows**: Directed graphs of processing nodes
2. **Nodes**: Individual processing units (AI agents, MCP clients, templates)
3. **Events**: Event-sourced architecture with CQRS
4. **MCP**: Model Context Protocol for external integrations
5. **Services**: Microservice architecture with specialized components

### Service Components
- **workflow-engine-api**: Main HTTP API server
- **workflow-engine-core**: Core workflow logic and AI integration
- **workflow-engine-mcp**: Model Context Protocol implementation
- **workflow-engine-nodes**: Built-in workflow nodes
- **Content Processing**: Document analysis and processing
- **Knowledge Graph**: Graph database integration
- **Realtime Communication**: WebSocket-based messaging

### Example Workflows
- [Customer Support Demo](../examples/customer-support/README.md) - Automated support workflows
- [Python Client Examples](../examples/python_client/) - Python integration examples
- [Basic Workflow Examples](../examples/) - Simple workflow demonstrations

## 🏗️ Component Documentation

### Core Crates
Each core crate has detailed documentation in its CLAUDE.md file:
- [workflow-engine-api](../crates/workflow-engine-api/CLAUDE.md) - HTTP API server
- [workflow-engine-core](../crates/workflow-engine-core/CLAUDE.md) - Core workflow engine
- [workflow-engine-mcp](../crates/workflow-engine-mcp/CLAUDE.md) - MCP protocol implementation
- [workflow-engine-nodes](../crates/workflow-engine-nodes/CLAUDE.md) - Built-in nodes
- [workflow-engine-app](../crates/workflow-engine-app/CLAUDE.md) - Main application

### Microservices
Specialized services with detailed documentation:
- [Content Processing](../services/content_processing/CLAUDE.md) - Document analysis
- [Knowledge Graph](../services/knowledge_graph/CLAUDE.md) - Graph database service
- [Realtime Communication](../services/realtime_communication/CLAUDE.md) - WebSocket messaging

## 🚀 Quick Commands

### Building and Running
```bash
# Build the project
cargo build --release

# Run the main server
cargo run --bin workflow-engine

# Run with Docker Compose (recommended)
docker-compose up -d

# View logs
docker-compose logs -f ai-workflow-system
```

### Testing
```bash
# Run all tests
cargo test

# Run integration tests (requires MCP servers)
./scripts/start_test_servers.sh
cargo test -- --ignored

# Run specific test suites
cargo test --test end_to_end_workflow_test -- --ignored
cargo test --test load_test -- --ignored --nocapture
```

### Development
```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Start development servers
./scripts/start_test_servers.sh
```

## 🤝 Contributing to Documentation

We welcome documentation improvements! To contribute:

1. Fork the repository
2. Create a new branch for your changes
3. Update or add documentation files in the appropriate `docs/` subdirectory
4. Ensure all links work correctly
5. Submit a pull request

### Documentation Standards
- Use clear, concise language
- Include code examples where relevant
- Keep formatting consistent with existing docs
- Test all code snippets
- Update navigation when adding new sections
- Follow the existing directory structure

### File Organization
- **Root docs/**: Main documentation and navigation
- **docs/architecture/**: System design and component architecture
- **docs/development/**: Development guides and setup
- **docs/deployment/**: Operations and deployment guides
- **docs/tutorials/**: Step-by-step learning materials
- **docs/workflows/**: Workflow diagrams and examples

## 📞 Getting Help

If you can't find what you need:

1. Check the component-specific CLAUDE.md files for detailed guidance
2. Review the tutorials for step-by-step instructions
3. Search [existing issues](https://github.com/yourusername/workflow-engine-rs/issues)
4. Open a [new issue](https://github.com/yourusername/workflow-engine-rs/issues/new) with the "documentation" label

## 📄 Project Files

### Essential Files
- [README.md](../README.md) - Project overview and quick start
- [CONTRIBUTING.md](../CONTRIBUTING.md) - Contribution guidelines
- [CHANGELOG.md](../CHANGELOG.md) - Version history
- [LICENSE](../LICENSE) - Project license
- [SECURITY.md](../SECURITY.md) - Security policy

### Configuration
- [Cargo.toml](../Cargo.toml) - Rust project configuration
- [docker-compose.yml](../docker-compose.yml) - Container orchestration
- [deny.toml](../deny.toml) - Security and license checks

## 📜 License

This documentation is part of the AI Workflow Engine project and is licensed under the same terms. See the [LICENSE](../LICENSE) file for details.