//! Presence Tracking System
//! 
//! User presence and status tracking with online/offline detection,
//! typing indicators, last seen timestamps, and subscription management.

use actix::{Actor, Addr, Context, Handler, AsyncContext};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use redis::AsyncCommands;

use crate::actors::messages::*;
use crate::actors::router::RouterActor;

/// Presence tracking actor
pub struct PresenceTrackingActor {
    /// Router for sending presence updates
    router_addr: Option<Addr<RouterActor>>,
    
    /// User presence information
    user_presence: HashMap<String, UserPresenceInfo>,
    
    /// Active typing indicators
    typing_indicators: HashMap<String, TypingSession>, // conversation_id -> typing session
    
    /// Presence subscriptions (who wants to know about whom)
    presence_subscriptions: HashMap<String, HashSet<String>>, // user_id -> set of subscribed users
    
    /// Connection to user mapping
    connection_users: HashMap<Uuid, String>,
    
    /// Presence metrics
    metrics: PresenceMetrics,
    
    /// Configuration
    config: PresenceConfig,
    
    /// Redis client for distributed presence
    redis_client: Option<Arc<redis::Client>>,
}

/// Detailed user presence information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPresenceInfo {
    pub user_id: String,
    pub status: PresenceStatus,
    pub custom_message: Option<String>,
    pub last_seen: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub active_connections: HashSet<Uuid>,
    pub device_info: Vec<DeviceInfo>,
    pub timezone: Option<String>,
    pub auto_away_enabled: bool,
    pub away_message: Option<String>,
}

/// Device information for presence tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub connection_id: Uuid,
    pub device_type: String, // "desktop", "mobile", "tablet"
    pub platform: String,    // "windows", "mac", "ios", "android"
    pub app_version: Option<String>,
    pub connected_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}

/// Typing session information
#[derive(Debug, Clone)]
struct TypingSession {
    conversation_id: String,
    typing_users: HashMap<String, TypingInfo>,
    last_updated: Instant,
}

/// Individual typing information
#[derive(Debug, Clone)]
struct TypingInfo {
    user_id: String,
    connection_id: Uuid,
    started_at: Instant,
    last_activity: Instant,
}

/// Presence metrics
#[derive(Debug, Default)]
struct PresenceMetrics {
    total_users_tracked: usize,
    online_users: usize,
    away_users: usize,
    busy_users: usize,
    offline_users: usize,
    active_typing_sessions: usize,
    presence_updates: u64,
    subscription_changes: u64,
    typing_events: u64,
}

/// Presence configuration
#[derive(Debug, Clone)]
pub struct PresenceConfig {
    pub offline_timeout: Duration,
    pub away_timeout: Duration,
    pub typing_timeout: Duration,
    pub cleanup_interval: Duration,
    pub presence_broadcast_interval: Duration,
    pub max_device_history: usize,
    pub enable_auto_away: bool,
    pub enable_redis_sync: bool,
    pub batch_updates: bool,
}

impl Default for PresenceConfig {
    fn default() -> Self {
        Self {
            offline_timeout: Duration::from_secs(300),     // 5 minutes
            away_timeout: Duration::from_secs(600),        // 10 minutes
            typing_timeout: Duration::from_secs(10),       // 10 seconds
            cleanup_interval: Duration::from_secs(60),     // 1 minute
            presence_broadcast_interval: Duration::from_secs(30), // 30 seconds
            max_device_history: 10,
            enable_auto_away: true,
            enable_redis_sync: false,
            batch_updates: true,
        }
    }
}

impl PresenceTrackingActor {
    pub fn new(
        config: PresenceConfig,
        redis_client: Option<Arc<redis::Client>>,
    ) -> Self {
        Self {
            router_addr: None,
            user_presence: HashMap::new(),
            typing_indicators: HashMap::new(),
            presence_subscriptions: HashMap::new(),
            connection_users: HashMap::new(),
            metrics: PresenceMetrics::default(),
            config,
            redis_client,
        }
    }

    /// Set router address
    pub fn set_router(&mut self, router_addr: Addr<RouterActor>) {
        self.router_addr = Some(router_addr);
        info!("Router address set in presence tracking actor");
    }

