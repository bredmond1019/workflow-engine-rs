use actix_web::{web, HttpResponse, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::api::uptime::{UptimeTracker, get_uptime_tracker};

/// Health check response structure
#[derive(Serialize, Deserialize, Debug)]
pub struct HealthResponse {
    /// Service status
    pub status: String,
    
    /// Current timestamp
    pub timestamp: String,
    
    /// Service version
    pub version: String,
    
    /// Service uptime in seconds (would be tracked in production)
    pub uptime_seconds: u64,
    
    /// Additional service information
    pub service_info: ServiceInfo,
}

/// Additional service information
#[derive(Serialize, Deserialize, Debug)]
pub struct ServiceInfo {
    /// Service name
    pub name: String,
    
    /// Service description
    pub description: String,
    
    /// Environment (dev, test, prod)
    pub environment: String,
    
    /// Available capabilities
    pub capabilities: Vec<String>,
}

/// Health check endpoint (GET /health)
/// This endpoint does not require authentication and provides basic service status
pub async fn health_check() -> Result<HttpResponse> {
    let uptime_seconds = get_uptime_tracker()
        .map(|tracker| tracker.uptime_seconds())
        .unwrap_or(0);
        
    let health_response = HealthResponse {
        status: "healthy".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds,
        service_info: ServiceInfo {
            name: "AI Workflow System".to_string(),
            description: "A robust AI system for building and managing AI workflows".to_string(),
            environment: std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
            capabilities: vec![
                "agent_registry".to_string(),
                "jwt_authentication".to_string(),
                "http_transport".to_string(),
                "mcp_protocol".to_string(),
                "workflow_management".to_string(),
            ],
        },
    };
    
    Ok(HttpResponse::Ok().json(health_response))
}

/// Detailed health check endpoint (GET /health/detailed)
/// This endpoint provides more detailed health information
/// This endpoint does not require authentication but might in production
pub async fn detailed_health_check(
    db_pool: Option<web::Data<crate::db::session::DbPool>>,
) -> Result<HttpResponse> {
    use std::process;
    use sysinfo::{System, ProcessesToUpdate};
    
    let pid = process::id();
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::All, true);
    
    let memory_usage = if let Some(process) = system.process(sysinfo::Pid::from(pid as usize)) {
        process.memory()
    } else {
        0
    };
    
    let uptime_info = get_uptime_tracker()
        .map(|tracker| tracker.get_uptime_info())
        .unwrap_or_else(|| crate::api::uptime::UptimeInfo {
            uptime_seconds: 0,
            uptime_duration: "unknown".to_string(),
            start_timestamp: Utc::now(),
            restart_count: 0,
        });
    
    let detailed_response = serde_json::json!({
        "status": "healthy",
        "timestamp": Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
        "uptime": uptime_info,
        "service_info": {
            "name": "AI Workflow System",
            "description": "A robust AI system for building and managing AI workflows",
            "environment": std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
            "capabilities": [
                "agent_registry",
                "jwt_authentication", 
                "http_transport",
                "mcp_protocol",
                "workflow_management"
            ]
        },
        "system_info": {
            "process_id": pid,
            "memory_usage_kb": memory_usage,
            "rust_version": "stable",
            "build_target": format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS),
        },
        "dependencies": {
            "database": check_database_health(db_pool),
            "external_services": {
                "status": "not_checked"
            }
        }
    });
    
    Ok(HttpResponse::Ok().json(detailed_response))
}

/// Check database connectivity and health
fn check_database_health(
    db_pool: Option<web::Data<crate::db::session::DbPool>>,
) -> serde_json::Value {
    match db_pool {
        Some(pool) => {
            match pool.get() {
                Ok(mut conn) => {
                    // Try a simple query to test database connectivity
                    use diesel::sql_query;
                    use diesel::RunQueryDsl;
                    
                    match sql_query("SELECT 1 as test").execute(&mut conn) {
                        Ok(_) => {
                            serde_json::json!({
                                "status": "healthy",
                                "connection_pool": "available",
                                "last_checked": chrono::Utc::now().to_rfc3339(),
                                "pool_state": {
                                    "size": pool.state().connections,
                                    "idle": pool.state().idle_connections,
                                    "max_size": pool.max_size()
                                }
                            })
                        },
                        Err(e) => {
                            serde_json::json!({
                                "status": "unhealthy",
                                "connection_pool": "available_but_query_failed",
                                "error": e.to_string(),
                                "last_checked": chrono::Utc::now().to_rfc3339(),
                                "pool_state": {
                                    "size": pool.state().connections,
                                    "idle": pool.state().idle_connections,
                                    "max_size": pool.max_size()
                                }
                            })
                        }
                    }
                },
                Err(e) => {
                    serde_json::json!({
                        "status": "unhealthy",
                        "connection_pool": "unavailable",
                        "error": e.to_string(),
                        "last_checked": chrono::Utc::now().to_rfc3339(),
                        "pool_state": {
                            "size": pool.state().connections,
                            "idle": pool.state().idle_connections,
                            "max_size": pool.max_size()
                        }
                    })
                }
            }
        },
        None => {
            serde_json::json!({
                "status": "unknown",
                "connection_pool": "not_configured",
                "message": "Database pool not available in application data",
                "last_checked": chrono::Utc::now().to_rfc3339()
            })
        }
    }
}

/// Configure health check routes
pub fn configure_health_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/health")
            .route(web::get().to(health_check))
    )
    .service(
        web::resource("/health/detailed")
            .route(web::get().to(detailed_health_check))
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_health_check_endpoint() {
        let app = test::init_service(
            App::new().configure(configure_health_routes)
        ).await;

        let req = test::TestRequest::get()
            .uri("/health")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: HealthResponse = test::read_body_json(resp).await;
        assert_eq!(body.status, "healthy");
        assert_eq!(body.service_info.name, "AI Workflow System");
        assert!(body.service_info.capabilities.contains(&"agent_registry".to_string()));
        assert!(body.service_info.capabilities.contains(&"jwt_authentication".to_string()));
        assert!(body.service_info.capabilities.contains(&"http_transport".to_string()));
    }

    #[actix_web::test]
    async fn test_detailed_health_check_endpoint() {
        let app = test::init_service(
            App::new().configure(configure_health_routes)
        ).await;

        let req = test::TestRequest::get()
            .uri("/health/detailed")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["status"], "healthy");
        assert!(body["system_info"].is_object());
        assert!(body["dependencies"].is_object());
        assert!(body["service_info"]["capabilities"].is_array());
    }

    #[actix_web::test]
    async fn test_health_check_no_auth_required() {
        // Verify that health check endpoint works without any authentication headers
        let app = test::init_service(
            App::new().configure(configure_health_routes)
        ).await;

        let req = test::TestRequest::get()
            .uri("/health")
            // Intentionally not adding any Authorization header
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body: HealthResponse = test::read_body_json(resp).await;
        assert_eq!(body.status, "healthy");
    }
}