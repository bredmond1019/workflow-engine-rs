# AI Workflow Engine Documentation

Welcome to the AI Workflow Engine documentation! This directory contains comprehensive guides and references for using and contributing to the project.

## 📚 Documentation Structure

### Getting Started
- [Development Setup](../DEVELOPMENT_SETUP.md) - Set up your development environment
- [Quick Start Guide](../QUICK_START.md) - Get running in 5 minutes
- [Testing Guide](TESTING.md) - How to run and write tests

### Architecture & Design
- [System Architecture](architecture.md) - High-level system design
- [API Reference](api-reference.md) - Complete API documentation
- [MCP Protocol Guide](mcp-protocol.md) - Model Context Protocol implementation

### Operations
- [Monitoring Guide](../monitoring/README.md) - Production monitoring setup
- [DevOps Setup](../DEVOPS_SETUP_REPORT.md) - Infrastructure and deployment
- [Performance Tuning](performance.md) - Optimization guidelines

### Development
- [Contributing Guide](../CONTRIBUTING.md) - How to contribute
- [Code Style Guide](code-style.md) - Coding standards and conventions
- [Release Process](release-process.md) - How we ship new versions

## 🔍 Quick Links

### API Documentation
- **REST API**: The main HTTP API is documented with OpenAPI/Swagger
  - Run the server and visit `http://localhost:8080/swagger-ui/`
  - Or view the [OpenAPI spec](../openapi.yaml)

### Core Concepts
1. **Workflows**: Directed graphs of processing nodes
2. **Nodes**: Individual processing units (AI, MCP, custom logic)
3. **Events**: Event-sourced architecture with CQRS
4. **MCP**: Model Context Protocol for external integrations

### Example Workflows
- [Customer Support Demo](../examples/customer-support/README.md)
- [AI Research Assistant](../examples/ai-research/README.md)
- [Content Processing Pipeline](../examples/content-pipeline/README.md)

## 📖 Documentation TODO

The following documentation is planned:
- [ ] Complete API reference with examples
- [ ] Video tutorials for common use cases
- [ ] Performance benchmarking results
- [ ] Security best practices guide
- [ ] Migration guide from v0.5 to v0.6

## 🤝 Contributing to Docs

We welcome documentation improvements! To contribute:

1. Fork the repository
2. Create a new branch for your changes
3. Update or add documentation files
4. Ensure all links work correctly
5. Submit a pull request

### Documentation Standards
- Use clear, concise language
- Include code examples where relevant
- Keep formatting consistent
- Test all code snippets
- Update the table of contents when adding new sections

## 📞 Getting Help

If you can't find what you need:

1. Check the [FAQ](faq.md)
2. Search [existing issues](https://github.com/yourusername/workflow-engine-rs/issues)
3. Join our [Discord community](https://discord.gg/workflow-engine)
4. Open a [new issue](https://github.com/yourusername/workflow-engine-rs/issues/new)

## 📜 License

This documentation is part of the AI Workflow Engine project and is licensed under the same terms. See the [LICENSE](../LICENSE) file for details.