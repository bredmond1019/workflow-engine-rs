# CLAUDE.md - Real-time Communication Service

This file provides guidance to Claude Code (claude.ai/code) when working with the Real-time Communication microservice in this repository.

## Service Overview

The Real-time Communication Service is a high-performance WebSocket-based messaging microservice built with Rust and Actix-Web. It implements an actor model architecture designed to handle 10,000+ concurrent connections with low latency and high throughput. This service provides real-time messaging capabilities for the AI workflow orchestration system.

## Purpose and Role

This service acts as the real-time communication backbone for the workflow engine, enabling:
- Real-time updates about workflow execution status
- Live notifications for system events
- Bi-directional communication between AI agents and users
- Collaborative features for multi-user workflows
- Event streaming for workflow monitoring

## Key Components

### 1. Actor System (`src/actors/`)
- **System Supervisor**: Root actor managing the entire system lifecycle
- **Connection Supervisor**: Manages all WebSocket connection actors
- **WebSocket Actor**: One actor per connection, handles individual client state
- **Message Router Actor**: Central hub for message routing and delivery
- **Session Actor**: Manages user sessions and state persistence

### 2. WebSocket Server (`src/server.rs`)
- Actix-web based HTTP server with WebSocket upgrade support
- Connection acceptance and authentication
- Health check and metrics endpoints
- Graceful shutdown handling

### 3. Message Routing (`src/routing/`)
- Topic-based publish/subscribe pattern
- Direct user-to-user messaging
- Rule-based routing engine
- Message filtering and transformation
- Priority-based delivery

### 4. Session Management (`src/session.rs`)
- User session lifecycle management
- State persistence in Redis
- Session recovery on reconnection
- Message replay for missed messages

### 5. Presence System (`src/presence.rs`)
- Real-time user presence tracking
- Connection status monitoring
- Multi-device presence aggregation
- Last seen timestamps

### 6. Protection Mechanisms (`src/protection/`)
- **Rate Limiter**: Multi-level rate limiting (connection, user, global)
- **Circuit Breaker**: Prevents cascading failures
- **Backpressure**: Flow control for overwhelmed clients

## Actor Model Implementation

### Actor Hierarchy
```
System Supervisor
    ├── Server Actor
    ├── Connection Supervisor
    │   └── WebSocket Actors (1 per connection)
    ├── Message Router Actor
    └── Metrics Collector Actor
```

### Supervision Strategies
- **One-for-One**: Individual WebSocket actor failures don't affect others
- **All-for-One**: Critical system component failures trigger full restart
- **Exponential Backoff**: Restart delays increase with repeated failures

### Message Passing
- Typed messages between actors
- Async message handling with Tokio
- Mailbox-based communication
- Priority message queues

## WebSocket Protocol

### Connection Establishment
```
ws://localhost:8081/ws?token=<JWT_TOKEN>
```

### Message Format
```json
{
  "type": "MessageType",
  "data": { ... },
  "id": "optional_message_id",
  "timestamp": 1234567890
}
```

### Client Commands
- `Ping/Pong`: Heartbeat mechanism
- `Subscribe`: Topic subscription
- `Unsubscribe`: Topic unsubscription
- `Broadcast`: Send to topic subscribers
- `DirectMessage`: Send to specific user

### Server Messages
- `Status`: Connection and operation status
- `Broadcast`: Topic message delivery
- `DirectMessage`: Direct message delivery
- `Notification`: System notifications
- `Error`: Error responses

## API Endpoints

### WebSocket
- `GET /ws` - WebSocket connection endpoint (requires JWT token)

### HTTP
- `GET /health` - Service health check
- `GET /metrics` - Prometheus metrics
- `GET /info` - Server information and configuration

## Integration with Main Workflow Engine

### Event Notifications
The service publishes real-time events about:
- Workflow state changes
- Node execution status
- Error conditions
- System alerts

### Communication Patterns
1. **Workflow Updates**: Engine publishes workflow events to topics
2. **Agent Communication**: AI agents subscribe to task topics
3. **User Notifications**: Users receive updates on their workflows
4. **System Monitoring**: Admin topics for system-wide events

### Integration Points
- Redis pub/sub for cross-service messaging
- JWT tokens shared with main API service
- Common user/tenant context
- Metrics aggregation in Prometheus

## Testing Approach

### Unit Tests
```bash
# Run unit tests
cd services/realtime_communication
cargo test
```

### Integration Tests
```bash
# Start Redis for testing
docker run -d -p 6379:6379 redis:7-alpine

# Run integration tests
cargo test -- --ignored
```

### Load Testing
```bash
# WebSocket load test with 1000 concurrent connections
cargo test test_concurrent_connections -- --ignored --nocapture
```

