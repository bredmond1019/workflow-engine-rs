//! Keyword extraction and ranking
//!
//! This module implements keyword extraction using various techniques:
//! - TF-IDF (Term Frequency-Inverse Document Frequency)
//! - Statistical frequency analysis
//! - N-gram extraction
//! - Stop word filtering

use std::collections::{HashMap, HashSet};

/// Keyword extractor using statistical and frequency-based methods
pub struct KeywordExtractor {
    name: &'static str,
    stop_words: HashSet<String>,
    min_keyword_length: usize,
    max_keyword_length: usize,
}

impl KeywordExtractor {
    pub fn new() -> Self {
        Self {
            name: "keyword_extractor",
            stop_words: Self::load_stop_words(),
            min_keyword_length: 3,
            max_keyword_length: 25,
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Extract keywords from text
    pub async fn extract_keywords(
        &self,
        text: &str,
        max_keywords: Option<usize>,
    ) -> crate::Result<Vec<String>> {
        let max_count = max_keywords.unwrap_or(10);
        
        // 1. Get candidate keywords
        let single_words = self.extract_single_word_candidates(text);
        let bigrams = self.extract_bigrams(text);
        let trigrams = self.extract_trigrams(text);
        
        // 2. Combine and score all candidates
        let mut all_candidates = HashMap::new();
        
        // Add single words with base scores
        for (word, score) in single_words {
            all_candidates.insert(word, score);
        }
        
        // Add bigrams with bonus for being multi-word
        for (bigram, score) in bigrams {
            all_candidates.insert(bigram, score * 1.5);
        }
        
        // Add trigrams with higher bonus
        for (trigram, score) in trigrams {
            all_candidates.insert(trigram, score * 2.0);
        }
        
        // 3. Sort by score and return top keywords
        let mut sorted_keywords: Vec<_> = all_candidates.into_iter().collect();
        sorted_keywords.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(sorted_keywords
            .into_iter()
            .take(max_count)
            .map(|(keyword, _)| keyword)
            .collect())
    }

    /// Extract single-word keyword candidates
    fn extract_single_word_candidates(&self, text: &str) -> HashMap<String, f32> {
        let mut candidates = HashMap::new();
        let words: Vec<&str> = text.split_whitespace().collect();
        let total_words = words.len();
        
        for word in words {
            let cleaned = self.clean_word(word);
            if self.is_valid_keyword(&cleaned) {
                let score = self.calculate_single_word_score(&cleaned, text, total_words);
                candidates.insert(cleaned, score);
            }
        }
        
        candidates
    }

    /// Extract bigram (2-word) candidates
    fn extract_bigrams(&self, text: &str) -> HashMap<String, f32> {
        let mut candidates = HashMap::new();
        let words: Vec<&str> = text.split_whitespace().collect();
        
        for window in words.windows(2) {
            let word1 = self.clean_word(window[0]);
            let word2 = self.clean_word(window[1]);
            
            if self.is_valid_keyword(&word1) && self.is_valid_keyword(&word2) {
                let bigram = format!("{} {}", word1, word2);
                if bigram.len() <= self.max_keyword_length {
                    let score = self.calculate_ngram_score(&bigram, text);
                    candidates.insert(bigram, score);
                }
            }
        }
        
        candidates
    }

    /// Extract trigram (3-word) candidates
    fn extract_trigrams(&self, text: &str) -> HashMap<String, f32> {
        let mut candidates = HashMap::new();
        let words: Vec<&str> = text.split_whitespace().collect();
        
        for window in words.windows(3) {
            let word1 = self.clean_word(window[0]);
            let word2 = self.clean_word(window[1]);
            let word3 = self.clean_word(window[2]);
            
            if self.is_valid_keyword(&word1) && self.is_valid_keyword(&word2) && self.is_valid_keyword(&word3) {
                let trigram = format!("{} {} {}", word1, word2, word3);
                if trigram.len() <= self.max_keyword_length {
                    let score = self.calculate_ngram_score(&trigram, text);
                    candidates.insert(trigram, score);
                }
            }
        }
        
        candidates
    }

    /// Calculate score for single words
    fn calculate_single_word_score(&self, word: &str, text: &str, total_words: usize) -> f32 {
        let frequency = text.matches(word).count();
        let mut score = frequency as f32;
        
        // TF score component
        let tf = frequency as f32 / total_words as f32;
        score += tf * 10.0;
        
        // Length bonus (longer words often more significant)
        if word.len() > 6 {
            score += 2.0;
        } else if word.len() > 4 {
            score += 1.0;
        }
        
        // Capitalization bonus (proper nouns)
        if word.chars().next().map_or(false, |c| c.is_uppercase()) {
            score += 3.0;
        }
        
        // Position bonus (words appearing early are often important)
        if let Some(first_occurrence) = text.find(word) {
            let relative_position = first_occurrence as f32 / text.len() as f32;
            score += (1.0 - relative_position) * 2.0;
        }
        
        // Technical term bonus
        if self.is_technical_term(word) {
            score += 2.0;
        }
        
        score
    }

    /// Calculate score for n-grams (bigrams, trigrams)
    fn calculate_ngram_score(&self, ngram: &str, text: &str) -> f32 {
        let frequency = text.matches(ngram).count();
        let mut score = frequency as f32 * 2.0; // Base score higher for multi-word terms
        
        // Phrase coherence bonus
        if self.is_coherent_phrase(ngram) {
            score += 3.0;
        }
        
        // Title case bonus (likely proper nouns or important phrases)
        if ngram.split_whitespace().all(|word| 
            word.chars().next().map_or(false, |c| c.is_uppercase())) {
            score += 2.0;
        }
        
        // Technical phrase bonus
        if self.is_technical_phrase(ngram) {
            score += 3.0;
        }
        
        score
    }

    /// Check if a phrase is coherent (not just random words)
    fn is_coherent_phrase(&self, phrase: &str) -> bool {
        let words: Vec<&str> = phrase.split_whitespace().collect();
        
        // Check for common phrase patterns
        if words.len() == 2 {
            let word1 = words[0].to_lowercase();
            let word2 = words[1].to_lowercase();
            
            // Adjective + Noun patterns
            if self.is_likely_adjective(&word1) && self.is_likely_noun(&word2) {
                return true;
            }
            
            // Noun + Noun patterns (compound nouns)
            if self.is_likely_noun(&word1) && self.is_likely_noun(&word2) {
                return true;
            }
        }
        
        // Check for preposition patterns that might not be useful
        let prepositions = vec!["of", "in", "on", "at", "by", "for", "with", "from", "to"];
        let phrase_lower = phrase.to_lowercase();
        
        // Check if phrase starts with preposition
        for prep in &prepositions {
            if phrase_lower.starts_with(&format!("{} ", prep)) {
                return false;
            }
        }
        
        // Check for common non-coherent patterns
        let non_coherent_patterns = vec!["of the", "in a", "on the", "at the", "by the"];
        for pattern in non_coherent_patterns {
            if phrase_lower == pattern {
                return false;
            }
        }
        
        true
    }

    /// Check if a word is likely an adjective
    fn is_likely_adjective(&self, word: &str) -> bool {
        word.ends_with("ing") || word.ends_with("ed") || word.ends_with("ive") 
            || word.ends_with("ous") || word.ends_with("ful") || word.ends_with("less")
            || word.ends_with("able") || word.ends_with("ible")
    }

    /// Check if a word is likely a noun
    fn is_likely_noun(&self, word: &str) -> bool {
        word.ends_with("tion") || word.ends_with("sion") || word.ends_with("ment")
            || word.ends_with("ness") || word.ends_with("ity") || word.ends_with("ism")
            || word.ends_with("er") || word.ends_with("or") || word.ends_with("ist")
            || word.chars().next().map_or(false, |c| c.is_uppercase()) // Proper nouns
    }

    /// Check if a word is a technical term
    fn is_technical_term(&self, word: &str) -> bool {
        let technical_suffixes = ["ism", "ology", "graphy", "metry", "scopy"];
        let technical_prefixes = ["bio", "geo", "micro", "macro", "proto", "pseudo"];
        let technical_words = [
            "algorithm", "methodology", "analysis", "framework", "protocol",
            "implementation", "optimization", "configuration", "architecture"
        ];
        
        let word_lower = word.to_lowercase();
        
        technical_suffixes.iter().any(|suffix| word_lower.ends_with(suffix))
            || technical_prefixes.iter().any(|prefix| word_lower.starts_with(prefix))
            || technical_words.contains(&word_lower.as_str())
            || word.len() > 8 && word.contains('-')
    }

    /// Check if a phrase contains technical terminology
    fn is_technical_phrase(&self, phrase: &str) -> bool {
        let technical_keywords = vec![
            "algorithm", "system", "process", "method", "technique", "approach",
            "framework", "model", "theory", "analysis", "implementation", "optimization",
            "protocol", "interface", "architecture", "design", "pattern", "structure"
        ];
        
        let phrase_lower = phrase.to_lowercase();
        technical_keywords.iter().any(|keyword| phrase_lower.contains(keyword))
    }

    /// Clean and normalize a word
    fn clean_word(&self, word: &str) -> String {
        word.trim_matches(|c: char| !c.is_alphanumeric() && c != '-')
            .to_string()
    }

    /// Check if a word is a valid keyword candidate
    fn is_valid_keyword(&self, word: &str) -> bool {
        !word.is_empty()
            && word.len() >= self.min_keyword_length
            && word.len() <= self.max_keyword_length
            && !self.stop_words.contains(&word.to_lowercase())
            && !word.chars().all(|c| c.is_numeric())
            && word.chars().any(|c| c.is_alphabetic())
    }

    /// Load stop words for filtering
    fn load_stop_words() -> HashSet<String> {
        let stop_words = vec![
            "a", "an", "and", "are", "as", "at", "be", "by", "for", "from",
            "has", "he", "in", "is", "it", "its", "of", "on", "that", "the",
            "to", "was", "will", "with", "we", "you", "i", "me", "my", "our",
            "us", "your", "yours", "his", "her", "hers", "their", "theirs",
            "this", "these", "those", "but", "or", "not", "can", "could",
            "would", "should", "may", "might", "must", "shall", "have",
            "had", "do", "did", "does", "been", "being", "am", "very",
            "much", "many", "most", "more", "some", "any", "all", "each",
            "every", "both", "either", "neither", "one", "two", "first",
            "second", "last", "next", "then", "now", "here", "there",
            "where", "when", "why", "how", "what", "which", "who", "whom",
            "whose", "if", "unless", "until", "while", "during", "before",
            "after", "above", "below", "up", "down", "out", "off", "over",
            "under", "again", "further", "than", "only", "just", "also",
            "too", "quite", "rather", "still", "yet"
        ];
        
        stop_words.into_iter().map(|s| s.to_string()).collect()
    }
}

impl Default for KeywordExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_keyword_extraction() {
        let extractor = KeywordExtractor::new();
        
        let text = "Machine learning algorithms are essential for artificial intelligence applications. \
                   Deep learning networks and neural networks process large datasets efficiently. \
                   Natural language processing enables computers to understand human language.";

        let keywords = extractor.extract_keywords(text, Some(10)).await.unwrap();
        
        assert!(!keywords.is_empty());
        assert!(keywords.len() <= 10);
        
        // Should contain some key technical terms
        let keywords_str = keywords.join(" ").to_lowercase();
        assert!(keywords_str.contains("machine") || keywords_str.contains("learning") 
               || keywords_str.contains("algorithm"));
    }

    #[test]
    fn test_clean_word() {
        let extractor = KeywordExtractor::new();
        assert_eq!(extractor.clean_word("hello,"), "hello");
        assert_eq!(extractor.clean_word("(world)"), "world");
        assert_eq!(extractor.clean_word("test-case"), "test-case");
        assert_eq!(extractor.clean_word("  trimmed  "), "trimmed");
    }

    #[test]
    fn test_is_valid_keyword() {
        let extractor = KeywordExtractor::new();
        assert!(extractor.is_valid_keyword("machine"));
        assert!(extractor.is_valid_keyword("learning"));
        assert!(!extractor.is_valid_keyword("the"));
        assert!(!extractor.is_valid_keyword("a"));
        assert!(!extractor.is_valid_keyword("123"));
        assert!(!extractor.is_valid_keyword(""));
    }

    #[test]
    fn test_technical_term_detection() {
        let extractor = KeywordExtractor::new();
        assert!(extractor.is_technical_term("algorithm"));
        assert!(extractor.is_technical_term("methodology"));
        assert!(extractor.is_technical_term("biology"));
        assert!(extractor.is_technical_term("microscopy"));
        assert!(!extractor.is_technical_term("simple"));
        assert!(!extractor.is_technical_term("word"));
    }

    #[test]
    fn test_coherent_phrase_detection() {
        let extractor = KeywordExtractor::new();
        assert!(extractor.is_coherent_phrase("machine learning"));
        assert!(extractor.is_coherent_phrase("artificial intelligence"));
        assert!(extractor.is_coherent_phrase("data science"));
        assert!(!extractor.is_coherent_phrase("of the"));
        assert!(!extractor.is_coherent_phrase("in a"));
    }
}