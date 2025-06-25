#!/bin/bash
#
# AI Workflow Engine - Shutdown Script
# 
# This script handles the graceful shutdown of the AI Workflow Engine
# and all its associated services.
#
# Usage:
#   ./shutdown.sh [options]
#
# Options:
#   --keep-data       Keep database volumes (don't remove data)
#   --force           Force stop without graceful shutdown
#   --clean           Remove all containers, volumes, and networks
#

set -e

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

# Process command line arguments
KEEP_DATA=false
FORCE_STOP=false
CLEAN_ALL=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --keep-data)
            KEEP_DATA=true
            ;;
        --force)
            FORCE_STOP=true
            ;;
        --clean)
            CLEAN_ALL=true
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

# Print banner
echo "=============================================="
echo "   AI Workflow Engine - Shutdown Script"
echo "=============================================="
echo

# Step 1: Stop frontend if running
print_status "Checking for running frontend process..."
FRONTEND_PIDS=$(lsof -ti:5173 2>/dev/null || true)
if [ -n "$FRONTEND_PIDS" ]; then
    print_status "Stopping frontend server..."
    for pid in $FRONTEND_PIDS; do
        kill $pid 2>/dev/null || true
    done
    print_success "Frontend server stopped"
else
    print_status "Frontend server not running"
fi

# Step 2: Stop MCP servers
print_status "Stopping MCP servers..."
# Find Python processes running MCP servers
MCP_PIDS=$(ps aux | grep -E "(customer_support_server|multi_service_mcp_server)" | grep -v grep | awk '{print $2}' || true)
if [ -n "$MCP_PIDS" ]; then
    for pid in $MCP_PIDS; do
        kill $pid 2>/dev/null || true
    done
    print_success "MCP servers stopped"
else
    print_status "No MCP servers running"
fi

# Step 3: Stop Docker services
print_status "Stopping Docker services..."

if [ "$FORCE_STOP" = true ]; then
    docker-compose kill
    print_warning "Services forcefully stopped"
else
    docker-compose stop
    print_success "Services gracefully stopped"
fi

# Step 4: Remove containers
if [ "$CLEAN_ALL" = true ]; then
    print_status "Removing containers..."
    docker-compose down
    print_success "Containers removed"
    
    # Stop Dgraph if it exists
    if [ -d "$PROJECT_ROOT/services/knowledge_graph/dgraph" ]; then
        print_status "Stopping Dgraph..."
        cd "$PROJECT_ROOT/services/knowledge_graph/dgraph"
        docker-compose down
        cd "$PROJECT_ROOT"
        print_success "Dgraph stopped"
    fi
fi

# Step 5: Remove volumes if requested
if [ "$CLEAN_ALL" = true ] && [ "$KEEP_DATA" = false ]; then
    print_warning "Removing all data volumes..."
    docker-compose down -v
    
    # Remove Dgraph volumes
    if [ -d "$PROJECT_ROOT/services/knowledge_graph/dgraph" ]; then
        cd "$PROJECT_ROOT/services/knowledge_graph/dgraph"
        docker-compose down -v
        cd "$PROJECT_ROOT"
    fi
    
    print_success "All volumes removed"
elif [ "$KEEP_DATA" = true ]; then
    print_status "Keeping data volumes as requested"
fi

# Step 6: Clean up any orphaned resources
if [ "$CLEAN_ALL" = true ]; then
    print_status "Cleaning up orphaned Docker resources..."
    
    # Remove orphaned containers
    docker container prune -f
    
    # Remove unused networks
    docker network prune -f
    
    # Remove unused volumes (only if not keeping data)
    if [ "$KEEP_DATA" = false ]; then
        docker volume prune -f
    fi
    
    print_success "Cleanup completed"
fi

# Step 7: Final status check
print_status "Checking final status..."

# Check for any remaining containers
RUNNING_CONTAINERS=$(docker-compose ps -q 2>/dev/null || true)
if [ -z "$RUNNING_CONTAINERS" ]; then
    print_success "All services stopped successfully"
else
    print_warning "Some containers may still be running:"
    docker-compose ps
fi

echo
echo "=============================================="
print_success "Shutdown complete!"

if [ "$CLEAN_ALL" = true ] && [ "$KEEP_DATA" = false ]; then
    print_warning "All data has been removed. The system is ready for a fresh start."
elif [ "$KEEP_DATA" = true ]; then
    print_status "Data volumes have been preserved for next startup."
fi

echo
echo "To restart the system, run: ./scripts/startup.sh"
echo "=============================================="