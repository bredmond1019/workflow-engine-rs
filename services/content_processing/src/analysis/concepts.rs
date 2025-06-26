//! Concept extraction using NLP techniques
//!
//! This module implements concept extraction from text using various
//! natural language processing techniques including:
//! - Term frequency analysis
//! - Noun phrase extraction
//! - Named entity recognition
//! - Topic modeling (future enhancement)

use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::models::*;

/// Concept extractor using statistical and rule-based approaches
pub struct ConceptExtractor {
    name: &'static str,
    stop_words: HashSet<String>,
    min_concept_length: usize,
    max_concept_length: usize,
}

impl ConceptExtractor {
    pub fn new() -> Self {
        Self {
            name: "concept_extractor",
            stop_words: Self::load_stop_words(),
            min_concept_length: 3,
            max_concept_length: 50,
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Extract concepts from text
    pub async fn extract_concepts(
        &self, 
        text: &str, 
        _context: &ProcessingContext
    ) -> crate::Result<Vec<Concept>> {
        let mut concepts = Vec::new();
        
        // 1. Extract noun phrases and important terms
        let noun_phrases = self.extract_noun_phrases(text);
        let term_frequencies = self.calculate_term_frequencies(text);
        
        // 2. Score and rank potential concepts
        let mut concept_candidates = HashMap::new();
        
        // Add noun phrases as concept candidates
        for phrase in noun_phrases {
            if self.is_valid_concept(&phrase) {
                let score = self.calculate_concept_score(&phrase, &term_frequencies, text);
                concept_candidates.insert(phrase.clone(), score);
            }
        }
        
        // Add high-frequency terms as concept candidates
        for (term, frequency) in term_frequencies {
            if frequency >= 3 && self.is_valid_concept(&term) {
                let score = self.calculate_concept_score(&term, &HashMap::from([(term.clone(), frequency)]), text);
                concept_candidates.entry(term.clone()).or_insert(score);
            }
        }
        
        // 3. Convert top candidates to Concept objects
        let mut sorted_candidates: Vec<_> = concept_candidates.into_iter().collect();
        sorted_candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Clone the sorted candidates for use in find_related_concepts
        let candidates_for_related = sorted_candidates.clone();
        
        for (concept_name, score) in sorted_candidates.into_iter().take(20) {
            let concept = Concept {
                id: Uuid::new_v4(),
                name: concept_name.clone(),
                description: Some(format!("Concept extracted from text: {}", concept_name)),
                confidence: (score * 0.1).min(1.0).max(0.1), // Normalize to 0.1-1.0 range
                category: self.categorize_concept(&concept_name),
                related_concepts: self.find_related_concepts(&concept_name, &candidates_for_related),
                mentions: self.find_concept_mentions(&concept_name, text),
                importance_score: score * 0.1,
            };
            concepts.push(concept);
        }
        
        Ok(concepts)
    }

    /// Extract noun phrases from text using simple pattern matching
    fn extract_noun_phrases(&self, text: &str) -> Vec<String> {
        let mut phrases = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();
        
        // Simple noun phrase extraction: look for patterns like "Adj Noun" or "Noun Noun"
        for window in words.windows(2) {
            let phrase = window.join(" ");
            let cleaned = self.clean_text(&phrase);
            
            if cleaned.len() >= self.min_concept_length 
                && cleaned.len() <= self.max_concept_length 
                && !self.stop_words.contains(&cleaned.to_lowercase()) {
                phrases.push(cleaned);
            }
        }
        
        // Also extract 3-word phrases
        for window in words.windows(3) {
            let phrase = window.join(" ");
            let cleaned = self.clean_text(&phrase);
            
            if cleaned.len() >= self.min_concept_length 
                && cleaned.len() <= self.max_concept_length {
                phrases.push(cleaned);
            }
        }
        
        phrases
    }

    /// Calculate term frequencies in the text
    fn calculate_term_frequencies(&self, text: &str) -> HashMap<String, usize> {
        let mut frequencies = HashMap::new();
        
        for word in text.split_whitespace() {
            let cleaned = self.clean_text(word);
            if cleaned.len() >= self.min_concept_length 
                && !self.stop_words.contains(&cleaned.to_lowercase()) {
                *frequencies.entry(cleaned).or_insert(0) += 1;
            }
        }
        
        frequencies
    }

    /// Calculate a score for a potential concept
    fn calculate_concept_score(
        &self, 
        concept: &str, 
        term_frequencies: &HashMap<String, usize>, 
        text: &str
    ) -> f32 {
        let mut score = 0.0;
        
        // Frequency score
        let frequency = term_frequencies.get(concept).unwrap_or(&0);
        score += *frequency as f32 * 2.0;
        
        // Length bonus (prefer multi-word concepts)
        let word_count = concept.split_whitespace().count();
        if word_count > 1 {
            score += word_count as f32 * 3.0;
        }
        
        // Position bonus (concepts appearing early get higher scores)
        if let Some(pos) = text.find(concept) {
            let relative_pos = pos as f32 / text.len() as f32;
            score += (1.0 - relative_pos) * 5.0;
        }
        
        // Capitalization bonus (proper nouns are often important concepts)
        if concept.chars().next().map_or(false, |c| c.is_uppercase()) {
            score += 2.0;
        }
        
        score
    }

    /// Determine the category of a concept
    fn categorize_concept(&self, concept: &str) -> ConceptCategory {
        let lower_concept = concept.to_lowercase();
        
        // Simple rule-based categorization
        if lower_concept.contains("algorithm") || lower_concept.contains("method") 
            || lower_concept.contains("technique") || lower_concept.contains("approach") {
            ConceptCategory::Technical
        } else if lower_concept.contains("business") || lower_concept.contains("market") 
                || lower_concept.contains("strategy") || lower_concept.contains("management") {
            ConceptCategory::Business
        } else if lower_concept.contains("research") || lower_concept.contains("study") 
                || lower_concept.contains("experiment") || lower_concept.contains("analysis") {
            ConceptCategory::Scientific
        } else if lower_concept.contains("learn") || lower_concept.contains("teach") 
                || lower_concept.contains("education") || lower_concept.contains("training") {
            ConceptCategory::Educational
        } else {
            ConceptCategory::General
        }
    }

    /// Find concepts related to the given concept
    fn find_related_concepts(&self, concept: &str, candidates: &[(String, f32)]) -> Vec<String> {
        let mut related = Vec::new();
        let concept_words: HashSet<&str> = concept.split_whitespace().collect();
        
        for (candidate, _) in candidates.iter().take(10) {
            if candidate != concept {
                let candidate_words: HashSet<&str> = candidate.split_whitespace().collect();
                let intersection_size = concept_words.intersection(&candidate_words).count();
                
                // If concepts share words, they might be related
                if intersection_size > 0 && intersection_size < concept_words.len().min(candidate_words.len()) {
                    related.push(candidate.clone());
                }
            }
        }
        
        related.into_iter().take(5).collect()
    }

    /// Find all mentions of a concept in the text
    fn find_concept_mentions(&self, concept: &str, text: &str) -> Vec<ConceptMention> {
        let mut mentions = Vec::new();
        let mut start = 0;
        
        while let Some(pos) = text[start..].find(concept) {
            let absolute_pos = start + pos;
            
            // Extract context around the mention
            let context_start = absolute_pos.saturating_sub(50);
            let context_end = (absolute_pos + concept.len() + 50).min(text.len());
            let context = text[context_start..context_end].to_string();
            
            mentions.push(ConceptMention {
                position: absolute_pos as u32,
                context,
                confidence: 0.8, // Fixed confidence for exact matches
            });
            
            start = absolute_pos + concept.len();
        }
        
        mentions
    }

    /// Check if a string is a valid concept candidate
    fn is_valid_concept(&self, text: &str) -> bool {
        let cleaned = text.trim();
        
        // Basic validation rules
        cleaned.len() >= self.min_concept_length
            && cleaned.len() <= self.max_concept_length
            && !cleaned.chars().all(|c| c.is_numeric())
            && !self.stop_words.contains(&cleaned.to_lowercase())
            && cleaned.chars().any(|c| c.is_alphabetic())
    }

    /// Clean and normalize text
    fn clean_text(&self, text: &str) -> String {
        text.trim()
            .replace(&['(', ')', ',', '\"', '.', ';', ':', '\''][..], "")
            .replace("  ", " ")
            .trim()
            .to_string()
    }

    /// Load common stop words
    fn load_stop_words() -> HashSet<String> {
        let stop_words = vec![
            "a", "an", "and", "are", "as", "at", "be", "by", "for", "from",
            "has", "he", "in", "is", "it", "its", "of", "on", "that", "the",
            "to", "was", "will", "with", "we", "you", "i", "me", "my", "our",
            "us", "your", "yours", "his", "her", "hers", "their", "theirs",
            "this", "these", "those", "but", "or", "not", "can", "could",
            "would", "should", "may", "might", "must", "shall", "will",
            "have", "had", "do", "did", "does", "been", "being", "am",
            "very", "much", "many", "most", "more", "some", "any", "all",
            "each", "every", "both", "either", "neither", "one", "two",
            "first", "second", "last", "next", "then", "now", "here", "there",
            "where", "when", "why", "how", "what", "which", "who", "whom",
            "whose", "if", "unless", "until", "while", "during", "before",
            "after", "above", "below", "up", "down", "out", "off", "over",
            "under", "again", "further", "than", "only", "just", "also",
            "too", "quite", "rather", "still", "yet", "however", "therefore",
            "thus", "hence", "so", "because", "since", "although", "though",
            "despite", "instead", "otherwise", "moreover", "furthermore",
            "besides", "meanwhile", "nevertheless", "nonetheless", "anyway",
            "anyhow", "indeed", "certainly", "surely", "obviously", "clearly",
            "apparently", "perhaps", "maybe", "probably", "possibly"
        ];
        
        stop_words.into_iter().map(|s| s.to_string()).collect()
    }
}

impl Default for ConceptExtractor {
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

    #[tokio::test]
    async fn test_concept_extraction() {
        let extractor = ConceptExtractor::new();
        let context = ProcessingContext::new(Uuid::new_v4());

        let text = "Machine learning algorithms are essential for artificial intelligence. \
                   Deep learning networks use neural networks to process data. \
                   Natural language processing helps computers understand human language.";

        let concepts = extractor.extract_concepts(text, &context).await.unwrap();
        
        assert!(!concepts.is_empty());
        
        // Check that we found some key concepts
        let concept_names: Vec<&str> = concepts.iter().map(|c| c.name.as_str()).collect();
        assert!(concept_names.iter().any(|name| name.contains("machine learning") || name.contains("Machine learning")));
    }

    #[test]
    fn test_clean_text() {
        let extractor = ConceptExtractor::new();
        assert_eq!(extractor.clean_text("  hello, world!  "), "hello world!");
        assert_eq!(extractor.clean_text("(test)"), "test");
        assert_eq!(extractor.clean_text("multiple  spaces"), "multiple spaces");
    }

    #[test]
    fn test_is_valid_concept() {
        let extractor = ConceptExtractor::new();
        assert!(extractor.is_valid_concept("machine learning"));
        assert!(!extractor.is_valid_concept("a"));
        assert!(!extractor.is_valid_concept("123"));
        assert!(!extractor.is_valid_concept("the"));
    }

    #[test]
    fn test_concept_categorization() {
        let extractor = ConceptExtractor::new();
        assert!(matches!(extractor.categorize_concept("machine learning algorithm"), ConceptCategory::Technical));
        assert!(matches!(extractor.categorize_concept("business strategy"), ConceptCategory::Business));
        assert!(matches!(extractor.categorize_concept("research study"), ConceptCategory::Scientific));
        assert!(matches!(extractor.categorize_concept("educational training"), ConceptCategory::Educational));
    }
}