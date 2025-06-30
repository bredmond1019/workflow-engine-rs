/*!
# Workflow API Endpoints

This module implements HTTP API endpoints for triggering and monitoring workflows.

Task 2.5: Create workflow trigger API endpoint (POST /api/v1/workflows/trigger)
Task 2.6: Implement workflow status endpoint (GET /api/v1/workflows/status/{id})
*/

use actix_web::{HttpResponse, Result as ActixResult, web};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use workflow_engine_core::error::WorkflowError;
use crate::workflows::{
    executor::{WorkflowExecutor, WorkflowFactory},
    parser::{WorkflowRegistry, create_default_registry},
    registry::{TemplateSearchCriteria, WorkflowTemplateMetadata, WorkflowTemplateRegistry},
    schema::{WorkflowInstance, WorkflowStatus},
};

/// Request payload for triggering a workflow
#[derive(Debug, Deserialize, Serialize)]
pub struct TriggerWorkflowRequest {
    /// Name of the workflow to execute
    pub workflow_name: String,

    /// Input data for the workflow
    pub inputs: Value,

    /// Optional workflow configuration overrides
    pub config: Option<WorkflowConfigOverrides>,
}

/// Configuration overrides for workflow execution
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkflowConfigOverrides {
    /// Override workflow timeout in seconds
    pub timeout: Option<u64>,

    /// Override retry count
    pub retries: Option<u32>,

    /// Override continue_on_error setting
    pub continue_on_error: Option<bool>,

    /// Additional environment variables
    pub environment: Option<HashMap<String, String>>,
}

/// Response for workflow trigger request
#[derive(Debug, Serialize)]
pub struct TriggerWorkflowResponse {
    /// Unique instance ID for the triggered workflow
    pub instance_id: Uuid,

    /// URL to check workflow status
    pub status_url: String,

    /// Initial workflow status
    pub status: WorkflowStatus,

    /// Workflow name that was triggered
    pub workflow_name: String,

    /// Timestamp when workflow was created
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Response for workflow status request
#[derive(Debug, Serialize)]
pub struct WorkflowStatusResponse {
    /// Workflow instance ID
    pub instance_id: Uuid,

    /// Current workflow status
    pub status: WorkflowStatus,

    /// Workflow definition name
    pub workflow_name: String,

    /// Input data provided to the workflow
    pub inputs: Value,

    /// Current step statuses
    pub steps: HashMap<String, StepStatusInfo>,

    /// Final workflow outputs (if completed)
    pub outputs: Option<Value>,

    /// Execution timestamps
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Error information (if failed)
    pub error: Option<WorkflowErrorInfo>,

    /// Execution progress information
    pub progress: WorkflowProgress,
}

/// Step status information for API response
#[derive(Debug, Serialize)]
pub struct StepStatusInfo {
    /// Step execution status
    pub status: crate::workflows::schema::StepStatus,

    /// Step output (if completed)
    pub output: Option<Value>,

    /// Execution timestamps
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Error message (if failed)
    pub error: Option<String>,

    /// Retry attempt number
    pub attempt: u32,
}

/// Workflow error information for API response
#[derive(Debug, Serialize)]
pub struct WorkflowErrorInfo {
    /// Error message
    pub message: String,

    /// Error code
    pub code: String,

    /// Step that caused the error
    pub step_id: Option<String>,

    /// Additional error details
    pub details: Option<Value>,
}

/// Workflow execution progress information
#[derive(Debug, Serialize)]
pub struct WorkflowProgress {
    /// Total number of steps
    pub total_steps: u32,

    /// Number of completed steps
    pub completed_steps: u32,

    /// Number of failed steps
    pub failed_steps: u32,

    /// Number of running steps
    pub running_steps: u32,

