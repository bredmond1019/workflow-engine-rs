#!/bin/bash

# AI Workflow Engine Setup Verification Script
# This script checks that all services are running correctly

set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🔍 Verifying AI Workflow Engine Setup..."
echo "========================================="

# Track overall status
OVERALL_STATUS=0

# Function to check service
check_service() {
    local name=$1
    local url=$2
    local expected=$3
    
    printf "Checking %-25s" "$name..."
    
    if response=$(curl -sf "$url" 2>/dev/null); then
        if [[ -z "$expected" ]] || [[ "$response" == *"$expected"* ]]; then
            echo -e "${GREEN}✓ OK${NC}"
        else
            echo -e "${RED}✗ FAIL${NC} (unexpected response)"
            OVERALL_STATUS=1
        fi
    else
        echo -e "${RED}✗ FAIL${NC} (not responding)"
        OVERALL_STATUS=1
    fi
}

# Function to check command availability
check_command() {
    local cmd=$1
    local min_version=$2
    
    printf "Checking %-25s" "$cmd..."
    
    if command -v "$cmd" &> /dev/null; then
        if [[ -n "$min_version" ]]; then
            version=$($cmd --version 2>&1 | head -n1)
            echo -e "${GREEN}✓ OK${NC} ($version)"
        else
            echo -e "${GREEN}✓ OK${NC}"
        fi
    else
        echo -e "${RED}✗ NOT FOUND${NC}"
        OVERALL_STATUS=1
    fi
}

# Function to check port
check_port() {
    local name=$1
    local port=$2
    
    printf "Checking %-25s" "$name (port $port)..."
    
    if nc -z localhost "$port" 2>/dev/null; then
        echo -e "${GREEN}✓ OPEN${NC}"
    else
        echo -e "${YELLOW}⚠ CLOSED${NC}"
    fi
}

echo
echo "📋 Prerequisites Check"
echo "---------------------"
check_command "rustc" "1.75"
check_command "node" "18"
check_command "npm" ""
check_command "psql" "15"
check_command "docker" "20"
check_command "python3" "3.11"
check_command "uv" ""

echo
echo "🌐 Core Services"
echo "----------------"
check_service "Main API" "http://localhost:8080/health" "healthy"
check_service "GraphQL Gateway" "http://localhost:4000/health" ""
check_service "Frontend Dev Server" "http://localhost:5173" ""
check_service "Swagger UI" "http://localhost:8080/swagger-ui/" ""

echo
echo "🔧 Microservices"
echo "----------------"
check_service "Content Processing" "http://localhost:8082/health" ""
check_service "Knowledge Graph" "http://localhost:3002/health" ""
check_service "Realtime Communication" "http://localhost:8081/health" ""

echo
echo "💾 Database"
echo "-----------"
printf "Checking %-25s" "PostgreSQL..."
if psql -h localhost -U aiworkflow -d ai_workflow_db -c "SELECT 1" &>/dev/null; then
    echo -e "${GREEN}✓ CONNECTED${NC}"
else
    echo -e "${RED}✗ FAIL${NC} (connection failed)"
    OVERALL_STATUS=1
fi

echo
echo "📊 Monitoring Stack"
echo "------------------"
check_service "Grafana" "http://localhost:3000" ""
check_service "Prometheus" "http://localhost:9090" ""
check_service "Jaeger UI" "http://localhost:16686" ""
check_service "Redis" "http://localhost:6379" ""

echo
echo "🔌 Port Status"
echo "--------------"
check_port "Main API" 8080
check_port "GraphQL Gateway" 4000
check_port "Frontend" 5173
check_port "PostgreSQL" 5432
check_port "Redis" 6379
check_port "Content Processing" 8082
check_port "Knowledge Graph" 3002
check_port "Realtime Comm" 8081

echo
echo "🧪 Quick Tests"
echo "--------------"

# Test GraphQL endpoint
printf "Testing GraphQL query...    "
if curl -sf -X POST http://localhost:4000/graphql \
    -H "Content-Type: application/json" \
    -d '{"query":"{ __schema { types { name } } }"}' &>/dev/null; then
    echo -e "${GREEN}✓ OK${NC}"
else
    echo -e "${RED}✗ FAIL${NC}"
    OVERALL_STATUS=1
fi

# Test API endpoint
printf "Testing REST API...         "
if curl -sf http://localhost:8080/api/v1/health &>/dev/null; then
    echo -e "${GREEN}✓ OK${NC}"
else
    echo -e "${RED}✗ FAIL${NC}"
    OVERALL_STATUS=1
fi

echo
echo "========================================="
if [ $OVERALL_STATUS -eq 0 ]; then
    echo -e "${GREEN}✅ All checks passed!${NC}"
    echo
    echo "🎉 Your AI Workflow Engine is ready to use!"
    echo
    echo "📚 Quick Links:"
    echo "  • Frontend:          http://localhost:5173"
    echo "  • GraphQL Playground: http://localhost:4000/graphql"
    echo "  • Swagger Docs:      http://localhost:8080/swagger-ui/"
    echo "  • Grafana:           http://localhost:3000 (admin/admin)"
else
    echo -e "${RED}❌ Some checks failed!${NC}"
    echo
    echo "🔧 Troubleshooting:"
    echo "  1. Check if Docker is running: docker info"
    echo "  2. Start services: docker-compose up -d"
    echo "  3. Check logs: docker-compose logs -f"
    echo "  4. For detailed help, see QUICK_START.md"
    exit 1
fi