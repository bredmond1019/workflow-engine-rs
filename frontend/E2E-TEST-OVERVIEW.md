# AI Workflow Engine - E2E Test Overview & Production Requirements

## 🎯 Executive Summary

This document provides a comprehensive overview of the end-to-end tests for the AI Workflow Engine, highlighting where mock data is used in testing and what would be required for production deployment.

## 📊 Test Coverage

### 1. **Authentication Flow** (`01-authentication.spec.ts`)
- ✅ Login/logout functionality
- ✅ Role-based access control
- ✅ Token persistence and expiration
- ✅ Error handling

**Production Requirements:**
- JWT secret key configuration
- User database or identity provider integration
- Token refresh mechanism
- Session management

### 2. **Workflow Trigger & Execution** (`02-workflow-trigger.spec.ts`)
- ✅ Workflow selection and triggering
- ✅ Input validation
- ✅ Configuration overrides
- ✅ Error handling

**Production Requirements:**
- Backend workflow orchestration service
- Workflow definition storage
- Input schema validation
- Queue management for workflow execution

### 3. **Workflow Monitoring** (`03-workflow-monitoring.spec.ts`)
- ✅ Real-time status updates
- ✅ Step-by-step progress tracking
- ✅ Output visualization
- ✅ Error reporting

**Production Requirements:**
- WebSocket or SSE for real-time updates
- Workflow state persistence
- Step execution tracking
- Log aggregation

### 4. **Customer Support Workflow** (`04-customer-support-workflow.spec.ts`)
- ✅ Complete workflow execution
- ✅ Multi-service integration
- ✅ Business outcome validation
- ✅ Error recovery

### 5. **Knowledge Base Workflow** (`05-knowledge-base-workflow.spec.ts`)
- ✅ Research and documentation generation
- ✅ Content creation and formatting
- ✅ Knowledge base integration
- ✅ Performance metrics

## 🔌 Integration Points & Requirements

### LLM API Integrations

| Provider | Mock Used In Tests | Production Requirements | Estimated Cost |
|----------|-------------------|------------------------|----------------|
| **OpenAI GPT-4** | - Ticket analysis<br>- Document formatting | `OPENAI_API_KEY` environment variable | ~$0.03/1K tokens |
| **Anthropic Claude** | - Response generation<br>- Content synthesis | `ANTHROPIC_API_KEY` environment variable | ~$0.025/1K tokens |
| **AWS Bedrock** | - Long-form content<br>- Documentation | - AWS credentials<br>- Bedrock model access | ~$0.012/1K tokens |

### MCP Server Integrations

| Service | Port | Mock Functionality | Production Requirements |
|---------|------|-------------------|------------------------|
| **HelpScout** | 8001 | - Ticket retrieval<br>- Customer info<br>- Order details | - Running MCP server<br>- `HELPSCOUT_API_KEY`<br>- Account configuration |
| **Notion** | 8002 | - Knowledge base search<br>- Page creation<br>- Content updates | - Running MCP server<br>- `NOTION_API_KEY`<br>- Workspace access |
| **Slack** | 8003 | - Team notifications<br>- Channel messages<br>- Escalations | - Running MCP server<br>- `SLACK_BOT_TOKEN`<br>- App installation |

## 📋 Mocked API Calls in Tests

### Authentication Mocks
```typescript
// Location: e2e/mocks/api-mocks.ts
- POST /auth/token → Returns mock JWT token
- POST /auth/verify → Returns token validation
```

### Workflow Mocks
```typescript
// Location: e2e/mocks/api-mocks.ts
- GET /api/v1/workflows/available → Returns workflow list
- POST /api/v1/workflows/trigger → Returns instance ID
- GET /api/v1/workflows/status/{id} → Returns progressive status updates
- GET /api/v1/workflows/instances → Returns workflow list
```

### External Service Mocks
```typescript
// All external calls intercepted in test-helpers.ts
- api.openai.com/* → Mock OpenAI responses
- api.anthropic.com/* → Mock Anthropic responses
- bedrock-runtime.*.amazonaws.com/* → Mock AWS Bedrock
- localhost:8001/* → Mock HelpScout MCP
- localhost:8002/* → Mock Notion MCP
- localhost:8003/* → Mock Slack MCP
```

## 🚀 Production Deployment Requirements