    /// Progress percentage (0-100)
    pub percentage: u8,
}

/// Workflow management service
pub struct WorkflowService {
    registry: Arc<RwLock<WorkflowRegistry>>,
    template_registry: Arc<RwLock<WorkflowTemplateRegistry>>,
    executor: Arc<WorkflowExecutor>,
    running_instances: Arc<RwLock<HashMap<Uuid, WorkflowInstance>>>,
}

impl WorkflowService {
    pub async fn new() -> Result<Self, WorkflowError> {
        let registry = create_default_registry()?;
        let template_registry = WorkflowTemplateRegistry::new();

        // Create executor with default configuration
        let registry_endpoint = std::env::var("REGISTRY_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let auth_token = std::env::var("AUTH_TOKEN").ok();

        let executor = WorkflowExecutor::new(registry_endpoint, auth_token);

        Ok(Self {
            registry: Arc::new(RwLock::new(registry)),
            template_registry: Arc::new(RwLock::new(template_registry)),
            executor: Arc::new(executor),
            running_instances: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Trigger a workflow execution
    pub async fn trigger_workflow(
        &self,
        request: TriggerWorkflowRequest,
    ) -> Result<TriggerWorkflowResponse, WorkflowError> {
        log::info!("Triggering workflow: {}", request.workflow_name);

        // Get workflow definition from registry
        let registry = self.registry.read().await;
        let workflow = registry
            .parser()
            .get_workflow(&request.workflow_name)
            .ok_or_else(|| {
                WorkflowError::invalid_input_simple(format!(
                    "Workflow '{}' not found",
                    request.workflow_name
                ))
            })?
            .clone();
        drop(registry);

        // Create workflow instance
        let mut instance = WorkflowFactory::create_instance(workflow, request.inputs)?;

        // Apply configuration overrides if provided
        if let Some(config_overrides) = request.config {
            if let Some(timeout) = config_overrides.timeout {
                instance.workflow.config.timeout = Some(timeout);
            }
            if let Some(retries) = config_overrides.retries {
                instance.workflow.config.retries = Some(retries);
            }
            if let Some(continue_on_error) = config_overrides.continue_on_error {
                instance.workflow.config.continue_on_error = Some(continue_on_error);
            }
            if let Some(env) = config_overrides.environment {
                instance.workflow.config.environment.extend(env);
            }
        }

        let instance_id = instance.id;
        let created_at = instance.created_at;

        // Store instance for tracking
        {
            let mut instances = self.running_instances.write().await;
            instances.insert(instance_id, instance.clone());
        }

        // Start execution in background
        let executor = Arc::clone(&self.executor);
        let instances = Arc::clone(&self.running_instances);

        tokio::spawn(async move {
            log::info!("Starting workflow execution for instance: {}", instance_id);

            match executor.execute(instance).await {
                Ok(completed_instance) => {
                    log::info!("Workflow execution completed for instance: {}", instance_id);

                    // Update stored instance
                    let mut instances_guard = instances.write().await;
                    instances_guard.insert(instance_id, completed_instance);
                }
                Err(e) => {
                    log::error!(
                        "Workflow execution failed for instance {}: {}",
                        instance_id,
                        e
                    );

                    // Update stored instance with error
                    let mut instances_guard = instances.write().await;
                    if let Some(instance) = instances_guard.get_mut(&instance_id) {
                        instance.status = WorkflowStatus::Failed;
                        instance.error = Some(crate::workflows::schema::WorkflowError {
                            message: e.to_string(),
                            code: "EXECUTION_FAILED".to_string(),
                            step_id: None,
                            details: None,
                        });
                        instance.completed_at = Some(chrono::Utc::now());
                    }
                }
            }
        });

        Ok(TriggerWorkflowResponse {
            instance_id,
            status_url: format!("/api/v1/workflows/status/{}", instance_id),
            status: WorkflowStatus::Created,
            workflow_name: request.workflow_name,
            created_at,
        })
    }

    /// Get workflow status
    pub async fn get_workflow_status(
        &self,
        instance_id: Uuid,
    ) -> Result<WorkflowStatusResponse, WorkflowError> {
        let instances = self.running_instances.read().await;
        let instance = instances.get(&instance_id).ok_or_else(|| {
            WorkflowError::invalid_input_simple(format!("Workflow instance '{}' not found", instance_id))
        })?;

        // Convert step executions to API format
        let steps: HashMap<String, StepStatusInfo> = instance
            .steps
            .iter()
            .map(|(id, execution)| {
                (
                    id.clone(),
                    StepStatusInfo {
                        status: execution.status.clone(),
                        output: execution.output.clone(),
                        started_at: execution.started_at,
                        completed_at: execution.completed_at,
                        error: execution.error.clone(),
                        attempt: execution.attempt,
                    },
                )
            })
            .collect();

        // Calculate progress
        let total_steps = instance.workflow.steps.len() as u32;
        let completed_steps = steps
            .values()
            .filter(|s| s.status == crate::workflows::schema::StepStatus::Completed)
            .count() as u32;
        let failed_steps = steps
            .values()
            .filter(|s| s.status == crate::workflows::schema::StepStatus::Failed)
            .count() as u32;
        let running_steps = steps
            .values()
            .filter(|s| s.status == crate::workflows::schema::StepStatus::Running)
            .count() as u32;

        let percentage = if total_steps > 0 {
            ((completed_steps * 100) / total_steps).min(100) as u8
        } else {
            0
        };

        let progress = WorkflowProgress {
            total_steps,
            completed_steps,
            failed_steps,
            running_steps,
            percentage,
        };

        // Convert error if present
        let error = instance.error.as_ref().map(|e| WorkflowErrorInfo {
            message: e.message.clone(),
            code: e.code.clone(),
            step_id: e.step_id.clone(),
            details: e.details.clone(),
        });

        Ok(WorkflowStatusResponse {
            instance_id: instance.id,
            status: instance.status.clone(),
            workflow_name: instance.workflow.name.clone(),
            inputs: instance.inputs.clone(),
            steps,
            outputs: instance.outputs.clone(),
            created_at: instance.created_at,
            started_at: instance.started_at,
            completed_at: instance.completed_at,
            error,
            progress,
        })
    }

    /// List all workflow instances
    pub async fn list_instances(&self) -> Vec<(Uuid, WorkflowStatus, String)> {
        let instances = self.running_instances.read().await;
        instances
            .iter()
            .map(|(id, instance)| (*id, instance.status.clone(), instance.workflow.name.clone()))
            .collect()
    }

    /// Get available workflows
    pub async fn list_workflows(&self) -> Vec<String> {
        let registry = self.registry.read().await;
        registry
            .parser()
            .list_workflows()
            .iter()
            .map(|w| w.name.clone())
            .collect()
    }

    /// List all workflow templates
    pub async fn list_templates(&self) -> Vec<WorkflowTemplateMetadata> {
        let template_registry = self.template_registry.read().await;
        template_registry
            .list_templates()
            .into_iter()
            .cloned()
            .collect()
    }

    /// Search workflow templates
    pub async fn search_templates(
        &self,
        criteria: TemplateSearchCriteria,
    ) -> Vec<WorkflowTemplateMetadata> {
        let template_registry = self.template_registry.read().await;
        template_registry
            .search_templates(&criteria)
            .into_iter()
            .cloned()
            .collect()
    }

    /// Get template by ID
    pub async fn get_template(&self, template_id: &str) -> Option<WorkflowTemplateMetadata> {
        let template_registry = self.template_registry.read().await;
        template_registry.get_metadata(template_id).cloned()
    }

    /// Get template categories
    pub async fn get_template_categories(&self) -> Vec<String> {
        let template_registry = self.template_registry.read().await;
        template_registry.get_categories()
    }

    /// Get template tags
    pub async fn get_template_tags(&self) -> Vec<String> {
        let template_registry = self.template_registry.read().await;
        template_registry.get_tags()
    }

    /// Trigger workflow from template
    pub async fn trigger_from_template(
        &self,
        template_id: &str,
        inputs: Value,
        config_overrides: Option<WorkflowConfigOverrides>,
    ) -> Result<TriggerWorkflowResponse, WorkflowError> {
        log::info!("Triggering workflow from template: {}", template_id);

        // Get workflow definition from template registry
        let template_registry = self.template_registry.read().await;
        let workflow = template_registry
            .get_template(template_id)
            .ok_or_else(|| {
                WorkflowError::invalid_input_simple(format!("Template '{}' not found", template_id))
            })?
            .clone();
        drop(template_registry);

        // Create workflow instance
        let mut instance = WorkflowFactory::create_instance(workflow, inputs)?;

        // Apply configuration overrides if provided
        if let Some(config_overrides) = config_overrides {
            if let Some(timeout) = config_overrides.timeout {
                instance.workflow.config.timeout = Some(timeout);
            }
            if let Some(retries) = config_overrides.retries {
                instance.workflow.config.retries = Some(retries);
            }
            if let Some(continue_on_error) = config_overrides.continue_on_error {
                instance.workflow.config.continue_on_error = Some(continue_on_error);
            }
            if let Some(env) = config_overrides.environment {
                instance.workflow.config.environment.extend(env);
            }
        }

        let instance_id = instance.id;
        let created_at = instance.created_at;
        let workflow_name = instance.workflow.name.clone();

        // Store instance for tracking
        {
            let mut instances = self.running_instances.write().await;
            instances.insert(instance_id, instance.clone());
        }

        // Start execution in background
        let executor = Arc::clone(&self.executor);
        let instances = Arc::clone(&self.running_instances);

        tokio::spawn(async move {
            log::info!(
                "Starting workflow execution for template instance: {}",
                instance_id
            );

            match executor.execute(instance).await {
                Ok(completed_instance) => {
                    log::info!(
                        "Template workflow execution completed for instance: {}",
                        instance_id
                    );

                    // Update stored instance
                    let mut instances_guard = instances.write().await;
                    instances_guard.insert(instance_id, completed_instance);
                }
                Err(e) => {
                    log::error!(
                        "Template workflow execution failed for instance {}: {}",
                        instance_id,
                        e
                    );

                    // Update stored instance with error
                    let mut instances_guard = instances.write().await;
                    if let Some(instance) = instances_guard.get_mut(&instance_id) {
                        instance.status = WorkflowStatus::Failed;
                        instance.error = Some(crate::workflows::schema::WorkflowError {
                            message: e.to_string(),
                            code: "EXECUTION_FAILED".to_string(),
                            step_id: None,
                            details: None,
                        });
                        instance.completed_at = Some(chrono::Utc::now());
                    }
                }
            }
        });

        Ok(TriggerWorkflowResponse {
            instance_id,
            status_url: format!("/api/v1/workflows/status/{}", instance_id),
            status: WorkflowStatus::Created,
            workflow_name,
            created_at,
        })
    }
}

/// HTTP handler for triggering workflows
pub async fn trigger_workflow(
    service: web::Data<WorkflowService>,
    request: web::Json<TriggerWorkflowRequest>,
) -> ActixResult<HttpResponse> {
    log::info!(
        "Received workflow trigger request for: {}",
        request.workflow_name
    );

    match service.trigger_workflow(request.into_inner()).await {
        Ok(response) => {
            log::info!("Successfully triggered workflow: {}", response.instance_id);
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            log::error!("Failed to trigger workflow: {}", e);
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "workflow_trigger_failed",
                "message": e.to_string()
            })))
        }
    }
}

/// HTTP handler for getting workflow status
pub async fn get_workflow_status(
    service: web::Data<WorkflowService>,
    path: web::Path<Uuid>,
) -> ActixResult<HttpResponse> {
    let instance_id = path.into_inner();
    log::debug!(
        "Received status request for workflow instance: {}",
        instance_id
    );

    match service.get_workflow_status(instance_id).await {
        Ok(response) => {
            log::debug!(
                "Successfully retrieved status for workflow: {}",
                instance_id
            );
            Ok(HttpResponse::Ok().json(response))
        }
        Err(WorkflowError::InvalidInput(details)) => {
            log::warn!("Workflow instance not found: {}", details.message);
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "workflow_not_found",
                "message": details.message
            })))
        }
        Err(e) => {
            log::error!("Failed to get workflow status: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "status_retrieval_failed",
                "message": e.to_string()
            })))
        }
    }
}

