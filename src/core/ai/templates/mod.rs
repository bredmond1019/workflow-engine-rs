//! # AI Prompt Template System
//!
//! This module provides a comprehensive prompt template and management system for AI agents.
//! It supports variable substitution, template composition, versioning, and performance metrics.
//!
//! ## Features
//!
//! - **Type-safe variable substitution**: Variables are validated at compile time
//! - **Template composition**: Templates can inherit from and include other templates
//! - **Multiple output formats**: Support for text, JSON, and structured data
//! - **Template versioning**: Track template history and changes
//! - **Performance metrics**: Monitor template rendering performance
//! - **Context-aware selection**: Automatically select appropriate templates
//! - **Safe interpolation**: Prevent code injection and unsafe operations
//!
//! ## Architecture
//!
//! The template system consists of several key components:
//!
//! 1. **Template Engine** (`engine.rs`): Core rendering logic with variable interpolation
//! 2. **Template Storage** (`storage.rs`): Persistence and retrieval of templates
//! 3. **Template Parser** (`parser.rs`): Parse and validate template syntax
//! 4. **Template Types** (`types.rs`): Core data structures and traits
//! 5. **Template Registry** (`registry.rs`): Runtime template management
//! 6. **Template Validator** (`validator.rs`): Validation and security checks
//!
//! ## Usage Examples
//!
//! ### Basic Template Usage
//!
//! ```rust
//! use ai_architecture_core::ai::templates::{Template, TemplateEngine};
//! use std::collections::HashMap;
//!
//! // Create a simple template
//! let template = Template::new(
//!     "greeting",
//!     "Hello {{name}}, welcome to {{system}}!",
//! )?;
//!
//! // Create engine and render
//! let engine = TemplateEngine::new();
//! let mut vars = HashMap::new();
//! vars.insert("name".to_string(), "Alice".to_string());
//! vars.insert("system".to_string(), "AI Workflow".to_string());
//!
//! let result = engine.render(&template, &vars)?;
//! assert_eq!(result, "Hello Alice, welcome to AI Workflow!");
//! ```
//!
//! ### Template Composition
//!
//! ```rust
//! use ai_architecture_core::ai::templates::{Template, TemplateEngine};
//!
//! // Base template
//! let base = Template::new(
//!     "base_prompt",
//!     "You are an AI assistant. {{content}}",
//! )?;
//!
//! // Child template that extends base
//! let child = Template::new(
//!     "customer_support",
//!     "{{> base_prompt content='Help the user with their issue: {{issue}}'}}"
//! )?;
//!
//! // Render with inheritance
//! let mut vars = HashMap::new();
//! vars.insert("issue".to_string(), "Password reset".to_string());
//! let result = engine.render(&child, &vars)?;
//! ```
//!
//! ### Conditional Logic
//!
//! ```rust
//! let template = Template::new(
//!     "conditional",
//!     r#"
//!     {{#if priority == "high"}}
//!         URGENT: {{message}}
//!     {{else}}
//!         {{message}}
//!     {{/if}}
//!     "#,
//! )?;
//! ```
//!
//! ### Loops and Iteration
//!
//! ```rust
//! let template = Template::new(
//!     "list_items",
//!     r#"
//!     Items to process:
//!     {{#each items}}
//!     - {{this.name}}: {{this.description}}
//!     {{/each}}
//!     "#,
//! )?;
//! ```
//!
//! ### JSON Output Format
//!
//! ```rust
//! use ai_architecture_core::ai::templates::{Template, OutputFormat};
//!
//! let template = Template::new(
//!     "json_response",
//!     r#"{
//!         "status": "{{status}}",
//!         "data": {
//!             "user": "{{user}}",
//!             "message": "{{message}}"
//!         }
//!     }"#,
//! )?
//! .with_output_format(OutputFormat::Json);
//!
//! let result = engine.render(&template, &vars)?;
//! let parsed: serde_json::Value = serde_json::from_str(&result)?;
//! ```
//!
//! ## Template Versioning
//!
//! ```rust
//! use ai_architecture_core::ai::templates::{TemplateStorage, Template};
//!
//! let storage = TemplateStorage::new("templates.db")?;
//!
//! // Save template with version
//! let template = Template::new("greeting", "Hello {{name}}!")?;
//! storage.save(&template)?;
//!
//! // Update template (creates new version)
//! let updated = template.with_content("Hi {{name}}, how are you?");
//! storage.save(&updated)?;
//!
//! // Retrieve specific version
//! let v1 = storage.get("greeting", Some(1))?;
//! let latest = storage.get("greeting", None)?;
//! ```
//!
//! ## Performance Metrics
//!
//! ```rust
//! use ai_architecture_core::ai::templates::{TemplateMetrics, TemplateEngine};
//!
//! let engine = TemplateEngine::new().with_metrics();
//!
//! // Render and collect metrics
//! let result = engine.render(&template, &vars)?;
//!
//! // Get metrics
//! let metrics = engine.metrics();
//! println!("Render time: {:?}", metrics.average_render_time());
//! println!("Cache hit rate: {:.2}%", metrics.cache_hit_rate() * 100.0);
//! ```
//!
//! ## Security Features
//!
//! The template system includes several security features:
//!
//! - **Input sanitization**: Automatic escaping of user input
//! - **Code injection prevention**: No arbitrary code execution
//! - **Resource limits**: Prevent infinite loops and excessive memory usage
//! - **Whitelist-based functions**: Only approved functions available in templates
//!
//! ## Best Practices
//!
//! 1. **Use typed variables**: Define variable schemas for type safety
//! 2. **Version templates**: Always version production templates
//! 3. **Monitor performance**: Track rendering times and cache efficiency
//! 4. **Validate early**: Validate templates at registration time
//! 5. **Use composition**: Break complex templates into reusable components
//! 6. **Test thoroughly**: Test templates with edge cases and invalid input

