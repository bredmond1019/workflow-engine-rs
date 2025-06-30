# AI Workflow Engine - Quick Start Guide

This guide helps you get started with the AI Workflow Engine crates in just a few minutes. Choose your path:

- **[📦 Using the Crates](#-using-the-crates)** - Add to your Rust project (5 minutes)
- **[🔧 Local Development](#-local-development)** - Full system setup (10 minutes)

## 📦 Using the Crates

The fastest way to add AI workflows to your existing Rust project.

### 1. Add Dependencies

```toml
[dependencies]
# Core workflow engine
workflow-engine-core = "0.6.0"

# Optional: MCP protocol support
workflow-engine-mcp = { version = "0.6.0", optional = true }

# Optional: Built-in nodes
workflow-engine-nodes = { version = "0.6.0", optional = true }

# Optional: API server
workflow-engine-api = { version = "0.6.0", optional = true }

# Core dependencies you'll need
tokio = { version = "1.0", features = ["full"] }
serde_json = "1.0"
async-trait = "0.1"
```

### 2. Hello World Workflow

```rust
use workflow_engine_core::prelude::*;
use serde_json::json;

#[derive(Debug)]
struct GreetingNode;

impl Node for GreetingNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let input: serde_json::Value = context.get_event_data()?;
        let name = input.get("name").and_then(|v| v.as_str()).unwrap_or("World");
        
        context.update_node("greeting", json!({
            "message": format!("Hello, {}!", name)
        }));
        
        Ok(context)
    }
}

#[tokio::main]
async fn main() -> Result<(), WorkflowError> {
    // Build workflow
    let workflow = TypedWorkflowBuilder::new("hello_workflow")
        .start_with_node(NodeId::new("greeting"))
        .build()?;
    
    // Register node
    workflow.register_node(NodeId::new("greeting"), GreetingNode);
    
    // Run workflow
    let result = workflow.run(json!({"name": "Alice"})).await?;
    
    if let Some(greeting) = result.get_node_data::<serde_json::Value>("greeting")? {
        println!("{}", greeting["message"]); // "Hello, Alice!"
    }
    
    Ok(())
}
```

### 3. Async Nodes with External APIs

```rust
use workflow_engine_core::prelude::*;
use async_trait::async_trait;

#[derive(Debug)]
struct ApiCallNode {
    base_url: String,
}

#[async_trait]
impl AsyncNode for ApiCallNode {
    async fn process_async(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let input: serde_json::Value = context.get_event_data()?;
        
        // Make HTTP request
        let response = reqwest::get(&format!("{}/api/data", self.base_url))
            .await
            .map_err(|e| WorkflowError::ProcessingError { 
                message: e.to_string() 
            })?;
            
        let data: serde_json::Value = response.json().await
            .map_err(|e| WorkflowError::ProcessingError { 
                message: e.to_string() 
            })?;
        
        context.update_node("api_response", data);
        Ok(context)
    }
}

// Use in async workflow
let workflow = TypedWorkflowBuilder::new("api_workflow")
    .start_with_node(NodeId::new("api_call"))
    .build()?;

workflow.register_async_node(
    NodeId::new("api_call"), 
    ApiCallNode { base_url: "https://api.example.com".to_string() }
);

let result = workflow.run_async(json!({})).await?;
```

### 4. Feature Flags

Enable optional functionality:

```toml
[dependencies]
workflow-engine-core = { version = "0.6.0", features = ["database", "monitoring"] }
workflow-engine-mcp = { version = "0.6.0", features = ["websocket", "stdio"] }
workflow-engine-nodes = { version = "0.6.0", features = ["ai-agents", "external-mcp"] }
workflow-engine-api = { version = "0.6.0", features = ["auth", "openapi"] }
```

**See [Feature Flags](#feature-flags) section below for complete list.**

### 5. Complete Examples

Check out the examples directory:

```bash
# Clone the repository for examples
git clone https://github.com/bredmond1019/workflow-engine-rs
cd workflow-engine-rs

# Run examples
cargo run --example 01_hello_world_workflow
cargo run --example 02_async_external_api_workflow
cargo run --example 03_custom_node_implementation
cargo run --example 04_error_handling_best_practices
```

---

## 🔧 Local Development

**For experienced developers who want to get the full system running quickly.**

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

---

## Feature Flags

### workflow-engine-core

Controls core functionality of the workflow engine:

```toml
[dependencies]
workflow-engine-core = { version = "0.6.0", features = ["database", "monitoring", "aws", "full"] }
```

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `database` | Database integration with Diesel ORM | `diesel` |
| `monitoring` | Prometheus metrics collection | `prometheus`, `lazy_static` |
| `aws` | AWS Bedrock AI integration | `aws-config`, `aws-sdk-bedrockruntime` |
| `full` | Enables all optional features | All above |

**Default:** None (minimal core functionality)

### workflow-engine-mcp

Controls MCP (Model Context Protocol) transport types:

```toml
[dependencies]
workflow-engine-mcp = { version = "0.6.0", features = ["http", "websocket", "stdio", "all"] }
```

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `http` | HTTP transport for MCP clients | `reqwest` |
| `websocket` | WebSocket transport for MCP clients | `tokio-tungstenite` |
| `stdio` | Standard I/O transport for MCP clients | `tokio-util` |
| `all` | All transport types | All above |

**Default:** `http`, `websocket`

### workflow-engine-nodes

Controls built-in node implementations:

```toml
[dependencies]
workflow-engine-nodes = { version = "0.6.0", features = ["ai-agents", "external-mcp", "research", "template", "all"] }
```

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `ai-agents` | AI service integration nodes (OpenAI, Anthropic) | AI provider SDKs |
| `external-mcp` | External MCP server integration nodes | `workflow-engine-mcp` |
| `research` | Research and analysis nodes | Text processing libraries |
| `template` | Template processing nodes | `handlebars` |
| `all` | All node types | All above |

**Default:** `ai-agents`, `external-mcp`

### workflow-engine-api

Controls API server features:

```toml
[dependencies]
workflow-engine-api = { version = "0.6.0", features = ["openapi", "auth", "monitoring", "database"] }
```

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `openapi` | OpenAPI documentation generation | `utoipa`, `utoipa-swagger-ui` |
| `auth` | JWT authentication support | `jsonwebtoken` |
| `monitoring` | Prometheus metrics endpoints | `prometheus` |
| `database` | Database integration | `diesel` |

**Default:** `openapi`, `auth`, `monitoring`

### Common Feature Combinations

**Minimal Setup** (just core workflow engine):
```toml
workflow-engine-core = "0.6.0"
```

**AI-Powered Workflows** (core + AI nodes):
```toml
workflow-engine-core = "0.6.0"
workflow-engine-nodes = { version = "0.6.0", features = ["ai-agents"] }
```

**Full MCP Integration** (core + MCP + all transports):
```toml
workflow-engine-core = "0.6.0"
workflow-engine-mcp = { version = "0.6.0", features = ["all"] }
workflow-engine-nodes = { version = "0.6.0", features = ["external-mcp"] }
```

**Complete API Server** (everything for web service):
```toml
workflow-engine-core = { version = "0.6.0", features = ["full"] }
workflow-engine-mcp = { version = "0.6.0", features = ["all"] }
workflow-engine-nodes = { version = "0.6.0", features = ["all"] }
workflow-engine-api = { version = "0.6.0", features = ["openapi", "auth", "monitoring", "database"] }
```

---

## Troubleshooting

### Common Build Issues

**1. Missing System Dependencies**
```bash
# macOS
xcode-select --install
brew install openssl

# Ubuntu/Debian
sudo apt update
sudo apt install build-essential pkg-config libssl-dev

# Alpine Linux
apk add build-base openssl-dev
```

**2. Diesel Database Errors**
```bash
# Install diesel CLI
cargo install diesel_cli --no-default-features --features postgres

# Set database URL
export DATABASE_URL=postgresql://username:password@localhost/database_name

# Run migrations
diesel migration run
```

**3. Feature Flag Conflicts**
```toml
# Avoid conflicting features - use specific combinations
workflow-engine-core = { version = "0.6.0", features = ["database"] }
# Not: features = ["database", "full"] - full already includes database
```

**4. Async Runtime Issues**
```rust
// Ensure you have tokio runtime
[dependencies]
tokio = { version = "1.0", features = ["full"] }

// In main.rs
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Your code here
    Ok(())
}
```

### Performance Optimization

**1. Minimal Feature Set**
Only enable features you need:
```toml
# Instead of "all", be specific
workflow-engine-nodes = { version = "0.6.0", features = ["ai-agents"] }
```

**2. Compile Time Optimization**
```toml
# In Cargo.toml
[profile.dev]
opt-level = 1  # Faster debug builds

[profile.release]
lto = true     # Link-time optimization
codegen-units = 1  # Better optimization
```

**3. Runtime Performance**
```rust
// Use async nodes for I/O operations
#[async_trait]
impl AsyncNode for MyNode {
    async fn process_async(&self, context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Non-blocking operations
        Ok(context)
    }
}
```

### Documentation and Examples

**1. View Local Documentation**
```bash
# Generate and open docs
cargo doc --open --all-features

# Specific crate docs
cargo doc -p workflow-engine-core --open
```

**2. Run Examples**
```bash
# List available examples
cargo run --example

# Run specific example
cargo run --example 01_hello_world_workflow

# Run with features
cargo run --example ai_integration --features "ai-agents"
```

**3. Testing Your Integration**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_my_workflow() {
        let workflow = TypedWorkflowBuilder::new("test")
            .start_with_node(NodeId::new("test_node"))
            .build()
            .unwrap();
            
        workflow.register_node(NodeId::new("test_node"), MyTestNode);
        
        let result = workflow.run(json!({"test": "data"})).await.unwrap();
        assert!(result.get_node_data::<serde_json::Value>("test_result").unwrap().is_some());
    }
}
```

### Getting Help

1. **Check Documentation**: `cargo doc --open --all-features`
2. **Run Examples**: Examples in the repository demonstrate patterns
3. **Enable Logging**: Use `RUST_LOG=debug` for detailed output
4. **Check Feature Flags**: Ensure you have the right features enabled
5. **Review Tests**: Integration tests show real usage patterns

**Ready to build AI workflows! 🚀**