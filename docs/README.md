# AI Workflow Engine Documentation Hub

Welcome to the AI Workflow Engine documentation! This is your central hub for all project documentation. The system is a production-ready AI workflow orchestration platform built in Rust with GraphQL Federation, featuring 174+ TDD tests and comprehensive security hardening.

## 🚀 Project Status: 95% Publication Ready

- ✅ All 224 compilation errors resolved
- ✅ TDD methodology successfully implemented
- ✅ Security hardening complete (70+ vulnerabilities prevented)
- ✅ GraphQL Federation operational
- ✅ Production monitoring integrated

## 📚 Documentation Index

### 🏗️ Architecture & Design
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - Comprehensive system architecture with diagrams
- **[FEDERATION.md](../FEDERATION.md)** - GraphQL Federation implementation guide
- **[System Design](architecture.md)** - Additional architecture details
- **[API Reference](api-reference.md)** - Complete API documentation
- **[MCP Protocol Guide](mcp-protocol.md)** - Model Context Protocol implementation

### 🚦 Getting Started
- **[CLAUDE.md](../CLAUDE.md)** - AI assistant guide and project overview
- **[Quick Start Guide](../QUICK_START.md)** - Get running in 5 minutes
- **[Development Setup](../DEVELOPMENT_SETUP.md)** - Complete dev environment setup
- **[Tutorial Series](tutorials/00-index.md)** - Step-by-step learning path

### 🧪 Testing & Quality
- **[USER_TESTING.md](../USER_TESTING.md)** - Comprehensive validation guide
- **[QUICK_TEST_REFERENCE.md](../QUICK_TEST_REFERENCE.md)** - Essential test commands
- **[TEST_COVERAGE_REPORT.md](../TEST_COVERAGE_REPORT.md)** - Detailed coverage analysis
- **[Testing Guide](TESTING.md)** - How to run and write tests

### 📊 Operations & Monitoring
- **[Performance Guide](performance.md)** - Optimization and benchmarks
- **[DEVOPS_SETUP_REPORT.md](../DEVOPS_SETUP_REPORT.md)** - Infrastructure setup
- **[Monitoring Guide](../monitoring/README.md)** - Prometheus, Grafana, Jaeger
- **[Security Guide](SECURITY.md)** - Security best practices

### 📦 Publication & Release
- **[PUBLICATION_STATUS.md](PUBLICATION_STATUS.md)** - Open source readiness (95% complete)
- **[Release Process](release-process.md)** - How we ship new versions
- **[PRICING_ENGINE_IMPLEMENTATION.md](PRICING_ENGINE_IMPLEMENTATION.md)** - Token usage and pricing

### 🔧 Component Documentation

#### Core Crates
- **[workflow-engine-core](../crates/workflow-engine-core/CLAUDE.md)** - Core engine and types
- **[workflow-engine-mcp](../crates/workflow-engine-mcp/CLAUDE.md)** - MCP protocol implementation
- **[workflow-engine-nodes](../crates/workflow-engine-nodes/CLAUDE.md)** - Pre-built workflow nodes
- **[workflow-engine-api](../crates/workflow-engine-api/CLAUDE.md)** - REST/GraphQL API server
- **[workflow-engine-gateway](../crates/workflow-engine-gateway/README.md)** - Federation gateway
- **[workflow-engine-app](../crates/workflow-engine-app/CLAUDE.md)** - Main application

#### Microservices
- **[Content Processing](../services/content_processing/CLAUDE.md)** - Document analysis service
- **[Knowledge Graph](../services/knowledge_graph/CLAUDE.md)** - Graph database service
- **[Realtime Communication](../services/realtime_communication/CLAUDE.md)** - WebSocket service

#### Frontend
- **[React Frontend](../frontend/README.md)** - TypeScript UI with 174+ TDD tests

### 📖 Tutorials & Examples

