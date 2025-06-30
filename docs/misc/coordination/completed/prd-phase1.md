# PRD Phase 1 MVP: Minimal Foundation for Integration

## Document Status

- **Created**: January 2025
- **Type**: Minimum Viable Product (MVP) Specification
- **Timeline**: 4 weeks
- **Goal**: Unblock Phase 2 integration with minimal but solid foundation

## Executive Summary

This PRD defines a streamlined Phase 1 that implements only the critical components needed to enable system integration. By focusing on the absolute essentials—basic authentication, simple service discovery, and core data models—we can quickly unblock Phase 2 work while maintaining architectural integrity.

## Goals

1. **Enable Service Discovery**: Implement minimal agent registry for systems to find each other
2. **Add Security Layer**: Basic JWT authentication that can be enhanced later
3. **Standardize Communication**: Complete HTTP transport and essential data models
4. **Maintain Momentum**: Deliver working foundation in 4 weeks

## Core Components

### Week 1: Minimal Authentication

#### JWT Token Validation

```rust
// Simple JWT middleware for Actix-web
pub struct JwtAuth;

impl JwtAuth {
    pub fn validate_token(token: &str) -> Result<Claims, Error> {
        // Start with symmetric key (HS256)
        // Move to RS256 with Auth0 in Phase 1.5
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(SECRET.as_ref()),
            &Validation::default()
        )
    }
}

// Minimal claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // User ID
    pub exp: usize,   // Expiration time
    pub role: String, // Simple role: admin|developer|service
}
```

#### Functional Requirements

1. The system must validate JWT tokens on protected endpoints
2. The system must support service-to-service authentication
3. The system must provide a temporary token generation endpoint for development
4. The system must return 401 for invalid/expired tokens

#### Out of Scope (Defer to Phase 1.5)

- Auth0 integration
- Google OAuth
- Refresh tokens
- Complex RBAC

### Week 2: Simple Agent Registry

#### Database Schema

```sql
-- Minimal agent registry
CREATE TABLE agents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    endpoint VARCHAR(500) NOT NULL,
    capabilities TEXT[] NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    last_seen TIMESTAMP NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'::jsonb
);

CREATE INDEX idx_agents_capabilities ON agents USING GIN(capabilities);
CREATE INDEX idx_agents_status ON agents(status);
```

#### Registry API

```rust
// Core registry operations
pub trait AgentRegistry {
    async fn register(&self, agent: AgentRegistration) -> Result<Agent>;
    async fn discover(&self, capability: &str) -> Result<Vec<Agent>>;
    async fn heartbeat(&self, agent_id: &Uuid) -> Result<()>;
    async fn list_active(&self) -> Result<Vec<Agent>>;
}

// Simple registration model
pub struct AgentRegistration {
    pub name: String,
    pub endpoint: String,
    pub capabilities: Vec<String>,
}
```

#### Functional Requirements

5. The system must allow agents to register with name, endpoint, and capabilities
6. The system must support capability-based discovery
7. The system must mark agents as inactive after 5 minutes without heartbeat
8. The system must prevent duplicate agent names

#### Out of Scope

- Complex health checks
- Agent versioning
- Hierarchical capabilities
- Multi-region support

### Week 3: HTTP Transport & Data Models

#### HTTP MCP Transport

```rust
// Extend existing transport enum
impl HttpTransport {
    pub async fn send_request(&self, request: MCPRequest) -> Result<MCPResponse> {
        let client = reqwest::Client::new();
        let response = client
            .post(&self.base_url)
            .json(&request)
            .header("Authorization", &self.auth_token)
            .send()
            .await?;

        response.json::<MCPResponse>().await
    }
}
```

#### Core Data Models

```rust
// Minimal shared models
#[derive(Serialize, Deserialize, Clone)]
pub struct UnifiedTask {
    pub id: Uuid,
    pub type_name: String,
    pub input: Value,
    pub status: TaskStatus,
    pub created_by: String,  // JWT subject
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ServiceMessage {
    pub from_service: String,
    pub to_service: String,
    pub correlation_id: Uuid,
    pub payload: Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}
```

#### Functional Requirements

9. The system must support HTTP as an MCP transport option
10. The system must include authentication headers in HTTP requests
11. The system must implement timeout handling (30s default)
12. The system must serialize/deserialize standard data models

### Week 4: Integration & Testing

#### Service Bootstrap

```rust
// Auto-registration on startup
pub async fn bootstrap_service(
    name: &str,
    endpoint: &str,
    capabilities: Vec<&str>,
    registry_url: &str,
) -> Result<()> {
    let registry_client = RegistryClient::new(registry_url);

    // Register self
    let agent = registry_client.register(AgentRegistration {
        name: name.to_string(),
        endpoint: endpoint.to_string(),
        capabilities: capabilities.iter().map(|c| c.to_string()).collect(),
    }).await?;

    // Start heartbeat task
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            let _ = registry_client.heartbeat(&agent.id).await;
        }
    });

    Ok(())
}
```

