# Configuration Guide

## Overview

The Real-time Communication Service is highly configurable to meet different deployment requirements. This guide covers all configuration options, from basic setup to advanced tuning for production environments.

## Environment Variables

### Core Configuration

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `HOST` | Server bind address | `0.0.0.0` | No |
| `PORT` | Server port | `8081` | No |
| `RUST_LOG` | Logging level | `info` | No |
| `WORKERS` | Number of worker threads | CPU count | No |

### Authentication Configuration

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `JWT_SECRET` | Secret key for JWT signing (min 32 bytes) | - | **Yes** |
| `JWT_ISSUER` | JWT token issuer | `ai-system-rust` | No |
| `JWT_AUDIENCE` | JWT token audience | `realtime-communication` | No |
| `JWT_EXPIRY_HOURS` | Access token expiry in hours | `24` | No |
| `JWT_REFRESH_EXPIRY_DAYS` | Refresh token expiry in days | `30` | No |

### WebSocket Configuration

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `MAX_CONNECTIONS` | Maximum concurrent connections | `10000` | No |
| `HEARTBEAT_INTERVAL_SECS` | Heartbeat interval in seconds | `30` | No |
| `CLIENT_TIMEOUT_SECS` | Client inactivity timeout | `60` | No |
| `MAX_FRAME_SIZE` | Maximum WebSocket frame size | `65536` | No |
| `MAX_MESSAGE_SIZE` | Maximum message size in bytes | `1048576` | No |

### Redis Configuration

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `REDIS_URL` | Redis connection URL | `redis://localhost:6379` | No |
| `REDIS_POOL_SIZE` | Connection pool size | `10` | No |
| `REDIS_TIMEOUT_SECS` | Connection timeout | `5` | No |
| `REDIS_RETRY_ATTEMPTS` | Retry attempts on failure | `3` | No |
| `REDIS_KEY_PREFIX` | Key prefix for namespacing | `rtc:` | No |

### Rate Limiting Configuration

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `RATE_LIMIT_ENABLED` | Enable rate limiting | `true` | No |
| `RATE_LIMIT_CONNECTION_RPS` | Per-connection requests/second | `100` | No |
| `RATE_LIMIT_CONNECTION_BURST` | Per-connection burst size | `200` | No |
| `RATE_LIMIT_USER_RPS` | Per-user requests/second | `500` | No |
| `RATE_LIMIT_USER_BURST` | Per-user burst size | `1000` | No |
| `RATE_LIMIT_GLOBAL_RPS` | Global requests/second | `10000` | No |
| `RATE_LIMIT_GLOBAL_BURST` | Global burst size | `20000` | No |

### Circuit Breaker Configuration

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `CIRCUIT_BREAKER_ENABLED` | Enable circuit breaker | `true` | No |
| `CIRCUIT_BREAKER_FAILURE_THRESHOLD` | Failures before opening | `5` | No |
| `CIRCUIT_BREAKER_SUCCESS_THRESHOLD` | Successes to close | `2` | No |
| `CIRCUIT_BREAKER_TIMEOUT_SECS` | Timeout in open state | `60` | No |
| `CIRCUIT_BREAKER_HALF_OPEN_REQUESTS` | Requests in half-open state | `3` | No |

## Configuration File

Create a `config.toml` file for more complex configurations:

```toml
# config.toml

[server]
host = "0.0.0.0"
port = 8081
workers = 0  # 0 means use all CPU cores

[websocket]
max_connections = 10000
heartbeat_interval_secs = 30
client_timeout_secs = 60
max_frame_size = 65536
max_message_size = 1048576

[auth]
jwt_secret = "${JWT_SECRET}"  # Load from environment
jwt_issuer = "ai-system-rust"
jwt_audience = "realtime-communication"
access_token_expiry_hours = 24
refresh_token_expiry_days = 30
validate_issuer = true
validate_audience = true
validate_exp = true
validate_nbf = true

[redis]
url = "${REDIS_URL:-redis://localhost:6379}"
pool_size = 10
timeout_secs = 5
retry_attempts = 3
retry_delay_ms = 100
key_prefix = "rtc:"
enable_cluster = false

[rate_limiting]
enabled = true

[rate_limiting.connection]
requests_per_second = 100.0
burst_size = 200
window_size_secs = 1

[rate_limiting.user]
requests_per_second = 500.0
burst_size = 1000
window_size_secs = 1

[rate_limiting.global]
requests_per_second = 10000.0
burst_size = 20000
window_size_secs = 1

[circuit_breaker]
enabled = true
failure_threshold = 5
success_threshold = 2
timeout_secs = 60
half_open_max_requests = 3

[logging]
level = "info"
format = "json"  # "json" or "pretty"
enable_telemetry = true
enable_metrics = true

[metrics]
enabled = true
port = 9090
path = "/metrics"
update_interval_secs = 10

[tracing]
enabled = true
service_name = "realtime-communication"
sampling_rate = 0.1
exporter = "otlp"  # "otlp" or "jaeger"
endpoint = "http://localhost:4317"

[message_routing]
enable_caching = true
cache_ttl_secs = 300
max_cache_entries = 10000
max_targets_per_message = 1000
enable_message_transformation = true
drop_invalid_messages = false

[actor_system]
mailbox_capacity = 1000
enable_priority_mailbox = true
overflow_strategy = "backpressure"  # "drop_oldest", "drop_newest", "block", "backpressure"
supervision_strategy = "one_for_one"  # "one_for_one", "all_for_one", "rest_for_one"
max_restarts = 3
restart_window_secs = 60

[tls]
enabled = false
cert_file = "/path/to/cert.pem"
key_file = "/path/to/key.pem"
ca_file = "/path/to/ca.pem"
verify_client = false
```