/// HTTP handler for listing workflow instances
pub async fn list_workflow_instances(
    service: web::Data<WorkflowService>,
) -> ActixResult<HttpResponse> {
    let instances = service.list_instances().await;

    let response = serde_json::json!({
        "instances": instances.iter().map(|(id, status, name)| {
            serde_json::json!({
                "instance_id": id,
                "status": status,
                "workflow_name": name,
                "status_url": format!("/api/v1/workflows/status/{}", id)
            })
        }).collect::<Vec<_>>()
    });

    Ok(HttpResponse::Ok().json(response))
}

/// HTTP handler for listing available workflows
pub async fn list_available_workflows(
    service: web::Data<WorkflowService>,
) -> ActixResult<HttpResponse> {
    let workflows = service.list_workflows().await;

    let response = serde_json::json!({
        "workflows": workflows
    });

    Ok(HttpResponse::Ok().json(response))
}

/// Request payload for triggering a workflow from template
#[derive(Debug, Deserialize)]
pub struct TriggerTemplateRequest {
    /// Template ID to use
    pub template_id: String,

    /// Input data for the workflow
    pub inputs: Value,

    /// Optional workflow configuration overrides
    pub config: Option<WorkflowConfigOverrides>,
}

/// HTTP handler for listing workflow templates
pub async fn list_templates(service: web::Data<WorkflowService>) -> ActixResult<HttpResponse> {
    let templates = service.list_templates().await;

    let response = serde_json::json!({
        "templates": templates
    });

    Ok(HttpResponse::Ok().json(response))
}

