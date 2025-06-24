# CLAUDE.md - Parent Guide

This file provides high-level guidance to Claude Code (claude.ai/code) when working with code in this repository. For detailed component-specific guidance, refer to the individual CLAUDE.md files in each crate and service directory.

## Project Overview

This is a production-ready AI workflow orchestration system built in Rust with Python MCP (Model Context Protocol) servers. The system provides a foundation for building AI-powered applications with external service integrations.

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

```bash
# Build the project
cargo build
cargo build --release

# Run the main server
cargo run --bin workflow-engine

# Run with Docker Compose (recommended for full stack)
docker-compose up -d

# View logs
docker-compose logs -f ai-workflow-system
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

## Architecture Overview

This section provides a high-level overview. For detailed component information, refer to the individual CLAUDE.md files linked above.

### Core Components

1. **HTTP API Server** - Main REST API gateway
   - Details in: [workflow-engine-api CLAUDE.md](crates/workflow-engine-api/CLAUDE.md)
   - Key features: Actix-web, JWT auth, rate limiting, OpenAPI docs

2. **MCP Framework** - Model Context Protocol implementation
   - Details in: [workflow-engine-mcp CLAUDE.md](crates/workflow-engine-mcp/CLAUDE.md)
   - Key features: Multi-transport support, connection pooling, load balancing

3. **Workflow Engine** - Core orchestration logic
   - Details in: [workflow-engine-core CLAUDE.md](crates/workflow-engine-core/CLAUDE.md)
   - Key features: Node-based execution, type-safe registry, AI integration

4. **Node Library** - Pre-built workflow nodes
   - Details in: [workflow-engine-nodes CLAUDE.md](crates/workflow-engine-nodes/CLAUDE.md)
   - Key features: AI agents, external MCP clients, templates

5. **Microservices** - Specialized processing services
   - **Content Processing**: Details in [content_processing CLAUDE.md](services/content_processing/CLAUDE.md)
   - **Knowledge Graph**: Details in [knowledge_graph CLAUDE.md](services/knowledge_graph/CLAUDE.md)
   - **Realtime Communication**: Details in [realtime_communication CLAUDE.md](services/realtime_communication/CLAUDE.md)

### External Services

MCP servers are implemented in Python (`mcp-servers/`):
- **HelpScout** (port 8001): Customer support integration
- **Notion** (port 8002): Knowledge base integration  
- **Slack** (port 8003): Team communication

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

## How to Use This Documentation

### When to Use This Parent Guide
- **Initial project understanding**: Start here to understand the overall architecture
- **Finding features**: Use the "Where to Search for Features" section to locate functionality
- **Cross-component work**: When working across multiple crates/services
- **General commands**: Reference the essential commands that apply system-wide

### When to Use Component-Specific CLAUDE.md Files
- **Deep diving into a crate/service**: Go directly to the component's CLAUDE.md
- **Component-specific tasks**: Each CLAUDE.md has tailored development tasks
- **Detailed architecture**: Component files have in-depth architectural details
- **Testing strategies**: Each component has specific testing approaches

### Navigation Tips
1. Start with this parent guide for orientation
2. Use "Where to Search for Features" to find the right component
3. Navigate to the specific component's CLAUDE.md for detailed work
4. Return to this guide for cross-component integration tasks