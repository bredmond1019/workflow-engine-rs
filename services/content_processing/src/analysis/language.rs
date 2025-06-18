//! Language detection for text content
//!
//! This module implements language detection using the whatlang library
//! combined with custom pattern matching for improved accuracy

use std::collections::HashMap;

/// Language detector using statistical and pattern-based methods
pub struct LanguageDetector {
    name: &'static str,
    language_patterns: HashMap<String, LanguageProfile>,
}

#[derive(Debug, Clone)]
struct LanguageProfile {
    common_words: Vec<String>,
    character_frequencies: HashMap<char, f32>,
    digrams: Vec<String>,
    trigrams: Vec<String>,
}

impl LanguageDetector {
    pub fn new() -> Self {
        Self {
            name: "language_detector",
            language_patterns: Self::load_language_patterns(),
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Detect the language of the given text
    pub async fn detect_language(&self, text: &str) -> crate::Result<String> {
        if text.trim().is_empty() {
            return Ok("unknown".to_string());
        }

        // Use whatlang for primary detection
        if let Some(info) = whatlang::detect(text) {
            let primary_lang = self.lang_code_to_string(info.lang());
            
            // If confidence is high enough, return the result
            if info.confidence() > 0.8 {
                return Ok(primary_lang);
            }
            
            // Otherwise, combine with custom analysis for better accuracy
            let custom_result = self.custom_language_detection(text).await?;
            
            // If both methods agree, use that result
            if primary_lang == custom_result {
                return Ok(primary_lang);
            }
            
            // If they disagree, use the one with higher confidence
            if info.confidence() > 0.6 {
                return Ok(primary_lang);
            } else {
                return Ok(custom_result);
            }
        }
        
        // Fallback to custom detection if whatlang fails
        self.custom_language_detection(text).await
    }
    
    /// Convert whatlang Lang enum to string
    fn lang_code_to_string(&self, lang: whatlang::Lang) -> String {
        use whatlang::Lang;
        match lang {
            Lang::Eng => "en".to_string(),
            Lang::Spa => "es".to_string(),
            Lang::Fra => "fr".to_string(),
            Lang::Deu => "de".to_string(),
            Lang::Ita => "it".to_string(),
            Lang::Por => "pt".to_string(),
            Lang::Rus => "ru".to_string(),
            Lang::Cmn => "zh".to_string(),
            Lang::Jpn => "ja".to_string(),
            Lang::Ara => "ar".to_string(),
            Lang::Nld => "nl".to_string(),
            Lang::Pol => "pl".to_string(),
            Lang::Tur => "tr".to_string(),
            Lang::Kor => "ko".to_string(),
            Lang::Hin => "hi".to_string(),
            _ => "en".to_string(), // Default to English for other languages
        }
    }
    
    /// Custom language detection as fallback
    async fn custom_language_detection(&self, text: &str) -> crate::Result<String> {
        let text_lower = text.to_lowercase();
        let mut language_scores = HashMap::new();

        // Initialize scores for all supported languages
        for language in self.language_patterns.keys() {
            language_scores.insert(language.clone(), 0.0);
        }

        // 1. Common word analysis
        self.analyze_common_words(&text_lower, &mut language_scores);

        // 2. Character frequency analysis
        self.analyze_character_frequencies(&text_lower, &mut language_scores);

        // 3. N-gram analysis
        self.analyze_ngrams(&text_lower, &mut language_scores);

        // 4. Special character patterns
        self.analyze_special_patterns(&text, &mut language_scores);

        // Find the language with the highest score
        let detected_language = language_scores
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(lang, _)| lang)
            .unwrap_or_else(|| "en".to_string()); // Default to English

        Ok(detected_language)
    }

    /// Analyze common words to determine language
    fn analyze_common_words(&self, text: &str, language_scores: &mut HashMap<String, f32>) {
        let words: Vec<&str> = text.split_whitespace().collect();
        let total_words = words.len() as f32;

        if total_words == 0.0 {
            return;
        }

        for (language, profile) in &self.language_patterns {
            let mut matches = 0;
            
            for word in &words {
                let cleaned_word = word.trim_matches(|c: char| !c.is_alphabetic());
                if profile.common_words.contains(&cleaned_word.to_string()) {
                    matches += 1;
                }
            }

            let match_ratio = matches as f32 / total_words;
            *language_scores.get_mut(language).unwrap() += match_ratio * 100.0;
        }
    }

    /// Analyze character frequencies
    fn analyze_character_frequencies(&self, text: &str, language_scores: &mut HashMap<String, f32>) {
        let mut char_counts = HashMap::new();
        let mut total_chars = 0;

        // Count characters in the text
        for ch in text.chars() {
            if ch.is_alphabetic() {
                *char_counts.entry(ch).or_insert(0) += 1;
                total_chars += 1;
            }
        }

        if total_chars == 0 {
            return;
        }

        // Calculate character frequencies
        let mut text_char_freq = HashMap::new();
        for (ch, count) in char_counts {
            text_char_freq.insert(ch, count as f32 / total_chars as f32);
        }

        // Compare with language profiles
        for (language, profile) in &self.language_patterns {
            let mut similarity_score = 0.0;
            let mut total_weight = 0.0;

            for (ch, expected_freq) in &profile.character_frequencies {
                let actual_freq = text_char_freq.get(ch).unwrap_or(&0.0);
                let weight = expected_freq * expected_freq; // Weight by expected frequency
                similarity_score += weight * (1.0 - (expected_freq - actual_freq).abs());
                total_weight += weight;
            }

            if total_weight > 0.0 {
                let normalized_score = similarity_score / total_weight;
                *language_scores.get_mut(language).unwrap() += normalized_score * 50.0;
            }
        }
    }

    /// Analyze n-grams (digrams and trigrams)
    fn analyze_ngrams(&self, text: &str, language_scores: &mut HashMap<String, f32>) {
        let chars: Vec<char> = text.chars().filter(|c| c.is_alphabetic()).collect();
        
        if chars.len() < 3 {
            return;
        }

        // Extract digrams and trigrams
        let mut digrams = Vec::new();
        let mut trigrams = Vec::new();

        for i in 0..chars.len() - 1 {
            digrams.push(chars[i..i+2].iter().collect::<String>());
        }

        for i in 0..chars.len() - 2 {
            trigrams.push(chars[i..i+3].iter().collect::<String>());
        }

        // Score based on n-gram matches
        for (language, profile) in &self.language_patterns {
            let mut score = 0.0;

            // Digram scoring
            for digram in &digrams {
                if profile.digrams.contains(digram) {
                    score += 1.0;
                }
            }

            // Trigram scoring (higher weight)
            for trigram in &trigrams {
                if profile.trigrams.contains(trigram) {
                    score += 2.0;
                }
            }

            let total_ngrams = digrams.len() + trigrams.len();
            if total_ngrams > 0 {
                let normalized_score = score / total_ngrams as f32;
                *language_scores.get_mut(language).unwrap() += normalized_score * 30.0;
            }
        }
    }

    /// Analyze special character patterns and language-specific features
    fn analyze_special_patterns(&self, text: &str, language_scores: &mut HashMap<String, f32>) {
        // English patterns
        if text.contains("the ") || text.contains(" the ") || text.contains("and ") {
            *language_scores.get_mut("en").unwrap() += 10.0;
        }

        // Spanish patterns
        if text.contains("ñ") || text.contains("¿") || text.contains("¡") {
            *language_scores.get_mut("es").unwrap() += 20.0;
        }
        if text.contains(" el ") || text.contains(" la ") || text.contains(" de ") {
            *language_scores.get_mut("es").unwrap() += 5.0;
        }

        // French patterns
        if text.contains("ç") || text.contains("è") || text.contains("é") || text.contains("à") {
            *language_scores.get_mut("fr").unwrap() += 20.0;
        }
        if text.contains(" le ") || text.contains(" la ") || text.contains(" les ") {
            *language_scores.get_mut("fr").unwrap() += 5.0;
        }

        // German patterns
        if text.contains("ä") || text.contains("ö") || text.contains("ü") || text.contains("ß") {
            *language_scores.get_mut("de").unwrap() += 20.0;
        }
        if text.contains(" der ") || text.contains(" die ") || text.contains(" das ") {
            *language_scores.get_mut("de").unwrap() += 5.0;
        }

        // Italian patterns
        if text.contains(" il ") || text.contains(" la ") || text.contains(" gli ") {
            *language_scores.get_mut("it").unwrap() += 5.0;
        }

        // Portuguese patterns
        if text.contains("ã") || text.contains("õ") || text.contains("ç") {
            *language_scores.get_mut("pt").unwrap() += 15.0;
        }

        // Russian patterns (Cyrillic)
        if text.chars().any(|c| c >= 'а' && c <= 'я') || text.chars().any(|c| c >= 'А' && c <= 'Я') {
            *language_scores.get_mut("ru").unwrap() += 50.0;
        }

        // Chinese patterns
        if text.chars().any(|c| c >= '\u{4e00}' && c <= '\u{9fff}') {
            *language_scores.get_mut("zh").unwrap() += 50.0;
        }

        // Japanese patterns
        if text.chars().any(|c| (c >= '\u{3040}' && c <= '\u{309f}') || (c >= '\u{30a0}' && c <= '\u{30ff}')) {
            *language_scores.get_mut("ja").unwrap() += 50.0;
        }

        // Arabic patterns
        if text.chars().any(|c| c >= '\u{0600}' && c <= '\u{06ff}') {
            *language_scores.get_mut("ar").unwrap() += 50.0;
        }
    }

    /// Load language patterns and profiles
    fn load_language_patterns() -> HashMap<String, LanguageProfile> {
        let mut patterns = HashMap::new();

        // English
        patterns.insert("en".to_string(), LanguageProfile {
            common_words: vec![
                "the", "be", "to", "of", "and", "a", "in", "that", "have", "i",
                "it", "for", "not", "on", "with", "he", "as", "you", "do", "at",
                "this", "but", "his", "by", "from", "they", "we", "say", "her", "she",
                "or", "an", "will", "my", "one", "all", "would", "there", "their"
            ].into_iter().map(|s| s.to_string()).collect(),
            character_frequencies: [
                ('e', 0.127), ('t', 0.091), ('a', 0.082), ('o', 0.075), ('i', 0.070),
                ('n', 0.067), ('s', 0.063), ('h', 0.061), ('r', 0.060), ('d', 0.043)
            ].iter().cloned().collect(),
            digrams: vec!["th", "he", "in", "er", "an", "re", "ed", "nd", "on", "en"]
                .into_iter().map(|s| s.to_string()).collect(),
            trigrams: vec!["the", "and", "tha", "ent", "ing", "ion", "tio", "for", "nde", "has"]
                .into_iter().map(|s| s.to_string()).collect(),
        });

        // Spanish
        patterns.insert("es".to_string(), LanguageProfile {
            common_words: vec![
                "el", "la", "de", "que", "y", "a", "en", "un", "es", "se",
                "no", "te", "lo", "le", "da", "su", "por", "son", "con", "para",
                "al", "los", "del", "las", "una", "está", "todo", "pero", "más", "me"
            ].into_iter().map(|s| s.to_string()).collect(),
            character_frequencies: [
                ('a', 0.125), ('e', 0.124), ('o', 0.091), ('s', 0.080), ('r', 0.069),
                ('n', 0.067), ('i', 0.063), ('d', 0.058), ('l', 0.050), ('c', 0.047)
            ].iter().cloned().collect(),
            digrams: vec!["es", "en", "de", "la", "el", "ar", "er", "or", "an", "al"]
                .into_iter().map(|s| s.to_string()).collect(),
            trigrams: vec!["que", "ent", "ion", "con", "ado", "par", "los", "del", "las", "una"]
                .into_iter().map(|s| s.to_string()).collect(),
        });

        // French
        patterns.insert("fr".to_string(), LanguageProfile {
            common_words: vec![
                "le", "de", "et", "à", "un", "il", "être", "et", "en", "avoir",
                "que", "pour", "dans", "ce", "son", "une", "sur", "avec", "ne", "se",
                "pas", "par", "tout", "plus", "pouvoir", "dire", "vous", "je", "leur", "que"
            ].into_iter().map(|s| s.to_string()).collect(),
            character_frequencies: [
                ('e', 0.146), ('a', 0.081), ('i', 0.076), ('s', 0.076), ('n', 0.071),
                ('r', 0.066), ('t', 0.062), ('o', 0.054), ('l', 0.054), ('u', 0.054)
            ].iter().cloned().collect(),
            digrams: vec!["es", "en", "de", "le", "re", "nt", "on", "er", "te", "el"]
                .into_iter().map(|s| s.to_string()).collect(),
            trigrams: vec!["ent", "les", "des", "que", "une", "ion", "tio", "men", "our", "ait"]
                .into_iter().map(|s| s.to_string()).collect(),
        });

        // German
        patterns.insert("de".to_string(), LanguageProfile {
            common_words: vec![
                "der", "die", "und", "in", "den", "von", "zu", "das", "mit", "sich",
                "des", "auf", "für", "ist", "im", "dem", "nicht", "ein", "eine", "als",
                "auch", "es", "an", "werden", "aus", "er", "hat", "dass", "sie", "nach"
            ].into_iter().map(|s| s.to_string()).collect(),
            character_frequencies: [
                ('e', 0.174), ('n', 0.098), ('i', 0.076), ('s', 0.073), ('r', 0.070),
                ('a', 0.065), ('t', 0.061), ('d', 0.051), ('h', 0.048), ('u', 0.043)
            ].iter().cloned().collect(),
            digrams: vec!["en", "er", "ch", "de", "ei", "nd", "te", "in", "es", "ie"]
                .into_iter().map(|s| s.to_string()).collect(),
            trigrams: vec!["der", "und", "den", "die", "ich", "sch", "ein", "che", "ent", "ich"]
                .into_iter().map(|s| s.to_string()).collect(),
        });

        // Add more languages...
        patterns.insert("it".to_string(), LanguageProfile {
            common_words: vec!["il", "la", "di", "che", "e", "un", "in", "per", "con", "non"]
                .into_iter().map(|s| s.to_string()).collect(),
            character_frequencies: [
                ('e', 0.117), ('a', 0.111), ('i', 0.101), ('o', 0.095), ('n', 0.069)
            ].iter().cloned().collect(),
            digrams: vec!["er", "en", "ch", "de", "ei"].into_iter().map(|s| s.to_string()).collect(),
            trigrams: vec!["che", "ent", "ion"].into_iter().map(|s| s.to_string()).collect(),
        });

        patterns.insert("pt".to_string(), LanguageProfile {
            common_words: vec!["o", "a", "de", "e", "do", "da", "em", "um", "para", "é"]
                .into_iter().map(|s| s.to_string()).collect(),
            character_frequencies: [
                ('a', 0.146), ('e', 0.125), ('o', 0.103), ('s', 0.078), ('r', 0.065)
            ].iter().cloned().collect(),
            digrams: vec!["de", "os", "as", "es", "em"].into_iter().map(|s| s.to_string()).collect(),
            trigrams: vec!["que", "ent", "ade"].into_iter().map(|s| s.to_string()).collect(),
        });

        // Placeholder for non-Latin scripts (would need different analysis)
        patterns.insert("ru".to_string(), LanguageProfile {
            common_words: vec!["и", "в", "не", "на", "я", "быть", "с", "что", "а", "по"]
                .into_iter().map(|s| s.to_string()).collect(),
            character_frequencies: HashMap::new(),
            digrams: Vec::new(),
            trigrams: Vec::new(),
        });

        patterns.insert("zh".to_string(), LanguageProfile {
            common_words: vec!["的", "一", "是", "在", "不", "了", "有", "和", "人", "这"]
                .into_iter().map(|s| s.to_string()).collect(),
            character_frequencies: HashMap::new(),
            digrams: Vec::new(),
            trigrams: Vec::new(),
        });

        patterns.insert("ja".to_string(), LanguageProfile {
            common_words: vec!["の", "に", "は", "を", "た", "が", "で", "て", "と", "し"]
                .into_iter().map(|s| s.to_string()).collect(),
            character_frequencies: HashMap::new(),
            digrams: Vec::new(),
            trigrams: Vec::new(),
        });

        patterns.insert("ar".to_string(), LanguageProfile {
            common_words: vec!["في", "من", "إلى", "على", "هذا", "أن", "كان", "قد", "لا", "ما"]
                .into_iter().map(|s| s.to_string()).collect(),
            character_frequencies: HashMap::new(),
            digrams: Vec::new(),
            trigrams: Vec::new(),
        });

        patterns
    }
}

impl Default for LanguageDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_english_detection() {
        let detector = LanguageDetector::new();
        let english_text = "This is a sample English text with common English words like the, and, in, for.";
        
