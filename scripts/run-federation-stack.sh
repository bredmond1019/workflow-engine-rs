#!/bin/bash
# Script to run the full GraphQL Federation stack for integration testing

set -e

echo "🚀 Starting GraphQL Federation Stack..."
echo "======================================"

# Function to kill processes on exit
cleanup() {
    echo ""
    echo "🛑 Shutting down services..."
    if [ ! -z "$GATEWAY_PID" ]; then
        kill $GATEWAY_PID 2>/dev/null || true
    fi
    if [ ! -z "$API_PID" ]; then
        kill $API_PID 2>/dev/null || true
    fi
    if [ ! -z "$CONTENT_PID" ]; then
        kill $CONTENT_PID 2>/dev/null || true
    fi
    if [ ! -z "$KNOWLEDGE_PID" ]; then
        kill $KNOWLEDGE_PID 2>/dev/null || true
    fi
    if [ ! -z "$REALTIME_PID" ]; then
        kill $REALTIME_PID 2>/dev/null || true
    fi
    echo "✅ All services stopped"
}

trap cleanup EXIT

# Check if services are already running
check_port() {
    local port=$1
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        echo "❌ Port $port is already in use. Please stop existing services first."
        exit 1
    fi
}

echo "Checking port availability..."
check_port 8080  # Main API
check_port 8082  # Content Processing
check_port 3002  # Knowledge Graph
check_port 8081  # Realtime Communication
check_port 4000  # GraphQL Gateway

# Start all services
echo ""
echo "Starting backend services..."
echo "--------------------------"

# 1. Start Main API (Workflow Engine)
echo "📦 Starting Workflow Engine API (port 8080)..."
cargo run --bin workflow-engine > logs/workflow-api.log 2>&1 &
API_PID=$!
echo "   PID: $API_PID"

# 2. Start Content Processing Service
echo "📄 Starting Content Processing Service (port 8082)..."
cd services/content_processing && cargo run > ../../logs/content-processing.log 2>&1 &
CONTENT_PID=$!
cd ../..
echo "   PID: $CONTENT_PID"

# 3. Start Knowledge Graph Service
echo "🧠 Starting Knowledge Graph Service (port 3002)..."
cd services/knowledge_graph && cargo run > ../../logs/knowledge-graph.log 2>&1 &
KNOWLEDGE_PID=$!
cd ../..
echo "   PID: $KNOWLEDGE_PID"

# 4. Start Realtime Communication Service
echo "💬 Starting Realtime Communication Service (port 8081)..."
cd services/realtime_communication && cargo run > ../../logs/realtime-communication.log 2>&1 &
REALTIME_PID=$!
cd ../..
echo "   PID: $REALTIME_PID"

# Wait for services to start
echo ""
echo "Waiting for services to be ready..."
sleep 5

# Function to check service health
check_service() {
    local name=$1
    local url=$2
    local max_retries=30
    local retry=0
    
    while [ $retry -lt $max_retries ]; do
        if curl -s -f "$url" > /dev/null 2>&1; then
            echo "✅ $name is ready"
            return 0
        fi
        retry=$((retry + 1))
        sleep 1
    done
    
    echo "❌ $name failed to start"
    return 1
}

# Check each service
check_service "Workflow API" "http://localhost:8080/health"
check_service "Content Processing" "http://localhost:8082/health"
check_service "Knowledge Graph" "http://localhost:3002/health"
check_service "Realtime Communication" "http://localhost:8081/health"

# 5. Start GraphQL Gateway
echo ""
echo "🌉 Starting GraphQL Gateway (port 4000)..."
cargo run --bin graphql-gateway > logs/gateway.log 2>&1 &
GATEWAY_PID=$!
echo "   PID: $GATEWAY_PID"

# Wait for gateway
sleep 3
check_service "GraphQL Gateway" "http://localhost:4000/health"

echo ""
echo "======================================"
echo "✨ Federation Stack is Ready!"
echo "======================================"
echo ""
echo "Services running at:"
echo "  - GraphQL Gateway:         http://localhost:4000/graphql"
echo "  - Workflow API:            http://localhost:8080/api/v1/graphql"
echo "  - Content Processing:      http://localhost:8082/graphql"
echo "  - Knowledge Graph:         http://localhost:3002/graphql"
echo "  - Realtime Communication:  http://localhost:8081/graphql"
echo ""
echo "Logs available in ./logs/"
echo ""
echo "Press Ctrl+C to stop all services..."
echo ""

# Keep the script running
wait