## JWT Configuration

### JWT Secret Generation

Generate a secure JWT secret:

```bash
# Generate 32-byte (256-bit) secret
openssl rand -base64 32

# Or using /dev/urandom
head -c 32 /dev/urandom | base64

# Set in environment
export JWT_SECRET="your-generated-secret-here"
```

### JWT Token Structure

```json
{
  "header": {
    "alg": "HS256",
    "typ": "JWT"
  },
  "payload": {
    "sub": "user_123",
    "iat": 1234567890,
    "exp": 1234654290,
    "nbf": 1234567890,
    "iss": "ai-system-rust",
    "aud": "realtime-communication",
    "user_id": "user_123",
    "session_id": "session_456",
    "roles": ["user", "subscriber"],
    "permissions": ["read", "write", "subscribe"],
    "refresh_threshold": 1234611490
  }
}
```

### Custom Claims Configuration

```rust
// Custom claims can be added via configuration
pub struct CustomClaimsConfig {
    pub include_user_metadata: bool,
    pub include_device_info: bool,
    pub include_location: bool,
    pub custom_fields: HashMap<String, String>,
}
```

## Redis Connection Settings

### Single Instance Configuration

```toml
[redis]
url = "redis://username:password@localhost:6379/0"
connection_timeout_ms = 5000
response_timeout_ms = 1000
max_retry_attempts = 3
```

### Redis Cluster Configuration

```toml
[redis.cluster]
enabled = true
nodes = [
    "redis://node1:6379",
    "redis://node2:6379",
    "redis://node3:6379"
]
read_from_replicas = true
```

### Redis Sentinel Configuration

```toml
[redis.sentinel]
enabled = true
master_name = "mymaster"
sentinels = [
    "sentinel1:26379",
    "sentinel2:26379",
    "sentinel3:26379"
]
password = "${REDIS_PASSWORD}"
```

### Connection Pool Tuning

```toml
[redis.pool]
min_idle = 5
max_size = 20
connection_timeout_ms = 5000
idle_timeout_secs = 300
max_lifetime_secs = 3600
```

## Rate Limiting Parameters

### Token Bucket Configuration

```toml
[rate_limiting.token_bucket]
refill_interval_ms = 100
token_precision = 0.1
cleanup_interval_secs = 300
cleanup_threshold_secs = 600
```

### Custom Rate Limits

```toml
# Define custom rate limits for specific endpoints or operations
[[rate_limiting.custom_limits]]
name = "subscription"
pattern = "Subscribe"
requests_per_second = 10.0
burst_size = 20

[[rate_limiting.custom_limits]]
name = "broadcast"
pattern = "Broadcast"
requests_per_second = 50.0
burst_size = 100

[[rate_limiting.custom_limits]]
name = "direct_message"
pattern = "DirectMessage"
requests_per_second = 200.0
burst_size = 400
```

### Rate Limit Headers

Configure rate limit headers in responses:

```toml
[rate_limiting.headers]
enabled = true
limit_header = "X-RateLimit-Limit"
remaining_header = "X-RateLimit-Remaining"
reset_header = "X-RateLimit-Reset"
retry_after_header = "Retry-After"
```

## Circuit Breaker Settings

### Service-Specific Circuit Breakers

```toml
[[circuit_breaker.services]]
name = "redis"
failure_threshold = 5
success_threshold = 2
timeout_secs = 30
half_open_requests = 1

[[circuit_breaker.services]]
name = "auth_service"
failure_threshold = 3
success_threshold = 1
timeout_secs = 60
half_open_requests = 2

[[circuit_breaker.services]]
name = "message_queue"
failure_threshold = 10
success_threshold = 3
timeout_secs = 120
half_open_requests = 5
```

### Circuit Breaker Events

```toml
[circuit_breaker.events]
log_state_changes = true
emit_metrics = true
webhook_url = "https://alerts.example.com/circuit-breaker"
```

## Performance Tuning

### Actor System Tuning

```toml
[performance.actors]
default_mailbox_size = 1000
priority_mailbox_size = 100
batch_size = 50
batch_timeout_ms = 10
use_work_stealing = true
pin_actors_to_cores = false
```

### Message Processing

