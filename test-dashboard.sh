#!/bin/bash

# AI Workflow System - Visual Test Dashboard
# Provides comprehensive system testing with beautiful visual output

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
GRAY='\033[0;90m'
NC='\033[0m' # No Color

# Unicode characters for visual elements
CHECK="✓"
CROSS="✗"
WARNING="⚠"
INFO="ℹ"
ARROW="→"
CIRCLE_FULL="●"
CIRCLE_EMPTY="○"
BOX_FULL="■"
BOX_EMPTY="□"

# Test statistics
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0
START_TIME=$(date +%s)

# Configuration
REPORT_DIR="test-reports"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
HTML_REPORT="${REPORT_DIR}/test-report-${TIMESTAMP}.html"
JSON_REPORT="${REPORT_DIR}/test-report-${TIMESTAMP}.json"

# Create report directory
mkdir -p "$REPORT_DIR"

# Function to print centered text
print_centered() {
    local text="$1"
    local width=$(tput cols)
    local padding=$(( (width - ${#text}) / 2 ))
    printf "%${padding}s%s\n" "" "$text"
}

# Function to print a line separator
print_separator() {
    printf '%*s\n' "${COLUMNS:-$(tput cols)}" '' | tr ' ' '─'
}

# Function to print section header
print_section() {
    local title="$1"
    echo
    print_separator
    echo -e "${CYAN}$(print_centered "$title")${NC}"
    print_separator
    echo
}

# Function to print progress bar
print_progress() {
    local current=$1
    local total=$2
    local width=50
    local percentage=$((current * 100 / total))
    local filled=$((width * current / total))
    
    printf "\r["
    printf "%${filled}s" '' | tr ' ' '█'
    printf "%$((width - filled))s" '' | tr ' ' '░'
    printf "] %3d%% (%d/%d)" $percentage $current $total
}

# Function to check service with visual feedback
check_service() {
    local name="$1"
    local url="$2"
    local expected="${3:-200}"
    local icon="$4"
    
    printf "  ${icon} %-30s " "$name"
    
    # Show spinner while checking
    local spinner=("⠋" "⠙" "⠹" "⠸" "⠼" "⠴" "⠦" "⠧" "⠇" "⠏")
    local spin_i=0
    
    # Start spinner in background
    (
        while true; do
            printf "\r  ${icon} %-30s ${CYAN}%s${NC} Checking..." "$name" "${spinner[$spin_i]}"
            spin_i=$(( (spin_i + 1) % ${#spinner[@]} ))
            sleep 0.1
        done
    ) &
    local spinner_pid=$!
    
    # Perform the actual check
    local response=$(curl -s -o /dev/null -w "%{http_code}" --connect-timeout 5 "$url" 2>/dev/null)
    
    # Kill spinner
    kill $spinner_pid 2>/dev/null
    wait $spinner_pid 2>/dev/null
    
    # Clear spinner line and show result
    if [ "$response" = "$expected" ]; then
        printf "\r  ${icon} %-30s [${GREEN}${CHECK} ONLINE${NC}]  ${GRAY}HTTP ${response}${NC}\n" "$name"
        ((PASSED_TESTS++))
        return 0
    else
        printf "\r  ${icon} %-30s [${RED}${CROSS} OFFLINE${NC}] ${GRAY}HTTP ${response}${NC}\n" "$name"
        ((FAILED_TESTS++))
        return 1
    fi
    ((TOTAL_TESTS++))
}

# Function to display system status dashboard
show_system_dashboard() {
    clear
    
    # Header
    echo -e "${PURPLE}"
    cat << "EOF"
    ╔═══════════════════════════════════════════════════════════════╗
    ║      AI Workflow Orchestration - System Test Dashboard        ║
    ╚═══════════════════════════════════════════════════════════════╝
EOF
    echo -e "${NC}"
    
    # System info
    echo -e "${WHITE}System Information:${NC}"
    echo -e "  ${GRAY}Date:${NC} $(date)"
    echo -e "  ${GRAY}Host:${NC} $(hostname)"
    echo -e "  ${GRAY}User:${NC} $(whoami)"
    echo
}

# Function to check infrastructure
check_infrastructure() {
    print_section "🏗️  INFRASTRUCTURE STATUS"
    
    echo -e "${WHITE}Core Services:${NC}"
    check_service "PostgreSQL Database" "localhost:5432" "000" "🗄️ "
    check_service "Redis Cache" "localhost:6379" "000" "💾"
    
    echo
    echo -e "${WHITE}Monitoring:${NC}"
    check_service "Prometheus" "http://localhost:9090/-/ready" "200" "📊"
    check_service "Grafana" "http://localhost:3000/api/health" "200" "📈"
    check_service "Jaeger Tracing" "http://localhost:16686/" "200" "🔍"
}

# Function to check backend services
check_backend_services() {
    print_section "🚀 BACKEND SERVICES"
    
    echo -e "${WHITE}Core API:${NC}"
    check_service "Main API Server" "http://localhost:8080/health" "200" "🌐"
    check_service "Swagger Documentation" "http://localhost:8080/swagger-ui/" "200" "📚"
    
    echo
    echo -e "${WHITE}GraphQL:${NC}"
    check_service "GraphQL Gateway" "http://localhost:4000/.well-known/apollo/server-health" "200" "🔷"
    check_service "GraphQL Playground" "http://localhost:4000/graphql" "200" "🎮"
    
    echo
    echo -e "${WHITE}MCP Servers:${NC}"
    check_service "HelpScout MCP" "http://localhost:8001/health" "200" "📧"
    check_service "Notion MCP" "http://localhost:8002/health" "200" "📝"
    check_service "Slack MCP" "http://localhost:8003/health" "200" "💬"
    
    echo
    echo -e "${WHITE}Microservices:${NC}"
    check_service "Content Processing" "http://localhost:8082/health" "200" "📄"
    check_service "Knowledge Graph" "http://localhost:8083/health" "200" "🧠"
    check_service "Realtime Communication" "http://localhost:8084/health" "200" "⚡"
}

# Function to run backend tests
run_backend_tests() {
    print_section "🧪 BACKEND TESTS"
    
    if [ -f "Cargo.toml" ]; then
        echo -e "${WHITE}Running Rust tests...${NC}"
        
        # Count total tests
        local total_tests=$(cargo test --workspace -- --list 2>/dev/null | grep -c "test " || echo "0")
        echo -e "${GRAY}Found $total_tests tests${NC}"
        echo
        
        # Run tests with progress
        local current=0
        cargo test --workspace --quiet 2>&1 | while IFS= read -r line; do
            if [[ $line =~ "test "[^[:space:]]+" ... ok" ]]; then
                ((current++))
                print_progress $current $total_tests
            elif [[ $line =~ "test "[^[:space:]]+" ... FAILED" ]]; then
                ((current++))
                print_progress $current $total_tests
                echo
                echo -e "${RED}Failed: ${line}${NC}"
            fi
        done
        
        echo
        echo
    else
        echo -e "${YELLOW}${WARNING} Not in Rust project directory${NC}"
    fi
}

# Function to check frontend
check_frontend() {
    print_section "🎨 FRONTEND STATUS"
    
    echo -e "${WHITE}Development Server:${NC}"
    check_service "Vite Dev Server" "http://localhost:5173" "200" "⚡"
    
    if [ -d "frontend" ]; then
        cd frontend
        
        echo
        echo -e "${WHITE}Running frontend tests...${NC}"
        
        # Check if node_modules exists
        if [ -d "node_modules" ]; then
            # Run Jest tests
            npm test -- --json --outputFile=../test-reports/jest-results.json 2>/dev/null
            
            # Parse results
            if [ -f "../test-reports/jest-results.json" ]; then
                local jest_passed=$(jq '.numPassedTests' ../test-reports/jest-results.json)
                local jest_failed=$(jq '.numFailedTests' ../test-reports/jest-results.json)
                local jest_total=$(jq '.numTotalTests' ../test-reports/jest-results.json)
                
                echo -e "  Test Results: ${GREEN}$jest_passed passed${NC}, ${RED}$jest_failed failed${NC}, $jest_total total"
                
                ((PASSED_TESTS += jest_passed))
                ((FAILED_TESTS += jest_failed))
                ((TOTAL_TESTS += jest_total))
            fi
        else
            echo -e "${YELLOW}${WARNING} Frontend dependencies not installed${NC}"
        fi
        
        cd ..
    fi
}

# Function to show summary
show_summary() {
    print_section "📊 TEST SUMMARY"
    
    local end_time=$(date +%s)
    local duration=$((end_time - START_TIME))
    
    # Calculate percentages
    local pass_rate=0
    if [ $TOTAL_TESTS -gt 0 ]; then
        pass_rate=$((PASSED_TESTS * 100 / TOTAL_TESTS))
    fi
    
    # Visual summary
    echo -e "${WHITE}Results Overview:${NC}"
    echo
    
    # Progress bar for pass rate
    local width=40
    local filled=$((width * pass_rate / 100))
    printf "  Pass Rate: ["
    printf "%${filled}s" '' | tr ' ' '█' | sed "s/^/${GREEN}/" | sed "s/$/${NC}/"
    printf "%$((width - filled))s" '' | tr ' ' '░' | sed "s/^/${RED}/" | sed "s/$/${NC}/"
    printf "] ${WHITE}%3d%%${NC}\n" $pass_rate
    
    echo
    echo -e "  ${GREEN}${CHECK} Passed:${NC}  $PASSED_TESTS"
    echo -e "  ${RED}${CROSS} Failed:${NC}  $FAILED_TESTS"
    echo -e "  ${YELLOW}${WARNING} Skipped:${NC} $SKIPPED_TESTS"
    echo -e "  ${GRAY}─────────────${NC}"
    echo -e "  ${WHITE}Total:${NC}     $TOTAL_TESTS"
    echo
    echo -e "  ${GRAY}Duration:${NC}  ${duration}s"
    
    # Status message
    echo
    if [ $FAILED_TESTS -eq 0 ] && [ $TOTAL_TESTS -gt 0 ]; then
        echo -e "${GREEN}${CHECK} All systems operational!${NC} 🎉"
    elif [ $FAILED_TESTS -le 3 ]; then
        echo -e "${YELLOW}${WARNING} Minor issues detected${NC}"
    else
        echo -e "${RED}${CROSS} Multiple failures detected${NC}"
    fi
}

# Function to generate HTML report
generate_html_report() {
    cat > "$HTML_REPORT" << EOF
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>AI Workflow System - Test Report</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #0a0a0a;
            color: #fff;
            line-height: 1.6;
        }
        .container {
            max-width: 1200px;
            margin: 0 auto;
            padding: 2rem;
        }
        .header {
            text-align: center;
            padding: 2rem 0;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            border-radius: 1rem;
            margin-bottom: 2rem;
        }
        .header h1 { font-size: 2.5rem; margin-bottom: 0.5rem; }
        .header p { opacity: 0.9; }
        .stats-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 1.5rem;
            margin-bottom: 2rem;
        }
        .stat-card {
            background: #1a1a1a;
            padding: 1.5rem;
            border-radius: 0.5rem;
            border: 1px solid #333;
        }
        .stat-card h3 { color: #888; font-size: 0.875rem; text-transform: uppercase; }
        .stat-card .value { font-size: 2rem; font-weight: bold; margin: 0.5rem 0; }
        .stat-card.passed .value { color: #4ade80; }
        .stat-card.failed .value { color: #f87171; }
        .stat-card.skipped .value { color: #fbbf24; }
        .progress-bar {
            width: 100%;
            height: 20px;
            background: #333;
            border-radius: 10px;
            overflow: hidden;
            margin: 1rem 0;
        }
        .progress-fill {
            height: 100%;
            background: linear-gradient(90deg, #4ade80 0%, #22c55e 100%);
            transition: width 0.3s ease;
        }
        .service-grid {
            display: grid;
            gap: 1rem;
            margin-bottom: 2rem;
        }
        .service-item {
            background: #1a1a1a;
            padding: 1rem;
            border-radius: 0.5rem;
            display: flex;
            justify-content: space-between;
            align-items: center;
            border: 1px solid #333;
        }
        .service-item.online { border-left: 3px solid #4ade80; }
        .service-item.offline { border-left: 3px solid #f87171; }
        .timestamp { color: #666; font-size: 0.875rem; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>AI Workflow System - Test Report</h1>
            <p>Generated on $(date)</p>
        </div>
        
        <div class="stats-grid">
            <div class="stat-card passed">
                <h3>Passed Tests</h3>
                <div class="value">$PASSED_TESTS</div>
            </div>
            <div class="stat-card failed">
                <h3>Failed Tests</h3>
                <div class="value">$FAILED_TESTS</div>
            </div>
            <div class="stat-card skipped">
                <h3>Skipped Tests</h3>
                <div class="value">$SKIPPED_TESTS</div>
            </div>
            <div class="stat-card">
                <h3>Total Tests</h3>
                <div class="value">$TOTAL_TESTS</div>
            </div>
        </div>
        
        <div class="progress-bar">
            <div class="progress-fill" style="width: ${pass_rate}%"></div>
        </div>
        
        <p class="timestamp">Report generated in ${duration} seconds</p>
    </div>
</body>
</html>
EOF
}

# Main execution
main() {
    # Parse command line arguments
    case "${1:-}" in
        --quick)
            show_system_dashboard
            check_infrastructure
            check_backend_services
            ;;
        --frontend)
            show_system_dashboard
            check_frontend
            ;;
        --backend)
            show_system_dashboard
            check_backend_services
            run_backend_tests
            ;;
        *)
            show_system_dashboard
            check_infrastructure
            check_backend_services
            check_frontend
            run_backend_tests
            ;;
    esac
    
    show_summary
    generate_html_report
    
    echo
    print_separator
    echo -e "${BLUE}${INFO} Reports generated:${NC}"
    echo -e "  ${GRAY}HTML:${NC} $HTML_REPORT"
    echo -e "  ${GRAY}JSON:${NC} $JSON_REPORT"
    echo
    
    # Exit with appropriate code
    [ $FAILED_TESTS -eq 0 ] && exit 0 || exit 1
}

# Run main function
main "$@"