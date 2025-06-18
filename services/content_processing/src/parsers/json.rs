//! JSON parser with structured data analysis

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

use crate::models::*;
use crate::traits::ContentParser;

/// Parser for JSON content
pub struct JsonParser {
    name: &'static str,
}

impl JsonParser {
    pub fn new() -> Self {
        Self {
            name: "json_parser",
        }
    }
    
    /// Extract text content from JSON values recursively
    fn extract_text(&self, value: &Value) -> String {
        let mut text_parts = Vec::new();
        self.extract_text_recursive(value, &mut text_parts);
        text_parts.join(" ")
    }
    
    fn extract_text_recursive(&self, value: &Value, text_parts: &mut Vec<String>) {
        match value {
            Value::String(s) => {
                if !s.trim().is_empty() {
                    text_parts.push(s.clone());
                }
            },
            Value::Array(arr) => {
                for item in arr {
                    self.extract_text_recursive(item, text_parts);
                }
            },
            Value::Object(obj) => {
                for (key, val) in obj {
                    // Include meaningful keys as context
                    if self.is_meaningful_key(key) {
                        text_parts.push(key.clone());
                    }
                    self.extract_text_recursive(val, text_parts);
                }
            },
            Value::Number(n) => {
                text_parts.push(n.to_string());
            },
            Value::Bool(b) => {
                text_parts.push(b.to_string());
            },
            Value::Null => {}, // Skip null values
        }
    }
    
    /// Check if a JSON key is meaningful for text extraction
    fn is_meaningful_key(&self, key: &str) -> bool {
        let meaningful_keys = [
            "title", "name", "description", "content", "text", "body", 
            "summary", "label", "caption", "message", "note", "comment"
        ];
        
        let key_lower = key.to_lowercase();
        meaningful_keys.iter().any(|&mk| key_lower.contains(mk))
    }
    
    /// Extract metadata from JSON structure
    fn extract_metadata(&self, value: &Value, content_size: u64) -> ContentMetadata {
        let mut title = None;
        let mut author = None;
        let mut custom_fields = HashMap::new();
        
        if let Value::Object(obj) = value {
            // Look for common metadata fields
            for (key, val) in obj {
                let key_lower = key.to_lowercase();
                match key_lower.as_str() {
                    "title" | "name" => {
                        if let Value::String(s) = val {
                            title = Some(s.clone());
                        }
                    },
                    "author" | "creator" | "by" => {
                        if let Value::String(s) = val {
                            author = Some(s.clone());
                        }
                    },
                    "description" | "summary" | "abstract" => {
                        if let Value::String(s) = val {
                            custom_fields.insert(key.clone(), serde_json::Value::String(s.clone()));
                        }
                    },
                    _ => {
                        // Store other fields as custom metadata
                        if matches!(val, Value::String(_) | Value::Number(_) | Value::Bool(_)) {
                            custom_fields.insert(key.clone(), val.clone());
                        }
                    }
                }
            }
        }
        
        ContentMetadata {
            id: Uuid::new_v4(),
            content_type: ContentType::Json,
            size_bytes: content_size,
            title,
            author,
            source_url: None,
            created_at: Some(Utc::now()),
            last_modified: Some(Utc::now()),
            encoding: Some("utf-8".to_string()),
            mime_type: Some("application/json".to_string()),
            language: None,
            version: None,
            tags: Vec::new(),
            custom_fields,
        }
    }
    
    /// Extract structure from JSON
    fn extract_structure(&self, value: &Value) -> ContentStructure {
        let mut sections = Vec::new();
        let mut current_pos = 0u32;
        
        self.extract_structure_recursive(value, &mut sections, &mut current_pos, 1, "root");
        
        ContentStructure {
            sections,
            table_of_contents: Vec::new(), // No standard TOC for JSON
            links: self.extract_links(value),
            citations: Vec::new(),
        }
    }
    
