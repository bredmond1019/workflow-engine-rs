# AI Workflow System - Quick Start Guide

**For experienced developers who want to get up and running quickly.**

This guide gets you from zero to running AI workflows in under 10 minutes. For detailed setup instructions, see [DEVELOPMENT_SETUP.md](DEVELOPMENT_SETUP.md).

## Prerequisites

**System Requirements:**
- CPU: 2+ cores, RAM: 8+ GB, Storage: 4+ GB
- macOS 11+, Ubuntu 20.04+, or Windows 11+ (WSL2)

**Required Software:**
- Rust 1.75+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- PostgreSQL 15+ (running on port 5432)
- Python 3.11+ with `uv` (`curl -LsSf https://astral.sh/uv/install.sh | sh`)
- Docker + Docker Compose (for Dgraph and monitoring stack)
- Git 2.30+

## 2-Minute Automated Setup

```bash
# Clone and enter directory
git clone <repository-url>
cd ai-system-rust

# One-command setup (installs everything)
chmod +x scripts/setup.sh && ./scripts/setup.sh

# ✓ Installs Rust, PostgreSQL, Python, uv
# ✓ Sets up database with schema
# ✓ Configures environment variables
# ✓ Builds main application + microservices
# ✓ Installs Python MCP server dependencies
# ✓ Creates development helper scripts
# ✓ Validates complete setup
```

## Manual Setup (5 minutes)

### 1. Prerequisites
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install uv (Python package manager)
curl -LsSf https://astral.sh/uv/install.sh | sh

# Install PostgreSQL (platform-specific)
# macOS: brew install postgresql && brew services start postgresql
# Ubuntu: sudo apt install postgresql postgresql-contrib
```

### 2. Database Setup
```bash
# PostgreSQL setup (assumes postgres is running)
sudo -u postgres createuser -s aiworkflow
sudo -u postgres createdb -O aiworkflow ai_workflow
psql -U aiworkflow -d ai_workflow -f scripts/init-db.sql

# Start Dgraph (for knowledge graph service)
cd services/knowledge_graph/dgraph && docker-compose up -d && cd ../../..
```

### 3. Environment and Build
```bash
cp .env.example .env
# Edit .env with your DATABASE_URL and JWT_SECRET

# Build everything
cargo build                              # Main application
cd scripts && uv sync && cd ..           # MCP servers
for service in content_processing knowledge_graph realtime_communication; do
    cd services/$service && cargo build && cd ../..
done
```

### 4. Start Services
```bash
# Option A: Full Docker stack (recommended)
docker-compose up -d

# Option B: Local development
./scripts/start_test_servers.sh &  # MCP servers
cargo run --bin backend            # Main application

# Option C: Individual microservices
# cd services/content_processing && cargo run &
# cd services/knowledge_graph && cargo run &
# cd services/realtime_communication && cargo run &
```

## Verification

```bash
# Health check (main application)
curl http://localhost:8080/api/v1/health
# Expected: {"status":"healthy","timestamp":"..."}

# Detailed health (all components)
curl http://localhost:8080/api/v1/health/detailed

# API documentation
open http://localhost:8080/swagger-ui/

# Run tests
cargo test                    # Unit tests
cargo test -- --ignored      # Integration tests (requires MCP servers)

# Check microservices (if running locally)
curl http://localhost:8081/health  # Content Processing
curl http://localhost:8082/health  # Knowledge Graph
curl http://localhost:8083/health  # Realtime Communication
```

## Key Services

| Service | URL | Purpose |
|---------|-----|---------|
| **Main API** | http://localhost:8080 | REST API + Workflow Engine |
| **Swagger UI** | http://localhost:8080/swagger-ui/ | Interactive API Documentation |
| **PostgreSQL** | localhost:5432 | Primary Database + Event Store |
| **Content Processing** | http://localhost:8081 | Document Analysis & AI Integration |
| **Knowledge Graph** | http://localhost:8082 | Graph Database & Algorithms |
| **Realtime Communication** | http://localhost:8083 | WebSocket & Actor System |
| **Dgraph UI** | http://localhost:8000 | Graph Database Interface |
| **Prometheus** | http://localhost:9090 | Metrics Collection |
| **Grafana** | http://localhost:3000 | Monitoring Dashboards (admin/admin) |
| **Jaeger** | http://localhost:16686 | Distributed Tracing |

## Essential Commands

```bash
# Development (auto-reload)
cargo watch -x "run --bin backend"  # Main app with auto-restart
./scripts/start_test_servers.sh     # Start MCP servers
cargo fmt && cargo clippy           # Format and lint code

