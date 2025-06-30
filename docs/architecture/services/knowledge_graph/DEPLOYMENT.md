# Knowledge Graph Deployment Guide

## Overview

This guide covers deploying the Knowledge Graph Service in various environments, from development to production-scale Kubernetes clusters. It includes Dgraph cluster setup, scaling strategies, and operational best practices.

## Prerequisites

- Docker 20.10+
- Kubernetes 1.24+ (for K8s deployment)
- Helm 3.0+ (for Helm charts)
- PostgreSQL 15+ with pgvector extension
- Redis 7.0+
- 8GB RAM minimum (development)
- 32GB RAM recommended (production)

## Development Deployment

### Local Docker Compose

1. **Clone and navigate to the service:**
```bash
cd services/knowledge_graph
```

2. **Create environment file:**
```bash
cat > .env << EOF
ENVIRONMENT=development
LOG_LEVEL=debug
DGRAPH_HOST=dgraph-alpha
REDIS_URL=redis://redis:6379
DATABASE_URL=postgresql://postgres:password@postgres:5432/knowledge_graph
JWT_SECRET=dev-secret-key
EOF
```

3. **Start all services:**
```bash
docker-compose up -d
```

4. **Initialize Dgraph schema:**
```bash
cd dgraph
./init-schema.sh
```

5. **Verify deployment:**
```bash
# Check service health
curl http://localhost:3002/health

# Access Dgraph Ratel UI
open http://localhost:8000

# View logs
docker-compose logs -f knowledge-graph
```

### Docker Compose Configuration

```yaml
version: '3.8'

services:
  # Dgraph Zero - Cluster coordinator
  dgraph-zero:
    image: dgraph/dgraph:v23.1.0
    container_name: kg_dgraph_zero
    volumes:
      - dgraph_zero_data:/dgraph
    ports:
      - "5080:5080"
      - "6080:6080"
    command: dgraph zero --my=dgraph-zero:5080
    
  # Dgraph Alpha - Data node
  dgraph-alpha:
    image: dgraph/dgraph:v23.1.0
    container_name: kg_dgraph_alpha
    volumes:
      - dgraph_alpha_data:/dgraph
    ports:
      - "8080:8080"
      - "9080:9080"
    command: dgraph alpha --my=dgraph-alpha:7080 --zero=dgraph-zero:5080
    depends_on:
      - dgraph-zero
      
  # Redis Cache
  redis:
    image: redis:7-alpine
    container_name: kg_redis
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
      
  # PostgreSQL for embeddings
  postgres:
    image: pgvector/pgvector:pg15
    container_name: kg_postgres
    environment:
      POSTGRES_DB: knowledge_graph
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: password
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
      
  # Knowledge Graph Service
  knowledge-graph:
    build: .
    container_name: kg_service
    ports:
      - "3002:3002"
    environment:
      - SERVICE_PORT=3002
      - DGRAPH_HOST=dgraph-alpha
      - REDIS_URL=redis://redis:6379
      - DATABASE_URL=postgresql://postgres:password@postgres:5432/knowledge_graph
    depends_on:
      - dgraph-alpha
      - redis
      - postgres
    env_file:
      - .env
      
volumes:
  dgraph_zero_data:
  dgraph_alpha_data:
  redis_data:
  postgres_data:
```

## Production Dgraph Cluster Setup

### Multi-Node Architecture

```mermaid
graph TB
    subgraph "Load Balancer"
        LB[HAProxy/NGINX]
    end
    
    subgraph "Dgraph Zeros (Raft Group)"
        Z1[Zero-1<br/>Leader]
        Z2[Zero-2<br/>Follower]
        Z3[Zero-3<br/>Follower]
    end
    
    subgraph "Dgraph Alphas Group 1"
        A1[Alpha-1<br/>Leader]
        A2[Alpha-2<br/>Follower]
        A3[Alpha-3<br/>Follower]
    end
    
    subgraph "Dgraph Alphas Group 2"
        A4[Alpha-4<br/>Leader]
        A5[Alpha-5<br/>Follower]
        A6[Alpha-6<br/>Follower]
    end
    
    LB --> A1
    LB --> A4
    
    Z1 -.-> Z2
    Z2 -.-> Z3
    Z3 -.-> Z1
    
    A1 -.-> A2
    A2 -.-> A3
    A3 -.-> A1
    
    A4 -.-> A5
    A5 -.-> A6
    A6 -.-> A4
```

### Production Setup Steps

