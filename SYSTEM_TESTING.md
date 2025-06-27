# System Testing Guide - AI Workflow Orchestration Platform

This comprehensive guide covers testing the entire application stack including backend services, frontend components, and their integration. Follow each phase systematically to ensure all components are functioning correctly.

## Overview

The AI Workflow Orchestration system consists of:
- **Backend**: Rust-based services with GraphQL Federation
- **Frontend**: React-based chat UI for workflow creation
- **Microservices**: Content Processing, Knowledge Graph, Real-time Communication
- **External Services**: MCP servers (HelpScout, Notion, Slack)
- **Infrastructure**: PostgreSQL, monitoring tools, message queues

## Phase 1: Infrastructure Verification

### 1.1 Prerequisites Check

Before testing, ensure all dependencies are installed:

```bash
# Check Rust installation
rustc --version  # Should be 1.75.0+
cargo --version

# Check Node.js installation
node --version   # Should be 18.x
npm --version

# Check Docker installation
docker --version
docker-compose --version

# Check PostgreSQL tools
psql --version
```

### 1.2 Database Setup and Verification

```bash
# Option 1: Local PostgreSQL
createdb ai_workflow_db
psql ai_workflow_db < scripts/init-db.sql

# Option 2: Docker PostgreSQL
docker-compose up -d postgres

# Verify database connection
psql -h localhost -U aiworkflow -d ai_workflow_db -c "SELECT version();"
```

**Verification Checklist:**
- [ ] Database is running on port 5432
- [ ] Can connect with credentials (user: aiworkflow, password: aiworkflow123)
- [ ] Schema is initialized (check for tables: workflows, workflow_instances, users)

### 1.3 Start Backend Services

```bash
# Start all services with Docker Compose
docker-compose up -d

# Or start individually:

# 1. Start MCP test servers
./scripts/start_test_servers.sh

# 2. Start main API server
cargo run --bin workflow-engine

# 3. Start GraphQL Gateway
cargo run --bin graphql-gateway

# 4. Start microservices (optional)
cd services/content_processing && cargo run &
cd services/knowledge_graph && cargo run &
cd services/realtime_communication && cargo run &
```

### 1.4 Service Health Checks

Verify each service is running:

```bash
# Main API health check
curl http://localhost:8080/health
# Expected: {"status":"healthy","database":"connected"}

# GraphQL Gateway
curl http://localhost:4000/.well-known/apollo/server-health
# Expected: {"status":"pass"}

# MCP Servers
curl http://localhost:8001/health  # HelpScout
curl http://localhost:8002/health  # Notion
curl http://localhost:8003/health  # Slack
# Expected: {"status":"ok"} for each

# Microservices (if running)
curl http://localhost:8082/health  # Content Processing
curl http://localhost:8083/health  # Knowledge Graph
curl http://localhost:8084/health  # Real-time Communication
```

**Service Status Checklist:**
- [ ] Main API: http://localhost:8080 ✓
- [ ] GraphQL Gateway: http://localhost:4000 ✓
- [ ] Swagger UI: http://localhost:8080/swagger-ui/ ✓
- [ ] GraphQL Playground: http://localhost:4000/graphql ✓
- [ ] MCP Servers: 8001, 8002, 8003 ✓
- [ ] PostgreSQL: localhost:5432 ✓

## Phase 2: Backend API Testing

### 2.1 Authentication Testing

#### Test JWT Authentication
```bash
# 1. Get JWT token
curl -X POST http://localhost:8080/auth/token \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123"}'

# Expected response:
# {"token":"eyJ0eXAiOi...","expires_in":3600}

# 2. Use token for authenticated request
TOKEN="<token-from-above>"
curl -X GET http://localhost:8080/api/v1/workflows \
  -H "Authorization: Bearer $TOKEN"
```

**Authentication Checklist:**
- [ ] Can obtain JWT token
- [ ] Token works for authenticated endpoints
- [ ] Unauthorized requests return 401
- [ ] Token expiration is enforced

### 2.2 REST API Testing

#### Workflow CRUD Operations
```bash
# 1. Create workflow
curl -X POST http://localhost:8080/api/v1/workflows \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Test Workflow",
    "description": "Customer support automation",
    "nodes": []
  }'

# 2. List workflows
curl -X GET http://localhost:8080/api/v1/workflows \
  -H "Authorization: Bearer $TOKEN"

# 3. Get specific workflow
curl -X GET http://localhost:8080/api/v1/workflows/{id} \
  -H "Authorization: Bearer $TOKEN"

# 4. Update workflow
curl -X PUT http://localhost:8080/api/v1/workflows/{id} \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name": "Updated Workflow"}'

# 5. Delete workflow
curl -X DELETE http://localhost:8080/api/v1/workflows/{id} \
  -H "Authorization: Bearer $TOKEN"
```