    /// Update user presence
    fn update_user_presence(
        &mut self,
        user_id: String,
        connection_id: Uuid,
        status: PresenceStatus,
        custom_message: Option<String>,
        device_info: Option<DeviceInfo>,
    ) {
        let now = Utc::now();
        let was_online = self.is_user_online(&user_id);

        // Get or create presence info
        let presence = self.user_presence.entry(user_id.clone()).or_insert_with(|| {
            UserPresenceInfo {
                user_id: user_id.clone(),
                status: PresenceStatus::Offline,
                custom_message: None,
                last_seen: now,
                last_activity: now,
                active_connections: HashSet::new(),
                device_info: Vec::new(),
                timezone: None,
                auto_away_enabled: self.config.enable_auto_away,
                away_message: None,
            }
        });

        let old_status = presence.status.clone();
        
        // Update basic presence info
        presence.status = status.clone();
        presence.custom_message = custom_message.clone();
        presence.last_activity = now;
        
        // Update connection tracking
        presence.active_connections.insert(connection_id);
        self.connection_users.insert(connection_id, user_id.clone());

        // Update device info if provided
        if let Some(device) = device_info {
            // Remove existing device info for this connection
            presence.device_info.retain(|d| d.connection_id != connection_id);
            
            // Add new device info
            presence.device_info.push(device);
            
            // Limit device history
            if presence.device_info.len() > self.config.max_device_history {
                presence.device_info.truncate(self.config.max_device_history);
            }
        }

        // Update last seen for online statuses
        if status != PresenceStatus::Offline {
            presence.last_seen = now;
        }

        self.metrics.presence_updates += 1;
        self.update_metrics();

        // Broadcast presence update if status changed or user came online
        let should_broadcast = old_status != status || (!was_online && self.is_user_online(&user_id));
        
        if should_broadcast {
            self.broadcast_presence_update(&user_id, presence);
        }

        info!("Presence updated: user_id={}, status={:?}, connections={}", 
              user_id, status, presence.active_connections.len());
    }

    /// Handle connection disconnect
    fn handle_connection_disconnect(&mut self, connection_id: Uuid) {
        if let Some(user_id) = self.connection_users.remove(&connection_id) {
            if let Some(presence) = self.user_presence.get_mut(&user_id) {
                presence.active_connections.remove(&connection_id);
                
                // Remove device info for this connection
                presence.device_info.retain(|d| d.connection_id != connection_id);
                
                // If no more active connections, mark as offline
                if presence.active_connections.is_empty() {
                    let old_status = presence.status.clone();
                    presence.status = PresenceStatus::Offline;
                    presence.last_seen = Utc::now();
                    
                    if old_status != PresenceStatus::Offline {
                        self.broadcast_presence_update(&user_id, presence);
                    }
                }
                
                self.update_metrics();
                
                info!("Connection disconnected: user_id={}, remaining_connections={}", 
                      user_id, presence.active_connections.len());
            }
        }
    }

    /// Update typing indicator
    fn update_typing_indicator(
        &mut self,
        user_id: String,
        conversation_id: String,
        connection_id: Uuid,
        is_typing: bool,
    ) {
        let now = Instant::now();
        
        // Get or create typing session
        let session = self.typing_indicators.entry(conversation_id.clone()).or_insert_with(|| {
            TypingSession {
                conversation_id: conversation_id.clone(),
                typing_users: HashMap::new(),
                last_updated: now,
            }
        });

        let was_typing = session.typing_users.contains_key(&user_id);
        
        if is_typing {
            // Add or update typing info
            session.typing_users.insert(user_id.clone(), TypingInfo {
                user_id: user_id.clone(),
                connection_id,
                started_at: if was_typing { 
                    session.typing_users[&user_id].started_at 
                } else { 
                    now 
                },
                last_activity: now,
            });
        } else {
            // Remove typing info
            session.typing_users.remove(&user_id);
        }
        
        session.last_updated = now;
        
        // Clean up empty sessions
        if session.typing_users.is_empty() {
            self.typing_indicators.remove(&conversation_id);
        }
        
        self.metrics.typing_events += 1;
        self.metrics.active_typing_sessions = self.typing_indicators.len();
        
        // Broadcast typing indicator update
        self.broadcast_typing_indicator(&conversation_id, &user_id, is_typing);
        
        debug!("Typing indicator updated: user_id={}, conversation_id={}, is_typing={}", 
               user_id, conversation_id, is_typing);
    }