# Testing
cargo test                               # Unit tests
cargo test -- --ignored                 # Integration tests
cargo test --test end_to_end_workflow_test  # E2E scenarios
cargo test --test load_test -- --ignored   # Performance tests

# Microservices
cd services/content_processing && cargo run     # Document analysis
cd services/knowledge_graph && cargo run        # Graph operations
cd services/realtime_communication && cargo run # WebSocket server

# Database Operations
psql $DATABASE_URL                    # Connect to main DB
./scripts/database-setup.sh           # Reset PostgreSQL
cd services/knowledge_graph/dgraph && docker-compose up -d  # Start Dgraph

# Docker Stack
docker-compose up -d                  # Start everything
docker-compose logs -f ai-workflow-system  # View main app logs
docker-compose down                   # Stop all services
docker-compose down -v                # Stop and remove data

# Helper Scripts
./dev.sh start    # Start development servers
./dev.sh test     # Run all tests
./dev.sh logs     # Tail logs
./dev.sh clean    # Clean builds
```

## Architecture Overview

### Core System
- **Main API Server**: Actix-web REST API with JWT auth, rate limiting, OpenAPI docs
- **Workflow Engine**: Node-based workflow execution with type safety
- **Event Store**: PostgreSQL-backed event sourcing with projections
- **MCP Framework**: Model Context Protocol for AI service integration

### Microservices
- **Content Processing**: Document analysis, AI integration, WASM plugins (SQLx + PostgreSQL)
- **Knowledge Graph**: Graph algorithms, Dgraph integration, complex querying
- **Realtime Communication**: WebSocket server, actor model, session management

### External Integration
- **Python MCP Servers**: Notion, Slack, HelpScout integration via stdio protocol
- **AI Providers**: OpenAI, Anthropic integration with token management
- **Monitoring Stack**: Prometheus metrics, Grafana dashboards, Jaeger tracing

## Common Issues

1. **Database connection failed**: 
   ```bash
   pg_isready && ./scripts/database-setup.sh
   ```

2. **Compilation errors**: 
   ```bash
   # macOS: xcode-select --install
   # Ubuntu: sudo apt install build-essential pkg-config libssl-dev
   ```

3. **Port conflicts**: 
   ```bash
   lsof -i :8080  # Find what's using the port
   # Or change PORT=8081 in .env
   ```

4. **MCP server failures**: 
   ```bash
   cd scripts && uv sync && cd ..
   ./scripts/start_test_servers.sh
   ```

5. **Dgraph connection issues**: 
   ```bash
   cd services/knowledge_graph/dgraph
   docker-compose up -d
   curl http://localhost:8080/health
   ```

6. **Microservice build failures**: 
   ```bash
   # Build individually to isolate issues
   cd services/content_processing && cargo check
   ```

## Next Steps

### Explore the System
- **API Playground**: http://localhost:8080/swagger-ui/
- **Monitor Performance**: http://localhost:3000 (Grafana dashboards)
- **View Traces**: http://localhost:16686 (Jaeger)
- **Query Knowledge Graph**: http://localhost:8000 (Dgraph Ratel)

### Run Examples
```bash
# Rust examples
cargo run --example basic-workflow
cargo run --example knowledge_base_example
cargo run --example ai-research-workflow

# Python client examples
cd examples/python_client
python ai_workflow_client.py
python ai_tutor_service.py
```

### Learn More
- **[DEVELOPMENT_SETUP.md](DEVELOPMENT_SETUP.md)**: Comprehensive setup guide
- **[CLAUDE.md](CLAUDE.md)**: Architecture and development guidelines
- **[docs/tutorials/](docs/tutorials/)**: Step-by-step learning guides
- **Service READMEs**: Documentation for each microservice

### Development Workflow
```bash
# Start development environment
./dev.sh start

# Make changes, tests run automatically with:
cargo watch -x test

# Before committing:
cargo fmt && cargo clippy
cargo test -- --ignored  # Run integration tests
```

---

**Need help?** 
1. Run `./scripts/validate-environment.sh` to diagnose issues
2. Check service health: `curl http://localhost:8080/api/v1/health/detailed`
3. View logs: `./dev.sh logs` or `docker-compose logs -f`

**Ready to build AI workflows! 🚀**