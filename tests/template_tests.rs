//! # Template System Integration Tests
//!
//! Comprehensive tests for the AI prompt template system.

use backend::core::ai::templates::{
    Template, TemplateEngine, TemplateManager, TemplateVariables,
    OutputFormat, VariableType, TemplateStorage, StorageBackend,
    TemplateValidator, ValidationRule, TemplateRegistry,
    create_default_registry, render_template,
};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_basic_template_rendering() {
    let engine = TemplateEngine::new();
    let template = Template::new("greeting", "Hello {{name}}, welcome to {{system}}!")
        .unwrap()
        .with_variable("name", VariableType::String)
        .with_variable("system", VariableType::String);
    
    let mut vars = TemplateVariables::new();
    vars.insert("name", json!("Alice"));
    vars.insert("system", json!("AI Workflow"));
    
    let result = engine.render(&template, &vars).unwrap();
    assert_eq!(result, "Hello Alice, welcome to AI Workflow!");
}

#[test]
fn test_template_with_conditionals() {
    let engine = TemplateEngine::new();
    let template = Template::new(
        "conditional",
        r#"{{#if premium}}Welcome, premium user {{name}}!{{else}}Hello {{name}}.{{/if}}"#
    ).unwrap();
    
    // Test premium user
    let mut vars = TemplateVariables::new();
    vars.insert("name", json!("Bob"));
    vars.insert("premium", json!(true));
    
    let result = engine.render(&template, &vars).unwrap();
    assert_eq!(result, "Welcome, premium user Bob!");
    
    // Test regular user
    vars.insert("premium", json!(false));
    let result = engine.render(&template, &vars).unwrap();
    assert_eq!(result, "Hello Bob.");
}

