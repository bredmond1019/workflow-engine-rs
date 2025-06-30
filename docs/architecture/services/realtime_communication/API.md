# API Reference

## WebSocket Connection Endpoint

### `GET /ws`

Establishes a WebSocket connection for real-time communication.

#### Request

**URL Parameters:** None

**Query Parameters:**
- `token` (optional): JWT authentication token

**Headers:**
- `Authorization` (optional): Bearer token authentication
- `Sec-WebSocket-Protocol` (optional): Subprotocol for token passing

**Example Request:**
```bash
# Using query parameter
wscat -c "ws://localhost:8081/ws?token=eyJhbGciOiJIUzI1NiIs..."

# Using Authorization header
wscat -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIs..." -c "ws://localhost:8081/ws"

# Using subprotocol
wscat -s "access_token.eyJhbGciOiJIUzI1NiIs..." -c "ws://localhost:8081/ws"
```

#### Response

**Success (101 Switching Protocols):**
- WebSocket connection established
- Welcome message sent upon connection

**Error Responses:**
- `401 Unauthorized`: Invalid or missing authentication token
- `403 Forbidden`: Token valid but insufficient permissions
- `503 Service Unavailable`: Server at capacity

## REST Management Endpoints

### `GET /health`

Health check endpoint for monitoring service status.

#### Request

**Example:**
```bash
curl http://localhost:8081/health
```

#### Response

**Success (200 OK):**
```json
{
  "status": "healthy",
  "active_connections": 1250,
  "max_connections": 10000,
  "uptime_seconds": 3600
}
```

**Service Degraded (503 Service Unavailable):**
```json
{
  "status": "unhealthy",
  "reason": "At capacity",
  "active_connections": 10000,
  "max_connections": 10000
}
```

### `GET /metrics`

Prometheus-compatible metrics endpoint.

#### Request

**Example:**
```bash
curl http://localhost:8081/metrics
```

#### Response

**Success (200 OK):**
```json
{
  "active_connections": 1250,
  "total_connections": 45678,
  "messages_sent": 1234567,
  "messages_received": 1234560,
  "errors": 23
}
```

### `GET /info`

Server information and configuration endpoint.

#### Request

**Example:**
```bash
curl http://localhost:8081/info
```

#### Response

**Success (200 OK):**
```json
{
  "server": {
    "name": "WebSocket Real-time Communication Server",
    "version": "1.0.0",
    "config": {
      "host": "0.0.0.0",
      "port": 8081,
      "max_connections": 10000,
      "heartbeat_interval_secs": 30,
      "client_timeout_secs": 60,
      "max_frame_size": 65536
    }
  },
  "stats": {
    "active_connections": 1250,
    "total_connections": 45678,
    "messages_sent": 1234567,
    "messages_received": 1234560,
    "errors": 23
  },
  "features": [
    "websocket",
    "heartbeat",
    "topic_subscription",
    "broadcast_messaging",
    "direct_messaging",
    "connection_management",
    "metrics"
  ]
}
```

## Session Management APIs

### `POST /sessions`

Create a new session (authentication endpoint).

#### Request

**Headers:**
- `Content-Type: application/json`

**Body:**
```json
{
  "username": "user@example.com",
  "password": "secure_password"
}
```

#### Response

**Success (200 OK):**
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
  "expires_in": 3600,
  "token_type": "Bearer",
  "user": {
    "id": "user_123",
    "username": "user@example.com",
    "roles": ["user", "subscriber"]
  }
}
```

### `POST /sessions/refresh`

Refresh an access token using a refresh token.

#### Request

**Headers:**
- `Content-Type: application/json`

**Body:**
```json
{
  "refresh_token": "eyJhbGciOiJIUzI1NiIs..."
}
```

#### Response

**Success (200 OK):**
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "expires_in": 3600,
  "token_type": "Bearer"
}
```

### `DELETE /sessions/{session_id}`

Terminate a specific session.

#### Request

**Headers:**
- `Authorization: Bearer <access_token>`

#### Response

