# CLAUDE.md - workflow-engine-app

This file provides guidance to Claude Code when working with the workflow-engine-app crate, which is the main binary entry point for the AI workflow orchestration system.

## Purpose and Role

The `workflow-engine-app` crate serves as the main executable binary that brings together all components of the workflow engine into a cohesive, production-ready application. It:

- Acts as the primary entry point for the entire workflow orchestration system
- Integrates all workspace crates (`workflow-engine-core`, `workflow-engine-api`)
- Configures and starts the HTTP server with all middleware
- Manages application lifecycle and graceful shutdown
- Provides the production deployment target

## Main Function and Startup Sequence

The application startup follows this sequence (`src/main.rs`):

1. **Environment Setup**
   - Load `.env` file via `dotenvy`
   - Configure logging based on `RUST_LOG` environment variable
   - Initialize process start time for uptime metrics

2. **Structured Logging**
   - Initialize correlation ID support for request tracing
   - Set up structured logging with JSON output when configured

3. **Server Configuration**
   - Read `HOST` (default: "127.0.0.1") and `PORT` (default: "8080") from environment
   - Build server URL for binding

4. **Database Initialization**
   - Create PostgreSQL connection pool using `DATABASE_URL`
   - Wrap pool in Arc for thread-safe sharing across handlers

5. **Authentication Setup**
   - Initialize JWT authentication with `JWT_SECRET`
   - Create shared JWT auth instance for middleware

6. **Rate Limiting Configuration**
   - Configure requests per minute (`RATE_LIMIT_PER_MINUTE`, default: 60)
   - Configure burst size (`RATE_LIMIT_BURST`, default: 10)

7. **HTTP Server Creation**
   - Configure CORS (currently allows any origin in development)
   - Apply middleware stack in order:
     - Logger middleware
     - CORS middleware
     - Rate limiting middleware
     - JWT authentication middleware
   - Mount API routes via `api::init_routes`
   - Start server and await connections

## Application Configuration

### Required Environment Variables

```bash
DATABASE_URL=postgresql://user:pass@localhost/workflow_db  # PostgreSQL connection
JWT_SECRET=your-secure-secret-key                         # JWT signing key
```

### Optional Environment Variables

```bash
HOST=0.0.0.0                    # Server bind address (default: 127.0.0.1)
PORT=8080                       # Server port (default: 8080)
RATE_LIMIT_PER_MINUTE=60       # Rate limit requests (default: 60)
RATE_LIMIT_BURST=10            # Rate limit burst size (default: 10)
RUST_LOG=info                  # Log level (default: info)
```

### Feature Flags

The crate supports these feature flags:

- `default`: Enables the `full` feature set
- `full`: Includes all features from core and API crates, database, and monitoring
- `database`: Enables database support across all crates
- `monitoring`: Enables metrics and monitoring capabilities

## Integration with Other Crates

The app crate integrates:

1. **workflow-engine-core** (v0.6.0)
   - Provides core workflow execution engine
   - Authentication utilities (`JwtAuth`)
   - Error types and utilities
   - With `full` features enabled

2. **workflow-engine-api** (v0.6.0)
   - HTTP API endpoints and handlers
   - Database models and repositories
   - Middleware implementations
   - Monitoring and metrics
   - Bootstrap and service management
   - With `default` features enabled

## Runtime Behavior

### Middleware Stack

The application applies middleware in this order (innermost to outermost):

1. **Route Handlers** - Core business logic
2. **JWT Authentication** - Validates tokens for protected endpoints
3. **Rate Limiting** - Prevents API abuse
4. **CORS** - Handles cross-origin requests
5. **Logger** - Logs all requests/responses

### API Endpoints

The application exposes these endpoint groups:

- `/api/v1/health/*` - Health check endpoints (no auth required)
- `/api/v1/auth/*` - Authentication endpoints
- `/api/v1/workflows/*` - Workflow management
- `/api/v1/registry/*` - Service registry
- `/events` - Event creation endpoint
- `/metrics` - Prometheus metrics
- `/swagger-ui/` - OpenAPI documentation

### Error Handling

- Graceful error responses with appropriate HTTP status codes
- Correlation IDs for request tracing
- Structured error logging
- Database connection retry logic

## Common Development Tasks

### Running the Application

```bash
# Development mode with auto-reload
cargo watch -x "run --bin workflow-engine"

# Production build and run
cargo build --release --bin workflow-engine
./target/release/workflow-engine

# With specific log level
RUST_LOG=debug cargo run --bin workflow-engine

# With custom configuration
HOST=0.0.0.0 PORT=3000 cargo run --bin workflow-engine
```

