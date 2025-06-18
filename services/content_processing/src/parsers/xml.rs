//! XML parser with schema-aware processing

use async_trait::async_trait;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

use crate::models::*;
use crate::traits::ContentParser;

/// Parser for XML content
pub struct XmlParser {
    name: &'static str,
}

impl XmlParser {
    pub fn new() -> Self {
        Self {
            name: "xml_parser",
        }
    }
    
    /// Extract text content from XML
    fn extract_text(&self, xml_content: &str) -> crate::Result<String> {
        let mut reader = Reader::from_str(xml_content);
        reader.trim_text(true);
        
        let mut buf = Vec::new();
        let mut text_parts = Vec::new();
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(_)) => {}, // Skip start tags
                Ok(Event::End(_)) => {}, // Skip end tags
                Ok(Event::Text(e)) => {
                    let text = e.unescape().map_err(|e| ProcessingError::ParseError {
                        message: format!("Failed to decode XML text: {}", e),
                        position: None,
                    })?;
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        text_parts.push(trimmed.to_string());
                    }
                },
                Ok(Event::CData(e)) => {
                    let text = std::str::from_utf8(&e).map_err(|e| ProcessingError::ParseError {
                        message: format!("Invalid UTF-8 in CDATA: {}", e),
                        position: None,
                    })?;
                    text_parts.push(text.to_string());
                },
                Ok(Event::Eof) => break,
                Err(e) => return Err(ProcessingError::ParseError {
                    message: format!("XML parsing error: {}", e),
                    position: Some(reader.buffer_position() as u32),
                }),
                _ => {},
            }
            buf.clear();
        }
        
        Ok(text_parts.join(" "))
    }
    
    /// Extract metadata from XML
    fn extract_metadata(&self, xml_content: &str, content_size: u64) -> crate::Result<ContentMetadata> {
        let mut reader = Reader::from_str(xml_content);
        reader.trim_text(true);
        
        let mut buf = Vec::new();
        let mut title = None;
        let mut author = None;
        let mut custom_fields = HashMap::new();
        let mut current_element = String::new();
        let mut capturing_title = false;
        let mut capturing_author = false;
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name_bytes = e.name().as_ref();
                    if let Ok(name) = std::str::from_utf8(name_bytes) {
                        current_element = name.to_lowercase();
                    } else {
                        current_element = "unknown".to_string();
                    }
                    
                    // Look for common metadata elements
                    match current_element.as_str() {
                        "title" | "name" | "heading" => capturing_title = true,
                        "author" | "creator" | "by" => capturing_author = true,
                        _ => {},
                    }
                    
                    // Extract attributes as metadata
                    for attr in e.attributes() {
                        if let Ok(attr) = attr {
                            let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("unknown");
                            let value = attr.unescape_value().unwrap_or_default();
                            custom_fields.insert(
                                format!("attr_{}", key), 
                                serde_json::Value::String(value.to_string())
                            );
                        }
                    }
                },
                Ok(Event::Text(e)) => {
                    let text = e.unescape().map_err(|e| ProcessingError::ParseError {
                        message: format!("Failed to decode XML text: {}", e),
                        position: None,
                    })?.trim().to_string();
                    
                    if capturing_title && !text.is_empty() && title.is_none() {
                        title = Some(text);
                        capturing_title = false;
                    } else if capturing_author && !text.is_empty() && author.is_none() {
                        author = Some(text);
                        capturing_author = false;
                    }
                },
                Ok(Event::End(_)) => {
                    capturing_title = false;
                    capturing_author = false;
                    current_element.clear();
                },
                Ok(Event::Eof) => break,
                Err(e) => return Err(ProcessingError::ParseError {
                    message: format!("XML parsing error: {}", e),
                    position: Some(reader.buffer_position() as u32),
                }),
                _ => {},
            }
            buf.clear();
        }
        
        Ok(ContentMetadata {
            id: Uuid::new_v4(),
            content_type: ContentType::Xml,
            size_bytes: content_size,
            title,
            author,
            source_url: None,
            created_at: Some(Utc::now()),
            last_modified: Some(Utc::now()),
            encoding: Some("utf-8".to_string()),
            mime_type: Some("application/xml".to_string()),
            language: None,
            version: None,
            tags: Vec::new(),
            custom_fields,
        })
    }
    
    /// Extract structure from XML
    fn extract_structure(&self, xml_content: &str) -> crate::Result<ContentStructure> {
        let mut reader = Reader::from_str(xml_content);
        reader.trim_text(true);
        
        let mut buf = Vec::new();
        let mut sections = Vec::new();
        let mut element_stack = Vec::new();
        let mut current_pos = 0u32;
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name_bytes = e.name().as_ref();
                    let name = if let Ok(name_str) = std::str::from_utf8(name_bytes) {
                        name_str.to_string()
                    } else {
                        "unknown".to_string()
                    };
                    let level = element_stack.len() as u32 + 1;
                    
                    element_stack.push(ElementInfo {
                        name,
                        start_pos: current_pos,
                        level,
                    });
                },
                Ok(Event::Text(e)) => {
                    let text = e.unescape().map_err(|e| ProcessingError::ParseError {
                        message: format!("Failed to decode XML text: {}", e),
                        position: None,
                    })?;
                    current_pos += text.len() as u32;
                },
                Ok(Event::End(_)) => {
                    if let Some(element_info) = element_stack.pop() {
                        let section = ContentSection {
                            id: format!("section_{}", sections.len()),
                            title: Some(element_info.name.clone()),
                            level: element_info.level,
                            content: format!("XML element: {}", element_info.name),
                            start_position: element_info.start_pos,
                            end_position: current_pos,
                            subsections: Vec::new(),
                        };
                        sections.push(section);
                    }
                },
                Ok(Event::Eof) => break,
                Err(e) => return Err(ProcessingError::ParseError {
                    message: format!("XML parsing error: {}", e),
                    position: Some(reader.buffer_position() as u32),
                }),
                _ => {},
            }
            buf.clear();
        }
        
        Ok(ContentStructure {
            sections,
            table_of_contents: Vec::new(), // TODO: Generate TOC from structure
            links: self.extract_links(xml_content)?,
            citations: Vec::new(),
        })
    }
    
    /// Extract links from XML
    fn extract_links(&self, xml_content: &str) -> crate::Result<Vec<Link>> {
        let mut reader = Reader::from_str(xml_content);
        reader.trim_text(true);
        
        let mut buf = Vec::new();
        let mut links = Vec::new();
        let mut current_pos = 0u32;
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    // Look for attributes that might contain URLs
                    for attr in e.attributes() {
                        if let Ok(attr) = attr {
                            let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
                            if key.to_lowercase().contains("url") || 
                               key.to_lowercase().contains("href") || 
                               key.to_lowercase().contains("link") {
                                let value = attr.unescape_value().unwrap_or_default();
                                let url = value.to_string();
                                
                                let link_type = if url.starts_with("http") {
                                    LinkType::External
                                } else if url.starts_with("mailto:") {
                                    LinkType::Email
                                } else if url.starts_with("#") {
                                    LinkType::Internal
                                } else {
                                    LinkType::File
                                };
                                
                                links.push(Link {
                                    url,
                                    text: None,
                                    link_type,
                                    position: current_pos,
                                });
                            }
                        }
                    }
                },
                Ok(Event::Text(e)) => {
                    let text = e.unescape().map_err(|e| ProcessingError::ParseError {
                        message: format!("Failed to decode XML text: {}", e),
                        position: None,
                    })?;
                    current_pos += text.len() as u32;
                },
                Ok(Event::Eof) => break,
                Err(e) => return Err(ProcessingError::ParseError {
                    message: format!("XML parsing error: {}", e),
                    position: Some(reader.buffer_position() as u32),
                }),
                _ => {},
            }
            buf.clear();
        }
        
        Ok(links)
    }
}

