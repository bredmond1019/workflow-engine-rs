# Actor System Guide

## Overview

The Real-time Communication Service uses an actor-based architecture to achieve high concurrency, fault tolerance, and scalability. Each WebSocket connection is managed by an isolated actor, ensuring that failures in one connection don't affect others.

## Actor Model Fundamentals

### What is an Actor?

An actor is a computational entity that:
- Processes messages sequentially
- Maintains private state
- Can create other actors
- Can send messages to other actors
- Makes local decisions based on its state

### Why Actors for WebSockets?

1. **Isolation**: Each connection's state is isolated
2. **Concurrency**: Thousands of actors run concurrently
3. **Fault Tolerance**: Actor failures don't cascade
4. **Scalability**: Linear scaling with connections
5. **Message Passing**: Natural fit for WebSocket messages

## Actor Types and Responsibilities

### 1. System Supervisor Actor

The root actor that supervises the entire system.

```rust
pub struct SystemSupervisor {
    server_actor: ActorRef<ServerMessage>,
    metrics_collector: ActorRef<MetricsMessage>,
    config: SystemConfig,
}

impl SystemSupervisor {
    pub async fn start() -> Result<(), ActorError> {
        // Initialize child actors
        let server = ServerActor::new(config.server).start();
        let metrics = MetricsCollector::new().start();
        
        // Start supervision
        self.supervise().await
    }
    
    async fn supervise(&mut self) {
        loop {
            tokio::select! {
                Some(msg) = self.mailbox.recv() => {
                    self.handle_message(msg).await;
                }
                _ = self.health_check() => {
                    self.check_system_health().await;
                }
            }
        }
    }
}
```

**Responsibilities:**
- System initialization and shutdown
- Global configuration management
- Top-level error recovery
- Resource allocation

### 2. Server Actor

Manages the HTTP server and WebSocket upgrades.

```rust
pub struct ServerActor {
    connection_supervisor: ActorRef<ConnectionMessage>,
    server_state: ServerState,
    config: ServerConfig,
}

impl ServerActor {
    pub async fn handle_websocket_upgrade(
        &mut self,
        req: HttpRequest,
        stream: Payload,
    ) -> Result<HttpResponse, ActorError> {
        // Validate connection limit
        if self.server_state.connections >= self.config.max_connections {
            return Err(ActorError::ConnectionLimitReached);
        }
        
        // Create new WebSocket actor
        let ws_actor = self.connection_supervisor
            .create_websocket_actor()
            .await?;
            
        // Upgrade connection
        ws_actor.upgrade(req, stream).await
    }
}
```

**Responsibilities:**
- HTTP server management
- WebSocket upgrade handling
- Connection acceptance/rejection
- Load balancing

### 3. Connection Supervisor Actor

Supervises all WebSocket connection actors.

```rust
pub struct ConnectionSupervisor {
    connections: HashMap<Uuid, ActorRef<WebSocketMessage>>,
    restart_policy: RestartPolicy,
    metrics: ConnectionMetrics,
}

impl ConnectionSupervisor {
    pub async fn create_websocket_actor(&mut self) -> Result<ActorRef<WebSocketMessage>, ActorError> {
        let connection_id = Uuid::new_v4();
        
        let ws_actor = WebSocketActor::new(connection_id)
            .with_supervisor(self.self_ref.clone())
            .start()
            .await?;
            
        self.connections.insert(connection_id, ws_actor.clone());
        self.metrics.increment_active_connections();
        
        Ok(ws_actor)
    }
    
    pub async fn handle_actor_failure(&mut self, connection_id: Uuid, error: ActorError) {
        match self.restart_policy.should_restart(&error) {
            RestartDecision::Restart(delay) => {
                tokio::time::sleep(delay).await;
                self.restart_actor(connection_id).await;
            }
            RestartDecision::Remove => {
                self.remove_actor(connection_id).await;
            }
            RestartDecision::Escalate => {
                self.escalate_to_parent(error).await;
            }
        }
    }
}
```

**Responsibilities:**
- WebSocket actor lifecycle management
- Failure detection and recovery
- Connection pool management
- Restart strategy implementation

### 4. WebSocket Actor

Handles individual WebSocket connections.

