#!/bin/bash

# Complete Federation Validation Script
# 
# This script validates the entire GraphQL Federation implementation
# by running all tests and providing comprehensive reporting.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
GATEWAY_PORT=4000
WORKFLOW_API_PORT=8080
CONTENT_PROCESSING_PORT=3001
KNOWLEDGE_GRAPH_PORT=3002
REALTIME_COMMUNICATION_PORT=3003

GATEWAY_URL="http://localhost:${GATEWAY_PORT}/graphql"
WORKFLOW_API_URL="http://localhost:${WORKFLOW_API_PORT}/api/v1/graphql"
CONTENT_PROCESSING_URL="http://localhost:${CONTENT_PROCESSING_PORT}/graphql"
KNOWLEDGE_GRAPH_URL="http://localhost:${KNOWLEDGE_GRAPH_PORT}/graphql"
REALTIME_COMMUNICATION_URL="http://localhost:${REALTIME_COMMUNICATION_PORT}/graphql"

# Logging
LOG_DIR="${PROJECT_ROOT}/logs"
mkdir -p "$LOG_DIR"
VALIDATION_LOG="${LOG_DIR}/federation_validation_$(date +%Y%m%d_%H%M%S).log"

# Test results tracking
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
WARNINGS=0

# PID tracking for cleanup
GATEWAY_PID=""
SERVICE_PIDS=()

echo_header() {
    echo -e "${PURPLE}========================================${NC}"
    echo -e "${PURPLE}$1${NC}"
    echo -e "${PURPLE}========================================${NC}"
}

echo_section() {
    echo -e "\n${BLUE}📋 $1${NC}"
    echo -e "${BLUE}$(printf '=%.0s' {1..40})${NC}"
}

echo_subsection() {
    echo -e "\n${CYAN}  🔍 $1${NC}"
}

echo_success() {
    echo -e "${GREEN}  ✅ $1${NC}"
    ((PASSED_TESTS++))
}

echo_warning() {
    echo -e "${YELLOW}  ⚠️  $1${NC}"
    ((WARNINGS++))
}

echo_error() {
    echo -e "${RED}  ❌ $1${NC}"
    ((FAILED_TESTS++))
}

echo_info() {
    echo -e "${CYAN}  ℹ️  $1${NC}"
}

# Logging function
log() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') - $1" >> "$VALIDATION_LOG"
}

# Cleanup function
cleanup() {
    echo_section "Cleaning up services"
    
    if [ -n "$GATEWAY_PID" ]; then
        echo_info "Stopping GraphQL Gateway (PID: $GATEWAY_PID)"
        kill $GATEWAY_PID 2>/dev/null || true
    fi
    
    for pid in "${SERVICE_PIDS[@]}"; do
        if [ -n "$pid" ]; then
            echo_info "Stopping service (PID: $pid)"
            kill $pid 2>/dev/null || true
        fi
    done
    
    # Wait for processes to terminate
    sleep 3
    
    # Force kill if still running
    if [ -n "$GATEWAY_PID" ]; then
        kill -9 $GATEWAY_PID 2>/dev/null || true
    fi
    
    for pid in "${SERVICE_PIDS[@]}"; do
        if [ -n "$pid" ]; then
            kill -9 $pid 2>/dev/null || true
        fi
    done
    
    echo_info "Cleanup completed"
}

# Set up trap for cleanup on exit
trap cleanup EXIT INT TERM

# Health check function
check_service_health() {
    local service_name="$1"
    local url="$2"
    local max_attempts=30
    local attempt=1
    
    echo_subsection "Health checking $service_name"
    
    while [ $attempt -le $max_attempts ]; do
        if curl -s -X POST \
            -H "Content-Type: application/json" \
            -d '{"query": "{ __schema { queryType { name } } }"}' \
            "$url" > /dev/null 2>&1; then
            echo_success "$service_name is healthy (attempt $attempt)"
            return 0
        fi
        
        echo_info "$service_name not ready, attempt $attempt/$max_attempts"
        sleep 2
        ((attempt++))
    done
    
    echo_error "$service_name failed health check after $max_attempts attempts"
    return 1
}

