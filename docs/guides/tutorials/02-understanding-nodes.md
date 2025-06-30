# Tutorial 2: Understanding Nodes and Data Flow

Welcome back! In the previous tutorial, you built your first AI workflow. Now it's time to dive deeper into the Node system - the heart of the workflow engine. You'll learn how to create sophisticated nodes, handle complex data flows, and build robust processing pipelines.

## Nodes: The Building Blocks of Intelligence

Think of nodes as specialized workers in a smart factory. Each worker has:
- **A specific skill** (what it does)
- **Input requirements** (what it needs to work)
- **Output format** (what it produces)
- **Error handling** (what to do when things go wrong)

The magic happens when these workers collaborate, passing information between each other to accomplish complex tasks.

## The Node Trait: Your Contract for Processing

Every node must implement the `Node` trait, which is like a job description that defines how the node behaves:

```rust
use backend::core::nodes::Node;
use backend::core::task::TaskContext;
use backend::core::error::WorkflowError;

pub trait Node: Send + Sync + Debug {
    // Optional: Give your node a human-readable name
    fn node_name(&self) -> String {
        // Default implementation extracts the type name
        std::any::type_name::<Self>()
            .split("::")
            .last()
            .unwrap_or("UnknownNode")
            .to_string()
    }

    // Required: The core processing method
    fn process(&self, task_context: TaskContext) -> Result<TaskContext, WorkflowError>;
}
```

## Building a Multi-Stage Document Processing Pipeline

Let's build a practical example: a document processing pipeline that takes raw text, cleans it, analyzes it, and generates a summary. This will show you key patterns you'll use in real applications.

### Node 1: Document Validation and Preprocessing

```rust
use backend::core::nodes::Node;
use backend::core::task::TaskContext;
use backend::core::error::WorkflowError;
use serde_json::json;
use serde::{Deserialize, Serialize};
use regex::Regex;

#[derive(Debug, Deserialize, Serialize)]
struct DocumentInput {
    title: String,
    content: String,
    author: Option<String>,
    document_type: String,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug)]
struct DocumentPreprocessorNode {
    max_content_length: usize,
    allowed_types: Vec<String>,
}

impl DocumentPreprocessorNode {
    fn new() -> Self {
        Self {
            max_content_length: 100_000, // 100KB limit
            allowed_types: vec![
                "article".to_string(),
                "report".to_string(), 
                "memo".to_string(),
                "email".to_string(),
            ],
        }
    }

    fn clean_text(&self, text: &str) -> String {
        // Remove excessive whitespace
        let whitespace_regex = Regex::new(r"\s+").unwrap();
        let cleaned = whitespace_regex.replace_all(text, " ");
        
        // Remove non-printable characters except newlines and tabs
        let printable_regex = Regex::new(r"[^\x20-\x7E\n\t]").unwrap();
        let cleaned = printable_regex.replace_all(&cleaned, "");
        
        // Trim and normalize
        cleaned.trim().to_string()
    }

    fn extract_metadata(&self, document: &DocumentInput) -> serde_json::Value {
        json!({
            "word_count": document.content.split_whitespace().count(),
            "character_count": document.content.len(),
            "estimated_reading_time_minutes": document.content.split_whitespace().count() / 200, // ~200 WPM
            "has_author": document.author.is_some(),
            "content_preview": document.content.chars().take(100).collect::<String>()
        })
    }
}

impl Node for DocumentPreprocessorNode {
    fn node_name(&self) -> String {
        "Document Preprocessor".to_string()
    }

    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("📄 Step 1: Preprocessing document...");
        
        // Extract and validate input
        let document: DocumentInput = context.get_event_data()?;
        
        // Validation checks
        let mut validation_issues = Vec::new();
        
        if document.title.trim().is_empty() {
            validation_issues.push("Document title is required".to_string());
        }
        
        if document.content.trim().is_empty() {
            validation_issues.push("Document content cannot be empty".to_string());
        }
        
        if document.content.len() > self.max_content_length {
            validation_issues.push(format!(
                "Content too long: {} chars (max: {})", 
                document.content.len(), 
                self.max_content_length
            ));
        }
        
        if !self.allowed_types.contains(&document.document_type.to_lowercase()) {
            validation_issues.push(format!(
                "Unsupported document type: '{}'. Allowed: {:?}",
                document.document_type,
                self.allowed_types
            ));
        }
        
        let is_valid = validation_issues.is_empty();
        
        if !is_valid {
            // Store validation errors but continue with cleaned data
            context.update_node("preprocessing", json!({
                "valid": false,
                "issues": validation_issues,
                "processed_at": chrono::Utc::now()
            }));
            
            println!("   ⚠️ Validation issues found: {:?}", validation_issues);
            return Ok(context);
        }
        
        // Clean and process the document
        let cleaned_title = self.clean_text(&document.title);
        let cleaned_content = self.clean_text(&document.content);
        let extracted_metadata = self.extract_metadata(&document);
        
        // Store preprocessing results
        context.update_node("preprocessing", json!({
            "valid": true,
            "original_title": document.title,
            "cleaned_title": cleaned_title,
            "original_content_length": document.content.len(),
            "cleaned_content_length": cleaned_content.len(),
            "document_type": document.document_type,
            "author": document.author,
            "metadata": extracted_metadata,
            "processed_at": chrono::Utc::now()
        }));
        
        // Store the cleaned document for next nodes
        context.update_node("cleaned_document", json!({
            "title": cleaned_title,
            "content": cleaned_content,
            "document_type": document.document_type,
            "author": document.author,
            "original_metadata": document.metadata
        }));
        
        // Add processing metadata
        context.set_metadata("preprocessing_completed", true)?;
        context.set_metadata("content_changes", document.content.len() != cleaned_content.len())?;
        
        println!("   ✅ Document preprocessed: {} -> {} chars", 
                 document.content.len(), cleaned_content.len());
        
        Ok(context)
    }
}
```

