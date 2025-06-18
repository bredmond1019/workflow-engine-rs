//! WebSocket Server Implementation
//! 
//! High-performance WebSocket server built on Actix-ws with support for
//! 10,000+ concurrent connections, heartbeat management, and graceful shutdown.

use actix_web::{web, App, HttpServer, HttpRequest, HttpResponse};
use actix_ws;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use tracing::{info, warn, error};
use std::time::{Duration, Instant};

use crate::connection::ConnectionManager;
use crate::actor::WebSocketActor;

/// WebSocket server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub heartbeat_interval: Duration,
    pub client_timeout: Duration,
    pub max_frame_size: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8081,
            max_connections: 10_000,
            heartbeat_interval: Duration::from_secs(30),
            client_timeout: Duration::from_secs(60),
            max_frame_size: 64 * 1024, // 64KB
        }
    }
}

/// WebSocket server state
#[derive(Clone)]
pub struct ServerState {
    pub connection_manager: Arc<ConnectionManager>,
    pub config: ServerConfig,
    pub metrics: Arc<ServerMetrics>,
}

/// Server metrics for monitoring
#[derive(Debug)]
pub struct ServerMetrics {
    pub active_connections: Arc<RwLock<usize>>,
    pub total_connections: Arc<RwLock<usize>>,
    pub messages_sent: Arc<RwLock<u64>>,
    pub messages_received: Arc<RwLock<u64>>,
    pub errors: Arc<RwLock<u64>>,
    pub last_heartbeat_check: Arc<RwLock<Instant>>,
}

impl Default for ServerMetrics {
    fn default() -> Self {
        Self {
            active_connections: Arc::new(RwLock::new(0)),
            total_connections: Arc::new(RwLock::new(0)),
            messages_sent: Arc::new(RwLock::new(0)),
            messages_received: Arc::new(RwLock::new(0)),
            errors: Arc::new(RwLock::new(0)),
            last_heartbeat_check: Arc::new(RwLock::new(Instant::now())),
        }
    }
}

impl ServerMetrics {
    pub async fn increment_connections(&self) {
        let mut active = self.active_connections.write().await;
        let mut total = self.total_connections.write().await;
        *active += 1;
        *total += 1;
    }

    pub async fn decrement_connections(&self) {
        let mut active = self.active_connections.write().await;
        if *active > 0 {
            *active -= 1;
        }
    }

    pub async fn increment_messages_sent(&self) {
        let mut sent = self.messages_sent.write().await;
        *sent += 1;
    }

    pub async fn increment_messages_received(&self) {
        let mut received = self.messages_received.write().await;
        *received += 1;
    }

    pub async fn increment_errors(&self) {
        let mut errors = self.errors.write().await;
        *errors += 1;
    }

    pub async fn get_stats(&self) -> ServerStats {
        ServerStats {
            active_connections: *self.active_connections.read().await,
            total_connections: *self.total_connections.read().await,
            messages_sent: *self.messages_sent.read().await,
            messages_received: *self.messages_received.read().await,
            errors: *self.errors.read().await,
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ServerStats {
    pub active_connections: usize,
    pub total_connections: usize,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub errors: u64,
}

/// WebSocket Server
pub struct WebSocketServer {
    state: ServerState,
}

impl WebSocketServer {
    /// Create a new WebSocket server
    pub fn new(config: ServerConfig) -> Self {
        let connection_manager = Arc::new(ConnectionManager::new(config.max_connections));
        let metrics = Arc::new(ServerMetrics::default());

        let state = ServerState {
            connection_manager,
            config,
            metrics,
        };

        Self { state }
    }

    /// Start the WebSocket server
    pub async fn start(self) -> std::io::Result<()> {
        let bind_address = format!("{}:{}", self.state.config.host, self.state.config.port);
        
        info!(
            "Starting WebSocket server on {} (max connections: {})",
            bind_address, self.state.config.max_connections
        );

        // Start heartbeat task
        let heartbeat_state = self.state.clone();
        tokio::spawn(async move {
            Self::heartbeat_task(heartbeat_state).await;
        });

        // Start metrics reporting task
        let metrics_state = self.state.clone();
        tokio::spawn(async move {
            Self::metrics_task(metrics_state).await;
        });

        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(self.state.clone()))
                .route("/ws", web::get().to(websocket_handler))
                .route("/health", web::get().to(health_handler))
                .route("/metrics", web::get().to(metrics_handler))
        })
        .workers(num_cpus::get())
        .bind(&bind_address)?
        .run()
        .await
    }

