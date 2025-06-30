# Deployment Documentation

This directory contains deployment and operations documentation for the AI Workflow Engine.

## Deployment Options

### Local Development
- [Development Setup](../development/DEVELOPMENT_SETUP.md) - Local development environment
- [Docker Compose](../../docker-compose.yml) - Local containerized deployment

### Production Deployment
- [Monitoring Setup](MONITORING.md) - Production monitoring with Prometheus and Grafana
- [Scripts Documentation](SCRIPTS.md) - Deployment and utility scripts

## Quick Deployment

### Docker Compose (Recommended)
```bash
# Clone repository
git clone <repository-url>
cd workflow-engine-rs

# Start all services
docker-compose up -d

# View logs
docker-compose logs -f ai-workflow-system

# Access services
# Main API: http://localhost:8080
# Swagger UI: http://localhost:8080/swagger-ui/
# Grafana: http://localhost:3000 (admin/admin)
# Prometheus: http://localhost:9090
```

### Native Deployment
```bash
# Build release version
cargo build --release

# Set up database
createdb ai_workflow_db
psql ai_workflow_db < scripts/init-db.sql

# Set environment variables
export DATABASE_URL="postgresql://username:password@localhost/ai_workflow_db"
export JWT_SECRET="your-secure-jwt-secret-key"

# Run the application
./target/release/workflow-engine-app
```

## Environment Configuration

### Required Environment Variables
```bash
# Database
DATABASE_URL=postgresql://username:password@localhost/ai_workflow_db

# Security
JWT_SECRET=your-secure-jwt-secret-key

# Optional AI Provider Keys
OPENAI_API_KEY=your_openai_key
ANTHROPIC_API_KEY=your_anthropic_key
```

### Optional Configuration
```bash
# Logging
RUST_LOG=info
LOG_LEVEL=info

# Server Configuration
HOST=0.0.0.0
PORT=8080

# Redis (for caching)
REDIS_URL=redis://localhost:6379

# Service URLs (for microservice deployment)
CONTENT_PROCESSING_URL=http://localhost:8081
KNOWLEDGE_GRAPH_URL=http://localhost:8082
REALTIME_COMMUNICATION_URL=http://localhost:8083
```

## Service Architecture

### Core Services
1. **Main API Server** (port 8080) - Primary HTTP API
2. **Content Processing** (port 8081) - Document analysis
3. **Knowledge Graph** (port 8082) - Graph database service
4. **Realtime Communication** (port 8083) - WebSocket messaging

### Supporting Services
1. **PostgreSQL** (port 5432) - Primary database
2. **Dgraph** (port 8080/9080) - Graph database
3. **Redis** (port 6379) - Caching layer
4. **Prometheus** (port 9090) - Metrics collection
5. **Grafana** (port 3000) - Monitoring dashboards

### MCP Servers
Python-based Model Context Protocol servers:
1. **HelpScout MCP** (port 8001) - Customer support integration
2. **Notion MCP** (port 8002) - Knowledge base integration
3. **Slack MCP** (port 8003) - Team communication

## Deployment Strategies

### Single-Node Deployment
- All services on one machine
- Suitable for development and small deployments
- Use Docker Compose for easy management

### Microservice Deployment
- Services deployed independently
- Better scalability and isolation
- Requires service discovery and load balancing

### Kubernetes Deployment
- Container orchestration with Kubernetes
- Automatic scaling and health management
- Production-ready high availability

## Health Checks and Monitoring

### Health Endpoints
```bash
# Basic health check
curl http://localhost:8080/health

# Detailed health status
curl http://localhost:8080/health/detailed

# Service-specific health checks
curl http://localhost:8081/health  # Content Processing
curl http://localhost:8082/health  # Knowledge Graph
curl http://localhost:8083/health  # Realtime Communication
```

### Monitoring Stack
- **Prometheus**: Metrics collection
- **Grafana**: Visualization and alerting
- **Loki**: Log aggregation
- **AlertManager**: Alert routing and management

### Key Metrics to Monitor
- Request throughput and latency
- Database connection pool usage
- Memory and CPU utilization
- Error rates and types
- MCP connection health
- Token usage and costs

## Database Management

### PostgreSQL Setup
```bash
# Create database
createdb ai_workflow_db

# Initialize schema
psql ai_workflow_db < scripts/init-db.sql

# Run migrations (if using Diesel CLI)
diesel setup
diesel migration run
```

### Dgraph Setup (for Knowledge Graph)
```bash
cd services/knowledge_graph/dgraph
docker-compose up -d
```

### Backup and Recovery
```bash
# PostgreSQL backup
pg_dump ai_workflow_db > backup.sql

# PostgreSQL restore
psql ai_workflow_db < backup.sql

# Dgraph backup (consult Dgraph documentation)
```

## Security Considerations

### Network Security
- Use HTTPS in production
- Configure firewall rules
- Implement VPN for internal services
- Use secure database connections

### Authentication
- Generate strong JWT secrets
- Rotate secrets regularly
- Use environment variables for secrets
- Implement proper access controls

### Data Protection
- Encrypt sensitive data at rest
- Use parameterized queries
- Validate all user inputs
- Implement rate limiting

## Scaling and Performance

### Horizontal Scaling
- Deploy multiple instances behind load balancer
- Use Redis for session sharing
- Implement database read replicas
- Scale MCP connections appropriately

### Performance Optimization
- Monitor database query performance
- Implement caching strategies
- Optimize connection pools
- Use CDN for static assets

### Capacity Planning
- Monitor resource utilization
- Plan for peak load scenarios
- Implement auto-scaling policies
- Regular performance testing

## Troubleshooting

### Common Issues
1. **Database Connection Issues**
   - Check DATABASE_URL environment variable
   - Verify PostgreSQL is running
   - Check network connectivity

2. **MCP Server Connection Issues**
   - Ensure MCP servers are running
   - Check server logs for errors
   - Verify network connectivity

3. **Performance Issues**
   - Check database query performance
   - Monitor connection pool usage
   - Review application logs

4. **Authentication Issues**
   - Verify JWT_SECRET is set correctly
   - Check token expiration
   - Review authentication logs

### Debugging Tools
```bash
# View application logs
docker-compose logs -f ai-workflow-system

# Check service status
curl http://localhost:8080/health/detailed

# Monitor metrics
# Access Prometheus at http://localhost:9090
# Access Grafana at http://localhost:3000

# Database debugging
psql ai_workflow_db
```

### Log Analysis
- Check correlation IDs for request tracing
- Monitor error patterns
- Review performance metrics
- Analyze authentication failures

## Backup and Disaster Recovery

### Regular Backups
- Automated database backups
- Configuration backups
- Application state snapshots
- Log retention policies

### Recovery Procedures
1. Database restoration
2. Service restart procedures
3. Configuration recovery
4. Data consistency checks

For specific deployment scenarios and advanced configurations, refer to the individual service documentation and monitoring guides.