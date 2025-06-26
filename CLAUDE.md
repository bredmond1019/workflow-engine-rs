# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a production-ready AI workflow orchestration system built in Rust with Python MCP (Model Context Protocol) servers and **GraphQL Federation support**. The system provides a foundation for building AI-powered applications with external service integrations.

**Current Branch**: `graphql-federation` - This branch contains the GraphQL Federation implementation that unifies multiple services under a single GraphQL gateway.

### Recent Major Changes
- Added GraphQL Gateway (`workflow-engine-gateway`) with Apollo Federation v2 support
- Enhanced Workflow API with federation compliance
- Implemented entity resolution and query planning
- Added GraphQL playground interfaces

## Component-Specific Documentation

Each crate and service has its own CLAUDE.md file with detailed guidance. Navigate to these files for component-specific information:

### Core Crates
- **[workflow-engine-api](crates/workflow-engine-api/CLAUDE.md)**: Main HTTP API server with authentication, workflow endpoints, service bootstrap, and **GraphQL federation subgraph support**
- **[workflow-engine-core](crates/workflow-engine-core/CLAUDE.md)**: Core workflow engine logic, AI integration, error handling, and shared types
- **[workflow-engine-mcp](crates/workflow-engine-mcp/CLAUDE.md)**: Model Context Protocol implementation with multi-transport support
- **[workflow-engine-nodes](crates/workflow-engine-nodes/CLAUDE.md)**: Built-in workflow nodes for AI agents, external MCP, and templates
- **[workflow-engine-app](crates/workflow-engine-app/CLAUDE.md)**: Main binary entry point that integrates all components
- **[workflow-engine-gateway](crates/workflow-engine-gateway/)**: **NEW** - GraphQL Federation gateway with schema composition and query planning

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
- **GraphQL Federation**: See `workflow-engine-gateway` for gateway implementation and `workflow-engine-api` for subgraph support
- **GraphQL Support**: See `knowledge_graph` CLAUDE.md (GraphQL parsing) and services for individual GraphQL APIs
- **OpenAPI/Swagger**: See `workflow-engine-api` CLAUDE.md (OpenAPI section)

### GraphQL Federation Features
- **Schema Composition**: `crates/workflow-engine-gateway/src/federation/schema_registry.rs` 
- **Query Planning**: `crates/workflow-engine-gateway/src/federation/query_planner.rs`
- **Entity Resolution**: `crates/workflow-engine-gateway/src/federation/entities.rs`
- **Federation Directives**: `crates/workflow-engine-gateway/src/federation/directives.rs`
- **Subgraph Client**: `crates/workflow-engine-gateway/src/subgraph.rs`
- **Federation Support in API**: `crates/workflow-engine-api/src/api/graphql/` (schema.rs, handlers.rs)

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

# Run the main server (subgraph)
cargo run --bin workflow-engine

# Run the GraphQL Gateway (NEW)
cargo run --bin graphql-gateway

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

# Test GraphQL Federation (NEW)
./validate_federation.sh
cargo run --example federated_query
cargo run --example test_federation

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
# GraphQL Gateway: http://localhost:4000/graphql (NEW - Federation endpoint)
# Main API: http://localhost:8080
# GraphQL Subgraph API: http://localhost:8080/api/v1/graphql (NEW)
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

5. **GraphQL Gateway** - Federation orchestration layer
   - Details in: [workflow-engine-gateway README](crates/workflow-engine-gateway/README.md)
   - Key features: Schema composition, query planning, entity resolution, multi-subgraph coordination

6. **Microservices** - Specialized processing services
   - **Content Processing**: Details in [content_processing CLAUDE.md](services/content_processing/CLAUDE.md)
   - **Knowledge Graph**: Details in [knowledge_graph CLAUDE.md](services/knowledge_graph/CLAUDE.md)
   - **Realtime Communication**: Details in [realtime_communication CLAUDE.md](services/realtime_communication/CLAUDE.md)

### External Services

MCP servers are implemented in Python (`mcp-servers/`):
- **HelpScout** (port 8001): Customer support integration
- **Notion** (port 8002): Knowledge base integration  
- **Slack** (port 8003): Team communication

### Key Design Patterns