```rust
pub struct WebSocketActor {
    connection_id: Uuid,
    session: Session,
    connection_info: ConnectionInfo,
    message_router: ActorRef<RoutingMessage>,
    last_heartbeat: Instant,
    subscriptions: HashSet<String>,
}

impl WebSocketActor {
    pub async fn run(mut self, mut stream: MessageStream) -> Result<(), ActorError> {
        loop {
            tokio::select! {
                // Handle incoming WebSocket messages
                Some(msg) = stream.next() => {
                    self.handle_websocket_message(msg).await?;
                }
                
                // Handle actor mailbox messages
                Some(msg) = self.mailbox.recv() => {
                    self.handle_actor_message(msg).await?;
                }
                
                // Heartbeat check
                _ = tokio::time::sleep_until(self.next_heartbeat()) => {
                    self.send_heartbeat().await?;
                }
                
                // Timeout check
                _ = tokio::time::sleep_until(self.timeout_deadline()) => {
                    return Err(ActorError::ConnectionTimeout);
                }
            }
        }
    }
    
    async fn handle_websocket_message(&mut self, msg: Message) -> Result<(), ActorError> {
        self.last_heartbeat = Instant::now();
        
        match msg {
            Message::Text(text) => {
                let ws_msg: WsMessage = serde_json::from_str(&text)?;
                self.process_client_message(ws_msg).await?;
            }
            Message::Binary(bytes) => {
                self.process_binary_message(&bytes).await?;
            }
            Message::Close(reason) => {
                return Err(ActorError::ConnectionClosed(reason));
            }
            _ => {}
        }
        
        Ok(())
    }
}
```

**Responsibilities:**
- WebSocket message processing
- Connection state management
- Subscription handling
- Heartbeat management
- Message validation

### 5. Message Router Actor

Central hub for message routing and delivery.

```rust
pub struct MessageRouterActor {
    routing_rules: RoutingRules,
    topic_subscribers: HashMap<String, HashSet<Uuid>>,
    user_connections: HashMap<String, HashSet<Uuid>>,
    delivery_queue: VecDeque<DeliveryTask>,
}

impl MessageRouterActor {
    pub async fn route_message(&mut self, msg: RoutingMessage) -> Result<RoutingDecision, ActorError> {
        // Apply routing rules
        let targets = self.determine_targets(&msg).await?;
        
        // Create delivery tasks
        for target in targets {
            let task = DeliveryTask {
                target_id: target.connection_id,
                message: msg.clone(),
                retry_count: 0,
                priority: msg.priority,
            };
            
            self.delivery_queue.push_back(task);
        }
        
        // Process delivery queue
        self.process_deliveries().await?;
        
        Ok(RoutingDecision {
            delivered_to: targets.len(),
            queued: self.delivery_queue.len(),
        })
    }
}
```

**Responsibilities:**
- Message routing logic
- Topic subscription management
- Delivery orchestration
- Rule evaluation

## Message Passing Patterns

### 1. Request-Response Pattern

```rust
// Client sends request
let response = websocket_actor
    .ask(GetConnectionInfo)
    .timeout(Duration::from_secs(5))
    .await?;

// Actor handles request
impl Handler<GetConnectionInfo> for WebSocketActor {
    type Result = ConnectionInfo;
    
    async fn handle(&mut self, _msg: GetConnectionInfo) -> Self::Result {
        self.connection_info.clone()
    }
}
```

### 2. Fire-and-Forget Pattern

```rust
// Send message without waiting for response
websocket_actor
    .tell(BroadcastMessage {
        topic: "notifications".to_string(),
        payload: json!({"alert": "System update"}),
    })
    .await?;
```

### 3. Publish-Subscribe Pattern

```rust
// Subscribe to topic
message_router
    .tell(Subscribe {
        connection_id: self.connection_id,
        topics: vec!["chat.general", "updates.*"],
    })
    .await?;

// Publish to topic
message_router
    .tell(Publish {
        topic: "chat.general",
        message: ChatMessage { /* ... */ },
    })
    .await?;
```

### 4. Request-Stream Pattern

```rust
// Stream of messages
let mut stream = websocket_actor
    .stream(GetMessageHistory {
        topic: "chat.general",
        limit: 100,
    })
    .await?;

while let Some(message) = stream.next().await {
    process_historical_message(message);
}
```

## Actor Supervision Strategies

### 1. One-for-One Strategy

Used for WebSocket actors where failures are independent.

```rust
pub struct OneForOneStrategy {
    max_restarts: u32,
    within: Duration,
}

impl SupervisionStrategy for OneForOneStrategy {
    fn handle_failure(&self, child: &ActorRef, error: &ActorError) -> SupervisionDecision {
        match error {
            ActorError::ConnectionTimeout => SupervisionDecision::Stop,
            ActorError::ProtocolError(_) => SupervisionDecision::Restart,
            ActorError::Fatal(_) => SupervisionDecision::Escalate,
            _ => SupervisionDecision::Resume,
        }
    }
}
```