### Node 2: Content Analysis and Feature Extraction

```rust
#[derive(Debug)]
struct ContentAnalyzerNode {
    min_sentence_length: usize,
    keyword_extraction_enabled: bool,
}

impl ContentAnalyzerNode {
    fn new() -> Self {
        Self {
            min_sentence_length: 10,
            keyword_extraction_enabled: true,
        }
    }

    fn analyze_structure(&self, content: &str) -> serde_json::Value {
        let paragraphs: Vec<&str> = content.split('\n').filter(|p| !p.trim().is_empty()).collect();
        let sentences: Vec<&str> = content.split('.').filter(|s| s.trim().len() > self.min_sentence_length).collect();
        
        // Simple complexity analysis
        let avg_words_per_sentence = if !sentences.is_empty() {
            content.split_whitespace().count() as f64 / sentences.len() as f64
        } else {
            0.0
        };
        
        json!({
            "paragraph_count": paragraphs.len(),
            "sentence_count": sentences.len(),
            "average_words_per_sentence": avg_words_per_sentence,
            "readability_score": self.calculate_readability_score(content),
            "structure_complexity": if avg_words_per_sentence > 20.0 { "high" } 
                                   else if avg_words_per_sentence > 15.0 { "medium" } 
                                   else { "low" }
        })
    }

    fn calculate_readability_score(&self, content: &str) -> f64 {
        // Simplified Flesch Reading Ease approximation
        let words = content.split_whitespace().count() as f64;
        let sentences = content.matches('.').count() as f64;
        let syllables = self.estimate_syllables(content) as f64;
        
        if sentences == 0.0 || words == 0.0 {
            return 0.0;
        }
        
        206.835 - (1.015 * (words / sentences)) - (84.6 * (syllables / words))
    }

    fn estimate_syllables(&self, content: &str) -> usize {
        // Simple syllable estimation based on vowel groups
        let vowel_groups = Regex::new(r"[aeiouyAEIOUY]+").unwrap();
        content.split_whitespace()
            .map(|word| vowel_groups.find_iter(word).count().max(1))
            .sum()
    }

    fn extract_keywords(&self, content: &str) -> Vec<String> {
        if !self.keyword_extraction_enabled {
            return Vec::new();
        }

        // Simple keyword extraction based on word frequency
        let stop_words = ["the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by"];
        let words: Vec<&str> = content.to_lowercase()
            .split_whitespace()
            .filter(|word| word.len() > 3 && !stop_words.contains(word))
            .collect();

        let mut word_count = std::collections::HashMap::new();
        for word in words {
            *word_count.entry(word).or_insert(0) += 1;
        }

        let mut sorted_words: Vec<_> = word_count.into_iter().collect();
        sorted_words.sort_by(|a, b| b.1.cmp(&a.1));
        
        sorted_words.into_iter()
            .take(10) // Top 10 keywords
            .map(|(word, _)| word.to_string())
            .collect()
    }

    fn detect_content_type(&self, content: &str) -> String {
        let content_lower = content.to_lowercase();
        
        if content_lower.contains("conclusion") && content_lower.contains("introduction") {
            "academic_paper"
        } else if content_lower.contains("dear") || content_lower.contains("sincerely") {
            "correspondence"
        } else if content_lower.contains("agenda") || content_lower.contains("meeting") {
            "meeting_notes"
        } else if content_lower.contains("policy") || content_lower.contains("procedure") {
            "documentation"
        } else {
            "general_content"
        }
    }
}

impl Node for ContentAnalyzerNode {
    fn node_name(&self) -> String {
        "Content Analyzer".to_string()
    }

    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("🔍 Step 2: Analyzing content structure and features...");
        
        // Check if preprocessing was successful
        let preprocessing_data = context.get_node_data::<serde_json::Value>("preprocessing")?
            .ok_or_else(|| WorkflowError::ProcessingError {
                message: "No preprocessing data found - ensure DocumentPreprocessorNode ran first".to_string()
            })?;
        
        let is_valid = preprocessing_data.get("valid")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        if !is_valid {
            // Skip analysis for invalid documents
            context.update_node("content_analysis", json!({
                "analyzed": false,
                "reason": "Document failed preprocessing validation",
                "timestamp": chrono::Utc::now()
            }));
            return Ok(context);
        }
        
        // Get the cleaned document
        let cleaned_doc = context.get_node_data::<serde_json::Value>("cleaned_document")?
            .ok_or_else(|| WorkflowError::ProcessingError {
                message: "No cleaned document data found".to_string()
            })?;
        
        let content = cleaned_doc.get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        let title = cleaned_doc.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        // Perform content analysis
        let structure_analysis = self.analyze_structure(content);
        let keywords = self.extract_keywords(content);
        let detected_type = self.detect_content_type(content);
        
        // Store comprehensive analysis results
        context.update_node("content_analysis", json!({
            "analyzed": true,
            "content_type_detected": detected_type,
            "structure": structure_analysis,
            "keywords": keywords,
            "title_analysis": {
                "title_length": title.len(),
                "title_word_count": title.split_whitespace().count(),
                "contains_numbers": title.chars().any(|c| c.is_numeric()),
                "contains_special_chars": title.chars().any(|c| !c.is_alphanumeric() && !c.is_whitespace())
            },
            "language_features": {
                "estimated_language": "english", // In production, use language detection
                "complexity_indicators": {
                    "long_sentences": structure_analysis.get("average_words_per_sentence")
                        .and_then(|v| v.as_f64())
                        .map(|avg| avg > 25.0)
                        .unwrap_or(false),
                    "technical_terms": keywords.iter().any(|k| k.len() > 10),
                    "formal_tone": content.contains("therefore") || content.contains("furthermore")
                }
            },
            "analysis_timestamp": chrono::Utc::now(),
            "processing_stats": {
                "keywords_found": keywords.len(),
                "readability_score": structure_analysis.get("readability_score")
            }
        }));
        
        // Add metadata for next nodes
        context.set_metadata("analysis_completed", true)?;
        context.set_metadata("content_complexity", structure_analysis.get("structure_complexity"))?;
        context.set_metadata("keywords_count", keywords.len())?;
        
        println!("   📊 Analysis complete: {} type, {} keywords, complexity: {}", 
                 detected_type, 
                 keywords.len(),
                 structure_analysis.get("structure_complexity").and_then(|v| v.as_str()).unwrap_or("unknown"));
        
        Ok(context)
    }
}
```

