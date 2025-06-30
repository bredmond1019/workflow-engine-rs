# CLAUDE.md - AI Workflow Engine (Main Branch)

This file provides comprehensive guidance to Claude Code (claude.ai/code) when working with the AI Workflow Engine project. This documentation reflects the **main branch** - the streamlined, monolithic version designed for learning, prototyping, and straightforward deployments.

## Project Overview

The AI Workflow Engine is a production-ready AI workflow orchestration platform built in Rust, featuring event sourcing, Model Context Protocol (MCP) integration, and advanced AI capabilities. The main branch provides a simplified, monolithic architecture that's ideal for getting started with AI workflows while maintaining enterprise-grade reliability.

### Branch Context
- **Current Branch**: `main` (Streamlined/Learning-focused)
- **Enterprise Branch**: `federation-ui` (Microservices + React frontend + GraphQL Federation)
- **Use Case**: Learning, prototyping, simple deployments
- **Architecture**: Monolithic with optional microservices

## Component-Specific Documentation

Each crate and service has its own CLAUDE.md file with detailed guidance. Navigate to these files for component-specific information:

### Core Crates
- **[workflow-engine-api](crates/workflow-engine-api/CLAUDE.md)**: Main HTTP API server with authentication, workflow endpoints, and service bootstrap
- **[workflow-engine-core](crates/workflow-engine-core/CLAUDE.md)**: Core workflow engine logic, AI integration, error handling, and shared types
- **[workflow-engine-mcp](crates/workflow-engine-mcp/CLAUDE.md)**: Model Context Protocol implementation with multi-transport support
- **[workflow-engine-nodes](crates/workflow-engine-nodes/CLAUDE.md)**: Built-in workflow nodes for AI agents, external MCP, and templates
- **[workflow-engine-app](crates/workflow-engine-app/CLAUDE.md)**: Main binary entry point that integrates all components

### Microservices
- **[content_processing](services/content_processing/CLAUDE.md)**: Document analysis service with WASM plugin support
- **[knowledge_graph](services/knowledge_graph/CLAUDE.md)**: Graph database service with Dgraph integration
- **[realtime_communication](services/realtime_communication/CLAUDE.md)**: WebSocket-based real-time messaging with actor model

## Where to Search for Features

### Authentication & Authorization
- **JWT Implementation**: See `workflow-engine-core` CLAUDE.md (auth module)
- **API Authentication Middleware**: See `workflow-engine-api` CLAUDE.md (middleware section)
- **Service-Level Auth**: See `realtime_communication` CLAUDE.md (JWT validation)

### Database & Persistence
- **Main PostgreSQL/Diesel**: See `workflow-engine-api` CLAUDE.md (database layer)
- **Content Storage (SQLx)**: See `content_processing` CLAUDE.md (database schema)
- **Graph Database (Dgraph)**: See `knowledge_graph` CLAUDE.md (Dgraph integration)
- **Event Sourcing**: See `workflow-engine-api` CLAUDE.md (event-driven architecture)

### API Development
- **REST Endpoints**: See `workflow-engine-api` CLAUDE.md (API endpoints)
- **WebSocket APIs**: See `realtime_communication` CLAUDE.md (WebSocket protocol)
- **GraphQL Support**: See `knowledge_graph` CLAUDE.md (GraphQL parsing)
- **OpenAPI/Swagger**: See `workflow-engine-api` CLAUDE.md (OpenAPI section)

### Workflow & Node Development
- **Core Workflow Engine**: See `workflow-engine-core` CLAUDE.md (workflow module)
- **Built-in Nodes**: See `workflow-engine-nodes` CLAUDE.md (available nodes)
- **Custom Node Creation**: See both `workflow-engine-core` and `workflow-engine-nodes` CLAUDE.md
- **Node Registration**: See `workflow-engine-api` CLAUDE.md (bootstrap section)

### MCP (Model Context Protocol)
- **Protocol Implementation**: See `workflow-engine-mcp` CLAUDE.md (protocol details)
- **Transport Layers**: See `workflow-engine-mcp` CLAUDE.md (HTTP/WebSocket/stdio)
- **External MCP Clients**: See `workflow-engine-nodes` CLAUDE.md (external MCP)
- **MCP Servers**: See Python MCP servers in `mcp-servers/`

