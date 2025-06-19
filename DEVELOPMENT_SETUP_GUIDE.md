# AI Workflow Engine - Complete Development Environment Setup Guide

This comprehensive guide provides step-by-step instructions for setting up the AI Workflow Engine development environment. This system is a production-ready AI workflow orchestration platform built in Rust with Python MCP servers, featuring microservices architecture, event sourcing, and comprehensive monitoring.

**Estimated Setup Time:** 20-40 minutes (depending on your existing tools and internet speed)

## Table of Contents

- [Prerequisites and System Requirements](#prerequisites-and-system-requirements)
- [Quick Start (Automated Setup)](#quick-start-automated-setup)
- [Manual Setup Instructions](#manual-setup-instructions)
- [Testing and Verification](#testing-and-verification)
- [Development Workflow](#development-workflow)
- [Troubleshooting](#troubleshooting)
- [Known Issues and Solutions](#known-issues-and-solutions)
- [Development Best Practices](#development-best-practices)

## Prerequisites and System Requirements

### Hardware Requirements

**Minimum:**
- CPU: 2 cores, 2.0 GHz
- RAM: 8 GB
- Storage: 4 GB free space
- Network: Internet connection

**Recommended:**
- CPU: 4+ cores, 3.0 GHz (for microservices development)
- RAM: 16 GB (allows running full stack with monitoring)
- Storage: 10 GB free space (includes Docker images)
- Network: Stable broadband connection

### Operating System Support

| OS | Version | Status | Notes |
|---|---|---|---|
| **macOS** | 11+ (Big Sur and later) | ✅ **Recommended** | Best development experience |
| **Ubuntu** | 20.04+ | ✅ Fully Supported | |
| **Debian** | 11+ | ✅ Fully Supported | |
| **Fedora** | 36+ | ✅ Supported | |
| **CentOS Stream** | 9+ | ✅ Supported | |
| **Windows 11** | with WSL2 | ⚠️ Limited Support | Use Ubuntu 20.04+ in WSL |

**Architecture Support:**
- x86_64 (Intel/AMD) ✅
- ARM64 (Apple Silicon M1/M2/M3) ✅

### Required Software

| Tool | Min Version | Purpose | Auto-Install |
|------|-------------|---------|--------------|
| **Rust** | 1.75+ | Main application language | ✅ |
| **PostgreSQL** | 15+ | Primary database | ✅ |
| **Python** | 3.10+ | MCP servers | ✅ |
| **Git** | 2.30+ | Version control | ❌ Manual |
| **uv** | Latest | Python package manager | ✅ |

### Optional Software

| Tool | Purpose | Auto-Install |
|------|---------|--------------|
| **Docker** | Containerization, Dgraph | ❌ Manual |
| **Docker Compose** | Multi-service orchestration | ❌ Manual |
| **curl** | API testing | Usually pre-installed |

## Quick Start (Automated Setup)

For experienced developers who want to get running quickly:

```bash
# 1. Clone the repository
git clone https://github.com/bredmond1019/workflow-engine-rs.git
cd workflow-engine-rs

# 2. Run automated setup (handles everything)
chmod +x scripts/setup.sh
./scripts/setup.sh

# 3. If successful, skip to Testing section
# If issues occur, continue with Manual Setup
```

**The automated script handles:**
- ✅ Installing Rust, PostgreSQL, Python, and uv
- ✅ Setting up PostgreSQL database and schema
- ✅ Configuring environment variables
- ✅ Building the Rust application
- ✅ Installing Python MCP server dependencies
- ✅ Creating development helper scripts
- ✅ Running comprehensive validation

## Manual Setup Instructions

Follow these steps if automated setup fails or you prefer manual control.

### Step 1: Install Prerequisites

#### 1.1 Install Rust

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow prompts, then reload shell
source $HOME/.cargo/env

# Verify installation
rustc --version  # Should show 1.75+
cargo --version

# Install development tools
cargo install cargo-watch    # Auto-rebuild on changes
cargo install cargo-audit    # Security scanning
```

#### 1.2 Install PostgreSQL

**macOS (Homebrew):**
```bash
# Install PostgreSQL
brew install postgresql@15
brew services start postgresql@15

# Add to PATH (add to ~/.zshrc or ~/.bash_profile)
export PATH="/opt/homebrew/opt/postgresql@15/bin:$PATH"

# Verify installation
psql --version
```

**Ubuntu/Debian:**
```bash
# Update package list
sudo apt update

# Install PostgreSQL
sudo apt install postgresql-15 postgresql-contrib-15

# Start and enable service
sudo systemctl start postgresql
sudo systemctl enable postgresql

# Verify installation
psql --version
```

**Fedora/RHEL:**
```bash
# Install PostgreSQL
sudo dnf install postgresql15-server postgresql15-contrib

# Initialize database
sudo postgresql-setup --initdb

# Start and enable service
sudo systemctl start postgresql
sudo systemctl enable postgresql
```

#### 1.3 Install Python and uv

**macOS:**
```bash
# Install Python 3.11+
brew install python@3.11

# Install uv (fast Python package manager)
curl -LsSf https://astral.sh/uv/install.sh | sh
```

**Ubuntu/Debian:**
```bash
# Install Python 3.11+
sudo apt install python3.11 python3.11-venv python3-pip

# Install uv
curl -LsSf https://astral.sh/uv/install.sh | sh
```

**Verify installations:**
```bash
python3 --version  # Should show 3.10+
uv --version
```

### Step 2: Clone and Configure Project

```bash
# Clone repository
git clone https://github.com/bredmond1019/workflow-engine-rs.git
cd workflow-engine-rs

# Copy environment configuration
cp .env.example .env

# Edit .env file with your preferred editor
# Key settings to review:
nano .env  # or vim, code, etc.
```

**Essential Environment Variables:**

```env
# Database Configuration
DATABASE_URL=postgres://aiworkflow:aiworkflow123@localhost:5432/ai_workflow
DB_USER=aiworkflow
DB_PASSWORD=aiworkflow123
DB_NAME=ai_workflow

# Application Configuration
HOST=127.0.0.1
PORT=8080
RUST_LOG=info

# Security (CHANGE IN PRODUCTION)
JWT_SECRET=development-jwt-secret-key-please-change-in-production-2024

# MCP Server URLs (for local development)
NOTION_MCP_URL=ws://localhost:3001
SLACK_MCP_URL=ws://localhost:3002
HELPSCOUT_MCP_URL=ws://localhost:3003
```

### Step 3: Database Setup

```bash
# Run database setup script
chmod +x scripts/database-setup.sh
./scripts/database-setup.sh
```

**Manual Database Setup (if script fails):**
```bash
# Connect as postgres superuser
sudo -u postgres psql

# Create user and database
CREATE USER aiworkflow WITH PASSWORD 'aiworkflow123';
CREATE DATABASE ai_workflow OWNER aiworkflow;
ALTER USER aiworkflow CREATEDB;
ALTER USER aiworkflow WITH SUPERUSER;  -- Development only
\q

# Initialize schema
psql -U aiworkflow -d ai_workflow -f scripts/init-db.sql
```

### Step 4: Build Application

```bash
# Download dependencies
cargo fetch

# Build main application
cargo build

# Note: There are currently some compilation warnings
# The core system compiles and runs with warnings
# See Known Issues section for details
```

### Step 5: Setup Python MCP Servers

```bash
# Navigate to scripts directory
cd scripts

# Install Python dependencies
uv sync

# Test MCP server installation
uv run python test_mcp_server.py

# Return to project root
cd ..
```

### Step 6: Validation and Testing

```bash
# Run comprehensive environment validation
chmod +x scripts/validate-environment.sh
./scripts/validate-environment.sh
```

## Testing and Verification

### Basic System Health Check

```bash
# 1. Start PostgreSQL (if not running)
# macOS: brew services start postgresql@15
# Linux: sudo systemctl start postgresql

# 2. Verify database connection
psql postgres://aiworkflow:aiworkflow123@localhost:5432/ai_workflow -c "SELECT version();"

# 3. Test Rust compilation
cargo check -p workflow-engine-core  # Core library should compile

# 4. Test Python MCP servers
cd scripts && uv run python test_mcp_server.py && cd ..
```

### Running the Application

**Current Status:** The application has some compilation issues in the API layer that need to be resolved. The core engine compiles successfully.

```bash
# Try building core components individually
cargo check -p workflow-engine-core     # ✅ Works with warnings
cargo check -p workflow-engine-mcp      # ✅ Works with warnings  
cargo check -p workflow-engine-nodes    # ❌ Has compilation errors
cargo check -p workflow-engine-api      # ❌ Has compilation errors
cargo check -p workflow-engine-app      # ❌ Depends on broken components
```

### Testing MCP Servers

```bash
# Start MCP test servers
./scripts/start_test_servers.sh

# Test individual servers
cd scripts
uv run python customer_support_server.py --test
uv run python multi_service_mcp_server.py --test
cd ..
```

### Docker Setup (Alternative)

If you prefer containerized development:

```bash
# Ensure Docker is installed and running
docker --version
docker-compose --version

# Start full development stack
docker-compose up -d

# Check service status
docker-compose ps
docker-compose logs -f ai-workflow-system
```

## Development Workflow

### Starting Development

```bash
# Option 1: Use helper script (created by setup.sh)
./dev.sh start    # Starts MCP servers + main app with auto-reload

# Option 2: Manual startup
# Terminal 1: Start MCP servers
./scripts/start_test_servers.sh

# Terminal 2: Start main application (when compilation issues are fixed)
cargo run -p workflow-engine-app

# Terminal 3: Start microservices (optional)
cd services/content_processing && cargo run
cd services/knowledge_graph && cargo run  
cd services/realtime_communication && cargo run
```

### Development Commands

```bash
# Code quality
cargo fmt                           # Format code
cargo clippy -- -D warnings         # Lint code
cargo audit                         # Security scan

# Testing
cargo test                          # Unit tests
cargo test -- --ignored            # Integration tests (requires MCP servers)

# Database operations
./dev.sh db                         # Connect to database
psql $DATABASE_URL -c "\dt"         # List tables

# Helper commands
./dev.sh test                       # Run tests
./dev.sh mcp                        # Start only MCP servers
./dev.sh logs                       # Tail logs
./dev.sh clean                      # Clean build artifacts
```

## Troubleshooting

### Environment Validation

**Always start with the validation script:**
```bash
./scripts/validate-environment.sh
```

This checks:
- ✅ Prerequisites installation and versions
- ✅ Environment variable configuration
- ✅ Database connectivity and schema
- ❌ Project compilation (currently failing)
- ✅ MCP server dependencies

### Common Issues and Solutions

#### 1. PostgreSQL Connection Issues

**Error:** `FATAL: password authentication failed`

**Solutions:**
```bash
# Check PostgreSQL status
pg_isready -h localhost -p 5432

# Start PostgreSQL
# macOS: brew services start postgresql@15
# Linux: sudo systemctl start postgresql

# Reset user password
sudo -u postgres psql
ALTER USER aiworkflow WITH PASSWORD 'aiworkflow123';
\q

# Re-run database setup
./scripts/database-setup.sh
```

#### 2. Rust Compilation Issues

**Error:** `error: linker 'cc' not found`

**Solutions:**
```bash
# macOS: Install Xcode Command Line Tools
xcode-select --install

# Ubuntu/Debian: Install build essentials
sudo apt install build-essential pkg-config libssl-dev

# Fedora/RHEL: Install development tools
sudo dnf install gcc gcc-c++ make openssl-devel
```

#### 3. Port Conflicts

**Error:** `Address already in use (os error 48)`

**Solutions:**
```bash
# Find what's using the port
lsof -i :8080

# Kill the process
kill -9 <PID>

# Or change port in .env
PORT=8081
```

#### 4. MCP Server Issues

**Error:** `ModuleNotFoundError: No module named 'mcp'`

**Solutions:**
```bash
# Reinstall MCP dependencies
cd scripts
uv sync
# Or manually: pip install mcp[cli]

# Test individual server
uv run python customer_support_server.py --test
```

## Known Issues and Solutions

### Current Compilation Issues

**Status:** The project has some compilation errors in the API and nodes packages that need to be resolved.

**Working Components:**
- ✅ Core engine (`workflow-engine-core`)
- ✅ MCP framework (`workflow-engine-mcp`) 
- ✅ Python MCP servers
- ✅ Database setup and connection
- ✅ Environment configuration

**Components with Issues:**
- ❌ API layer (`workflow-engine-api`) - Type errors and missing imports
- ❌ Nodes package (`workflow-engine-nodes`) - Dependency issues
- ❌ Main application (`workflow-engine-app`) - Depends on broken components

**Immediate Development Options:**

1. **Work on Core Engine:**
   ```bash
   cd crates/workflow-engine-core
   cargo test    # Run core tests
   cargo run --example basic_workflow  # Run examples
   ```

2. **Work on MCP Integration:**
   ```bash
   cd scripts
   uv run python test_mcp_server.py    # Test MCP servers
   uv run python customer_support_server.py --test
   ```

3. **Work on Microservices:**
   ```bash
   cd services/content_processing && cargo build  # Should work
   cd services/knowledge_graph && cargo build     # Should work
   cd services/realtime_communication && cargo build  # Should work
   ```

### Fixing Compilation Issues

The main compilation errors are related to:

1. **Missing imports and type mismatches** in the API layer
2. **Dependency version conflicts** between workspace crates
3. **Unused imports** causing warnings (non-blocking)

**Recommended approach:**
1. Start with core components that work
2. Fix API layer compilation errors gradually
3. Run tests incrementally as components are fixed
4. Use Docker setup as alternative for full-stack development

## Development Best Practices

### Code Quality

```bash
# Before committing code
cargo fmt                    # Format
cargo clippy -- -D warnings  # Lint
cargo test                   # Test
cargo audit                  # Security
```

### Database Management

```bash
# Backup database
pg_dump $DATABASE_URL > backup.sql

# Reset database
./scripts/database-setup.sh

# Connect to database
psql $DATABASE_URL
```

### MCP Server Development

```bash
# Test MCP protocol compliance
cd scripts
uv run python test_mcp_server.py

# Test specific functionality
uv run python customer_support_server.py --test
```

### Monitoring and Debugging

```bash
# View application logs
./dev.sh logs

# Check system health (when app is running)
curl http://localhost:8080/api/v1/health

# Monitor database activity
psql $DATABASE_URL -c "SELECT * FROM pg_stat_activity;"
```

## Next Steps

Once the development environment is set up:

1. **Review Documentation:**
   - `README.md` - Project overview
   - `CLAUDE.md` - Architecture and guidelines
   - `docs/tutorials/` - Learning guides

2. **Explore Examples:**
   - `examples/` - Working code examples
   - `services/*/examples/` - Service-specific examples

3. **Fix Compilation Issues:**
   - Focus on `workflow-engine-api` package first
   - Resolve import and type errors
   - Enable full application build

4. **Start Development:**
   - Create feature branches
   - Write tests first
   - Use provided development tools

## Getting Help

### Self-Diagnosis
1. Run `./scripts/validate-environment.sh`
2. Check logs with `./dev.sh logs`
3. Verify services with health checks
4. Review error messages carefully

### When Creating Issues
Include:
- **OS and version**
- **Tool versions** (`rustc --version`, `python3 --version`)
- **Complete error output**
- **Steps to reproduce**
- **Environment validation results**

---

**Happy coding! The AI Workflow Engine development environment is ready for your contributions! 🚀**

*This guide is continuously updated. Please report issues and suggest improvements.*