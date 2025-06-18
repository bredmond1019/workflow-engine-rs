/*!
# Workflow Template Registry

This module provides a registry for workflow templates, enabling discovery,
instantiation, and management of predefined workflow patterns.

Task 4.4: Create workflow template registry and discovery
*/

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::core::error::WorkflowError;
use crate::workflows::schema::{WorkflowDefinition, templates};

/// Metadata about a workflow template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplateMetadata {
    /// Template identifier
    pub id: String,
    
    /// Human-readable name
    pub name: String,
    
    /// Template description
    pub description: String,
    
    /// Template version
    pub version: String,
    
    /// Template category
    pub category: String,
    
    /// Required inputs for this template
    pub required_inputs: Vec<String>,
    
    /// Optional inputs with defaults
    pub optional_inputs: Vec<String>,
    
    /// Expected outputs from this template
    pub outputs: Vec<String>,
    
    /// Estimated execution time in seconds
    pub estimated_duration: Option<u32>,
    
    /// Template complexity level
    pub complexity: TemplateComplexity,
    
    /// Template tags for filtering and search
    pub tags: Vec<String>,
    
    /// Template author/maintainer
    pub author: Option<String>,
    
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// Last modified timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Template complexity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TemplateComplexity {
    /// Simple single-step or few-step workflows
    Simple,
    /// Multi-step workflows with some dependencies
    Moderate,
    /// Complex workflows with multiple dependencies and conditional logic
    Complex,
    /// Advanced workflows with loops, conditions, and extensive cross-system calls
    Advanced,
}

/// Search criteria for templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSearchCriteria {
    /// Search by category
    pub category: Option<String>,
    
    /// Search by tags
    pub tags: Option<Vec<String>>,
    
    /// Search by complexity level
    pub complexity: Option<TemplateComplexity>,
    
    /// Search by text in name or description
    pub text: Option<String>,
    
    /// Filter by required inputs
    pub has_inputs: Option<Vec<String>>,
    
    /// Filter by expected outputs
    pub has_outputs: Option<Vec<String>>,
    
    /// Maximum estimated duration
    pub max_duration: Option<u32>,
}

/// Workflow template registry
pub struct WorkflowTemplateRegistry {
    templates: HashMap<String, WorkflowDefinition>,
    metadata: HashMap<String, WorkflowTemplateMetadata>,
}

impl WorkflowTemplateRegistry {
    /// Create a new template registry with built-in templates
    pub fn new() -> Self {
        let mut registry = Self {
            templates: HashMap::new(),
            metadata: HashMap::new(),
        };
        
        // Register built-in templates
        registry.register_builtin_templates();
        
        registry
    }
    
