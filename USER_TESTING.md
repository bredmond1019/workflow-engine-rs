# User Testing Guide

This document provides step-by-step instructions to validate all components of the AI Workflow Engine system. Follow each section to verify functionality and report any issues.

## Prerequisites

Before starting, ensure you have:
- Node.js 18+ and npm installed
- Rust 1.75+ and cargo installed
- Docker and Docker Compose installed
- PostgreSQL client (psql) installed
- At least 8GB of free RAM
- Ports 3000, 4000, 8080-8084 available

## 1. Frontend Testing

### 1.1 Setup and Basic Functionality

```bash
# Navigate to frontend directory
cd frontend

# Install dependencies
npm install

# Run the test suite (should show 174+ tests passing)
npm test

# Start the development server
npm run dev
```

**Expected Results:**
- ✅ All tests pass (174+ tests)
- ✅ Development server starts on http://localhost:3000
- ✅ No console errors on page load

### 1.2 Chat Interface Testing

1. Open http://localhost:3000 in your browser
2. You should see the TDD Demo page with chat interface

**Test the Chat Components:**
- Type "Create a customer support workflow" in the chat input
- Press Enter or click Send
- Verify assistant responds with workflow creation message
- Check that the workflow builder appears on the right

**Expected Results:**
- ✅ Messages appear in chat with proper styling (user/assistant)
- ✅ Timestamp shows on messages
- ✅ Chat scrolls automatically for new messages
- ✅ Loading indicator appears while processing

### 1.3 Workflow Builder Testing

After triggering workflow creation:

**Test Multi-Step Progress:**
- Verify progress tracker shows 3 steps
- Click on different steps to navigate
- Fill in form fields for each step
- Use "Previous" and "Next" buttons

**Expected Results:**
- ✅ Progress bar updates as you complete steps
- ✅ Form validation works (try submitting empty required fields)
- ✅ Data persists when navigating between steps
- ✅ Completed steps show checkmark

### 1.4 Visual Components Testing

**Test Workflow Preview:**
- Look for the workflow visualization at the bottom
- Click on workflow nodes
- Verify connections between nodes are visible

**Expected Results:**
- ✅ Workflow nodes display with icons and labels
- ✅ Node connections show with animated lines
- ✅ Clicking nodes logs to console
- ✅ Different node types have different colors

## 2. GraphQL Federation Testing

### 2.1 Start the Federation Stack

```bash
# From project root
./scripts/run-federation-stack.sh
```

**Expected Results:**
- ✅ All services start without port conflicts
- ✅ Health checks pass for all services
- ✅ No error messages in startup logs

### 2.2 Test Federation Connectivity

```bash
# In a new terminal
./scripts/test-federation.sh
```

**Expected Results:**
- ✅ Gateway health check: SUCCESS
- ✅ Simple query: SUCCESS (returns workflow list)
- ✅ Federated query: SUCCESS (returns workflow data)
- ✅ Service discovery: SUCCESS
- ✅ Entity resolution: SUCCESS

### 2.3 GraphQL Playground Testing

1. Open http://localhost:4000/graphql in your browser
2. You should see GraphQL Playground interface

**Test Queries:**

```graphql
# Simple health check
{
  _service {
    sdl
  }
}
```

```graphql
# Federated query across services
{
  workflows {
    id
    name
    status
    nodes {
      id
      type
    }
  }
}
```

**Expected Results:**
- ✅ Playground loads without errors
- ✅ Queries execute successfully
- ✅ Schema documentation is visible
- ✅ Auto-completion works for fields

## 3. Backend Testing (Core Services)

### 3.1 Database Setup

```bash
# Create database if not exists
createdb ai_workflow_db

# Run migrations
cd crates/workflow-engine-api
diesel migration run
```

**Expected Results:**
- ✅ Database created successfully
- ✅ All migrations applied
- ✅ No SQL errors

### 3.2 Main API Server

```bash
# From project root
cargo run --bin workflow-engine
```

**Test Endpoints:**
1. Health check: `curl http://localhost:8080/health`
2. Detailed health: `curl http://localhost:8080/health/detailed`
3. API docs: Open http://localhost:8080/swagger-ui/

**Expected Results:**
- ✅ Health endpoint returns `{"status":"healthy"}`
- ✅ Detailed health shows all components
- ✅ Swagger UI loads with API documentation
- ✅ No panic messages in server logs

### 3.3 Authentication Testing

```bash
# Get auth token
curl -X POST http://localhost:8080/auth/token \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123"}'
```

**Expected Results:**
- ✅ Returns JWT token
- ✅ Token has proper structure (header.payload.signature)
- ✅ Token can be decoded (use jwt.io to verify)

## 4. Individual Microservices Testing

### 4.1 Content Processing Service

```bash
# Start the service
cd services/content_processing
cargo run
```

**Test the Service:**
```bash
# Health check
curl http://localhost:8082/health

# GraphQL endpoint
curl -X POST http://localhost:8082/graphql \
  -H "Content-Type: application/json" \
  -d '{"query":"{ _service { sdl } }"}'
```

**Expected Results:**
- ✅ Service starts on port 8082
- ✅ Health check returns success
- ✅ GraphQL endpoint responds with SDL

### 4.2 Knowledge Graph Service

