# Deployment Guide

## Overview

This guide covers deploying the Real-time Communication Service in production environments, with a focus on handling 10,000+ concurrent WebSocket connections. It includes strategies for load balancing, clustering, monitoring, and scaling.

## WebSocket Load Balancing

### HAProxy Configuration

HAProxy is recommended for WebSocket load balancing due to its excellent performance and WebSocket support.

```haproxy
# /etc/haproxy/haproxy.cfg
global
    maxconn 50000
    tune.ssl.default-dh-param 2048
    log /dev/log local0
    
defaults
    mode http
    timeout connect 5s
    timeout client 30s
    timeout server 30s
    timeout tunnel 1h  # WebSocket connections
    option httplog
    
frontend websocket_frontend
    bind *:443 ssl crt /etc/ssl/certs/server.pem
    bind *:80
    
    # Force HTTPS
    redirect scheme https if !{ ssl_fc }
    
    # Add security headers
    http-response set-header Strict-Transport-Security "max-age=31536000; includeSubDomains"
    http-response set-header X-Frame-Options "DENY"
    http-response set-header X-Content-Type-Options "nosniff"
    
    # WebSocket detection
    acl is_websocket hdr(Upgrade) -i websocket
    acl is_websocket hdr_beg(Host) -i ws
    
    # Use WebSocket backend for WebSocket connections
    use_backend websocket_backend if is_websocket
    default_backend http_backend
    
backend websocket_backend
    balance leastconn  # Best for long-lived connections
    
    # Sticky sessions using source IP hash
    # hash-type consistent
    # stick-table type ip size 100k expire 30m
    # stick on src
    
    # Health checks
    option httpchk GET /health
    http-check expect status 200
    
    # WebSocket servers
    server ws1 10.0.1.10:8081 check maxconn 3000 weight 100
    server ws2 10.0.1.11:8081 check maxconn 3000 weight 100
    server ws3 10.0.1.12:8081 check maxconn 3000 weight 100
    server ws4 10.0.1.13:8081 check maxconn 3000 weight 100
    
backend http_backend
    balance roundrobin
    
    # Regular HTTP endpoints
    server http1 10.0.1.10:8081 check
    server http2 10.0.1.11:8081 check
    server http3 10.0.1.12:8081 check
    server http4 10.0.1.13:8081 check
```

### NGINX Configuration

Alternative configuration using NGINX:

```nginx
# /etc/nginx/nginx.conf
worker_processes auto;
worker_rlimit_nofile 65535;

events {
    worker_connections 10000;
    use epoll;
    multi_accept on;
}

http {
    # WebSocket upstream
    upstream websocket_backend {
        least_conn;  # Better for WebSocket connections
        
        server 10.0.1.10:8081 max_conns=3000 weight=100;
        server 10.0.1.11:8081 max_conns=3000 weight=100;
        server 10.0.1.12:8081 max_conns=3000 weight=100;
        server 10.0.1.13:8081 max_conns=3000 weight=100;
        
        # Passive health checks
        server 10.0.1.10:8081 max_fails=3 fail_timeout=30s;
        
        # Keepalive connections to backend
        keepalive 32;
    }
    
    # Map to extract JWT from various sources
    map $http_upgrade $connection_upgrade {
        default upgrade;
        '' close;
    }
    
    server {
        listen 80;
        listen 443 ssl http2;
        server_name ws.example.com;
        
        # SSL configuration
        ssl_certificate /etc/nginx/ssl/cert.pem;
        ssl_certificate_key /etc/nginx/ssl/key.pem;
        ssl_protocols TLSv1.2 TLSv1.3;
        ssl_ciphers HIGH:!aNULL:!MD5;
        
        # WebSocket endpoint
        location /ws {
            proxy_pass http://websocket_backend;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection $connection_upgrade;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            
            # Timeouts
            proxy_connect_timeout 7d;
            proxy_send_timeout 7d;
            proxy_read_timeout 7d;
            
            # Disable buffering for WebSocket
            proxy_buffering off;
            
            # Larger buffers for WebSocket frames
            proxy_buffer_size 64k;
            proxy_buffers 8 64k;
        }
        
        # Health check endpoint
        location /health {
            proxy_pass http://websocket_backend;
            proxy_http_version 1.1;
            proxy_set_header Connection "";
        }
        
        # Metrics endpoint (restricted)
        location /metrics {
            allow 10.0.0.0/8;  # Internal network only
            deny all;
            proxy_pass http://websocket_backend;
        }
    }
}
```

