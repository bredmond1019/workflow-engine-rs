//! Core content processing logic

use async_trait::async_trait;
use std::time::Duration;
use uuid::Uuid;

use crate::models::*;
use crate::traits::{ContentProcessor as ContentProcessorTrait, ProcessorCapabilities, TextAnalyzer, ContentParser};
use crate::parsers::UniversalParser;
use crate::analysis::ComprehensiveAnalyzer;

/// Default content processor implementation
pub struct DefaultContentProcessor {
    name: &'static str,
    parser: UniversalParser,
    analyzer: ComprehensiveAnalyzer,
}

impl DefaultContentProcessor {
    pub fn new() -> Self {
        Self {
            name: "default_processor",
            parser: UniversalParser::new(),
            analyzer: ComprehensiveAnalyzer::new(),
        }
    }
}

impl Default for DefaultContentProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ContentProcessorTrait for DefaultContentProcessor {
    async fn process(
        &self,
        content: &[u8],
        content_type: ContentType,
        options: ProcessingOptions,
        context: &ProcessingContext,
    ) -> crate::Result<ProcessingResult> {
        let start_time = std::time::Instant::now();
        
        // Step 1: Parse the content
        let parsed_content = self.parser.parse(content).await?;
        
        // Step 2: Run analysis based on options
        let mut concepts = Vec::new();
        let mut quality_metrics = None;
        let mut difficulty_analysis = None;
        let mut learning_objectives = Vec::new();
        let mut keywords = Vec::new();
        let mut entities = Vec::new();
        let mut summary = None;
        let mut language = None;
        
        let text = &parsed_content.text;
        
        // Language detection
        if options.detect_language {
            match self.analyzer.detect_language(text).await {
                Ok(lang) => language = Some(lang),
                Err(_) => {}, // Continue processing even if language detection fails
            }
        }
        
        // Concept extraction
        if options.extract_concepts {
            match self.analyzer.extract_concepts(text, context).await {
                Ok(extracted_concepts) => concepts = extracted_concepts,
                Err(_) => {}, // Continue processing
            }
        }
        
        // Quality assessment
        if options.assess_quality {
            match self.analyzer.assess_quality(text, context).await {
                Ok(quality) => quality_metrics = Some(quality),
                Err(_) => {},
            }
        }
        
        // Difficulty analysis
        if options.analyze_difficulty {
            match self.analyzer.analyze_difficulty(text, context).await {
                Ok(difficulty) => difficulty_analysis = Some(difficulty),
                Err(_) => {},
            }
        }
        
        // Learning objectives extraction
        if options.extract_objectives {
            match self.analyzer.extract_objectives(text, context).await {
                Ok(objectives) => learning_objectives = objectives,
                Err(_) => {},
            }
        }
        
        // Keyword extraction
        if options.extract_keywords {
            match self.analyzer.extract_keywords(text, Some(15)).await {
                Ok(extracted_keywords) => keywords = extracted_keywords,
                Err(_) => {},
            }
        }
        
        // Entity extraction
        match self.analyzer.extract_entities(text, context).await {
            Ok(extracted_entities) => entities = extracted_entities,
            Err(_) => {},
        }
        
        // Summary generation
        if options.generate_summary {
            match self.analyzer.generate_summary(text, Some(500), context).await {
                Ok(generated_summary) => summary = Some(generated_summary),
                Err(_) => {},
            }
        }
        
        let processing_time = start_time.elapsed().as_millis() as u64;
        
        // Update metadata with extracted language
        let mut content_metadata = parsed_content.metadata;
        content_metadata.id = context.job_id;
        if content_metadata.language.is_none() {
            content_metadata.language = language.clone();
        }
        
        let output = ProcessingOutput {
            id: Uuid::new_v4(),
            content_metadata,
            concepts,
            quality_metrics,
            difficulty_analysis,
            learning_objectives,
            keywords,
            entities,
            summary,
            language,
            processing_time_ms: processing_time,
            processed_at: chrono::Utc::now(),
            plugin_results: std::collections::HashMap::new(), // TODO: Plugin processing
        };
        
        Ok(ProcessingResult::Success(output))
    }

    fn supported_types(&self) -> Vec<ContentType> {
        vec![
            ContentType::Html,
            ContentType::Markdown,
            ContentType::PlainText,
            ContentType::Json,
            ContentType::Xml,
            ContentType::Pdf,
            ContentType::Code,
        ]
    }
    