# Start services function
start_services() {
    echo_section "Starting Federation Services"
    
    cd "$PROJECT_ROOT"
    
    # Start microservices
    echo_subsection "Starting Content Processing Service"
    cd services/content_processing
    cargo run --release > "$LOG_DIR/content_processing.log" 2>&1 &
    local content_pid=$!
    SERVICE_PIDS+=($content_pid)
    echo_info "Content Processing Service started (PID: $content_pid)"
    cd "$PROJECT_ROOT"
    
    echo_subsection "Starting Knowledge Graph Service"
    cd services/knowledge_graph
    cargo run --release > "$LOG_DIR/knowledge_graph.log" 2>&1 &
    local knowledge_pid=$!
    SERVICE_PIDS+=($knowledge_pid)
    echo_info "Knowledge Graph Service started (PID: $knowledge_pid)"
    cd "$PROJECT_ROOT"
    
    echo_subsection "Starting Realtime Communication Service"
    cd services/realtime_communication
    cargo run --release > "$LOG_DIR/realtime_communication.log" 2>&1 &
    local realtime_pid=$!
    SERVICE_PIDS+=($realtime_pid)
    echo_info "Realtime Communication Service started (PID: $realtime_pid)"
    cd "$PROJECT_ROOT"
    
    echo_subsection "Starting Workflow Engine API"
    cargo run --bin workflow-engine --release > "$LOG_DIR/workflow_engine.log" 2>&1 &
    local workflow_pid=$!
    SERVICE_PIDS+=($workflow_pid)
    echo_info "Workflow Engine API started (PID: $workflow_pid)"
    
    echo_subsection "Starting GraphQL Gateway"
    cargo run --bin graphql-gateway --release > "$LOG_DIR/gateway.log" 2>&1 &
    GATEWAY_PID=$!
    echo_info "GraphQL Gateway started (PID: $GATEWAY_PID)"
    
    # Allow services to start
    echo_info "Waiting for services to initialize..."
    sleep 10
}

# Validate service health
validate_service_health() {
    echo_section "Validating Service Health"
    
    local all_healthy=true
    
    # Check all services
    if ! check_service_health "Content Processing" "$CONTENT_PROCESSING_URL"; then
        all_healthy=false
    fi
    
    if ! check_service_health "Knowledge Graph" "$KNOWLEDGE_GRAPH_URL"; then
        all_healthy=false
    fi
    
    if ! check_service_health "Realtime Communication" "$REALTIME_COMMUNICATION_URL"; then
        all_healthy=false
    fi
    
    if ! check_service_health "Workflow Engine API" "$WORKFLOW_API_URL"; then
        all_healthy=false
    fi
    
    if ! check_service_health "GraphQL Gateway" "$GATEWAY_URL"; then
        all_healthy=false
    fi
    
    if [ "$all_healthy" = true ]; then
        echo_success "All services are healthy and ready for testing"
        return 0
    else
        echo_error "Some services failed health checks"
        return 1
    fi
}

# Run federation schema validation
validate_federation_schemas() {
    echo_section "Validating Federation Schemas"
    
    echo_subsection "Running federation validation example"
    if cd crates/workflow-engine-gateway && cargo run --example validate_federation > "$LOG_DIR/schema_validation.log" 2>&1; then
        echo_success "Federation schema validation passed"
    else
        echo_error "Federation schema validation failed"
        echo_info "Check $LOG_DIR/schema_validation.log for details"
    fi
    
    cd "$PROJECT_ROOT"
}

# Run gateway integration tests
run_gateway_integration_tests() {
    echo_section "Running Gateway Integration Tests (Tests 16-18)"
    
    echo_subsection "Test 16: Multi-Subgraph Query Test"
    ((TOTAL_TESTS++))
    if cargo test test_16_multi_subgraph_query -- --ignored --nocapture > "$LOG_DIR/test_16.log" 2>&1; then
        echo_success "Test 16 passed"
    else
        echo_error "Test 16 failed"
        echo_info "Check $LOG_DIR/test_16.log for details"
    fi
    
    echo_subsection "Test 17: Entity Reference Resolution Test"
    ((TOTAL_TESTS++))
    if cargo test test_17_entity_reference_resolution -- --ignored --nocapture > "$LOG_DIR/test_17.log" 2>&1; then
        echo_success "Test 17 passed"
    else
        echo_error "Test 17 failed"
        echo_info "Check $LOG_DIR/test_17.log for details"
    fi
    
    echo_subsection "Test 18: Schema Composition Test"
    ((TOTAL_TESTS++))
    if cargo test test_18_schema_composition -- --ignored --nocapture > "$LOG_DIR/test_18.log" 2>&1; then
        echo_success "Test 18 passed"
    else
        echo_error "Test 18 failed"
        echo_info "Check $LOG_DIR/test_18.log for details"
    fi
}

