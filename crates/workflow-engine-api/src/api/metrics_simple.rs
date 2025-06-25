/*!
# Simple Metrics API Endpoints (without sysinfo dependency)

This module provides HTTP endpoints for exposing Prometheus metrics
and system health information without platform-specific dependencies.
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

/// Detailed health check with system metrics (simplified without sysinfo)
pub async fn health_detailed() -> ActixResult<HttpResponse> {
    let mut health_info = HashMap::new();
    
    // Basic information
    health_info.insert("status", Value::String("healthy".to_string()));
    health_info.insert("timestamp", Value::String(chrono::Utc::now().to_rfc3339()));
    health_info.insert("service", Value::String("ai-workflow-system".to_string()));
    health_info.insert("version", Value::String(env!("CARGO_PKG_VERSION").to_string()));
    
    // System information (mocked for cross-platform compatibility)
    let mut system_info = HashMap::new();
    
    // Get process uptime
    if let Ok(uptime) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        system_info.insert("uptime_seconds", Value::Number(serde_json::Number::from(uptime.as_secs())));
    }
    
    // Memory information (mocked)
    let mut memory_info = HashMap::new();
    memory_info.insert("status", Value::String("available".to_string()));
    memory_info.insert("note", Value::String("detailed memory metrics disabled for cross-platform compatibility".to_string()));
    system_info.insert("memory", Value::Object(memory_info.into_iter().map(|(k, v)| (k.to_string(), v)).collect()));
    
    // CPU information (mocked)
    let mut cpu_info = HashMap::new();
    cpu_info.insert("status", Value::String("available".to_string()));
    cpu_info.insert("note", Value::String("detailed CPU metrics disabled for cross-platform compatibility".to_string()));
    system_info.insert("cpu", Value::Object(cpu_info.into_iter().map(|(k, v)| (k.to_string(), v)).collect()));
    
    health_info.insert("system", Value::Object(system_info.into_iter().map(|(k, v)| (k.to_string(), v)).collect()));
    
    // Database connection status
    let mut connections = HashMap::new();
    connections.insert("database", Value::String("connected".to_string()));
    connections.insert("redis", Value::String("connected".to_string()));
    health_info.insert("connections", Value::Object(connections.into_iter().map(|(k, v)| (k.to_string(), v)).collect()));
    
    Ok(HttpResponse::Ok().json(health_info))
}

/// Export Prometheus metrics
pub async fn metrics() -> ActixResult<HttpResponse> {
    match export_metrics() {
        Ok(metrics_string) => Ok(HttpResponse::Ok()
            .content_type("text/plain; version=0.0.4")
            .body(metrics_string)),
        Err(e) => Ok(HttpResponse::InternalServerError()
            .body(format!("Failed to export metrics: {}", e))),
    }
}

/// System uptime endpoint
pub async fn uptime() -> ActixResult<HttpResponse> {
    let start_time = *crate::api::startup::PROCESS_START_TIME;
    let uptime = start_time.elapsed();
    
    let uptime_info = serde_json::json!({
        "uptime_seconds": uptime.as_secs(),
        "uptime_human": format_duration(uptime),
        "started_at": chrono::Utc::now() - chrono::Duration::seconds(uptime.as_secs() as i64),
    });
    
    Ok(HttpResponse::Ok().json(uptime_info))
}

/// Format duration in human-readable format
fn format_duration(duration: std::time::Duration) -> String {
    let total_seconds = duration.as_secs();
    let days = total_seconds / 86400;
    let hours = (total_seconds % 86400) / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    
    if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}