```toml
[performance.messages]
max_batch_size = 100
batch_window_ms = 5
compression_threshold_bytes = 1024
compression_algorithm = "zstd"  # "none", "gzip", "zstd", "lz4"
```

### Memory Configuration

```toml
[performance.memory]
# Preallocate memory pools
connection_pool_size = 10000
message_pool_size = 100000
buffer_pool_size = 50000

# Garbage collection tuning
gc_interval_secs = 300
gc_threshold_mb = 1024
```

## Security Configuration

### CORS Settings

```toml
[security.cors]
enabled = true
allowed_origins = ["https://app.example.com", "https://admin.example.com"]
allowed_methods = ["GET", "POST", "OPTIONS"]
allowed_headers = ["Authorization", "Content-Type"]
allow_credentials = true
max_age_secs = 86400
```

### TLS/SSL Configuration

```toml
[security.tls]
enabled = true
cert_file = "/etc/certs/server.crt"
key_file = "/etc/certs/server.key"
ca_file = "/etc/certs/ca.crt"
min_version = "1.2"  # "1.2" or "1.3"
ciphers = [
    "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384",
    "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256"
]
verify_client = false
client_auth = "optional"  # "none", "optional", "required"
```

### Authentication Providers

```toml
[[auth.providers]]
name = "jwt"
type = "jwt"
enabled = true
priority = 1

[[auth.providers]]
name = "api_key"
type = "api_key"
enabled = false
header = "X-API-Key"
query_param = "api_key"

[[auth.providers]]
name = "oauth2"
type = "oauth2"
enabled = false
issuer = "https://auth.example.com"
audience = "realtime-api"
```

## Monitoring Configuration

### Prometheus Metrics

```toml
[monitoring.prometheus]
enabled = true
port = 9090
path = "/metrics"
namespace = "realtime_comm"
subsystem = "websocket"
buckets = [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
```

### Custom Metrics

```toml
[[monitoring.custom_metrics]]
name = "message_latency"
type = "histogram"
help = "Message delivery latency in seconds"
labels = ["message_type", "topic"]

[[monitoring.custom_metrics]]
name = "active_subscriptions"
type = "gauge"
help = "Number of active topic subscriptions"
labels = ["topic"]
```

### Health Check Configuration

```toml
[monitoring.health_check]
enabled = true
path = "/health"
detailed = true
check_redis = true
check_dependencies = true
timeout_secs = 5
```

## Development Configuration

### Development Profile

```toml
# config.dev.toml
[server]
host = "localhost"
port = 8081

[websocket]
max_connections = 100
heartbeat_interval_secs = 60
client_timeout_secs = 300

[logging]
level = "debug"
format = "pretty"

[development]
enable_debug_endpoints = true
enable_profiling = true
mock_external_services = true
```

### Testing Configuration

```toml
# config.test.toml
[server]
host = "127.0.0.1"
port = 0  # Random port

[testing]
enable_test_mode = true
bypass_auth = true
use_in_memory_storage = true
deterministic_ids = true
```

## Production Configuration

### Production Profile

```toml
# config.prod.toml
[server]
host = "0.0.0.0"
port = 8081
workers = 0  # Use all cores

[websocket]
max_connections = 10000
heartbeat_interval_secs = 30
client_timeout_secs = 60

[logging]
level = "warn"
format = "json"

[security]
enforce_tls = true
strict_transport_security = true
content_security_policy = "default-src 'self'"

[production]
enable_graceful_shutdown = true
shutdown_timeout_secs = 30
drain_timeout_secs = 60
```

## Configuration Loading Order

1. Default values (hardcoded)
2. Configuration file (`config.toml`)
3. Profile-specific file (`config.{profile}.toml`)
4. Environment variables
5. Command-line arguments

```bash
# Example with different profiles
./realtime_communication --profile=dev --config=/etc/rtc/config.toml

# Override specific values
./realtime_communication --port=8082 --max-connections=5000

# Using environment variables
RUST_LOG=debug PORT=8082 ./realtime_communication
```

## Configuration Validation

The service validates configuration on startup:

```rust
pub struct ConfigValidator {
    pub fn validate(config: &Config) -> Result<(), Vec<ConfigError>> {
        let mut errors = Vec::new();
        
        // Validate JWT secret length
        if config.auth.jwt_secret.len() < 32 {
            errors.push(ConfigError::InvalidJwtSecret);
        }
        
        // Validate port range
        if config.server.port == 0 || config.server.port > 65535 {
            errors.push(ConfigError::InvalidPort);
        }
        
        // Validate rate limits
        if config.rate_limiting.connection.requests_per_second <= 0.0 {
            errors.push(ConfigError::InvalidRateLimit);
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
```

## Dynamic Configuration

Some settings can be updated at runtime:

```bash
# Update rate limits
curl -X PUT http://localhost:8081/admin/config/rate-limits \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "connection_rps": 200,
    "user_rps": 1000,
    "global_rps": 20000
  }'

# Update log level
curl -X PUT http://localhost:8081/admin/config/log-level \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"level": "debug"}'
```