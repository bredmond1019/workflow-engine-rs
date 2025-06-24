//! # Template Registry
//!
//! This module provides runtime template management and discovery functionality.

use super::{
    Template, TemplateId, TemplateError, CompiledTemplate,
    VariableType, OutputFormat,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Registry-specific errors
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("Template already exists: {id}")]
    AlreadyExists { id: String },
    
    #[error("Template not found: {id}")]
    NotFound { id: String },
    
    #[error("Circular dependency detected: {id}")]
    CircularDependency { id: String },
    
    #[error("Invalid template reference: {reference}")]
    InvalidReference { reference: String },
}

impl From<RegistryError> for TemplateError {
    fn from(error: RegistryError) -> Self {
        match error {
            RegistryError::NotFound { id } => TemplateError::NotFound { id },
            RegistryError::CircularDependency { id } => TemplateError::CircularDependency { 
                template_id: id 
            },
            _ => TemplateError::ValidationError {
                message: error.to_string(),
            },
        }
    }
}

impl From<TemplateError> for RegistryError {
    fn from(error: TemplateError) -> Self {
        match error {
            TemplateError::NotFound { id } => RegistryError::NotFound { id },
            TemplateError::CircularDependency { template_id } => RegistryError::CircularDependency { 
                id: template_id 
            },
            _ => RegistryError::InvalidReference {
                reference: error.to_string(),
            },
        }
    }
}

/// Runtime template registry
#[derive(Debug, Clone)]
pub struct TemplateRegistry {
    templates: Arc<RwLock<HashMap<TemplateId, Arc<Template>>>>,
    compiled_cache: Arc<RwLock<HashMap<TemplateId, CompiledTemplate>>>,
    context_index: Arc<RwLock<HashMap<String, Vec<TemplateId>>>>,
}

