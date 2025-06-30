# workflow-engine-api

Production-ready REST and GraphQL API server for the AI workflow engine.

[![Crates.io](https://img.shields.io/crates/v/workflow-engine-api.svg)](https://crates.io/crates/workflow-engine-api)
[![Documentation](https://docs.rs/workflow-engine-api/badge.svg)](https://docs.rs/workflow-engine-api)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

## Features

- **Dual API Support**: REST and GraphQL APIs with full documentation
- **Event Sourcing**: Complete CQRS implementation with snapshots and replay
- **JWT Authentication**: Secure authentication with role-based access control
- **Rate Limiting**: Per-endpoint configurable limits with burst support
- **Service Bootstrap**: Advanced dependency injection and service discovery
- **Production Monitoring**: Prometheus metrics, distributed tracing, health checks
- **Multi-tenancy**: Schema, row-level, and hybrid tenant isolation

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
workflow-engine-api = "0.6.0"
tokio = { version = "1.0", features = ["full"] }
```

Start the API server:

```rust
use workflow_engine_api::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let container = ServiceContainer::new().await?;
    let server = ApiServer::new(container);
    server.run("127.0.0.1:8080").await?;
    Ok(())
}
```

## API Endpoints

### REST API
```
# Core
GET  /health                     - Basic health check
GET  /health/detailed            - Detailed system health  
GET  /metrics                    - Prometheus metrics
GET  /swagger-ui/                - Interactive API documentation

# Authentication
POST /api/v1/auth/login          - JWT authentication
POST /api/v1/auth/refresh        - Token refresh
POST /api/v1/auth/logout         - Logout

# Workflows
POST /api/v1/workflows/trigger   - Trigger workflow execution
GET  /api/v1/workflows/{id}/status - Check workflow status
GET  /api/v1/workflows/{id}/results - Get workflow results
GET  /api/v1/workflows/templates - List available templates
```

### GraphQL API
```
POST /graphql                    - GraphQL endpoint
GET  /graphql/playground         - Interactive GraphQL IDE
```

## Advanced Usage

### Event Sourcing

```rust
use workflow_engine_api::db::events::{EventStore, Event};

// Initialize event store
let event_store = EventStore::new(pool.clone());

// Configure snapshots
event_store.configure_snapshots(SnapshotConfig {
    events_per_snapshot: 100,
    max_snapshot_age: Duration::from_days(30),
});

// Store events
let event = Event::workflow_started(workflow_id, metadata);
event_store.append_event(&event).await?;

// Query and replay
let events = event_store.get_events_after(timestamp).await?;
let state = event_store.replay_aggregate(aggregate_id).await?;
```

### Service Bootstrap

```rust
use workflow_engine_api::bootstrap::{ServiceContainer, ServiceConfig};

// Configure services
let config = ServiceConfig {
    database_url: "postgresql://localhost/workflow_db",
    jwt_secret: "secure-secret",
    enable_monitoring: true,
    enable_graphql: true,
};

// Initialize container
let container = ServiceContainer::from_config(config).await?;

// Register custom services
container.register_singleton(MyService::new());
container.register_factory(|| Box::new(MyFactory::new()));

// Start API server
let server = ApiServer::new(container);
server.run("0.0.0.0:8080").await?;
```

### Multi-tenancy

```rust
use workflow_engine_api::db::tenant::{TenantIsolation, TenantContext};

// Configure tenant isolation
let isolation = TenantIsolation::Hybrid {
    schema_prefix: "tenant_",
    row_level_tables: vec!["workflows", "events"],
};

// Create tenant context
let context = TenantContext::new(tenant_id, isolation);

// Use in queries
let workflows = context.query::<Workflow>()
    .filter(status.eq("active"))
    .load(&conn)?;
```

## Configuration

### Required Environment Variables
```bash
DATABASE_URL=postgresql://user:pass@localhost/workflow_db
JWT_SECRET=your-secure-secret-key  # No default for security
```

### Optional Configuration
```bash
# API Settings
API_HOST=0.0.0.0
API_PORT=8080
RATE_LIMIT_PER_MINUTE=60
RATE_LIMIT_BURST=10

# Features
ENABLE_GRAPHQL=true
ENABLE_MONITORING=true
ENABLE_TRACING=true

# Event Sourcing
EVENTS_PER_SNAPSHOT=100
EVENT_RETENTION_DAYS=90

# External Services  
MCP_HELPSCOUT_URL=http://localhost:8001
MCP_NOTION_URL=http://localhost:8002
MCP_SLACK_URL=http://localhost:8003
```

## Feature Flags

- `default = ["openapi", "auth", "monitoring"]` - Core features
- `graphql` - GraphQL API support
- `openapi` - OpenAPI documentation generation
- `auth` - JWT authentication support
- `monitoring` - Prometheus metrics endpoints
- `database` - PostgreSQL integration
- `event-sourcing` - CQRS event store
- `multi-tenant` - Multi-tenancy support
- `full` - All features enabled

## Docker Support

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin workflow-engine-api

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/workflow-engine-api /usr/local/bin/
EXPOSE 8080
CMD ["workflow-engine-api"]
```

## Testing

```bash
# Unit tests
cargo test -p workflow-engine-api

# Integration tests (requires database)
DATABASE_URL=postgresql://localhost/test_db cargo test -p workflow-engine-api -- --ignored

# Specific test suites
cargo test -p workflow-engine-api event_sourcing
cargo test -p workflow-engine-api graphql
cargo test -p workflow-engine-api auth
```

## Monitoring

The API provides comprehensive monitoring capabilities:

### Prometheus Metrics
- `http_requests_total` - Total HTTP requests by method and path
- `http_request_duration_seconds` - Request latency histogram
- `workflow_executions_total` - Workflow execution counts
- `event_store_operations` - Event sourcing metrics
- `db_pool_connections` - Database connection pool stats

### Health Checks
```json
// GET /health/detailed
{
  "status": "healthy",
  "components": {
    "database": { "status": "up", "latency_ms": 2 },
    "event_store": { "status": "up", "pending_events": 0 },
    "mcp_services": {
      "helpscout": { "status": "up" },
      "notion": { "status": "up" }
    }
  },
  "uptime_seconds": 3600
}
```

## Documentation

For comprehensive documentation, visit [docs.rs/workflow-engine-api](https://docs.rs/workflow-engine-api).

## Examples

See the [examples directory](../../examples/) for:
- Basic API server setup
- Custom workflow registration
- Event sourcing patterns
- Multi-tenant configuration
- GraphQL schema extension

## Dependencies

This crate depends on:
- `workflow-engine-core` - Core types and workflow engine
- `workflow-engine-mcp` - MCP protocol support
- `workflow-engine-nodes` - Built-in workflow nodes

Key external dependencies:
- `actix-web` - Web framework
- `diesel` - ORM and query builder
- `juniper` - GraphQL implementation
- `jsonwebtoken` - JWT handling

## Contributing

Contributions are welcome! Please read our [Contributing Guide](../../CONTRIBUTING.md) for details.

## License

Licensed under the MIT License. See [LICENSE](../../LICENSE) for details.