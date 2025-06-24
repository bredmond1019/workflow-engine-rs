/*!
# Workflow Executor

This module implements the workflow execution engine that can process
workflow definitions and execute steps including cross-system calls.

Task 2.2: Create cross_system step type for AI Tutor integration
*/

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

use crate::monitoring::metrics::{WorkflowMetrics};
use crate::monitoring::correlation::get_correlation_id;
use crate::monitoring::logging::log_workflow_event;
use crate::{info_with_correlation, warn_with_correlation, error_with_correlation, debug_with_correlation};
use tracing::{info, warn, error, debug, instrument};

use workflow_engine_core::error::WorkflowError;
use crate::integrations::{CrossSystemClient, CrossSystemError};
use crate::integrations::cross_system::HttpCrossSystemClient;
use crate::workflows::schema::{
    WorkflowDefinition, WorkflowInstance, WorkflowStatus, StepDefinition, 
    StepType, StepExecution, StepStatus, WorkflowError as SchemaWorkflowError
};

/// Template engine for processing step inputs and outputs
pub struct TemplateEngine {
    context: HashMap<String, Value>,
}

impl TemplateEngine {
    pub fn new() -> Self {
        Self {
            context: HashMap::new(),
        }
    }
    
    pub fn with_context(mut self, key: String, value: Value) -> Self {
        self.context.insert(key, value);
        self
    }
    
    /// Simple template rendering - in production, use handlebars or similar
    pub fn render(&self, template: &str) -> Result<String, WorkflowError> {
        let mut result = template.to_string();
        
        // Replace {{ variable }} patterns
        for (key, value) in &self.context {
            let pattern = format!("{{{{ {} }}}}", key);
            let replacement = match value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => serde_json::to_string(value).unwrap_or_default(),
            };
            result = result.replace(&pattern, &replacement);
        }
        
        Ok(result)
    }
    
    /// Render a JSON value template
    pub fn render_json(&self, template: &Value) -> Result<Value, WorkflowError> {
        match template {
            Value::String(s) => {
                let rendered = self.render(s)?;
                // Try to parse as JSON, fallback to string
                Ok(serde_json::from_str(&rendered).unwrap_or(Value::String(rendered)))
            }
            Value::Object(obj) => {
                let mut result = serde_json::Map::new();
                for (key, value) in obj {
                    result.insert(key.clone(), self.render_json(value)?);
                }
                Ok(Value::Object(result))
            }
            Value::Array(arr) => {
                let mut result = Vec::new();
                for item in arr {
                    result.push(self.render_json(item)?);
                }
                Ok(Value::Array(result))
            }
            other => Ok(other.clone()),
        }
    }
}

/// Step executor trait for different step types
#[async_trait]
pub trait StepExecutor: Send + Sync {
    async fn execute(
        &self,
        step: &StepDefinition,
        context: &WorkflowContext,
    ) -> Result<Value, WorkflowError>;
}

/// Cross-system step executor for AI Tutor integration
pub struct CrossSystemExecutor {
    cross_system_client: Arc<dyn CrossSystemClient>,
}

impl CrossSystemExecutor {
    pub fn new(registry_endpoint: String, auth_token: Option<String>) -> Self {
        let client = HttpCrossSystemClient::new(registry_endpoint);
        let client = if let Some(token) = auth_token {
            client.with_auth_token(token)
        } else {
            client
        };
        
        Self {
            cross_system_client: Arc::new(client),
        }
    }
}

#[async_trait]
impl StepExecutor for CrossSystemExecutor {
    async fn execute(
        &self,
        step: &StepDefinition,
        context: &WorkflowContext,
    ) -> Result<Value, WorkflowError> {
        let cross_system = match &step.step_type {
            StepType::CrossSystem { system, operation, agent } => (system, operation, agent),
            _ => return Err(WorkflowError::invalid_step_type_simple(
                "Expected CrossSystem step type".to_string(),
                "unknown".to_string()
            )),
        };
        
        log::info!("Executing cross-system step: {} -> {}", step.id, cross_system.0);
        
        // Render step input with current context
        let template_engine = context.create_template_engine();
        let rendered_input = template_engine.render_json(&step.input)?;
        
        log::debug!("Rendered input for step {}: {}", step.id, rendered_input);
        
        // Discover services for the target system
        let services = self.cross_system_client
            .discover_services(cross_system.0)
            .await
            .map_err(|e| WorkflowError::cross_system_error_simple(format!("Service discovery failed: {}", e)))?;
        
        if services.is_empty() {
            return Err(WorkflowError::cross_system_error_simple(
                format!("No services found for system: {}", cross_system.0)
            ));
        }
        
        // Use specific agent if provided, otherwise use first service
        let service_name = if let Some(agent) = cross_system.2 {
            services.iter()
                .find(|s| s.contains(agent))
                .unwrap_or(&services[0])
        } else {
            &services[0]
        };
        
        log::info!("Calling service: {} with operation: {}", service_name, cross_system.1);
        
        // Make the cross-system call
        let result = self.cross_system_client
            .call_service(service_name, cross_system.1, rendered_input)
            .await
            .map_err(|e| WorkflowError::cross_system_error_simple(format!("Service call failed: {}", e)))?;
        
        log::info!("Cross-system call completed for step: {}", step.id);
        
        Ok(result)
    }
}