**Success (204 No Content):** Session terminated successfully

## JavaScript/TypeScript Client Examples

### Basic WebSocket Client

```javascript
class RealtimeClient {
  constructor(url, token) {
    this.url = url;
    this.token = token;
    this.ws = null;
    this.handlers = new Map();
    this.reconnectAttempts = 0;
    this.maxReconnectAttempts = 5;
    this.reconnectDelay = 1000;
  }

  connect() {
    return new Promise((resolve, reject) => {
      const wsUrl = `${this.url}?token=${this.token}`;
      this.ws = new WebSocket(wsUrl);

      this.ws.onopen = () => {
        console.log('Connected to realtime service');
        this.reconnectAttempts = 0;
        resolve();
      };

      this.ws.onmessage = (event) => {
        this.handleMessage(JSON.parse(event.data));
      };

      this.ws.onerror = (error) => {
        console.error('WebSocket error:', error);
        reject(error);
      };

      this.ws.onclose = () => {
        console.log('Disconnected from realtime service');
        this.handleReconnection();
      };
    });
  }

  handleMessage(message) {
    const handler = this.handlers.get(message.type);
    if (handler) {
      handler(message.data);
    }
  }

  on(messageType, handler) {
    this.handlers.set(messageType, handler);
  }

  send(message) {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    }
  }

  handleReconnection() {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++;
      const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1);
      console.log(`Reconnecting in ${delay}ms... (attempt ${this.reconnectAttempts})`);
      setTimeout(() => this.connect(), delay);
    }
  }

  disconnect() {
    if (this.ws) {
      this.ws.close();
    }
  }
}
```

### React Hook Example

```typescript
import { useEffect, useState, useCallback, useRef } from 'react';

interface UseRealtimeOptions {
  url: string;
  token: string;
  autoConnect?: boolean;
}

interface RealtimeState {
  connected: boolean;
  connecting: boolean;
  error: Error | null;
}

export function useRealtime({ url, token, autoConnect = true }: UseRealtimeOptions) {
  const [state, setState] = useState<RealtimeState>({
    connected: false,
    connecting: false,
    error: null
  });
  
  const clientRef = useRef<RealtimeClient | null>(null);
  const handlersRef = useRef<Map<string, (data: any) => void>>(new Map());

  const connect = useCallback(async () => {
    if (clientRef.current?.connected) return;

    setState(prev => ({ ...prev, connecting: true, error: null }));

    try {
      const client = new RealtimeClient(url, token);
      
      // Register all existing handlers
      handlersRef.current.forEach((handler, type) => {
        client.on(type, handler);
      });

      await client.connect();
      clientRef.current = client;
      setState({ connected: true, connecting: false, error: null });
    } catch (error) {
      setState({ connected: false, connecting: false, error: error as Error });
    }
  }, [url, token]);

  const disconnect = useCallback(() => {
    clientRef.current?.disconnect();
    clientRef.current = null;
    setState({ connected: false, connecting: false, error: null });
  }, []);

  const subscribe = useCallback((topics: string[]) => {
    clientRef.current?.send({
      type: 'Subscribe',
      data: { topics }
    });
  }, []);

  const broadcast = useCallback((topic: string, payload: any) => {
    clientRef.current?.send({
      type: 'Broadcast',
      data: { topic, payload }
    });
  }, []);

  const sendDirectMessage = useCallback((targetUser: string, payload: any) => {
    clientRef.current?.send({
      type: 'DirectMessage',
      data: { target_user: targetUser, payload }
    });
  }, []);

  const on = useCallback((messageType: string, handler: (data: any) => void) => {
    handlersRef.current.set(messageType, handler);
    clientRef.current?.on(messageType, handler);
  }, []);

  const off = useCallback((messageType: string) => {
    handlersRef.current.delete(messageType);
  }, []);

  useEffect(() => {
    if (autoConnect) {
      connect();
    }

    return () => {
      disconnect();
    };
  }, [autoConnect, connect, disconnect]);

  return {
    ...state,
    connect,
    disconnect,
    subscribe,
    broadcast,
    sendDirectMessage,
    on,
    off
  };
}

// Usage example
function ChatComponent() {
  const { connected, subscribe, broadcast, on } = useRealtime({
    url: 'ws://localhost:8081/ws',
    token: 'your-jwt-token'
  });

  useEffect(() => {
    if (connected) {
      subscribe(['chat.general']);

      on('Broadcast', (data) => {
        console.log('New message:', data);
      });
    }
  }, [connected, subscribe, on]);

  const sendMessage = (message: string) => {
    broadcast('chat.general', { message });
  };

  return (
    <div>
      <p>Status: {connected ? 'Connected' : 'Disconnected'}</p>
      {/* Chat UI */}
    </div>
  );
}
```

