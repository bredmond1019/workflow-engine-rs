//! HTML parser with structure preservation

use async_trait::async_trait;
use scraper::{Html, Selector};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

use crate::models::*;
use crate::traits::ContentParser;

/// Parser for HTML content
pub struct HtmlParser {
    name: &'static str,
}

impl HtmlParser {
    pub fn new() -> Self {
        Self {
            name: "html_parser",
        }
    }
    
    /// Extract text content from HTML, preserving structure
    fn extract_text(&self, document: &Html) -> String {
        let mut text_parts = Vec::new();
        
        // Extract text from paragraphs, headings, and other text elements
        let text_selectors = [
            "p", "h1", "h2", "h3", "h4", "h5", "h6", 
            "li", "td", "th", "div", "span", "article", "section"
        ];
        
        for selector_str in &text_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element in document.select(&selector) {
                    let text = element.text().collect::<Vec<_>>().join(" ").trim().to_string();
                    if !text.is_empty() {
                        text_parts.push(text);
                    }
                }
            }
        }
        
        text_parts.join("\n\n")
    }
    
    /// Extract metadata from HTML document
    fn extract_metadata(&self, document: &Html, content_size: u64) -> ContentMetadata {
        let mut title = None;
        let mut author = None;
        let mut custom_fields = HashMap::new();
        
        // Extract title
        if let Ok(title_selector) = Selector::parse("title") {
            if let Some(title_elem) = document.select(&title_selector).next() {
                title = Some(title_elem.text().collect::<String>());
            }
        }
        
        // Extract meta tags
        if let Ok(meta_selector) = Selector::parse("meta") {
            for meta in document.select(&meta_selector) {
                if let Some(name) = meta.value().attr("name") {
                    if let Some(content) = meta.value().attr("content") {
                        match name {
                            "author" => author = Some(content.to_string()),
                            "description" => {
                                custom_fields.insert("description".to_string(), 
                                    serde_json::Value::String(content.to_string()));
                            },
                            "keywords" => {
                                custom_fields.insert("keywords".to_string(), 
                                    serde_json::Value::String(content.to_string()));
                            },
                            _ => {
                                custom_fields.insert(name.to_string(), 
                                    serde_json::Value::String(content.to_string()));
                            }
                        }
                    }
                }
            }
        }
        
        ContentMetadata {
            id: Uuid::new_v4(),
            content_type: ContentType::Html,
            size_bytes: content_size,
            title,
            author,
            source_url: None,
            created_at: Some(Utc::now()),
            last_modified: Some(Utc::now()),
            encoding: Some("utf-8".to_string()),
            mime_type: Some("text/html".to_string()),
            language: None,
            version: None,
            tags: Vec::new(),
            custom_fields,
        }
    }
    
    /// Extract hierarchical structure from HTML
    fn extract_structure(&self, document: &Html, text: &str) -> ContentStructure {
        let mut sections = Vec::new();
        let mut toc_entries = Vec::new();
        let _current_pos = 0u32;
        
        // Extract headings for structure
        let heading_selectors = ["h1", "h2", "h3", "h4", "h5", "h6"];
        
        for (level, selector_str) in heading_selectors.iter().enumerate() {
            if let Ok(selector) = Selector::parse(selector_str) {
                for heading in document.select(&selector) {
                    let heading_text = heading.text().collect::<String>();
                    let section_id = format!("section_{}", sections.len());
                    
                    // Find position in extracted text
                    if let Some(pos) = text.find(&heading_text) {
                        let section = ContentSection {
                            id: section_id.clone(),
                            title: Some(heading_text.clone()),
                            level: (level + 1) as u32,
                            content: heading_text.clone(),
                            start_position: pos as u32,
                            end_position: (pos + heading_text.len()) as u32,
                            subsections: Vec::new(),
                        };
                        sections.push(section);
                        
                        // Add to table of contents
                        toc_entries.push(TocEntry {
                            title: heading_text,
                            level: (level + 1) as u32,
                            position: pos as u32,
                            section_id,
                        });
                    }
                }
            }
        }
        
        ContentStructure {
            sections,
            table_of_contents: toc_entries,
            links: self.extract_links(document),
            citations: Vec::new(),
        }
    }
    
    /// Extract links from HTML
    fn extract_links(&self, document: &Html) -> Vec<Link> {
        let mut links = Vec::new();
        
        if let Ok(link_selector) = Selector::parse("a[href]") {
            for link_elem in document.select(&link_selector) {
                if let Some(href) = link_elem.value().attr("href") {
                    let text = link_elem.text().collect::<String>();
                    let link_type = if href.starts_with("http") {
                        LinkType::External
                    } else if href.starts_with("#") {
                        LinkType::Internal
                    } else if href.starts_with("mailto:") {
                        LinkType::Email
                    } else if href.starts_with("tel:") {
                        LinkType::Phone
                    } else {
                        LinkType::File
                    };
                    
                    links.push(Link {
                        url: href.to_string(),
                        text: if text.is_empty() { None } else { Some(text) },
                        link_type,
                        position: 0, // Would need more complex logic to find exact position
                    });
                }
            }
        }
        
        links
    }
    
    /// Extract media elements
    fn extract_media_elements(&self, document: &Html) -> Vec<MediaElement> {
        let mut media = Vec::new();
        
        // Extract images
        if let Ok(img_selector) = Selector::parse("img") {
            for img in document.select(&img_selector) {
                let element = MediaElement {
                    element_type: MediaType::Image,
                    url: img.value().attr("src").map(|s| s.to_string()),
                    alt_text: img.value().attr("alt").map(|s| s.to_string()),
                    caption: img.value().attr("title").map(|s| s.to_string()),
                    metadata: HashMap::new(),
                };
                media.push(element);
            }
        }
        
        // Extract videos
        if let Ok(video_selector) = Selector::parse("video") {
            for video in document.select(&video_selector) {
                let element = MediaElement {
                    element_type: MediaType::Video,
                    url: video.value().attr("src").map(|s| s.to_string()),
                    alt_text: None,
                    caption: video.value().attr("title").map(|s| s.to_string()),
                    metadata: HashMap::new(),
                };
                media.push(element);
            }
        }
        
        // Extract tables
        if let Ok(table_selector) = Selector::parse("table") {
            for _table in document.select(&table_selector) {
                let element = MediaElement {
                    element_type: MediaType::Table,
                    url: None,
                    alt_text: None,
                    caption: None,
                    metadata: HashMap::new(),
                };
                media.push(element);
            }
        }
        
        media
    }
}

