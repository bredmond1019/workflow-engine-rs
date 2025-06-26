# Development Environment Setup Guide

## System Requirements

### Operating System
- **macOS**: 12.0 or later
- **Linux**: Ubuntu 20.04+ or equivalent
- **Windows**: Windows 10/11 with WSL2

### Hardware Requirements
- **RAM**: Minimum 8GB, recommended 16GB
- **Storage**: At least 10GB free space
- **CPU**: x86_64 or ARM64 architecture

## Required Software

### 1. Rust Toolchain (Required)

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the prompts and then reload your shell
source $HOME/.cargo/env

# Verify installation and ensure version 1.75+
rustc --version
cargo --version

# Install required components
rustup component add rustfmt clippy
```

### 2. PostgreSQL (Required)

#### macOS
```bash
brew install postgresql@14
brew services start postgresql@14
```

#### Linux
```bash
sudo apt update
sudo apt install postgresql postgresql-contrib
sudo systemctl start postgresql
```

#### Windows (WSL2)
```bash
sudo apt update
sudo apt install postgresql postgresql-contrib
sudo service postgresql start
```

### 3. Python Environment (Required for MCP servers)

```bash
# Install Python 3.8 or later
# macOS
brew install python@3.11

# Linux/WSL2
sudo apt install python3.11 python3.11-pip python3.11-venv

# Create virtual environment
python3 -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install uv for faster package management (optional)
pip install uv
```

### 4. Docker & Docker Compose (Recommended)

#### macOS
- Download and install Docker Desktop from https://www.docker.com/products/docker-desktop

#### Linux
```bash
# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sh get-docker.sh

# Install Docker Compose
sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose
```

### 5. Development Tools

```bash
# Install Diesel CLI for database migrations
cargo install diesel_cli --no-default-features --features postgres

# Install SQLx CLI for service databases
cargo install sqlx-cli --no-default-features --features postgres,sqlite

# Install useful development tools
cargo install cargo-watch   # Auto-recompilation
cargo install cargo-audit   # Security vulnerability scanning
cargo install cargo-expand  # Macro expansion debugging
cargo install cargo-edit    # Cargo.toml management
```

## Database Setup

### 1. Main Application Database

```bash
# Create PostgreSQL user and database
sudo -u postgres createuser -P workflow_user  # Enter password when prompted
sudo -u postgres createdb -O workflow_user ai_workflow_db

# Test connection
psql -U workflow_user -d ai_workflow_db -h localhost

# Set environment variable
export DATABASE_URL="postgresql://workflow_user:your_password@localhost/ai_workflow_db"

# Run migrations
diesel setup
diesel migration run
```

### 2. Service-Specific Databases

#### Content Processing Service (SQLite)
```bash
cd services/content_processing
sqlx database create
sqlx migrate run
```

#### Knowledge Graph Service (Dgraph)
```bash
cd services/knowledge_graph/dgraph
docker-compose up -d

# Verify Dgraph is running
curl http://localhost:8080/health
```

## Environment Configuration

### 1. Create .env File

```bash
# In project root
cat > .env << EOF
# Database
DATABASE_URL=postgresql://workflow_user:your_password@localhost/ai_workflow_db

# Authentication
JWT_SECRET=$(openssl rand -base64 32)

# AI Providers (optional)
OPENAI_API_KEY=your_openai_key_here
ANTHROPIC_API_KEY=your_anthropic_key_here

# AWS (optional, for Bedrock)
AWS_ACCESS_KEY_ID=your_aws_key
AWS_SECRET_ACCESS_KEY=your_aws_secret
AWS_REGION=us-east-1

# Monitoring (optional)
PROMETHEUS_PORT=9090
GRAFANA_PORT=3000

# Service URLs
CONTENT_PROCESSING_URL=http://localhost:3001
KNOWLEDGE_GRAPH_URL=http://localhost:3002
REALTIME_COMM_URL=http://localhost:3003
EOF
```

### 2. VS Code Setup (Recommended)

```bash
# Install recommended extensions
code --install-extension rust-lang.rust-analyzer
code --install-extension vadimcn.vscode-lldb
code --install-extension serayuzgur.crates
code --install-extension tamasfe.even-better-toml
```

Create `.vscode/settings.json`:
```json
{
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.checkOnSave.command": "clippy",
    "editor.formatOnSave": true,
    "[rust]": {
        "editor.defaultFormatter": "rust-lang.rust-analyzer"
    }
}
```

## Building and Running

### 1. Initial Build

```bash
# Clone repository (if not already done)
git clone https://github.com/bredmond1019/workflow-engine-rs.git
cd workflow-engine-rs

# Build all components
cargo build

# Run tests to verify setup
cargo test
```

### 2. Running the Application

#### Option A: Direct Execution
```bash
# Run the main application
cargo run --bin workflow-engine