        let detected = detector.detect_language(english_text).await.unwrap();
        assert_eq!(detected, "en");
    }

    #[tokio::test]
    async fn test_spanish_detection() {
        let detector = LanguageDetector::new();
        let spanish_text = "Este es un texto en español con palabras comunes como el, la, de, que, y.";
        
        let detected = detector.detect_language(spanish_text).await.unwrap();
        assert_eq!(detected, "es");
    }

    #[tokio::test]
    async fn test_french_detection() {
        let detector = LanguageDetector::new();
        let french_text = "Ceci est un texte en français avec des mots communs comme le, de, et, à, un.";
        
        let detected = detector.detect_language(french_text).await.unwrap();
        assert_eq!(detected, "fr");
    }

    #[tokio::test]
    async fn test_empty_text() {
        let detector = LanguageDetector::new();
        let empty_text = "";
        
        let detected = detector.detect_language(empty_text).await.unwrap();
        assert_eq!(detected, "unknown");
    }

    #[tokio::test]
    async fn test_mixed_language_text() {
        let detector = LanguageDetector::new();
        // Text with predominantly English words
        let mixed_text = "This text has some español words but is mostly English with the and words.";
        
        let detected = detector.detect_language(mixed_text).await.unwrap();
        assert_eq!(detected, "en");
    }
}