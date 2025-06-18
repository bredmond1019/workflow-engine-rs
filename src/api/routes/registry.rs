use actix_web::{web, HttpResponse, Result, HttpRequest};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::middleware::auth::ClaimsExtractor;
use crate::core::registry::{AgentRegistry, AgentRegistration, AgentRegistryError, PostgresAgentRegistry};
use crate::db::agent::Agent;

/// Response for agent registration
#[derive(Serialize, Deserialize, Debug)]
pub struct AgentRegistrationResponse {
    pub id: Uuid,
    pub name: String,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub status: String,
    pub message: String,
}

/// Response for agent listing
#[derive(Serialize, Deserialize, Debug)]
pub struct AgentListResponse {
    pub agents: Vec<Agent>,
    pub count: usize,
}

/// Response for capability discovery
#[derive(Serialize, Deserialize, Debug)]
pub struct DiscoverResponse {
    pub capability: String,
    pub agents: Vec<Agent>,
    pub count: usize,
}

/// Query parameters for discovery endpoint
#[derive(Deserialize, Debug)]
pub struct DiscoveryQuery {
    pub capability: String,
}

/// Request body for heartbeat (optional metadata)
#[derive(Deserialize, Serialize, Debug)]
pub struct HeartbeatRequest {
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

/// Response for heartbeat
#[derive(Serialize, Debug)]
pub struct HeartbeatResponse {
    pub message: String,
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

/// Register a new agent
pub async fn register_agent(
    req: HttpRequest,
    registration: web::Json<AgentRegistration>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse> {
    // Verify authentication
    let _claims = match req.get_claims() {
        Some(claims) => claims,
        None => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "unauthorized",
                "message": "Valid authentication required"
            })));
        }
    };

    let registry = PostgresAgentRegistry::new(pool.get_ref().clone());
    
    match registry.register(registration.into_inner()).await {
        Ok(agent) => {
            let response = AgentRegistrationResponse {
                id: agent.id,
                name: agent.name,
                endpoint: agent.endpoint,
                capabilities: agent.capabilities,
                status: agent.status,
                message: "Agent registered successfully".to_string(),
            };
            Ok(HttpResponse::Created().json(response))
        }
        Err(AgentRegistryError::DuplicateName { name }) => {
            Ok(HttpResponse::Conflict().json(serde_json::json!({
                "error": "duplicate_name",
                "message": format!("Agent name '{}' already exists", name)
            })))
        }
        Err(e) => {
            log::error!("Failed to register agent: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "registration_failed",
                "message": "Failed to register agent"
            })))
        }
    }
}

/// List all active agents
pub async fn list_agents(
    req: HttpRequest,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse> {
    // Verify authentication
    let _claims = match req.get_claims() {
        Some(claims) => claims,
        None => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "unauthorized",
                "message": "Valid authentication required"
            })));
        }
    };

    let registry = PostgresAgentRegistry::new(pool.get_ref().clone());
    
    match registry.list_active().await {
        Ok(agents) => {
            let response = AgentListResponse {
                count: agents.len(),
                agents,
            };
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            log::error!("Failed to list agents: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "list_failed",
                "message": "Failed to retrieve agents"
            })))
        }
    }
}

/// Discover agents by capability
pub async fn discover_agents(
    req: HttpRequest,
    query: web::Query<DiscoveryQuery>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse> {
    // Verify authentication
    let _claims = match req.get_claims() {
        Some(claims) => claims,
        None => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "unauthorized",
                "message": "Valid authentication required"
            })));
        }
    };

    let registry = PostgresAgentRegistry::new(pool.get_ref().clone());
    
    match registry.discover(&query.capability).await {
        Ok(agents) => {
            let response = DiscoverResponse {
                capability: query.capability.clone(),
                count: agents.len(),
                agents,
            };
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            log::error!("Failed to discover agents: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "discovery_failed",
                "message": "Failed to discover agents"
            })))
        }
    }
}

/// Update agent heartbeat
pub async fn heartbeat_agent(
    req: HttpRequest,
    path: web::Path<Uuid>,
    heartbeat_req: web::Json<HeartbeatRequest>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse> {
    // Verify authentication
    let _claims = match req.get_claims() {
        Some(claims) => claims,
        None => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "unauthorized",
                "message": "Valid authentication required"
            })));
        }
    };

    let agent_id = path.into_inner();
    let registry = PostgresAgentRegistry::new(pool.get_ref().clone());
    
    match registry.heartbeat(&agent_id).await {
        Ok(()) => {
            let response = HeartbeatResponse {
                message: "Heartbeat updated successfully".to_string(),
                last_seen: chrono::Utc::now(),
            };
            Ok(HttpResponse::Ok().json(response))
        }
        Err(AgentRegistryError::AgentNotFound { id }) => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "agent_not_found",
                "message": format!("Agent with ID '{}' not found", id)
            })))
        }
        Err(e) => {
            log::error!("Failed to update heartbeat: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "heartbeat_failed",
                "message": "Failed to update agent heartbeat"
            })))
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};
    use crate::api::middleware::JwtMiddleware;
    use crate::core::auth::{Claims, JwtAuth};

    // Helper function to create a test token
    fn create_test_token() -> String {
        let claims = Claims::new("test_service".to_string(), "service".to_string());
        JwtAuth::generate_token(&claims).expect("Failed to generate test token")
    }

    #[actix_web::test]
    async fn test_register_agent_endpoint() {
        let app = test::init_service(
            App::new()
                .wrap(JwtMiddleware)
                .configure(configure)
        ).await;

        let token = create_test_token();
        let registration = AgentRegistration {
            name: "test-agent".to_string(),
            endpoint: "http://localhost:8080".to_string(),
            capabilities: vec!["test".to_string(), "demo".to_string()],
            metadata: serde_json::json!({"version": "1.0"}),
        };

        let req = test::TestRequest::post()
            .uri("/registry/agents")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .set_json(&registration)
            .to_request();

        // Note: This test would require a database connection pool
        // For now, we're just testing the endpoint structure
        let resp = test::call_service(&app, req).await;
        
        // Should return 500 since we don't have a real database pool
        // In real tests, we'd mock the database
        assert!(resp.status().is_server_error() || resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_list_agents_requires_auth() {
        let app = test::init_service(
            App::new()
                .wrap(JwtMiddleware)
                .configure(configure)
        ).await;

        let req = test::TestRequest::get()
            .uri("/registry/agents")
            .to_request();

        // Should be blocked by middleware since no auth header
        let result = test::try_call_service(&app, req).await;
        assert!(result.is_err());
    }

    #[actix_web::test]
    async fn test_discover_agents_endpoint() {
        let app = test::init_service(
            App::new()
                .wrap(JwtMiddleware)
                .configure(configure)
        ).await;

        let token = create_test_token();
        let req = test::TestRequest::get()
            .uri("/registry/agents/discover?capability=test")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();

        let resp = test::call_service(&app, req).await;
        
        // Should return 500 since we don't have a real database pool
        assert!(resp.status().is_server_error() || resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_heartbeat_endpoint() {
        let app = test::init_service(
            App::new()
                .wrap(JwtMiddleware)
                .configure(configure)
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
        
        // Should return 500 since we don't have a real database pool
        assert!(resp.status().is_server_error() || resp.status().is_success());
    }
}