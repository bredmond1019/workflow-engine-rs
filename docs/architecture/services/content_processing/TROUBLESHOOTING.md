# Content Processing Service Troubleshooting Guide

## Overview

This guide helps diagnose and resolve common issues with the Content Processing Service. It covers error patterns, debugging techniques, and solutions for frequent problems.

## Quick Diagnostics

### Health Check

```bash
# Basic health check
curl http://localhost:8082/health

# Detailed health check with component status
curl http://localhost:8082/health/detailed
```

Expected healthy response:
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "checks": {
    "database": {"status": "up", "latency_ms": 2},
    "redis": {"status": "up", "latency_ms": 1},
    "plugins": {"status": "up", "loaded_count": 5}
  }
}
```

### Quick Debug Commands

```bash
# Check service logs
docker logs content-processing --tail 100

# Check database connectivity
psql $DATABASE_URL -c "SELECT 1"

# Check Redis connectivity
redis-cli -u $REDIS_URL ping

# Check disk space
df -h /app/plugins /tmp

# Check memory usage
docker stats content-processing
```

## Common Issues and Solutions

### 1. Service Won't Start

#### Symptom
Service fails to start or crashes immediately after starting.

#### Diagnosis
```bash
# Check logs for startup errors
docker logs content-processing | grep -E "ERROR|PANIC"

# Check environment variables
docker exec content-processing env | grep -E "DATABASE_URL|REDIS_URL"

# Check file permissions
docker exec content-processing ls -la /app/
```

#### Common Causes and Solutions

**Missing environment variables:**
```bash
# Ensure required variables are set
export DATABASE_URL="postgresql://user:pass@localhost/content_db"
export REDIS_URL="redis://localhost:6379"
```

**Database connection failure:**
```bash
# Test database connection
psql $DATABASE_URL -c "\conninfo"

# Check if pgvector extension is installed
psql $DATABASE_URL -c "SELECT * FROM pg_extension WHERE extname = 'vector';"

# Install pgvector if missing
psql $DATABASE_URL -c "CREATE EXTENSION IF NOT EXISTS vector;"
```

**Port already in use:**
```bash
# Find process using port 8082
lsof -i :8082

# Use different port
export PORT=8083
```

### 2. Processing Timeouts

#### Symptom
Requests timeout or take too long to process.

#### Diagnosis
```bash
# Check processing metrics
curl http://localhost:8082/metrics | grep processing_duration

# Monitor active jobs
curl http://localhost:8082/metrics | grep active_processing_jobs

# Check system resources
top -p $(pgrep content_processing)
```

#### Solutions

**Increase timeout settings:**
```bash
# Increase processing timeout
export MAX_PROCESSING_TIME=60

# Increase plugin timeout
export PLUGIN_TIMEOUT=30
```

**Optimize worker configuration:**
```bash
# Increase worker threads for CPU-bound tasks
export WORKER_THREADS=16

# Increase concurrent jobs
export MAX_CONCURRENT_JOBS=200
```

**Check for memory issues:**
```bash
# Monitor memory usage
watch -n 1 'docker stats content-processing --no-stream'

# Increase memory limits
docker update --memory="4g" content-processing
```

### 3. Plugin Loading Issues

#### Symptom
Plugins fail to load or execute.

#### Diagnosis
```bash
# List loaded plugins
curl http://localhost:8082/plugins

# Check plugin directory
docker exec content-processing ls -la /app/plugins/

# Check plugin logs
docker logs content-processing | grep -i plugin
```

#### Common Issues

**Invalid WASM module:**
```bash
# Validate WASM file
file /app/plugins/my_plugin.wasm

# Check WASM exports
wasm-objdump -x /app/plugins/my_plugin.wasm | grep Export
```

**Missing plugin dependencies:**
```bash
# Enable plugin debug mode
export RUST_LOG=debug,content_processing::plugins=trace

# Check plugin metadata
curl http://localhost:8082/plugins/my_plugin/metadata
```

**Permission issues:**
```bash
# Fix plugin directory permissions
chmod -R 755 /app/plugins
chown -R appuser:appuser /app/plugins
```

### 4. Memory Errors

#### Symptom
Out of memory errors or service crashes due to memory.

#### Diagnosis
```bash
# Check memory usage patterns
docker stats content-processing

# Check for memory leaks
curl http://localhost:8082/metrics | grep memory

# Review memory-related logs
docker logs content-processing | grep -i memory
```

#### Solutions

**Reduce memory usage:**
```bash
# Limit content size
export MAX_CONTENT_SIZE=5242880  # 5MB

# Reduce plugin memory
export PLUGIN_MEMORY_LIMIT=10  # 10MB per plugin

