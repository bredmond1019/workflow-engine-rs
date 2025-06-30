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
| **🏗️ Architecture** | Monolithic + Optional Services | Microservices + GraphQL Federation |
| **🎨 Frontend** | API-only (bring your own UI) | React with 174+ TDD tests |
| **🔐 Security** | JWT + Rate Limiting | Enterprise-grade (70+ vulnerabilities prevented) |
| **🧪 Testing** | Unit + Integration tests | Comprehensive TDD methodology |
| **🚀 Deployment** | Single Docker Compose | Production-ready with full monitoring |
| **📖 Documentation** | Comprehensive guides | API docs + examples + tutorials |
| **🎯 Use Case** | Learning, prototypes, simple prod | Enterprise, complex production |
| **⏱️ Setup Time** | 5 minutes | 10-15 minutes |
| **📊 Complexity** | Low to Medium | High |
| **🔧 Maintenance** | Minimal | Ongoing microservice management |
| **📈 Scalability** | Vertical scaling | Horizontal + vertical scaling |
| **💰 Cost** | Lower operational overhead | Higher infrastructure costs |

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

### System Overview
```
                    ┌─ Internet ─┐
                    │            │
                    ▼            ▼
              ┌──────────┐  ┌──────────┐
              │   HTTP   │  │   AI     │
              │ Clients  │  │ Services │
              └──────────┘  └──────────┘
                    │            │
                    ▼            ▼
┌─────────────────────────────────────────────────────────────────┐
│                AI Workflow Engine (Port 8080)                   │
├─────────────────────────────────────────────────────────────────┤
│  REST API │ Auth │ Workflows │ MCP Client │ Event Sourcing     │
│           │ JWT  │ Engine    │ Framework  │ PostgreSQL         │
└─────────────────────────────────────────────────────────────────┘
                    │                        │
                    ▼                        ▼
         ┌─────────────────────┐   ┌─────────────────────┐
         │  Optional Services  │   │   Monitoring Stack  │
         │                     │   │                     │
         │ • Content Proc.     │   │ • Prometheus        │
         │ • Knowledge Graph   │   │ • Grafana           │
         │ • Realtime Comm.    │   │ • Jaeger            │
         └─────────────────────┘   └─────────────────────┘
```

### Core Data Flow
```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Client    │    │     API     │    │  Workflow   │    │  External   │
│  Request    │───▶│   Gateway   │───▶│   Engine    │───▶│  AI/Tools   │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
      │                   │                   │                   │
      │            ┌─────────────┐    ┌─────────────┐           │
      │            │    Auth     │    │    Event    │           │
      └────────────│ Middleware  │    │    Store    │───────────┘
                   └─────────────┘    └─────────────┘
                          │                   │
                    ┌─────────────┐    ┌─────────────┐
                    │    Rate     │    │  Monitoring │
                    │  Limiting   │    │   & Logs    │
                    └─────────────┘    └─────────────┘
```

### Component Relationships
```
workflow-engine-app (Binary Entry Point)
    │
    ├── workflow-engine-api (HTTP Server)
    │   ├── Auth & Middleware
    │   ├── REST Endpoints
    │   ├── Health Checks
    │   └── OpenAPI Documentation
    │
    ├── workflow-engine-core (Business Logic)
    │   ├── Workflow Orchestration
    │   ├── Event Sourcing
    │   ├── Error Handling
    │   └── AI Integration Utils
    │
    ├── workflow-engine-mcp (External Communication)
    │   ├── HTTP/WebSocket/Stdio Transports
    │   ├── Connection Pooling
    │   └── Load Balancing
    │
    └── workflow-engine-nodes (Workflow Components)
        ├── AI Agents (OpenAI, Anthropic)
        ├── Template Processing
        ├── Research Nodes
        └── External MCP Clients
```

**Monolithic Design Benefits:**
- Single process deployment
- Simplified configuration
- Easier debugging and development
- Optional microservice integration
- Production-ready out of the box

## 📦 Core Features (Main Branch)

### 🤖 AI Integration
- **Multi-Provider Support**: OpenAI GPT models, Anthropic Claude, and custom AI services
- **Streaming Responses**: Real-time AI output with backpressure handling
- **Token Management**: Cost tracking, usage limits, and optimization analytics
- **Template Engine**: Dynamic prompt generation with Handlebars syntax
- **Error Recovery**: Automatic retry logic and graceful degradation

### 📊 Event Sourcing & CQRS
- **PostgreSQL Event Store**: ACID-compliant event persistence with partitioning
- **Event Replay**: Complete system state reconstruction from events
- **Snapshots**: Performance optimization for large datasets and fast recovery
- **CQRS Pattern**: Optimized read/write models with projection rebuilding
- **Event Versioning**: Schema evolution and backward compatibility

### 🔌 MCP (Model Context Protocol)
- **Multi-Transport**: HTTP REST, WebSocket, and stdio communication
- **Connection Pooling**: Efficient resource management with health checks
- **Load Balancing**: Distribute requests across multiple external services
- **Protocol Compliance**: Full MCP 1.0 specification implementation
- **External Tool Integration**: Seamless integration with external AI tools and services

