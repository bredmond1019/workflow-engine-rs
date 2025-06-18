# Knowledge Graph API Reference

## Overview

The Knowledge Graph Service provides both GraphQL and REST APIs for interacting with the concept graph. GraphQL is the primary interface for complex queries, while REST endpoints provide simplified access to common operations.

## Base URL

```
Development: http://localhost:3002
Production: https://api.knowledge-graph.example.com
```

## Authentication

All API requests require authentication via JWT tokens:

```http
Authorization: Bearer <jwt-token>
```

## GraphQL API

### Endpoint

```
POST /graphql
```

### Schema Overview

```graphql
type Query {
  # Get a specific concept by ID
  getConcept(id: ID!): Concept
  
  # Search concepts with filters
  searchConcepts(
    query: String
    category: String
    difficulty: String
    tags: [String]
    limit: Int = 20
    offset: Int = 0
  ): ConceptSearchResult!
  
  # Find prerequisites for a concept
  findPrerequisites(
    conceptId: ID!
    maxDepth: Int = 3
  ): [Concept!]!
  
  # Generate learning path between concepts
  findLearningPath(
    fromConceptId: ID!
    toConceptId: ID!
    constraints: PathConstraints
  ): LearningPath
  
  # Get related concepts
  getRelatedConcepts(
    conceptId: ID!
    relationshipType: RelationType
    limit: Int = 10
  ): [RelatedConcept!]!
  
  # Find similar concepts using embeddings
  findSimilarConcepts(
    conceptId: ID!
    threshold: Float = 0.8
    limit: Int = 10
  ): [SimilarConcept!]!
  
  # Get user progress
  getUserProgress(
    userId: String!
    conceptIds: [ID!]
  ): [UserProgress!]!
}

type Mutation {
  # Create a new concept
  createConcept(input: CreateConceptInput!): Concept!
  
  # Update an existing concept
  updateConcept(
    id: ID!
    input: UpdateConceptInput!
  ): Concept!
  
  # Add prerequisite relationship
  addPrerequisite(
    conceptId: ID!
    prerequisiteId: ID!
  ): Concept!
  
  # Create custom learning path
  createLearningPath(
    input: CreateLearningPathInput!
  ): LearningPath!
  
  # Update user progress
  updateUserProgress(
    input: UpdateUserProgressInput!
  ): UserProgress!
  
  # Add learning resource
  addLearningResource(
    input: CreateResourceInput!
  ): LearningResource!
}

type Subscription {
  # Subscribe to concept updates
  conceptUpdated(conceptId: ID!): Concept!
  
  # Subscribe to learning path progress
  progressUpdated(userId: String!): UserProgress!
}
```

### GraphQL Examples

#### Search Concepts

```graphql
query SearchConcepts {
  searchConcepts(
    query: "machine learning"
    category: "programming"
    difficulty: "intermediate"
    limit: 10
  ) {
    total
    concepts {
      id
      name
      description
      difficulty
      category
      tags
      qualityScore
    }
  }
}
```

#### Find Learning Path

```graphql
query GenerateLearningPath {
  findLearningPath(
    fromConceptId: "123e4567-e89b-12d3-a456-426614174000"
    toConceptId: "987fcdeb-51a2-43d1-b2c3-426614174000"
    constraints: {
      maxLength: 10
      maxDifficulty: "advanced"
      preferredCategories: ["programming", "algorithms"]
    }
  ) {
    id
    name
    estimatedTime
    concepts {
      concept {
        id
        name
        difficulty
      }
      order
      isOptional
    }
  }
}
```

#### Get Concept with Prerequisites

```graphql
query GetConceptDetails {
  getConcept(id: "123e4567-e89b-12d3-a456-426614174000") {
    id
    name
    description
    difficulty
    prerequisites {
      id
      name
      difficulty
    }
    resources {
      id
      title
      url
      resourceType
      quality
    }
  }
}
```

#### Create New Concept

```graphql
mutation CreateConcept {
  createConcept(input: {
    name: "GraphQL Fundamentals"
    description: "Introduction to GraphQL query language"
    difficulty: "beginner"
    category: "programming"
    subcategory: "api"
    tags: ["graphql", "api", "query-language"]
    estimatedTime: 3.5
  }) {
    id
    name
    createdAt
  }
}
```

## REST API Endpoints

### Health Check

```http
GET /health
```

