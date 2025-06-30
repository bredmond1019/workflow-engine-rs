# CLAUDE.md - Content Processing Service

This file provides guidance to Claude Code (claude.ai/code) when working with the content processing service, a high-performance microservice for intelligent document analysis and content extraction that serves as a GraphQL Federation subgraph.

## Service Overview

The Content Processing Service is a Rust-based microservice that provides comprehensive document analysis capabilities with WASM plugin support. It's designed to process various content formats and extract meaningful insights using NLP and AI techniques. The service operates as part of the GraphQL Federation architecture, exposing its capabilities through both REST and GraphQL APIs.

### Key Capabilities

- **Multi-format Document Processing**: HTML, PDF, Markdown, Video, Code, Plain Text, JSON, XML
- **Intelligent Analysis**: Concepts, quality metrics, difficulty levels, keywords, entities, summaries
- **WASM Plugin System**: Extensible architecture for custom processing capabilities
- **Vector Embeddings**: Semantic search support with pgvector
- **Batch Processing**: Concurrent processing with job queuing
- **SQLx Database**: Type-safe database operations with PostgreSQL
- **GraphQL Federation**: Subgraph implementation with entity resolution and schema composition
- **Security Hardening**: JWT authentication, input validation, rate limiting, and sandboxed plugin execution

## Architecture Components

### 1. Core Processing Engine (`src/processor.rs`)
The main orchestrator that coordinates all analysis modules:
- Validates input content
- Routes to appropriate parsers based on content type
- Manages analysis pipeline execution
- Aggregates results from multiple analyzers

### 2. Document Parsers (`src/parsers/`)
Format-specific parsers that extract text and structure:
- **HTML Parser** (`html.rs`): Uses scraper crate for DOM parsing
- **Markdown Parser** (`markdown.rs`): Uses pulldown-cmark
- **PDF Parser** (`pdf.rs`): Uses pdf-extract for text extraction
- **XML Parser** (`xml.rs`): Uses quick-xml for parsing
- **Text Parser** (`text.rs`): Plain text processing
- **JSON Parser** (`json.rs`): Structured data extraction

### 3. Analysis Modules (`src/analysis/`)
Specialized analyzers for different aspects:
- **Concept Extraction** (`concepts.rs`): NLP-based concept identification
- **Quality Assessment** (`quality.rs`): Readability, coherence, completeness scoring
- **Difficulty Analysis** (`difficulty.rs`): Content complexity evaluation
- **Keyword Extraction** (`keywords.rs`): TF-IDF based keyword identification
- **Entity Recognition** (`entities.rs`): Named entity extraction
- **Summarization** (`summarization.rs`): Extractive text summarization
- **Language Detection** (`language.rs`): Uses whatlang crate

### 4. AI Integration (`src/ai_integration.rs`)
Interfaces with external AI services for enhanced analysis:
- OpenAI API integration for advanced NLP
- Anthropic API support for content understanding
- Configurable AI provider selection

### 5. Plugin System (WASM)
Extensible architecture for custom processing:
- Wasmtime runtime for secure plugin execution
- Sandboxed environment with resource limits
- JSON-based communication interface
- Hot-reloading capabilities

## Database Schema (SQLx)

### Primary Tables

```sql
-- Content metadata with vector embeddings
content_metadata (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    source_url TEXT,
    content_type VARCHAR(50) NOT NULL,
    format VARCHAR(50) NOT NULL,
    size_bytes BIGINT,
    hash VARCHAR(64) NOT NULL,
    quality_score FLOAT,
    difficulty_level VARCHAR(20),
    concepts JSONB DEFAULT '[]'::jsonb,
    embeddings vector(1536),  -- pgvector for similarity search
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ
)

-- Asynchronous processing jobs
processing_jobs (
    id UUID PRIMARY KEY,
    content_id UUID REFERENCES content_metadata(id),
    job_type VARCHAR(50) NOT NULL,
    status job_status NOT NULL DEFAULT 'pending',
    priority job_priority NOT NULL DEFAULT 'medium',
    options JSONB DEFAULT '{}'::jsonb,
    result JSONB,
    error_message TEXT,
    retry_count INT DEFAULT 0,
    worker_id VARCHAR(100),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ
)

-- WASM plugin registry
plugin_registry (
    id UUID PRIMARY KEY,
    name VARCHAR(255) UNIQUE NOT NULL,
    version VARCHAR(50) NOT NULL,
    description TEXT,
    wasm_module BYTEA NOT NULL,
    metadata JSONB DEFAULT '{}'::jsonb,
    enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ
)
```