impl TemplateRegistry {
    /// Create new empty registry
    pub fn new() -> Self {
        Self {
            templates: Arc::new(RwLock::new(HashMap::new())),
            compiled_cache: Arc::new(RwLock::new(HashMap::new())),
            context_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a template
    pub fn register(&mut self, template: Template) -> Result<(), TemplateError> {
        let id = template.id.clone();
        
        // Check for circular dependencies
        self.validate_dependencies(&template)?;
        
        // Add to main registry
        let mut templates = self.templates.write()
            .map_err(|_| TemplateError::LockError {
                message: "Failed to acquire write lock on templates".to_string(),
            })?;
        if templates.contains_key(&id) {
            return Err(TemplateError::ValidationError {
                message: format!("Template already exists: {}", id),
            });
        }
        
        // Index by context if specified
        if let Some(context) = &template.context {
            let mut index = self.context_index.write()
                .map_err(|_| TemplateError::LockError {
                    message: "Failed to acquire write lock on context index".to_string(),
                })?;
            index.entry(context.clone())
                .or_insert_with(Vec::new)
                .push(id.clone());
        }
        
        templates.insert(id, Arc::new(template));
        Ok(())
    }
    
    /// Get a template by ID
    pub fn get(&self, id: &str) -> Result<Arc<Template>, TemplateError> {
        let templates = self.templates.read()
            .map_err(|_| TemplateError::LockError {
                message: "Failed to acquire read lock on templates".to_string(),
            })?;
        let template_id = TemplateId::from(id);
        
        templates.get(&template_id)
            .cloned()
            .ok_or_else(|| TemplateError::NotFound {
                id: id.to_string(),
            })
    }
    
    /// Get a compiled template
    pub fn get_compiled(&self, id: &str) -> Result<CompiledTemplate, TemplateError> {
        // Check cache first
        {
            let cache = self.compiled_cache.read()
                .map_err(|_| TemplateError::LockError {
                    message: "Failed to acquire read lock on compiled cache".to_string(),
                })?;
            if let Some(compiled) = cache.get(&TemplateId::from(id)) {
                return Ok(compiled.clone());
            }
        }
        
        // Compile and cache
        let template = self.get(id)?;
        let compiled = self.compile_template(&template)?;
        
        let mut cache = self.compiled_cache.write()
            .map_err(|_| TemplateError::LockError {
                message: "Failed to acquire write lock on compiled cache".to_string(),
            })?;
        cache.insert(template.id.clone(), compiled.clone());
        
        Ok(compiled)
    }
    
    /// Find templates by context
    pub fn find_by_context(&self, context: &str) -> Result<Vec<Arc<Template>>, TemplateError> {
        let index = self.context_index.read()
            .map_err(|_| TemplateError::LockError {
                message: "Failed to acquire read lock on context index".to_string(),
            })?;
        let templates = self.templates.read()
            .map_err(|_| TemplateError::LockError {
                message: "Failed to acquire read lock on templates".to_string(),
            })?;
        
        if let Some(template_ids) = index.get(context) {
            Ok(template_ids.iter()
                .filter_map(|id| templates.get(id).cloned())
                .collect())
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Update a template
    pub fn update(&mut self, template: Template) -> Result<(), TemplateError> {
        let id = template.id.clone();
        
        // Validate dependencies
        self.validate_dependencies(&template)?;
        
        // Update registry
        let mut templates = self.templates.write()
            .map_err(|_| TemplateError::LockError {
                message: "Failed to acquire write lock on templates".to_string(),
            })?;
        if !templates.contains_key(&id) {
            return Err(TemplateError::NotFound {
                id: id.to_string(),
            });
        }
        
        // Clear compiled cache for this template
        {
            let mut cache = self.compiled_cache.write()
                .map_err(|_| TemplateError::LockError {
                    message: "Failed to acquire write lock on compiled cache".to_string(),
                })?;
            cache.remove(&id);
        }
        
        // Update context index if needed
        if let Some(old_template) = templates.get(&id) {
            if old_template.context != template.context {
                let mut index = self.context_index.write()
                    .map_err(|_| TemplateError::LockError {
                        message: "Failed to acquire write lock on context index".to_string(),
                    })?;
                
                // Remove from old context
                if let Some(old_context) = &old_template.context {
                    if let Some(ids) = index.get_mut(old_context) {
                        ids.retain(|tid| tid != &id);
                    }
                }
                
                // Add to new context
                if let Some(new_context) = &template.context {
                    index.entry(new_context.clone())
                        .or_insert_with(Vec::new)
                        .push(id.clone());
                }
            }
        }
        
        templates.insert(id, Arc::new(template));
        Ok(())
    }
    
    /// Remove a template
    pub fn remove(&mut self, id: &str) -> Result<(), TemplateError> {
        let template_id = TemplateId::from(id);
        
        let mut templates = self.templates.write()
            .map_err(|_| TemplateError::LockError {
                message: "Failed to acquire write lock on templates".to_string(),
            })?;
        let template = templates.remove(&template_id)
            .ok_or_else(|| TemplateError::NotFound {
                id: id.to_string(),
            })?;
        
        // Remove from context index
        if let Some(context) = &template.context {
            let mut index = self.context_index.write()
                .map_err(|_| TemplateError::LockError {
                    message: "Failed to acquire write lock on context index".to_string(),
                })?;
            if let Some(ids) = index.get_mut(context) {
                ids.retain(|tid| tid != &template_id);
            }
        }
        
        // Clear compiled cache
        {
            let mut cache = self.compiled_cache.write()
                .map_err(|_| TemplateError::LockError {
                    message: "Failed to acquire write lock on compiled cache".to_string(),
                })?;
            cache.remove(&template_id);
        }
        
        Ok(())
    }
    
    /// List all templates
    pub fn list(&self) -> Result<Vec<Arc<Template>>, TemplateError> {
        let templates = self.templates.read()
            .map_err(|_| TemplateError::LockError {
                message: "Failed to acquire read lock on templates".to_string(),
            })?;
        Ok(templates.values().cloned().collect())
    }
    
    /// Clear the registry
    pub fn clear(&mut self) -> Result<(), TemplateError> {
        let mut templates = self.templates.write()
            .map_err(|_| TemplateError::LockError {
                message: "Failed to acquire write lock on templates".to_string(),
            })?;
        let mut cache = self.compiled_cache.write()
            .map_err(|_| TemplateError::LockError {
                message: "Failed to acquire write lock on compiled cache".to_string(),
            })?;
        let mut index = self.context_index.write()
            .map_err(|_| TemplateError::LockError {
                message: "Failed to acquire write lock on context index".to_string(),
            })?;
        
        templates.clear();
        cache.clear();
        index.clear();
        Ok(())
    }
    
    /// Validate template dependencies
    fn validate_dependencies(&self, template: &Template) -> Result<(), RegistryError> {
        let mut visited = std::collections::HashSet::new();
        let mut path = Vec::new();
        
        self.check_circular_deps_for_template(template, &mut visited, &mut path)?;
        
        // Validate parent exists
        if let Some(parent_id) = &template.parent {
            let templates = self.templates.read().unwrap();
            if !templates.contains_key(parent_id) {
                return Err(RegistryError::InvalidReference {
                    reference: parent_id.to_string(),
                });
            }
        }
        
        // Validate includes exist
        for include_id in &template.includes {
            let templates = self.templates.read().unwrap();
            if !templates.contains_key(include_id) {
                return Err(RegistryError::InvalidReference {
                    reference: include_id.to_string(),
                });
            }
        }
        
        Ok(())
    }
    
    /// Check for circular dependencies for a specific template being added/updated
    fn check_circular_deps_for_template(
        &self,
        template: &Template,
        visited: &mut std::collections::HashSet<TemplateId>,
        path: &mut Vec<TemplateId>,
    ) -> Result<(), TemplateError> {
        let id = &template.id;
        
        if path.contains(id) {
            return Err(TemplateError::CircularDependency {
                template_id: id.to_string(),
            });
        }
        
        if visited.contains(id) {
            return Ok(());
        }
        
        visited.insert(id.clone());
        path.push(id.clone());
        
        // Check parent
        if let Some(parent_id) = &template.parent {
            self.check_circular_deps(parent_id, visited, path)?;
        }
        
        // Check includes
        for include_id in &template.includes {
            self.check_circular_deps(include_id, visited, path)?;
        }
        
        path.pop();
        Ok(())
    }

    /// Check for circular dependencies
    fn check_circular_deps(
        &self,
        id: &TemplateId,
        visited: &mut std::collections::HashSet<TemplateId>,
        path: &mut Vec<TemplateId>,
    ) -> Result<(), TemplateError> {
        if path.contains(id) {
            return Err(TemplateError::CircularDependency {
                template_id: id.to_string(),
            });
        }
        
        if visited.contains(id) {
            return Ok(());
        }
        
        visited.insert(id.clone());
        path.push(id.clone());
        
        let templates = self.templates.read()
            .map_err(|_| TemplateError::LockError {
                message: "Failed to acquire read lock on templates".to_string(),
            })?;
        if let Some(template) = templates.get(id) {
            // Check parent
            if let Some(parent_id) = &template.parent {
                self.check_circular_deps(parent_id, visited, path)?;
            }
            
            // Check includes
            for include_id in &template.includes {
                self.check_circular_deps(include_id, visited, path)?;
            }
        }
        
        path.pop();
        Ok(())
    }
    
    /// Compile a template
    fn compile_template(&self, template: &Template) -> Result<CompiledTemplate, RegistryError> {
        // For now, just create a simple compiled template
        // In production, this would actually parse and compile the template
        Ok(CompiledTemplate {
            id: template.id.clone(),
            ast: Arc::new(super::parser::TemplateAst::Text(template.content.clone())),
            variables: template.variables.clone(),
            output_format: template.output_format,
        })
    }
    
    /// Get registry statistics
    pub fn stats(&self) -> Result<RegistryStats, TemplateError> {
        let templates = self.templates.read()
            .map_err(|_| TemplateError::LockError {
                message: "Failed to acquire read lock on templates".to_string(),
            })?;
        let cache = self.compiled_cache.read()
            .map_err(|_| TemplateError::LockError {
                message: "Failed to acquire read lock on compiled cache".to_string(),
            })?;
        let index = self.context_index.read()
            .map_err(|_| TemplateError::LockError {
                message: "Failed to acquire read lock on context index".to_string(),
            })?;
        
        Ok(RegistryStats {
            total_templates: templates.len(),
            compiled_templates: cache.len(),
            contexts: index.len(),
            total_includes: templates.values()
                .map(|t| t.includes.len())
                .sum(),
        })
    }
}

/// Registry statistics
#[derive(Debug, Clone)]
pub struct RegistryStats {
    pub total_templates: usize,
    pub compiled_templates: usize,
    pub contexts: usize,
    pub total_includes: usize,
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Pre-populated registry with common templates
pub fn create_default_registry() -> Result<TemplateRegistry, RegistryError> {
    let mut registry = TemplateRegistry::new();
    
    // System prompt template
    let system_prompt = Template::new(
        "system_prompt",
        "You are {{agent_name}}, an AI assistant. {{instructions}}"
    ).map_err(|e| TemplateError::ValidationError {
        message: format!("Failed to create system_prompt template: {}", e),
    })?
    .with_variable("agent_name", VariableType::String)
    .with_variable("instructions", VariableType::String);
    
    registry.register(system_prompt)?;
    
    // User message template
    let user_message = Template::new(
        "user_message",
        "{{#if context}}Context: {{context}}\n\n{{/if}}{{message}}"
    ).map_err(|e| TemplateError::ValidationError {
        message: format!("Failed to create user_message template: {}", e),
    })?
    .with_variable("message", VariableType::String)
    .with_variable("context", VariableType::String);
    
    registry.register(user_message)?;
    
    // JSON response template
    let json_response = Template::new(
        "json_response",
        r#"{
    "status": "{{status}}",
    "data": {{json data}},
    "timestamp": "{{now}}"
}"#
    ).map_err(|e| TemplateError::ValidationError {
        message: format!("Failed to create json_response template: {}", e),
    })?
    .with_output_format(OutputFormat::Json)
    .with_variable("status", VariableType::String)
    .with_variable("data", VariableType::Any);
    
    registry.register(json_response)?;
    
    // Error response template
    let error_response = Template::new(
        "error_response",
        "Error: {{error_message}}\nCode: {{error_code}}"
    ).map_err(|e| TemplateError::ValidationError {
        message: format!("Failed to create error_response template: {}", e),
    })?
    .with_variable("error_message", VariableType::String)
    .with_variable("error_code", VariableType::String)
    .with_context("error");
    
    registry.register(error_response)?;
    
    Ok(registry)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_registry_basic_operations() {
        let mut registry = TemplateRegistry::new();
        
        // Register template
        let template = Template::new("test", "Hello {{name}}!").expect("Failed to create test template");
        registry.register(template.clone()).expect("Failed to register template");
        
        // Get template
        let retrieved = registry.get("test").unwrap();
        assert_eq!(retrieved.id, template.id);
        
        // Update template
        let updated = template.with_content("Hi {{name}}!");
        registry.update(updated).unwrap();
        
        let retrieved = registry.get("test").unwrap();
        assert_eq!(retrieved.content, "Hi {{name}}!");
        
        // Remove template
        registry.remove("test").unwrap();
        assert!(registry.get("test").is_err());
    }
    
    #[test]
    fn test_context_index() {
        let mut registry = TemplateRegistry::new();
        
        // Register templates with contexts
        let greeting = Template::new("greeting", "Hello!")
            .unwrap()
            .with_context("casual");
        let formal = Template::new("formal", "Good day.")
            .unwrap()
            .with_context("formal");
        let casual2 = Template::new("casual2", "Hey!")
            .unwrap()
            .with_context("casual");
        
        registry.register(greeting).unwrap();
        registry.register(formal).unwrap();
        registry.register(casual2).unwrap();
        
        // Find by context
        let casual_templates = registry.find_by_context("casual").unwrap();
        assert_eq!(casual_templates.len(), 2);
        
        let formal_templates = registry.find_by_context("formal").unwrap();
        assert_eq!(formal_templates.len(), 1);
    }
    
    #[test]
    fn test_circular_dependency_detection() {
        let mut registry = TemplateRegistry::new();
        
        // Create templates with circular dependency
        let template_a = Template::new("a", "Template A").unwrap();
        let template_b = Template::new("b", "Template B").unwrap();
        
        // Register both templates first
        registry.register(template_a).unwrap();
        registry.register(template_b).unwrap();
        
        // Now try to update with includes that create circular dependency
        let template_a_with_include = Template::new("a", "Template A")
            .unwrap()
            .with_include("b");
        
        let template_b_with_include = Template::new("b", "Template B")
            .unwrap()
            .with_include("a");
        
        // Update first template
        registry.update(template_a_with_include).unwrap();
        
        // Updating second should fail due to circular dependency
        let result = registry.update(template_b_with_include);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_stats() {
        let mut registry = TemplateRegistry::new();
        
        // Register some templates
        let t1 = Template::new("t1", "Template 1")
            .unwrap()
            .with_include("t2");
        let t2 = Template::new("t2", "Template 2").unwrap();
        let t3 = Template::new("t3", "Template 3")
            .unwrap()
            .with_context("test");
        
        registry.register(t2).unwrap(); // Register t2 first
        registry.register(t1).unwrap();
        registry.register(t3).unwrap();
        
        let stats = registry.stats().unwrap();
        assert_eq!(stats.total_templates, 3);
        assert_eq!(stats.total_includes, 1);
        assert_eq!(stats.contexts, 1);
    }
    
    #[test]
    fn test_default_registry() {
        let registry = create_default_registry().unwrap();
        
        // Check that default templates are registered
        assert!(registry.get("system_prompt").is_ok());
        assert!(registry.get("user_message").is_ok());
        assert!(registry.get("json_response").is_ok());
        assert!(registry.get("error_response").is_ok());
        
        // Check context index
        let error_templates = registry.find_by_context("error").unwrap();
        assert_eq!(error_templates.len(), 1);
    }
}