# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Production-ready AI workflow orchestration platform built in Rust with GraphQL Federation, featuring event sourcing, microservices architecture, and Model Context Protocol (MCP) integration. The system includes a React frontend with 174+ TDD tests and comprehensive monitoring.

**Key Architecture**: Microservices connected via GraphQL Federation gateway (port 4000), with separate services for content processing, knowledge graphs, and real-time communication.

## Essential Commands

### Running the System

```bash
# Quick start - Full stack with Docker
docker-compose up -d

# Development - Run individual components
cargo run --bin workflow-engine          # Main API (8080)
cargo run --bin graphql-gateway          # Federation Gateway (4000)
cd frontend && npm run dev               # Frontend (5173)

# Start all services for federation
./scripts/run-federation-stack.sh
```

### Testing

```bash
# Run all Rust tests
cargo test

# Run single test
cargo test test_name -- --exact

# Integration tests (requires external services)
./scripts/start_test_servers.sh
cargo test -- --ignored

# MCP tests (run sequentially to avoid race conditions)
cargo test mcp_config -- --test-threads=1

# Frontend tests (174+ TDD tests)
cd frontend && npm test
cd frontend && npm test -- --coverage

# Federation tests
./scripts/test-federation.sh
cargo test graphql_federation_integration_test -- --ignored
```

### Development

```bash
# Format and lint
cargo fmt
cargo clippy -- -D warnings

# Database setup
createdb ai_workflow_db
diesel migration run

# Check federation health
curl http://localhost:4000/health/detailed

# Environment setup
export JWT_SECRET="your-secure-secret"  # Required - no default
export DATABASE_URL="postgresql://user:pass@localhost/ai_workflow_db"
```

## High-Level Architecture

### Core System Design

```
┌─────────────┐     ┌──────────────────┐     ┌─────────────────┐
│   Frontend  │────▶│ GraphQL Gateway  │────▶│   Microservices │
│   (React)   │     │   (Port 4000)    │     │  (8080-8084)   │
└─────────────┘     └──────────────────┘     └─────────────────┘
                            │
                    ┌───────┴────────┐
                    │                │
              Federation      Entity Resolution
```

### Service Architecture

1. **GraphQL Gateway** (`workflow-engine-gateway`): Apollo Federation v2 gateway that unifies all services
2. **Main API** (`workflow-engine-api`): Core workflow management, auth, event sourcing
3. **Content Processing** (port 8082): Document analysis with WASM plugins
4. **Knowledge Graph** (port 3002): Dgraph-backed graph database
5. **Realtime Communication** (port 8081): WebSocket messaging with actor model

### Key Patterns

- **Event Sourcing**: PostgreSQL-backed with CQRS, snapshots, and replay (`crates/workflow-engine-api/src/db/events/`)
- **Service Bootstrap**: Dependency injection container (`crates/workflow-engine-api/src/bootstrap/`)
- **MCP Protocol**: Multi-transport (HTTP/WebSocket/stdio) with connection pooling
- **Federation**: Schema composition and query planning across services
- **Multi-tenancy**: Schema, row-level, and hybrid isolation modes

### AI Integration Points

- **Providers**: OpenAI, Anthropic, AWS Bedrock (with `aws` feature)
- **Token Management**: Usage tracking, budgets, analytics (`workflow-engine-core/src/ai/tokens/`)
- **Node System**: Type-safe workflow nodes with AI agents (`workflow-engine-nodes/`)
- **Template Engine**: Handlebars with AI context injection

## Critical Implementation Details

### Authentication
- JWT-based auth requires `JWT_SECRET` environment variable (no default for security)
- Token validation middleware in `workflow-engine-api/src/middleware/auth.rs`
- Multi-tenant context in `workflow-engine-api/src/db/tenant.rs`

### GraphQL Federation
- Subgraph implementations must include `_service` and `_entities` resolvers
- Entity resolution uses `@key` directives for cross-service queries
- Gateway handles partial failures gracefully
- See `FEDERATION.md` for complete guide

### Database Migrations
- Main app: Diesel migrations in `crates/workflow-engine-api/migrations/`
- Content service: SQLx migrations in `services/content_processing/migrations/`
- Always run migrations before starting services

### Testing Infrastructure
- Frontend: TDD with React Testing Library, 174+ tests
- Backend: Unit + integration tests, some require `--ignored` flag
- MCP tests: May have race conditions, use `--test-threads=1`
- Visual dashboard: `frontend/test-dashboard/`

## Service-Specific Notes

### Frontend
- Vite 4.4.0 (pinned for Node.js 18 compatibility)
- Zustand for state management
- GraphQL client with federation support
- TDD methodology throughout

### MCP Servers
- Python-based in `mcp-servers/`
- HelpScout (8001), Notion (8002), Slack (8003)
- Use stdio protocol for communication
- Start with `./scripts/start_test_servers.sh`

### Monitoring Stack
- Prometheus metrics at `:9090`
- Grafana dashboards at `:3000` (admin/admin)
- Jaeger tracing included
- Health endpoints on all services

## Current Branch Context

Working on `graphql-federation` branch with completed:
- GraphQL Federation gateway implementation
- Frontend TDD implementation (174+ tests)
- Security hardening (removed hardcoded secrets)
- Open source preparation (95% ready)

## Testing Documentation

- **[USER_TESTING.md](USER_TESTING.md)** - Step-by-step validation guide
- **[QUICK_TEST_REFERENCE.md](QUICK_TEST_REFERENCE.md)** - Essential test commands
- **[TEST_COVERAGE_REPORT.md](TEST_COVERAGE_REPORT.md)** - Detailed coverage analysis
- **[FEDERATION.md](FEDERATION.md)** - GraphQL Federation architecture and testing