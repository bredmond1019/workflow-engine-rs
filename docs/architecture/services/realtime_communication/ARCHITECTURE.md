# Real-time Communication Service Architecture

## Overview

The Real-time Communication Service implements a highly scalable WebSocket-based messaging system using an actor model architecture. This design enables handling 10,000+ concurrent connections while maintaining low latency and high throughput.

## Architecture Diagram

```mermaid
graph TB
    subgraph "Client Layer"
        C1[Web Client]
        C2[Mobile Client]
        C3[Service Client]
    end

    subgraph "Load Balancer"
        LB[HAProxy/Nginx]
    end

    subgraph "WebSocket Server"
        WS[WebSocket Handler]
        HB[Heartbeat Manager]
        AUTH[Auth Middleware]
        RL[Rate Limiter]
    end

    subgraph "Actor System"
        SA[Supervisor Actor]
        WA1[WebSocket Actor 1]
        WA2[WebSocket Actor 2]
        WA3[WebSocket Actor N]
        CM[Connection Manager]
    end

    subgraph "Message Routing"
        MR[Message Router]
        RC[Route Cache]
        RR[Routing Rules]
        TV[Topic Validator]
    end

    subgraph "Persistence Layer"
        REDIS[(Redis)]
        SM[Session Manager]
        MQ[Message Queue]
    end

    subgraph "Monitoring"
        PROM[Prometheus]
        GRAF[Grafana]
        ALERT[AlertManager]
    end

    C1 --> LB
    C2 --> LB
    C3 --> LB
    
    LB --> WS
    WS --> AUTH
    AUTH --> RL
    RL --> SA
    
    SA --> WA1
    SA --> WA2
    SA --> WA3
    
    WA1 --> CM
    WA2 --> CM
    WA3 --> CM
    
    CM --> MR
    MR --> RC
    MR --> RR
    MR --> TV
    
    MR --> REDIS
    CM --> SM
    SM --> REDIS
    MR --> MQ
    MQ --> REDIS
    
    WS --> HB
    HB --> CM
    
    WS --> PROM
    CM --> PROM
    MR --> PROM
    PROM --> GRAF
    PROM --> ALERT
```

## Actor Model Architecture

### Actor Hierarchy

```
                    System Supervisor
                           |
                    Server Actor
                    /      |      \
            Connection  Message   Metrics
            Supervisor  Router    Collector
                |         |          |
         WebSocket    Route      Stats
          Actors     Handlers   Aggregator
```

### Actor Types and Responsibilities

1. **System Supervisor**
   - Root actor supervising the entire system
   - Handles system-wide failures and restarts
   - Manages graceful shutdown

2. **Server Actor**
   - Manages HTTP server and WebSocket upgrades
   - Spawns connection supervisors
   - Handles server-level configuration

3. **Connection Supervisor**
   - Supervises all WebSocket actors
   - Implements restart strategies
   - Manages connection limits

4. **WebSocket Actor**
   - One actor per WebSocket connection
   - Handles message processing for a single client
   - Manages connection state and subscriptions
   - Isolated failure domain

5. **Message Router Actor**
   - Central message routing hub
   - Applies routing rules and filters
   - Manages topic subscriptions

## WebSocket Connection Lifecycle

### Connection Establishment

```mermaid
sequenceDiagram
    participant Client
    participant LB as Load Balancer
    participant WS as WebSocket Server
    participant Auth as Auth Service
    participant SA as Supervisor Actor
    participant WA as WebSocket Actor
    participant CM as Connection Manager

    Client->>LB: HTTP Upgrade Request
    LB->>WS: Forward Request
    WS->>Auth: Validate JWT Token
    Auth-->>WS: Token Valid
    WS->>SA: Request New Actor
    SA->>WA: Spawn WebSocket Actor
    WA->>CM: Register Connection
    CM-->>WA: Connection ID
    WA-->>WS: Actor Ready
    WS-->>Client: WebSocket Established
    WA->>Client: Welcome Message
```

### Message Flow

```mermaid
sequenceDiagram
    participant Client
    participant WA as WebSocket Actor
    participant RL as Rate Limiter
    participant MR as Message Router
    participant CM as Connection Manager
    participant Target as Target Clients

    Client->>WA: Send Message
    WA->>RL: Check Rate Limit
    RL-->>WA: Allowed
    WA->>MR: Route Message
    MR->>CM: Get Target Connections
    CM-->>MR: Connection List
    MR->>Target: Deliver Message
    Target-->>MR: Delivery Confirmation
    MR-->>WA: Routing Complete
    WA->>Client: Send Acknowledgment
```

## Message Routing System

### Routing Architecture

The message routing system implements a flexible, rule-based approach:

