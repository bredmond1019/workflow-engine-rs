# Content Processing Data Models

This document describes all data structures used in the Content Processing Service.

## Table of Contents

- [Request Models](#request-models)
- [Response Models](#response-models)
- [Core Models](#core-models)
- [Analysis Models](#analysis-models)
- [Database Models](#database-models)
- [Plugin Models](#plugin-models)

## Request Models

### AnalysisRequest

The main request structure for content analysis.

```rust
pub struct ProcessRequest {
    pub content: String,
    pub content_type: ContentType,
    pub options: ProcessingOptions,
}
```

#### Example JSON

```json
{
  "content": "Your document content here",
  "content_type": "PlainText",
  "options": {
    "extract_concepts": true,
    "assess_quality": true,
    "analyze_difficulty": true,
    "extract_objectives": true,
    "generate_summary": true,
    "extract_keywords": true,
    "detect_language": true,
    "plugins": ["sentiment_analyzer"],
    "timeout_seconds": 30,
    "plugin_params": {
      "sentiment_analyzer": {
        "threshold": 0.7
      }
    },
    "verbose_logging": false
  }
}
```

### ProcessingOptions

Configuration options for content processing.

```rust
pub struct ProcessingOptions {
    pub extract_concepts: bool,
    pub assess_quality: bool,
    pub analyze_difficulty: bool,
    pub extract_objectives: bool,
    pub generate_summary: bool,
    pub extract_keywords: bool,
    pub detect_language: bool,
    pub plugins: Vec<String>,
    pub timeout_seconds: Option<u32>,
    pub plugin_params: HashMap<String, serde_json::Value>,
    pub verbose_logging: bool,
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `extract_concepts` | bool | true | Extract key concepts from content |
| `assess_quality` | bool | true | Assess content quality metrics |
| `analyze_difficulty` | bool | true | Analyze difficulty level |
| `extract_objectives` | bool | true | Extract learning objectives |
| `generate_summary` | bool | true | Generate content summary |
| `extract_keywords` | bool | true | Extract keywords and entities |
| `detect_language` | bool | true | Detect content language |
| `plugins` | Vec<String> | [] | List of plugins to apply |
| `timeout_seconds` | Option<u32> | Some(30) | Processing timeout in seconds |
| `plugin_params` | HashMap | {} | Custom parameters for plugins |
| `verbose_logging` | bool | false | Enable detailed logging |

## Response Models

### ProcessingResult

The result of a content processing operation.

```rust
pub enum ProcessingResult {
    Success(ProcessingOutput),
    Error(ProcessingError),
    Partial(ProcessingOutput, Vec<ProcessingError>),
}
```

### ProcessingOutput

Successful processing output containing all analysis results.

```rust
pub struct ProcessingOutput {
    pub id: Uuid,
    pub content_metadata: ContentMetadata,
    pub concepts: Vec<Concept>,
    pub quality_metrics: Option<QualityMetrics>,
    pub difficulty_analysis: Option<DifficultyAnalysis>,
    pub learning_objectives: Vec<LearningObjective>,
    pub keywords: Vec<String>,
    pub entities: Vec<Entity>,
    pub summary: Option<String>,
    pub language: Option<String>,
    pub processing_time_ms: u64,
    pub processed_at: DateTime<Utc>,
    pub plugin_results: HashMap<String, serde_json::Value>,
}
```

#### Example JSON Response

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "content_metadata": {
    "id": "660e8400-e29b-41d4-a716-446655440000",
    "content_type": "PlainText",
    "size_bytes": 2048,
    "title": "Introduction to Machine Learning",
    "language": "en"
  },
  "concepts": [
    {
      "id": "770e8400-e29b-41d4-a716-446655440000",
      "name": "Supervised Learning",
      "confidence": 0.92,
      "category": "Technical",
      "importance_score": 0.85
    }
  ],
  "quality_metrics": {
    "overall_score": 0.87,
    "readability_score": 0.82,
    "completeness_score": 0.90
  },
  "keywords": ["machine learning", "algorithms", "neural networks"],
  "summary": "An introduction to fundamental machine learning concepts...",
  "processing_time_ms": 342,
  "processed_at": "2023-12-09T10:30:00Z"
}
```

## Core Models

### ContentType

Supported content types for processing.

```rust
pub enum ContentType {
    Html,
    Pdf,
    Markdown,
    Video,
    Code,
    PlainText,
    Json,
    Xml,
}
```

### ContentMetadata

Metadata about the processed content.

```rust
pub struct ContentMetadata {
    pub id: Uuid,
    pub content_type: ContentType,
    pub size_bytes: u64,
    pub title: Option<String>,
    pub author: Option<String>,
    pub source_url: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub last_modified: Option<DateTime<Utc>>,
    pub encoding: Option<String>,
    pub mime_type: Option<String>,
    pub language: Option<String>,
    pub version: Option<String>,
    pub tags: Vec<String>,
    pub custom_fields: HashMap<String, serde_json::Value>,
}
```

### ProcessingError

Errors that can occur during processing.

```rust
pub enum ProcessingError {
    ParseError {
        message: String,
        position: Option<u32>,
    },
    UnsupportedFormat {
        content_type: String,
    },
    TimeoutError {
        operation: String,
        timeout_seconds: u32,
    },
    MemoryError {
        message: String,
        memory_used_mb: u32,
    },
    PluginError {
        plugin_name: String,
        error_message: String,
    },
    NetworkError {
        url: Option<String>,
        error_message: String,
    },
    ValidationError {
        field: String,
        message: String,
    },
    InternalError {
        message: String,
        trace: Option<String>,
    },
}
```

## Analysis Models

### Concept

Extracted concept with confidence score.

```rust
pub struct Concept {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub confidence: f32,
    pub category: ConceptCategory,
    pub related_concepts: Vec<String>,
    pub mentions: Vec<ConceptMention>,
    pub importance_score: f32,
}
```

### ConceptCategory

Categories for concepts.

```rust
pub enum ConceptCategory {
    Technical,
    Business,
    Scientific,
    Educational,
    General,
    Domain(String),
}
```

### QualityMetrics

Content quality assessment metrics.

```rust
pub struct QualityMetrics {
    pub overall_score: f32,
    pub readability_score: f32,
    pub completeness_score: f32,
    pub accuracy_score: f32,
    pub coherence_score: f32,
    pub grammar_score: f32,
    pub vocabulary_richness: f32,
    pub structure_quality: f32,
    pub issues: Vec<QualityIssue>,
}
```

### QualityIssue

Identified quality issues in content.

```rust
pub struct QualityIssue {
    pub issue_type: QualityIssueType,
    pub severity: IssueSeverity,
    pub description: String,
    pub position: Option<u32>,
    pub suggestions: Vec<String>,
}
```

### DifficultyAnalysis

Content difficulty analysis results.

```rust
pub struct DifficultyAnalysis {
    pub overall_level: DifficultyLevel,
    pub vocabulary_complexity: f32,
    pub concept_density: f32,
    pub sentence_complexity: f32,
    pub prerequisite_knowledge: Vec<String>,
    pub estimated_reading_time: u32,
    pub cognitive_load_score: f32,
    pub target_audience: Vec<String>,
}
```

### DifficultyLevel

Difficulty level classifications.

```rust
pub enum DifficultyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}
```

### Entity

Named entity extracted from content.

```rust
pub struct Entity {
    pub name: String,
    pub entity_type: EntityType,
    pub confidence: f32,
    pub mentions: Vec<EntityMention>,
    pub linked_data_uri: Option<String>,
}
```

### EntityType

Types of named entities.

```rust
pub enum EntityType {
    Person,
    Organization,
    Location,
    Date,
    Money,
    Technology,
    Concept,
    Other(String),
}
```

### LearningObjective

Learning objective extracted from educational content.

```rust
pub struct LearningObjective {
    pub id: Uuid,
    pub description: String,
    pub bloom_taxonomy_level: BloomLevel,
    pub confidence: f32,
    pub required_concepts: Vec<String>,
    pub assessment_suggestions: Vec<String>,
}
```

### BloomLevel

Bloom's taxonomy levels for learning objectives.

```rust
pub enum BloomLevel {
    Remember,
    Understand,
    Apply,
    Analyze,
    Evaluate,
    Create,
}
```

## Database Models

### content_metadata Table

Stores metadata for processed content.

```sql
CREATE TABLE content_metadata (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title TEXT NOT NULL,
    source_url TEXT,
    content_type VARCHAR(50) NOT NULL,
    format VARCHAR(50) NOT NULL,
    size_bytes BIGINT,
    hash VARCHAR(64) NOT NULL,
    quality_score FLOAT,
    difficulty_level VARCHAR(20),
    concepts JSONB DEFAULT '[]'::jsonb,
    embeddings vector(1536),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### processing_jobs Table

Tracks processing job status and results.

```sql
CREATE TABLE processing_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    content_id UUID REFERENCES content_metadata(id),
    job_type VARCHAR(50) NOT NULL,
    status job_status NOT NULL DEFAULT 'pending',
    priority job_priority NOT NULL DEFAULT 'medium',
    options JSONB DEFAULT '{}'::jsonb,
    result JSONB,
    error_message TEXT,
    retry_count INT DEFAULT 0,
    max_retries INT DEFAULT 3,
    worker_id VARCHAR(100),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### Enums

```sql
CREATE TYPE job_status AS ENUM ('pending', 'running', 'completed', 'failed', 'cancelled');
CREATE TYPE job_priority AS ENUM ('low', 'medium', 'high', 'critical');
```

## Plugin Models

### PluginInput

Input structure for plugin processing.

```rust
pub struct PluginInput {
    pub content: String,
    pub content_type: ContentType,
    pub metadata: ContentMetadata,
    pub context: HashMap<String, serde_json::Value>,
}
```

### PluginOutput

Output structure from plugin processing.

```rust
pub struct PluginOutput {
    pub modified_content: Option<String>,
    pub extracted_data: HashMap<String, serde_json::Value>,
    pub metadata_updates: HashMap<String, serde_json::Value>,
    pub processing_time_ms: u64,
}
```

### PluginMetadata

Plugin information and capabilities.

```rust
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub supported_content_types: Vec<ContentType>,
    pub required_permissions: Vec<String>,
    pub configuration_schema: Option<serde_json::Value>,
    pub performance_metrics: Option<PluginPerformanceMetrics>,
}
```

## Content Structure Models

### ParsedContent

Parsed content with structural information.

```rust
pub struct ParsedContent {
    pub content_type: ContentType,
    pub text: String,
    pub metadata: ContentMetadata,
    pub structure: ContentStructure,
    pub media_elements: Vec<MediaElement>,
}
```

### ContentStructure

Hierarchical structure of content.

```rust
pub struct ContentStructure {
    pub sections: Vec<ContentSection>,
    pub table_of_contents: Vec<TocEntry>,
    pub links: Vec<Link>,
    pub citations: Vec<Citation>,
}
```

### ContentSection

Content section with hierarchy information.

```rust
pub struct ContentSection {
    pub id: String,
    pub title: Option<String>,
    pub level: u32,
    pub content: String,
    pub start_position: u32,
    pub end_position: u32,
    pub subsections: Vec<ContentSection>,
}
```

## Batch Processing Models

### BatchOptions

Options for batch processing jobs.

```rust
pub struct BatchOptions {
    pub max_parallel_jobs: Option<u32>,
    pub timeout_seconds: Option<u32>,
    pub continue_on_error: bool,
    pub priority: ProcessingPriority,
    pub progress_callback_url: Option<String>,
    pub result_storage: BatchResultStorage,
}
```

### BatchResult

Result of batch processing operation.

```rust
pub struct BatchResult {
    pub job_id: Uuid,
    pub total_items: u32,
    pub completed_items: u32,
    pub failed_items: u32,
    pub processing_time_ms: u64,
    pub results: Vec<ProcessingResult>,
    pub errors: Vec<(Uuid, ProcessingError)>,
    pub status: BatchStatus,
}
```

### BatchStatus

Status of batch processing job.

```rust
pub enum BatchStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    PartiallyCompleted,
}
```

## Type Constraints and Validation

### Field Constraints

| Model | Field | Constraints |
|-------|-------|-------------|
| ProcessingOptions | timeout_seconds | 1-600 |
| ContentMetadata | size_bytes | Max 10MB |
| Concept | confidence | 0.0-1.0 |
| QualityMetrics | all scores | 0.0-1.0 |
| DifficultyAnalysis | cognitive_load_score | 0.0-1.0 |
| Entity | confidence | 0.0-1.0 |

### Required Fields

- **ProcessRequest**: content, content_type, options
- **ContentMetadata**: id, content_type, size_bytes
- **Concept**: id, name, confidence, category
- **Entity**: name, entity_type, confidence

### Default Values

- **ProcessingOptions**: All analysis options default to `true`
- **timeout_seconds**: 30 seconds
- **retry_count**: 0
- **max_retries**: 3
- **priority**: "medium"