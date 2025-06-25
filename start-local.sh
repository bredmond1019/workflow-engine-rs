#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
LOG_DIR="$SCRIPT_DIR/logs"
PID_DIR="$SCRIPT_DIR/pids"

# Create necessary directories
mkdir -p "$LOG_DIR" "$PID_DIR"

# Function to print colored output
print_status() {
    echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[$(date '+%H:%M:%S')] ✓${NC} $1"
}

print_error() {
    echo -e "${RED}[$(date '+%H:%M:%S')] ✗${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[$(date '+%H:%M:%S')] ⚠${NC} $1"
}

# Function to check if a command exists
check_command() {
    if ! command -v $1 &> /dev/null; then
        print_error "$1 is not installed. Please install it first."
        return 1
    fi
    return 0
}

# Function to check if a port is in use
check_port() {
    if lsof -Pi :$1 -sTCP:LISTEN -t >/dev/null ; then
        return 0
    else
        return 1
    fi
}

# Function to wait for a service to be ready
wait_for_service() {
    local service_name=$1
    local port=$2
    local max_attempts=30
    local attempt=0
    
    print_status "Waiting for $service_name to be ready on port $port..."
    
    while [ $attempt -lt $max_attempts ]; do
        if check_port $port; then
            print_success "$service_name is ready!"
            return 0
        fi
        attempt=$((attempt + 1))
        sleep 2
    done
    
    print_error "$service_name failed to start within 60 seconds"
    return 1
}

# Function to start a service in the background
start_service() {
    local name=$1
    local command=$2
    local log_file="$LOG_DIR/$name.log"
    local pid_file="$PID_DIR/$name.pid"
    
    print_status "Starting $name..."
    
    # Start the service
    nohup bash -c "$command" > "$log_file" 2>&1 &
    local pid=$!
    echo $pid > "$pid_file"
    
    sleep 2
    
    # Check if the process is still running
    if kill -0 $pid 2>/dev/null; then
        print_success "$name started (PID: $pid)"
        return 0
    else
        print_error "$name failed to start. Check $log_file for details"
        return 1
    fi
}

