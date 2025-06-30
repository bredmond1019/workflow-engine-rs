# API Documentation

## Overview

The AI Workflow Engine provides multiple API interfaces for different use cases:

## REST API

- **Base URL**: `http://localhost:8080`
- **Swagger UI**: `http://localhost:8080/swagger-ui/`
- **OpenAPI Spec**: Available at `/openapi.json`

### Core Endpoints

- `GET /health` - Health check endpoint
- `GET /health/detailed` - Detailed health status
- `POST /api/workflows` - Create workflow
- `GET /api/workflows` - List workflows
- `POST /api/workflows/{id}/execute` - Execute workflow

## GraphQL API

- **Federation Gateway**: `http://localhost:4000/graphql`
- **Playground**: `http://localhost:4000/playground`
- **Schema**: Available via introspection

### Core Queries

```graphql
query GetWorkflows {
  workflows {
    id
    name
    status
    nodes {
      id
      type
      configuration
    }
  }
}

mutation CreateWorkflow($input: CreateWorkflowInput!) {
  createWorkflow(input: $input) {
    id
    name
    status
  }
}
```

## Authentication

All APIs use JWT Bearer token authentication:

```bash
curl -H "Authorization: Bearer <jwt-token>" \
     http://localhost:8080/api/workflows
```

## Rate Limiting

- **Global**: 1000 requests/minute
- **Per-user**: 100 requests/minute
- **Headers**: Rate limit status in response headers

## Error Handling

Standard HTTP status codes with detailed error responses:

```json
{
  "error": "ValidationError",
  "message": "Invalid workflow configuration",
  "details": {
    "field": "nodes[0].type",
    "reason": "Unknown node type"
  }
}
```

For detailed API documentation, see:
- [REST API Reference](rest-api.md)
- [GraphQL Schema](graphql-schema.md)
- [Authentication Guide](authentication.md)