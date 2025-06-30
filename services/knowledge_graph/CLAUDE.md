# CLAUDE.md - Knowledge Graph Service

This file provides guidance to Claude Code when working with the Knowledge Graph service, a high-performance graph database microservice for managing concept relationships and learning paths using Dgraph. The service operates as a GraphQL Federation subgraph, providing seamless integration with the federation gateway.

## Service Overview

The Knowledge Graph Service is a critical microservice in the AI workflow system that provides:
- Graph-based knowledge management for educational concepts
- Advanced graph algorithms for learning path generation
- Relationship tracking between concepts (prerequisites, related topics)
- Performance-optimized query execution with caching
- GraphQL Federation subgraph with entity resolution
- Enterprise-grade security with JWT authentication and rate limiting

## Purpose and Role

This service acts as the knowledge foundation for AI-powered educational applications by:
1. **Storing Knowledge Structures**: Managing concepts, their relationships, and metadata
2. **Generating Learning Paths**: Using graph algorithms to find optimal learning sequences
3. **Ranking and Discovery**: Identifying important concepts using PageRank and centrality measures
4. **Search and Similarity**: Finding related concepts through vector embeddings and graph traversal
5. **Progress Tracking**: Monitoring user advancement through learning materials

## Key Components

### 1. Dgraph Client (`src/client/`)
- **Connection Pool**: Manages persistent connections with health checking
- **Response Parser**: Handles GraphQL response parsing with alias/fragment support
- **Query Builder**: Type-safe query construction
- **Transaction Support**: ACID-compliant operations

### 2. Graph Algorithms (`src/algorithms/`)
- **Shortest Path**: Dijkstra and A* for optimal learning paths
- **PageRank**: Concept importance ranking
- **Topological Sort**: Prerequisite ordering with cycle detection
- **Graph Traversal**: BFS/DFS for exploration and analysis

### 3. API Layer (`src/api/`)
- **GraphQL Endpoint**: Primary interface at `/graphql` with federation support
- **REST Endpoints**: Simplified access for common operations
- **Authentication**: JWT-based with role permissions
- **Rate Limiting**: Tier-based request throttling
- **Federation**: Full Apollo Federation v2 implementation with `_service` and `_entities` resolvers

### 4. Service Core (`src/service.rs`)
- **Query Engine**: Optimized query execution with caching
- **Algorithm Engine**: Manages graph algorithm execution
- **Cache Manager**: Multi-level caching (in-memory + Redis)
- **Error Handling**: Comprehensive error types with context

## Dgraph Integration

### Schema Structure
The service uses a rich GraphQL schema (`dgraph/schema.graphql`) defining:
- **Concept**: Core knowledge units with difficulty, category, tags
- **LearningResource**: Educational content linked to concepts
- **LearningPath**: Curated sequences of concepts
- **UserProgress**: Learning advancement tracking

### Connection Details
```
Development:
- HTTP: localhost:8080
- gRPC: localhost:9080

Testing:
- HTTP: localhost:18080  
- gRPC: localhost:19080
```

### Key Operations
1. **Queries**: Complex graph traversals, search, aggregations
2. **Mutations**: Create/update/delete with relationship management
3. **Transactions**: Multi-operation consistency
4. **Subscriptions**: Real-time updates (planned)

## Available Graph Algorithms

### Path Finding
- **Dijkstra's Algorithm**: Optimal paths considering edge weights
- **A* Algorithm**: Heuristic-guided path finding
- **Alternative Paths**: K-shortest paths for variety

### Ranking & Analysis
- **PageRank**: Identifies most important concepts
- **Centrality Measures**: Degree, betweenness, closeness
- **Community Detection**: Concept clustering
- **Topological Sort**: Dependency ordering

### Traversal
- **BFS/DFS**: Standard graph exploration
- **Iterative Deepening**: Memory-efficient search
- **Connected Components**: Isolated cluster detection

## GraphQL Parsing

The service includes sophisticated GraphQL response parsing:
- **Type-safe parsing**: Converts JSON to domain objects
- **Alias resolution**: Handles GraphQL field aliases
- **Fragment expansion**: Supports GraphQL fragments
- **Nested relationships**: Recursive parsing of graph structures

## API Endpoints

### GraphQL Federation Endpoint
```
POST /graphql
```

The service implements a complete GraphQL Federation subgraph with:
- **Entities**: `Concept`, `LearningResource`, `UserProgress` with `@key` directives
- **Extended Types**: Extends `User` from the main API with learning-specific fields
- **Federation Queries**: `_service` for schema introspection, `_entities` for entity resolution