1. **Prepare nodes:**
```bash
# On each node
sudo sysctl -w vm.max_map_count=262144
echo "vm.max_map_count=262144" | sudo tee -a /etc/sysctl.conf

# Create data directories
sudo mkdir -p /data/dgraph
sudo chown 1000:1000 /data/dgraph
```

2. **Deploy Dgraph Zeros:**
```bash
# On Zero-1
docker run -d --name dgraph-zero-1 \
  -p 5080:5080 -p 6080:6080 \
  -v /data/dgraph:/dgraph \
  dgraph/dgraph:v23.1.0 \
  dgraph zero --my=zero-1:5080 \
  --replicas=3 --raft="idx=1"

# On Zero-2
docker run -d --name dgraph-zero-2 \
  -p 5080:5080 -p 6080:6080 \
  -v /data/dgraph:/dgraph \
  dgraph/dgraph:v23.1.0 \
  dgraph zero --my=zero-2:5080 \
  --peer=zero-1:5080 \
  --replicas=3 --raft="idx=2"

# On Zero-3
docker run -d --name dgraph-zero-3 \
  -p 5080:5080 -p 6080:6080 \
  -v /data/dgraph:/dgraph \
  dgraph/dgraph:v23.1.0 \
  dgraph zero --my=zero-3:5080 \
  --peer=zero-1:5080 \
  --replicas=3 --raft="idx=3"
```

3. **Deploy Dgraph Alphas:**
```bash
# Group 1 - Alpha-1
docker run -d --name dgraph-alpha-1 \
  -p 8080:8080 -p 9080:9080 \
  -v /data/dgraph:/dgraph \
  dgraph/dgraph:v23.1.0 \
  dgraph alpha --my=alpha-1:7080 \
  --zero=zero-1:5080,zero-2:5080,zero-3:5080 \
  --cache_mb=4096

# Continue for all Alpha nodes...
```

## Kubernetes Deployment

### Namespace and Storage

```yaml
# namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: knowledge-graph

---
# storage-class.yaml
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: fast-ssd
provisioner: kubernetes.io/aws-ebs
parameters:
  type: gp3
  iops: "10000"
  throughput: "250"
allowVolumeExpansion: true
```

### Dgraph StatefulSet

```yaml
# dgraph-zero.yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: dgraph-zero
  namespace: knowledge-graph
spec:
  serviceName: dgraph-zero
  replicas: 3
  selector:
    matchLabels:
      app: dgraph-zero
  template:
    metadata:
      labels:
        app: dgraph-zero
    spec:
      containers:
      - name: zero
        image: dgraph/dgraph:v23.1.0
        ports:
        - containerPort: 5080
          name: grpc-zero
        - containerPort: 6080
          name: http-zero
        volumeMounts:
        - name: data
          mountPath: /dgraph
        env:
        - name: POD_NAME
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        command:
          - bash
          - -c
          - |
            set -ex
            ordinal=${POD_NAME##*-}
            idx=$(($ordinal + 1))
            if [[ $ordinal -eq 0 ]]; then
              exec dgraph zero --my=$(hostname):5080 --raft="idx=$idx" --replicas=3
            else
              exec dgraph zero --my=$(hostname):5080 --peer=dgraph-zero-0.dgraph-zero:5080 --raft="idx=$idx" --replicas=3
            fi
        resources:
          requests:
            memory: "2Gi"
            cpu: "1"
          limits:
            memory: "4Gi"
            cpu: "2"
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      storageClassName: fast-ssd
      resources:
        requests:
          storage: 50Gi
```

### Service Deployment

```yaml
# knowledge-graph-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: knowledge-graph
  namespace: knowledge-graph
spec:
  replicas: 3
  selector:
    matchLabels:
      app: knowledge-graph
  template:
    metadata:
      labels:
        app: knowledge-graph
    spec:
      containers:
      - name: service
        image: knowledge-graph:latest
        ports:
        - containerPort: 3002
        env:
        - name: SERVICE_PORT
          value: "3002"
        - name: DGRAPH_HOST
          value: "dgraph-alpha"
        - name: REDIS_URL
          value: "redis://redis-master:6379"
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: kg-secrets
              key: database-url
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: kg-secrets
              key: jwt-secret
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 3002
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health/ready
            port: 3002
          initialDelaySeconds: 5
          periodSeconds: 5
```

### Helm Chart Structure

