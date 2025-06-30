# Knowledge Graph Configuration Guide

## Overview

The Knowledge Graph Service uses environment variables and configuration files to manage its behavior. This guide covers all configuration options, their defaults, and best practices for different deployment scenarios.

## Environment Variables

### Core Service Configuration

```bash
# Service Port
SERVICE_PORT=3002
# Default: 3002
# The port on which the service listens for HTTP requests

# Service Name
SERVICE_NAME=knowledge-graph
# Default: knowledge-graph
# Used for logging, metrics, and service discovery

# Environment
ENVIRONMENT=development
# Options: development, staging, production
# Affects logging levels, security settings, and feature flags

# Log Level
LOG_LEVEL=info
# Options: trace, debug, info, warn, error
# Controls the verbosity of application logs

# Enable Metrics
ENABLE_METRICS=true
# Default: true
# Enables Prometheus metrics endpoint at /metrics
```

### Dgraph Configuration

```bash
# Dgraph Host
DGRAPH_HOST=localhost
# Default: localhost
# Hostname or IP address of Dgraph Alpha node

# Dgraph gRPC Port
DGRAPH_GRPC_PORT=9080
# Default: 9080
# Port for Dgraph gRPC communication

# Dgraph HTTP Port
DGRAPH_HTTP_PORT=8080
# Default: 8080
# Port for Dgraph HTTP API (GraphQL)

# Maximum Connections
DGRAPH_MAX_CONNECTIONS=20
# Default: 20
# Maximum number of connections in the pool

# Minimum Connections
DGRAPH_MIN_CONNECTIONS=5
# Default: 5
# Minimum number of idle connections to maintain

# Connection Idle Timeout (seconds)
DGRAPH_IDLE_TIMEOUT_SECS=300
# Default: 300 (5 minutes)
# Time before idle connections are closed

# Query Timeout (milliseconds)
DGRAPH_QUERY_TIMEOUT_MS=30000
# Default: 30000 (30 seconds)
# Maximum time for query execution

# Mutation Timeout (milliseconds)
DGRAPH_MUTATION_TIMEOUT_MS=60000
# Default: 60000 (60 seconds)
# Maximum time for mutation execution

# Health Check Interval (seconds)
DGRAPH_HEALTH_CHECK_INTERVAL_SECS=30
# Default: 30
# Interval between connection health checks

# Retry Attempts
DGRAPH_RETRY_ATTEMPTS=3
# Default: 3
# Number of retry attempts for failed operations

# Retry Delay (milliseconds)
DGRAPH_RETRY_DELAY_MS=1000
# Default: 1000
# Initial delay between retry attempts (exponential backoff)
```

### Redis Configuration

```bash
# Redis URL
REDIS_URL=redis://localhost:6379
# Default: redis://localhost:6379
# Full Redis connection URL including auth if needed

# Redis Pool Size
REDIS_POOL_SIZE=10
# Default: 10
# Maximum number of Redis connections

# Redis Connection Timeout (seconds)
REDIS_CONNECT_TIMEOUT_SECS=10
# Default: 10
# Timeout for establishing Redis connection

# Redis Command Timeout (seconds)
REDIS_COMMAND_TIMEOUT_SECS=5
# Default: 5
# Timeout for Redis command execution

# Cache TTL (seconds)
CACHE_DEFAULT_TTL_SECS=3600
# Default: 3600 (1 hour)
# Default time-to-live for cached items

# Cache Key Prefix
CACHE_KEY_PREFIX=kg:
# Default: kg:
# Prefix for all cache keys to avoid collisions
```

### PostgreSQL Configuration (for embeddings)

```bash
# Database URL
DATABASE_URL=postgresql://user:password@localhost:5432/knowledge_graph
# Required for vector similarity features
# Format: postgresql://[user[:password]@][host][:port][/dbname][?param1=value1&...]

# Database Pool Size
DATABASE_POOL_SIZE=20
# Default: 20
# Maximum number of database connections

# Database Connection Timeout (seconds)
DATABASE_CONNECT_TIMEOUT_SECS=30
# Default: 30
# Timeout for establishing database connection
```