### Node.js Client Example

```javascript
const WebSocket = require('ws');
const EventEmitter = require('events');

class NodeRealtimeClient extends EventEmitter {
  constructor(url, token, options = {}) {
    super();
    this.url = url;
    this.token = token;
    this.options = {
      heartbeatInterval: 30000,
      reconnectDelay: 1000,
      maxReconnectAttempts: 5,
      ...options
    };
    this.ws = null;
    this.reconnectAttempts = 0;
    this.heartbeatTimer = null;
  }

  connect() {
    return new Promise((resolve, reject) => {
      const wsUrl = `${this.url}?token=${this.token}`;
      
      this.ws = new WebSocket(wsUrl, {
        perMessageDeflate: false,
        handshakeTimeout: 10000
      });

      this.ws.on('open', () => {
        console.log('Connected to realtime service');
        this.reconnectAttempts = 0;
        this.startHeartbeat();
        this.emit('connected');
        resolve();
      });

      this.ws.on('message', (data) => {
        try {
          const message = JSON.parse(data);
          this.emit(message.type, message.data);
          this.emit('message', message);
        } catch (error) {
          console.error('Failed to parse message:', error);
        }
      });

      this.ws.on('error', (error) => {
        console.error('WebSocket error:', error);
        this.emit('error', error);
        reject(error);
      });

      this.ws.on('close', (code, reason) => {
        console.log(`Disconnected: ${code} - ${reason}`);
        this.stopHeartbeat();
        this.emit('disconnected', { code, reason });
        this.handleReconnection();
      });

      this.ws.on('pong', () => {
        this.emit('pong');
      });
    });
  }

  startHeartbeat() {
    this.heartbeatTimer = setInterval(() => {
      if (this.ws.readyState === WebSocket.OPEN) {
        this.ws.ping();
        this.send({
          type: 'Ping',
          data: { timestamp: Date.now() }
        });
      }
    }, this.options.heartbeatInterval);
  }

  stopHeartbeat() {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = null;
    }
  }

  send(message) {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
      return true;
    }
    return false;
  }

  subscribe(topics) {
    return this.send({
      type: 'Subscribe',
      data: { topics }
    });
  }

  broadcast(topic, payload) {
    return this.send({
      type: 'Broadcast',
      data: { topic, payload }
    });
  }

  sendDirectMessage(targetUser, payload) {
    return this.send({
      type: 'DirectMessage',
      data: { target_user: targetUser, payload }
    });
  }

  handleReconnection() {
    if (this.reconnectAttempts < this.options.maxReconnectAttempts) {
      this.reconnectAttempts++;
      const delay = this.options.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1);
      console.log(`Reconnecting in ${delay}ms... (attempt ${this.reconnectAttempts})`);
      setTimeout(() => {
        this.connect().catch(console.error);
      }, delay);
    } else {
      this.emit('max_reconnect_attempts_reached');
    }
  }

  disconnect() {
    this.stopHeartbeat();
    if (this.ws) {
      this.ws.close(1000, 'Client disconnect');
      this.ws = null;
    }
  }
}

// Usage example
async function main() {
  const client = new NodeRealtimeClient('ws://localhost:8081/ws', 'your-jwt-token');

  // Set up event handlers
  client.on('connected', () => {
    console.log('Connected!');
    client.subscribe(['notifications', 'updates']);
  });

  client.on('Broadcast', (data) => {
    console.log('Broadcast received:', data);
  });

  client.on('DirectMessage', (data) => {
    console.log('Direct message from', data.from_user, ':', data.payload);
  });

  client.on('error', (error) => {
    console.error('Client error:', error);
  });

  // Connect to service
  try {
    await client.connect();
    
    // Send a broadcast message
    client.broadcast('notifications', {
      title: 'Hello from Node.js',
      message: 'This is a test notification'
    });
  } catch (error) {
    console.error('Failed to connect:', error);
  }
}

module.exports = NodeRealtimeClient;
```