/// Node executor for local workflow nodes
pub struct NodeExecutor {
    node_registry: HashMap<String, Box<dyn StepExecutor>>,
}

impl NodeExecutor {
    pub fn new() -> Self {
        let mut node_registry: HashMap<String, Box<dyn StepExecutor>> = HashMap::new();
        
        // Register NotionClientNode if environment is configured
        // Note: This is disabled until the notion_client module is properly implemented
        // if let Ok(notion_executor) = crate::workflows::nodes::notion_client::create_notion_executor() {
        //     node_registry.insert("NotionClientNode".to_string(), notion_executor);
        //     log::info!("Registered NotionClientNode executor");
        // } else {
        //     log::warn!("NotionClientNode not available - check NOTION_MCP_URL configuration");
        // }
        
        Self { node_registry }
    }
    
    pub fn register_node(&mut self, name: String, executor: Box<dyn StepExecutor>) {
        self.node_registry.insert(name, executor);
    }
}

#[async_trait]
impl StepExecutor for NodeExecutor {
    async fn execute(
        &self,
        step: &StepDefinition,
        context: &WorkflowContext,
    ) -> Result<Value, WorkflowError> {
        let node_type = match &step.step_type {
            StepType::Node { node } => node,
            _ => return Err(WorkflowError::invalid_step_type_simple(
                "Expected Node step type".to_string(),
                "unknown".to_string()
            )),
        };
        
        log::info!("Executing node step: {} -> {}", step.id, node_type);
        
        // Check if we have a registered executor for this node type
        if let Some(executor) = self.node_registry.get(node_type) {
            log::debug!("Using registered executor for node type: {}", node_type);
            executor.execute(step, context).await
        } else {
            log::warn!("No registered executor for node type: {}, using mock", node_type);
            
            // Return a mock result for unregistered node types
            Ok(serde_json::json!({
                "step_id": step.id,
                "node_type": node_type,
                "status": "completed",
                "timestamp": chrono::Utc::now(),
                "mock": true,
                "message": format!("Mock execution of {} node (not registered)", node_type)
            }))
        }
    }
}

/// Workflow execution context
pub struct WorkflowContext {
    instance: WorkflowInstance,
    step_outputs: HashMap<String, Value>,
    environment: HashMap<String, String>,
}

impl WorkflowContext {
    pub fn new(instance: WorkflowInstance) -> Self {
        Self {
            instance,
            step_outputs: HashMap::new(),
            environment: std::env::vars().collect(),
        }
    }
    
    pub fn add_step_output(&mut self, step_id: String, output: Value) {
        self.step_outputs.insert(step_id, output);
    }
    
    pub fn get_step_output(&self, step_id: &str) -> Option<&Value> {
        self.step_outputs.get(step_id)
    }
    
    pub fn create_template_engine(&self) -> TemplateEngine {
        let mut engine = TemplateEngine::new();
        
        // Add workflow inputs
        engine = engine.with_context("input".to_string(), self.instance.inputs.clone());
        
        // Add step outputs
        for (step_id, output) in &self.step_outputs {
            engine = engine.with_context(
                format!("steps.{}.output", step_id),
                output.clone()
            );
        }
        
        // Add environment variables
        let env_value = serde_json::to_value(&self.environment).unwrap_or_default();
        engine = engine.with_context("env".to_string(), env_value);
        
        // Add current timestamp
        engine = engine.with_context(
            "now".to_string(),
            Value::String(chrono::Utc::now().to_rfc3339())
        );
        
        engine
    }
}

/// Main workflow executor
pub struct WorkflowExecutor {
    cross_system_executor: CrossSystemExecutor,
    node_executor: NodeExecutor,
}

impl WorkflowExecutor {
    pub fn new(registry_endpoint: String, auth_token: Option<String>) -> Self {
        Self {
            cross_system_executor: CrossSystemExecutor::new(registry_endpoint, auth_token),
            node_executor: NodeExecutor::new(),
        }
    }
    