### Testing the Binary

```bash
# Run unit tests for the app crate
cargo test --bin workflow-engine

# Integration test with real server
# Start the server in one terminal
cargo run --bin workflow-engine

# In another terminal, test endpoints
curl http://localhost:8080/api/v1/health
curl http://localhost:8080/metrics
```

### Adding New Configuration

1. Add environment variable reading in `main.rs`:
   ```rust
   let my_config = env::var("MY_CONFIG").unwrap_or_else(|_| "default".to_string());
   ```

2. Pass to middleware or app data:
   ```rust
   .app_data(web::Data::new(my_config))
   ```

### Modifying Startup Behavior

1. **Add startup task**: Insert between database init and server start
2. **Change middleware order**: Modify the `.wrap()` call order in `main.rs`
3. **Add demo workflows**: Uncomment lines 73-74 to run demos on startup

### Debugging Startup Issues

Common issues and solutions:

1. **Database connection fails**
   - Check `DATABASE_URL` is set correctly
   - Ensure PostgreSQL is running
   - Verify database exists

2. **Port already in use**
   - Change `PORT` environment variable
   - Kill existing process on port 8080

3. **JWT errors**
   - Ensure `JWT_SECRET` is set
   - Use a secure, random secret in production

### Production Deployment

#### 1. Environment Setup

```bash
# Create production environment file
cat > .env.production << EOF
# Database Configuration
DATABASE_URL=postgresql://workflow_user:secure_password@localhost:5432/workflow_prod_db
DB_MAX_CONNECTIONS=20
DB_CONNECTION_TIMEOUT=30

# Authentication
JWT_SECRET=$(openssl rand -hex 64)
JWT_EXPIRATION=3600
JWT_REFRESH_EXPIRATION=604800

# Server Configuration
HOST=0.0.0.0
PORT=8080

# Rate Limiting
RATE_LIMIT_PER_MINUTE=100
RATE_LIMIT_BURST=20

# Monitoring
METRICS_ENABLED=true
TRACING_ENABLED=true
LOG_LEVEL=info

# External Services
NOTION_MCP_URL=http://notion-mcp:8002
SLACK_MCP_URL=http://slack-mcp:8003
HELPSCOUT_MCP_URL=http://helpscout-mcp:8001

# AI Services (Optional)
OPENAI_API_KEY=your_openai_key
ANTHROPIC_API_KEY=your_anthropic_key
EOF
```

#### 2. Build and Package

```bash
# Build optimized release binary
cargo build --release --bin workflow-engine

# Verify binary
./target/release/workflow-engine --version

# Create deployment package
mkdir -p deploy/bin deploy/config
cp target/release/workflow-engine deploy/bin/
cp .env.production deploy/config/
cp -r config/ deploy/config/
```

#### 3. Docker Deployment

**Multi-stage Dockerfile for Production:**

```dockerfile
FROM rust:1.75-slim as builder
WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy source and build
COPY . .
RUN cargo build --release --bin workflow-engine

# Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false workflow

# Copy binary and set permissions
COPY --from=builder /app/target/release/workflow-engine /usr/local/bin/
RUN chmod +x /usr/local/bin/workflow-engine

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

# Run as non-root user
USER workflow
EXPOSE 8080

CMD ["workflow-engine"]
```

**Docker Compose for Production:**

```yaml
version: '3.8'

services:
  workflow-engine:
    build: .
    restart: unless-stopped
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgresql://workflow_user:${DB_PASSWORD}@postgres:5432/workflow_db
      - JWT_SECRET=${JWT_SECRET}
      - RUST_LOG=info
    depends_on:
      - postgres
      - redis
    networks:
      - workflow-net
    volumes:
      - ./logs:/app/logs
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  postgres:
    image: postgres:15-alpine
    restart: unless-stopped
    environment:
      POSTGRES_DB: workflow_db
      POSTGRES_USER: workflow_user
      POSTGRES_PASSWORD: ${DB_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./scripts/init-db.sql:/docker-entrypoint-initdb.d/init.sql
    networks:
      - workflow-net

  redis:
    image: redis:7-alpine
    restart: unless-stopped
    volumes:
      - redis_data:/data
    networks:
      - workflow-net

  # Monitoring stack
  prometheus:
    image: prom/prometheus:latest
    restart: unless-stopped
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus
    networks:
      - workflow-net

  grafana:
    image: grafana/grafana:latest
    restart: unless-stopped
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_PASSWORD}
    volumes:
      - grafana_data:/var/lib/grafana
      - ./monitoring/grafana:/etc/grafana/provisioning
    networks:
      - workflow-net

volumes:
  postgres_data:
  redis_data:
  prometheus_data:
  grafana_data:

networks:
  workflow-net:
    driver: bridge
```