### AI Integration
- **AI Providers**: See `workflow-engine-nodes` CLAUDE.md (OpenAI/Anthropic agents)
- **Token Management**: See `workflow-engine-core` CLAUDE.md (AI token section)
- **Templates**: See `workflow-engine-core` CLAUDE.md (template engine)
- **Content Analysis**: See `content_processing` CLAUDE.md (AI integration)

### Real-time Features
- **WebSocket Communication**: See `realtime_communication` CLAUDE.md
- **Actor Model**: See `realtime_communication` CLAUDE.md (actor system)
- **Presence Tracking**: See `realtime_communication` CLAUDE.md (presence features)
- **Message Routing**: See `realtime_communication` CLAUDE.md (routing section)

### Monitoring & Observability
- **Metrics Collection**: See `workflow-engine-api` CLAUDE.md (monitoring section)
- **Structured Logging**: See `workflow-engine-api` CLAUDE.md (correlation tracking)
- **Health Checks**: See individual service CLAUDE.md files
- **Performance Monitoring**: See `workflow-engine-mcp` CLAUDE.md (metrics section)

### Testing
- **Unit Testing Patterns**: See individual crate CLAUDE.md files
- **Integration Testing**: See `workflow-engine-api` CLAUDE.md (testing section)
- **MCP Testing**: See `workflow-engine-mcp` CLAUDE.md (test servers)
- **Load Testing**: See root testing commands below

### Microservice Patterns
- **Service Discovery**: See `workflow-engine-api` CLAUDE.md (bootstrap/discovery)
- **Circuit Breakers**: See `realtime_communication` CLAUDE.md (protection mechanisms)
- **Rate Limiting**: See both `workflow-engine-api` and `realtime_communication` CLAUDE.md
- **Connection Pooling**: See `workflow-engine-mcp` CLAUDE.md (connection pool)

## Essential Commands

### Building and Running

#### Quick Start (Monolithic)
```bash
# Clone and navigate to the project
git clone <repo-url>
cd workflow-engine-rs

# Ensure you're on the main branch
git checkout main

# Build the project
cargo build --release

# Run the main server (single process)
cargo run --bin workflow-engine

# Access the API at http://localhost:8080
```

#### Full Stack with Docker (Recommended)
```bash
# Start the complete stack (includes optional microservices)
docker-compose up -d

# View logs for main application
docker-compose logs -f ai-workflow-system

# Check system health
curl http://localhost:8080/api/v1/health

# Access services:
# - Main API: http://localhost:8080
# - Swagger UI: http://localhost:8080/swagger-ui/
# - Prometheus: http://localhost:9090
# - Grafana: http://localhost:3000 (admin/admin)
```

#### Environment Setup
```bash
# Create .env file with required variables
cat > .env << EOF
DATABASE_URL=postgresql://aiworkflow:aiworkflow123@localhost:5432/ai_workflow
JWT_SECRET=your-secure-jwt-secret-key
OPENAI_API_KEY=your_openai_key  # Optional
ANTHROPIC_API_KEY=your_anthropic_key  # Optional
EOF

# Initialize database
createdb ai_workflow
psql ai_workflow < scripts/init-db.sql
```

### Testing

```bash
# Run all tests
cargo test

# Integration tests (requires MCP servers)
./scripts/start_test_servers.sh
cargo test -- --ignored

# Run new integration test suites
cargo test --test end_to_end_workflow_test -- --ignored
cargo test --test mcp_communication_test -- --ignored
cargo test --test workflow_external_tools_test -- --ignored
cargo test --test load_test -- --ignored --nocapture
cargo test --test chaos_test -- --ignored --nocapture

# Run specific test categories
cargo test mcp_client
cargo test external_mcp_integration -- --ignored
cargo test --test workflow_test

# Python MCP server tests
cd mcp-servers && python -m pytest tests/
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Check for vulnerabilities
cargo audit
```

### Database Setup

```bash
# Create database
createdb ai_workflow_db

# Initialize schema
psql ai_workflow_db < scripts/init-db.sql

# Run migrations (if using Diesel CLI)
diesel setup && diesel migration run

# Service-specific database setup (if using services)
# Content Processing service (uses SQLx)
cd services/content_processing && sqlx migrate run
# Knowledge Graph service (uses Dgraph)
cd services/knowledge_graph/dgraph && docker-compose up -d
```

### Development Servers