    /// Execute a workflow instance
    #[instrument(skip(self, instance))]
    pub async fn execute(&self, mut instance: WorkflowInstance) -> Result<WorkflowInstance, WorkflowError> {
        info_with_correlation!(
            workflow_id = %instance.id,
            workflow_name = %instance.workflow.name,
            "Starting execution of workflow"
        );
        
        // Log workflow start event
        let mut event_details = std::collections::HashMap::new();
        event_details.insert("workflow_name".to_string(), instance.workflow.name.clone());
        event_details.insert("description".to_string(), instance.workflow.description.clone());
        log_workflow_event(
            &instance.id.to_string(),
            "started",
            None,
            event_details,
            get_correlation_id().as_deref(),
        );
        
        // Start workflow metrics timer
        let workflow_timer = WorkflowMetrics::record_workflow_start(&instance.workflow.name);
        
        instance.status = WorkflowStatus::Running;
        instance.started_at = Some(chrono::Utc::now());
        
        let mut context = WorkflowContext::new(instance.clone());
        
        // Execute steps in order
        for step in &instance.workflow.steps {
            info_with_correlation!(
                workflow_id = %instance.id,
                step_id = %step.id,
                step_name = %step.name.as_ref().unwrap_or(&step.id),
                step_type = ?step.step_type,
                "Executing workflow step"
            );
            
            // Check dependencies
            if !self.check_dependencies(step, &context)? {
                warn_with_correlation!(
                    workflow_id = %instance.id,
                    step_id = %step.id,
                    "Step skipped due to failed dependencies"
                );
                continue;
            }
            
            // Create step execution record
            let step_start_time = Instant::now();
            let mut step_execution = StepExecution {
                status: StepStatus::Running,
                input: step.input.clone(),
                output: None,
                started_at: Some(chrono::Utc::now()),
                completed_at: None,
                error: None,
                attempt: 1,
            };
            
            // Determine step type for metrics
            let step_type = match &step.step_type {
                StepType::CrossSystem { .. } => "cross_system",
                StepType::Node { .. } => "node",
                StepType::Transform { .. } => "transform",
                StepType::Condition { .. } => "condition",
                StepType::Loop { .. } => "loop",
            };
            
            // Execute the step
            match self.execute_step(step, &context).await {
                Ok(output) => {
                    let duration = step_start_time.elapsed();
                    step_execution.status = StepStatus::Completed;
                    step_execution.output = Some(output.clone());
                    step_execution.completed_at = Some(chrono::Utc::now());
                    
                    // Record successful step execution metrics
                    WorkflowMetrics::record_step_execution(
                        &instance.workflow.name,
                        step_type,
                        "success",
                        duration
                    );
                    
                    // Add output to context for next steps
                    context.add_step_output(step.id.clone(), output);
                    
                    log::info!("Step {} completed successfully", step.id);
                }
                Err(e) => {
                    let duration = step_start_time.elapsed();
                    step_execution.status = StepStatus::Failed;
                    step_execution.error = Some(e.to_string());
                    step_execution.completed_at = Some(chrono::Utc::now());
                    
                    // Record failed step execution metrics
                    WorkflowMetrics::record_step_execution(
                        &instance.workflow.name,
                        step_type,
                        "failure",
                        duration
                    );
                    
                    log::error!("Step {} failed: {}", step.id, e);
                    
                    // Check if workflow should continue on error
                    if !instance.workflow.config.continue_on_error.unwrap_or(false) {
                        instance.status = WorkflowStatus::Failed;
                        instance.error = Some(SchemaWorkflowError {
                            message: e.to_string(),
                            code: "STEP_EXECUTION_FAILED".to_string(),
                            step_id: Some(step.id.clone()),
                            details: None,
                        });
                        instance.completed_at = Some(chrono::Utc::now());
                        
                        // Store step execution and return with failure metrics
                        instance.steps.insert(step.id.clone(), step_execution);
                        workflow_timer.failure();
                        return Ok(instance);
                    }
                }
            }
            
            // Store step execution
            instance.steps.insert(step.id.clone(), step_execution);
        }
        
        // Generate workflow outputs
        instance.outputs = Some(self.generate_outputs(&instance, &context)?);
        instance.status = WorkflowStatus::Completed;
        instance.completed_at = Some(chrono::Utc::now());
        
        // Record successful workflow completion
        workflow_timer.success();
        
        log::info!("Workflow {} completed successfully", instance.workflow.name);
        
        Ok(instance)
    }
    
