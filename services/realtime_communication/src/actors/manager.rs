//! Session Manager Actor
//! 
//! Supervises all session actors and provides system-wide coordination
//! including presence management, session discovery, and health monitoring.

use actix::{Actor, Addr, Context, Handler, AsyncContext, Supervisor, SystemService};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use redis::AsyncCommands;

use super::messages::*;
use super::router::RouterActor;
use super::session::{SessionActor, SessionConfig};

/// Session Manager Actor - Coordinates all sessions
pub struct SessionManagerActor {
    /// Active session actors
    sessions: HashMap<Uuid, Addr<SessionActor>>,
    
    /// User presence tracking
    user_presence: HashMap<String, UserPresence>,
    
    /// Typing indicators
    typing_indicators: HashMap<String, HashSet<String>>, // conversation_id -> set of user_ids
    
    /// Router actor address
    router_addr: Option<Addr<RouterActor>>,
    
    /// Redis client for distributed state
    redis_client: Option<Arc<redis::Client>>,
    
    /// Manager metrics
    metrics: ManagerMetrics,
    
    /// Configuration
    config: ManagerConfig,
    
    /// System startup time
    startup_time: Instant,
}

/// User presence information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct UserPresence {
    user_id: String,
    status: PresenceStatus,
    last_seen: DateTime<Utc>,
    connection_count: usize,
    custom_message: Option<String>,
    metadata: HashMap<String, String>,
}

/// Manager metrics
#[derive(Debug, Default)]
struct ManagerMetrics {
    total_sessions_created: u64,
    total_sessions_destroyed: u64,
    active_sessions: usize,
    unique_users_online: usize,
    presence_updates: u64,
    typing_events: u64,
    cleanup_operations: u64,
    health_checks: u64,
}

/// Manager configuration
#[derive(Debug, Clone)]
pub struct ManagerConfig {
    pub presence_timeout: Duration,
    pub typing_timeout: Duration,
    pub cleanup_interval: Duration,
    pub health_check_interval: Duration,
    pub max_sessions_per_user: usize,
    pub enable_redis_persistence: bool,
    pub session_config: SessionConfig,
}

impl Default for ManagerConfig {
    fn default() -> Self {
        Self {
            presence_timeout: Duration::from_secs(300), // 5 minutes
            typing_timeout: Duration::from_secs(10),
            cleanup_interval: Duration::from_secs(60),
            health_check_interval: Duration::from_secs(30),
            max_sessions_per_user: 5,
            enable_redis_persistence: false,
            session_config: SessionConfig::default(),
        }
    }
}

impl SessionManagerActor {
    pub fn new(config: ManagerConfig, redis_client: Option<Arc<redis::Client>>) -> Self {
        Self {
            sessions: HashMap::new(),
            user_presence: HashMap::new(),
            typing_indicators: HashMap::new(),
            router_addr: None,
            redis_client,
            metrics: ManagerMetrics::default(),
            config,
            startup_time: Instant::now(),
        }
    }

    /// Set router address
    pub fn set_router(&mut self, router_addr: Addr<RouterActor>) {
        self.router_addr = Some(router_addr);
        info!("Router address set in session manager");
    }

    /// Create a new session
    fn create_session(&mut self, connection_id: Uuid) -> Result<Addr<SessionActor>, String> {
        if let Some(router_addr) = &self.router_addr {
            let session_actor = SessionActor::new(
                connection_id,
                router_addr.clone(),
                self.config.session_config.clone(),
                self.redis_client.clone(),
            );
            
            let session_addr = Supervisor::start(|_| session_actor);
            self.sessions.insert(connection_id, session_addr.clone());
            
            self.metrics.total_sessions_created += 1;
            self.metrics.active_sessions = self.sessions.len();
            
            info!("Session created: connection_id={}, total_sessions={}", 
                  connection_id, self.sessions.len());
            
            Ok(session_addr)
        } else {
            Err("Router not available".to_string())
        }
    }

    /// Remove a session
    fn remove_session(&mut self, connection_id: &Uuid) -> bool {
        if let Some(session_addr) = self.sessions.remove(connection_id) {
            // Stop the session actor
            session_addr.do_send(CleanupSession {
                connection_id: *connection_id,
                reason: "Session removed by manager".to_string(),
            });
            
            self.metrics.total_sessions_destroyed += 1;
            self.metrics.active_sessions = self.sessions.len();
            
            info!("Session removed: connection_id={}, remaining_sessions={}", 
                  connection_id, self.sessions.len());
            
            true
        } else {
            false
        }
    }

