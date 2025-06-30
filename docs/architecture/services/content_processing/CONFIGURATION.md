# Content Processing Service Configuration Guide

## Overview

The Content Processing Service is highly configurable through environment variables, configuration files, and runtime parameters. This guide covers all configuration options and best practices for different deployment scenarios.

## Environment Variables

### Core Settings

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `PORT` | HTTP server port | `8082` | `8082` |
| `HOST` | Server bind address | `0.0.0.0` | `127.0.0.1` |
| `RUST_LOG` | Log level | `info` | `debug,content_processing=trace` |
| `ENVIRONMENT` | Deployment environment | `development` | `production` |

### Database Configuration

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `DATABASE_URL` | PostgreSQL connection URL | Required | `postgresql://user:pass@localhost/content_db` |
| `DATABASE_MAX_CONNECTIONS` | Max DB connections | `20` | `50` |
| `DATABASE_MIN_CONNECTIONS` | Min DB connections | `5` | `10` |
| `DATABASE_CONNECT_TIMEOUT` | Connection timeout (seconds) | `30` | `60` |
| `DATABASE_IDLE_TIMEOUT` | Idle connection timeout (seconds) | `600` | `300` |
| `DATABASE_MAX_LIFETIME` | Max connection lifetime (seconds) | `1800` | `3600` |

### Redis Configuration

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `REDIS_URL` | Redis connection URL | `redis://localhost:6379` | `redis://:password@redis:6379/0` |
| `REDIS_MAX_CONNECTIONS` | Max Redis connections | `10` | `20` |
| `REDIS_CONNECT_TIMEOUT` | Connection timeout (ms) | `5000` | `10000` |
| `REDIS_READ_TIMEOUT` | Read timeout (ms) | `3000` | `5000` |
| `REDIS_WRITE_TIMEOUT` | Write timeout (ms) | `3000` | `5000` |

### Performance Settings

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `WORKER_THREADS` | Number of worker threads | CPU count | `8` |
| `MAX_CONTENT_SIZE` | Max content size (bytes) | `10485760` (10MB) | `52428800` (50MB) |
| `MAX_PROCESSING_TIME` | Max processing time (seconds) | `30` | `60` |
| `MAX_CONCURRENT_JOBS` | Max concurrent processing jobs | `100` | `200` |
| `QUEUE_SIZE` | Internal job queue size | `1000` | `5000` |
| `BATCH_SIZE` | Batch processing size | `100` | `250` |

### Plugin Configuration

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `PLUGIN_DIR` | Plugin directory path | `/app/plugins` | `/opt/plugins` |
| `PLUGIN_MEMORY_LIMIT` | Memory limit per plugin (MB) | `10` | `50` |
| `PLUGIN_TIMEOUT` | Plugin execution timeout (seconds) | `10` | `30` |
| `PLUGIN_MAX_INSTANCES` | Max plugin instances | `10` | `20` |
| `PLUGIN_CACHE_SIZE` | Plugin result cache size | `1000` | `5000` |
| `PLUGIN_AUTO_RELOAD` | Auto-reload plugins on change | `false` | `true` |

### Security Settings

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `ENABLE_AUTH` | Enable authentication | `false` | `true` |
| `JWT_SECRET` | JWT signing secret | Required if auth enabled | `your-secret-key` |
| `API_RATE_LIMIT` | Requests per minute | `100` | `500` |
| `API_BURST_LIMIT` | Burst requests allowed | `10` | `50` |
| `ALLOWED_ORIGINS` | CORS allowed origins | `*` | `https://app.example.com` |
| `ENABLE_TLS` | Enable TLS/HTTPS | `false` | `true` |
| `TLS_CERT_PATH` | TLS certificate path | - | `/certs/cert.pem` |
| `TLS_KEY_PATH` | TLS key path | - | `/certs/key.pem` |

