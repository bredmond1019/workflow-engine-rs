# Development Environment Setup Guide

This comprehensive guide will help you set up a complete development environment for the AI Workflow Engine project, covering Rust backend services, React/TypeScript frontend, Python MCP servers, and GraphQL Federation.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Backend Setup (Rust)](#backend-setup-rust)
3. [Frontend Setup (TypeScript/React)](#frontend-setup-typescriptreact)
4. [Python MCP Servers Setup](#python-mcp-servers-setup)
5. [IDE Configuration](#ide-configuration)
6. [Git Hooks and Pre-commit Setup](#git-hooks-and-pre-commit-setup)
7. [Database Setup and Migrations](#database-setup-and-migrations)
8. [Environment Variables Configuration](#environment-variables-configuration)
9. [Local Development Workflow](#local-development-workflow)
10. [Debugging Setup](#debugging-setup)
11. [Performance Profiling Setup](#performance-profiling-setup)
12. [Development Best Practices](#development-best-practices)
13. [Contribution Guidelines](#contribution-guidelines)
14. [Troubleshooting](#troubleshooting)

## Prerequisites

### System Requirements

- **Operating System**: macOS, Linux, or Windows with WSL2
- **Memory**: Minimum 8GB RAM (16GB recommended)
- **Storage**: At least 10GB free space

### Required Software Versions

#### Rust Toolchain
```bash
# Install Rust (1.75.0 or later)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Verify installation
rustc --version  # Should show 1.75.0 or later
cargo --version

# Install additional targets and components
rustup component add rustfmt clippy
rustup target add wasm32-unknown-unknown  # For WASM plugins
```

#### Node.js and Package Managers
```bash
# Install Node.js (18.x or later) via nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
source ~/.bashrc  # or ~/.zshrc
nvm install 18
nvm use 18

# Verify installation
node --version  # Should show v18.x.x or later
npm --version

# Optional: Install pnpm for faster package management
npm install -g pnpm
```

#### Python Environment
```bash
# Install Python (3.8 or later)
# macOS (using Homebrew)
brew install python@3.11

# Ubuntu/Debian
sudo apt update
sudo apt install python3.11 python3.11-venv python3-pip

# Verify installation
python3 --version  # Should show Python 3.8+

# Install UV for fast Python package management (optional but recommended)
curl -LsSf https://astral.sh/uv/install.sh | sh
```

#### PostgreSQL Database
```bash
# macOS (using Homebrew)
brew install postgresql@15
brew services start postgresql@15

# Ubuntu/Debian
sudo apt update
sudo apt install postgresql-15 postgresql-contrib-15
sudo systemctl start postgresql

# Windows (WSL2)
sudo apt install postgresql postgresql-contrib
sudo service postgresql start

# Verify installation
psql --version  # Should show 13.x or later
```

#### Docker and Docker Compose
```bash
# Install Docker Desktop (recommended for all platforms)
# https://docs.docker.com/get-docker/

# Or install Docker Engine (Linux)
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Install Docker Compose v2
sudo apt install docker-compose-plugin  # Ubuntu/Debian
brew install docker-compose  # macOS

# Verify installation
docker --version
docker compose version
```

#### Additional Development Tools
```bash
# Redis (for caching and session storage)
# macOS
brew install redis
brew services start redis

# Ubuntu/Debian
sudo apt install redis-server
sudo systemctl start redis

# Dgraph (for Knowledge Graph service)
docker pull dgraph/dgraph:latest

# Just (command runner)
cargo install just

# SQLx CLI (for database migrations)
cargo install sqlx-cli --no-default-features --features rustls,postgres

# Diesel CLI (for main app migrations)
cargo install diesel_cli --no-default-features --features postgres

# Development utilities
cargo install cargo-watch cargo-audit cargo-outdated
cargo install flamegraph  # For performance profiling
```

## Backend Setup (Rust)

### 1. Clone and Initial Setup

```bash
# Clone the repository
git clone <repository-url>
cd workflow-engine-rs

# Switch to the GraphQL Federation branch if needed
git checkout graphql-federation

# Install Rust dependencies
cargo build

# Run initial tests to verify setup
cargo test
```

### 2. Project Structure Overview

```
workflow-engine-rs/
├── crates/                          # Core Rust crates
│   ├── workflow-engine-api/         # Main HTTP API server
│   ├── workflow-engine-core/        # Core workflow logic
│   ├── workflow-engine-mcp/         # MCP protocol implementation
│   ├── workflow-engine-nodes/       # Workflow node implementations
│   ├── workflow-engine-app/         # Main binary
│   └── workflow-engine-gateway/     # GraphQL Federation gateway
├── services/                        # Microservices
│   ├── content_processing/          # Document analysis service
│   ├── knowledge_graph/             # Graph database service
│   └── realtime_communication/      # WebSocket messaging service
├── mcp-servers/                     # Python MCP servers
├── frontend/                        # React/TypeScript frontend
├── tests/                           # Integration tests
├── scripts/                         # Development scripts
└── monitoring/                      # Monitoring configuration
```

### 3. Building Individual Components

```bash
# Build all crates
cargo build --workspace

# Build specific crate
cargo build -p workflow-engine-api

# Build with release optimizations
cargo build --release

# Build and run specific binary
cargo run --bin workflow-engine
cargo run --bin graphql-gateway

# Build services
cd services/content_processing && cargo build
cd services/knowledge_graph && cargo build
cd services/realtime_communication && cargo build
```

### 4. Running Tests

```bash
# Run all unit tests
cargo test

# Run tests for specific crate
cargo test -p workflow-engine-core

# Run integration tests (requires external services)
./scripts/start_test_servers.sh
cargo test -- --ignored

# Run specific test suites
cargo test --test end_to_end_workflow_test -- --ignored
cargo test --test graphql_federation_integration_test -- --ignored

# Run tests with output
cargo test -- --nocapture

# Run tests in watch mode
cargo watch -x test
```

## Frontend Setup (TypeScript/React)

### 1. Initial Setup

```bash
cd frontend

# Install dependencies
npm install
# or with pnpm
pnpm install

# Verify setup
npm run type-check
npm test
```

### 2. Development Server

```bash
# Start development server with hot reload
npm run dev
# Access at http://localhost:5173

# Build for production
npm run build

# Preview production build
npm run preview

# Run tests in watch mode
npm test -- --watch

# Run tests with coverage
npm test -- --coverage

# Run E2E tests
npm run test:e2e
```

### 3. Frontend Project Structure

```
frontend/
├── src/
│   ├── api/                # API clients and GraphQL
│   ├── components/         # Reusable components
│   ├── features/          # Feature-specific modules
│   ├── hooks/             # Custom React hooks
│   ├── services/          # Business logic services
│   ├── stores/            # State management
│   └── types/             # TypeScript type definitions
├── e2e/                   # End-to-end tests
├── public/                # Static assets
└── test-dashboard/        # Visual test monitoring
```

### 4. Testing Infrastructure

```bash
# Run all tests (129+ tests)
npm test

# Run visual test dashboard
./test-dashboard.sh
# Open frontend/test-dashboard/index.html in browser

# Run specific test file
npm test -- WorkflowIntentAnalyzer.test

# Debug tests
npm test -- --verbose --no-coverage
```

## Python MCP Servers Setup

### 1. Traditional Setup (pip)

```bash
cd mcp-servers

# Create virtual environment
python3 -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install dependencies
pip install -r requirements.txt

# Run tests
python -m pytest tests/
```

### 2. Modern Setup with UV (Recommended)

```bash
cd mcp-servers

# Install UV if not already installed
curl -LsSf https://astral.sh/uv/install.sh | sh

# Create virtual environment and install dependencies (10-100x faster)
uv venv
source .venv/bin/activate
uv pip install -r requirements.txt

# Run MCP servers
uv run python -m servers.helpscout_server
uv run python -m servers.notion_server
uv run python -m servers.slack_server
```

### 3. Running MCP Servers

```bash
# Start all MCP servers at once
./scripts/start_test_servers.sh

# Or run individually:
# HelpScout server (port 8001)
python -m servers.helpscout_server

# Notion server (port 8002)
python -m servers.notion_server

# Slack server (port 8003)
python -m servers.slack_server

# Test individual server
python scripts/test_mcp_server.py --port 8001
```

## IDE Configuration

### Visual Studio Code

1. **Install Extensions**:
   ```bash
   # Install recommended extensions
   code --install-extension rust-lang.rust-analyzer
   code --install-extension tamasfe.even-better-toml
   code --install-extension vadimcn.vscode-lldb
   code --install-extension serayuzgur.crates
   code --install-extension dbaeumer.vscode-eslint
   code --install-extension esbenp.prettier-vscode
   code --install-extension ms-python.python
   code --install-extension apollographql.vscode-apollo
   ```

2. **Workspace Settings** (`.vscode/settings.json`):
   ```json
   {
     "rust-analyzer.cargo.features": "all",
     "rust-analyzer.checkOnSave.command": "clippy",
     "editor.formatOnSave": true,
     "editor.codeActionsOnSave": {
       "source.fixAll.eslint": true
     },
     "[rust]": {
       "editor.defaultFormatter": "rust-lang.rust-analyzer"
     },
     "[typescript]": {
       "editor.defaultFormatter": "esbenp.prettier-vscode"
     },
     "[typescriptreact]": {
       "editor.defaultFormatter": "esbenp.prettier-vscode"
     },
     "files.watcherExclude": {
       "**/target/**": true,
       "**/node_modules/**": true
     }
   }
   ```

3. **Debug Configuration** (`.vscode/launch.json`):
   ```json
   {
     "version": "0.2.0",
     "configurations": [
       {
         "type": "lldb",
         "request": "launch",
         "name": "Debug Workflow Engine",
         "cargo": {
           "args": ["build", "--bin=workflow-engine"],
           "filter": {
             "name": "workflow-engine",
             "kind": "bin"
           }
         },
         "args": [],
         "cwd": "${workspaceFolder}",
         "env": {
           "RUST_LOG": "debug",
           "DATABASE_URL": "postgresql://localhost/ai_workflow_db"
         }
       },
       {
         "type": "node",
         "request": "launch",
         "name": "Debug Jest Tests",
         "runtimeExecutable": "npm",
         "runtimeArgs": ["test", "--", "--runInBand", "--no-coverage"],
         "cwd": "${workspaceFolder}/frontend",
         "console": "integratedTerminal"
       }
     ]
   }
   ```

### IntelliJ IDEA / RustRover

1. **Install Plugins**:
   - Rust plugin
   - TOML plugin
   - Database Tools
   - GraphQL plugin

2. **Project Setup**:
   ```
   File → Open → Select project root
   Trust project when prompted
   Configure SDK: File → Project Structure → SDKs → Add Rust toolchain
   ```

3. **Run Configurations**:
   - Add Cargo configuration for `workflow-engine`
   - Add npm configuration for frontend dev server
   - Add Python configuration for MCP servers

## Git Hooks and Pre-commit Setup

### 1. Install Pre-commit Framework

```bash
# Install pre-commit
pip install pre-commit

# Or with UV
uv pip install pre-commit
```

### 2. Create Pre-commit Configuration

Create `.pre-commit-config.yaml` in project root:

```yaml
repos:
  # Rust formatting and linting
  - repo: local
    hooks:
      - id: rust-fmt
        name: Rust Format
        entry: cargo fmt --
        language: system
        types: [rust]
        pass_filenames: false
        
      - id: rust-clippy
        name: Rust Clippy
        entry: cargo clippy -- -D warnings
        language: system
        types: [rust]
        pass_filenames: false

  # Frontend linting
  - repo: local
    hooks:
      - id: eslint
        name: ESLint
        entry: bash -c 'cd frontend && npm run lint'
        language: system
        types: [typescript, typescriptreact]
        pass_filenames: false

  # Python linting
  - repo: https://github.com/psf/black
    rev: 24.3.0
    hooks:
      - id: black
        files: ^mcp-servers/

  - repo: https://github.com/charliermarsh/ruff-pre-commit
    rev: v0.3.0
    hooks:
      - id: ruff
        files: ^mcp-servers/

  # Security scanning
  - repo: local
    hooks:
      - id: cargo-audit
        name: Cargo Audit
        entry: cargo audit
        language: system
        pass_filenames: false
        types: [rust]

  # Commit message
  - repo: https://github.com/commitizen-tools/commitizen
    rev: v3.20.0
    hooks:
      - id: commitizen
```

### 3. Install Git Hooks

```bash
# Install pre-commit hooks
pre-commit install

# Run hooks manually
pre-commit run --all-files

# Update hooks
pre-commit autoupdate
```

### 4. Commit Message Convention

Follow conventional commits:
- `feat:` New features
- `fix:` Bug fixes
- `docs:` Documentation changes
- `style:` Code style changes (formatting)
- `refactor:` Code refactoring
- `perf:` Performance improvements
- `test:` Test additions/changes
- `build:` Build system changes
- `ci:` CI/CD changes
- `chore:` Maintenance tasks

## Database Setup and Migrations

### 1. PostgreSQL Setup

```bash
# Create development database
createdb ai_workflow_db

# Create test database
createdb ai_workflow_db_test

# Initialize with schema
psql ai_workflow_db < scripts/init-db.sql

# Create database user (optional)
psql -c "CREATE USER workflow_user WITH PASSWORD 'dev_password';"
psql -c "GRANT ALL PRIVILEGES ON DATABASE ai_workflow_db TO workflow_user;"
```

### 2. Running Migrations

#### Main Application (Diesel)
```bash
# Install Diesel CLI if not already installed
cargo install diesel_cli --no-default-features --features postgres

# Setup Diesel
diesel setup

# Run migrations
diesel migration run

# Revert migrations
diesel migration revert

# Create new migration
diesel migration generate add_user_preferences
```

#### Microservices (SQLx)
```bash
# Content Processing Service
cd services/content_processing
sqlx database create
sqlx migrate run

# Create new migration
sqlx migrate add create_content_tags

# Check migration status
sqlx migrate info
```

### 3. Dgraph Setup (Knowledge Graph)

```bash
# Start Dgraph using Docker
cd services/knowledge_graph/dgraph
docker-compose up -d

# Initialize schema
./init-schema.sh

# Access Dgraph UI
open http://localhost:8000
```

### 4. Database Backup and Restore

```bash
# Backup database
pg_dump ai_workflow_db > backup_$(date +%Y%m%d_%H%M%S).sql

# Restore database
psql ai_workflow_db < backup_20241220_120000.sql

# Backup with compression
pg_dump -Fc ai_workflow_db > backup.dump

# Restore from compressed backup
pg_restore -d ai_workflow_db backup.dump
```

## Environment Variables Configuration

### 1. Backend Configuration

Create `.env` in project root:

```bash
# Core Database Configuration
DATABASE_URL=postgresql://username:password@localhost/ai_workflow_db
TEST_DATABASE_URL=postgresql://username:password@localhost/ai_workflow_db_test

# JWT Configuration
JWT_SECRET=your-secure-jwt-secret-key-at-least-32-chars
JWT_EXPIRATION=86400  # 24 hours in seconds

# Server Configuration
HOST=0.0.0.0
PORT=8080
RUST_LOG=debug,sqlx=warn,hyper=info

# AI Provider Keys (optional for development)
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...

# Redis Configuration
REDIS_URL=redis://localhost:6379/0

# GraphQL Federation
GATEWAY_PORT=4000
SUBGRAPH_URLS=http://localhost:8080/api/v1/graphql,http://localhost:8004/graphql

# Service Ports
CONTENT_PROCESSING_PORT=8004
KNOWLEDGE_GRAPH_PORT=8005
REALTIME_COMM_PORT=8006

# Monitoring
PROMETHEUS_PORT=9090
GRAFANA_PORT=3000

# Feature Flags
ENABLE_METRICS=true
ENABLE_TRACING=true
ENABLE_RATE_LIMITING=true
```

### 2. Frontend Configuration

Create `.env` in `frontend/`:

```bash
# API Configuration
VITE_API_BASE_URL=http://localhost:8080
VITE_GRAPHQL_URL=http://localhost:4000/graphql
VITE_WS_URL=ws://localhost:8006

# Feature Flags
VITE_ENABLE_MOCK_DATA=false
VITE_ENABLE_DEBUG_MODE=true

# Analytics (optional)
VITE_GA_TRACKING_ID=UA-XXXXXXXXX-X
```

### 3. MCP Servers Configuration

Create `.env` in `mcp-servers/`:

```bash
# Server Ports
HELPSCOUT_PORT=8001
NOTION_PORT=8002
SLACK_PORT=8003

# Mock API Keys (for testing)
HELPSCOUT_API_KEY=test_key
NOTION_API_KEY=test_key
SLACK_API_KEY=test_key

# Logging
LOG_LEVEL=INFO
LOG_FORMAT=json
```

### 4. Docker Compose Environment

Create `.env.docker`:

```bash
# Docker-specific overrides
DATABASE_URL=postgresql://postgres:postgres@db:5432/ai_workflow_db
REDIS_URL=redis://redis:6379/0

# Service discovery
CONTENT_PROCESSING_URL=http://content-processing:8004
KNOWLEDGE_GRAPH_URL=http://knowledge-graph:8005
REALTIME_COMM_URL=http://realtime-comm:8006
```

## Local Development Workflow

### 1. Standard Development Flow

```bash
# 1. Start infrastructure services
docker compose up -d postgres redis dgraph

# 2. Start MCP servers
./scripts/start_test_servers.sh

# 3. Start backend services (in separate terminals)
# Terminal 1: Main API
cargo run --bin workflow-engine

# Terminal 2: GraphQL Gateway
cargo run --bin graphql-gateway

# Terminal 3: Frontend
cd frontend && npm run dev

# 4. Access services
# Frontend: http://localhost:5173
# API: http://localhost:8080
# GraphQL Gateway: http://localhost:4000/graphql
# Swagger UI: http://localhost:8080/swagger-ui/
```

### 2. Microservices Development

```bash
# Start all microservices
./scripts/start-local.sh

# Or start individually:
# Content Processing
cd services/content_processing && cargo run

# Knowledge Graph
cd services/knowledge_graph && cargo run

# Realtime Communication
cd services/realtime_communication && cargo run
```

### 3. Running with Docker Compose

```bash
# Development environment (with hot reload)
docker compose -f docker-compose.yml -f docker-compose.dev.yml up

# Production-like environment
docker compose -f docker-compose.yml -f docker-compose.prod.yml up

# Minimal setup (just core services)
docker compose -f docker-compose.minimal.yml up

# View logs
docker compose logs -f ai-workflow-system

# Rebuild after changes
docker compose build --no-cache
```

### 4. Quick Commands with Just

Create `justfile` in project root:

```makefile
# List available commands
default:
  @just --list

# Setup development environment
setup:
  cargo build
  cd frontend && npm install
  cd mcp-servers && pip install -r requirements.txt
  createdb ai_workflow_db || true
  diesel migration run

# Run all services
run-all:
  ./scripts/start-all.sh

# Run tests
test:
  cargo test
  cd frontend && npm test

# Format code
fmt:
  cargo fmt
  cd frontend && npm run format

# Lint code
lint:
  cargo clippy -- -D warnings
  cd frontend && npm run lint

# Clean build artifacts
clean:
  cargo clean
  cd frontend && rm -rf dist node_modules
```

## Debugging Setup

### 1. Rust Debugging

#### VS Code with CodeLLDB
```json
{
  "type": "lldb",
  "request": "launch",
  "name": "Debug unit tests",
  "cargo": {
    "args": ["test", "--no-run", "--lib"],
    "filter": {
      "name": "workflow-engine-core",
      "kind": "lib"
    }
  },
  "args": ["test_name"],
  "cwd": "${workspaceFolder}"
}
```

#### Command Line Debugging
```bash
# Build with debug symbols
cargo build

# Use rust-gdb
rust-gdb target/debug/workflow-engine

# Or use lldb
rust-lldb target/debug/workflow-engine

# Set breakpoints and run
(lldb) b main
(lldb) r
```

### 2. Frontend Debugging

#### Browser DevTools
```javascript
// Add debugger statements
debugger;

// Or use console methods
console.log('Debug info:', variable);
console.trace();
```

#### VS Code Debugging
```json
{
  "type": "chrome",
  "request": "launch",
  "name": "Debug Frontend",
  "url": "http://localhost:5173",
  "webRoot": "${workspaceFolder}/frontend",
  "sourceMaps": true
}
```

### 3. Remote Debugging

```bash
# Enable remote debugging for Node.js
node --inspect=0.0.0.0:9229 index.js

# For Rust, use remote GDB
gdbserver :9999 ./target/debug/workflow-engine
```

### 4. Logging and Tracing

```bash
# Set log levels
export RUST_LOG=debug,tower_http=trace,sqlx=warn

# Enable backtraces
export RUST_BACKTRACE=1  # or full

# Enable async tracing
export TOKIO_CONSOLE_ENABLE=1

# Use correlation IDs
curl -H "X-Correlation-ID: test-123" http://localhost:8080/api/v1/health
```

## Performance Profiling Setup

### 1. CPU Profiling

```bash
# Install flamegraph
cargo install flamegraph

# Profile with flamegraph
cargo flamegraph --bin workflow-engine

# Profile specific test
cargo flamegraph --test load_test

# On Linux, may need to enable perf
echo -1 | sudo tee /proc/sys/kernel/perf_event_paranoid
```

### 2. Memory Profiling

```bash
# Using Valgrind (Linux)
valgrind --leak-check=full --show-leak-kinds=all \
  target/debug/workflow-engine

# Using heaptrack
heaptrack target/debug/workflow-engine
heaptrack --analyze heaptrack.workflow-engine.12345.gz
```

### 3. Async Runtime Profiling

```bash
# Install tokio-console
cargo install tokio-console

# Run with console enabled
TOKIO_CONSOLE_ENABLE=1 cargo run --bin workflow-engine

# In another terminal
tokio-console
```

### 4. Benchmarking

```bash
# Run benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench api_throughput

# Compare benchmarks
cargo install cargo-criterion
cargo criterion

# Continuous benchmarking
cargo bench -- --save-baseline master
git checkout feature-branch
cargo bench -- --baseline master
```

## Development Best Practices

### 1. Code Style Guidelines

#### Rust
- Follow standard Rust naming conventions
- Use `clippy` pedantic lints for better code quality
- Document public APIs with examples
- Prefer `Result<T, E>` over panics
- Use structured logging with `tracing`

```rust
/// Example of good documentation
/// 
/// # Examples
/// 
/// ```
/// use workflow_engine_core::Workflow;
/// 
/// let workflow = Workflow::new("example");
/// assert_eq!(workflow.name(), "example");
/// ```
pub fn create_workflow(name: &str) -> Result<Workflow, Error> {
    // Implementation
}
```

#### TypeScript/React
- Use functional components with hooks
- Implement proper error boundaries
- Follow React testing library best practices
- Use TypeScript strict mode
- Implement proper loading and error states

```typescript
// Good component example
export const WorkflowList: React.FC = () => {
  const { workflows, loading, error } = useWorkflows();

  if (loading) return <LoadingSpinner />;
  if (error) return <ErrorMessage error={error} />;

  return (
    <div>
      {workflows.map(workflow => (
        <WorkflowCard key={workflow.id} workflow={workflow} />
      ))}
    </div>
  );
};
```

### 2. Testing Best Practices

#### Test Organization
```
tests/
├── unit/           # Fast, isolated tests
├── integration/    # Tests with real dependencies
├── e2e/           # End-to-end user scenarios
└── performance/   # Load and stress tests
```

#### Test Patterns
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::prelude::*;

    #[test]
    fn test_workflow_creation() {
        // Arrange
        let name = "test-workflow";
        
        // Act
        let workflow = Workflow::new(name);
        
        // Assert
        assert_eq!(workflow.name(), name);
    }

    #[tokio::test]
    async fn test_async_operation() {
        // Test async code
    }
}
```

### 3. Security Best Practices

- Never commit secrets or API keys
- Use environment variables for configuration
- Implement proper authentication and authorization
- Validate all inputs
- Use prepared statements for database queries
- Keep dependencies updated

```bash
# Security audit
cargo audit

# Check for outdated dependencies
cargo outdated

# Update dependencies safely
cargo update --dry-run
```

### 4. Performance Best Practices

- Profile before optimizing
- Use connection pooling for databases
- Implement proper caching strategies
- Use pagination for large datasets
- Optimize database queries with indexes
- Use async/await appropriately

```rust
// Good: Connection pooling
let pool = sqlx::PgPoolOptions::new()
    .max_connections(20)
    .connect(&database_url)
    .await?;

// Good: Pagination
pub async fn list_workflows(
    page: u32,
    page_size: u32,
) -> Result<Page<Workflow>, Error> {
    // Implementation
}
```

## Contribution Guidelines

### 1. Development Process

1. **Fork and Clone**
   ```bash
   git clone https://github.com/yourusername/workflow-engine-rs.git
   cd workflow-engine-rs
   git remote add upstream https://github.com/original/workflow-engine-rs.git
   ```

2. **Create Feature Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make Changes**
   - Write tests first (TDD)
   - Implement feature
   - Update documentation
   - Add examples if applicable

4. **Run Quality Checks**
   ```bash
   # Format code
   cargo fmt
   cd frontend && npm run format

   # Lint
   cargo clippy -- -D warnings
   cd frontend && npm run lint

   # Test
   cargo test
   cd frontend && npm test

   # Security audit
   cargo audit
   ```

5. **Commit with Conventional Commits**
   ```bash
   git add .
   git commit -m "feat: add new workflow node type"
   ```

6. **Push and Create PR**
   ```bash
   git push origin feature/your-feature-name
   # Create PR on GitHub
   ```

### 2. PR Requirements

- [ ] All tests pass
- [ ] Code is formatted and linted
- [ ] Documentation is updated
- [ ] Commit messages follow convention
- [ ] No security vulnerabilities
- [ ] Performance impact considered
- [ ] Breaking changes documented

### 3. Code Review Process

1. Automated checks must pass
2. At least one maintainer approval
3. No unresolved conversations
4. Rebase on main before merge

## Troubleshooting

### Common Issues and Solutions

#### 1. Database Connection Errors

**Problem**: `FATAL: password authentication failed`

**Solution**:
```bash
# Check PostgreSQL is running
sudo systemctl status postgresql

# Fix authentication
sudo -u postgres psql
ALTER USER postgres PASSWORD 'yourpassword';

# Update DATABASE_URL in .env
DATABASE_URL=postgresql://postgres:yourpassword@localhost/ai_workflow_db
```

#### 2. Port Conflicts

**Problem**: `Address already in use`

**Solution**:
```bash
# Find process using port
lsof -i :8080  # macOS/Linux
netstat -ano | findstr :8080  # Windows

# Kill process
kill -9 <PID>

# Or use different ports in .env
PORT=8081
```

#### 3. Rust Compilation Errors

**Problem**: `error[E0282]: type annotations needed`

**Solution**:
```bash
# Clean build cache
cargo clean

# Update dependencies
cargo update

# Check for breaking changes
cargo tree --duplicates
```

#### 4. Frontend Build Errors

**Problem**: `Module not found` errors

**Solution**:
```bash
# Clear cache and reinstall
cd frontend
rm -rf node_modules package-lock.json
npm install

# Clear build cache
rm -rf dist .parcel-cache
```

#### 5. MCP Server Connection Issues

**Problem**: `Connection refused` to MCP servers

**Solution**:
```bash
# Check servers are running
ps aux | grep python | grep server

# Restart servers
./scripts/start_test_servers.sh

# Check logs
tail -f logs/mcp-*.log
```

#### 6. Docker Issues

**Problem**: `Cannot connect to Docker daemon`

**Solution**:
```bash
# Start Docker daemon
sudo systemctl start docker  # Linux
open -a Docker  # macOS

# Add user to docker group (Linux)
sudo usermod -aG docker $USER
newgrp docker
```

#### 7. Test Failures

**Problem**: Integration tests failing

**Solution**:
```bash
# Ensure all services are running
./scripts/start_test_servers.sh

# Check test database
psql ai_workflow_db_test -c "\dt"

# Run with debug output
RUST_LOG=debug cargo test -- --nocapture
```

### Getting Help

1. **Check Documentation**:
   - Project README files
   - CLAUDE.md files for AI assistance
   - API documentation: `cargo doc --open`

2. **Debug Logging**:
   ```bash
   RUST_LOG=debug cargo run
   RUST_BACKTRACE=full cargo test
   ```

3. **Community Resources**:
   - GitHub Issues
   - Discord/Slack community
   - Stack Overflow tags

4. **Diagnostic Commands**:
   ```bash
   # System info
   cargo version
   node --version
   python3 --version
   postgres --version
   
   # Project info
   cargo tree
   npm list
   
   # Check services
   ./scripts/health-check.sh
   ```

### Performance Troubleshooting

1. **Slow Builds**:
   ```bash
   # Use sccache
   cargo install sccache
   export RUSTC_WRAPPER=sccache
   
   # Incremental compilation
   export CARGO_INCREMENTAL=1
   ```

2. **Slow Tests**:
   ```bash
   # Run tests in parallel
   cargo test -- --test-threads=8
   
   # Skip slow tests
   cargo test -- --skip slow
   ```

3. **Memory Issues**:
   ```bash
   # Limit memory usage
   export CARGO_BUILD_JOBS=2
   
   # Monitor memory
   watch -n 1 free -h
   ```

---

This development setup guide is maintained by the project team. For updates or corrections, please submit a PR or open an issue.

Last updated: December 2024