**Response:**
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "dgraph": "connected",
  "redis": "connected",
  "uptime": 3600
}
```

### Search Concepts

```http
POST /api/v1/search
Content-Type: application/json

{
  "query": "machine learning",
  "filters": {
    "category": "programming",
    "difficulty": ["beginner", "intermediate"],
    "tags": ["ai", "ml"]
  },
  "pagination": {
    "limit": 20,
    "offset": 0
  }
}
```

**Response:**
```json
{
  "total": 42,
  "concepts": [
    {
      "id": "123e4567-e89b-12d3-a456-426614174000",
      "name": "Introduction to Machine Learning",
      "description": "Basic concepts and algorithms in ML",
      "difficulty": "beginner",
      "category": "programming",
      "tags": ["ai", "ml", "algorithms"],
      "qualityScore": 0.95,
      "estimatedTime": 5.0
    }
  ],
  "facets": {
    "categories": {
      "programming": 35,
      "mathematics": 7
    },
    "difficulties": {
      "beginner": 15,
      "intermediate": 20,
      "advanced": 7
    }
  }
}
```

### Get Concept Details

```http
GET /api/v1/concept/{conceptId}
```

**Response:**
```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "name": "Neural Networks",
  "description": "Deep learning fundamentals",
  "difficulty": "intermediate",
  "category": "programming",
  "subcategory": "machine-learning",
  "tags": ["ai", "deep-learning", "neural-networks"],
  "prerequisites": [
    {
      "id": "456e7890-e89b-12d3-a456-426614174000",
      "name": "Linear Algebra",
      "difficulty": "intermediate"
    }
  ],
  "enabledConcepts": [
    {
      "id": "789e0123-e89b-12d3-a456-426614174000",
      "name": "Convolutional Neural Networks",
      "difficulty": "advanced"
    }
  ],
  "resources": [
    {
      "id": "res-123",
      "title": "Neural Networks and Deep Learning",
      "url": "https://example.com/nn-course",
      "resourceType": "course",
      "quality": 0.9,
      "duration": 240
    }
  ],
  "metadata": {
    "createdAt": "2024-01-15T10:00:00Z",
    "updatedAt": "2024-01-20T15:30:00Z",
    "version": 2
  }
}
```

### Generate Learning Path

```http
POST /api/v1/learning-path
Content-Type: application/json

{
  "fromConceptId": "123e4567-e89b-12d3-a456-426614174000",
  "toConceptId": "987fcdeb-51a2-43d1-b2c3-426614174000",
  "constraints": {
    "maxLength": 8,
    "maxDifficulty": "advanced",
    "includeOptional": true,
    "preferredCategories": ["programming"],
    "avoidCategories": ["theoretical"]
  },
  "userId": "user-123"
}
```

**Response:**
```json
{
  "path": {
    "id": "path-generated-123",
    "name": "Path to Advanced Machine Learning",
    "estimatedTime": 45.5,
    "difficultyProgression": "gradual",
    "concepts": [
      {
        "order": 1,
        "concept": {
          "id": "123e4567-e89b-12d3-a456-426614174000",
          "name": "Python Basics",
          "difficulty": "beginner",
          "estimatedTime": 5.0
        },
        "isOptional": false,
        "alternatives": []
      },
      {
        "order": 2,
        "concept": {
          "id": "234e5678-e89b-12d3-a456-426614174000",
          "name": "NumPy and Pandas",
          "difficulty": "beginner",
          "estimatedTime": 8.0
        },
        "isOptional": false,
        "alternatives": []
      }
    ],
    "metadata": {
      "algorithm": "a-star",
      "score": 0.92,
      "alternativePaths": 3
    }
  }
}
```

### Find Related Concepts

```http
GET /api/v1/related/{conceptId}?type=similar&limit=5
```

**Response:**
```json
{
  "conceptId": "123e4567-e89b-12d3-a456-426614174000",
  "relatedConcepts": [
    {
      "concept": {
        "id": "234e5678-e89b-12d3-a456-426614174000",
        "name": "Support Vector Machines",
        "category": "programming",
        "difficulty": "intermediate"
      },
      "relationshipType": "similar",
      "score": 0.89,
      "explanation": "Both are supervised learning algorithms"
    }
  ]
}
```

### Get Concept Graph

```http
POST /api/v1/graph/subgraph
Content-Type: application/json

