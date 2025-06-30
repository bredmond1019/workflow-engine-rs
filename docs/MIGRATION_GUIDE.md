# Migration Guide

## Overview

This guide provides step-by-step instructions for migrating between versions of the AI Workflow Engine, including breaking changes, configuration updates, and best practices for smooth upgrades.

## Migration Checklist

Before starting any migration:

- [ ] **Backup your data**: Full database and configuration backup
- [ ] **Review release notes**: Check for breaking changes and new features
- [ ] **Test in staging**: Always test migrations in a non-production environment
- [ ] **Plan downtime**: Schedule maintenance windows for production updates
- [ ] **Prepare rollback**: Have a rollback plan ready

## Version Migration Paths

### From v0.x to v1.0 (Major Release)

#### Breaking Changes
- **JWT Configuration**: `JWT_SECRET` is now required (no default value)
- **Database Schema**: New event sourcing tables and indices
- **API Endpoints**: GraphQL Federation gateway now required
- **Configuration Format**: Environment variables renamed and restructured

#### Migration Steps

1. **Update Configuration**
   ```bash
   # Old configuration
   export JWT_SECRET_KEY="your-secret"
   export DB_URL="postgresql://..."
   
   # New configuration
   export JWT_SECRET="your-secure-jwt-secret"  # Required, no default
   export DATABASE_URL="postgresql://..."
   export GRAPHQL_GATEWAY_URL="http://localhost:4000"
   ```

2. **Database Migration**
   ```bash
   # Backup existing database
   pg_dump ai_workflow_db > backup_pre_v1.sql
   
   # Run new migrations
   diesel migration run
   
   # Verify migration success
   diesel migration status
   ```

3. **Update Docker Configuration**
   ```yaml
   # docker-compose.yml changes
   services:
     ai-workflow-system:
       image: workflow-engine:v1.0
       environment:
         - JWT_SECRET=${JWT_SECRET}  # Now required
         - DATABASE_URL=${DATABASE_URL}
       ports:
         - "8080:8080"
     
     graphql-gateway:  # New service
       image: workflow-engine-gateway:v1.0
       ports:
         - "4000:4000"
       depends_on:
         - ai-workflow-system
   ```

4. **Frontend Updates**
   ```typescript
   // Update API endpoints
   const API_BASE_URL = process.env.REACT_APP_API_URL || 'http://localhost:8080';
   const GRAPHQL_URL = process.env.REACT_APP_GRAPHQL_URL || 'http://localhost:4000/graphql';
   
   // Update authentication
   const authConfig = {
     tokenKey: 'auth_token',  // Changed from 'jwt_token'
     refreshTokenKey: 'refresh_token',
     tokenExpiration: 3600000  // 1 hour
   };
   ```

#### Post-Migration Verification

```bash
# Verify services are running
curl http://localhost:8080/health/detailed
curl http://localhost:4000/health

# Run integration tests
cargo test -- --ignored

# Check frontend functionality
cd frontend && npm test
```

### From v1.x to v1.y (Minor Updates)

Minor version updates typically include:
- New optional features
- Performance improvements
- Security patches
- Non-breaking API additions

#### Standard Minor Update Process

1. **Review Release Notes**
   ```bash
   # Check what's new
   git log v1.x..v1.y --oneline
   ```

2. **Update Dependencies**
   ```bash
   # Update Rust dependencies
   cargo update
   
   # Update frontend dependencies
   cd frontend && npm update
   ```

3. **Apply Configuration Changes**
   ```bash
   # Check for new optional configuration
   grep -r "env::" src/ | grep -v ".git"
   ```

4. **Run Tests**
   ```bash
   # Full test suite
   cargo test --all
   cd frontend && npm test
   ```

### From v1.y.x to v1.y.z (Patch Updates)

Patch updates include:
- Bug fixes
- Security patches
- Documentation updates
- Minor improvements

#### Quick Patch Update

```bash
# Simple update process
docker-compose pull
docker-compose up -d

# Verify update
curl http://localhost:8080/health
```

## Database Migrations

### Automatic Migrations

