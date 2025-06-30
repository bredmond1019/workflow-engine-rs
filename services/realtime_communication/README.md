# Real-time Communication Service

A high-performance WebSocket-based messaging microservice built with Rust and Actix-Web, designed to handle 10,000+ concurrent connections with an actor model architecture. The service operates as a GraphQL Federation subgraph, providing both WebSocket and GraphQL APIs with comprehensive subscription support.

## Overview

The Real-time Communication Service provides WebSocket-based real-time messaging capabilities for the AI Workflow Orchestration platform. It implements a scalable actor-based architecture with support for topic subscriptions, direct messaging, presence tracking, and room-based communication. As part of the federation architecture, it extends user entities and provides real-time subscriptions through the GraphQL gateway.

### Key Features

- **WebSocket Server**: High-performance WebSocket server supporting 10,000+ concurrent connections
- **Actor Model**: Isolated actors for each WebSocket connection with supervision
- **Topic-based Messaging**: Publish/subscribe pattern with flexible topic routing
- **Direct Messaging**: Point-to-point communication between users
- **Presence System**: Real-time user presence and connection tracking
- **Room Support**: Create and join communication rooms/channels
- **Rate Limiting**: Multi-level rate limiting (connection, user, global)
- **JWT Authentication**: Secure token-based authentication with refresh support
- **Circuit Breaker**: Protection against cascading failures
- **Message Routing**: Rule-based message routing with filtering and transformation
- **GraphQL Federation**: Apollo Federation v2 subgraph with subscriptions
- **Real-time Subscriptions**: Live updates for messages, presence, and typing indicators
- **Metrics & Monitoring**: Prometheus metrics and health endpoints

## Quick Start

### Prerequisites

- Rust 1.75 or higher
- Redis 7.0+ (for session persistence and message queuing)
- Docker and Docker Compose (optional)

### Running with Docker

```bash
# Build and run with Docker Compose
docker-compose up -d realtime-communication

# View logs
docker-compose logs -f realtime-communication
```

### Running Locally

```bash
# Install dependencies
cargo build --release

# Set environment variables
export JWT_SECRET="your-secure-jwt-secret"
export REDIS_URL="redis://localhost:6379"

# Run the service
cargo run --release

# The service will start on http://localhost:8081
```

### WebSocket Client Example

```javascript
// JavaScript/TypeScript WebSocket client example
class RealtimeClient {
  constructor(token) {
    this.token = token;
    this.ws = null;
    this.reconnectAttempts = 0;
  }

  connect() {
    // Connect with JWT token in query parameter
    const wsUrl = `ws://localhost:8081/ws?token=${this.token}`;
    this.ws = new WebSocket(wsUrl);

    this.ws.onopen = () => {
      console.log('Connected to real-time service');
      this.reconnectAttempts = 0;
      
      // Subscribe to topics
      this.subscribe(['notifications', 'updates']);
    };

    this.ws.onmessage = (event) => {
      const message = JSON.parse(event.data);
      this.handleMessage(message);
    };

    this.ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    this.ws.onclose = () => {
      console.log('Disconnected from real-time service');
      this.handleReconnect();
    };
  }

  subscribe(topics) {
    this.send({
      type: 'Subscribe',
      data: { topics }
    });
  }

  broadcast(topic, payload) {
    this.send({
      type: 'Broadcast',
      data: { topic, payload }
    });
  }

  sendDirectMessage(targetUser, payload) {
    this.send({
      type: 'DirectMessage',
      data: { target_user: targetUser, payload }
    });
  }

  send(message) {
    if (this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    }
  }

  handleMessage(message) {
    switch (message.type) {
      case 'Status':
        console.log('Status:', message.data);
        break;
      case 'Broadcast':
        console.log('Broadcast message:', message.data);
        break;
      case 'DirectMessage':
        console.log('Direct message:', message.data);
        break;
      case 'Error':
        console.error('Error:', message.data);
        break;
      default:
        console.log('Unknown message type:', message);
    }
  }

  handleReconnect() {
    if (this.reconnectAttempts < 5) {
      this.reconnectAttempts++;
      const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 30000);
      console.log(`Reconnecting in ${delay}ms...`);
      setTimeout(() => this.connect(), delay);
    }
  }

  disconnect() {
    if (this.ws) {
      this.ws.close();
    }
  }
}

// Usage
const client = new RealtimeClient('your-jwt-token');
client.connect();
```

## Technology Stack

- **Rust**: System programming language for performance and safety
- **Actix-Web**: High-performance async web framework
- **actix-ws**: WebSocket support for Actix-Web
- **Tokio**: Async runtime for Rust
- **Redis**: In-memory data store for session persistence and pub/sub
- **JWT**: JSON Web Tokens for authentication
- **DashMap**: Concurrent HashMap for efficient state management
- **Prometheus**: Metrics collection and monitoring

## Development Setup

### Local Development

1. Clone the repository:
```bash
git clone <repository-url>
cd services/realtime_communication
```

2. Install dependencies:
```bash
cargo fetch
```

3. Set up Redis:
```bash
# Using Docker
docker run -d -p 6379:6379 redis:7-alpine

