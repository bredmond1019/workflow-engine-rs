#!/bin/bash
# Test setup script for AI Workflow Engine
# This script prepares the test environment without requiring external services

set -e

echo "🧪 Setting up test environment..."

# Set test environment variables
export TEST_USE_IN_MEMORY_DB=true
export TEST_USE_MOCK_MCP=true
export TEST_DISABLE_EXTERNAL_SERVICES=true
export RUST_LOG=debug
export RUST_BACKTRACE=1

# Create test directories
mkdir -p tests/fixtures
mkdir -p /tmp/workflow-engine-tests

# Check if we should run with real infrastructure
if [ "$1" == "--with-infrastructure" ]; then
    echo "📦 Running with real infrastructure..."
    export TEST_USE_IN_MEMORY_DB=false
    export TEST_USE_MOCK_MCP=false
    export TEST_DISABLE_EXTERNAL_SERVICES=false
    
    # Start test MCP servers if needed
    if [ -f "./scripts/start_test_servers.sh" ]; then
        echo "🚀 Starting MCP test servers..."
        ./scripts/start_test_servers.sh
    fi
    
    # Create test database if needed
    if command -v psql &> /dev/null; then
        echo "🗄️ Setting up test database..."
        createdb workflow_test_db 2>/dev/null || true
        psql workflow_test_db < scripts/init-db.sql 2>/dev/null || true
    fi
fi

# Run specific test category if provided
if [ -n "$2" ]; then
    case "$2" in
        "unit")
            echo "🧩 Running unit tests..."
            cargo test --lib
            ;;
        "integration")
            echo "🔗 Running integration tests..."
            cargo test --test '*' -- --ignored
            ;;
        "api")
            echo "🌐 Running API tests..."
            cargo test -p workflow-engine-api
            ;;
        "mcp")
            echo "🔌 Running MCP tests..."
            cargo test mcp_ -- --ignored
            ;;
        "workflow")
            echo "⚙️ Running workflow tests..."
            cargo test -p workflow-engine-core workflow_
            ;;
        *)
            echo "❓ Unknown test category: $2"
            echo "Available categories: unit, integration, api, mcp, workflow"
            exit 1
            ;;
    esac
else
    # Run all tests
    echo "🚀 Running all tests..."
    cargo test
fi

echo "✅ Test run complete!"