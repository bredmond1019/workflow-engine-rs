//! Data models for content processing

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

/// Supported content types for processing
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContentType::Html => write!(f, "html"),
            ContentType::Pdf => write!(f, "pdf"),
            ContentType::Markdown => write!(f, "markdown"),
            ContentType::Video => write!(f, "video"),
            ContentType::Code => write!(f, "code"),
            ContentType::PlainText => write!(f, "text"),
            ContentType::Json => write!(f, "json"),
            ContentType::Xml => write!(f, "xml"),
        }
    }
}

/// Configuration options for content processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingOptions {
    /// Extract key concepts from content
    pub extract_concepts: bool,
    /// Assess content quality metrics
    pub assess_quality: bool,
    /// Analyze difficulty level
    pub analyze_difficulty: bool,
    /// Extract learning objectives
    pub extract_objectives: bool,
    /// Generate content summary
    pub generate_summary: bool,
    /// Extract keywords and entities
    pub extract_keywords: bool,
    /// Detect content language
    pub detect_language: bool,
    /// List of plugins to apply
    pub plugins: Vec<String>,
    /// Maximum processing time in seconds
    pub timeout_seconds: Option<u32>,
    /// Custom parameters for plugins
    pub plugin_params: HashMap<String, serde_json::Value>,
    /// Enable detailed logging
    pub verbose_logging: bool,
}

impl Default for ProcessingOptions {
    fn default() -> Self {
        Self {
            extract_concepts: true,
            assess_quality: true,
            analyze_difficulty: true,
            extract_objectives: true,
            generate_summary: true,
            extract_keywords: true,
            detect_language: true,
            plugins: Vec::new(),
            timeout_seconds: Some(30),
            plugin_params: HashMap::new(),
            verbose_logging: false,
        }
    }
}

/// Result of content processing operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessingResult {
    Success(ProcessingOutput),
    Error(ProcessingError),
    Partial(ProcessingOutput, Vec<ProcessingError>),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_content_type_display() {
        assert_eq!(ContentType::Html.to_string(), "html");
        assert_eq!(ContentType::Pdf.to_string(), "pdf");
        assert_eq!(ContentType::Markdown.to_string(), "markdown");
        assert_eq!(ContentType::Video.to_string(), "video");
        assert_eq!(ContentType::Code.to_string(), "code");
        assert_eq!(ContentType::PlainText.to_string(), "text");
        assert_eq!(ContentType::Json.to_string(), "json");
        assert_eq!(ContentType::Xml.to_string(), "xml");
    }
    
    #[test]
    fn test_processing_options_default() {
        let options = ProcessingOptions::default();
        
        assert!(options.extract_concepts);
        assert!(options.assess_quality);
        assert!(options.analyze_difficulty);
        assert!(options.extract_objectives);
        assert!(options.generate_summary);
        assert!(options.extract_keywords);
        assert!(options.detect_language);
        assert!(options.plugins.is_empty());
        assert_eq!(options.timeout_seconds, Some(30));
        assert!(options.plugin_params.is_empty());
        assert!(!options.verbose_logging);
    }
    
    #[test]
    fn test_processing_options_serialization() {
        let mut options = ProcessingOptions::default();
        options.plugins = vec!["test_plugin".to_string()];
        options.timeout_seconds = Some(60);
        options.verbose_logging = true;
        
        let serialized = serde_json::to_string(&options).unwrap();
        let deserialized: ProcessingOptions = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(deserialized.plugins.len(), 1);
        assert_eq!(deserialized.plugins[0], "test_plugin");
        assert_eq!(deserialized.timeout_seconds, Some(60));
        assert!(deserialized.verbose_logging);
    }
    
    #[test]
    fn test_processing_result_variants() {
        let output = ProcessingOutput {
            id: Uuid::new_v4(),
            content_metadata: ContentMetadata {
                id: Uuid::new_v4(),
                content_type: ContentType::PlainText,
                size_bytes: 100,
                created_at: Some(chrono::Utc::now()),
                last_modified: Some(chrono::Utc::now()),
                title: Some("Test Title".to_string()),
                author: None,
                source_url: None,
                encoding: Some("utf-8".to_string()),
                mime_type: Some("text/plain".to_string()),
                language: Some("en".to_string()),
                version: None,
                tags: vec!["test".to_string()],
                custom_fields: HashMap::new(),
            },
            concepts: vec![],
            quality_metrics: None,
            difficulty_analysis: None,
            learning_objectives: vec![],
            keywords: vec!["test".to_string()],
            entities: vec![],
            summary: Some("Test summary".to_string()),
            language: Some("en".to_string()),
            processing_time_ms: 100,
            processed_at: chrono::Utc::now(),
            plugin_results: HashMap::new(),
        };
        
        // Test Success variant
        let success_result = ProcessingResult::Success(output.clone());
        matches!(success_result, ProcessingResult::Success(_));
        
        // Test Error variant
        let error = ProcessingError::InvalidInput("test error".to_string());
        let error_result = ProcessingResult::Error(error.clone());
        matches!(error_result, ProcessingResult::Error(_));
        
        // Test Partial variant
        let partial_result = ProcessingResult::Partial(output, vec![error]);
        matches!(partial_result, ProcessingResult::Partial(_, _));
    }
}

