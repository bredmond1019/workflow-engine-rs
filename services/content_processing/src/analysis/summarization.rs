//! Text summarization capabilities
//!
//! This module implements text summarization using:
//! - AI-powered abstractive summarization (primary)
//! - Extractive summarization as fallback
//! - Sentence ranking based on importance scores
//! - TF-IDF scoring for key terms

use std::collections::HashMap;

use crate::models::*;
use crate::ai_integration::AIContentAnalyzer;

/// Text summarizer using AI and extractive summarization techniques
pub struct TextSummarizer {
    name: &'static str,
    stop_words: std::collections::HashSet<String>,
    ai_analyzer: AIContentAnalyzer,
}

impl TextSummarizer {
    pub fn new() -> Self {
        Self {
            name: "text_summarizer",
            stop_words: Self::load_stop_words(),
            ai_analyzer: AIContentAnalyzer::new(),
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Generate a summary of the given text
    pub async fn generate_summary(
        &self,
        text: &str,
        max_length: Option<usize>,
        _context: &ProcessingContext,
    ) -> crate::Result<String> {
        let target_length = max_length.unwrap_or(200);
        
        if text.trim().is_empty() {
            return Ok(String::new());
        }

        // Try AI-powered summarization first
        match self.ai_analyzer.generate_summary(text, target_length).await {
            Ok(ai_summary) => {
                if !ai_summary.trim().is_empty() && ai_summary.len() <= target_length * 2 {
                    return Ok(ai_summary);
                }
            }
            Err(_) => {
                // Continue with extractive fallback
            }
        }

        // Fallback to extractive summarization
        self.extractive_summary(text, target_length).await
    }

    /// Generate extractive summary as fallback
    async fn extractive_summary(&self, text: &str, target_length: usize) -> crate::Result<String> {
        // 1. Split text into sentences
        let sentences = self.split_into_sentences(text);
        
        if sentences.is_empty() {
            return Ok(String::new());
        }

        // If text is already short enough, return it as-is
        if text.len() <= target_length {
            return Ok(text.to_string());
        }

        // 2. Calculate importance scores for each sentence
        let sentence_scores = self.calculate_sentence_scores(&sentences, text);

        // 3. Select top sentences while maintaining order and coherence
        let selected_sentences = self.select_sentences(&sentences, &sentence_scores, target_length);

        // 4. Reconstruct summary maintaining original order
        let summary = self.reconstruct_summary(&sentences, &selected_sentences);

        Ok(summary)
    }

    /// Split text into sentences
    fn split_into_sentences(&self, text: &str) -> Vec<String> {
        // Simple sentence splitting on periods, exclamation marks, and question marks
        let mut sentences = Vec::new();
        let mut current_sentence = String::new();
        
        for ch in text.chars() {
            current_sentence.push(ch);
            
            if ch == '.' || ch == '!' || ch == '?' {
                let trimmed = current_sentence.trim();
                if !trimmed.is_empty() && trimmed.len() > 10 { // Filter out very short fragments
                    sentences.push(trimmed.to_string());
                }
                current_sentence.clear();
            }
        }
        
        // Add any remaining content
        let trimmed = current_sentence.trim();
        if !trimmed.is_empty() && trimmed.len() > 10 {
            sentences.push(trimmed.to_string());
        }
        
        sentences
    }

    /// Calculate importance scores for sentences
    fn calculate_sentence_scores(&self, sentences: &[String], full_text: &str) -> Vec<f32> {
        let word_frequencies = self.calculate_word_frequencies(full_text);
        let mut scores = Vec::new();

        for (index, sentence) in sentences.iter().enumerate() {
            let mut score = 0.0;

            // 1. TF-IDF based scoring
            score += self.calculate_tf_idf_score(sentence, &word_frequencies);

            // 2. Position-based scoring (earlier sentences often more important)
            let position_score = 1.0 - (index as f32 / sentences.len() as f32);
            score += position_score * 2.0;

            // 3. Length-based scoring (prefer moderate length sentences)
            let words_count = sentence.split_whitespace().count();
            let length_score = match words_count {
                0..=5 => 0.5,    // Too short
                6..=15 => 1.0,   // Good length
                16..=25 => 0.8,  // Acceptable
                _ => 0.3,        // Too long
            };
            score += length_score;

            // 4. Keyword density
            score += self.calculate_keyword_density(sentence);

            // 5. Numerical data bonus (facts and figures often important)
            if sentence.chars().any(|c| c.is_numeric()) {
                score += 1.0;
            }

            // 6. Question sentences (often important)
            if sentence.contains('?') {
                score += 0.5;
            }

            // 7. Sentences with proper nouns (often contain key entities)
            let proper_noun_count = sentence.split_whitespace()
                .filter(|word| word.chars().next().map_or(false, |c| c.is_uppercase()))
                .count();
            score += proper_noun_count as f32 * 0.3;

            scores.push(score);
        }

        scores
    }

    /// Calculate TF-IDF score for a sentence
    fn calculate_tf_idf_score(&self, sentence: &str, word_frequencies: &HashMap<String, usize>) -> f32 {
        let words: Vec<&str> = sentence.split_whitespace().collect();
        let mut score = 0.0;

        for word in words {
            let cleaned_word = word.to_lowercase().trim_matches(|c: char| !c.is_alphanumeric()).to_string();
            
            if !cleaned_word.is_empty() && !self.stop_words.contains(&cleaned_word) {
                if let Some(&frequency) = word_frequencies.get(&cleaned_word) {
                    // Simple TF-IDF approximation
                    let tf = frequency as f32;
                    let idf = if frequency > 1 { 1.0 / frequency as f32 } else { 1.0 };
                    score += tf * idf;
                }
            }
        }

        score
    }

    /// Calculate word frequencies in the entire text
    fn calculate_word_frequencies(&self, text: &str) -> HashMap<String, usize> {
        let mut frequencies = HashMap::new();
        
        for word in text.split_whitespace() {
            let cleaned = word.to_lowercase().trim_matches(|c: char| !c.is_alphanumeric()).to_string();
            if !cleaned.is_empty() && !self.stop_words.contains(&cleaned) {
                *frequencies.entry(cleaned).or_insert(0) += 1;
            }
        }
        
        frequencies
    }

    /// Calculate keyword density score
    fn calculate_keyword_density(&self, sentence: &str) -> f32 {
        let important_words = vec![
            "important", "significant", "key", "main", "primary", "essential",
            "critical", "major", "fundamental", "core", "central", "crucial",
            "notable", "remarkable", "outstanding", "exceptional", "unique",
            "innovative", "advanced", "sophisticated", "complex", "comprehensive"
        ];

        let sentence_lower = sentence.to_lowercase();
        let keyword_count = important_words.iter()
            .filter(|&keyword| sentence_lower.contains(keyword))
            .count();

        keyword_count as f32 * 0.5
    }

    /// Select sentences for the summary
    fn select_sentences(
        &self,
        sentences: &[String],
        scores: &[f32],
        target_length: usize,
    ) -> Vec<usize> {
        // Create pairs of (index, score) and sort by score
        let mut sentence_indices: Vec<(usize, f32)> = scores.iter()
            .enumerate()
            .map(|(i, &score)| (i, score))
            .collect();
        
        sentence_indices.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Greedily select sentences until we reach target length
        let mut selected = Vec::new();
        let mut current_length = 0;

        for (index, _score) in sentence_indices {
            let sentence_length = sentences[index].len();
            
            // Check if adding this sentence would exceed target length
            if current_length + sentence_length > target_length && !selected.is_empty() {
                break;
            }
            
            selected.push(index);
            current_length += sentence_length;
            
            // Stop if we have a good number of sentences
            if selected.len() >= 5 && current_length >= target_length / 2 {
                break;
            }
        }

        // Ensure we have at least one sentence
        if selected.is_empty() && !sentences.is_empty() {
            selected.push(0);
        }

        // Sort selected indices to maintain original order
        selected.sort_unstable();
        selected
    }

    /// Reconstruct summary from selected sentences
    fn reconstruct_summary(&self, sentences: &[String], selected_indices: &[usize]) -> String {
        selected_indices.iter()
            .map(|&index| sentences[index].trim())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Load stop words for filtering
    fn load_stop_words() -> std::collections::HashSet<String> {
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

impl Default for TextSummarizer {
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
    async fn test_text_summarization() {
        let summarizer = TextSummarizer::new();
        let context = ProcessingContext::new(Uuid::new_v4());

        let long_text = "Machine learning is a powerful subset of artificial intelligence. \
                        It enables computers to learn and improve from experience without being explicitly programmed. \
                        There are three main types of machine learning: supervised learning, unsupervised learning, and reinforcement learning. \
                        Supervised learning uses labeled training data to learn a mapping from inputs to outputs. \
                        Unsupervised learning finds hidden patterns in data without labeled examples. \
                        Reinforcement learning learns through interaction with an environment using rewards and penalties. \
                        Deep learning is a specialized form of machine learning that uses neural networks with multiple layers. \
                        These networks can automatically learn hierarchical representations of data. \
                        Applications of machine learning include image recognition, natural language processing, and recommendation systems. \
                        The field continues to evolve rapidly with new algorithms and techniques being developed regularly.";

        let summary = summarizer.generate_summary(long_text, Some(200), &context).await.unwrap();
        
        assert!(!summary.is_empty());
        assert!(summary.len() <= 500); // Allow buffer for AI response
        assert!(summary.contains("machine learning") || summary.contains("Machine learning"));
    }

    #[tokio::test]
    async fn test_short_text_summarization() {
        let summarizer = TextSummarizer::new();
        let context = ProcessingContext::new(Uuid::new_v4());

        let short_text = "This is a short text that doesn't need summarization.";
        let summary = summarizer.generate_summary(short_text, Some(200), &context).await.unwrap();
        
        // The AI might return a message about the text being too short, or return the original text
        // For short text, it should either return the original text or a meaningful summary
        assert!(!summary.is_empty() && 
               (summary == short_text || 
                summary.contains("text") || 
                summary.contains("short") || 
                summary.len() <= short_text.len()));
    }

    #[test]
    fn test_sentence_splitting() {
        let summarizer = TextSummarizer::new();
        let text = "First sentence. Second sentence! Third sentence? Fourth sentence.";
        let sentences = summarizer.split_into_sentences(text);
        
        assert_eq!(sentences.len(), 4);
        assert!(sentences[0].contains("First sentence"));
        assert!(sentences[1].contains("Second sentence"));
    }

    #[test]
    fn test_word_frequency_calculation() {
        let summarizer = TextSummarizer::new();
        let text = "machine learning machine learning algorithm";
        let frequencies = summarizer.calculate_word_frequencies(text);
        
        assert_eq!(frequencies.get("machine"), Some(&2));
        assert_eq!(frequencies.get("learning"), Some(&2));
        assert_eq!(frequencies.get("algorithm"), Some(&1));
    }
}