    /// Heartbeat task to check client timeouts
    async fn heartbeat_task(state: ServerState) {
        let mut interval = tokio::time::interval(state.config.heartbeat_interval);
        
        loop {
            interval.tick().await;
            
            let now = Instant::now();
            let timeout_duration = state.config.client_timeout;
            
            // Check for timed out connections
            let timed_out_connections = state.connection_manager
                .get_timed_out_connections(now, timeout_duration)
                .await;
            
            for connection_id in timed_out_connections {
                warn!("Connection {} timed out, removing", connection_id);
                state.connection_manager.remove_connection(&connection_id).await;
                state.metrics.decrement_connections().await;
            }

            // Update heartbeat timestamp
            let mut last_check = state.metrics.last_heartbeat_check.write().await;
            *last_check = now;
        }
    }

    /// Metrics reporting task
    async fn metrics_task(state: ServerState) {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        
        loop {
            interval.tick().await;
            
            let stats = state.metrics.get_stats().await;
            info!(
                "Server metrics - Active: {}, Total: {}, Sent: {}, Received: {}, Errors: {}",
                stats.active_connections,
                stats.total_connections,
                stats.messages_sent,
                stats.messages_received,
                stats.errors
            );
        }
    }
}

/// WebSocket connection handler
pub async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    // Check connection limit
    let current_connections = *state.connection_manager.get_connection_count().await.read().await;
    if current_connections >= state.config.max_connections {
        warn!("Connection limit reached ({}), rejecting new connection", state.config.max_connections);
        return Ok(HttpResponse::ServiceUnavailable().json(
            serde_json::json!({"error": "Server at capacity"})
        ));
    }

    // Upgrade to WebSocket
    let (response, session, stream) = actix_ws::handle(&req, stream)?;
    
    let connection_id = Uuid::new_v4();
    info!("New WebSocket connection: {}", connection_id);

    // Create WebSocket actor
    let actor = WebSocketActor::new(
        connection_id,
        session,
        state.connection_manager.clone(),
        state.metrics.clone(),
        state.config.clone(),
    );

    // Spawn actor task
    actix_web::rt::spawn(async move {
        if let Err(e) = actor.run(stream).await {
            error!("WebSocket actor error for connection {}: {}", connection_id, e);
        }
    });

    // Update metrics
    state.metrics.increment_connections().await;

    Ok(response)
}

/// Health check endpoint
pub async fn health_handler(state: web::Data<ServerState>) -> HttpResponse {
    let stats = state.metrics.get_stats().await;
    let is_healthy = stats.active_connections < state.config.max_connections;
    
    if is_healthy {
        HttpResponse::Ok().json(serde_json::json!({
            "status": "healthy",
            "active_connections": stats.active_connections,
            "max_connections": state.config.max_connections,
            "uptime_seconds": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        }))
    } else {
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "unhealthy",
            "reason": "At capacity",
            "active_connections": stats.active_connections,
            "max_connections": state.config.max_connections
        }))
    }
}

