//! Core traits for content processing components

use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::*;

/// Core trait for content processing implementations
#[async_trait]
pub trait ContentProcessor: Send + Sync {
    /// Process content with the given options
    async fn process(
        &self,
        content: &[u8],
        content_type: ContentType,
        options: ProcessingOptions,
        context: &ProcessingContext,
    ) -> crate::Result<ProcessingResult>;

    /// Get supported content types
    fn supported_types(&self) -> Vec<ContentType>;
    
    /// Get processor name for identification
    fn name(&self) -> &'static str;
    
    /// Validate input before processing
    fn validate_input(&self, content: &[u8], content_type: &ContentType) -> crate::Result<()>;
    
    /// Get processor capabilities
    fn capabilities(&self) -> ProcessorCapabilities;
}

/// Capabilities of a content processor
#[derive(Debug, Clone)]
pub struct ProcessorCapabilities {
    pub max_content_size_bytes: u64,
    pub supports_streaming: bool,
    pub supports_cancellation: bool,
    pub estimated_processing_time_per_mb: std::time::Duration,
}

/// Trait for content parsers that convert raw content to structured format
#[async_trait]
pub trait ContentParser: Send + Sync {
    /// Parse content into structured format
    async fn parse(&self, raw_content: &[u8]) -> crate::Result<ParsedContent>;
    
    /// Check if parser can handle the content type
    fn supports(&self, content_type: &ContentType) -> bool;
    
    /// Get parser name
    fn name(&self) -> &'static str;
    
    /// Estimate parsing time for given content size
    fn estimate_parsing_time(&self, content_size_bytes: u64) -> std::time::Duration;
    
    /// Validate content before parsing
    fn validate_content(&self, raw_content: &[u8], content_type: &ContentType) -> crate::Result<()>;
}

/// Trait for text analysis components
#[async_trait]
pub trait TextAnalyzer: Send + Sync {
    /// Extract concepts from text
    async fn extract_concepts(&self, text: &str, context: &ProcessingContext) -> crate::Result<Vec<Concept>>;
    
    /// Assess content quality
    async fn assess_quality(&self, text: &str, context: &ProcessingContext) -> crate::Result<QualityMetrics>;
    
    /// Analyze content difficulty
    async fn analyze_difficulty(&self, text: &str, context: &ProcessingContext) -> crate::Result<DifficultyAnalysis>;
    
    /// Extract learning objectives
    async fn extract_objectives(&self, text: &str, context: &ProcessingContext) -> crate::Result<Vec<LearningObjective>>;
    
    /// Extract named entities
    async fn extract_entities(&self, text: &str, context: &ProcessingContext) -> crate::Result<Vec<Entity>>;
    
    /// Generate summary
    async fn generate_summary(&self, text: &str, max_length: Option<usize>, context: &ProcessingContext) -> crate::Result<String>;
    
    /// Detect language
    async fn detect_language(&self, text: &str) -> crate::Result<String>;
    
    /// Extract keywords
    async fn extract_keywords(&self, text: &str, max_keywords: Option<usize>) -> crate::Result<Vec<String>>;
}

/// Trait for batch processing strategies
#[async_trait]
pub trait BatchProcessor: Send + Sync {
    /// Process multiple items in batch
    async fn process_batch(
        &self,
        items: Vec<BatchItem>,
        options: BatchOptions,
        context: &ProcessingContext,
    ) -> crate::Result<BatchResult>;
    
    /// Get batch processing capabilities
    fn capabilities(&self) -> BatchCapabilities;
    
    /// Cancel a batch job
    async fn cancel_batch(&self, job_id: Uuid) -> crate::Result<()>;
    
    /// Get batch job status
    async fn get_batch_status(&self, job_id: Uuid) -> crate::Result<BatchStatus>;
    
    /// Pause a batch job
    async fn pause_batch(&self, job_id: Uuid) -> crate::Result<()>;
    
    /// Resume a paused batch job
    async fn resume_batch(&self, job_id: Uuid) -> crate::Result<()>;
}

/// Trait for WebAssembly plugins
#[async_trait]
pub trait WasmPlugin: Send + Sync {
    /// Initialize the plugin with configuration
    async fn init(&mut self, config: HashMap<String, String>) -> crate::Result<()>;
    
    /// Process content using the plugin
    async fn process(&self, input: PluginInput, context: &ProcessingContext) -> crate::Result<PluginOutput>;
    
    /// Get plugin metadata
    fn metadata(&self) -> PluginMetadata;
    
    /// Cleanup resources
    async fn cleanup(&mut self) -> crate::Result<()>;
    
    /// Validate plugin configuration
    fn validate_config(&self, config: &HashMap<String, String>) -> crate::Result<()>;
    
    /// Check if plugin is healthy
    async fn health_check(&self) -> crate::Result<PluginHealthStatus>;
}

/// Health status of a plugin
#[derive(Debug, Clone)]
pub enum PluginHealthStatus {
    Healthy,
    Degraded { reason: String },
    Unhealthy { reason: String },
}

/// Trait for caching strategies
#[async_trait]  
pub trait CacheStrategy: Send + Sync {
    /// Get cached result
    async fn get(&self, key: &str) -> crate::Result<Option<ProcessingResult>>;
    
    /// Store result in cache
    async fn set(&self, key: &str, result: &ProcessingResult, ttl: Option<u64>) -> crate::Result<()>;
    
    /// Invalidate cache entry
    async fn invalidate(&self, key: &str) -> crate::Result<()>;
    
    /// Clear all cache entries
    async fn clear(&self) -> crate::Result<()>;
    
    /// Get cache statistics
    async fn stats(&self) -> crate::Result<CacheStats>;
    
    /// Check if cache is healthy
    async fn health_check(&self) -> crate::Result<bool>;
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hit_count: u64,
    pub miss_count: u64,
    pub entry_count: u64,
    pub total_size_bytes: u64,
    pub hit_rate: f64,
}

/// Trait for progress tracking
#[async_trait]
pub trait ProgressTracker: Send + Sync {
    /// Update progress for a job
    async fn update_progress(&self, job_id: Uuid, progress: f32, message: Option<String>) -> crate::Result<()>;
    
    /// Mark job as completed
    async fn mark_completed(&self, job_id: Uuid, result: &ProcessingResult) -> crate::Result<()>;
    
    /// Mark job as failed
    async fn mark_failed(&self, job_id: Uuid, error: &ProcessingError) -> crate::Result<()>;
    
    /// Get current progress
    async fn get_progress(&self, job_id: Uuid) -> crate::Result<Option<JobProgress>>;
    
    /// Subscribe to progress updates for a job
    async fn subscribe_progress(&self, job_id: Uuid) -> crate::Result<Box<dyn futures::Stream<Item = JobProgress> + Send + Unpin>>;
    
    /// Clean up completed job progress data
    async fn cleanup_job(&self, job_id: Uuid) -> crate::Result<()>;
}