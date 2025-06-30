# Troubleshooting Guide

## Overview

This guide helps diagnose and resolve common issues with the Real-time Communication Service. It covers connection problems, message delivery issues, performance bottlenecks, and debugging strategies.

## Common Issues and Solutions

### Connection Issues

#### 1. WebSocket Connection Failed

**Symptoms:**
- Client receives connection error
- WebSocket upgrade fails
- 401/403 errors during connection

**Diagnostic Steps:**

```bash
# Check if service is running
curl http://localhost:8081/health

# Test WebSocket connection with wscat
wscat -c "ws://localhost:8081/ws?token=YOUR_JWT_TOKEN"

# Check logs for errors
docker logs realtime-communication | grep ERROR

# Verify JWT token
jwt decode YOUR_JWT_TOKEN
```

**Common Causes and Solutions:**

1. **Invalid JWT Token**
   ```javascript
   // Verify token expiration
   const decoded = jwt.decode(token);
   if (decoded.exp < Date.now() / 1000) {
     console.log('Token expired');
   }
   
   // Solution: Refresh token
   const newToken = await refreshAccessToken(refreshToken);
   ```

2. **Server at Capacity**
   ```bash
   # Check current connections
   curl http://localhost:8081/metrics | jq '.active_connections'
   
   # Solution: Scale service
   kubectl scale statefulset realtime-communication --replicas=6
   ```

3. **Network/Firewall Issues**
   ```bash
   # Test connectivity
   telnet localhost 8081
   
   # Check firewall rules
   sudo iptables -L -n | grep 8081
   
   # Solution: Open required ports
   sudo ufw allow 8081/tcp
   ```

#### 2. Connection Drops Frequently

**Symptoms:**
- Connections close unexpectedly
- Frequent reconnection attempts
- Heartbeat timeouts

**Diagnostic Steps:**

```javascript
// Client-side connection monitoring
class ConnectionMonitor {
  constructor(client) {
    this.client = client;
    this.disconnects = [];
    this.lastPing = Date.now();
    
    client.on('disconnect', (reason) => {
      this.disconnects.push({
        time: new Date(),
        reason: reason,
        duration: Date.now() - this.connectionStart
      });
      
      console.log('Disconnect pattern:', this.analyzeDisconnects());
    });
    
    client.on('pong', () => {
      const latency = Date.now() - this.lastPing;
      console.log('Ping latency:', latency, 'ms');
    });
  }
  
  analyzeDisconnects() {
    // Check for patterns
    const recentDisconnects = this.disconnects.filter(
      d => Date.now() - d.time < 300000 // Last 5 minutes
    );
    
    return {
      count: recentDisconnects.length,
      avgDuration: recentDisconnects.reduce((acc, d) => acc + d.duration, 0) / recentDisconnects.length,
      reasons: recentDisconnects.map(d => d.reason)
    };
  }
}
```

**Common Causes and Solutions:**

1. **Heartbeat Timeout**
   ```toml
   # Increase timeout in config
   [websocket]
   heartbeat_interval_secs = 60  # Increase from 30
   client_timeout_secs = 120     # Increase from 60
   ```

2. **Network Instability**
   ```javascript
   // Implement aggressive keepalive
   setInterval(() => {
     if (ws.readyState === WebSocket.OPEN) {
       ws.send(JSON.stringify({ type: 'Ping', data: { timestamp: Date.now() } }));
     }
   }, 15000); // Every 15 seconds
   ```

3. **Proxy/LB Timeout**
   ```nginx
   # Increase proxy timeouts
   location /ws {
     proxy_connect_timeout 7d;
     proxy_send_timeout 7d;
     proxy_read_timeout 7d;
   }
   ```

#### 3. Authentication Failures

**Symptoms:**
- 401 Unauthorized errors
- Token validation failures
- Permission denied messages

**Diagnostic Tools:**

```bash
#!/bin/bash
# jwt-debug.sh

TOKEN=$1

# Decode header and payload
HEADER=$(echo $TOKEN | cut -d. -f1 | base64 -d 2>/dev/null)
PAYLOAD=$(echo $TOKEN | cut -d. -f2 | base64 -d 2>/dev/null)

echo "Header: $HEADER"
echo "Payload: $PAYLOAD"

# Check expiration
EXP=$(echo $PAYLOAD | jq -r '.exp')
NOW=$(date +%s)

if [ "$EXP" -lt "$NOW" ]; then
  echo "Token EXPIRED"
else
  echo "Token valid for $((EXP - NOW)) seconds"
fi
```

