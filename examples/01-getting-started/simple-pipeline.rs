//! # Simple Pipeline - Building Your First Complete Workflow
//!
//! This example demonstrates how to build a complete processing pipeline
//! with multiple nodes working together to solve a real-world problem.
//!
//! ## What You'll Learn
//! - Building multi-stage workflows
//! - Real-world processing patterns
//! - Error handling in pipelines
//! - Conditional processing logic
//! - Pipeline monitoring and debugging
//!
//! ## Use Case: Content Processing Pipeline
//! We'll build a content processing pipeline that:
//! 1. Validates incoming content
//! 2. Extracts metadata and keywords
//! 3. Applies content transformations
//! 4. Generates tags and categories
//! 5. Creates a publication-ready result
//!
//! ## Usage
//! ```bash
//! cargo run --bin simple-pipeline
//! ```

use workflow_engine_core::prelude::*;
use serde_json::json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Content input structure
#[derive(Debug, Deserialize, Serialize, Clone)]
struct ContentInput {
    title: String,
    body: String,
    author: String,
    content_type: String, // "article", "blog", "news", etc.
    tags: Option<Vec<String>>,
    metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Validation results
#[derive(Debug, Deserialize, Serialize)]
struct ValidationResult {
    is_valid: bool,
    issues: Vec<String>,
    warnings: Vec<String>,
    score: f64,
    validated_at: String,
}

/// Extracted metadata and analysis
#[derive(Debug, Deserialize, Serialize)]
struct ContentAnalysis {
    word_count: usize,
    estimated_reading_time: f64, // in minutes
    keywords: Vec<String>,
    language: String,
    readability_score: f64,
    sentiment_score: f64,
    analyzed_at: String,
}

/// Processed content ready for publication
#[derive(Debug, Deserialize, Serialize)]
struct ProcessedContent {
    original: ContentInput,
    formatted_title: String,
    formatted_body: String,
    slug: String,
    generated_tags: Vec<String>,
    category: String,
    seo_metadata: HashMap<String, String>,
    processed_at: String,
}

/// Final publication package
#[derive(Debug, Deserialize, Serialize)]
struct PublicationResult {
    content: ProcessedContent,
    analysis: ContentAnalysis,
    validation: ValidationResult,
    publication_ready: bool,
    next_steps: Vec<String>,
    quality_score: f64,
    created_at: String,
}

/// Stage 1: Content Validation Node
/// Validates incoming content for basic requirements
#[derive(Debug)]
struct ContentValidationNode {
    min_title_length: usize,
    min_body_length: usize,
    required_fields: Vec<String>,
}

impl ContentValidationNode {
    fn new() -> Self {
        Self {
            min_title_length: 10,
            min_body_length: 100,
            required_fields: vec!["title".to_string(), "body".to_string(), "author".to_string()],
        }
    }
    
