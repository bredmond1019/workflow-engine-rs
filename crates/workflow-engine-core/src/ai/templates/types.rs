//! # Template Types and Data Structures
//!
//! This module defines the core types used throughout the template system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use chrono::{DateTime, Utc};

/// Unique identifier for templates
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TemplateId(pub String);

impl From<String> for TemplateId {
    fn from(s: String) -> Self {
        TemplateId(s)
    }
}

impl From<&str> for TemplateId {
    fn from(s: &str) -> Self {
        TemplateId(s.to_string())
    }
}

impl std::fmt::Display for TemplateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Template version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVersion {
    pub version: u32,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub comment: Option<String>,
    pub content_hash: String,
}

/// Template metadata for storage and management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    pub id: TemplateId,
    pub name: String,
    pub description: Option<String>,
    pub version: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub context: Option<String>,
    pub author: Option<String>,
}

/// Variable type definitions for type-safe substitution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VariableType {
    String,
    Number,
    Boolean,
    Array(Box<VariableType>),
    Object(HashMap<String, VariableType>),
    Any,
}

impl VariableType {
    /// Check if a JSON value matches this variable type
    pub fn matches_value(&self, value: &serde_json::Value) -> bool {
        match (self, value) {
            (VariableType::String, serde_json::Value::String(_)) => true,
            (VariableType::Number, serde_json::Value::Number(_)) => true,
            (VariableType::Boolean, serde_json::Value::Bool(_)) => true,
            (VariableType::Array(inner), serde_json::Value::Array(arr)) => {
                arr.iter().all(|v| inner.matches_value(v))
            }
            (VariableType::Object(schema), serde_json::Value::Object(map)) => {
                schema.iter().all(|(key, vtype)| {
                    map.get(key).map_or(false, |v| vtype.matches_value(v))
                })
            }
            (VariableType::Any, _) => true,
            _ => false,
        }
    }
}

/// Variable definition with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub var_type: VariableType,
    pub required: bool,
    pub default: Option<serde_json::Value>,
    pub description: Option<String>,
    pub validation: Option<String>, // Regex pattern for strings
}

/// Output format for rendered templates
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OutputFormat {
    Text,
    Json,
    Yaml,
    Markdown,
    Html,
}

/// Main template structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: TemplateId,
    pub name: String,
    pub description: Option<String>,
    pub content: String,
    pub variables: HashMap<String, VariableType>,
    pub output_format: OutputFormat,
    pub parent: Option<TemplateId>, // For template inheritance
    pub includes: Vec<TemplateId>,   // For template composition
    pub metadata: TemplateMetadata,
    pub context: Option<String>,     // Context type for selection
    pub tags: Vec<String>,
}

impl Template {
    /// Create a new template with minimal configuration
    pub fn new(id: impl Into<String>, content: impl Into<String>) -> Result<Self, TemplateError> {
        let id = TemplateId::from(id.into());
        let name = id.0.clone();
        let now = Utc::now();
        
        Ok(Self {
            id: id.clone(),
            name: name.clone(),
            description: None,
            content: content.into(),
            variables: HashMap::new(),
            output_format: OutputFormat::Text,
            parent: None,
            includes: Vec::new(),
            metadata: TemplateMetadata {
                id,
                name,
                description: None,
                version: 1,
                created_at: now,
                updated_at: now,
                tags: Vec::new(),
                context: None,
                author: None,
            },
            context: None,
            tags: Vec::new(),
        })
    }
    
    /// Parse a template from string
    pub fn parse(id: impl Into<String>, content: impl Into<String>) -> Result<Self, TemplateError> {
        let mut template = Self::new(id, content)?;
        
        // Extract variables from content
        template.variables = extract_variables(&template.content)?;
        
        Ok(template)
    }
    
    /// Add a variable definition
    pub fn with_variable(mut self, name: impl Into<String>, var_type: VariableType) -> Self {
        self.variables.insert(name.into(), var_type);
        self
    }
    
    /// Set the output format
    pub fn with_output_format(mut self, format: OutputFormat) -> Self {
        self.output_format = format;
        self
    }
    
    /// Set the parent template for inheritance
    pub fn with_parent(mut self, parent_id: impl Into<TemplateId>) -> Self {
        self.parent = Some(parent_id.into());
        self
    }
    
    /// Add an include template
    pub fn with_include(mut self, include_id: impl Into<TemplateId>) -> Self {
        self.includes.push(include_id.into());
        self
    }
    
    /// Set the context type
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self.metadata.context = self.context.clone();
        self
    }
    
    /// Update content
    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self.metadata.version += 1;
        self.metadata.updated_at = Utc::now();
        self
    }
    
    /// Add tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags.clone();
        self.metadata.tags = tags;
        self
    }
}

/// Compiled template for efficient rendering
#[derive(Debug, Clone)]
pub struct CompiledTemplate {
    pub id: TemplateId,
    pub ast: Arc<super::parser::TemplateAst>,
    pub variables: HashMap<String, VariableType>,
    pub output_format: OutputFormat,
}

/// Template variables container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariables {
    values: HashMap<String, serde_json::Value>,
}

