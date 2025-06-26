#![allow(clippy::too_many_arguments, clippy::large_enum_variant)]

use async_graphql::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    ContentType as DomainContentType, ProcessingOptions as DomainProcessingOptions,
    ProcessingContext, ProcessingResult as DomainProcessingResult,
    ProcessingOutput, ProcessingError as DomainProcessingError,
    EntityType as DomainEntityType,
    DifficultyLevel as DomainDifficultyLevel,
};
use crate::processor::DefaultContentProcessor;
use crate::traits::ContentProcessor as ContentProcessorTrait;

// The QueryRoot already includes federation support, so no need for a merged object

pub type ContentProcessingSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

#[derive(Default)]
pub struct QueryRoot;
pub struct MutationRoot;

// GraphQL types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMetadata {
    pub id: ID,
    pub title: String,
    pub source_url: Option<String>,
    pub content_type: ContentType,
    pub format: ContentFormat,
    pub size_bytes: Option<i32>,
    pub hash: String,
    pub quality_score: Option<f64>,
    pub difficulty_level: Option<DifficultyLevel>,
    pub concepts: Vec<ContentConcept>,
    pub keywords: Vec<String>,
    pub entities: Vec<ExtractedEntity>,
    pub summary: Option<String>,
    pub language: Option<String>,
    pub has_embeddings: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingJob {
    pub id: ID,
    pub content_id: Option<ID>,
    pub job_type: JobType,
    pub status: JobStatus,
    pub priority: JobPriority,
    pub options: ProcessingOptions,
    pub result: Option<ProcessingResult>,
    pub error_message: Option<String>,
    pub retry_count: i32,
    pub worker_id: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub user_id: Option<ID>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: ID,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: ID,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentConcept {
    pub name: String,
    pub relevance: f64,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntity {
    pub text: String,
    pub entity_type: EntityType,
    pub confidence: f64,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingResult {
    pub success: bool,
    pub job_id: ID,
    pub content: Option<ContentMetadata>,
    pub processing_time: i32,
    pub errors: Vec<ProcessingError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSearchResult {
    pub content: Vec<ContentMetadata>,
    pub total_count: i32,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingOptions {
    pub extract_concepts: bool,
    pub assess_quality: bool,
    pub analyze_difficulty: bool,
    pub extract_objectives: bool,
    pub generate_summary: bool,
    pub extract_keywords: bool,
    pub detect_language: bool,
    pub plugins: Vec<String>,
    pub timeout_seconds: Option<i32>,
    pub plugin_params: Option<serde_json::Value>,
    pub verbose_logging: bool,
}

// Enums
#[derive(Debug, Clone, Serialize, Deserialize, Enum, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Enum, Copy, PartialEq, Eq)]
pub enum ContentFormat {
    Text,
    Binary,
    Structured,
    Media,
}

#[derive(Debug, Clone, Serialize, Deserialize, Enum, Copy, PartialEq, Eq)]
pub enum JobType {
    Analysis,
    Extraction,
    Transformation,
    PluginExecution,
}

#[derive(Debug, Clone, Serialize, Deserialize, Enum, Copy, PartialEq, Eq)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, Enum, Copy, PartialEq, Eq)]
pub enum JobPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, Enum, Copy, PartialEq, Eq)]
pub enum DifficultyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize, Enum, Copy, PartialEq, Eq)]
pub enum EntityType {
    Person,
    Organization,
    Location,
    Date,
    Technology,
    Concept,
    Other,
}

// Input types
#[derive(Debug, Clone, Serialize, Deserialize, InputObject)]
pub struct AnalyzeContentInput {
    pub content: String,
    pub content_type: ContentType,
    pub source_url: Option<String>,
    pub options: ProcessingOptionsInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, InputObject)]
pub struct CreateProcessingJobInput {
    pub content_url: Option<String>,
    pub content_data: Option<String>,
    pub content_type: ContentType,
    pub priority: Option<JobPriority>,
    pub options: ProcessingOptionsInput,
    pub webhook_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, InputObject)]
pub struct ProcessingOptionsInput {
    pub extract_concepts: Option<bool>,
    pub assess_quality: Option<bool>,
    pub analyze_difficulty: Option<bool>,
    pub extract_objectives: Option<bool>,
    pub generate_summary: Option<bool>,
    pub extract_keywords: Option<bool>,
    pub detect_language: Option<bool>,
    pub plugins: Option<Vec<String>>,
    pub timeout_seconds: Option<i32>,
    pub plugin_params: Option<serde_json::Value>,
    pub verbose_logging: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, InputObject)]
