# Quick Start Verification Checklist

This checklist ensures that the QUICK_START.md guide works correctly for new users. Use this to validate the setup process.

## Pre-Setup Verification

### System Requirements
- [ ] Verify minimum system specs (2+ cores, 8GB RAM, 4GB storage)
- [ ] Test on supported OS:
  - [ ] macOS 11+
  - [ ] Ubuntu 20.04+
  - [ ] Windows 11 with WSL2

### Prerequisites Installation
- [ ] Rust installation command works
- [ ] Node.js installation provides v18+
- [ ] PostgreSQL installation provides v15+
- [ ] Docker installation works
- [ ] Python 3.11+ installation works
- [ ] uv installation completes successfully

## 5-Minute Quick Start Path

### Step 1: Clone and Setup
- [ ] Repository clones successfully
- [ ] `graphql-federation` branch exists and checks out
- [ ] uv installation script works
- [ ] Database creation succeeds
- [ ] init-db.sql script exists and runs without errors
- [ ] .env.example file exists
- [ ] .env file can be created and edited

### Step 2: Docker Services
- [ ] docker-compose.yml exists in root
- [ ] `docker-compose up -d` starts without errors
- [ ] All containers start successfully:
  - [ ] postgres
  - [ ] ai-workflow-system
  - [ ] content-processing
  - [ ] knowledge-graph
  - [ ] realtime-communication
  - [ ] graphql-gateway
  - [ ] prometheus
  - [ ] grafana
  - [ ] jaeger
  - [ ] redis

### Step 3: Frontend Setup
- [ ] Frontend directory exists
- [ ] `npm install` completes without errors
- [ ] `npm run dev` starts development server
- [ ] Frontend accessible at http://localhost:5173

### Step 4: Verification
- [ ] verify-setup.sh script exists and is executable
- [ ] Script runs without errors
- [ ] All health checks pass:
  - [ ] Main API (8080)
  - [ ] GraphQL Gateway (4000)
  - [ ] Frontend (5173)
  - [ ] All microservices
  - [ ] Database connection
  - [ ] Monitoring stack

## Rust Library Usage Path

### Basic Example
- [ ] Cargo.toml dependencies are correct
- [ ] Hello World example compiles
- [ ] Hello World example runs successfully
- [ ] Output shows expected greeting

### AI Example
- [ ] AI-powered example compiles
- [ ] Handles missing API keys gracefully
- [ ] With valid API key, runs successfully

## Full Development Setup Path

### Automated Setup
- [ ] setup.sh script exists
- [ ] Script is executable
- [ ] Script completes all steps:
  - [ ] Dependency installation
  - [ ] Database setup
  - [ ] Service building
  - [ ] Environment configuration
  - [ ] MCP server setup
  - [ ] Verification

### Manual Setup Steps
- [ ] Each dependency installation command works
- [ ] Database user and database creation succeeds
- [ ] Schema initialization works
- [ ] Dgraph setup completes
- [ ] All builds complete successfully:
  - [ ] Main application
  - [ ] GraphQL Gateway
  - [ ] Each microservice
  - [ ] Python MCP servers
  - [ ] Frontend

## Service Verification

### Core Services
- [ ] Main API responds at http://localhost:8080/health
- [ ] GraphQL Gateway playground loads at http://localhost:4000/graphql
- [ ] Frontend loads at http://localhost:5173
- [ ] Swagger UI loads at http://localhost:8080/swagger-ui/

### Microservices
- [ ] Content Processing: http://localhost:8082/health
- [ ] Knowledge Graph: http://localhost:3002/health
- [ ] Realtime Communication: http://localhost:8081/health

### MCP Servers
- [ ] start_test_servers.sh exists and runs
- [ ] All MCP servers start:
  - [ ] Customer Support
  - [ ] HelpScout
  - [ ] Notion
  - [ ] Slack