impl TemplateVariables {
    /// Create new empty variables
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }
    
    /// Create from a HashMap
    pub fn from_map(map: HashMap<String, serde_json::Value>) -> Self {
        Self { values: map }
    }
    
    /// Add a variable
    pub fn insert(&mut self, name: impl Into<String>, value: serde_json::Value) {
        self.values.insert(name.into(), value);
    }
    
    /// Get a variable
    pub fn get(&self, name: &str) -> Option<&serde_json::Value> {
        self.values.get(name)
    }
    
    /// Check if a variable exists
    pub fn contains(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }
    
    /// Get inner map
    pub fn inner(&self) -> &HashMap<String, serde_json::Value> {
        &self.values
    }
}

impl Default for TemplateVariables {
    fn default() -> Self {
        Self::new()
    }
}

/// Template performance metrics
#[derive(Debug, Clone)]
pub struct TemplateMetrics {
    render_times: std::sync::Arc<std::sync::Mutex<HashMap<String, Vec<Duration>>>>,
    cache_hits: std::sync::Arc<std::sync::atomic::AtomicU64>,
    cache_misses: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl TemplateMetrics {
    /// Create new metrics collector
    pub fn new() -> Self {
        Self {
            render_times: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            cache_hits: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
            cache_misses: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }
    
    /// Record a render operation
    pub fn record_render(&self, template_id: &str, duration: Duration) {
        if let Ok(mut times) = self.render_times.lock() {
            times.entry(template_id.to_string())
                .or_insert_with(Vec::new)
                .push(duration);
        }
    }
    
    /// Record a cache hit
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    
    /// Record a cache miss
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    
    /// Get average render time for a template
    pub fn average_render_time(&self, template_id: &str) -> Option<Duration> {
        if let Ok(times) = self.render_times.lock() {
            if let Some(durations) = times.get(template_id) {
                if !durations.is_empty() {
                    let total: Duration = durations.iter().sum();
                    return Some(total / durations.len() as u32);
                }
            }
        }
        None
    }
    
    /// Get cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hits.load(std::sync::atomic::Ordering::Relaxed);
        let misses = self.cache_misses.load(std::sync::atomic::Ordering::Relaxed);
        let total = hits + misses;
        if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        }
    }
}

/// Template-specific errors
#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("Template not found: {id}")]
    NotFound { id: String },
    
    #[error("Template parse error: {message}")]
    ParseError { message: String },
    
    #[error("Variable not found: {name}")]
    VariableNotFound { name: String },
    
    #[error("Type mismatch for variable {name}: expected {expected}, got {actual}")]
    TypeMismatch {
        name: String,
        expected: String,
        actual: String,
    },
    
    #[error("Validation error: {message}")]
    ValidationError { message: String },
    
    #[error("Render error: {message}")]
    RenderError { message: String },
    
    #[error("Storage error: {message}")]
    StorageError { message: String },
    
    #[error("Circular dependency detected: {template_id}")]
    CircularDependency { template_id: String },
    
    #[error("Invalid template syntax: {message}")]
    SyntaxError { message: String },
    
    #[error("Security violation: {message}")]
    SecurityError { message: String },
}

impl From<TemplateError> for crate::error::WorkflowError {
    fn from(error: TemplateError) -> Self {
        crate::error::WorkflowError::ProcessingError {
            message: format!("Template error: {}", error),
        }
    }
}

/// Extract variables from template content
fn extract_variables(content: &str) -> Result<HashMap<String, VariableType>, TemplateError> {
    let mut variables = HashMap::new();
    
    // Simple regex-based extraction for now
    // In production, this would be done by the parser
    let var_pattern = regex::Regex::new(r"\{\{(\w+)\}\}").unwrap();
    
    for capture in var_pattern.captures_iter(content) {
        if let Some(var_name) = capture.get(1) {
            // Default to string type for now
            variables.insert(var_name.as_str().to_string(), VariableType::String);
        }
    }
    
    Ok(variables)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_template_creation() {
        let template = Template::new("test", "Hello {{name}}!").unwrap();
        assert_eq!(template.id.0, "test");
        assert_eq!(template.content, "Hello {{name}}!");
        assert_eq!(template.output_format, OutputFormat::Text);
    }
    
    #[test]
    fn test_variable_type_matching() {
        assert!(VariableType::String.matches_value(&json!("hello")));
        assert!(VariableType::Number.matches_value(&json!(42)));
        assert!(VariableType::Boolean.matches_value(&json!(true)));
        assert!(VariableType::Any.matches_value(&json!({"key": "value"})));
        
        assert!(!VariableType::String.matches_value(&json!(123)));
        assert!(!VariableType::Number.matches_value(&json!("not a number")));
    }
    
    #[test]
    fn test_template_variables() {
        let mut vars = TemplateVariables::new();
        vars.insert("name", json!("Alice"));
        vars.insert("age", json!(30));
        
        assert_eq!(vars.get("name"), Some(&json!("Alice")));
        assert_eq!(vars.get("age"), Some(&json!(30)));
        assert!(vars.contains("name"));
        assert!(!vars.contains("unknown"));
    }
    
    #[test]
    fn test_template_metrics() {
        let metrics = TemplateMetrics::new();
        
        metrics.record_render("test", Duration::from_millis(10));
        metrics.record_render("test", Duration::from_millis(20));
        metrics.record_cache_hit();
        metrics.record_cache_hit();
        metrics.record_cache_miss();
        
        let avg = metrics.average_render_time("test").unwrap();
        assert_eq!(avg.as_millis(), 15);
        
        let hit_rate = metrics.cache_hit_rate();
        assert!((hit_rate - 0.6667).abs() < 0.01);
    }
}