#[test]
fn test_template_with_loops() {
    let engine = TemplateEngine::new();
    let template = Template::new(
        "list",
        r#"Tasks:
{{#each tasks}}
- {{this.name}}: {{this.status}}
{{/each}}"#
    ).unwrap();
    
    let mut vars = TemplateVariables::new();
    vars.insert("tasks", json!([
        {"name": "Task 1", "status": "completed"},
        {"name": "Task 2", "status": "in_progress"},
        {"name": "Task 3", "status": "pending"}
    ]));
    
    let result = engine.render(&template, &vars).unwrap();
    assert!(result.contains("- Task 1: completed"));
    assert!(result.contains("- Task 2: in_progress"));
    assert!(result.contains("- Task 3: pending"));
}

#[test]
fn test_json_output_format() {
    let engine = TemplateEngine::new();
    let template = Template::new(
        "json_response",
        r#"{
    "user": "{{user}}",
    "data": {{json data}},
    "timestamp": "{{timestamp}}"
}"#
    ).unwrap()
    .with_output_format(OutputFormat::Json);
    
    let mut vars = TemplateVariables::new();
    vars.insert("user", json!("alice"));
    vars.insert("data", json!({"count": 42, "items": ["a", "b", "c"]}));
    vars.insert("timestamp", json!("2024-01-01T00:00:00Z"));
    
    let result = engine.render(&template, &vars).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
    
    assert_eq!(parsed["user"], "alice");
    assert_eq!(parsed["data"]["count"], 42);
    assert_eq!(parsed["data"]["items"], json!(["a", "b", "c"]));
}

#[test]
fn test_helper_functions() {
    let engine = TemplateEngine::new();
    
    // Test string manipulation helpers
    let template = Template::new(
        "helpers",
        r#"Upper: {{uppercase name}}
Lower: {{lowercase name}}
Capitalize: {{capitalize word}}"#
    ).unwrap();
    
    let mut vars = TemplateVariables::new();
    vars.insert("name", json!("John Doe"));
    vars.insert("word", json!("hello"));
    
    let result = engine.render(&template, &vars).unwrap();
    assert!(result.contains("Upper: JOHN DOE"));
    assert!(result.contains("Lower: john doe"));
    assert!(result.contains("Capitalize: Hello"));
}

#[test]
fn test_template_manager() {
    let mut manager = TemplateManager::new().unwrap();
    
    // Register templates
    let greeting = Template::new("greeting", "Hello {{name}}!")
        .unwrap()
        .with_context("casual");
    
    let formal = Template::new("formal_greeting", "Good day, {{title}} {{name}}.")
        .unwrap()
        .with_context("formal")
        .with_variable("title", VariableType::String);
    
    manager.register(greeting).unwrap();
    manager.register(formal).unwrap();
    
    // Test direct rendering
    let mut vars = HashMap::new();
    vars.insert("name".to_string(), json!("Smith"));
    
    let result = manager.render("greeting", &vars).unwrap();
    assert_eq!(result, "Hello Smith!");
    
    // Test contextual rendering
    vars.insert("title".to_string(), json!("Dr."));
    let result = manager.render_contextual("formal", &vars).unwrap();
    assert_eq!(result, "Good day, Dr. Smith.");
}

#[test]
fn test_template_storage() {
    let storage = TemplateStorage::memory();
    
    // Create and save template
    let template = Template::new("test", "Version 1: {{content}}")
        .unwrap()
        .with_tags(vec!["test".to_string(), "v1".to_string()]);
    
    storage.save(&template).unwrap();
    
    // Update template
    let updated = template.with_content("Version 2: {{content}}")
        .with_tags(vec!["test".to_string(), "v2".to_string()]);
    
    storage.save(&updated).unwrap();
    
    // Get latest version
    let latest = storage.get("test", None).unwrap();
    assert_eq!(latest.content, "Version 2: {{content}}");
    assert_eq!(latest.metadata.version, 2);
    
    // Get specific version
    let v1 = storage.get("test", Some(1)).unwrap();
    assert_eq!(v1.content, "Version 1: {{content}}");
    
    // Check history
    let history = storage.get_history("test").unwrap();
    assert_eq!(history.len(), 2);
}

#[test]
fn test_template_validation() {
    let validator = TemplateValidator::new();
    
    // Valid template
    let valid = Template::new("valid", "Hello {{name}}!").unwrap();
    assert!(validator.validate(&valid).is_ok());
    
    // Invalid syntax
    let invalid_syntax = Template::new("invalid", "Hello {{name").unwrap();
    assert!(validator.validate(&invalid_syntax).is_err());
    
    // Security violation
    let unsafe_template = Template::new(
        "unsafe",
        "Hello <script>alert('xss')</script>"
    ).unwrap();
    assert!(validator.validate(&unsafe_template).is_err());
}

#[test]
fn test_strict_validation() {
    let validator = TemplateValidator::strict();
    
    // Template with undefined variable
    let template = Template::new("test", "Hello {{name}} from {{location}}!")
        .unwrap()
        .with_variable("name", VariableType::String);
    // Missing 'location' variable definition
    
    let result = validator.validate(&template);
    assert!(result.is_err());
}

#[test]
fn test_template_inheritance() {
    let mut registry = TemplateRegistry::new();
    
    // Base template
    let base = Template::new(
        "base_layout",
        r#"<html>
<head><title>{{title}}</title></head>
<body>
{{content}}
</body>
</html>"#
    ).unwrap();
    
    registry.register(base).unwrap();
    
    // Child template that extends base
    let child = Template::new(
        "page",
        r#"{{> base_layout}}
<h1>{{heading}}</h1>
<p>{{body}}</p>"#
    ).unwrap()
    .with_parent("base_layout");
    
    registry.register(child).unwrap();
    
    // Verify no circular dependencies
    assert!(registry.get("page").is_ok());
}

#[test]
fn test_template_metrics() {
    let engine = TemplateEngine::new().with_metrics();
    let template = Template::new("test", "Hello {{name}}!").unwrap();
    
    let mut vars = TemplateVariables::new();
    vars.insert("name", json!("World"));
    
    // Render multiple times
    for _ in 0..5 {
        engine.render(&template, &vars).unwrap();
    }
    
    let metrics = engine.metrics().unwrap();
    
    // Check that metrics were recorded
    let avg_time = metrics.average_render_time("test");
    assert!(avg_time.is_some());
    
    // Cache should have some hits after first render
    let hit_rate = metrics.cache_hit_rate();
    assert!(hit_rate > 0.0);
}

#[test]
fn test_complex_template_scenario() {
    let mut manager = TemplateManager::new().unwrap();
    
    // Register a complex AI prompt template
    let ai_prompt = Template::new(
        "ai_analysis",
        r#"You are an AI assistant analyzing {{data_type}} data.

{{#if context}}
Context: {{context}}
{{/if}}

Data points to analyze:
{{#each data_points}}
- {{this.label}}: {{this.value}} ({{this.unit}})
{{/each}}

{{#if priority == "high"}}
PRIORITY: This is a high-priority analysis. Please provide detailed insights.
{{else}}
Please provide a summary analysis.
{{/if}}

Output format: {{output_format}}"#
    ).unwrap()
    .with_variable("data_type", VariableType::String)
    .with_variable("context", VariableType::String)
    .with_variable("data_points", VariableType::Array(Box::new(VariableType::Object(
        vec![
            ("label".to_string(), VariableType::String),
            ("value".to_string(), VariableType::Number),
            ("unit".to_string(), VariableType::String),
        ].into_iter().collect()
    ))))
    .with_variable("priority", VariableType::String)
    .with_variable("output_format", VariableType::String);
    
    manager.register(ai_prompt).unwrap();
    
    // Prepare complex data
    let mut vars = HashMap::new();
    vars.insert("data_type".to_string(), json!("performance"));
    vars.insert("context".to_string(), json!("Q4 2023 metrics"));
    vars.insert("data_points".to_string(), json!([
        {"label": "Response Time", "value": 125, "unit": "ms"},
        {"label": "Throughput", "value": 5000, "unit": "req/s"},
        {"label": "Error Rate", "value": 0.1, "unit": "%"}
    ]));
    vars.insert("priority".to_string(), json!("high"));
    vars.insert("output_format".to_string(), json!("detailed JSON report"));
    
    let result = manager.render("ai_analysis", &vars).unwrap();
    
    // Verify all components are present
    assert!(result.contains("analyzing performance data"));
    assert!(result.contains("Context: Q4 2023 metrics"));
    assert!(result.contains("Response Time: 125 (ms)"));
    assert!(result.contains("PRIORITY: This is a high-priority analysis"));
    assert!(result.contains("Output format: detailed JSON report"));
}

#[test]
fn test_default_registry() {
    let registry = create_default_registry().unwrap();
    
    // Verify default templates exist
    assert!(registry.get("system_prompt").is_ok());
    assert!(registry.get("user_message").is_ok());
    assert!(registry.get("json_response").is_ok());
    assert!(registry.get("error_response").is_ok());
    
    // Test error context templates
    let error_templates = registry.find_by_context("error").unwrap();
    assert_eq!(error_templates.len(), 1);
}

#[test]
fn test_quick_render_function() {
    let mut vars = HashMap::new();
    vars.insert("greeting".to_string(), json!("Welcome"));
    vars.insert("name".to_string(), json!("User"));
    vars.insert("time".to_string(), json!("morning"));
    
    let result = render_template(
        "{{greeting}}, {{name}}! Good {{time}}.",
        &vars
    ).unwrap();
    
    assert_eq!(result, "Welcome, User! Good morning.");
}

#[test]
fn test_template_with_nested_objects() {
    let engine = TemplateEngine::new();
    let template = Template::new(
        "nested",
        r#"User: {{user.name}}
Email: {{user.email}}
Role: {{user.role.name}} (Level {{user.role.level}})"#
    ).unwrap();
    
    let mut vars = TemplateVariables::new();
    vars.insert("user", json!({
        "name": "John Doe",
        "email": "john@example.com",
        "role": {
            "name": "Admin",
            "level": 5
        }
    }));
    
    let result = engine.render(&template, &vars).unwrap();
    assert!(result.contains("User: John Doe"));
    assert!(result.contains("Email: john@example.com"));
    assert!(result.contains("Role: Admin (Level 5)"));
}

#[test]
fn test_template_error_handling() {
    let mut manager = TemplateManager::new().unwrap();
    
    // Try to render non-existent template
    let vars = HashMap::new();
    let result = manager.render("non_existent", &vars);
    assert!(result.is_err());
    
    // Register template with strict variable requirements
    let strict_template = Template::new("strict", "Name: {{name}}, Age: {{age}}")
        .unwrap()
        .with_variable("name", VariableType::String)
        .with_variable("age", VariableType::Number);
    
    manager.register(strict_template).unwrap();
    
    // Try to render with wrong variable types
    let mut bad_vars = HashMap::new();
    bad_vars.insert("name".to_string(), json!(123)); // Wrong type
    bad_vars.insert("age".to_string(), json!("thirty")); // Wrong type
    
    // This should work in non-strict mode
    let result = manager.render("strict", &bad_vars);
    assert!(result.is_ok()); // Default engine is not strict
}

#[test]
fn test_template_composition() {
    let mut manager = TemplateManager::new().unwrap();
    
    // Register component templates
    let header = Template::new("header", "=== {{title}} ===\n").unwrap();
    let footer = Template::new("footer", "\n--- End of {{section}} ---").unwrap();
    
    manager.register(header).unwrap();
    manager.register(footer).unwrap();
    
    // Main template that uses components
    let main = Template::new(
        "report",
        r#"{{> header title=report_title}}
{{content}}
{{> footer section=report_section}}"#
    ).unwrap()
    .with_include("header")
    .with_include("footer");
    
    manager.register(main).unwrap();
    
    // Render with all required variables
    let mut vars = HashMap::new();
    vars.insert("report_title".to_string(), json!("Monthly Report"));
    vars.insert("content".to_string(), json!("This is the report content."));
    vars.insert("report_section".to_string(), json!("Finance"));
    
    let result = manager.render("report", &vars).unwrap();
    assert!(result.contains("=== Monthly Report ==="));
    assert!(result.contains("This is the report content."));
    assert!(result.contains("--- End of Finance ---"));
}