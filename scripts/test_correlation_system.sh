#!/bin/bash

# Test script for end-to-end correlation ID tracking across the AI Workflow System
# This script validates that correlation IDs are properly propagated across all services

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BASE_URL_WORKFLOW="http://localhost:8080"
BASE_URL_TUTOR="http://localhost:3001"
LOKI_URL="http://localhost:3100"
PROMETHEUS_URL="http://localhost:9090"
CORRELATION_ID="test-correlation-$(date +%s)-$(openssl rand -hex 4)"

echo -e "${BLUE}🔍 Starting AI Workflow System Correlation Test${NC}"
echo -e "${BLUE}Correlation ID: ${CORRELATION_ID}${NC}"
echo "=================================================="

# Function to check if service is running
check_service() {
    local url=$1
    local name=$2
    
    echo -n "Checking $name... "
    if curl -s --max-time 5 "$url" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Running${NC}"
        return 0
    else
        echo -e "${RED}❌ Not running${NC}"
        return 1
    fi
}

# Function to test correlation ID propagation
test_correlation_propagation() {
    local url=$1
    local service_name=$2
    local method=${3:-GET}
    local payload=${4:-"{}"}
    
    echo -n "Testing $service_name correlation... "
    
    if [ "$method" = "POST" ]; then
        response=$(curl -s -w "\n%{http_code}\n%{header_json}" \
            -X POST \
            -H "X-Correlation-ID: $CORRELATION_ID" \
            -H "Content-Type: application/json" \
            -d "$payload" \
            "$url" 2>/dev/null)
    else
        response=$(curl -s -w "\n%{http_code}\n%{header_json}" \
            -H "X-Correlation-ID: $CORRELATION_ID" \
            "$url" 2>/dev/null)
    fi
    
    # Parse response
    body=$(echo "$response" | head -n -2)
    http_code=$(echo "$response" | tail -n 2 | head -n 1)
    headers=$(echo "$response" | tail -n 1)
    
    if [ "$http_code" = "200" ] || [ "$http_code" = "201" ]; then
        # Check if correlation ID is returned in headers
        returned_correlation=$(echo "$headers" | jq -r '.["x-correlation-id"] // .["X-Correlation-ID"] // empty' 2>/dev/null || echo "")
        
        if [ "$returned_correlation" = "$CORRELATION_ID" ]; then
            echo -e "${GREEN}✅ Passed${NC}"
            return 0
        else
            echo -e "${YELLOW}⚠️  Response missing correlation ID${NC}"
            return 1
        fi
    else
        echo -e "${RED}❌ Failed (HTTP $http_code)${NC}"
        return 1
    fi
}

# Function to test log aggregation
test_log_aggregation() {
    echo -n "Testing log aggregation... "
    
    # Wait for logs to be processed
    sleep 2
    
    # Query Loki for logs with our correlation ID
    query="{job=~\"ai.*\"} | json | correlation_id = \"$CORRELATION_ID\""
    encoded_query=$(python3 -c "import urllib.parse; print(urllib.parse.quote('$query'))")
    
    response=$(curl -s "$LOKI_URL/loki/api/v1/query_range?query=$encoded_query&limit=100&start=$(date -d '5 minutes ago' --iso-8601)&end=$(date --iso-8601)" 2>/dev/null || echo "error")
    
    if echo "$response" | jq -e '.data.result | length > 0' > /dev/null 2>&1; then
        log_count=$(echo "$response" | jq '.data.result | length')
        echo -e "${GREEN}✅ Found $log_count log streams${NC}"
        return 0
    else
        echo -e "${YELLOW}⚠️  No logs found (may take time to aggregate)${NC}"
        return 1
    fi
}

# Function to test metrics correlation
test_metrics_correlation() {
    echo -n "Testing metrics correlation... "
    
    # Query Prometheus for metrics
    query="up{job=~\"ai.*\"}"
    response=$(curl -s "$PROMETHEUS_URL/api/v1/query?query=$query" 2>/dev/null || echo "error")
    
    if echo "$response" | jq -e '.status == "success"' > /dev/null 2>&1; then
        metric_count=$(echo "$response" | jq '.data.result | length')
        echo -e "${GREEN}✅ Found $metric_count metrics${NC}"
        return 0
    else
        echo -e "${YELLOW}⚠️  Metrics query failed${NC}"
        return 1
    fi
}

