//! Content difficulty analysis for educational assessment
//!
//! This module analyzes text complexity and determines appropriate difficulty levels
//! based on various linguistic and cognitive factors including:
//! - Vocabulary complexity and frequency
//! - Sentence structure and length
//! - Concept density and abstraction
//! - Required background knowledge

use std::collections::HashSet;

use crate::models::*;

/// Difficulty analyzer for educational content
pub struct DifficultyAnalyzer {
    name: &'static str,
    basic_vocabulary: HashSet<String>,
    academic_vocabulary: HashSet<String>,
    technical_indicators: Vec<String>,
}

impl DifficultyAnalyzer {
    pub fn new() -> Self {
        Self {
            name: "difficulty_analyzer",
            basic_vocabulary: Self::load_basic_vocabulary(),
            academic_vocabulary: Self::load_academic_vocabulary(),
            technical_indicators: Self::load_technical_indicators(),
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Analyze the difficulty level of text content
    pub async fn analyze_difficulty(
        &self,
        text: &str,
        _context: &ProcessingContext,
    ) -> crate::Result<DifficultyAnalysis> {
        let vocabulary_complexity = self.calculate_vocabulary_complexity(text);
        let concept_density = self.calculate_concept_density(text);
        let sentence_complexity = self.calculate_sentence_complexity(text);
        let prerequisite_knowledge = self.identify_prerequisite_knowledge(text);
        let estimated_reading_time = self.estimate_reading_time(text);
        let cognitive_load_score = self.calculate_cognitive_load(text);
        let target_audience = self.determine_target_audience(text);
        
        let overall_level = self.determine_overall_difficulty(
            vocabulary_complexity,
            concept_density,
            sentence_complexity,
            cognitive_load_score,
        );

        Ok(DifficultyAnalysis {
            overall_level,
            vocabulary_complexity,
            concept_density,
            sentence_complexity,
            prerequisite_knowledge,
            estimated_reading_time,
            cognitive_load_score,
            target_audience,
        })
    }

    /// Calculate vocabulary complexity based on word frequency and sophistication
    fn calculate_vocabulary_complexity(&self, text: &str) -> f32 {
        let words: Vec<&str> = text.split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphabetic()))
            .filter(|w| !w.is_empty())
            .collect();

        if words.is_empty() {
            return 0.0;
        }

        let mut complexity_score = 0.0;
        let mut total_words = 0;

        for word in words {
            let word_lower = word.to_lowercase();
            total_words += 1;

            if self.basic_vocabulary.contains(&word_lower) {
                complexity_score += 0.1; // Very easy words
            } else if self.academic_vocabulary.contains(&word_lower) {
                complexity_score += 0.6; // Academic vocabulary
            } else if word.len() > 8 {
                complexity_score += 0.8; // Long words are typically more complex
            } else if word.len() > 6 {
                complexity_score += 0.5; // Medium complexity
            } else {
                complexity_score += 0.3; // Unknown but shorter words
            }

            // Technical terms bonus
            if self.technical_indicators.iter().any(|indicator| word_lower.contains(indicator)) {
                complexity_score += 0.3;
            }

            // Morphological complexity (prefixes, suffixes)
            if word.contains('-') || word.ends_with("tion") || word.ends_with("sion") 
                || word.ends_with("ment") || word.ends_with("ness") {
                complexity_score += 0.2;
            }
        }

        (complexity_score / total_words as f32).min(1.0)
    }

    /// Calculate concept density - how many complex ideas per unit of text
    fn calculate_concept_density(&self, text: &str) -> f32 {
        let sentences = self.split_into_sentences(text);
        if sentences.is_empty() {
            return 0.0;
        }

        let mut total_concepts = 0;
        let abstract_indicators = vec![
            "theory", "concept", "principle", "framework", "methodology",
            "paradigm", "hypothesis", "algorithm", "system", "process",
            "analysis", "synthesis", "evaluation", "implementation",
            "optimization", "integration", "abstraction", "generalization"
        ];

        for sentence in &sentences {
            let sentence_lower = sentence.to_lowercase();
            
            // Count abstract concepts
            for indicator in &abstract_indicators {
                if sentence_lower.contains(indicator) {
                    total_concepts += 1;
                }
            }

            // Count technical jargon
            let words: Vec<&str> = sentence.split_whitespace().collect();
            for word in words {
                if word.len() > 10 || self.technical_indicators.iter()
                    .any(|indicator| word.to_lowercase().contains(indicator)) {
                    total_concepts += 1;
                }
            }

            // Count complex sentence structures
            if sentence_lower.contains("therefore") || sentence_lower.contains("however")
                || sentence_lower.contains("furthermore") || sentence_lower.contains("nevertheless") {
                total_concepts += 1;
            }
        }

        (total_concepts as f32 / sentences.len() as f32).min(1.0)
    }