    fn validate_content(&self, content: &ContentInput) -> ValidationResult {
        let mut issues = Vec::new();
        let mut warnings = Vec::new();
        let mut score = 1.0;
        
        // Title validation
        if content.title.len() < self.min_title_length {
            issues.push(format!("Title too short: {} chars (min: {})", 
                content.title.len(), self.min_title_length));
            score -= 0.3;
        }
        
        if content.title.len() > 100 {
            warnings.push("Title might be too long for SEO".to_string());
            score -= 0.1;
        }
        
        // Body validation
        if content.body.len() < self.min_body_length {
            issues.push(format!("Body too short: {} chars (min: {})", 
                content.body.len(), self.min_body_length));
            score -= 0.4;
        }
        
        // Author validation
        if content.author.trim().is_empty() {
            issues.push("Author name is required".to_string());
            score -= 0.2;
        }
        
        // Content type validation
        let valid_types = ["article", "blog", "news", "tutorial", "guide"];
        if !valid_types.contains(&content.content_type.as_str()) {
            warnings.push(format!("Unusual content type: '{}'", content.content_type));
            score -= 0.05;
        }
        
        // Quality checks
        if content.body.matches('\n').count() < 3 {
            warnings.push("Content has very few paragraphs".to_string());
            score -= 0.1;
        }
        
        ValidationResult {
            is_valid: issues.is_empty(),
            issues,
            warnings,
            score: score.max(0.0),
            validated_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

impl Node for ContentValidationNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("🔍 ContentValidationNode: Validating content...");
        
        let content: ContentInput = context.get_event_data()?;
        
        println!("   📝 Validating: '{}' by {}", content.title, content.author);
        
        let validation_result = self.validate_content(&content);
        
        if validation_result.is_valid {
            println!("   ✅ Content validation passed (score: {:.2})", validation_result.score);
        } else {
            println!("   ❌ Content validation failed ({} issues)", validation_result.issues.len());
            for issue in &validation_result.issues {
                println!("      • {}", issue);
            }
        }
        
        if !validation_result.warnings.is_empty() {
            println!("   ⚠️  Warnings ({}):", validation_result.warnings.len());
            for warning in &validation_result.warnings {
                println!("      • {}", warning);
            }
        }
        
        context.update_node("validation", &validation_result);
        context.set_metadata("validation_score", validation_result.score)?;
        context.set_metadata("validation_passed", validation_result.is_valid)?;
        
        Ok(context)
    }
}

/// Stage 2: Content Analysis Node
/// Extracts metadata, keywords, and performs content analysis
#[derive(Debug)]
struct ContentAnalysisNode;

impl ContentAnalysisNode {
    fn extract_keywords(&self, text: &str) -> Vec<String> {
        // Simple keyword extraction (in real implementation, use NLP libraries)
        let words: Vec<&str> = text
            .to_lowercase()
            .split_whitespace()
            .filter(|word| word.len() > 4)
            .collect();
        
        let mut word_counts = HashMap::new();
        for word in words {
            *word_counts.entry(word).or_insert(0) += 1;
        }
        
        let mut keywords: Vec<(String, usize)> = word_counts
            .into_iter()
            .filter(|(_, count)| *count >= 2)
            .map(|(word, count)| (word.to_string(), count))
            .collect();
        
        keywords.sort_by(|a, b| b.1.cmp(&a.1));
        keywords.into_iter().take(10).map(|(word, _)| word).collect()
    }
    
    fn estimate_reading_time(&self, word_count: usize) -> f64 {
        // Average reading speed: 200-250 words per minute
        word_count as f64 / 225.0
    }
    
    fn calculate_readability_score(&self, text: &str) -> f64 {
        // Simplified readability score based on sentence and word length
        let sentences = text.split('.').count();
        let words = text.split_whitespace().count();
        let avg_words_per_sentence = if sentences > 0 { words as f64 / sentences as f64 } else { 0.0 };
        
        // Score based on average sentence length (lower is better)
        let score = if avg_words_per_sentence <= 15.0 { 0.9 } 
                   else if avg_words_per_sentence <= 20.0 { 0.7 }
                   else if avg_words_per_sentence <= 25.0 { 0.5 }
                   else { 0.3 };
        
        score
    }
    
    fn analyze_sentiment(&self, text: &str) -> f64 {
        // Simple sentiment analysis (in real implementation, use ML models)
        let positive_words = ["good", "great", "excellent", "amazing", "wonderful", "fantastic"];
        let negative_words = ["bad", "terrible", "awful", "horrible", "disappointing"];
        
        let text_lower = text.to_lowercase();
        let positive_count = positive_words.iter()
            .map(|word| text_lower.matches(word).count())
            .sum::<usize>();
        let negative_count = negative_words.iter()
            .map(|word| text_lower.matches(word).count())
            .sum::<usize>();
        
        let total_words = text.split_whitespace().count();
        if total_words == 0 { return 0.5; }
        
        let sentiment_score = 0.5 + 
            (positive_count as f64 - negative_count as f64) / (total_words as f64 * 0.1);
        
        sentiment_score.max(0.0).min(1.0)
    }
    
    fn detect_language(&self, _text: &str) -> String {
        // Simplified language detection (would use proper libraries in production)
        "english".to_string()
    }
}

impl Node for ContentAnalysisNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("📊 ContentAnalysisNode: Analyzing content...");
        
        let content: ContentInput = context.get_event_data()?;
        let full_text = format!("{} {}", content.title, content.body);
        
        let word_count = content.body.split_whitespace().count();
        
        let analysis = ContentAnalysis {
            word_count,
            estimated_reading_time: self.estimate_reading_time(word_count),
            keywords: self.extract_keywords(&full_text),
            language: self.detect_language(&content.body),
            readability_score: self.calculate_readability_score(&content.body),
            sentiment_score: self.analyze_sentiment(&content.body),
            analyzed_at: chrono::Utc::now().to_rfc3339(),
        };
        