{
  "rootConceptId": "123e4567-e89b-12d3-a456-426614174000",
  "depth": 2,
  "includeRelationships": ["prerequisites", "related"],
  "filters": {
    "difficulty": ["beginner", "intermediate"]
  }
}
```

**Response:**
```json
{
  "nodes": [
    {
      "id": "123e4567-e89b-12d3-a456-426614174000",
      "name": "Machine Learning",
      "type": "concept",
      "properties": {
        "difficulty": "intermediate",
        "category": "programming"
      }
    }
  ],
  "edges": [
    {
      "source": "123e4567-e89b-12d3-a456-426614174000",
      "target": "234e5678-e89b-12d3-a456-426614174000",
      "type": "prerequisite",
      "weight": 1.0
    }
  ],
  "metadata": {
    "totalNodes": 15,
    "totalEdges": 22,
    "depth": 2
  }
}
```

### Update User Progress

```http
PUT /api/v1/progress
Content-Type: application/json

{
  "userId": "user-123",
  "conceptId": "123e4567-e89b-12d3-a456-426614174000",
  "status": "in_progress",
  "percentComplete": 45,
  "timeSpent": 120,
  "resourcesCompleted": 2
}
```

**Response:**
```json
{
  "progress": {
    "id": "progress-123",
    "userId": "user-123",
    "conceptId": "123e4567-e89b-12d3-a456-426614174000",
    "status": "in_progress",
    "percentComplete": 45,
    "timeSpent": 120,
    "resourcesCompleted": 2,
    "lastAccessedAt": "2024-01-20T15:30:00Z"
  }
}
```

## Error Responses

### Standard Error Format

```json
{
  "error": {
    "code": "CONCEPT_NOT_FOUND",
    "message": "Concept with ID '123' not found",
    "details": {
      "conceptId": "123",
      "timestamp": "2024-01-20T15:30:00Z"
    }
  },
  "requestId": "req-abc123"
}
```

### Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `CONCEPT_NOT_FOUND` | 404 | Requested concept does not exist |
| `INVALID_INPUT` | 400 | Invalid request parameters |
| `UNAUTHORIZED` | 401 | Missing or invalid authentication |
| `FORBIDDEN` | 403 | Insufficient permissions |
| `RATE_LIMITED` | 429 | Too many requests |
| `GRAPH_ERROR` | 500 | Dgraph operation failed |
| `ALGORITHM_ERROR` | 500 | Algorithm execution failed |
| `TIMEOUT` | 504 | Request timeout |

## Rate Limiting

API requests are rate limited based on the authentication tier:

- **Anonymous**: 10 requests/minute
- **Authenticated**: 100 requests/minute
- **Premium**: 1000 requests/minute

Rate limit headers:
```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1642694400
```

## Pagination

For endpoints returning lists, use pagination parameters:

```json
{
  "pagination": {
    "limit": 20,
    "offset": 0,
    "cursor": "eyJpZCI6MTIzfQ=="
  }
}
```

Response includes pagination metadata:
```json
{
  "data": [...],
  "pagination": {
    "total": 150,
    "limit": 20,
    "offset": 0,
    "hasNext": true,
    "hasPrev": false,
    "nextCursor": "eyJpZCI6MTQzfQ=="
  }
}
```

## Webhooks

The service supports webhooks for real-time notifications:

### Webhook Events

- `concept.created` - New concept added
- `concept.updated` - Concept modified
- `path.generated` - Learning path created
- `progress.milestone` - User reached milestone

### Webhook Payload

```json
{
  "event": "concept.updated",
  "timestamp": "2024-01-20T15:30:00Z",
  "data": {
    "conceptId": "123e4567-e89b-12d3-a456-426614174000",
    "changes": ["description", "tags"],
    "userId": "user-who-made-change"
  }
}
```

## API Versioning

The API uses URL versioning:
- Current version: `/api/v1/`
- Legacy support: 6 months after new version release
- Deprecation notices in headers: `X-API-Deprecation-Date`

## SDK Support

Official SDKs available for:
- JavaScript/TypeScript
- Python
- Go
- Rust

Example (TypeScript):
```typescript
import { KnowledgeGraphClient } from '@knowledge-graph/sdk';

const client = new KnowledgeGraphClient({
  apiKey: 'your-api-key',
  baseUrl: 'https://api.knowledge-graph.example.com'
});

const concepts = await client.searchConcepts({
  query: 'machine learning',
  category: 'programming',
  limit: 10
});
```