**Solutions:**

1. **Clock Skew**
   ```rust
   // Add clock skew tolerance
   let mut validation = Validation::new(Algorithm::HS256);
   validation.leeway = 60; // 60 seconds tolerance
   ```

2. **Wrong Secret**
   ```bash
   # Verify JWT secret matches
   echo $JWT_SECRET | sha256sum
   kubectl get secret jwt-secret -o jsonpath='{.data.secret}' | base64 -d | sha256sum
   ```

### Message Delivery Problems

#### 1. Messages Not Delivered

**Symptoms:**
- Broadcast messages not received
- Direct messages lost
- Acknowledgments not received

**Diagnostic Steps:**

```rust
// Server-side message tracing
#[derive(Debug)]
pub struct MessageTrace {
    pub message_id: String,
    pub timestamp: Instant,
    pub stage: TraceStage,
    pub details: String,
}

pub enum TraceStage {
    Received,
    Validated,
    Routed,
    Queued,
    Delivered,
    Failed,
}

// Enable message tracing
impl MessageRouter {
    pub async fn trace_message(&self, msg: &RoutingMessage) -> Vec<MessageTrace> {
        let traces = self.message_traces.get(&msg.id).await;
        traces.unwrap_or_default()
    }
}
```

**Common Causes:**

1. **Not Subscribed to Topic**
   ```javascript
   // Verify subscription
   client.on('Status', (data) => {
     if (data.status === 'subscribed') {
       console.log('Subscribed to:', data.details.topics);
       subscribedTopics = data.details.topics;
     }
   });
   
   // Before sending
   if (!subscribedTopics.includes(topic)) {
     console.error('Not subscribed to topic:', topic);
   }
   ```

2. **Message Size Limit**
   ```javascript
   // Check message size
   function checkMessageSize(message) {
     const size = new Blob([JSON.stringify(message)]).size;
     const limit = 65536; // 64KB default
     
     if (size > limit) {
       console.error(`Message too large: ${size} bytes (limit: ${limit})`);
       return false;
     }
     return true;
   }
   ```

3. **Rate Limiting**
   ```bash
   # Check rate limit metrics
   curl http://localhost:8081/metrics | jq '.rate_limit_stats'
   
   # Monitor rate limit headers
   curl -i http://localhost:8081/api/endpoint | grep -i rate
   ```

#### 2. Message Ordering Issues

**Symptoms:**
- Messages arrive out of order
- Duplicate messages
- Missing sequence numbers

**Solution - Message Ordering:**

```javascript
class OrderedMessageHandler {
  constructor() {
    this.sequences = new Map(); // topic -> next expected sequence
    this.buffers = new Map();   // topic -> out-of-order messages
  }
  
  handleMessage(topic, message) {
    const seq = message.sequence;
    const expected = this.sequences.get(topic) || 0;
    
    if (seq === expected) {
      // Process message
      this.processMessage(message);
      this.sequences.set(topic, seq + 1);
      
      // Check buffer for next messages
      this.processBuffered(topic);
    } else if (seq > expected) {
      // Buffer out-of-order message
      this.bufferMessage(topic, message);
    } else {
      // Duplicate, ignore
      console.warn('Duplicate message:', seq);
    }
  }
  
  processBuffered(topic) {
    const buffer = this.buffers.get(topic) || [];
    const expected = this.sequences.get(topic);
    
    // Sort by sequence
    buffer.sort((a, b) => a.sequence - b.sequence);
    
    // Process sequential messages
    while (buffer.length > 0 && buffer[0].sequence === expected) {
      const msg = buffer.shift();
      this.processMessage(msg);
      this.sequences.set(topic, expected + 1);
    }
  }
}
```

### Actor System Debugging

#### 1. Actor Mailbox Overflow

**Symptoms:**
- Slow message processing
- High memory usage
- Backpressure warnings

**Diagnostic Commands:**

```rust
// Actor mailbox monitoring
pub async fn monitor_actor_health(actor_id: Uuid) -> ActorHealth {
    let actor = actor_registry.get(actor_id).await;
    
    ActorHealth {
        mailbox_size: actor.mailbox_size(),
        processing_time: actor.avg_processing_time(),
        error_rate: actor.error_rate(),
        restart_count: actor.restart_count(),
        last_error: actor.last_error(),
    }
}
```