#[derive(Debug)]
struct ElementInfo {
    name: String,
    start_pos: u32,
    level: u32,
}

impl Default for XmlParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ContentParser for XmlParser {
    async fn parse(&self, raw_content: &[u8]) -> crate::Result<ParsedContent> {
        // Convert bytes to string
        let xml_content = String::from_utf8(raw_content.to_vec())
            .map_err(|e| ProcessingError::ParseError {
                message: format!("Invalid UTF-8 encoding: {}", e),
                position: None,
            })?;
        
        // Extract components
        let text = self.extract_text(&xml_content)?;
        let content_size = raw_content.len() as u64;
        let metadata = self.extract_metadata(&xml_content, content_size)?;
        let structure = self.extract_structure(&xml_content)?;
        
        Ok(ParsedContent {
            content_type: ContentType::Xml,
            text,
            metadata,
            structure,
            media_elements: Vec::new(), // TODO: Extract media references from XML
        })
    }
    
    fn supports(&self, content_type: &ContentType) -> bool {
        matches!(content_type, ContentType::Xml)
    }
    
    fn name(&self) -> &'static str {
        self.name
    }
    
    fn estimate_parsing_time(&self, content_size_bytes: u64) -> std::time::Duration {
        // XML parsing is moderately complex - 1.5ms per KB
        std::time::Duration::from_millis((content_size_bytes / 666).max(1))
    }
    
    fn validate_content(&self, raw_content: &[u8], _content_type: &ContentType) -> crate::Result<()> {
        // Try to parse as UTF-8
        let xml_content = String::from_utf8(raw_content.to_vec())
            .map_err(|e| ProcessingError::ValidationError {
                field: "content".to_string(),
                message: format!("Content must be valid UTF-8: {}", e),
            })?;
        
        // Basic XML validation - try to parse
        let mut reader = Reader::from_str(&xml_content);
        let mut buf = Vec::new();
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Eof) => break,
                Err(e) => return Err(ProcessingError::ValidationError {
                    field: "content".to_string(),
                    message: format!("Invalid XML: {}", e),
                }),
                _ => {},
            }
            buf.clear();
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_simple_xml() {
        let parser = XmlParser::new();
        let content = b"<?xml version=\"1.0\"?><root><title>Test Document</title><content>This is test content</content></root>";
        
        let result = parser.parse(content).await.unwrap();
        
        assert_eq!(result.content_type, ContentType::Xml);
        assert!(result.text.contains("Test Document"));
        assert!(result.text.contains("This is test content"));
        assert_eq!(result.metadata.title, Some("Test Document".to_string()));
    }
    
    #[tokio::test]
    async fn test_parse_xml_with_attributes() {
        let parser = XmlParser::new();
        let content = b"<document id=\"123\" version=\"1.0\"><title>Test</title></document>";
        
        let result = parser.parse(content).await.unwrap();
        
        assert_eq!(result.content_type, ContentType::Xml);
        assert!(result.metadata.custom_fields.contains_key("attr_id"));
        assert!(result.metadata.custom_fields.contains_key("attr_version"));
    }
    
    #[tokio::test]
    async fn test_parse_xml_with_links() {
        let parser = XmlParser::new();
        let content = b"<document><link href=\"https://example.com\">External</link></document>";
        
        let result = parser.parse(content).await.unwrap();
        
        assert_eq!(result.structure.links.len(), 1);
        assert_eq!(result.structure.links[0].link_type, LinkType::External);
    }
    
    #[test]
    fn test_supports() {
        let parser = XmlParser::new();
        assert!(parser.supports(&ContentType::Xml));
        assert!(!parser.supports(&ContentType::Json));
    }
    
    #[tokio::test]
    async fn test_validate_content() {
        let parser = XmlParser::new();
        
        // Valid XML
        assert!(parser.validate_content(b"<root><child>text</child></root>", &ContentType::Xml).is_ok());
        
        // Invalid XML
        assert!(parser.validate_content(b"<root><child>text</root>", &ContentType::Xml).is_err());
        
        // Invalid UTF-8
        let invalid_utf8 = vec![0xFF, 0xFE];
        assert!(parser.validate_content(&invalid_utf8, &ContentType::Xml).is_err());
    }
}