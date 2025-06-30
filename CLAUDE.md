# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Production-ready AI workflow orchestration platform built in Rust with GraphQL Federation, featuring event sourcing, microservices architecture, and Model Context Protocol (MCP) integration. The system includes a React frontend with 174+ TDD tests and comprehensive monitoring.

**Key Architecture**: Microservices connected via GraphQL Federation gateway (port 4000), with separate services for content processing, knowledge graphs, and real-time communication.

## Project Status

**Publication Readiness: 95% Complete**

### Major Achievements
- ✅ **All 224 compilation errors resolved** - Project builds cleanly
- ✅ **TDD methodology successfully implemented** - Tests 1-3 & 7 complete with 174+ frontend tests
- ✅ **Comprehensive security validation** - 70+ vulnerabilities prevented, hardcoded secrets removed
- ✅ **GraphQL Federation gateway operational** - Running on port 4000 with entity resolution
- ✅ **Full microservices architecture** - All services independently deployable
- ✅ **Production-ready monitoring** - Prometheus, Grafana, Jaeger integrated

### Remaining Items (5%)
- [ ] Final crates.io metadata validation
- [ ] Dependency version pinning for publication
- [ ] Open source license selection
- [ ] README files for each publishable crate

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

## TDD Achievements

### Test Coverage Summary
- **Frontend**: 174+ tests with comprehensive component coverage
- **Backend Integration**: Complete test suites for:
  - End-to-end workflows (`tests/end_to_end_workflow_test.rs`)
  - MCP communication (`tests/mcp_communication_test.rs`)
  - External tool integration (`tests/workflow_external_tools_test.rs`)
  - Load testing (`tests/load_test.rs`)
  - Chaos engineering (`tests/chaos_test.rs`)
- **GraphQL Federation**: Full integration test suite
- **Security Validation**: 70+ vulnerability prevention tests

### Security Improvements
1. **Authentication Hardening**
   - Removed all hardcoded secrets
   - JWT_SECRET now required from environment (no defaults)
   - Enhanced token validation middleware
   
2. **Data Protection**
   - SQL injection prevention via parameterized queries
   - XSS protection in frontend components
   - CSRF token validation
   
3. **Infrastructure Security**
   - Rate limiting on all endpoints
   - Connection pooling with limits
   - Circuit breakers for service resilience

## Current Branch Context

Working on `federation-ui` branch with completed:
- GraphQL Federation gateway implementation
- Frontend TDD implementation (174+ tests)
- Comprehensive security hardening
- Open source preparation (95% ready)
- Full documentation suite

## Testing Documentation

- **[USER_TESTING.md](USER_TESTING.md)** - Step-by-step validation guide
- **[QUICK_TEST_REFERENCE.md](QUICK_TEST_REFERENCE.md)** - Essential test commands
- **[TEST_COVERAGE_REPORT.md](TEST_COVERAGE_REPORT.md)** - Detailed coverage analysis
- **[FEDERATION.md](FEDERATION.md)** - GraphQL Federation architecture and testing