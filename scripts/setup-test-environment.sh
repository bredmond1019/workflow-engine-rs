#!/bin/bash

# Test Environment Setup Script
# Automatically starts all required services for testing

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get the project root directory
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SCRIPTS_DIR="$PROJECT_ROOT/scripts"
MCP_SERVERS_DIR="$PROJECT_ROOT/mcp-servers"

# PID tracking
declare -a PIDS=()
declare -a SERVICE_NAMES=()

# Function to print colored messages
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Function to check if a port is available
check_port() {
    local port=$1
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        return 1
    else
        return 0
    fi
}

# Cleanup function
cleanup() {
    print_status "Stopping all test services..."
    
    for i in "${!PIDS[@]}"; do
        if kill -0 "${PIDS[$i]}" 2>/dev/null; then
            print_status "Stopping ${SERVICE_NAMES[$i]} (PID: ${PIDS[$i]})"
            kill "${PIDS[$i]}" 2>/dev/null || true
        fi
    done
    
    # Wait for processes to terminate
    for pid in "${PIDS[@]}"; do
        wait "$pid" 2>/dev/null || true
    done
    
    print_status "All test services stopped."
}

# Set up trap to cleanup on script exit
trap cleanup EXIT INT TERM

# Function to start a service and track its PID
start_service() {
    local name=$1
    local command=$2
    local port=$3
    local check_startup=$4
    
    if [ -n "$port" ] && ! check_port "$port"; then
        print_warning "$name already running on port $port, skipping..."
        return 0
    fi
    
    print_status "Starting $name..."
    eval "$command" &
    local pid=$!
    
    PIDS+=("$pid")
    SERVICE_NAMES+=("$name")
    
    # Wait for service to start
    if [ "$check_startup" = "true" ] && [ -n "$port" ]; then
        local count=0
        while check_port "$port" && [ $count -lt 30 ]; do
            sleep 0.5
            count=$((count + 1))
        done
        
        if check_port "$port"; then
            print_error "$name failed to start on port $port"
            return 1
        fi
    else
        sleep 2
    fi
    
    print_status "$name started (PID: $pid)"
    return 0
}

# Check for required tools
print_status "Checking required tools..."

# Check for Python
if ! command -v python3 &> /dev/null; then
    print_error "Python 3 is required but not installed."
    exit 1
fi

# Check for uv (if using it for MCP servers)
if ! command -v uv &> /dev/null; then
    print_warning "uv not found, will try using python directly for MCP servers"
    USE_UV=false
else
    USE_UV=true
fi

# Start MCP test servers from scripts directory
print_status "Starting MCP test servers..."

cd "$SCRIPTS_DIR"

# Define MCP servers configuration (using simple arrays for compatibility)
MCP_SERVICES=("HelpScout:8001" "Notion:8002" "Slack:8003")

# Start multi-service MCP servers
for service_config in "${MCP_SERVICES[@]}"; do
    service="${service_config%:*}"
    port="${service_config#*:}"
    service_lower=$(echo "$service" | tr '[:upper:]' '[:lower:]')
    
    if [ "$USE_UV" = true ]; then
        start_service "$service MCP Server" \
            "uv run python multi_service_mcp_server.py --service $service_lower" \
            "" \
            "false"
    else
        start_service "$service MCP Server" \
            "python3 multi_service_mcp_server.py --service $service_lower" \
            "" \
            "false"
    fi
done

# Start Customer Support MCP server (stdio mode)
if [ "$USE_UV" = true ]; then
    start_service "Customer Support MCP Server" \
        "uv run python customer_support_server.py" \
        "" \
        "false"
else
    start_service "Customer Support MCP Server" \
        "python3 customer_support_server.py" \
        "" \
        "false"
fi

# Check if we should also start Python MCP servers from mcp-servers directory
if [ -d "$MCP_SERVERS_DIR" ] && [ -f "$MCP_SERVERS_DIR/start_servers.sh" ]; then
    print_status "Starting Python MCP servers from mcp-servers directory..."
    cd "$MCP_SERVERS_DIR"
    
    # Start the servers
    start_service "MCP Servers (Python)" \
        "./start_servers.sh" \
        "" \
        "false"
fi

# Export environment variables for tests
print_status "Setting up test environment variables..."

export MCP_TEST_SERVERS_RUNNING=1
export HELPSCOUT_MCP_URL="http://localhost:8001"
export NOTION_MCP_URL="http://localhost:8002"
export SLACK_MCP_URL="http://localhost:8003"

# Create a temporary env file for tests
ENV_FILE="$PROJECT_ROOT/.env.test"
cat > "$ENV_FILE" << EOF
# Test Environment Configuration
MCP_TEST_SERVERS_RUNNING=1
HELPSCOUT_MCP_URL=http://localhost:8001
NOTION_MCP_URL=http://localhost:8002
SLACK_MCP_URL=http://localhost:8003

# Database configuration for tests
DATABASE_URL=postgresql://postgres:postgres@localhost/ai_workflow_test
JWT_SECRET=test-jwt-secret-key-for-testing-only

# Optional AI provider keys (add your own if needed)
# OPENAI_API_KEY=your_key
# ANTHROPIC_API_KEY=your_key
EOF

print_status "Test environment file created at: $ENV_FILE"

# Summary
echo ""
print_status "Test environment setup complete!"
echo ""
echo "Active services:"
echo "  - HelpScout MCP Server: http://localhost:8001"
echo "  - Notion MCP Server: http://localhost:8002"
echo "  - Slack MCP Server: http://localhost:8003"
echo "  - Customer Support MCP Server: stdio mode"
echo ""
echo "Environment variables exported:"
echo "  - MCP_TEST_SERVERS_RUNNING=1"
echo "  - HELPSCOUT_MCP_URL, NOTION_MCP_URL, SLACK_MCP_URL"
echo ""
echo "To run tests:"
echo "  cargo test                              # Run unit tests"
echo "  cargo test -- --ignored                 # Run all integration tests"
echo "  cargo test external_mcp -- --ignored    # Run MCP integration tests"
echo ""
echo "To use the test environment file:"
echo "  source $ENV_FILE"
echo ""
print_warning "Press Ctrl+C to stop all services..."

# Keep the script running
wait