### ⚡ Workflow Engine
- **Node-Based Execution**: Composable, reusable workflow components
- **Type Safety**: Compile-time validation and runtime type checking
- **Error Handling**: Comprehensive error recovery with context preservation
- **Async Processing**: Non-blocking execution with concurrent workflow support
- **Dynamic Composition**: Runtime workflow building and modification

### 🔐 Security & Authentication
- **JWT Authentication**: Secure token-based authentication with refresh tokens
- **Role-Based Authorization**: Fine-grained permissions and access control
- **Rate Limiting**: Request throttling and abuse prevention
- **CORS Support**: Cross-origin resource sharing configuration
- **Audit Logging**: Complete activity tracking and compliance reporting

### 📈 Monitoring & Observability
- **Prometheus Metrics**: Comprehensive system and business metrics
- **Distributed Tracing**: Request correlation with Jaeger integration
- **Structured Logging**: JSON-formatted logs with correlation IDs
- **Health Checks**: Multi-level health monitoring with detailed status
- **Performance Monitoring**: Real-time performance dashboards

## 📚 Documentation

### Getting Started
- [**5-Minute Quick Start**](#-quick-start-main-branch) - Get running immediately
- [**Tutorial Series**](docs/tutorials/) - Step-by-step learning guides
- [**API Documentation**](http://localhost:8080/swagger-ui/) - Interactive REST API reference
- [**CLAUDE.md**](CLAUDE.md) - Comprehensive AI assistant guidance

### Component Documentation
- [**workflow-engine-api**](crates/workflow-engine-api/CLAUDE.md) - HTTP API server
- [**workflow-engine-core**](crates/workflow-engine-core/CLAUDE.md) - Core engine logic
- [**workflow-engine-mcp**](crates/workflow-engine-mcp/CLAUDE.md) - MCP protocol implementation
- [**workflow-engine-nodes**](crates/workflow-engine-nodes/CLAUDE.md) - Pre-built workflow nodes

### Advanced Topics
- [**Event Sourcing**](docs/tutorials/05-event-sourcing.md) - Event-driven architecture patterns
- [**MCP Integration**](docs/tutorials/04-mcp-integration.md) - External service integration
- [**Security Guide**](docs/SECURITY.md) - Security implementation details
- [**Performance Guide**](docs/performance.md) - Optimization and scaling

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

### Production Deployment (Docker)
```bash
# Production stack with all services
docker-compose up -d

# Check all services are healthy
docker-compose ps

# View system logs
docker-compose logs -f ai-workflow-system

# Monitor resources
docker stats
```

### Development Deployment
```bash
# Quick development setup
cargo run --bin workflow-engine

# Or with environment file
cp .env.example .env
# Edit .env with your configuration
cargo run --bin workflow-engine
```

### Environment Configuration
```bash
# Required variables
DATABASE_URL=postgresql://aiworkflow:aiworkflow123@localhost:5432/ai_workflow
JWT_SECRET=your-secure-256-bit-secret-key

# Optional AI provider keys
OPENAI_API_KEY=sk-your-openai-key
ANTHROPIC_API_KEY=your-anthropic-key

# Optional microservice URLs (if using services)
CONTENT_PROCESSING_URL=http://localhost:8082
KNOWLEDGE_GRAPH_URL=http://localhost:3002
REALTIME_COMM_URL=http://localhost:8081

# Monitoring configuration
RUST_LOG=info
PROMETHEUS_ENDPOINT=http://localhost:9090
JAEGER_ENDPOINT=http://localhost:14268/api/traces
```

### Service Health Checks
```bash
# Main API health
curl http://localhost:8080/api/v1/health

# Detailed system status
curl http://localhost:8080/api/v1/health/detailed

# Individual service health (if running microservices)
curl http://localhost:8082/health  # Content Processing
curl http://localhost:3002/health  # Knowledge Graph
curl http://localhost:8081/health  # Realtime Communication
```

### Performance Tuning
```bash
# Build for production
cargo build --release

# Run with optimized settings
RUST_LOG=warn cargo run --release --bin workflow-engine

# Monitor performance
# - Prometheus: http://localhost:9090
# - Grafana: http://localhost:3000
# - Jaeger: http://localhost:16686
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

## 🎓 Learning Path

### New to AI Workflows?
1. Start with the `main` branch (you're here!)
2. Follow the [5-minute quick start](#-quick-start-main-branch)
3. Work through the [tutorial series](docs/tutorials/)
4. Build your first workflow with the examples
5. When ready for production scale, consider [`federation-ui`](../../tree/federation-ui)

### Experienced with Microservices?
- Jump to [`federation-ui` branch](../../tree/federation-ui) for enterprise features
- Use `main` branch for prototyping and learning new concepts
- Reference this branch for simplified architecture patterns

## 📊 Project Status

- **🟢 Main Branch**: Production-ready, actively maintained
- **🟢 Federation-UI Branch**: Enterprise-ready, full-featured
- **📈 Maturity**: Beta (API stable, features complete)
- **🔄 Release Cycle**: Monthly releases with semantic versioning
- **🛡️ Security**: Regular audits and vulnerability scanning
- **📖 Documentation**: Comprehensive guides and examples

---

**🎯 Quick Decision Guide**:
- **Learning/Prototyping**: Use `main` branch (this one)
- **Enterprise Production**: Use [`federation-ui` branch](../../tree/federation-ui)
- **Simple Production**: `main` branch is production-ready too!