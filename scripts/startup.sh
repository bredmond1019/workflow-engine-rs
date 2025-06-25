#!/bin/bash
#
# AI Workflow Engine - Comprehensive Startup Script
# 
# This script handles the complete startup process for the AI Workflow Engine,
# including all prerequisites, databases, services, and health checks.
#
# Usage:
#   ./startup.sh [dev|prod] [options]
#
# Options:
#   --skip-checks     Skip prerequisite checks
#   --skip-build      Skip building the backend
#   --skip-frontend   Skip starting the frontend
#   --no-monitoring   Disable monitoring services (Prometheus, Grafana, Jaeger)
#   --detached        Run services in detached mode
#   --clean           Clean start (remove volumes and rebuild)
#

set -e  # Exit on error

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
FRONTEND_DIR="$PROJECT_ROOT/frontend"
LOG_DIR="$PROJECT_ROOT/logs"
MODE="${1:-dev}"  # Default to development mode

# Process command line arguments
SKIP_CHECKS=false
SKIP_BUILD=false
SKIP_FRONTEND=false
NO_MONITORING=false
DETACHED=false
CLEAN_START=false

while [[ $# -gt 0 ]]; do
    case $1 in
        dev|prod)
            MODE=$1
            ;;
        --skip-checks)
            SKIP_CHECKS=true
            ;;
        --skip-build)
            SKIP_BUILD=true
            ;;
        --skip-frontend)
            SKIP_FRONTEND=true
            ;;
        --no-monitoring)
            NO_MONITORING=true
            ;;
        --detached)
            DETACHED=true
            ;;
        --clean)
            CLEAN_START=true
            ;;
        *)
            ;;
    esac
    shift
done

# Function to print colored output
print_status() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check if a port is in use
port_in_use() {
    lsof -i:$1 >/dev/null 2>&1
}

# Function to wait for a service to be healthy
wait_for_service() {
    local service_name=$1
    local check_command=$2
    local max_attempts=${3:-30}
    local attempt=0
    
    print_status "Waiting for $service_name to be healthy..."
    
    while [ $attempt -lt $max_attempts ]; do
        if eval "$check_command" >/dev/null 2>&1; then
            print_success "$service_name is healthy"
            return 0
        fi
        attempt=$((attempt + 1))
        echo -n "."
        sleep 2
    done
    
    print_error "$service_name failed to become healthy"
    return 1
}

# Function to cleanup on exit
cleanup() {
    print_status "Cleaning up..."
    
    # Stop any running Python MCP servers
    if [ -n "$MCP_PIDS" ]; then
        print_status "Stopping MCP servers..."
        for pid in $MCP_PIDS; do
            kill $pid 2>/dev/null || true
        done
    fi
    
    # Stop frontend dev server if running
    if [ -n "$FRONTEND_PID" ]; then
        print_status "Stopping frontend server..."
        kill $FRONTEND_PID 2>/dev/null || true
    fi
}

# Set up trap for cleanup
trap cleanup EXIT

# Print banner
echo "=============================================="
echo "   AI Workflow Engine - Startup Script"
echo "   Mode: $MODE"
echo "=============================================="
echo

# Create necessary directories
print_status "Creating necessary directories..."
mkdir -p "$LOG_DIR"
mkdir -p "$PROJECT_ROOT/workflows"

# Step 1: Check prerequisites
if [ "$SKIP_CHECKS" = false ]; then
    print_status "Checking prerequisites..."
    
    # Check Docker
    if ! command_exists docker; then
        print_error "Docker is not installed. Please install Docker: https://docs.docker.com/get-docker/"
        exit 1
    fi
    
    # Check Docker Compose
    if ! command_exists docker-compose && ! docker compose version >/dev/null 2>&1; then
        print_error "Docker Compose is not installed. Please install Docker Compose."
        exit 1
    fi
    
    # Check Rust
    if ! command_exists cargo; then
        print_warning "Rust is not installed. Installing via rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi
    
    # Check Node.js
    if ! command_exists node; then
        print_error "Node.js is not installed. Please install Node.js: https://nodejs.org/"
        exit 1
    fi
    
    # Check Python
    if ! command_exists python3; then
        print_error "Python 3 is not installed. Please install Python 3."
        exit 1
    fi
    
    # Check for required ports
    REQUIRED_PORTS=(5432 6379 8080 8081 8082 3002)
    if [ "$NO_MONITORING" = false ]; then
        REQUIRED_PORTS+=(9090 3000 16686)
    fi
    
    for port in "${REQUIRED_PORTS[@]}"; do
        if port_in_use $port; then
            print_warning "Port $port is already in use. This may cause conflicts."
        fi
    done
    
    print_success "All prerequisites checked"
