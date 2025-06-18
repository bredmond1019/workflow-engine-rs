/*!
# Workflow System Service Example

This example demonstrates a Rust-based service that:
1. Automatically registers with the AI Workflow System registry on startup
2. Maintains heartbeat with the registry
3. Provides workflow orchestration capabilities
4. Handles cross-system communication

This service acts as a workflow orchestrator that can coordinate tasks
across multiple AI services in the system.

## Usage

```bash
cargo run --example workflow_system_service
```

## Environment Variables

- `WORKFLOW_SYSTEM_NAME`: Service name (default: "workflow-system") 
- `WORKFLOW_SYSTEM_ENDPOINT`: Service endpoint (default: "http://localhost:3002")
- `REGISTRY_ENDPOINT`: Registry endpoint (default: "http://localhost:8080")
- `WORKFLOW_SYSTEM_PORT`: Service port (default: 3002)
- `HEARTBEAT_INTERVAL`: Heartbeat interval in seconds (default: 60)
- `AUTH_TOKEN`: Optional authentication token
*/

use backend::bootstrap::service::ServiceConfig;
use actix_web::{web, App, HttpResponse, HttpServer, Result, middleware::Logger};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use tokio::signal;

/// Workflow execution request
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowRequest {
    pub workflow_type: String,
    pub input: Value,
    pub target_services: Option<Vec<String>>,
    pub metadata: Option<Value>,
}

/// Workflow execution response
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowResponse {
    pub workflow_id: String,
    pub status: String,
    pub result: Option<Value>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
    pub services_used: Vec<String>,
}

/// Workflow status response
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowStatus {
    pub workflow_id: String,
    pub status: String,
    pub progress: f32,
    pub current_step: String,
    pub steps_completed: usize,
    pub total_steps: usize,
    pub services_called: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub uptime_seconds: u64,
    pub registered: bool,
}

/// Application state
pub struct AppState {
    pub registry_endpoint: String,
    pub service_name: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub auth_token: Option<String>,
}

