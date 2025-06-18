# Content Processing Service Deployment Guide

## Overview

This guide covers deployment strategies for the Content Processing Service across different environments, from local development to production Kubernetes clusters.

## Prerequisites

- Docker 20.10+
- Kubernetes 1.25+ (for K8s deployment)
- PostgreSQL 14+ with pgvector extension
- Redis 7+
- 2GB+ RAM minimum (4GB+ recommended)
- 10GB+ disk space

## Docker Deployment

### Building the Docker Image

```bash
# Build from the repository root
docker build -f services/content_processing/Dockerfile -t content-processing:latest .

# Build with specific version tag
docker build -f services/content_processing/Dockerfile \
  -t content-processing:v1.0.0 \
  -t content-processing:latest .

# Multi-architecture build (ARM64 + AMD64)
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -f services/content_processing/Dockerfile \
  -t content-processing:latest \
  --push .
```

### Running with Docker

```bash
# Basic run
docker run -d \
  --name content-processing \
  -p 8082:8082 \
  -e DATABASE_URL="postgresql://user:pass@host.docker.internal/content_db" \
  -e REDIS_URL="redis://host.docker.internal:6379" \
  content-processing:latest

# Production run with all options
docker run -d \
  --name content-processing \
  --restart unless-stopped \
  -p 8082:8082 \
  -v /opt/plugins:/app/plugins:ro \
  -v /var/log/content-processing:/app/logs \
  --memory="2g" \
  --cpus="2" \
  --health-cmd="curl -f http://localhost:8082/health || exit 1" \
  --health-interval=30s \
  --health-timeout=10s \
  --health-retries=3 \
  --env-file .env.production \
  content-processing:latest
```

### Docker Compose Deployment

```yaml
# docker-compose.yml
version: '3.8'

services:
  postgres:
    image: pgvector/pgvector:pg14
    environment:
      POSTGRES_DB: content_db
      POSTGRES_USER: content_user
      POSTGRES_PASSWORD: content_pass
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U content_user"]
      interval: 10s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    command: redis-server --requirepass redis_pass
    volumes:
      - redis_data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5

  content-processing:
    image: content-processing:latest
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
    ports:
      - "8082:8082"
    environment:
      DATABASE_URL: postgresql://content_user:content_pass@postgres/content_db
      REDIS_URL: redis://:redis_pass@redis:6379
      RUST_LOG: info
      WORKER_THREADS: 8
      MAX_CONCURRENT_JOBS: 100
    volumes:
      - ./plugins:/app/plugins:ro
      - ./logs:/app/logs
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8082/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    restart: unless-stopped

volumes:
  postgres_data:
  redis_data:
```

Deploy with Docker Compose:

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f content-processing

# Scale the service
docker-compose up -d --scale content-processing=3

# Update service
docker-compose pull content-processing
docker-compose up -d content-processing
```

## Kubernetes Deployment

### Namespace and ConfigMap

```yaml
# namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: content-processing
  
---
# configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: content-processing-config
  namespace: content-processing
data:
  PORT: "8082"
  RUST_LOG: "info"
  WORKER_THREADS: "16"
  MAX_CONCURRENT_JOBS: "200"
  METRICS_ENABLED: "true"
  PLUGIN_DIR: "/app/plugins"
```

### Secrets

```yaml
# secrets.yaml
apiVersion: v1
kind: Secret
metadata:
  name: content-processing-secrets
  namespace: content-processing
type: Opaque
stringData:
  DATABASE_URL: "postgresql://user:pass@postgres-service/content_db"
  REDIS_URL: "redis://:password@redis-service:6379"
  JWT_SECRET: "your-secret-key"
```

### Deployment

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: content-processing
  namespace: content-processing
  labels:
    app: content-processing
spec:
  replicas: 3
  selector:
    matchLabels:
      app: content-processing
  template:
    metadata:
      labels:
        app: content-processing
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "8082"
        prometheus.io/path: "/metrics"
    spec:
      serviceAccountName: content-processing
      containers:
      - name: content-processing
        image: content-processing:v1.0.0
        imagePullPolicy: IfNotPresent
        ports:
        - containerPort: 8082
          name: http
          protocol: TCP
        envFrom:
        - configMapRef:
            name: content-processing-config
        - secretRef:
            name: content-processing-secrets
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"
        livenessProbe:
          httpGet:
            path: /health
            port: http
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /health
            port: http
          initialDelaySeconds: 10
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 3
        volumeMounts:
        - name: plugins
          mountPath: /app/plugins
          readOnly: true
        - name: temp
          mountPath: /tmp
      volumes:
      - name: plugins
        configMap:
          name: content-processing-plugins
      - name: temp
        emptyDir: {}
```

