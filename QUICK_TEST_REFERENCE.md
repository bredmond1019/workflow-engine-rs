# Quick Test Reference Card

## 🚀 Essential Test Commands

### Frontend Tests
```bash
cd frontend
npm test                    # Run all 174+ tests
npm run dev                 # Start dev server (http://localhost:3000)
```

### GraphQL Federation
```bash
./scripts/run-federation-stack.sh    # Start all services
./scripts/test-federation.sh         # Test connectivity
# Gateway: http://localhost:4000/graphql
```

### Backend Services
```bash
cargo run --bin workflow-engine      # Main API (port 8080)
cargo run --bin graphql-gateway      # Gateway (port 4000)
# Swagger: http://localhost:8080/swagger-ui/
```

### Quick Health Checks
```bash
curl http://localhost:8080/health    # Main API
curl http://localhost:4000/health    # Gateway
curl http://localhost:8082/health    # Content Processing
curl http://localhost:3002/health    # Knowledge Graph
curl http://localhost:8081/health    # Realtime Comm
```

### Integration Tests
```bash
./scripts/start_test_servers.sh      # Start MCP servers
cargo test -- --ignored              # Run all integration tests
```

## 🎯 Test Workflows

### 1. Quick Smoke Test (2 min)
```bash
# Terminal 1
./scripts/run-federation-stack.sh

# Terminal 2
cd frontend && npm run dev

# Browser
Open http://localhost:3000
Type: "Create a customer support workflow"
```

### 2. Full Test Suite (10 min)
```bash
# Run all tests
cd frontend && npm test
cargo test
cargo test -- --ignored
./scripts/test-federation.sh
```

### 3. Visual Testing
```bash
# Open test dashboard
cd frontend/test-dashboard
python -m http.server 8088
open http://localhost:8088
```

## 🔍 Debug Commands

```bash
# Check ports
lsof -i :8080,8081,8082,3002,4000

# View logs
docker-compose logs -f

# Database check
psql ai_workflow_db -c "SELECT * FROM workflows LIMIT 5;"

# GraphQL introspection
curl -X POST http://localhost:4000/graphql \
  -H "Content-Type: application/json" \
  -d '{"query":"{ __schema { types { name } } }"}'
```

## ✅ Success Indicators

- Frontend: 174+ tests passing
- Gateway: All services connected
- Chat: Creates workflows successfully
- GraphQL: Federation queries work
- Health: All endpoints return 200 OK

## 🚨 Common Fixes

```bash
# Port conflict
lsof -i :8080 | grep LISTEN
kill -9 <PID>

# Clean restart
docker-compose down
rm -rf node_modules frontend/node_modules
npm install && cd frontend && npm install

# Database reset
dropdb ai_workflow_db
createdb ai_workflow_db
diesel migration run
```