#### Example Federation Queries
```graphql
# Cross-service query through gateway
query UserLearningStatus($userId: ID!) {
  user(id: $userId) {
    id
    name  # From main API
    learningProgress {  # From knowledge graph
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
```
GET  /health                    # Health check
GET  /health/detailed           # Detailed component health
GET  /metrics                   # Prometheus metrics
POST /api/v1/search            # Concept search
GET  /api/v1/concept/:id       # Get concept details
POST /api/v1/learning-path     # Generate learning path
GET  /api/v1/related/:id       # Find related concepts
```

## Integration with Main Engine

The Knowledge Graph service integrates with the workflow engine and federation gateway through:
1. **GraphQL Federation**: Primary integration via Apollo Gateway (port 4000)
2. **Service Discovery**: Registered in the service registry
3. **MCP Protocol**: Exposed as an MCP server for workflow nodes
4. **Event Integration**: Publishes concept updates to event bus
5. **Shared Database**: Uses same PostgreSQL for vector embeddings
6. **Cross-Service Queries**: Supports federated queries across services

### Workflow Node Usage
```rust
// In workflow definitions
let kg_node = ExternalMcpNode::new(
    "knowledge_graph",
    "http://localhost:3002/mcp",
    "Find learning path"
);
```

## Testing Approach

### Unit Tests
```bash
cargo test
```

### Integration Tests
Require Dgraph running:
```bash
# Start test environment
./scripts/test-dgraph-setup.sh

# Run integration tests
cargo test -- --ignored

# Cleanup
./scripts/test-dgraph-teardown.sh
```

### Test Categories
- `knowledge_graph_integration`: Query and traversal tests
- `knowledge_graph_mutation`: Create/update/delete tests
- `knowledge_graph_transaction`: Transaction consistency tests
- `graph_algorithms`: Algorithm implementation tests

### Performance Tests
```bash
cargo test performance -- --ignored --nocapture
```

### Federation Tests
```bash
# Test federation integration
cargo test graphql_federation_test -- --ignored

# Verify subgraph schema
curl http://localhost:3002/graphql \
  -H "Content-Type: application/json" \
  -d '{"query": "{ _service { sdl } }"}'

# Test entity resolution
curl http://localhost:3002/graphql \
  -H "Content-Type: application/json" \
  -d '{
    "query": "query($_representations: [_Any!]!) { _entities(representations: $_representations) { ... on Concept { id name } } }",
    "variables": {
      "_representations": [
        { "__typename": "Concept", "id": "123" }
      ]
    }
  }'
```

## Common Development Tasks

### 1. Adding New Graph Algorithms
```rust
// 1. Create algorithm module in src/algorithms/
// 2. Implement algorithm trait
// 3. Add to algorithm engine
// 4. Update API endpoints
// 5. Add tests
```

### 2. Extending the Schema
```graphql
# 1. Update dgraph/schema.graphql
# 2. Run schema initialization
cd dgraph && ./init-schema.sh
# 3. Update domain models in src/graph.rs
# 4. Update parser in src/client/dgraph.rs
```

### 3. Adding API Endpoints
```rust
// 1. Define endpoint in src/api.rs
// 2. Implement handler logic
// 3. Add to router configuration
// 4. Update API documentation
// 5. Add integration tests
```

### 4. Optimizing Queries
```rust
// 1. Analyze slow queries with metrics
// 2. Add appropriate indexes in schema
// 3. Implement query caching
// 4. Consider algorithm alternatives
// 5. Profile with benchmarks
```

### 5. Debugging Dgraph Issues
```bash
# Check Dgraph health
curl http://localhost:8080/health

# View schema
curl http://localhost:8080/admin/schema

# Test queries directly
curl -X POST http://localhost:8080/query \
  -H "Content-Type: application/json" \
  -d '{"query": "{ concepts(func: type(Concept)) { count(uid) } }"}'

# Access Ratel UI
open http://localhost:8000
```

### 6. Performance Tuning
```bash
# Monitor metrics
curl http://localhost:3002/metrics | grep knowledge_graph

# Adjust connection pool
export DGRAPH_MAX_CONNECTIONS=50

# Tune cache sizes
export REDIS_POOL_SIZE=20
export CACHE_MAX_SIZE_MB=200
```

### 7. Working with Test Data
```bash
# Load sample data
cd test-data
dgraph live -f sample-data.json -a localhost:9080

