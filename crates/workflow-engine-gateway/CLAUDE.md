# CLAUDE.md - workflow-engine-gateway

This file provides guidance for Claude Code when working with the workflow-engine-gateway crate.

## Crate Overview

The workflow-engine-gateway crate implements a GraphQL Federation gateway that unifies multiple GraphQL services into a single API endpoint. It follows the Apollo Federation v2 specification and provides production-ready features for composing distributed GraphQL schemas.

**Key Features (v0.6.0)**:
- **Apollo Federation v2**: Full specification compliance with entity resolution
- **Schema Composition**: Automatic schema merging from multiple subgraphs
- **Query Planning**: Intelligent query distribution and optimization
- **Health Monitoring**: Subgraph health checks and circuit breakers
- **Security**: Authentication propagation and query complexity limits
- **Performance**: Query caching, batching, and parallel execution

### Purpose and Role

- **API Gateway**: Single entry point for all GraphQL queries
- **Schema Federation**: Composes schemas from multiple microservices
- **Query Router**: Distributes queries to appropriate subgraphs
- **Entity Resolution**: Resolves references across service boundaries
- **Performance Optimization**: Caches, batches, and optimizes queries

## Architecture

### Core Components

1. **Gateway Module** (`src/gateway.rs`)
   - Main gateway server implementation
   - Request handling and response aggregation
   - Middleware pipeline management
   - WebSocket subscription support

2. **Federation Module** (`src/federation/`)
   - **Schema Registry** (`schema_registry.rs`): Manages subgraph schemas
   - **Query Planner** (`query_planner.rs`): Plans optimal query execution
   - **Entity Resolution** (`entities.rs`): Resolves federated entities
   - **Directives** (`directives.rs`): Handles federation directives

3. **Subgraph Module** (`src/subgraph.rs`)
   - Subgraph client implementation
   - Connection pooling and health checks
   - Request/response transformation
   - Error handling and retries

4. **Configuration** (`src/config.rs`)
   - Gateway configuration management
   - Subgraph discovery and registration
   - Environment-based settings
   - Dynamic configuration updates

5. **Health Monitoring** (`src/health.rs`)
   - Subgraph health checks
   - Circuit breaker implementation
   - Metrics collection
   - Status aggregation

## Federation Implementation

### Schema Composition

```graphql
# Workflow subgraph schema
type Workflow @key(fields: "id") {
  id: ID!
  name: String!
  status: WorkflowStatus!
  createdBy: User! @external
}

# User subgraph schema  
type User @key(fields: "id") {
  id: ID!
  email: String!
  workflows: [Workflow!]!
}
```

### Entity Resolution

```rust
// Implementing _entities resolver
impl SubgraphSchema {
    async fn resolve_entities(
        &self,
        representations: Vec<Representation>,
    ) -> Result<Vec<Entity>, Error> {
        // Resolve entities based on __typename and key fields
        let entities = representations
            .into_iter()
            .map(|rep| self.resolve_entity(rep))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(entities)
    }
}
```

### Query Planning

```rust
// Query planning example
let plan = QueryPlanner::new(&schema_registry)
    .plan_query(&query_document)?;

// Execute plan with optimizations
let results = plan.execute_parallel(&subgraph_clients).await?;
```

## Configuration

### Gateway Configuration

```toml
[gateway]
port = 4000
host = "0.0.0.0"
playground = true
introspection = true

[performance]
query_cache_size = 1000
max_query_depth = 10
query_timeout_ms = 30000
enable_batching = true

[security]
enable_auth = true
auth_header = "Authorization"
max_query_complexity = 1000

[[subgraphs]]
name = "workflow"
url = "http://localhost:8080/graphql"
poll_interval_secs = 30
timeout_ms = 5000

[[subgraphs]]
name = "content"
url = "http://localhost:8082/graphql"
poll_interval_secs = 30
timeout_ms = 5000
```

### Environment Variables

```bash
# Gateway configuration
GATEWAY_PORT=4000
GATEWAY_HOST=0.0.0.0
ENABLE_PLAYGROUND=true

# Subgraph URLs
WORKFLOW_SUBGRAPH_URL=http://workflow-api:8080/graphql
CONTENT_SUBGRAPH_URL=http://content-api:8082/graphql
KNOWLEDGE_SUBGRAPH_URL=http://knowledge-api:3002/graphql

# Performance
QUERY_CACHE_SIZE=1000
MAX_QUERY_DEPTH=10
ENABLE_QUERY_BATCHING=true

# Security
JWT_PUBLIC_KEY=<public-key>
ENABLE_INTROSPECTION=false
```

## Common Development Tasks

### Adding a New Subgraph

1. Define the subgraph configuration:
```rust
let subgraph = SubgraphConfig {
    name: "new_service".to_string(),
    url: "http://localhost:8090/graphql".to_string(),
    schema_url: None,
    headers: HashMap::new(),
};
```