```bash
# Start MCP test servers
./scripts/start_test_servers.sh

# Start individual services (if working with microservices)
# Content Processing service
cd services/content_processing && cargo run
# Knowledge Graph service
cd services/knowledge_graph && cargo run
# Realtime Communication service
cd services/realtime_communication && cargo run

# Access services
# Main API: http://localhost:8080
# Swagger UI: http://localhost:8080/swagger-ui/
# Grafana: http://localhost:3000 (admin/admin)
# Prometheus: http://localhost:9090
# MCP Test Servers: HelpScout (8001), Notion (8002), Slack (8003)
```

## System Architecture (Main Branch)

The main branch implements a streamlined, monolithic architecture that can optionally integrate with microservices. This design prioritizes simplicity and ease of deployment while maintaining production-grade capabilities.

### Core Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                    AI Workflow Engine (Main)                    │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐    ┌─────────────────┐    ┌──────────────┐ │
│  │   API Gateway   │    │  Workflow Core  │    │  MCP Client  │ │
│  │  (Port 8080)    │◄──►│    Engine       │◄──►│   Framework  │ │
│  │                 │    │                 │    │              │ │
│  │ • REST APIs     │    │ • Orchestration │    │ • Multi-transport│
│  │ • Authentication│    │ • Node Execution│    │ • Load Balancing│
│  │ • Rate Limiting │    │ • Error Handling│    │ • Health Checks │
│  │ • OpenAPI Docs  │    │ • Event Sourcing│    │                │
│  └─────────────────┘    └─────────────────┘    └──────────────┘ │
│            │                       │                     │      │
│            ▼                       ▼                     ▼      │
│  ┌─────────────────┐    ┌─────────────────┐    ┌──────────────┐ │
│  │   PostgreSQL    │    │   Node Library  │    │  AI Providers│ │
│  │  Event Store    │    │                 │    │              │ │
│  │                 │    │ • AI Agents     │    │ • OpenAI     │ │
│  │ • Events        │    │ • Templates     │    │ • Anthropic  │ │
│  │ • Snapshots     │    │ • Research      │    │ • Custom     │ │
│  │ • Projections   │    │ • External MCP  │    │              │ │
│  └─────────────────┘    └─────────────────┘    └──────────────┘ │
└─────────────────────────────────────────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────────────────────────────┐
│                Optional Microservices Layer                     │
├─────────────────────────────────────────────────────────────────┤
│ ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────────┐ │
│ │ Content Proc.   │ │ Knowledge Graph │ │ Realtime Comm.      │ │
│ │ (Port 8082)     │ │ (Port 3002)     │ │ (Port 8081)         │ │
│ │                 │ │                 │ │                     │ │
│ │ • Doc Analysis  │ │ • Graph DB      │ │ • WebSocket         │ │
│ │ • AI Integration│ │ • GraphQL       │ │ • Actor Model       │ │
│ │ • WASM Plugins  │ │ • Algorithms    │ │ • Presence Tracking │ │
│ └─────────────────┘ └─────────────────┘ └─────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Core Components

1. **API Gateway** (`workflow-engine-api`) - Central REST API server
   - **Purpose**: Primary HTTP interface for all client interactions
   - **Key Features**: JWT authentication, rate limiting, OpenAPI documentation, health checks
   - **Location**: [crates/workflow-engine-api/](crates/workflow-engine-api/)
   - **Documentation**: [workflow-engine-api CLAUDE.md](crates/workflow-engine-api/CLAUDE.md)

2. **Workflow Engine** (`workflow-engine-core`) - Core orchestration logic
   - **Purpose**: Node-based workflow execution with event sourcing
   - **Key Features**: Type-safe node system, error handling, AI integration, template engine
   - **Location**: [crates/workflow-engine-core/](crates/workflow-engine-core/)
   - **Documentation**: [workflow-engine-core CLAUDE.md](crates/workflow-engine-core/CLAUDE.md)

3. **MCP Framework** (`workflow-engine-mcp`) - Model Context Protocol implementation
   - **Purpose**: Standardized communication with AI services and external tools
   - **Key Features**: Multi-transport (HTTP/WebSocket/stdio), connection pooling, load balancing
   - **Location**: [crates/workflow-engine-mcp/](crates/workflow-engine-mcp/)
   - **Documentation**: [workflow-engine-mcp CLAUDE.md](crates/workflow-engine-mcp/CLAUDE.md)