## Kubernetes StatefulSet Setup

### StatefulSet Configuration

```yaml
# realtime-communication-statefulset.yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: realtime-communication
  namespace: rtc
spec:
  serviceName: realtime-communication
  replicas: 4
  podManagementPolicy: Parallel
  updateStrategy:
    type: RollingUpdate
    rollingUpdate:
      partition: 0
  selector:
    matchLabels:
      app: realtime-communication
  template:
    metadata:
      labels:
        app: realtime-communication
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "9090"
        prometheus.io/path: "/metrics"
    spec:
      affinity:
        podAntiAffinity:
          requiredDuringSchedulingIgnoredDuringExecution:
          - labelSelector:
              matchExpressions:
              - key: app
                operator: In
                values:
                - realtime-communication
            topologyKey: kubernetes.io/hostname
      containers:
      - name: realtime-communication
        image: your-registry/realtime-communication:latest
        imagePullPolicy: Always
        ports:
        - containerPort: 8081
          name: websocket
        - containerPort: 9090
          name: metrics
        env:
        - name: POD_NAME
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        - name: POD_NAMESPACE
          valueFrom:
            fieldRef:
              fieldPath: metadata.namespace
        - name: NODE_ID
          valueFrom:
            fieldRef:
              fieldPath: spec.nodeName
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: jwt-secret
              key: secret
        - name: REDIS_URL
          value: "redis://redis-cluster:6379"
        - name: MAX_CONNECTIONS
          value: "3000"  # Per pod limit
        resources:
          requests:
            memory: "2Gi"
            cpu: "1000m"
          limits:
            memory: "4Gi"
            cpu: "2000m"
        readinessProbe:
          httpGet:
            path: /health
            port: 8081
          initialDelaySeconds: 10
          periodSeconds: 5
        livenessProbe:
          httpGet:
            path: /health
            port: 8081
          initialDelaySeconds: 30
          periodSeconds: 10
        lifecycle:
          preStop:
            exec:
              command: ["/bin/sh", "-c", "sleep 15"]
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      resources:
        requests:
          storage: 10Gi
```

### Service Configuration

```yaml
# realtime-communication-service.yaml
apiVersion: v1
kind: Service
metadata:
  name: realtime-communication
  namespace: rtc
  annotations:
    service.beta.kubernetes.io/aws-load-balancer-type: "nlb"
    service.beta.kubernetes.io/aws-load-balancer-cross-zone-load-balancing-enabled: "true"
spec:
  type: LoadBalancer
  sessionAffinity: ClientIP  # Sticky sessions
  sessionAffinityConfig:
    clientIP:
      timeoutSeconds: 3600
  ports:
  - port: 80
    targetPort: 8081
    protocol: TCP
    name: websocket
  selector:
    app: realtime-communication
---
# Headless service for StatefulSet
apiVersion: v1
kind: Service
metadata:
  name: realtime-communication-headless
  namespace: rtc
spec:
  clusterIP: None
  ports:
  - port: 8081
    name: websocket
  selector:
    app: realtime-communication
```

### Horizontal Pod Autoscaler

```yaml
# realtime-communication-hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: realtime-communication-hpa
  namespace: rtc
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: StatefulSet
    name: realtime-communication
  minReplicas: 4
  maxReplicas: 20
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
        name: active_connections
      target:
        type: AverageValue
        averageValue: "2500"  # Scale when avg connections > 2500
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 10
        periodSeconds: 60
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
      - type: Percent
        value: 50
        periodSeconds: 60
      - type: Pods
        value: 2
        periodSeconds: 60
```

## Redis Cluster Configuration

### Redis StatefulSet

