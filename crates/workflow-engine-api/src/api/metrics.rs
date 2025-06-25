/*!
# Metrics API Endpoints

This module provides HTTP endpoints for exposing Prometheus metrics
and system health information.

Task 3.7: Add metrics endpoint exposure for Prometheus scraping
*/

use actix_web::{web, HttpResponse, Result as ActixResult};
use serde_json::Value;
use std::collections::HashMap;

use crate::monitoring::metrics::{export_metrics, SystemMetrics};

/// Health check endpoint with basic system information
pub async fn health_check() -> ActixResult<HttpResponse> {
    let health_info = serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "service": "ai-workflow-system",
        "version": env!("CARGO_PKG_VERSION")
    });
    
    Ok(HttpResponse::Ok().json(health_info))
}

/// Detailed health check with system metrics
pub async fn health_detailed() -> ActixResult<HttpResponse> {
    let mut health_info = HashMap::new();
    
    // Basic information
    health_info.insert("status", Value::String("healthy".to_string()));
    health_info.insert("timestamp", Value::String(chrono::Utc::now().to_rfc3339()));
    health_info.insert("service", Value::String("ai-workflow-system".to_string()));
    health_info.insert("version", Value::String(env!("CARGO_PKG_VERSION").to_string()));
    
    // System information
    let mut system_info = HashMap::new();
    
    // Get system uptime
    if let Ok(uptime) = uptime_lib::get() {
        system_info.insert("uptime_seconds", Value::Number(serde_json::Number::from(uptime.as_secs())));
    }
    
    // Memory information (simplified example)
    #[cfg(target_os = "linux")]
    {
        {
            use sysinfo::System;
            let mut sys = System::new_all();
            sys.refresh_memory();
            let mut memory_info = HashMap::new();
            memory_info.insert("total_kb", Value::Number(serde_json::Number::from(sys.total_memory())));
            memory_info.insert("available_kb", Value::Number(serde_json::Number::from(sys.available_memory())));
            memory_info.insert("used_kb", Value::Number(serde_json::Number::from(sys.used_memory())));
            system_info.insert("memory", Value::Object(memory_info.into_iter().map(|(k, v)| (k.to_string(), v)).collect()));
        }
    }
    
    // CPU information
    #[cfg(target_os = "linux")]
    {
        {
            use sysinfo::System;
            let mut sys = System::new_all();
            let load_avg = System::load_average();
            let mut cpu_info = HashMap::new();
            cpu_info.insert("load_1m", Value::Number(serde_json::Number::from_f64(load_avg.one).unwrap_or(serde_json::Number::from(0))));
            cpu_info.insert("load_5m", Value::Number(serde_json::Number::from_f64(load_avg.five).unwrap_or(serde_json::Number::from(0))));
            cpu_info.insert("load_15m", Value::Number(serde_json::Number::from_f64(load_avg.fifteen).unwrap_or(serde_json::Number::from(0))));
            system_info.insert("cpu", Value::Object(cpu_info.into_iter().map(|(k, v)| (k.to_string(), v)).collect()));
        }
    }
    
    health_info.insert("system", Value::Object(system_info.into_iter().map(|(k, v)| (k.to_string(), v)).collect()));
    
    Ok(HttpResponse::Ok().json(health_info))
}

/// Prometheus metrics endpoint
pub async fn metrics() -> ActixResult<HttpResponse> {
    match export_metrics() {
        Ok(metrics_output) => {
            Ok(HttpResponse::Ok()
                .content_type("text/plain; version=0.0.4; charset=utf-8")
                .body(metrics_output))
        }
        Err(e) => {
            log::error!("Failed to export metrics: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "metrics_export_failed",
                "message": e.to_string()
            })))
        }
    }
}