fi

# Step 2: Environment setup
print_status "Setting up environment variables..."

# Load .env file if it exists
if [ -f "$PROJECT_ROOT/.env" ]; then
    export $(cat "$PROJECT_ROOT/.env" | grep -v '^#' | xargs)
else
    print_warning ".env file not found. Using default values."
    # Create a basic .env file
    cat > "$PROJECT_ROOT/.env" <<EOF
# Database Configuration
DB_USER=aiworkflow
DB_PASSWORD=aiworkflow123
DB_NAME=ai_workflow
DATABASE_URL=postgres://\${DB_USER}:\${DB_PASSWORD}@localhost:5432/\${DB_NAME}

# JWT Configuration
JWT_SECRET=your-secret-key-here-$(openssl rand -base64 32)

# Redis Configuration
REDIS_PASSWORD=redis123

# Service Ports
API_PORT=8080
CONTENT_PROCESSING_PORT=8082
KNOWLEDGE_GRAPH_PORT=3002
REALTIME_COMM_PORT=8081

# Monitoring Ports
PROMETHEUS_PORT=9090
GRAFANA_PORT=3000
JAEGER_UI_PORT=16686

# Logging
RUST_LOG=info

# AI Provider Keys (optional)
# OPENAI_API_KEY=your_key_here
# ANTHROPIC_API_KEY=your_key_here
EOF
    print_success "Created default .env file"
fi

# Step 3: Clean start if requested
if [ "$CLEAN_START" = true ]; then
    print_status "Performing clean start..."
    docker-compose down -v 2>/dev/null || true
    print_success "Cleaned up previous containers and volumes"
fi

# Step 4: Start databases and infrastructure
print_status "Starting databases and infrastructure services..."

# Prepare docker-compose command
COMPOSE_CMD="docker-compose"
if [ "$MODE" = "prod" ]; then
    COMPOSE_CMD="$COMPOSE_CMD -f docker-compose.yml -f docker-compose.prod.yml"
fi

if [ "$NO_MONITORING" = true ]; then
    COMPOSE_PROFILES=""
else
    COMPOSE_PROFILES="--profile monitoring"
fi

# Start core infrastructure services
$COMPOSE_CMD up -d postgres redis

# Wait for PostgreSQL
wait_for_service "PostgreSQL" "docker exec ai-workflow-db pg_isready -U ${DB_USER:-aiworkflow}"

# Wait for Redis
wait_for_service "Redis" "docker exec ai-workflow-redis redis-cli -a ${REDIS_PASSWORD:-redis123} ping"

# Step 5: Start Dgraph for Knowledge Graph service
if [ -d "$PROJECT_ROOT/services/knowledge_graph/dgraph" ]; then
    print_status "Starting Dgraph for Knowledge Graph service..."
    cd "$PROJECT_ROOT/services/knowledge_graph/dgraph"
    docker-compose up -d
    cd "$PROJECT_ROOT"
    wait_for_service "Dgraph" "curl -s http://localhost:8080/health"
fi

# Step 6: Start monitoring services
if [ "$NO_MONITORING" = false ]; then
    print_status "Starting monitoring services..."
    $COMPOSE_CMD up -d prometheus grafana jaeger
    
    wait_for_service "Prometheus" "curl -s http://localhost:${PROMETHEUS_PORT:-9090}/-/healthy"
    wait_for_service "Grafana" "curl -s http://localhost:${GRAFANA_PORT:-3000}/api/health"
    wait_for_service "Jaeger" "curl -s http://localhost:${JAEGER_UI_PORT:-16686}/"
fi

# Step 7: Start MCP servers
print_status "Starting MCP test servers..."

# Check if MCP server scripts exist
if [ -f "$SCRIPT_DIR/start_test_servers.sh" ]; then
    cd "$SCRIPT_DIR"
    
    # Start MCP servers and capture PIDs
    if [ -f "customer_support_server.py" ] && [ -f "multi_service_mcp_server.py" ]; then
        python3 customer_support_server.py &
        MCP_PIDS="$!"
        
        python3 multi_service_mcp_server.py --service notion &
        MCP_PIDS="$MCP_PIDS $!"
        
        python3 multi_service_mcp_server.py --service helpscout &
        MCP_PIDS="$MCP_PIDS $!"
        
        python3 multi_service_mcp_server.py --service slack &
        MCP_PIDS="$MCP_PIDS $!"
        
        sleep 3
        print_success "MCP servers started"
    else
        print_warning "MCP server scripts not found. Skipping MCP server startup."
    fi
    
    cd "$PROJECT_ROOT"