    /// Add presence subscription
    fn add_presence_subscription(&mut self, subscriber: String, target_user: String) {
        self.presence_subscriptions
            .entry(target_user.clone())
            .or_insert_with(HashSet::new)
            .insert(subscriber.clone());
        
        self.metrics.subscription_changes += 1;
        
        // Send current presence status to subscriber
        if let Some(presence) = self.user_presence.get(&target_user) {
            self.send_presence_update_to_user(&subscriber, &target_user, presence);
        }
        
        debug!("Presence subscription added: subscriber={}, target={}", subscriber, target_user);
    }

    /// Remove presence subscription
    fn remove_presence_subscription(&mut self, subscriber: String, target_user: String) {
        if let Some(subscribers) = self.presence_subscriptions.get_mut(&target_user) {
            subscribers.remove(&subscriber);
            
            if subscribers.is_empty() {
                self.presence_subscriptions.remove(&target_user);
            }
            
            self.metrics.subscription_changes += 1;
            
            debug!("Presence subscription removed: subscriber={}, target={}", subscriber, target_user);
        }
    }

    /// Broadcast presence update to subscribers
    fn broadcast_presence_update(&self, user_id: &str, presence: &UserPresenceInfo) {
        if let Some(subscribers) = self.presence_subscriptions.get(user_id) {
            for subscriber in subscribers {
                self.send_presence_update_to_user(subscriber, user_id, presence);
            }
        }
    }

    /// Send presence update to specific user
    fn send_presence_update_to_user(&self, subscriber: &str, target_user: &str, presence: &UserPresenceInfo) {
        if let Some(router_addr) = &self.router_addr {
            let presence_message = ServerMessage::PresenceUpdate {
                user_id: target_user.to_string(),
                status: presence.status.clone(),
                last_seen: Some(presence.last_seen.timestamp()),
            };
            
            // We'd need a way to route to specific user through router
            debug!("Would send presence update to {}: {:?}", subscriber, presence_message);
        }
    }

    /// Broadcast typing indicator
    fn broadcast_typing_indicator(&self, conversation_id: &str, user_id: &str, is_typing: bool) {
        if let Some(router_addr) = &self.router_addr {
            let typing_message = ServerMessage::TypingIndicator {
                user_id: user_id.to_string(),
                conversation_id: conversation_id.to_string(),
                is_typing,
            };
            
            // Broadcast to all participants in the conversation
            debug!("Would broadcast typing indicator: {:?}", typing_message);
        }
    }

    /// Check if user is online
    fn is_user_online(&self, user_id: &str) -> bool {
        self.user_presence
            .get(user_id)
            .map(|p| !p.active_connections.is_empty() && p.status != PresenceStatus::Offline)
            .unwrap_or(false)
    }

    /// Update metrics
    fn update_metrics(&mut self) {
        self.metrics.total_users_tracked = self.user_presence.len();
        self.metrics.online_users = self.user_presence
            .values()
            .filter(|p| p.status == PresenceStatus::Online)
            .count();
        self.metrics.away_users = self.user_presence
            .values()
            .filter(|p| p.status == PresenceStatus::Away)
            .count();
        self.metrics.busy_users = self.user_presence
            .values()
            .filter(|p| p.status == PresenceStatus::Busy)
            .count();
        self.metrics.offline_users = self.user_presence
            .values()
            .filter(|p| p.status == PresenceStatus::Offline)
            .count();
        self.metrics.active_typing_sessions = self.typing_indicators.len();
    }