1. **GraphQL Federation**: Apollo Federation v2 with schema composition and entity resolution
2. **Service Bootstrap**: Dependency injection container in `crates/workflow-engine-api/src/bootstrap/`
3. **Repository Pattern**: Database access through repositories
4. **Middleware Architecture**: Auth, rate limiting, correlation tracking
5. **Protocol Abstraction**: Multi-transport support for MCP
6. **Type-Safe Node System**: Compile-time checked workflow nodes

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
6. **Adding GraphQL Federation subgraph**: Follow patterns in `workflow-engine-api` GraphQL implementation
7. **Adding microservice**: Follow patterns in existing service CLAUDE.md files
8. **Testing external integrations**: See [workflow-engine-mcp CLAUDE.md](crates/workflow-engine-mcp/CLAUDE.md#testing-approach)

### Debugging Tips

- **Correlation tracking**: Check correlation IDs in logs for request tracing
- **Health checks**: Use `/health/detailed` endpoint for system status
- **Metrics**: Monitor Prometheus metrics at `http://localhost:9090`
- **Dashboards**: View Grafana dashboards for performance insights
- **MCP testing**: Test servers individually with `scripts/test_mcp_server.py`
- **Service debugging**: Each service logs independently, check service-specific ports
- **Integration failures**: Ensure external MCP servers are running before integration tests

### Key Architecture Patterns

1. **GraphQL Federation**: Schema composition and query planning in `crates/workflow-engine-gateway/src/federation/`
2. **Multi-transport MCP**: HTTP, WebSocket, and stdio support in `crates/workflow-engine-mcp/src/transport.rs`
3. **Connection pooling**: MCP client connections managed in `crates/workflow-engine-mcp/src/connection_pool.rs`
4. **Service bootstrap**: Dependency injection container in `crates/workflow-engine-api/src/bootstrap/service.rs`
5. **External integration**: Pattern for external MCP clients in `crates/workflow-engine-nodes/src/external_mcp_client.rs`
6. **Microservice isolation**: Each service in `services/` has independent database and configuration
7. **Federated entities**: Entity resolution across subgraphs with `@key` directives

## GraphQL Federation Development Guide

### Working with the Federation

#### Starting the Federation Stack
```bash
# Terminal 1: Start the main API (subgraph)
cargo run --bin workflow-engine

# Terminal 2: Start the GraphQL Gateway
cargo run --bin graphql-gateway

# Access GraphQL Playground
# Gateway: http://localhost:4000/graphql
# Subgraph: http://localhost:8080/api/v1/graphql
```

#### Example Federation Queries
```graphql
# Simple workflow query
{
  workflow(id: "123") {
    id
    name
    status
  }
}

# Federation service query
{
  _service {
    sdl
  }
}

# Entity resolution
query ResolveEntities($representations: [_Any!]!) {
  _entities(representations: $representations) {
    ... on Workflow {
      id
      name
      status
    }
  }
}
```

#### Adding New Subgraphs
1. Implement federation directives (`@key`, `@extends`)
2. Add `_service` and `_entities` resolvers
3. Register subgraph in gateway configuration
4. Update schema composition

#### Federation Testing
```bash
# Validate federation setup
./validate_federation.sh

# Run federation examples
cargo run --example federated_query
cargo run --example test_federation

# Test gateway independently
cd crates/workflow-engine-gateway && cargo test
```

### Code Quality Requirements

**Rust Version**: 1.75.0+ (MSRV)

**Code Style**:
```bash
# Format code
cargo fmt

# Run linter (must pass)
cargo clippy -- -D warnings

# Security audit
cargo audit
```

**Testing Requirements**:
- Unit tests for all new functionality
- Integration tests for API endpoints
- Federation tests for GraphQL gateway
- External service integration tests with `--ignored` flag

**Commit Convention**:
- `feat:` - New features
- `fix:` - Bug fixes
- `docs:` - Documentation changes
- `refactor:` - Code refactoring
- `test:` - Test additions/changes
- `chore:` - Maintenance tasks

## How to Use This Documentation

### Navigation Tips
1. Start with this guide for GraphQL Federation and overall architecture
2. Use "Where to Search for Features" to find the right component
3. Navigate to component-specific CLAUDE.md files for detailed work
4. Check recent commits and federation documentation for latest changes