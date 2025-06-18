//! Markdown parser with CommonMark support

use async_trait::async_trait;
use pulldown_cmark::{Parser, Event, Tag, HeadingLevel};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

use crate::models::*;
use crate::traits::ContentParser;

/// Parser for Markdown content
pub struct MarkdownParser {
    name: &'static str,
}

impl MarkdownParser {
    pub fn new() -> Self {
        Self {
            name: "markdown_parser",
        }
    }
    
    /// Convert markdown to plain text while preserving structure
    fn extract_text(&self, markdown: &str) -> String {
        let parser = Parser::new(markdown);
        let mut text_parts = Vec::new();
        let mut in_code_block = false;
        
        for event in parser {
            match event {
                Event::Text(text) => {
                    if !in_code_block {
                        text_parts.push(text.to_string());
                    }
                },
                Event::Code(code) => {
                    text_parts.push(code.to_string());
                },
                Event::Html(_) => {}, // Skip HTML in markdown
                Event::Start(Tag::CodeBlock(_)) => {
                    in_code_block = true;
                },
                Event::End(Tag::CodeBlock(_)) => {
                    in_code_block = false;
                },
                Event::SoftBreak | Event::HardBreak => {
                    text_parts.push(" ".to_string());
                },
                _ => {},
            }
        }
        
        text_parts.join("").trim().to_string()
    }
    
    /// Extract metadata from markdown frontmatter and content
    fn extract_metadata(&self, markdown: &str, content_size: u64) -> ContentMetadata {
        let mut title = None;
        let mut custom_fields = HashMap::new();
        
        // Try to extract title from first heading
        let parser = Parser::new(markdown);
        for event in parser {
            if let Event::Start(Tag::Heading(HeadingLevel::H1, _, _)) = event {
                // The next text event should be the title
                continue;
            }
            if let Event::Text(text) = event {
                if title.is_none() {
                    // Check if this follows an H1
                    title = Some(text.to_string());
                    break;
                }
            }
        }
        
        // Parse YAML frontmatter if present
        if markdown.starts_with("---") {
            if let Some(end_pos) = markdown[3..].find("---") {
                let frontmatter = &markdown[3..end_pos + 3];
                // Simple key-value extraction (would use a YAML parser in production)
                for line in frontmatter.lines() {
                    if let Some(colon_pos) = line.find(':') {
                        let key = line[..colon_pos].trim();
                        let value = line[colon_pos + 1..].trim();
                        
                        match key {
                            "title" => title = Some(value.trim_matches('"').to_string()),
                            "author" => {
                                custom_fields.insert("author".to_string(), 
                                    serde_json::Value::String(value.trim_matches('"').to_string()));
                            },
                            _ => {
                                custom_fields.insert(key.to_string(), 
                                    serde_json::Value::String(value.trim_matches('"').to_string()));
                            }
                        }
                    }
                }
            }
        }
        
        ContentMetadata {
            id: Uuid::new_v4(),
            content_type: ContentType::Markdown,
            size_bytes: content_size,
            title,
            author: custom_fields.get("author").and_then(|v| v.as_str()).map(|s| s.to_string()),
            source_url: None,
            created_at: Some(Utc::now()),
            last_modified: Some(Utc::now()),
            encoding: Some("utf-8".to_string()),
            mime_type: Some("text/markdown".to_string()),
            language: None,
            version: None,
            tags: Vec::new(),
            custom_fields,
        }
    }
    
    /// Extract hierarchical structure from markdown
    fn extract_structure(&self, markdown: &str) -> ContentStructure {
        let parser = Parser::new(markdown);
        let mut sections = Vec::new();
        let mut toc_entries = Vec::new();
        let mut current_heading: Option<(String, u32)> = None;
        let mut current_text = String::new();
        let mut position = 0u32;
        
        for event in parser {
            match event {
                Event::Start(Tag::Heading(_level, _, _)) => {
                    // Save previous section if exists
                    if let Some((heading, level)) = current_heading.take() {
                        let section_id = format!("section_{}", sections.len());
                        sections.push(ContentSection {
                            id: section_id.clone(),
                            title: Some(heading.clone()),
                            level,
                            content: current_text.clone(),
                            start_position: position.saturating_sub(current_text.len() as u32),
                            end_position: position,
                            subsections: Vec::new(),
                        });
                        
                        let content_len = current_text.len() as u32;
                        toc_entries.push(TocEntry {
                            title: heading,
                            level,
                            position: position.saturating_sub(content_len),
                            section_id,
                        });
                        
                        current_text.clear();
                    }
                },
                Event::Text(text) => {
                    if current_heading.is_some() {
                        current_text.push_str(&text);
                    } else {
                        // This might be a heading text
                        current_heading = Some((text.to_string(), 1)); // Default to H1
                    }
                    position += text.len() as u32;
                },
                Event::End(Tag::Heading(level, _, _)) => {
                    if let Some((heading, _)) = current_heading.take() {
                        current_heading = Some((heading, heading_level_to_u32(level)));
                    }
                },
                _ => {},
            }
        }
        
        // Don't forget the last section
        if let Some((heading, level)) = current_heading {
            let section_id = format!("section_{}", sections.len());
            let content_len = current_text.len() as u32;
            sections.push(ContentSection {
                id: section_id.clone(),
                title: Some(heading.clone()),
                level,
                content: current_text,
                start_position: position.saturating_sub(content_len),
                end_position: position,
                subsections: Vec::new(),
            });
            
            toc_entries.push(TocEntry {
                title: heading,
                level,
                position: position.saturating_sub(content_len),
                section_id,
            });
        }
        
        ContentStructure {
            sections,
            table_of_contents: toc_entries,
            links: self.extract_links(markdown),
            citations: Vec::new(),
        }
    }
    