    /// Cleanup expired presence and typing data
    fn cleanup_expired(&mut self) {
        let now = Instant::now();
        let utc_now = Utc::now();
        
        // Clean up expired typing indicators
        let mut expired_conversations = Vec::new();
        for (conversation_id, session) in &mut self.typing_indicators {
            let mut expired_users = Vec::new();
            
            for (user_id, typing_info) in &session.typing_users {
                if now.duration_since(typing_info.last_activity) > self.config.typing_timeout {
                    expired_users.push(user_id.clone());
                }
            }
            
            for user_id in expired_users {
                session.typing_users.remove(&user_id);
                self.broadcast_typing_indicator(conversation_id, &user_id, false);
            }
            
            if session.typing_users.is_empty() {
                expired_conversations.push(conversation_id.clone());
            }
        }
        
        for conversation_id in expired_conversations {
            self.typing_indicators.remove(&conversation_id);
        }
        
        // Handle auto-away for inactive users
        if self.config.enable_auto_away {
            let mut users_to_update = Vec::new();
            
            for (user_id, presence) in &mut self.user_presence {
                if presence.auto_away_enabled && 
                   presence.status == PresenceStatus::Online &&
                   utc_now.signed_duration_since(presence.last_activity).num_seconds() > 
                   self.config.away_timeout.as_secs() as i64 {
                    
                    presence.status = PresenceStatus::Away;
                    presence.away_message = Some("Auto away due to inactivity".to_string());
                    users_to_update.push((user_id.clone(), presence.clone()));
                }
            }
            
            for (user_id, presence) in users_to_update {
                self.broadcast_presence_update(&user_id, &presence);
                info!("User automatically set to away: {}", user_id);
            }
        }
        
        // Clean up offline users with no connections
        let offline_threshold = utc_now - chrono::Duration::seconds(self.config.offline_timeout.as_secs() as i64);
        let mut users_to_remove = Vec::new();
        
        for (user_id, presence) in &self.user_presence {
            if presence.active_connections.is_empty() && 
               presence.last_seen < offline_threshold {
                users_to_remove.push(user_id.clone());
            }
        }
        
        for user_id in users_to_remove {
            self.user_presence.remove(&user_id);
            self.presence_subscriptions.remove(&user_id);
            debug!("Cleaned up offline user: {}", user_id);
        }
        
        self.update_metrics();
        
        debug!("Cleanup completed: {} typing sessions, {} users tracked", 
               self.typing_indicators.len(), self.user_presence.len());
    }

    /// Get presence statistics
    fn get_presence_stats(&self) -> PresenceStats {
        PresenceStats {
            total_users_tracked: self.metrics.total_users_tracked,
            online_users: self.metrics.online_users,
            away_users: self.metrics.away_users,
            busy_users: self.metrics.busy_users,
            offline_users: self.metrics.offline_users,
            active_typing_sessions: self.metrics.active_typing_sessions,
            presence_updates: self.metrics.presence_updates,
            typing_events: self.metrics.typing_events,
            subscription_changes: self.metrics.subscription_changes,
        }
    }

    /// Sync presence data with Redis
    async fn sync_with_redis(&self) -> Result<(), String> {
        if !self.config.enable_redis_sync {
            return Ok(());
        }
        
        if let Some(redis_client) = &self.redis_client {
            let mut conn = redis_client.get_async_connection().await
                .map_err(|e| format!("Redis connection failed: {}", e))?;
            
            // Sync presence data
            for (user_id, presence) in &self.user_presence {
                let key = format!("presence:{}", user_id);
                let value = serde_json::to_string(presence)
                    .map_err(|e| format!("Serialization failed: {}", e))?;
                
                let _: () = conn.setex(key, self.config.offline_timeout.as_secs(), value).await
                    .map_err(|e| format!("Redis set failed: {}", e))?;
            }
            
            debug!("Synced {} presence entries to Redis", self.user_presence.len());
        }
        
        Ok(())
    }
}

/// Presence statistics
#[derive(Debug, Serialize)]
pub struct PresenceStats {
    pub total_users_tracked: usize,
    pub online_users: usize,
    pub away_users: usize,
    pub busy_users: usize,
    pub offline_users: usize,
    pub active_typing_sessions: usize,
    pub presence_updates: u64,
    pub typing_events: u64,
    pub subscription_changes: u64,
}