    /// Calculate sentence complexity based on structure and length
    fn calculate_sentence_complexity(&self, text: &str) -> f32 {
        let sentences = self.split_into_sentences(text);
        if sentences.is_empty() {
            return 0.0;
        }

        let mut total_complexity = 0.0;

        for sentence in &sentences {
            let words = sentence.split_whitespace().count();
            let mut sentence_score = 0.0;

            // Length-based complexity
            sentence_score += match words {
                0..=8 => 0.1,      // Very simple
                9..=15 => 0.3,     // Simple
                16..=25 => 0.5,    // Medium
                26..=35 => 0.7,    // Complex
                _ => 0.9,          // Very complex
            };

            // Structural complexity indicators
            let commas = sentence.matches(',').count();
            let semicolons = sentence.matches(';').count();
            let parentheses = sentence.matches('(').count();
            
            sentence_score += (commas as f32 * 0.05).min(0.2);
            sentence_score += (semicolons as f32 * 0.1).min(0.2);
            sentence_score += (parentheses as f32 * 0.1).min(0.2);

            // Subordinate clauses
            let subordinate_indicators = vec!["because", "although", "while", "since", "whereas", "if"];
            for indicator in subordinate_indicators {
                if sentence.to_lowercase().contains(indicator) {
                    sentence_score += 0.1;
                }
            }

            // Passive voice indicators (simplified)
            if sentence.to_lowercase().contains("was") && sentence.to_lowercase().contains("by") {
                sentence_score += 0.1;
            }

            total_complexity += sentence_score.min(1.0);
        }

        total_complexity / sentences.len() as f32
    }

    /// Identify prerequisite knowledge areas
    fn identify_prerequisite_knowledge(&self, text: &str) -> Vec<String> {
        let mut prerequisites = HashSet::new();
        let text_lower = text.to_lowercase();

        // Domain-specific knowledge areas
        let knowledge_domains = vec![
            ("mathematics", vec!["equation", "formula", "calculation", "algebra", "calculus", "statistics"]),
            ("computer_science", vec!["algorithm", "programming", "software", "data_structure", "database"]),
            ("science", vec!["experiment", "hypothesis", "methodology", "research", "analysis"]),
            ("business", vec!["strategy", "management", "market", "finance", "economics"]),
            ("engineering", vec!["design", "system", "technical", "specification", "implementation"]),
            ("linguistics", vec!["grammar", "syntax", "semantics", "morphology", "phonetics"]),
            ("psychology", vec!["behavior", "cognitive", "mental", "psychological", "brain"]),
            ("philosophy", vec!["ethics", "moral", "logic", "reasoning", "argument"]),
        ];

        for (domain, keywords) in knowledge_domains {
            let matches = keywords.iter().filter(|&keyword| text_lower.contains(keyword)).count();
            if matches >= 2 {
                prerequisites.insert(domain.to_string());
            }
        }

        // Technical skill prerequisites
        if text_lower.contains("code") || text_lower.contains("programming") {
            prerequisites.insert("programming_fundamentals".to_string());
        }

        if text_lower.contains("statistical") || text_lower.contains("probability") {
            prerequisites.insert("statistics_basics".to_string());
        }

        prerequisites.into_iter().collect()
    }

    /// Estimate reading time in minutes
    fn estimate_reading_time(&self, text: &str) -> u32 {
        let words = text.split_whitespace().count();
        let difficulty_factor = self.calculate_vocabulary_complexity(text);
        
        // Base reading speed: 200-250 words per minute for average adult
        // Adjust based on difficulty
        let adjusted_speed = 225.0 * (1.0 - difficulty_factor * 0.5);
        
        ((words as f32 / adjusted_speed) * 60.0).ceil() as u32
    }