### Monitoring Configuration

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `METRICS_ENABLED` | Enable Prometheus metrics | `true` | `true` |
| `METRICS_PATH` | Metrics endpoint path | `/metrics` | `/admin/metrics` |
| `TRACING_ENABLED` | Enable OpenTelemetry tracing | `false` | `true` |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | OpenTelemetry endpoint | - | `http://otel:4317` |
| `OTEL_SERVICE_NAME` | Service name for tracing | `content_processing` | `content_processing_prod` |
| `HEALTH_CHECK_INTERVAL` | Health check interval (seconds) | `30` | `60` |

## Configuration File

For complex configurations, use a YAML configuration file:

```yaml
# config.yaml
server:
  host: 0.0.0.0
  port: 8082
  workers: 8
  
database:
  url: postgresql://user:pass@localhost/content_db
  pool:
    max_connections: 20
    min_connections: 5
    connect_timeout: 30s
    idle_timeout: 10m
    max_lifetime: 30m
    
redis:
  url: redis://localhost:6379
  pool:
    max_connections: 10
    timeout: 5s
    
processing:
  max_content_size: 10MB
  max_processing_time: 30s
  max_concurrent_jobs: 100
  batch_size: 100
  
plugins:
  directory: /app/plugins
  memory_limit: 10MB
  timeout: 10s
  auto_reload: false
  
security:
  enable_auth: false
  rate_limit:
    requests_per_minute: 100
    burst: 10
  cors:
    allowed_origins:
      - https://app.example.com
      - https://admin.example.com
      
monitoring:
  metrics:
    enabled: true
    path: /metrics
  tracing:
    enabled: true
    endpoint: http://otel:4317
  logging:
    level: info
    format: json
```

Load configuration file:

```bash
CONFIG_FILE=/path/to/config.yaml cargo run
```

## Performance Tuning

### Database Optimization

```bash
# High-traffic production settings
DATABASE_MAX_CONNECTIONS=100
DATABASE_MIN_CONNECTIONS=20
DATABASE_CONNECT_TIMEOUT=10
DATABASE_IDLE_TIMEOUT=300
DATABASE_MAX_LIFETIME=1800

# Enable statement caching
DATABASE_STATEMENT_CACHE_SIZE=100
```

### Redis Optimization

```bash
# Production Redis settings
REDIS_MAX_CONNECTIONS=50
REDIS_CONNECT_TIMEOUT=2000
REDIS_READ_TIMEOUT=1000
REDIS_WRITE_TIMEOUT=1000

# Enable connection pooling
REDIS_POOL_MIN_IDLE=10
REDIS_POOL_MAX_IDLE=30
```

### Worker Thread Tuning

```bash
# CPU-bound workloads
WORKER_THREADS=16  # 2x CPU cores

# I/O-bound workloads
WORKER_THREADS=32  # 4x CPU cores

# Mixed workloads
WORKER_THREADS=24  # 3x CPU cores
```

### Memory Configuration

```bash
# Large document processing
MAX_CONTENT_SIZE=52428800  # 50MB
PLUGIN_MEMORY_LIMIT=100    # 100MB per plugin

# Memory-constrained environment
MAX_CONTENT_SIZE=5242880   # 5MB
PLUGIN_MEMORY_LIMIT=5      # 5MB per plugin
```

## Security Configuration

### Authentication Setup

```bash
# Enable JWT authentication
ENABLE_AUTH=true
JWT_SECRET=$(openssl rand -base64 32)
JWT_EXPIRY=3600  # 1 hour
JWT_REFRESH_EXPIRY=86400  # 24 hours

# OAuth2 integration
OAUTH_PROVIDER=https://auth.example.com
OAUTH_CLIENT_ID=content-processing
OAUTH_CLIENT_SECRET=secret
```

### TLS/HTTPS Configuration