### Database Migrations

Located in `migrations/` directory:
- `20241209_000001_create_content_metadata.sql`: Core content storage
- `20241209_000002_create_processing_jobs.sql`: Job queue tables
- `20241209_000003_create_plugin_registry.sql`: Plugin management

Run migrations with:
```bash
sqlx migrate run
```

## API Endpoints

### REST API

#### POST /analyze
Main content analysis endpoint:
```json
{
  "content": "Document content here",
  "content_type": "PlainText",
  "options": {
    "extract_concepts": true,
    "generate_summary": true,
    "analyze_difficulty": true,
    "plugins": ["sentiment_analyzer"]
  }
}
```

#### GET /health
Service health check with subsystem status

#### GET /metrics
Prometheus metrics for monitoring

### GraphQL API (Port 8082/graphql)

The service implements a GraphQL Federation subgraph with the following capabilities:

#### Federation Schema
- **Entities**: `ContentMetadata`, `ProcessingJob`, extended `User` and `Workflow` types
- **Key Directives**: Uses `@key` for entity identification
- **Federation Resolvers**: Implements `_service` and `_entities` for query planning

#### Key Queries
```graphql
# Get content by ID
query GetContent($id: ID!) {
  content(id: $id) {
    id
    title
    qualityScore
    difficultyLevel
    concepts {
      name
      relevance
    }
  }
}

# Search content
query SearchContent($query: String!) {
  searchContent(query: $query) {
    content {
      id
      title
      summary
    }
    totalCount
  }
}
```

#### Key Mutations
```graphql
# Analyze content
mutation AnalyzeContent($input: AnalyzeContentInput!) {
  analyzeContent(input: $input) {
    success
    jobId
    content {
      id
      qualityScore
    }
  }
}
```

## Integration with Main Workflow Engine

This service integrates with the main workflow engine and GraphQL Federation gateway:

1. **GraphQL Federation**: Primary integration through Apollo Federation Gateway (port 4000)
2. **MCP Communication**: The workflow engine can invoke this service via MCP protocol
3. **Direct HTTP**: REST API calls for synchronous processing
4. **Async Jobs**: Queue-based processing for large documents
5. **Shared Database**: Can optionally share PostgreSQL instance with proper schema isolation
6. **Entity Resolution**: Supports cross-service queries through federation `@key` directives

### Usage in Workflows

```rust
// Example workflow node that uses content processing
pub struct ContentAnalysisNode {
    service_url: String,
}

impl WorkflowNode for ContentAnalysisNode {
    async fn execute(&self, input: NodeInput) -> NodeOutput {
        // Call content processing service
        let analysis = analyze_content(&self.service_url, input.content).await?;
        // Use results in workflow
    }
}
```

## Testing Strategy

### Unit Tests
- Located alongside source files
- Test individual components in isolation
- Use mockall for external dependencies

### Integration Tests
```bash
# Run all tests
cargo test

# Integration tests only (requires database)
cargo test -- --ignored

# Specific test categories
cargo test analysis::
cargo test parsers::
cargo test api::
```

### Database Tests
- Use test database: `test_content_processing`
- Automatic migration rollback after tests
- Parallel test execution with transaction isolation

## Common Development Tasks

### 1. Adding a New Parser
Create new parser in `src/parsers/`:
```rust
// src/parsers/new_format.rs
pub struct NewFormatParser;

impl DocumentParser for NewFormatParser {
    async fn parse(&self, content: &[u8]) -> Result<ParsedContent> {
        // Implementation
    }
}
```

### 2. Creating a New Analyzer
Add analyzer in `src/analysis/`:
```rust
// src/analysis/new_analyzer.rs
pub struct NewAnalyzer;

impl TextAnalyzer for NewAnalyzer {
    async fn analyze(&self, text: &str) -> Result<AnalysisResult> {
        // Implementation
    }
}
```

### 3. Adding a WASM Plugin
See `docs/PLUGINS.md` for detailed plugin development guide:
1. Create plugin in Rust/AssemblyScript/Go
2. Compile to WASM
3. Deploy via API or file system
4. Configure in plugin registry

