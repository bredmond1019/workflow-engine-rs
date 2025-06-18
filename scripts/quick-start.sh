#!/bin/bash
# AI Workflow System - Quick Start Script
# This script provides a fast way to get the development environment running

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

print_status() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_header() {
    echo
    echo -e "${BLUE}═══════════════════════════════════════${NC}"
    echo -e "${BLUE}    AI Workflow System - Quick Start   ${NC}"
    echo -e "${BLUE}═══════════════════════════════════════${NC}"
    echo
}

# Main execution
cd "$PROJECT_ROOT"

print_header

# Step 1: Check prerequisites
print_info "Checking prerequisites..."

MISSING_PREREQS=false

if ! command -v rustc &> /dev/null; then
    print_error "Rust not found. Please install from https://rustup.rs/"
    MISSING_PREREQS=true
else
    print_status "Rust found: $(rustc --version | cut -d' ' -f2)"
fi

if ! command -v psql &> /dev/null; then
    print_error "PostgreSQL not found. Please install PostgreSQL 15+"
    MISSING_PREREQS=true
else
    print_status "PostgreSQL found: $(psql --version | cut -d' ' -f3)"
fi

if ! command -v python3 &> /dev/null; then
    print_error "Python 3 not found. Please install Python 3.11+"
    MISSING_PREREQS=true
else
    print_status "Python found: $(python3 --version | cut -d' ' -f2)"
fi

if [ "$MISSING_PREREQS" = true ]; then
    echo
    print_error "Missing prerequisites. Please install them and run again."
    print_info "For detailed instructions, see DEVELOPMENT_SETUP.md"
    exit 1
fi

# Step 2: Environment setup
print_info "Setting up environment configuration..."

if [ ! -f ".env" ]; then
    if [ -f ".env.example" ]; then
        cp .env.example .env
        print_status "Created .env file from template"
    else
        print_error ".env.example not found!"
        exit 1
    fi
else
    print_status ".env file already exists"
fi

# Step 3: Database check
print_info "Checking database connection..."

# Source .env file
set -a
source .env
set +a

if psql "$DATABASE_URL" -c "SELECT 1;" &> /dev/null; then
    print_status "Database connection successful"
else
    print_warning "Database connection failed. Running setup..."
    if [ -f "scripts/database-setup.sh" ]; then
        chmod +x scripts/database-setup.sh
        ./scripts/database-setup.sh
    else
        print_error "Database setup script not found!"
        exit 1
    fi
fi

# Step 4: Build check
print_info "Checking Rust build..."

if [ -d "target" ] && [ -f "target/debug/backend" ]; then
    print_status "Project already built"
else
    print_info "Building project (this may take a few minutes)..."
    cargo build
    print_status "Project built successfully"
fi

# Step 5: MCP servers setup
print_info "Setting up MCP servers..."

if [ -d "mcp-servers" ]; then
    # Check each MCP server subdirectory
    for server in notion-server helpscout-server slack-server; do
        if [ -d "mcp-servers/$server" ]; then
            print_status "Found $server"
        fi
    done
    
    # Install uv if not present
    if ! command -v uv &> /dev/null; then
        print_warning "uv not found. Installing..."
        curl -LsSf https://astral.sh/uv/install.sh | sh
        export PATH="$HOME/.local/bin:$PATH"
    fi
else
    print_warning "MCP servers directory not found"
fi

# Step 6: Create start script
cat > "$PROJECT_ROOT/start-dev.sh" << 'EOF'
#!/bin/bash
# Start development servers

echo "Starting AI Workflow System development environment..."

# Function to cleanup on exit
cleanup() {
    echo "Stopping servers..."
    kill $MCP_PID $APP_PID 2>/dev/null
    exit 0
}

trap cleanup EXIT INT TERM

# Start MCP servers in background
if [ -f "./scripts/start_test_servers.sh" ]; then
    echo "Starting MCP test servers..."
    ./scripts/start_test_servers.sh &
    MCP_PID=$!
    sleep 3
fi

# Start main application
echo "Starting main application..."
cargo run &
APP_PID=$!

echo ""
echo "Services started:"
echo "  Main API: http://localhost:8080"
echo "  Health Check: http://localhost:8080/api/v1/health"
echo "  Swagger UI: http://localhost:8080/swagger-ui/"
echo ""
echo "Press Ctrl+C to stop all services..."

# Wait for processes
wait $APP_PID
EOF

chmod +x "$PROJECT_ROOT/start-dev.sh"

# Final summary
echo
echo -e "${GREEN}═══════════════════════════════════════${NC}"
echo -e "${GREEN}        Quick Start Complete! 🚀       ${NC}"
echo -e "${GREEN}═══════════════════════════════════════${NC}"
echo
echo "Your development environment is ready!"
echo
echo "To start the application:"
echo "  ${GREEN}./start-dev.sh${NC}"
echo
echo "To test the setup:"
echo "  ${GREEN}curl http://localhost:8080/api/v1/health${NC}"
echo
echo "For detailed setup instructions:"
echo "  ${BLUE}cat DEVELOPMENT_SETUP.md${NC}"
echo
echo "For troubleshooting:"
echo "  ${BLUE}./scripts/validate-environment.sh${NC}"
echo

print_status "Quick start completed successfully!"