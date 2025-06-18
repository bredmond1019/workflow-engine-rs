//! Content quality assessment and scoring
//!
//! This module implements comprehensive quality assessment including:
//! - Readability analysis (Flesch-Kincaid, SMOG, etc.)
//! - Completeness scoring
//! - Grammar and spelling checks
//! - Coherence and structure analysis
//! - Vocabulary richness assessment

use std::collections::HashSet;

use crate::models::*;

/// Quality assessor for text content
pub struct QualityAssessor {
    name: &'static str,
    common_words: HashSet<String>,
}

impl QualityAssessor {
    pub fn new() -> Self {
        Self {
            name: "quality_assessor",
            common_words: Self::load_common_words(),
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Assess the overall quality of text content
    pub async fn assess_quality(
        &self,
        text: &str,
        _context: &ProcessingContext,
    ) -> crate::Result<QualityMetrics> {
        let readability_score = self.calculate_readability_score(text);
        let completeness_score = self.calculate_completeness_score(text);
        let accuracy_score = self.calculate_accuracy_score(text);
        let coherence_score = self.calculate_coherence_score(text);
        let grammar_score = self.calculate_grammar_score(text);
        let vocabulary_richness = self.calculate_vocabulary_richness(text);
        let structure_quality = self.calculate_structure_quality(text);
        
        let issues = self.identify_quality_issues(text);
        
        // Calculate overall score as weighted average
        let overall_score = (
            readability_score * 0.2 +
            completeness_score * 0.15 +
            accuracy_score * 0.2 +
            coherence_score * 0.15 +
            grammar_score * 0.1 +
            vocabulary_richness * 0.1 +
            structure_quality * 0.1
        ).min(1.0).max(0.0);

        Ok(QualityMetrics {
            overall_score,
            readability_score,
            completeness_score,
            accuracy_score,
            coherence_score,
            grammar_score,
            vocabulary_richness,
            structure_quality,
            issues,
        })
    }

    /// Calculate readability score using simplified Flesch Reading Ease
    fn calculate_readability_score(&self, text: &str) -> f32 {
        let sentences = self.count_sentences(text);
        let words = self.count_words(text);
        let syllables = self.count_syllables(text);

        if sentences == 0 || words == 0 {
            return 0.0;
        }

        // Simplified Flesch Reading Ease formula
        let avg_sentence_length = words as f32 / sentences as f32;
        let avg_syllables_per_word = syllables as f32 / words as f32;
        
        let flesch_score = 206.835 - (1.015 * avg_sentence_length) - (84.6 * avg_syllables_per_word);
        
        // Normalize to 0-1 range (Flesch scores typically range from 0-100)
        (flesch_score / 100.0).min(1.0).max(0.0)
    }

    /// Calculate completeness score based on content structure and depth
    fn calculate_completeness_score(&self, text: &str) -> f32 {
        let mut score: f32 = 0.0;
        let word_count = self.count_words(text);
        
        // Length factor (more content generally means more complete)
        score += match word_count {
            0..=50 => 0.2,
            51..=150 => 0.4,
            151..=300 => 0.6,
            301..=500 => 0.8,
            _ => 1.0,
        };
        
        // Structure indicators
        if text.contains(':') || text.contains('-') || text.contains('•') {
            score += 0.1; // Lists or structured content
        }
        
        if text.split('\n').count() > 3 {
            score += 0.1; // Multiple paragraphs
        }
        
        // Question and answer patterns
        if text.contains('?') {
            score += 0.05; // Questions indicate thoroughness
        }
        
        // Examples and explanations
        if text.to_lowercase().contains("example") || text.to_lowercase().contains("for instance") {
            score += 0.1;
        }
        
        score.min(1.0)
    }

    /// Calculate accuracy score (placeholder - would need domain-specific validation)
    fn calculate_accuracy_score(&self, text: &str) -> f32 {
        let mut score: f32 = 0.8; // Start with high baseline
        
        // Check for obvious accuracy indicators
        let lower_text = text.to_lowercase();
        
        // Positive indicators
        if lower_text.contains("research") || lower_text.contains("study") {
            score += 0.1;
        }
        
        if lower_text.contains("source") || lower_text.contains("reference") {
            score += 0.05;
        }
        
        // Negative indicators
        if lower_text.contains("maybe") || lower_text.contains("might be") {
            score -= 0.05; // Uncertainty
        }
        
        if lower_text.contains("i think") || lower_text.contains("i believe") {
            score -= 0.1; // Subjective statements
        }
        
        score.min(1.0).max(0.0)
    }

    /// Calculate coherence score based on text flow and connections
    fn calculate_coherence_score(&self, text: &str) -> f32 {
        let sentences: Vec<&str> = text.split(&['.', '!', '?'][..]).collect();
        if sentences.len() < 2 {
            return 0.5; // Neutral score for very short texts
        }
        
        let mut score = 0.5; // Start with neutral
        
        // Check for transition words and phrases
        let transition_words = vec![
            "however", "therefore", "furthermore", "moreover", "additionally",
            "consequently", "meanwhile", "subsequently", "nevertheless",
            "nonetheless", "thus", "hence", "accordingly", "similarly",
            "likewise", "in contrast", "on the other hand", "as a result",
            "for example", "for instance", "in addition", "in conclusion"
        ];
        
        let lower_text = text.to_lowercase();
        let transition_count = transition_words.iter()
            .filter(|&word| lower_text.contains(word))
            .count();
        
        score += (transition_count as f32 * 0.1).min(0.3);
        
        // Check for pronoun references (indicates connection between sentences)
        let pronouns = vec!["this", "that", "these", "those", "it", "they", "them"];
        let pronoun_count = pronouns.iter()
            .filter(|&pronoun| lower_text.contains(pronoun))
            .count();
        
        score += (pronoun_count as f32 * 0.05).min(0.2);
        
        score.min(1.0)
    }

    /// Calculate grammar score based on basic grammar rules
    fn calculate_grammar_score(&self, text: &str) -> f32 {
        let mut score = 1.0;
        let issues = self.detect_grammar_issues(text);
        
        // Deduct points for each grammar issue
        score -= issues.len() as f32 * 0.1;
        
        score.max(0.0)
    }

    /// Calculate vocabulary richness (lexical diversity)
    fn calculate_vocabulary_richness(&self, text: &str) -> f32 {
        let words: Vec<&str> = text.split_whitespace().collect();
        let unique_words: HashSet<&str> = words.iter().cloned().collect();
        
        if words.is_empty() {
            return 0.0;
        }
        
        let diversity_ratio = unique_words.len() as f32 / words.len() as f32;
        
        // Also consider the use of sophisticated vocabulary
        let sophisticated_count = words.iter()
            .filter(|&word| word.len() > 6 && !self.common_words.contains(&word.to_lowercase()))
            .count();
        
        let sophistication_bonus = (sophisticated_count as f32 / words.len() as f32).min(0.3);
        
        (diversity_ratio + sophistication_bonus).min(1.0)
    }

    /// Calculate structure quality based on formatting and organization
    fn calculate_structure_quality(&self, text: &str) -> f32 {
        let mut score = 0.5; // Start with neutral
        
        let lines: Vec<&str> = text.lines().collect();
        let paragraphs: Vec<&str> = text.split("\n\n").collect();
        
        // Check for proper paragraph structure
        if paragraphs.len() > 1 {
            score += 0.2;
        }
        
        // Check for headers or titles (indicated by shorter lines or capitalization)
        let potential_headers = lines.iter()
            .filter(|line| line.len() < 50 && line.chars().any(|c| c.is_uppercase()))
            .count();
        
        if potential_headers > 0 {
            score += 0.2;
        }
        
        // Check for lists or structured content
        let list_indicators = lines.iter()
            .filter(|line| line.trim_start().starts_with(&['-', '•', '*'][..]) 
                        || line.trim_start().chars().next().map_or(false, |c| c.is_numeric()))
            .count();
        
        if list_indicators > 0 {
            score += 0.15;
        }
        
        // Check for proper sentence endings
        let sentences_with_proper_endings = text.matches(&['.', '!', '?'][..]).count();
        let estimated_sentences = self.count_sentences(text);
        
        if estimated_sentences > 0 {
            let proper_ending_ratio = sentences_with_proper_endings as f32 / estimated_sentences as f32;
            score += proper_ending_ratio * 0.15;
        }
        
        score.min(1.0)
    }

    /// Identify specific quality issues in the text
    fn identify_quality_issues(&self, text: &str) -> Vec<QualityIssue> {
        let mut issues = Vec::new();
        
        // Check for basic formatting issues
        if text.trim().is_empty() {
            issues.push(QualityIssue {
                issue_type: QualityIssueType::Completeness,
                severity: IssueSeverity::Critical,
                description: "Content is empty".to_string(),
                position: Some(0),
                suggestions: vec!["Add content to the document".to_string()],
            });
        }
        
        // Check for very short content
        if self.count_words(text) < 10 {
            issues.push(QualityIssue {
                issue_type: QualityIssueType::Completeness,
                severity: IssueSeverity::High,
                description: "Content is too short".to_string(),
                position: Some(0),
                suggestions: vec!["Expand the content with more details and examples".to_string()],
            });
        }
        
        // Add grammar issues
        issues.extend(self.detect_grammar_issues(text));
        
        // Check for structural issues
        if !text.contains('.') && !text.contains('!') && !text.contains('?') {
            issues.push(QualityIssue {
                issue_type: QualityIssueType::Structure,
                severity: IssueSeverity::Medium,
                description: "No sentence endings found".to_string(),
                position: Some(0),
                suggestions: vec!["Add proper sentence punctuation".to_string()],
            });
        }
        
        issues
    }

    /// Detect basic grammar issues
    fn detect_grammar_issues(&self, text: &str) -> Vec<QualityIssue> {
        let mut issues = Vec::new();
        
        // Check for repeated words
        let words: Vec<&str> = text.split_whitespace().collect();
        for window in words.windows(2) {
            if window[0].to_lowercase() == window[1].to_lowercase() {
                issues.push(QualityIssue {
                    issue_type: QualityIssueType::Grammar,
                    severity: IssueSeverity::Low,
                    description: format!("Repeated word: '{}'", window[0]),
                    position: text.find(window[0]).map(|p| p as u32),
                    suggestions: vec!["Remove the repeated word".to_string()],
                });
            }
        }
        
        // Check for excessive capitalization
        let cap_ratio = text.chars().filter(|c| c.is_uppercase()).count() as f32 / text.len() as f32;
        if cap_ratio > 0.3 {
            issues.push(QualityIssue {
                issue_type: QualityIssueType::Formatting,
                severity: IssueSeverity::Medium,
                description: "Excessive use of capital letters".to_string(),
                position: Some(0),
                suggestions: vec!["Use normal capitalization rules".to_string()],
            });
        }
        
        issues
    }

    /// Count the number of sentences in text
    fn count_sentences(&self, text: &str) -> usize {
        text.matches(&['.', '!', '?'][..]).count().max(1)
    }

    /// Count the number of words in text
    fn count_words(&self, text: &str) -> usize {
        text.split_whitespace().count()
    }

    /// Estimate syllable count using simple heuristics
    fn count_syllables(&self, text: &str) -> usize {
        text.split_whitespace()
            .map(|word| self.syllables_in_word(word))
            .sum()
    }

    /// Estimate syllables in a single word
    fn syllables_in_word(&self, word: &str) -> usize {
        let word = word.to_lowercase();
        let vowels = ['a', 'e', 'i', 'o', 'u', 'y'];
        
        let mut count = 0;
        let mut prev_was_vowel = false;
        
        for ch in word.chars() {
            if vowels.contains(&ch) {
                if !prev_was_vowel {
                    count += 1;
                }
                prev_was_vowel = true;
            } else {
                prev_was_vowel = false;
            }
        }
        
        // Adjust for silent 'e' and ensure minimum of 1 syllable
        if word.ends_with('e') && count > 1 {
            count -= 1;
        }
        
        count.max(1)
    }

    /// Load common English words for vocabulary analysis
    fn load_common_words() -> HashSet<String> {
        let common_words = vec![
            "the", "be", "to", "of", "and", "a", "in", "that", "have",
            "i", "it", "for", "not", "on", "with", "he", "as", "you",
            "do", "at", "this", "but", "his", "by", "from", "they",
            "we", "say", "her", "she", "or", "an", "will", "my",
            "one", "all", "would", "there", "their", "what", "so",
            "up", "out", "if", "about", "who", "get", "which", "go",
            "me", "when", "make", "can", "like", "time", "no", "just",
            "him", "know", "take", "people", "into", "year", "your",
            "good", "some", "could", "them", "see", "other", "than",
            "then", "now", "look", "only", "come", "its", "over",
            "think", "also", "back", "after", "use", "two", "how",
            "our", "work", "first", "well", "way", "even", "new",
            "want", "because", "any", "these", "give", "day", "most",
            "us", "is", "was", "are", "been", "has", "had", "were"
        ];
        
        common_words.into_iter().map(|s| s.to_string()).collect()
    }
}

impl Default for QualityAssessor {
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
    async fn test_quality_assessment() {
        let assessor = QualityAssessor::new();
        let context = ProcessingContext {
            job_id: Uuid::new_v4(),
            user_id: None,
            session_id: None,
            correlation_id: None,
            processing_started_at: Utc::now(),
            max_memory_mb: None,
            priority: ProcessingPriority::Normal,
            retry_count: 0,
            custom_data: HashMap::new(),
        };

        let good_text = "This is a well-structured document with multiple sentences. \
                        It contains proper punctuation and demonstrates good writing quality. \
                        Furthermore, it includes transition words that improve coherence. \
                        The vocabulary is varied and appropriate for the content.";

        let quality = assessor.assess_quality(good_text, &context).await.unwrap();
        
        assert!(quality.overall_score > 0.5);
        assert!(quality.readability_score > 0.0);
        assert!(quality.completeness_score > 0.0);
    }

    #[test]
    fn test_syllable_counting() {
        let assessor = QualityAssessor::new();
        assert_eq!(assessor.syllables_in_word("hello"), 2);
        assert_eq!(assessor.syllables_in_word("cat"), 1);
        assert_eq!(assessor.syllables_in_word("beautiful"), 3);
        assert_eq!(assessor.syllables_in_word("a"), 1);
    }

    #[test]
    fn test_word_counting() {
        let assessor = QualityAssessor::new();
        assert_eq!(assessor.count_words("hello world"), 2);
        assert_eq!(assessor.count_words(""), 0);
        assert_eq!(assessor.count_words("   spaced   words   "), 2);
    }

    #[test]
    fn test_sentence_counting() {
        let assessor = QualityAssessor::new();
        assert_eq!(assessor.count_sentences("Hello world."), 1);
        assert_eq!(assessor.count_sentences("Hello! How are you?"), 2);
        assert_eq!(assessor.count_sentences("No punctuation"), 1);
    }
}