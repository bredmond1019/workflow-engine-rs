# Knowledge Graph Service Architecture

## System Overview

The Knowledge Graph Service is a high-performance microservice that manages concept relationships and learning paths using Dgraph as the primary graph database. The architecture is designed for scalability, reliability, and performance.

```mermaid
graph TB
    subgraph "Client Layer"
        API[REST/GraphQL API]
        WS[WebSocket Support]
    end
    
    subgraph "Service Layer"
        Router[Request Router]
        Auth[Authentication]
        RateLimit[Rate Limiter]
        
        subgraph "Core Components"
            QueryEngine[Query Engine]
            AlgoEngine[Algorithm Engine]
            CacheManager[Cache Manager]
            
            subgraph "Algorithms"
                ShortestPath[Shortest Path]
                PageRank[PageRank]
                TopSort[Topological Sort]
                Traversal[Graph Traversal]
            end
        end
    end
    
    subgraph "Data Layer"
        ConnPool[Connection Pool]
        DgraphClient[Dgraph Client]
        RedisClient[Redis Client]
        PgClient[PostgreSQL Client]
        
        subgraph "Storage"
            Dgraph[(Dgraph Cluster)]
            Redis[(Redis Cache)]
            PostgreSQL[(PostgreSQL)]
        end
    end
    
    API --> Router
    WS --> Router
    Router --> Auth
    Auth --> RateLimit
    RateLimit --> QueryEngine
    RateLimit --> AlgoEngine
    
    QueryEngine --> CacheManager
    AlgoEngine --> CacheManager
    
    QueryEngine --> ConnPool
    AlgoEngine --> ConnPool
    CacheManager --> RedisClient
    
    ConnPool --> DgraphClient
    DgraphClient --> Dgraph
    RedisClient --> Redis
    QueryEngine --> PgClient
    PgClient --> PostgreSQL
    
    ShortestPath --> AlgoEngine
    PageRank --> AlgoEngine
    TopSort --> AlgoEngine
    Traversal --> AlgoEngine
```

## Dgraph Cluster Architecture

### Single-Node Development Setup

```mermaid
graph LR
    subgraph "Dgraph Single Node"
        Zero[Dgraph Zero<br/>Port: 5080/6080]
        Alpha[Dgraph Alpha<br/>Port: 8080/9080]
        Ratel[Ratel UI<br/>Port: 8000]
    end
    
    Service[Knowledge Graph Service] --> Alpha
    Zero --> Alpha
    Ratel --> Alpha
```

### Production Multi-Node Setup

```mermaid
graph TB
    subgraph "Dgraph Production Cluster"
        subgraph "Zero Group"
            Z1[Zero Leader]
            Z2[Zero Follower 1]
            Z3[Zero Follower 2]
        end
        
        subgraph "Alpha Group 1"
            A1[Alpha Leader]
            A2[Alpha Follower 1]
            A3[Alpha Follower 2]
        end
        
        subgraph "Alpha Group 2"
            A4[Alpha Leader]
            A5[Alpha Follower 1]
            A6[Alpha Follower 2]
        end
    end
    
    LoadBalancer[Load Balancer] --> A1
    LoadBalancer --> A4
    
    Z1 -.-> Z2
    Z1 -.-> Z3
    
    A1 -.-> A2
    A1 -.-> A3
    A4 -.-> A5
    A4 -.-> A6
    
    Z1 --> A1
    Z1 --> A4
```

## Connection Pooling Strategy

The service implements a sophisticated connection pooling mechanism to optimize Dgraph interactions:

### Pool Architecture

```mermaid
graph TB
    subgraph "Connection Pool"
        Manager[Pool Manager]
        
        subgraph "Active Connections"
            C1[Connection 1]
            C2[Connection 2]
            C3[Connection 3]
        end
        
        subgraph "Idle Connections"
            C4[Connection 4]
            C5[Connection 5]
        end
        
        HealthChecker[Health Checker]
        Stats[Statistics Collector]
    end
    
    Request[Incoming Request] --> Manager
    Manager --> C1
    Manager --> Stats
    HealthChecker --> C1
    HealthChecker --> C2
    HealthChecker --> C3
    HealthChecker --> C4
    HealthChecker --> C5
```

### Connection Lifecycle

1. **Connection Creation**: New connections created on demand up to max pool size
2. **Health Checking**: Background task validates connections every 30 seconds
3. **Connection Recycling**: Idle connections closed after timeout
4. **Retry Logic**: Exponential backoff for failed operations
5. **Statistics Tracking**: Performance metrics for monitoring

## Caching Layer Design

### Multi-Level Caching Strategy

```mermaid
graph LR
    subgraph "Cache Hierarchy"
        L1[L1: In-Memory Cache<br/>LRU, 100MB]
        L2[L2: Redis Cache<br/>Distributed, 1GB]
        L3[L3: Dgraph<br/>Source of Truth]
    end
    
    Query[Query Request] --> L1
    L1 -->|Miss| L2
    L2 -->|Miss| L3
    L3 -->|Data| L2
    L2 -->|Data| L1
    L1 -->|Data| Response[Response]
```

### Cache Invalidation

- **TTL-based**: Automatic expiration after configurable time
- **Event-based**: Invalidation on concept updates
- **Manual**: API endpoints for cache clearing
- **Partial**: Selective invalidation by category/tag

## Graph Algorithm Implementations

### Algorithm Processing Pipeline

```mermaid
graph TB
    Request[Algorithm Request] --> Validator[Input Validator]
    Validator --> Selector[Algorithm Selector]
    
    Selector --> Dijkstra[Dijkstra's Algorithm]
    Selector --> AStar[A* Algorithm]
    Selector --> PR[PageRank]
    Selector --> TSort[Topological Sort]
    
    Dijkstra --> Optimizer[Result Optimizer]
    AStar --> Optimizer
    PR --> Optimizer
    TSort --> Optimizer
    
    Optimizer --> Cache[Cache Results]
    Cache --> Response[Response]
```

