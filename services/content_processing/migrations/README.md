# Content Processing Database Migrations

This directory contains SQLx migrations for the Content Processing Engine.

## Running Migrations

```bash
# Install SQLx CLI if not already installed
cargo install sqlx-cli --no-default-features --features postgres

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert

# Create new migration
sqlx migrate add <migration_name>
```

## Schema Overview

### content_metadata
Stores metadata about processed content including:
- Basic info: title, source URL, type, format, size
- Content hash for deduplication
- Quality score and difficulty level
- Extracted concepts (JSONB)
- Vector embeddings for similarity search (pgvector)

### processing_jobs
Tracks content processing jobs with:
- Job type, status, and priority
- Processing options and results (JSONB)
- Error handling and retry logic
- Worker assignment for distributed processing
- Performance metrics tracking

### plugin_registry
Manages WebAssembly plugins:
- Plugin metadata and versioning
- SHA256 hash verification
- Capability declarations
- Execution statistics
- Health monitoring

### plugin_execution_logs
Detailed logging of plugin executions:
- Performance metrics (execution time, memory usage)
- Success/failure tracking
- Input/output size monitoring

## Indexes

The schema includes comprehensive indexes for:
- Efficient job queue operations
- Content search by type, quality, difficulty
- Vector similarity search using IVFFlat
- JSONB field searches using GIN indexes
- Time-based queries for metrics and logs