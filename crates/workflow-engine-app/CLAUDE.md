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

1. **Environment Setup**
   ```bash
   # Create production .env file
   cp .env.example .env.production
   # Edit with production values
   ```

2. **Build Release Binary**
   ```bash
   cargo build --release --bin workflow-engine
   # Binary at: target/release/workflow-engine
   ```

3. **Docker Deployment**
   ```bash
   docker build -t workflow-engine-app .
   docker run -p 8080:8080 --env-file .env.production workflow-engine-app
   ```

4. **Systemd Service**
   ```ini
   [Unit]
   Description=AI Workflow Engine
   After=network.target postgresql.service

   [Service]
   Type=simple
   User=workflow
   WorkingDirectory=/opt/workflow-engine
   ExecStart=/opt/workflow-engine/workflow-engine
   Restart=always
   Environment="RUST_LOG=info"
   EnvironmentFile=/opt/workflow-engine/.env

   [Install]
   WantedBy=multi-user.target
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