impl Default for HtmlParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ContentParser for HtmlParser {
    async fn parse(&self, raw_content: &[u8]) -> crate::Result<ParsedContent> {
        // Convert bytes to string
        let html_content = String::from_utf8(raw_content.to_vec())
            .map_err(|e| ProcessingError::ParseError {
                message: format!("Invalid UTF-8 encoding: {}", e),
                position: None,
            })?;
        
        // Parse HTML
        let document = Html::parse_document(&html_content);
        
        // Extract components
        let text = self.extract_text(&document);
        let content_size = raw_content.len() as u64;
        let metadata = self.extract_metadata(&document, content_size);
        let structure = self.extract_structure(&document, &text);
        let media_elements = self.extract_media_elements(&document);
        
        Ok(ParsedContent {
            content_type: ContentType::Html,
            text,
            metadata,
            structure,
            media_elements,
        })
    }
    
    fn supports(&self, content_type: &ContentType) -> bool {
        matches!(content_type, ContentType::Html)
    }
    
    fn name(&self) -> &'static str {
        self.name
    }
    
    fn estimate_parsing_time(&self, content_size_bytes: u64) -> std::time::Duration {
        // HTML parsing is more complex - 2ms per KB
        std::time::Duration::from_millis((content_size_bytes / 512).max(1))
    }
    
    fn validate_content(&self, raw_content: &[u8], _content_type: &ContentType) -> crate::Result<()> {
        // Try to parse as UTF-8
        let html_content = String::from_utf8(raw_content.to_vec())
            .map_err(|e| ProcessingError::ValidationError {
                field: "content".to_string(),
                message: format!("Content must be valid UTF-8: {}", e),
            })?;
        
        // Basic HTML validation - check for some HTML structure
        if !html_content.contains('<') || !html_content.contains('>') {
            return Err(ProcessingError::ValidationError {
                field: "content".to_string(),
                message: "Content does not appear to be valid HTML".to_string(),
            });
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_simple_html() {
        let parser = HtmlParser::new();
        let content = b"<html><head><title>Test Page</title></head><body><h1>Hello World</h1><p>This is a test.</p></body></html>";
        
        let result = parser.parse(content).await.unwrap();
        
        assert_eq!(result.content_type, ContentType::Html);
        assert!(result.text.contains("Hello World"));
        assert!(result.text.contains("This is a test"));
        assert_eq!(result.metadata.title, Some("Test Page".to_string()));
        assert_eq!(result.metadata.content_type, ContentType::Html);
    }
    
    #[tokio::test]
    async fn test_parse_html_with_links() {
        let parser = HtmlParser::new();
        let content = b"<html><body><a href=\"https://example.com\">External Link</a><a href=\"#section1\">Internal Link</a></body></html>";
        
        let result = parser.parse(content).await.unwrap();
        
        assert_eq!(result.structure.links.len(), 2);
        assert_eq!(result.structure.links[0].link_type, LinkType::External);
        assert_eq!(result.structure.links[1].link_type, LinkType::Internal);
    }
    
    #[tokio::test]
    async fn test_parse_html_with_media() {
        let parser = HtmlParser::new();
        let content = b"<html><body><img src=\"test.jpg\" alt=\"Test Image\"><video src=\"test.mp4\"></video></body></html>";
        
        let result = parser.parse(content).await.unwrap();
        
        assert_eq!(result.media_elements.len(), 2);
        assert!(matches!(result.media_elements[0].element_type, MediaType::Image));
        assert!(matches!(result.media_elements[1].element_type, MediaType::Video));
    }
    
    #[test]
    fn test_supports() {
        let parser = HtmlParser::new();
        assert!(parser.supports(&ContentType::Html));
        assert!(!parser.supports(&ContentType::PlainText));
    }
    
    #[tokio::test]
    async fn test_validate_content() {
        let parser = HtmlParser::new();
        
        // Valid HTML
        assert!(parser.validate_content(b"<html><body>test</body></html>", &ContentType::Html).is_ok());
        
        // Invalid - no HTML tags
        assert!(parser.validate_content(b"Just plain text", &ContentType::Html).is_err());
        
        // Invalid UTF-8
        let invalid_utf8 = vec![0xFF, 0xFE];
        assert!(parser.validate_content(&invalid_utf8, &ContentType::Html).is_err());
    }
}