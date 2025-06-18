# DGraph Configuration for Knowledge Graph Engine

This directory contains the DGraph database configuration for the Knowledge Graph Query Engine.

## Quick Start

1. Copy the environment configuration:
   ```bash
   cp .env.example .env
   ```

2. Start DGraph using Docker Compose:
   ```bash
   docker-compose up -d
   ```

3. The schema will be automatically loaded by the schema-loader container.

4. Access DGraph interfaces:
   - GraphQL endpoint: http://localhost:8080/graphql
   - Admin endpoint: http://localhost:8080/admin
   - Ratel UI: http://localhost:8000
   - Metrics: http://localhost:8080/debug/prometheus_metrics

## Schema Overview

The GraphQL schema defines the following main types:

### Concept
- Core knowledge unit with relationships to other concepts
- Supports prerequisite tracking and topic hierarchies
- Includes vector embeddings for similarity search
- Tracks difficulty, category, and quality metrics

### LearningResource
- Educational content associated with concepts
- Tracks resource type, format, quality, and duration
- Links to one or more concepts

### LearningPath
- Structured sequences of concepts for learning
- Supports custom and system-generated paths
- Tracks estimated time and difficulty progression

### UserProgress
- Tracks user progress through concepts
- Records completion status, time spent, and difficulty ratings

## DGraph Features Used

1. **Vector Similarity Search**: Using HNSW index for concept embeddings
2. **Full-text Search**: On names, descriptions, and content
3. **Faceted Search**: On categories, difficulty levels, and tags
4. **Graph Traversal**: For prerequisites and learning path generation
5. **Transactions**: ACID compliance for data consistency

## Common Operations

### Add a new concept
```graphql
mutation {
  addConcept(input: {
    name: "Machine Learning"
    description: "Introduction to ML algorithms"
    difficulty: "intermediate"
    category: "AI"
    embedding: [0.1, 0.2, ...] # 1536 dimensions
  }) {
    concept {
      id
      name
    }
  }
}
```

### Find prerequisites
```graphql
query {
  getConcept(id: "0x123") {
    name
    prerequisites {
      name
      difficulty
    }
  }
}
```

### Vector similarity search
```graphql
query {
  queryConcept(
    filter: {
      embedding: {
        near: {
          vector: [0.1, 0.2, ...]
          distance: 0.8
        }
      }
    }
    first: 10
  ) {
    name
    description
    difficulty
  }
}
```

## Monitoring

- Health check: `curl http://localhost:8080/health`
- Metrics: `curl http://localhost:8080/debug/prometheus_metrics`
- State: `curl http://localhost:6080/state`

## Production Considerations

1. **Clustering**: Use multiple Alpha nodes for high availability
2. **Security**: Enable ACL and encryption
3. **Backup**: Regular exports using `dgraph export`
4. **Monitoring**: Integrate with Prometheus/Grafana
5. **Tuning**: Adjust cache size and connection limits based on load