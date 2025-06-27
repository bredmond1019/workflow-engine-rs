#!/bin/bash

# AI Workflow System Test Suite
# This script runs comprehensive tests across the entire application stack

echo "🧪 AI Workflow System Test Suite"
echo "================================"
echo "Started at: $(date)"
echo

# Color codes for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counters
PASSED=0
FAILED=0
SKIPPED=0

# Test report file
REPORT_FILE="test-report-$(date +%Y%m%d-%H%M%S).md"

# Function to print colored output
print_status() {
    local status=$1
    local message=$2
    
    case $status in
        "PASS")
            echo -e "${GREEN}✓ PASS${NC} - $message"
            ((PASSED++))
            echo "✓ PASS - $message" >> $REPORT_FILE
            ;;
        "FAIL")
            echo -e "${RED}✗ FAIL${NC} - $message"
            ((FAILED++))
            echo "✗ FAIL - $message" >> $REPORT_FILE
            ;;
        "SKIP")
            echo -e "${YELLOW}⚠ SKIP${NC} - $message"
            ((SKIPPED++))
            echo "⚠ SKIP - $message" >> $REPORT_FILE
            ;;
        "INFO")
            echo -e "${BLUE}ℹ INFO${NC} - $message"
            echo "ℹ INFO - $message" >> $REPORT_FILE
            ;;
    esac
}

# Function to check if a service is running
check_service() {
    local name=$1
    local url=$2
    local expected_code=${3:-200}
    local timeout=${4:-5}
    
    if command -v curl &> /dev/null; then
        response=$(curl -s -o /dev/null -w "%{http_code}" --connect-timeout $timeout $url 2>/dev/null)
        
        if [ "$response" = "$expected_code" ]; then
            print_status "PASS" "$name is running at $url"
            return 0
        else
            print_status "FAIL" "$name not responding at $url (HTTP $response)"
            return 1
        fi
    else
        print_status "SKIP" "curl not installed, cannot check $name"
        return 2
    fi
}

# Function to check if a port is open
check_port() {
    local name=$1
    local host=$2
    local port=$3
    
    if command -v nc &> /dev/null; then
        if nc -z -w 2 $host $port 2>/dev/null; then
            print_status "PASS" "$name is listening on $host:$port"
            return 0
        else
            print_status "FAIL" "$name not listening on $host:$port"
            return 1
        fi
    else
        print_status "SKIP" "netcat not installed, cannot check $name port"
        return 2
    fi
}

# Initialize test report
echo "# AI Workflow System Test Report" > $REPORT_FILE
echo "Date: $(date)" >> $REPORT_FILE
echo "Tester: $(whoami)" >> $REPORT_FILE
echo "" >> $REPORT_FILE

# Phase 1: Environment Check
echo -e "\n${YELLOW}=== Phase 1: Environment Check ===${NC}\n"
echo "## Phase 1: Environment Check" >> $REPORT_FILE
echo "" >> $REPORT_FILE

print_status "INFO" "Checking required tools..."

# Check Rust
if command -v rustc &> /dev/null; then
    rust_version=$(rustc --version | cut -d' ' -f2)
    print_status "PASS" "Rust installed (version $rust_version)"
else
    print_status "FAIL" "Rust not installed"
fi

# Check Node.js
if command -v node &> /dev/null; then
    node_version=$(node --version)
    print_status "PASS" "Node.js installed (version $node_version)"
else
    print_status "FAIL" "Node.js not installed"
fi

# Check Docker
if command -v docker &> /dev/null; then
    if docker ps &> /dev/null; then
        print_status "PASS" "Docker installed and running"
    else
        print_status "FAIL" "Docker installed but not running or no permissions"
    fi
else
    print_status "FAIL" "Docker not installed"
fi

# Check PostgreSQL client
if command -v psql &> /dev/null; then
    print_status "PASS" "PostgreSQL client installed"
else
    print_status "FAIL" "PostgreSQL client not installed"
fi

# Phase 2: Infrastructure Services
echo -e "\n${YELLOW}=== Phase 2: Infrastructure Services ===${NC}\n"
echo "" >> $REPORT_FILE
echo "## Phase 2: Infrastructure Services" >> $REPORT_FILE
echo "" >> $REPORT_FILE

print_status "INFO" "Checking infrastructure services..."

