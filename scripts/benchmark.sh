#!/bin/bash
#
# Benchmark Script for AI Workflow Engine
#
# This script runs performance benchmarks to validate the performance claims:
# - 15,000+ requests/second API throughput
# - Sub-millisecond node processing
#
# Usage:
#   ./scripts/benchmark.sh [options]
#
# Options:
#   --all           Run all benchmarks
#   --api           Run API throughput benchmarks
#   --node          Run node processing benchmarks
#   --workflow      Run workflow execution benchmarks
#   --quick         Run quick benchmarks (reduced time)
#   --save          Save results to file
#   --compare FILE  Compare with previous results
#

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BENCHMARK_DIR="target/criterion"
RESULTS_DIR="benchmark-results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Parse arguments
RUN_ALL=false
RUN_API=false
RUN_NODE=false
RUN_WORKFLOW=false
QUICK_MODE=false
SAVE_RESULTS=false
COMPARE_FILE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --all)
            RUN_ALL=true
            shift
            ;;
        --api)
            RUN_API=true
            shift
            ;;
        --node)
            RUN_NODE=true
            shift
            ;;
        --workflow)
            RUN_WORKFLOW=true
            shift
            ;;
        --quick)
            QUICK_MODE=true
            shift
            ;;
        --save)
            SAVE_RESULTS=true
            shift
            ;;
        --compare)
            COMPARE_FILE="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Default to running all if no specific benchmark selected
if [[ "$RUN_ALL" == false && "$RUN_API" == false && "$RUN_NODE" == false && "$RUN_WORKFLOW" == false ]]; then
    RUN_ALL=true
fi