1. **Topic-based Routing**
   - Hierarchical topic structure (e.g., `system.notifications.alert`)
   - Wildcard subscriptions supported (`system.*`, `*.alert`)
   - Topic validation and normalization

2. **Direct Routing**
   - User-to-user messaging
   - Connection-specific targeting
   - Presence-aware routing

3. **Broadcast Routing**
   - Room/channel broadcasts
   - Global announcements
   - Filtered broadcasts

### Routing Rules Engine

```rust
pub struct RoutingRule {
    pub name: String,
    pub condition: RuleCondition,
    pub target_filter: Option<TargetFilter>,
    pub message_transform: Option<Transform>,
    pub priority: i32,
}
```

## Session Management Design

### Session States

```mermaid
stateDiagram-v2
    [*] --> Connecting: WebSocket Upgrade
    Connecting --> Authenticating: Connection Established
    Authenticating --> Active: Auth Success
    Authenticating --> Disconnected: Auth Failed
    Active --> Inactive: No Activity
    Inactive --> Active: Activity Detected
    Inactive --> Disconnecting: Timeout
    Active --> Disconnecting: Client Disconnect
    Disconnecting --> Disconnected: Cleanup Complete
    Disconnected --> [*]
```

### Session Persistence

- Sessions stored in Redis with TTL
- Automatic session recovery on reconnection
- Message replay for missed messages
- Presence tracking across multiple connections

## Scaling Architecture

### Horizontal Scaling

```mermaid
graph LR
    subgraph "Load Balancer Layer"
        LB1[LB Instance 1]
        LB2[LB Instance 2]
    end

    subgraph "Service Instances"
        S1[Server 1]
        S2[Server 2]
        S3[Server 3]
        S4[Server N]
    end

    subgraph "Redis Cluster"
        R1[(Redis Master)]
        R2[(Redis Replica 1)]
        R3[(Redis Replica 2)]
    end

    LB1 --> S1
    LB1 --> S2
    LB2 --> S3
    LB2 --> S4

    S1 --> R1
    S2 --> R1
    S3 --> R1
    S4 --> R1

    R1 --> R2
    R1 --> R3
```

### Scaling Strategies

1. **Connection Distribution**
   - Sticky sessions via consistent hashing
   - Connection migration support
   - Load-based routing

2. **Message Distribution**
   - Redis pub/sub for cross-instance messaging
   - Sharded topics for high-volume channels
   - Local delivery optimization

3. **State Distribution**
   - Distributed session storage in Redis
   - Connection registry sharding
   - Eventual consistency for presence

## High Concurrency Design

### Concurrency Model

1. **Actor Isolation**
   - Each connection runs in isolated actor
   - No shared mutable state
   - Message passing for communication

2. **Async I/O**
   - Tokio runtime with work-stealing scheduler
   - Non-blocking I/O operations
   - Efficient task scheduling

3. **Lock-Free Data Structures**
   - DashMap for concurrent hash maps
   - Arc<RwLock> for shared state
   - Atomic operations for counters

### Performance Optimizations

1. **Connection Pooling**
   - Redis connection pooling
   - HTTP client connection reuse
   - Database connection management

2. **Message Batching**
   - Batch small messages together
   - Nagle's algorithm for TCP
   - Configurable batch windows

3. **Zero-Copy Operations**
   - Efficient buffer management
   - Direct memory access
   - Minimal allocations

## Failure Handling

### Actor Supervision Strategies

1. **One-for-One**
   - Individual actor restart
   - Isolated failure domains
   - Used for WebSocket actors

2. **All-for-One**
   - Restart all children
   - Used for critical subsystems
   - Applied to routing components

3. **Rest-for-One**
   - Restart failed and subsequent
   - Ordered dependency handling
   - Used for pipeline actors

### Circuit Breaker Pattern

```rust
pub struct CircuitBreaker {
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub timeout: Duration,
    pub state: CircuitState,
}

pub enum CircuitState {
    Closed,      // Normal operation
    Open,        // Failing, reject requests
    HalfOpen,    // Testing recovery
}
```

## Security Architecture

### Authentication Flow

1. **JWT Validation**
   - Token extraction from multiple sources
   - Signature verification
   - Claims validation

2. **Authorization**
   - Role-based access control (RBAC)
   - Permission checking
   - Resource-level authorization

3. **Connection Security**
   - TLS/SSL encryption
   - Origin validation
   - CSRF protection

## Monitoring and Observability

### Metrics Collection

1. **Connection Metrics**
   - Active connections
   - Connection rate
   - Connection duration

2. **Message Metrics**
   - Messages per second
   - Message latency
   - Routing performance

3. **System Metrics**
   - CPU and memory usage
   - Actor mailbox sizes
   - Error rates

### Distributed Tracing

- OpenTelemetry integration
- Request correlation IDs
- Cross-service tracing
- Performance profiling