# Check PostgreSQL
check_port "PostgreSQL" "localhost" "5432"

# Check if we can connect to the database
if command -v psql &> /dev/null && nc -z -w 2 localhost 5432 2>/dev/null; then
    if PGPASSWORD=aiworkflow123 psql -h localhost -U aiworkflow -d ai_workflow_db -c "SELECT 1" &> /dev/null; then
        print_status "PASS" "PostgreSQL database connection successful"
    else
        print_status "FAIL" "PostgreSQL database connection failed"
    fi
fi

# Phase 3: Backend Services
echo -e "\n${YELLOW}=== Phase 3: Backend Services ===${NC}\n"
echo "" >> $REPORT_FILE
echo "## Phase 3: Backend Services" >> $REPORT_FILE
echo "" >> $REPORT_FILE

print_status "INFO" "Checking backend services..."

# Main API
check_service "Main API Health" "http://localhost:8080/health"
check_service "Swagger UI" "http://localhost:8080/swagger-ui/"

# GraphQL Gateway
check_service "GraphQL Gateway Health" "http://localhost:4000/.well-known/apollo/server-health"
check_service "GraphQL Playground" "http://localhost:4000/graphql"

# MCP Servers
check_service "HelpScout MCP Server" "http://localhost:8001/health"
check_service "Notion MCP Server" "http://localhost:8002/health"
check_service "Slack MCP Server" "http://localhost:8003/health"

# Microservices (optional)
check_service "Content Processing Service" "http://localhost:8082/health" 200 2
check_service "Knowledge Graph Service" "http://localhost:8083/health" 200 2
check_service "Realtime Communication Service" "http://localhost:8084/health" 200 2

# Phase 4: Backend Tests
echo -e "\n${YELLOW}=== Phase 4: Backend Tests ===${NC}\n"
echo "" >> $REPORT_FILE
echo "## Phase 4: Backend Tests" >> $REPORT_FILE
echo "" >> $REPORT_FILE

print_status "INFO" "Running backend unit tests..."

# Check if we're in the right directory
if [ -f "Cargo.toml" ]; then
    # Run Rust tests
    if cargo test --workspace --quiet 2>&1 | grep -q "test result: ok"; then
        print_status "PASS" "Backend unit tests passed"
    else
        print_status "FAIL" "Backend unit tests failed"
        print_status "INFO" "Run 'cargo test' for detailed output"
    fi
else
    print_status "SKIP" "Not in project root directory, skipping backend tests"
fi

# Phase 5: Frontend Tests
echo -e "\n${YELLOW}=== Phase 5: Frontend Tests ===${NC}\n"
echo "" >> $REPORT_FILE
echo "## Phase 5: Frontend Tests" >> $REPORT_FILE
echo "" >> $REPORT_FILE

print_status "INFO" "Checking frontend..."

# Check if frontend directory exists
if [ -d "frontend" ]; then
    cd frontend
    
    # Check if node_modules exists
    if [ -d "node_modules" ]; then
        # Check if frontend is running
        check_service "Frontend Dev Server" "http://localhost:5173"
        
        # Run frontend tests
        print_status "INFO" "Running frontend tests..."
        if npm test -- --watchAll=false --passWithNoTests 2>&1 | grep -q "Test Suites:.*passed"; then
            print_status "PASS" "Frontend tests passed"
        else
            print_status "FAIL" "Frontend tests failed"
            print_status "INFO" "Run 'npm test' in frontend directory for details"
        fi
    else
        print_status "SKIP" "Frontend dependencies not installed (run 'npm install' in frontend/)"
    fi
    
    cd ..
else
    print_status "SKIP" "Frontend directory not found"
fi

# Phase 6: Integration Tests
echo -e "\n${YELLOW}=== Phase 6: Integration Tests ===${NC}\n"
echo "" >> $REPORT_FILE
echo "## Phase 6: Integration Tests" >> $REPORT_FILE
echo "" >> $REPORT_FILE

print_status "INFO" "Testing API endpoints..."