### Algorithm Parameters

```bash
# PageRank Damping Factor
PAGERANK_DAMPING_FACTOR=0.85
# Default: 0.85
# Probability of following links vs. random jump

# PageRank Max Iterations
PAGERANK_MAX_ITERATIONS=100
# Default: 100
# Maximum iterations before stopping

# PageRank Convergence Tolerance
PAGERANK_TOLERANCE=0.000001
# Default: 1e-6
# Stop when rank changes are below this threshold

# A* Heuristic Weight
ASTAR_HEURISTIC_WEIGHT=0.5
# Default: 0.5
# Balance between cost and heuristic (0-1)

# Max Path Length
MAX_PATH_LENGTH=20
# Default: 20
# Maximum nodes in a learning path

# Alternative Paths Count
ALTERNATIVE_PATHS_COUNT=3
# Default: 3
# Number of alternative paths to generate

# Path Deviation Threshold
PATH_DEVIATION_THRESHOLD=0.3
# Default: 0.3
# Minimum difference for alternative paths
```

### Performance Tuning

```bash
# Worker Threads
WORKER_THREADS=4
# Default: Number of CPU cores
# Number of worker threads for async operations

# In-Memory Cache Size (MB)
MEMORY_CACHE_SIZE_MB=100
# Default: 100
# Size of in-memory LRU cache

# Query Batch Size
QUERY_BATCH_SIZE=100
# Default: 100
# Maximum items per batch query

# Enable Query Caching
ENABLE_QUERY_CACHE=true
# Default: true
# Cache frequently accessed queries

# Enable Result Streaming
ENABLE_STREAMING=true
# Default: true
# Stream large result sets instead of buffering
```

### Security Configuration

```bash
# JWT Secret
JWT_SECRET=your-secret-key-here
# Required in production
# Secret key for JWT token validation

# CORS Origins
CORS_ALLOWED_ORIGINS=http://localhost:3000,https://app.example.com
# Default: * (development only)
# Comma-separated list of allowed origins

# Rate Limit - Requests per Minute
RATE_LIMIT_RPM=100
# Default: 100
# Maximum requests per minute per IP

# Enable Authentication
ENABLE_AUTH=true
# Default: true (false in development)
# Require authentication for API access

# Admin API Key
ADMIN_API_KEY=your-admin-key
# Optional: Enable admin endpoints
# Key for administrative operations
```

### Monitoring Configuration

```bash
# Metrics Port
METRICS_PORT=9090
# Default: 9090
# Port for Prometheus metrics (if separate)

# Enable Tracing
ENABLE_TRACING=true
# Default: true
# Enable OpenTelemetry tracing

# Tracing Endpoint
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
# Default: http://localhost:4317
# OpenTelemetry collector endpoint

# Service Instance ID
SERVICE_INSTANCE_ID=kg-1
# Default: Generated UUID
# Unique identifier for this instance

# Health Check Path
HEALTH_CHECK_PATH=/health
# Default: /health
# Path for health check endpoint
```

## Configuration Files

### Service Configuration (config.toml)

```toml
[service]
name = "knowledge-graph"
version = "0.1.0"
port = 3002
workers = 4

[dgraph]
host = "localhost"
grpc_port = 9080
http_port = 8080

[dgraph.pool]
min_connections = 5
max_connections = 20
idle_timeout_secs = 300
health_check_interval_secs = 30

[dgraph.timeouts]
query_ms = 30000
mutation_ms = 60000
connect_ms = 10000

[redis]
url = "redis://localhost:6379"
pool_size = 10

[cache]
default_ttl_secs = 3600
max_memory_mb = 100
eviction_policy = "lru"

[algorithms]
pagerank_damping = 0.85
pagerank_iterations = 100
max_path_length = 20
alternative_paths = 3

[security]
enable_auth = true
cors_origins = ["http://localhost:3000"]
rate_limit_rpm = 100
```

### Logging Configuration (log4rs.yml)