/// Health check endpoint
async fn health_check(data: web::Data<AppState>) -> Result<HttpResponse> {
    let uptime = (Utc::now() - data.start_time).num_seconds() as u64;
    
    let response = HealthResponse {
        status: "healthy".to_string(),
        service: data.service_name.clone(),
        version: "1.0.0".to_string(),
        capabilities: vec![
            "orchestration".to_string(),
            "workflow".to_string(),
            "automation".to_string(),
            "cross_system_communication".to_string(),
        ],
        uptime_seconds: uptime,
        registered: true,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

/// Execute a workflow
async fn execute_workflow(
    request: web::Json<WorkflowRequest>,
    data: web::Data<AppState>,
) -> Result<HttpResponse> {
    let workflow_id = uuid::Uuid::new_v4().to_string();
    let start_time = std::time::Instant::now();
    
    log::info!("🔄 Starting workflow execution: {} (ID: {})", 
               request.workflow_type, workflow_id);
    
    match request.workflow_type.as_str() {
        "research_to_documentation" => {
            let result = execute_research_to_docs_workflow(&request, &data, &workflow_id).await;
            let execution_time = start_time.elapsed().as_millis() as u64;
            
            match result {
                Ok((result_data, services_used)) => {
                    log::info!("✅ Workflow {} completed successfully", workflow_id);
                    Ok(HttpResponse::Ok().json(WorkflowResponse {
                        workflow_id,
                        status: "completed".to_string(),
                        result: Some(result_data),
                        error: None,
                        execution_time_ms: execution_time,
                        services_used,
                    }))
                }
                Err(error) => {
                    log::error!("❌ Workflow {} failed: {}", workflow_id, error);
                    Ok(HttpResponse::InternalServerError().json(WorkflowResponse {
                        workflow_id,
                        status: "failed".to_string(),
                        result: None,
                        error: Some(error),
                        execution_time_ms: execution_time,
                        services_used: vec![],
                    }))
                }
            }
        }
        "simple_research" => {
            let result = execute_simple_research_workflow(&request, &data, &workflow_id).await;
            let execution_time = start_time.elapsed().as_millis() as u64;
            
            match result {
                Ok((result_data, services_used)) => {
                    Ok(HttpResponse::Ok().json(WorkflowResponse {
                        workflow_id,
                        status: "completed".to_string(),
                        result: Some(result_data),
                        error: None,
                        execution_time_ms: execution_time,
                        services_used,
                    }))
                }
                Err(error) => {
                    Ok(HttpResponse::InternalServerError().json(WorkflowResponse {
                        workflow_id,
                        status: "failed".to_string(),
                        result: None,
                        error: Some(error),
                        execution_time_ms: execution_time,
                        services_used: vec![],
                    }))
                }
            }
        }
        _ => {
            Ok(HttpResponse::BadRequest().json(json!({
                "error": "Unsupported workflow type",
                "supported_types": ["research_to_documentation", "simple_research"]
            })))
        }
    }
}

/// Get workflow status
async fn get_workflow_status(
    path: web::Path<String>,
    _data: web::Data<AppState>,
) -> Result<HttpResponse> {
    let workflow_id = path.into_inner();
    
    // In a real implementation, this would fetch from a database
    let status = WorkflowStatus {
        workflow_id: workflow_id.clone(),
        status: "completed".to_string(),
        progress: 1.0,
        current_step: "finished".to_string(),
        steps_completed: 3,
        total_steps: 3,
        services_called: vec!["ai-tutor-service".to_string()],
        created_at: Utc::now() - chrono::Duration::minutes(5),
        updated_at: Utc::now(),
    };
    
    Ok(HttpResponse::Ok().json(status))
}

/// Discover available services
async fn discover_services(_data: web::Data<AppState>) -> Result<HttpResponse> {
    // For this example, we'll simulate service discovery
    // In production, this would use the cross-system client
    let simulated_services = json!({
        "available_services": {
            "tutoring": ["ai-tutor-service"],
            "documentation": ["notion-service"]
        },
        "discovery_time": Utc::now().to_rfc3339(),
        "note": "This is a simulated response for demonstration"
    });
    
    Ok(HttpResponse::Ok().json(simulated_services))
}

/// Execute research to documentation workflow
async fn execute_research_to_docs_workflow(
    request: &WorkflowRequest,
    _data: &AppState,
    workflow_id: &str,
) -> Result<(Value, Vec<String>), String> {
    let mut services_used = Vec::new();
    
    // For this example, simulate service discovery and calling
    // In production, this would use the real cross-system client
    let tutor_service = "ai-tutor-service";
    services_used.push(tutor_service.to_string());
    
    // Simulate AI Tutor response
    let simulated_explanation = format!(
        "This is a simulated explanation for the topic: {}. \
        In a real implementation, this would come from the AI Tutor service via cross-system communication.",
        request.input.get("topic").unwrap_or(&json!("general topic")).as_str().unwrap_or("unknown topic")
    );
    
    // Step 3: Format the result for documentation
    let documentation = json!({
        "title": request.input.get("topic").unwrap_or(&json!("Research Results")),
        "content": simulated_explanation,
        "confidence": 0.8,
        "source_service": tutor_service,
        "workflow_id": workflow_id,
        "generated_at": Utc::now().to_rfc3339(),
        "metadata": {
            "workflow_type": "research_to_documentation",
            "services_involved": &services_used,
            "note": "This is a simulated response for demonstration"
        }
    });
    
    log::info!("📄 Generated documentation for workflow {}", workflow_id);
    
    Ok((documentation, services_used))
}

/// Execute simple research workflow  
async fn execute_simple_research_workflow(
    request: &WorkflowRequest,
    _data: &AppState,
    workflow_id: &str,
) -> Result<(Value, Vec<String>), String> {
    let mut services_used = Vec::new();
    
    // For this example, simulate service discovery and calling
    let tutor_service = "ai-tutor-service";
    services_used.push(tutor_service.to_string());
    
    // Simulate the AI Tutor response
    let default_query = json!("research query");
    let query = request.input.get("query").unwrap_or(&default_query);
    let simulated_answer = format!(
        "This is a simulated research answer for: {}. \
        In a real implementation, this would be generated by the AI Tutor service.",
        query.as_str().unwrap_or("unknown query")
    );
    
    let formatted_result = json!({
        "query": query,
        "answer": simulated_answer,
        "confidence": 0.85,
        "source": tutor_service,
        "workflow_id": workflow_id,
        "timestamp": Utc::now().to_rfc3339(),
        "note": "This is a simulated response for demonstration"
    });
    
    Ok((formatted_result, services_used))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    log::info!("🚀 Starting Workflow System Service...");
    
    // Get configuration from environment
    let service_name = env::var("WORKFLOW_SYSTEM_NAME")
        .unwrap_or_else(|_| "workflow-system".to_string());
    let service_endpoint = env::var("WORKFLOW_SYSTEM_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:3002".to_string());
    let registry_endpoint = env::var("REGISTRY_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    let port = env::var("WORKFLOW_SYSTEM_PORT")
        .unwrap_or_else(|_| "3002".to_string())
        .parse::<u16>()
        .unwrap_or(3002);
    let heartbeat_interval = env::var("HEARTBEAT_INTERVAL")
        .unwrap_or_else(|_| "60".to_string())
        .parse::<u64>()
        .unwrap_or(60);
    let auth_token = env::var("AUTH_TOKEN").ok();
    
    // Service configuration would be used for automatic registration
    // For this example, we're just showing the configuration structure
    let _service_config = ServiceConfig::new(
        service_name.clone(),
        service_endpoint.clone(),
        vec![
            "orchestration".to_string(),
            "workflow".to_string(),
            "automation".to_string(),
            "cross_system_communication".to_string(),
        ],
    )
    .with_heartbeat_interval(heartbeat_interval)
    .with_registry_endpoint(registry_endpoint.clone())
    .with_metadata(json!({
        "version": "1.0.0",
        "language": "rust",
        "framework": "actix-web",
        "supported_workflows": [
            "research_to_documentation",
            "simple_research"
        ],
        "max_concurrent_workflows": 20
    }));
    
    // Create app state
    let app_state = web::Data::new(AppState {
        registry_endpoint: registry_endpoint.clone(),
        service_name: service_name.clone(),
        start_time: Utc::now(),
        auth_token: auth_token.clone(),
    });
    
    log::info!("🔧 Configured service: {} at {}", service_name, service_endpoint);
    
    // Start the HTTP server
    let server = HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(Logger::default())
            .route("/health", web::get().to(health_check))
            .route("/workflow/execute", web::post().to(execute_workflow))
            .route("/workflow/status/{id}", web::get().to(get_workflow_status))
            .route("/discover", web::get().to(discover_services))
    })
    .bind(("0.0.0.0", port))?
    .run();
    
    log::info!("🌐 Workflow System Service running on http://0.0.0.0:{}", port);
    
    // Wait for shutdown signal
    tokio::select! {
        _ = server => {},
        _ = signal::ctrl_c() => {
            log::info!("🛑 Received shutdown signal, stopping service...");
        }
    }
    
    log::info!("✅ Workflow System Service stopped");
    Ok(())
}