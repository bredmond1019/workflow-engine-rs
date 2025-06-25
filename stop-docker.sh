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
COMPOSE_FILE="${COMPOSE_FILE:-docker-compose.simple.yml}"

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

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}         Stopping AI Workflow Engine Docker Services            ${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

cd "$SCRIPT_DIR"

# Check if docker-compose is available
if ! command -v docker-compose &> /dev/null; then
    # Try docker compose (newer syntax)
    if docker compose version &> /dev/null; then
        alias docker-compose='docker compose'
    else
        print_error "Neither 'docker-compose' nor 'docker compose' found"
        exit 1
    fi
fi

# Check which compose file to use
if [ ! -f "$SCRIPT_DIR/docker-compose.yml" ] || [ "$1" == "--simple" ]; then
    COMPOSE_FILE="docker-compose.simple.yml"
    print_status "Using simplified docker-compose configuration"
fi

# Get list of running services
print_status "Checking running services..."
running_services=$(docker-compose -f $COMPOSE_FILE ps --services 2>/dev/null || echo "")

if [ -z "$running_services" ]; then
    print_status "No services are currently running"
    exit 0
fi

# Show what will be stopped
echo
print_status "The following services will be stopped:"
echo "$running_services" | while read service; do
    echo "  - $service"
done
echo

# Ask for confirmation if not forced
if [ "$1" != "--force" ] && [ "$1" != "-f" ]; then
    read -p "Do you want to stop all services? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_warning "Operation cancelled"
        exit 0
    fi
fi

# Stop all services
print_status "Stopping all services..."
if docker-compose -f $COMPOSE_FILE down; then
    print_success "All services stopped successfully"
else
    print_error "Failed to stop services"
    exit 1
fi

# Ask about volumes
if [ "$1" != "--force" ] && [ "$1" != "-f" ]; then
    echo
    read -p "Do you want to remove volumes (data will be lost)? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        print_status "Removing volumes..."
        if docker-compose -f $COMPOSE_FILE down -v; then
            print_success "Volumes removed"
        else
            print_error "Failed to remove volumes"
        fi
    fi
fi

# Clean up dangling images
print_status "Cleaning up dangling images..."
docker image prune -f > /dev/null 2>&1 || true

echo
print_success "Docker cleanup complete!"
echo
echo "To restart services, run: ./start-docker.sh"