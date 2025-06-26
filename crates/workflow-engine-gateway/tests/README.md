# GraphQL Federation Integration Tests

This directory contains comprehensive integration tests for the GraphQL Federation Gateway implementation.

## Test Overview

### Tests 16-18: Gateway Integration Tests

- **Test 16: Multi-Subgraph Query Test** - Verifies the gateway can execute complex queries across multiple subgraphs
- **Test 17: Entity Reference Resolution Test** - Tests cross-service entity resolution using `_entities` queries and federation directives  
- **Test 18: Schema Composition Test** - Verifies the gateway properly composes schemas from all subgraphs without conflicts

## Test Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Test Client   │───▶│ GraphQL Gateway  │───▶│   Subgraphs     │
│                 │    │    (Port 4000)   │    │                 │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                              │                          │
                              │                          ├─ Workflow API (8080)
                              │                          ├─ Content Processing (3001)
                              │                          ├─ Knowledge Graph (3002)
                              └──────────────────────────├─ Realtime Communication (3003)
```

## Federation Services

### 1. Workflow API (Main API)
- **Port:** 8080
- **Endpoint:** `/api/v1/graphql`
- **Entities:** User, Workflow (owned)
- **Federation:** Entity owner and extender

### 2. Content Processing Service  
- **Port:** 3001
- **Endpoint:** `/graphql`
- **Entities:** ContentMetadata, ProcessingJob (owned), User (extended)
- **Federation:** Entity owner and extender

### 3. Knowledge Graph Service
- **Port:** 3002  
- **Endpoint:** `/graphql`
- **Entities:** Concept, LearningResource (owned), User (extended)
- **Federation:** Entity owner and extender

### 4. Realtime Communication Service
- **Port:** 3003
- **Endpoint:** `/graphql`
- **Entities:** Message, Conversation, Session (owned), User (extended)
- **Federation:** Entity owner and extender

### 5. GraphQL Gateway
- **Port:** 4000
- **Endpoint:** `/graphql`
- **Role:** Schema composition, query planning, entity resolution

## Running Tests

### Prerequisites

1. **Build the project:**
   ```bash
   cargo build
   ```

2. **Start required services:**
   ```bash
   # Option 1: Use the test script (recommended)
   ./scripts/test_federation.sh start
   
   # Option 2: Start services manually
   # Terminal 1: Workflow API
   cd crates/workflow-engine-api && cargo run --bin workflow-engine
   
   # Terminal 2: Content Processing
   cd services/content_processing && cargo run
   
   # Terminal 3: Knowledge Graph  
   cd services/knowledge_graph && cargo run
   
   # Terminal 4: Realtime Communication
   cd services/realtime_communication && cargo run
   
   # Terminal 5: GraphQL Gateway
   cd crates/workflow-engine-gateway && cargo run --bin graphql-gateway
   ```

### Test Execution

#### Full Test Suite
```bash
# Run all federation integration tests
./scripts/test_federation.sh

# Or run tests manually after starting services
cd crates/workflow-engine-gateway
cargo test integration_tests -- --ignored --nocapture
```

#### Individual Tests
```bash
cd crates/workflow-engine-gateway

# Test 16: Multi-Subgraph Queries
cargo test test_16_multi_subgraph_query -- --ignored --nocapture

# Test 17: Entity Resolution
cargo test test_17_entity_reference_resolution -- --ignored --nocapture

# Test 18: Schema Composition
cargo test test_18_schema_composition -- --ignored --nocapture
```

#### Health Checks
```bash
# Check service health and federation endpoints
./scripts/test_federation.sh health
```

## Test Categories

### Test 16: Multi-Subgraph Query Test

**Purpose:** Verify cross-service query execution and optimization.

**Sub-tests:**
- **16a:** Cross-service query spanning multiple subgraphs
- **16b:** Query with entity references across services  
- **16c:** Complex nested query with relationships
- **16d:** Batch query optimization test

**Example Queries:**
```graphql
# Cross-service query
query CrossServiceQuery {
  workflow(id: "wf_123") { id name status }
  content(id: "content_123") { id title contentType }
  searchConcepts(query: "rust") { concepts { id name } }
  conversations { id name type }
}