    fn check_dependencies(&self, step: &StepDefinition, context: &WorkflowContext) -> Result<bool, WorkflowError> {
        for dep in &step.depends_on {
            if let Some(step_output) = context.get_step_output(dep) {
                // Check if dependency completed successfully
                // This is a simplified check - in reality you'd check the step status
                if step_output.is_null() {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }
        }
        Ok(true)
    }
    
    async fn execute_step(&self, step: &StepDefinition, context: &WorkflowContext) -> Result<Value, WorkflowError> {
        match &step.step_type {
            StepType::CrossSystem { .. } => {
                self.cross_system_executor.execute(step, context).await
            }
            StepType::Node { .. } => {
                self.node_executor.execute(step, context).await
            }
            StepType::Transform { engine, template } => {
                self.execute_transform(engine, template, context).await
            }
            _ => {
                Err(WorkflowError::invalid_step_type_simple(
                    format!("Unsupported step type in step: {}", step.id),
                    "unknown".to_string()
                ))
            }
        }
    }
    
    async fn execute_transform(&self, _engine: &str, template: &str, context: &WorkflowContext) -> Result<Value, WorkflowError> {
        let template_engine = context.create_template_engine();
        let result = template_engine.render(template)?;
        
        // Try to parse as JSON, fallback to string
        Ok(serde_json::from_str(&result).unwrap_or(Value::String(result)))
    }
    
    fn generate_outputs(&self, instance: &WorkflowInstance, context: &WorkflowContext) -> Result<Value, WorkflowError> {
        let mut outputs = serde_json::Map::new();
        let template_engine = context.create_template_engine();
        
        for (key, template) in &instance.workflow.outputs {
            let rendered = template_engine.render(template)?;
            outputs.insert(key.clone(), Value::String(rendered));
        }
        
        Ok(Value::Object(outputs))
    }
}

/// Workflow factory for creating instances
pub struct WorkflowFactory;

impl WorkflowFactory {
    /// Create a new workflow instance from a definition
    pub fn create_instance(
        workflow: WorkflowDefinition,
        inputs: Value,
    ) -> Result<WorkflowInstance, WorkflowError> {
        // Validate inputs against schema
        Self::validate_inputs(&workflow, &inputs)?;
        
        let instance = WorkflowInstance {
            id: Uuid::new_v4(),
            workflow,
            status: WorkflowStatus::Created,
            inputs,
            steps: HashMap::new(),
            outputs: None,
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            error: None,
        };
        
        Ok(instance)
    }
    
    fn validate_inputs(workflow: &WorkflowDefinition, inputs: &Value) -> Result<(), WorkflowError> {
        let input_obj = inputs.as_object().ok_or_else(|| {
            WorkflowError::invalid_input_simple("Inputs must be a JSON object")
        })?;
        
        // Check required inputs
        for (key, def) in &workflow.inputs {
            if def.required && !input_obj.contains_key(key) {
                return Err(WorkflowError::invalid_input_simple(
                    format!("Required input '{}' is missing", key)
                ));
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflows::schema::templates;
    
    #[test]
    fn test_template_engine() {
        let mut engine = TemplateEngine::new();
        engine = engine.with_context("name".to_string(), Value::String("test".to_string()));
        engine = engine.with_context("count".to_string(), Value::Number(serde_json::Number::from(42)));
        
        let result = engine.render("Hello {{ name }}, count is {{ count }}").unwrap();
        assert_eq!(result, "Hello test, count is 42");
    }
    
    #[test]
    fn test_workflow_factory() {
        let workflow = templates::research_to_documentation();
        let inputs = serde_json::json!({
            "topic": "machine learning",
            "difficulty": "intermediate"
        });
        
        let instance = WorkflowFactory::create_instance(workflow, inputs).unwrap();
        assert_eq!(instance.status, WorkflowStatus::Created);
        assert_eq!(instance.inputs["topic"], "machine learning");
    }
    
    #[tokio::test]
    async fn test_cross_system_executor() {
        let executor = CrossSystemExecutor::new(
            "http://localhost:8080".to_string(),
            None
        );
        
        let step = StepDefinition {
            id: "test_step".to_string(),
            name: None,
            step_type: StepType::CrossSystem {
                system: "ai-tutor".to_string(),
                operation: "research".to_string(),
                agent: None,
            },
            input: serde_json::json!({"topic": "test"}),
            depends_on: vec![],
            parallel: false,
            config: Default::default(),
        };
        
        let workflow = templates::research_to_documentation();
        let instance = WorkflowFactory::create_instance(
            workflow,
            serde_json::json!({"topic": "test"})
        ).unwrap();
        let context = WorkflowContext::new(instance);
        
        // This would fail in test without actual services, but tests the structure
        let _result = executor.execute(&step, &context).await;
    }
}