        println!("   📈 Analysis results:");
        println!("      Words: {}", analysis.word_count);
        println!("      Reading time: {:.1} minutes", analysis.estimated_reading_time);
        println!("      Keywords: {}", analysis.keywords.len());
        println!("      Readability: {:.2}", analysis.readability_score);
        println!("      Sentiment: {:.2}", analysis.sentiment_score);
        
        context.update_node("analysis", &analysis);
        context.set_metadata("word_count", analysis.word_count)?;
        context.set_metadata("readability_score", analysis.readability_score)?;
        context.set_metadata("sentiment_score", analysis.sentiment_score)?;
        
        println!("   ✅ Content analysis completed");
        
        Ok(context)
    }
}

/// Stage 3: Content Processing Node
/// Formats content and generates publication-ready version
#[derive(Debug)]
struct ContentProcessingNode;

impl ContentProcessingNode {
    fn format_title(&self, title: &str) -> String {
        // Proper title case formatting
        title.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars.as_str().to_lowercase().chars()).collect(),
                }
            })
            .collect::<Vec<String>>()
            .join(" ")
    }
    
    fn format_body(&self, body: &str) -> String {
        // Basic body formatting: proper paragraphs, clean whitespace
        body.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<&str>>()
            .join("\n\n")
    }
    
    fn generate_slug(&self, title: &str) -> String {
        title.to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>()
            .join("-")
    }
    
    fn generate_tags(&self, content: &ContentInput, keywords: &[String]) -> Vec<String> {
        let mut tags = Vec::new();
        
        // Add existing tags
        if let Some(existing_tags) = &content.tags {
            tags.extend(existing_tags.clone());
        }
        
        // Add content type as tag
        tags.push(content.content_type.clone());
        
        // Add top keywords as tags
        tags.extend(keywords.iter().take(5).cloned());
        
        // Remove duplicates and clean up
        tags.sort();
        tags.dedup();
        tags.into_iter()
            .filter(|tag| !tag.is_empty() && tag.len() >= 3)
            .take(10)
            .collect()
    }
    
    fn determine_category(&self, content: &ContentInput, keywords: &[String]) -> String {
        // Simple category determination based on content type and keywords
        let content_type = &content.content_type;
        
        if keywords.iter().any(|k| k.contains("technology") || k.contains("programming")) {
            "Technology".to_string()
        } else if keywords.iter().any(|k| k.contains("business") || k.contains("market")) {
            "Business".to_string()
        } else if content_type == "tutorial" || content_type == "guide" {
            "Education".to_string()
        } else if content_type == "news" {
            "News".to_string()
        } else {
            "General".to_string()
        }
    }
    
    fn generate_seo_metadata(&self, content: &ProcessedContent) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        
        // Meta description (first 160 chars of body)
        let description = content.formatted_body
            .chars()
            .take(157)
            .collect::<String>() + "...";
        metadata.insert("description".to_string(), description);
        
        // Keywords for SEO
        metadata.insert("keywords".to_string(), content.generated_tags.join(", "));
        
        // Open Graph tags
        metadata.insert("og:title".to_string(), content.formatted_title.clone());
        metadata.insert("og:type".to_string(), "article".to_string());
        
        // Twitter tags
        metadata.insert("twitter:title".to_string(), content.formatted_title.clone());
        metadata.insert("twitter:card".to_string(), "summary".to_string());
        
        metadata
    }
}

impl Node for ContentProcessingNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("🔄 ContentProcessingNode: Processing content for publication...");
        
        let content: ContentInput = context.get_event_data()?;
        let analysis: ContentAnalysis = context
            .get_node_data("analysis")?
            .ok_or_else(|| WorkflowError::validation_error("Missing content analysis"))?;
        
        let formatted_title = self.format_title(&content.title);
        let formatted_body = self.format_body(&content.body);
        let slug = self.generate_slug(&content.title);
        let generated_tags = self.generate_tags(&content, &analysis.keywords);
        let category = self.determine_category(&content, &analysis.keywords);
        
        let mut processed_content = ProcessedContent {
            original: content.clone(),
            formatted_title,
            formatted_body,
            slug,
            generated_tags,
            category,
            seo_metadata: HashMap::new(),
            processed_at: chrono::Utc::now().to_rfc3339(),
        };
        
        // Generate SEO metadata after we have the processed content
        processed_content.seo_metadata = self.generate_seo_metadata(&processed_content);
        
        println!("   📝 Processing results:");
        println!("      Slug: {}", processed_content.slug);
        println!("      Category: {}", processed_content.category);
        println!("      Tags: {} generated", processed_content.generated_tags.len());
        println!("      SEO metadata: {} fields", processed_content.seo_metadata.len());
        
        context.update_node("processed_content", &processed_content);
        context.set_metadata("slug", &processed_content.slug)?;
        context.set_metadata("category", &processed_content.category)?;
        context.set_metadata("tags_count", processed_content.generated_tags.len())?;
        
        println!("   ✅ Content processing completed");
        
        Ok(context)
    }
}

