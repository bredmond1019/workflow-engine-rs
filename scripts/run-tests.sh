#!/bin/bash

# Test Runner Script with Automatic Environment Setup
# Usage: ./scripts/run-tests.sh [test-args...]

set -e

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ENV_FILE="$PROJECT_ROOT/.env.test"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Check if test environment is already running
if [ -z "$MCP_TEST_SERVERS_RUNNING" ] && [ ! -f "$ENV_FILE" ]; then
    print_warning "Test environment not detected. Starting test servers..."
    
    # Start test environment in background
    "$PROJECT_ROOT/scripts/setup-test-environment.sh" &
    SETUP_PID=$!
    
    # Wait for environment file to be created
    count=0
    while [ ! -f "$ENV_FILE" ] && [ $count -lt 60 ]; do
        sleep 1
        count=$((count + 1))
    done
    
    if [ ! -f "$ENV_FILE" ]; then
        echo "Failed to start test environment"
        kill $SETUP_PID 2>/dev/null || true
        exit 1
    fi
    
    print_status "Test environment started"
    
    # Cleanup function
    cleanup() {
        kill $SETUP_PID 2>/dev/null || true
        rm -f "$ENV_FILE"
    }
    trap cleanup EXIT
fi

# Source environment variables
if [ -f "$ENV_FILE" ]; then
    source "$ENV_FILE"
    print_status "Loaded test environment variables"
fi

# Change to project root
cd "$PROJECT_ROOT"

# Run tests with provided arguments
print_status "Running tests: cargo test $*"

if [ $# -eq 0 ]; then
    # Run all tests if no arguments provided
    cargo test
else
    # Run with provided arguments
    cargo test "$@"
fi