# Limit concurrent jobs
export MAX_CONCURRENT_JOBS=50
```

**Configure memory limits:**
```yaml
# Docker Compose
services:
  content-processing:
    mem_limit: 4g
    mem_reservation: 2g
```

**Enable memory monitoring:**
```bash
# Set memory alerts
export MEMORY_ALERT_THRESHOLD=80  # Alert at 80% usage
```

### 5. Database Performance Issues

#### Symptom
Slow queries or database connection pool exhaustion.

#### Diagnosis
```sql
-- Check active connections
SELECT count(*) FROM pg_stat_activity 
WHERE datname = 'content_processing_db';

-- Find slow queries
SELECT query, mean_exec_time, calls 
FROM pg_stat_statements 
ORDER BY mean_exec_time DESC 
LIMIT 10;

-- Check table sizes
SELECT 
  schemaname,
  tablename,
  pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size
FROM pg_tables 
WHERE schemaname = 'public' 
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;
```

#### Solutions

**Optimize database connections:**
```bash
# Increase connection pool
export DATABASE_MAX_CONNECTIONS=50
export DATABASE_MIN_CONNECTIONS=10

# Reduce connection lifetime
export DATABASE_MAX_LIFETIME=300
```

**Add missing indexes:**
```sql
-- Check for missing indexes
SELECT 
  schemaname,
  tablename,
  attname,
  n_distinct,
  correlation
FROM pg_stats
WHERE schemaname = 'public'
  AND n_distinct > 100
  AND correlation < 0.1
ORDER BY n_distinct DESC;

-- Add suggested indexes
CREATE INDEX CONCURRENTLY idx_content_metadata_created_at 
ON content_metadata(created_at);
```

**Vacuum and analyze:**
```sql
-- Manual vacuum
VACUUM ANALYZE content_metadata;
VACUUM ANALYZE processing_jobs;

-- Check autovacuum status
SELECT 
  schemaname,
  tablename,
  last_vacuum,
  last_autovacuum,
  last_analyze,
  last_autoanalyze
FROM pg_stat_user_tables;
```

### 6. Cache Issues

#### Symptom
High cache miss rate or Redis connection errors.

#### Diagnosis
```bash
# Check Redis connectivity
redis-cli -u $REDIS_URL ping

# Monitor cache hit rate
curl http://localhost:8082/metrics | grep cache_hit_rate

# Check Redis memory
redis-cli -u $REDIS_URL info memory
```

#### Solutions

**Fix Redis connection:**
```bash
# Test Redis connection
redis-cli -u $REDIS_URL --latency

# Increase connection pool
export REDIS_MAX_CONNECTIONS=20

# Enable connection retry
export REDIS_RETRY_ENABLED=true
export REDIS_RETRY_MAX_ATTEMPTS=3
```

**Optimize cache usage:**
```bash
# Increase cache TTL
export CACHE_TTL=7200  # 2 hours

# Increase cache size
export CACHE_MAX_SIZE=5000

# Enable cache warming
export CACHE_WARM_ON_START=true
```

### 7. API Rate Limiting

#### Symptom
429 Too Many Requests errors.

#### Diagnosis
```bash
# Check rate limit headers
curl -I http://localhost:8082/analyze | grep X-RateLimit

# Monitor rate limit metrics
curl http://localhost:8082/metrics | grep rate_limit
```

#### Solutions

**Adjust rate limits:**
```bash
# Increase rate limits
export API_RATE_LIMIT=500  # 500 requests per minute
export API_BURST_LIMIT=50

# Configure per-user limits
export USER_RATE_LIMIT=200
export USER_BURST_LIMIT=20
```

**Implement backoff:**
```python
import time
import requests

def request_with_backoff(url, data, max_retries=3):
    for i in range(max_retries):
        response = requests.post(url, json=data)
        if response.status_code == 429:
            retry_after = int(response.headers.get('Retry-After', 60))
            time.sleep(retry_after)
        else:
            return response
    raise Exception("Max retries exceeded")
```

## Performance Debugging

### Enable Profiling

```bash
# Enable CPU profiling
export ENABLE_PROFILING=true
export PROFILE_OUTPUT_DIR=/tmp/profiles

# Enable memory profiling
export MEMORY_PROFILING=true
export MEMORY_PROFILE_INTERVAL=60
```

### Analyze Flamegraphs

```bash
# Generate flamegraph
cargo flamegraph --bin content_processing

# Analyze with perf
perf record -g ./target/release/content_processing
perf report
```

### Trace Requests

```bash
# Enable request tracing
export RUST_LOG=trace,content_processing=trace
export TRACE_REQUESTS=true