    /// Extract links from markdown
    fn extract_links(&self, markdown: &str) -> Vec<Link> {
        let parser = Parser::new(markdown);
        let mut links = Vec::new();
        let mut link_url: Option<String> = None;
        let mut link_text: Option<String> = None;
        let mut position = 0u32;
        
        for event in parser {
            match event {
                Event::Start(Tag::Link(_, url, _)) => {
                    link_url = Some(url.to_string());
                },
                Event::Text(text) => {
                    if link_url.is_some() {
                        link_text = Some(text.to_string());
                    }
                    position += text.len() as u32;
                },
                Event::End(Tag::Link(_, _, _)) => {
                    if let Some(url) = link_url.take() {
                        let link_type = if url.starts_with("http") {
                            LinkType::External
                        } else if url.starts_with("#") {
                            LinkType::Internal
                        } else if url.starts_with("mailto:") {
                            LinkType::Email
                        } else {
                            LinkType::File
                        };
                        
                        links.push(Link {
                            url,
                            text: link_text.take(),
                            link_type,
                            position,
                        });
                    }
                },
                _ => {},
            }
        }
        
        links
    }
}

/// Convert HeadingLevel to u32
fn heading_level_to_u32(level: HeadingLevel) -> u32 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ContentParser for MarkdownParser {
    async fn parse(&self, raw_content: &[u8]) -> crate::Result<ParsedContent> {
        // Convert bytes to string
        let markdown_content = String::from_utf8(raw_content.to_vec())
            .map_err(|e| ProcessingError::ParseError {
                message: format!("Invalid UTF-8 encoding: {}", e),
                position: None,
            })?;
        
        // Extract components
        let text = self.extract_text(&markdown_content);
        let content_size = raw_content.len() as u64;
        let metadata = self.extract_metadata(&markdown_content, content_size);
        let structure = self.extract_structure(&markdown_content);
        
        Ok(ParsedContent {
            content_type: ContentType::Markdown,
            text,
            metadata,
            structure,
            media_elements: Vec::new(), // TODO: Extract images and other media from markdown
        })
    }
    
    fn supports(&self, content_type: &ContentType) -> bool {
        matches!(content_type, ContentType::Markdown)
    }
    
    fn name(&self) -> &'static str {
        self.name
    }
    
    fn estimate_parsing_time(&self, content_size_bytes: u64) -> std::time::Duration {
        // Markdown parsing is moderate complexity - 1ms per KB
        std::time::Duration::from_millis((content_size_bytes / 1024).max(1))
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
    async fn test_parse_simple_markdown() {
        let parser = MarkdownParser::new();
        let content = b"# Hello World\n\nThis is a **test** document.\n\n## Section 2\n\nMore content here.";
        
        let result = parser.parse(content).await.unwrap();
        
        assert_eq!(result.content_type, ContentType::Markdown);
        assert!(result.text.contains("Hello World"));
        assert!(result.text.contains("test document"));
        assert_eq!(result.metadata.content_type, ContentType::Markdown);
        assert!(result.structure.sections.len() >= 1);
        assert!(result.structure.table_of_contents.len() >= 1);
    }
    
    #[tokio::test]
    async fn test_parse_markdown_with_frontmatter() {
        let parser = MarkdownParser::new();
        let content = b"---\ntitle: Test Document\nauthor: John Doe\n---\n\n# Content\n\nBody text.";
        
        let result = parser.parse(content).await.unwrap();
        
        assert_eq!(result.metadata.title, Some("Test Document".to_string()));
        assert_eq!(result.metadata.author, Some("John Doe".to_string()));
    }
    
    #[tokio::test]
    async fn test_parse_markdown_with_links() {
        let parser = MarkdownParser::new();
        let content = b"# Test\n\nCheck out [this link](https://example.com) and [internal](#section).";
        
        let result = parser.parse(content).await.unwrap();
        
        assert_eq!(result.structure.links.len(), 2);
        assert_eq!(result.structure.links[0].link_type, LinkType::External);
        assert_eq!(result.structure.links[1].link_type, LinkType::Internal);
    }
    
    #[test]
    fn test_supports() {
        let parser = MarkdownParser::new();
        assert!(parser.supports(&ContentType::Markdown));
        assert!(!parser.supports(&ContentType::Html));
    }
    
    #[test]
    fn test_heading_level_conversion() {
        assert_eq!(heading_level_to_u32(HeadingLevel::H1), 1);
        assert_eq!(heading_level_to_u32(HeadingLevel::H6), 6);
    }
}