    /// Update user presence
    fn update_user_presence(
        &mut self,
        user_id: String,
        connection_id: Uuid,
        status: PresenceStatus,
        custom_message: Option<String>,
    ) {
        let now = Utc::now();
        
        // Update or create presence entry
        let presence = self.user_presence.entry(user_id.clone()).or_insert_with(|| {
            UserPresence {
                user_id: user_id.clone(),
                status: PresenceStatus::Offline,
                last_seen: now,
                connection_count: 0,
                custom_message: None,
                metadata: HashMap::new(),
            }
        });
        
        let old_status = presence.status.clone();
        presence.status = status.clone();
        presence.last_seen = now;
        presence.custom_message = custom_message;
        
        // Count active connections for this user
        presence.connection_count = self.sessions.iter()
            .filter(|(_, _)| {
                // We'd need to track user_id per session to properly count
                // For now, we'll estimate based on presence updates
                true
            })
            .count();
        
        // If user went offline and has no connections, mark as offline
        if presence.connection_count == 0 && status != PresenceStatus::Offline {
            presence.status = PresenceStatus::Offline;
        }
        
        self.metrics.presence_updates += 1;
        self.metrics.unique_users_online = self.user_presence
            .values()
            .filter(|p| p.status != PresenceStatus::Offline)
            .count();
        
        // Notify router if status changed
        if old_status != presence.status {
            if let Some(router_addr) = &self.router_addr {
                let update_message = UpdatePresence {
                    user_id: user_id.clone(),
                    connection_id,
                    status: presence.status.clone(),
                    message: presence.custom_message.clone(),
                };
                router_addr.do_send(update_message);
            }
        }
        
        debug!("Presence updated: user_id={}, status={:?}, connections={}", 
               user_id, presence.status, presence.connection_count);
    }

    /// Handle typing indicator
    fn handle_typing_indicator(&mut self, user_id: String, conversation_id: String, is_typing: bool) {
        if is_typing {
            self.typing_indicators
                .entry(conversation_id.clone())
                .or_insert_with(HashSet::new)
                .insert(user_id.clone());
        } else {
            if let Some(typing_users) = self.typing_indicators.get_mut(&conversation_id) {
                typing_users.remove(&user_id);
                if typing_users.is_empty() {
                    self.typing_indicators.remove(&conversation_id);
                }
            }
        }
        
        self.metrics.typing_events += 1;
        
        // Broadcast typing indicator through router
        if let Some(router_addr) = &self.router_addr {
            // The router would need to handle typing indicators
            debug!("Typing indicator: user_id={}, conversation_id={}, is_typing={}", 
                   user_id, conversation_id, is_typing);
        }
    }

    /// Cleanup expired sessions and presence
    fn cleanup_expired(&mut self) {
        let now = Utc::now();
        let mut expired_users = Vec::new();
        
        // Find expired presence entries
        for (user_id, presence) in &self.user_presence {
            if now.signed_duration_since(presence.last_seen).num_seconds() > 
               self.config.presence_timeout.as_secs() as i64 {
                expired_users.push(user_id.clone());
            }
        }
        
        // Remove expired presence
        for user_id in expired_users {
            if let Some(mut presence) = self.user_presence.remove(&user_id) {
                presence.status = PresenceStatus::Offline;
                self.user_presence.insert(user_id.clone(), presence);
                
                info!("User presence expired: user_id={}", user_id);
                
                // Notify router
                if let Some(router_addr) = &self.router_addr {
                    let update_message = UpdatePresence {
                        user_id,
                        connection_id: Uuid::new_v4(), // Dummy connection ID for system update
                        status: PresenceStatus::Offline,
                        message: None,
                    };
                    router_addr.do_send(update_message);
                }
            }
        }
        
        // Cleanup expired typing indicators
        let typing_timeout_secs = self.config.typing_timeout.as_secs() as i64;
        self.typing_indicators.retain(|conversation_id, typing_users| {
            // In a real implementation, we'd track timestamps for typing events
            // For now, we'll just clean up periodically
            if typing_users.is_empty() {
                debug!("Removing empty typing indicator for conversation: {}", conversation_id);
                false
            } else {
                true
            }
        });
        
        self.metrics.cleanup_operations += 1;
        
        debug!("Cleanup completed: active_sessions={}, online_users={}, typing_conversations={}", 
               self.sessions.len(), 
               self.user_presence.len(),
               self.typing_indicators.len());
    }