# Follow specific request
curl -H "X-Correlation-ID: test-123" http://localhost:8082/analyze
docker logs content-processing | grep test-123
```

## Log Analysis

### Common Log Patterns

```bash
# Find errors
grep -E "ERROR|PANIC" /app/logs/content_processing.log

# Find slow requests
grep -E "processing_time_ms\":[0-9]{4,}" /app/logs/content_processing.log

# Find plugin errors
grep -E "plugin.*error|Plugin.*failed" /app/logs/content_processing.log

# Find memory warnings
grep -i "memory" /app/logs/content_processing.log | grep -E "WARN|ERROR"
```

### Structured Log Queries

```bash
# Parse JSON logs with jq
tail -f /app/logs/content_processing.log | jq 'select(.level == "ERROR")'

# Filter by correlation ID
tail -f /app/logs/content_processing.log | jq 'select(.correlation_id == "abc-123")'

# Find slow operations
tail -f /app/logs/content_processing.log | jq 'select(.processing_time_ms > 1000)'
```

## Health Check Failures

### Database Health Check

```bash
# Test database connection manually
psql $DATABASE_URL -c "SELECT version();"

# Check connection pool status
curl http://localhost:8082/metrics | grep database_connections

# Reset connection pool
curl -X POST http://localhost:8082/admin/reset-db-pool
```

### Redis Health Check

```bash
# Test Redis manually
redis-cli -u $REDIS_URL ping
redis-cli -u $REDIS_URL info server

# Check Redis persistence
redis-cli -u $REDIS_URL config get save

# Flush Redis cache (careful!)
redis-cli -u $REDIS_URL flushdb
```

### Plugin Health Check

```bash
# List plugin status
curl http://localhost:8082/plugins/status

# Reload plugins
curl -X POST http://localhost:8082/plugins/reload

# Disable problematic plugin
curl -X PATCH http://localhost:8082/plugins/bad_plugin \
  -d '{"enabled": false}'
```

## Emergency Procedures

### Service Restart

```bash
# Graceful restart
docker restart content-processing

# Force restart
docker kill content-processing
docker start content-processing

# Kubernetes restart
kubectl rollout restart deployment/content-processing
```

### Emergency Scaling

```bash
# Docker Compose scale
docker-compose up -d --scale content-processing=5

# Kubernetes scale
kubectl scale deployment/content-processing --replicas=10
```

### Circuit Breaker

```bash
# Enable circuit breaker
export CIRCUIT_BREAKER_ENABLED=true
export CIRCUIT_BREAKER_THRESHOLD=50
export CIRCUIT_BREAKER_TIMEOUT=60

# Check circuit breaker status
curl http://localhost:8082/admin/circuit-breaker/status
```

## Monitoring Commands

### Real-time Monitoring

```bash
# Watch metrics
watch -n 1 'curl -s http://localhost:8082/metrics | grep -E "active_|_total|_rate"'

# Monitor logs
tail -f /app/logs/content_processing.log | grep -E "ERROR|WARN|processing_time"

# System resources
htop -p $(pgrep content_processing)
```

### Debugging Checklist

1. **Check service health**: `/health/detailed`
2. **Review recent logs**: Last 1000 lines for errors
3. **Check system resources**: CPU, memory, disk
4. **Verify dependencies**: Database, Redis, plugins
5. **Review configuration**: Environment variables
6. **Check metrics**: Processing times, error rates
7. **Test with minimal config**: Disable plugins, reduce load
8. **Enable debug logging**: `RUST_LOG=debug`
9. **Capture thread dump**: For deadlock analysis
10. **Review recent changes**: Git log, deployments

## Getting Help

### Collecting Debug Information

```bash
#!/bin/bash
# debug-info.sh

echo "=== System Information ==="
uname -a
docker version
df -h

echo "=== Service Status ==="
docker ps | grep content-processing
curl -s http://localhost:8082/health/detailed | jq .

echo "=== Recent Logs ==="
docker logs content-processing --tail 100 | grep -E "ERROR|WARN"

echo "=== Metrics Snapshot ==="
curl -s http://localhost:8082/metrics | grep -E "error|duration|active"

echo "=== Configuration ==="
docker exec content-processing env | grep -E "^[A-Z_]+=" | sort
```

### Reporting Issues

When reporting issues, include:

1. **Service version**: From `/health` endpoint
2. **Error messages**: Complete stack traces
3. **Configuration**: Sanitized environment variables
4. **Steps to reproduce**: Minimal example
5. **Debug information**: Output from debug script
6. **Expected behavior**: What should happen
7. **Actual behavior**: What actually happens