    fn extract_structure_recursive(
        &self, 
        value: &Value, 
        sections: &mut Vec<ContentSection>, 
        current_pos: &mut u32, 
        level: u32, 
        parent_key: &str
    ) {
        match value {
            Value::Object(obj) => {
                for (key, val) in obj {
                    let section_id = format!("{}_{}", parent_key, key);
                    let content = self.value_to_text_preview(val);
                    let start_pos = *current_pos;
                    *current_pos += content.len() as u32;
                    
                    let mut subsections = Vec::new();
                    if matches!(val, Value::Object(_) | Value::Array(_)) {
                        self.extract_structure_recursive(val, &mut subsections, current_pos, level + 1, &section_id);
                    }
                    
                    sections.push(ContentSection {
                        id: section_id,
                        title: Some(key.clone()),
                        level,
                        content,
                        start_position: start_pos,
                        end_position: *current_pos,
                        subsections,
                    });
                }
            },
            Value::Array(arr) => {
                for (i, item) in arr.iter().enumerate() {
                    let section_id = format!("{}_item_{}", parent_key, i);
                    let content = self.value_to_text_preview(item);
                    let start_pos = *current_pos;
                    *current_pos += content.len() as u32;
                    
                    let mut subsections = Vec::new();
                    if matches!(item, Value::Object(_) | Value::Array(_)) {
                        self.extract_structure_recursive(item, &mut subsections, current_pos, level + 1, &section_id);
                    }
                    
                    sections.push(ContentSection {
                        id: section_id,
                        title: Some(format!("Item {}", i)),
                        level,
                        content,
                        start_position: start_pos,
                        end_position: *current_pos,
                        subsections,
                    });
                }
            },
            _ => {}, // Primitive values handled by parent
        }
    }
    
    /// Convert a JSON value to a text preview
    fn value_to_text_preview(&self, value: &Value) -> String {
        match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::Array(arr) => format!("Array with {} items", arr.len()),
            Value::Object(obj) => format!("Object with {} fields", obj.len()),
        }
    }
    
    /// Extract URLs from JSON values
    fn extract_links(&self, value: &Value) -> Vec<Link> {
        let mut links = Vec::new();
        self.extract_links_recursive(value, &mut links, 0);
        links
    }
    
    fn extract_links_recursive(&self, value: &Value, links: &mut Vec<Link>, position: u32) {
        match value {
            Value::String(s) => {
                // Check if string looks like a URL
                if s.starts_with("http://") || s.starts_with("https://") {
                    links.push(Link {
                        url: s.clone(),
                        text: Some(s.clone()),
                        link_type: LinkType::External,
                        position,
                    });
                } else if s.starts_with("mailto:") {
                    links.push(Link {
                        url: s.clone(),
                        text: Some(s.clone()),
                        link_type: LinkType::Email,
                        position,
                    });
                }
            },
            Value::Array(arr) => {
                for item in arr {
                    self.extract_links_recursive(item, links, position);
                }
            },
            Value::Object(obj) => {
                for (key, val) in obj {
                    // Look for URL-like keys
                    let key_lower = key.to_lowercase();
                    if key_lower.contains("url") || key_lower.contains("link") || key_lower.contains("href") {
                        if let Value::String(url) = val {
                            let link_type = if url.starts_with("http") {
                                LinkType::External
                            } else if url.starts_with("mailto:") {
                                LinkType::Email
                            } else {
                                LinkType::File
                            };
                            
                            links.push(Link {
                                url: url.clone(),
                                text: Some(url.clone()),
                                link_type,
                                position,
                            });
                        }
                    }
                    self.extract_links_recursive(val, links, position);
                }
            },
            _ => {},
        }
    }
}

