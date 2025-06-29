//! # Template Engine Implementation
//!
//! This module provides the core template rendering engine with support for
//! variable interpolation, conditionals, loops, and template composition.

use super::{
    Template, TemplateId, TemplateVariables, TemplateError, OutputFormat,
    CompiledTemplate, TemplateMetrics, TemplateRegistry,
};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use handlebars::{Handlebars, Helper, Context, Output, HelperResult, RenderErrorReason};
use handlebars::RenderContext as HandlebarsRenderContext;

/// Template engine configuration
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Enable template caching
    pub enable_cache: bool,
    /// Maximum cache size
    pub max_cache_size: usize,
    /// Enable strict mode (fail on missing variables)
    pub strict_mode: bool,
    /// Enable HTML escaping
    pub escape_html: bool,
    /// Maximum template depth for includes/inheritance
    pub max_depth: usize,
    /// Enable performance metrics
    pub enable_metrics: bool,
    /// Custom helpers
    pub custom_helpers: HashMap<String, String>,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            enable_cache: true,
            max_cache_size: 1000,
            strict_mode: false,
            escape_html: true,
            max_depth: 10,
            enable_metrics: true,
            custom_helpers: HashMap::new(),
        }
    }
}

/// Render context for template execution
#[derive(Debug, Clone)]
pub struct TemplateRenderContext {
    variables: HashMap<String, Value>,
    depth: usize,
    registry: Option<Arc<TemplateRegistry>>,
}

impl TemplateRenderContext {
    /// Create new render context
    pub fn new(variables: HashMap<String, Value>) -> Self {
        Self {
            variables,
            depth: 0,
            registry: None,
        }
    }
    
    /// Set template registry for includes/inheritance
    pub fn with_registry(mut self, registry: &TemplateRegistry) -> Self {
        self.registry = Some(Arc::new(registry.clone()));
        self
    }
    
    /// Increment depth for nested templates
    pub fn increment_depth(&mut self) -> Result<(), TemplateError> {
        self.depth += 1;
        if self.depth > 10 {
            return Err(TemplateError::RenderError {
                message: "Maximum template depth exceeded".to_string(),
            });
        }
        Ok(())
    }
}

/// Main template engine
#[derive(Debug)]
pub struct TemplateEngine {
    config: EngineConfig,
    handlebars: Arc<RwLock<Handlebars<'static>>>,
    cache: Arc<RwLock<HashMap<TemplateId, CompiledTemplate>>>,
    metrics: Option<Arc<TemplateMetrics>>,
}

impl TemplateEngine {
    /// Create new engine with default config
    pub fn new() -> Self {
        Self::with_config(EngineConfig::default())
    }
    
    /// Create new engine with custom config
    pub fn with_config(config: EngineConfig) -> Self {
        let mut handlebars = Handlebars::new();
        
        // Configure handlebars
        handlebars.set_strict_mode(config.strict_mode);
        if !config.escape_html {
            handlebars.register_escape_fn(handlebars::no_escape);
        }
        
        // Register built-in helpers
        Self::register_builtin_helpers(&mut handlebars);
        
        let metrics = if config.enable_metrics {
            Some(Arc::new(TemplateMetrics::new()))
        } else {
            None
        };
        
        Self {
            config,
            handlebars: Arc::new(RwLock::new(handlebars)),
            cache: Arc::new(RwLock::new(HashMap::new())),
            metrics,
        }
    }
    
    /// Enable metrics collection
    pub fn with_metrics(mut self) -> Self {
        self.metrics = Some(Arc::new(TemplateMetrics::new()));
        self
    }
    
    /// Register built-in template helpers
    fn register_builtin_helpers(handlebars: &mut Handlebars) {
        // JSON helper
        handlebars.register_helper("json", Box::new(json_helper));
        
        // String manipulation helpers
        handlebars.register_helper("uppercase", Box::new(uppercase_helper));
        handlebars.register_helper("lowercase", Box::new(lowercase_helper));
        handlebars.register_helper("capitalize", Box::new(capitalize_helper));
        
        // Comparison helpers
        handlebars.register_helper("eq", Box::new(eq_helper));
        handlebars.register_helper("ne", Box::new(ne_helper));
        handlebars.register_helper("gt", Box::new(gt_helper));
        handlebars.register_helper("lt", Box::new(lt_helper));
        
        // Array/Object helpers
        handlebars.register_helper("len", Box::new(len_helper));
        handlebars.register_helper("contains", Box::new(contains_helper));
        
        // Date/Time helper
        handlebars.register_helper("now", Box::new(now_helper));
        handlebars.register_helper("format_date", Box::new(format_date_helper));
    }
    
