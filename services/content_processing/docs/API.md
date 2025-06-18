# Content Processing API Reference

## Base URL

```
http://localhost:8082
```

## Authentication

Currently, the API does not require authentication. In production, integrate with your authentication service.

## Endpoints

### POST /analyze

Analyze content and extract insights based on specified options.

#### Request

```http
POST /analyze
Content-Type: application/json
```

##### Body Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `content` | string | Yes | The content to analyze |
| `content_type` | string | Yes | Type of content: `Html`, `Pdf`, `Markdown`, `Video`, `Code`, `PlainText`, `Json`, `Xml` |
| `options` | object | Yes | Processing options (see below) |

##### Processing Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `extract_concepts` | boolean | true | Extract key concepts from content |
| `assess_quality` | boolean | true | Assess content quality metrics |
| `analyze_difficulty` | boolean | true | Analyze difficulty level |
| `extract_objectives` | boolean | true | Extract learning objectives |
| `generate_summary` | boolean | true | Generate content summary |
| `extract_keywords` | boolean | true | Extract keywords and entities |
| `detect_language` | boolean | true | Detect content language |
| `plugins` | array | [] | List of plugin names to apply |
| `timeout_seconds` | number | 30 | Maximum processing time |
| `plugin_params` | object | {} | Custom parameters for plugins |
| `verbose_logging` | boolean | false | Enable detailed logging |

#### Response

##### Success Response (200 OK)

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "content_metadata": {
    "id": "660e8400-e29b-41d4-a716-446655440000",
    "content_type": "PlainText",
    "size_bytes": 1024,
    "title": "Sample Document",
    "language": "en",
    "created_at": "2023-12-09T10:30:00Z"
  },
  "concepts": [
    {
      "id": "770e8400-e29b-41d4-a716-446655440000",
      "name": "Machine Learning",
      "description": "Branch of AI focusing on learning from data",
      "confidence": 0.95,
      "category": "Technical",
      "importance_score": 0.88
    }
  ],
  "quality_metrics": {
    "overall_score": 0.85,
    "readability_score": 0.78,
    "completeness_score": 0.90,
    "accuracy_score": 0.88,
    "coherence_score": 0.82,
    "grammar_score": 0.95,
    "vocabulary_richness": 0.73,
    "structure_quality": 0.80,
    "issues": []
  },
  "difficulty_analysis": {
    "overall_level": "Intermediate",
    "vocabulary_complexity": 0.65,
    "concept_density": 0.70,
    "sentence_complexity": 0.60,
    "prerequisite_knowledge": ["Basic programming", "Statistics"],
    "estimated_reading_time": 10,
    "cognitive_load_score": 0.68
  },
  "keywords": ["machine learning", "algorithms", "data science"],
  "entities": [
    {
      "name": "Python",
      "entity_type": "Technology",
      "confidence": 0.92
    }
  ],
  "summary": "This document explores machine learning fundamentals...",
  "language": "en",
  "processing_time_ms": 250,
  "processed_at": "2023-12-09T10:30:00Z"
}
```

##### Error Response (400 Bad Request)

```json
{
  "error": {
    "code": "INVALID_INPUT",
    "message": "Content type 'Unknown' is not supported",
    "details": {
      "field": "content_type",
      "accepted_values": ["Html", "Pdf", "Markdown", "Video", "Code", "PlainText", "Json", "Xml"]
    }
  }
}
```

##### Error Response (500 Internal Server Error)

```json
{
  "error": {
    "code": "PROCESSING_ERROR",
    "message": "Failed to process content",
    "details": {
      "reason": "Plugin 'custom_analyzer' failed to load"
    }
  }
}
```

#### Example Requests

##### Basic Text Analysis

```bash
curl -X POST http://localhost:8082/analyze \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Artificial Intelligence (AI) is revolutionizing how we interact with technology...",
    "content_type": "PlainText",
    "options": {
      "extract_concepts": true,
      "generate_summary": true,
      "analyze_difficulty": true,
      "assess_quality": true,
      "extract_keywords": true,
      "detect_language": true
    }
  }'
```

##### HTML Content with Custom Plugins

```bash
curl -X POST http://localhost:8082/analyze \
  -H "Content-Type: application/json" \
  -d '{
    "content": "<html><body><h1>Machine Learning Guide</h1><p>Learn ML basics...</p></body></html>",
    "content_type": "Html",
    "options": {
      "extract_concepts": true,
      "generate_summary": true,
      "plugins": ["sentiment_analyzer", "readability_enhancer"],
      "plugin_params": {
        "sentiment_analyzer": {
          "model": "bert-base",
          "threshold": 0.7
        }
      },
      "timeout_seconds": 60
    }
  }'
```

##### Markdown Analysis with Minimal Options

```bash
curl -X POST http://localhost:8082/analyze \
  -H "Content-Type: application/json" \
  -d '{
    "content": "# Introduction to Python\n\n## Getting Started\n\nPython is a versatile programming language...",
    "content_type": "Markdown",
    "options": {
      "extract_concepts": false,
      "assess_quality": false,
      "analyze_difficulty": true,
      "extract_objectives": false,
      "generate_summary": true,
      "extract_keywords": true,
      "detect_language": false
    }
  }'