```
knowledge-graph-helm/
├── Chart.yaml
├── values.yaml
├── templates/
│   ├── deployment.yaml
│   ├── service.yaml
│   ├── configmap.yaml
│   ├── secret.yaml
│   ├── ingress.yaml
│   ├── hpa.yaml
│   └── pdb.yaml
└── charts/
    ├── dgraph/
    ├── redis/
    └── postgresql/
```

### Helm Values

```yaml
# values.yaml
replicaCount: 3

image:
  repository: knowledge-graph
  pullPolicy: IfNotPresent
  tag: "latest"

service:
  type: ClusterIP
  port: 3002

ingress:
  enabled: true
  className: nginx
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
  hosts:
    - host: kg-api.example.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: kg-tls
      hosts:
        - kg-api.example.com

resources:
  limits:
    cpu: 1000m
    memory: 1Gi
  requests:
    cpu: 500m
    memory: 512Mi

autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 10
  targetCPUUtilizationPercentage: 70
  targetMemoryUtilizationPercentage: 80

dgraph:
  enabled: true
  zero:
    replicas: 3
    persistence:
      size: 50Gi
  alpha:
    replicas: 6
    persistence:
      size: 100Gi

redis:
  enabled: true
  architecture: replication
  auth:
    enabled: true
  master:
    persistence:
      size: 10Gi

postgresql:
  enabled: true
  auth:
    postgresPassword: "changeme"
    database: knowledge_graph
  primary:
    persistence:
      size: 50Gi
```

## Data Backup Strategies

### Dgraph Backup

1. **Online Backup (Recommended):**
```bash
# Full backup
curl -X POST http://dgraph-alpha:8080/admin \
  -H "Content-Type: application/json" \
  -d '{
    "query": "mutation { backup(input: {destination: \"s3://bucket/backup\"}) { response { message } } }"
  }'

# Incremental backup
curl -X POST http://dgraph-alpha:8080/admin \
  -H "Content-Type: application/json" \
  -d '{
    "query": "mutation { backup(input: {destination: \"s3://bucket/backup\", forceFull: false}) { response { message } } }"
  }'
```

2. **Backup Schedule (CronJob):**
```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: dgraph-backup
  namespace: knowledge-graph
spec:
  schedule: "0 2 * * *"  # Daily at 2 AM
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: backup
            image: curlimages/curl:latest
            command:
            - /bin/sh
            - -c
            - |
              curl -X POST http://dgraph-alpha:8080/admin \
                -H "Content-Type: application/json" \
                -d '{
                  "query": "mutation { backup(input: {destination: \"s3://kg-backups/dgraph/$(date +%Y%m%d)\"}) { response { message } } }"
                }'
          restartPolicy: OnFailure
```

### PostgreSQL Backup

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: postgres-backup
spec:
  schedule: "0 3 * * *"  # Daily at 3 AM
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: postgres-backup
            image: postgres:15
            env:
            - name: PGPASSWORD
              valueFrom:
                secretKeyRef:
                  name: postgres-secret
                  key: password
            command:
            - /bin/bash
            - -c
            - |
              DATE=$(date +%Y%m%d_%H%M%S)
              pg_dump -h postgres -U postgres knowledge_graph | \
                gzip > /backup/kg_backup_${DATE}.sql.gz
              # Upload to S3
              aws s3 cp /backup/kg_backup_${DATE}.sql.gz \
                s3://kg-backups/postgres/
            volumeMounts:
            - name: backup
              mountPath: /backup
          volumes:
          - name: backup
            emptyDir: {}
          restartPolicy: OnFailure
```

## Monitoring Setup

### Prometheus Configuration

```yaml
# prometheus-scrape-config.yaml
- job_name: 'knowledge-graph'
  kubernetes_sd_configs:
  - role: pod
    namespaces:
      names:
      - knowledge-graph
  relabel_configs:
  - source_labels: [__meta_kubernetes_pod_label_app]
    action: keep
    regex: knowledge-graph
  - source_labels: [__meta_kubernetes_pod_name]
    target_label: instance
  - target_label: __metrics_path__
    replacement: /metrics
