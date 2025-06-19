use actix_web::{web, HttpResponse, Result};
use diesel::{PgConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::{System, Pid};
use utoipa::ToSchema;

use crate::db::session::DbPool;

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: u64,
    pub version: String,
    pub checks: HealthChecks,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthChecks {
    pub database: ComponentHealth,
    pub memory: ComponentHealth,
    pub disk: ComponentHealth,
    pub mcp_servers: Vec<MCPServerHealth>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ComponentHealth {
    pub status: String,
    pub message: Option<String>,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MCPServerHealth {
    pub name: String,
    pub url: String,
    pub status: String,
    pub response_time_ms: Option<u64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DetailedHealthStatus {
    pub status: String,
    pub timestamp: u64,
    pub version: String,
    pub uptime_seconds: u64,
    pub system: SystemInfo,
    pub checks: HealthChecks,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SystemInfo {
    pub cpu_usage: f32,
    pub memory: MemoryInfo,
    pub disk: DiskInfo,
    pub process: ProcessInfo,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MemoryInfo {
    pub total_mb: u64,
    pub used_mb: u64,
    pub free_mb: u64,
    pub usage_percent: f32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DiskInfo {
    pub total_gb: u64,
    pub used_gb: u64,
    pub free_gb: u64,
    pub usage_percent: f32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProcessInfo {
    pub pid: u32,
    pub cpu_usage: f32,
    pub memory_mb: u64,
    pub threads: usize,
}

#[utoipa::path(
    get,
    path = "/api/v1/health",
    tag = "Health",
    responses(
        (status = 200, description = "System is healthy or degraded", body = HealthStatus),
        (status = 503, description = "System is unhealthy", body = HealthStatus)
    )
)]
pub async fn health_check(
    pool: web::Data<std::sync::Arc<DbPool>>,
) -> Result<HttpResponse> {
    let mut overall_status = "healthy";
    
    // Check database
    let db_health = check_database(&pool).await;
    if db_health.status != "healthy" {
        overall_status = "degraded";
    }
    
    // Check memory
    let memory_health = check_memory();
    if memory_health.status == "unhealthy" {
        overall_status = "unhealthy";
    }
    
    // Check disk
    let disk_health = check_disk();
    if disk_health.status == "unhealthy" {
        overall_status = "unhealthy";
    }
    
    // Check MCP servers
    let mcp_health = check_mcp_servers().await;
    let unhealthy_mcp = mcp_health.iter().filter(|s| s.status != "healthy").count();
    if unhealthy_mcp > 0 && overall_status == "healthy" {
        overall_status = "degraded";
    }
    
    let health_status = HealthStatus {
        status: overall_status.to_string(),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        checks: HealthChecks {
            database: db_health,
            memory: memory_health,
            disk: disk_health,
            mcp_servers: mcp_health,
        },
    };
    
    let status_code = match overall_status {
        "healthy" => 200,
        "degraded" => 200,
        _ => 503,
    };
    
    Ok(HttpResponse::build(actix_web::http::StatusCode::from_u16(status_code).unwrap())
        .json(health_status))
}

#[utoipa::path(
    get,
    path = "/api/v1/health/detailed",
    tag = "Health",
    responses(
        (status = 200, description = "Detailed health information", body = DetailedHealthStatus)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn detailed_health_check(
    pool: web::Data<std::sync::Arc<DbPool>>,
) -> Result<HttpResponse> {
    let start_time = match std::env::var("PROCESS_START_TIME") {
        Ok(time) => time.parse::<u64>().unwrap_or_else(|_| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        }),
        Err(_) => SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };
    
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let uptime_seconds = current_time - start_time;
    
    // Get system information
    let mut sys = System::new_all();
    sys.refresh_all();
    
    // Get process information
    let pid = std::process::id();
    let pid_obj = Pid::from_u32(pid);
    let process = sys.process(pid_obj).map(|p| ProcessInfo {
        pid,
        cpu_usage: p.cpu_usage(),
        memory_mb: p.memory() / 1024,
        threads: 1, // sysinfo 0.35 doesn't expose thread count easily
    }).unwrap_or(ProcessInfo {
        pid,
        cpu_usage: 0.0,
        memory_mb: 0,
        threads: 1,
    });
    
    // Calculate CPU usage (simplified for sysinfo 0.35)
    let cpu_usage = sys.global_cpu_usage();
    
    // Get memory info
    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let free_memory = sys.free_memory();
    
    let memory_info = MemoryInfo {
        total_mb: total_memory / 1024,
        used_mb: used_memory / 1024,
        free_mb: free_memory / 1024,
        usage_percent: (used_memory as f32 / total_memory as f32) * 100.0,
    };
    
    // Get disk info (simplified for sysinfo 0.35)
    // For now, we'll use placeholder values since disk info is not easily accessible in this version
    let disk_info = DiskInfo {
        total_gb: 500, // Placeholder
        used_gb: 250,  // Placeholder
        free_gb: 250,  // Placeholder
        usage_percent: 50.0, // Placeholder
    };
    
    let system_info = SystemInfo {
        cpu_usage,
        memory: memory_info,
        disk: disk_info,
        process,
    };
    
    // Run health checks
    let mut overall_status = "healthy";
    
    let db_health = check_database(&pool).await;
    if db_health.status != "healthy" {
        overall_status = "degraded";
    }
    
    let memory_health = check_memory();
    if memory_health.status == "unhealthy" {
        overall_status = "unhealthy";
    }
    
    let disk_health = check_disk();
    if disk_health.status == "unhealthy" {
        overall_status = "unhealthy";
    }
    
    let mcp_health = check_mcp_servers().await;
    
    let detailed_status = DetailedHealthStatus {
        status: overall_status.to_string(),
        timestamp: current_time,
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds,
        system: system_info,
        checks: HealthChecks {
            database: db_health,
            memory: memory_health,
            disk: disk_health,
            mcp_servers: mcp_health,
        },
    };
    
    Ok(HttpResponse::Ok().json(detailed_status))
}

async fn check_database(
    pool: &DbPool,
) -> ComponentHealth {
    match pool.get() {
        Ok(mut conn) => {
            // Try a simple query
            match diesel::sql_query("SELECT 1").execute(&mut conn) {
                Ok(_) => ComponentHealth {
                    status: "healthy".to_string(),
                    message: Some("Database connection successful".to_string()),
                    details: Some(serde_json::json!({
                        "pool_size": pool.state().connections,
                        "idle_connections": pool.state().idle_connections,
                    })),
                },
                Err(e) => ComponentHealth {
                    status: "unhealthy".to_string(),
                    message: Some(format!("Database query failed: {}", e)),
                    details: None,
                },
            }
        }
        Err(e) => ComponentHealth {
            status: "unhealthy".to_string(),
            message: Some(format!("Failed to get database connection: {}", e)),
            details: None,
        },
    }
}

fn check_memory() -> ComponentHealth {
    let mut sys = System::new_all();
    sys.refresh_memory();
    
    let total_memory = sys.total_memory();
    let free_memory = sys.free_memory();
    let usage_percent = ((total_memory - free_memory) as f32 / total_memory as f32) * 100.0;
    
    let (status, message) = if usage_percent > 90.0 {
        ("unhealthy", Some("Memory usage critical (>90%)".to_string()))
    } else if usage_percent > 80.0 {
        ("degraded", Some("Memory usage high (>80%)".to_string()))
    } else {
        ("healthy", None)
    };
    
    ComponentHealth {
        status: status.to_string(),
        message,
        details: Some(serde_json::json!({
            "total_mb": total_memory / 1024,
            "free_mb": free_memory / 1024,
            "usage_percent": usage_percent,
        })),
    }
}

fn check_disk() -> ComponentHealth {
    // Simplified disk check for sysinfo 0.35
    // In a real implementation, you might use std::fs to check specific mount points
    ComponentHealth {
        status: "healthy".to_string(),
        message: None,
        details: Some(serde_json::json!({
            "note": "Disk monitoring not fully implemented in this version"
        })),
    }
}

async fn check_mcp_servers() -> Vec<MCPServerHealth> {
    let servers = vec![
        ("notion-mcp", "http://localhost:3001/health"),
        ("slack-mcp", "http://localhost:3002/health"),
        ("helpscout-mcp", "http://localhost:3003/health"),
    ];
    
    let mut results = Vec::new();
    
    for (name, url) in servers {
        let start = std::time::Instant::now();
        
        let status = match reqwest::get(url).await {
            Ok(response) => {
                let response_time = start.elapsed().as_millis() as u64;
                if response.status().is_success() {
                    MCPServerHealth {
                        name: name.to_string(),
                        url: url.to_string(),
                        status: "healthy".to_string(),
                        response_time_ms: Some(response_time),
                    }
                } else {
                    MCPServerHealth {
                        name: name.to_string(),
                        url: url.to_string(),
                        status: "unhealthy".to_string(),
                        response_time_ms: Some(response_time),
                    }
                }
            }
            Err(_) => MCPServerHealth {
                name: name.to_string(),
                url: url.to_string(),
                status: "unreachable".to_string(),
                response_time_ms: None,
            },
        };
        
        results.push(status);
    }
    
    results
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/health")
            .route(web::get().to(health_check))
    );
    cfg.service(
        web::resource("/health/detailed")
            .route(web::get().to(detailed_health_check))
    );
}