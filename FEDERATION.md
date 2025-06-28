# GraphQL Federation Architecture

This document describes the GraphQL Federation setup for the AI Workflow Engine, implemented using **Test-Driven Development (TDD)** methodology.

## Overview

The system uses Apollo Federation v2 to unify multiple GraphQL services into a single gateway API. This provides:

- **Unified API**: Single GraphQL endpoint for all services
- **Service Independence**: Each service maintains its own schema and logic
- **Cross-Service Queries**: Seamlessly query data across multiple services
- **Partial Failure Handling**: Graceful degradation when services are unavailable

## Architecture

```
┌─────────────────┐     ┌─────────────────────┐
│   Frontend      │────▶│  GraphQL Gateway    │
│ (React + Apollo)│     │   (Port 4000)       │
└─────────────────┘     └──────────┬──────────┘
                                   │
        ┌──────────────────────────┼──────────────────────────┐
        │                          │                          │
        ▼                          ▼                          ▼
┌───────────────┐       ┌──────────────────┐      ┌──────────────────┐
│  Workflow API │       │Content Processing│      │ Knowledge Graph  │
│  (Port 8080)  │       │   (Port 8082)    │      │  (Port 3002)    │
└───────────────┘       └──────────────────┘      └──────────────────┘
                                   │
                                   ▼
                        ┌──────────────────┐
                        │Realtime Comm.    │
                        │  (Port 8081)     │
                        └──────────────────┘
```

## Service Endpoints

### GraphQL Gateway
- **URL**: `http://localhost:4000/graphql`
- **Health**: `http://localhost:4000/health`
- **WebSocket**: `ws://localhost:4000/graphql`

### Subgraph Services

| Service | GraphQL Endpoint | Health Check | Description |
|---------|-----------------|--------------|-------------|
| Workflow API | `http://localhost:8080/api/v1/graphql` | `/health` | Core workflow management |
| Content Processing | `http://localhost:8082/graphql` | `/health` | Document analysis & WASM plugins |
| Knowledge Graph | `http://localhost:3002/graphql` | `/health` | Graph database integration |
| Realtime Communication | `http://localhost:8081/graphql` | `/health` | WebSocket messaging |

## TDD Implementation Journey

### 1. RED Phase - Writing Failing Tests

We started by writing comprehensive tests that defined our expected behavior:

```rust
// Backend federation configuration tests
#[test]
fn test_gateway_uses_correct_service_ports() {
    // Verify all services use correct ports
}

#[test]
fn test_gateway_handles_service_discovery_from_env() {
    // Test environment-based configuration
}

#[test]
fn test_gateway_validates_subgraph_health_on_startup() {
    // Ensure health checks work
}
```

```typescript
// Frontend federation client tests
describe('FederationClient', () => {
  it('should execute cross-service queries through gateway');
  it('should handle partial service failures gracefully');
  it('should support entity resolution across services');
  it('should check gateway health status');
  it('should handle federated subscriptions');
});
```

### 2. GREEN Phase - Making Tests Pass

Implemented the minimum code required to pass all tests:

- Created `GatewayConfig` with correct port mappings
- Implemented `FederationClient` extending GraphQLClient
- Added health checking functionality
- Configured Docker Compose integration

### 3. REFACTOR Phase - Improving Code Quality

Applied "Tidy First" principles to clean up the implementation:

- Extracted configuration into dedicated modules
- Created reusable health checking components
- Standardized error handling patterns
- Added comprehensive documentation

## Running the Federation Stack

### Quick Start

```bash
# Start all services
./scripts/run-federation-stack.sh

# Test federation connectivity
./scripts/test-federation.sh

# Run integration tests
cargo test graphql_federation_integration_test -- --ignored
```

### Docker Compose

```bash
# Start with Docker Compose
docker-compose up -d

# View logs
docker-compose logs -f graphql-gateway
```

### Manual Service Startup

```bash
# 1. Start backend services
cargo run --bin workflow-engine            # Port 8080
cd services/content_processing && cargo run # Port 8082
cd services/knowledge_graph && cargo run    # Port 3002
cd services/realtime_communication && cargo run # Port 8081

# 2. Start GraphQL Gateway
cargo run --bin graphql-gateway            # Port 4000

# 3. Start frontend (optional)
cd frontend && npm start                   # Port 3000
```

## Federation Features

### Cross-Service Queries

Query data from multiple services in a single request:

```graphql
query FederatedWorkflow($workflowId: ID!) {
  workflow(id: $workflowId) {
    # From workflow-api service
    id
    name
    nodes { id type }
    
    # From content-processing service
    processedContent {
      summary
      extractedEntities
    }
    
    # From knowledge-graph service
    relatedKnowledge {
      concepts
      relationships { from to type }
    }
    
    # From realtime-communication service
    activeCollaborators {
      userId
      presence
    }
  }
}
```

### Entity Resolution

Services can extend entities from other services:

```graphql
# In workflow-api service
type Workflow @key(fields: "id") {
  id: ID!
  name: String!
  nodes: [Node!]!
}

# In content-processing service
extend type Workflow @key(fields: "id") {
  id: ID! @external
  processedContent: ProcessedContent
}
```

### Partial Failure Handling

The gateway gracefully handles service failures:

```json
{
  "data": {
    "workflow": {
      "id": "123",
      "name": "My Workflow",
      "processedContent": null
    }
  },
  "errors": [{
    "message": "Content processing service unavailable",
    "path": ["workflow", "processedContent"],
    "extensions": {
      "code": "SERVICE_UNAVAILABLE",
      "service": "content-processing"
    }
  }]
}
```

## Frontend Integration

The frontend uses a specialized `FederationClient`:

```typescript
import { FederationClient } from '@/api/graphql/FederationClient';

const client = new FederationClient('http://localhost:4000/graphql');

// Execute federated queries
const result = await client.query(FEDERATED_QUERY, variables);

// Check service health
const health = await client.checkHealth();

// Subscribe to real-time updates
const unsubscribe = await client.subscribe(
  SUBSCRIPTION,
  variables,
  {
    next: (data) => console.log('Update:', data),
    error: (err) => console.error('Error:', err),
    complete: () => console.log('Complete')
  }
);
```

## Configuration

### Environment Variables

Configure service URLs via environment:

```bash
# Gateway configuration
WORKFLOW_API_URL=http://api:8080/api/v1/graphql
CONTENT_PROCESSING_URL=http://content:8082/graphql
KNOWLEDGE_GRAPH_URL=http://knowledge:3002/graphql
REALTIME_COMM_URL=http://realtime:8081/graphql
```

### Docker Compose Configuration

```yaml
graphql-gateway:
  build:
    context: .
    target: gateway
  ports:
    - "4000:4000"
  environment:
    - WORKFLOW_API_URL=http://workflow-api:8080/api/v1/graphql
    - CONTENT_PROCESSING_URL=http://content-processing:8082/graphql
  depends_on:
    - workflow-api
    - content-processing
    - knowledge-graph
    - realtime-communication
```

## Testing

### Unit Tests

```bash
# Test gateway configuration
cargo test -p workflow-engine-gateway

# Test frontend federation client
cd frontend && npm test FederationClient.test.ts
```

### Integration Tests

```bash
# Run federation integration tests
cargo test graphql_federation_integration_test -- --ignored

# Test with running services
./scripts/run-federation-stack.sh
./scripts/test-federation.sh
```

### Manual Testing

Access the GraphQL Playground:
- Gateway: http://localhost:4000/graphql
- Individual services: See service endpoints table above

## Troubleshooting

### Common Issues

1. **Port Conflicts**
   ```bash
   # Check if ports are in use
   lsof -i :4000
   lsof -i :8080
   ```

2. **Service Discovery Failures**
   - Ensure all services are running
   - Check health endpoints manually
   - Verify network connectivity in Docker

3. **Schema Composition Errors**
   - Run schema validation: `./validate_federation.sh`
   - Check for conflicting type definitions
   - Ensure federation directives are correct

4. **Subscription Connection Issues**
   - Verify WebSocket support in gateway
   - Check CORS configuration
   - Ensure authentication tokens are passed

## Best Practices

1. **Service Independence**: Each service should be deployable independently
2. **Schema Ownership**: Services own their data and expose only what's needed
3. **Error Boundaries**: Handle partial failures gracefully
4. **Performance**: Use DataLoader pattern for N+1 query prevention
5. **Monitoring**: Track query complexity and service latency

## Future Enhancements

- [ ] Implement query complexity analysis
- [ ] Add distributed tracing support
- [ ] Implement schema versioning
- [ ] Add automatic service discovery
- [ ] Implement rate limiting per service
- [ ] Add GraphQL subscription scaling

## Conclusion

The GraphQL Federation implementation provides a robust, scalable architecture for the AI Workflow Engine. By following TDD principles, we've created a well-tested, maintainable system that elegantly handles the complexity of distributed GraphQL services.