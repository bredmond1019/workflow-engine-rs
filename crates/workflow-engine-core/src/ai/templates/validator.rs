//! # Template Validator
//!
//! This module provides validation functionality for templates,
//! ensuring security, correctness, and best practices.

use super::{Template, TemplateError};
use super::parser::TemplateParser;
use regex::Regex;
use std::collections::HashSet;

/// Validation error types
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid template syntax: {message}")]
    SyntaxError { message: String },
    
    #[error("Security violation: {message}")]
    SecurityError { message: String },
    
    #[error("Invalid variable name: {name}")]
    InvalidVariable { name: String },
    
    #[error("Missing required variable: {name}")]
    MissingVariable { name: String },
    
    #[error("Resource limit exceeded: {message}")]
    ResourceLimit { message: String },
    
    #[error("Invalid output format: {message}")]
    InvalidFormat { message: String },
    
    #[error("Circular reference detected: {reference}")]
    CircularReference { reference: String },
}

impl From<ValidationError> for TemplateError {
    fn from(error: ValidationError) -> Self {
        TemplateError::ValidationError {
            message: error.to_string(),
        }
    }
}

/// Validation rules that can be applied
pub enum ValidationRule {
    /// Require all variables to be defined
    RequireVariableDefinitions,
    /// Limit template depth for includes/inheritance
    MaxDepth(usize),
    /// Limit template size
    MaxSize(usize),
    /// Disallow certain functions/helpers
    DisallowedFunctions(HashSet<String>),
    /// Require specific output format
    RequireOutputFormat(super::OutputFormat),
}

/// Template validator
pub struct TemplateValidator {
    rules: Vec<ValidationRule>,
    parser: TemplateParser,
    /// Patterns that might indicate security issues
    dangerous_patterns: Vec<(Regex, String)>,
}

impl TemplateValidator {
    /// Create a new validator with default rules
    pub fn new() -> Self {
        let mut validator = Self {
            rules: Vec::new(),
            parser: TemplateParser::new(),
            dangerous_patterns: Self::default_dangerous_patterns(),
        };
        
        // Add default rules
        validator.add_rule(ValidationRule::MaxDepth(10));
        validator.add_rule(ValidationRule::MaxSize(1024 * 1024)); // 1MB
        
        validator
    }
    
    /// Create a strict validator
    pub fn strict() -> Self {
        let mut validator = Self::new();
        
        validator.add_rule(ValidationRule::RequireVariableDefinitions);
        validator.add_rule(ValidationRule::MaxDepth(5));
        validator.add_rule(ValidationRule::MaxSize(100 * 1024)); // 100KB
        
        // Disallow potentially dangerous helpers
        let mut disallowed = HashSet::new();
        disallowed.insert("exec".to_string());
        disallowed.insert("system".to_string());
        disallowed.insert("eval".to_string());
        validator.add_rule(ValidationRule::DisallowedFunctions(disallowed));
        
        validator
    }
    
    /// Add a validation rule
    pub fn add_rule(&mut self, rule: ValidationRule) {
        self.rules.push(rule);
    }
    
    /// Validate a template
    pub fn validate(&self, template: &Template) -> Result<(), ValidationError> {
        // Check syntax
        self.validate_syntax(template)?;
        
        // Check security
        self.validate_security(template)?;
        
        // Check variables
        self.validate_variables(template)?;
        
        // Apply custom rules
        for rule in &self.rules {
            self.apply_rule(template, rule)?;
        }
        
        // Validate output format
        self.validate_output_format(template)?;
        
        Ok(())
    }
    
    /// Validate template syntax
    fn validate_syntax(&self, template: &Template) -> Result<(), ValidationError> {
        // Try to parse the template
        match self.parser.parse(&template.content) {
            Ok(_) => Ok(()),
            Err(e) => Err(ValidationError::SyntaxError {
                message: e.to_string(),
            }),
        }
    }
    
    /// Validate security aspects
    fn validate_security(&self, template: &Template) -> Result<(), ValidationError> {
        // Check for dangerous patterns
        for (pattern, description) in &self.dangerous_patterns {
            if pattern.is_match(&template.content) {
                return Err(ValidationError::SecurityError {
                    message: format!("Dangerous pattern detected: {}", description),
                });
            }
        }
        
        // Check variable names for injection attempts
        for var_name in template.variables.keys() {
            if !self.is_safe_variable_name(var_name) {
                return Err(ValidationError::SecurityError {
                    message: format!("Unsafe variable name: {}", var_name),
                });
            }
        }
        
        Ok(())
    }
    
