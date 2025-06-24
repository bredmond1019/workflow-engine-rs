# CLAUDE.md - workflow-engine-api

This file provides guidance for Claude Code when working with the workflow-engine-api crate.

## Crate Overview

The workflow-engine-api crate is the main HTTP API server for the AI workflow orchestration system. It provides a production-ready REST API with authentication, monitoring, and workflow execution capabilities built on top of Actix-web.

### Purpose and Role

- **Primary API Gateway**: Serves as the main entry point for all HTTP requests
- **Workflow Orchestration**: Triggers and monitors workflow executions
- **Service Bootstrap**: Manages dependency injection and service lifecycle
- **Authentication & Authorization**: JWT-based security with middleware
- **Monitoring & Observability**: Prometheus metrics, health checks, and distributed tracing

## Key Components and Modules

### 1. API Module (`src/api/`)
- **Core Endpoints**: Health checks, metrics, authentication, and workflow management
- **Middleware**: Authentication, rate limiting, CORS, and correlation ID tracking
- **OpenAPI**: Auto-generated API documentation with Swagger UI
- **Routes**: Modular route configuration for different API domains

### 2. Bootstrap Module (`src/bootstrap/`)
- **Service Container**: Dependency injection and service resolution
- **Service Discovery**: Dynamic service registration and discovery
- **Configuration Management**: Environment-based configuration with validation
- **Lifecycle Management**: Service startup, shutdown, and health monitoring

### 3. Database Module (`src/db/`)
- **Connection Pooling**: Efficient database connection management
- **Event Sourcing**: Complete event-driven architecture implementation
- **Multi-tenancy**: Tenant isolation and data partitioning
- **Repository Pattern**: Clean data access abstractions

### 4. Workflows Module (`src/workflows/`)
- **Workflow Execution**: Runtime engine for executing workflow definitions
- **Node Registry**: Dynamic registration of workflow nodes
- **Demo Workflows**: Customer support and knowledge base examples
- **MCP Integration**: External service integration through Model Context Protocol

### 5. Monitoring Module (`src/monitoring/`)
- **Metrics Collection**: Prometheus metrics for API performance
- **Correlation Tracking**: Request tracing across distributed systems
- **Structured Logging**: JSON-formatted logs with context
- **Distributed Tracing**: OpenTelemetry integration

## Important Files and Their Functions

### Core API Files
- `src/api/mod.rs` - Main route configuration and API initialization
- `src/api/startup.rs` - Server startup and configuration
- `src/api/workflows.rs` - Workflow trigger and status endpoints
- `src/api/middleware/auth.rs` - JWT authentication middleware
- `src/api/openapi.rs` - OpenAPI schema generation

### Bootstrap Files
- `src/bootstrap/container.rs` - Dependency injection container
- `src/bootstrap/manager.rs` - Service bootstrap orchestration
- `src/bootstrap/discovery.rs` - Service discovery mechanisms
- `src/bootstrap/lifecycle.rs` - Service state management

### Database Files
- `src/db/connection_pool.rs` - PostgreSQL connection pooling
- `src/db/events/store.rs` - Event store implementation
- `src/db/tenant.rs` - Multi-tenant data isolation
- `src/db/schema.rs` - Diesel schema definitions

### Workflow Files
- `src/workflows/executor.rs` - Workflow execution engine
- `src/workflows/registry.rs` - Workflow template registry
- `src/workflows/parser.rs` - Workflow definition parsing
- `src/workflows/nodes/` - Custom workflow nodes

## Main APIs and Endpoints

### Authentication Endpoints
```
POST /api/v1/auth/login - User authentication
POST /api/v1/auth/refresh - Token refresh
POST /api/v1/auth/logout - User logout
```

### Workflow Management
```
POST /api/v1/workflows/trigger - Trigger workflow execution
GET /api/v1/workflows/status/{id} - Get workflow status
GET /api/v1/workflows/results/{id} - Get workflow results
GET /api/v1/workflows/templates - List available templates
```

### Health & Monitoring
```
GET /health - Basic health check
GET /health/detailed - Detailed system health
GET /metrics - Prometheus metrics
GET /api/v1/uptime - System uptime information
```

### Service Discovery
```
GET /api/v1/registry/services - List registered services
POST /api/v1/registry/register - Register new service
DELETE /api/v1/registry/deregister/{id} - Deregister service
```

## Database Interactions

### Connection Management
```rust
// Use connection pool for all database operations
use crate::db::connection_pool::DbPool;

let pool = DbPool::new(&database_url)?;
let conn = pool.get()?;
```

### Event Sourcing Pattern
```rust
// Store events for audit and replay
use crate::db::events::store::EventStore;

let event_store = EventStore::new(pool);
event_store.append_event(aggregate_id, event).await?;
```

### Repository Pattern
```rust
// Access data through repositories
use crate::db::user::UserRepository;

let user_repo = UserRepository::new(pool);
let user = user_repo.find_by_id(user_id).await?;
```

## Integration Patterns

### MCP Client Integration
```rust
// Integrate external services via MCP
use workflow_engine_mcp::client::McpClient;

let client = McpClient::new("http://notion-mcp:8002")?;
let response = client.call_tool("search", params).await?;
```

