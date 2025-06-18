#!/bin/bash

# AI Workflow System - Environment Validation Script
# This script validates prerequisites and environment setup

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Track validation status
VALIDATION_PASSED=true

echo -e "${BLUE}AI Workflow System - Environment Validation${NC}"
echo "============================================="

# Function to print colored output
print_status() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
    VALIDATION_PASSED=false
}

print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_section() {
    echo ""
    echo -e "${BLUE}=== $1 ===${NC}"
}

# Check prerequisite tools
print_section "Checking Prerequisites"

# Check Rust
if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version | cut -d' ' -f2)
    print_status "Rust found: $RUST_VERSION"
    
    # Check minimum version (1.75+)
    RUST_MAJOR=$(echo $RUST_VERSION | cut -d'.' -f1)
    RUST_MINOR=$(echo $RUST_VERSION | cut -d'.' -f2)
    
    if [ "$RUST_MAJOR" -gt 1 ] || ([ "$RUST_MAJOR" -eq 1 ] && [ "$RUST_MINOR" -ge 75 ]); then
        print_status "Rust version meets requirements (1.75+)"
    else
        print_error "Rust version $RUST_VERSION is too old. Please upgrade to 1.75+"
    fi
else
    print_error "Rust not found. Please install from https://rustup.rs/"
fi

# Check Cargo
if command -v cargo &> /dev/null; then
    CARGO_VERSION=$(cargo --version | cut -d' ' -f2)
    print_status "Cargo found: $CARGO_VERSION"
else
    print_error "Cargo not found (should come with Rust)"
fi

# Check PostgreSQL
if command -v psql &> /dev/null; then
    PG_VERSION=$(psql --version | cut -d' ' -f3)
    print_status "PostgreSQL found: $PG_VERSION"
    
    # Check if PostgreSQL is running
    if pg_isready &> /dev/null; then
        print_status "PostgreSQL service is running"
    else
        print_warning "PostgreSQL service is not running"
        print_info "Start with: brew services start postgresql (macOS) or sudo systemctl start postgresql (Linux)"
    fi
else
    print_error "PostgreSQL not found. Please install PostgreSQL 15+"
fi

# Check Python
if command -v python3 &> /dev/null; then
    PYTHON_VERSION=$(python3 --version | cut -d' ' -f2)
    print_status "Python found: $PYTHON_VERSION"
    
    # Check minimum version (3.11+)
    PYTHON_MAJOR=$(echo $PYTHON_VERSION | cut -d'.' -f1)
    PYTHON_MINOR=$(echo $PYTHON_VERSION | cut -d'.' -f2)
    
    if [ "$PYTHON_MAJOR" -gt 3 ] || ([ "$PYTHON_MAJOR" -eq 3 ] && [ "$PYTHON_MINOR" -ge 11 ]); then
        print_status "Python version meets requirements (3.11+)"
    else
        if command -v uv &> /dev/null; then
            print_warning "Python version $PYTHON_VERSION is older than 3.11+, but uv can manage Python versions"
            print_info "uv will automatically use a compatible Python version for MCP servers"
        else
            print_error "Python version $PYTHON_VERSION is too old. Please upgrade to 3.11+"
        fi
    fi
else
    print_error "Python 3 not found. Please install Python 3.11+"
fi

# Check uv (Python package manager)
if command -v uv &> /dev/null; then
    UV_VERSION=$(uv --version | cut -d' ' -f2)
    print_status "uv found: $UV_VERSION"
else
    print_warning "uv not found. Install with: curl -LsSf https://astral.sh/uv/install.sh | sh"
    print_info "uv is required for Python MCP server dependencies"
fi

# Check environment file
print_section "Checking Environment Configuration"

if [ -f ".env" ]; then
    print_status ".env file found"
    
    # Check required environment variables
    required_vars=("DATABASE_URL" "JWT_SECRET" "HOST" "PORT")
    
    for var in "${required_vars[@]}"; do
        if grep -q "^$var=" .env; then
            value=$(grep "^$var=" .env | cut -d'=' -f2)
            if [ -n "$value" ] && [ "$value" != "your-secret-key-here-change-in-production" ]; then
                print_status "$var is set"
            else
                print_warning "$var is set but appears to be a placeholder value"
            fi
        else
            print_error "$var is not set in .env file"
        fi
    done
else
    print_error ".env file not found"
    print_info "Copy .env.example to .env and configure it"
    print_info "cp .env.example .env"
