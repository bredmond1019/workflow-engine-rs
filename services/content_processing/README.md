# Content Processing Service

A high-performance microservice for intelligent document analysis and content extraction, featuring a WASM plugin system for extensible processing capabilities.

## Overview

The Content Processing Service provides advanced document analysis capabilities including:

- **Multi-format Support**: Process HTML, PDF, Markdown, Video, Code, Plain Text, JSON, and XML
- **Intelligent Analysis**: Extract concepts, keywords, entities, and generate summaries
- **Quality Assessment**: Evaluate content quality, readability, and completeness
- **Difficulty Analysis**: Assess content complexity and target audience
- **WASM Plugins**: Extend functionality with custom WebAssembly processors
- **Vector Embeddings**: Generate and store semantic embeddings for similarity search
- **Batch Processing**: Process multiple documents concurrently with job queuing
- **Real-time Caching**: Redis-backed caching for improved performance

## Quick Start

### Using Docker

```bash
# Build the image
docker build -f services/content_processing/Dockerfile -t content-processing:latest .

# Run with docker-compose
docker-compose up -d content-processing

# Or run standalone
docker run -d \
  -p 8082:8082 \
  -e DATABASE_URL="postgresql://user:pass@localhost/content_db" \
  -e REDIS_URL="redis://localhost:6379" \
  content-processing:latest
```

### Local Development

```bash
# Install dependencies
cd services/content_processing
cargo build

# Set up database
createdb content_processing_db
sqlx migrate run

# Run the service
DATABASE_URL="postgresql://localhost/content_processing_db" \
REDIS_URL="redis://localhost:6379" \
cargo run
```

## Features

### 🔍 Content Analysis
- **Concept Extraction**: Identify key concepts with confidence scores
- **Entity Recognition**: Extract people, organizations, locations, dates
- **Keyword Extraction**: Identify important terms and phrases
- **Language Detection**: Automatic language identification

### 📊 Quality Metrics
- **Readability Scoring**: Assess text complexity and clarity
- **Completeness Analysis**: Evaluate content thoroughness
- **Grammar & Structure**: Identify quality issues
- **Coherence Assessment**: Analyze logical flow

### 🎯 Learning Analysis
- **Difficulty Levels**: Classify content as Beginner/Intermediate/Advanced/Expert
- **Learning Objectives**: Extract educational goals
- **Prerequisite Knowledge**: Identify required background
- **Cognitive Load**: Estimate mental effort required

### 🔌 Plugin System
- **WASM Runtime**: Secure sandboxed plugin execution
- **Custom Processors**: Extend analysis capabilities
- **Hot Reloading**: Update plugins without service restart
- **Performance Isolation**: Plugins run in isolated environments

## Technology Stack

- **Language**: Rust 1.75+
- **Web Framework**: Actix-web 4.11
- **Database**: PostgreSQL with pgvector extension
- **Cache**: Redis
- **Plugin Runtime**: Wasmtime
- **Async Runtime**: Tokio
- **Observability**: OpenTelemetry + Prometheus

## API Endpoints

### POST /analyze
Analyze content and extract insights.

```bash
curl -X POST http://localhost:8082/analyze \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Your document content here",
    "content_type": "PlainText",
    "options": {
      "extract_concepts": true,
      "generate_summary": true,
      "analyze_difficulty": true
    }
  }'
```

### GET /health
Check service health status.

```bash
curl http://localhost:8082/health
```

### GET /metrics
Get Prometheus metrics.

```bash
curl http://localhost:8082/metrics
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | Required |
| `REDIS_URL` | Redis connection string | `redis://localhost:6379` |
| `PORT` | Service port | `8082` |
| `RUST_LOG` | Log level | `info` |
| `MAX_CONTENT_SIZE` | Maximum content size (bytes) | `10485760` (10MB) |
| `WORKER_THREADS` | Number of worker threads | CPU count |
| `PLUGIN_DIR` | WASM plugin directory | `/app/plugins` |

## Development Setup

### Prerequisites

- Rust 1.75+
- PostgreSQL 14+ with pgvector extension
- Redis 7+
- Docker & Docker Compose (optional)

### Building from Source

```bash
# Clone the repository
git clone <repository-url>
cd services/content_processing

# Install dependencies
cargo build

# Run tests
cargo test

# Run with hot-reloading
cargo watch -x run
```

### Database Setup

```bash
# Create database
createdb content_processing_db

# Install pgvector extension
psql content_processing_db -c "CREATE EXTENSION IF NOT EXISTS vector;"

# Run migrations
sqlx migrate run
```

## Performance

- **Throughput**: ~1000 documents/second (varies by content size)
- **Latency**: p50 < 100ms, p99 < 500ms
- **Memory**: ~256MB baseline, scales with content
- **Concurrent Jobs**: Configurable, default 100

## License

See LICENSE file in the repository root.