/// Stage 4: Publication Assembly Node
/// Creates the final publication-ready package
#[derive(Debug)]
struct PublicationAssemblyNode;

impl PublicationAssemblyNode {
    fn calculate_quality_score(&self, validation: &ValidationResult, analysis: &ContentAnalysis) -> f64 {
        let mut quality_score = 0.0;
        
        // Validation score (40% weight)
        quality_score += validation.score * 0.4;
        
        // Readability score (30% weight)
        quality_score += analysis.readability_score * 0.3;
        
        // Content length score (20% weight)
        let length_score = if analysis.word_count >= 500 { 1.0 }
                          else if analysis.word_count >= 300 { 0.8 }
                          else if analysis.word_count >= 150 { 0.6 }
                          else { 0.3 };
        quality_score += length_score * 0.2;
        
        // Keyword richness (10% weight)
        let keyword_score = (analysis.keywords.len() as f64 / 10.0).min(1.0);
        quality_score += keyword_score * 0.1;
        
        quality_score.min(1.0)
    }
    
    fn determine_next_steps(&self, publication_ready: bool, validation: &ValidationResult) -> Vec<String> {
        let mut steps = Vec::new();
        
        if !publication_ready {
            steps.push("Content requires revision before publication".to_string());
            
            if !validation.is_valid {
                steps.push("Address validation issues".to_string());
            }
            
            if !validation.warnings.is_empty() {
                steps.push("Review and address warnings".to_string());
            }
        } else {
            steps.push("Content is ready for publication".to_string());
            steps.push("Review final formatting and metadata".to_string());
            steps.push("Schedule publication or publish immediately".to_string());
        }
        
        steps.push("Monitor engagement after publication".to_string());
        steps
    }
}

impl Node for PublicationAssemblyNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("📦 PublicationAssemblyNode: Assembling final publication package...");
        
        // Gather all data from previous stages
        let validation: ValidationResult = context
            .get_node_data("validation")?
            .ok_or_else(|| WorkflowError::validation_error("Missing validation results"))?;
        let analysis: ContentAnalysis = context
            .get_node_data("analysis")?
            .ok_or_else(|| WorkflowError::validation_error("Missing content analysis"))?;
        let processed_content: ProcessedContent = context
            .get_node_data("processed_content")?
            .ok_or_else(|| WorkflowError::validation_error("Missing processed content"))?;
        
        // Calculate overall quality score
        let quality_score = self.calculate_quality_score(&validation, &analysis);
        
        // Determine if content is ready for publication
        let publication_ready = validation.is_valid && quality_score >= 0.6;
        
        // Generate next steps
        let next_steps = self.determine_next_steps(publication_ready, &validation);
        
        // Create final publication result
        let publication_result = PublicationResult {
            content: processed_content,
            analysis,
            validation,
            publication_ready,
            next_steps,
            quality_score,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        
        println!("   📊 Publication package assembled:");
        println!("      Quality score: {:.2}", quality_score);
        println!("      Publication ready: {}", if publication_ready { "✅ Yes" } else { "❌ No" });
        println!("      Next steps: {} items", publication_result.next_steps.len());
        
        context.update_node("publication_result", &publication_result);
        context.set_metadata("publication_ready", publication_ready)?;
        context.set_metadata("quality_score", quality_score)?;
        context.set_metadata("pipeline_completed", true)?;
        
        println!("   ✅ Publication assembly completed");
        
        Ok(context)
    }
}