fi

# Check database connection
print_section "Checking Database Connection"

if [ -f ".env" ]; then
    # Source the .env file
    set -a
    source .env
    set +a
    
    if [ -n "$DATABASE_URL" ]; then
        print_info "Testing database connection..."
        if psql "$DATABASE_URL" -c "SELECT version();" &> /dev/null; then
            print_status "Database connection successful"
            
            # Check if required tables exist
            TABLES=("agents" "events" "sessions")
            for table in "${TABLES[@]}"; do
                if psql "$DATABASE_URL" -c "SELECT 1 FROM $table LIMIT 1;" &> /dev/null; then
                    print_status "Table '$table' exists and accessible"
                else
                    print_warning "Table '$table' does not exist or is not accessible"
                    print_info "Run: ./scripts/database-setup.sh"
                fi
            done
        else
            print_error "Database connection failed"
            print_info "Check your DATABASE_URL and ensure PostgreSQL is running"
            print_info "Run: ./scripts/database-setup.sh"
        fi
    else
        print_error "DATABASE_URL not set in .env file"
    fi
fi

# Check Rust project dependencies
print_section "Checking Rust Dependencies"

if [ -f "Cargo.toml" ]; then
    print_status "Cargo.toml found"
    
    print_info "Checking if project compiles..."
    if cargo check --quiet &> /dev/null; then
        print_status "Project compiles successfully"
    else
        print_error "Project compilation failed"
        print_info "Run 'cargo check' for detailed error information"
    fi
else
    print_error "Cargo.toml not found - not in a Rust project directory"
fi

# Check MCP server dependencies
print_section "Checking MCP Server Dependencies"

if [ -d "scripts" ] && [ -f "scripts/pyproject.toml" ]; then
    print_status "MCP servers found in scripts directory"
    
    cd scripts
    
    if [ -f "pyproject.toml" ]; then
        print_status "pyproject.toml found"
        
        # Check for required MCP server files
        mcp_files=("customer_support_server.py" "multi_service_mcp_server.py" "test_mcp_server.py")
        for file in "${mcp_files[@]}"; do
            if [ -f "$file" ]; then
                print_status "MCP server file '$file' found"
            else
                print_warning "MCP server file '$file' not found"
            fi
        done
        
        if command -v uv &> /dev/null; then
            print_info "Checking MCP server dependencies..."
            if uv sync --quiet &> /dev/null; then
                print_status "MCP server dependencies resolved"
            else
                print_warning "MCP server dependencies not fully resolved"
                print_info "Run: cd scripts && uv sync"
            fi
        else
            print_warning "Cannot check MCP dependencies without uv"
        fi
    else
        print_warning "MCP servers pyproject.toml not found"
    fi
    
    cd ..
else
    print_error "MCP servers not found in scripts directory"
fi

# Check Docker (optional)
print_section "Checking Optional Tools"

if command -v docker &> /dev/null; then
    DOCKER_VERSION=$(docker --version | cut -d' ' -f3 | sed 's/,//')
    print_status "Docker found: $DOCKER_VERSION"
    
    if docker info &> /dev/null; then
        print_status "Docker daemon is running"
    else
        print_warning "Docker daemon is not running"
    fi
else
    print_warning "Docker not found (optional for containerized development)"
fi

if command -v docker-compose &> /dev/null; then
    COMPOSE_VERSION=$(docker-compose --version | cut -d' ' -f4 | sed 's/,//')
    print_status "Docker Compose found: $COMPOSE_VERSION"
else
    print_warning "Docker Compose not found (optional)"
fi

# Final validation result
print_section "Validation Result"

if [ "$VALIDATION_PASSED" = true ]; then
    echo -e "${GREEN}✅ Environment validation passed!${NC}"
    echo ""
    echo "Your development environment is ready. Next steps:"
    echo "1. Run the application: cargo run --bin backend"
    echo "2. Test the API: curl http://localhost:8080/api/v1/health"
    echo "3. View logs: tail -f logs/*.log"
    echo ""
    exit 0
else
    echo -e "${RED}❌ Environment validation failed!${NC}"
    echo ""
    echo "Please fix the issues above before proceeding."
    echo "Common fixes:"
    echo "1. Install missing prerequisites"
    echo "2. Copy .env.example to .env and configure it"
    echo "3. Run: ./scripts/database-setup.sh"
    echo "4. Run: cd scripts && uv sync"
    echo ""
    exit 1
fi