pub struct UpdateContentMetadataInput {
    pub title: Option<String>,
    pub tags: Option<Vec<String>>,
    pub quality_score: Option<f64>,
    pub difficulty_level: Option<DifficultyLevel>,
    pub metadata: Option<serde_json::Value>,
}

// Federation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub sdl: String,
}

#[Object]
impl Service {
    async fn sdl(&self) -> &str {
        &self.sdl
    }
}

// Object implementations
#[Object]
impl ContentMetadata {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn title(&self) -> &str {
        &self.title
    }

    async fn source_url(&self) -> Option<&str> {
        self.source_url.as_deref()
    }

    async fn content_type(&self) -> ContentType {
        self.content_type
    }

    async fn format(&self) -> ContentFormat {
        self.format
    }

    async fn size_bytes(&self) -> Option<i32> {
        self.size_bytes
    }

    async fn hash(&self) -> &str {
        &self.hash
    }

    async fn quality_score(&self) -> Option<f64> {
        self.quality_score
    }

    async fn difficulty_level(&self) -> Option<DifficultyLevel> {
        self.difficulty_level
    }

    async fn concepts(&self) -> &[ContentConcept] {
        &self.concepts
    }

    async fn keywords(&self) -> &[String] {
        &self.keywords
    }

    async fn entities(&self) -> &[ExtractedEntity] {
        &self.entities
    }

    async fn summary(&self) -> Option<&str> {
        self.summary.as_deref()
    }

    async fn language(&self) -> Option<&str> {
        self.language.as_deref()
    }

    async fn has_embeddings(&self) -> bool {
        self.has_embeddings
    }

    async fn created_at(&self) -> String {
        self.created_at.to_rfc3339()
    }

    async fn updated_at(&self) -> String {
        self.updated_at.to_rfc3339()
    }

    async fn processing_jobs(&self, _ctx: &Context<'_>) -> Result<Vec<ProcessingJob>> {
        // Would fetch related processing jobs
        Ok(vec![])
    }

    async fn related_content(&self, _ctx: &Context<'_>, limit: Option<i32>) -> Result<Vec<ContentMetadata>> {
        // Would fetch related content based on embeddings
        let _limit = limit.unwrap_or(10);
        Ok(vec![])
    }
}

#[Object]
impl ProcessingJob {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn content_id(&self) -> Option<&ID> {
        self.content_id.as_ref()
    }

    async fn content(&self, _ctx: &Context<'_>) -> Result<Option<ContentMetadata>> {
        // Would fetch content by ID
        Ok(None)
    }

    async fn job_type(&self) -> JobType {
        self.job_type
    }

    async fn status(&self) -> JobStatus {
        self.status
    }

    async fn priority(&self) -> JobPriority {
        self.priority
    }

    async fn options(&self) -> &ProcessingOptions {
        &self.options
    }

    async fn result(&self) -> Option<&ProcessingResult> {
        self.result.as_ref()
    }

    async fn error_message(&self) -> Option<&str> {
        self.error_message.as_deref()
    }

    async fn retry_count(&self) -> i32 {
        self.retry_count
    }

    async fn worker_id(&self) -> Option<&str> {
        self.worker_id.as_deref()
    }

    async fn started_at(&self) -> Option<String> {
        self.started_at.map(|dt| dt.to_rfc3339())
    }

    async fn completed_at(&self) -> Option<String> {
        self.completed_at.map(|dt| dt.to_rfc3339())
    }

    async fn created_at(&self) -> String {
        self.created_at.to_rfc3339()
    }

    async fn user_id(&self) -> Option<&ID> {
        self.user_id.as_ref()
    }

    async fn user(&self) -> Option<User> {
        self.user_id.as_ref().map(|id| User { id: id.clone() })
    }
}

#[Object]
impl User {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn processing_jobs(&self, _ctx: &Context<'_>) -> Result<Vec<ProcessingJob>> {
        // Would fetch user's processing jobs
        Ok(vec![])
    }

    async fn processed_content(&self, _ctx: &Context<'_>) -> Result<Vec<ContentMetadata>> {
        // Would fetch user's processed content
        Ok(vec![])
    }
}

#[Object]
impl Workflow {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn processed_content(&self, _ctx: &Context<'_>) -> Result<Vec<ContentMetadata>> {
        // Would fetch workflow's processed content
        Ok(vec![])
    }
}

