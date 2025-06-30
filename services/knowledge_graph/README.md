# Knowledge Graph Service

A high-performance graph database microservice for managing concept relationships, learning paths, and knowledge exploration using Dgraph. The service operates as a GraphQL Federation subgraph, enabling seamless integration with the AI Workflow Orchestration platform.

## Overview

The Knowledge Graph Service provides a comprehensive solution for building and querying educational knowledge graphs. It enables intelligent learning path generation, prerequisite tracking, and concept relationship management through advanced graph algorithms. As part of the federation architecture, it extends entities from other services and provides cross-service query capabilities.

### Key Features

- **Graph Database Backend**: Powered by Dgraph for scalable graph operations
- **Learning Path Generation**: Automatic generation of optimal learning sequences
- **Advanced Algorithms**: PageRank, shortest path, topological sorting, and graph traversal
- **Concept Relationships**: Prerequisites, related topics, and difficulty progression
- **Similarity Search**: Vector embedding support for finding similar concepts
- **Performance Optimized**: Connection pooling, caching, and query optimization
- **GraphQL Federation**: Apollo Federation v2 subgraph with entity resolution
- **Enterprise Security**: JWT authentication, rate limiting, and query complexity analysis

### Technology Stack

- **Language**: Rust
- **Graph Database**: Dgraph v23.1.0
- **Cache**: Redis
- **API**: GraphQL + REST
- **Database**: PostgreSQL (for vector embeddings)
- **Container**: Docker & Kubernetes ready

## Quick Start

### Prerequisites

- Docker and Docker Compose
- Rust 1.70+ (for development)
- PostgreSQL 15+ (for vector embeddings)
- Redis 7+ (for caching)

### Running with Docker

1. Clone the repository and navigate to the service directory:
```bash
cd services/knowledge_graph
```

2. Start Dgraph and dependencies:
```bash
cd dgraph
docker-compose up -d
```

3. Build and run the service:
```bash
docker build -t knowledge-graph-service .
docker run -p 3002:3002 \
  -e DATABASE_URL=postgresql://user:pass@localhost/knowledge_graph \
  -e REDIS_URL=redis://localhost:6379 \
  -e DGRAPH_HOST=localhost \
  knowledge-graph-service
```

### Development Setup

1. Install dependencies:
```bash
cargo build
```

2. Set up environment variables:
```bash
cp .env.example .env
# Edit .env with your configuration
```

3. Start Dgraph:
```bash
cd dgraph
docker-compose up -d
./init-schema.sh  # Initialize GraphQL schema
```

4. Run the service:
```bash
cargo run
```

## API Endpoints

### GraphQL Federation Endpoint
- `POST /graphql` - Main GraphQL endpoint with federation support

The service implements a complete Apollo Federation v2 subgraph:
- **Entities**: `Concept`, `LearningResource`, `UserProgress` with `@key` directives
- **Extended Types**: Extends `User` from the main API with learning progress
- **Cross-Service Queries**: Query through the federation gateway (port 4000)

#### Example Federation Query
```graphql
# Query through gateway - combines data from multiple services
query UserLearningDashboard($userId: ID!) {
  user(id: $userId) {
    id
    name                    # From main API
    email                   # From main API
    learningProgress {      # From knowledge graph
      totalConceptsCompleted
      currentLearningPaths {
        toConcept {
          name
          difficulty
        }
      }
    }
  }
}
```

### REST Endpoints
- `GET /health` - Health check
- `GET /health/detailed` - Detailed component health
- `GET /metrics` - Prometheus metrics
- `POST /api/v1/search` - Concept search
- `GET /api/v1/concept/:id` - Get concept details
- `POST /api/v1/learning-path` - Generate learning path
- `GET /api/v1/related/:id` - Find related concepts

## Features in Detail

### Graph Algorithms

1. **Shortest Path Algorithms**
   - Dijkstra's algorithm for optimal paths
   - A* algorithm with heuristics
   - Alternative path finding
   - Configurable cost constraints

2. **Ranking Algorithms**
   - PageRank for concept importance
   - Centrality measures (degree, betweenness, closeness)
   - Community detection
   - Difficulty-based ranking

3. **Topological Sorting**
   - Kahn's algorithm for prerequisite ordering
   - DFS-based sorting
   - Cycle detection
   - Parallel learning level identification

4. **Graph Traversal**
   - BFS and DFS implementations
   - Depth-limited search
   - Iterative deepening
   - Connected components detection

### Data Model

The service uses a rich graph schema including:

- **Concepts**: Core knowledge units with difficulty, category, and metadata
- **Prerequisites**: Directed edges representing learning dependencies
- **Learning Resources**: Associated content for each concept
- **Learning Paths**: Curated sequences of concepts
- **User Progress**: Tracking of learning advancement

### Performance Features

- Connection pooling with health checks
- Redis caching for frequent queries
- Query optimization and indexing
- Batch operations support
- Async processing with Tokio

## Configuration

Key configuration options (via environment variables):

```bash
# Dgraph Configuration
DGRAPH_HOST=localhost
DGRAPH_GRPC_PORT=9080
DGRAPH_HTTP_PORT=8080
DGRAPH_MAX_CONNECTIONS=20

# Redis Configuration
REDIS_URL=redis://localhost:6379
REDIS_POOL_SIZE=10

# Database Configuration (for embeddings)
DATABASE_URL=postgresql://user:pass@localhost/knowledge_graph

# Service Configuration
SERVICE_PORT=3002
LOG_LEVEL=info
ENABLE_METRICS=true

# Security Configuration
JWT_SECRET=your-secret-key
RATE_LIMIT_RPM=100
MAX_QUERY_DEPTH=15
MAX_QUERY_COMPLEXITY=1000

# Federation Configuration
FEDERATION_ENABLED=true
GATEWAY_URL=http://localhost:4000
```

## Testing

Run the test suite:
```bash
# Unit tests
cargo test

# Integration tests (requires Dgraph)
cargo test -- --ignored

# Federation tests
cargo test graphql_federation_test -- --ignored

# Specific test categories
cargo test algorithms
cargo test graph_integration -- --ignored

# Test federation through gateway
curl -X POST http://localhost:4000/graphql \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "query": "{ concept(id: \"123\") { name difficulty } }"
  }'
```

## Monitoring

The service exposes Prometheus metrics at `/metrics`:

- Query performance metrics
- Algorithm execution times
- Connection pool statistics
- Cache hit/miss rates
- Error rates by operation

## Security Features

- **JWT Authentication**: All API endpoints require valid JWT tokens
- **Rate Limiting**: Configurable request limits per tenant and globally
- **Query Complexity Analysis**: Protection against expensive queries
- **Input Validation**: Comprehensive validation for all graph mutations
- **TLS Encryption**: All communications encrypted in transit
- **Multi-tenant Isolation**: Data scoped by tenant ID from JWT claims

## Deployment

The service supports multiple deployment strategies:

- **Docker**: Multi-stage builds with optimized runtime images
- **Kubernetes**: Includes HPA, PDB, and network policies
- **Docker Compose**: Complete stack with Dgraph and dependencies

### Federation Integration

The service automatically registers with the Apollo Federation gateway:

```bash
# Verify federation health
curl http://localhost:4000/health/detailed

# Check subgraph registration
curl http://localhost:4000/graphql \
  -d '{"query": "{ _service { sdl } }"}'
```

## Contributing

See the main project CONTRIBUTING.md for guidelines.

## License

Part of the AI System Rust project - see the main LICENSE file.