    /// Render a template with variables
    pub fn render(
        &self,
        template: &Template,
        variables: &TemplateVariables,
    ) -> Result<String, TemplateError> {
        let context = TemplateRenderContext::new(variables.inner().clone());
        self.render_with_context(template, context)
    }
    
    /// Render a template with full context
    pub fn render_with_context(
        &self,
        template: &Template,
        mut context: TemplateRenderContext,
    ) -> Result<String, TemplateError> {
        let start = std::time::Instant::now();
        
        // Check cache if enabled
        if self.config.enable_cache {
            if let Ok(cache) = self.cache.read() {
                if cache.contains_key(&template.id) {
                    if let Some(metrics) = &self.metrics {
                        metrics.record_cache_hit();
                    }
                }
            }
        }
        
        // Compile template if not cached
        let compiled = self.compile_template(template)?;
        
        // Validate variables
        self.validate_variables(template, &context.variables)?;
        
        // Render with handlebars
        let result = self.render_compiled(&compiled, &mut context)?;
        
        // Post-process based on output format
        let formatted = self.format_output(&result, template.output_format)?;
        
        // Record metrics
        if let Some(metrics) = &self.metrics {
            metrics.record_render(&template.id.0, start.elapsed());
        }
        
        Ok(formatted)
    }
    
    /// Compile a template
    fn compile_template(&self, template: &Template) -> Result<CompiledTemplate, TemplateError> {
        // Check cache first
        if self.config.enable_cache {
            if let Ok(cache) = self.cache.read() {
                if let Some(compiled) = cache.get(&template.id) {
                    return Ok(compiled.clone());
                }
            }
        }
        
        // Register template in handlebars
        let mut handlebars = self.handlebars.write()
            .map_err(|_| TemplateError::RenderError {
                message: "Failed to acquire handlebars lock".to_string(),
            })?;
            
        handlebars.register_template_string(&template.id.0, &template.content)
            .map_err(|e| TemplateError::ParseError {
                message: e.to_string(),
            })?;
        
        // Create compiled template
        let compiled = CompiledTemplate {
            id: template.id.clone(),
            ast: Arc::new(super::parser::TemplateAst::Text(template.content.clone())),
            variables: template.variables.clone(),
            output_format: template.output_format,
        };
        
        // Cache if enabled
        if self.config.enable_cache {
            if let Ok(mut cache) = self.cache.write() {
                if cache.len() >= self.config.max_cache_size {
                    // Simple eviction - remove first item
                    if let Some(first_key) = cache.keys().next().cloned() {
                        cache.remove(&first_key);
                    }
                }
                cache.insert(template.id.clone(), compiled.clone());
            }
        }
        
        if let Some(metrics) = &self.metrics {
            metrics.record_cache_miss();
        }
        
        Ok(compiled)
    }
    
    /// Render a compiled template
    fn render_compiled(
        &self,
        compiled: &CompiledTemplate,
        context: &mut TemplateRenderContext,
    ) -> Result<String, TemplateError> {
        let handlebars = self.handlebars.read()
            .map_err(|_| TemplateError::RenderError {
                message: "Failed to acquire handlebars lock".to_string(),
            })?;
            
        handlebars.render(&compiled.id.0, &context.variables)
            .map_err(|e| TemplateError::RenderError {
                message: e.to_string(),
            })
    }
    
    /// Validate variables against template requirements
    fn validate_variables(
        &self,
        template: &Template,
        variables: &HashMap<String, Value>,
    ) -> Result<(), TemplateError> {
        for (name, var_type) in &template.variables {
            if let Some(value) = variables.get(name) {
                if !var_type.matches_value(value) {
                    return Err(TemplateError::TypeMismatch {
                        name: name.clone(),
                        expected: format!("{:?}", var_type),
                        actual: format!("{:?}", value),
                    });
                }
            } else if self.config.strict_mode {
                return Err(TemplateError::VariableNotFound {
                    name: name.clone(),
                });
            }
        }
        Ok(())
    }
    
