#!/bin/bash
# Integration Test Runner for CI/CD
# This script sets up Dgraph, runs integration tests, and cleans up

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../../" && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
MAX_SETUP_TIME=300  # Maximum time to wait for setup (seconds)
CLEANUP_ON_FAILURE=true
KEEP_RUNNING=false

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_debug() {
    echo -e "${BLUE}[DEBUG]${NC} $1"
}

usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --keep-running     Keep Dgraph running after tests (for debugging)"
    echo "  --no-cleanup       Don't cleanup on test failure"
    echo "  --setup-only       Only setup Dgraph, don't run tests"
    echo "  --teardown-only    Only teardown existing Dgraph"
    echo "  --help             Show this help message"
    echo ""
    echo "Test categories:"
    echo "  --query-tests      Run query integration tests only"
    echo "  --mutation-tests   Run mutation tests only"
    echo "  --transaction-tests Run transaction tests only"
    echo "  --all-tests        Run all integration tests (default)"
    echo ""
    echo "Examples:"
    echo "  $0                           # Run all tests with cleanup"
    echo "  $0 --keep-running            # Run tests and keep Dgraph running"
    echo "  $0 --setup-only              # Only setup test environment"
    echo "  $0 --mutation-tests          # Run only mutation tests"
}

setup_environment() {
    log_info "Setting up test environment..."
    
    if ! "$SCRIPT_DIR/test-dgraph-setup.sh"; then
        log_error "Failed to setup test environment"
        return 1
    fi
    
    log_info "Test environment ready"
    return 0
}

teardown_environment() {
    log_info "Tearing down test environment..."
    
    if "$SCRIPT_DIR/test-dgraph-teardown.sh"; then
        log_info "Test environment cleaned up"
    else
        log_warn "Cleanup may not have completed successfully"
    fi
}

run_test_category() {
    local test_pattern="$1"
    local test_name="$2"
    
    log_info "Running $test_name..."
    
    cd "$PROJECT_ROOT"
    
    # Run the specific test category
    if cargo test "$test_pattern" --ignored -- --test-threads=1; then
        log_info "$test_name completed successfully"
        return 0
    else
        log_error "$test_name failed"
        return 1
    fi
}