2. Ensure the subgraph implements federation:
```graphql
extend type Query {
  _service: _Service!
  _entities(representations: [_Any!]!): [_Entity]!
}
```

3. Register with the gateway:
```rust
gateway.add_subgraph(subgraph).await?;
```

### Implementing Custom Directives

```rust
use crate::federation::directives::DirectiveHandler;

struct CustomDirective;

impl DirectiveHandler for CustomDirective {
    fn name(&self) -> &str {
        "custom"
    }
    
    async fn process(
        &self,
        ctx: &DirectiveContext,
        next: Next,
    ) -> Result<Value, Error> {
        // Pre-processing
        let result = next.run(ctx).await?;
        // Post-processing
        Ok(result)
    }
}
```

### Query Optimization

```rust
// Enable query batching
let gateway = Gateway::builder()
    .enable_batching(true)
    .batch_size(20)
    .batch_timeout(Duration::from_millis(10))
    .build()?;

// Add caching layer
let cache = QueryCache::new(1000);
gateway.with_cache(cache);

// Enable parallel execution
gateway.with_parallel_execution(true);
```

## Testing

### Unit Tests
```bash
# Run all gateway tests
cargo test -p workflow-engine-gateway

# Run specific test modules
cargo test -p workflow-engine-gateway federation
cargo test -p workflow-engine-gateway query_planner
```

### Integration Tests
```bash
# Start required services
docker-compose up -d workflow-api content-api

# Run integration tests
cargo test -p workflow-engine-gateway -- --ignored

# Run federation validation
cargo run --example validate_federation
```

### Load Testing
```bash
# Run load test against gateway
cargo run --example load_test_gateway

# Monitor metrics
curl http://localhost:4000/metrics
```

## Performance Optimization

### Query Caching
- Implements intelligent query result caching
- Cache keys based on query + variables
- TTL-based expiration
- Cache invalidation on mutations

### Request Batching
- Batches multiple queries to same subgraph
- Reduces network overhead
- Configurable batch size and timeout

### Parallel Execution
- Executes independent subgraph queries in parallel
- Optimizes query plan for parallelism
- Respects data dependencies

### Connection Pooling
- Maintains persistent connections to subgraphs
- Configurable pool size per subgraph
- Health-based connection selection

## Security Considerations

### Authentication
- Propagates authentication headers to subgraphs
- Supports JWT validation at gateway level
- Per-subgraph authentication configuration

### Query Complexity
- Calculates and limits query complexity
- Prevents DoS through expensive queries
- Configurable complexity limits

### Rate Limiting
- Per-client rate limiting
- Query complexity-based throttling
- Subgraph-level rate limits

### Introspection Control
- Disable introspection in production
- Whitelist allowed operations
- Schema visibility controls

## Monitoring and Observability

### Health Endpoints
```
GET /health - Basic health check
GET /health/detailed - Detailed subgraph health
```

### Metrics
- Query execution time
- Subgraph response times
- Cache hit rates
- Error rates by subgraph
- Active connections

### Logging
- Structured JSON logging
- Query tracing with correlation IDs
- Error aggregation
- Performance logging

## Error Handling

### Partial Failures
- Returns partial results when possible
- Includes errors in response extensions
- Graceful degradation

### Circuit Breakers
- Per-subgraph circuit breakers
- Automatic recovery attempts
- Fallback responses

### Error Propagation
- Preserves error context from subgraphs
- Adds gateway-level error information
- Structured error responses

## Best Practices

1. **Schema Design**: Keep schemas focused and cohesive
2. **Entity Keys**: Use stable, unique identifiers
3. **Field Ownership**: Clear field ownership across services
4. **Performance**: Monitor and optimize slow queries
5. **Testing**: Test federation with all subgraphs
6. **Documentation**: Document entity relationships
7. **Versioning**: Plan for schema evolution
8. **Monitoring**: Track subgraph health and performance
9. **Security**: Validate at gateway and subgraph levels
10. **Caching**: Cache at appropriate levels

## Troubleshooting

### Common Issues

1. **Schema Composition Failures**
   - Check for conflicting type definitions
   - Verify federation directives
   - Ensure unique type names

2. **Entity Resolution Errors**
   - Verify @key fields are correct
   - Check _entities resolver implementation
   - Ensure consistent entity representations

3. **Performance Issues**
   - Enable query caching
   - Check for N+1 queries
   - Monitor subgraph response times
   - Optimize query planning

4. **Connection Problems**
   - Verify subgraph URLs
   - Check network connectivity
   - Review timeout settings
   - Monitor circuit breaker status

## Future Enhancements

- **Subscription Federation**: Full subscription support
- **Schema Versioning**: Multiple schema versions
- **Advanced Caching**: Entity-level caching
- **Query Cost Analysis**: Predictive query costing
- **Schema Registry**: Centralized schema storage
- **Distributed Tracing**: Full trace propagation
- **GraphQL Mesh**: Support for non-GraphQL services