## Python Client Example

```python
import asyncio
import json
import websockets
from typing import Dict, Any, Callable, Optional
from datetime import datetime

class RealtimeClient:
    def __init__(self, url: str, token: str):
        self.url = url
        self.token = token
        self.websocket: Optional[websockets.WebSocketClientProtocol] = None
        self.handlers: Dict[str, Callable] = {}
        self.running = False
        
    async def connect(self):
        """Connect to the WebSocket server"""
        uri = f"{self.url}?token={self.token}"
        self.websocket = await websockets.connect(uri)
        self.running = True
        
        # Start message handler
        asyncio.create_task(self._handle_messages())
        
    async def _handle_messages(self):
        """Handle incoming messages"""
        async for message in self.websocket:
            try:
                data = json.loads(message)
                message_type = data.get('type')
                
                if message_type in self.handlers:
                    await self.handlers[message_type](data.get('data'))
                    
            except json.JSONDecodeError:
                print(f"Failed to parse message: {message}")
            except Exception as e:
                print(f"Error handling message: {e}")
                
    def on(self, message_type: str, handler: Callable):
        """Register a message handler"""
        self.handlers[message_type] = handler
        
    async def send(self, message_type: str, data: Any):
        """Send a message to the server"""
        if self.websocket:
            message = {
                'type': message_type,
                'data': data,
                'timestamp': int(datetime.now().timestamp() * 1000)
            }
            await self.websocket.send(json.dumps(message))
            
    async def subscribe(self, topics: list):
        """Subscribe to topics"""
        await self.send('Subscribe', {'topics': topics})
        
    async def broadcast(self, topic: str, payload: Any):
        """Send a broadcast message"""
        await self.send('Broadcast', {'topic': topic, 'payload': payload})
        
    async def send_direct_message(self, target_user: str, payload: Any):
        """Send a direct message"""
        await self.send('DirectMessage', {'target_user': target_user, 'payload': payload})
        
    async def disconnect(self):
        """Disconnect from the server"""
        self.running = False
        if self.websocket:
            await self.websocket.close()

# Usage example
async def main():
    client = RealtimeClient('ws://localhost:8081/ws', 'your-jwt-token')
    
    # Register handlers
    client.on('Status', lambda data: print(f"Status: {data}"))
    client.on('Broadcast', lambda data: print(f"Broadcast: {data}"))
    client.on('DirectMessage', lambda data: print(f"DM: {data}"))
    
    # Connect and interact
    await client.connect()
    await client.subscribe(['chat.general', 'notifications'])
    
    # Send a broadcast
    await client.broadcast('chat.general', {
        'message': 'Hello from Python!',
        'sender': 'python_client'
    })
    
    # Keep running
    await asyncio.sleep(60)
    await client.disconnect()

if __name__ == "__main__":
    asyncio.run(main())
```

## Error Handling

All API endpoints return consistent error responses:

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "details": {
      "field": "Additional context"
    }
  }
}
```

Common error codes:
- `INVALID_TOKEN`: Authentication token is invalid
- `TOKEN_EXPIRED`: Authentication token has expired
- `RATE_LIMITED`: Too many requests
- `SERVER_FULL`: Server at capacity
- `INVALID_REQUEST`: Malformed request
- `INTERNAL_ERROR`: Server error