    /// Validate variables
    fn validate_variables(&self, template: &Template) -> Result<(), ValidationError> {
        // Extract variables from content
        let extracted_vars = self.parser.extract_variables(&template.content)
            .map_err(|e| ValidationError::SyntaxError {
                message: e.to_string(),
            })?;
        
        // Check that all extracted variables are defined (if strict mode)
        for rule in &self.rules {
            if matches!(rule, ValidationRule::RequireVariableDefinitions) {
                for (var_name, _) in &extracted_vars {
                    if !template.variables.contains_key(var_name) {
                        return Err(ValidationError::MissingVariable {
                            name: var_name.clone(),
                        });
                    }
                }
            }
        }
        
        // Validate variable names
        for var_name in template.variables.keys() {
            if !self.is_valid_variable_name(var_name) {
                return Err(ValidationError::InvalidVariable {
                    name: var_name.clone(),
                });
            }
        }
        
        Ok(())
    }
    
    /// Apply a validation rule
    fn apply_rule(&self, template: &Template, rule: &ValidationRule) -> Result<(), ValidationError> {
        match rule {
            ValidationRule::RequireVariableDefinitions => {
                // Already handled in validate_variables
                Ok(())
            }
            ValidationRule::MaxDepth(max) => {
                let depth = self.calculate_depth(template);
                if depth > *max {
                    Err(ValidationError::ResourceLimit {
                        message: format!("Template depth {} exceeds maximum {}", depth, max),
                    })
                } else {
                    Ok(())
                }
            }
            ValidationRule::MaxSize(max) => {
                let size = template.content.len();
                if size > *max {
                    Err(ValidationError::ResourceLimit {
                        message: format!("Template size {} exceeds maximum {}", size, max),
                    })
                } else {
                    Ok(())
                }
            }
            ValidationRule::DisallowedFunctions(functions) => {
                // Parse template and check for disallowed functions
                if let Ok(ast) = self.parser.parse(&template.content) {
                    if self.contains_disallowed_function(&ast, functions) {
                        Err(ValidationError::SecurityError {
                            message: "Template contains disallowed function".to_string(),
                        })
                    } else {
                        Ok(())
                    }
                } else {
                    Ok(())
                }
            }
            ValidationRule::RequireOutputFormat(format) => {
                if template.output_format != *format {
                    Err(ValidationError::InvalidFormat {
                        message: format!(
                            "Expected output format {:?}, got {:?}",
                            format, template.output_format
                        ),
                    })
                } else {
                    Ok(())
                }
            }
            // Custom validation was removed for simplicity
        }
    }
    