/// Successful processing output
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Metadata about the processed content
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Processing context for shared state
#[derive(Debug, Clone)]
pub struct ProcessingContext {
    pub job_id: Uuid,
    pub correlation_id: Uuid,
    pub user_id: Option<Uuid>,
    pub webhook_url: Option<String>,
    pub priority: ProcessingPriority,
    pub metadata: HashMap<String, String>,
}

impl ProcessingContext {
    pub fn new(job_id: Uuid) -> Self {
        Self {
            job_id,
            correlation_id: Uuid::new_v4(),
            user_id: None,
            webhook_url: None,
            priority: ProcessingPriority::Normal,
            metadata: HashMap::new(),
        }
    }
}

/// Processing priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessingPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Extracted concept with confidence score
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Categories for concepts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConceptCategory {
    Technical,
    Business,
    Scientific,
    Educational,
    General,
    Domain(String),
}

/// Location where a concept was mentioned
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptMention {
    pub position: u32,
    pub context: String,
    pub confidence: f32,
}

/// Content quality assessment metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Identified quality issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIssue {
    pub issue_type: QualityIssueType,
    pub severity: IssueSeverity,
    pub description: String,
    pub position: Option<u32>,
    pub suggestions: Vec<String>,
}

/// Types of quality issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityIssueType {
    Grammar,
    Spelling,
    Structure,
    Clarity,
    Completeness,
    Factual,
    Formatting,
}

/// Severity levels for issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Content difficulty analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Difficulty levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DifficultyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// Learning objective extracted from content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningObjective {
    pub id: Uuid,
    pub description: String,
    pub bloom_taxonomy_level: BloomLevel,
    pub confidence: f32,
    pub required_concepts: Vec<String>,
    pub assessment_suggestions: Vec<String>,
}

/// Bloom's taxonomy levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BloomLevel {
    Remember,
    Understand,
    Apply,
    Analyze,
    Evaluate,
    Create,
}

/// Named entity extracted from content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    pub entity_type: EntityType,
    pub confidence: f32,
    pub mentions: Vec<EntityMention>,
    pub linked_data_uri: Option<String>,
}

/// Types of named entities
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
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

/// Entity mention in content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMention {
    pub position: u32,
    pub context: String,
    pub confidence: f32,
}

/// Errors that can occur during processing
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::ParseError { message, position } => {
                if let Some(pos) = position {
                    write!(f, "Parse error at position {}: {}", pos, message)
                } else {
                    write!(f, "Parse error: {}", message)
                }
            }
            ProcessingError::UnsupportedFormat { content_type } => {
                write!(f, "Unsupported content type: {}", content_type)
            }
            ProcessingError::TimeoutError { operation, timeout_seconds } => {
                write!(f, "Operation '{}' timed out after {} seconds", operation, timeout_seconds)
            }
            ProcessingError::MemoryError { message, memory_used_mb } => {
                write!(f, "Memory error ({}MB used): {}", memory_used_mb, message)
            }
            ProcessingError::PluginError { plugin_name, error_message } => {
                write!(f, "Plugin '{}' error: {}", plugin_name, error_message)
            }
            ProcessingError::NetworkError { url, error_message } => {
                if let Some(u) = url {
                    write!(f, "Network error for {}: {}", u, error_message)
                } else {
                    write!(f, "Network error: {}", error_message)
                }
            }
            ProcessingError::ValidationError { field, message } => {
                write!(f, "Validation error for field '{}': {}", field, message)
            }
            ProcessingError::InternalError { message, .. } => {
                write!(f, "Internal error: {}", message)
            }
        }
    }
}