```bash
# Enable HTTPS
ENABLE_TLS=true
TLS_CERT_PATH=/certs/fullchain.pem
TLS_KEY_PATH=/certs/privkey.pem
TLS_MIN_VERSION=1.2
TLS_CIPHERS=ECDHE-RSA-AES128-GCM-SHA256:ECDHE-RSA-AES256-GCM-SHA384
```

### Rate Limiting

```bash
# API rate limiting
API_RATE_LIMIT=1000      # 1000 requests per minute
API_BURST_LIMIT=50       # Allow bursts up to 50
RATE_LIMIT_BY=ip         # Rate limit by IP address

# Per-user rate limiting (requires auth)
USER_RATE_LIMIT=500      # 500 requests per minute per user
USER_BURST_LIMIT=25      # User burst limit
```

## Logging Configuration

### Log Levels

```bash
# Development logging
RUST_LOG=debug,content_processing=trace

# Production logging
RUST_LOG=info,content_processing=info,sqlx=warn

# Troubleshooting specific modules
RUST_LOG=info,content_processing::plugins=debug,content_processing::api=trace
```

### Log Formats

```bash
# JSON logs for production
LOG_FORMAT=json

# Pretty logs for development
LOG_FORMAT=pretty

# Compact logs
LOG_FORMAT=compact
```

### Log Output

```bash
# Log to file
LOG_OUTPUT=file
LOG_FILE_PATH=/var/log/content_processing.log
LOG_FILE_ROTATION=daily
LOG_FILE_MAX_SIZE=100MB
LOG_FILE_MAX_BACKUPS=7

# Log to stdout (default)
LOG_OUTPUT=stdout

# Log to syslog
LOG_OUTPUT=syslog
SYSLOG_HOST=syslog.example.com:514
```

## Plugin Configuration

### Plugin Discovery

```bash
# Plugin directory structure
PLUGIN_DIR=/app/plugins
PLUGIN_DISCOVERY=auto     # Auto-discover plugins
PLUGIN_WHITELIST=sentiment,readability  # Only load specific plugins
PLUGIN_BLACKLIST=experimental  # Exclude specific plugins
```

### Plugin Resources

```bash
# Per-plugin resource limits
PLUGIN_MEMORY_LIMIT=50    # 50MB per plugin
PLUGIN_CPU_SHARES=100     # CPU shares (relative weight)
PLUGIN_TIMEOUT=30         # 30 second timeout
PLUGIN_MAX_INSTANCES=5    # Max 5 concurrent instances
```

### Plugin Configuration

```bash
# Plugin-specific configuration
PLUGIN_CONFIG_sentiment='{"model":"advanced","threshold":0.7}'
PLUGIN_CONFIG_readability='{"algorithm":"flesch","target_level":"high_school"}'
```

## Cache Configuration

### Result Caching

```bash
# Cache configuration
CACHE_ENABLED=true
CACHE_TTL=3600           # 1 hour TTL
CACHE_MAX_SIZE=1000      # Max 1000 entries
CACHE_EVICTION=lru       # LRU eviction policy

# Cache key configuration
CACHE_KEY_PREFIX=content_proc
CACHE_KEY_HASH_ALGO=xxhash64
```

### Plugin Cache

```bash
# Plugin result caching
PLUGIN_CACHE_ENABLED=true
PLUGIN_CACHE_TTL=1800    # 30 minutes
PLUGIN_CACHE_MAX_SIZE=500
```

## Deployment Configurations

### Development

```bash
# .env.development
ENVIRONMENT=development
PORT=8082
DATABASE_URL=postgresql://dev:dev@localhost/content_dev
REDIS_URL=redis://localhost:6379
RUST_LOG=debug
PLUGIN_AUTO_RELOAD=true
ENABLE_AUTH=false
```

### Staging

```bash
# .env.staging
ENVIRONMENT=staging
PORT=8082
DATABASE_URL=postgresql://stage_user:stage_pass@db-staging/content_staging
REDIS_URL=redis://:stage_pass@redis-staging:6379
RUST_LOG=info
METRICS_ENABLED=true
ENABLE_AUTH=true
JWT_SECRET=${STAGING_JWT_SECRET}
```