### Node 3: Intelligent Summary Generation

```rust
#[derive(Debug)]
struct SummaryGeneratorNode {
    target_summary_length: usize,
    include_key_quotes: bool,
    summary_style: String,
}

impl SummaryGeneratorNode {
    fn new() -> Self {
        Self {
            target_summary_length: 500, // Target 500 characters
            include_key_quotes: true,
            summary_style: "executive".to_string(), // executive, academic, casual
        }
    }

    fn extract_key_sentences(&self, content: &str, keywords: &[String]) -> Vec<String> {
        let sentences: Vec<&str> = content.split('.')
            .filter(|s| s.trim().len() > 20)
            .collect();
        
        let mut scored_sentences = Vec::new();
        
        for sentence in sentences {
            let mut score = 0.0;
            
            // Score based on keyword presence
            for keyword in keywords {
                if sentence.to_lowercase().contains(&keyword.to_lowercase()) {
                    score += 2.0;
                }
            }
            
            // Score based on position (first and last sentences are often important)
            if scored_sentences.len() < 2 {
                score += 1.0; // First sentence bonus
            }
            
            // Score based on sentence length (not too short, not too long)
            let word_count = sentence.split_whitespace().count();
            if word_count >= 8 && word_count <= 30 {
                score += 1.0;
            }
            
            // Score based on certain phrases
            if sentence.contains("important") || sentence.contains("significant") || 
               sentence.contains("concluded") || sentence.contains("findings") {
                score += 1.5;
            }
            
            scored_sentences.push((sentence.trim().to_string(), score));
        }
        
        // Sort by score and take top sentences
        scored_sentences.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scored_sentences.into_iter()
            .take(5)
            .map(|(sentence, _)| sentence)
            .collect()
    }

    fn generate_summary(&self, content: &str, keywords: &[String], analysis: &serde_json::Value) -> String {
        let key_sentences = self.extract_key_sentences(content, keywords);
        
        // Extract some metrics from analysis
        let word_count = analysis.get("structure")
            .and_then(|s| s.get("sentence_count"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        
        let complexity = analysis.get("structure")
            .and_then(|s| s.get("structure_complexity"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        
        let detected_type = analysis.get("content_type_detected")
            .and_then(|v| v.as_str())
            .unwrap_or("general_content");
        
        // Build summary based on style
        match self.summary_style.as_str() {
            "executive" => {
                let mut summary = format!(
                    "EXECUTIVE SUMMARY: This {} document contains {} sentences with {} complexity. ",
                    detected_type, word_count, complexity
                );
                
                if !keywords.is_empty() {
                    summary.push_str(&format!("Key topics include: {}. ", keywords[..3.min(keywords.len())].join(", ")));
                }
                
                if !key_sentences.is_empty() {
                    summary.push_str("Key insights: ");
                    summary.push_str(&key_sentences[0]);
                    if key_sentences.len() > 1 {
                        summary.push_str(" Additionally, ");
                        summary.push_str(&key_sentences[1]);
                    }
                }
                
                summary
            },
            "academic" => {
                let mut summary = format!(
                    "This document presents a {} with {} complexity structure. ",
                    detected_type, complexity
                );
                
                if !key_sentences.is_empty() {
                    summary.push_str("The main findings suggest: ");
                    for (i, sentence) in key_sentences.iter().take(3).enumerate() {
                        if i > 0 { summary.push_str(" Furthermore, "); }
                        summary.push_str(sentence);
                    }
                }
                
                summary
            },
            _ => {
                // Casual style
                let mut summary = format!("This is a {} that's {} to read. ", detected_type, 
                    if complexity == "high" { "challenging" } else { "easy" });
                
                if !keywords.is_empty() {
                    summary.push_str(&format!("It talks about {}. ", keywords[..2.min(keywords.len())].join(" and ")));
                }
                
                if !key_sentences.is_empty() {
                    summary.push_str("The main point is: ");
                    summary.push_str(&key_sentences[0]);
                }
                
                summary
            }
        }
    }

    fn extract_action_items(&self, content: &str) -> Vec<String> {
        let action_patterns = [
            r"(?i)should\s+([^.]+)",
            r"(?i)must\s+([^.]+)", 
            r"(?i)recommend\s+([^.]+)",
            r"(?i)action\s+item[s]?:\s*([^.]+)",
            r"(?i)next\s+steps?\s*:?\s*([^.]+)"
        ];
        
        let mut actions = Vec::new();
        
        for pattern in &action_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                for cap in regex.captures_iter(content) {
                    if let Some(action) = cap.get(1) {
                        let action_text = action.as_str().trim();
                        if action_text.len() > 5 && action_text.len() < 200 {
                            actions.push(action_text.to_string());
                        }
                    }
                }
            }
        }
        
        actions.into_iter().take(5).collect() // Limit to 5 action items
    }
}

impl Node for SummaryGeneratorNode {
    fn node_name(&self) -> String {
        "Summary Generator".to_string()
    }

    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("📝 Step 3: Generating intelligent summary...");
        
        // Check if previous analysis was successful
        let analysis_data = context.get_node_data::<serde_json::Value>("content_analysis")?
            .ok_or_else(|| WorkflowError::ProcessingError {
                message: "No content analysis found - ensure ContentAnalyzerNode ran first".to_string()
            })?;
        
        let analyzed = analysis_data.get("analyzed")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        if !analyzed {
            context.update_node("summary", json!({
                "generated": false,
                "reason": "Content analysis was not successful",
                "timestamp": chrono::Utc::now()
            }));
            return Ok(context);
        }
        
        // Get the cleaned document and analysis results
        let cleaned_doc = context.get_node_data::<serde_json::Value>("cleaned_document")?
            .ok_or_else(|| WorkflowError::ProcessingError {
                message: "No cleaned document found".to_string()
            })?;
        
        let content = cleaned_doc.get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        let title = cleaned_doc.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled Document");
        
        let keywords = analysis_data.get("keywords")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_else(Vec::new);
        
        // Generate the summary
        let summary_text = self.generate_summary(content, &keywords, &analysis_data);
        let action_items = self.extract_action_items(content);
        
        // Calculate summary statistics
        let compression_ratio = if !content.is_empty() {
            summary_text.len() as f64 / content.len() as f64
        } else {
            0.0
        };
        
        // Store comprehensive summary results
        context.update_node("summary", json!({
            "generated": true,
            "document_title": title,
            "summary_text": summary_text,
            "summary_style": self.summary_style,
            "action_items": action_items,
            "statistics": {
                "original_length": content.len(),
                "summary_length": summary_text.len(),
                "compression_ratio": compression_ratio,
                "action_items_found": action_items.len(),
                "keywords_used": keywords.len()
            },
            "metadata": {
                "target_length": self.target_summary_length,
                "actual_length": summary_text.len(),
                "length_within_target": summary_text.len() <= self.target_summary_length * 2,
                "includes_action_items": !action_items.is_empty()
            },
            "generated_at": chrono::Utc::now()
        }));
        
        // Add final metadata
        context.set_metadata("summary_generated", true)?;
        context.set_metadata("compression_ratio", compression_ratio)?;
        context.set_metadata("pipeline_completed", true)?;
        
        println!("   ✅ Summary generated: {:.1}% compression, {} action items", 
                 compression_ratio * 100.0, action_items.len());
        
        Ok(context)
    }
}
```

