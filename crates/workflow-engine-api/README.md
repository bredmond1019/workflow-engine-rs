# workflow-engine-api

REST API server for the AI workflow engine.

## Features

- **REST API**: Complete HTTP API with OpenAPI documentation
- **JWT Authentication**: Secure authentication and authorization
- **Rate Limiting**: Configurable rate limiting and CORS support
- **Health Checks**: Comprehensive health monitoring endpoints
- **Service Bootstrap**: Dependency injection and service discovery
- **Prometheus Metrics**: Built-in metrics collection

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

### Core Endpoints
- `GET /health` - Basic health check
- `GET /health/detailed` - Detailed system health
- `GET /metrics` - Prometheus metrics
- `GET /swagger-ui/` - Interactive API documentation

### Authentication
- `POST /auth/login` - JWT authentication
- `POST /auth/refresh` - Token refresh

### Workflows
- `POST /workflows/trigger` - Trigger workflow execution
- `GET /workflows/{id}/status` - Check workflow status
- `GET /workflows/{id}/results` - Get workflow results

## Configuration

Set environment variables:

```bash
export DATABASE_URL=postgresql://user:pass@localhost/db
export JWT_SECRET=your-secret-key
export RATE_LIMIT_PER_MINUTE=60
export RATE_LIMIT_BURST=10
```

## Feature Flags

- `openapi` - OpenAPI documentation generation (default)
- `auth` - JWT authentication support (default)
- `monitoring` - Prometheus metrics endpoints (default)
- `database` - Database integration

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

## Documentation

For comprehensive documentation, visit [docs.rs/workflow-engine-api](https://docs.rs/workflow-engine-api).

## License

Licensed under the MIT License. See [LICENSE](../../LICENSE) for details.