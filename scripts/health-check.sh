#!/bin/bash
#
# AI Workflow Engine - Health Check Script
# 
# This script performs comprehensive health checks on all components
# of the AI Workflow Engine system.
#
# Usage:
#   ./health-check.sh [options]
#
# Options:
#   --json            Output results in JSON format
#   --watch           Continuously monitor (refresh every 5 seconds)
#   --verbose         Show detailed information
#

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Process command line arguments
JSON_OUTPUT=false
WATCH_MODE=false
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --json)
            JSON_OUTPUT=true
            ;;
        --watch)
            WATCH_MODE=true
            ;;
        --verbose)
            VERBOSE=true
            ;;
        *)
            ;;
    esac
    shift
done

# Function to check service health
check_service() {
    local name=$1
    local url=$2
    local port=$3
    
    if curl -s "$url" >/dev/null 2>&1; then
        echo "UP"
        return 0
    else
        # Check if port is listening
        if lsof -i:$port >/dev/null 2>&1; then
            echo "UNHEALTHY"
            return 1
        else
            echo "DOWN"
            return 1
        fi
    fi
}

# Function to get service metrics
get_metrics() {
    local url=$1
    curl -s "$url" 2>/dev/null || echo "{}"
}

# Function to perform health checks
perform_checks() {
    local results=()
    
    # Define services to check
    declare -A services=(
        ["PostgreSQL"]="5432|http://localhost:5432|Database"
        ["Redis"]="6379|http://localhost:6379|Cache"
        ["AI Workflow Engine"]="8080|http://localhost:8080/api/v1/health|Main API"
        ["Content Processing"]="8082|http://localhost:8082/health|Microservice"
        ["Knowledge Graph"]="3002|http://localhost:3002/health|Microservice"
        ["Realtime Communication"]="8081|http://localhost:8081/health|WebSocket Service"
        ["Prometheus"]="9090|http://localhost:9090/-/healthy|Metrics"
        ["Grafana"]="3000|http://localhost:3000/api/health|Dashboards"
        ["Jaeger"]="16686|http://localhost:16686/|Tracing"
        ["Frontend"]="5173|http://localhost:5173|Web UI"
    )
    
    # Special check for PostgreSQL
    pg_status="DOWN"
    if docker exec ai-workflow-db pg_isready -U aiworkflow >/dev/null 2>&1; then
        pg_status="UP"
    fi
    
    # Special check for Redis
    redis_status="DOWN"
    if docker exec ai-workflow-redis redis-cli -a redis123 ping >/dev/null 2>&1; then
        redis_status="UP"
    fi
    
    if [ "$JSON_OUTPUT" = true ]; then
        echo "{"
        echo "  \"timestamp\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\","
        echo "  \"services\": {"
        
        first=true
        for service in "${!services[@]}"; do
            IFS='|' read -r port url desc <<< "${services[$service]}"
            
            if [ "$first" = false ]; then
                echo ","
            fi
            first=false
            
            # Special handling for PostgreSQL and Redis
            if [ "$service" = "PostgreSQL" ]; then
                status=$pg_status
            elif [ "$service" = "Redis" ]; then
                status=$redis_status
            else
                status=$(check_service "$service" "$url" "$port")
            fi
            
            echo -n "    \"$service\": {"
            echo -n "\"status\": \"$status\", "
            echo -n "\"port\": $port, "
            echo -n "\"description\": \"$desc\""
            
            if [ "$VERBOSE" = true ] && [ "$status" = "UP" ]; then
                metrics=$(get_metrics "${url%/health}/metrics" 2>/dev/null || echo "{}")
                if [ -n "$metrics" ] && [ "$metrics" != "{}" ]; then
                    echo -n ", \"metrics\": $metrics"
                fi
            fi
            
            echo -n "}"
        done
        
        echo ""
        echo "  }"
        echo "}"
    else
        # Terminal output
        clear
        echo "=============================================="
        echo "   AI Workflow Engine - System Health Check"
        echo "   $(date)"
        echo "=============================================="
        echo
        
        printf "%-25s %-12s %-6s %s\n" "SERVICE" "STATUS" "PORT" "DESCRIPTION"
        echo "----------------------------------------------------------------------"
        
        for service in "${!services[@]}"; do
            IFS='|' read -r port url desc <<< "${services[$service]}"
            
            # Special handling for PostgreSQL and Redis
            if [ "$service" = "PostgreSQL" ]; then
                status=$pg_status
            elif [ "$service" = "Redis" ]; then
                status=$redis_status
            else
                status=$(check_service "$service" "$url" "$port")
            fi
            
            # Color code the status
            case $status in
                "UP")
                    status_color="${GREEN}[UP]${NC}"
                    ;;
                "UNHEALTHY")
                    status_color="${YELLOW}[UNHEALTHY]${NC}"
                    ;;
                "DOWN")
                    status_color="${RED}[DOWN]${NC}"
                    ;;
            esac
            
            printf "%-25s %-20s %-6s %s\n" "$service" "$status_color" "$port" "$desc"
        done
        
        echo "----------------------------------------------------------------------"
        
        # Additional checks
        echo
        echo "Additional Information:"
        echo "----------------------"
        
        # Check Docker status
        if docker info >/dev/null 2>&1; then
            echo "Docker Engine: ${GREEN}[RUNNING]${NC}"
        else
            echo "Docker Engine: ${RED}[NOT RUNNING]${NC}"
        fi
        
        # Check disk space
        disk_usage=$(df -h "$PROJECT_ROOT" | awk 'NR==2 {print $5}' | sed 's/%//')
        if [ "$disk_usage" -gt 90 ]; then
            echo "Disk Usage: ${RED}${disk_usage}%${NC} (Critical)"
        elif [ "$disk_usage" -gt 80 ]; then
            echo "Disk Usage: ${YELLOW}${disk_usage}%${NC} (Warning)"
        else
            echo "Disk Usage: ${GREEN}${disk_usage}%${NC}"
        fi
        
        # Check memory usage
        if command -v free >/dev/null 2>&1; then
            mem_usage=$(free | grep Mem | awk '{print int($3/$2 * 100)}')
            if [ "$mem_usage" -gt 90 ]; then
                echo "Memory Usage: ${RED}${mem_usage}%${NC} (Critical)"
            elif [ "$mem_usage" -gt 80 ]; then
                echo "Memory Usage: ${YELLOW}${mem_usage}%${NC} (Warning)"
            else
                echo "Memory Usage: ${GREEN}${mem_usage}%${NC}"
            fi
        fi
        
        # Check for recent errors in logs
        if [ "$VERBOSE" = true ]; then
            echo
            echo "Recent Errors (last 5 minutes):"
            echo "------------------------------"
            
            # Check main service logs
            recent_errors=$(docker-compose logs --tail=100 2>/dev/null | grep -i "error" | tail -5 || true)
            if [ -n "$recent_errors" ]; then
                echo "$recent_errors"
            else
                echo "No recent errors found"
            fi
        fi
        
        echo
        echo "=============================================="
        
        if [ "$WATCH_MODE" = false ]; then
            echo
            echo "Tips:"
            echo "- Run with --watch for continuous monitoring"
            echo "- Run with --verbose for detailed information"
            echo "- Run with --json for machine-readable output"
        fi
    fi
}

# Main execution
if [ "$WATCH_MODE" = true ]; then
    while true; do
        perform_checks
        sleep 5
    done
else
    perform_checks
fi