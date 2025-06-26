//! Text analysis pipeline for content processing
//!
//! This module provides comprehensive text analysis capabilities including:
//! - Concept extraction using NLP techniques
//! - Quality assessment and scoring
//! - Difficulty analysis for educational content
//! - Keyword and entity extraction
//! - Text summarization
//! - Language detection

pub mod concepts;
pub mod quality;
pub mod difficulty;
pub mod keywords;
pub mod entities;
pub mod summarization;
pub mod language;

use async_trait::async_trait;
use uuid::Uuid;

use crate::models::*;
use crate::traits::TextAnalyzer;

/// Comprehensive text analyzer that combines all analysis capabilities
pub struct ComprehensiveAnalyzer {
    concept_extractor: concepts::ConceptExtractor,
    quality_assessor: quality::QualityAssessor,
    difficulty_analyzer: difficulty::DifficultyAnalyzer,
    keyword_extractor: keywords::KeywordExtractor,
    entity_recognizer: entities::EntityRecognizer,
    summarizer: summarization::TextSummarizer,
    language_detector: language::LanguageDetector,
}

impl ComprehensiveAnalyzer {
    pub fn new() -> Self {
        Self {
            concept_extractor: concepts::ConceptExtractor::new(),
            quality_assessor: quality::QualityAssessor::new(),
            difficulty_analyzer: difficulty::DifficultyAnalyzer::new(),
            keyword_extractor: keywords::KeywordExtractor::new(),
            entity_recognizer: entities::EntityRecognizer::new(),
            summarizer: summarization::TextSummarizer::new(),
            language_detector: language::LanguageDetector::new(),
        }
    }
}

impl Default for ComprehensiveAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TextAnalyzer for ComprehensiveAnalyzer {
    async fn extract_concepts(&self, text: &str, context: &ProcessingContext) -> crate::Result<Vec<Concept>> {
        self.concept_extractor.extract_concepts(text, context).await
    }
    
    async fn assess_quality(&self, text: &str, context: &ProcessingContext) -> crate::Result<QualityMetrics> {
        self.quality_assessor.assess_quality(text, context).await
    }
    
    async fn analyze_difficulty(&self, text: &str, context: &ProcessingContext) -> crate::Result<DifficultyAnalysis> {
        self.difficulty_analyzer.analyze_difficulty(text, context).await
    }
    
    async fn extract_objectives(&self, text: &str, context: &ProcessingContext) -> crate::Result<Vec<LearningObjective>> {
        // Extract learning objectives from concepts and text structure
        let concepts = self.extract_concepts(text, context).await?;
        
        // For now, create basic learning objectives from concepts
        let mut objectives = Vec::new();
        
        for (i, concept) in concepts.iter().take(5).enumerate() {
            let objective = LearningObjective {
                id: Uuid::new_v4(),
                description: format!("Understand and apply the concept of {}", concept.name),
                bloom_taxonomy_level: match i % 6 {
                    0 => BloomLevel::Remember,
                    1 => BloomLevel::Understand,
                    2 => BloomLevel::Apply,
                    3 => BloomLevel::Analyze,
                    4 => BloomLevel::Evaluate,
                    5 => BloomLevel::Create,
                    _ => BloomLevel::Understand,
                },
                confidence: concept.confidence * 0.8, // Slightly lower confidence for derived objectives
                required_concepts: vec![concept.name.clone()],
                assessment_suggestions: vec![
                    format!("Define {}", concept.name),
                    format!("Explain the significance of {}", concept.name),
                    format!("Provide examples of {}", concept.name),
                ],
            };
            objectives.push(objective);
        }
        
        Ok(objectives)
    }
    
    async fn extract_entities(&self, text: &str, context: &ProcessingContext) -> crate::Result<Vec<Entity>> {
        self.entity_recognizer.extract_entities(text, context).await
    }
    
    async fn generate_summary(&self, text: &str, max_length: Option<usize>, context: &ProcessingContext) -> crate::Result<String> {
        self.summarizer.generate_summary(text, max_length, context).await
    }
    
    async fn detect_language(&self, text: &str) -> crate::Result<String> {
        self.language_detector.detect_language(text).await
    }
    
    async fn extract_keywords(&self, text: &str, max_keywords: Option<usize>) -> crate::Result<Vec<String>> {
        self.keyword_extractor.extract_keywords(text, max_keywords).await
    }
}

/// Configuration for analysis pipeline
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    pub max_concepts: usize,
    pub concept_confidence_threshold: f32,
    pub max_keywords: usize,
    pub keyword_frequency_threshold: usize,
    pub summary_max_length: usize,
    pub enable_entity_linking: bool,
    pub quality_weights: QualityWeights,
}

#[derive(Debug, Clone)]
pub struct QualityWeights {
    pub readability: f32,
    pub completeness: f32,
    pub accuracy: f32,
    pub coherence: f32,
    pub grammar: f32,
    pub vocabulary: f32,
    pub structure: f32,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            max_concepts: 20,
            concept_confidence_threshold: 0.6,
            max_keywords: 15,
            keyword_frequency_threshold: 2,
            summary_max_length: 500,
            enable_entity_linking: true,
            quality_weights: QualityWeights {
                readability: 0.2,
                completeness: 0.15,
                accuracy: 0.2,
                coherence: 0.15,
                grammar: 0.1,
                vocabulary: 0.1,
                structure: 0.1,
            },
        }
    }
}

impl Default for QualityWeights {
    fn default() -> Self {
        Self {
            readability: 0.2,
            completeness: 0.15,
            accuracy: 0.2,
            coherence: 0.15,
            grammar: 0.1,
            vocabulary: 0.1,
            structure: 0.1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_comprehensive_analyzer_creation() {
        let analyzer = ComprehensiveAnalyzer::new();
        assert_eq!(analyzer.concept_extractor.name(), "concept_extractor");
    }

    #[tokio::test]
    async fn test_analysis_config_defaults() {
        let config = AnalysisConfig::default();
        assert_eq!(config.max_concepts, 20);
        assert_eq!(config.concept_confidence_threshold, 0.6);
        assert!(config.enable_entity_linking);
    }

    #[tokio::test]
    async fn test_basic_text_analysis() {
        let analyzer = ComprehensiveAnalyzer::new();
        let _context = ProcessingContext::new(Uuid::new_v4());

        let text = "This is a simple test document about machine learning algorithms.";
        
        // Test language detection
        let language = analyzer.detect_language(text).await.unwrap();
        assert_eq!(language, "en");
        
        // Test keyword extraction
        let keywords = analyzer.extract_keywords(text, Some(5)).await.unwrap();
        assert!(!keywords.is_empty());
    }
}