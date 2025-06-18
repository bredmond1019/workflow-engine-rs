//! Document parsing module for various content formats
//!
//! This module provides parsers for different document formats including:
//! - HTML documents with structure preservation
//! - Markdown documents with CommonMark support
//! - PDF documents with text extraction
//! - JSON structured data
//! - XML documents with schema awareness
//! - Plain text with metadata detection

pub mod html;
pub mod markdown;
pub mod pdf;
pub mod json;
pub mod xml;
pub mod text;

use async_trait::async_trait;
use crate::models::*;
use crate::traits::ContentParser;

/// Universal document parser that dispatches to format-specific parsers
pub struct UniversalParser {
    html_parser: html::HtmlParser,
    markdown_parser: markdown::MarkdownParser,
    pdf_parser: pdf::PdfParser,
    json_parser: json::JsonParser,
    xml_parser: xml::XmlParser,
    text_parser: text::TextParser,
}

impl UniversalParser {
    pub fn new() -> Self {
        Self {
            html_parser: html::HtmlParser::new(),
            markdown_parser: markdown::MarkdownParser::new(),
            pdf_parser: pdf::PdfParser::new(),
            json_parser: json::JsonParser::new(),
            xml_parser: xml::XmlParser::new(),
            text_parser: text::TextParser::new(),
        }
    }
    
    /// Get the appropriate parser for a content type
    fn get_parser(&self, content_type: &ContentType) -> Option<&dyn ContentParser> {
        match content_type {
            ContentType::Html => Some(&self.html_parser),
            ContentType::Markdown => Some(&self.markdown_parser),
            ContentType::Pdf => Some(&self.pdf_parser),
            ContentType::Json => Some(&self.json_parser),
            ContentType::Xml => Some(&self.xml_parser),
            ContentType::PlainText => Some(&self.text_parser),
            ContentType::Code => Some(&self.text_parser), // Treat code as text for now
            ContentType::Video => None, // Not supported yet
        }
    }
    
    /// Auto-detect content type from content
    pub fn detect_content_type(&self, content: &[u8]) -> crate::Result<ContentType> {
        let text = String::from_utf8_lossy(content);
        let trimmed = text.trim();
        
        // Try to detect format from content
        if trimmed.starts_with("<!DOCTYPE html") || trimmed.starts_with("<html") {
            return Ok(ContentType::Html);
        }
        
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            return Ok(ContentType::Json);
        }
        
        if trimmed.starts_with("<?xml") || trimmed.starts_with('<') && !trimmed.starts_with("<!--") {
            return Ok(ContentType::Xml);
        }
        
        if trimmed.contains("# ") || trimmed.contains("## ") || trimmed.contains("**") || trimmed.contains("*") {
            return Ok(ContentType::Markdown);
        }
        
        // PDF detection would be done on binary content
        if content.len() >= 4 && &content[0..4] == b"%PDF" {
            return Ok(ContentType::Pdf);
        }
        
        // Default to plain text
        Ok(ContentType::PlainText)
    }
}

impl Default for UniversalParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ContentParser for UniversalParser {
    async fn parse(&self, raw_content: &[u8]) -> crate::Result<ParsedContent> {
        // First detect content type
        let detected_type = self.detect_content_type(raw_content)?;
        
        // Get appropriate parser and parse
        if let Some(parser) = self.get_parser(&detected_type) {
            parser.parse(raw_content).await
        } else {
            Err(ProcessingError::UnsupportedFormat {
                content_type: detected_type.to_string(),
            })
        }
    }
    
    fn supports(&self, content_type: &ContentType) -> bool {
        matches!(content_type, 
            ContentType::Html | ContentType::Markdown | ContentType::Pdf |
            ContentType::Json | ContentType::Xml | ContentType::PlainText |
            ContentType::Code
        )
    }
    
    fn name(&self) -> &'static str {
        "universal_parser"
    }
    
    fn estimate_parsing_time(&self, content_size_bytes: u64) -> std::time::Duration {
        // Base estimate: 1ms per KB
        std::time::Duration::from_millis((content_size_bytes / 1024).max(1))
    }
    
    fn validate_content(&self, raw_content: &[u8], content_type: &ContentType) -> crate::Result<()> {
        if raw_content.is_empty() {
            return Err(ProcessingError::ValidationError {
                field: "content".to_string(),
                message: "Content cannot be empty".to_string(),
            });
        }
        
        // Check size limits (10MB max)
        if raw_content.len() > 10 * 1024 * 1024 {
            return Err(ProcessingError::ValidationError {
                field: "content".to_string(),
                message: "Content size exceeds 10MB limit".to_string(),
            });
        }
        
        // Format-specific validation
        if let Some(parser) = self.get_parser(content_type) {
            parser.validate_content(raw_content, content_type)
        } else {
            Err(ProcessingError::UnsupportedFormat {
                content_type: content_type.to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_detection() {
        let parser = UniversalParser::new();
        
        assert_eq!(parser.detect_content_type(b"<!DOCTYPE html><html></html>").unwrap(), ContentType::Html);
        assert_eq!(parser.detect_content_type(b"<html><body>test</body></html>").unwrap(), ContentType::Html);
        assert_eq!(parser.detect_content_type(b"{\"key\": \"value\"}").unwrap(), ContentType::Json);
        assert_eq!(parser.detect_content_type(b"<?xml version=\"1.0\"?><root></root>").unwrap(), ContentType::Xml);
        assert_eq!(parser.detect_content_type(b"# Heading\n\nThis is **markdown**.").unwrap(), ContentType::Markdown);
        assert_eq!(parser.detect_content_type(b"%PDF-1.4").unwrap(), ContentType::Pdf);
        assert_eq!(parser.detect_content_type(b"Plain text content").unwrap(), ContentType::PlainText);
    }
    
    #[test]
    fn test_parser_supports() {
        let parser = UniversalParser::new();
        
        assert!(parser.supports(&ContentType::Html));
        assert!(parser.supports(&ContentType::Markdown));
        assert!(parser.supports(&ContentType::Pdf));
        assert!(parser.supports(&ContentType::Json));
        assert!(parser.supports(&ContentType::Xml));
        assert!(parser.supports(&ContentType::PlainText));
        assert!(parser.supports(&ContentType::Code));
        assert!(!parser.supports(&ContentType::Video));
    }
    
    #[tokio::test]
    async fn test_validate_content() {
        let parser = UniversalParser::new();
        
        // Test empty content
        let result = parser.validate_content(b"", &ContentType::PlainText);
        assert!(result.is_err());
        
        // Test valid content
        let result = parser.validate_content(b"Valid content", &ContentType::PlainText);
        assert!(result.is_ok());
        
        // Test content too large (simulate > 10MB)
        let large_content = vec![b'a'; 11 * 1024 * 1024];
        let result = parser.validate_content(&large_content, &ContentType::PlainText);
        assert!(result.is_err());
    }
}