# Function to stop all services
stop_all() {
    print_status "Stopping all services..."
    
    # Stop services in reverse order
    for pid_file in "$PID_DIR"/*.pid; do
        if [ -f "$pid_file" ]; then
            pid=$(cat "$pid_file")
            service_name=$(basename "$pid_file" .pid)
            
            if kill -0 $pid 2>/dev/null; then
                print_status "Stopping $service_name (PID: $pid)..."
                kill $pid
                rm "$pid_file"
            fi
        fi
    done
    
    print_success "All services stopped"
}

# Trap to clean up on exit
trap stop_all EXIT

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}      AI Workflow Engine - Local Development Startup Script     ${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# Step 1: Check prerequisites
print_status "Checking prerequisites..."

# Check required commands
REQUIRED_COMMANDS=(cargo python3 npm psql redis-cli)
for cmd in "${REQUIRED_COMMANDS[@]}"; do
    if ! check_command $cmd; then
        exit 1
    fi
done

print_success "All prerequisites met"

# Step 2: Check and start databases
print_status "Checking database services..."

# Check PostgreSQL
if ! pg_isready -q; then
    print_error "PostgreSQL is not running. Please start it with:"
    echo "  brew services start postgresql@14  # macOS"
    echo "  sudo systemctl start postgresql    # Linux"
    exit 1
fi
print_success "PostgreSQL is running"

# Check Redis
if ! redis-cli ping > /dev/null 2>&1; then
    print_warning "Redis is not running. Starting Redis..."
    start_service "redis" "redis-server"
    wait_for_service "Redis" 6379
fi
print_success "Redis is running"

# Step 3: Set up database if needed
print_status "Checking database setup..."
if ! psql -lqt | cut -d \| -f 1 | grep -qw ai_workflow_db; then
    print_status "Creating database..."
    createdb ai_workflow_db
    print_success "Database created"
fi

# Initialize schema if needed
if [ -f "$SCRIPT_DIR/scripts/init-db.sql" ]; then
    print_status "Initializing database schema..."
    psql ai_workflow_db < "$SCRIPT_DIR/scripts/init-db.sql" 2>/dev/null || true
fi

# Step 4: Start MCP servers
print_status "Starting MCP servers..."

cd "$SCRIPT_DIR/mcp-servers"

# Install Python dependencies if needed
if [ ! -d "venv" ]; then
    print_status "Creating Python virtual environment..."
    python3 -m venv venv
    source venv/bin/activate
    pip install -r requirements.txt
else
    source venv/bin/activate
fi

# Start MCP servers
start_service "mcp-helpscout" "cd '$SCRIPT_DIR/mcp-servers' && source venv/bin/activate && python -m servers.helpscout --port 8001"
start_service "mcp-notion" "cd '$SCRIPT_DIR/mcp-servers' && source venv/bin/activate && python -m servers.notion --port 8002"
start_service "mcp-slack" "cd '$SCRIPT_DIR/mcp-servers' && source venv/bin/activate && python -m servers.slack --port 8003"

# Wait for MCP servers
wait_for_service "HelpScout MCP" 8001
wait_for_service "Notion MCP" 8002
wait_for_service "Slack MCP" 8003

# Step 5: Start microservices (if enabled)
if [ "${START_MICROSERVICES:-false}" = "true" ]; then
    print_status "Starting microservices..."
    
    # Content Processing Service
    start_service "content-processing" "cd '$SCRIPT_DIR/services/content_processing' && cargo run --release"
    wait_for_service "Content Processing" 8010
    
    # Knowledge Graph Service (requires Dgraph)
    if check_port 8080; then
        start_service "knowledge-graph" "cd '$SCRIPT_DIR/services/knowledge_graph' && cargo run --release"
        wait_for_service "Knowledge Graph" 8011
    else
        print_warning "Dgraph not running, skipping Knowledge Graph service"
    fi
    
    # Realtime Communication Service
    start_service "realtime-comm" "cd '$SCRIPT_DIR/services/realtime_communication' && cargo run --release"
    wait_for_service "Realtime Communication" 8012
fi

# Step 6: Build and start the main backend
print_status "Building backend..."
cd "$SCRIPT_DIR"

# Set required environment variables
export DATABASE_URL="${DATABASE_URL:-postgresql://localhost/ai_workflow_db}"
export JWT_SECRET="${JWT_SECRET:-your-secure-jwt-secret-key-change-in-production}"
export RUST_LOG="${RUST_LOG:-info,workflow_engine=debug}"

# Build in release mode for better performance
cargo build --release

# Start the backend
start_service "workflow-engine" "cd '$SCRIPT_DIR' && cargo run --release --bin workflow-engine"
wait_for_service "Workflow Engine API" 8080

# Step 7: Start the frontend
print_status "Starting frontend..."
cd "$SCRIPT_DIR/frontend"

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    print_status "Installing frontend dependencies..."
    npm install
fi

# Start the frontend
start_service "frontend" "cd '$SCRIPT_DIR/frontend' && npm run dev"
wait_for_service "Frontend" 5173

# Step 8: Health check
print_status "Performing health checks..."

# Check API health
if curl -s http://localhost:8080/health > /dev/null; then
    print_success "API server is healthy"
else
    print_warning "API server health check failed"
fi

# Print summary
echo
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}                    All services started!                       ${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo
echo "🌐 Frontend:          http://localhost:5173"
echo "🔧 API Server:        http://localhost:8080"
echo "📚 API Documentation: http://localhost:8080/swagger-ui/"
echo "🤖 MCP Servers:"
echo "   - HelpScout:      http://localhost:8001"
echo "   - Notion:         http://localhost:8002"
echo "   - Slack:          http://localhost:8003"
echo
echo "📁 Log files:         $LOG_DIR/"
echo "🔍 Process IDs:       $PID_DIR/"
echo
echo -e "${YELLOW}Press Ctrl+C to stop all services${NC}"
echo

# Keep the script running
tail -f /dev/null