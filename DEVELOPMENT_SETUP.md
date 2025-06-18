# AI Workflow System - Development Environment Setup Guide

This comprehensive guide walks you through setting up the AI Workflow System development environment from scratch. The system is a production-ready AI workflow orchestration platform built in Rust with Python MCP servers, featuring microservices architecture, event sourcing, and comprehensive monitoring.

**Estimated Setup Time:** 15-30 minutes (depending on your existing tools)

## Table of Contents

- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Detailed Setup Instructions](#detailed-setup-instructions)
  - [1. Install Prerequisites](#1-install-prerequisites)
  - [2. Clone the Repository](#2-clone-the-repository)
  - [3. Environment Configuration](#3-environment-configuration)
  - [4. Database Setup](#4-database-setup)
  - [5. Build the Main Application](#5-build-the-main-application)
  - [6. Python MCP Servers Setup](#6-python-mcp-servers-setup)
  - [7. Start the System](#7-start-the-system)
- [Testing the Setup](#testing-the-setup)
- [Docker Setup (Alternative)](#docker-setup-alternative)
- [Troubleshooting](#troubleshooting)
- [Next Steps](#next-steps)

## Prerequisites

Before you begin, ensure you have the following installed:

### System Requirements

**Minimum Hardware:**
- CPU: 2 cores, 2.0 GHz (4+ cores recommended for microservices)
- RAM: 8 GB (16 GB recommended for full development stack)
- Storage: 4 GB free space (includes Docker images and databases)
- Network: Internet connection for dependencies and external services

**Supported Operating Systems:**
- macOS 11+ (Big Sur and later) - **Recommended for development**
- Ubuntu 20.04+ / Debian 11+
- Fedora 36+ / CentOS Stream 9+
- Windows 11+ with WSL2 (Ubuntu 20.04+ in WSL)

**Architecture Support:**
- x86_64 (Intel/AMD)
- ARM64 (Apple Silicon M1/M2/M3, ARM servers)

### Required Software

| Software | Minimum Version | Purpose | Installation Guide |
|----------|----------------|---------|-------------------|
| **Rust** | 1.75+ (1.78+ recommended) | Main application and microservices | [rustup.rs](https://rustup.rs) |
| **PostgreSQL** | 15+ | Primary database and event store | See [PostgreSQL Installation](#postgresql-installation) |
| **Python** | 3.10+ (3.11+ recommended) | MCP servers and integration scripts | [python.org](https://www.python.org) |
| **Git** | 2.30+ | Version control | [git-scm.com](https://git-scm.com) |
| **Docker** | 24.0+ | Dgraph, monitoring, optional containerization | [docker.com](https://docs.docker.com/get-docker/) |
| **uv** | 0.1.0+ | Fast Python package management | [astral.sh/uv](https://github.com/astral-sh/uv) |

### Optional Software

| Software | Purpose | Installation Guide |
|----------|---------|-------------------|
| **Docker Compose** | Multi-service orchestration | Included with Docker Desktop |
| **curl** | API testing and downloads | Pre-installed on most systems |
| **Build Tools** | Rust compilation dependencies | Platform-specific (see troubleshooting) |

### PostgreSQL Installation

#### macOS
```bash
# Using Homebrew
brew install postgresql@15
brew services start postgresql@15

# Add to PATH (add to ~/.zshrc or ~/.bash_profile)
export PATH="/opt/homebrew/opt/postgresql@15/bin:$PATH"
```

#### Ubuntu/Debian
```bash
sudo apt update
sudo apt install postgresql-15 postgresql-contrib-15
sudo systemctl start postgresql
sudo systemctl enable postgresql
```

#### Windows
Download and install from [PostgreSQL Windows Downloads](https://www.postgresql.org/download/windows/)

### uv Installation

uv is a fast Python package manager that replaces pip and virtualenv:

```bash
# Install uv
curl -LsSf https://astral.sh/uv/install.sh | sh

# Add to PATH (usually done automatically)
export PATH="$HOME/.local/bin:$PATH"
```

## Quick Start (Automated Setup)

For experienced developers who want to get up and running quickly:

```bash
# Clone the repository
git clone <your-repository-url>
cd ai-system-rust

# Run the automated setup (handles all prerequisites)
chmod +x scripts/setup.sh
./scripts/setup.sh

# The automated script will:
# ✓ Install Rust, PostgreSQL, Python, and uv
# ✓ Set up PostgreSQL database and user
# ✓ Configure environment variables
# ✓ Build the main Rust application
# ✓ Install Python MCP server dependencies
# ✓ Create development helper scripts
# ✓ Validate the complete setup
```

**If automated setup succeeds, skip to [Testing the Setup](#testing-the-setup).**

**If you encounter issues or prefer manual control, continue with the detailed instructions below.**

## Detailed Setup Instructions

Follow these steps if you prefer manual setup or if the automated script encounters issues.

### 1. Install Prerequisites

#### Install Rust
```bash
# Install Rust via rustup (stable toolchain)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the prompts, then reload your shell configuration
source $HOME/.cargo/env

# Verify installation
rustc --version  # Should show 1.75 or higher
cargo --version

# Install additional tools for development
cargo install cargo-watch  # Auto-rebuild on file changes
cargo install cargo-audit  # Security vulnerability scanning
```

#### Install PostgreSQL
Follow the platform-specific instructions in the [PostgreSQL Installation](#postgresql-installation) section above.

#### Install Python
```bash
# macOS (using Homebrew)
brew install python@3.11

# Ubuntu/Debian
sudo apt update
sudo apt install python3.11 python3.11-venv python3-pip

# Verify installation
python3 --version  # Should show 3.11 or higher
```

#### Install uv (Python Package Manager)
```bash
# Install uv - fast Python package manager
curl -LsSf https://astral.sh/uv/install.sh | sh

# Add to PATH (add this to your shell profile)
export PATH="$HOME/.local/bin:$PATH"

# Verify installation
uv --version

# uv provides:
# - Fast package installation (10-100x faster than pip)
# - Automatic Python version management
# - Built-in virtual environment handling
```

### 2. Clone the Repository

```bash
# Clone the repository
git clone <your-repository-url>
cd ai-system-rust

# Verify you're on the correct branch
git branch -a
git status

# The repository structure:
# ├── src/                 # Main Rust application
# ├── services/            # Microservices (content_processing, knowledge_graph, realtime_communication)
# ├── scripts/             # Setup and MCP server scripts
# ├── monitoring/          # Prometheus, Grafana configuration
# ├── docker-compose.yml   # Full stack deployment
# └── tests/               # Integration and end-to-end tests
```

### 3. Environment Configuration

Create your environment configuration file:

```bash
# Copy the example configuration
cp .env.example .env

# Edit the .env file with your preferred editor
# Update the following values:
# - DATABASE_URL (if using non-default database settings)
# - JWT_SECRET (use a secure random string for production)
# - API keys for external services (if you have them)
```

**Key Environment Variables:**
```env
# Database Configuration (PostgreSQL)
DATABASE_URL=postgres://aiworkflow:aiworkflow123@localhost:5432/ai_workflow
DB_USER=aiworkflow
DB_PASSWORD=aiworkflow123
DB_NAME=ai_workflow
DB_PORT=5432

# Main API Configuration
HOST=127.0.0.1
PORT=8080
RUST_LOG=info

# Security
JWT_SECRET=development-jwt-secret-key-please-change-in-production-2024
RATE_LIMIT_PER_MINUTE=60

# MCP Server URLs (Python servers running locally)
NOTION_MCP_URL=ws://localhost:3001
SLACK_MCP_URL=ws://localhost:3002
HELPSCOUT_MCP_URL=ws://localhost:3003

# External API Keys (optional - for full functionality)
NOTION_API_KEY=your-notion-api-key
SLACK_BOT_TOKEN=your-slack-bot-token
HELPSCOUT_API_KEY=your-helpscout-api-key

# Monitoring Stack
PROMETHEUS_PORT=9090
GRAFANA_PORT=3000
JAEGER_UI_PORT=16686
```

### 4. Database Setup

**The system uses multiple databases:**
- **PostgreSQL**: Primary database, event store, user sessions
- **Dgraph**: Knowledge graph database (for knowledge_graph service)
- **Redis**: Caching and real-time features (optional)

#### PostgreSQL Setup
```bash
# Run the database setup script
chmod +x scripts/database-setup.sh
./scripts/database-setup.sh

# This script will:
# ✓ Create the database user 'aiworkflow' with appropriate permissions
# ✓ Create the database 'ai_workflow'
# ✓ Initialize the database schema (tables, indexes, constraints)
# ✓ Test the database connection
# ✓ Set up event store tables for event sourcing
```

**Manual Database Setup (if automated script fails):**
```bash
# Connect to PostgreSQL as superuser
sudo -u postgres psql

# In the PostgreSQL prompt:
CREATE USER aiworkflow WITH PASSWORD 'aiworkflow123';
CREATE DATABASE ai_workflow OWNER aiworkflow;
ALTER USER aiworkflow CREATEDB;
ALTER USER aiworkflow WITH SUPERUSER;  -- For development only
\q

# Initialize the schema
psql -U aiworkflow -d ai_workflow -f scripts/init-db.sql

# Verify tables were created
psql -U aiworkflow -d ai_workflow -c "\dt"
```

#### Knowledge Graph Database (Dgraph)
```bash
# Start Dgraph using Docker Compose (recommended)
cd services/knowledge_graph/dgraph
docker-compose up -d

# Verify Dgraph is running
curl http://localhost:8080/health

# The Dgraph UI will be available at:
# - Alpha (queries): http://localhost:8080
# - Ratel UI: http://localhost:8000
```

### 5. Build the System

**The system consists of multiple components:**
- Main Rust application (HTTP API and workflow engine)
- Three Rust microservices
- Python MCP servers

#### Build Main Application
```bash
# Fetch all dependencies
cargo fetch

# Build in debug mode (faster compilation, includes debug symbols)
cargo build

# Build for release (optimized, production-ready)
cargo build --release

# Run unit tests
cargo test
```

#### Build Microservices
```bash
# Build all microservices
for service in content_processing knowledge_graph realtime_communication; do
    echo "Building $service..."
    cd services/$service
    cargo build
    cd ../..
done

# Or build individually:
cd services/content_processing && cargo build
cd ../knowledge_graph && cargo build  
cd ../realtime_communication && cargo build
cd ../..
```

### 6. Python MCP Servers Setup

**MCP (Model Context Protocol) servers provide external service integration:**
- **Customer Support**: Ticket management and response generation
- **Notion**: Knowledge base integration
- **Slack**: Team communication
- **HelpScout**: Customer support platform

```bash
# Navigate to scripts directory
cd scripts

# Install Python dependencies using uv (much faster than pip)
uv sync

# This installs:
# - mcp[cli] for Model Context Protocol
# - Required dependencies for external service integration

# Return to project root
cd ..

# Test MCP server functionality
cd scripts
uv run python test_mcp_server.py
cd ..

# The MCP servers support stdio transport protocol
# and can be integrated with external AI systems
```

### 7. Start the System

**The system can be started in multiple ways depending on your development needs:**

#### Option A: Start Everything Manually (Recommended for Development)

```bash
# Terminal 1: Start Dgraph (Knowledge Graph)
cd services/knowledge_graph/dgraph
docker-compose up -d
cd ../../..

# Terminal 2: Start MCP servers
./scripts/start_test_servers.sh

# Terminal 3: Start main application
cargo run --bin backend

# Terminal 4: Start microservices (optional)
cd services/content_processing && cargo run &
cd ../knowledge_graph && cargo run &
cd ../realtime_communication && cargo run &
cd ../..

# Services will be available at:
# - Main API: http://localhost:8080
# - Content Processing: http://localhost:8081
# - Knowledge Graph: http://localhost:8082  
# - Realtime Communication: http://localhost:8083
```

#### Option B: Use Development Helper Scripts

```bash
# The setup script creates a helpful dev.sh script
chmod +x dev.sh

# Start core services
./dev.sh start      # Starts MCP servers + main app with auto-reload

# Other helpful commands:
./dev.sh mcp        # Start only MCP servers
./dev.sh test       # Run all tests
./dev.sh db         # Connect to database
./dev.sh logs       # Tail application logs
./dev.sh clean      # Clean build artifacts
```

#### Option C: Full Docker Compose Stack

```bash
# Start the complete containerized stack
docker-compose up -d

# This starts:
# ✓ PostgreSQL database
# ✓ Main AI Workflow System
# ✓ All MCP servers (Notion, Slack, HelpScout)
# ✓ Prometheus (metrics)
# ✓ Grafana (dashboards)
# ✓ Jaeger (distributed tracing)
# ✓ Redis (caching)
# ✓ Nginx (optional reverse proxy)

# View logs
docker-compose logs -f ai-workflow-system
docker-compose logs -f        # All services

# Stop services
docker-compose down

# Stop and remove volumes (reset databases)
docker-compose down -v
```

## Testing the Setup

**Comprehensive verification of your development environment:**

### 1. Check Application Health

```bash
# Test main application health
curl http://localhost:8080/api/v1/health
# Expected: {"status":"healthy","timestamp":"..."}

# Test detailed health check (includes database, MCP servers)
curl http://localhost:8080/api/v1/health/detailed
# Expected: Detailed status of all components

# Check API documentation
open http://localhost:8080/swagger-ui/  # Opens Swagger UI

# Test microservices (if running)
curl http://localhost:8081/health  # Content Processing
curl http://localhost:8082/health  # Knowledge Graph
curl http://localhost:8083/health  # Realtime Communication
```

### 2. Run Comprehensive Tests

```bash
# Run unit tests for main application
cargo test

# Run integration tests (requires MCP servers)
./scripts/start_test_servers.sh &
sleep 3  # Wait for servers to start
cargo test -- --ignored

# Run specific test categories
cargo test mcp_client                    # MCP client tests
cargo test external_mcp_integration      # External service integration
cargo test workflow_test                 # Workflow engine tests  
cargo test --test end_to_end_workflow_test  # End-to-end scenarios
cargo test --test load_test -- --ignored   # Performance tests

# Test microservices individually
cd services/content_processing && cargo test
cd ../knowledge_graph && cargo test
cd ../realtime_communication && cargo test
cd ../..

# Run Python MCP server tests
cd scripts && uv run python -m pytest tests/ && cd ..
```

### 3. Test MCP Server Connectivity

```bash
# Test MCP server functionality
cd scripts
uv run python test_mcp_server.py

# Test specific MCP servers
uv run python customer_support_server.py --test
uv run python multi_service_mcp_server.py --service notion --test

# The MCP servers should respond to protocol commands:
# - initialize: Set up server capabilities
# - tools/list: List available tools
# - tools/call: Execute specific tools
cd ..
```

### 4. Check Database Connections

```bash
# PostgreSQL (Main database)
psql postgres://aiworkflow:aiworkflow123@localhost:5432/ai_workflow

# In PostgreSQL prompt:
\dt                          # List all tables
SELECT COUNT(*) FROM events; # Check events table
SELECT COUNT(*) FROM agents; # Check agents table
\q                           # Quit

# Dgraph (Knowledge Graph) - if running
curl http://localhost:8080/health
curl -X POST http://localhost:8080/query -d '{"query":"{ q(func: has(dgraph.type)) { count(uid) } }"}'

# Redis (if running)
redis-cli ping  # Should return PONG
```

## Container-Based Development

**Docker provides a consistent, isolated development environment:**

### 1. Ensure Docker is Installed

```bash
# Check Docker installation
docker --version
docker-compose --version
```

### 2. Start Full Development Stack

```bash
# Copy and configure environment
cp .env.example .env
# Edit .env with your specific values

# Start complete development stack
docker-compose up -d

# Available services:
# ┌─────────────────────────────────────────────────────────────┐
# │ Service             │ URL                    │ Purpose       │
# ├─────────────────────────────────────────────────────────────┤
# │ Main API            │ http://localhost:8080  │ REST API      │
# │ Swagger UI          │ http://localhost:8080/swagger-ui/ │ API Docs │
# │ PostgreSQL          │ localhost:5432         │ Database      │
# │ Prometheus          │ http://localhost:9090  │ Metrics       │
# │ Grafana             │ http://localhost:3000  │ Dashboards    │
# │ Jaeger UI           │ http://localhost:16686 │ Tracing       │
# │ Notion MCP          │ localhost:3001         │ MCP Server    │
# │ Slack MCP           │ localhost:3002         │ MCP Server    │
# │ HelpScout MCP       │ localhost:3003         │ MCP Server    │
# └─────────────────────────────────────────────────────────────┘
```

### 3. View Logs

```bash
# View all logs
docker-compose logs -f

# View specific service logs
docker-compose logs -f ai-workflow-system
docker-compose logs -f postgres
```

### 4. Stop Services

```bash
# Stop all services
docker-compose down

# Stop and remove volumes (database data)
docker-compose down -v
```

## Troubleshooting

### Environment Validation

**Always start with the comprehensive validation script:**

```bash
chmod +x scripts/validate-environment.sh
./scripts/validate-environment.sh

# This script checks:
# ✓ Prerequisites (Rust, PostgreSQL, Python, uv)
# ✓ Version requirements (Rust 1.75+, Python 3.11+)
# ✓ Environment variables configuration
# ✓ Database connectivity and schema
# ✓ Project compilation
# ✓ MCP server dependencies
# ✓ Optional tools (Docker, Docker Compose)

# If validation fails, follow the specific error messages
```

### Common Issues and Solutions

#### PostgreSQL Connection Issues

**Error**: `FATAL: password authentication failed for user "aiworkflow"`

**Solutions**:
```bash
# Check PostgreSQL service status
pg_isready -h localhost -p 5432

# Check if PostgreSQL is running
# macOS (Homebrew):
brew services list | grep postgresql
brew services start postgresql

# Linux (systemd):
sudo systemctl status postgresql
sudo systemctl start postgresql

# Reset user password
sudo -u postgres psql
ALTER USER aiworkflow WITH PASSWORD 'aiworkflow123';
\q

# Re-run database setup
./scripts/database-setup.sh
```

#### Rust Compilation Issues

**Error**: `error: linker 'cc' not found` or `error: Microsoft Visual C++ 14.0 is required`

**Solutions**:
```bash
# macOS
xcode-select --install
# Verify: xcode-select -p

# Ubuntu/Debian
sudo apt update
sudo apt install build-essential pkg-config libssl-dev

# Fedora/RHEL
sudo dnf install gcc gcc-c++ make openssl-devel

# Windows (in PowerShell as Administrator)
# Install Visual Studio Build Tools or Visual Studio Community
# Or install via chocolatey:
# choco install visualstudio2022buildtools

# For Apple Silicon Macs, ensure you have the right toolchain:
rustup target add aarch64-apple-darwin
```

#### Build and Runtime Issues

**Error**: `cargo run could not determine which binary to run`

**Solution**:
```bash
# Always specify the binary name
cargo run --bin backend

# Check available binaries
ls target/debug/
grep "[[bin]]" Cargo.toml

# Clean and rebuild if needed
cargo clean
cargo build
```

**Error**: `Failed to connect to database`

**Solution**:
```bash
# Verify DATABASE_URL in .env
cat .env | grep DATABASE_URL

# Test connection manually
psql "$DATABASE_URL" -c "SELECT version();"

# Check if database exists
psql postgres://aiworkflow:aiworkflow123@localhost:5432/postgres -c "\l"
```

#### MCP Server Issues

**Error**: `Failed to connect to MCP server` or `ModuleNotFoundError: No module named 'mcp'`

**Solutions**:
```bash
# Check if MCP servers are running
ps aux | grep python | grep -E "(customer_support|multi_service)"

# Install/update MCP dependencies
cd scripts
uv sync
# Or if uv isn't working:
pip install mcp[cli]

# Test MCP server individually
uv run python customer_support_server.py --test

# Check Python version
python3 --version  # Should be 3.10+

# Restart all MCP servers
pkill -f "python.*mcp"
./scripts/start_test_servers.sh
cd ..
```

#### Port Conflicts

**Error**: `Address already in use (os error 48)`

**Solutions**:
```bash
# Find what's using the port
lsof -i :8080  # macOS/Linux
netstat -tulpn | grep :8080  # Linux
netstat -ano | findstr :8080  # Windows

# Kill the process (replace PID with actual process ID)
kill -9 <PID>

# Or change ports in .env
PORT=8081
API_PORT=8081

# Common port conflicts:
# 8080: Often used by other development servers
# 5432: Another PostgreSQL instance
# 3000: React/Node.js development servers
```

### Docker and Microservices Issues

**Error**: `Cannot connect to the Docker daemon`

**Solutions**:
```bash
# Start Docker Desktop (macOS/Windows)
# Or start Docker daemon (Linux)
sudo systemctl start docker

# Verify Docker is running
docker info

# Check Docker Compose version
docker-compose --version  # Should be 2.0+
```

**Error**: `Dgraph connection failed` (Knowledge Graph service)

**Solutions**:
```bash
# Start Dgraph separately
cd services/knowledge_graph/dgraph
docker-compose up -d

# Check Dgraph health
curl http://localhost:8080/health

# View Dgraph logs
docker-compose logs dgraph-alpha
cd ../../..
```

**Error**: Microservice build failures

**Solutions**:
```bash
# Build services individually to isolate issues
cd services/content_processing
cargo check  # Check for compile errors
cargo build  # Build if check passes

# Check for missing dependencies
cargo update  # Update dependencies

# SQLx compilation issues (content_processing service)
cargo install sqlx-cli
sqlx migrate run  # Run database migrations
cd ../..
```

## Next Steps

**Your development environment is ready! Here's how to make the most of it:**

### 1. Explore the System

**Main Application:**
- **API Documentation**: http://localhost:8080/swagger-ui/
- **Health Check**: http://localhost:8080/api/v1/health
- **Detailed Health**: http://localhost:8080/api/v1/health/detailed
- **Metrics**: http://localhost:8080/api/v1/metrics
- **Uptime**: http://localhost:8080/api/v1/uptime

**Monitoring Stack:**
- **Grafana Dashboards**: http://localhost:3000 (admin/admin)
- **Prometheus Metrics**: http://localhost:9090
- **Jaeger Tracing**: http://localhost:16686

### 2. Run Example Workflows

```bash
# Run example workflows to understand the system
cargo run --example basic-workflow
cargo run --example knowledge_base_example
cargo run --example ai-research-workflow

# Test MCP integration examples
cargo run --example http_mcp_client_demo
cargo run --example cross_system_integration_demo

# Python client examples
cd examples/python_client
python ai_workflow_client.py
python ai_tutor_service.py
cd ../..
```

### 3. Development Workflow

```bash
# Start development with auto-reload
cargo watch -x "run --bin backend"  # Main app with auto-restart
./dev.sh start                      # All services with helper script

# Code quality checks
cargo fmt                   # Format code
cargo clippy -- -D warnings # Lint code
cargo audit                 # Security vulnerability scan

# Run tests in watch mode
cargo watch -x test                    # Unit tests
cargo watch -x "test -- --ignored"    # Integration tests

# Database operations
./dev.sh db                 # Connect to database
psql $DATABASE_URL -c "SELECT * FROM events ORDER BY created_at DESC LIMIT 10;"
```

### 4. Explore Documentation

**Core Documentation:**
- **[CLAUDE.md](CLAUDE.md)**: Architecture overview and development guidelines
- **[QUICK_START.md](QUICK_START.md)**: Fast setup for experienced developers
- **[README.md](README.md)**: Project overview and examples

**Service-Specific Documentation:**
- **[services/content_processing/README.md](services/content_processing/README.md)**: Content analysis and processing
- **[services/knowledge_graph/README.md](services/knowledge_graph/README.md)**: Graph database and algorithms
- **[services/realtime_communication/README.md](services/realtime_communication/README.md)**: WebSocket and real-time features

**Tutorials:**
- **[docs/tutorials/](docs/tutorials/)**: Step-by-step learning guides
- **[examples/](examples/)**: Working code examples in Rust and Python

### 5. Essential Development Commands

```bash
# Service Management
./dev.sh start          # Start all development services
./dev.sh mcp            # Start only MCP servers
./dev.sh clean          # Clean all build artifacts

# Testing
cargo test                                  # Unit tests
cargo test -- --ignored                    # Integration tests
cargo test --test end_to_end_workflow_test  # E2E tests
cargo test --test load_test -- --ignored   # Performance tests

# Database Management
./dev.sh db                                 # Connect to main database
psql $DATABASE_URL -c "\dt"                 # List all tables
./scripts/database-setup.sh                # Reinitialize database

# Microservice Operations
cd services/content_processing && cargo run     # Start content processing
cd services/knowledge_graph && cargo run        # Start knowledge graph
cd services/realtime_communication && cargo run # Start realtime comms

# Monitoring and Debugging
./dev.sh logs                                    # Tail all logs
curl http://localhost:8080/api/v1/health/detailed # System health
docker-compose logs -f                           # Container logs
```

## Getting Help

**If you encounter issues not covered in this guide:**

### Self-Diagnosis Steps
1. **Run validation**: `./scripts/validate-environment.sh`
2. **Check logs**: `./dev.sh logs` or `docker-compose logs -f`
3. **Verify services**: `curl http://localhost:8080/api/v1/health/detailed`
4. **Review error messages**: Often contain solution hints

### Common Resolution Patterns
- **Permission issues**: Check file permissions and user/group ownership
- **Network issues**: Verify ports aren't blocked by firewall
- **Version issues**: Ensure minimum versions are met
- **Path issues**: Check environment variables and shell PATH

### When Creating Issues
Include the following information:
- **Operating System**: Version and architecture (x86_64/ARM64)
- **Tool Versions**: Output of `rustc --version`, `python3 --version`, `docker --version`
- **Error Messages**: Complete error output, not just summaries
- **Environment**: Output of `./scripts/validate-environment.sh`
- **Steps to Reproduce**: Exact commands that trigger the issue
- **Configuration**: Relevant parts of `.env` file (without secrets)

### Development Community
- Check repository issues for existing solutions
- Review project documentation and examples
- Use discussion forums for questions and best practices

**Happy coding! The AI Workflow System is ready for your innovations! 🚀**