/// HTTP handler for searching workflow templates
pub async fn search_templates(
    service: web::Data<WorkflowService>,
    query: web::Query<TemplateSearchCriteria>,
) -> ActixResult<HttpResponse> {
    let templates = service.search_templates(query.into_inner()).await;

    let response = serde_json::json!({
        "templates": templates
    });

    Ok(HttpResponse::Ok().json(response))
}

/// HTTP handler for getting a specific template
pub async fn get_template(
    service: web::Data<WorkflowService>,
    path: web::Path<String>,
) -> ActixResult<HttpResponse> {
    let template_id = path.into_inner();

    match service.get_template(&template_id).await {
        Some(template) => Ok(HttpResponse::Ok().json(template)),
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "template_not_found",
            "message": format!("Template '{}' not found", template_id)
        }))),
    }
}

/// HTTP handler for getting template categories
pub async fn get_template_categories(
    service: web::Data<WorkflowService>,
) -> ActixResult<HttpResponse> {
    let categories = service.get_template_categories().await;

    let response = serde_json::json!({
        "categories": categories
    });

    Ok(HttpResponse::Ok().json(response))
}

/// HTTP handler for getting template tags
pub async fn get_template_tags(service: web::Data<WorkflowService>) -> ActixResult<HttpResponse> {
    let tags = service.get_template_tags().await;

    let response = serde_json::json!({
        "tags": tags
    });

    Ok(HttpResponse::Ok().json(response))
}

