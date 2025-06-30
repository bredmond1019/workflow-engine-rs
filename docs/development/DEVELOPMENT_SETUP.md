# Development Setup Guide

This guide will help you set up a development environment for the AI Workflow Engine project.

## Prerequisites

### Required Software

- **Rust** (1.75 or later)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  source ~/.cargo/env
  ```

- **PostgreSQL** (13 or later)
  ```bash
  # macOS (using Homebrew)
  brew install postgresql
  brew services start postgresql
  
  # Ubuntu/Debian
  sudo apt update
  sudo apt install postgresql postgresql-contrib
  sudo systemctl start postgresql
  
  # Create database
  createdb ai_workflow_db
  ```

- **Docker** (optional, for external services)
  ```bash
  # Install Docker Desktop or Docker Engine
  # https://docs.docker.com/get-docker/
  ```

- **Python** (3.8 or later, for MCP servers)
  ```bash
  # Ensure you have Python 3.8+
  python3 --version
  
  # Install dependencies for MCP servers
  cd mcp-servers
  pip install -r requirements.txt
  ```

### Optional Tools

- **Redis** (for session storage and caching)
  ```bash
  # macOS
  brew install redis
  brew services start redis
  
  # Ubuntu/Debian  
  sudo apt install redis-server
  sudo systemctl start redis
  ```

- **Just** (command runner, alternative to make)
  ```bash
  cargo install just
  ```

## Environment Configuration

### 1. Database Setup

Create your development database:

```bash
# Create the database
createdb ai_workflow_db

# Initialize schema (if using migrations)
# Note: This will be available once migrations are implemented
# psql ai_workflow_db < scripts/init-db.sql
```

### 2. Environment Variables

Create a `.env` file in the project root:

```bash
# Database Configuration
DATABASE_URL=postgresql://username:password@localhost/ai_workflow_db

# JWT Configuration
JWT_SECRET=your-secure-jwt-secret-key-for-development

# Optional: AI Provider Keys (for testing AI features)
OPENAI_API_KEY=your_openai_api_key_here
ANTHROPIC_API_KEY=your_anthropic_api_key_here

# Optional: Service Configuration
REDIS_URL=redis://localhost:6379
LOG_LEVEL=debug
```

### 3. Build the Project

```bash
# Clone the repository
git clone <repository-url>
cd workflow-engine-rs

# Build all packages
cargo build

# Run tests (unit tests only, no external dependencies)
cargo test

# Build optimized release version
cargo build --release
```

## Development Workflow

### 1. Running the Main Server

```bash
# Development mode with debug logging
cargo run

# The server will start on http://localhost:8080
# Swagger UI available at http://localhost:8080/swagger-ui/
```

### 2. Running Individual Services

```bash
# Content Processing service
cd services/content_processing
cargo run

# Knowledge Graph service  
cd services/knowledge_graph
cargo run

# Realtime Communication service
cd services/realtime_communication
cargo run
```

### 3. Running MCP Test Servers

```bash
# Start all MCP test servers
./scripts/start_test_servers.sh

# Individual servers:
cd mcp-servers
python -m servers.helpscout_server  # Port 8001
python -m servers.notion_server     # Port 8002
python -m servers.slack_server      # Port 8003
```

### 4. Development Commands

```bash
# Format code
cargo fmt

# Run linter with strict settings
cargo clippy -- -D warnings

# Run all tests (including integration tests requiring external services)
./scripts/start_test_servers.sh
cargo test -- --ignored

# Run specific test suites
cargo test --test end_to_end_workflow_test -- --ignored
cargo test --test mcp_communication_test -- --ignored
cargo test --test workflow_external_tools_test -- --ignored

# Check for security vulnerabilities
cargo audit

# Generate documentation
cargo doc --open
```

## Testing Strategy

### Unit Tests
Run without external dependencies:
```bash
cargo test
```

### Integration Tests  
Require external services to be running:
```bash
./scripts/start_test_servers.sh
cargo test -- --ignored
```

### Load Tests
Performance and scalability testing:
```bash
cargo test --test load_test -- --ignored --nocapture
```

### Chaos Tests
Resilience and failure testing:
```bash
cargo test --test chaos_test -- --ignored --nocapture
```

## Architecture Overview

```
workflow-engine-rs/
├── crates/
│   ├── workflow-engine-core/    # Core workflow engine
│   ├── workflow-engine-api/     # REST API server
│   └── workflow-engine-mcp/     # MCP protocol implementation
├── services/                    # Microservices
│   ├── content_processing/      # Content analysis
│   ├── knowledge_graph/         # Graph database operations
│   └── realtime_communication/  # WebSocket messaging
├── mcp-servers/                 # Python MCP servers
├── tests/                       # Integration tests
└── scripts/                     # Development scripts
```

## Development Tips

### 1. Debugging

- Enable debug logging: `RUST_LOG=debug cargo run`
- Use correlation IDs to trace requests across services
- Monitor metrics at `http://localhost:9090` (Prometheus)
- View dashboards at `http://localhost:3000` (Grafana, admin/admin)

### 2. Database Development

- Use `diesel` CLI for migrations (when available)
- Each service can have its own database schema
- Main app uses PostgreSQL, services can use different databases

### 3. MCP Development

- Test individual MCP servers with `scripts/test_mcp_server.py`
- MCP protocol supports HTTP, WebSocket, and stdio transports
- Use connection pooling for external MCP services

### 4. Performance

- Profile with `cargo flamegraph` (install with `cargo install flamegraph`)
- Monitor memory usage with built-in metrics
- Use `--release` builds for performance testing

## Troubleshooting

### Common Issues

1. **Database Connection Errors**
   - Ensure PostgreSQL is running: `brew services list | grep postgres`
   - Check DATABASE_URL in `.env` file
   - Verify database exists: `psql -l | grep ai_workflow`

2. **Port Conflicts**
   - Main server: 8080
   - MCP servers: 8001-8003
   - Services: 8004-8006
   - Prometheus: 9090
   - Grafana: 3000

3. **Build Errors**
   - Update Rust: `rustup update`
   - Clean build: `cargo clean && cargo build`
   - Check dependencies: `cargo tree`

4. **Test Failures**
   - External services not running: Check `./scripts/start_test_servers.sh`
   - Database not initialized: Recreate test database
   - Network issues: Verify localhost connectivity

### Getting Help

- Check the [README.md](./README.md) for project overview
- Review [QUICK_START.md](./QUICK_START.md) for immediate setup
- Monitor [CHANGELOG.md](./CHANGELOG.md) for recent changes
- Open an issue for bugs or feature requests

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/your-feature`
3. Make your changes and add tests
4. Run the full test suite: `cargo test && cargo test -- --ignored`
5. Ensure code formatting: `cargo fmt && cargo clippy`
6. Submit a pull request

### Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Address all clippy warnings (`cargo clippy -- -D warnings`)
- Add documentation for public APIs
- Include tests for new functionality
- Update this guide when adding new setup requirements