**REST API Checklist:**
- [ ] Create workflow returns 201
- [ ] List workflows returns array
- [ ] Get workflow returns details
- [ ] Update workflow returns 200
- [ ] Delete workflow returns 204

### 2.3 GraphQL API Testing

Access GraphQL Playground at http://localhost:4000/graphql

#### Test Queries
```graphql
# 1. Query workflows
query GetWorkflows {
  workflows {
    id
    name
    description
    createdAt
    status
  }
}

# 2. Query single workflow
query GetWorkflow($id: ID!) {
  workflow(id: $id) {
    id
    name
    nodes {
      id
      type
      config
    }
  }
}

# 3. Federation test
query FederationTest {
  _service {
    sdl
  }
}
```

#### Test Mutations
```graphql
# 1. Create workflow
mutation CreateWorkflow($input: CreateWorkflowInput!) {
  createWorkflow(input: $input) {
    id
    name
    status
  }
}

# 2. Execute workflow
mutation ExecuteWorkflow($id: ID!) {
  executeWorkflow(id: $id) {
    instanceId
    status
    startedAt
  }
}
```

**GraphQL Checklist:**
- [ ] Queries return expected data
- [ ] Mutations modify data correctly
- [ ] Federation schema is valid
- [ ] Subscriptions work (if implemented)

### 2.4 WebSocket Testing

Test real-time communication:

```javascript
// WebSocket test (run in browser console)
const ws = new WebSocket('ws://localhost:8084/ws');

ws.onopen = () => {
  console.log('Connected');
  ws.send(JSON.stringify({
    type: 'subscribe',
    channel: 'workflow-updates'
  }));
};

ws.onmessage = (event) => {
  console.log('Received:', event.data);
};
```

**WebSocket Checklist:**
- [ ] Can establish connection
- [ ] Can subscribe to channels
- [ ] Receives real-time updates
- [ ] Handles reconnection

### 2.5 MCP Protocol Testing

```bash
# Test MCP server communication
curl -X POST http://localhost:8001/mcp/execute \
  -H "Content-Type: application/json" \
  -d '{
    "method": "tools/list",
    "params": {}
  }'

# Expected: List of available tools
```

**MCP Checklist:**
- [ ] HelpScout MCP responds
- [ ] Notion MCP responds
- [ ] Slack MCP responds
- [ ] Tool execution works

## Phase 3: Frontend Testing

### 3.1 Start Frontend Development Server

```bash
cd frontend
npm install
npm run dev
```

Access at http://localhost:5173

### 3.2 Component Unit Tests

```bash
# Run all frontend tests
npm test

# Run specific test suites
npm test -- ChatMessage
npm test -- ChatInput
npm test -- ChatContainer
npm test -- WorkflowIntentAnalyzer
npm test -- DynamicForm
npm test -- WorkflowPreview
```

**Unit Test Results:**
- [ ] ChatMessage: ___/7 tests passing
- [ ] ChatInput: ___/13 tests passing
- [ ] ChatContainer: ___/10 tests passing
- [ ] WorkflowIntentAnalyzer: ___/31 tests passing
- [ ] DynamicForm: ___/27 tests passing
- [ ] WorkflowPreview: ___/32 tests passing

### 3.3 Integration Testing

Create the test page as described in USER_TESTING.md and verify:

**Component Integration:**
- [ ] Chat interface renders correctly
- [ ] Messages display with proper styling
- [ ] Intent analysis works on message send
- [ ] Dynamic form generates based on intent
- [ ] Workflow preview updates in real-time
- [ ] All components communicate properly

### 3.4 API Integration Testing

Configure frontend to connect to backend:

```javascript
// In frontend .env
VITE_API_URL=http://localhost:8080
VITE_GRAPHQL_URL=http://localhost:4000/graphql
```

**API Integration Checklist:**
- [ ] Can authenticate with backend
- [ ] Can fetch workflows via GraphQL
- [ ] Can create workflows through UI
- [ ] Real-time updates work
- [ ] Error handling displays properly

## Phase 4: End-to-End Workflow Testing

### 4.1 Complete Workflow Creation Flow

Follow this scenario:

1. **Start conversation**:
   - Type: "I want to create a customer support workflow"
   - Verify: Intent detected, form appears

2. **Configure workflow**:
   - Fill in: Workflow name, HelpScout API key
   - Select: Ticket priority filter
   - Choose: Slack as output channel

3. **Submit and execute**:
   - Submit form
   - Verify workflow created in backend
   - Execute workflow
   - Check execution logs

**E2E Checklist:**
- [ ] Conversation flow works naturally
- [ ] Form validation prevents errors
- [ ] Workflow saves to database
- [ ] Execution triggers properly
- [ ] Results appear in target service

### 4.2 Error Scenarios

Test error handling:

1. **Invalid credentials**: Wrong API keys
2. **Network failures**: Disconnect services
3. **Invalid input**: Malformed data
4. **Service unavailable**: Stop MCP server