# Functions
print_header() {
    echo -e "\n${BLUE}===================================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}===================================================${NC}\n"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

check_dependencies() {
    print_header "Checking Dependencies"
    
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo not found. Please install Rust."
        exit 1
    fi
    
    if ! cargo --list | grep -q "criterion"; then
        print_warning "Installing cargo-criterion..."
        cargo install cargo-criterion
    fi
    
    print_success "All dependencies satisfied"
}

prepare_environment() {
    print_header "Preparing Benchmark Environment"
    
    # Create results directory
    mkdir -p "$RESULTS_DIR"
    
    # Set performance mode if available (Linux)
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if command -v cpupower &> /dev/null; then
            print_warning "Setting CPU to performance mode (may require sudo)"
            sudo cpupower frequency-set -g performance 2>/dev/null || true
        fi
    fi
    
    # Build in release mode
    print_success "Building in release mode..."
    cargo build --release --features full
    
    print_success "Environment prepared"
}

run_api_benchmarks() {
    print_header "Running API Throughput Benchmarks"
    
    BENCH_ARGS=""
    if [[ "$QUICK_MODE" == true ]]; then
        BENCH_ARGS="--warm-up-time 1 --measurement-time 5"
    fi
    
    cargo bench --bench api_throughput $BENCH_ARGS
    
    # Extract throughput results
    if [[ -f "$BENCHMARK_DIR/api_throughput/report/index.html" ]]; then
        print_success "API throughput benchmark completed"
        
        # Parse and display key metrics
        echo -e "\n${YELLOW}Key Metrics:${NC}"
        
        # Extract throughput from criterion output
        if command -v jq &> /dev/null && [[ -f "$BENCHMARK_DIR/api_throughput/small_payload/base/estimates.json" ]]; then
            THROUGHPUT=$(jq -r '.mean.point_estimate' < "$BENCHMARK_DIR/api_throughput/small_payload/base/estimates.json" 2>/dev/null || echo "N/A")
            if [[ "$THROUGHPUT" != "N/A" ]]; then
                # Convert to requests/second
                RPS=$(echo "scale=2; 15000 / $THROUGHPUT * 1000000000" | bc 2>/dev/null || echo "N/A")
                echo "Small payload throughput: ~$RPS requests/second"
            fi
        fi
    fi
}

run_node_benchmarks() {
    print_header "Running Node Processing Benchmarks"
    
    BENCH_ARGS=""
    if [[ "$QUICK_MODE" == true ]]; then
        BENCH_ARGS="--warm-up-time 1 --measurement-time 5"
    fi
    
    cargo bench --bench node_processing $BENCH_ARGS
    
    if [[ -f "$BENCHMARK_DIR/node_processing/report/index.html" ]]; then
        print_success "Node processing benchmark completed"
        
        # Parse and display key metrics
        echo -e "\n${YELLOW}Key Metrics:${NC}"
        
        # Check for sub-millisecond processing
        for bench in simple_compute_low simple_compute_medium json_transform; do
            if [[ -f "$BENCHMARK_DIR/single_node_processing/$bench/base/estimates.json" ]]; then
                TIME_NS=$(jq -r '.mean.point_estimate' < "$BENCHMARK_DIR/single_node_processing/$bench/base/estimates.json" 2>/dev/null || echo "N/A")
                if [[ "$TIME_NS" != "N/A" ]]; then
                    TIME_MS=$(echo "scale=3; $TIME_NS / 1000000" | bc 2>/dev/null || echo "N/A")
                    echo "$bench: $TIME_MS ms"
                    
                    # Check if sub-millisecond
                    if (( $(echo "$TIME_MS < 1.0" | bc -l) )); then
                        print_success "✓ Sub-millisecond processing achieved"
                    fi
                fi
            fi
        done
    fi
}

run_workflow_benchmarks() {
    print_header "Running Workflow Execution Benchmarks"
    
    BENCH_ARGS=""
    if [[ "$QUICK_MODE" == true ]]; then
        BENCH_ARGS="--warm-up-time 1 --measurement-time 5"
    fi
    
    cargo bench --bench workflow_execution $BENCH_ARGS
    
    if [[ -f "$BENCHMARK_DIR/workflow_execution/report/index.html" ]]; then
        print_success "Workflow execution benchmark completed"
    fi
}

generate_report() {
    print_header "Generating Benchmark Report"
    
    REPORT_FILE="$RESULTS_DIR/benchmark_report_$TIMESTAMP.md"
    
    cat > "$REPORT_FILE" << EOF
# AI Workflow Engine Performance Benchmark Report

Generated: $(date)
System: $(uname -a)
Rust Version: $(rustc --version)

## Executive Summary

This report validates the performance claims of the AI Workflow Engine:
- **API Throughput**: Target 15,000+ requests/second
- **Node Processing**: Target sub-millisecond execution

## Benchmark Results

### API Throughput Benchmarks

EOF

    # Add API results
    if [[ -d "$BENCHMARK_DIR/api_throughput" ]]; then
        echo "#### Small Payload (256 bytes)" >> "$REPORT_FILE"
        echo "- Concurrent clients: 100" >> "$REPORT_FILE"
        echo "- Keep-alive: enabled" >> "$REPORT_FILE"
        
        # Extract and add metrics
        for metric in small_payload medium_payload large_payload; do
            if [[ -f "$BENCHMARK_DIR/api_throughput/$metric/base/estimates.json" ]]; then
                TIME=$(jq -r '.mean.point_estimate' < "$BENCHMARK_DIR/api_throughput/$metric/base/estimates.json" 2>/dev/null || echo "N/A")
                echo "- $metric: $TIME ns" >> "$REPORT_FILE"
            fi
        done
    fi
    
    cat >> "$REPORT_FILE" << EOF

### Node Processing Benchmarks

| Node Type | Processing Time | Sub-millisecond? |
|-----------|----------------|------------------|
EOF

    # Add node processing results
    for bench in simple_compute_low simple_compute_medium json_transform; do
        if [[ -f "$BENCHMARK_DIR/single_node_processing/$bench/base/estimates.json" ]]; then
            TIME_NS=$(jq -r '.mean.point_estimate' < "$BENCHMARK_DIR/single_node_processing/$bench/base/estimates.json" 2>/dev/null || echo "N/A")
            if [[ "$TIME_NS" != "N/A" ]]; then
                TIME_MS=$(echo "scale=3; $TIME_NS / 1000000" | bc 2>/dev/null || echo "N/A")
                SUB_MS="No"
                if (( $(echo "$TIME_MS < 1.0" | bc -l) )); then
                    SUB_MS="Yes ✓"
                fi
                echo "| $bench | $TIME_MS ms | $SUB_MS |" >> "$REPORT_FILE"
            fi
        fi
    done
    
    cat >> "$REPORT_FILE" << EOF

### Workflow Execution Benchmarks

Benchmarks for complete workflow execution including routing, parallel processing, and error handling.

EOF

    # Add workflow results summary
    
    cat >> "$REPORT_FILE" << EOF

## Performance Validation

### API Throughput Goal: 15,000+ requests/second
**Status**: [PENDING VALIDATION]

### Node Processing Goal: Sub-millisecond
**Status**: [PENDING VALIDATION]

## Recommendations

Based on the benchmark results:
1. [Add specific recommendations based on results]
2. [Add performance tuning suggestions]

## Hardware Specifications

- CPU: $(sysctl -n machdep.cpu.brand_string 2>/dev/null || lscpu | grep "Model name" | cut -d: -f2 | xargs)
- Memory: $(free -h 2>/dev/null | grep Mem | awk '{print $2}' || sysctl -n hw.memsize 2>/dev/null | awk '{print $1/1024/1024/1024 " GB"}')
- OS: $(uname -s) $(uname -r)

## Appendix

Full benchmark results available in: \`$BENCHMARK_DIR\`

To view detailed results:
\`\`\`bash
open $BENCHMARK_DIR/report/index.html
\`\`\`
EOF

    print_success "Report generated: $REPORT_FILE"
    
    if [[ "$SAVE_RESULTS" == true ]]; then
        # Archive full results
        tar -czf "$RESULTS_DIR/benchmark_results_$TIMESTAMP.tar.gz" -C target criterion
        print_success "Results archived: $RESULTS_DIR/benchmark_results_$TIMESTAMP.tar.gz"
    fi
}

compare_results() {
    if [[ -n "$COMPARE_FILE" && -f "$COMPARE_FILE" ]]; then
        print_header "Comparing with Previous Results"
        
        # Basic comparison logic
        echo "Comparison functionality to be implemented"
        # TODO: Implement comparison logic
    fi
}

cleanup() {
    print_header "Cleanup"
    
    # Reset CPU governor if changed
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if command -v cpupower &> /dev/null; then
            sudo cpupower frequency-set -g ondemand 2>/dev/null || true
        fi
    fi
    
    print_success "Cleanup completed"
}

# Main execution
main() {
    print_header "AI Workflow Engine Performance Benchmarks"
    
    check_dependencies
    prepare_environment
    
    # Run selected benchmarks
    if [[ "$RUN_ALL" == true || "$RUN_API" == true ]]; then
        run_api_benchmarks
    fi
    
    if [[ "$RUN_ALL" == true || "$RUN_NODE" == true ]]; then
        run_node_benchmarks
    fi
    
    if [[ "$RUN_ALL" == true || "$RUN_WORKFLOW" == true ]]; then
        run_workflow_benchmarks
    fi
    
    # Generate report
    generate_report
    
    # Compare if requested
    compare_results
    
    # Cleanup
    cleanup
    
    print_header "Benchmark Complete"
    echo -e "${GREEN}View detailed results:${NC}"
    echo "  open $BENCHMARK_DIR/report/index.html"
    echo ""
    echo -e "${GREEN}View summary report:${NC}"
    echo "  cat $RESULTS_DIR/benchmark_report_$TIMESTAMP.md"
}

# Trap to ensure cleanup runs
trap cleanup EXIT

# Run main
main