**Solutions:**

1. **Increase Mailbox Size**
   ```toml
   [actor_system]
   mailbox_capacity = 5000  # Increase from 1000
   overflow_strategy = "drop_oldest"
   ```

2. **Add Actor Pool**
   ```rust
   // Scale out with actor pool
   let pool = ActorPool::new(10, || {
       MessageProcessorActor::new()
   });
   
   // Distribute load
   pool.route(message).await?;
   ```

#### 2. Actor Restart Loops

**Symptoms:**
- Actors restarting frequently
- Escalating errors
- System instability

**Debug Strategy:**

```rust
// Implement detailed supervision logging
impl Supervisor {
    async fn handle_failure(&mut self, child_id: Uuid, error: ActorError) {
        error!(
            child_id = %child_id,
            error = %error,
            restart_count = self.restart_counts.get(&child_id),
            "Actor failed"
        );
        
        // Check for restart loops
        let count = self.restart_counts.entry(child_id).or_insert(0);
        *count += 1;
        
        if *count > 5 {
            error!("Actor {} in restart loop, stopping", child_id);
            self.stop_actor(child_id).await;
            
            // Alert operations
            self.send_alert(Alert {
                severity: Severity::Critical,
                message: format!("Actor {} stopped due to restart loop", child_id),
            }).await;
        }
    }
}
```

### Performance Bottlenecks

#### 1. High CPU Usage

**Symptoms:**
- CPU consistently above 80%
- Slow response times
- Thread pool exhaustion

**Performance Profiling:**

```bash
# CPU profiling with perf
sudo perf record -F 99 -p $(pgrep realtime_comm) -g -- sleep 30
sudo perf report

# Flame graph generation
git clone https://github.com/brendangregg/FlameGraph
sudo perf script | ./FlameGraph/stackcollapse-perf.pl | ./FlameGraph/flamegraph.pl > flame.svg
```

**Common Causes:**

1. **JSON Parsing Overhead**
   ```rust
   // Use faster JSON parser
   use simd_json;
   
   // Or implement binary protocol
   pub enum BinaryMessage {
       Ping { timestamp: u64 },
       Subscribe { topic_count: u16, topics: Vec<String> },
       // ...
   }
   ```

2. **Lock Contention**
   ```rust
   // Replace mutex with RwLock
   let data = Arc::new(RwLock::new(HashMap::new()));
   
   // Or use lock-free structures
   use dashmap::DashMap;
   let concurrent_map = Arc::new(DashMap::new());
   ```

#### 2. Memory Leaks

**Symptoms:**
- Steadily increasing memory usage
- OOM kills
- Performance degradation over time

**Memory Profiling:**

```bash
# Using Valgrind
valgrind --leak-check=full --show-leak-kinds=all ./target/release/realtime_communication

# Using heaptrack
heaptrack ./target/release/realtime_communication
heaptrack --analyze heaptrack.realtime_communication.12345.gz
```

**Common Memory Issues:**

1. **Connection Cleanup**
   ```rust
   // Ensure proper cleanup
   impl Drop for WebSocketActor {
       fn drop(&mut self) {
           // Clean up subscriptions
           self.subscriptions.clear();
           
           // Remove from connection manager
           if let Some(manager) = self.connection_manager.upgrade() {
               manager.remove_connection(&self.connection_id);
           }
           
           debug!("WebSocket actor {} cleaned up", self.connection_id);
       }
   }
   ```

2. **Message Buffer Leaks**
   ```rust
   // Implement message buffer limits
   pub struct MessageBuffer {
       messages: VecDeque<Message>,
       max_size: usize,
   }
   
   impl MessageBuffer {
       pub fn push(&mut self, msg: Message) {
           if self.messages.len() >= self.max_size {
               self.messages.pop_front(); // Remove oldest
           }
           self.messages.push_back(msg);
       }
   }
   ```

### Client Reconnection Strategies

#### 1. Exponential Backoff