#### Tutorial Series
1. **[Getting Started](tutorials/01-getting-started.md)** - Your first workflow
2. **[Understanding Nodes](tutorials/02-understanding-nodes.md)** - Node architecture
3. **[AI-Powered Automation](tutorials/03-ai-powered-automation.md)** - AI integration
4. **[MCP Integration](tutorials/04-mcp-integration.md)** - External services
5. **[Event Sourcing](tutorials/05-event-sourcing.md)** - CQRS/ES patterns
6. **[Debugging & Monitoring](tutorials/06-debugging-and-monitoring.md)** - Observability
7. **[Best Practices](tutorials/07-best-practices.md)** - Production tips

#### Workflow Examples
- **[Workflow Diagrams](workflows/workflow_diagrams.md)** - Visual workflow examples
- **[Advanced Workflows](workflows/advanced_workflow_diagram.md)** - Complex patterns
- **[Additional Examples](workflows/additional_workflow_diagram.md)** - More use cases

### 🛠️ Development Resources

#### Guides
- **[Contributing Guide](../CONTRIBUTING.md)** - How to contribute
- **[Code Style Guide](code-style.md)** - Rust coding standards
- **[PR Guidelines](pr-guidelines.md)** - Pull request best practices

#### API Access
- **REST API**: `http://localhost:8080/swagger-ui/`
- **GraphQL Playground**: `http://localhost:4000/graphql`
- **[OpenAPI Spec](../openapi.yaml)** - REST API specification

## 🎯 Key Features Documentation

### GraphQL Federation
- Unified API gateway on port 4000
- Schema composition across microservices
- Entity resolution with `@key` directives
- See [FEDERATION.md](../FEDERATION.md) for details

### Event Sourcing & CQRS
- PostgreSQL-backed event store
- Snapshot support for performance
- Event replay capabilities
- See [Event Sourcing Tutorial](tutorials/05-event-sourcing.md)

### AI Integration
- OpenAI GPT-3.5/4 support
- Anthropic Claude integration
- AWS Bedrock (optional)
- Token management and budgeting
- See [AI Tutorial](tutorials/03-ai-powered-automation.md)

### Security Features
- JWT authentication (no hardcoded secrets)
- Rate limiting on all endpoints
- SQL injection prevention
- XSS protection
- See [Security Guide](SECURITY.md)

## 📊 Performance Metrics

- **API Throughput**: 15,000+ req/sec
- **WebSocket Connections**: 10,000+ concurrent
- **Frontend Tests**: 174+ passing (TDD)
- **Security Tests**: 70+ vulnerabilities prevented
- **Memory Usage**: ~100MB base + 2MB/workflow

## 🔍 Quick Reference

### Service Ports
- **GraphQL Federation Gateway**: 4000
- **Main API**: 8080
- **Frontend**: 5173
- **Content Processing**: 8082
- **Knowledge Graph**: 3002
- **Realtime Communication**: 8081
- **Prometheus**: 9090
- **Grafana**: 3000

### Essential Commands
```bash
# Quick start
docker-compose up -d

# Run all tests
cargo test --all
cd frontend && npm test

# Check federation health
curl http://localhost:4000/health/detailed

# View documentation
cargo doc --open
```

## 📞 Getting Help

If you can't find what you need:

1. Search this documentation
2. Check [GitHub Issues](https://github.com/bredmond1019/workflow-engine-rs/issues)
3. Review [Discussions](https://github.com/bredmond1019/workflow-engine-rs/discussions)
4. Open a [new issue](https://github.com/bredmond1019/workflow-engine-rs/issues/new)

## 🤝 Contributing to Documentation

We welcome documentation improvements! Please:
- Use clear, concise language
- Include code examples
- Test all code snippets
- Update navigation when adding pages
- Follow our [Contributing Guide](../CONTRIBUTING.md)

## 📜 License

This documentation is part of the AI Workflow Engine project. License details coming soon as part of the 5% remaining publication tasks.

---

*Last updated: December 2024 - 95% ready for open source publication*