### Building and Running the Complete Pipeline

Now let's create a main function that demonstrates the complete pipeline:

```rust
use backend::core::task::TaskContext;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏭 Document Processing Pipeline");
    println!("===============================\n");
    
    // Create our processing nodes
    let preprocessor = DocumentPreprocessorNode::new();
    let analyzer = ContentAnalyzerNode::new();
    let summarizer = SummaryGeneratorNode::new();
    
    // Test documents with different characteristics
    let test_documents = vec![
        json!({
            "title": "Quarterly Sales Report",
            "content": "This report provides a comprehensive analysis of our quarterly sales performance. During Q3, we achieved significant growth in key market segments. The automotive division exceeded expectations with a 25% increase in revenue. Our analysis indicates that customer satisfaction scores improved by 15 points. Key recommendations include expanding our digital marketing efforts and investing in customer service training. Action items: 1) Develop Q4 marketing strategy, 2) Hire additional customer service representatives, 3) Implement new CRM system by December.",
            "author": "Sales Team",
            "document_type": "report",
            "metadata": {"department": "sales", "quarter": "Q3"}
        }),
        json!({
            "title": "Meeting Notes - Project Alpha",
            "content": "Agenda: 1) Project status update, 2) Budget review, 3) Timeline adjustments. The project is currently 80% complete and on track for December delivery. Budget utilization is at 75% with remaining funds allocated for final testing phase. Team reported some integration challenges with the new API. Decision made to extend testing period by two weeks. Action items assigned to team leads. Next meeting scheduled for next Friday.",
            "document_type": "memo"
        }),
        json!({
            "title": "Research Paper: Machine Learning Applications",
            "content": "Abstract: This paper explores the applications of machine learning in modern business environments. Introduction: Machine learning has revolutionized data analysis and decision-making processes. Methodology: We conducted a comprehensive survey of 500 companies across various industries. Results: 78% of companies reported improved efficiency after implementing ML solutions. The most significant improvements were observed in predictive analytics and customer segmentation. Conclusion: Machine learning provides substantial benefits when properly implemented. Organizations should invest in training and infrastructure to maximize returns.",
            "author": "Dr. Jane Smith",
            "document_type": "article",
            "metadata": {"journal": "Business Technology Review", "year": 2024}
        })
    ];
    
    // Process each document through the pipeline
    for (index, document_data) in test_documents.iter().enumerate() {
        println!("🔄 Processing Document #{}: {}", 
                 index + 1, 
                 document_data.get("title").and_then(|v| v.as_str()).unwrap_or("Untitled"));
        println!("─".repeat(60));
        
        // Create task context for this document
        let mut context = TaskContext::new(
            "document_processing_pipeline".to_string(),
            document_data.clone()
        );
        
        // Execute the pipeline: Preprocessing → Analysis → Summarization
        println!("Starting pipeline execution...\n");
        
        // Step 1: Preprocessing
        context = preprocessor.process(context)?;
        
        // Step 2: Content Analysis
        context = analyzer.process(context)?;
        
        // Step 3: Summary Generation
        context = summarizer.process(context)?;
        
        // Display comprehensive results
        println!("\n📊 Pipeline Results:");
        println!("─".repeat(40));
        
        // Show preprocessing results
        if let Some(preprocessing) = context.get_node_data::<serde_json::Value>("preprocessing")? {
            if let Some(metadata) = preprocessing.get("metadata") {
                println!("📄 Document Info:");
                println!("   Words: {}", metadata.get("word_count").and_then(|v| v.as_u64()).unwrap_or(0));
                println!("   Reading time: {} min", metadata.get("estimated_reading_time_minutes").and_then(|v| v.as_u64()).unwrap_or(0));
            }
        }
        
        // Show analysis results
        if let Some(analysis) = context.get_node_data::<serde_json::Value>("content_analysis")? {
            if let Some(keywords) = analysis.get("keywords").and_then(|v| v.as_array()) {
                let keyword_strings: Vec<String> = keywords.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                println!("🔍 Analysis:");
                println!("   Type: {}", analysis.get("content_type_detected").and_then(|v| v.as_str()).unwrap_or("unknown"));
                println!("   Keywords: {}", keyword_strings.join(", "));
                
                if let Some(structure) = analysis.get("structure") {
                    println!("   Complexity: {}", structure.get("structure_complexity").and_then(|v| v.as_str()).unwrap_or("unknown"));
                    println!("   Readability: {:.1}", structure.get("readability_score").and_then(|v| v.as_f64()).unwrap_or(0.0));
                }
            }
        }
        
        // Show summary results
        if let Some(summary) = context.get_node_data::<serde_json::Value>("summary")? {
            if let Some(summary_text) = summary.get("summary_text").and_then(|v| v.as_str()) {
                println!("\n📝 Generated Summary:");
                println!("   {}", summary_text);
                
                if let Some(action_items) = summary.get("action_items").and_then(|v| v.as_array()) {
                    if !action_items.is_empty() {
                        println!("\n✅ Action Items:");
                        for (i, item) in action_items.iter().enumerate() {
                            if let Some(item_text) = item.as_str() {
                                println!("   {}. {}", i + 1, item_text);
                            }
                        }
                    }
                }
                
                if let Some(stats) = summary.get("statistics") {
                    println!("\n📈 Compression: {:.1}%", 
                             stats.get("compression_ratio").and_then(|v| v.as_f64()).unwrap_or(0.0) * 100.0);
                }
            }
        }
        
        println!("\n");
    }
    
    println!("✨ Document processing pipeline demonstration completed!");
    Ok(())
}
```

