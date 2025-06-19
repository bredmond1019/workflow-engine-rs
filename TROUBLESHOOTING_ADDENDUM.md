# Development Setup Troubleshooting Addendum

This document provides specific error details and solutions discovered during comprehensive testing of the development environment setup.

## Quick Diagnosis Commands

```bash
# Always start with these commands for troubleshooting
./scripts/validate-environment.sh              # Comprehensive validation
cargo check -p workflow-engine-core           # Test core compilation
cd scripts && uv run python -c "import mcp"   # Test MCP availability
psql $DATABASE_URL -c "SELECT version();"     # Test database
```

## Specific Error Catalog

### Rust Compilation Errors

#### 1. Main Workspace Build Failure
```bash
# Command: cargo build
# Error: Multiple compilation errors across workspace crates

# Root Cause: Type mismatches and import issues in API layer
# Status: Blocks main application from running
# Workaround: Build individual packages that work
```

**Working Packages:**
```bash
cargo check -p workflow-engine-core  # ✅ Builds with warnings only
```

**Failing Packages:**
```bash
cargo check -p workflow-engine-api    # ❌ 52 compilation errors
cargo check -p workflow-engine-nodes  # ❌ Multiple dependency issues
cargo check -p workflow-engine-app    # ❌ Depends on broken packages
```

#### 2. Binary Target Confusion
```bash
# Command: cargo run --bin backend
# Error: no bin target named `backend` in default-run packages

# Solution: Use correct binary name
cargo run -p workflow-engine-app --bin workflow-engine
```

#### 3. Workspace Configuration Issues
```bash
# Error in services: current package believes it's in a workspace when it's not
# Services are excluded from workspace but reference workspace dependencies

# Current workspace excludes:
# exclude = ["services/*", "mcp-servers/*"]
```

### Database Issues

#### 1. PostgreSQL Not Running
```bash
# Error: could not connect to server: No such file or directory
# Solutions:
brew services start postgresql@15    # macOS
sudo systemctl start postgresql      # Linux
```

#### 2. Authentication Issues
```bash
# Error: FATAL: password authentication failed for user "aiworkflow"
# Solution: Reset password and re-run setup
sudo -u postgres psql
ALTER USER aiworkflow WITH PASSWORD 'aiworkflow123';
\q
./scripts/database-setup.sh
```

### MCP Server Issues

#### 1. Broken Pipe Error
```bash
# Error when running: cd scripts && uv run python test_mcp_server.py
# Output: ERROR - ❌ Test failed: [Errno 32] Broken pipe

# Cause: Communication issue in test script
# Status: MCP servers start correctly but test communication fails
# Workaround: Test servers individually
./scripts/start_test_servers.sh  # This works correctly
```

#### 2. Python Version Warnings
```bash
# Warning: Python version 3.9.6 is older than 3.11+
# Impact: MCP servers work, but warnings appear
# Solution: uv manages Python versions automatically
```

### Environment Configuration

#### 1. Missing .env File
```bash
# Error: Environment variables not found
# Solution: Copy from example
cp .env.example .env
# Edit with your specific values
```

#### 2. Port Conflicts
```bash
# Error: Address already in use (os error 48)
# Find what's using the port:
lsof -i :8080
# Kill the process or change port in .env
```

## Verified Working Components

### ✅ Database Setup
```bash
# These commands work perfectly:
./scripts/database-setup.sh           # Creates database and user
psql $DATABASE_URL -c "\dt"          # Lists tables successfully
psql $DATABASE_URL -c "SELECT COUNT(*) FROM events;"  # Queries work
```

### ✅ MCP Server Infrastructure
```bash
# These work correctly:
cd scripts && uv sync                 # Installs dependencies
./scripts/start_test_servers.sh       # Starts all MCP servers
# Individual server tests:
uv run python customer_support_server.py --test  # Partial success
```

### ✅ Core Rust Engine
```bash
# Core engine compiles with warnings only:
cargo check -p workflow-engine-core
cargo build -p workflow-engine-core
# Examples in core package can be run (when fixed)
```

## Development Workarounds

### 1. Core Engine Development
```bash
# Work directly on the core engine
cd crates/workflow-engine-core
cargo build
cargo test  # Some tests work
```

### 2. MCP Server Development
```bash
# Develop MCP integrations independently
cd scripts
uv run python customer_support_server.py
# Modify and test MCP servers
```

### 3. Database Development
```bash
# Work with database directly
psql $DATABASE_URL
# Create migrations, test queries
```

### 4. Docker Alternative
```bash
# Use containerized development
docker-compose up -d database  # Start just database
# Develop against containerized services
```

## Fixing Strategy

### Phase 1: Enable Basic Compilation
1. **Fix workflow-engine-api compilation errors**
   - Resolve missing imports
   - Fix type mismatches
   - Update dependency references

2. **Fix workflow-engine-nodes issues**
   - Resolve workspace dependency conflicts
   - Fix import statements

### Phase 2: Enable Application Build
1. **Fix workflow-engine-app**
   - Resolve dependency chain issues
   - Enable main binary compilation

2. **Test main application**
   - Verify basic startup
   - Test API endpoints

### Phase 3: Enable Full Testing
1. **Fix unit tests**
   - Resolve test compilation issues
   - Verify test infrastructure

2. **Fix integration tests**
   - Resolve MCP communication issues
   - Enable full test suite

## Recovery Commands

### Complete Environment Reset
```bash
# Nuclear option: start fresh
cargo clean
rm -rf target/
./scripts/database-setup.sh
cd scripts && uv sync && cd ..
./scripts/validate-environment.sh
```

### Partial Reset Options
```bash
# Database only
./scripts/database-setup.sh

# Dependencies only
cargo clean && cargo fetch
cd scripts && uv sync && cd ..

# Configuration only
cp .env.example .env
# Edit .env with your values
```

## Current Status Summary

| Component | Status | Can Develop | Notes |
|-----------|--------|-------------|--------|
| Database | ✅ Working | Yes | Full functionality |
| MCP Servers | ✅ Working | Yes | Some test issues |
| Core Engine | ⚠️ Compiles | Yes | Warnings only |
| API Layer | ❌ Broken | No | Compilation errors |
| Main App | ❌ Broken | No | Dependency issues |
| Tests | ❌ Broken | No | Compilation issues |
| Documentation | ✅ Complete | Yes | Comprehensive guides |

## Next Steps for Developers

### If You Want to Start Development Now:
1. **Choose MCP development:** Work on Python MCP servers
2. **Choose core development:** Work on workflow engine core
3. **Choose documentation:** Improve guides and examples
4. **Choose infrastructure:** Work on database schemas and migrations

### If You Want to Fix the Build:
1. **Start with API layer:** Focus on `workflow-engine-api` compilation errors
2. **Use incremental approach:** Fix one error at a time
3. **Test frequently:** Use `cargo check` after each fix
4. **Document fixes:** Update this troubleshooting guide

---

*This document will be updated as issues are resolved and new ones are discovered.*