# Main test execution
main() {
    local failed_tests=0
    local total_tests=0
    
    echo -e "\n${BLUE}📋 Step 1: Checking Service Availability${NC}"
    echo "----------------------------------------"
    
    services=(
        "$BASE_URL_WORKFLOW/api/v1/health:AI Workflow System"
        "$BASE_URL_TUTOR/health:AI Tutor Service"
        "$LOKI_URL/ready:Loki"
        "$PROMETHEUS_URL/-/ready:Prometheus"
    )
    
    for service in "${services[@]}"; do
        url="${service%%:*}"
        name="${service##*:}"
        total_tests=$((total_tests + 1))
        if ! check_service "$url" "$name"; then
            failed_tests=$((failed_tests + 1))
        fi
    done
    
    echo -e "\n${BLUE}📊 Step 2: Testing Correlation ID Propagation${NC}"
    echo "---------------------------------------------"
    
    # Test workflow system health endpoint
    total_tests=$((total_tests + 1))
    if ! test_correlation_propagation "$BASE_URL_WORKFLOW/api/v1/health" "Workflow System Health"; then
        failed_tests=$((failed_tests + 1))
    fi
    
    # Test AI tutor health endpoint
    total_tests=$((total_tests + 1))
    if ! test_correlation_propagation "$BASE_URL_TUTOR/health" "AI Tutor Health"; then
        failed_tests=$((failed_tests + 1))
    fi
    
    # Test AI tutor tutoring endpoint
    tutor_payload='{
        "student_query": "What is correlation tracking?",
        "subject": "computer science",
        "difficulty_level": "intermediate"
    }'
    total_tests=$((total_tests + 1))
    if ! test_correlation_propagation "$BASE_URL_TUTOR/tutor" "AI Tutor Service" "POST" "$tutor_payload"; then
        failed_tests=$((failed_tests + 1))
    fi
    
    # Test workflow triggering
    workflow_payload='{
        "workflow_name": "research_to_documentation",
        "inputs": {
            "topic": "correlation testing",
            "difficulty": "beginner"
        },
        "config": {
            "timeout": 60
        }
    }'
    total_tests=$((total_tests + 1))
    if ! test_correlation_propagation "$BASE_URL_WORKFLOW/api/v1/workflows/trigger" "Workflow Trigger" "POST" "$workflow_payload"; then
        failed_tests=$((failed_tests + 1))
    fi
    
    echo -e "\n${BLUE}📝 Step 3: Testing Log Aggregation${NC}"
    echo "-----------------------------------"
    total_tests=$((total_tests + 1))
    if ! test_log_aggregation; then
        failed_tests=$((failed_tests + 1))
    fi
    
    echo -e "\n${BLUE}📈 Step 4: Testing Metrics Collection${NC}"
    echo "------------------------------------"
    total_tests=$((total_tests + 1))
    if ! test_metrics_correlation; then
        failed_tests=$((failed_tests + 1))
    fi
    
    # Test correlation ID generation (request without correlation ID)
    echo -e "\n${BLUE}🆔 Step 5: Testing Correlation ID Generation${NC}"
    echo "-------------------------------------------"
    echo -n "Testing ID generation... "
    
    response=$(curl -s -w "\n%{header_json}" "$BASE_URL_WORKFLOW/api/v1/health" 2>/dev/null)
    headers=$(echo "$response" | tail -n 1)
    generated_id=$(echo "$headers" | jq -r '.["x-correlation-id"] // .["X-Correlation-ID"] // empty' 2>/dev/null || echo "")
    
    total_tests=$((total_tests + 1))
    if [ -n "$generated_id" ]; then
        echo -e "${GREEN}✅ Generated ID: $generated_id${NC}"
    else
        echo -e "${RED}❌ No ID generated${NC}"
        failed_tests=$((failed_tests + 1))
    fi
    
    # Print summary
    echo -e "\n${BLUE}📊 Test Summary${NC}"
    echo "==============="
    echo "Total Tests: $total_tests"
    echo "Passed: $((total_tests - failed_tests))"
    echo "Failed: $failed_tests"
    
    if [ $failed_tests -eq 0 ]; then
        echo -e "\n${GREEN}🎉 All tests passed! Correlation tracking is working correctly.${NC}"
        echo -e "\n${BLUE}🔍 Next Steps:${NC}"
        echo "1. View correlation dashboard: http://localhost:3000/d/ai-workflow-correlation"
        echo "2. Search logs by correlation ID: http://localhost:3100"
        echo "3. View traces: http://localhost:16686"
        echo "4. Monitor metrics: http://localhost:9090"
        echo -e "\n${BLUE}📝 Your test correlation ID: ${CORRELATION_ID}${NC}"
        exit 0
    else
        echo -e "\n${RED}❌ Some tests failed. Please check the service logs and configurations.${NC}"
        echo -e "\n${YELLOW}💡 Troubleshooting Tips:${NC}"
        echo "1. Ensure all services are running: docker-compose up -d"
        echo "2. Start monitoring stack: docker-compose -f docker-compose.monitoring.yml up -d"
        echo "3. Check service logs: docker-compose logs [service-name]"
        echo "4. Verify correlation middleware is enabled in Python services"
        echo "5. Check that structured logging is configured in Rust services"
        exit 1
    fi
}

# Check dependencies
check_dependencies() {
    local missing_deps=()
    
    if ! command -v curl &> /dev/null; then
        missing_deps+=("curl")
    fi
    
    if ! command -v jq &> /dev/null; then
        missing_deps+=("jq")
    fi
    
    if ! command -v python3 &> /dev/null; then
        missing_deps+=("python3")
    fi
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        echo -e "${RED}❌ Missing required dependencies: ${missing_deps[*]}${NC}"
        echo "Please install them and try again."
        exit 1
    fi
}

# Print usage
print_usage() {
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  -h, --help              Show this help message"
    echo "  -c, --correlation-id    Use specific correlation ID"
    echo "  -v, --verbose           Enable verbose output"
    echo ""
    echo "Examples:"
    echo "  $0                      Run full correlation test"
    echo "  $0 -c my-test-id        Use specific correlation ID"
    echo "  $0 -v                   Run with verbose output"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            print_usage
            exit 0
            ;;
        -c|--correlation-id)
            CORRELATION_ID="$2"
            shift 2
            ;;
        -v|--verbose)
            set -x
            shift
            ;;
        *)
            echo "Unknown option: $1"
            print_usage
            exit 1
            ;;
    esac
done

# Run the tests
check_dependencies
main