### Service Bootstrap
```rust
// Bootstrap services with dependency injection
use crate::bootstrap::{ServiceBootstrapManager, ServiceConfiguration};

let manager = ServiceBootstrapManager::builder()
    .add_service("api", api_config)
    .add_service("workflow", workflow_config)
    .build()?;

manager.start_all().await?;
```

### Middleware Chain
```rust
// Configure middleware pipeline
use crate::api::middleware::{auth, correlation, rate_limit};

App::new()
    .wrap(correlation::CorrelationId)
    .wrap(rate_limit::RateLimiter::default())
    .wrap(auth::JwtAuth)
```

## Testing Approach

### Unit Tests
```bash
# Run unit tests for API logic
cargo test --package workflow-engine-api --lib

# Test specific modules
cargo test --package workflow-engine-api bootstrap::
cargo test --package workflow-engine-api api::workflows
```

### Integration Tests
```bash
# Start required services first
docker-compose up -d postgres redis

# Run integration tests
cargo test --package workflow-engine-api --test '*' -- --ignored

# Test with external MCP servers
./scripts/start_test_servers.sh
cargo test --package workflow-engine-api workflow_integration -- --ignored
```

### API Testing
```bash
# Use httpie or curl for manual testing
http POST localhost:8080/api/v1/auth/login email=test@example.com password=secret
http POST localhost:8080/api/v1/workflows/trigger workflow_name=customer_support Authorization:"Bearer $TOKEN"

# Load testing with vegeta
echo "POST http://localhost:8080/api/v1/workflows/trigger" | vegeta attack -rate=100/s -duration=30s
```

## Common Development Tasks

### Adding a New API Endpoint

1. Create route handler in `src/api/routes/`
2. Define request/response types with serde
3. Add OpenAPI documentation attributes
4. Register route in `src/api/mod.rs`
5. Add integration tests

Example:
```rust
// src/api/routes/custom.rs
#[post("/api/v1/custom/action")]
pub async fn custom_action(
    req: web::Json<CustomRequest>,
    auth: AuthUser,
) -> Result<HttpResponse, ApiError> {
    // Implementation
}
```

### Creating a Workflow Node

1. Implement node in `src/workflows/nodes/`
2. Register in workflow registry
3. Add configuration schema
4. Create unit tests

Example:
```rust
// src/workflows/nodes/custom_node.rs
pub struct CustomNode;

impl WorkflowNode for CustomNode {
    async fn execute(&self, inputs: Value) -> Result<Value> {
        // Node logic
    }
}
```

### Adding Database Models

1. Define schema in `src/db/schema.rs`
2. Create model struct with Diesel derives
3. Implement repository in `src/db/`
4. Run migrations: `diesel migration generate <name>`

### Implementing Service Integration

1. Create MCP client wrapper in `src/integrations/`
2. Add service configuration to bootstrap
3. Implement health checks
4. Add to service discovery

### Adding Metrics

1. Define metric in `src/monitoring/metrics.rs`
2. Instrument code with metric updates
3. Add Grafana dashboard panel
4. Document metric in README

## Environment Variables

### Required
```bash
DATABASE_URL=postgresql://user:pass@localhost/workflow_db
JWT_SECRET=your-secure-secret-key
```

### Optional
```bash
# API Configuration
API_HOST=0.0.0.0
API_PORT=8080
RATE_LIMIT_PER_MINUTE=60
RATE_LIMIT_BURST=10

# Service Discovery
DISCOVERY_ENDPOINT=http://consul:8500
SERVICE_NAME=workflow-api
SERVICE_ID=workflow-api-1

# Monitoring
METRICS_ENABLED=true
TRACING_ENABLED=true
JAEGER_ENDPOINT=http://jaeger:14268/api/traces

# External Services
NOTION_MCP_URL=http://notion-mcp:8002
SLACK_MCP_URL=http://slack-mcp:8003
HELPSCOUT_MCP_URL=http://helpscout-mcp:8001
```

## Debugging Tips

### Request Tracing
- Check correlation ID in logs: `grep "correlation_id: abc123" logs/api.log`
- Use X-Correlation-ID header for testing
- Enable debug logging: `RUST_LOG=workflow_engine_api=debug`

### Database Issues
- Check connection pool metrics: `GET /metrics | grep db_pool`
- Enable query logging: `RUST_LOG=diesel=debug`
- Monitor slow queries in PostgreSQL

### Workflow Debugging
- Use workflow status endpoint for execution details
- Check workflow logs: `grep "workflow_id: xyz789" logs/workflow.log`
- Enable step-by-step execution in workflow config

### Performance Analysis
- Monitor API latency: `GET /metrics | grep http_request_duration`
- Check Grafana dashboards at `http://localhost:3000`
- Use flamegraph for CPU profiling

## Security Considerations

1. **JWT Tokens**: Rotate JWT_SECRET regularly
2. **Rate Limiting**: Configure appropriate limits per endpoint
3. **CORS**: Restrict allowed origins in production
4. **Database**: Use connection SSL and row-level security
5. **Monitoring**: Don't log sensitive data in metrics

## Production Deployment

1. Use environment-specific configs
2. Enable all monitoring features
3. Configure proper health check timeouts
4. Set up log aggregation (ELK/CloudWatch)
5. Implement graceful shutdown handlers
6. Use container orchestration (K8s/ECS)