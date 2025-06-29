# AI Workflow Engine - Quick Start Guide

Get up and running with the AI Workflow Engine in under 10 minutes! This guide covers everything you need to start building AI-powered workflows with GraphQL Federation support.

## 🚀 Quick Setup Options

Choose your path:
1. **[5-Minute Quick Start](#5-minute-quick-start)** - Get the system running fast
2. **[Using as a Rust Library](#using-as-a-rust-library)** - Add to your existing project
3. **[Full Development Setup](#full-development-setup)** - Complete environment with all features

---

## 🎯 5-Minute Quick Start

### Prerequisites

**System Requirements:**
- CPU: 2+ cores, RAM: 8+ GB, Storage: 4+ GB
- macOS 11+, Ubuntu 20.04+, or Windows 11+ (WSL2)

**Required Software:**
```bash
# Check if you have these installed
rustc --version    # Need: 1.75+
node --version     # Need: 18+
psql --version     # Need: PostgreSQL 15+
docker --version   # Need: Docker 20+
python --version   # Need: Python 3.11+
```

### Step 1: Clone and Setup

```bash
# Clone repository
git clone <repository-url>
cd workflow-engine-rs

# Switch to GraphQL Federation branch
git checkout graphql-federation

# Install uv (fast Python package manager)
curl -LsSf https://astral.sh/uv/install.sh | sh

# Setup database
createdb ai_workflow_db
psql ai_workflow_db < scripts/init-db.sql

# Copy environment file
cp .env.example .env
# Edit .env with your DATABASE_URL and JWT_SECRET
```

### Step 2: Start Services with Docker

```bash
# Start all services (recommended)
docker-compose up -d

# Wait for services to be ready (about 30 seconds)
sleep 30

# Verify services are running
docker-compose ps
```

### Step 3: Start Frontend (Optional)

```bash
# In a new terminal
cd frontend
npm install
npm run dev
```

### Step 4: Verify Your Setup ✅

Run the verification script:
```bash
./scripts/verify-setup.sh
```

Or manually check each service:

| Service | URL | Expected Response |
|---------|-----|-------------------|
| **Main API** | http://localhost:8080/health | `{"status":"healthy"}` |
| **GraphQL Gateway** | http://localhost:4000/graphql | GraphQL Playground UI |
| **Frontend** | http://localhost:5173 | React Application |
| **Swagger UI** | http://localhost:8080/swagger-ui/ | API Documentation |

---

## 📦 Using as a Rust Library

Add AI workflow capabilities to your existing Rust project:

### Basic Setup

```toml
[dependencies]
# Core workflow engine
workflow-engine-core = "0.6.0"
workflow-engine-nodes = "0.6.0"

# Required async runtime
tokio = { version = "1.0", features = ["full"] }
serde_json = "1.0"
async-trait = "0.1"
```

### Hello World Example

```rust
use workflow_engine_core::prelude::*;
use workflow_engine_nodes::prelude::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), WorkflowError> {
    // Create a simple workflow
    let workflow = TypedWorkflowBuilder::new("hello_workflow")
        .start_with_node(NodeId::new("greeting"))
        .build()?;
    
    // Register a greeting node
    workflow.register_node(
        NodeId::new("greeting"), 
        GreetingNode::new("Hello from AI Workflow!")
    );
    
    // Run the workflow
    let result = workflow.run(json!({"name": "Developer"})).await?;
    println!("Workflow completed: {:?}", result);
    
    Ok(())
}
```

### AI-Powered Workflow Example

```rust
use workflow_engine_nodes::ai_agents::{OpenAIAgent, AgentConfig};

#[tokio::main]
async fn main() -> Result<(), WorkflowError> {
    // Configure AI agent
    let ai_config = AgentConfig {
        api_key: std::env::var("OPENAI_API_KEY")?,
        model: "gpt-4".to_string(),
        ..Default::default()
    };
    
    // Build AI workflow
    let workflow = TypedWorkflowBuilder::new("ai_analysis")
        .start_with_node(NodeId::new("analyze"))
        .build()?;
    
    // Register AI node
    workflow.register_async_node(
        NodeId::new("analyze"),
        OpenAIAgent::new(ai_config)
    );
    
    // Run with input
    let result = workflow.run_async(json!({
        "prompt": "Analyze this customer feedback and suggest improvements"
    })).await?;
    
    Ok(())
}
```

---

## 🔧 Full Development Setup

### Automated Setup (Recommended)

```bash
# One-command setup
chmod +x scripts/setup.sh && ./scripts/setup.sh

# This will:
# ✓ Install all dependencies
# ✓ Setup databases
# ✓ Build all services
# ✓ Configure environment
# ✓ Start MCP servers
# ✓ Verify installation
```

### Manual Setup

#### 1. Install Dependencies

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js (for frontend)
curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash -
sudo apt-get install -y nodejs

# PostgreSQL
# macOS
brew install postgresql && brew services start postgresql

# Ubuntu
sudo apt install postgresql postgresql-contrib
sudo systemctl start postgresql

# Python with uv
curl -LsSf https://astral.sh/uv/install.sh | sh
```

#### 2. Database Setup

```bash
# Create database and user
sudo -u postgres createuser -s aiworkflow
sudo -u postgres createdb -O aiworkflow ai_workflow_db

# Run initial schema
psql -U aiworkflow -d ai_workflow_db -f scripts/init-db.sql

# Start Dgraph (for Knowledge Graph service)
cd services/knowledge_graph/dgraph
docker-compose up -d
cd ../../..
```

#### 3. Build Everything

```bash
# Build main application
cargo build --release

# Build GraphQL Gateway
cargo build --bin graphql-gateway --release

# Build microservices
for service in content_processing knowledge_graph realtime_communication; do
    cd services/$service && cargo build --release && cd ../..
done

# Setup Python MCP servers
cd scripts && uv sync && cd ..

# Build frontend
cd frontend && npm install && npm run build && cd ..
```

#### 4. Start Services

```bash
# Option A: Use provided start script
./scripts/start-all-services.sh

# Option B: Start individually
# Terminal 1: Main API
cargo run --bin workflow-engine

# Terminal 2: GraphQL Gateway
cargo run --bin graphql-gateway

# Terminal 3: MCP Servers
./scripts/start_test_servers.sh

# Terminal 4: Frontend
cd frontend && npm run dev
```

---

## 🧪 Running Tests

### Quick Test Suite

```bash
# Run all unit tests
cargo test

# Run frontend tests (174+ tests)
cd frontend && npm test

# Visual test dashboard
./test-dashboard.sh
open frontend/test-dashboard/index.html
```

### Integration Tests

```bash
# Setup test environment
./scripts/setup-test-environment.sh

# Run integration tests
cargo test -- --ignored

# GraphQL Federation tests
./validate_federation.sh
cargo run --example federated_query
```

### Test Categories

| Test Type | Command | Description |
|-----------|---------|-------------|
| Unit Tests | `cargo test` | Fast, isolated tests |
| Frontend Tests | `cd frontend && npm test` | React component tests |
| Integration Tests | `cargo test -- --ignored` | External service tests |
| Federation Tests | `./validate_federation.sh` | GraphQL gateway tests |
| Load Tests | `cargo test --test load_test -- --ignored` | Performance tests |
| E2E Tests | `cargo test --test end_to_end_workflow_test -- --ignored` | Full workflow tests |

---

## 📍 Service Map

### Core Services

| Service | Port | Purpose | Health Check |
|---------|------|---------|--------------|
| **Main API** | 8080 | REST API + Workflow Engine | `/health` |
| **GraphQL Gateway** | 4000 | Federated GraphQL endpoint | `/health` |
| **PostgreSQL** | 5432 | Primary database | `pg_isready` |
| **Frontend** | 5173 | React UI (dev mode) | Browser |

### Microservices

| Service | Port | Purpose | Health Check |
|---------|------|---------|--------------|
| **Content Processing** | 8082 | Document analysis & AI | `/health` |
| **Knowledge Graph** | 3002 | Graph database operations | `/health` |
| **Realtime Communication** | 8081 | WebSocket & messaging | `/health` |

### MCP Servers (Python)

| Service | Type | Purpose |
|---------|------|---------|
| **HelpScout** | stdio | Customer support integration |
| **Notion** | stdio | Knowledge base integration |
| **Slack** | stdio | Team communication |
| **Customer Support** | stdio | Unified support workflows |

### Monitoring Stack

| Service | Port | Purpose | Credentials |
|---------|------|---------|-------------|
| **Grafana** | 3000 | Metrics dashboards | admin/admin |
| **Prometheus** | 9090 | Metrics collection | None |
| **Jaeger** | 16686 | Distributed tracing | None |
| **Redis** | 6379 | Caching layer | redis123 |

---

## 🔍 Verify Your Setup

### Automated Verification

Create and run this verification script:

```bash
#!/bin/bash
# Save as scripts/verify-setup.sh

echo "🔍 Verifying AI Workflow Engine Setup..."

# Check core services
echo "✓ Checking core services..."
curl -sf http://localhost:8080/health || echo "❌ Main API not responding"
curl -sf http://localhost:4000/health || echo "❌ GraphQL Gateway not responding"
curl -sf http://localhost:5173 || echo "❌ Frontend not responding"

# Check microservices
echo "✓ Checking microservices..."
curl -sf http://localhost:8082/health || echo "❌ Content Processing not responding"
curl -sf http://localhost:3002/health || echo "❌ Knowledge Graph not responding"
curl -sf http://localhost:8081/health || echo "❌ Realtime Communication not responding"

# Check database
echo "✓ Checking database..."
psql -h localhost -U aiworkflow -d ai_workflow_db -c "SELECT 1" || echo "❌ Database not accessible"

# Check monitoring
echo "✓ Checking monitoring stack..."
curl -sf http://localhost:3000 || echo "❌ Grafana not responding"
curl -sf http://localhost:9090 || echo "❌ Prometheus not responding"

echo "✅ Verification complete!"
```

### Manual Verification Checklist

- [ ] **Main API**: `curl http://localhost:8080/api/v1/health`
- [ ] **GraphQL Playground**: Open http://localhost:4000/graphql
- [ ] **Frontend**: Open http://localhost:5173
- [ ] **Swagger Docs**: Open http://localhost:8080/swagger-ui/
- [ ] **GraphQL Query**: Run test query in playground:
  ```graphql
  {
    workflows {
      id
      name
      status
    }
  }
  ```

---

## 🔧 Common Issues & Solutions

### Issue: Port Already in Use

```bash
# Find what's using port 8080
lsof -i :8080

# Kill process or change port in .env
PORT=8081 cargo run --bin workflow-engine
```

### Issue: Database Connection Failed

```bash
# Check PostgreSQL is running
pg_isready

# Check connection
psql -h localhost -U aiworkflow -d ai_workflow_db

# Reset database if needed
./scripts/database-setup.sh
```

### Issue: MCP Servers Not Starting

```bash
# Ensure Python and uv are installed
python --version  # Should be 3.11+
uv --version

# Reinstall dependencies
cd scripts && uv sync && cd ..

# Start servers manually
cd scripts && uv run python customer_support_server.py
```

### Issue: Frontend Build Errors

```bash
# Clear cache and reinstall
cd frontend
rm -rf node_modules package-lock.json
npm install
npm run dev
```

### Issue: Docker Services Failing

```bash
# Check Docker is running
docker info

# Reset Docker stack
docker-compose down -v
docker-compose up -d

# Check logs
docker-compose logs -f [service-name]
```

---

## 🎓 Next Steps

### 1. Explore the UI
- Open http://localhost:5173 for the chat-based workflow builder
- Try creating workflows through natural language
- Test the 174+ component tests: `cd frontend && npm test`

### 2. Try Example Workflows
```bash
# Basic workflow
cargo run --example 01_hello_world_workflow

# AI-powered workflow
cargo run --example ai-research-workflow

# GraphQL Federation example
cargo run --example federated_query
```

### 3. Build Your First Workflow
```typescript
// In the frontend chat interface
"Create a customer support workflow that monitors HelpScout 
for urgent tickets and notifies the team on Slack"
```

### 4. Access Development Tools
- **API Documentation**: http://localhost:8080/swagger-ui/
- **GraphQL Playground**: http://localhost:4000/graphql
- **Metrics Dashboard**: http://localhost:3000 (admin/admin)
- **Trace Analysis**: http://localhost:16686

### 5. Read Component Documentation
- [Main API Guide](crates/workflow-engine-api/CLAUDE.md)
- [GraphQL Gateway Guide](crates/workflow-engine-gateway/README.md)
- [Frontend Development](frontend/README.md)
- [Testing Guide](frontend/USER_TESTING.md)

---

## 📚 Additional Resources

### Documentation
- **Architecture Overview**: [CLAUDE.md](CLAUDE.md)
- **Development Setup**: [DEVELOPMENT_SETUP.md](DEVELOPMENT_SETUP.md)
- **Testing Guide**: [SYSTEM_TESTING.md](SYSTEM_TESTING.md)
- **API Reference**: http://localhost:8080/swagger-ui/

### Commands Reference
```bash
# Development
./dev.sh start    # Start all services
./dev.sh test     # Run all tests
./dev.sh logs     # View logs
./dev.sh clean    # Clean build artifacts

# Testing
./test-dashboard.sh              # Visual test dashboard
./scripts/setup-test-environment.sh  # Setup test env
./validate_federation.sh         # Validate GraphQL

# Docker
docker-compose up -d             # Start stack
docker-compose logs -f           # View logs
docker-compose down -v           # Stop and clean
```

---

**🎉 Congratulations!** You now have a fully functional AI Workflow Engine with:
- ✅ GraphQL Federation support
- ✅ 174+ passing frontend tests
- ✅ Chat-based workflow builder
- ✅ Multiple AI integrations
- ✅ Real-time monitoring

Need help? Check the logs: `docker-compose logs -f` or `./dev.sh logs`