run_all_tests() {
    local failed_tests=()
    local test_results=()
    
    log_info "Running all Dgraph integration tests..."
    
    # Test categories to run
    local tests=(
        "knowledge_graph_integration:Integration Tests"
        "knowledge_graph_mutation:Mutation Tests"
        "knowledge_graph_transaction:Transaction Tests"
    )
    
    for test_info in "${tests[@]}"; do
        IFS=':' read -r test_pattern test_name <<< "$test_info"
        
        log_info "Starting $test_name..."
        
        if run_test_category "$test_pattern" "$test_name"; then
            test_results+=("✓ $test_name")
        else
            test_results+=("✗ $test_name")
            failed_tests+=("$test_name")
        fi
        
        echo ""
    done
    
    # Print summary
    echo ""
    log_info "Test Summary:"
    for result in "${test_results[@]}"; do
        if [[ $result == ✓* ]]; then
            echo -e "  ${GREEN}$result${NC}"
        else
            echo -e "  ${RED}$result${NC}"
        fi
    done
    
    if [ ${#failed_tests[@]} -eq 0 ]; then
        log_info "All tests passed!"
        return 0
    else
        log_error "Failed tests: ${failed_tests[*]}"
        return 1
    fi
}

check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check if we're in the right directory
    if [ ! -f "$PROJECT_ROOT/Cargo.toml" ]; then
        log_error "Not in a Rust project root directory"
        return 1
    fi
    
    # Check if test files exist
    local test_files=(
        "$PROJECT_ROOT/tests/knowledge_graph_integration_tests.rs"
        "$PROJECT_ROOT/tests/knowledge_graph_mutation_tests.rs"
        "$PROJECT_ROOT/tests/knowledge_graph_transaction_tests.rs"
    )
    
    for test_file in "${test_files[@]}"; do
        if [ ! -f "$test_file" ]; then
            log_error "Test file not found: $test_file"
            return 1
        fi
    done
    
    # Check if Docker is available
    if ! command -v docker &> /dev/null; then
        log_error "Docker is required but not installed"
        return 1
    fi
    
    # Check if cargo is available
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo is required but not installed"
        return 1
    fi
    
    log_info "Prerequisites check passed"
    return 0
}

monitor_services() {
    log_info "Monitoring service health..."
    
    # Check if services are still running
    local alpha_healthy=false
    local zero_healthy=false
    
    if curl -s -f "http://localhost:18080/health" > /dev/null 2>&1; then
        alpha_healthy=true
    fi
    
    if curl -s -f "http://localhost:16080/health" > /dev/null 2>&1; then
        zero_healthy=true
    fi
    
    if [ "$alpha_healthy" = true ] && [ "$zero_healthy" = true ]; then
        log_info "All services are healthy"
        return 0
    else
        log_warn "Some services may not be healthy (Alpha: $alpha_healthy, Zero: $zero_healthy)"
        return 1
    fi
}

cleanup_on_exit() {
    local exit_code=$?
    
    if [ $exit_code -ne 0 ] && [ "$CLEANUP_ON_FAILURE" = true ]; then
        log_info "Test failed, cleaning up..."
        teardown_environment
    elif [ "$KEEP_RUNNING" = false ]; then
        log_info "Tests completed, cleaning up..."
        teardown_environment
    else
        log_info "Keeping test environment running for debugging"
        echo ""
        echo "Dgraph endpoints:"
        echo "  Alpha: http://localhost:18080"
        echo "  Zero:  http://localhost:16080"
        echo ""
        echo "To cleanup later: $SCRIPT_DIR/test-dgraph-teardown.sh"
    fi
    
    exit $exit_code
}

main() {
    local setup_only=false
    local teardown_only=false
    local test_category=""
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --keep-running)
                KEEP_RUNNING=true
                shift
                ;;
            --no-cleanup)
                CLEANUP_ON_FAILURE=false
                shift
                ;;
            --setup-only)
                setup_only=true
                shift
                ;;
            --teardown-only)
                teardown_only=true
                shift
                ;;
            --query-tests)
                test_category="knowledge_graph_integration"
                shift
                ;;
            --mutation-tests)
                test_category="knowledge_graph_mutation"
                shift
                ;;
            --transaction-tests)
                test_category="knowledge_graph_transaction"
                shift
                ;;
            --all-tests)
                test_category="all"
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
    
    # Set default test category
    if [ -z "$test_category" ]; then
        test_category="all"
    fi
    
    # Handle teardown-only
    if [ "$teardown_only" = true ]; then
        teardown_environment
        exit 0
    fi
    
    # Setup exit trap
    trap cleanup_on_exit EXIT INT TERM
    
    # Check prerequisites
    if ! check_prerequisites; then
        exit 1
    fi
    
    # Setup environment
    if ! setup_environment; then
        log_error "Failed to setup test environment"
        exit 1
    fi
    
    # Handle setup-only
    if [ "$setup_only" = true ]; then
        log_info "Setup completed. Environment is ready for manual testing."
        KEEP_RUNNING=true
        exit 0
    fi
    
    # Monitor services before running tests
    if ! monitor_services; then
        log_error "Services are not healthy, aborting tests"
        exit 1
    fi
    
    # Run tests based on category
    case $test_category in
        "all")
            run_all_tests
            ;;
        "knowledge_graph_integration")
            run_test_category "knowledge_graph_integration" "Integration Tests"
            ;;
        "knowledge_graph_mutation")
            run_test_category "knowledge_graph_mutation" "Mutation Tests"
            ;;
        "knowledge_graph_transaction")
            run_test_category "knowledge_graph_transaction" "Transaction Tests"
            ;;
        *)
            log_error "Unknown test category: $test_category"
            exit 1
            ;;
    esac
}

# Print header
echo ""
log_info "Dgraph Integration Test Runner"
log_info "=============================="
echo ""

# Run main function
main "$@"