    /// Perform health check
    fn health_check(&mut self) {
        self.metrics.health_checks += 1;
        
        // Check for dead sessions
        let mut dead_sessions = Vec::new();
        
        for (&connection_id, session_addr) in &self.sessions {
            // We'd normally ping the session actor, but for simplicity we'll just log
            debug!("Health check for session: {}", connection_id);
        }
        
        // Remove dead sessions
        for connection_id in dead_sessions {
            self.remove_session(&connection_id);
        }
        
        // Log health metrics
        info!("Health check completed: active_sessions={}, online_users={}", 
              self.sessions.len(), 
              self.user_presence.values().filter(|p| p.status != PresenceStatus::Offline).count());
    }

    /// Get system statistics
    fn get_system_statistics(&self) -> SystemStats {
        SystemStats {
            total_connections: self.sessions.len(),
            active_connections: self.sessions.len(),
            unique_users: self.user_presence.len(),
            messages_routed: 0, // Would be maintained by router
            messages_delivered: 0, // Would be maintained by router
            messages_failed: 0, // Would be maintained by router
            topics_active: 0, // Would be maintained by router
            uptime_seconds: self.startup_time.elapsed().as_secs(),
        }
    }

    /// Persist state to Redis
    async fn persist_state(&self) -> Result<(), String> {
        if !self.config.enable_redis_persistence {
            return Ok(());
        }
        
        if let Some(redis_client) = &self.redis_client {
            let mut conn = redis_client.get_async_connection().await
                .map_err(|e| format!("Redis connection failed: {}", e))?;
            
            // Persist user presence
            for (user_id, presence) in &self.user_presence {
                let key = format!("presence:{}", user_id);
                let value = serde_json::to_string(presence)
                    .map_err(|e| format!("Serialization failed: {}", e))?;
                
                let _: () = redis::cmd("SETEX")
                    .arg(&key)
                    .arg(self.config.presence_timeout.as_secs())
                    .arg(&value)
                    .query_async(&mut conn).await
                    .map_err(|e| format!("Redis set failed: {}", e))?;
            }
            
            debug!("State persisted to Redis: {} presence entries", self.user_presence.len());
        }
        
        Ok(())
    }

    /// Load state from Redis
    async fn load_state(&mut self) -> Result<(), String> {
        if !self.config.enable_redis_persistence {
            return Ok(());
        }
        
        if let Some(redis_client) = &self.redis_client {
            let mut conn = redis_client.get_async_connection().await
                .map_err(|e| format!("Redis connection failed: {}", e))?;
            
            // Load presence data
            let pattern = "presence:*";
            let keys: Vec<String> = conn.keys(pattern).await
                .map_err(|e| format!("Redis keys query failed: {}", e))?;
            
            for key in keys {
                let value: String = conn.get(&key).await
                    .map_err(|e| format!("Redis get failed: {}", e))?;
                
                let presence: UserPresence = serde_json::from_str(&value)
                    .map_err(|e| format!("Deserialization failed: {}", e))?;
                
                self.user_presence.insert(presence.user_id.clone(), presence);
            }
            
            info!("State loaded from Redis: {} presence entries", self.user_presence.len());
        }
        
        Ok(())
    }
}

impl Actor for SessionManagerActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Session manager actor started");
        
        // Start periodic cleanup
        ctx.run_interval(self.config.cleanup_interval, |act, _ctx| {
            act.cleanup_expired();
        });
        
        // Start periodic health checks
        ctx.run_interval(self.config.health_check_interval, |act, _ctx| {
            act.health_check();
        });
        
        // Load initial state from Redis
        if self.config.enable_redis_persistence {
            let manager = ctx.address();
            actix::spawn(async move {
                // We'd load state here in a real implementation
                debug!("Would load initial state from Redis");
            });
        }
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("Session manager actor stopped");
        
        // Final metrics log
        info!("Final metrics - Sessions created: {}, destroyed: {}, active: {}", 
              self.metrics.total_sessions_created,
              self.metrics.total_sessions_destroyed,
              self.metrics.active_sessions);
    }
}

