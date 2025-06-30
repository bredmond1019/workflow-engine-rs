# CLAUDE.md - Federation-UI Branch (Enterprise Edition)

This file provides comprehensive guidance to Claude Code (claude.ai/code) when working with the enterprise federation-ui branch of the AI Workflow Engine.

## 🎯 Branch Context

> **You are working on the `federation-ui` branch** - the enterprise, production-ready version with GraphQL Federation, microservices, React frontend, and comprehensive testing.
> 
> For the simpler, learning-focused version, see the `main` branch.

## 🏢 Project Overview

**AI Workflow Engine (Federation-UI)** is a production-ready AI workflow orchestration platform featuring:

- **GraphQL Federation Gateway** - Unified API across all microservices (Apollo Federation v2)
- **Microservices Architecture** - Independent, scalable services with dedicated databases
- **React Frontend** - Modern UI with 174+ TDD tests and real-time updates
- **Enterprise Security** - JWT auth, multi-tenancy, rate limiting, 70+ vulnerabilities prevented
- **Production Monitoring** - Prometheus, Grafana, Jaeger, distributed tracing
- **Event Sourcing** - PostgreSQL-based with CQRS, snapshots, and replay capabilities
- **AI Integration** - OpenAI, Anthropic, AWS Bedrock with token management
- **Model Context Protocol** - Multi-transport MCP implementation

## 📊 Project Status

**95% Ready for Open Source Publication**

### ✅ Completed
- All 224 compilation errors resolved
- TDD implementation with 174+ frontend tests
- GraphQL Federation gateway operational
- Security hardening complete
- Production monitoring integrated
- Comprehensive documentation

### 🔄 Remaining (5%)
- Final crates.io metadata validation
- Dependency version pinning
- Open source license finalization
- Crate-specific README files

## 🏗️ Architecture Overview

### System Architecture
```
Frontend (React:5173) → GraphQL Gateway (4000) → Microservices (8080-8084)
                                ↓
                        Schema Composition
                        Query Planning
                        Entity Resolution
```

### Core Services

1. **GraphQL Gateway** (`workflow-engine-gateway`)
   - Port: 4000
   - Apollo Federation v2
   - Schema composition across all services
   - Intelligent query planning

2. **Main API Server** (`workflow-engine-api`)
   - Port: 8080
   - Authentication & authorization
   - Workflow management
   - Event sourcing
   - MCP integration

3. **Content Processing Service**
   - Port: 8082
   - Document analysis
   - WASM plugin support
   - Vector embeddings (pgvector)
   - Batch processing

4. **Knowledge Graph Service**
   - Port: 3002
   - Dgraph integration
   - Graph algorithms
   - Learning path generation
   - GraphQL API

5. **Realtime Communication Service**
   - Port: 8081
   - WebSocket server
   - Actor model
   - Presence tracking
   - Message routing

## 🚀 Essential Commands

### Quick Start
```bash
# Full stack with Docker
docker-compose up -d

# Development mode
./scripts/run-federation-stack.sh

# Individual services
cargo run --bin workflow-engine       # Main API (8080)
cargo run --bin graphql-gateway       # Federation (4000)
cd frontend && npm run dev            # React UI (5173)
```

### Testing
```bash
# All tests
cargo test
cd frontend && npm test

# Integration tests
./scripts/start_test_servers.sh
cargo test -- --ignored

# Federation tests
./scripts/test-federation.sh
cargo test graphql_federation_integration_test -- --ignored

# Frontend TDD tests
cd frontend && npm test -- --coverage

# E2E tests
cd frontend && npm run test:e2e
```

### Health Checks
```bash
# Federation gateway health
curl http://localhost:4000/health/detailed

# Individual services
curl http://localhost:8080/health
curl http://localhost:8081/health
curl http://localhost:8082/health
curl http://localhost:3002/health
```

## 💡 Development Guidelines

### Working with GraphQL Federation

1. **Adding New Subgraphs**
   ```rust
   // Implement _service and _entities resolvers
   #[Object]
   impl Query {
       async fn _service(&self) -> Service { ... }
       async fn _entities(&self, representations: Vec<Any>) -> Vec<Entity> { ... }
   }
   ```