    /// Format output based on specified format
    fn format_output(&self, content: &str, format: OutputFormat) -> Result<String, TemplateError> {
        match format {
            OutputFormat::Text => Ok(content.to_string()),
            OutputFormat::Json => {
                // Validate JSON
                serde_json::from_str::<Value>(content)
                    .map_err(|e| TemplateError::RenderError {
                        message: format!("Invalid JSON output: {}", e),
                    })?;
                Ok(content.to_string())
            }
            OutputFormat::Yaml => {
                // For now, just return as-is
                // In production, would validate YAML
                Ok(content.to_string())
            }
            OutputFormat::Markdown => {
                // Could apply markdown-specific formatting
                Ok(content.to_string())
            }
            OutputFormat::Html => {
                // Could apply HTML-specific formatting
                Ok(content.to_string())
            }
        }
    }
    
    /// Clear template cache
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }
    
    /// Get metrics
    pub fn metrics(&self) -> Option<&TemplateMetrics> {
        self.metrics.as_ref().map(|m| m.as_ref())
    }
}

// Helper functions for handlebars

fn json_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut HandlebarsRenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h.param(0).ok_or_else(|| {
        RenderErrorReason::ParamNotFoundForIndex("json", 0)
    })?;
    
    let json_str = serde_json::to_string_pretty(param.value())
        .map_err(|e| RenderErrorReason::NestedError(Box::new(e)))?;
    
    out.write(&json_str)?;
    Ok(())
}

fn uppercase_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut HandlebarsRenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h.param(0).ok_or_else(|| {
        RenderErrorReason::ParamNotFoundForIndex("uppercase", 0)
    })?;
    
    if let Some(s) = param.value().as_str() {
        out.write(&s.to_uppercase())?;
    }
    Ok(())
}

fn lowercase_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut HandlebarsRenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h.param(0).ok_or_else(|| {
        RenderErrorReason::ParamNotFoundForIndex("lowercase", 0)
    })?;
    
    if let Some(s) = param.value().as_str() {
        out.write(&s.to_lowercase())?;
    }
    Ok(())
}

fn capitalize_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut HandlebarsRenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h.param(0).ok_or_else(|| {
        RenderErrorReason::ParamNotFoundForIndex("capitalize", 0)
    })?;
    
    if let Some(s) = param.value().as_str() {
        let mut chars = s.chars();
        if let Some(first) = chars.next() {
            out.write(&first.to_uppercase().to_string())?;
            out.write(chars.as_str())?;
        }
    }
    Ok(())
}

fn eq_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut HandlebarsRenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param1 = h.param(0).ok_or_else(|| {
        RenderErrorReason::ParamNotFoundForIndex("eq", 0)
    })?;
    let param2 = h.param(1).ok_or_else(|| {
        RenderErrorReason::ParamNotFoundForIndex("eq", 1)
    })?;
    
    let result = param1.value() == param2.value();
    out.write(&result.to_string())?;
    Ok(())
}

fn ne_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut HandlebarsRenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param1 = h.param(0).ok_or_else(|| {
        RenderErrorReason::ParamNotFoundForIndex("ne", 0)
    })?;
    let param2 = h.param(1).ok_or_else(|| {
        RenderErrorReason::ParamNotFoundForIndex("ne", 1)
    })?;
    
    let result = param1.value() != param2.value();
    out.write(&result.to_string())?;
    Ok(())
}

fn gt_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut HandlebarsRenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param1 = h.param(0).ok_or_else(|| {
        RenderErrorReason::ParamNotFoundForIndex("gt", 0)
    })?;
    let param2 = h.param(1).ok_or_else(|| {
        RenderErrorReason::ParamNotFoundForIndex("gt", 1)
    })?;
    
    let result = if let (Some(n1), Some(n2)) = (param1.value().as_f64(), param2.value().as_f64()) {
        n1 > n2
    } else {
        false
    };
    
    out.write(&result.to_string())?;
    Ok(())
}

fn lt_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut HandlebarsRenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param1 = h.param(0).ok_or_else(|| {
        RenderErrorReason::ParamNotFoundForIndex("lt", 0)
    })?;
    let param2 = h.param(1).ok_or_else(|| {
        RenderErrorReason::ParamNotFoundForIndex("lt", 1)
    })?;
    
    let result = if let (Some(n1), Some(n2)) = (param1.value().as_f64(), param2.value().as_f64()) {
        n1 < n2
    } else {
        false
    };
    
    out.write(&result.to_string())?;
    Ok(())
}