    /// Validate output format
    fn validate_output_format(&self, template: &Template) -> Result<(), ValidationError> {
        use super::OutputFormat;
        
        match template.output_format {
            OutputFormat::Json => {
                // For JSON output, try to parse the template as JSON
                // This is a simple check - in production would be more sophisticated
                let test_content = template.content
                    .replace("{{", "\"")
                    .replace("}}", "\"");
                
                if serde_json::from_str::<serde_json::Value>(&test_content).is_err() {
                    // Don't fail on this - template might be valid with variables
                    // Just log a warning in production
                }
            }
            OutputFormat::Yaml => {
                // Similar validation for YAML
            }
            OutputFormat::Html => {
                // Check for basic HTML structure if needed
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Calculate template depth (for circular reference detection)
    fn calculate_depth(&self, template: &Template) -> usize {
        // Count includes and parent
        let include_depth = template.includes.len();
        let parent_depth = if template.parent.is_some() { 1 } else { 0 };
        
        include_depth + parent_depth
    }
    
    /// Check if AST contains disallowed functions
    fn contains_disallowed_function(
        &self,
        ast: &super::parser::TemplateAst,
        disallowed: &HashSet<String>,
    ) -> bool {
        use super::parser::TemplateAst;
        
        match ast {
            TemplateAst::Helper { name, .. } => disallowed.contains(name),
            TemplateAst::Block(statements) => {
                statements.iter().any(|s| self.contains_disallowed_function(s, disallowed))
            }
            TemplateAst::Conditional { condition, then_branch, else_branch } => {
                self.contains_disallowed_function(condition, disallowed) ||
                self.contains_disallowed_function(then_branch, disallowed) ||
                else_branch.as_ref()
                    .map(|b| self.contains_disallowed_function(b, disallowed))
                    .unwrap_or(false)
            }
            TemplateAst::Loop { body, .. } => {
                self.contains_disallowed_function(body, disallowed)
            }
            _ => false,
        }
    }
    
    /// Check if variable name is valid
    fn is_valid_variable_name(&self, name: &str) -> bool {
        // Must start with letter or underscore
        // Can contain letters, numbers, underscores
        let pattern = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
        pattern.is_match(name)
    }
    
    /// Check if variable name is safe (no injection attempts)
    fn is_safe_variable_name(&self, name: &str) -> bool {
        // Disallow special characters that might be used for injection
        !name.contains("__") &&
        !name.contains("prototype") &&
        !name.contains("constructor") &&
        !name.contains("$") &&
        self.is_valid_variable_name(name)
    }
    
    /// Get default dangerous patterns
    fn default_dangerous_patterns() -> Vec<(Regex, String)> {
        vec![
            (
                Regex::new(r"<script[^>]*>").unwrap(),
                "Script tags not allowed".to_string()
            ),
            (
                Regex::new(r"javascript:").unwrap(),
                "JavaScript URLs not allowed".to_string()
            ),
            (
                Regex::new(r"on\w+\s*=").unwrap(),
                "Event handlers not allowed".to_string()
            ),
            (
                Regex::new(r"__proto__").unwrap(),
                "Prototype pollution attempt".to_string()
            ),
            (
                Regex::new(r"constructor\s*\[").unwrap(),
                "Constructor access not allowed".to_string()
            ),
        ]
    }
}

impl Default for TemplateValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Validate a template with specific rules
pub fn validate_with_rules(
    template: &Template,
    rules: Vec<ValidationRule>,
) -> Result<(), ValidationError> {
    let mut validator = TemplateValidator::new();
    for rule in rules {
        validator.add_rule(rule);
    }
    validator.validate(template)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_validation() {
        let validator = TemplateValidator::new();
        
        let template = Template::new("test", "Hello {{name}}!").unwrap();
        assert!(validator.validate(&template).is_ok());
    }
    
    #[test]
    fn test_syntax_validation() {
        let validator = TemplateValidator::new();
        
        let template = Template::new("test", "Hello {{name").unwrap(); // Missing closing }}
        let result = validator.validate(&template);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::SyntaxError { .. }));
    }
    
    #[test]
    fn test_security_validation() {
        let validator = TemplateValidator::new();
        
        // Test script tag
        let template = Template::new("test", "Hello <script>alert('xss')</script>").unwrap();
        let result = validator.validate(&template);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::SecurityError { .. }));
        
        // Test dangerous variable name
        let template = Template::new("test", "Hello {{name}}!")
            .unwrap()
            .with_variable("__proto__", VariableType::String);
        let result = validator.validate(&template);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_strict_validation() {
        let validator = TemplateValidator::strict();
        
        // Template with undefined variable
        let template = Template::new("test", "Hello {{name}} {{undefined}}!").unwrap();
        let result = validator.validate(&template);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::MissingVariable { .. }));
    }
    
    #[test]
    fn test_size_limit() {
        let mut validator = TemplateValidator::new();
        validator.add_rule(ValidationRule::MaxSize(100));
        
        let large_content = "x".repeat(101);
        let template = Template::new("test", large_content).unwrap();
        let result = validator.validate(&template);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::ResourceLimit { .. }));
    }
    
    #[test]
    fn test_validation_rules() {
        let mut validator = TemplateValidator::new();
        
        // Test size limit rule
        validator.add_rule(ValidationRule::MaxSize(50));
        
        let small_template = Template::new("test", "Hello!").unwrap();
        assert!(validator.validate(&small_template).is_ok());
        
        let large_template = Template::new("test", "x".repeat(100)).unwrap();
        assert!(validator.validate(&large_template).is_err());
    }
}