pub mod engine;
pub mod parser;
pub mod registry;
pub mod storage;
pub mod types;
pub mod validator;

// Re-export core types
pub use engine::{TemplateEngine, EngineConfig, TemplateRenderContext};
pub use parser::{TemplateParser, ParseError, TemplateAst};
pub use registry::{TemplateRegistry, RegistryError, create_default_registry};
pub use storage::{TemplateStorage, StorageBackend, StorageError};
pub use types::{
    Template, TemplateId, TemplateVersion, TemplateMetadata,
    Variable, VariableType, OutputFormat, TemplateError,
    CompiledTemplate, TemplateMetrics, TemplateVariables,
};
pub use validator::{TemplateValidator, ValidationError, ValidationRule};

use crate::core::error::WorkflowError;
use std::collections::HashMap;

/// Main template manager that coordinates all template operations
pub struct TemplateManager {
    engine: TemplateEngine,
    registry: TemplateRegistry,
    storage: Box<dyn StorageBackend>,
    metrics: TemplateMetrics,
}

impl std::fmt::Debug for TemplateManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TemplateManager")
            .field("engine", &self.engine)
            .field("registry", &self.registry)
            .field("storage", &"<StorageBackend>")
            .field("metrics", &self.metrics)
            .finish()
    }
}

impl TemplateManager {
    /// Create a new template manager with default configuration
    pub fn new() -> Result<Self, TemplateError> {
        Self::with_config(EngineConfig::default())
    }
    
    /// Create a new template manager with custom configuration
    pub fn with_config(config: EngineConfig) -> Result<Self, TemplateError> {
        Ok(Self {
            engine: TemplateEngine::with_config(config),
            registry: TemplateRegistry::new(),
            storage: Box::new(storage::MemoryStorage::new()),
            metrics: TemplateMetrics::new(),
        })
    }
    
    /// Set the storage backend
    pub fn with_storage(mut self, storage: Box<dyn StorageBackend>) -> Self {
        self.storage = storage;
        self
    }
    
    /// Register a template
    pub fn register(&mut self, template: Template) -> Result<(), TemplateError> {
        // Validate template
        let validator = TemplateValidator::new();
        validator.validate(&template)?;
        
        // Store template
        self.storage.save(&template)?;
        
        // Register in runtime registry
        self.registry.register(template)?;
        
        Ok(())
    }
    
    /// Render a template by ID
    pub fn render(
        &self,
        template_id: &str,
        variables: &HashMap<String, serde_json::Value>,
    ) -> Result<String, TemplateError> {
        let start = std::time::Instant::now();
        
        // Get template from registry
        let template = self.registry.get(template_id)?;
        
        // Create render context
        let context = TemplateRenderContext::new(variables.clone())
            .with_registry(&self.registry);
        
        // Render template
        let result = self.engine.render_with_context(&template, context)?;
        
        // Update metrics
        self.metrics.record_render(template_id, start.elapsed());
        
        Ok(result)
    }
    