## Key Patterns You've Learned

### 1. Progressive Data Enrichment
Each node adds more information to the context:
```rust
// Node 1 adds: cleaned data + validation
// Node 2 adds: analysis + keywords + structure
// Node 3 adds: summary + action items + statistics
```

### 2. Conditional Processing
Nodes can skip processing based on previous results:
```rust
let is_valid = validation_data.get("valid").and_then(|v| v.as_bool()).unwrap_or(false);
if !is_valid {
    // Skip processing and store reason
    context.update_node("analysis", json!({"skipped": true, "reason": "Invalid input"}));
    return Ok(context);
}
```

### 3. Error Recovery and Fallbacks
Handle missing data gracefully:
```rust
let keywords = analysis_data.get("keywords")
    .and_then(|v| v.as_array())
    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
    .unwrap_or_else(Vec::new); // Fallback to empty vec
```

### 4. Rich Metadata Tracking
Use metadata for debugging and monitoring:
```rust
context.set_metadata("processing_duration", duration.as_millis())?;
context.set_metadata("complexity_level", complexity)?;
context.set_metadata("pipeline_stage", "analysis_complete")?;
```

### 5. Structured Result Storage
Store results with consistent structure:
```rust
context.update_node("analysis_results", json!({
    "success": true,
    "data": processed_data,
    "metadata": {
        "processed_at": chrono::Utc::now(),
        "processing_version": "1.0",
        "confidence": confidence_score
    },
    "statistics": performance_stats
}));
```