#[Object]
impl ContentConcept {
    async fn name(&self) -> &str {
        &self.name
    }

    async fn relevance(&self) -> f64 {
        self.relevance
    }

    async fn category(&self) -> Option<&str> {
        self.category.as_deref()
    }
}

#[Object]
impl ExtractedEntity {
    async fn text(&self) -> &str {
        &self.text
    }

    async fn entity_type(&self) -> EntityType {
        self.entity_type
    }

    async fn confidence(&self) -> f64 {
        self.confidence
    }

    async fn metadata(&self) -> Option<serde_json::Value> {
        self.metadata.clone()
    }
}

#[Object]
impl ProcessingResult {
    async fn success(&self) -> bool {
        self.success
    }

    async fn job_id(&self) -> &ID {
        &self.job_id
    }

    async fn content(&self) -> Option<&ContentMetadata> {
        self.content.as_ref()
    }

    async fn processing_time(&self) -> i32 {
        self.processing_time
    }

    async fn errors(&self) -> &[ProcessingError] {
        &self.errors
    }
}

#[Object]
impl ProcessingError {
    async fn code(&self) -> &str {
        &self.code
    }

    async fn message(&self) -> &str {
        &self.message
    }

    async fn details(&self) -> Option<serde_json::Value> {
        self.details.clone()
    }
}

#[Object]
impl ProcessingOptions {
    async fn extract_concepts(&self) -> bool {
        self.extract_concepts
    }

    async fn assess_quality(&self) -> bool {
        self.assess_quality
    }

    async fn analyze_difficulty(&self) -> bool {
        self.analyze_difficulty
    }

    async fn extract_objectives(&self) -> bool {
        self.extract_objectives
    }

    async fn generate_summary(&self) -> bool {
        self.generate_summary
    }

    async fn extract_keywords(&self) -> bool {
        self.extract_keywords
    }

    async fn detect_language(&self) -> bool {
        self.detect_language
    }

    async fn plugins(&self) -> &[String] {
        &self.plugins
    }

    async fn timeout_seconds(&self) -> Option<i32> {
        self.timeout_seconds
    }

    async fn plugin_params(&self) -> Option<serde_json::Value> {
        self.plugin_params.clone()
    }

    async fn verbose_logging(&self) -> bool {
        self.verbose_logging
    }
}

#[Object]
impl ContentSearchResult {
    async fn content(&self) -> &[ContentMetadata] {
        &self.content
    }

    async fn total_count(&self) -> i32 {
        self.total_count
    }

    async fn has_more(&self) -> bool {
        self.has_more
    }
}

// Query implementation
#[Object]
impl QueryRoot {
    async fn content(&self, _ctx: &Context<'_>, _id: ID) -> Result<Option<ContentMetadata>> {
        // Would fetch content by ID from database
        Ok(None)
    }

    async fn search_content(
        &self,
        _ctx: &Context<'_>,
        _query: Option<String>,
        _content_type: Option<ContentType>,
        _min_quality: Option<f64>,
        _tags: Option<Vec<String>>,
        _limit: Option<i32>,
        _offset: Option<i32>,
    ) -> Result<ContentSearchResult> {
        // Would search content in database
        Ok(ContentSearchResult {
            content: vec![],
            total_count: 0,
            has_more: false,
        })
    }

    async fn processing_job(&self, _ctx: &Context<'_>, _id: ID) -> Result<Option<ProcessingJob>> {
        // Would fetch job by ID
        Ok(None)
    }

    async fn user_processing_history(
        &self,
        _ctx: &Context<'_>,
        _user_id: ID,
        limit: Option<i32>,
    ) -> Result<Vec<ProcessingJob>> {
        let _limit = limit.unwrap_or(50);
        // Would fetch user's processing history
        Ok(vec![])
    }

    // Federation support
    async fn _service(&self) -> Service {
        Service {
            sdl: include_str!("schema.graphql").to_string(),
        }
    }