impl Default for JsonParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ContentParser for JsonParser {
    async fn parse(&self, raw_content: &[u8]) -> crate::Result<ParsedContent> {
        // Convert bytes to string
        let json_content = String::from_utf8(raw_content.to_vec())
            .map_err(|e| ProcessingError::ParseError {
                message: format!("Invalid UTF-8 encoding: {}", e),
                position: None,
            })?;
        
        // Parse JSON
        let value: Value = serde_json::from_str(&json_content)
            .map_err(|e| ProcessingError::ParseError {
                message: format!("Invalid JSON: {}", e),
                position: Some(e.line() as u32),
            })?;
        
        // Extract components
        let text = self.extract_text(&value);
        let content_size = raw_content.len() as u64;
        let metadata = self.extract_metadata(&value, content_size);
        let structure = self.extract_structure(&value);
        
        Ok(ParsedContent {
            content_type: ContentType::Json,
            text,
            metadata,
            structure,
            media_elements: Vec::new(), // No media in pure JSON
        })
    }
    
    fn supports(&self, content_type: &ContentType) -> bool {
        matches!(content_type, ContentType::Json)
    }
    
    fn name(&self) -> &'static str {
        self.name
    }
    
    fn estimate_parsing_time(&self, content_size_bytes: u64) -> std::time::Duration {
        // JSON parsing is fast - 0.5ms per KB
        std::time::Duration::from_micros((content_size_bytes / 2).max(100))
    }
    
    fn validate_content(&self, raw_content: &[u8], _content_type: &ContentType) -> crate::Result<()> {
        // Try to parse as UTF-8
        let json_content = String::from_utf8(raw_content.to_vec())
            .map_err(|e| ProcessingError::ValidationError {
                field: "content".to_string(),
                message: format!("Content must be valid UTF-8: {}", e),
            })?;
        
        // Validate JSON syntax
        serde_json::from_str::<Value>(&json_content)
            .map_err(|e| ProcessingError::ValidationError {
                field: "content".to_string(),
                message: format!("Invalid JSON syntax: {}", e),
            })?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_simple_json() {
        let parser = JsonParser::new();
        let content = br#"{"title": "Test Document", "content": "This is test content", "author": "John Doe"}"#;
        
        let result = parser.parse(content).await.unwrap();
        
        assert_eq!(result.content_type, ContentType::Json);
        assert!(result.text.contains("Test Document"));
        assert!(result.text.contains("This is test content"));
        assert_eq!(result.metadata.title, Some("Test Document".to_string()));
        assert_eq!(result.metadata.author, Some("John Doe".to_string()));
    }
    
    #[tokio::test]
    async fn test_parse_json_array() {
        let parser = JsonParser::new();
        let content = br#"[{"name": "Item 1"}, {"name": "Item 2"}]"#;
        
        let result = parser.parse(content).await.unwrap();
        
        assert_eq!(result.content_type, ContentType::Json);
        assert!(result.text.contains("Item 1"));
        assert!(result.text.contains("Item 2"));
        assert!(result.structure.sections.len() >= 2);
    }
    
    #[tokio::test]
    async fn test_parse_json_with_urls() {
        let parser = JsonParser::new();
        let content = br#"{"website": "https://example.com", "email": "mailto:test@example.com"}"#;
        
        let result = parser.parse(content).await.unwrap();
        
        assert_eq!(result.structure.links.len(), 2);
        assert_eq!(result.structure.links[0].link_type, LinkType::External);
        assert_eq!(result.structure.links[1].link_type, LinkType::Email);
    }
    
    #[test]
    fn test_supports() {
        let parser = JsonParser::new();
        assert!(parser.supports(&ContentType::Json));
        assert!(!parser.supports(&ContentType::Html));
    }
    
    #[tokio::test]
    async fn test_validate_content() {
        let parser = JsonParser::new();
        
        // Valid JSON
        assert!(parser.validate_content(br#"{"valid": true}"#, &ContentType::Json).is_ok());
        
        // Invalid JSON
        assert!(parser.validate_content(b"{invalid json", &ContentType::Json).is_err());
        
        // Invalid UTF-8
        let invalid_utf8 = vec![0xFF, 0xFE];
        assert!(parser.validate_content(&invalid_utf8, &ContentType::Json).is_err());
    }
}