```bash
# Start Dgraph first (if using)
cd services/knowledge_graph/dgraph
docker-compose up -d

# Start the service
cd services/knowledge_graph
cargo run
```

**Test the Service:**
```bash
# Health check
curl http://localhost:3002/health

# GraphQL query
curl -X POST http://localhost:3002/graphql \
  -H "Content-Type: application/json" \
  -d '{"query":"{ concepts { id name } }"}'
```

**Expected Results:**
- ✅ Service starts on port 3002
- ✅ Connects to Dgraph successfully
- ✅ GraphQL queries work

### 4.3 Realtime Communication Service

```bash
# Start the service
cd services/realtime_communication
cargo run
```

**Test WebSocket Connection:**
```bash
# Install wscat if needed
npm install -g wscat

# Connect to WebSocket
wscat -c ws://localhost:8081/ws
```

**Expected Results:**
- ✅ Service starts on port 8081
- ✅ WebSocket connection established
- ✅ Can send/receive messages
- ✅ Presence updates work

## 5. End-to-End Integration Testing

### 5.1 Full Stack Test

1. Ensure all services are running (use `./scripts/run-federation-stack.sh`)
2. Start the frontend (`cd frontend && npm run dev`)
3. Open http://localhost:3000

**Complete Workflow Test:**
1. Create a new workflow via chat
2. Fill in all workflow details
3. Submit the workflow
4. Check that workflow appears in the list
5. Click on the workflow to see details

**Expected Results:**
- ✅ Workflow creation succeeds
- ✅ Data flows through all services
- ✅ Real-time updates appear
- ✅ No errors in any service logs

### 5.2 MCP Integration Test

```bash
# Start MCP test servers
./scripts/start_test_servers.sh

# Run MCP integration tests
cargo test external_mcp_integration -- --ignored
```

**Expected Results:**
- ✅ HelpScout server starts (port 8001)
- ✅ Notion server starts (port 8002)
- ✅ Slack server starts (port 8003)
- ✅ All MCP integration tests pass

## 6. Testing Library Validation

### 6.1 Frontend Test Infrastructure

```bash
cd frontend

# Run tests with coverage
npm test -- --coverage

# Run specific test suites
npm test ChatMessage.test
npm test WorkflowPreview.test
npm test FederationClient.test
```

**Expected Results:**
- ✅ 174+ tests passing
- ✅ Good coverage (aim for >80%)
- ✅ No flaky tests
- ✅ Tests run quickly (<30 seconds)

### 6.2 Backend Test Infrastructure

```bash
# Unit tests
cargo test

# Integration tests (requires services)
cargo test -- --ignored

# Specific test categories
cargo test graphql_federation
cargo test mcp_client
cargo test workflow_engine
```

**Expected Results:**
- ✅ All unit tests pass
- ✅ Integration tests pass with services running
- ✅ No test timeouts
- ✅ Clear test output

### 6.3 Visual Test Dashboard

```bash
# Start the test dashboard
cd frontend/test-dashboard
python -m http.server 8088

# Open in browser
open http://localhost:8088
```

**Expected Results:**
- ✅ Dashboard shows all test results
- ✅ Real-time updates work
- ✅ Test categories are organized
- ✅ Pass/fail status is clear

## Troubleshooting Guide

### Common Issues and Solutions

1. **Port Already in Use**
   ```bash
   # Find process using port
   lsof -i :8080
   # Kill process
   kill -9 <PID>
   ```

2. **Database Connection Failed**
   - Check DATABASE_URL in .env
   - Ensure PostgreSQL is running
   - Verify database exists

3. **GraphQL Federation Errors**
   - Check all services are running
   - Verify port configurations match
   - Look for schema conflicts

4. **Frontend Build Errors**
   - Clear node_modules and reinstall
   - Check Node.js version (18+)
   - Verify no port conflicts

5. **Test Failures**
   - Check service dependencies
   - Ensure clean database state
   - Review recent code changes

## Reporting Issues

When reporting issues, please include:

1. **Which section failed**: (e.g., "Frontend Testing 1.2")
2. **Exact error message**: Copy full error output
3. **Steps to reproduce**: What you did before the error
4. **Environment details**: OS, versions, etc.
5. **Logs**: Relevant service logs if applicable

### Report Format Example:
```
Section: GraphQL Federation Testing 2.2
Error: "Gateway health check failed: Connection refused"
Steps: Ran ./scripts/run-federation-stack.sh, then test-federation.sh
Environment: macOS 14.5, Node 18.17, Rust 1.75
Logs: [Include relevant logs]
```

## Success Criteria

The system is considered fully functional when:

- [ ] All frontend tests pass (174+)
- [ ] Chat interface creates workflows
- [ ] GraphQL Federation connects all services
- [ ] All health checks return success
- [ ] End-to-end workflow creation works
- [ ] No errors in any service logs
- [ ] Visual test dashboard shows all green

## Next Steps

After completing all tests:

1. **If all tests pass**: System is ready for use! Try creating custom workflows.
2. **If some tests fail**: Report issues using the format above.
3. **For performance testing**: Run load tests with `cargo test load_test -- --ignored`
4. **For production deployment**: See deployment guide in main README.

Remember: This is a comprehensive test suite. Some failures might be environmental. Focus on core functionality first, then address edge cases.