## Advanced Node Patterns

### Router Nodes for Conditional Logic
```rust
#[derive(Debug)]
struct DocumentTypeRouter;

impl Node for DocumentTypeRouter {
    fn process(&self, context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let doc_type = /* extract document type */;
        
        // Set routing metadata for workflow engine
        context.set_metadata("next_node_type", match doc_type {
            "academic" => "AcademicProcessor",
            "business" => "BusinessProcessor", 
            "legal" => "LegalProcessor",
            _ => "GenericProcessor"
        })?;
        
        Ok(context)
    }
}
```

### Aggregator Nodes for Parallel Results
```rust
#[derive(Debug)]
struct ResultAggregatorNode;

impl Node for ResultAggregatorNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Collect results from parallel processing nodes
        let sentiment_result = context.get_node_data::<serde_json::Value>("sentiment_analysis")?;
        let keyword_result = context.get_node_data::<serde_json::Value>("keyword_extraction")?;
        let topic_result = context.get_node_data::<serde_json::Value>("topic_modeling")?;
        
        // Combine and reconcile results
        let combined_analysis = json!({
            "sentiment": sentiment_result,
            "keywords": keyword_result, 
            "topics": topic_result,
            "confidence": self.calculate_combined_confidence(&[sentiment_result, keyword_result, topic_result])
        });
        
        context.update_node("aggregated_analysis", combined_analysis);
        Ok(context)
    }
}
```