### Algorithm Complexity

| Algorithm | Time Complexity | Space Complexity | Use Case |
|-----------|----------------|------------------|----------|
| Dijkstra | O((V + E) log V) | O(V) | Optimal learning paths |
| A* | O((V + E) log V) | O(V) | Heuristic-guided paths |
| PageRank | O(k(V + E)) | O(V) | Concept importance |
| DFS/BFS | O(V + E) | O(V) | Graph exploration |
| Topological Sort | O(V + E) | O(V) | Prerequisite ordering |

## Data Flow Architecture

### Query Processing Flow

```mermaid
sequenceDiagram
    participant Client
    participant API
    participant Auth
    participant QueryEngine
    participant Cache
    participant Dgraph
    
    Client->>API: GraphQL Query
    API->>Auth: Validate Token
    Auth->>API: Authorization OK
    API->>QueryEngine: Parse Query
    QueryEngine->>Cache: Check Cache
    
    alt Cache Hit
        Cache->>QueryEngine: Return Data
    else Cache Miss
        QueryEngine->>Dgraph: Execute Query
        Dgraph->>QueryEngine: Return Results
        QueryEngine->>Cache: Store Results
    end
    
    QueryEngine->>API: Format Response
    API->>Client: Return Data
```

### Learning Path Generation Flow

```mermaid
sequenceDiagram
    participant User
    participant API
    participant AlgoEngine
    participant GraphDB
    participant Cache
    
    User->>API: Request Learning Path
    API->>AlgoEngine: Generate Path Request
    AlgoEngine->>GraphDB: Fetch Concept Graph
    GraphDB->>AlgoEngine: Graph Data
    
    AlgoEngine->>AlgoEngine: Run Shortest Path
    AlgoEngine->>AlgoEngine: Apply Constraints
    AlgoEngine->>AlgoEngine: Optimize Path
    
    AlgoEngine->>Cache: Store Path
    AlgoEngine->>API: Return Path
    API->>User: Learning Path Response
```

## Security Architecture

### Authentication & Authorization

```mermaid
graph TB
    subgraph "Security Layer"
        JWT[JWT Validator]
        RBAC[Role-Based Access]
        RateLimit[Rate Limiter]
        
        subgraph "Access Control"
            Read[Read Permissions]
            Write[Write Permissions]
            Admin[Admin Permissions]
        end
    end
    
    Request[API Request] --> JWT
    JWT --> RBAC
    RBAC --> RateLimit
    
    RBAC --> Read
    RBAC --> Write
    RBAC --> Admin
    
    RateLimit --> Service[Service Layer]
```

## Performance Optimizations

### Query Optimization Strategies

1. **Index Management**
   - Full-text search indexes on name, description
   - Exact match indexes on category, difficulty
   - Vector indexes for similarity search

2. **Query Planning**
   - Query cost estimation
   - Automatic query rewriting
   - Parallel query execution

3. **Batch Operations**
   - Bulk concept creation
   - Batch relationship updates
   - Transaction batching

### Memory Management

```mermaid
graph TB
    subgraph "Memory Allocation"
        Pool[Connection Pool<br/>~100MB]
        Cache[In-Memory Cache<br/>100MB]
        Buffers[Query Buffers<br/>50MB]
        Working[Working Memory<br/>200MB]
    end
    
    Total[Total: ~450MB Base]
```

## Monitoring & Observability

### Metrics Collection

```mermaid
graph LR
    subgraph "Metrics Pipeline"
        Service[Knowledge Graph Service]
        Collector[Prometheus Collector]
        Metrics[(Prometheus)]
        Grafana[Grafana Dashboards]
    end
    
    Service -->|Export| Collector
    Collector -->|Scrape| Metrics
    Metrics -->|Query| Grafana
```

### Key Metrics

- **Performance Metrics**
  - Query latency (p50, p95, p99)
  - Algorithm execution time
  - Cache hit/miss ratio
  - Connection pool utilization

- **Business Metrics**
  - Concepts created/updated
  - Learning paths generated
  - Search queries processed
  - Active user sessions

- **System Metrics**
  - CPU and memory usage
  - Network I/O
  - Dgraph cluster health
  - Error rates by type

## Deployment Architecture

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: knowledge-graph-service
spec:
  replicas: 3
  selector:
    matchLabels:
      app: knowledge-graph
  template:
    spec:
      containers:
      - name: service
        image: knowledge-graph:latest
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
```

### Service Mesh Integration

The service is designed to work within a service mesh (Istio/Linkerd) for:
- Automatic mTLS
- Circuit breaking
- Load balancing
- Distributed tracing

## Scalability Considerations

### Horizontal Scaling

- **Service Instances**: Stateless design allows linear scaling
- **Dgraph Cluster**: Add Alpha nodes for read scaling
- **Cache Layer**: Redis cluster for distributed caching
- **Load Balancing**: Round-robin with health checks

### Vertical Scaling

- **Connection Pool**: Increase max connections
- **Cache Size**: Expand in-memory and Redis cache
- **Algorithm Workers**: More concurrent algorithm executions
- **Query Complexity**: Tune for larger graphs

## Failure Handling

### Resilience Patterns

1. **Circuit Breaker**: Prevent cascading failures
2. **Retry with Backoff**: Handle transient errors
3. **Fallback Mechanisms**: Degraded service options
4. **Health Checks**: Proactive failure detection
5. **Graceful Degradation**: Partial service availability

### Recovery Procedures

- Automatic connection pool recovery
- Cache rebuild on corruption
- Transaction rollback support
- Point-in-time recovery from backups