/// Connect handler - Create new session
impl Handler<Connect> for SessionManagerActor {
    type Result = ();

    fn handle(&mut self, msg: Connect, _ctx: &mut Self::Context) -> Self::Result {
        // Session creation is handled by the router typically
        // This handler is for coordination purposes
        debug!("Session manager received connect for: {}", msg.connection_id);
    }
}

/// Disconnect handler - Remove session
impl Handler<Disconnect> for SessionManagerActor {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _ctx: &mut Self::Context) -> Self::Result {
        self.remove_session(&msg.connection_id);
    }
}

/// Update presence handler
impl Handler<UpdatePresence> for SessionManagerActor {
    type Result = ();

    fn handle(&mut self, msg: UpdatePresence, _ctx: &mut Self::Context) -> Self::Result {
        self.update_user_presence(msg.user_id, msg.connection_id, msg.status, msg.message);
    }
}

/// Typing indicator handler
impl Handler<TypingIndicator> for SessionManagerActor {
    type Result = ();

    fn handle(&mut self, msg: TypingIndicator, _ctx: &mut Self::Context) -> Self::Result {
        self.handle_typing_indicator(msg.user_id, msg.conversation_id, msg.is_typing);
    }
}

/// Get connections handler
impl Handler<GetConnections> for SessionManagerActor {
    type Result = Vec<ConnectionSummary>;

    fn handle(&mut self, msg: GetConnections, _ctx: &mut Self::Context) -> Self::Result {
        let mut connections = Vec::new();
        
        for (&connection_id, _session_addr) in &self.sessions {
            // In a real implementation, we'd query the session for details
            // For now, create a basic summary
            if let Some(ref filter_user_id) = msg.user_id {
                // Filter logic would go here
            }
            
            connections.push(ConnectionSummary {
                connection_id,
                user_id: None, // Would be populated from session
                connected_at: Utc::now(), // Would be from session
                last_activity: Utc::now(), // Would be from session
                subscriptions: Vec::new(), // Would be from session
                presence_status: PresenceStatus::Online,
            });
        }
        
        connections
    }
}

/// Get system stats handler
impl Handler<GetSystemStats> for SessionManagerActor {
    type Result = SystemStats;

    fn handle(&mut self, _msg: GetSystemStats, _ctx: &mut Self::Context) -> Self::Result {
        self.get_system_statistics()
    }
}

/// Cleanup session handler
impl Handler<CleanupSession> for SessionManagerActor {
    type Result = ();

    fn handle(&mut self, msg: CleanupSession, _ctx: &mut Self::Context) -> Self::Result {
        self.remove_session(&msg.connection_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_manager_config_default() {
        let config = ManagerConfig::default();
        assert_eq!(config.presence_timeout, Duration::from_secs(300));
        assert_eq!(config.typing_timeout, Duration::from_secs(10));
        assert_eq!(config.cleanup_interval, Duration::from_secs(60));
        assert_eq!(config.health_check_interval, Duration::from_secs(30));
        assert_eq!(config.max_sessions_per_user, 5);
        assert!(!config.enable_redis_persistence);
    }

    #[test]
    fn test_user_presence() {
        let presence = UserPresence {
            user_id: "test_user".to_string(),
            status: PresenceStatus::Online,
            last_seen: Utc::now(),
            connection_count: 1,
            custom_message: Some("Working".to_string()),
            metadata: HashMap::new(),
        };
        
        assert_eq!(presence.user_id, "test_user");
        assert_eq!(presence.status, PresenceStatus::Online);
        assert_eq!(presence.connection_count, 1);
        assert_eq!(presence.custom_message, Some("Working".to_string()));
    }

    #[test]
    fn test_manager_metrics() {
        let mut metrics = ManagerMetrics::default();
        assert_eq!(metrics.total_sessions_created, 0);
        assert_eq!(metrics.active_sessions, 0);
        
        metrics.total_sessions_created += 1;
        metrics.active_sessions = 1;
        
        assert_eq!(metrics.total_sessions_created, 1);
        assert_eq!(metrics.active_sessions, 1);
    }
}