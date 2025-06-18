#!/bin/bash
# Docker development helper script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_usage() {
    echo "Usage: $0 {up|down|build|logs|ps|exec|clean}"
    echo ""
    echo "Commands:"
    echo "  up      - Start all services in development mode"
    echo "  down    - Stop all services"
    echo "  build   - Build/rebuild all images"
    echo "  logs    - Show logs (optionally specify service)"
    echo "  ps      - Show running services"
    echo "  exec    - Execute command in service"
    echo "  clean   - Clean up volumes and images"
    echo ""
    echo "Examples:"
    echo "  $0 up"
    echo "  $0 logs ai-workflow-system"
    echo "  $0 exec postgres psql -U aiworkflow"
}

# Check if Docker is running
check_docker() {
    if ! docker info >/dev/null 2>&1; then
        echo -e "${RED}Error: Docker is not running${NC}"
        echo "Please start Docker and try again"
        exit 1
    fi
}

# Generate SSL certificates if needed
check_ssl() {
    if [ ! -f "nginx/ssl/cert.pem" ] || [ ! -f "nginx/ssl/key.pem" ]; then
        echo -e "${YELLOW}SSL certificates not found. Generating...${NC}"
        ./nginx/generate-ssl-dev.sh
    fi
}

case "$1" in
    up)
        check_docker
        check_ssl
        echo -e "${BLUE}Starting development environment...${NC}"
        docker-compose -f docker-compose.yml -f docker-compose.dev.yml up -d
        echo -e "${GREEN}Development environment started!${NC}"
        echo ""
        echo "Services available at:"
        echo "  - API:          http://localhost:8080"
        echo "  - Swagger UI:   http://localhost:8081"
        echo "  - Grafana:      http://localhost:3000 (admin/admin)"
        echo "  - Prometheus:   http://localhost:9090"
        echo "  - PgAdmin:      http://localhost:5050 (admin@example.com/admin)"
        echo "  - RedisInsight: http://localhost:8001"
        echo "  - MailCatcher:  http://localhost:1080"
        echo "  - Jaeger:       http://localhost:16686"
        ;;
    
    down)
        echo -e "${BLUE}Stopping development environment...${NC}"
        docker-compose -f docker-compose.yml -f docker-compose.dev.yml down
        echo -e "${GREEN}Development environment stopped${NC}"
        ;;
    
    build)
        check_docker
        echo -e "${BLUE}Building development images...${NC}"
        docker-compose -f docker-compose.yml -f docker-compose.dev.yml build
        echo -e "${GREEN}Build complete${NC}"
        ;;
    
    logs)
        if [ -z "$2" ]; then
            docker-compose -f docker-compose.yml -f docker-compose.dev.yml logs -f
        else
            docker-compose -f docker-compose.yml -f docker-compose.dev.yml logs -f "$2"
        fi
        ;;
    
    ps)
        docker-compose -f docker-compose.yml -f docker-compose.dev.yml ps
        ;;
    
    exec)
        if [ -z "$2" ]; then
            echo -e "${RED}Error: Please specify a service${NC}"
            echo "Example: $0 exec postgres psql -U aiworkflow"
            exit 1
        fi
        shift
        docker-compose -f docker-compose.yml -f docker-compose.dev.yml exec "$@"
        ;;
    
    clean)
        echo -e "${YELLOW}Warning: This will remove all volumes and images${NC}"
        read -p "Are you sure? (y/N) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            docker-compose -f docker-compose.yml -f docker-compose.dev.yml down -v --rmi all
            echo -e "${GREEN}Cleanup complete${NC}"
        else
            echo "Cleanup cancelled"
        fi
        ;;
    
    *)
        print_usage
        exit 1
        ;;
esac