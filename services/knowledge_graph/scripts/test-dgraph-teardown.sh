#!/bin/bash
# Test Dgraph Teardown Script
# This script cleans up the Dgraph test environment

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
TEST_COMPOSE_FILE="$PROJECT_DIR/docker-compose.test.yml"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

cleanup_containers() {
    log_info "Stopping and removing test containers..."
    
    cd "$PROJECT_DIR"
    
    # Stop and remove containers defined in docker-compose.test.yml
    if docker-compose -f "$TEST_COMPOSE_FILE" ps -q | grep -q .; then
        docker-compose -f "$TEST_COMPOSE_FILE" down --remove-orphans
    else
        log_info "No containers from test compose to stop"
    fi
    
    # Force remove any remaining test containers
    local test_containers
    test_containers=$(docker ps -a --filter "name=knowledge_graph_.*_test" --format "{{.ID}}" 2>/dev/null || echo "")
    
    if [ -n "$test_containers" ]; then
        log_info "Removing remaining test containers..."
        echo "$test_containers" | xargs docker rm -f
    fi
    
    log_info "Containers cleaned up"
}

cleanup_volumes() {
    log_info "Removing test volumes..."
    
    cd "$PROJECT_DIR"
    
    # Remove volumes defined in docker-compose.test.yml
    docker-compose -f "$TEST_COMPOSE_FILE" down -v 2>/dev/null || true
    
    # Force remove any remaining test volumes
    local test_volumes
    test_volumes=$(docker volume ls --filter "name=knowledge_graph_.*_test" --format "{{.Name}}" 2>/dev/null || echo "")
    
    if [ -n "$test_volumes" ]; then
        log_info "Removing remaining test volumes..."
        echo "$test_volumes" | xargs docker volume rm 2>/dev/null || true
    fi
    
    log_info "Volumes cleaned up"
}

cleanup_networks() {
    log_info "Removing test networks..."
    
    # Remove test network if it exists
    if docker network ls --filter "name=knowledge_graph_test_network" --format "{{.Name}}" | grep -q "knowledge_graph_test_network"; then
        docker network rm knowledge_graph_test_network 2>/dev/null || true
    fi
    
    log_info "Networks cleaned up"
}

cleanup_images() {
    local remove_images="${1:-false}"
    
    if [ "$remove_images" = "true" ] || [ "$remove_images" = "--images" ]; then
        log_info "Removing unused Docker images..."
        
        # Remove dangling images
        local dangling_images
        dangling_images=$(docker images -f "dangling=true" -q 2>/dev/null || echo "")
        
        if [ -n "$dangling_images" ]; then
            echo "$dangling_images" | xargs docker rmi 2>/dev/null || true
        fi
        
        log_info "Unused images cleaned up"
    fi
}

verify_cleanup() {
    log_info "Verifying cleanup..."
    
    # Check for remaining test containers
    local remaining_containers
    remaining_containers=$(docker ps -a --filter "name=knowledge_graph_.*_test" --format "{{.Names}}" 2>/dev/null || echo "")
    
    if [ -n "$remaining_containers" ]; then
        log_warn "Some test containers may still exist:"
        echo "$remaining_containers"
    fi
    
    # Check for remaining test volumes
    local remaining_volumes
    remaining_volumes=$(docker volume ls --filter "name=knowledge_graph_.*_test" --format "{{.Name}}" 2>/dev/null || echo "")
    
    if [ -n "$remaining_volumes" ]; then
        log_warn "Some test volumes may still exist:"
        echo "$remaining_volumes"
    fi
    
    # Check if test ports are still in use
    if netstat -ln 2>/dev/null | grep -q ":18080\|:19080\|:15080\|:16080"; then
        log_warn "Some test ports may still be in use"
    fi
    
    log_info "Cleanup verification completed"
}

show_cleanup_info() {
    log_info "Dgraph test environment cleaned up!"
    echo ""
    echo "If you need to restart the test environment:"
    echo "  $SCRIPT_DIR/test-dgraph-setup.sh"
    echo ""
    echo "To run a complete Docker cleanup (removes all unused containers, networks, images):"
    echo "  docker system prune -f"
    echo ""
}

usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --images    Also remove unused Docker images"
    echo "  --help      Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                 # Basic cleanup"
    echo "  $0 --images        # Cleanup including unused images"
}

main() {
    local remove_images="false"
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --images)
                remove_images="true"
                shift
                ;;
            --help)
                usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done
    
    log_info "Cleaning up Dgraph test environment..."
    
    cleanup_containers
    cleanup_volumes
    cleanup_networks
    cleanup_images "$remove_images"
    
    verify_cleanup
    show_cleanup_info
    
    log_info "Teardown completed successfully!"
}

# Handle script termination
trap 'log_error "Teardown interrupted"' INT TERM

# Run main function
main "$@"