//! Plain text parser with metadata extraction

use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

use crate::models::*;
use crate::traits::ContentParser;

/// Parser for plain text content
pub struct TextParser {
    name: &'static str,
}

impl TextParser {
    pub fn new() -> Self {
        Self {
            name: "text_parser",
        }
    }
    
    /// Extract basic metadata from text content
    fn extract_metadata(&self, text: &str, content_size: u64) -> ContentMetadata {
        let lines: Vec<&str> = text.lines().collect();
        let mut title = None;
        
        // Try to extract title from first non-empty line
        for line in &lines {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                title = Some(trimmed.to_string());
                break;
            }
        }
        
        // Detect encoding (assume UTF-8 for text)
        let encoding = Some("utf-8".to_string());
        
        ContentMetadata {
            id: Uuid::new_v4(),
            content_type: ContentType::PlainText,
            size_bytes: content_size,
            title,
            author: None,
            source_url: None,
            created_at: Some(Utc::now()),
            last_modified: Some(Utc::now()),
            encoding,
            mime_type: Some("text/plain".to_string()),
            language: None, // Will be detected separately
            version: None,
            tags: Vec::new(),
            custom_fields: HashMap::new(),
        }
    }
    
    /// Extract basic structure from text
    fn extract_structure(&self, text: &str) -> ContentStructure {
        let _lines: Vec<&str> = text.lines().collect();
        let mut sections = Vec::new();
        let mut current_pos = 0u32;
        
        // Create a single section for the entire text
        let section = ContentSection {
            id: "main".to_string(),
            title: None,
            level: 1,
            content: text.to_string(),
            start_position: 0,
            end_position: text.len() as u32,
            subsections: Vec::new(),
        };
        sections.push(section);
        
        // Extract paragraphs as subsections
        let mut paragraph_sections = Vec::new();
        let paragraphs: Vec<&str> = text.split("\n\n").collect();
        
        for (i, paragraph) in paragraphs.iter().enumerate() {
            if !paragraph.trim().is_empty() {
                let para_section = ContentSection {
                    id: format!("paragraph_{}", i),
                    title: None,
                    level: 2,
                    content: paragraph.to_string(),
                    start_position: current_pos,
                    end_position: current_pos + paragraph.len() as u32,
                    subsections: Vec::new(),
                };
                paragraph_sections.push(para_section);
                current_pos += paragraph.len() as u32 + 2; // +2 for \n\n
            }
        }
        
        // Add paragraph sections to main section
        if let Some(main_section) = sections.get_mut(0) {
            main_section.subsections = paragraph_sections;
        }
        
        ContentStructure {
            sections,
            table_of_contents: Vec::new(), // No TOC for plain text
            links: self.extract_links(text),
            citations: Vec::new(), // No formal citations in plain text
        }
    }
    
    /// Extract URLs from text
    fn extract_links(&self, text: &str) -> Vec<Link> {
        let mut links = Vec::new();
        let url_regex = regex::Regex::new(r"https?://[^\s]+").unwrap();
        
        for mat in url_regex.find_iter(text) {
            let url = mat.as_str().to_string();
            let link = Link {
                url: url.clone(),
                text: Some(url.clone()),
                link_type: LinkType::External,
                position: mat.start() as u32,
            };
            links.push(link);
        }
        
        links
    }
}

impl Default for TextParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ContentParser for TextParser {
    async fn parse(&self, raw_content: &[u8]) -> crate::Result<ParsedContent> {
        // Convert bytes to string
        let text = String::from_utf8(raw_content.to_vec())
            .map_err(|e| ProcessingError::ParseError {
                message: format!("Invalid UTF-8 encoding: {}", e),
                position: None,
            })?;
        
        let content_size = raw_content.len() as u64;
        let metadata = self.extract_metadata(&text, content_size);
        let structure = self.extract_structure(&text);
        
        Ok(ParsedContent {
            content_type: ContentType::PlainText,
            text: text.clone(),
            metadata,
            structure,
            media_elements: Vec::new(), // No media in plain text
        })
    }
    
    fn supports(&self, content_type: &ContentType) -> bool {
        matches!(content_type, ContentType::PlainText | ContentType::Code)
    }
    
    fn name(&self) -> &'static str {
        self.name
    }
    
    fn estimate_parsing_time(&self, content_size_bytes: u64) -> std::time::Duration {
        // Very fast for text - 0.1ms per KB
        std::time::Duration::from_micros((content_size_bytes / 10).max(100))
    }
    
    fn validate_content(&self, raw_content: &[u8], _content_type: &ContentType) -> crate::Result<()> {
        // Try to parse as UTF-8
        String::from_utf8(raw_content.to_vec())
            .map_err(|e| ProcessingError::ValidationError {
                field: "content".to_string(),
                message: format!("Content must be valid UTF-8: {}", e),
            })?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_simple_text() {
        let parser = TextParser::new();
        let content = b"This is a simple text document.\n\nWith multiple paragraphs.";
        
        let result = parser.parse(content).await.unwrap();
        
        assert_eq!(result.content_type, ContentType::PlainText);
        assert_eq!(result.text, "This is a simple text document.\n\nWith multiple paragraphs.");
        assert_eq!(result.metadata.content_type, ContentType::PlainText);
        assert_eq!(result.metadata.size_bytes, content.len() as u64);
        assert_eq!(result.metadata.mime_type, Some("text/plain".to_string()));
        assert!(result.structure.sections.len() > 0);
    }
    
    #[tokio::test]
    async fn test_parse_text_with_links() {
        let parser = TextParser::new();
        let content = b"Visit https://example.com for more info.";
        
        let result = parser.parse(content).await.unwrap();
        
        assert_eq!(result.structure.links.len(), 1);
        assert_eq!(result.structure.links[0].url, "https://example.com");
        assert_eq!(result.structure.links[0].link_type, LinkType::External);
    }
    
    #[test]
    fn test_supports() {
        let parser = TextParser::new();
        assert!(parser.supports(&ContentType::PlainText));
        assert!(parser.supports(&ContentType::Code));
        assert!(!parser.supports(&ContentType::Html));
    }
    
    #[tokio::test]
    async fn test_validate_content() {
        let parser = TextParser::new();
        
        // Valid UTF-8
        assert!(parser.validate_content(b"Valid text", &ContentType::PlainText).is_ok());
        
        // Invalid UTF-8
        let invalid_utf8 = vec![0xFF, 0xFE];
        assert!(parser.validate_content(&invalid_utf8, &ContentType::PlainText).is_err());
    }
}