fn len_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut HandlebarsRenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h.param(0).ok_or_else(|| {
        RenderErrorReason::ParamNotFoundForIndex("len", 0)
    })?;
    
    let length = if let Some(arr) = param.value().as_array() {
        arr.len()
    } else if let Some(obj) = param.value().as_object() {
        obj.len()
    } else if let Some(s) = param.value().as_str() {
        s.len()
    } else {
        0
    };
    
    out.write(&length.to_string())?;
    Ok(())
}

fn contains_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut HandlebarsRenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param1 = h.param(0).ok_or_else(|| {
        RenderErrorReason::ParamNotFoundForIndex("contains", 0)
    })?;
    let param2 = h.param(1).ok_or_else(|| {
        RenderErrorReason::ParamNotFoundForIndex("contains", 1)
    })?;
    
    let result = if let Some(arr) = param1.value().as_array() {
        arr.contains(param2.value())
    } else if let Some(s) = param1.value().as_str() {
        if let Some(needle) = param2.value().as_str() {
            s.contains(needle)
        } else {
            false
        }
    } else {
        false
    };
    
    out.write(&result.to_string())?;
    Ok(())
}

fn now_helper(
    _: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut HandlebarsRenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let now = chrono::Utc::now();
    out.write(&now.to_rfc3339())?;
    Ok(())
}

fn format_date_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut HandlebarsRenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h.param(0).ok_or_else(|| {
        RenderErrorReason::ParamNotFoundForIndex("format_date", 0)
    })?;
    
    let format = h.param(1)
        .and_then(|p| p.value().as_str())
        .unwrap_or("%Y-%m-%d %H:%M:%S");
    
    if let Some(date_str) = param.value().as_str() {
        if let Ok(date) = chrono::DateTime::parse_from_rfc3339(date_str) {
            out.write(&date.format(format).to_string())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_basic_rendering() {
        let engine = TemplateEngine::new();
        let template = Template::new("test", "Hello {{name}}!").unwrap();
        
        let mut vars = TemplateVariables::new();
        vars.insert("name", json!("World"));
        
        let result = engine.render(&template, &vars).unwrap();
        assert_eq!(result, "Hello World!");
    }
    
    #[test]
    fn test_helper_functions() {
        let engine = TemplateEngine::new();
        
        // Test uppercase helper
        let template = Template::new("upper", "{{uppercase name}}").unwrap();
        let mut vars = TemplateVariables::new();
        vars.insert("name", json!("hello"));
        let result = engine.render(&template, &vars).unwrap();
        assert_eq!(result, "HELLO");
        
        // Test json helper
        let template = Template::new("json", "{{json data}}").unwrap();
        vars.insert("data", json!({"key": "value", "num": 42}));
        let result = engine.render(&template, &vars).unwrap();
        assert!(result.contains("\"key\": \"value\""));
        assert!(result.contains("\"num\": 42"));
    }
    
    #[test]
    fn test_conditionals() {
        let engine = TemplateEngine::new();
        let template = Template::new(
            "cond",
            "{{#if active}}Active{{else}}Inactive{{/if}}"
        ).unwrap();
        
        let mut vars = TemplateVariables::new();
        vars.insert("active", json!(true));
        let result = engine.render(&template, &vars).unwrap();
        assert_eq!(result, "Active");
        
        vars.insert("active", json!(false));
        let result = engine.render(&template, &vars).unwrap();
        assert_eq!(result, "Inactive");
    }
    
    #[test]
    fn test_loops() {
        let engine = TemplateEngine::new();
        let template = Template::new(
            "loop",
            "{{#each items}}{{this}}, {{/each}}"
        ).unwrap();
        
        let mut vars = TemplateVariables::new();
        vars.insert("items", json!(["apple", "banana", "orange"]));
        
        let result = engine.render(&template, &vars).unwrap();
        assert_eq!(result, "apple, banana, orange, ");
    }
    
    #[test]
    fn test_strict_mode() {
        let config = EngineConfig {
            strict_mode: true,
            ..Default::default()
        };
        let engine = TemplateEngine::with_config(config);
        
        let template = Template::new("strict", "Hello {{name}}!")
            .unwrap()
            .with_variable("name", super::super::VariableType::String);
        
        let vars = TemplateVariables::new();
        let result = engine.render(&template, &vars);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TemplateError::VariableNotFound { .. }));
    }
}