    /// Calculate cognitive load score
    fn calculate_cognitive_load(&self, text: &str) -> f32 {
        let vocabulary_load = self.calculate_vocabulary_complexity(text);
        let concept_load = self.calculate_concept_density(text);
        let structure_load = self.calculate_sentence_complexity(text);
        
        // Information density
        let words_per_sentence = text.split_whitespace().count() as f32 / 
                                self.split_into_sentences(text).len().max(1) as f32;
        let density_load = (words_per_sentence / 20.0).min(1.0);
        
        // Abstract reasoning requirements
        let abstract_indicators = ["theory", "concept", "abstract", "hypothetical", "theoretical"];
        let abstract_count = abstract_indicators.iter()
            .filter(|&indicator| text.to_lowercase().contains(indicator))
            .count();
        let abstract_load = (abstract_count as f32 / 10.0).min(1.0);

        // Weighted combination
        (vocabulary_load * 0.3 + concept_load * 0.25 + structure_load * 0.2 + 
         density_load * 0.15 + abstract_load * 0.1).min(1.0)
    }

    /// Determine target audience based on complexity analysis
    fn determine_target_audience(&self, text: &str) -> Vec<String> {
        let vocabulary_complexity = self.calculate_vocabulary_complexity(text);
        let concept_density = self.calculate_concept_density(text);
        let cognitive_load = self.calculate_cognitive_load(text);
        
        let overall_complexity = (vocabulary_complexity + concept_density + cognitive_load) / 3.0;
        
        match overall_complexity {
            0.0..=0.3 => vec!["elementary_students".to_string(), "general_public".to_string()],
            0.3..=0.5 => vec!["middle_school_students".to_string(), "high_school_students".to_string()],
            0.5..=0.7 => vec!["high_school_students".to_string(), "undergraduate_students".to_string()],
            0.7..=0.85 => vec!["undergraduate_students".to_string(), "graduate_students".to_string()],
            _ => vec!["graduate_students".to_string(), "professionals".to_string(), "experts".to_string()],
        }
    }

    /// Determine overall difficulty level
    fn determine_overall_difficulty(
        &self,
        vocabulary_complexity: f32,
        concept_density: f32,
        sentence_complexity: f32,
        cognitive_load: f32,
    ) -> DifficultyLevel {
        let overall_score = (vocabulary_complexity + concept_density + 
                           sentence_complexity + cognitive_load) / 4.0;

        match overall_score {
            0.0..=0.3 => DifficultyLevel::Beginner,
            0.3..=0.6 => DifficultyLevel::Intermediate,
            0.6..=0.8 => DifficultyLevel::Advanced,
            _ => DifficultyLevel::Expert,
        }
    }