#### 4. Kubernetes Deployment

**Namespace and ConfigMap:**

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: workflow-engine

---
apiVersion: v1
kind: ConfigMap
metadata:
  name: workflow-config
  namespace: workflow-engine
data:
  RUST_LOG: "info"
  HOST: "0.0.0.0"
  PORT: "8080"
  RATE_LIMIT_PER_MINUTE: "100"
  RATE_LIMIT_BURST: "20"
  METRICS_ENABLED: "true"
```

**Deployment and Service:**

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: workflow-engine
  namespace: workflow-engine
spec:
  replicas: 3
  selector:
    matchLabels:
      app: workflow-engine
  template:
    metadata:
      labels:
        app: workflow-engine
    spec:
      containers:
      - name: workflow-engine
        image: workflow-engine:latest
        ports:
        - containerPort: 8080
        envFrom:
        - configMapRef:
            name: workflow-config
        - secretRef:
            name: workflow-secrets
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health/detailed
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"

---
apiVersion: v1
kind: Service
metadata:
  name: workflow-engine-service
  namespace: workflow-engine
spec:
  selector:
    app: workflow-engine
  ports:
  - port: 80
    targetPort: 8080
  type: LoadBalancer
```

#### 5. Systemd Service

```ini
[Unit]
Description=AI Workflow Engine
Documentation=https://docs.rs/workflow-engine-app
After=network.target postgresql.service redis.service
Wants=postgresql.service redis.service

[Service]
Type=simple
User=workflow
Group=workflow
WorkingDirectory=/opt/workflow-engine
ExecStart=/opt/workflow-engine/bin/workflow-engine
ExecReload=/bin/kill -HUP $MAINPID

# Restart configuration
Restart=always
RestartSec=5
StartLimitInterval=60
StartLimitBurst=3

# Environment
Environment="RUST_LOG=info"
EnvironmentFile=/opt/workflow-engine/config/.env.production

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/workflow-engine/logs
PrivateTmp=true

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=workflow-engine

[Install]
WantedBy=multi-user.target
```

**Installation Script:**

```bash
#!/bin/bash
# install-workflow-engine.sh

set -e

# Create user
sudo useradd -r -s /bin/false workflow

# Create directories
sudo mkdir -p /opt/workflow-engine/{bin,config,logs}
sudo chown -R workflow:workflow /opt/workflow-engine

# Copy files
sudo cp deploy/bin/workflow-engine /opt/workflow-engine/bin/
sudo cp deploy/config/.env.production /opt/workflow-engine/config/
sudo cp workflow-engine.service /etc/systemd/system/

# Set permissions
sudo chmod +x /opt/workflow-engine/bin/workflow-engine
sudo chmod 600 /opt/workflow-engine/config/.env.production

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable workflow-engine
sudo systemctl start workflow-engine

# Verify installation
sleep 5
sudo systemctl status workflow-engine
curl -f http://localhost:8080/health
```

### Performance Tuning

1. **Database Connections**
   - Adjust pool size in `init_pool()` based on load
   - Monitor connection usage via metrics

2. **Rate Limiting**
   - Tune `RATE_LIMIT_PER_MINUTE` based on client needs
   - Adjust `RATE_LIMIT_BURST` for traffic patterns

3. **Server Workers**
   - Actix-web automatically uses multiple workers
   - Set `ACTIX_WORKERS` environment variable if needed

### Monitoring and Observability

1. **Metrics**: Available at `/metrics` endpoint
2. **Health Checks**: Use `/api/v1/health` for basic, `/api/v1/health/detailed` for comprehensive
3. **Logs**: Structured JSON logs with correlation IDs when `RUST_LOG` is set
4. **Uptime**: Check `/api/v1/uptime` for server runtime statistics

## Integration Points

The app crate serves as the integration point for:

- External HTTP clients via REST API
- Monitoring systems via Prometheus metrics
- Container orchestration via health checks
- Authentication systems via JWT tokens
- Database via connection pooling
- MCP servers via workflow execution