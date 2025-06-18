//! PDF parser with text extraction

use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

use crate::models::*;
use crate::traits::ContentParser;

/// Parser for PDF content  
pub struct PdfParser {
    name: &'static str,
}

impl PdfParser {
    pub fn new() -> Self {
        Self {
            name: "pdf_parser",
        }
    }
    
    /// Extract text from PDF bytes
    fn extract_text_from_pdf(&self, pdf_bytes: &[u8]) -> crate::Result<String> {
        // Use pdf-extract crate for text extraction
        match pdf_extract::extract_text_from_mem(pdf_bytes) {
            Ok(text) => Ok(text),
            Err(e) => Err(ProcessingError::ParseError {
                message: format!("Failed to extract text from PDF: {}", e),
                position: None,
            }),
        }
    }
    
    /// Extract metadata from PDF
    fn extract_metadata(&self, _pdf_bytes: &[u8], text: &str, content_size: u64) -> ContentMetadata {
        let mut title = None;
        let author = None;
        let custom_fields = HashMap::new();
        
        // Try to extract title from first line of text
        if let Some(first_line) = text.lines().next() {
            let trimmed = first_line.trim();
            if !trimmed.is_empty() && trimmed.len() < 200 {
                title = Some(trimmed.to_string());
            }
        }
        
        // TODO: Extract PDF metadata using a more sophisticated PDF library
        // For now, we'll use basic heuristics
        
        ContentMetadata {
            id: Uuid::new_v4(),
            content_type: ContentType::Pdf,
            size_bytes: content_size,
            title,
            author,
            source_url: None,
            created_at: Some(Utc::now()),
            last_modified: Some(Utc::now()),
            encoding: Some("utf-8".to_string()),
            mime_type: Some("application/pdf".to_string()),
            language: None,
            version: None,
            tags: Vec::new(),
            custom_fields,
        }
    }
    
    /// Extract structure from PDF text
    fn extract_structure(&self, text: &str) -> ContentStructure {
        let mut sections = Vec::new();
        let mut toc_entries = Vec::new();
        let lines: Vec<&str> = text.lines().collect();
        let mut current_pos = 0u32;
        
        // Simple heuristic: look for lines that might be headings
        // (short lines, possibly in all caps, or starting with numbers)
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                current_pos += line.len() as u32 + 1;
                continue;
            }
            
            // Heuristic for detecting headings
            let is_heading = self.is_likely_heading(trimmed, i, &lines);
            
            if is_heading {
                let section_id = format!("section_{}", sections.len());
                let level = self.estimate_heading_level(trimmed);
                
                // Collect content until next heading or end
                let mut section_content = vec![trimmed];
                let mut j = i + 1;
                while j < lines.len() {
                    let next_line = lines[j].trim();
                    if !next_line.is_empty() && self.is_likely_heading(next_line, j, &lines) {
                        break;
                    }
                    if !next_line.is_empty() {
                        section_content.push(next_line);
                    }
                    j += 1;
                }
                
                let content = section_content.join(" ");
                let section = ContentSection {
                    id: section_id.clone(),
                    title: Some(trimmed.to_string()),
                    level,
                    content: content.clone(),
                    start_position: current_pos,
                    end_position: current_pos + content.len() as u32,
                    subsections: Vec::new(),
                };
                sections.push(section);
                
                toc_entries.push(TocEntry {
                    title: trimmed.to_string(),
                    level,
                    position: current_pos,
                    section_id,
                });
            }
            
            current_pos += line.len() as u32 + 1;
        }
        
        // If no headings found, create a single section
        if sections.is_empty() {
            sections.push(ContentSection {
                id: "main".to_string(),
                title: None,
                level: 1,
                content: text.to_string(),
                start_position: 0,
                end_position: text.len() as u32,
                subsections: Vec::new(),
            });
        }
        
        ContentStructure {
            sections,
            table_of_contents: toc_entries,
            links: self.extract_links(text),
            citations: Vec::new(), // TODO: Extract citations from PDF
        }
    }
    
    /// Heuristic to determine if a line is likely a heading
    fn is_likely_heading(&self, line: &str, index: usize, all_lines: &[&str]) -> bool {
        // Skip very long lines
        if line.len() > 100 {
            return false;
        }
        
        // Check if it's a numbered heading (e.g., "1. Introduction", "Chapter 1")
        if line.starts_with(char::is_numeric) && (line.contains('.') || line.contains(' ')) {
            return true;
        }
        
        // Check if it's all caps and short
        if line.len() < 50 && line.chars().all(|c| c.is_uppercase() || c.is_whitespace() || c.is_numeric()) {
            return true;
        }
        
        // Check if next line is empty (common after headings)
        if index + 1 < all_lines.len() && all_lines[index + 1].trim().is_empty() {
            return true;
        }
        
        false
    }
    
    /// Estimate heading level based on content
    fn estimate_heading_level(&self, heading: &str) -> u32 {
        // Simple heuristics for heading levels
        if heading.starts_with("Chapter") || heading.starts_with("CHAPTER") {
            return 1;
        }
        
        if heading.chars().next().map_or(false, char::is_numeric) {
            // Count dots to estimate level (1.1.1 = level 3)
            let dot_count = heading.chars().filter(|&c| c == '.').count();
            return (dot_count + 1).min(6) as u32;
        }
        
        // Default to level 2
        2
    }
    
    /// Extract URLs from text
    fn extract_links(&self, text: &str) -> Vec<Link> {
        let mut links = Vec::new();
        
        // Simple URL regex pattern
        let url_patterns = [
            r"https?://[^\s]+",
            r"www\.[^\s]+",
        ];
        
        for pattern in &url_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                for mat in regex.find_iter(text) {
                    let url = mat.as_str().to_string();
                    links.push(Link {
                        url: url.clone(),
                        text: Some(url),
                        link_type: LinkType::External,
                        position: mat.start() as u32,
                    });
                }
            }
        }
        
        links
    }
}