### 1. **Environment Variables**
```bash
# Required for production
JWT_SECRET=your-secure-jwt-secret
DATABASE_URL=postgresql://user:pass@host/db

# LLM Provider Keys
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
AWS_ACCESS_KEY_ID=...
AWS_SECRET_ACCESS_KEY=...

# MCP Server Keys
HELPSCOUT_API_KEY=...
NOTION_API_KEY=...
SLACK_BOT_TOKEN=xoxb-...
```

### 2. **Infrastructure Requirements**

#### Backend Services
- **Rust API Server**: Main orchestration service
- **PostgreSQL**: Workflow state and user data
- **Redis**: Caching and queue management
- **MCP Servers**: Python services on ports 8001-8003

#### Monitoring & Observability
- **Prometheus**: Metrics collection
- **Grafana**: Dashboards and alerts
- **Log aggregation**: ELK stack or similar
- **Distributed tracing**: Jaeger or similar

### 3. **Running MCP Servers**
```bash
# Start all MCP servers
cd mcp-servers

# HelpScout MCP Server
python -m servers.helpscout --port 8001

# Notion MCP Server  
python -m servers.notion --port 8002

# Slack MCP Server
python -m servers.slack --port 8003
```

### 4. **Cost Considerations**

#### Per Workflow Execution (Average)
- **Customer Support Workflow**
  - LLM tokens: ~545 tokens
  - Estimated cost: ~$0.017
  - Execution time: ~45 seconds

- **Knowledge Base Workflow**
  - LLM tokens: ~2500 tokens
  - Estimated cost: ~$0.044
  - Execution time: ~60 seconds

#### Monthly Estimates (1000 workflows)
- LLM API costs: ~$30-50
- Infrastructure: ~$200-500
- MCP server hosting: ~$100

## 🧪 Running the Tests

### Setup
```bash
# Install dependencies
npm install

# Install Playwright browsers
npx playwright install
```

### Run Tests
```bash
# Run all tests
npm run test:e2e

# Run specific test suite
npm run test:e2e -- --grep "Authentication"

# Run in headed mode (see browser)
npm run test:e2e -- --headed

# Run with debug mode
npm run test:e2e -- --debug
```

### Test Reports
```bash
# Generate HTML report
npx playwright show-report

# View test traces
npx playwright show-trace trace.zip
```

## 📈 Next Steps

### Immediate Actions
1. **Set up environment variables** for LLM providers
2. **Deploy MCP servers** with proper authentication
3. **Configure backend API** with production database
4. **Set up monitoring** infrastructure

### Phase 1: Core Functionality (Week 1)
- [ ] Deploy backend API server
- [ ] Configure PostgreSQL database
- [ ] Set up basic authentication
- [ ] Deploy one MCP server for testing

### Phase 2: External Integrations (Week 2)
- [ ] Obtain and configure API keys
- [ ] Deploy all MCP servers
- [ ] Test LLM integrations
- [ ] Implement rate limiting and retry logic

### Phase 3: Production Hardening (Week 3)
- [ ] Set up monitoring and alerts
- [ ] Implement backup and recovery
- [ ] Load testing and optimization
- [ ] Security audit and penetration testing

### Phase 4: Scaling (Week 4+)
- [ ] Implement horizontal scaling
- [ ] Add caching layers
- [ ] Optimize LLM token usage
- [ ] Set up CI/CD pipelines

## 🔒 Security Considerations

1. **API Key Management**
   - Use secrets management service (Vault, AWS Secrets Manager)
   - Rotate keys regularly
   - Implement key-level rate limiting

2. **Network Security**
   - TLS for all communications
   - VPN or private networking for MCP servers
   - API gateway with rate limiting

3. **Data Protection**
   - Encrypt sensitive data at rest
   - PII handling compliance
   - Audit logging for all operations

## 📞 Support & Documentation

- **Backend Documentation**: See `/CLAUDE.md` in root
- **MCP Server Docs**: See `/mcp-servers/README.md`
- **API Reference**: Available at `/swagger-ui` when running
- **Support**: File issues in GitHub repository

---

**Note**: This test suite uses extensive mocking to simulate production behavior. All external API calls and MCP server interactions are mocked to allow testing without incurring costs or requiring live services. In production, these mocks would be replaced with actual API calls requiring proper authentication and configuration.