```

### GET /health

Check the health status of the service.

#### Request

```http
GET /health
```

#### Response

##### Healthy Response (200 OK)

```json
{
  "status": "healthy",
  "timestamp": "2023-12-09T10:30:00Z",
  "version": "0.1.0",
  "checks": {
    "database": {
      "status": "up",
      "latency_ms": 2
    },
    "redis": {
      "status": "up",
      "latency_ms": 1
    },
    "plugins": {
      "status": "up",
      "loaded_count": 5
    }
  }
}
```

##### Unhealthy Response (503 Service Unavailable)

```json
{
  "status": "unhealthy",
  "timestamp": "2023-12-09T10:30:00Z",
  "version": "0.1.0",
  "checks": {
    "database": {
      "status": "down",
      "error": "Connection timeout"
    },
    "redis": {
      "status": "up",
      "latency_ms": 1
    },
    "plugins": {
      "status": "up",
      "loaded_count": 5
    }
  }
}
```

#### Example Request

```bash
curl http://localhost:8082/health
```

### GET /metrics

Get Prometheus metrics for monitoring.

#### Request

```http
GET /metrics
```

#### Response

```text
# HELP content_processing_requests_total Total number of processing requests
# TYPE content_processing_requests_total counter
content_processing_requests_total{status="success"} 1523
content_processing_requests_total{status="error"} 42

# HELP content_processing_duration_seconds Processing time in seconds
# TYPE content_processing_duration_seconds histogram
content_processing_duration_seconds_bucket{le="0.1"} 892
content_processing_duration_seconds_bucket{le="0.5"} 1420
content_processing_duration_seconds_bucket{le="1.0"} 1500
content_processing_duration_seconds_bucket{le="5.0"} 1523
content_processing_duration_seconds_bucket{le="+Inf"} 1523
content_processing_duration_seconds_sum 482.5
content_processing_duration_seconds_count 1523

# HELP cache_hit_rate Cache hit rate percentage
# TYPE cache_hit_rate gauge
cache_hit_rate 0.72

# HELP active_processing_jobs Number of currently processing jobs
# TYPE active_processing_jobs gauge
active_processing_jobs 5
```

#### Example Request

```bash
curl http://localhost:8082/metrics
```

## Error Codes

| Code | Description | HTTP Status |
|------|-------------|-------------|
| `INVALID_INPUT` | Invalid request parameters | 400 |
| `UNSUPPORTED_FORMAT` | Content type not supported | 400 |
| `CONTENT_TOO_LARGE` | Content exceeds size limit | 413 |
| `TIMEOUT_ERROR` | Processing timeout exceeded | 408 |
| `PLUGIN_ERROR` | Plugin execution failed | 500 |
| `INTERNAL_ERROR` | Internal server error | 500 |
| `SERVICE_UNAVAILABLE` | Service temporarily unavailable | 503 |

## Rate Limiting

The API implements rate limiting to prevent abuse:

- **Default Limit**: 100 requests per minute per IP
- **Burst Limit**: 10 requests per second
- **Headers**: Rate limit information included in response headers

```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1702116600
```

## Content Size Limits

- **Maximum Content Size**: 10MB (configurable)
- **Maximum Processing Time**: 30 seconds (configurable per request)
- **Maximum Plugin Execution Time**: 10 seconds per plugin

## Best Practices

### 1. Use Appropriate Content Types

Always specify the correct content type for optimal processing:

```json
{
  "content_type": "Markdown"  // Not "PlainText" for markdown content
}
```

### 2. Enable Only Needed Options

Disable unnecessary analysis options for better performance:

```json
{
  "options": {
    "extract_concepts": true,
    "assess_quality": false,  // Disable if not needed
    "analyze_difficulty": false,
    "generate_summary": true
  }
}
```

### 3. Handle Timeouts Gracefully

Set appropriate timeouts and handle timeout errors:

```bash
curl -X POST http://localhost:8082/analyze \
  --max-time 35 \
  -d '{
    "options": {
      "timeout_seconds": 30
    }
  }'
```

### 4. Use Caching Headers

The API returns cache headers for GET requests:

```http
Cache-Control: public, max-age=300
ETag: "686897696a7c876b7e"
```

### 5. Monitor Rate Limits

Check rate limit headers and implement backoff:

```python
if response.headers.get('X-RateLimit-Remaining') == '0':
    reset_time = int(response.headers.get('X-RateLimit-Reset'))
    wait_time = reset_time - time.time()
    time.sleep(wait_time)
```

## Webhooks (Coming Soon)

Future versions will support webhooks for async processing:

```json
{
  "options": {
    "webhook_url": "https://your-app.com/webhook",
    "webhook_events": ["processing.completed", "processing.failed"]
  }
}
```

## SDK Support

Official SDKs are planned for:

- Python
- JavaScript/TypeScript
- Go
- Java

Example (Python SDK preview):

```python
from content_processing import Client

client = Client(base_url="http://localhost:8082")

result = client.analyze(
    content="Your content here",
    content_type="PlainText",
    extract_concepts=True,
    generate_summary=True
)

print(f"Summary: {result.summary}")
print(f"Concepts: {[c.name for c in result.concepts]}")
```