//! # Simple Template System Demo
//!
//! This example demonstrates basic template functionality.

use backend::core::ai::templates::{
    Template, TemplateEngine, TemplateVariables, OutputFormat, 
    VariableType, render_template,
};
use serde_json::json;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("AI Template System Demo");
    println!("========================");
    
    // Demo 1: Basic template rendering
    println!("\n1. Basic Template Rendering");
    println!("----------------------------");
    
    let engine = TemplateEngine::new();
    let template = Template::new("greeting", "Hello {{name}}, welcome to {{system}}!")
        .unwrap()
        .with_variable("name", VariableType::String)
        .with_variable("system", VariableType::String);
    
    let mut vars = TemplateVariables::new();
    vars.insert("name", json!("Alice"));
    vars.insert("system", json!("AI Workflow System"));
    
    let result = engine.render(&template, &vars)?;
    println!("Result: {}", result);
    
    // Demo 2: Quick render function
    println!("\n2. Quick Render Function");
    println!("------------------------");
    
    let mut quick_vars = HashMap::new();
    quick_vars.insert("item".to_string(), json!("template"));
    quick_vars.insert("status".to_string(), json!("working"));
    
    let quick_result = render_template(
        "The {{item}} is {{status}} perfectly!",
        &quick_vars
    )?;
    println!("Result: {}", quick_result);
    
    // Demo 3: Conditional templates
    println!("\n3. Conditional Logic");
    println!("--------------------");
    
    let conditional_template = Template::new(
        "conditional",
        r#"{{#if premium}}Welcome, premium user {{name}}!{{else}}Hello {{name}}.{{/if}}"#
    )?;
    
    // Test premium user
    let mut premium_vars = TemplateVariables::new();
    premium_vars.insert("name", json!("Bob"));
    premium_vars.insert("premium", json!(true));
    
    let premium_result = engine.render(&conditional_template, &premium_vars)?;
    println!("Premium user: {}", premium_result);
    
    // Test regular user
    premium_vars.insert("premium", json!(false));
    let regular_result = engine.render(&conditional_template, &premium_vars)?;
    println!("Regular user: {}", regular_result);
    
    // Demo 4: Loop templates
    println!("\n4. Loop Templates");
    println!("-----------------");
    
    let loop_template = Template::new(
        "task_list",
        r#"Tasks:
{{#each tasks}}
- {{this.name}}: {{this.status}}
{{/each}}"#
    )?;
    
    let mut loop_vars = TemplateVariables::new();
    loop_vars.insert("tasks", json!([
        {"name": "Setup Template System", "status": "completed"},
        {"name": "Write Documentation", "status": "in_progress"},
        {"name": "Add More Features", "status": "pending"}
    ]));
    
    let loop_result = engine.render(&loop_template, &loop_vars)?;
    println!("{}", loop_result);
    
    // Demo 5: JSON output format
    println!("\n5. JSON Output Format");
    println!("---------------------");
    
    let json_template = Template::new(
        "api_response",
        r#"{
    "status": "{{status}}",
    "user": "{{user}}",
    "data": {{json data}},
    "timestamp": "{{timestamp}}"
}"#
    )?
    .with_output_format(OutputFormat::Json);
    
    let mut json_vars = TemplateVariables::new();
    json_vars.insert("status", json!("success"));
    json_vars.insert("user", json!("alice"));
    json_vars.insert("data", json!({"count": 42, "items": ["a", "b", "c"]}));
    json_vars.insert("timestamp", json!("2024-01-01T00:00:00Z"));
    
    let json_result = engine.render(&json_template, &json_vars)?;
    println!("JSON Response:\n{}", json_result);
    
    // Verify it's valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&json_result)?;
    println!("Parsed successfully! User: {}", parsed["user"]);
    
    // Demo 6: Helper functions
    println!("\n6. Helper Functions");
    println!("-------------------");
    
    let helper_template = Template::new(
        "helpers",
        r#"Original: {{name}}
Uppercase: {{uppercase name}}
Lowercase: {{lowercase name}}
Length: {{len name}} characters"#
    )?;
    
    let mut helper_vars = TemplateVariables::new();
    helper_vars.insert("name", json!("Template System"));
    
    let helper_result = engine.render(&helper_template, &helper_vars)?;
    println!("{}", helper_result);
    
    println!("\n✅ Template system demo completed successfully!");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_functionality() {
        let engine = TemplateEngine::new();
        let template = Template::new("test", "Hello {{name}}!").unwrap();
        
        let mut vars = TemplateVariables::new();
        vars.insert("name", json!("World"));
        
        let result = engine.render(&template, &vars).unwrap();
        assert_eq!(result, "Hello World!");
    }
    
    #[test]
    fn test_quick_render() {
        let mut vars = HashMap::new();
        vars.insert("greeting".to_string(), json!("Hi"));
        vars.insert("name".to_string(), json!("User"));
        
        let result = render_template("{{greeting}} {{name}}!", &vars).unwrap();
        assert_eq!(result, "Hi User!");
    }
}