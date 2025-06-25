#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PID_DIR="$SCRIPT_DIR/pids"

# Function to print colored output
print_status() {
    echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[$(date '+%H:%M:%S')] ✓${NC} $1"
}

print_error() {
    echo -e "${RED}[$(date '+%H:%M:%S')] ✗${NC} $1"
}

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}              Stopping AI Workflow Engine Services              ${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# Check if PID directory exists
if [ ! -d "$PID_DIR" ]; then
    print_error "No PID directory found. Services may not be running."
    exit 1
fi

# Stop all services
services_stopped=0
for pid_file in "$PID_DIR"/*.pid; do
    if [ -f "$pid_file" ]; then
        pid=$(cat "$pid_file")
        service_name=$(basename "$pid_file" .pid)
        
        if kill -0 $pid 2>/dev/null; then
            print_status "Stopping $service_name (PID: $pid)..."
            kill $pid
            
            # Wait for process to stop
            timeout=10
            while [ $timeout -gt 0 ] && kill -0 $pid 2>/dev/null; do
                sleep 1
                timeout=$((timeout - 1))
            done
            
            if kill -0 $pid 2>/dev/null; then
                print_error "Failed to stop $service_name gracefully, forcing..."
                kill -9 $pid
            fi
            
            rm "$pid_file"
            print_success "$service_name stopped"
            services_stopped=$((services_stopped + 1))
        else
            print_status "$service_name (PID: $pid) is not running"
            rm "$pid_file"
        fi
    fi
done

if [ $services_stopped -eq 0 ]; then
    print_status "No running services found"
else
    print_success "Stopped $services_stopped service(s)"
fi

# Optional: Stop Redis if it was started by us
if [ -f "$PID_DIR/redis.pid" ]; then
    print_status "Note: Redis was started by the startup script and has been stopped."
    print_status "To stop system Redis: brew services stop redis (macOS) or sudo systemctl stop redis (Linux)"
fi

echo
print_success "All services stopped successfully!"