## Relevant Files

- `src/core/auth/jwt.rs` - JWT token validation middleware and claims structure (created)
- `src/core/auth/mod.rs` - Auth module exports (created)
- `src/core/auth/jwt_test.rs` - Unit tests for JWT authentication
- `src/api/middleware/auth.rs` - Authentication middleware for Actix-web (created)
- `src/api/middleware/mod.rs` - Middleware module exports (created)
- `src/db/schema.rs` - Database schema including agents table
- `src/db/repository.rs` - Database repository for agent operations
- `src/core/registry/agent_registry.rs` - Agent registry trait and implementation (created)
- `src/core/registry/mod.rs` - Registry module exports (created)
- `src/db/agent.rs` - Agent database models (created)
- `migrations/create_agents_table.sql` - Database migration for agents table (created)
- `src/core/mcp/transport/http.rs` - HTTP transport implementation for MCP
- `src/core/mcp/transport/http_test.rs` - Unit tests for HTTP transport
- `src/core/models/unified.rs` - Unified task and service message models
- `src/core/models/unified_test.rs` - Unit tests for unified models
- `src/api/routes/auth.rs` - Authentication API endpoints (created)
- `src/api/routes/mod.rs` - Routes module configuration (created)
- `src/api/routes/registry.rs` - Agent registry API endpoints (created)
- `src/core/registry/background_tasks.rs` - Background tasks for agent cleanup (created)
- `src/api/routes/health.rs` - Health check endpoint
- `src/bootstrap/service.rs` - Service bootstrap utility
- `src/bootstrap/service_test.rs` - Unit tests for service bootstrap
- `docker-compose.yml` - Docker Compose configuration for local development
- `.env.example` - Example environment configuration

### Notes

- Unit tests should typically be placed alongside the code files they are testing (e.g., `jwt.rs` and `jwt_test.rs` in the same directory).
- Use `cargo test` to run all tests or `cargo test [module_name]` to run specific test modules.

## Tasks

- [x] 1.0 Implement Minimal JWT Authentication System
  - [x] 1.1 Create JWT validation middleware with HS256 symmetric key support
  - [x] 1.2 Define Claims struct with sub, exp, and role fields
  - [x] 1.3 Implement token generation endpoint for development (POST /auth/token)
  - [x] 1.4 Create token verification endpoint (GET /auth/verify)
  - [x] 1.5 Add authentication middleware to protected routes
  - [x] 1.6 Write unit tests for JWT validation and middleware
  - [x] 1.7 Test 401 responses for invalid/expired tokens

- [x] 2.0 Create Simple Agent Registry with Database Schema
  - [x] 2.1 Create agents table schema with UUID, name, endpoint, capabilities, status, and metadata fields
  - [x] 2.2 Add database indexes for capabilities (GIN) and status columns
  - [x] 2.3 Implement AgentRegistry trait with register, discover, heartbeat, and list_active methods
  - [x] 2.4 Create AgentRegistration struct for registration payload
  - [x] 2.5 Implement POST /registry/agents endpoint for agent registration
  - [x] 2.6 Implement GET /registry/agents endpoint to list active agents
  - [x] 2.7 Implement GET /registry/agents/discover endpoint for capability-based discovery
  - [x] 2.8 Implement POST /registry/agents/{id}/heartbeat endpoint
  - [x] 2.9 Create background task to mark agents inactive after 5 minutes without heartbeat
  - [x] 2.10 Add validation to prevent duplicate agent names
  - [x] 2.11 Write unit tests for registry operations

- [x] 3.0 Build HTTP Transport and Core Data Models
  - [x] 3.1 Extend transport enum to include HttpTransport implementation
  - [x] 3.2 Implement send_request method with authentication headers
  - [x] 3.3 Add timeout handling with 30-second default
  - [x] 3.4 Create UnifiedTask model with id, type_name, input, status, created_by, and created_at fields
  - [x] 3.5 Create ServiceMessage model with from_service, to_service, correlation_id, payload, and timestamp
  - [x] 3.6 Define TaskStatus enum (Pending, Running, Completed, Failed)
  - [x] 3.7 Implement serialization/deserialization for all models
  - [x] 3.8 Write unit tests for HTTP transport and model serialization

- [x] 4.0 Develop Service Bootstrap and Integration Testing
  - [x] 4.1 Create bootstrap_service function for automatic service registration
  - [x] 4.2 Implement heartbeat task spawning in bootstrap function
  - [x] 4.3 Create integration test for service discovery workflow
  - [x] 4.4 Test HTTP transport with registered services
  - [x] 4.5 Implement health check endpoint (GET /health) without authentication
  - [x] 4.6 Document all API endpoints and models
  - [x] 4.7 Create example service registration code

- [x] 5.0 Set Up Docker Compose Development Environment
  - [x] 5.1 Configure PostgreSQL service with dev credentials
  - [x] 5.2 Set up registry service with environment variables
  - [x] 5.3 Configure ai-tutor service with auto-registration
  - [x] 5.4 Configure workflow-system service with auto-registration
  - [x] 5.5 Create .env.example file with all required variables
  - [x] 5.6 Write setup instructions for local development
  - [x] 5.7 Test complete stack startup and service discovery