## Testing Your Nodes

### Unit Testing Individual Nodes
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_document_preprocessor_validation() {
        let node = DocumentPreprocessorNode::new();
        let context = TaskContext::new(
            "test".to_string(),
            json!({
                "title": "",  // Invalid: empty title
                "content": "Valid content here",
                "document_type": "report"
            })
        );
        
        let result = node.process(context).unwrap();
        let preprocessing = result.get_node_data::<serde_json::Value>("preprocessing").unwrap().unwrap();
        
        assert_eq!(preprocessing.get("valid").unwrap(), false);
        assert!(preprocessing.get("issues").unwrap().as_array().unwrap().len() > 0);
    }
    
    #[test]
    fn test_content_analyzer_keyword_extraction() {
        let node = ContentAnalyzerNode::new();
        // Create context with preprocessed data
        let mut context = TaskContext::new("test".to_string(), json!({}));
        context.update_node("preprocessing", json!({"valid": true}));
        context.update_node("cleaned_document", json!({
            "content": "Machine learning and artificial intelligence are transforming business operations",
            "title": "AI in Business"
        }));
        
        let result = node.process(context).unwrap();
        let analysis = result.get_node_data::<serde_json::Value>("content_analysis").unwrap().unwrap();
        
        let keywords = analysis.get("keywords").unwrap().as_array().unwrap();
        assert!(keywords.len() > 0);
        assert!(keywords.iter().any(|k| k.as_str().unwrap().contains("machine")));
    }
}
```

### Integration Testing Complete Pipelines
```rust
#[tokio::test]
async fn test_complete_document_pipeline() {
    let preprocessor = DocumentPreprocessorNode::new();
    let analyzer = ContentAnalyzerNode::new();
    let summarizer = SummaryGeneratorNode::new();
    
    let mut context = TaskContext::new(
        "integration_test".to_string(),
        json!({
            "title": "Test Document",
            "content": "This is a test document with sufficient content for analysis. It contains multiple sentences and should produce meaningful results.",
            "document_type": "article"
        })
    );
    
    // Execute pipeline
    context = preprocessor.process(context).unwrap();
    context = analyzer.process(context).unwrap();
    context = summarizer.process(context).unwrap();
    
    // Verify final results
    let summary = context.get_node_data::<serde_json::Value>("summary").unwrap().unwrap();
    assert_eq!(summary.get("generated").unwrap(), true);
    assert!(summary.get("summary_text").unwrap().as_str().unwrap().len() > 0);
}
```

## Next Steps

Now that you understand nodes and data flow:

1. **Practice**: Build your own document classifier or sentiment analyzer
2. **Experiment**: Try parallel processing with multiple analysis nodes
3. **Integrate**: Connect your nodes to external APIs or databases
4. **Continue**: Move to [Tutorial 3: AI-Powered Automation](./03-ai-powered-automation.md) to add real AI capabilities

## Quick Reference

```rust
// Node implementation template
#[derive(Debug)]
struct MyProcessorNode {
    config_param: String,
}

impl Node for MyProcessorNode {
    fn node_name(&self) -> String {
        "My Processor".to_string()
    }
    
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // 1. Get input data (with validation)
        let input: MyInputType = context.get_event_data()?;
        
        // 2. Get previous node results (if needed)
        let prev_result = context.get_node_data::<serde_json::Value>("previous_node")?;
        
        // 3. Validate and process
        if !self.validate_input(&input) {
            return Err(WorkflowError::ValidationError {
                message: "Invalid input data".to_string()
            });
        }
        
        let result = self.process_data(&input)?;
        
        // 4. Store results
        context.update_node("my_results", json!({
            "success": true,
            "data": result,
            "processed_at": chrono::Utc::now()
        }));
        
        // 5. Add metadata
        context.set_metadata("processing_completed", true)?;
        
        Ok(context)
    }
}

// Pipeline execution
let mut context = TaskContext::new("workflow_type".to_string(), input_data);
context = node1.process(context)?;
context = node2.process(context)?;
context = node3.process(context)?;
```

You now have the foundation to build sophisticated, multi-stage processing pipelines! Each node focuses on a specific task while contributing to the overall intelligence of your workflow.