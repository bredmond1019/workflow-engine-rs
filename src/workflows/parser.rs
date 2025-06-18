/*!
# Workflow Parser

This module implements parsing and loading of workflow definitions from YAML files
and other sources, enabling declarative workflow management.

Task 2.4: Build workflow parser to handle YAML definitions
*/

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde_json::Value;

use crate::core::error::WorkflowError;
use crate::workflows::schema::{
    WorkflowDefinition, WorkflowInstance, StepDefinition, StepType
};
use crate::workflows::executor::WorkflowFactory;

/// Workflow parser for loading definitions from various sources
pub struct WorkflowParser {
    loaded_workflows: HashMap<String, WorkflowDefinition>,
}

impl WorkflowParser {
    pub fn new() -> Self {
        Self {
            loaded_workflows: HashMap::new(),
        }
    }
    
    /// Load workflow definition from YAML file
    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<WorkflowDefinition, WorkflowError> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| WorkflowError::RuntimeError {
                message: format!("Failed to read workflow file: {}", e)
            })?;
        
        self.load_from_yaml(&content)
    }
    
    /// Load workflow definition from YAML string
    pub fn load_from_yaml(&mut self, yaml_content: &str) -> Result<WorkflowDefinition, WorkflowError> {
        let workflow: WorkflowDefinition = serde_yaml::from_str(yaml_content)
            .map_err(|e| WorkflowError::DeserializationError {
                message: format!("Failed to parse YAML workflow: {}", e)
            })?;
        
        // Validate the workflow
        self.validate_workflow(&workflow)?;
        
        // Store the workflow
        self.loaded_workflows.insert(workflow.name.clone(), workflow.clone());
        
        log::info!("Loaded workflow: {} (version {})", workflow.name, workflow.version);
        
        Ok(workflow)
    }
    
    /// Load workflow definition from JSON
    pub fn load_from_json(&mut self, json_content: &str) -> Result<WorkflowDefinition, WorkflowError> {
        let workflow: WorkflowDefinition = serde_json::from_str(json_content)
            .map_err(|e| WorkflowError::DeserializationError {
                message: format!("Failed to parse JSON workflow: {}", e)
            })?;
        
        // Validate the workflow
        self.validate_workflow(&workflow)?;
        
        // Store the workflow
        self.loaded_workflows.insert(workflow.name.clone(), workflow.clone());
        
        log::info!("Loaded workflow: {} (version {})", workflow.name, workflow.version);
        
        Ok(workflow)
    }
    
    /// Load workflow from a template
    pub fn load_template(&mut self, template_name: &str) -> Result<WorkflowDefinition, WorkflowError> {
        let workflow = match template_name {
            "research_to_documentation" => {
                crate::workflows::schema::templates::research_to_documentation()
            }
            "research_to_slack" => {
                crate::workflows::schema::templates::research_to_slack()
            }
            _ => {
                return Err(WorkflowError::InvalidInput(
                    format!("Unknown template: {}", template_name)
                ));
            }
        };
        
        // Store the workflow
        self.loaded_workflows.insert(workflow.name.clone(), workflow.clone());
        
        log::info!("Loaded template workflow: {} (version {})", workflow.name, workflow.version);
        
        Ok(workflow)
    }
    
    /// Get a loaded workflow by name
    pub fn get_workflow(&self, name: &str) -> Option<&WorkflowDefinition> {
        self.loaded_workflows.get(name)
    }
    
    /// List all loaded workflows
    pub fn list_workflows(&self) -> Vec<&WorkflowDefinition> {
        self.loaded_workflows.values().collect()
    }
    
    /// Create a workflow instance from a loaded definition
    pub fn create_instance(
        &self,
        workflow_name: &str,
        inputs: Value,
    ) -> Result<WorkflowInstance, WorkflowError> {
        let workflow = self.get_workflow(workflow_name)
            .ok_or_else(|| WorkflowError::InvalidInput(
                format!("Workflow '{}' not found", workflow_name)
            ))?;
        
        WorkflowFactory::create_instance(workflow.clone(), inputs)
    }
    
    /// Validate a workflow definition
    fn validate_workflow(&self, workflow: &WorkflowDefinition) -> Result<(), WorkflowError> {
        // Basic validation
        if workflow.name.is_empty() {
            return Err(WorkflowError::ValidationError {
                message: "Workflow name cannot be empty".to_string()
            });
        }
        
        if workflow.steps.is_empty() {
            return Err(WorkflowError::ValidationError {
                message: "Workflow must have at least one step".to_string()
            });
        }
        
        // Validate step dependencies
        let step_ids: std::collections::HashSet<_> = workflow.steps.iter()
            .map(|step| &step.id)
            .collect();
        
        for step in &workflow.steps {
            // Check for duplicate step IDs
            let step_count = workflow.steps.iter()
                .filter(|s| s.id == step.id)
                .count();
            
            if step_count > 1 {
                return Err(WorkflowError::ValidationError {
                    message: format!("Duplicate step ID: {}", step.id)
                });
            }
            
            // Check dependencies exist
            for dep in &step.depends_on {
                if !step_ids.contains(dep) {
                    return Err(WorkflowError::ValidationError {
                        message: format!("Step '{}' depends on unknown step '{}'", step.id, dep)
                    });
                }
            }
            
            // Validate step type specific requirements
            self.validate_step(step)?;
        }
        
        // Check for cycles in dependencies
        self.check_dependency_cycles(workflow)?;
        
        Ok(())
    }
    
    /// Validate individual step configuration
    fn validate_step(&self, step: &StepDefinition) -> Result<(), WorkflowError> {
        match &step.step_type {
            StepType::CrossSystem { system, operation, .. } => {
                if system.is_empty() {
                    return Err(WorkflowError::ValidationError {
                        message: format!("Step '{}': system cannot be empty", step.id)
                    });
                }
                if operation.is_empty() {
                    return Err(WorkflowError::ValidationError {
                        message: format!("Step '{}': operation cannot be empty", step.id)
                    });
                }
            }
            StepType::Node { node } => {
                if node.is_empty() {
                    return Err(WorkflowError::ValidationError {
                        message: format!("Step '{}': node type cannot be empty", step.id)
                    });
                }
            }
            StepType::Transform { engine, template } => {
                if engine.is_empty() {
                    return Err(WorkflowError::ValidationError {
                        message: format!("Step '{}': template engine cannot be empty", step.id)
                    });
                }
                if template.is_empty() {
                    return Err(WorkflowError::ValidationError {
                        message: format!("Step '{}': template cannot be empty", step.id)
                    });
                }
            }
            StepType::Condition { condition, then_steps, .. } => {
                if condition.is_empty() {
                    return Err(WorkflowError::ValidationError {
                        message: format!("Step '{}': condition cannot be empty", step.id)
                    });
                }
                if then_steps.is_empty() {
                    return Err(WorkflowError::ValidationError {
                        message: format!("Step '{}': condition must have at least one then_step", step.id)
                    });
                }
            }
            StepType::Loop { items, steps } => {
                if items.is_empty() {
                    return Err(WorkflowError::ValidationError {
                        message: format!("Step '{}': loop items cannot be empty", step.id)
                    });
                }
                if steps.is_empty() {
                    return Err(WorkflowError::ValidationError {
                        message: format!("Step '{}': loop must have at least one step", step.id)
                    });
                }
            }
        }
        
        Ok(())
    }
    
    /// Check for cycles in workflow dependencies
    fn check_dependency_cycles(&self, workflow: &WorkflowDefinition) -> Result<(), WorkflowError> {
        let mut visited = std::collections::HashSet::new();
        let mut rec_stack = std::collections::HashSet::new();
        
        // Build dependency graph
        let mut graph: HashMap<&String, Vec<&String>> = HashMap::new();
        for step in &workflow.steps {
            graph.insert(&step.id, step.depends_on.iter().collect());
        }
        
        // DFS to detect cycles
        for step in &workflow.steps {
            if self.has_cycle(&graph, &step.id, &mut visited, &mut rec_stack) {
                return Err(WorkflowError::CycleDetected);
            }
        }
        
        Ok(())
    }
    
    fn has_cycle(
        &self,
        graph: &HashMap<&String, Vec<&String>>,
        node: &String,
        visited: &mut std::collections::HashSet<String>,
        rec_stack: &mut std::collections::HashSet<String>,
    ) -> bool {
        if rec_stack.contains(node) {
            return true;
        }
        
        if visited.contains(node) {
            return false;
        }
        
        visited.insert(node.clone());
        rec_stack.insert(node.clone());
        
        if let Some(deps) = graph.get(node) {
            for dep in deps {
                if self.has_cycle(graph, dep, visited, rec_stack) {
                    return true;
                }
            }
        }
        
        rec_stack.remove(node);
        false
    }
}

