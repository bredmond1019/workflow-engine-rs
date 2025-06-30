# AI Workflow System - Monitoring Stack

This directory contains the complete monitoring and observability stack for the AI Workflow System, including Grafana dashboards, Prometheus metrics collection, Loki log aggregation, and distributed tracing with Jaeger.

## 🚀 Quick Start

### Start the Monitoring Stack

```bash
# Start the full monitoring stack
docker-compose -f docker-compose.yml -f docker-compose.monitoring.yml up -d

# Or start only monitoring services
docker-compose -f docker-compose.monitoring.yml up -d
```

### Access the Dashboards

- **Grafana**: http://localhost:3000 (admin/admin123)
- **Prometheus**: http://localhost:9090
- **Jaeger**: http://localhost:16686
- **AlertManager**: http://localhost:9093

## 📊 Dashboards

### 1. System Health Dashboard
**URL**: http://localhost:3000/d/ai-workflow-health

**Key Metrics**:
- Service availability and uptime
- Request rates and response times
- Workflow execution metrics
- Cross-system call performance
- Resource utilization (CPU, Memory, Disk)
- Error rates and health status

**Use Cases**:
- Real-time system monitoring
- Performance troubleshooting
- Capacity planning
- SLA monitoring

### 2. Correlation Tracking Dashboard
**URL**: http://localhost:3000/d/ai-workflow-correlation

**Key Features**:
- Active correlation ID tracking
- Request journey visualization
- Cross-service request flow
- Correlation-based log filtering
- Workflow execution correlation

**Use Cases**:
- Distributed request tracing
- Debugging cross-system issues
- Performance optimization
- Request flow analysis

## 🔧 Configuration

### Prometheus Configuration
Location: `monitoring/prometheus/prometheus.yml`

**Scrape Targets**:
- AI Workflow System (port 8080)
- AI Tutor Service (port 3001)
- MCP Servers (ports 8001-8003)
- Database and Redis
- System metrics (Node Exporter)

**Alert Rules**: `monitoring/prometheus/alert_rules.yml`

### Grafana Configuration
**Dashboards**: `monitoring/grafana/dashboards/`
**Provisioning**: `monitoring/grafana/provisioning/`

**Pre-configured Data Sources**:
- Prometheus (metrics)
- Loki (logs)
- Jaeger (traces)

### Loki Configuration
Location: `monitoring/loki/loki.yml`

**Features**:
- JSON log parsing
- Correlation ID indexing
- 7-day log retention
- Structured log queries

### Promtail Configuration
Location: `monitoring/promtail/promtail.yml`

**Log Sources**:
- Application logs (/app/logs/)
- System logs (/var/log/)
- Docker container logs
- Structured JSON logs

## 📈 Metrics

### Core Application Metrics

```prometheus
# Request metrics
http_requests_total{job="ai-workflow-system",method,endpoint,status}
http_request_duration_seconds{job="ai-workflow-system",method,endpoint}

# Workflow metrics
ai_workflow_executions_total{workflow_name,status}
ai_workflow_execution_duration_seconds{workflow_name}

# Cross-system call metrics
ai_workflow_cross_system_calls_total{target_system,operation,status}
ai_workflow_cross_system_call_duration_seconds{target_system,operation}

# Database metrics
db_connections_active{pool}
db_query_duration_seconds{query_type}
```

### System Metrics

```prometheus
# System resources
node_cpu_seconds_total{mode}
node_memory_MemAvailable_bytes
node_disk_io_time_seconds_total

# Service availability
up{job,instance}
```

## 🚨 Alerting

### Alert Levels

**Critical** (Immediate response required):
- Service down
- Database connection failures
- High error rates (>5%)

**Warning** (Investigation required):
- High resource usage
- Elevated response times
- Workflow execution failures

**Info** (Awareness):
- Missing correlation IDs
- Performance degradation

### Alert Channels

**Webhook Notifications**:
- Critical: Immediate notification
- Warning: Grouped notifications (5min intervals)
- Info: Batched notifications (2hr intervals)

**Email Notifications**:
- Critical alerts only
- Configurable SMTP settings

### Silence and Inhibition Rules
- Service down alerts suppress related error alerts
- High CPU alerts suppress response time alerts

## 🔍 Correlation ID Tracking

### How It Works

1. **Generation**: Correlation IDs are generated for each request
2. **Propagation**: IDs are passed between services via HTTP headers
3. **Logging**: All log entries include the correlation ID
4. **Indexing**: Loki indexes logs by correlation ID for fast queries

### Correlation Headers
- `X-Correlation-ID` (primary)
- `X-Request-ID` (fallback)
- `X-Trace-ID` (fallback)

### Query Examples

```logql
# Find all logs for a specific correlation ID
{job="ai-workflow-system"} | json | correlation_id = "abc-123-def"

# Count requests by correlation ID
sum by (correlation_id) (
  count_over_time({job=~"ai.*"} | json | correlation_id != "" [5m])
)

# Error rate by correlation
sum(rate({job=~"ai.*"} | json | level = "ERROR" [5m])) by (correlation_id)
```

## 🔧 Troubleshooting

### Common Issues

**1. No Metrics Appearing**
- Check service `/metrics` endpoints
- Verify Prometheus scrape configs
- Check network connectivity between containers

**2. Missing Logs in Grafana**
- Verify Loki is receiving logs from Promtail
- Check log file paths and permissions
- Verify JSON log format

**3. Correlation IDs Not Showing**
- Ensure correlation middleware is enabled
- Check log format includes correlation_id field
- Verify JSON parsing in Promtail

**4. Alerts Not Firing**
- Check AlertManager configuration
- Verify alert rule syntax
- Check webhook endpoints

### Debug Commands

```bash
# Check Prometheus targets
curl http://localhost:9090/api/v1/targets

# Check Loki status
curl http://localhost:3100/ready

# Test log ingestion
curl -X POST http://localhost:3100/loki/api/v1/push \
  -H "Content-Type: application/json" \
  -d @test-log.json

# Check Grafana datasources
curl -u admin:admin123 http://localhost:3000/api/datasources
```

## 📦 Production Deployment

### Resource Requirements

**Minimum**:
- CPU: 2 cores
- Memory: 4GB
- Disk: 50GB

**Recommended**:
- CPU: 4+ cores
- Memory: 8GB+
- Disk: 200GB+ (depending on retention)

### Security Considerations

1. **Change default passwords**
2. **Enable HTTPS/TLS**
3. **Configure authentication**
4. **Set up proper firewall rules**
5. **Enable audit logging**

### Scaling

**Prometheus**:
- Use federation for large deployments
- Configure remote storage
- Implement sharding

**Loki**:
- Configure S3/GCS storage
- Set up distributed mode
- Implement proper retention policies

**Grafana**:
- Use external database
- Configure high availability
- Set up load balancing

## 📚 Additional Resources

- [Prometheus Documentation](https://prometheus.io/docs/)
- [Grafana Documentation](https://grafana.com/docs/)
- [Loki Documentation](https://grafana.com/docs/loki/)
- [Jaeger Documentation](https://www.jaegertracing.io/docs/)
- [AlertManager Documentation](https://prometheus.io/docs/alerting/latest/alertmanager/)

## 🤝 Contributing

When adding new metrics or dashboards:

1. Follow naming conventions
2. Add appropriate labels
3. Update documentation
4. Test with sample data
5. Create appropriate alerts