    // Federation entity resolution (using entities instead of _entities for now)
    async fn entities(&self, representations: Vec<Json<serde_json::Value>>) -> Result<Vec<Option<Json<serde_json::Value>>>> {
        let mut results = Vec::new();
        
        for representation in representations {
            let value = representation.0;
            
            if let Some(typename) = value.get("__typename").and_then(|t| t.as_str()) {
                match typename {
                    "ContentMetadata" => {
                        if let Some(id) = value.get("id").and_then(|i| i.as_str()) {
                            // TODO: Fetch from database
                            let metadata = serde_json::json!({
                                "__typename": "ContentMetadata",
                                "id": id,
                                "title": "Sample Content",
                                "contentType": "Markdown",
                                "format": "Text",
                                "language": "en",
                                "createdAt": Utc::now().to_rfc3339(),
                                "updatedAt": Utc::now().to_rfc3339()
                            });
                            results.push(Some(Json(metadata)));
                        } else {
                            results.push(None);
                        }
                    }
                    "ProcessingJob" => {
                        if let Some(id) = value.get("id").and_then(|i| i.as_str()) {
                            // TODO: Fetch from database
                            let job = serde_json::json!({
                                "__typename": "ProcessingJob",
                                "id": id,
                                "contentId": "content-1",
                                "status": "Completed",
                                "createdAt": Utc::now().to_rfc3339(),
                                "completedAt": Utc::now().to_rfc3339()
                            });
                            results.push(Some(Json(job)));
                        } else {
                            results.push(None);
                        }
                    }
                    "User" | "Workflow" => {
                        // External entities - just return the representation
                        results.push(Some(Json(value.clone())));
                    }
                    _ => {
                        // Unknown entity type - return null
                        results.push(None);
                    }
                }
            } else {
                results.push(None);
            }
        }
        
        Ok(results)
    }

    async fn _entities(&self, representations: Vec<Json<serde_json::Value>>) -> Result<Vec<Option<Json<serde_json::Value>>>> {
        let mut results = Vec::new();
        
        for representation in representations {
            let value = representation.0;
            
            if let Some(typename) = value.get("__typename").and_then(|t| t.as_str()) {
                match typename {
                    "ContentMetadata" => {
                        if let Some(id) = value.get("id").and_then(|i| i.as_str()) {
                            // TODO: Fetch from database
                            let metadata = serde_json::json!({
                                "__typename": "ContentMetadata",
                                "id": id,
                                "title": "Sample Content",
                                "contentType": "Markdown",
                                "format": "Text",
                                "language": "en",
                                "createdAt": Utc::now().to_rfc3339(),
                                "updatedAt": Utc::now().to_rfc3339()
                            });
                            results.push(Some(Json(metadata)));
                        } else {
                            results.push(None);
                        }
                    }
                    "ProcessingJob" => {
                        if let Some(id) = value.get("id").and_then(|i| i.as_str()) {
                            // TODO: Fetch from database
                            let job = serde_json::json!({
                                "__typename": "ProcessingJob",
                                "id": id,
                                "contentId": "content-1",
                                "status": "Completed",
                                "createdAt": Utc::now().to_rfc3339(),
                                "completedAt": Utc::now().to_rfc3339()
                            });
                            results.push(Some(Json(job)));
                        } else {
                            results.push(None);
                        }
                    }
                    "User" | "Workflow" => {
                        // External entities - just return the representation
                        results.push(Some(Json(value.clone())));
                    }
                    _ => results.push(None),
                }
            } else {
                results.push(None);
            }
        }
        
        Ok(results)
    }
}

// Mutation implementation
#[Object]
impl MutationRoot {
    async fn analyze_content(
        &self,
        _ctx: &Context<'_>,
        input: AnalyzeContentInput,
    ) -> Result<ProcessingResult> {
        let processor = DefaultContentProcessor::new();
        
        // Convert input options
        let options = convert_processing_options(input.options);
        let content_type = convert_content_type(input.content_type);
        
        // Create processing context
        let job_id = Uuid::new_v4();
        let mut context = ProcessingContext::new(job_id);
        context.processing_started_at = Some(chrono::Utc::now());
        
        // Process content
        let start_time = std::time::Instant::now();
        let result = processor.process(
            input.content.as_bytes(),
            content_type,
            options,
            &context
        ).await;
        
        let processing_time = start_time.elapsed().as_millis() as i32;
        
        match result {
            Ok(DomainProcessingResult::Success(output)) => {
                let content_metadata = convert_content_metadata(&output);
                Ok(ProcessingResult {
                    success: true,
                    job_id: ID(job_id.to_string()),
                    content: Some(content_metadata),
                    processing_time,
                    errors: vec![],
                })
            }
            Ok(DomainProcessingResult::Error(error)) => {
                Ok(ProcessingResult {
                    success: false,
                    job_id: ID(job_id.to_string()),
                    content: None,
                    processing_time,
                    errors: vec![convert_processing_error(&error)],
                })
            }
            Ok(DomainProcessingResult::Partial(output, errors)) => {
                let content_metadata = convert_content_metadata(&output);
                Ok(ProcessingResult {
                    success: true,
                    job_id: ID(job_id.to_string()),
                    content: Some(content_metadata),
                    processing_time,
                    errors: errors.iter().map(convert_processing_error).collect(),
                })
            }
            Err(e) => {
                Err(Error::new(format!("Processing failed: {}", e)))
            }
        }
    }

