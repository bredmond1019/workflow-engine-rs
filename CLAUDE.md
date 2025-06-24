# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a production-ready AI workflow orchestration system built in Rust with Python MCP (Model Context Protocol) servers. The system provides a foundation for building AI-powered applications with external service integrations.

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

### Core Components

1. **HTTP API Server** (`crates/workflow-engine-api/src/api/`)
   - Actix-web REST API with JWT authentication
   - Rate limiting and OpenAPI documentation
   - Correlation ID tracking across requests

2. **MCP Framework** (`crates/workflow-engine-mcp/src/`)
   - Complete Model Context Protocol implementation
   - Support for HTTP, WebSocket, and stdio transports
   - Client implementations for external services

3. **Workflow Engine** (`crates/workflow-engine-core/src/workflow/`)
   - Node-based workflow execution
   - Type-safe node registration and discovery
   - Built-in nodes for AI operations

4. **Database Layer** (`crates/workflow-engine-api/src/db/`)
   - PostgreSQL with Diesel ORM
   - Repository pattern for data access
   - Event and session storage

5. **Monitoring** (`crates/workflow-engine-api/src/monitoring/`)
   - Prometheus metrics with custom collectors
   - Structured logging with correlation IDs
   - Grafana dashboards for visualization

6. **Microservices** (`services/`)
   - **Content Processing**: SQLx-based content analysis with WASM plugins
   - **Knowledge Graph**: Dgraph integration with graph algorithms
   - **Realtime Communication**: WebSocket messaging with actor model

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

1. **Adding a new API endpoint**: Update `crates/workflow-engine-api/src/api/routes/`
2. **Creating a workflow node**: Implement in `crates/workflow-engine-nodes/src/` or `crates/workflow-engine-core/src/nodes/`
3. **Adding MCP integration**: Create client in `crates/workflow-engine-mcp/src/clients/`
4. **Database changes**: Main app uses `crates/workflow-engine-api/src/db/models/`, services use their own schemas
5. **Monitoring metrics**: Update `crates/workflow-engine-api/src/monitoring/metrics.rs`
6. **Adding microservice**: Create new service in `services/` with own Cargo.toml
7. **Testing external integrations**: Start test servers first, then use `--ignored` flag

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