```javascript
class ReconnectStrategy {
  constructor(options = {}) {
    this.baseDelay = options.baseDelay || 1000;
    this.maxDelay = options.maxDelay || 30000;
    this.maxAttempts = options.maxAttempts || 10;
    this.factor = options.factor || 2;
    this.jitter = options.jitter || true;
    this.attempt = 0;
  }
  
  nextDelay() {
    if (this.attempt >= this.maxAttempts) {
      return null; // Give up
    }
    
    let delay = Math.min(
      this.baseDelay * Math.pow(this.factor, this.attempt),
      this.maxDelay
    );
    
    if (this.jitter) {
      // Add random jitter to prevent thundering herd
      delay = delay * (0.5 + Math.random() * 0.5);
    }
    
    this.attempt++;
    return Math.floor(delay);
  }
  
  reset() {
    this.attempt = 0;
  }
  
  shouldRetry() {
    return this.attempt < this.maxAttempts;
  }
}

// Usage
const reconnect = new ReconnectStrategy({
  baseDelay: 1000,
  maxDelay: 60000,
  maxAttempts: 20,
  jitter: true
});

function handleDisconnect() {
  const delay = reconnect.nextDelay();
  
  if (delay !== null) {
    console.log(`Reconnecting in ${delay}ms (attempt ${reconnect.attempt})`);
    setTimeout(() => connect(), delay);
  } else {
    console.error('Max reconnection attempts reached');
    notifyUserOfPermanentDisconnect();
  }
}
```

#### 2. Connection State Recovery

```javascript
class StatefulReconnection {
  constructor(client) {
    this.client = client;
    this.state = {
      subscriptions: new Set(),
      pendingMessages: [],
      lastMessageId: null,
      sessionId: null
    };
  }
  
  saveState() {
    // Persist to localStorage for page reload recovery
    localStorage.setItem('rtc_state', JSON.stringify({
      subscriptions: Array.from(this.state.subscriptions),
      lastMessageId: this.state.lastMessageId,
      sessionId: this.state.sessionId,
      timestamp: Date.now()
    }));
  }
  
  async restoreState() {
    const saved = localStorage.getItem('rtc_state');
    if (!saved) return;
    
    const state = JSON.parse(saved);
    
    // Check if state is still valid (< 5 minutes old)
    if (Date.now() - state.timestamp > 300000) {
      localStorage.removeItem('rtc_state');
      return;
    }
    
    // Resubscribe to topics
    if (state.subscriptions.length > 0) {
      await this.client.subscribe(state.subscriptions);
    }
    
    // Request missed messages
    if (state.lastMessageId) {
      await this.client.send({
        type: 'GetMissedMessages',
        data: {
          since: state.lastMessageId,
          session_id: state.sessionId
        }
      });
    }
  }
  
  onConnect() {
    this.restoreState();
  }
  
  onMessage(message) {
    this.state.lastMessageId = message.id;
    this.saveState();
  }
  
  onSubscribe(topics) {
    topics.forEach(topic => this.state.subscriptions.add(topic));
    this.saveState();
  }
}
```

## Debug Logging

### Enable Detailed Logging

```bash
# Set log level
export RUST_LOG=realtime_communication=debug,actix_web=debug,actix_ws=debug

# Enable specific modules
export RUST_LOG=realtime_communication::actor=trace,realtime_communication::routing=debug

# Pretty print logs for development
export RUST_LOG=debug
export LOG_FORMAT=pretty
```

### Structured Logging Queries

```bash
# Find connection errors
jq 'select(.level == "ERROR" and .target == "realtime_communication::connection")' logs.json

# Track specific connection
jq 'select(.connection_id == "550e8400-e29b-41d4-a716-446655440000")' logs.json

# Message routing analysis
jq 'select(.message == "Message routed") | {
  message_id: .message_id,
  targets: .target_count,
  duration: .duration_ms
}' logs.json | jq -s 'group_by(.targets) | map({
  targets: .[0].targets,
  count: length,
  avg_duration: (map(.duration) | add / length)
})'
```

## Performance Analysis

### Load Testing Script