    async fn create_processing_job(
        &self,
        _ctx: &Context<'_>,
        input: CreateProcessingJobInput,
    ) -> Result<ProcessingJob> {
        // Would create job in database and queue for processing
        let job_id = Uuid::new_v4();
        Ok(ProcessingJob {
            id: ID(job_id.to_string()),
            content_id: None,
            job_type: JobType::Analysis,
            status: JobStatus::Pending,
            priority: input.priority.unwrap_or(JobPriority::Medium),
            options: ProcessingOptions {
                extract_concepts: input.options.extract_concepts.unwrap_or(true),
                assess_quality: input.options.assess_quality.unwrap_or(true),
                analyze_difficulty: input.options.analyze_difficulty.unwrap_or(true),
                extract_objectives: input.options.extract_objectives.unwrap_or(false),
                generate_summary: input.options.generate_summary.unwrap_or(true),
                extract_keywords: input.options.extract_keywords.unwrap_or(true),
                detect_language: input.options.detect_language.unwrap_or(true),
                plugins: input.options.plugins.unwrap_or_default(),
                timeout_seconds: input.options.timeout_seconds.or(Some(300)),
                plugin_params: input.options.plugin_params,
                verbose_logging: input.options.verbose_logging.unwrap_or(false),
            },
            result: None,
            error_message: None,
            retry_count: 0,
            worker_id: None,
            started_at: None,
            completed_at: None,
            created_at: Utc::now(),
            user_id: None, // Would extract from auth context
        })
    }

    async fn cancel_processing_job(&self, _ctx: &Context<'_>, _job_id: ID) -> Result<ProcessingJob> {
        // Would update job status in database
        Err(Error::new("Not implemented"))
    }

    async fn update_content_metadata(
        &self,
        _ctx: &Context<'_>,
        _id: ID,
        _input: UpdateContentMetadataInput,
    ) -> Result<ContentMetadata> {
        // Would update content metadata in database
        Err(Error::new("Not implemented"))
    }
}