# Run end-to-end federation tests
run_end_to_end_tests() {
    echo_section "Running End-to-End Federation Tests (Tests 19-20)"
    
    echo_subsection "Test 19: Complete Workflow Query Test"
    ((TOTAL_TESTS++))
    if cargo test test_19_complete_workflow_query -- --ignored --nocapture > "$LOG_DIR/test_19.log" 2>&1; then
        echo_success "Test 19 passed"
    else
        echo_error "Test 19 failed"
        echo_info "Check $LOG_DIR/test_19.log for details"
    fi
    
    echo_subsection "Test 20: Performance Test with Caching"
    ((TOTAL_TESTS++))
    if cargo test test_20_performance_with_caching -- --ignored --nocapture > "$LOG_DIR/test_20.log" 2>&1; then
        echo_success "Test 20 passed"
    else
        echo_error "Test 20 failed"
        echo_info "Check $LOG_DIR/test_20.log for details"
    fi
}

# Run comprehensive performance suite
run_performance_suite() {
    echo_section "Running Comprehensive Performance Suite"
    
    echo_subsection "Complete Federation Performance Test Suite"
    ((TOTAL_TESTS++))
    if cargo test run_complete_federation_performance_suite -- --ignored --nocapture > "$LOG_DIR/performance_suite.log" 2>&1; then
        echo_success "Performance suite passed"
    else
        echo_error "Performance suite failed"
        echo_info "Check $LOG_DIR/performance_suite.log for details"
    fi
}

# Service-specific tests
run_service_specific_tests() {
    echo_section "Running Service-Specific Federation Tests"
    
    echo_subsection "Content Processing Federation Tests"
    ((TOTAL_TESTS++))
    if cd services/content_processing && cargo test graphql_federation_test -- --ignored --nocapture > "$LOG_DIR/content_federation.log" 2>&1; then
        echo_success "Content Processing federation tests passed"
    else
        echo_error "Content Processing federation tests failed"
        echo_info "Check $LOG_DIR/content_federation.log for details"
    fi
    cd "$PROJECT_ROOT"
    
    echo_subsection "Knowledge Graph Federation Tests"
    ((TOTAL_TESTS++))
    if cd services/knowledge_graph && cargo test graphql_federation_test -- --ignored --nocapture > "$LOG_DIR/knowledge_federation.log" 2>&1; then
        echo_success "Knowledge Graph federation tests passed"
    else
        echo_error "Knowledge Graph federation tests failed"
        echo_info "Check $LOG_DIR/knowledge_federation.log for details"
    fi
    cd "$PROJECT_ROOT"
    
    echo_subsection "Realtime Communication Federation Tests"
    ((TOTAL_TESTS++))
    if cd services/realtime_communication && cargo test graphql_federation_test -- --ignored --nocapture > "$LOG_DIR/realtime_federation.log" 2>&1; then
        echo_success "Realtime Communication federation tests passed"
    else
        echo_error "Realtime Communication federation tests failed"
        echo_info "Check $LOG_DIR/realtime_federation.log for details"
    fi
    cd "$PROJECT_ROOT"
}

# Validate federation examples
validate_federation_examples() {
    echo_section "Validating Federation Examples"
    
    echo_subsection "Federated Query Example"
    if cd crates/workflow-engine-gateway && cargo run --example federated_query > "$LOG_DIR/federated_query_example.log" 2>&1; then
        echo_success "Federated query example passed"
    else
        echo_warning "Federated query example failed (may require test data)"
        echo_info "Check $LOG_DIR/federated_query_example.log for details"
    fi
    
    echo_subsection "Test Federation Example"
    if cargo run --example test_federation > "$LOG_DIR/test_federation_example.log" 2>&1; then
        echo_success "Test federation example passed"
    else
        echo_warning "Test federation example failed (may require test data)"
        echo_info "Check $LOG_DIR/test_federation_example.log for details"
    fi
    
    cd "$PROJECT_ROOT"
}

# Load testing
run_load_tests() {
    echo_section "Running Load Tests"
    
    echo_subsection "Gateway Load Testing"
    if command -v curl > /dev/null 2>&1; then
        echo_info "Running simple load test with curl"
        
        local success_count=0
        local total_requests=20
        
        for i in $(seq 1 $total_requests); do
            if curl -s -X POST \
                -H "Content-Type: application/json" \
                -d '{"query": "{ __schema { queryType { name } } }"}' \
                "$GATEWAY_URL" > /dev/null 2>&1; then
                ((success_count++))
            fi
        done
        
        local success_rate=$((success_count * 100 / total_requests))
        
        if [ $success_rate -ge 90 ]; then
            echo_success "Load test passed: $success_count/$total_requests requests successful ($success_rate%)"
        else
            echo_warning "Load test warning: $success_count/$total_requests requests successful ($success_rate%)"
        fi
    else
        echo_warning "curl not available for load testing"
    fi
}