/// HTTP handler for triggering workflow from template
pub async fn trigger_from_template(
    service: web::Data<WorkflowService>,
    request: web::Json<TriggerTemplateRequest>,
) -> ActixResult<HttpResponse> {
    log::info!(
        "Received template trigger request for: {}",
        request.template_id
    );

    match service
        .trigger_from_template(
            &request.template_id,
            request.inputs.clone(),
            request.config.clone(),
        )
        .await
    {
        Ok(response) => {
            log::info!(
                "Successfully triggered workflow from template: {}",
                response.instance_id
            );
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            log::error!("Failed to trigger workflow from template: {}", e);
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "template_trigger_failed",
                "message": e.to_string()
            })))
        }
    }
}

/// Configure workflow API routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/workflows")
            .route("/trigger", web::post().to(trigger_workflow))
            .route("/status/{instance_id}", web::get().to(get_workflow_status))
            .route("/instances", web::get().to(list_workflow_instances))
            .route("/available", web::get().to(list_available_workflows)),
    );

    cfg.service(
        web::scope("/api/v1/templates")
            .route("", web::get().to(list_templates))
            .route("/search", web::get().to(search_templates))
            .route("/{template_id}", web::get().to(get_template))
            .route("/categories", web::get().to(get_template_categories))
            .route("/tags", web::get().to(get_template_tags))
            .route("/trigger", web::post().to(trigger_from_template)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, test};

    #[tokio::test]
    async fn test_workflow_service_creation() {
        let service = WorkflowService::new().await;
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_list_workflows() {
        let service = WorkflowService::new().await.unwrap();
        let workflows = service.list_workflows().await;

        // Should have at least the built-in templates
        assert!(!workflows.is_empty());
        assert!(workflows.contains(&"research_to_documentation".to_string()));
    }

    #[tokio::test]
    async fn test_trigger_workflow_api() {
        let service = web::Data::new(WorkflowService::new().await.unwrap());

        let app = test::init_service(
            App::new()
                .app_data(service.clone())
                .service(web::resource("/trigger").route(web::post().to(trigger_workflow))),
        )
        .await;

        let request_payload = TriggerWorkflowRequest {
            workflow_name: "research_to_documentation".to_string(),
            inputs: serde_json::json!({
                "topic": "machine learning",
                "difficulty": "intermediate"
            }),
            config: None,
        };

        let req = test::TestRequest::post()
            .uri("/trigger")
            .set_json(&request_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