# Or install locally
brew install redis  # macOS
sudo apt-get install redis-server  # Ubuntu
```

4. Configure environment:
```bash
cp .env.example .env
# Edit .env with your configuration
```

5. Run tests:
```bash
cargo test
```

6. Run with hot reload:
```bash
cargo watch -x run
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `HOST` | Server bind address | `0.0.0.0` |
| `PORT` | Server port | `8081` |
| `JWT_SECRET` | Secret key for JWT signing | Required |
| `REDIS_URL` | Redis connection URL | `redis://localhost:6379` |
| `MAX_CONNECTIONS` | Maximum concurrent connections | `10000` |
| `HEARTBEAT_INTERVAL` | WebSocket heartbeat interval | `30s` |
| `CLIENT_TIMEOUT` | Client inactivity timeout | `60s` |
| `RATE_LIMIT_ENABLED` | Enable rate limiting | `true` |
| `LOG_LEVEL` | Logging level | `info` |

### Building for Production

```bash
# Build optimized binary
cargo build --release

# Run production build
./target/release/realtime_communication

# Or use Docker
docker build -t realtime-communication:latest .
docker run -p 8081:8081 realtime-communication:latest
```

## API Endpoints

### GraphQL Federation Endpoint
- `POST /graphql` - GraphQL endpoint with federation support and subscriptions

The service implements a complete Apollo Federation v2 subgraph with:
- **Entities**: `Message`, `Conversation`, `Session` with `@key` directives
- **Extended Types**: Extends `User` from the main API with messaging and presence fields
- **Subscriptions**: Real-time subscriptions for messages, presence, and typing indicators

#### Example GraphQL Subscriptions

```graphql
# Subscribe to new messages in specific conversations
subscription MessageReceived {
  messageReceived(conversationIds: ["conv-123", "conv-456"]) {
    id
    content
    senderId
    timestamp
    user {
      id
      name  # From main API through federation
    }
  }
}

# Subscribe to user presence updates
subscription PresenceUpdated {
  presenceUpdated(userIds: ["user-123", "user-456"]) {
    userId
    status
    lastSeenAt
    devices {
      deviceId
      connectionType
    }
  }
}

# Subscribe to typing indicators
subscription TypingIndicator {
  typingIndicator(conversationIds: ["conv-123"]) {
    conversationId
    userId
    isTyping
  }
}
```

#### Cross-Service User Extension

```graphql
# Query user with messaging data through federation
query UserWithMessaging($userId: ID!) {
  user(id: $userId) {
    id
    name                    # From main API
    email                   # From main API
    status                  # From realtime communication
    conversations {         # From realtime communication
      id
      name
      lastActivityAt
      unreadMessageCount
    }
  }
}
```

### WebSocket Endpoint
- `GET /ws` - WebSocket connection endpoint
  - Query params: `token` (JWT access token)
  - Subprotocol: `access_token.<token>` (alternative auth method)

### REST Endpoints
- `GET /health` - Health check endpoint
- `GET /health/detailed` - Detailed component health
- `GET /metrics` - Prometheus metrics endpoint
- `GET /info` - Server information and configuration

## Security Features

- **JWT Authentication**: All WebSocket and GraphQL connections require valid JWT tokens
- **Rate Limiting**: Multi-level rate limiting for connections, messages, and subscriptions
- **Origin Validation**: Strict CORS policies for WebSocket connections
- **TLS Encryption**: WSS protocol enforced in production environments
- **Input Sanitization**: All user inputs validated and sanitized
- **Query Depth Limiting**: Protection against deeply nested GraphQL queries
- **Multi-tenant Isolation**: Message and conversation scoping by tenant ID

## Deployment

The service supports multiple deployment strategies:

- **Docker**: Multi-stage builds with optimized runtime images
- **Kubernetes**: Includes HPA, PDB, and network policies
- **WebSocket Load Balancing**: Sticky sessions for WebSocket connections

### Federation Integration

The service automatically registers with the Apollo Federation gateway:

```bash
# Verify federation health
curl http://localhost:4000/health/detailed

# Test subscription through gateway
curl -X POST http://localhost:4000/graphql \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "query": "subscription { messageReceived(conversationIds: [\"123\"]) { id content } }"
  }'

# Check subgraph schema
curl http://localhost:8081/graphql \
  -d '{"query": "{ _service { sdl } }"}'
```

### WebSocket Protocol Documentation

The WebSocket protocol supports both direct connections and federation integration:

- **Direct WebSocket**: `ws://localhost:8081/ws?token=JWT_TOKEN`
- **Federation GraphQL**: Subscriptions through `http://localhost:4000/graphql`
- **Actor Model**: Each connection runs in an isolated actor for fault tolerance
- **Message Routing**: Topic-based pub/sub with filtering and transformation

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is part of the AI System Rust platform and follows the same license terms.