2. **Entity Resolution**
   - Use `@key` directives for cross-service queries
   - Implement entity resolvers in each service
   - Gateway handles query planning automatically

### Frontend Development

1. **TDD Approach**
   - Write tests first
   - Use React Testing Library
   - Maintain >80% coverage
   - See `frontend/src/**/*.test.tsx` for examples

2. **State Management**
   - Zustand for global state
   - React Query for server state
   - Local state for component-specific data

### Microservice Communication

1. **Internal APIs**
   - Service discovery via bootstrap system
   - Circuit breakers for resilience
   - Correlation IDs for tracing

2. **Event-Driven**
   - Events published to PostgreSQL event store
   - Services subscribe to relevant events
   - Eventual consistency model

## 🔒 Security Considerations

### Authentication
- JWT tokens required (no defaults)
- Set `JWT_SECRET` environment variable
- Token validation in all services
- Multi-tenant context propagation

### Data Protection
- SQL injection prevention
- XSS protection in React
- CSRF tokens
- Rate limiting on all endpoints

## 📁 Key File Locations

### Configuration
- `docker-compose.yml` - Service orchestration
- `.env.example` - Environment template
- `federation_test_config.toml` - Test configuration

### Documentation
- `FEDERATION.md` - GraphQL Federation guide
- `frontend/README.md` - Frontend documentation
- `docs/` - Comprehensive documentation
- Service-specific docs in each service directory

### Testing
- `tests/` - Integration tests
- `frontend/src/**/*.test.tsx` - Frontend tests
- `frontend/e2e/` - End-to-end tests

## 🐛 Common Issues & Solutions

### Federation Gateway Not Starting
```bash
# Check all services are running
docker-compose ps

# Verify subgraph health
curl http://localhost:8080/graphql
curl http://localhost:8082/graphql
```

### Frontend Build Issues
```bash
# Node.js 18 required
nvm use 18

# Clean install
cd frontend
rm -rf node_modules package-lock.json
npm install
```

### Test Failures
```bash
# MCP tests may have race conditions
cargo test mcp_config -- --test-threads=1

# Start test servers for integration tests
./scripts/start_test_servers.sh
```

## 🎯 Current Priorities

1. **Documentation Polish**
   - Ensure all services have comprehensive docs
   - Update examples for federation features
   - Complete API documentation

2. **Testing Coverage**
   - Maintain frontend coverage >80%
   - Add missing integration tests
   - Performance benchmarks

3. **Production Readiness**
   - Kubernetes manifests
   - Monitoring dashboards
   - Deployment guides

## 📚 Learning Path

For developers new to this branch:

1. **Start with Frontend**
   - Run `cd frontend && npm run dev`
   - Explore the UI at http://localhost:5173
   - Review TDD tests in `*.test.tsx` files

2. **Understand Federation**
   - Read `FEDERATION.md`
   - Explore gateway at http://localhost:4000/graphql
   - Trace a query through multiple services

3. **Explore Microservices**
   - Each service has its own README
   - Start with main API (port 8080)
   - Understand event sourcing patterns

4. **Advanced Features**
   - Real-time updates via WebSocket
   - AI integration examples
   - MCP protocol implementation

## 🔧 Useful Development Scripts

```bash
# Run complete federation stack
./scripts/run-federation-stack.sh

# Test federation health
./scripts/test-federation.sh

# Generate GraphQL schema
./scripts/generate-schema.sh

# Run all tests with coverage
./scripts/test-all.sh

# Clean and rebuild
./scripts/clean-build.sh
```

## 🌟 Key Differentiators from Main Branch

1. **GraphQL Federation** - Unified API gateway vs REST only
2. **React Frontend** - Full UI vs CLI only
3. **Microservices** - Distributed vs monolithic
4. **Production Stack** - Complete monitoring vs basic
5. **Advanced Testing** - TDD + E2E vs unit tests only
6. **Enterprise Features** - Multi-tenancy, scaling, etc.

Remember: This branch is for **production deployments**. Use the main branch for learning and prototyping.