**Error Handling Checklist:**
- [ ] Clear error messages shown
- [ ] System remains stable
- [ ] Can recover from errors
- [ ] No data loss occurs

## Phase 5: Performance Testing

### 5.1 Load Testing

```bash
# Run load tests
cargo test --test load_test -- --ignored --nocapture

# Or use Apache Bench
ab -n 1000 -c 10 -H "Authorization: Bearer $TOKEN" \
   http://localhost:8080/api/v1/workflows
```

**Performance Metrics:**
- [ ] Response time < 100ms (avg)
- [ ] Can handle 100 concurrent users
- [ ] Memory usage stable
- [ ] No memory leaks detected

### 5.2 Stress Testing

1. Create 100 workflows rapidly
2. Execute 50 workflows simultaneously
3. Send 1000 chat messages quickly
4. Open 10 WebSocket connections

**Stress Test Results:**
- [ ] System remains responsive
- [ ] No crashes or hangs
- [ ] Error rate < 1%
- [ ] Recovery time < 5s

## Test Report Template

```markdown
# Test Execution Report

Date: _____________
Tester: ___________
Environment: ______

## Summary
- Total Tests Run: ___
- Passed: ___
- Failed: ___
- Skipped: ___

## Infrastructure Status
✅ Database: Operational
✅ Backend API: Operational
⚠️ GraphQL Gateway: Issues with...
❌ MCP Servers: Not responding

## Test Results by Phase

### Phase 1: Infrastructure
- Database setup: PASS
- Service health: PASS
- Dependencies: PASS

### Phase 2: Backend
- Authentication: PASS
- REST API: PASS
- GraphQL: FAIL - mutation error
- WebSocket: PASS
- MCP: PARTIAL - Slack server timeout

### Phase 3: Frontend
- Unit tests: 96/100 PASS
- Integration: PASS
- API connection: PASS

### Phase 4: E2E
- Workflow creation: PASS
- Error handling: PASS

### Phase 5: Performance
- Load test: PASS
- Stress test: FAIL - memory spike at 80 users

## Issues Found
1. GraphQL mutation returns null for...
2. Slack MCP timeout after 30s
3. Memory usage increases with...

## Recommendations
1. Fix GraphQL resolver for...
2. Increase Slack timeout to 60s
3. Implement connection pooling for...
```

## Automated Testing Script

Save as `test-system.sh`:

```bash
#!/bin/bash

echo "🧪 AI Workflow System Test Suite"
echo "================================"

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Test results
PASSED=0
FAILED=0

# Function to check service
check_service() {
    local name=$1
    local url=$2
    local expected=$3
    
    echo -n "Checking $name... "
    response=$(curl -s -o /dev/null -w "%{http_code}" $url)
    
    if [ "$response" = "$expected" ]; then
        echo -e "${GREEN}✓ PASS${NC}"
        ((PASSED++))
    else
        echo -e "${RED}✗ FAIL (HTTP $response)${NC}"
        ((FAILED++))
    fi
}

# Phase 1: Infrastructure
echo -e "\n${YELLOW}Phase 1: Infrastructure Checks${NC}"
check_service "PostgreSQL" "http://localhost:5432" "000"
check_service "Main API" "http://localhost:8080/health" "200"
check_service "GraphQL Gateway" "http://localhost:4000/.well-known/apollo/server-health" "200"
check_service "HelpScout MCP" "http://localhost:8001/health" "200"
check_service "Notion MCP" "http://localhost:8002/health" "200"
check_service "Slack MCP" "http://localhost:8003/health" "200"

# Phase 2: Backend Tests
echo -e "\n${YELLOW}Phase 2: Backend Tests${NC}"
echo "Running Rust tests..."
if cargo test --workspace; then
    echo -e "${GREEN}✓ Backend tests passed${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ Backend tests failed${NC}"
    ((FAILED++))
fi

# Phase 3: Frontend Tests
echo -e "\n${YELLOW}Phase 3: Frontend Tests${NC}"
cd frontend
echo "Running frontend tests..."
if npm test -- --watchAll=false; then
    echo -e "${GREEN}✓ Frontend tests passed${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ Frontend tests failed${NC}"
    ((FAILED++))
fi
cd ..

# Summary
echo -e "\n${YELLOW}Test Summary${NC}"
echo "============"
echo -e "Passed: ${GREEN}$PASSED${NC}"
echo -e "Failed: ${RED}$FAILED${NC}"

if [ $FAILED -eq 0 ]; then
    echo -e "\n${GREEN}All tests passed! ✨${NC}"
    exit 0
else
    echo -e "\n${RED}Some tests failed. Please check the logs above.${NC}"
    exit 1
fi
```

Make it executable: `chmod +x test-system.sh`

## Next Steps

1. Run through each phase systematically
2. Document any failures in the test report
3. Create issues for any bugs found
4. Re-test after fixes are applied
5. Automate more tests as needed

This comprehensive testing approach ensures both frontend and backend components work correctly individually and together as a complete system.