# Or with auto-reload during development
cargo watch -x 'run --bin workflow-engine'
```

#### Option B: Docker Compose (Recommended)
```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

### 3. Running Individual Services

```bash
# Terminal 1: Content Processing Service
cd services/content_processing && cargo run

# Terminal 2: Knowledge Graph Service
cd services/knowledge_graph && cargo run

# Terminal 3: Realtime Communication Service
cd services/realtime_communication && cargo run

# Terminal 4: Main API
cargo run --bin workflow-engine
```

## Testing

### 1. Unit Tests

```bash
# Run all unit tests
cargo test

# Run tests for specific crate
cargo test -p workflow-engine-core

# Run with output
cargo test -- --nocapture
```

### 2. Integration Tests

```bash
# Start test MCP servers first
./scripts/start_test_servers.sh

# Run integration tests
cargo test -- --ignored

# Run specific integration test suites
cargo test --test end_to_end_workflow_test -- --ignored
cargo test --test mcp_communication_test -- --ignored
cargo test --test workflow_external_tools_test -- --ignored
```

### 3. Load and Chaos Tests

```bash
# Run load tests
cargo test --test load_test -- --ignored --nocapture

# Run chaos tests (requires all services running)
cargo test --test chaos_test -- --ignored --nocapture
```

## Debugging

### 1. Enable Debug Logging

```bash
# Set log level
export RUST_LOG=debug
export RUST_BACKTRACE=1

# Or for specific modules
export RUST_LOG=workflow_engine_core=debug,workflow_engine_api=info
```

### 2. Using LLDB

```bash
# Debug with LLDB
rust-lldb target/debug/workflow-engine

# In LLDB
(lldb) breakpoint set --name main
(lldb) run
(lldb) continue
```

### 3. Health Checks

```bash
# Basic health check
curl http://localhost:8080/health

# Detailed health check
curl http://localhost:8080/health/detailed | jq .
```

### 4. Monitoring URLs

- **API Documentation**: http://localhost:8080/swagger-ui/
- **Prometheus Metrics**: http://localhost:9090
- **Grafana Dashboards**: http://localhost:3000 (admin/admin)
- **Jaeger Tracing**: http://localhost:16686 (if enabled)

## Common Development Tasks

### 1. Adding a New API Endpoint

1. Create route handler in `crates/workflow-engine-api/src/api/routes/`
2. Add to router in `crates/workflow-engine-api/src/api/mod.rs`
3. Update OpenAPI spec if needed
4. Add tests in same file

### 2. Creating a New Workflow Node

1. Implement node in `crates/workflow-engine-nodes/src/`
2. Implement `WorkflowNode` trait
3. Register in `NodeRegistry`
4. Add tests

### 3. Database Migrations

```bash
# Create new migration
diesel migration generate my_migration_name

# Edit migrations in migrations/*/up.sql and down.sql

# Run migrations
diesel migration run

# Revert last migration
diesel migration revert
```

### 4. Adding Dependencies

```bash
# Add to specific crate
cd crates/workflow-engine-core
cargo add tokio --features full

# Add dev dependency
cargo add --dev mockall
```

## Troubleshooting

### PostgreSQL Connection Issues

```bash
# Check PostgreSQL is running
pg_isready

# Check connection
psql -U workflow_user -d ai_workflow_db -h localhost

# Reset database
diesel database reset
```

### Port Conflicts

```bash
# Find process using port
lsof -i :8080  # macOS/Linux
netstat -ano | findstr :8080  # Windows

# Kill process
kill -9 <PID>
```

### Compilation Errors

```bash
# Clean build
cargo clean
cargo build

# Update dependencies
cargo update

# Check for outdated dependencies
cargo outdated
```

### Test Failures

```bash
# Run single test with output
cargo test test_name -- --nocapture

# Check test database
psql -U workflow_user -d ai_workflow_db_test -h localhost
```

## Code Style and Quality

### Pre-commit Checks

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Check for security vulnerabilities
cargo audit

# Run all checks
cargo fmt && cargo clippy -- -D warnings && cargo test
```

### Git Hooks (Optional)

Create `.git/hooks/pre-commit`:
```bash
#!/bin/sh
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
```

## Next Steps

1. Review [QUICK_START.md](QUICK_START.md) for usage examples
2. Check [CLAUDE.md](CLAUDE.md) for AI assistant guidance
3. Explore [examples/](examples/) directory for sample workflows
4. Join our Discord/Slack for community support

## Getting Help

- **Documentation**: See `/docs` directory
- **Issues**: https://github.com/bredmond1019/workflow-engine-rs/issues
- **Discussions**: https://github.com/bredmond1019/workflow-engine-rs/discussions

Happy coding! 🚀