### Test Categories
- Actor supervision and recovery
- Message routing correctness
- Rate limiting behavior
- Circuit breaker triggers
- Session persistence
- Presence tracking

## Common Development Tasks

### 1. Adding a New Message Type

1. Define message in `src/messaging.rs`:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WsMessage {
    // ... existing messages
    YourNewMessage { field: String },
}
```

2. Add handler in WebSocket actor:
```rust
match msg {
    WsMessage::YourNewMessage { field } => {
        self.handle_your_message(field).await?;
    }
    // ...
}
```

### 2. Creating a New Actor

1. Define actor in `src/actors/your_actor.rs`:
```rust
pub struct YourActor {
    // state fields
}

impl Actor for YourActor {
    type Context = Context<Self>;
}
```

2. Register with supervisor in `src/actors/manager.rs`

### 3. Adding Routing Rules

1. Define rule in `src/routing/rules.rs`:
```rust
pub struct YourRoutingRule;

impl RoutingRule for YourRoutingRule {
    fn evaluate(&self, msg: &RoutingMessage) -> bool {
        // your logic
    }
}
```

2. Register in router initialization

### 4. Implementing Custom Authentication

1. Extend JWT validator in `src/auth/jwt.rs`
2. Add custom claims validation
3. Update middleware in `src/auth/middleware.rs`

### 5. Adding Metrics

1. Define metric in `src/server.rs`:
```rust
lazy_static! {
    static ref YOUR_METRIC: IntCounter = 
        register_int_counter!("your_metric", "Description").unwrap();
}
```

2. Update metric in relevant code:
```rust
YOUR_METRIC.inc();
```

### 6. Debugging WebSocket Connections

1. Enable debug logging:
```bash
RUST_LOG=realtime_communication=debug cargo run
```

2. Use WebSocket test client:
```bash
# Install wscat
npm install -g wscat

# Connect with token
wscat -c "ws://localhost:8081/ws?token=YOUR_JWT_TOKEN"
```

3. Monitor actor metrics at `/metrics` endpoint

### 7. Performance Tuning

1. Adjust mailbox sizes in `src/actors/messages.rs`
2. Configure connection pools in `src/persistence.rs`
3. Tune rate limits in `src/protection/rate_limiter.rs`
4. Modify batch sizes in message routing

## Environment Variables

```bash
# Server configuration
HOST=0.0.0.0
PORT=8081
MAX_CONNECTIONS=10000

# Authentication
JWT_SECRET=your-secret-key

# Redis connection
REDIS_URL=redis://localhost:6379

# Timeouts
HEARTBEAT_INTERVAL=30s
CLIENT_TIMEOUT=60s

# Rate limiting
RATE_LIMIT_ENABLED=true
RATE_LIMIT_WINDOW=60s
RATE_LIMIT_MAX_REQUESTS=100

# Logging
RUST_LOG=info,realtime_communication=debug
```

## Architecture Patterns

### 1. Actor Isolation
- Each connection runs in an isolated actor
- Failures don't cascade between connections
- State is encapsulated within actors

### 2. Message-Driven Design
- All communication via typed messages
- No shared mutable state
- Async message processing

### 3. Supervision Trees
- Hierarchical failure handling
- Automatic recovery strategies
- Graceful degradation

### 4. Backpressure Handling
- Mailbox size limits
- Client-side flow control
- Message dropping strategies

### 5. Multi-Transport Support
- WebSocket for real-time
- HTTP for REST operations
- Redis pub/sub for internal

## Performance Considerations

### Connection Scaling
- Use connection pooling for Redis
- Implement lazy actor creation
- Configure OS limits for file descriptors

### Message Throughput
- Batch small messages
- Use binary protocol for large payloads
- Implement message compression

### Memory Management
- Limit mailbox sizes
- Implement message TTLs
- Use Arc for shared immutable data

### CPU Optimization
- Use work-stealing scheduler
- Avoid blocking operations
- Profile with flamegraph

## Security Considerations

### Authentication
- JWT validation on connection
- Token refresh handling
- Role-based access control

### Authorization
- Topic-level permissions
- Message filtering by user role
- Resource-based access control

### Data Protection
- TLS for production WebSocket
- Message encryption options
- Audit logging for sensitive operations

## Monitoring and Debugging

### Key Metrics
- `websocket_connections_active`: Current connection count
- `websocket_messages_per_second`: Message throughput
- `actor_mailbox_size`: Actor queue depths
- `circuit_breaker_state`: Protection status

### Health Checks
- `/health` - Basic liveness check
- `/health/ready` - Readiness with dependencies
- `/metrics` - Prometheus metrics

### Logging
- Structured JSON logs
- Correlation IDs for request tracing
- Actor-specific log contexts

### Debugging Tools
- Actor state inspection via admin API
- Message trace logging
- Connection dump utilities