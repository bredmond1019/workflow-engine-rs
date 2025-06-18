#!/bin/bash
# Test Dgraph Setup Script
# This script sets up the Dgraph test environment for integration tests

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
TEST_COMPOSE_FILE="$PROJECT_DIR/docker-compose.test.yml"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
MAX_WAIT_TIME=120  # Maximum time to wait for services (seconds)
HEALTH_CHECK_INTERVAL=5  # Interval between health checks (seconds)

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_dependencies() {
    log_info "Checking dependencies..."
    
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed or not in PATH"
        exit 1
    fi
    
    if ! command -v docker-compose &> /dev/null; then
        log_error "Docker Compose is not installed or not in PATH"
        exit 1
    fi
    
    if ! command -v curl &> /dev/null; then
        log_error "curl is not installed or not in PATH"
        exit 1
    fi
    
    log_info "All dependencies are available"
}

cleanup_existing() {
    log_info "Cleaning up any existing test containers..."
    
    cd "$PROJECT_DIR"
    
    # Stop and remove containers, networks, and volumes
    docker-compose -f "$TEST_COMPOSE_FILE" down -v --remove-orphans 2>/dev/null || true
    
    # Remove any dangling containers
    docker ps -a --filter "name=knowledge_graph_.*_test" --format "{{.ID}}" | xargs -r docker rm -f 2>/dev/null || true
    
    # Remove test volumes
    docker volume ls --filter "name=knowledge_graph_.*_test" --format "{{.Name}}" | xargs -r docker volume rm 2>/dev/null || true
    
    log_info "Cleanup completed"
}

start_services() {
    log_info "Starting Dgraph test services..."
    
    cd "$PROJECT_DIR"
    
    # Start services in detached mode
    if ! docker-compose -f "$TEST_COMPOSE_FILE" up -d; then
        log_error "Failed to start test services"
        exit 1
    fi
    
    log_info "Services started, waiting for health checks..."
}

wait_for_service() {
    local service_name="$1"
    local health_url="$2"
    local max_attempts=$((MAX_WAIT_TIME / HEALTH_CHECK_INTERVAL))
    local attempt=0
    
    log_info "Waiting for $service_name to be healthy..."
    
    while [ $attempt -lt $max_attempts ]; do
        if curl -s -f "$health_url" > /dev/null 2>&1; then
            log_info "$service_name is healthy"
            return 0
        fi
        
        attempt=$((attempt + 1))
        if [ $attempt -lt $max_attempts ]; then
            log_info "Waiting for $service_name... (attempt $attempt/$max_attempts)"
            sleep $HEALTH_CHECK_INTERVAL
        fi
    done
    
    log_error "$service_name failed to become healthy within $MAX_WAIT_TIME seconds"
    return 1
}

verify_schema_and_data() {
    log_info "Verifying schema and test data..."
    
    # Check if schema is loaded
    local schema_response
    schema_response=$(curl -s "http://localhost:18080/admin/schema" || echo "")
    
    if echo "$schema_response" | grep -q "TestConcept"; then
        log_info "Schema loaded successfully"
    else
        log_warn "Schema may not be loaded properly"
    fi
    
    # Check if test data is available
    local data_query='{"query": "{ concepts(func: type(TestConcept), first: 1) { uid name } }"}'
    local data_response
    data_response=$(curl -s -H "Content-Type: application/json" -d "$data_query" "http://localhost:18080/query" || echo "")
    
    if echo "$data_response" | grep -q '"concepts"'; then
        log_info "Test data loaded successfully"
    else
        log_warn "Test data may not be loaded properly"
    fi
}

show_service_info() {
    log_info "Dgraph test environment is ready!"
    echo ""
    echo "Service endpoints:"
    echo "  Dgraph Alpha (GraphQL): http://localhost:18080"
    echo "  Dgraph Alpha (gRPC):    localhost:19080"
    echo "  Dgraph Zero (HTTP):     http://localhost:16080"
    echo "  Dgraph Zero (gRPC):     localhost:15080"
    echo ""
    echo "To run integration tests:"
    echo "  cd $(dirname "$PROJECT_DIR")"
    echo "  cargo test knowledge_graph_integration --ignored"
    echo "  cargo test knowledge_graph_mutation --ignored"
    echo "  cargo test knowledge_graph_transaction --ignored"
    echo ""
    echo "To view logs:"
    echo "  docker-compose -f $TEST_COMPOSE_FILE logs -f"
    echo ""
    echo "To cleanup:"
    echo "  $SCRIPT_DIR/test-dgraph-teardown.sh"
}

main() {
    log_info "Setting up Dgraph test environment..."
    
    check_dependencies
    cleanup_existing
    start_services
    
    # Wait for services to be healthy
    if ! wait_for_service "Dgraph Zero" "http://localhost:16080/health"; then
        log_error "Dgraph Zero failed to start"
        exit 1
    fi
    
    if ! wait_for_service "Dgraph Alpha" "http://localhost:18080/health"; then
        log_error "Dgraph Alpha failed to start"
        exit 1
    fi
    
    # Give a bit more time for data loader to complete
    log_info "Waiting for data loader to complete..."
    sleep 10
    
    verify_schema_and_data
    show_service_info
    
    log_info "Setup completed successfully!"
}

# Handle script termination
trap 'log_error "Setup interrupted"; cleanup_existing; exit 1' INT TERM

# Run main function
main "$@"