    /// Split text into sentences
    fn split_into_sentences(&self, text: &str) -> Vec<String> {
        text.split(&['.', '!', '?'][..])
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Load basic vocabulary (most common English words)
    fn load_basic_vocabulary() -> HashSet<String> {
        let basic_words = vec![
            "the", "be", "to", "of", "and", "a", "in", "that", "have", "i", "it", "for",
            "not", "on", "with", "he", "as", "you", "do", "at", "this", "but", "his",
            "by", "from", "they", "we", "say", "her", "she", "or", "an", "will", "my",
            "one", "all", "would", "there", "their", "what", "so", "up", "out", "if",
            "about", "who", "get", "which", "go", "me", "when", "make", "can", "like",
            "time", "no", "just", "him", "know", "take", "people", "into", "year",
            "your", "good", "some", "could", "them", "see", "other", "than", "then",
            "now", "look", "only", "come", "its", "over", "think", "also", "back",
            "after", "use", "two", "how", "our", "work", "first", "well", "way",
            "even", "new", "want", "because", "any", "these", "give", "day", "most", "us"
        ];
        
        basic_words.into_iter().map(|s| s.to_string()).collect()
    }

    /// Load academic vocabulary
    fn load_academic_vocabulary() -> HashSet<String> {
        let academic_words = vec![
            "analyze", "approach", "area", "assessment", "assume", "authority", "available",
            "benefit", "concept", "consistent", "constitutional", "context", "contract",
            "create", "data", "definition", "derived", "distribution", "economic", "environment",
            "established", "estimate", "evidence", "export", "factors", "financial", "formula",
            "function", "identified", "income", "indicate", "individual", "interpretation",
            "involved", "issues", "labor", "legal", "legislation", "major", "method",
            "occur", "percent", "period", "policy", "principle", "procedure", "process",
            "required", "research", "response", "role", "section", "significant", "similar",
            "source", "specific", "structure", "theory", "variables", "administration",
            "construction", "consumer", "credit", "cultural", "design", "distinction",
            "elements", "equation", "evaluation", "features", "final", "focus", "impact",
            "injury", "instruction", "maintenance", "normal", "obtained", "participation",
            "perceived", "positive", "potential", "previous", "primary", "purchase",
            "range", "region", "regulations", "relevant", "resident", "resources",
            "restricted", "security", "sought", "style", "survey", "text", "tradition", "transfer"
        ];
        
        academic_words.into_iter().map(|s| s.to_string()).collect()
    }

    /// Load technical indicators
    fn load_technical_indicators() -> Vec<String> {
        vec![
            "tech", "system", "algorithm", "process", "method", "technique", "protocol",
            "framework", "architecture", "implementation", "optimization", "configuration",
            "specification", "parameter", "variable", "function", "module", "component",
            "interface", "database", "server", "client", "network", "security", "encryption",
            "authentication", "authorization", "validation", "verification", "testing",
            "debugging", "deployment", "integration", "migration", "scalability", "performance",
            "efficiency", "throughput", "latency", "bandwidth", "capacity", "redundancy",
            "fault", "tolerance", "reliability", "availability", "maintainability"
        ].into_iter().map(|s| s.to_string()).collect()
    }
}

impl Default for DifficultyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ProcessingPriority;
    use chrono::Utc;
    use std::collections::HashMap;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_difficulty_analysis() {
        let analyzer = DifficultyAnalyzer::new();
        let context = ProcessingContext::new(Uuid::new_v4());

        // Test simple text
        let simple_text = "The cat sat on the mat. It was a sunny day.";
        let analysis = analyzer.analyze_difficulty(simple_text, &context).await.unwrap();
        assert!(matches!(analysis.overall_level, DifficultyLevel::Beginner));

        // Test complex text
        let complex_text = "The implementation of sophisticated algorithmic frameworks \
                           necessitates comprehensive understanding of computational complexity \
                           theory and advanced data structure optimization techniques.";
        let analysis = analyzer.analyze_difficulty(complex_text, &context).await.unwrap();
        assert!(matches!(analysis.overall_level, DifficultyLevel::Advanced | DifficultyLevel::Expert));
    }

    #[test]
    fn test_vocabulary_complexity() {
        let analyzer = DifficultyAnalyzer::new();
        
        let simple_text = "the cat and dog are good friends";
        let simple_score = analyzer.calculate_vocabulary_complexity(simple_text);
        
        let complex_text = "sophisticated algorithmic implementation optimization";
        let complex_score = analyzer.calculate_vocabulary_complexity(complex_text);
        
        assert!(complex_score > simple_score);
    }

    #[test]
    fn test_sentence_complexity() {
        let analyzer = DifficultyAnalyzer::new();
        
        let simple_text = "The cat sat. The dog ran.";
        let simple_score = analyzer.calculate_sentence_complexity(simple_text);
        
        let complex_text = "Although the implementation was challenging, the team successfully \
                           deployed the sophisticated system that incorporated multiple advanced \
                           algorithms, which had been developed over several months.";
        let complex_score = analyzer.calculate_sentence_complexity(complex_text);
        
        assert!(complex_score > simple_score);
    }

    #[test]
    fn test_reading_time_estimation() {
        let analyzer = DifficultyAnalyzer::new();
        
        let short_text = "Hello world.";
        let long_text = "This is a much longer text with many more words that should take \
                        longer to read and understand, especially when it contains complex \
                        vocabulary and technical terminology.".repeat(10);
        
        let short_time = analyzer.estimate_reading_time(short_text);
        let long_time = analyzer.estimate_reading_time(&long_text);
        
        assert!(long_time > short_time);
    }
}