### Monitoring Stack
- [ ] Grafana loads at http://localhost:3000
- [ ] Default credentials (admin/admin) work
- [ ] Prometheus loads at http://localhost:9090
- [ ] Jaeger UI loads at http://localhost:16686

## Testing Verification

### Unit Tests
- [ ] `cargo test` runs successfully
- [ ] No failing tests

### Frontend Tests
- [ ] `cd frontend && npm test` runs
- [ ] 174+ tests pass
- [ ] No test failures

### Integration Tests
- [ ] setup-test-environment.sh exists and runs
- [ ] `cargo test -- --ignored` completes
- [ ] Federation tests pass

### Visual Test Dashboard
- [ ] test-dashboard.sh exists and runs
- [ ] Dashboard opens in browser
- [ ] Shows test results

## Common Issues Resolution

### Port Conflicts
- [ ] Port checking commands work
- [ ] Environment variable override works
- [ ] Services start on alternate ports

### Database Issues
- [ ] pg_isready command available
- [ ] Database connection test works
- [ ] database-setup.sh exists for reset

### MCP Server Issues
- [ ] Python version check works
- [ ] uv reinstall command works
- [ ] Manual server start works

### Frontend Issues
- [ ] Cache clearing commands work
- [ ] Fresh install resolves issues

### Docker Issues
- [ ] docker info provides output
- [ ] Docker reset commands work
- [ ] Log viewing works

## GraphQL Federation Features

### Federation Endpoints
- [ ] GraphQL Gateway playground loads
- [ ] Federation health query works
- [ ] Subgraph introspection works
- [ ] Example queries execute successfully:
  ```graphql
  {
    workflows {
      id
      name
      status
    }
  }
  ```

### Federation Validation
- [ ] validate_federation.sh exists
- [ ] Script runs successfully
- [ ] Federation examples work:
  - [ ] federated_query example
  - [ ] test_federation example

## Documentation Links

### Verify all links work:
- [ ] CLAUDE.md exists
- [ ] DEVELOPMENT_SETUP.md exists
- [ ] SYSTEM_TESTING.md exists
- [ ] frontend/USER_TESTING.md exists
- [ ] Component CLAUDE.md files exist:
  - [ ] workflow-engine-api/CLAUDE.md
  - [ ] workflow-engine-gateway/README.md
  - [ ] frontend/README.md

## First User Experience

### New User Flow
- [ ] User can follow 5-minute setup
- [ ] No unexplained errors occur
- [ ] All services start successfully
- [ ] Verification confirms working setup
- [ ] User can access frontend
- [ ] User can run a test workflow

### Developer Experience
- [ ] API documentation is accessible
- [ ] GraphQL playground works
- [ ] Example workflows run
- [ ] Tests can be executed
- [ ] Monitoring dashboards load

## Performance Checks

### Startup Times
- [ ] Docker services start within 30 seconds
- [ ] Frontend dev server starts within 10 seconds
- [ ] First page load is responsive

### Resource Usage
- [ ] System remains responsive during setup
- [ ] Memory usage stays under 8GB
- [ ] CPU usage is reasonable

## Final Validation

### Complete System Test
1. [ ] Create a test workflow via chat UI
2. [ ] Execute the workflow
3. [ ] View results in monitoring
4. [ ] Check logs for errors
5. [ ] Verify data persistence

### Success Criteria
- [ ] All core services running
- [ ] No critical errors in logs
- [ ] Frontend fully functional
- [ ] API endpoints responsive
- [ ] GraphQL queries work
- [ ] Tests pass
- [ ] Monitoring shows healthy metrics

## Notes Section

Record any issues found during verification:

```
Date: _____________
Tester: ___________
Environment: ______

Issues Found:
1. 
2. 
3. 

Recommendations:
1. 
2. 
3. 
```

## Sign-off

- [ ] Quick Start guide verified as working
- [ ] All paths tested successfully
- [ ] Documentation is accurate
- [ ] Ready for new users

Verified by: _________________ Date: _________________