```javascript
// load-test.js
const WebSocket = require('ws');
const jwt = require('jsonwebtoken');

class LoadTester {
  constructor(config) {
    this.config = {
      url: 'ws://localhost:8081/ws',
      connections: 1000,
      messagesPerSecond: 10,
      duration: 60000, // 1 minute
      ...config
    };
    this.stats = {
      connected: 0,
      disconnected: 0,
      messagesSent: 0,
      messagesReceived: 0,
      errors: 0,
      latencies: []
    };
  }
  
  async run() {
    console.log(`Starting load test: ${this.config.connections} connections`);
    
    const clients = [];
    const startTime = Date.now();
    
    // Create connections
    for (let i = 0; i < this.config.connections; i++) {
      const client = await this.createClient(i);
      clients.push(client);
      
      // Stagger connections
      if (i % 10 === 0) {
        await new Promise(resolve => setTimeout(resolve, 10));
      }
    }
    
    // Send messages
    const messageInterval = setInterval(() => {
      clients.forEach(client => {
        if (client.readyState === WebSocket.OPEN) {
          const start = Date.now();
          client.send(JSON.stringify({
            type: 'Broadcast',
            data: {
              topic: 'load.test',
              payload: { timestamp: start }
            }
          }));
          this.stats.messagesSent++;
        }
      });
    }, 1000 / this.config.messagesPerSecond);
    
    // Run for duration
    await new Promise(resolve => setTimeout(resolve, this.config.duration));
    
    // Cleanup
    clearInterval(messageInterval);
    clients.forEach(client => client.close());
    
    // Report results
    this.report();
  }
  
  createClient(id) {
    return new Promise((resolve, reject) => {
      const token = this.generateToken(id);
      const ws = new WebSocket(`${this.config.url}?token=${token}`);
      
      ws.on('open', () => {
        this.stats.connected++;
        resolve(ws);
      });
      
      ws.on('message', (data) => {
        const message = JSON.parse(data);
        if (message.type === 'Broadcast') {
          const latency = Date.now() - message.data.payload.timestamp;
          this.stats.latencies.push(latency);
          this.stats.messagesReceived++;
        }
      });
      
      ws.on('error', (error) => {
        this.stats.errors++;
        reject(error);
      });
      
      ws.on('close', () => {
        this.stats.disconnected++;
      });
    });
  }
  
  generateToken(userId) {
    return jwt.sign(
      {
        sub: `load_test_${userId}`,
        user_id: `load_test_${userId}`,
        roles: ['user']
      },
      process.env.JWT_SECRET,
      { expiresIn: '1h' }
    );
  }
  
  report() {
    const avgLatency = this.stats.latencies.reduce((a, b) => a + b, 0) / this.stats.latencies.length;
    const p99Latency = this.stats.latencies.sort((a, b) => a - b)[Math.floor(this.stats.latencies.length * 0.99)];
    
    console.log('Load Test Results:');
    console.log(`- Connections: ${this.stats.connected}/${this.config.connections}`);
    console.log(`- Messages Sent: ${this.stats.messagesSent}`);
    console.log(`- Messages Received: ${this.stats.messagesReceived}`);
    console.log(`- Errors: ${this.stats.errors}`);
    console.log(`- Avg Latency: ${avgLatency.toFixed(2)}ms`);
    console.log(`- P99 Latency: ${p99Latency}ms`);
    console.log(`- Message Loss: ${((1 - this.stats.messagesReceived / this.stats.messagesSent) * 100).toFixed(2)}%`);
  }
}

// Run load test
new LoadTester({
  connections: 5000,
  messagesPerSecond: 100,
  duration: 300000 // 5 minutes
}).run();
```

## Emergency Procedures

### Service Degradation

```bash
#!/bin/bash
# emergency-mode.sh

# Enable read-only mode
curl -X POST http://localhost:8081/admin/emergency-mode \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -d '{"mode": "read_only"}'

# Disable new connections
curl -X POST http://localhost:8081/admin/connections/disable-new

# Increase rate limits temporarily
curl -X PUT http://localhost:8081/admin/config/rate-limits \
  -d '{"connection_rps": 10, "user_rps": 50}'

# Send system notification
curl -X POST http://localhost:8081/admin/broadcast \
  -d '{
    "type": "SystemNotification",
    "data": {
      "level": "warning",
      "message": "Service operating in degraded mode. Some features may be unavailable."
    }
  }'
```

### Data Recovery

```rust
// Recover from Redis failure
pub async fn recover_sessions_from_backup() -> Result<(), Error> {
    let backup_path = "/backup/sessions.json";
    let sessions: Vec<SessionBackup> = serde_json::from_reader(
        File::open(backup_path)?
    )?;
    
    for session in sessions {
        // Restore to Redis
        redis_client
            .set(
                format!("session:{}", session.id),
                serde_json::to_string(&session)?,
            )
            .expire(3600)
            .await?;
            
        info!("Restored session: {}", session.id);
    }
    
    Ok(())
}
```