/// Workflow registry for managing multiple workflows
pub struct WorkflowRegistry {
    parser: WorkflowParser,
    workflow_directory: Option<String>,
}

impl WorkflowRegistry {
    pub fn new() -> Self {
        Self {
            parser: WorkflowParser::new(),
            workflow_directory: None,
        }
    }
    
    /// Set the directory to scan for workflow files
    pub fn with_directory<P: AsRef<Path>>(mut self, directory: P) -> Self {
        self.workflow_directory = Some(directory.as_ref().to_string_lossy().to_string());
        self
    }
    
    /// Load all workflows from the configured directory
    pub fn load_all_workflows(&mut self) -> Result<Vec<String>, WorkflowError> {
        let directory = self.workflow_directory.as_ref()
            .ok_or_else(|| WorkflowError::RuntimeError {
                message: "No workflow directory configured".to_string()
            })?;
        
        let mut loaded_workflows = Vec::new();
        
        let entries = fs::read_dir(directory)
            .map_err(|e| WorkflowError::RuntimeError {
                message: format!("Failed to read workflow directory '{}': {}", directory, e)
            })?;
        
        for entry in entries {
            let entry = entry.map_err(|e| WorkflowError::RuntimeError {
                message: format!("Failed to read directory entry: {}", e)
            })?;
            
            let path = entry.path();
            if let Some(extension) = path.extension() {
                if extension == "yaml" || extension == "yml" {
                    match self.parser.load_from_file(&path) {
                        Ok(workflow) => {
                            loaded_workflows.push(workflow.name.clone());
                            log::info!("Loaded workflow from file: {:?}", path);
                        }
                        Err(e) => {
                            log::error!("Failed to load workflow from {:?}: {}", path, e);
                        }
                    }
                }
            }
        }
        
        Ok(loaded_workflows)
    }
    
