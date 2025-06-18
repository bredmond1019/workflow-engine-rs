use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// Workflow types
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TriggerWorkflowRequest {
    pub workflow_name: String,
    pub inputs: serde_json::Value,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TriggerWorkflowResponse {
    pub instance_id: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WorkflowStatusResponse {
    pub instance_id: String,
    pub workflow_name: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub outputs: Option<serde_json::Value>,
    pub error: Option<String>,
}

// Template types
#[derive(Debug, Serialize, ToSchema)]
pub struct WorkflowTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub tags: Vec<String>,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct TemplateSearchParams {
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub keyword: Option<String>,
}

// Agent Registry types
#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterAgentRequest {
    pub name: String,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RegisterAgentResponse {
    pub id: String,
    pub name: String,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub status: String,
    pub registered_at: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub status: String,
    pub last_heartbeat: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthCheckResponse {
    pub status: String,
    pub message: Option<String>,
}