# Agent C Tasks: Realtime Communication Service Implementation

## Agent Role

You are Agent C responsible for implementing the Realtime Communication Service business logic. Your primary focus is building the actor-based WebSocket messaging system with session management and persistence.

## Your Tasks

### Task 2.3: Complete Realtime Communication Service

- [x] **2.3.1 Implement message routing logic in `src/handlers.rs`**
  - [x] Create message router actor
  - [x] Implement topic-based routing
  - [x] Add direct message routing
  - [x] Build broadcast message handling

- [x] **2.3.2 Add WebSocket session management**
  - [x] Implement session actor for each connection
  - [x] Add session state persistence to Redis
  - [x] Create session lifecycle management
  - [x] Implement reconnection handling

- [x] **2.3.3 Implement message persistence and history**
  - [x] Store messages in PostgreSQL
  - [x] Add message delivery tracking
  - [x] Implement message history retrieval
  - [x] Create message archival system

- [x] **2.3.4 Add real-time notification delivery**
  - [x] Implement push notification system
  - [x] Add notification preferences
  - [x] Create notification queuing
  - [x] Build retry mechanisms

- [x] **2.3.5 Implement user presence and status tracking**
  - [x] Track online/offline status
  - [x] Implement "typing" indicators
  - [x] Add last seen timestamps
  - [x] Create presence subscription system

## Implementation Plan

### Phase 1: Core Actor System
1. Set up actor hierarchy
2. Implement basic message routing
3. Create WebSocket session management
4. Add connection handling

### Phase 2: Persistence and State
1. Implement message storage
2. Add Redis session state
3. Create history retrieval
4. Build delivery tracking

### Phase 3: Advanced Features
1. Add presence system
2. Implement notifications
3. Complete subscription management
4. Optimize for scale

## Key Files to Modify

- `services/realtime_communication/src/handlers.rs` - Message routing
- `services/realtime_communication/src/actors/` - Actor implementations
- `services/realtime_communication/src/websocket.rs` - WebSocket handling
- `services/realtime_communication/src/session/` - Session management
- `services/realtime_communication/tests/` - Comprehensive tests

## Technical Requirements

### Actor System Design
```
┌─────────────────────────────────────────────┐
│             System Supervisor               │
├─────────────────┬───────────────────────────┤
│   Router Actor  │      Session Manager      │
├─────────────────┼───────────────────────────┤
│ Session Actor 1 │ Session Actor 2 │  ...    │
└─────────────────┴───────────────────────────┘
```

### Message Types
- **DirectMessage**: One-to-one messaging
- **TopicMessage**: Publish to topic subscribers
- **BroadcastMessage**: Send to all connected users
- **PresenceUpdate**: User status changes
- **SystemMessage**: Server notifications

### Actor Responsibilities

**Router Actor:**
- Route messages to appropriate sessions
- Manage topic subscriptions
- Handle broadcast distribution
- Load balance message delivery

**Session Actor:**
- Manage individual WebSocket connection
- Handle message buffering
- Track delivery status
- Manage reconnections

**Session Manager:**
- Track all active sessions
- Handle session discovery
- Manage presence updates
- Coordinate system-wide operations

## Implementation Details

### Message Routing
```rust
// Example message routing
match message {
    Message::Direct { to, content } => {
        session_manager.send_to_user(to, content).await
    }
    Message::Topic { topic, content } => {
        topic_manager.publish(topic, content).await
    }
    Message::Broadcast { content } => {
        session_manager.broadcast(content).await
    }
}
```

### Session Management
- Use Redis for distributed session state
- Implement session timeout handling
- Support graceful reconnection
- Handle authentication via JWT

### Message Persistence
- Store messages with delivery status
- Implement message retention policies
- Add full-text search capabilities
- Support message editing/deletion

## Testing Requirements

- [x] Unit tests for each actor type
- [x] Integration tests for message flow
- [x] Load tests for concurrent connections
- [x] Reconnection scenario tests
- [x] Message delivery guarantee tests

## Performance Requirements

- Support 10,000+ concurrent connections
- Message latency < 50ms
- Horizontal scaling ready
- Efficient memory usage per connection
- Message delivery guarantees

## Success Criteria

1. WebSocket connections are stable and scalable
2. Messages route correctly based on type
3. Presence system accurately tracks user status
4. Message history is persistent and queryable
5. System handles reconnections gracefully
6. All tests pass with >80% coverage

## Dependencies

- Actix-web with actix-web-actors
- Redis for session state
- PostgreSQL for message storage
- JWT for authentication

## Example Implementation

```rust
// Session Actor
pub struct SessionActor {
    id: String,
    user_id: String,
    addr: Addr<RouterActor>,
    hb: Instant,
}

impl Actor for SessionActor {
    type Context = ws::WebsocketContext<Self>;
    
    fn started(&mut self, ctx: &mut Self::Context) {
        // Start heartbeat
        self.hb(ctx);
        
        // Register with router
        self.addr.do_send(Connect {
            addr: ctx.address(),
            user_id: self.user_id.clone(),
        });
    }
}
```

## Notes

- Follow actor model best practices
- Implement supervision strategies
- Use message passing, not shared state
- Consider backpressure handling
- Plan for distributed deployment