# Generate test report
generate_test_report() {
    echo_section "Test Results Summary"
    
    local total_executed=$((PASSED_TESTS + FAILED_TESTS))
    local success_rate=0
    
    if [ $total_executed -gt 0 ]; then
        success_rate=$((PASSED_TESTS * 100 / total_executed))
    fi
    
    echo_info "Total Tests Executed: $total_executed"
    echo_info "Passed: $PASSED_TESTS"
    echo_info "Failed: $FAILED_TESTS"
    echo_info "Warnings: $WARNINGS"
    echo_info "Success Rate: $success_rate%"
    
    echo ""
    echo_info "Detailed logs available in: $LOG_DIR"
    echo_info "Main validation log: $VALIDATION_LOG"
    
    # Log summary
    log "Test execution completed"
    log "Total tests: $total_executed"
    log "Passed: $PASSED_TESTS"
    log "Failed: $FAILED_TESTS"
    log "Warnings: $WARNINGS"
    log "Success rate: $success_rate%"
    
    if [ $FAILED_TESTS -eq 0 ]; then
        echo_success "All tests passed! GraphQL Federation is production-ready."
        return 0
    else
        echo_error "Some tests failed. Please review the logs and fix issues."
        return 1
    fi
}

# Main execution
main() {
    echo_header "GraphQL Federation Complete Validation"
    
    log "Starting complete federation validation"
    log "Project root: $PROJECT_ROOT"
    log "Log directory: $LOG_DIR"
    
    # Build the project first
    echo_section "Building Project"
    echo_info "Building all binaries..."
    if cargo build --release > "$LOG_DIR/build.log" 2>&1; then
        echo_success "Project built successfully"
    else
        echo_error "Project build failed"
        echo_info "Check $LOG_DIR/build.log for details"
        exit 1
    fi
    
    # Start services
    start_services
    
    # Validate service health
    if ! validate_service_health; then
        echo_error "Service health validation failed"
        exit 1
    fi
    
    # Run all validation phases
    validate_federation_schemas
    run_gateway_integration_tests
    run_end_to_end_tests
    run_performance_suite
    run_service_specific_tests
    validate_federation_examples
    run_load_tests
    
    # Generate final report
    if generate_test_report; then
        echo_header "🎉 Federation Validation Completed Successfully!"
        exit 0
    else
        echo_header "❌ Federation Validation Failed"
        exit 1
    fi
}

# Help function
show_help() {
    echo "GraphQL Federation Complete Validation Script"
    echo ""
    echo "Usage: $0 [OPTION]"
    echo ""
    echo "Options:"
    echo "  help         Show this help message"
    echo "  start        Start services only"
    echo "  health       Check service health only"
    echo "  test         Run tests against running services"
    echo "  stop         Stop all services"
    echo "  logs         Show validation logs"
    echo "  clean        Clean up logs and temporary files"
    echo ""
    echo "If no option is provided, the complete validation suite will run."
}

# Command line argument handling
case "${1:-}" in
    "help"|"-h"|"--help")
        show_help
        exit 0
        ;;
    "start")
        start_services
        echo_success "All services started"
        echo_info "Services will continue running. Use '$0 stop' to stop them."
        trap - EXIT  # Disable cleanup trap
        exit 0
        ;;
    "health")
        validate_service_health
        exit $?
        ;;
    "test")
        echo_info "Running tests against existing services..."
        validate_federation_schemas
        run_gateway_integration_tests
        run_end_to_end_tests
        run_performance_suite
        run_service_specific_tests
        validate_federation_examples
        run_load_tests
        generate_test_report
        exit $?
        ;;
    "stop")
        cleanup
        exit 0
        ;;
    "logs")
        echo_info "Recent validation logs:"
        ls -la "$LOG_DIR" 2>/dev/null || echo_warning "No logs found"
        exit 0
        ;;
    "clean")
        echo_info "Cleaning up logs and temporary files..."
        rm -rf "$LOG_DIR"
        cargo clean > /dev/null 2>&1 || true
        echo_success "Cleanup completed"
        exit 0
        ;;
    "")
        # Run complete validation
        main
        ;;
    *)
        echo_error "Unknown option: $1"
        echo_info "Use '$0 help' for usage information"
        exit 1
        ;;
esac