impl Actor for PresenceTrackingActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Presence tracking actor started");

        // Start periodic cleanup
        ctx.run_interval(self.config.cleanup_interval, |act, _ctx| {
            act.cleanup_expired();
        });

        // Start periodic Redis sync if enabled
        if self.config.enable_redis_sync {
            ctx.run_interval(self.config.presence_broadcast_interval, |act, _ctx| {
                actix::spawn(async move {
                    if let Err(e) = act.sync_with_redis().await {
                        error!("Redis sync failed: {}", e);
                    }
                });
            });
        }
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("Presence tracking actor stopped");
        
        info!("Final presence stats - Online: {}, Away: {}, Busy: {}, Offline: {}",
              self.metrics.online_users,
              self.metrics.away_users,
              self.metrics.busy_users,
              self.metrics.offline_users);
    }
}

/// Update presence handler
impl Handler<UpdatePresence> for PresenceTrackingActor {
    type Result = ();

    fn handle(&mut self, msg: UpdatePresence, _ctx: &mut Self::Context) -> Self::Result {
        self.update_user_presence(
            msg.user_id,
            msg.connection_id,
            msg.status,
            msg.message,
            None, // Device info would need to be added to message
        );
    }
}

/// Typing indicator handler
impl Handler<TypingIndicator> for PresenceTrackingActor {
    type Result = ();

    fn handle(&mut self, msg: TypingIndicator, _ctx: &mut Self::Context) -> Self::Result {
        self.update_typing_indicator(
            msg.user_id,
            msg.conversation_id,
            msg.connection_id,
            msg.is_typing,
        );
    }
}

/// Disconnect handler
impl Handler<Disconnect> for PresenceTrackingActor {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _ctx: &mut Self::Context) -> Self::Result {
        self.handle_connection_disconnect(msg.connection_id);
    }
}

/// Presence subscription messages
#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct SubscribeToPresence {
    pub subscriber: String,
    pub target_user: String,
}

impl Handler<SubscribeToPresence> for PresenceTrackingActor {
    type Result = ();

    fn handle(&mut self, msg: SubscribeToPresence, _ctx: &mut Self::Context) -> Self::Result {
        self.add_presence_subscription(msg.subscriber, msg.target_user);
    }
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct UnsubscribeFromPresence {
    pub subscriber: String,
    pub target_user: String,
}

impl Handler<UnsubscribeFromPresence> for PresenceTrackingActor {
    type Result = ();

    fn handle(&mut self, msg: UnsubscribeFromPresence, _ctx: &mut Self::Context) -> Self::Result {
        self.remove_presence_subscription(msg.subscriber, msg.target_user);
    }
}

/// Get presence stats message
#[derive(actix::Message)]
#[rtype(result = "PresenceStats")]
pub struct GetPresenceStats;

impl Handler<GetPresenceStats> for PresenceTrackingActor {
    type Result = PresenceStats;

    fn handle(&mut self, _msg: GetPresenceStats, _ctx: &mut Self::Context) -> Self::Result {
        self.get_presence_stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presence_config_default() {
        let config = PresenceConfig::default();
        assert_eq!(config.offline_timeout, Duration::from_secs(300));
        assert_eq!(config.away_timeout, Duration::from_secs(600));
        assert_eq!(config.typing_timeout, Duration::from_secs(10));
        assert!(config.enable_auto_away);
        assert!(!config.enable_redis_sync);
        assert!(config.batch_updates);
    }

    #[test]
    fn test_device_info() {
        let device = DeviceInfo {
            connection_id: Uuid::new_v4(),
            device_type: "mobile".to_string(),
            platform: "ios".to_string(),
            app_version: Some("1.0.0".to_string()),
            connected_at: Utc::now(),
            last_activity: Utc::now(),
        };
        
        assert_eq!(device.device_type, "mobile");
        assert_eq!(device.platform, "ios");
        assert_eq!(device.app_version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_user_presence_info() {
        let presence = UserPresenceInfo {
            user_id: "test_user".to_string(),
            status: PresenceStatus::Online,
            custom_message: Some("Working".to_string()),
            last_seen: Utc::now(),
            last_activity: Utc::now(),
            active_connections: HashSet::new(),
            device_info: Vec::new(),
            timezone: Some("UTC".to_string()),
            auto_away_enabled: true,
            away_message: None,
        };
        
        assert_eq!(presence.user_id, "test_user");
        assert_eq!(presence.status, PresenceStatus::Online);
        assert_eq!(presence.custom_message, Some("Working".to_string()));
        assert!(presence.auto_away_enabled);
    }
}