    /// Register all built-in workflow templates
    fn register_builtin_templates(&mut self) {
        // Research to Documentation template
        let research_template = templates::research_to_documentation();
        let research_metadata = WorkflowTemplateMetadata {
            id: "research_to_documentation".to_string(),
            name: "Research to Documentation".to_string(),
            description: "Research a topic and create comprehensive documentation in Notion".to_string(),
            version: "1.0".to_string(),
            category: "research".to_string(),
            required_inputs: vec!["topic".to_string()],
            optional_inputs: vec!["difficulty".to_string(), "max_sources".to_string()],
            outputs: vec![
                "research_summary".to_string(),
                "notion_page_url".to_string(),
                "notion_page_id".to_string(),
                "source_count".to_string(),
            ],
            estimated_duration: Some(600), // 10 minutes
            complexity: TemplateComplexity::Moderate,
            tags: vec![
                "research".to_string(),
                "documentation".to_string(),
                "notion".to_string(),
                "ai-tutor".to_string(),
            ],
            author: Some("AI Workflow System".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        self.templates.insert("research_to_documentation".to_string(), research_template);
        self.metadata.insert("research_to_documentation".to_string(), research_metadata);
        
        // Research to Slack template
        let slack_template = templates::research_to_slack();
        let slack_metadata = WorkflowTemplateMetadata {
            id: "research_to_slack".to_string(),
            name: "Research to Slack".to_string(),
            description: "Research a topic and post summary to Slack channel".to_string(),
            version: "1.0".to_string(),
            category: "research".to_string(),
            required_inputs: vec!["topic".to_string(), "channel".to_string()],
            optional_inputs: vec![],
            outputs: vec!["slack_message_id".to_string()],
            estimated_duration: Some(180), // 3 minutes
            complexity: TemplateComplexity::Simple,
            tags: vec![
                "research".to_string(),
                "slack".to_string(),
                "communication".to_string(),
                "ai-tutor".to_string(),
            ],
            author: Some("AI Workflow System".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        self.templates.insert("research_to_slack".to_string(), slack_template);
        self.metadata.insert("research_to_slack".to_string(), slack_metadata);
        
        // User Query Processing template
        let query_template = templates::user_query_processing();
        let query_metadata = WorkflowTemplateMetadata {
            id: "user_query_processing".to_string(),
            name: "User Query Processing".to_string(),
            description: "Process user queries through research, analysis, and response generation".to_string(),
            version: "1.0".to_string(),
            category: "query_processing".to_string(),
            required_inputs: vec!["query".to_string()],
            optional_inputs: vec!["context".to_string(), "response_format".to_string()],
            outputs: vec![
                "response".to_string(),
                "confidence".to_string(),
                "sources".to_string(),
                "intent".to_string(),
                "keywords".to_string(),
            ],
            estimated_duration: Some(300), // 5 minutes
            complexity: TemplateComplexity::Complex,
            tags: vec![
                "query".to_string(),
                "analysis".to_string(),
                "response".to_string(),
                "knowledge_base".to_string(),
                "ai-tutor".to_string(),
            ],
            author: Some("AI Workflow System".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        self.templates.insert("user_query_processing".to_string(), query_template);
        self.metadata.insert("user_query_processing".to_string(), query_metadata);
        
        // AI Content Generation template
        let content_template = templates::ai_content_generation();
        let content_metadata = WorkflowTemplateMetadata {
            id: "ai_content_generation".to_string(),
            name: "AI Content Generation".to_string(),
            description: "Generate AI-powered content based on requirements and templates".to_string(),
            version: "1.0".to_string(),
            category: "content_generation".to_string(),
            required_inputs: vec!["content_type".to_string(), "topic".to_string()],
            optional_inputs: vec!["requirements".to_string(), "template_id".to_string()],
            outputs: vec![
                "content".to_string(),
                "outline".to_string(),
                "quality_score".to_string(),
                "word_count".to_string(),
                "notion_page_url".to_string(),
                "sources_count".to_string(),
            ],
            estimated_duration: Some(900), // 15 minutes
            complexity: TemplateComplexity::Advanced,
            tags: vec![
                "content".to_string(),
                "generation".to_string(),
                "writing".to_string(),
                "notion".to_string(),
                "research".to_string(),
                "ai-tutor".to_string(),
            ],
            author: Some("AI Workflow System".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        self.templates.insert("ai_content_generation".to_string(), content_template);
        self.metadata.insert("ai_content_generation".to_string(), content_metadata);
        
        log::info!("Registered {} built-in workflow templates", self.templates.len());
    }
    
    /// Register a custom workflow template
    pub fn register_template(
        &mut self,
        template: WorkflowDefinition,
        metadata: WorkflowTemplateMetadata,
    ) -> Result<(), WorkflowError> {
        // Validate template
        self.validate_template(&template)?;
        
        // Check for ID conflicts
        if self.templates.contains_key(&template.name) {
            return Err(WorkflowError::ConfigurationError(
                format!("Template with ID '{}' already exists", template.name)
            ));
        }
        
        log::info!("Registering workflow template: {}", template.name);
        
        self.templates.insert(template.name.clone(), template);
        self.metadata.insert(metadata.id.clone(), metadata);
        
        Ok(())
    }
    
    /// Get a template by ID
    pub fn get_template(&self, template_id: &str) -> Option<&WorkflowDefinition> {
        self.templates.get(template_id)
    }
    
    /// Get template metadata by ID
    pub fn get_metadata(&self, template_id: &str) -> Option<&WorkflowTemplateMetadata> {
        self.metadata.get(template_id)
    }
    
    /// List all available templates
    pub fn list_templates(&self) -> Vec<&WorkflowTemplateMetadata> {
        self.metadata.values().collect()
    }
    
    /// Search templates by criteria
    pub fn search_templates(&self, criteria: &TemplateSearchCriteria) -> Vec<&WorkflowTemplateMetadata> {
        self.metadata
            .values()
            .filter(|metadata| self.matches_criteria(metadata, criteria))
            .collect()
    }
    
    /// Check if metadata matches search criteria
    fn matches_criteria(&self, metadata: &WorkflowTemplateMetadata, criteria: &TemplateSearchCriteria) -> bool {
        // Filter by category
        if let Some(ref category) = criteria.category {
            if &metadata.category != category {
                return false;
            }
        }
        
        // Filter by complexity
        if let Some(ref complexity) = criteria.complexity {
            if &metadata.complexity != complexity {
                return false;
            }
        }
        
        // Filter by tags
        if let Some(ref required_tags) = criteria.tags {
            if !required_tags.iter().all(|tag| metadata.tags.contains(tag)) {
                return false;
            }
        }
        
        // Filter by text search
        if let Some(ref text) = criteria.text {
            let text_lower = text.to_lowercase();
            if !metadata.name.to_lowercase().contains(&text_lower) &&
               !metadata.description.to_lowercase().contains(&text_lower) {
                return false;
            }
        }
        
        // Filter by required inputs
        if let Some(ref inputs) = criteria.has_inputs {
            if !inputs.iter().all(|input| metadata.required_inputs.contains(input)) {
                return false;
            }
        }
        
        // Filter by outputs
        if let Some(ref outputs) = criteria.has_outputs {
            if !outputs.iter().all(|output| metadata.outputs.contains(output)) {
                return false;
            }
        }
        
        // Filter by duration
        if let Some(max_duration) = criteria.max_duration {
            if let Some(duration) = metadata.estimated_duration {
                if duration > max_duration {
                    return false;
                }
            }
        }
        
        true
    }
    
    /// Get templates by category
    pub fn get_templates_by_category(&self, category: &str) -> Vec<&WorkflowTemplateMetadata> {
        self.metadata
            .values()
            .filter(|metadata| metadata.category == category)
            .collect()
    }
    
    /// Get templates by tags
    pub fn get_templates_by_tags(&self, tags: &[String]) -> Vec<&WorkflowTemplateMetadata> {
        self.metadata
            .values()
            .filter(|metadata| {
                tags.iter().any(|tag| metadata.tags.contains(tag))
            })
            .collect()
    }
    
    /// Get all available categories
    pub fn get_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self.metadata
            .values()
            .map(|metadata| metadata.category.clone())
            .collect();
        categories.sort();
        categories.dedup();
        categories
    }
    
    /// Get all available tags
    pub fn get_tags(&self) -> Vec<String> {
        let mut tags: Vec<String> = self.metadata
            .values()
            .flat_map(|metadata| metadata.tags.clone())
            .collect();
        tags.sort();
        tags.dedup();
        tags
    }
    
    /// Validate a workflow template
    fn validate_template(&self, template: &WorkflowDefinition) -> Result<(), WorkflowError> {
        // Basic validation
        if template.name.is_empty() {
            return Err(WorkflowError::ConfigurationError(
                "Template name cannot be empty".to_string()
            ));
        }
        
        if template.steps.is_empty() {
            return Err(WorkflowError::ConfigurationError(
                "Template must have at least one step".to_string()
            ));
        }
        
        // Validate step IDs are unique
        let mut step_ids = std::collections::HashSet::new();
        for step in &template.steps {
            if !step_ids.insert(&step.id) {
                return Err(WorkflowError::ConfigurationError(
                    format!("Duplicate step ID: {}", step.id)
                ));
            }
        }
        
        // Validate dependencies exist
        for step in &template.steps {
            for dep in &step.depends_on {
                if !step_ids.contains(&dep) {
                    return Err(WorkflowError::ConfigurationError(
                        format!("Step '{}' depends on non-existent step '{}'", step.id, dep)
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    /// Remove a template from the registry
    pub fn remove_template(&mut self, template_id: &str) -> Result<(), WorkflowError> {
        if !self.templates.contains_key(template_id) {
            return Err(WorkflowError::ConfigurationError(
                format!("Template '{}' not found", template_id)
            ));
        }
        
        self.templates.remove(template_id);
        self.metadata.remove(template_id);
        
        log::info!("Removed workflow template: {}", template_id);
        Ok(())
    }
    
    /// Get statistics about the registry
    pub fn get_statistics(&self) -> RegistryStatistics {
        let total_templates = self.templates.len();
        
        let mut categories_count = HashMap::new();
        let mut complexity_count = HashMap::new();
        
        for metadata in self.metadata.values() {
            *categories_count.entry(metadata.category.clone()).or_insert(0) += 1;
            *complexity_count.entry(metadata.complexity.clone()).or_insert(0) += 1;
        }
        
        let avg_duration = self.metadata
            .values()
            .filter_map(|m| m.estimated_duration)
            .map(|d| d as f64)
            .collect::<Vec<_>>();
        
        let average_duration = if !avg_duration.is_empty() {
            Some(avg_duration.iter().sum::<f64>() / avg_duration.len() as f64)
        } else {
            None
        };
        
        RegistryStatistics {
            total_templates,
            categories_count,
            complexity_count,
            average_duration,
        }
    }
}

/// Statistics about the template registry
#[derive(Debug, Serialize, Deserialize)]
pub struct RegistryStatistics {
    pub total_templates: usize,
    pub categories_count: HashMap<String, usize>,
    pub complexity_count: HashMap<TemplateComplexity, usize>,
    pub average_duration: Option<f64>,
}

impl Default for WorkflowTemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_registry_creation() {
        let registry = WorkflowTemplateRegistry::new();
        assert!(!registry.templates.is_empty());
        assert!(!registry.metadata.is_empty());
        assert_eq!(registry.templates.len(), registry.metadata.len());
    }
    
    #[test]
    fn test_template_retrieval() {
        let registry = WorkflowTemplateRegistry::new();
        
        let template = registry.get_template("research_to_documentation");
        assert!(template.is_some());
        assert_eq!(template.unwrap().name, "research_to_documentation");
        
        let metadata = registry.get_metadata("research_to_documentation");
        assert!(metadata.is_some());
        assert_eq!(metadata.unwrap().name, "Research to Documentation");
    }
    
    #[test]
    fn test_template_search() {
        let registry = WorkflowTemplateRegistry::new();
        
        // Search by category
        let criteria = TemplateSearchCriteria {
            category: Some("research".to_string()),
            tags: None,
            complexity: None,
            text: None,
            has_inputs: None,
            has_outputs: None,
            max_duration: None,
        };
        
        let results = registry.search_templates(&criteria);
        assert!(!results.is_empty());
        assert!(results.iter().all(|r| r.category == "research"));
    }
    
    #[test]
    fn test_template_search_by_tags() {
        let registry = WorkflowTemplateRegistry::new();
        
        let criteria = TemplateSearchCriteria {
            category: None,
            tags: Some(vec!["notion".to_string()]),
            complexity: None,
            text: None,
            has_inputs: None,
            has_outputs: None,
            max_duration: None,
        };
        
        let results = registry.search_templates(&criteria);
        assert!(!results.is_empty());
        assert!(results.iter().all(|r| r.tags.contains(&"notion".to_string())));
    }
    
    #[test]
    fn test_template_search_by_complexity() {
        let registry = WorkflowTemplateRegistry::new();
        
        let criteria = TemplateSearchCriteria {
            category: None,
            tags: None,
            complexity: Some(TemplateComplexity::Simple),
            text: None,
            has_inputs: None,
            has_outputs: None,
            max_duration: None,
        };
        
        let results = registry.search_templates(&criteria);
        assert!(results.iter().all(|r| r.complexity == TemplateComplexity::Simple));
    }
    
    #[test]
    fn test_categories_and_tags() {
        let registry = WorkflowTemplateRegistry::new();
        
        let categories = registry.get_categories();
        assert!(!categories.is_empty());
        assert!(categories.contains(&"research".to_string()));
        
        let tags = registry.get_tags();
        assert!(!tags.is_empty());
        assert!(tags.contains(&"notion".to_string()));
    }
    
    #[test]
    fn test_statistics() {
        let registry = WorkflowTemplateRegistry::new();
        let stats = registry.get_statistics();
        
        assert!(stats.total_templates > 0);
        assert!(!stats.categories_count.is_empty());
        assert!(!stats.complexity_count.is_empty());
    }
}