    /// Load built-in template workflows
    pub fn load_templates(&mut self) -> Result<Vec<String>, WorkflowError> {
        let mut loaded_templates = Vec::new();
        
        let templates = ["research_to_documentation", "research_to_slack"];
        
        for template_name in &templates {
            match self.parser.load_template(template_name) {
                Ok(workflow) => {
                    loaded_templates.push(workflow.name.clone());
                    log::info!("Loaded template workflow: {}", template_name);
                }
                Err(e) => {
                    log::error!("Failed to load template '{}': {}", template_name, e);
                }
            }
        }
        
        Ok(loaded_templates)
    }
    
    /// Get workflow parser
    pub fn parser(&self) -> &WorkflowParser {
        &self.parser
    }
    
    /// Get mutable workflow parser
    pub fn parser_mut(&mut self) -> &mut WorkflowParser {
        &mut self.parser
    }
}

/// Helper function to create a workflow registry with common configurations
pub fn create_default_registry() -> Result<WorkflowRegistry, WorkflowError> {
    let mut registry = WorkflowRegistry::new();
    
    // Load built-in templates
    registry.load_templates()?;
    
    // Load from workflows directory if it exists
    let workflows_dir = std::env::var("WORKFLOWS_DIR")
        .unwrap_or_else(|_| "./workflows".to_string());
    
    if Path::new(&workflows_dir).exists() {
        registry = registry.with_directory(&workflows_dir);
        registry.load_all_workflows()?;
    }
    
    Ok(registry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;
    
    #[test]
    fn test_yaml_parsing() {
        let yaml_content = r#"
name: test_workflow
description: A test workflow
version: "1.0"
inputs:
  topic:
    type: string
    description: The topic to process
    required: true
steps:
  - id: step1
    name: First Step
    type:
      kind: cross_system
      system: ai-tutor
      operation: research
    input:
      topic: "{{ input.topic }}"
  - id: step2
    name: Second Step
    type:
      kind: node
      node: NotionClientNode
    input:
      title: "Research: {{ input.topic }}"
      content: "{{ steps.step1.output }}"
    depends_on:
      - step1
outputs:
  result: "{{ steps.step2.output }}"
"#;
        
        let mut parser = WorkflowParser::new();
        let workflow = parser.load_from_yaml(yaml_content).unwrap();
        
        assert_eq!(workflow.name, "test_workflow");
        assert_eq!(workflow.steps.len(), 2);
        assert_eq!(workflow.steps[0].id, "step1");
        assert_eq!(workflow.steps[1].depends_on, vec!["step1"]);
    }
    
    #[test]
    fn test_template_loading() {
        let mut parser = WorkflowParser::new();
        let workflow = parser.load_template("research_to_documentation").unwrap();
        
        assert_eq!(workflow.name, "research_to_documentation");
        assert!(!workflow.steps.is_empty());
    }
    
    #[test]
    fn test_cycle_detection() {
        let yaml_content = r#"
name: cyclic_workflow
description: A workflow with a cycle
version: "1.0"
steps:
  - id: step1
    type:
      kind: node
      node: TestNode
    depends_on:
      - step2
  - id: step2
    type:
      kind: node
      node: TestNode
    depends_on:
      - step1
"#;
        
        let mut parser = WorkflowParser::new();
        let result = parser.load_from_yaml(yaml_content);
        
        assert!(matches!(result, Err(WorkflowError::CycleDetected)));
    }
    
    #[test]
    fn test_workflow_registry() {
        let temp_dir = tempdir().unwrap();
        let workflow_path = temp_dir.path().join("test_workflow.yaml");
        
        let yaml_content = r#"
name: registry_test_workflow
description: Test workflow for registry
version: "1.0"
steps:
  - id: step1
    type:
      kind: node
      node: TestNode
"#;
        
        let mut file = File::create(&workflow_path).unwrap();
        file.write_all(yaml_content.as_bytes()).unwrap();
        
        let mut registry = WorkflowRegistry::new()
            .with_directory(temp_dir.path());
        
        let loaded = registry.load_all_workflows().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0], "registry_test_workflow");
        
        let workflow = registry.parser().get_workflow("registry_test_workflow");
        assert!(workflow.is_some());
    }
    
    #[test]
    fn test_instance_creation() {
        let mut parser = WorkflowParser::new();
        let workflow = parser.load_template("research_to_documentation").unwrap();
        
        let inputs = serde_json::json!({
            "topic": "machine learning",
            "difficulty": "intermediate"
        });
        
        let instance = parser.create_instance(&workflow.name, inputs).unwrap();
        assert_eq!(instance.inputs["topic"], "machine learning");
        assert_eq!(instance.inputs["difficulty"], "intermediate");
    }
}