else
    print_warning "MCP server startup script not found. Skipping MCP servers."
fi

# Step 8: Build and start microservices
print_status "Starting microservices..."

$COMPOSE_CMD up -d content-processing knowledge-graph realtime-communication

# Wait for microservices to be healthy
wait_for_service "Content Processing" "curl -s http://localhost:${CONTENT_PROCESSING_PORT:-8082}/health"
wait_for_service "Knowledge Graph" "curl -s http://localhost:${KNOWLEDGE_GRAPH_PORT:-3002}/health"
wait_for_service "Realtime Communication" "curl -s http://localhost:${REALTIME_COMM_PORT:-8081}/health"

# Step 9: Build and start the main backend
if [ "$SKIP_BUILD" = false ]; then
    print_status "Building the backend..."
    cargo build --release
    print_success "Backend built successfully"
fi

print_status "Starting the main AI Workflow Engine..."
$COMPOSE_CMD up -d ai-workflow-system

# Wait for the main service to be healthy
wait_for_service "AI Workflow Engine" "curl -s http://localhost:${API_PORT:-8080}/api/v1/health"

# Step 10: Start the frontend
if [ "$SKIP_FRONTEND" = false ] && [ -d "$FRONTEND_DIR" ]; then
    print_status "Starting the frontend..."
    cd "$FRONTEND_DIR"
    
    # Install dependencies if needed
    if [ ! -d "node_modules" ]; then
        print_status "Installing frontend dependencies..."
        npm install
    fi
    
    # Start the frontend
    if [ "$DETACHED" = true ]; then
        npm run dev > "$LOG_DIR/frontend.log" 2>&1 &
        FRONTEND_PID=$!
        print_success "Frontend started in background (PID: $FRONTEND_PID)"
    else
        print_status "Starting frontend in foreground mode..."
        npm run dev &
        FRONTEND_PID=$!
    fi
    
    cd "$PROJECT_ROOT"
fi

# Step 11: Health check summary
print_status "Performing final health checks..."
echo

echo "Service Status:"
echo "=============================================="

# Check all services
services=(
    "PostgreSQL|5432|Database"
    "Redis|6379|Cache"
    "AI Workflow Engine|8080|Main API"
    "Content Processing|8082|Microservice"
    "Knowledge Graph|3002|Microservice"
    "Realtime Communication|8081|WebSocket Service"
)

if [ "$NO_MONITORING" = false ]; then
    services+=(
        "Prometheus|9090|Metrics"
        "Grafana|3000|Dashboards"
        "Jaeger|16686|Tracing"
    )
fi

for service in "${services[@]}"; do
    IFS='|' read -r name port desc <<< "$service"
    if curl -s "http://localhost:$port" >/dev/null 2>&1 || curl -s "http://localhost:$port/health" >/dev/null 2>&1; then
        printf "%-25s ${GREEN}[RUNNING]${NC} - %s\n" "$name" "$desc"
    else
        printf "%-25s ${RED}[DOWN]${NC} - %s\n" "$name" "$desc"
    fi
done

echo "=============================================="
echo

# Print access information
print_success "AI Workflow Engine is ready!"
echo
echo "Access Points:"
echo "- Main API: http://localhost:${API_PORT:-8080}"
echo "- API Documentation: http://localhost:${API_PORT:-8080}/swagger-ui/"
echo "- Frontend: http://localhost:5173"

if [ "$NO_MONITORING" = false ]; then
    echo "- Prometheus: http://localhost:${PROMETHEUS_PORT:-9090}"
    echo "- Grafana: http://localhost:${GRAFANA_PORT:-3000} (admin/admin)"
    echo "- Jaeger UI: http://localhost:${JAEGER_UI_PORT:-16686}"
fi

echo
echo "Useful Commands:"
echo "- View logs: docker-compose logs -f [service-name]"
echo "- Stop all services: docker-compose down"
echo "- Clean restart: ./startup.sh --clean"
echo "- Production mode: ./startup.sh prod"
echo

# Keep script running if not in detached mode
if [ "$DETACHED" = false ]; then
    print_status "Services are running. Press Ctrl+C to stop..."
    
    # Wait for interrupt
    wait
fi