The system includes automatic database migration support:

```rust
// Automatic migration on startup (configurable)
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if std::env::var("AUTO_MIGRATE").unwrap_or_default() == "true" {
        run_migrations().await?;
    }
    // ... rest of startup
}
```

### Manual Migration Process

For production environments, manual migration is recommended:

```bash
# 1. Backup database
pg_dump ai_workflow_db > backup_$(date +%Y%m%d_%H%M%S).sql

# 2. Check migration status
diesel migration status

# 3. Run migrations
diesel migration run

# 4. Verify schema
psql ai_workflow_db -c "\dt"
```

### Migration Rollback

If migration fails:

```bash
# Rollback migrations
diesel migration revert

# Restore from backup
dropdb ai_workflow_db
createdb ai_workflow_db
psql ai_workflow_db < backup_pre_migration.sql
```

## Configuration Migration

### Environment Variable Changes

| Version | Old Variable | New Variable | Notes |
|---------|-------------|--------------|-------|
| v0.x → v1.0 | `JWT_SECRET_KEY` | `JWT_SECRET` | Now required, no default |
| v0.x → v1.0 | `DB_URL` | `DATABASE_URL` | Standard naming |
| v0.x → v1.0 | `LOG_LEVEL` | `RUST_LOG` | Standard Rust logging |
| v1.0 → v1.1 | N/A | `GRAPHQL_INTROSPECTION` | New security feature |

### Configuration File Updates

```toml
# Old configuration (v0.x)
[server]
host = "0.0.0.0"
port = 8080

[database]
url = "postgresql://..."

[jwt]
secret = "default-secret"  # Security issue!

# New configuration (v1.0+)
[server]
host = "0.0.0.0"
port = 8080
cors_origins = ["http://localhost:3000"]

[database]
url = "postgresql://..."
pool_size = 10
timeout = 30

[auth]
jwt_secret = "${JWT_SECRET}"  # From environment
token_expiration = 3600

[federation]
gateway_url = "http://localhost:4000"
```

## API Migration

### GraphQL Schema Changes

#### v0.x to v1.0
- Added GraphQL Federation support
- New `_service` and `_entities` queries
- Schema composition across microservices

```graphql
# Old schema (v0.x)
type Query {
  workflows: [Workflow]
  users: [User]
}

# New federated schema (v1.0)
type Query {
  workflows: [Workflow]
  users: [User]
  _service: _Service!
}

type Workflow @key(fields: "id") {
  id: ID!
  name: String!
  # ... other fields
}
```

#### Client Updates Required

```typescript
// Update GraphQL client configuration
const client = new ApolloClient({
  uri: 'http://localhost:4000/graphql',  // Federation gateway
  cache: new InMemoryCache({
    possibleTypes: {
      // Add federation types
      _Entity: ['Workflow', 'User', 'Node']
    }
  })
});
```

### REST API Changes

Most REST API endpoints remain backward compatible. Changes include:

- **Authentication**: Bearer token format unchanged
- **Rate Limiting**: New headers added (non-breaking)
- **Error Format**: Enhanced error responses (backward compatible)

## Frontend Migration

### React Component Updates

```typescript
// v0.x - Old authentication hook
const useAuth = () => {
  const [token, setToken] = useState(localStorage.getItem('jwt_token'));
  // ...
};

// v1.0+ - New authentication hook
const useAuth = () => {
  const [token, setToken] = useState(localStorage.getItem('auth_token'));
  const [refreshToken, setRefreshToken] = useState(localStorage.getItem('refresh_token'));
  // ...
};
```

### State Management Updates

```typescript
// Update Zustand store structure
interface AuthState {
  token: string | null;
  refreshToken: string | null;  // New in v1.0
  user: User | null;
  isAuthenticated: boolean;
  login: (credentials: LoginCredentials) => Promise<void>;
  logout: () => void;
  refreshAuth: () => Promise<void>;  // New in v1.0
}
```

## Testing Migration

### Test Suite Updates