```yaml
# redis-cluster.yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: redis-cluster
  namespace: rtc
spec:
  serviceName: redis-cluster
  replicas: 6  # 3 masters, 3 replicas
  selector:
    matchLabels:
      app: redis-cluster
  template:
    metadata:
      labels:
        app: redis-cluster
    spec:
      containers:
      - name: redis
        image: redis:7-alpine
        command: ["redis-server"]
        args:
          - "/conf/redis.conf"
          - "--protected-mode"
          - "no"
        ports:
        - containerPort: 6379
          name: client
        - containerPort: 16379
          name: gossip
        resources:
          requests:
            cpu: 100m
            memory: 256Mi
          limits:
            cpu: 500m
            memory: 1Gi
        volumeMounts:
        - name: conf
          mountPath: /conf
          readOnly: false
        - name: data
          mountPath: /data
          readOnly: false
      volumes:
      - name: conf
        configMap:
          name: redis-cluster-config
          defaultMode: 0755
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      resources:
        requests:
          storage: 10Gi
```

### Redis Configuration

```yaml
# redis-configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: redis-cluster-config
  namespace: rtc
data:
  redis.conf: |
    port 6379
    cluster-enabled yes
    cluster-config-file nodes.conf
    cluster-node-timeout 5000
    appendonly yes
    appendfsync everysec
    maxmemory 1gb
    maxmemory-policy allkeys-lru
    save 900 1
    save 300 10
    save 60 10000
    
    # Performance tuning
    tcp-backlog 511
    timeout 0
    tcp-keepalive 300
    
    # Persistence
    dir /data
    dbfilename dump.rdb
    
    # Logging
    loglevel notice
    logfile ""
    
    # Clients
    maxclients 10000
```

## Monitoring WebSocket Connections

### Prometheus Configuration

```yaml
# prometheus-config.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: prometheus-config
  namespace: monitoring
data:
  prometheus.yml: |
    global:
      scrape_interval: 15s
      evaluation_interval: 15s
    
    scrape_configs:
    - job_name: 'realtime-communication'
      kubernetes_sd_configs:
      - role: pod
        namespaces:
          names:
          - rtc
      relabel_configs:
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_scrape]
        action: keep
        regex: true
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_path]
        action: replace
        target_label: __metrics_path__
        regex: (.+)
      - source_labels: [__address__, __meta_kubernetes_pod_annotation_prometheus_io_port]
        action: replace
        regex: ([^:]+)(?::\d+)?;(\d+)
        replacement: $1:$2
        target_label: __address__
      - action: labelmap
        regex: __meta_kubernetes_pod_label_(.+)
      - source_labels: [__meta_kubernetes_namespace]
        action: replace
        target_label: kubernetes_namespace
      - source_labels: [__meta_kubernetes_pod_name]
        action: replace
        target_label: kubernetes_pod_name
```

### Grafana Dashboard

```json
{
  "dashboard": {
    "title": "Real-time Communication Service",
    "panels": [
      {
        "title": "Active WebSocket Connections",
        "targets": [
          {
            "expr": "sum(active_connections) by (pod)",
            "legendFormat": "{{ pod }}"
          }
        ],
        "type": "graph"
      },
      {
        "title": "Message Throughput",
        "targets": [
          {
            "expr": "rate(messages_sent_total[5m])",
            "legendFormat": "Sent"
          },
          {
            "expr": "rate(messages_received_total[5m])",
            "legendFormat": "Received"
          }
        ],
        "type": "graph"
      },
      {
        "title": "Connection Rate",
        "targets": [
          {
            "expr": "rate(total_connections[5m])",
            "legendFormat": "New Connections/sec"
          }
        ],
        "type": "graph"
      },
      {
        "title": "Error Rate",
        "targets": [
          {
            "expr": "rate(errors_total[5m])",
            "legendFormat": "Errors/sec"
          }
        ],
        "type": "graph"
      },
      {
        "title": "Rate Limiting",
        "targets": [
          {
            "expr": "rate(rate_limit_denied_total[5m])",
            "legendFormat": "Denied Requests/sec"
          }
        ],
        "type": "graph"
      },
      {
        "title": "Memory Usage",
        "targets": [
          {
            "expr": "container_memory_usage_bytes{pod=~\"realtime-communication-.*\"}",
            "legendFormat": "{{ pod }}"
          }
        ],
        "type": "graph"
      }
    ]
  }
}
```