    /// Render a template with context-aware selection
    pub fn render_contextual(
        &self,
        context_type: &str,
        variables: &HashMap<String, serde_json::Value>,
    ) -> Result<String, TemplateError> {
        // Select appropriate template based on context
        let template_id = self.select_template_for_context(context_type, variables)?;
        
        self.render(&template_id, variables)
    }
    
    /// Select the best template for a given context
    fn select_template_for_context(
        &self,
        context_type: &str,
        variables: &HashMap<String, serde_json::Value>,
    ) -> Result<String, TemplateError> {
        // Get all templates matching the context type
        let candidates = self.registry.find_by_context(context_type)?;
        
        if candidates.is_empty() {
            return Err(TemplateError::NotFound {
                id: context_type.to_string(),
            });
        }
        
        // Score templates based on variable compatibility
        let mut best_match = None;
        let mut best_score = 0;
        
        for template in candidates {
            let score = self.score_template_match(&template, variables);
            if score > best_score {
                best_score = score;
                best_match = Some(template.id.0.clone());
            }
        }
        
        best_match.ok_or_else(|| TemplateError::NotFound {
            id: context_type.to_string(),
        })
    }
    
    /// Score how well a template matches the provided variables
    fn score_template_match(
        &self,
        template: &Template,
        variables: &HashMap<String, serde_json::Value>,
    ) -> usize {
        let mut score = 1; // Base score for any template
        
        // Check required variables
        for (var_name, var_type) in &template.variables {
            if let Some(value) = variables.get(var_name) {
                if var_type.matches_value(value) {
                    score += 2; // Correct type match
                } else {
                    score += 1; // Variable exists but wrong type
                }
            }
        }
        
        // Bonus for exact variable count match
        if template.variables.len() == variables.len() {
            score += 1;
        }
        
        score
    }
    
    /// Get template metrics
    pub fn metrics(&self) -> &TemplateMetrics {
        &self.metrics
    }
    
    /// List all registered templates
    pub fn list_templates(&self) -> Result<Vec<TemplateMetadata>, TemplateError> {
        Ok(self.storage.list()?)
    }
    
    /// Get template history
    pub fn get_history(&self, template_id: &str) -> Result<Vec<TemplateVersion>, TemplateError> {
        Ok(self.storage.get_history(template_id)?)
    }
}

/// Quick render function for one-off template rendering
pub fn render_template(
    template_str: &str,
    variables: &HashMap<String, serde_json::Value>,
) -> Result<String, TemplateError> {
    let template = Template::parse("temp", template_str)?;
    let engine = TemplateEngine::new();
    engine.render(&template, &TemplateVariables::from_map(variables.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_template_manager_basic() {
        let mut manager = TemplateManager::new().unwrap();
        
        // Register a template
        let template = Template::new("greeting", "Hello {{name}}!").unwrap();
        manager.register(template).unwrap();
        
        // Render template
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), json!("World"));
        
        let result = manager.render("greeting", &vars).unwrap();
        assert_eq!(result, "Hello World!");
    }
    
    #[test]
    fn test_contextual_selection() {
        let mut manager = TemplateManager::new().unwrap();
        
        // Register templates for different contexts - need to also register them in registry
        let casual = Template::new("greeting_casual", "Hey {{name}}!")
            .unwrap()
            .with_context("casual");
        let formal = Template::new("greeting_formal", "Good day, {{name}}.")
            .unwrap()
            .with_context("formal");
            
        manager.register(casual).unwrap();
        manager.register(formal).unwrap();
        
        // Test contextual rendering
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), json!("Alice"));
        
        let casual_result = manager.render_contextual("casual", &vars).unwrap();
        assert_eq!(casual_result, "Hey Alice!");
        
        let formal_result = manager.render_contextual("formal", &vars).unwrap();
        assert_eq!(formal_result, "Good day, Alice.");
    }
    
    #[test]
    fn test_quick_render() {
        let mut vars = HashMap::new();
        vars.insert("item".to_string(), json!("template"));
        vars.insert("status".to_string(), json!("working"));
        
        let result = render_template(
            "The {{item}} is {{status}}!",
            &vars
        ).unwrap();
        
        assert_eq!(result, "The template is working!");
    }
}