4. **Node Library** (`workflow-engine-nodes`) - Pre-built workflow components
   - **Purpose**: Ready-to-use workflow nodes for common operations
   - **Key Features**: AI agents (OpenAI/Anthropic), template processing, research nodes
   - **Location**: [crates/workflow-engine-nodes/](crates/workflow-engine-nodes/)
   - **Documentation**: [workflow-engine-nodes CLAUDE.md](crates/workflow-engine-nodes/CLAUDE.md)

5. **Application Binary** (`workflow-engine-app`) - Main executable
   - **Purpose**: Entry point that integrates all components
   - **Key Features**: Configuration management, service startup, graceful shutdown
   - **Location**: [crates/workflow-engine-app/](crates/workflow-engine-app/)
   - **Documentation**: [workflow-engine-app CLAUDE.md](crates/workflow-engine-app/CLAUDE.md)

### Optional Microservices

The main branch can optionally integrate with specialized microservices for advanced features:

1. **Content Processing Service** - Document analysis and AI integration
   - **Purpose**: Intelligent document processing with WASM plugin support
   - **Location**: [services/content_processing/](services/content_processing/)
   - **Documentation**: [content_processing CLAUDE.md](services/content_processing/CLAUDE.md)

2. **Knowledge Graph Service** - Graph-based knowledge management
   - **Purpose**: Graph database operations with GraphQL support
   - **Location**: [services/knowledge_graph/](services/knowledge_graph/)
   - **Documentation**: [knowledge_graph CLAUDE.md](services/knowledge_graph/CLAUDE.md)

3. **Realtime Communication Service** - WebSocket-based messaging
   - **Purpose**: Real-time communication with actor model architecture
   - **Location**: [services/realtime_communication/](services/realtime_communication/)
   - **Documentation**: [realtime_communication CLAUDE.md](services/realtime_communication/CLAUDE.md)

### External Integrations

The system integrates with external services through the MCP (Model Context Protocol) framework:

- **AI Providers**: OpenAI, Anthropic, and custom AI services
- **External APIs**: RESTful services, webhooks, and third-party integrations
- **MCP Servers**: Standardized protocol for external tool integration
- **Monitoring Stack**: Prometheus, Grafana, Jaeger for observability

### Key Design Patterns

1. **Service Bootstrap**: Dependency injection container in `crates/workflow-engine-api/src/bootstrap/`
2. **Repository Pattern**: Database access through repositories
3. **Middleware Architecture**: Auth, rate limiting, correlation tracking
4. **Protocol Abstraction**: Multi-transport support for MCP
5. **Type-Safe Node System**: Compile-time checked workflow nodes

### Environment Configuration

Required environment variables:
```
DATABASE_URL=postgresql://username:password@localhost/ai_workflow_db
JWT_SECRET=your-secure-jwt-secret-key
```

Optional AI provider keys:
```
OPENAI_API_KEY=your_key
ANTHROPIC_API_KEY=your_key
```

### Testing Strategy

- **Unit tests**: Alongside source code with `mockall` for mocking
- **Integration tests**: In `/tests` directory, use `--ignored` for external dependencies
- **MCP protocol tests**: Require Python servers (`./scripts/start_test_servers.sh`)
- **Service tests**: Each service has its own test commands
- **External integrations**: Use `cargo test external_mcp_integration -- --ignored`
- **End-to-end tests**: Complete workflow scenarios in `tests/end_to_end_workflow_test.rs`
- **Load tests**: Performance and scalability tests in `tests/load_test.rs`
- **Chaos tests**: Resilience and failure testing in `tests/chaos_test.rs`

#### Service-Specific Testing

```bash
# Content Processing service tests
cd services/content_processing && cargo test

# Knowledge Graph service tests
cd services/knowledge_graph && cargo test

# Realtime Communication service tests
cd services/realtime_communication && cargo test
```

### Common Development Tasks

For detailed step-by-step instructions, see the relevant component CLAUDE.md files:

