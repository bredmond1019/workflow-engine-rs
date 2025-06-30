# AI Workflow Engine - Main Branch (Streamlined Version)

[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/bredmond1019/workflow-engine-rs/actions)
[![Docker](https://img.shields.io/badge/docker-ready-blue.svg)](https://docker.com)

> **🎯 You are viewing the `main` branch** - the streamlined, monolithic version ideal for learning and prototyping.
> 
> 🚀 **Want enterprise features?** Switch to the [`federation-ui` branch](../../tree/federation-ui) for GraphQL Federation, microservices, React frontend, and production-ready features.

A powerful AI workflow orchestration platform built in Rust, featuring event sourcing, Model Context Protocol (MCP) integration, and AI capabilities. Designed for simplicity and ease of use while maintaining production-grade reliability.

## 🌟 Branch Comparison

| Feature | `main` Branch (This) | [`federation-ui` Branch](../../tree/federation-ui) |
|---------|-------------|---------------------|
| **Architecture** | Monolithic | Microservices + GraphQL Federation |
| **Frontend** | Basic/None | React with 174+ TDD tests |
| **Security** | Basic | Enterprise-grade (70+ vulnerabilities prevented) |
| **Testing** | Unit tests | Comprehensive TDD methodology |
| **Deployment** | Simple Docker | Production-ready with monitoring |
| **Documentation** | Getting started | Comprehensive API docs + examples |
| **Use Case** | Learning, prototypes | Production, enterprise |
| **Setup Time** | 5 minutes | 10-15 minutes |
| **Complexity** | Low | High |

## 🎯 When to Use Each Branch

### Choose `main` Branch (This) If You:
- 🚀 **Getting started** with AI workflows in Rust
- 🧪 **Prototyping** or learning the concepts
- 🏠 **Simple deployments** without complex infrastructure
- ⚡ **Want quick setup** and immediate results
- 📚 **Learning** event sourcing and MCP patterns
- 🔧 **Building simple integrations** with AI services

### Choose [`federation-ui` Branch](../../tree/federation-ui) If You:
- 🏢 **Enterprise production** deployments
- 📈 **Need to scale** across multiple services
- 🎨 **Want a React frontend** with advanced UI
- 🔒 **Require enterprise security** features
- 📊 **Need comprehensive monitoring** and observability
- 🧪 **Want extensive testing** coverage (174+ tests)
- 🌐 **Building GraphQL Federation** architectures

## 🚀 Quick Start (Main Branch)

### Prerequisites
- Rust 1.75+
- Docker & Docker Compose
- PostgreSQL (included in Docker setup)

### 5-Minute Setup

```bash
# Clone repository
git clone <repo-url>
cd workflow-engine-rs

# Ensure you're on main branch
git checkout main

# Start with Docker (simplest)
docker-compose up -d

# OR run locally
cargo run --bin workflow-engine
```

### Access Points
- **Main API**: http://localhost:8080
- **Health Check**: http://localhost:8080/health
- **API Documentation**: http://localhost:8080/swagger-ui/

## 🏗️ Main Branch Architecture

```
┌─────────────────┐
│   API Gateway   │ ← Single unified service
│   (Port 8080)   │
└─────────────────┘
         │
         ├── Event Store (PostgreSQL)
         ├── JWT Authentication
         ├── MCP Protocol Support
         ├── AI Provider Integration
         └── Workflow Execution Engine
```

**Simple, monolithic architecture** - everything runs in one process for ease of development and deployment.

## 📦 Core Features (Main Branch)

### AI Integration
- **OpenAI Support**: GPT models with streaming responses
- **Anthropic Support**: Claude models with advanced reasoning
- **Token Management**: Usage tracking and cost optimization
- **Template Engine**: Dynamic prompt generation

### Event Sourcing
- **PostgreSQL Event Store**: Reliable persistence with ACID guarantees
- **Event Replay**: Reconstruct state from events
- **Snapshots**: Performance optimization for large datasets
- **CQRS Pattern**: Separate read/write models

### MCP (Model Context Protocol)
- **Multi-transport**: HTTP, WebSocket, and stdio support
- **External Integrations**: Connect to external AI services
- **Protocol Compliance**: Full MCP specification implementation

### Workflow Engine
- **Node-based Execution**: Composable workflow components
- **Type Safety**: Compile-time workflow validation
- **Error Handling**: Graceful failure recovery
- **Async Processing**: Non-blocking workflow execution

## 📚 Documentation

- [**Getting Started Guide**](docs/getting-started.md) - Quick introduction
- [**API Documentation**](docs/api.md) - REST API reference
- [**Workflow Guide**](docs/workflows.md) - Building workflows
- [**MCP Integration**](docs/mcp.md) - External service integration
- [**Event Sourcing**](docs/event-sourcing.md) - Event-driven patterns

## 🔧 Development

### Running Tests
```bash
cargo test                    # Unit tests
cargo test -- --ignored     # Integration tests (requires services)
```

### Building
```bash
cargo build                  # Debug build
cargo build --release       # Production build
```

### Code Quality
```bash
cargo fmt                    # Format code
cargo clippy                 # Lint code
```

## 🚀 Deployment

### Docker (Recommended)
```bash
docker-compose up -d
```

### Local Development
```bash
# Set environment variables
export DATABASE_URL="postgresql://user:pass@localhost/ai_workflow_db"
export JWT_SECRET="your-secret-key"

# Run the application
cargo run --bin workflow-engine
```

## 🆙 Upgrading to Federation-UI Branch

When you're ready for production features:

```bash
# Switch to federation-ui branch
git checkout federation-ui

# Note: This includes breaking changes and additional services
# See migration guide: docs/MIGRATION_GUIDE.md
```

## 🤝 Contributing

1. Start with the `main` branch for learning
2. Contribute features to `federation-ui` for production use
3. Follow our [Contributing Guide](CONTRIBUTING.md)

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

---

**🎯 Remember**: Use `main` for simplicity, [`federation-ui`](../../tree/federation-ui) for production!