### 2. All-for-One Strategy

Used for tightly coupled actors like routing components.

```rust
pub struct AllForOneStrategy {
    restart_all: bool,
}

impl SupervisionStrategy for AllForOneStrategy {
    fn handle_failure(&self, child: &ActorRef, error: &ActorError) -> SupervisionDecision {
        match error {
            ActorError::RoutingTableCorrupted => SupervisionDecision::RestartAll,
            ActorError::ConfigurationError => SupervisionDecision::RestartAll,
            _ => SupervisionDecision::Restart,
        }
    }
}
```

### 3. Custom Restart Strategy

```rust
pub struct ExponentialBackoffStrategy {
    base_delay: Duration,
    max_delay: Duration,
    max_attempts: u32,
}

impl RestartStrategy for ExponentialBackoffStrategy {
    fn next_restart_delay(&self, attempt: u32) -> Option<Duration> {
        if attempt >= self.max_attempts {
            return None;
        }
        
        let delay = self.base_delay * 2u32.pow(attempt);
        Some(delay.min(self.max_delay))
    }
}
```

## State Management in Actors

### 1. Immutable State Pattern

```rust
pub struct WebSocketActor {
    state: Arc<RwLock<ConnectionState>>,
}

impl WebSocketActor {
    async fn update_state<F>(&self, f: F) -> Result<(), ActorError>
    where
        F: FnOnce(&mut ConnectionState),
    {
        let mut state = self.state.write().await;
        f(&mut state);
        Ok(())
    }
}
```

### 2. Event Sourcing Pattern

```rust
pub struct EventSourcedActor {
    state: ActorState,
    events: Vec<ActorEvent>,
}

impl EventSourcedActor {
    fn apply_event(&mut self, event: ActorEvent) {
        // Update state based on event
        match event {
            ActorEvent::Connected { user_id, .. } => {
                self.state.user_id = Some(user_id);
            }
            ActorEvent::Subscribed { topic } => {
                self.state.subscriptions.insert(topic);
            }
            // ... other events
        }
        
        // Store event
        self.events.push(event);
    }
    
    fn replay_events(&mut self, events: Vec<ActorEvent>) {
        for event in events {
            self.apply_event(event);
        }
    }
}
```

### 3. Snapshotting

```rust
pub struct SnapshottableActor {
    state: ActorState,
    snapshot_interval: usize,
    events_since_snapshot: usize,
}

impl SnapshottableActor {
    async fn maybe_snapshot(&mut self) {
        if self.events_since_snapshot >= self.snapshot_interval {
            self.save_snapshot().await;
            self.events_since_snapshot = 0;
        }
    }
    
    async fn save_snapshot(&self) {
        let snapshot = ActorSnapshot {
            state: self.state.clone(),
            timestamp: Utc::now(),
        };
        
        self.snapshot_store.save(self.actor_id, snapshot).await;
    }
}
```

## Performance Considerations

### 1. Mailbox Sizing

```rust
pub struct ActorConfig {
    pub mailbox_capacity: usize,  // Default: 1000
    pub priority_mailbox: bool,   // Enable priority queue
    pub overflow_strategy: OverflowStrategy,
}

pub enum OverflowStrategy {
    DropOldest,    // Drop oldest messages
    DropNewest,    // Drop new messages
    Block,         // Block sender
    Backpressure,  // Apply backpressure
}
```

### 2. Message Batching

```rust
impl WebSocketActor {
    async fn batch_messages(&mut self) {
        let mut batch = Vec::new();
        let deadline = Instant::now() + Duration::from_millis(10);
        
        // Collect messages for batching
        while let Ok(Some(msg)) = timeout_at(deadline, self.mailbox.recv()).await {
            batch.push(msg);
            
            if batch.len() >= 100 {
                break;
            }
        }
        
        // Process batch
        if !batch.is_empty() {
            self.process_message_batch(batch).await;
        }
    }
}
```

### 3. Actor Pool Pattern