#[tokio::main]
async fn main() -> Result<(), WorkflowError> {
    println!("🚀 Simple Pipeline Example - Complete Content Processing Workflow");
    println!("=".repeat(70));
    println!("This example demonstrates a real-world content processing pipeline.\n");
    
    // Create the processing pipeline
    println!("📦 Building content processing pipeline...");
    let validation_node = ContentValidationNode::new();
    let analysis_node = ContentAnalysisNode;
    let processing_node = ContentProcessingNode;
    let assembly_node = PublicationAssemblyNode;
    println!("   ✅ Created 4-stage content processing pipeline\n");
    
    // Test content with different characteristics
    let test_content = vec![
        ContentInput {
            title: "the ultimate guide to rust programming".to_string(),
            body: "Rust is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety. It accomplishes these goals by being memory safe without using garbage collection.\n\nRust has great documentation, a friendly compiler with useful error messages, and top-notch tooling — an integrated package manager and build tool, smart multi-editor support with auto-completion and type inspections, an auto-formatter, and more.\n\nIn this comprehensive guide, we'll explore the fundamental concepts of Rust programming, from basic syntax to advanced features like ownership, borrowing, and lifetimes. Whether you're a beginner or an experienced developer, this guide will help you understand why Rust is becoming the language of choice for system programming.".to_string(),
            author: "Jane Developer".to_string(),
            content_type: "tutorial".to_string(),
            tags: Some(vec!["programming".to_string(), "rust".to_string()]),
            metadata: None,
        },
        ContentInput {
            title: "short".to_string(),
            body: "This is too short.".to_string(),
            author: "".to_string(),
            content_type: "unknown".to_string(),
            tags: None,
            metadata: None,
        },
        ContentInput {
            title: "Amazing Business Strategies for Modern Entrepreneurs".to_string(),
            body: "In today's rapidly evolving business landscape, entrepreneurs face unprecedented challenges and opportunities. The traditional business models are being disrupted by technological advances, changing consumer behaviors, and global market dynamics.\n\nSuccessful entrepreneurs understand that adaptability is key to survival and growth. They embrace innovation, leverage technology, and build strong networks to navigate the complex business environment. Market research becomes crucial for understanding customer needs and identifying emerging trends.\n\nDigital transformation has revolutionized how businesses operate, communicate with customers, and deliver value. Companies that fail to adapt to these changes risk becoming obsolete, while those that embrace digital tools and strategies can achieve remarkable growth and success.\n\nBuilding a strong brand presence, implementing effective marketing strategies, and maintaining excellent customer relationships are fundamental aspects of modern business success. Entrepreneurs must also focus on building resilient teams and creating sustainable business practices.".to_string(),
            author: "Business Expert".to_string(),
            content_type: "article".to_string(),
            tags: Some(vec!["business".to_string(), "entrepreneurship".to_string()]),
            metadata: Some({
                let mut map = HashMap::new();
                map.insert("target_audience".to_string(), json!("entrepreneurs"));
                map.insert("difficulty".to_string(), json!("intermediate"));
                map
            }),
        },
    ];
    
    // Process each piece of content through the complete pipeline
    for (i, content) in test_content.into_iter().enumerate() {
        println!("🔄 Processing Content {} of 3", i + 1);
        println!("   📝 Title: {}", content.title);
        println!("   👤 Author: {}", content.author);
        println!("   📊 Type: {}", content.content_type);
        println!("   📏 Length: {} characters", content.body.len());
        
        // Create workflow context
        let mut context = TaskContext::new(
            "content_processing_pipeline".to_string(),
            serde_json::to_value(&content)?
        );
        
        // Execute the complete processing pipeline
        println!("   🔄 Stage 1: Content Validation");
        context = validation_node.process(context)?;
        
        println!("   🔄 Stage 2: Content Analysis");
        context = analysis_node.process(context)?;
        
        println!("   🔄 Stage 3: Content Processing");
        context = processing_node.process(context)?;
        
        println!("   🔄 Stage 4: Publication Assembly");
        context = assembly_node.process(context)?;
        
        // Display final results
        if let Some(result) = context.get_node_data::<PublicationResult>("publication_result")? {
            println!("   📋 PIPELINE RESULTS:");
            println!("      📊 Quality Score: {:.2}/1.0", result.quality_score);
            println!("      📈 Publication Ready: {}", 
                if result.publication_ready { "✅ YES" } else { "❌ NO" });
            
            println!("      📝 Processed Title: {}", result.content.formatted_title);
            println!("      🏷️  Category: {}", result.content.category);
            println!("      🔗 Slug: {}", result.content.slug);
            println!("      🏷️  Tags: {}", result.content.generated_tags.join(", "));
            
            println!("      📊 Analysis Summary:");
            println!("         Words: {}", result.analysis.word_count);
            println!("         Reading time: {:.1} min", result.analysis.estimated_reading_time);
            println!("         Readability: {:.2}", result.analysis.readability_score);
            println!("         Sentiment: {:.2}", result.analysis.sentiment_score);
            
            if !result.validation.is_valid {
                println!("      ❌ Validation Issues:");
                for issue in &result.validation.issues {
                    println!("         • {}", issue);
                }
            }
            
            if !result.validation.warnings.is_empty() {
                println!("      ⚠️  Warnings:");
                for warning in &result.validation.warnings {
                    println!("         • {}", warning);
                }
            }
            
            println!("      📋 Next Steps:");
            for (i, step) in result.next_steps.iter().enumerate() {
                println!("         {}. {}", i + 1, step);
            }
        }
        
        // Show pipeline metadata
        println!("   🔍 Pipeline Metadata:");
        for (key, value) in context.get_all_metadata() {
            println!("      {}: {}", key, value);
        }
        
        println!("   ✅ Content processing completed\n");
    }
    
    println!("🎉 Simple Pipeline Example Complete!");
    println!("=".repeat(70));
    println!("🎓 What you learned:");
    println!("   • Building multi-stage processing pipelines");
    println!("   • Real-world content processing patterns");
    println!("   • Error handling and validation in workflows");
    println!("   • Data transformation through pipeline stages");
    println!("   • Quality scoring and conditional logic");
    println!("   • Creating publication-ready output packages");
    println!();
    println!("🏗️  Pipeline Architecture:");
    println!("   1. Validation → Quality checks and requirement verification");
    println!("   2. Analysis → Content analysis and metadata extraction");
    println!("   3. Processing → Formatting and tag generation");
    println!("   4. Assembly → Final package creation and quality scoring");
    println!();
    println!("➡️  Next steps:");
    println!("   • Try creating your own processing stages");
    println!("   • Experiment with different validation rules");
    println!("   • Add conditional branching based on content type");
    println!("   • Move on to the 02-core-concepts examples");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_content() -> ContentInput {
        ContentInput {
            title: "Test Article Title".to_string(),
            body: "This is a test article body with enough content to pass validation. It contains multiple sentences and provides meaningful information for testing purposes. The content is structured in a way that should pass most validation checks and provide good analysis results.".to_string(),
            author: "Test Author".to_string(),
            content_type: "article".to_string(),
            tags: Some(vec!["test".to_string()]),
            metadata: None,
        }
    }
    
    #[test]
    fn test_content_validation_node() {
        let node = ContentValidationNode::new();
        let content = create_test_content();
        let context = TaskContext::new(
            "test".to_string(),
            serde_json::to_value(content).unwrap()
        );
        
        let result = node.process(context).unwrap();
        let validation: ValidationResult = result.get_node_data("validation").unwrap().unwrap();
        
        assert!(validation.is_valid);
        assert!(validation.score > 0.5);
    }
    
    #[test]
    fn test_content_analysis_node() {
        let node = ContentAnalysisNode;
        let content = create_test_content();
        let context = TaskContext::new(
            "test".to_string(),
            serde_json::to_value(content).unwrap()
        );
        
        let result = node.process(context).unwrap();
        let analysis: ContentAnalysis = result.get_node_data("analysis").unwrap().unwrap();
        
        assert!(analysis.word_count > 0);
        assert!(analysis.estimated_reading_time > 0.0);
        assert!(!analysis.keywords.is_empty());
        assert_eq!(analysis.language, "english");
    }
    
    #[test]
    fn test_complete_pipeline() {
        let validation_node = ContentValidationNode::new();
        let analysis_node = ContentAnalysisNode;
        let processing_node = ContentProcessingNode;
        let assembly_node = PublicationAssemblyNode;
        let content = create_test_content();
        
        let mut context = TaskContext::new(
            "test".to_string(),
            serde_json::to_value(content).unwrap()
        );
        
        // Execute complete pipeline
        context = validation_node.process(context).unwrap();
        context = analysis_node.process(context).unwrap();
        context = processing_node.process(context).unwrap();
        context = assembly_node.process(context).unwrap();
        
        // Verify final result
        let result: PublicationResult = context.get_node_data("publication_result").unwrap().unwrap();
        
        assert!(result.publication_ready);
        assert!(result.quality_score > 0.0);
        assert!(!result.next_steps.is_empty());
        assert!(!result.content.generated_tags.is_empty());
        
        // Verify pipeline completion metadata
        assert_eq!(
            context.get_metadata::<bool>("pipeline_completed").unwrap().unwrap(),
            true
        );
    }
}