## Production Scaling Strategies

### 1. Connection Distribution

```yaml
# connection-router.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: connection-router
  namespace: rtc
data:
  router.lua: |
    -- Consistent hashing for connection distribution
    local function hash_djb2(str)
      local hash = 5381
      for i = 1, #str do
        hash = ((hash << 5) + hash) + string.byte(str, i)
      end
      return hash
    end
    
    local function get_backend_for_user(user_id, backends)
      local hash = hash_djb2(user_id)
      local index = (hash % #backends) + 1
      return backends[index]
    end
    
    -- Extract user ID from JWT
    local jwt = ngx.var.arg_token or ngx.var.http_authorization
    if jwt then
      -- Decode JWT and extract user_id
      local user_id = decode_jwt(jwt).user_id
      local backend = get_backend_for_user(user_id, backends)
      ngx.var.upstream = backend
    end
```

### 2. Graceful Scaling

```bash
#!/bin/bash
# scale-websocket-service.sh

NAMESPACE="rtc"
STATEFULSET="realtime-communication"
NEW_REPLICAS=$1

# Get current replicas
CURRENT_REPLICAS=$(kubectl get statefulset $STATEFULSET -n $NAMESPACE -o jsonpath='{.spec.replicas}')

if [ "$NEW_REPLICAS" -gt "$CURRENT_REPLICAS" ]; then
  echo "Scaling up from $CURRENT_REPLICAS to $NEW_REPLICAS replicas"
  kubectl scale statefulset $STATEFULSET -n $NAMESPACE --replicas=$NEW_REPLICAS
else
  echo "Scaling down from $CURRENT_REPLICAS to $NEW_REPLICAS replicas"
  
  # Mark pods for graceful shutdown
  for i in $(seq $NEW_REPLICAS $((CURRENT_REPLICAS-1))); do
    POD_NAME="$STATEFULSET-$i"
    echo "Draining connections from $POD_NAME"
    
    # Send drain signal to pod
    kubectl exec $POD_NAME -n $NAMESPACE -- curl -X POST http://localhost:8081/admin/drain
    
    # Wait for connections to drain
    while [ $(kubectl exec $POD_NAME -n $NAMESPACE -- curl -s http://localhost:8081/metrics | jq '.active_connections') -gt 0 ]; do
      echo "Waiting for connections to drain from $POD_NAME..."
      sleep 5
    done
  done
  
  # Scale down
  kubectl scale statefulset $STATEFULSET -n $NAMESPACE --replicas=$NEW_REPLICAS
fi
```

### 3. Connection Migration

```rust
// Connection migration during scaling
pub async fn migrate_connections(
    from_node: String,
    to_nodes: Vec<String>,
    batch_size: usize,
) -> Result<MigrationReport, Error> {
    let mut report = MigrationReport::new();
    
    // Get connections to migrate
    let connections = get_node_connections(&from_node).await?;
    let total = connections.len();
    
    // Distribute connections across target nodes
    for (i, chunk) in connections.chunks(batch_size).enumerate() {
        let target_node = &to_nodes[i % to_nodes.len()];
        
        for conn in chunk {
            // Notify client about migration
            conn.send_message(WsMessage::Status {
                status: "migrating".to_string(),
                details: Some(json!({
                    "new_endpoint": target_node,
                    "reconnect_after": 1000,
                })),
            }).await?;
            
            // Transfer session state
            transfer_session_state(conn, target_node).await?;
            
            // Close connection gracefully
            conn.close(CloseCode::Normal).await?;
            
            report.migrated += 1;
        }
        
        // Progress update
        info!(
            "Migration progress: {}/{} connections migrated",
            report.migrated, total
        );
        
        // Rate limit migration
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    Ok(report)
}
```

## Zero-Downtime Deployment

### Blue-Green Deployment

