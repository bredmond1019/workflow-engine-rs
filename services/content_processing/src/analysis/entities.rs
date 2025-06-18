//! Named Entity Recognition (NER)
//!
//! This module implements entity recognition to identify and classify
//! named entities in text such as:
//! - Persons, Organizations, Locations
//! - Dates, Times, Money amounts
//! - Technologies, Concepts
//! - Custom domain-specific entities

use std::collections::HashMap;
use regex::Regex;

use crate::models::*;
use crate::ai_integration::AIContentAnalyzer;

/// Named entity recognizer using AI and pattern matching methods
pub struct EntityRecognizer {
    name: &'static str,
    ai_analyzer: AIContentAnalyzer,
    person_patterns: Vec<Regex>,
    organization_patterns: Vec<Regex>,
    location_patterns: Vec<Regex>,
    date_patterns: Vec<Regex>,
    money_patterns: Vec<Regex>,
    technology_keywords: Vec<String>,
}

impl EntityRecognizer {
    pub fn new() -> Self {
        Self {
            name: "entity_recognizer",
            ai_analyzer: AIContentAnalyzer::new(),
            person_patterns: Self::compile_person_patterns(),
            organization_patterns: Self::compile_organization_patterns(),
            location_patterns: Self::compile_location_patterns(),
            date_patterns: Self::compile_date_patterns(),
            money_patterns: Self::compile_money_patterns(),
            technology_keywords: Self::load_technology_keywords(),
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Extract named entities from text
    pub async fn extract_entities(
        &self,
        text: &str,
        _context: &ProcessingContext,
    ) -> crate::Result<Vec<Entity>> {
        // Try AI-powered entity extraction first
        match self.ai_analyzer.extract_entities(text).await {
            Ok(ai_entities) => {
                if !ai_entities.is_empty() {
                    return Ok(ai_entities);
                }
            }
            Err(_) => {
                // Continue with pattern-based fallback
            }
        }

        // Fallback to pattern-based extraction
        self.pattern_based_extraction(text).await
    }

    /// Pattern-based entity extraction as fallback
    async fn pattern_based_extraction(&self, text: &str) -> crate::Result<Vec<Entity>> {
        let mut entities = Vec::new();
        
        // Extract different types of entities
        entities.extend(self.extract_persons(text));
        entities.extend(self.extract_organizations(text));
        entities.extend(self.extract_locations(text));
        entities.extend(self.extract_dates(text));
        entities.extend(self.extract_money(text));
        entities.extend(self.extract_technologies(text));
        entities.extend(self.extract_concepts(text));
        
        // Remove duplicates and merge overlapping entities
        entities = self.deduplicate_entities(entities);
        
        Ok(entities)
    }

    /// Extract person names
    fn extract_persons(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();
        
        for pattern in &self.person_patterns {
            for mat in pattern.find_iter(text) {
                let name = mat.as_str().trim();
                if self.is_likely_person_name(name) {
                    entities.push(Entity {
                        name: name.to_string(),
                        entity_type: EntityType::Person,
                        confidence: 0.7,
                        mentions: vec![EntityMention {
                            position: mat.start() as u32,
                            context: self.extract_context(text, mat.start(), mat.end()),
                            confidence: 0.7,
                        }],
                        linked_data_uri: None,
                    });
                }
            }
        }
        
        // Look for titles that indicate persons
        let title_pattern = Regex::new(r"\b(?:Dr|Mr|Mrs|Ms|Prof|Professor|CEO|CTO|President|Director)\.?\s+([A-Z][a-z]+(?: [A-Z][a-z]+)*)").unwrap();
        for mat in title_pattern.find_iter(text) {
            let full_match = mat.as_str();
            let name_part = full_match.split_whitespace().skip(1).collect::<Vec<_>>().join(" ");
            
            entities.push(Entity {
                name: name_part.clone(),
                entity_type: EntityType::Person,
                confidence: 0.9, // High confidence due to title
                mentions: vec![EntityMention {
                    position: mat.start() as u32,
                    context: self.extract_context(text, mat.start(), mat.end()),
                    confidence: 0.9,
                }],
                linked_data_uri: None,
            });
        }
        
        entities
    }

    /// Extract organization names
    fn extract_organizations(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();
        
        for pattern in &self.organization_patterns {
            for mat in pattern.find_iter(text) {
                let name = mat.as_str().trim();
                entities.push(Entity {
                    name: name.to_string(),
                    entity_type: EntityType::Organization,
                    confidence: 0.8,
                    mentions: vec![EntityMention {
                        position: mat.start() as u32,
                        context: self.extract_context(text, mat.start(), mat.end()),
                        confidence: 0.8,
                    }],
                    linked_data_uri: None,
                });
            }
        }
        
        // Look for company suffixes
        let company_pattern = Regex::new(r"\b([A-Z][a-zA-Z\s&]+)(?:\s+(?:Inc|Corp|Corporation|LLC|Ltd|Limited|Company|Co|Group|Systems|Technologies|Solutions|Services))\b").unwrap();
        for mat in company_pattern.find_iter(text) {
            let full_name = mat.as_str().trim();
            entities.push(Entity {
                name: full_name.to_string(),
                entity_type: EntityType::Organization,
                confidence: 0.85,
                mentions: vec![EntityMention {
                    position: mat.start() as u32,
                    context: self.extract_context(text, mat.start(), mat.end()),
                    confidence: 0.85,
                }],
                linked_data_uri: None,
            });
        }
        
        entities
    }

    /// Extract location names
    fn extract_locations(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();
        
        for pattern in &self.location_patterns {
            for mat in pattern.find_iter(text) {
                let name = mat.as_str().trim();
                if self.is_likely_location(name) {
                    entities.push(Entity {
                        name: name.to_string(),
                        entity_type: EntityType::Location,
                        confidence: 0.75,
                        mentions: vec![EntityMention {
                            position: mat.start() as u32,
                            context: self.extract_context(text, mat.start(), mat.end()),
                            confidence: 0.75,
                        }],
                        linked_data_uri: None,
                    });
                }
            }
        }
        
        entities
    }

    /// Extract dates and times
    fn extract_dates(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();
        
        for pattern in &self.date_patterns {
            for mat in pattern.find_iter(text) {
                let date_str = mat.as_str().trim();
                entities.push(Entity {
                    name: date_str.to_string(),
                    entity_type: EntityType::Date,
                    confidence: 0.9,
                    mentions: vec![EntityMention {
                        position: mat.start() as u32,
                        context: self.extract_context(text, mat.start(), mat.end()),
                        confidence: 0.9,
                    }],
                    linked_data_uri: None,
                });
            }
        }
        
        entities
    }

    /// Extract monetary amounts
    fn extract_money(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();
        
        for pattern in &self.money_patterns {
            for mat in pattern.find_iter(text) {
                let money_str = mat.as_str().trim();
                entities.push(Entity {
                    name: money_str.to_string(),
                    entity_type: EntityType::Money,
                    confidence: 0.95,
                    mentions: vec![EntityMention {
                        position: mat.start() as u32,
                        context: self.extract_context(text, mat.start(), mat.end()),
                        confidence: 0.95,
                    }],
                    linked_data_uri: None,
                });
            }
        }
        
        entities
    }

    /// Extract technology-related entities
    fn extract_technologies(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();
        
        for tech_keyword in &self.technology_keywords {
            if let Some(pos) = text.find(tech_keyword) {
                entities.push(Entity {
                    name: tech_keyword.clone(),
                    entity_type: EntityType::Technology,
                    confidence: 0.8,
                    mentions: vec![EntityMention {
                        position: pos as u32,
                        context: self.extract_context(text, pos, pos + tech_keyword.len()),
                        confidence: 0.8,
                    }],
                    linked_data_uri: None,
                });
            }
        }
        
        // Look for programming languages and frameworks
        let tech_pattern = Regex::new(r"\b(?:JavaScript|TypeScript|Python|Java|C\+\+|C#|Ruby|Go|Rust|Swift|Kotlin|PHP|React|Angular|Vue|Django|Flask|Spring|Express|Node\.js|TensorFlow|PyTorch|Docker|Kubernetes)\b").unwrap();
        for mat in tech_pattern.find_iter(text) {
            let tech_name = mat.as_str();
            entities.push(Entity {
                name: tech_name.to_string(),
                entity_type: EntityType::Technology,
                confidence: 0.9,
                mentions: vec![EntityMention {
                    position: mat.start() as u32,
                    context: self.extract_context(text, mat.start(), mat.end()),
                    confidence: 0.9,
                }],
                linked_data_uri: None,
            });
        }
        
        entities
    }

    /// Extract conceptual entities
    fn extract_concepts(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();
        
        // Look for academic/scientific concepts
        let concept_indicators = vec![
            "algorithm", "methodology", "framework", "paradigm", "theory",
            "principle", "approach", "technique", "method", "process",
            "model", "system", "architecture", "design", "pattern"
        ];
        
        for indicator in concept_indicators {
            // Look for the indicator with context
            let pattern = Regex::new(&format!(r"\b([A-Z][a-z]*(?:\s+[A-Z][a-z]*)*\s+{}|{}\s+[a-z]+(?:\s+[a-z]+)*)\b", indicator, indicator)).unwrap();
            for mat in pattern.find_iter(text) {
                let concept_name = mat.as_str().trim();
                if concept_name.len() > indicator.len() + 2 { // Ensure it's not just the indicator
                    entities.push(Entity {
                        name: concept_name.to_string(),
                        entity_type: EntityType::Concept,
                        confidence: 0.7,
                        mentions: vec![EntityMention {
                            position: mat.start() as u32,
                            context: self.extract_context(text, mat.start(), mat.end()),
                            confidence: 0.7,
                        }],
                        linked_data_uri: None,
                    });
                }
            }
        }
        
        entities
    }

    /// Remove duplicate entities and merge similar ones
    fn deduplicate_entities(&self, entities: Vec<Entity>) -> Vec<Entity> {
        let mut unique_entities: HashMap<(String, EntityType), Entity> = HashMap::new();
        
        for entity in entities {
            let key = (entity.name.to_lowercase(), entity.entity_type.clone());
            
            if let Some(existing) = unique_entities.get_mut(&key) {
                // Merge mentions
                existing.mentions.extend(entity.mentions.clone());
                // Use higher confidence
                if entity.confidence > existing.confidence {
                    existing.confidence = entity.confidence;
                }
            } else {
                unique_entities.insert(key, entity);
            }
        }
        
        unique_entities.into_values().collect()
    }

    /// Extract context around an entity mention
    fn extract_context(&self, text: &str, start: usize, end: usize) -> String {
        let context_window = 50;
        let context_start = start.saturating_sub(context_window);
        let context_end = (end + context_window).min(text.len());
        
        text[context_start..context_end].to_string()
    }

    /// Check if a name is likely a person name
    fn is_likely_person_name(&self, name: &str) -> bool {
        let words: Vec<&str> = name.split_whitespace().collect();
        
        // Basic heuristics for person names
        words.len() >= 2 && words.len() <= 4 && // Reasonable number of words
        words.iter().all(|word| word.chars().next().map_or(false, |c| c.is_uppercase())) && // Title case
        words.iter().all(|word| word.len() >= 2) && // Reasonable word length
        !name.to_lowercase().contains("the") && // Not likely to be an organization
        !name.to_lowercase().contains("of") &&
        !name.to_lowercase().contains("and")
    }

    /// Check if a name is likely a location
    fn is_likely_location(&self, name: &str) -> bool {
        let location_indicators = vec![
            "city", "state", "country", "county", "province", "region",
            "district", "area", "zone", "territory", "island", "mountain",
            "river", "lake", "ocean", "sea", "bay", "valley", "forest"
        ];
        
        let name_lower = name.to_lowercase();
        location_indicators.iter().any(|indicator| name_lower.contains(indicator)) ||
        name.chars().next().map_or(false, |c| c.is_uppercase()) // Proper noun
    }

    /// Compile person name patterns
    fn compile_person_patterns() -> Vec<Regex> {
        vec![
            // First Last
            Regex::new(r"\b[A-Z][a-z]+ [A-Z][a-z]+\b").unwrap(),
            // First Middle Last
            Regex::new(r"\b[A-Z][a-z]+ [A-Z][a-z]+ [A-Z][a-z]+\b").unwrap(),
            // First M. Last (with middle initial)
            Regex::new(r"\b[A-Z][a-z]+ [A-Z]\. [A-Z][a-z]+\b").unwrap(),
        ]
    }

    /// Compile organization patterns
    fn compile_organization_patterns() -> Vec<Regex> {
        vec![
            // Organizations with common suffixes
            Regex::new(r"\b[A-Z][a-zA-Z\s&]+ (?:Inc|Corp|Corporation|LLC|Ltd|Limited|Company|Co|Group|Institute|Foundation|Association|Organization|Society)\b").unwrap(),
            // Universities
            Regex::new(r"\b(?:University of [A-Z][a-z]+|[A-Z][a-z]+ University)\b").unwrap(),
        ]
    }

    /// Compile location patterns
    fn compile_location_patterns() -> Vec<Regex> {
        vec![
            // Cities, states, countries (simple pattern)
            Regex::new(r"\b[A-Z][a-z]+(?:,\s*[A-Z][a-z]+)*\b").unwrap(),
        ]
    }

    /// Compile date patterns
    fn compile_date_patterns() -> Vec<Regex> {
        vec![
            // MM/DD/YYYY
            Regex::new(r"\b\d{1,2}/\d{1,2}/\d{4}\b").unwrap(),
            // Month DD, YYYY
            Regex::new(r"\b(?:January|February|March|April|May|June|July|August|September|October|November|December) \d{1,2}, \d{4}\b").unwrap(),
            // DD Month YYYY
            Regex::new(r"\b\d{1,2} (?:January|February|March|April|May|June|July|August|September|October|November|December) \d{4}\b").unwrap(),
            // YYYY-MM-DD
            Regex::new(r"\b\d{4}-\d{2}-\d{2}\b").unwrap(),
        ]
    }

    /// Compile money patterns
    fn compile_money_patterns() -> Vec<Regex> {
        vec![
            // $1,000.00
            Regex::new(r"\$\d{1,3}(?:,\d{3})*(?:\.\d{2})?\b").unwrap(),
            // $1M, $1B, $1K
            Regex::new(r"\$\d+(?:\.\d+)?[KMB]\b").unwrap(),
            // USD amounts
            Regex::new(r"\b\d+(?:,\d{3})*(?:\.\d{2})?\s*USD\b").unwrap(),
        ]
    }

    /// Load technology keywords
    fn load_technology_keywords() -> Vec<String> {
        vec![
            "artificial intelligence", "machine learning", "deep learning",
            "neural network", "natural language processing", "computer vision",
            "blockchain", "cryptocurrency", "cloud computing", "big data",
            "data science", "software engineering", "web development",
            "mobile development", "database", "API", "microservices",
            "DevOps", "cybersecurity", "quantum computing", "IoT",
            "augmented reality", "virtual reality", "robotics"
        ].into_iter().map(|s| s.to_string()).collect()
    }
}

impl Default for EntityRecognizer {
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
    async fn test_entity_extraction() {
        let recognizer = EntityRecognizer::new();
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

        let text = "John Smith works at Google Inc. in Mountain View, California. \
                   He earned $150,000 last year and specializes in machine learning. \
                   The project started on January 15, 2023.";

        let entities = recognizer.extract_entities(text, &context).await.unwrap();
        
        assert!(!entities.is_empty());
        
        // Check for different entity types
        let entity_types: Vec<_> = entities.iter().map(|e| &e.entity_type).collect();
        assert!(entity_types.iter().any(|t| matches!(t, EntityType::Person)));
        assert!(entity_types.iter().any(|t| matches!(t, EntityType::Money)));
        assert!(entity_types.iter().any(|t| matches!(t, EntityType::Date)));
    }

    #[test]
    fn test_person_name_validation() {
        let recognizer = EntityRecognizer::new();
        assert!(recognizer.is_likely_person_name("John Smith"));
        assert!(recognizer.is_likely_person_name("Mary Jane Watson"));
        assert!(!recognizer.is_likely_person_name("The Company"));
        assert!(!recognizer.is_likely_person_name("Department of Defense"));
    }

    #[test]
    fn test_location_validation() {
        let recognizer = EntityRecognizer::new();
        assert!(recognizer.is_likely_location("New York City"));
        assert!(recognizer.is_likely_location("California"));
        assert!(recognizer.is_likely_location("Mountain View"));
    }

    #[test]
    fn test_context_extraction() {
        let recognizer = EntityRecognizer::new();
        let text = "This is a test document with some content for context extraction.";
        let context = recognizer.extract_context(text, 10, 14); // "test"
        
        assert!(context.contains("test"));
        assert!(context.len() <= text.len());
    }
}