```rust
pub struct ActorPool<T> {
    actors: Vec<ActorRef<T>>,
    router: Router,
}

impl<T> ActorPool<T> {
    pub fn new(size: usize, factory: impl Fn() -> Actor<T>) -> Self {
        let actors = (0..size)
            .map(|_| factory().start())
            .collect();
            
        Self {
            actors,
            router: Router::RoundRobin,
        }
    }
    
    pub async fn route(&self, message: T) -> Result<(), ActorError> {
        let actor = self.router.select(&self.actors);
        actor.tell(message).await
    }
}
```

### 4. Lazy Actor Creation

```rust
pub struct LazyActorRef<T> {
    factory: Box<dyn Fn() -> Actor<T>>,
    actor: OnceCell<ActorRef<T>>,
}

impl<T> LazyActorRef<T> {
    pub async fn get_or_create(&self) -> &ActorRef<T> {
        self.actor.get_or_init(|| {
            (self.factory)().start()
        })
    }
}
```

## Best Practices

### 1. Actor Design

- **Single Responsibility**: Each actor should have one clear purpose
- **Immutable Messages**: Messages should be immutable
- **No Blocking**: Avoid blocking operations in actors
- **Timeout Handling**: Always set timeouts for external calls

### 2. Message Design

```rust
// Good: Specific, typed messages
pub enum WebSocketMessage {
    Subscribe { topics: Vec<String> },
    Unsubscribe { topics: Vec<String> },
    SendMessage { content: MessageContent },
}

// Bad: Generic, untyped messages
pub struct GenericMessage {
    action: String,
    data: serde_json::Value,
}
```

### 3. Error Handling

```rust
impl WebSocketActor {
    async fn handle_message(&mut self, msg: WebSocketMessage) -> Result<(), ActorError> {
        match msg {
            WebSocketMessage::Subscribe { topics } => {
                self.subscribe_to_topics(topics)
                    .await
                    .map_err(|e| {
                        // Log error with context
                        error!(connection_id = %self.connection_id, error = %e, "Failed to subscribe");
                        
                        // Convert to actor error
                        ActorError::SubscriptionFailed(e)
                    })?;
            }
            // ... other messages
        }
        
        Ok(())
    }
}
```

### 4. Testing Actors

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;
    
    #[tokio::test]
    async fn test_websocket_actor_subscription() {
        // Create test actor
        let (tx, rx) = mpsc::channel(100);
        let mut actor = WebSocketActor::new_test(tx);
        
        // Send subscribe message
        actor.handle_message(WebSocketMessage::Subscribe {
            topics: vec!["test.topic".to_string()],
        }).await.unwrap();
        
        // Verify subscription
        assert!(actor.subscriptions.contains("test.topic"));
        
        // Verify notification sent
        let notification = rx.try_recv().unwrap();
        assert_eq!(notification.type_, "subscription_confirmed");
    }
}
```

## Monitoring Actors

### 1. Actor Metrics

```rust
pub struct ActorMetrics {
    pub messages_processed: Counter,
    pub processing_time: Histogram,
    pub mailbox_size: Gauge,
    pub errors: Counter,
    pub restarts: Counter,
}

impl WebSocketActor {
    async fn record_metrics(&self) {
        self.metrics.mailbox_size.set(self.mailbox.len() as f64);
        self.metrics.messages_processed.inc();
    }
}
```

### 2. Actor Inspection

```rust
pub trait ActorInspector {
    fn get_state(&self) -> ActorStateSnapshot;
    fn get_mailbox_size(&self) -> usize;
    fn get_processing_stats(&self) -> ProcessingStats;
}

impl ActorInspector for WebSocketActor {
    fn get_state(&self) -> ActorStateSnapshot {
        ActorStateSnapshot {
            actor_id: self.connection_id,
            state: "active",
            subscriptions: self.subscriptions.len(),
            last_activity: self.last_heartbeat,
        }
    }
}
```

## Advanced Patterns

### 1. Actor Persistence

```rust
pub trait PersistentActor {
    type State: Serialize + DeserializeOwned;
    
    async fn persist_state(&self) -> Result<(), ActorError>;
    async fn recover_state(&mut self) -> Result<(), ActorError>;
}
```

### 2. Actor Clustering

```rust
pub struct ClusteredActor {
    node_id: String,
    cluster: ClusterRef,
    shard_id: u32,
}

impl ClusteredActor {
    async fn handle_cluster_message(&mut self, msg: ClusterMessage) {
        match msg {
            ClusterMessage::Rebalance { new_shard } => {
                self.migrate_to_shard(new_shard).await;
            }
            ClusterMessage::Replicate { data } => {
                self.replicate_state(data).await;
            }
        }
    }
}
```