#### Integration Test Suite

```rust
#[cfg(test)]
mod integration_tests {
    #[tokio::test]
    async fn test_service_discovery() {
        // Register a test service
        let agent = registry.register(test_agent()).await.unwrap();

        // Discover by capability
        let found = registry.discover("test_capability").await.unwrap();
        assert_eq!(found.len(), 1);

        // Verify HTTP transport works
        let transport = HttpTransport::new(&agent.endpoint);
        let response = transport.send_request(test_request()).await.unwrap();
        assert!(response.is_success());
    }
}
```

#### Functional Requirements

13. The system must provide a bootstrap function for service registration
14. The system must include integration tests for cross-service communication
15. The system must provide Docker Compose for local development
16. The system must document all endpoints and models

## API Endpoints

### Authentication (Minimal)

- `POST /auth/token` - Generate development JWT token
- `GET /auth/verify` - Verify token validity

### Agent Registry

- `POST /registry/agents` - Register new agent
- `GET /registry/agents` - List all active agents
- `GET /registry/agents/discover?capability={cap}` - Find agents by capability
- `POST /registry/agents/{id}/heartbeat` - Update last seen time

### Health

- `GET /health` - Basic health check (no auth required)

## Technical Decisions

### Why This Approach?

1. **Simple JWT**: Faster to implement than Auth0, can upgrade later
2. **Basic Registry**: Just enough for service discovery
3. **Focus on HTTP**: Most universal transport, WebSocket already works
4. **Minimal Models**: Only what's needed for integration

### Technology Stack

- **Auth**: jsonwebtoken crate with HS256
- **Database**: Existing PostgreSQL
- **HTTP Client**: reqwest with timeout support
- **Testing**: Built-in Rust test framework

## Success Metrics

1. **Completion Time**: Delivered within 4 weeks
2. **Service Discovery**: All services can find each other
3. **Authentication**: All endpoints protected except health
4. **Integration Test**: Cross-service call succeeds
5. **Documentation**: API fully documented

## Migration Path

### To Full Phase 1

1. **Auth Enhancement**: Add Auth0 integration while keeping JWT validation
2. **Registry Features**: Add health monitoring, versioning, groups
3. **Model Extension**: Add validation, schemas, transformations
4. **Monitoring**: Add metrics and logging

### Backward Compatibility

- JWT tokens will work with future Auth0 implementation
- Registry API will be extended, not changed
- HTTP transport is additive to existing transports

## Docker Compose Development Environment

```yaml
version: "3.8"
services:
  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: ai_system
      POSTGRES_USER: ai_user
      POSTGRES_PASSWORD: dev_password
    ports:
      - "5432:5432"

  registry:
    build: .
    environment:
      DATABASE_URL: postgresql://ai_user:dev_password@postgres/ai_system
      JWT_SECRET: dev_secret_change_in_production
      RUST_LOG: info
    ports:
      - "8080:8080"
    depends_on:
      - postgres

  ai-tutor:
    image: ai-tutor:latest
    environment:
      REGISTRY_URL: http://registry:8080
      SERVICE_NAME: ai-tutor
      SERVICE_ENDPOINT: http://ai-tutor:8000
    ports:
      - "8000:8000"
    depends_on:
      - registry

  workflow-system:
    build: .
    environment:
      REGISTRY_URL: http://registry:8080
      SERVICE_NAME: workflow-system
      SERVICE_ENDPOINT: http://workflow-system:8081
    ports:
      - "8081:8081"
    depends_on:
      - registry
```

## Development Checklist

### Week 1 Deliverables

- [ ] JWT validation middleware
- [ ] Token generation endpoint
- [ ] Protected route tests
- [ ] Development tokens for testing

### Week 2 Deliverables

- [ ] Agent database schema
- [ ] Registry API implementation
- [ ] Service discovery logic
- [ ] Heartbeat mechanism

### Week 3 Deliverables

- [ ] HTTP transport for MCP
- [ ] Core data models
- [ ] Serialization tests
- [ ] Transport integration

### Week 4 Deliverables

- [ ] Service bootstrap utility
- [ ] Integration test suite
- [ ] Docker Compose setup
- [ ] API documentation

## Risk Mitigation

### Risk: Scope Creep

**Mitigation**: Strictly enforce "out of scope" items. Create Phase 1.5 PRD for enhancements.

### Risk: Integration Issues

**Mitigation**: Week 4 dedicated to integration testing. Daily integration tests from Week 3.

### Risk: Performance

**Mitigation**: Simple implementations optimized later. Focus on correctness first.

## Conclusion

This MVP approach delivers a working foundation in 4 weeks by focusing only on what's essential for integration. It provides real value quickly while maintaining a clear upgrade path to the full Phase 1 vision. The key is resisting feature creep and delivering a solid, simple foundation that unblocks Phase 2 work.