### Service

```yaml
# service.yaml
apiVersion: v1
kind: Service
metadata:
  name: content-processing
  namespace: content-processing
  labels:
    app: content-processing
spec:
  type: ClusterIP
  ports:
  - port: 80
    targetPort: 8082
    protocol: TCP
    name: http
  selector:
    app: content-processing
```

### Horizontal Pod Autoscaler

```yaml
# hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: content-processing
  namespace: content-processing
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: content-processing
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 50
        periodSeconds: 60
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
      - type: Percent
        value: 100
        periodSeconds: 60
```

### Ingress

```yaml
# ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: content-processing
  namespace: content-processing
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
    nginx.ingress.kubernetes.io/rate-limit: "100"
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
spec:
  ingressClassName: nginx
  tls:
  - hosts:
    - content-api.example.com
    secretName: content-processing-tls
  rules:
  - host: content-api.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: content-processing
            port:
              number: 80
```

### Deploy to Kubernetes

```bash
# Create namespace and resources
kubectl apply -f namespace.yaml
kubectl apply -f configmap.yaml
kubectl apply -f secrets.yaml
kubectl apply -f deployment.yaml
kubectl apply -f service.yaml
kubectl apply -f hpa.yaml
kubectl apply -f ingress.yaml

# Check deployment status
kubectl -n content-processing get pods
kubectl -n content-processing describe deployment content-processing

# View logs
kubectl -n content-processing logs -f deployment/content-processing

# Port forward for local testing
kubectl -n content-processing port-forward svc/content-processing 8082:80
```

## Database Setup and Migrations

### PostgreSQL with pgvector

```bash
# Create database
createdb content_processing_db

# Install pgvector extension
psql content_processing_db -c "CREATE EXTENSION IF NOT EXISTS vector;"

# Run migrations
export DATABASE_URL="postgresql://user:pass@localhost/content_processing_db"
sqlx migrate run

# Verify migrations
sqlx migrate info
```

### Migration Scripts

```sql
-- Rollback script example
-- rollback.sql
DROP TABLE IF EXISTS processing_jobs;
DROP TABLE IF EXISTS content_metadata;
DROP TYPE IF EXISTS job_status;
DROP TYPE IF EXISTS job_priority;
```

### Database Backup

```bash
# Backup database
pg_dump -h localhost -U user -d content_processing_db > backup.sql

# Backup with compression
pg_dump -h localhost -U user -d content_processing_db -Fc > backup.dump

# Restore database
pg_restore -h localhost -U user -d content_processing_db backup.dump
```

## Monitoring Setup

### Prometheus Configuration

```yaml
# prometheus-config.yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'content-processing'
    kubernetes_sd_configs:
    - role: pod
      namespaces:
        names:
        - content-processing
    relabel_configs:
    - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_scrape]
      action: keep
      regex: true
    - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_path]
      action: replace
      target_label: __metrics_path__
      regex: (.+)
```

### Grafana Dashboard

```json
{
  "dashboard": {
    "title": "Content Processing Service",
    "panels": [
      {
        "title": "Request Rate",
        "targets": [
          {
            "expr": "rate(content_processing_requests_total[5m])"
          }
        ]
      },
      {
        "title": "Processing Duration",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, content_processing_duration_seconds_bucket)"
          }
        ]
      },
      {
        "title": "Active Jobs",
        "targets": [
          {
            "expr": "active_processing_jobs"
          }
        ]
      }
    ]
  }
}
```

## Production Best Practices

### 1. Resource Planning

```yaml
# Minimum production resources
resources:
  requests:
    memory: "1Gi"
    cpu: "1000m"
  limits:
    memory: "4Gi"
    cpu: "4000m"
```

### 2. Security Hardening

```yaml
# Security context
securityContext:
  runAsNonRoot: true
  runAsUser: 1000
  fsGroup: 1000
  capabilities:
    drop:
    - ALL
  readOnlyRootFilesystem: true
```

### 3. Network Policies