# Query test concepts
cargo run --example parser_demo
```

## Environment Configuration

### Required Variables
```bash
DGRAPH_HOST=localhost
DGRAPH_GRPC_PORT=9080
DGRAPH_HTTP_PORT=8080
DATABASE_URL=postgresql://user:pass@localhost/knowledge_graph
REDIS_URL=redis://localhost:6379
```

### Optional Variables
```bash
DGRAPH_MAX_CONNECTIONS=20
REDIS_POOL_SIZE=10
SERVICE_PORT=3002
LOG_LEVEL=info
ENABLE_METRICS=true
CACHE_TTL_SECONDS=300
```

## Architecture Patterns

### Connection Pooling
- Maintains persistent gRPC connections
- Health checks every 30 seconds
- Automatic reconnection with backoff
- Statistics tracking for monitoring

### Caching Strategy
- L1: In-memory LRU cache (100MB)
- L2: Redis distributed cache (1GB)
- L3: Dgraph source of truth
- TTL and event-based invalidation

### Error Handling
- Custom error types with context
- Circuit breaker for external calls
- Retry policies with exponential backoff
- Graceful degradation strategies

### Algorithm Optimization
- Early termination when goal reached
- Constraint-based pruning
- Result caching for expensive operations
- Parallel execution where applicable

## Monitoring & Debugging

### Key Metrics
```
knowledge_graph_query_duration_seconds
knowledge_graph_algorithm_execution_time_seconds
knowledge_graph_cache_hit_ratio
knowledge_graph_connection_pool_active
knowledge_graph_errors_total
```

### Health Endpoints
- `/health` - Basic health check
- `/health/detailed` - Component status
- `/metrics` - Prometheus metrics

### Logging
```bash
# Enable debug logging
export RUST_LOG=knowledge_graph=debug

# Filter specific modules
export RUST_LOG=knowledge_graph::algorithms=trace
```

## Security Features

### Authentication & Authorization
1. **JWT Validation**: All GraphQL/REST requests require valid JWT tokens
2. **Role-Based Access**: Different permissions for read/write operations
3. **Multi-tenant Support**: Data isolation based on tenant ID from JWT
4. **Rate Limiting**: Per-tenant and global request limits

### Data Security
1. **Input Validation**: All mutations validated for malicious content
2. **Query Depth Limiting**: Prevents deeply nested queries
3. **Query Complexity Analysis**: Rejects overly complex queries
4. **Parameterized Queries**: Protection against injection attacks

### Network Security
1. **TLS Encryption**: All communications encrypted in transit
2. **CORS Configuration**: Restrictive cross-origin policies
3. **Security Headers**: Standard security headers on all responses

## Best Practices

1. **Query Optimization**: Use indexes and limit result sets
2. **Transaction Scope**: Keep transactions small and focused
3. **Error Recovery**: Implement retry logic for transient failures
4. **Cache Warming**: Pre-load frequently accessed data
5. **Monitoring**: Track algorithm performance and query patterns
6. **Testing**: Include both unit and integration tests
7. **Documentation**: Update API docs when adding endpoints
8. **Federation**: Test entity resolution when modifying types
9. **Security**: Always validate JWT tokens and enforce rate limits

## Troubleshooting

### Common Issues

1. **Connection Refused**
   - Check Dgraph is running: `docker ps`
   - Verify ports: `netstat -ln | grep 9080`
   - Check firewall rules

2. **Slow Queries**
   - Review query complexity
   - Check indexes with `/admin/schema`
   - Monitor cache hit rates

3. **Memory Issues**
   - Tune connection pool size
   - Adjust cache limits
   - Profile with `heaptrack`

4. **Test Failures**
   - Ensure test Dgraph is running
   - Check for port conflicts
   - Review test data initialization

### Debug Commands
```bash
# View service logs
docker logs knowledge-graph-service

# Check Dgraph logs
docker logs knowledge_graph_dgraph_alpha

# Monitor connections
lsof -i :9080

# Profile performance
cargo build --release --features profiling
perf record --call-graph=dwarf ./target/release/knowledge_graph
```

## Future Enhancements

- Graph Neural Networks for embeddings
- Distributed algorithm execution
- Real-time graph updates via subscriptions
- Advanced caching with predictive loading
- Multi-language concept support
- Collaborative filtering for recommendations

Remember: This service is critical for the AI system's knowledge management. Always test thoroughly and monitor performance when making changes.