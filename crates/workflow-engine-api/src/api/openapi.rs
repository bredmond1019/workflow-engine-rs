use utoipa::{OpenApi, ToSchema};
use utoipa::openapi::security::{SecurityScheme, HttpBuilder, HttpAuthScheme};

use crate::{
    api::{
        login::{LoginRequest, LoginResponse},
        health::{HealthStatus, DetailedHealthStatus, HealthChecks, ComponentHealth, MCPServerHealth, SystemInfo, MemoryInfo, DiskInfo, ProcessInfo},
        openapi_types::{
            TriggerWorkflowRequest, TriggerWorkflowResponse, WorkflowStatusResponse,
            WorkflowTemplate, TemplateSearchParams,
            RegisterAgentRequest, RegisterAgentResponse, AgentInfo, HealthCheckResponse,
        },
    },
};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::login::login,
        crate::api::health::health_check,
        crate::api::health::detailed_health_check,
    ),
    components(
        schemas(
            LoginRequest,
            LoginResponse,
            HealthStatus,
            DetailedHealthStatus,
            HealthChecks,
            ComponentHealth,
            MCPServerHealth,
            SystemInfo,
            MemoryInfo,
            DiskInfo,
            ProcessInfo,
            TriggerWorkflowRequest,
            TriggerWorkflowResponse,
            WorkflowStatusResponse,
            WorkflowTemplate,
            TemplateSearchParams,
            RegisterAgentRequest,
            RegisterAgentResponse,
            AgentInfo,
            HealthCheckResponse,
            ErrorResponse,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Authentication", description = "Authentication endpoints"),
        (name = "Health", description = "System health monitoring endpoints"),
        (name = "Workflows", description = "Workflow management and execution"),
        (name = "Templates", description = "Workflow template management"),
        (name = "Agent Registry", description = "Agent registration and discovery"),
        (name = "Metrics", description = "System metrics and monitoring"),
    ),
    info(
        title = "AI Workflow System API",
        version = "0.1.0",
        description = "A comprehensive system for managing AI agents, executing workflows, and monitoring system health",
        contact(
            name = "AI Workflow Team",
            email = "support@aiworkflow.example.com",
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "http://localhost:8080", description = "Local development server"),
        (url = "https://api.aiworkflow.example.com", description = "Production server"),
    ),
    external_docs(
        url = "https://github.com/your-org/ai-workflow-system",
        description = "GitHub repository"
    )
)]
pub struct ApiDoc;

#[derive(ToSchema)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
    pub request_id: String,
    pub timestamp: String,
}

#[derive(ToSchema)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build()
                ),
            );
        }
    }
}

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    use utoipa_swagger_ui::SwaggerUi;
    
    cfg.service(
        SwaggerUi::new("/swagger-ui/{_:.*}")
            .url("/api-docs/openapi.json", ApiDoc::openapi())
    );
}