```yaml
# network-policy.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: content-processing
  namespace: content-processing
spec:
  podSelector:
    matchLabels:
      app: content-processing
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: ingress-nginx
    ports:
    - protocol: TCP
      port: 8082
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          name: database
    ports:
    - protocol: TCP
      port: 5432
  - to:
    - namespaceSelector:
        matchLabels:
          name: redis
    ports:
    - protocol: TCP
      port: 6379
```

### 4. Pod Disruption Budget

```yaml
# pdb.yaml
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: content-processing
  namespace: content-processing
spec:
  minAvailable: 2
  selector:
    matchLabels:
      app: content-processing
```

### 5. Blue-Green Deployment

```bash
# Deploy new version as green
kubectl apply -f deployment-green.yaml

# Test green deployment
kubectl port-forward deployment/content-processing-green 8083:8082

# Switch traffic to green
kubectl patch service content-processing -p '{"spec":{"selector":{"version":"green"}}}'

# Remove blue deployment
kubectl delete deployment content-processing-blue
```

## Rollback Procedures

### Docker Rollback

```bash
# List available versions
docker images content-processing

# Rollback to previous version
docker stop content-processing
docker run -d --name content-processing content-processing:v0.9.0
```

### Kubernetes Rollback

```bash
# Check rollout history
kubectl -n content-processing rollout history deployment/content-processing

# Rollback to previous version
kubectl -n content-processing rollout undo deployment/content-processing

# Rollback to specific revision
kubectl -n content-processing rollout undo deployment/content-processing --to-revision=2

# Monitor rollback
kubectl -n content-processing rollout status deployment/content-processing
```

## Health Checks

### Startup Script

```bash
#!/bin/bash
# startup.sh

# Wait for database
until pg_isready -h $DB_HOST -p $DB_PORT; do
  echo "Waiting for database..."
  sleep 2
done

# Run migrations
sqlx migrate run

# Start service
exec /usr/local/bin/content_processing
```

### Health Check Endpoint

```bash
# Basic health check
curl http://localhost:8082/health

# Detailed health check
curl http://localhost:8082/health/detailed
```

## Troubleshooting Deployment

### Common Issues

1. **Database Connection Failed**
   ```bash
   # Check database connectivity
   kubectl exec -it deployment/content-processing -- pg_isready -h postgres-service
   
   # Check credentials
   kubectl get secret content-processing-secrets -o yaml
   ```

2. **Plugin Loading Issues**
   ```bash
   # Check plugin volume
   kubectl exec -it deployment/content-processing -- ls -la /app/plugins
   
   # Check plugin logs
   kubectl logs deployment/content-processing | grep plugin
   ```

3. **Memory Issues**
   ```bash
   # Check resource usage
   kubectl top pods -n content-processing
   
   # Increase memory limits
   kubectl set resources deployment/content-processing -c content-processing --limits=memory=4Gi
   ```

### Debug Mode

```yaml
# Enable debug mode in deployment
env:
- name: RUST_LOG
  value: "debug,content_processing=trace"
- name: RUST_BACKTRACE
  value: "full"
```

## Maintenance

### Zero-Downtime Updates

```bash
# Update image without downtime
kubectl set image deployment/content-processing content-processing=content-processing:v1.1.0

# Monitor update
kubectl rollout status deployment/content-processing
```

### Database Maintenance

```bash
# Schedule maintenance window
kubectl scale deployment/content-processing --replicas=0

# Perform database maintenance
psql $DATABASE_URL -c "VACUUM ANALYZE;"
psql $DATABASE_URL -c "REINDEX DATABASE content_processing_db;"

# Resume service
kubectl scale deployment/content-processing --replicas=3
```

## Disaster Recovery

### Backup Strategy

```bash
# Automated backup CronJob
apiVersion: batch/v1
kind: CronJob
metadata:
  name: content-processing-backup
spec:
  schedule: "0 2 * * *"  # Daily at 2 AM
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: backup
            image: postgres:14
            command:
            - /bin/bash
            - -c
            - |
              pg_dump $DATABASE_URL | gzip > /backup/content_db_$(date +%Y%m%d).sql.gz
              find /backup -name "*.sql.gz" -mtime +7 -delete
```

### Restore Procedure

```bash
# Stop service
kubectl scale deployment/content-processing --replicas=0

# Restore database
gunzip < backup.sql.gz | psql $DATABASE_URL

# Restart service
kubectl scale deployment/content-processing --replicas=3
```