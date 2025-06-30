# workflow-engine-app

Production-ready AI workflow orchestration platform with complete observability and enterprise features.

## Features

- **Complete Workflow Platform**: Full-featured AI workflow orchestration with node-based processing
- **Enterprise Ready**: Authentication, rate limiting, monitoring, and production-grade security
- **Microservices Architecture**: Scalable design with optional service decomposition
- **Advanced Observability**: Prometheus metrics, structured logging, distributed tracing, and health monitoring
- **Multi-transport Support**: HTTP, WebSocket, and stdio transport for external integrations
- **Container Native**: Docker and Kubernetes ready with health checks and auto-scaling
- **High Availability**: Built-in resilience patterns with circuit breakers and retry policies

## Quick Start

### Prerequisites

```bash
# Install PostgreSQL
sudo apt-get install postgresql postgresql-contrib

# Create database
sudo -u postgres createdb workflow_db

# Set required environment variables
export DATABASE_URL=postgresql://postgres:password@localhost/workflow_db
export JWT_SECRET=$(openssl rand -hex 64)
```

### Using Pre-built Binary

```bash
# Download latest release
curl -L https://github.com/bredmond1019/workflow-engine-rs/releases/latest/download/workflow-engine > workflow-engine
chmod +x workflow-engine

# Run with basic configuration
./workflow-engine
```

### Building from Source

```bash
# Clone and build
git clone https://github.com/bredmond1019/workflow-engine-rs
cd workflow-engine-rs

# Build release binary
cargo build --release --bin workflow-engine

# Run the application
./target/release/workflow-engine
```

### Using Docker Compose (Recommended)

```bash
# Clone the repository
git clone https://github.com/bredmond1019/workflow-engine-rs
cd workflow-engine-rs

# Start the full stack
docker-compose up -d

# View logs
docker-compose logs -f workflow-engine

# Access services
# Main API: http://localhost:8080
# Swagger UI: http://localhost:8080/swagger-ui/
# Grafana: http://localhost:3000 (admin/admin)
# Prometheus: http://localhost:9090
```

### Quick Test

```bash
# Check health
curl http://localhost:8080/health

# View API documentation
open http://localhost:8080/swagger-ui/

# Trigger a sample workflow
curl -X POST http://localhost:8080/api/v1/workflows/trigger \
  -H "Content-Type: application/json" \
  -d '{
    "workflow_name": "greeting_workflow", 
    "data": {"name": "World", "message": "Hello from workflow engine!"}
  }'
```

## Configuration

### Environment Variables

```bash
# Required
export DATABASE_URL=postgresql://user:pass@localhost/workflow_db
export JWT_SECRET=your-secure-secret-key

# Optional
export HOST=0.0.0.0
export PORT=8080
export RATE_LIMIT_PER_MINUTE=60
export RATE_LIMIT_BURST=10
export RUST_LOG=info
```

### Configuration File

Create `config.yml`:

```yaml
server:
  host: "0.0.0.0"
  port: 8080
  
database:
  url: "postgresql://user:pass@localhost/workflow_db"
  max_connections: 10
  
auth:
  jwt_secret: "your-secret-key"
  token_expiry: 3600
  
rate_limiting:
  requests_per_minute: 60
  burst_size: 10
```

## Services

When running, the application provides:

| Service | URL | Description |
|---------|-----|-------------|
| **Main API** | http://localhost:8080 | REST API endpoints |
| **Swagger UI** | http://localhost:8080/swagger-ui/ | Interactive API docs |
| **Health Check** | http://localhost:8080/health | System health status |
| **Metrics** | http://localhost:8080/metrics | Prometheus metrics |

## Usage Examples

### Trigger a Workflow

```bash
curl -X POST http://localhost:8080/workflows/trigger \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -d '{
    "workflow_type": "data_processing",
    "data": {"input": "sample data"}
  }'
```

### Check Health

```bash
curl http://localhost:8080/health
```

### View API Documentation

Open http://localhost:8080/swagger-ui/ in your browser.

## Docker Compose

For full stack deployment:

```yaml
version: '3.8'
services:
  workflow-engine:
    image: workflow-engine-app:latest
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgresql://postgres:password@db:5432/workflow_db
      - JWT_SECRET=your-secret-key
    depends_on:
      - db
      
  db:
    image: postgres:15
    environment:
      POSTGRES_DB: workflow_db
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: password
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
```

## Development

```bash
# Start in development mode
cargo run --bin workflow-engine-app

# With auto-reload
cargo watch -x "run --bin workflow-engine-app"

# Run tests
cargo test --bin workflow-engine-app
```

## Documentation

For comprehensive documentation, visit [docs.rs/workflow-engine-app](https://docs.rs/workflow-engine-app).

## License

Licensed under the MIT License. See [LICENSE](../../LICENSE) for details.