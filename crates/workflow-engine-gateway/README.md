# workflow-engine-gateway

GraphQL Federation gateway for composing distributed GraphQL services into a unified API.

[![Crates.io](https://img.shields.io/crates/v/workflow-engine-gateway.svg)](https://crates.io/crates/workflow-engine-gateway)
[![Documentation](https://docs.rs/workflow-engine-gateway/badge.svg)](https://docs.rs/workflow-engine-gateway)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

## Features

- **Apollo Federation v2**: Full specification compliance with entity resolution
- **Schema Composition**: Automatic merging of subgraph schemas
- **Query Planning**: Intelligent query distribution and optimization
- **Performance**: Query caching, batching, and parallel execution
- **Health Monitoring**: Subgraph health checks with circuit breakers
- **Security**: Authentication propagation and query complexity limits
- **Developer Experience**: GraphQL Playground and introspection
- **Production Ready**: Monitoring, tracing, and error handling

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
workflow-engine-gateway = "0.6.0"
tokio = { version = "1.0", features = ["full"] }
```

Create a gateway server:

```rust
use workflow_engine_gateway::{Gateway, SubgraphConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure subgraphs
    let subgraphs = vec![
        SubgraphConfig {
            name: "workflow".to_string(),
            url: "http://localhost:8080/graphql".to_string(),
            schema_url: None,
        },
        SubgraphConfig {
            name: "content".to_string(), 
            url: "http://localhost:8082/graphql".to_string(),
            schema_url: None,
        },
    ];
    
    // Create and start gateway
    let gateway = Gateway::builder()
        .port(4000)
        .subgraphs(subgraphs)
        .enable_playground(true)
        .build()?;
        
    gateway.serve().await?;
    Ok(())
}
```

## Usage Examples

### Basic Queries

```graphql
# Query across services with entity resolution
{
  workflow(id: "123") {
    id
    name
    status
    createdBy {      # Resolved from User service
      email
      name
    }
    content {        # Resolved from Content service
      documents {
        title
        url
      }
    }
  }
}

# Federation introspection
{
  _service {
    sdl
  }
}
```

### Advanced Configuration

```rust
use workflow_engine_gateway::{Gateway, GatewayConfig, SecurityConfig};

let config = GatewayConfig {
    port: 4000,
    host: "0.0.0.0".to_string(),
    playground: true,
    introspection: true,
    query_cache_size: 1000,
    max_query_depth: 10,
    enable_batching: true,
};

let security = SecurityConfig {
    enable_auth: true,
    auth_header: "Authorization".to_string(),
    max_query_complexity: 1000,
    rate_limit_per_minute: 60,
};

let gateway = Gateway::builder()
    .config(config)
    .security(security)
    .health_check_interval(Duration::from_secs(30))
    .build()?;

```

### Custom Directives

```rust
use workflow_engine_gateway::DirectiveHandler;

struct RateLimitDirective;

impl DirectiveHandler for RateLimitDirective {
    fn name(&self) -> &str {
        "rateLimit"
    }
    
    async fn process(&self, ctx: &DirectiveContext, next: Next) -> Result<Value> {
        // Apply rate limiting logic
        next.run(ctx).await
    }
}

gateway.register_directive(Box::new(RateLimitDirective));
```

## Architecture

```
┌─────────────┐
│   Client    │
└──────┬──────┘
       │ GraphQL Query
       ▼
┌─────────────────────────┐
│   GraphQL Gateway       │
│  • Query planning       │
│  • Schema composition   │
│  • Entity resolution    │
│  • Response caching     │
└─────────┬───────────────┘
          │ Parallel execution
    ┌─────┴─────┬─────────┐
    ▼           ▼         ▼
┌─────────┐ ┌─────────┐ ┌─────────┐
│Workflow │ │ Content │ │Knowledge│
│Subgraph │ │Subgraph │ │Subgraph │
└─────────┘ └─────────┘ └─────────┘
```

## Federation Features

### Entity Resolution
```graphql
# Define entities with @key directive
type Workflow @key(fields: "id") {
  id: ID!
  name: String!
  createdBy: User! @external
}

type User @key(fields: "id") {
  id: ID!
  workflows: [Workflow!]!
}
```

### Schema Extensions
```graphql
# Extend types across services
extend type Workflow {
  analytics: WorkflowAnalytics!
}

extend type User {
  preferences: UserPreferences!
}
```

## Feature Flags

- `default = ["playground", "introspection"]` - Development features
- `playground` - GraphQL Playground UI
- `introspection` - Schema introspection
- `auth` - Authentication support
- `tracing` - Distributed tracing
- `cache` - Query result caching
- `full` - All features enabled

## Performance

### Query Optimization
- Automatic query planning for optimal execution
- Parallel subgraph queries when possible
- Intelligent result caching
- Request batching to reduce round trips

### Monitoring
```bash
# Metrics endpoint
GET /metrics

# Health check
GET /health

# Detailed health with subgraph status
GET /health/detailed
```

## Testing

```bash
# Unit tests
cargo test -p workflow-engine-gateway

# Integration tests
cargo test -p workflow-engine-gateway -- --ignored

# Federation validation
cargo run --example validate_federation

# Load testing
cargo run --example load_test_gateway
```

## Configuration

### Environment Variables
```bash
GATEWAY_PORT=4000
GATEWAY_HOST=0.0.0.0
ENABLE_PLAYGROUND=true
ENABLE_INTROSPECTION=false

# Subgraph URLs
WORKFLOW_SUBGRAPH_URL=http://workflow-api:8080/graphql
CONTENT_SUBGRAPH_URL=http://content-api:8082/graphql

# Performance
QUERY_CACHE_SIZE=1000
MAX_QUERY_DEPTH=10
QUERY_TIMEOUT_MS=30000

# Security
JWT_PUBLIC_KEY=<public-key>
MAX_QUERY_COMPLEXITY=1000
```

## Documentation

For comprehensive documentation, visit [docs.rs/workflow-engine-gateway](https://docs.rs/workflow-engine-gateway).

## Examples

See the [examples directory](examples/) for:
- Federation validation
- Performance testing
- Custom directive implementation
- Subgraph integration patterns

## Dependencies

This crate depends on:
- `workflow-engine-core` - Core types
- `juniper` - GraphQL implementation
- `tokio` - Async runtime
- `reqwest` - HTTP client for subgraphs

## Contributing

Contributions are welcome! Please read our [Contributing Guide](../../CONTRIBUTING.md) for details.

## License

Licensed under the MIT License. See [LICENSE](../../LICENSE) for details.