# Entity references query
query EntityReferencesQuery($userId: ID!) {
  user(id: $userId) {
    id
    processedContent { id title }
    completedConcepts { id name }
    conversations { id name }
  }
}
```

### Test 17: Entity Reference Resolution Test

**Purpose:** Test federation entity resolution mechanisms.

**Sub-tests:**
- **17a:** Basic entity resolution across services
- **17b:** Complex entity resolution with multiple keys
- **17c:** Entity resolution error handling
- **17d:** Federation directive compliance

**Example Entity Resolution:**
```graphql
query EntityResolution($representations: [_Any!]!) {
  _entities(representations: $representations) {
    ... on User { id __typename }
    ... on Workflow { id name status __typename }
    ... on ContentMetadata { id title __typename }
    ... on Concept { id name difficulty __typename }
  }
}
```

### Test 18: Schema Composition Test

**Purpose:** Verify proper schema composition and consistency.

**Sub-tests:**
- **18a:** Schema composition without conflicts
- **18b:** Type system consistency across subgraphs
- **18c:** Gateway introspection capabilities
- **18d:** Schema evolution compatibility

**Example Introspection:**
```graphql
query GatewayIntrospection {
  __schema {
    queryType { name }
    mutationType { name }
    subscriptionType { name }
    directives { name locations }
    types { name kind }
  }
}
```

## Federation Implementation Details

### Schema Registry
- Collects and stores subgraph schemas
- Validates federation directives
- Manages schema composition

### Query Planner
- Analyzes incoming queries
- Plans execution across subgraphs
- Optimizes query performance

### Entity Resolver
- Resolves entities across service boundaries
- Handles `_entities` queries
- Manages entity relationships

### Federation Directives

```graphql
# Entity key definition
type User @key(fields: "id") {
  id: ID!
  name: String
}

# Entity extension
extend type User @key(fields: "id") {
  processedContent: [ContentMetadata]
}

# External field reference
type ProcessingJob @key(fields: "id") {
  id: ID!
  user: User @external
}
```

## Test Data

### Sample Entities
```json
{
  "users": [
    {"id": "user_123", "name": "Test User"}
  ],
  "workflows": [
    {"id": "wf_123", "name": "Test Workflow", "status": "Active"}
  ],
  "content": [
    {"id": "content_123", "title": "Test Content", "type": "Markdown"}
  ],
  "concepts": [
    {"id": "concept_123", "name": "Rust Programming", "difficulty": "Intermediate"}
  ],
  "messages": [
    {"id": "msg_123", "content": "Hello World", "timestamp": "2023-01-01T00:00:00Z"}
  ]
}
```

## Error Handling

### Common Issues
1. **Service Not Running:** Ensure all 5 services are started
2. **Port Conflicts:** Check ports 4000, 8080, 3001-3003 are available
3. **Schema Conflicts:** Verify federation directives are properly implemented
4. **Entity Resolution Failures:** Check `_entities` resolver implementations

### Debug Commands
```bash
# Check service health
curl -X POST -H "Content-Type: application/json" \
  -d '{"query":"{ __schema { queryType { name } } }"}' \
  http://localhost:4000/graphql

# Test federation service query
curl -X POST -H "Content-Type: application/json" \
  -d '{"query":"{ _service { sdl } }"}' \
  http://localhost:8080/api/v1/graphql

# Test entity resolution
curl -X POST -H "Content-Type: application/json" \
  -d '{"query":"query { _entities(representations: [{\"__typename\": \"User\", \"id\": \"user_123\"}]) { ... on User { id __typename } } }"}' \
  http://localhost:4000/graphql
```

## Performance Considerations

### Query Optimization
- **Batch Loading:** Multiple entity requests in single query
- **Query Planning:** Optimal execution order across subgraphs
- **Caching:** Query plan and schema caching
- **Connection Pooling:** Efficient subgraph connections

### Monitoring
- Query execution time
- Subgraph response times
- Error rates and types
- Cache hit rates

## CI/CD Integration

### GitHub Actions Example
```yaml
name: Federation Tests
on: [push, pull_request]

jobs:
  federation-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Build
        run: cargo build
      - name: Start Services
        run: ./scripts/test_federation.sh start &
      - name: Wait for Services
        run: sleep 30
      - name: Run Federation Tests
        run: ./scripts/test_federation.sh test
      - name: Stop Services
        run: ./scripts/test_federation.sh stop
```

## Contributing

### Adding New Tests
1. Add test functions to `integration_tests.rs`
2. Follow naming convention: `test_XX_description`
3. Include comprehensive assertions
4. Add documentation and examples

### Extending Federation
1. Update subgraph schemas with federation directives
2. Implement `_entities` resolvers in services
3. Add corresponding integration tests
4. Update this documentation

## Troubleshooting

### Service Startup Issues
```bash
# Check if ports are available
netstat -tulpn | grep -E "(4000|8080|3001|3002|3003)"

# Check service logs
tail -f logs/*.log
```

### Federation Schema Issues
```bash
# Validate individual service schemas
cd services/content_processing
cargo run --bin validate_schema

# Check gateway schema composition
cd crates/workflow-engine-gateway  
cargo run --example validate_federation
```

### Query Execution Issues
```bash
# Test direct subgraph queries
curl -X POST -H "Content-Type: application/json" \
  -d '{"query":"{ __typename }"}' \
  http://localhost:8080/api/v1/graphql

# Test gateway query planning
cd crates/workflow-engine-gateway
cargo run --example test_query_planning
```