```bash
# v0.x test commands
cargo test
npm test

# v1.0+ test commands (more comprehensive)
cargo test --all
cargo test -- --ignored  # Integration tests
cd frontend && npm test -- --coverage
./scripts/test-federation.sh
```

### New Test Categories

- **Federation Tests**: GraphQL schema composition testing
- **Security Tests**: Enhanced authentication and authorization testing
- **Performance Tests**: Load testing for the new architecture
- **End-to-End Tests**: Complete user journey testing

## Monitoring & Observability

### Metrics Migration

```yaml
# Old metrics (v0.x)
- workflow_executions_total
- api_requests_total
- database_connections

# New metrics (v1.0+)
- workflow_executions_total
- api_requests_total
- database_connections
- federation_requests_total      # New
- jwt_validations_total         # New
- mcp_connections_active        # New
- event_store_events_total      # New
```

### Dashboard Updates

Grafana dashboards need updates for:
- Federation gateway metrics
- Enhanced security metrics
- Event sourcing metrics
- Multi-service monitoring

## Troubleshooting

### Common Migration Issues

#### 1. JWT Secret Not Set
```
Error: Environment variable JWT_SECRET is required
Solution: Export JWT_SECRET environment variable
```

#### 2. Database Connection Issues
```
Error: Connection refused to PostgreSQL
Solution: Check DATABASE_URL format and database availability
```

#### 3. Federation Gateway Not Responding
```
Error: GraphQL gateway unreachable
Solution: Ensure graphql-gateway service is running on port 4000
```

#### 4. Frontend Authentication Failures
```
Error: Token validation failed
Solution: Clear browser storage and re-authenticate
```

### Migration Validation

```bash
# Health check script
#!/bin/bash
set -e

echo "Checking service health..."
curl -f http://localhost:8080/health/detailed
curl -f http://localhost:4000/health

echo "Running basic tests..."
cargo test --bin workflow-engine basic_integration_test

echo "Checking frontend..."
cd frontend && npm run build

echo "Migration validation complete!"
```

## Rollback Procedures

### Emergency Rollback

If migration fails in production:

1. **Stop Services**
   ```bash
   docker-compose down
   ```

2. **Restore Database**
   ```bash
   dropdb ai_workflow_db
   createdb ai_workflow_db
   psql ai_workflow_db < backup_pre_migration.sql
   ```

3. **Deploy Previous Version**
   ```bash
   git checkout v0.x
   docker-compose up -d
   ```

4. **Verify Rollback**
   ```bash
   curl http://localhost:8080/health
   ```

### Planned Rollback

For planned rollbacks:

1. **Prepare rollback plan** before migration
2. **Test rollback procedure** in staging
3. **Document rollback steps** for team
4. **Monitor system** after rollback

## Version-Specific Notes

### v1.0.0 (GraphQL Federation Release)
- **Major**: GraphQL Federation implementation
- **Breaking**: JWT configuration required
- **New**: Event sourcing with PostgreSQL
- **Enhanced**: Security hardening (70+ vulnerabilities fixed)

### v1.1.0 (Performance & Monitoring)
- **Major**: Enhanced monitoring and observability
- **New**: Prometheus metrics and Grafana dashboards
- **Improved**: WebSocket performance and connection pooling
- **Added**: Advanced rate limiting and circuit breakers

### v1.2.0 (AI Integration Enhanced)
- **Major**: Multi-provider AI support (OpenAI, Anthropic, AWS Bedrock)
- **New**: Token usage budgeting and cost tracking
- **Enhanced**: MCP protocol with connection pooling
- **Added**: Template engine with AI context injection

## Support and Resources

### Migration Support
- **Documentation**: Complete migration guides in `/docs/`
- **Examples**: Sample configurations in `/examples/`
- **Scripts**: Automated migration scripts in `/scripts/`
- **Community**: GitHub Discussions for migration questions

### Emergency Contacts
- **Technical Support**: support@workflow-engine.dev
- **Migration Issues**: migration-help@workflow-engine.dev
- **Security Issues**: security@workflow-engine.dev

---

**Last Updated**: December 2024  
**Version**: 1.0.0  
**Next Review**: With each major release