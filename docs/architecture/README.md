# Architecture Documentation

This directory contains comprehensive architectural documentation for the AI Workflow Engine.

## System Overview

The AI Workflow Engine is a production-ready microservice-based system built in Rust with Python MCP servers. It provides a foundation for building AI-powered applications with external service integrations.

## Core Architecture Documents

### System Design
- [Performance Architecture](performance.md) - Performance considerations and optimization strategies
- [Pricing Engine Implementation](PRICING_ENGINE_IMPLEMENTATION.md) - Token usage tracking and pricing
- [GraphQL Federation](GRAPHQL_FEDERATION.md) - GraphQL federation setup and configuration

### Service Architecture
Each service has detailed architectural documentation:

#### Core Services
- [Content Processing Service](services/content_processing/) - Document analysis and processing
- [Knowledge Graph Service](services/knowledge_graph/) - Graph database integration with Dgraph
- [Realtime Communication Service](services/realtime_communication/) - WebSocket-based messaging

#### Service Documentation Structure
Each service includes:
- **ARCHITECTURE.md** - Service-specific architectural overview
- **API.md** - API endpoints and contracts
- **CONFIGURATION.md** - Configuration options and environment variables
- **DEPLOYMENT.md** - Deployment strategies and requirements
- **TROUBLESHOOTING.md** - Common issues and debugging

## Component Architecture

### Core Crates
The system is built with a modular crate structure:

1. **workflow-engine-api** - Main HTTP API server
   - Actix-web based REST API
   - Authentication and authorization
   - Rate limiting and middleware
   - OpenAPI documentation

2. **workflow-engine-core** - Core workflow engine logic
   - Workflow execution engine
   - AI integration layer
   - Error handling and recovery
   - Event sourcing

3. **workflow-engine-mcp** - Model Context Protocol implementation
   - Multi-transport support (HTTP, WebSocket, stdio)
   - Connection pooling and load balancing
   - Protocol abstraction layer

4. **workflow-engine-nodes** - Built-in workflow nodes
   - AI agent nodes (OpenAI, Anthropic)
   - External MCP client nodes
   - Template processing nodes

5. **workflow-engine-app** - Main application binary
   - Service bootstrap and configuration
   - Dependency injection container
   - Application lifecycle management

### Data Flow Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Client Apps   │───▶│  API Gateway    │───▶│  Core Engine    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │                        │
                                ▼                        ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Monitoring    │◀───│  Event Store    │◀───│  Node Registry  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │                        │
                                ▼                        ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Knowledge      │    │   Content       │    │   Realtime      │
│  Graph          │    │   Processing    │    │   Communication │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Integration Patterns

#### Model Context Protocol (MCP)
- **Transport Layer**: HTTP, WebSocket, and stdio support
- **Connection Management**: Pool-based connection management with health checks
- **Load Balancing**: Round-robin and weighted load balancing
- **Error Handling**: Circuit breakers and retry logic

#### Event Sourcing
- **Event Store**: PostgreSQL-based event storage
- **CQRS**: Command Query Responsibility Segregation
- **Event Replay**: Snapshot and replay capabilities
- **Event Versioning**: Schema evolution support

#### Microservice Communication
- **Service Discovery**: Bootstrap-based service registration
- **Health Checks**: Detailed health monitoring endpoints
- **Circuit Breakers**: Protection against cascading failures
- **Rate Limiting**: Request throttling and quota management

## Security Architecture

### Authentication & Authorization
- **JWT Tokens**: Stateless authentication
- **Role-Based Access**: Hierarchical permission system
- **API Keys**: Service-to-service authentication
- **Rate Limiting**: DDoS protection and quota enforcement

### Data Protection
- **Encryption**: TLS for transport, encryption at rest
- **Input Validation**: Comprehensive request validation
- **SQL Injection Protection**: Parameterized queries
- **CORS**: Cross-origin request security

## Scalability Patterns

### Horizontal Scaling
- **Stateless Services**: Session-free service design
- **Load Balancing**: Multi-instance deployment support
- **Database Sharding**: Tenant-based data partitioning
- **Caching Strategy**: Redis-based caching layer

### Performance Optimization
- **Connection Pooling**: Database and MCP connection pools
- **Async Processing**: Tokio-based async runtime
- **Memory Management**: Zero-copy optimizations
- **Monitoring**: Prometheus metrics and Grafana dashboards

## Technology Stack

### Core Technologies
- **Language**: Rust (primary), Python (MCP servers)
- **Web Framework**: Actix-web
- **Database**: PostgreSQL (primary), Dgraph (graph data)
- **Message Queue**: Built-in event system
- **Caching**: Redis
- **Monitoring**: Prometheus + Grafana

### External Integrations
- **AI Providers**: OpenAI, Anthropic
- **External Services**: HelpScout, Notion, Slack (via MCP)
- **Protocols**: HTTP, WebSocket, GraphQL
- **Standards**: OpenAPI, JSON Schema

## Development Patterns

### Code Organization
- **Monorepo Structure**: Multiple crates in single repository
- **Dependency Injection**: Bootstrap-based DI container
- **Error Handling**: Comprehensive error types and recovery
- **Testing Strategy**: Unit, integration, and end-to-end tests

### Quality Assurance
- **Static Analysis**: Clippy linting
- **Security Scanning**: Cargo audit
- **Code Coverage**: Test coverage reporting
- **Performance Testing**: Load and chaos testing

For detailed implementation guidance, refer to the individual component CLAUDE.md files and service-specific documentation.