1. **Adding a new API endpoint**: See [workflow-engine-api CLAUDE.md](crates/workflow-engine-api/CLAUDE.md#common-development-tasks)
2. **Creating a workflow node**: See [workflow-engine-nodes CLAUDE.md](crates/workflow-engine-nodes/CLAUDE.md#common-development-tasks)
3. **Adding MCP integration**: See [workflow-engine-mcp CLAUDE.md](crates/workflow-engine-mcp/CLAUDE.md#common-development-tasks)
4. **Database changes**: 
   - Main app: See [workflow-engine-api CLAUDE.md](crates/workflow-engine-api/CLAUDE.md#database-interactions)
   - Services: See individual service CLAUDE.md files
5. **Monitoring metrics**: See [workflow-engine-api CLAUDE.md](crates/workflow-engine-api/CLAUDE.md#monitoring-module)
6. **Adding microservice**: Follow patterns in existing service CLAUDE.md files
7. **Testing external integrations**: See [workflow-engine-mcp CLAUDE.md](crates/workflow-engine-mcp/CLAUDE.md#testing-approach)

### Debugging Tips

- **Correlation tracking**: Check correlation IDs in logs for request tracing
- **Health checks**: Use `/health/detailed` endpoint for system status
- **Metrics**: Monitor Prometheus metrics at `http://localhost:9090`
- **Dashboards**: View Grafana dashboards for performance insights
- **MCP testing**: Test servers individually with `scripts/test_mcp_server.py`
- **Service debugging**: Each service logs independently, check service-specific ports
- **Integration failures**: Ensure external MCP servers are running before integration tests

### Key Architecture Patterns

1. **Multi-transport MCP**: HTTP, WebSocket, and stdio support in `crates/workflow-engine-mcp/src/transport.rs`
2. **Connection pooling**: MCP client connections managed in `crates/workflow-engine-mcp/src/connection_pool.rs`
3. **Service bootstrap**: Dependency injection container in `crates/workflow-engine-api/src/bootstrap/service.rs`
4. **External integration**: Pattern for external MCP clients in `crates/workflow-engine-nodes/src/external_mcp_client.rs`
5. **Microservice isolation**: Each service in `services/` has independent database and configuration

## Development Workflow

### Getting Started with the Main Branch

1. **Initial Setup**: Use this guide for overall project understanding and setup
2. **Choose Your Deployment**: Decide between monolithic (simple) or with microservices (advanced)
3. **Component Deep-Dive**: Navigate to specific component CLAUDE.md files for detailed work
4. **Cross-Component Integration**: Return to this guide for system-wide integration tasks

### Main Branch Philosophy

The main branch prioritizes:
- **Simplicity**: Minimal configuration and setup requirements
- **Learning**: Clear examples and straightforward architecture
- **Rapid Prototyping**: Quick iteration and testing capabilities
- **Production Viability**: Enterprise-grade features in a simplified package

### Upgrade Path to Federation-UI

When you need enterprise features:
```bash
# Switch to the enterprise branch
git checkout federation-ui

# Follow migration guide
# Note: This includes breaking changes and additional complexity
```

## How to Use This Documentation

### Start Here If You're:
- **New to the project**: Read this entire guide for system overview
- **Setting up development**: Use the Essential Commands section
- **Looking for specific features**: Use the "Where to Search for Features" section
- **Working across components**: Reference the architecture diagrams and component relationships

### Navigate to Component Guides If You're:
- **Implementing API endpoints**: See [workflow-engine-api CLAUDE.md](crates/workflow-engine-api/CLAUDE.md)
- **Creating workflow nodes**: See [workflow-engine-nodes CLAUDE.md](crates/workflow-engine-nodes/CLAUDE.md)
- **Working with MCP**: See [workflow-engine-mcp CLAUDE.md](crates/workflow-engine-mcp/CLAUDE.md)
- **Modifying core logic**: See [workflow-engine-core CLAUDE.md](crates/workflow-engine-core/CLAUDE.md)
- **Working with microservices**: See individual service CLAUDE.md files

### Documentation Navigation Strategy

1. **Orientation Phase**: Read this guide's architecture section and component overview
2. **Planning Phase**: Use "Where to Search for Features" to identify relevant components
3. **Implementation Phase**: Deep-dive into specific component CLAUDE.md files
4. **Integration Phase**: Return here for cross-component patterns and system-wide testing
5. **Deployment Phase**: Reference Essential Commands and environment configuration

### Key Principles for Contributors

- **Monolithic First**: Start with the main branch for development and learning
- **Component Isolation**: Each crate has clear boundaries and responsibilities
- **Event-Driven Design**: Leverage event sourcing for system state management
- **Type Safety**: Use Rust's type system for compile-time guarantees
- **Observability**: Implement comprehensive logging, metrics, and tracing
- **Testing**: Follow the testing patterns established in each component