impl Default for PdfParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ContentParser for PdfParser {
    async fn parse(&self, raw_content: &[u8]) -> crate::Result<ParsedContent> {
        // Validate that this is actually a PDF
        if raw_content.len() < 4 || &raw_content[0..4] != b"%PDF" {
            return Err(ProcessingError::ParseError {
                message: "Content does not appear to be a valid PDF".to_string(),
                position: None,
            });
        }
        
        // Extract text from PDF
        let text = self.extract_text_from_pdf(raw_content)?;
        
        if text.trim().is_empty() {
            return Err(ProcessingError::ParseError {
                message: "No text could be extracted from PDF".to_string(),
                position: None,
            });
        }
        
        // Extract components
        let content_size = raw_content.len() as u64;
        let metadata = self.extract_metadata(raw_content, &text, content_size);
        let structure = self.extract_structure(&text);
        
        Ok(ParsedContent {
            content_type: ContentType::Pdf,
            text,
            metadata,
            structure,
            media_elements: Vec::new(), // TODO: Extract images and other media from PDF
        })
    }
    
    fn supports(&self, content_type: &ContentType) -> bool {
        matches!(content_type, ContentType::Pdf)
    }
    
    fn name(&self) -> &'static str {
        self.name
    }
    
    fn estimate_parsing_time(&self, content_size_bytes: u64) -> std::time::Duration {
        // PDF parsing is complex and slow - 10ms per KB
        std::time::Duration::from_millis((content_size_bytes / 100).max(10))
    }
    
    fn validate_content(&self, raw_content: &[u8], _content_type: &ContentType) -> crate::Result<()> {
        // Check PDF magic number
        if raw_content.len() < 4 {
            return Err(ProcessingError::ValidationError {
                field: "content".to_string(),
                message: "Content too short to be a valid PDF".to_string(),
            });
        }
        
        if &raw_content[0..4] != b"%PDF" {
            return Err(ProcessingError::ValidationError {
                field: "content".to_string(),
                message: "Content does not have PDF magic number".to_string(),
            });
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supports() {
        let parser = PdfParser::new();
        assert!(parser.supports(&ContentType::Pdf));
        assert!(!parser.supports(&ContentType::Html));
    }
    
    #[tokio::test]
    async fn test_validate_content() {
        let parser = PdfParser::new();
        
        // Valid PDF magic number
        let pdf_header = b"%PDF-1.4";
        assert!(parser.validate_content(pdf_header, &ContentType::Pdf).is_ok());
        
        // Invalid - not a PDF
        assert!(parser.validate_content(b"Not a PDF", &ContentType::Pdf).is_err());
        
        // Too short
        assert!(parser.validate_content(b"PDF", &ContentType::Pdf).is_err());
    }
    
    #[test]
    fn test_is_likely_heading() {
        let parser = PdfParser::new();
        let lines = vec!["1. Introduction", "", "Some content here"];
        
        assert!(parser.is_likely_heading("1. Introduction", 0, &lines));
        assert!(parser.is_likely_heading("CHAPTER ONE", 0, &lines));
        assert!(!parser.is_likely_heading("This is just regular text that goes on and on", 0, &lines));
    }
    
    #[test]
    fn test_estimate_heading_level() {
        let parser = PdfParser::new();
        
        assert_eq!(parser.estimate_heading_level("Chapter 1"), 1);
        assert_eq!(parser.estimate_heading_level("1. Introduction"), 1);
        assert_eq!(parser.estimate_heading_level("1.1 Overview"), 2);
        assert_eq!(parser.estimate_heading_level("1.1.1 Details"), 3);
    }
}