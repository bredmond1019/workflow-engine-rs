#!/bin/bash
# Quick Docker status check script

echo "=== Docker Container Status ==="
docker ps -a --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"

echo -e "\n=== Service Health ==="
# Check PostgreSQL
if docker exec ai-workflow-db pg_isready -U postgres >/dev/null 2>&1; then
    echo "✓ PostgreSQL: Healthy"
else
    echo "✗ PostgreSQL: Not ready"
fi

# Check Redis
if docker exec ai-workflow-redis redis-cli ping >/dev/null 2>&1; then
    echo "✓ Redis: Healthy"
else
    echo "✗ Redis: Not ready"
fi

# Check if API is responding
if curl -s http://localhost:8080/health >/dev/null 2>&1; then
    echo "✓ API: Responding"
else
    echo "✗ API: Not responding"
fi

# Check if Frontend is responding
if curl -s http://localhost:5173 >/dev/null 2>&1; then
    echo "✓ Frontend: Responding"
else
    echo "✗ Frontend: Not responding"
fi

echo -e "\n=== Available Endpoints ==="
echo "Frontend: http://localhost:5173"
echo "API: http://localhost:8080"
echo "API Docs: http://localhost:8080/swagger-ui/"