```yaml
refresh_rate: 30 seconds

appenders:
  stdout:
    kind: console
    encoder:
      pattern: "{d(%Y-%m-%d %H:%M:%S)} {h({l})} {t} - {m}{n}"
      
  file:
    kind: rolling_file
    path: logs/knowledge-graph.log
    encoder:
      pattern: "{d(%Y-%m-%d %H:%M:%S)} [{l}] {t} - {m}{n}"
    policy:
      kind: compound
      trigger:
        kind: size
        limit: 10mb
      roller:
        kind: fixed_window
        pattern: logs/archive/knowledge-graph.{}.log
        count: 5

root:
  level: info
  appenders:
    - stdout
    - file

loggers:
  knowledge_graph:
    level: debug
    appenders:
      - stdout
      - file
    additive: false
    
  dgraph:
    level: warn
    
  redis:
    level: info
```

## Environment-Specific Configurations

### Development

```bash
# .env.development
ENVIRONMENT=development
LOG_LEVEL=debug
ENABLE_AUTH=false
CORS_ALLOWED_ORIGINS=*
DGRAPH_HOST=localhost
REDIS_URL=redis://localhost:6379
DATABASE_URL=postgresql://dev:devpass@localhost:5432/kg_dev
ENABLE_METRICS=true
ENABLE_TRACING=false
```

### Staging

```bash
# .env.staging
ENVIRONMENT=staging
LOG_LEVEL=info
ENABLE_AUTH=true
JWT_SECRET=${JWT_SECRET_STAGING}
CORS_ALLOWED_ORIGINS=https://staging.example.com
DGRAPH_HOST=dgraph-staging.internal
REDIS_URL=redis://redis-staging.internal:6379
DATABASE_URL=${DATABASE_URL_STAGING}
ENABLE_METRICS=true
ENABLE_TRACING=true
OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector.internal:4317
```

### Production

```bash
# .env.production
ENVIRONMENT=production
LOG_LEVEL=warn
ENABLE_AUTH=true
JWT_SECRET=${JWT_SECRET_PROD}
CORS_ALLOWED_ORIGINS=https://app.example.com,https://www.example.com
DGRAPH_HOST=dgraph-prod-lb.internal
DGRAPH_MAX_CONNECTIONS=50
REDIS_URL=redis://redis-cluster.internal:6379
DATABASE_URL=${DATABASE_URL_PROD}
ENABLE_METRICS=true
ENABLE_TRACING=true
OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector-prod.internal:4317
RATE_LIMIT_RPM=1000
WORKER_THREADS=8
```

## Performance Tuning Guide

### Connection Pool Optimization

```bash
# For high-traffic production
DGRAPH_MIN_CONNECTIONS=10
DGRAPH_MAX_CONNECTIONS=50
DGRAPH_IDLE_TIMEOUT_SECS=600

# For development
DGRAPH_MIN_CONNECTIONS=2
DGRAPH_MAX_CONNECTIONS=10
DGRAPH_IDLE_TIMEOUT_SECS=300
```

### Cache Configuration

```bash
# Memory-constrained environments
MEMORY_CACHE_SIZE_MB=50
CACHE_DEFAULT_TTL_SECS=1800

# High-memory environments
MEMORY_CACHE_SIZE_MB=500
CACHE_DEFAULT_TTL_SECS=7200
```

### Algorithm Tuning

```bash
# For faster responses (less accurate)
PAGERANK_MAX_ITERATIONS=50
PAGERANK_TOLERANCE=0.001
ALTERNATIVE_PATHS_COUNT=2

# For better accuracy (slower)
PAGERANK_MAX_ITERATIONS=200
PAGERANK_TOLERANCE=0.0000001
ALTERNATIVE_PATHS_COUNT=5
```

## Configuration Validation

The service validates configuration on startup:

```rust
// Example validation rules
fn validate_config(config: &Config) -> Result<()> {
    // Port range
    ensure!(config.port > 0 && config.port < 65536, "Invalid port");
    
    // Connection pool
    ensure!(
        config.dgraph.min_connections <= config.dgraph.max_connections,
        "Min connections must be <= max connections"
    );
    
    // Timeouts
    ensure!(
        config.dgraph.query_timeout_ms > 0,
        "Query timeout must be positive"
    );
    
    // Algorithm parameters
    ensure!(
        config.algorithms.pagerank_damping > 0.0 
        && config.algorithms.pagerank_damping < 1.0,
        "Damping factor must be between 0 and 1"
    );
    
    Ok(())
}
```

## Configuration Best Practices

### 1. Security

- Never commit secrets to version control
- Use environment variables for sensitive data
- Rotate JWT secrets regularly
- Use strong admin API keys

### 2. Performance

- Start with conservative connection pool settings
- Monitor and adjust based on metrics
- Enable caching for read-heavy workloads
- Use appropriate timeouts for your use case

### 3. Monitoring

- Always enable metrics in production
- Use structured logging with correlation IDs
- Set up alerts for configuration issues
- Track configuration changes

### 4. Deployment

- Use configuration management tools
- Validate configurations before deployment
- Keep environment-specific configs separate
- Document all custom configurations

## Dynamic Configuration

Some settings can be changed at runtime via admin API:

```bash
# Update cache TTL
curl -X PUT http://localhost:3002/admin/config \
  -H "Authorization: Bearer ${ADMIN_API_KEY}" \
  -H "Content-Type: application/json" \
  -d '{
    "cache_ttl_secs": 7200
  }'

# Update rate limits
curl -X PUT http://localhost:3002/admin/config \
  -H "Authorization: Bearer ${ADMIN_API_KEY}" \
  -H "Content-Type: application/json" \
  -d '{
    "rate_limit_rpm": 200
  }'
```

## Configuration Templates

### Docker Compose

```yaml
version: '3.8'
services:
  knowledge-graph:
    image: knowledge-graph:latest
    environment:
      - SERVICE_PORT=3002
      - DGRAPH_HOST=dgraph
      - REDIS_URL=redis://redis:6379
      - DATABASE_URL=postgresql://postgres:password@db:5432/knowledge_graph
    env_file:
      - .env
```

### Kubernetes ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: knowledge-graph-config
data:
  SERVICE_PORT: "3002"
  DGRAPH_HOST: "dgraph-service"
  DGRAPH_GRPC_PORT: "9080"
  REDIS_URL: "redis://redis-service:6379"
  LOG_LEVEL: "info"
  ENABLE_METRICS: "true"
```

### Kubernetes Secret

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: knowledge-graph-secrets
type: Opaque
stringData:
  JWT_SECRET: "your-secret-key"
  DATABASE_URL: "postgresql://user:pass@postgres:5432/kg"
  ADMIN_API_KEY: "admin-key"
```

## Troubleshooting Configuration Issues

### Common Problems

1. **Connection Pool Exhaustion**
   ```bash
   # Increase max connections
   DGRAPH_MAX_CONNECTIONS=50
   # Reduce idle timeout
   DGRAPH_IDLE_TIMEOUT_SECS=120
   ```

2. **Slow Queries**
   ```bash
   # Increase timeouts
   DGRAPH_QUERY_TIMEOUT_MS=60000
   # Enable query caching
   ENABLE_QUERY_CACHE=true
   ```

3. **Memory Issues**
   ```bash
   # Reduce cache size
   MEMORY_CACHE_SIZE_MB=50
   # Lower connection pool
   DGRAPH_MAX_CONNECTIONS=20
   ```

4. **Authentication Failures**
   ```bash
   # Check JWT secret
   JWT_SECRET=correct-secret
   # Verify CORS origins
   CORS_ALLOWED_ORIGINS=https://your-domain.com
   ```

### Debug Mode

Enable debug mode for detailed configuration logging:

```bash
LOG_LEVEL=trace
DEBUG_CONFIG=true
PRINT_CONFIG_ON_STARTUP=true
```

This will log all configuration values (excluding secrets) on startup.