### Production

```bash
# .env.production
ENVIRONMENT=production
PORT=8082
DATABASE_URL=${PROD_DATABASE_URL}
REDIS_URL=${PROD_REDIS_URL}
RUST_LOG=warn,content_processing=info
WORKER_THREADS=16
MAX_CONCURRENT_JOBS=500
METRICS_ENABLED=true
TRACING_ENABLED=true
ENABLE_AUTH=true
JWT_SECRET=${PROD_JWT_SECRET}
ENABLE_TLS=true
TLS_CERT_PATH=/certs/cert.pem
TLS_KEY_PATH=/certs/key.pem
```

## Kubernetes ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: content-processing-config
  namespace: default
data:
  PORT: "8082"
  RUST_LOG: "info"
  DATABASE_MAX_CONNECTIONS: "50"
  REDIS_MAX_CONNECTIONS: "20"
  WORKER_THREADS: "16"
  MAX_CONTENT_SIZE: "10485760"
  METRICS_ENABLED: "true"
  PLUGIN_DIR: "/app/plugins"
```

## Docker Compose Configuration

```yaml
version: '3.8'

services:
  content-processing:
    image: content-processing:latest
    environment:
      - PORT=8082
      - DATABASE_URL=postgresql://user:pass@postgres/content_db
      - REDIS_URL=redis://redis:6379
      - RUST_LOG=info
      - WORKER_THREADS=8
      - MAX_CONCURRENT_JOBS=100
    env_file:
      - .env.production
    volumes:
      - ./plugins:/app/plugins:ro
      - ./config.yaml:/app/config.yaml:ro
    command: ["--config", "/app/config.yaml"]
```

## Health Check Configuration

```bash
# Health check settings
HEALTH_CHECK_INTERVAL=30     # Check every 30 seconds
HEALTH_CHECK_TIMEOUT=5       # 5 second timeout
HEALTH_CHECK_DB=true         # Check database connection
HEALTH_CHECK_REDIS=true      # Check Redis connection
HEALTH_CHECK_PLUGINS=true    # Check plugin system
```

## Troubleshooting Configuration Issues

### Debug Configuration Loading

```bash
# Enable configuration debug logging
CONFIG_DEBUG=true
RUST_LOG=debug,content_processing::config=trace
```

### Validate Configuration

```bash
# Dry run to validate configuration
cargo run -- --validate-config

# Test specific configuration
cargo run -- --test-config database
cargo run -- --test-config redis
cargo run -- --test-config plugins
```

### Common Issues

1. **Database connection failures**
   ```bash
   # Check connection string
   DATABASE_URL=postgresql://user:pass@host:5432/dbname?sslmode=require
   
   # Increase timeouts
   DATABASE_CONNECT_TIMEOUT=60
   ```

2. **Redis connection issues**
   ```bash
   # Check Redis URL format
   REDIS_URL=redis://username:password@hostname:6379/0
   
   # Disable TLS if not needed
   REDIS_TLS_ENABLED=false
   ```

3. **Plugin loading failures**
   ```bash
   # Check plugin directory permissions
   ls -la $PLUGIN_DIR
   
   # Enable plugin debug logging
   RUST_LOG=debug,content_processing::plugins=trace
   ```

## Best Practices

1. **Use environment-specific files**: Separate configurations for dev/staging/prod
2. **Secure sensitive data**: Use secrets management for passwords and keys
3. **Monitor configuration changes**: Log configuration values at startup
4. **Validate before deployment**: Test configuration in staging first
5. **Document custom settings**: Keep README updated with custom configurations
6. **Use reasonable defaults**: Set sensible defaults for optional settings
7. **Plan for scaling**: Configure for expected load with headroom