# Test authentication
if curl -s http://localhost:8080/health &> /dev/null; then
    # Try to get a token
    token_response=$(curl -s -X POST http://localhost:8080/auth/token \
        -H "Content-Type: application/json" \
        -d '{"username":"admin","password":"admin123"}' 2>/dev/null)
    
    if echo $token_response | grep -q "token"; then
        print_status "PASS" "Authentication endpoint working"
        
        # Extract token for further tests
        token=$(echo $token_response | grep -o '"token":"[^"]*' | cut -d'"' -f4)
        
        # Test authenticated endpoint
        workflow_response=$(curl -s -X GET http://localhost:8080/api/v1/workflows \
            -H "Authorization: Bearer $token" \
            -w "\n%{http_code}" 2>/dev/null)
        
        if echo $workflow_response | tail -1 | grep -q "200"; then
            print_status "PASS" "Authenticated API requests working"
        else
            print_status "FAIL" "Authenticated API requests failing"
        fi
    else
        print_status "FAIL" "Authentication endpoint not working"
    fi
fi

# Test GraphQL
if curl -s http://localhost:4000/graphql &> /dev/null; then
    graphql_response=$(curl -s -X POST http://localhost:4000/graphql \
        -H "Content-Type: application/json" \
        -d '{"query":"{ __schema { queryType { name } } }"}' 2>/dev/null)
    
    if echo $graphql_response | grep -q "Query"; then
        print_status "PASS" "GraphQL endpoint working"
    else
        print_status "FAIL" "GraphQL endpoint not working properly"
    fi
fi

# Phase 7: Summary
echo -e "\n${YELLOW}=== Test Summary ===${NC}\n"
echo "" >> $REPORT_FILE
echo "## Summary" >> $REPORT_FILE
echo "" >> $REPORT_FILE

total=$((PASSED + FAILED + SKIPPED))
echo -e "Total Tests: $total"
echo -e "${GREEN}Passed: $PASSED${NC}"
echo -e "${RED}Failed: $FAILED${NC}"
echo -e "${YELLOW}Skipped: $SKIPPED${NC}"

echo "Total Tests: $total" >> $REPORT_FILE
echo "Passed: $PASSED" >> $REPORT_FILE
echo "Failed: $FAILED" >> $REPORT_FILE
echo "Skipped: $SKIPPED" >> $REPORT_FILE

# Calculate pass rate
if [ $total -gt 0 ]; then
    pass_rate=$((PASSED * 100 / total))
    echo -e "\nPass Rate: ${pass_rate}%"
    echo "" >> $REPORT_FILE
    echo "Pass Rate: ${pass_rate}%" >> $REPORT_FILE
fi

# Final status
echo "" >> $REPORT_FILE
if [ $FAILED -eq 0 ]; then
    echo -e "\n${GREEN}✨ All required tests passed! ✨${NC}"
    echo "Status: ✅ All required tests passed!" >> $REPORT_FILE
    exit_code=0
else
    echo -e "\n${RED}❌ Some tests failed. Please check the logs above.${NC}"
    echo "Status: ❌ Some tests failed." >> $REPORT_FILE
    exit_code=1
fi

echo -e "\n📄 Test report saved to: ${BLUE}$REPORT_FILE${NC}"
echo "" >> $REPORT_FILE
echo "Report generated at: $(date)" >> $REPORT_FILE

# Recommendations section
if [ $FAILED -gt 0 ] || [ $SKIPPED -gt 0 ]; then
    echo "" >> $REPORT_FILE
    echo "## Recommendations" >> $REPORT_FILE
    echo "" >> $REPORT_FILE
    
    if ! command -v docker &> /dev/null || ! docker ps &> /dev/null; then
        echo "- Install/start Docker to use docker-compose for easy service management" >> $REPORT_FILE
    fi
    
    if ! nc -z -w 2 localhost 5432 2>/dev/null; then
        echo "- Start PostgreSQL database (port 5432)" >> $REPORT_FILE
    fi
    
    if ! curl -s http://localhost:8080/health &> /dev/null; then
        echo "- Start the main API server: cargo run --bin workflow-engine" >> $REPORT_FILE
    fi
    
    if ! curl -s http://localhost:4000/graphql &> /dev/null; then
        echo "- Start the GraphQL gateway: cargo run --bin graphql-gateway" >> $REPORT_FILE
    fi
    
    if ! curl -s http://localhost:8001/health &> /dev/null; then
        echo "- Start MCP test servers: ./scripts/start_test_servers.sh" >> $REPORT_FILE
    fi
fi

exit $exit_code