/// Metrics endpoint
pub async fn metrics_handler(state: web::Data<ServerState>) -> HttpResponse {
    let stats = state.metrics.get_stats().await;
    HttpResponse::Ok().json(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;
    
    #[tokio::test]
    async fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8081);
        assert_eq!(config.max_connections, 10_000);
        assert_eq!(config.heartbeat_interval, Duration::from_secs(30));
        assert_eq!(config.client_timeout, Duration::from_secs(60));
        assert_eq!(config.max_frame_size, 64 * 1024);
    }

    #[tokio::test]
    async fn test_server_metrics() {
        let metrics = ServerMetrics::default();
        
        metrics.increment_connections().await;
        metrics.increment_messages_sent().await;
        metrics.increment_messages_received().await;
        metrics.increment_errors().await;
        
        let stats = metrics.get_stats().await;
        assert_eq!(stats.active_connections, 1);
        assert_eq!(stats.total_connections, 1);
        assert_eq!(stats.messages_sent, 1);
        assert_eq!(stats.messages_received, 1);
        assert_eq!(stats.errors, 1);
    }
    
    #[tokio::test]
    async fn test_metrics_decrement_connections() {
        let metrics = ServerMetrics::default();
        
        // Test decrement when no connections
        metrics.decrement_connections().await;
        let stats = metrics.get_stats().await;
        assert_eq!(stats.active_connections, 0);
        
        // Test normal decrement
        metrics.increment_connections().await;
        metrics.increment_connections().await;
        metrics.decrement_connections().await;
        
        let stats = metrics.get_stats().await;
        assert_eq!(stats.active_connections, 1);
        assert_eq!(stats.total_connections, 2); // Total should not decrease
    }
    
    #[tokio::test]
    async fn test_multiple_metrics_updates() {
        let metrics = ServerMetrics::default();
        
        // Simulate multiple connections and messages
        for _ in 0..5 {
            metrics.increment_connections().await;
        }
        
        for _ in 0..10 {
            metrics.increment_messages_sent().await;
            metrics.increment_messages_received().await;
        }
        
        for _ in 0..3 {
            metrics.decrement_connections().await;
        }
        
        let stats = metrics.get_stats().await;
        assert_eq!(stats.active_connections, 2);
        assert_eq!(stats.total_connections, 5);
        assert_eq!(stats.messages_sent, 10);
        assert_eq!(stats.messages_received, 10);
    }
    
    #[tokio::test]
    async fn test_server_creation() {
        let config = ServerConfig::default();
        let server = WebSocketServer::new(config.clone());
        
        assert_eq!(server.state.config.max_connections, config.max_connections);
    }
    
    #[tokio::test]
    async fn test_health_handler() {
        let config = ServerConfig::default();
        let connection_manager = Arc::new(ConnectionManager::new(config.max_connections));
        let metrics = Arc::new(ServerMetrics::default());
        
        let state = ServerState {
            connection_manager,
            config,
            metrics,
        };
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state.clone()))
                .route("/health", web::get().to(health_handler))
        ).await;
        
        let req = test::TestRequest::get()
            .uri("/health")
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["status"], "healthy");
        assert_eq!(body["active_connections"], 0);
        assert_eq!(body["max_connections"], 10_000);
    }
    
    #[tokio::test]
    async fn test_health_handler_at_capacity() {
        let mut config = ServerConfig::default();
        config.max_connections = 1;
        
        let connection_manager = Arc::new(ConnectionManager::new(config.max_connections));
        let metrics = Arc::new(ServerMetrics::default());
        
        // Simulate being at capacity
        metrics.increment_connections().await;
        
        let state = ServerState {
            connection_manager,
            config,
            metrics,
        };
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state.clone()))
                .route("/health", web::get().to(health_handler))
        ).await;
        
        let req = test::TestRequest::get()
            .uri("/health")
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 503); // Service Unavailable
        
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["status"], "unhealthy");
        assert_eq!(body["reason"], "At capacity");
    }
    
    #[tokio::test]
    async fn test_metrics_handler() {
        let config = ServerConfig::default();
        let connection_manager = Arc::new(ConnectionManager::new(config.max_connections));
        let metrics = Arc::new(ServerMetrics::default());
        
        // Add some metrics
        metrics.increment_connections().await;
        metrics.increment_messages_sent().await;
        metrics.increment_messages_sent().await;
        metrics.increment_messages_received().await;
        
        let state = ServerState {
            connection_manager,
            config,
            metrics,
        };
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state.clone()))
                .route("/metrics", web::get().to(metrics_handler))
        ).await;
        
        let req = test::TestRequest::get()
            .uri("/metrics")
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body: ServerStats = test::read_body_json(resp).await;
        assert_eq!(body.active_connections, 1);
        assert_eq!(body.total_connections, 1);
        assert_eq!(body.messages_sent, 2);
        assert_eq!(body.messages_received, 1);
        assert_eq!(body.errors, 0);
    }
    
    #[tokio::test]
    async fn test_concurrent_metrics_updates() {
        let metrics = Arc::new(ServerMetrics::default());
        let mut handles = vec![];
        
        // Spawn multiple tasks to update metrics concurrently
        for _ in 0..10 {
            let metrics_clone = metrics.clone();
            let handle = tokio::spawn(async move {
                for _ in 0..100 {
                    metrics_clone.increment_messages_sent().await;
                    metrics_clone.increment_messages_received().await;
                }
            });
            handles.push(handle);
        }
        
        // Wait for all tasks
        for handle in handles {
            handle.await.unwrap();
        }
        
        let stats = metrics.get_stats().await;
        assert_eq!(stats.messages_sent, 1000);
        assert_eq!(stats.messages_received, 1000);
    }
    
    #[tokio::test]
    async fn test_server_stats_serialization() {
        let stats = ServerStats {
            active_connections: 100,
            total_connections: 500,
            messages_sent: 10000,
            messages_received: 9500,
            errors: 50,
        };
        
        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: ServerStats = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.active_connections, stats.active_connections);
        assert_eq!(deserialized.total_connections, stats.total_connections);
        assert_eq!(deserialized.messages_sent, stats.messages_sent);
        assert_eq!(deserialized.messages_received, stats.messages_received);
        assert_eq!(deserialized.errors, stats.errors);
    }
}