impl std::error::Error for ProcessingError {}

/// Parsed content structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedContent {
    pub content_type: ContentType,
    pub text: String,
    pub metadata: ContentMetadata,
    pub structure: ContentStructure,
    pub media_elements: Vec<MediaElement>,
}

/// Hierarchical structure of content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentStructure {
    pub sections: Vec<ContentSection>,
    pub table_of_contents: Vec<TocEntry>,
    pub links: Vec<Link>,
    pub citations: Vec<Citation>,
}

/// Content section with hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSection {
    pub id: String,
    pub title: Option<String>,
    pub level: u32,
    pub content: String,
    pub start_position: u32,
    pub end_position: u32,
    pub subsections: Vec<ContentSection>,
}

/// Table of contents entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocEntry {
    pub title: String,
    pub level: u32,
    pub position: u32,
    pub section_id: String,
}

/// Link found in content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub url: String,
    pub text: Option<String>,
    pub link_type: LinkType,
    pub position: u32,
}

/// Types of links
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LinkType {
    External,
    Internal,
    Email,
    Phone,
    File,
}

/// Citation or reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub id: String,
    pub text: String,
    pub url: Option<String>,
    pub position: u32,
}

/// Media element found in content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaElement {
    pub element_type: MediaType,
    pub url: Option<String>,
    pub alt_text: Option<String>,
    pub caption: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Types of media elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MediaType {
    Image,
    Video,
    Audio,
    Diagram,
    Chart,
    Code,
    Table,
}

// Batch processing models

/// Item in a batch processing job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchItem {
    pub id: Uuid,
    pub content: Vec<u8>,
    pub content_type: ContentType,
    pub metadata: ContentMetadata,
    pub processing_options: ProcessingOptions,
}

/// Options for batch processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOptions {
    pub max_parallel_jobs: Option<u32>,
    pub timeout_seconds: Option<u32>,
    pub continue_on_error: bool,
    pub priority: ProcessingPriority,
    pub progress_callback_url: Option<String>,
    pub result_storage: BatchResultStorage,
}

/// Where to store batch results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchResultStorage {
    Memory,
    Database,
    FileSystem { path: String },
    S3 { bucket: String, prefix: String },
}

/// Result of batch processing
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Status of batch processing job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    PartiallyCompleted,
}

/// Capabilities of a batch processor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCapabilities {
    pub max_concurrent_jobs: u32,
    pub max_items_per_batch: u32,
    pub supported_content_types: Vec<ContentType>,
    pub supports_cancellation: bool,
    pub supports_prioritization: bool,
}

/// Progress information for a job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobProgress {
    pub job_id: Uuid,
    pub progress_percent: f32,
    pub current_step: String,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub last_updated: DateTime<Utc>,
}

// Plugin system models

/// Input for plugin processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInput {
    pub content: String,
    pub content_type: ContentType,
    pub metadata: ContentMetadata,
    pub context: HashMap<String, serde_json::Value>,
}

/// Output from plugin processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginOutput {
    pub modified_content: Option<String>,
    pub extracted_data: HashMap<String, serde_json::Value>,
    pub metadata_updates: HashMap<String, serde_json::Value>,
    pub processing_time_ms: u64,
}

/// Plugin metadata and information
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Performance metrics for plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPerformanceMetrics {
    pub average_processing_time_ms: f64,
    pub memory_usage_mb: f64,
    pub success_rate: f64,
    pub last_benchmark: DateTime<Utc>,
}