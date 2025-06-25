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
COMPOSE_FILE="${COMPOSE_FILE:-docker-compose.minimal.yml}"

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

# Function to wait for a service to be healthy
wait_for_healthy() {
    local service_name=$1
    local max_attempts=30
    local attempt=0
    
    print_status "Waiting for $service_name to be healthy..."
    
    while [ $attempt -lt $max_attempts ]; do
        if docker-compose -f $COMPOSE_FILE ps | grep $service_name | grep -q "healthy\|Up"; then
            print_success "$service_name is ready!"
            return 0
        fi
        attempt=$((attempt + 1))
        sleep 2
    done
    
    print_error "$service_name failed to become healthy within 60 seconds"
    return 1
}

# Function to check Docker service status
check_docker_service() {
    local service=$1
    if docker-compose -f $COMPOSE_FILE ps | grep -q "$service.*Up"; then
        return 0
    else
        return 1
    fi
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --full)
            COMPOSE_FILE="docker-compose.simple.yml"
            shift
            ;;
        --minimal)
            COMPOSE_FILE="docker-compose.minimal.yml"
            shift
            ;;
        --help)
            echo "Usage: $0 [options]"
            echo "Options:"
            echo "  --minimal  Run minimal setup (default: PostgreSQL, Redis, API, Frontend)"
            echo "  --full     Run full setup with MCP servers and monitoring"
            echo "  --help     Show this help message"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}        AI Workflow Engine - Docker Startup Script              ${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

print_status "Using compose file: $COMPOSE_FILE"

# Step 1: Check prerequisites
print_status "Checking prerequisites..."

# Check Docker and Docker Compose
if ! check_command docker; then
    exit 1
fi

if ! check_command docker-compose; then
    # Try docker compose (newer syntax)
    if docker compose version &> /dev/null; then
        print_status "Using 'docker compose' (v2) syntax"
        alias docker-compose='docker compose'
    else
        print_error "Neither 'docker-compose' nor 'docker compose' found"
        exit 1
    fi
fi

# Check if Docker daemon is running
if ! docker info > /dev/null 2>&1; then
    print_error "Docker daemon is not running. Please start Docker Desktop."
    exit 1
fi

print_success "All prerequisites met"

# Step 2: Create .env file if it doesn't exist
if [ ! -f "$SCRIPT_DIR/.env" ]; then
    print_status "Creating .env file..."
    cat > "$SCRIPT_DIR/.env" << EOF
# Database
POSTGRES_USER=postgres
POSTGRES_PASSWORD=postgres
POSTGRES_DB=ai_workflow_db

# JWT Secret
JWT_SECRET=your-secure-jwt-secret-key-change-in-production

# Logging
RUST_LOG=info,workflow_engine=debug

# API Keys (add your own)
# OPENAI_API_KEY=sk-...
# ANTHROPIC_API_KEY=sk-ant-...
# AWS_ACCESS_KEY_ID=...
# AWS_SECRET_ACCESS_KEY=...

# Service Ports
API_PORT=8080
FRONTEND_PORT=5173
DB_PORT=5432
REDIS_PORT=6379
EOF
    print_success ".env file created (please add your API keys)"
fi

# Step 3: Stop any existing containers
print_status "Stopping any existing containers..."
docker-compose -f $COMPOSE_FILE down --remove-orphans 2>/dev/null || true

# Step 4: Build images
print_status "Building Docker images..."
docker-compose -f $COMPOSE_FILE build --no-cache --parallel

# Step 5: Start infrastructure services
print_status "Starting infrastructure services..."
docker-compose -f $COMPOSE_FILE up -d postgres redis

# Wait for PostgreSQL to be ready
print_status "Waiting for PostgreSQL to be ready..."
until docker-compose -f $COMPOSE_FILE exec -T postgres pg_isready -U postgres > /dev/null 2>&1; do
    sleep 2
done
print_success "PostgreSQL is ready"

# Step 6: Start the main application
print_status "Starting main application..."
docker-compose -f $COMPOSE_FILE up -d ai-workflow-system

# Wait for main app to be healthy
wait_for_healthy "ai-workflow-system"

# Step 7: Start the frontend
print_status "Starting frontend..."
docker-compose -f $COMPOSE_FILE up -d frontend

# Step 8: Health checks
print_status "Performing health checks..."

# Check all services based on compose file
if [ "$COMPOSE_FILE" = "docker-compose.minimal.yml" ]; then
    services=(postgres redis ai-workflow-system frontend)
else
    services=(postgres redis ai-workflow-system frontend)
fi

all_healthy=true

for service in "${services[@]}"; do
    if check_docker_service $service; then
        print_success "$service is running"
    else
        print_error "$service is not running"
        all_healthy=false
    fi
done

# Print logs command for debugging
if [ "$all_healthy" = false ]; then
    echo
    print_warning "Some services failed to start. Check logs with:"
    echo "  docker-compose -f $COMPOSE_FILE logs -f [service-name]"
fi

# Print summary
echo
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}               All services started in Docker!                  ${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo
echo "🌐 Frontend:          http://localhost:5173"
echo "🔧 API Server:        http://localhost:8080"
echo "📚 API Documentation: http://localhost:8080/swagger-ui/"
echo
if [ "$COMPOSE_FILE" != "docker-compose.minimal.yml" ]; then
    echo "🤖 MCP Servers:"
    echo "   Note: MCP servers need to be run locally with:"
    echo "   cd scripts && ./start_test_servers.sh"
    echo
fi
echo "📋 Useful Docker commands:"
echo "   View logs:         docker-compose -f $COMPOSE_FILE logs -f [service]"
echo "   List services:     docker-compose -f $COMPOSE_FILE ps"
echo "   Stop all:          docker-compose -f $COMPOSE_FILE down"
echo "   Stop & remove:     docker-compose -f $COMPOSE_FILE down -v"
echo
echo -e "${YELLOW}To stop all services: ./stop-docker.sh${NC}"
echo
if [ "$COMPOSE_FILE" = "docker-compose.minimal.yml" ]; then
    echo -e "${YELLOW}Note: Running minimal setup. For full setup with monitoring, use: ./start-docker.sh --full${NC}"
fi