### 4. Database Schema Changes
1. Create new migration: `sqlx migrate add description`
2. Write SQL in generated file
3. Run migration: `sqlx migrate run`
4. Update `sqlx-data.json`: `cargo sqlx prepare`

### 5. Adding AI Provider
Extend `src/ai_integration.rs`:
```rust
pub trait AIProvider {
    async fn analyze(&self, content: &str) -> Result<AIAnalysis>;
}

pub struct NewAIProvider {
    api_key: String,
}
```

### 6. Performance Optimization
- Use connection pooling (already configured)
- Enable Redis caching for repeated analyses
- Batch similar requests
- Profile with `cargo flamegraph`

## Configuration

### Environment Variables
```bash
# Database
DATABASE_URL=postgresql://user:pass@localhost/content_processing_db

# Redis (optional)
REDIS_URL=redis://localhost:6379

# Server
PORT=8082
WORKER_THREADS=8

# AI Providers (optional)
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...

# Plugin System
PLUGIN_DIR=/app/plugins
PLUGIN_MEMORY_LIMIT=10485760  # 10MB
```

### Service Configuration
Located in `src/bootstrap/config.rs` if following main engine pattern

## Debugging Tips

### 1. Enable Verbose Logging
```bash
RUST_LOG=content_processing=debug cargo run
```

### 2. Database Queries
Enable SQLx query logging:
```bash
RUST_LOG=sqlx=debug cargo run
```

### 3. Plugin Issues
- Check plugin logs in database
- Verify WASM module validity
- Test with plugin-tester utility

### 4. Performance Issues
- Monitor `/metrics` endpoint
- Check database query performance
- Profile CPU/memory usage

## Production Deployment

### Docker
```bash
# Build
docker build -f services/content_processing/Dockerfile -t content-processing:latest .

# Run
docker run -d \
  -p 8082:8082 \
  -e DATABASE_URL="postgresql://prod_user:pass@db/content_db" \
  content-processing:latest
```

### Health Checks
- Liveness: `/health`
- Readiness: `/health` with database check

### Monitoring
- Prometheus metrics at `/metrics`
- Custom Grafana dashboards available
- Alert on high error rates or latency

## GraphQL Federation Integration

### Subgraph Configuration
The service operates as a federation subgraph with:
- **Apollo Federation v2**: Full support for `@key`, `@extends`, `@external` directives
- **Entity Resolution**: Implements `_entities` resolver for cross-service queries
- **Schema SDL**: Exposed via `_service` query for gateway composition

### Cross-Service Queries
```graphql
# Example: Get user with their processed content
query UserWithContent($userId: ID!) {
  user(id: $userId) {
    id
    name
    processingJobs {
      id
      status
      content {
        title
        qualityScore
      }
    }
  }
}
```

### Federation Testing
```bash
# Test federation integration
cargo test graphql_federation_test -- --ignored

# Verify subgraph schema
curl http://localhost:8082/graphql \
  -H "Content-Type: application/json" \
  -d '{"query": "{ _service { sdl } }"}'
```

## Security Considerations

### Authentication & Authorization
1. **JWT Validation**: All GraphQL requests require valid JWT tokens
2. **Multi-tenant Isolation**: Content scoped by tenant ID from JWT claims
3. **Rate Limiting**: Configurable per-tenant and global limits
4. **CORS Protection**: Configurable allowed origins

### Content Security
1. **Input Validation**: All content is validated before processing
2. **Plugin Sandboxing**: WASM plugins run in isolated environment with:
   - Memory limits (default 10MB per plugin)
   - CPU time limits
   - No filesystem access
   - No network access
3. **Resource Limits**: Memory and CPU limits enforced
4. **SQL Injection**: SQLx provides compile-time query verification
5. **Content Size Limits**: Configurable max content size (default 10MB)

### Security Headers
```rust
// Automatically applied to all responses
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
Content-Security-Policy: default-src 'none'
```

## Future Enhancements

1. **GPU Acceleration**: For ML-based analysis
2. **Streaming Support**: For large documents
3. **GraphQL API**: Alternative to REST
4. **Multi-language Plugins**: Support more WASM languages
5. **Distributed Processing**: Cluster support for scale