```

### Grafana Dashboard

```json
{
  "dashboard": {
    "title": "Knowledge Graph Service",
    "panels": [
      {
        "title": "Request Rate",
        "targets": [
          {
            "expr": "rate(http_requests_total{service=\"knowledge-graph\"}[5m])"
          }
        ]
      },
      {
        "title": "Query Latency",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(query_duration_seconds_bucket[5m]))"
          }
        ]
      },
      {
        "title": "Dgraph Connection Pool",
        "targets": [
          {
            "expr": "dgraph_connection_pool_active{service=\"knowledge-graph\"}"
          }
        ]
      },
      {
        "title": "Cache Hit Rate",
        "targets": [
          {
            "expr": "rate(cache_hits_total[5m]) / rate(cache_requests_total[5m])"
          }
        ]
      }
    ]
  }
}
```

## Scaling Considerations

### Horizontal Scaling

1. **Service Instances:**
```yaml
kubectl scale deployment knowledge-graph -n knowledge-graph --replicas=5
```

2. **Dgraph Alpha Nodes:**
```bash
# Add new Alpha node
docker run -d --name dgraph-alpha-7 \
  -p 8087:8080 -p 9087:9080 \
  dgraph/dgraph:v23.1.0 \
  dgraph alpha --my=alpha-7:7080 \
  --zero=zero-1:5080,zero-2:5080,zero-3:5080
```

3. **Auto-scaling Configuration:**
```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: knowledge-graph-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: knowledge-graph
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
  - type: Pods
    pods:
      metric:
        name: http_requests_per_second
      target:
        type: AverageValue
        averageValue: "100"
```

### Vertical Scaling

```yaml
# Update resource limits
kubectl set resources deployment knowledge-graph \
  -n knowledge-graph \
  --limits=cpu=2000m,memory=2Gi \
  --requests=cpu=1000m,memory=1Gi
```

## Security Hardening

### Network Policies

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: knowledge-graph-netpol
spec:
  podSelector:
    matchLabels:
      app: knowledge-graph
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
      port: 3002
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: dgraph-alpha
    ports:
    - protocol: TCP
      port: 9080
  - to:
    - podSelector:
        matchLabels:
          app: redis
    ports:
    - protocol: TCP
      port: 6379
```

### Pod Security Policy

```yaml
apiVersion: policy/v1beta1
kind: PodSecurityPolicy
metadata:
  name: knowledge-graph-psp
spec:
  privileged: false
  allowPrivilegeEscalation: false
  requiredDropCapabilities:
    - ALL
  volumes:
    - 'configMap'
    - 'secret'
    - 'persistentVolumeClaim'
  runAsUser:
    rule: 'MustRunAsNonRoot'
  seLinux:
    rule: 'RunAsAny'
  fsGroup:
    rule: 'RunAsAny'
```

## Deployment Checklist

### Pre-deployment
- [ ] Environment variables configured
- [ ] Secrets created in Kubernetes
- [ ] Database migrations completed
- [ ] Dgraph schema initialized
- [ ] SSL certificates provisioned
- [ ] Monitoring dashboards created
- [ ] Backup strategy implemented

### Deployment
- [ ] Deploy Dgraph cluster
- [ ] Deploy Redis
- [ ] Deploy PostgreSQL
- [ ] Deploy Knowledge Graph service
- [ ] Configure ingress/load balancer
- [ ] Verify health endpoints
- [ ] Test API endpoints

### Post-deployment
- [ ] Monitor metrics for 24 hours
- [ ] Verify backup execution
- [ ] Performance testing
- [ ] Security scanning
- [ ] Documentation updated
- [ ] Team training completed

## Rollback Procedures

### Service Rollback

```bash
# View deployment history
kubectl rollout history deployment/knowledge-graph -n knowledge-graph

# Rollback to previous version
kubectl rollback undo deployment/knowledge-graph -n knowledge-graph

# Rollback to specific revision
kubectl rollout undo deployment/knowledge-graph --to-revision=2 -n knowledge-graph
```

### Database Rollback

```bash
# Restore Dgraph from backup
dgraph restore -p /dgraph/backup -z zero-1:5080

# Restore PostgreSQL
psql -h postgres -U postgres knowledge_graph < backup.sql
```

## Troubleshooting Deployment Issues

### Common Issues

1. **Dgraph Connection Failures**
```bash
# Check Dgraph health
curl http://dgraph-alpha:8080/health

# View Dgraph logs
kubectl logs -n knowledge-graph dgraph-alpha-0
```

2. **Memory Issues**
```bash
# Check pod resources
kubectl top pods -n knowledge-graph

# Increase memory limits
kubectl edit deployment knowledge-graph -n knowledge-graph
```

3. **Slow Startup**
```bash
# Increase probe delays
initialDelaySeconds: 60
periodSeconds: 20
```

4. **Network Issues**
```bash
# Test connectivity
kubectl exec -it knowledge-graph-pod -- nc -zv dgraph-alpha 9080
```