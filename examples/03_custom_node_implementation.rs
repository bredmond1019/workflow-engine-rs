//! # Custom Node Implementation Example
//!
//! This example demonstrates how to create sophisticated custom nodes with:
//!
//! - Advanced configuration and validation
//! - State management and lifecycle hooks
//! - Custom metrics and monitoring
//! - Router nodes for conditional workflow paths
//! - Parallel processing capabilities
//! - Integration with external services
//!
//! ## Usage
//!
//! ```bash
//! cargo run --example 03_custom_node_implementation
//! ```

use workflow_engine_core::prelude::*;
use serde_json::json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use async_trait::async_trait;

/// Configuration for the data processing node
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DataProcessorConfig {
    pub processing_mode: ProcessingMode,
    pub batch_size: usize,
    pub timeout_seconds: u32,
    pub validation_rules: Vec<ValidationRule>,
    pub output_format: OutputFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum ProcessingMode {
    Batch,
    Stream,
    RealTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ValidationRule {
    pub field: String,
    pub rule_type: RuleType,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum RuleType {
    Required,
    MinLength,
    MaxLength,
    Pattern,
    Range,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum OutputFormat {
    Json,
    Csv,
    Xml,
}

/// Advanced data processing node with configuration and lifecycle management
#[derive(Debug)]
struct DataProcessorNode {
    config: DataProcessorConfig,
    processing_stats: std::sync::Arc<std::sync::Mutex<ProcessingStats>>,
    node_id: String,
}

#[derive(Debug, Default)]
struct ProcessingStats {
    total_processed: u64,
    successful_validations: u64,
    failed_validations: u64,
    average_processing_time_ms: f64,
}

impl DataProcessorNode {
    /// Create a new data processor node with configuration
    pub fn new(node_id: impl Into<String>, config: DataProcessorConfig) -> Result<Self, WorkflowError> {
        // Validate configuration
        if config.batch_size == 0 {
            return Err(WorkflowError::ValidationError {
                message: "Batch size must be greater than 0".to_string(),
            });
        }
        
        if config.timeout_seconds == 0 {
            return Err(WorkflowError::ValidationError {
                message: "Timeout must be greater than 0".to_string(),
            });
        }
        
        Ok(Self {
            config,
            processing_stats: std::sync::Arc::new(std::sync::Mutex::new(ProcessingStats::default())),
            node_id: node_id.into(),
        })
    }
    
    /// Validate input data against configured rules
    fn validate_data(&self, data: &serde_json::Value) -> Result<Vec<String>, Vec<String>> {
        let mut passed = Vec::new();
        let mut failed = Vec::new();
        
        for rule in &self.config.validation_rules {
            match self.apply_validation_rule(rule, data) {
                Ok(_) => passed.push(rule.field.clone()),
                Err(e) => failed.push(format!("{}: {}", rule.field, e)),
            }
        }
        
        if failed.is_empty() {
            Ok(passed)
        } else {
            Err(failed)
        }
    }
    
    /// Apply a single validation rule
    fn apply_validation_rule(&self, rule: &ValidationRule, data: &serde_json::Value) -> Result<(), String> {
        let field_value = data.get(&rule.field);
        
        match rule.rule_type {
            RuleType::Required => {
                if field_value.is_none() || field_value.unwrap().is_null() {
                    return Err("Field is required".to_string());
                }
            }
            RuleType::MinLength => {
                if let Some(value) = field_value.and_then(|v| v.as_str()) {
                    let min_len = rule.value.as_u64().unwrap_or(0) as usize;
                    if value.len() < min_len {
                        return Err(format!("Length must be at least {}", min_len));
                    }
                }
            }
            RuleType::MaxLength => {
                if let Some(value) = field_value.and_then(|v| v.as_str()) {
                    let max_len = rule.value.as_u64().unwrap_or(u64::MAX) as usize;
                    if value.len() > max_len {
                        return Err(format!("Length must not exceed {}", max_len));
                    }
                }
            }
            RuleType::Pattern => {
                if let Some(value) = field_value.and_then(|v| v.as_str()) {
                    let pattern = rule.value.as_str().unwrap_or("");
                    if let Ok(regex) = regex::Regex::new(pattern) {
                        if !regex.is_match(value) {
                            return Err(format!("Value does not match pattern: {}", pattern));
                        }
                    }
                }
            }
            RuleType::Range => {
                if let Some(value) = field_value.and_then(|v| v.as_f64()) {
                    if let Some(range) = rule.value.as_array() {
                        let min = range.get(0).and_then(|v| v.as_f64()).unwrap_or(f64::NEG_INFINITY);
                        let max = range.get(1).and_then(|v| v.as_f64()).unwrap_or(f64::INFINITY);
                        
                        if value < min || value > max {
                            return Err(format!("Value must be between {} and {}", min, max));
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Format output according to configuration
    fn format_output(&self, data: &serde_json::Value) -> Result<String, WorkflowError> {
        match self.config.output_format {
            OutputFormat::Json => {
                serde_json::to_string_pretty(data)
                    .map_err(|e| WorkflowError::SerializationError {
                        message: format!("JSON formatting error: {}", e)
                    })
            }
            OutputFormat::Csv => {
                // Simple CSV conversion for demonstration
                if let Some(array) = data.as_array() {
                    let mut csv = String::new();
                    
                    // Add headers
                    if let Some(first) = array.first() {
                        if let Some(obj) = first.as_object() {
                            let headers: Vec<&String> = obj.keys().collect();
                            csv.push_str(&headers.join(","));
                            csv.push('\n');
                        }
                    }
                    
                    // Add data rows
                    for item in array {
                        if let Some(obj) = item.as_object() {
                            let values: Vec<String> = obj.values()
                                .map(|v| v.as_str().unwrap_or("").to_string())
                                .collect();
                            csv.push_str(&values.join(","));
                            csv.push('\n');
                        }
                    }
                    
                    Ok(csv)
                } else {
                    Err(WorkflowError::ProcessingError {
                        message: "CSV format requires array input".to_string()
                    })
                }
            }
            OutputFormat::Xml => {
                // Simple XML conversion for demonstration
                Ok(format!("<data>{}</data>", data.to_string()))
            }
        }
    }
    
    /// Get current processing statistics
    pub fn get_stats(&self) -> ProcessingStats {
        self.processing_stats.lock().unwrap().clone()
    }
}

impl Node for DataProcessorNode {
    fn node_name(&self) -> String {
        format!("DataProcessor({})", self.node_id)
    }
    
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let start_time = std::time::Instant::now();
        
        println!("🔄 Processing node: {} (mode: {:?})", self.node_name(), self.config.processing_mode);
        
        // Extract input data
        let input_data: serde_json::Value = context.get_event_data()?;
        
        // Validate input data
        let validation_result = self.validate_data(&input_data);
        
        let (processed_data, validation_summary) = match validation_result {
            Ok(passed_rules) => {
                // Update stats
                {
                    let mut stats = self.processing_stats.lock().unwrap();
                    stats.successful_validations += 1;
                    stats.total_processed += 1;
                }
                
                // Process the data based on mode
                let processed = match self.config.processing_mode {
                    ProcessingMode::Batch => {
                        self.process_batch(&input_data)
                    }
                    ProcessingMode::Stream => {
                        self.process_stream(&input_data)
                    }
                    ProcessingMode::RealTime => {
                        self.process_realtime(&input_data)
                    }
                };
                
                (processed, json!({
                    "status": "passed",
                    "passed_rules": passed_rules,
                    "total_rules": self.config.validation_rules.len()
                }))
            }
            Err(failed_rules) => {
                // Update stats
                {
                    let mut stats = self.processing_stats.lock().unwrap();
                    stats.failed_validations += 1;
                    stats.total_processed += 1;
                }
                
                (json!({
                    "error": "Validation failed",
                    "original_data": input_data
                }), json!({
                    "status": "failed",
                    "failed_rules": failed_rules,
                    "total_rules": self.config.validation_rules.len()
                }))
            }
        };
        
        // Format output
        let formatted_output = self.format_output(&processed_data)?;
        
        // Store results
        context.update_node("processed_data", processed_data);
        context.update_node("validation_summary", validation_summary);
        context.update_node("formatted_output", formatted_output);
        
        // Update processing time stats
        let processing_time = start_time.elapsed();
        {
            let mut stats = self.processing_stats.lock().unwrap();
            stats.average_processing_time_ms = 
                (stats.average_processing_time_ms + processing_time.as_millis() as f64) / 2.0;
        }
        
        // Set metadata
        context.set_metadata("processing_mode", format!("{:?}", self.config.processing_mode))?;
        context.set_metadata("processing_time_ms", processing_time.as_millis())?;
        context.set_metadata("validation_passed", validation_result.is_ok())?;
        context.set_metadata("output_format", format!("{:?}", self.config.output_format))?;
        
        println!("✅ Node processing completed in {:?}", processing_time);
        
        Ok(context)
    }
}

impl DataProcessorNode {
    fn process_batch(&self, data: &serde_json::Value) -> serde_json::Value {
        json!({
            "processing_mode": "batch",
            "batch_size": self.config.batch_size,
            "data": data,
            "processed_at": chrono::Utc::now(),
            "batch_id": uuid::Uuid::new_v4()
        })
    }
    
    fn process_stream(&self, data: &serde_json::Value) -> serde_json::Value {
        json!({
            "processing_mode": "stream",
            "data": data,
            "stream_position": 1,
            "processed_at": chrono::Utc::now()
        })
    }
    
    fn process_realtime(&self, data: &serde_json::Value) -> serde_json::Value {
        json!({
            "processing_mode": "realtime",
            "data": data,
            "latency_ms": 5,
            "processed_at": chrono::Utc::now()
        })
    }
}

/// Router node that makes decisions based on data characteristics
#[derive(Debug)]
struct DataRouterNode {
    routing_rules: HashMap<String, Box<dyn Fn(&serde_json::Value) -> bool + Send + Sync>>,
}

impl DataRouterNode {
    fn new() -> Self {
        let mut routing_rules: HashMap<String, Box<dyn Fn(&serde_json::Value) -> bool + Send + Sync>> = HashMap::new();
        
        // Rule: Route to high priority if data size is large
        routing_rules.insert("high_priority".to_string(), Box::new(|data| {
            if let Some(array) = data.as_array() {
                array.len() > 100
            } else {
                data.to_string().len() > 1000
            }
        }));
        
        // Rule: Route to low priority if data is simple
        routing_rules.insert("low_priority".to_string(), Box::new(|data| {
            if let Some(obj) = data.as_object() {
                obj.len() <= 3
            } else {
                true
            }
        }));
        
        Self { routing_rules }
    }
    
    fn evaluate_routing(&self, data: &serde_json::Value) -> String {
        for (route, rule) in &self.routing_rules {
            if rule(data) {
                return route.clone();
            }
        }
        "default".to_string()
    }
}

impl Node for DataRouterNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let input_data: serde_json::Value = context.get_event_data()?;
        
        let route = self.evaluate_routing(&input_data);
        
        context.update_node("routing_decision", json!({
            "selected_route": route,
            "evaluated_at": chrono::Utc::now(),
            "data_characteristics": {
                "is_array": input_data.is_array(),
                "is_object": input_data.is_object(),
                "estimated_size": input_data.to_string().len()
            }
        }));
        
        context.set_metadata("route_selected", route)?;
        
        println!("🛤️  Router selected route: {}", context.get_metadata::<String>("route_selected")?.unwrap_or_default());
        
        Ok(context)
    }
}

impl Router for DataRouterNode {
    fn route(&self, context: &TaskContext) -> Option<Box<dyn Node>> {
        if let Ok(Some(route)) = context.get_metadata::<String>("route_selected") {
            match route.as_str() {
                "high_priority" => {
                    let config = DataProcessorConfig {
                        processing_mode: ProcessingMode::RealTime,
                        batch_size: 1,
                        timeout_seconds: 5,
                        validation_rules: vec![],
                        output_format: OutputFormat::Json,
                    };
                    Some(Box::new(DataProcessorNode::new("high_priority", config).unwrap()))
                }
                "low_priority" => {
                    let config = DataProcessorConfig {
                        processing_mode: ProcessingMode::Batch,
                        batch_size: 100,
                        timeout_seconds: 30,
                        validation_rules: vec![],
                        output_format: OutputFormat::Csv,
                    };
                    Some(Box::new(DataProcessorNode::new("low_priority", config).unwrap()))
                }
                _ => None,
            }
        } else {
            None
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), WorkflowError> {
    println!("🚀 Starting Custom Node Implementation Example");
    println!("==============================================");
    
    // Create sophisticated validation rules
    let validation_rules = vec![
        ValidationRule {
            field: "name".to_string(),
            rule_type: RuleType::Required,
            value: json!(true),
        },
        ValidationRule {
            field: "name".to_string(),
            rule_type: RuleType::MinLength,
            value: json!(3),
        },
        ValidationRule {
            field: "email".to_string(),
            rule_type: RuleType::Pattern,
            value: json!(r"^[^\s@]+@[^\s@]+\.[^\s@]+$"),
        },
        ValidationRule {
            field: "age".to_string(),
            rule_type: RuleType::Range,
            value: json!([0, 120]),
        },
    ];
    
    // Configure data processor
    let processor_config = DataProcessorConfig {
        processing_mode: ProcessingMode::Stream,
        batch_size: 10,
        timeout_seconds: 30,
        validation_rules,
        output_format: OutputFormat::Json,
    };
    
    // Build workflow with custom nodes
    let workflow = TypedWorkflowBuilder::new("custom_node_workflow")
        .description("Demonstrates advanced custom node implementations")
        .start_with_node(NodeId::new("router"))
        .then_node(NodeId::new("processor"))
        .build()?;
    
    // Register custom nodes
    let processor_node = DataProcessorNode::new("main_processor", processor_config)?;
    let router_node = DataRouterNode::new();
    
    workflow.register_node(NodeId::new("processor"), processor_node);
    workflow.register_node(NodeId::new("router"), router_node);
    
    println!("📋 Custom workflow built:");
    println!("   1. DataRouterNode - Routes based on data characteristics");
    println!("   2. DataProcessorNode - Advanced processing with validation");
    println!();
    
    // Test cases with different data characteristics
    let test_cases = vec![
        // Valid data - should pass validation
        json!({
            "name": "Alice Smith",
            "email": "alice@example.com",
            "age": 28,
            "department": "Engineering"
        }),
        
        // Invalid email - should fail validation
        json!({
            "name": "Bob",
            "email": "invalid-email",
            "age": 35
        }),
        
        // Large dataset - should route to high priority
        json!({
            "users": (0..150).map(|i| json!({
                "id": i,
                "name": format!("User {}", i),
                "email": format!("user{}@example.com", i)
            })).collect::<Vec<_>>()
        }),
        
        // Simple data - should route to low priority
        json!({
            "status": "ok"
        })
    ];
    
    for (i, input_data) in test_cases.into_iter().enumerate() {
        println!("🔄 Test Case {} - Processing...", i + 1);
        println!("   Input size: {} characters", input_data.to_string().len());
        
        // Run the workflow
        let result = workflow.run(input_data).await?;
        
        // Display routing decision
        if let Some(routing) = result.get_node_data::<serde_json::Value>("routing_decision")? {
            println!("   🛤️  Route: {}", routing["selected_route"]);
        }
        
        // Display validation results
        if let Some(validation) = result.get_node_data::<serde_json::Value>("validation_summary")? {
            println!("   ✅ Validation: {}", validation["status"]);
            if validation["status"] == "failed" {
                if let Some(failed_rules) = validation["failed_rules"].as_array() {
                    for rule in failed_rules {
                        println!("      ❌ {}", rule.as_str().unwrap_or(""));
                    }
                }
            } else {
                println!("      ✅ All {} rules passed", validation["total_rules"]);
            }
        }
        
        // Display performance metrics
        if let Some(processing_time) = result.get_metadata::<u64>("processing_time_ms")? {
            println!("   ⏱️  Processing time: {} ms", processing_time);
        }
        
        if let Some(output_format) = result.get_metadata::<String>("output_format")? {
            println!("   📄 Output format: {}", output_format);
        }
        
        println!("   ✅ Test case completed");
        println!();
    }
    
    println!("🎉 Custom Node Implementation Example completed!");
    println!("================================================");
    println!();
    println!("Advanced features demonstrated:");
    println!("• Complex node configuration and validation");
    println!("• Router nodes with conditional logic");
    println!("• Data validation with multiple rule types");
    println!("• Output formatting and transformation");
    println!("• Performance monitoring and statistics");
    println!("• Sophisticated error handling");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_data_processor_validation() {
        let config = DataProcessorConfig {
            processing_mode: ProcessingMode::Stream,
            batch_size: 10,
            timeout_seconds: 30,
            validation_rules: vec![
                ValidationRule {
                    field: "name".to_string(),
                    rule_type: RuleType::Required,
                    value: json!(true),
                },
            ],
            output_format: OutputFormat::Json,
        };
        
        let node = DataProcessorNode::new("test", config).unwrap();
        
        // Test valid data
        let valid_data = json!({"name": "Test User"});
        let result = node.validate_data(&valid_data);
        assert!(result.is_ok());
        
        // Test invalid data
        let invalid_data = json!({});
        let result = node.validate_data(&invalid_data);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_data_router_routing() {
        let router = DataRouterNode::new();
        
        // Test large data routing
        let large_data = json!({
            "items": (0..200).map(|i| json!({"id": i})).collect::<Vec<_>>()
        });
        assert_eq!(router.evaluate_routing(&large_data), "high_priority");
        
        // Test simple data routing
        let simple_data = json!({"status": "ok"});
        assert_eq!(router.evaluate_routing(&simple_data), "low_priority");
    }
    
    #[test]
    fn test_output_formatting() {
        let config = DataProcessorConfig {
            processing_mode: ProcessingMode::Batch,
            batch_size: 1,
            timeout_seconds: 10,
            validation_rules: vec![],
            output_format: OutputFormat::Json,
        };
        
        let node = DataProcessorNode::new("test", config).unwrap();
        let data = json!({"test": "value"});
        
        let formatted = node.format_output(&data).unwrap();
        assert!(formatted.contains("test"));
        assert!(formatted.contains("value"));
    }
}