// Federation entity references
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ContentMetadataRef {
    id: ID,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProcessingJobRef {
    id: ID,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserRef {
    id: ID,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkflowRef {
    id: ID,
}

// Entity union for federation
#[derive(Union, Clone)]
enum Entity {
    ContentMetadata(ContentMetadata),
    ProcessingJob(Box<ProcessingJob>),
    User(User),
    Workflow(Workflow),
}


// Schema creation function
pub fn create_schema() -> ContentProcessingSchema {
    Schema::build(QueryRoot::default(), MutationRoot, EmptySubscription)
        .enable_federation()
        .finish()
}

// Helper conversion functions
fn convert_content_type(graphql_type: ContentType) -> DomainContentType {
    match graphql_type {
        ContentType::Html => DomainContentType::Html,
        ContentType::Pdf => DomainContentType::Pdf,
        ContentType::Markdown => DomainContentType::Markdown,
        ContentType::Video => DomainContentType::Video,
        ContentType::Code => DomainContentType::Code,
        ContentType::PlainText => DomainContentType::PlainText,
        ContentType::Json => DomainContentType::Json,
        ContentType::Xml => DomainContentType::Xml,
    }
}

fn convert_processing_options(input: ProcessingOptionsInput) -> DomainProcessingOptions {
    DomainProcessingOptions {
        extract_concepts: input.extract_concepts.unwrap_or(true),
        assess_quality: input.assess_quality.unwrap_or(true),
        analyze_difficulty: input.analyze_difficulty.unwrap_or(true),
        extract_objectives: input.extract_objectives.unwrap_or(false),
        generate_summary: input.generate_summary.unwrap_or(true),
        extract_keywords: input.extract_keywords.unwrap_or(true),
        detect_language: input.detect_language.unwrap_or(true),
        plugins: input.plugins.unwrap_or_default(),
        timeout_seconds: input.timeout_seconds.map(|t| t as u32),
        plugin_params: input.plugin_params.map(|p| {
            p.as_object()
                .map(|obj| {
                    obj.iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect()
                })
                .unwrap_or_default()
        }).unwrap_or_default(),
        verbose_logging: input.verbose_logging.unwrap_or(false),
    }
}

fn convert_content_metadata(output: &ProcessingOutput) -> ContentMetadata {
    ContentMetadata {
        id: ID(output.id.to_string()),
        title: output.content_metadata.title.clone().unwrap_or_else(|| "Untitled".to_string()),
        source_url: output.content_metadata.source_url.clone(),
        content_type: match &output.content_metadata.content_type {
            DomainContentType::Html => ContentType::Html,
            DomainContentType::Pdf => ContentType::Pdf,
            DomainContentType::Markdown => ContentType::Markdown,
            DomainContentType::Video => ContentType::Video,
            DomainContentType::Code => ContentType::Code,
            DomainContentType::PlainText => ContentType::PlainText,
            DomainContentType::Json => ContentType::Json,
            DomainContentType::Xml => ContentType::Xml,
        },
        format: ContentFormat::Text, // Would determine based on content
        size_bytes: Some(output.content_metadata.size_bytes as i32),
        hash: format!("{:x}", md5::compute(output.id.to_string())), // Placeholder
        quality_score: output.quality_metrics.as_ref().map(|m| m.overall_score as f64),
        difficulty_level: output.difficulty_analysis.as_ref().map(|d| match d.overall_level {
            DomainDifficultyLevel::Beginner => DifficultyLevel::Beginner,
            DomainDifficultyLevel::Intermediate => DifficultyLevel::Intermediate,
            DomainDifficultyLevel::Advanced => DifficultyLevel::Advanced,
            DomainDifficultyLevel::Expert => DifficultyLevel::Expert,
        }),
        concepts: output.concepts.iter().map(|c| ContentConcept {
            name: c.name.clone(),
            relevance: c.importance_score as f64,
            category: match &c.category {
                crate::ConceptCategory::Domain(s) => Some(s.clone()),
                _ => None,
            },
        }).collect(),
        keywords: output.keywords.clone(),
        entities: output.entities.iter().map(|e| ExtractedEntity {
            text: e.name.clone(),
            entity_type: match &e.entity_type {
                DomainEntityType::Person => EntityType::Person,
                DomainEntityType::Organization => EntityType::Organization,
                DomainEntityType::Location => EntityType::Location,
                DomainEntityType::Date => EntityType::Date,
                DomainEntityType::Technology => EntityType::Technology,
                DomainEntityType::Concept => EntityType::Concept,
                _ => EntityType::Other,
            },
            confidence: e.confidence as f64,
            metadata: None,
        }).collect(),
        summary: output.summary.clone(),
        language: output.language.clone(),
        has_embeddings: false, // Would check if embeddings exist
        created_at: output.processed_at,
        updated_at: output.processed_at,
    }
}

fn convert_processing_error(error: &DomainProcessingError) -> ProcessingError {
    let (code, message) = match error {
        DomainProcessingError::ParseError { message, .. } => ("PARSE_ERROR", message.clone()),
        DomainProcessingError::UnsupportedFormat { content_type } => 
            ("UNSUPPORTED_FORMAT", format!("Unsupported content type: {}", content_type)),
        DomainProcessingError::TimeoutError { operation, timeout_seconds } =>
            ("TIMEOUT", format!("Operation '{}' timed out after {} seconds", operation, timeout_seconds)),
        DomainProcessingError::MemoryError { message, .. } => ("MEMORY_ERROR", message.clone()),
        DomainProcessingError::PluginError { plugin_name, error_message } =>
            ("PLUGIN_ERROR", format!("Plugin '{}' error: {}", plugin_name, error_message)),
        DomainProcessingError::NetworkError { error_message, .. } => ("NETWORK_ERROR", error_message.clone()),
        DomainProcessingError::ValidationError { field, message } =>
            ("VALIDATION_ERROR", format!("Field '{}': {}", field, message)),
        DomainProcessingError::InternalError { message, .. } => ("INTERNAL_ERROR", message.clone()),
        DomainProcessingError::InvalidInput { message } => ("INVALID_INPUT", message.clone()),
    };

    ProcessingError {
        code: code.to_string(),
        message,
        details: None,
    }
}