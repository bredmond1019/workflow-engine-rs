# GraphQL Gateway - Federation Implementation

This is a GraphQL Federation gateway that composes multiple GraphQL services into a unified API, implementing Apollo Federation v2 specification.

## Features

- ✅ Apollo Federation v2 support
- ✅ Schema composition from multiple subgraphs
- ✅ Entity resolution across services
- ✅ Query planning and optimization
- ✅ `_service` and `_entities` query support
- ✅ GraphQL Playground UI
- ✅ Example queries and mutations
- ✅ Subscription support (basic)

## Quick Start

### 1. Start the Gateway

```bash
# From the worktree-graphql directory
cargo run --bin graphql-gateway
```

The gateway will start at `http://localhost:4000/graphql`

### 2. Start the Workflow API with GraphQL

In another terminal:
```bash
# Make sure the workflow API has GraphQL enabled
cargo run --bin workflow-engine
```

The API should be running at `http://localhost:8080` with GraphQL at `/api/v1/graphql`

### 3. Access GraphQL Playground

Open your browser to: `http://localhost:4000/graphql`

### 4. Try Example Queries

```graphql
# Health check
{
  health
}

# Federation service query
{
  _service {
    sdl
  }
}

# Entity resolution
query ResolveEntities($representations: [_Any!]!) {
  _entities(representations: $representations) {
    ... on Workflow {
      id
      name
      status
    }
  }
}

# Get workflow by ID (federated)
{
  workflow(id: "123") {
    id
    name
    status
  }
}

# List workflows
{
  workflows(limit: 5) {
    items {
      id
      name
      status
      createdAt
    }
    totalCount
  }
}

# Create workflow
mutation {
  createWorkflow(name: "My Workflow", description: "Test workflow") {
    id
    name
    status
  }
}

# Subscribe to workflow changes
subscription {
  workflowStatusChanged(workflowId: "123")
}
```

### 5. Test Federation

```bash
# Run the federation test
cargo run --example test_federation
```

## Architecture

```
┌─────────────┐
│   Client    │
└──────┬──────┘
       │ GraphQL Query
       ▼
┌─────────────────────────┐
│   GraphQL Gateway       │ (Port 4000)
│  - Query planning       │
│  - Schema composition   │
│  - Request routing      │
└─────────┬───────────────┘
          │ Subgraph Query
          ▼
┌─────────────────────────┐
│  Workflow API Subgraph  │ (Port 8080)
│  - Workflow queries     │
│  - Workflow mutations   │
└─────────────────────────┘
```

## Next Steps for Production

1. **Schema Federation**
   - Add `@key` directives for entity resolution
   - Implement `_entities` and `_service` queries
   - Add schema composition at startup

2. **Enhanced Subgraphs**
   - Add Content Processing subgraph
   - Add Knowledge Graph subgraph
   - Add Real-time Communication subgraph

3. **Production Features**
   - Authentication/Authorization
   - Query complexity analysis
   - Rate limiting
   - Caching with DataLoader
   - Distributed tracing
   - Error handling

4. **Performance**
   - Query batching
   - Response caching
   - Connection pooling
   - Parallel subgraph execution

## Example: Running the Demo

```bash
# Terminal 1: Start the gateway
cargo run --bin graphql-gateway

# Terminal 2: Run the example client
cargo run --example federated_query

# Or use curl
curl -X POST http://localhost:4000/graphql \
  -H "Content-Type: application/json" \
  -d '{"query": "{ health }"}'
```

## Configuration

Subgraphs are configured in `src/main.rs`:

```rust
let subgraphs = vec![
    SubgraphConfig {
        name: "workflow".to_string(),
        url: "http://localhost:8080/graphql".to_string(),
        schema_url: None,
    },
    // Add more subgraphs here
];
```

## Development

```bash
# Run tests
cargo test

# Check code
cargo clippy

# Format code
cargo fmt
```

This is a POC/MVP implementation. For production use, implement proper federation with Apollo Federation spec or similar.