# workflow-engine-api

Production-ready REST API server for the AI workflow orchestration system.

## Features

- **Complete REST API**: Comprehensive HTTP API with auto-generated OpenAPI documentation
- **Enterprise Authentication**: JWT-based auth with role-based access control
- **Advanced Rate Limiting**: Configurable per-endpoint rate limiting with burst handling
- **Health & Monitoring**: Detailed health checks and Prometheus metrics integration
- **Service Bootstrap**: Advanced dependency injection and service discovery framework
- **Event-Driven Architecture**: Full event sourcing with audit trails and replay capabilities
- **Multi-tenancy**: Built-in tenant isolation and data partitioning
- **Production Ready**: CORS, correlation IDs, structured logging, and graceful shutdown

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
workflow-engine-api = "0.6.0"
workflow-engine-core = "0.6.0"
tokio = { version = "1.0", features = ["full"] }
actix-web = "4.0"
```

### Basic Server Setup

```rust
use workflow_engine_api::prelude::*;
use actix_web::{web, App, HttpServer, Result};

#[actix_web::main]
async fn main() -> Result<()> {
    // Initialize database pool
    let db_pool = workflow_engine_api::db::session::init_pool()?;
    
    // Configure JWT authentication
    let jwt_auth = workflow_engine_core::auth::JwtAuth::new("your-secret-key");
    
    // Start server with all features
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(jwt_auth.clone()))
            .configure(workflow_engine_api::api::init_routes)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

### With Service Bootstrap

```rust
use workflow_engine_api::bootstrap::{ServiceBootstrapManager, ServiceConfiguration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure services with dependency injection
    let manager = ServiceBootstrapManager::builder()
        .add_service("database", database_config())
        .add_service("auth", auth_config())
        .add_service("workflow_engine", workflow_config())
        .build()?;

    // Start all services
    manager.start_all().await?;
    
    // Server runs until shutdown signal
    manager.wait_for_shutdown().await?;
    Ok(())
}
```

## API Endpoints

### Health & Monitoring
```bash
GET /health                    # Basic health check
GET /health/detailed           # Comprehensive system health with dependencies
GET /metrics                   # Prometheus metrics endpoint
GET /api/v1/uptime            # Server uptime and performance stats
```

### Authentication
```bash
POST /api/v1/auth/login       # JWT authentication with credentials
POST /api/v1/auth/refresh     # Refresh JWT token
POST /api/v1/auth/logout      # Logout and invalidate token
```

### Workflow Management
```bash
POST /api/v1/workflows/trigger           # Trigger workflow execution
GET  /api/v1/workflows/{id}/status       # Get workflow execution status
GET  /api/v1/workflows/{id}/results      # Get workflow results and outputs
GET  /api/v1/workflows/templates         # List available workflow templates
POST /api/v1/workflows/{id}/cancel       # Cancel running workflow
GET  /api/v1/workflows/{id}/logs         # Get workflow execution logs
```

### Service Discovery & Registry
```bash
GET    /api/v1/registry/services         # List all registered services
POST   /api/v1/registry/register         # Register new service instance
DELETE /api/v1/registry/deregister/{id}  # Deregister service instance
GET    /api/v1/registry/health           # Service health overview
```

### Event Management
```bash
POST /events                             # Create new event (event sourcing)
GET  /api/v1/events/stream              # Server-sent events stream
GET  /api/v1/events/{aggregate_id}      # Get events for aggregate
```

### Documentation
```bash
GET /swagger-ui/                         # Interactive OpenAPI documentation
GET /api/v1/openapi.json               # OpenAPI specification
```

## Usage Examples

### Trigger a Workflow

```bash
# Authenticate first
TOKEN=$(curl -s -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "password"}' | jq -r '.token')

# Trigger customer support workflow
curl -X POST http://localhost:8080/api/v1/workflows/trigger \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "workflow_name": "customer_support",
    "data": {
      "ticket_id": "12345",
      "customer_message": "I need help with my order"
    }
  }'

# Check workflow status
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:8080/api/v1/workflows/{workflow_id}/status"
```

### Health Monitoring

```bash
# Basic health check
curl http://localhost:8080/health

# Detailed health with dependencies
curl http://localhost:8080/health/detailed

# Prometheus metrics
curl http://localhost:8080/metrics
```

### Service Registration

```bash
# Register a new service
curl -X POST http://localhost:8080/api/v1/registry/register \
  -H "Content-Type: application/json" \
  -d '{
    "service_name": "my-service",
    "service_url": "http://localhost:3000",
    "health_check_url": "http://localhost:3000/health",
    "metadata": {"version": "1.0.0"}
  }'
```

## Configuration

### Environment Variables

#### Required
```bash
export DATABASE_URL=postgresql://user:pass@localhost/workflow_db
export JWT_SECRET=your-secure-random-secret-key
```

#### Optional
```bash
# Server Configuration
export HOST=0.0.0.0                    # Default: 127.0.0.1
export PORT=8080                       # Default: 8080

# Rate Limiting
export RATE_LIMIT_PER_MINUTE=60       # Default: 60
export RATE_LIMIT_BURST=10            # Default: 10

# Database
export DB_MAX_CONNECTIONS=10           # Default: 10
export DB_CONNECTION_TIMEOUT=30        # Default: 30 seconds

# Authentication
export JWT_EXPIRATION=3600             # Default: 1 hour
export JWT_REFRESH_EXPIRATION=604800   # Default: 7 days

# Monitoring
export METRICS_ENABLED=true           # Default: true
export TRACING_ENABLED=true           # Default: true
export LOG_LEVEL=info                 # Default: info

# External Services
export NOTION_MCP_URL=http://localhost:8002
export SLACK_MCP_URL=http://localhost:8003
export HELPSCOUT_MCP_URL=http://localhost:8001
```

### Configuration File (Optional)

Create `config.toml`:

```toml
[server]
host = "0.0.0.0"
port = 8080

[database]
url = "postgresql://user:pass@localhost/workflow_db"
max_connections = 10
connection_timeout = 30

[auth]
jwt_secret = "your-secret-key"
jwt_expiration = 3600
refresh_expiration = 604800

[rate_limiting]
requests_per_minute = 60
burst_size = 10

[monitoring]
metrics_enabled = true
tracing_enabled = true
log_level = "info"

[services]
notion_mcp_url = "http://localhost:8002"
slack_mcp_url = "http://localhost:8003"
helpscout_mcp_url = "http://localhost:8001"
```

## Feature Flags

- `default` - Enables `openapi`, `auth`, `monitoring`, `database`
- `openapi` - OpenAPI documentation generation with Swagger UI
- `auth` - JWT authentication and authorization middleware
- `monitoring` - Prometheus metrics and health check endpoints
- `database` - PostgreSQL database integration with Diesel ORM

## Docker Support

### Basic Dockerfile

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin workflow-engine

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libpq5 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/workflow-engine /usr/local/bin/
EXPOSE 8080
CMD ["workflow-engine"]
```

### Multi-stage with Health Checks

```dockerfile
FROM rust:1.75-slim as builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev libpq-dev
COPY . .
RUN cargo build --release --bin workflow-engine

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libpq5 curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/workflow-engine /usr/local/bin/
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1
CMD ["workflow-engine"]
```

## Documentation

For comprehensive documentation, visit [docs.rs/workflow-engine-api](https://docs.rs/workflow-engine-api).

## License

Licensed under the MIT License. See [LICENSE](../../LICENSE) for details.