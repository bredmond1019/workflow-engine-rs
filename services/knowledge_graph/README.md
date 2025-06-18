# Knowledge Graph Service

A high-performance graph database microservice for managing concept relationships, learning paths, and knowledge exploration using Dgraph.

## Overview

The Knowledge Graph Service provides a comprehensive solution for building and querying educational knowledge graphs. It enables intelligent learning path generation, prerequisite tracking, and concept relationship management through advanced graph algorithms.

### Key Features

- **Graph Database Backend**: Powered by Dgraph for scalable graph operations
- **Learning Path Generation**: Automatic generation of optimal learning sequences
- **Advanced Algorithms**: PageRank, shortest path, topological sorting, and graph traversal
- **Concept Relationships**: Prerequisites, related topics, and difficulty progression
- **Similarity Search**: Vector embedding support for finding similar concepts
- **Performance Optimized**: Connection pooling, caching, and query optimization
- **GraphQL API**: Full GraphQL support for flexible queries

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

### GraphQL Endpoint
- `POST /graphql` - Main GraphQL endpoint

### REST Endpoints
- `GET /health` - Health check
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
```

## Testing

Run the test suite:
```bash
# Unit tests
cargo test

# Integration tests (requires Dgraph)
cargo test -- --ignored

# Specific test categories
cargo test algorithms
cargo test graph_integration -- --ignored
```

## Monitoring

The service exposes Prometheus metrics at `/metrics`:

- Query performance metrics
- Algorithm execution times
- Connection pool statistics
- Cache hit/miss rates
- Error rates by operation

## Contributing

See the main project CONTRIBUTING.md for guidelines.

## License

Part of the AI System Rust project - see the main LICENSE file.