    fn name(&self) -> &'static str {
        self.name
    }
    
    fn validate_input(&self, content: &[u8], _content_type: &ContentType) -> crate::Result<()> {
        if content.is_empty() {
            return Err(ProcessingError::ValidationError {
                field: "content".to_string(),
                message: "Content cannot be empty".to_string(),
            });
        }
        
        // Check maximum size (10MB default)
        if content.len() > 10 * 1024 * 1024 {
            return Err(ProcessingError::ValidationError {
                field: "content".to_string(),
                message: "Content size exceeds maximum limit of 10MB".to_string(),
            });
        }
        
        Ok(())
    }
    
    fn capabilities(&self) -> ProcessorCapabilities {
        ProcessorCapabilities {
            max_content_size_bytes: 10 * 1024 * 1024, // 10MB
            supports_streaming: false,
            supports_cancellation: false,
            estimated_processing_time_per_mb: Duration::from_millis(100),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;
    use std::collections::HashMap;
    
    #[tokio::test]
    async fn test_processor_creation() {
        let processor = DefaultContentProcessor::new();
        assert_eq!(processor.name(), "default_processor");
    }
    
    #[tokio::test]
    async fn test_processor_default() {
        let processor = DefaultContentProcessor::default();
        assert_eq!(processor.name(), "default_processor");
    }
    
    #[tokio::test]
    async fn test_supported_types() {
        let processor = DefaultContentProcessor::new();
        let types = processor.supported_types();
        
        assert_eq!(types.len(), 5);
        assert!(types.contains(&ContentType::Html));
        assert!(types.contains(&ContentType::Markdown));
        assert!(types.contains(&ContentType::PlainText));
        assert!(types.contains(&ContentType::Json));
        assert!(types.contains(&ContentType::Xml));
        
        // Ensure unsupported types are not included
        assert!(!types.contains(&ContentType::Pdf));
        assert!(!types.contains(&ContentType::Video));
        assert!(!types.contains(&ContentType::Code));
    }
    
    #[tokio::test]
    async fn test_validate_input_empty_content() {
        let processor = DefaultContentProcessor::new();
        let result = processor.validate_input(&[], &ContentType::PlainText);
        
        assert!(result.is_err());
        match result {
            Err(ProcessingError::ValidationError { field, message }) => {
                assert_eq!(field, "content");
                assert_eq!(message, "Content cannot be empty");
            }
            _ => panic!("Expected ValidationError for empty content"),
        }
    }
    
    #[tokio::test]
    async fn test_validate_input_content_too_large() {
        let processor = DefaultContentProcessor::new();
        let large_content = vec![0u8; 11 * 1024 * 1024]; // 11MB
        let result = processor.validate_input(&large_content, &ContentType::PlainText);
        
        assert!(result.is_err());
        match result {
            Err(ProcessingError::ValidationError { field, message }) => {
                assert_eq!(field, "content");
                assert!(message.contains("exceeds maximum limit"));
            }
            _ => panic!("Expected ValidationError for large content"),
        }
    }
    
    #[tokio::test]
    async fn test_validate_input_valid_content() {
        let processor = DefaultContentProcessor::new();
        let content = b"This is valid content";
        let result = processor.validate_input(content, &ContentType::PlainText);
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_validate_input_edge_case_max_size() {
        let processor = DefaultContentProcessor::new();
        let max_content = vec![0u8; 10 * 1024 * 1024]; // Exactly 10MB
        let result = processor.validate_input(&max_content, &ContentType::PlainText);
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_process_simple_text() {
        let processor = DefaultContentProcessor::new();
        let content = b"This is a simple test text for processing.";
        let options = ProcessingOptions::default();
        let context = ProcessingContext {
            job_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            user_id: Some(Uuid::new_v4()),
            webhook_url: None,
            priority: ProcessingPriority::Normal,
            metadata: HashMap::new(),
        };
        
        let result = processor.process(content, ContentType::PlainText, options, &context).await;
        assert!(result.is_ok());
        
        match result.unwrap() {
            ProcessingResult::Success(output) => {
                assert_eq!(output.content_metadata.content_type, ContentType::PlainText);
                assert_eq!(output.content_metadata.size_bytes, content.len() as u64);
                assert_eq!(output.content_metadata.id, context.job_id);
                assert_eq!(output.processing_time_ms, 100);
                assert!(output.concepts.is_empty());
                assert!(output.keywords.is_empty());
                assert!(output.entities.is_empty());
                assert!(output.learning_objectives.is_empty());
                assert!(output.quality_metrics.is_none());
                assert!(output.difficulty_analysis.is_none());
                assert!(output.summary.is_none());
                assert!(output.language.is_none());
            }
            _ => panic!("Expected Success result"),
        }
    }
    
    #[tokio::test]
    async fn test_process_with_all_options() {
        let processor = DefaultContentProcessor::new();
        let content = b"Advanced machine learning concepts in neural networks.";
        let mut options = ProcessingOptions::default();
        options.extract_concepts = true;
        options.assess_quality = true;
        options.analyze_difficulty = true;
        options.extract_objectives = true;
        options.generate_summary = true;
        options.extract_keywords = true;
        options.detect_language = true;
        options.timeout_seconds = Some(60);
        options.verbose_logging = true;
        
        let context = ProcessingContext {
            job_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            user_id: None,
            webhook_url: Some("https://example.com/webhook".to_string()),
            priority: ProcessingPriority::High,
            metadata: HashMap::new(),
        };
        
        let result = processor.process(content, ContentType::PlainText, options, &context).await;
        assert!(result.is_ok());
        
        match result.unwrap() {
            ProcessingResult::Success(output) => {
                assert_eq!(output.content_metadata.size_bytes, content.len() as u64);
                assert_eq!(output.content_metadata.encoding, Some("utf-8".to_string()));
            }
            _ => panic!("Expected Success result"),
        }
    }
    
    #[tokio::test]
    async fn test_process_different_content_types() {
        let processor = DefaultContentProcessor::new();
        let context = ProcessingContext {
            job_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            user_id: None,
            webhook_url: None,
            priority: ProcessingPriority::Normal,
            metadata: HashMap::new(),
        };
        
        let test_cases = vec![
            (ContentType::Html, b"<html><body>Test</body></html>".to_vec()),
            (ContentType::Markdown, b"# Test\n\nThis is markdown".to_vec()),
            (ContentType::Json, b"{\"test\": \"data\"}".to_vec()),
            (ContentType::Xml, b"<root><test>data</test></root>".to_vec()),
        ];
        
        for (content_type, content) in test_cases {
            let result = processor.process(&content, content_type.clone(), ProcessingOptions::default(), &context).await;
            assert!(result.is_ok());
            
            match result.unwrap() {
                ProcessingResult::Success(output) => {
                    assert_eq!(output.content_metadata.content_type, content_type);
                }
                _ => panic!("Expected Success result for {:?}", content_type),
            }
        }
    }
    
    #[tokio::test]
    async fn test_capabilities() {
        let processor = DefaultContentProcessor::new();
        let caps = processor.capabilities();
        
        assert_eq!(caps.max_content_size_bytes, 10 * 1024 * 1024);
        assert!(!caps.supports_streaming);
        assert!(!caps.supports_cancellation);
        assert_eq!(caps.estimated_processing_time_per_mb, Duration::from_millis(100));
    }
    
    #[tokio::test]
    async fn test_process_with_metadata() {
        let processor = DefaultContentProcessor::new();
        let content = b"Test content with metadata";
        let options = ProcessingOptions::default();
        
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test_suite".to_string());
        metadata.insert("version".to_string(), "1.0".to_string());
        
        let context = ProcessingContext {
            job_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            user_id: Some(Uuid::new_v4()),
            webhook_url: None,
            priority: ProcessingPriority::Low,
            metadata,
        };
        
        let result = processor.process(content, ContentType::PlainText, options, &context).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_process_with_plugin_params() {
        let processor = DefaultContentProcessor::new();
        let content = b"Content for plugin processing";
        let mut options = ProcessingOptions::default();
        
        options.plugins = vec!["test_plugin".to_string()];
        options.plugin_params.insert("test_plugin".to_string(), serde_json::json!({
            "param1": "value1",
            "param2": 42
        }));
        
        let context = ProcessingContext {
            job_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            user_id: None,
            webhook_url: None,
            priority: ProcessingPriority::Normal,
            metadata: HashMap::new(),
        };
        
        let result = processor.process(content, ContentType::PlainText, options, &context).await;
        assert!(result.is_ok());
    }
}