```yaml
# blue-green-deployment.yaml
apiVersion: v1
kind: Service
metadata:
  name: realtime-communication-active
  namespace: rtc
spec:
  selector:
    app: realtime-communication
    version: blue  # Switch between blue/green
  ports:
  - port: 80
    targetPort: 8081
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: realtime-communication-blue
  namespace: rtc
spec:
  replicas: 4
  selector:
    matchLabels:
      app: realtime-communication
      version: blue
  template:
    metadata:
      labels:
        app: realtime-communication
        version: blue
    spec:
      containers:
      - name: app
        image: your-registry/realtime-communication:v1.0.0
        # ... rest of configuration
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: realtime-communication-green
  namespace: rtc
spec:
  replicas: 4
  selector:
    matchLabels:
      app: realtime-communication
      version: green
  template:
    metadata:
      labels:
        app: realtime-communication
        version: green
    spec:
      containers:
      - name: app
        image: your-registry/realtime-communication:v1.1.0
        # ... rest of configuration
```

### Canary Deployment

```yaml
# canary-deployment.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: realtime-communication-canary
  namespace: rtc
  annotations:
    nginx.ingress.kubernetes.io/canary: "true"
    nginx.ingress.kubernetes.io/canary-weight: "10"  # 10% traffic
spec:
  rules:
  - host: ws.example.com
    http:
      paths:
      - path: /ws
        pathType: Prefix
        backend:
          service:
            name: realtime-communication-canary
            port:
              number: 80
```

## Security Hardening

### Network Policies

```yaml
# network-policy.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: realtime-communication-netpol
  namespace: rtc
spec:
  podSelector:
    matchLabels:
      app: realtime-communication
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: ingress
    - podSelector:
        matchLabels:
          app: prometheus
    ports:
    - protocol: TCP
      port: 8081
    - protocol: TCP
      port: 9090
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: redis-cluster
    ports:
    - protocol: TCP
      port: 6379
  - to:
    - namespaceSelector: {}
      podSelector:
        matchLabels:
          k8s-app: kube-dns
    ports:
    - protocol: UDP
      port: 53
```

### Pod Security Policy

```yaml
# pod-security-policy.yaml
apiVersion: policy/v1beta1
kind: PodSecurityPolicy
metadata:
  name: realtime-communication-psp
spec:
  privileged: false
  allowPrivilegeEscalation: false
  requiredDropCapabilities:
  - ALL
  volumes:
  - 'configMap'
  - 'emptyDir'
  - 'projected'
  - 'secret'
  - 'persistentVolumeClaim'
  hostNetwork: false
  hostIPC: false
  hostPID: false
  runAsUser:
    rule: 'MustRunAsNonRoot'
  seLinux:
    rule: 'RunAsAny'
  supplementalGroups:
    rule: 'RunAsAny'
  fsGroup:
    rule: 'RunAsAny'
  readOnlyRootFilesystem: true
```

## Production Checklist

### Pre-Deployment

- [ ] SSL/TLS certificates configured
- [ ] JWT secrets stored in secure vault
- [ ] Redis cluster operational
- [ ] Monitoring stack deployed
- [ ] Load balancer configured
- [ ] Network policies applied
- [ ] Resource limits set
- [ ] Autoscaling configured
- [ ] Backup strategy implemented

### Post-Deployment

- [ ] Health checks passing
- [ ] Metrics being collected
- [ ] Logs aggregated
- [ ] Alerts configured
- [ ] Performance baseline established
- [ ] Load testing completed
- [ ] Disaster recovery tested
- [ ] Documentation updated

### Monitoring Alerts

```yaml
# alerting-rules.yaml
groups:
- name: realtime_communication
  rules:
  - alert: HighConnectionCount
    expr: sum(active_connections) > 9000
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "High connection count"
      description: "Total connections {{ $value }} exceeds threshold"
  
  - alert: HighErrorRate
    expr: rate(errors_total[5m]) > 10
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "High error rate"
      description: "Error rate {{ $value }} errors/sec"
  
  - alert: PodMemoryUsage
    expr: container_memory_usage_bytes / container_spec_memory_limit_bytes > 0.9
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "High memory usage"
      description: "Pod {{ $labels.pod }} memory usage above 90%"
```