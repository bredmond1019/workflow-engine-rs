use actix_web::{web, HttpRequest, HttpResponse, Result};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::middleware::auth::ClaimsExtractor;
use workflow_engine_core::registry::{AgentRegistry, AgentRegistration, AgentRegistryError};
use crate::db::agent::Agent;

/// Response for agent registration
#[derive(Serialize, Deserialize, Debug)]
pub struct AgentRegistrationResponse {
    pub id: Uuid,
    pub name: String,
    pub endpoint: String,
    pub capabilities: serde_json::Value,
    pub status: String,
}

/// Agent heartbeat request structure
#[derive(Serialize, Deserialize, Debug)]
pub struct HeartbeatRequest {
    pub metadata: Option<serde_json::Value>,
}

/// Agent heartbeat response structure
#[derive(Serialize, Deserialize, Debug)]
pub struct HeartbeatResponse {
    pub message: String,
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

/// Agent discovery response
#[derive(Serialize, Deserialize, Debug)]
pub struct AgentDiscoveryResponse {
    pub agents: Vec<Agent>,
    pub total_count: i64,
    pub page: i32,
    pub limit: i32,
}

/// Agent listing response
#[derive(Serialize, Deserialize, Debug)]
pub struct AgentListResponse {
    pub agents: Vec<Agent>,
    pub total_count: i64,
    pub active_count: i64,
}

/// Register a new agent in the registry
pub async fn register_agent(
    _req: HttpRequest,
    _registration: web::Json<AgentRegistration>,
    _pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse> {
    // Registry endpoint not implemented - requires PostgresAgentRegistry integration
    // Returns proper HTTP 501 Not Implemented status
    Ok(HttpResponse::NotImplemented().json(serde_json::json!({
        "error": "not_implemented",
        "message": "Agent registry functionality not yet implemented"
    })))
}

/// List all active agents
pub async fn list_agents(
    _req: HttpRequest,
    _pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse> {
    // Registry endpoint not implemented - requires PostgresAgentRegistry integration
    // Returns proper HTTP 501 Not Implemented status
    Ok(HttpResponse::NotImplemented().json(serde_json::json!({
        "error": "not_implemented",
        "message": "Agent listing functionality not yet implemented"
    })))
}

/// Discover agents with filtering capabilities
pub async fn discover_agents(
    _req: HttpRequest,
    _query: web::Query<serde_json::Value>,
    _pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse> {
    // Discovery endpoint not implemented - requires PostgresAgentRegistry integration
    // Returns proper HTTP 501 Not Implemented status
    Ok(HttpResponse::NotImplemented().json(serde_json::json!({
        "error": "not_implemented", 
        "message": "Agent discovery functionality not yet implemented"
    })))
}

/// Update agent heartbeat timestamp
pub async fn heartbeat_agent(
    _req: HttpRequest,
    _path: web::Path<Uuid>,
    _heartbeat: web::Json<HeartbeatRequest>,
    _pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse> {
    // Heartbeat endpoint not implemented - requires PostgresAgentRegistry integration
    // Returns proper HTTP 501 Not Implemented status
    Ok(HttpResponse::NotImplemented().json(serde_json::json!({
        "error": "not_implemented",
        "message": "Agent heartbeat functionality not yet implemented"
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};
    use actix_web::http::{header, StatusCode};
    use actix_web::middleware::Logger;

    fn create_test_token() -> String {
        "test_token".to_string()
    }

    #[actix_web::test]
    async fn test_register_agent() {
        let app = test::init_service(
            App::new()
                .wrap(Logger::default())
                .route("/registry/agents", web::post().to(register_agent))
        ).await;

        let registration = AgentRegistration {
            name: "test-agent".to_string(),
            endpoint: "http://localhost:8000".to_string(),
            capabilities: vec!["test".to_string()],
            metadata: serde_json::json!({}),
        };

        let req = test::TestRequest::post()
            .uri("/registry/agents")
            .insert_header((header::AUTHORIZATION, "Bearer test_token"))
            .set_json(&registration)
            .to_request();

        let resp = test::call_service(&app, req).await;
        
        // Should return 500 Internal Server Error without database pool
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[actix_web::test]
    async fn test_list_agents() {
        let app = test::init_service(
            App::new()
                .wrap(Logger::default())
                .route("/registry/agents", web::get().to(list_agents))
        ).await;

        let req = test::TestRequest::get()
            .uri("/registry/agents")
            .to_request();

        let resp = test::call_service(&app, req).await;
        
        // Should return 500 Internal Server Error without database pool
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[actix_web::test]
    async fn test_heartbeat_agent() {
        let app = test::init_service(
            App::new()
                .wrap(Logger::default())
                .route("/registry/agents/{id}/heartbeat", web::post().to(heartbeat_agent))
        ).await;

        let token = create_test_token();
        let agent_id = Uuid::new_v4();
        let heartbeat_req = HeartbeatRequest {
            metadata: Some(serde_json::json!({"status": "healthy"})),
        };

        let req = test::TestRequest::post()
            .uri(&format!("/registry/agents/{}/heartbeat", agent_id))
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .set_json(&heartbeat_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        
        // Should return 500 Internal Server Error without database pool
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}

/// Configure registry routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/registry")
            .route("/agents", web::post().to(register_agent))
            .route("/agents", web::get().to(list_agents))
            .route("/agents/discover", web::get().to(discover_agents))
            .route("/agents/{id}/heartbeat", web::post().to(heartbeat_agent))
    );
}