/// System metrics summary for monitoring dashboards
pub async fn metrics_summary() -> ActixResult<HttpResponse> {
    // This would typically gather current metric values
    // For now, we'll return a summary structure
    let summary = serde_json::json!({
        "metrics": {
            "cross_system_calls": {
                "description": "Total cross-system calls made",
                "endpoint": "/metrics",
                "prometheus_name": "ai_workflow_cross_system_calls_total"
            },
            "workflow_executions": {
                "description": "Total workflow executions",
                "endpoint": "/metrics", 
                "prometheus_name": "ai_workflow_workflows_triggered_total"
            },
            "http_requests": {
                "description": "Total HTTP requests",
                "endpoint": "/metrics",
                "prometheus_name": "ai_workflow_api_http_requests_total"
            },
            "system_uptime": {
                "description": "System uptime in seconds",
                "endpoint": "/metrics",
                "prometheus_name": "ai_workflow_system_uptime_seconds"
            }
        },
        "endpoints": {
            "metrics": "/api/v1/metrics",
            "health": "/api/v1/health",
            "health_detailed": "/api/v1/health/detailed"
        }
    });
    
    Ok(HttpResponse::Ok().json(summary))
}

/// Readiness probe for Kubernetes
pub async fn ready() -> ActixResult<HttpResponse> {
    // Check if all required services are ready
    // For now, we'll do basic checks
    
    let mut ready = true;
    let mut checks = HashMap::new();
    
    // Check if metrics are working
    match export_metrics() {
        Ok(_) => {
            checks.insert("metrics", serde_json::json!({"status": "ok"}));
        }
        Err(e) => {
            ready = false;
            checks.insert("metrics", serde_json::json!({
                "status": "error", 
                "error": e.to_string()
            }));
        }
    }
    
    // Check if workflow service is available
    // This would typically check if the workflow engine is responsive
    checks.insert("workflow_engine", serde_json::json!({"status": "ok"}));
    
    // Check if cross-system client is available
    // This would typically check if we can reach the registry
    checks.insert("cross_system", serde_json::json!({"status": "ok"}));
    
    let response = serde_json::json!({
        "ready": ready,
        "checks": checks,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    if ready {
        Ok(HttpResponse::Ok().json(response))
    } else {
        Ok(HttpResponse::ServiceUnavailable().json(response))
    }
}

/// Liveness probe for Kubernetes
pub async fn live() -> ActixResult<HttpResponse> {
    // Basic liveness check - if we can respond, we're alive
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "alive": true,
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Configure monitoring API routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/health", web::get().to(health_check))
            .route("/health/detailed", web::get().to(health_detailed))
            .route("/metrics", web::get().to(metrics))
            .route("/metrics/summary", web::get().to(metrics_summary))
            .route("/ready", web::get().to(ready))
            .route("/live", web::get().to(live))
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};
    use crate::monitoring::metrics::init_metrics;

    #[actix_web::test]
    async fn test_health_check() {
        let app = test::init_service(
            App::new().route("/health", web::get().to(health_check))
        ).await;

        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["status"], "healthy");
        assert!(body["timestamp"].is_string());
    }

    #[actix_web::test]
    async fn test_metrics_endpoint() {
        // Initialize metrics first
        let _ = init_metrics();
        
        let app = test::init_service(
            App::new().route("/metrics", web::get().to(metrics))
        ).await;

        let req = test::TestRequest::get().uri("/metrics").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        
        let content_type = resp.headers().get("content-type").unwrap();
        assert!(content_type.to_str().unwrap().starts_with("text/plain"));
    }

    #[actix_web::test]
    async fn test_ready_endpoint() {
        let app = test::init_service(
            App::new().route("/ready", web::get().to(ready))
        ).await;

        let req = test::TestRequest::get().uri("/ready").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["ready"].is_boolean());
        assert!(body["checks"].is_object());
    }

    #[actix_web::test]
    async fn test_live_endpoint() {
        let app = test::init_service(
            App::new().route("/live", web::get().to(live))
        ).await;

        let req = test::TestRequest::get().uri("/live").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["alive"], true);
    }
}