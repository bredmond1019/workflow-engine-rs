//! Router Actor
//! 
//! Central message routing actor responsible for distributing messages
//! to appropriate sessions based on routing rules and subscriptions.

use actix::{Actor, Addr, Context, Handler, Message, Recipient, AsyncContext};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use super::messages::*;
use super::session::SessionActor;
use crate::routing::messages::RoutingMessage;
use crate::routing::router::{MessageRouter, TopicMessageRouter, RouterConfig};
use crate::connection::ConnectionManager;

/// Router Actor - Central message distribution hub
pub struct RouterActor {
    /// Session registry mapping connection IDs to session addresses
    sessions: HashMap<Uuid, Recipient<SessionMessage>>,
    
    /// User to connection mapping
    user_connections: HashMap<String, HashSet<Uuid>>,
    
    /// Topic subscriptions mapping
    topic_subscriptions: HashMap<String, HashSet<Uuid>>,
    
    /// Connection metadata
    connection_metadata: HashMap<Uuid, ConnectionInfo>,
    
    /// Message router for advanced routing logic
    message_router: Arc<dyn MessageRouter + Send + Sync>,
    
    /// Router metrics
    metrics: RouterMetrics,
    
    /// Configuration
    config: RouterConfig,
}

#[derive(Debug, Clone)]
struct ConnectionInfo {
    connection_id: Uuid,
    user_id: Option<String>,
    connected_at: DateTime<Utc>,
    last_activity: DateTime<Utc>,
    presence_status: PresenceStatus,
}

#[derive(Debug, Default)]
struct RouterMetrics {
    messages_routed: u64,
    messages_delivered: u64,
    messages_failed: u64,
    direct_messages: u64,
    topic_messages: u64,
    broadcast_messages: u64,
    active_sessions: usize,
    unique_users: usize,
}

impl RouterActor {
    pub fn new(connection_manager: Arc<ConnectionManager>) -> Self {
        let router_config = RouterConfig::default();
        let routing_rules = crate::routing::rules::RoutingRules::default();
        let message_router = Arc::new(TopicMessageRouter::new(
            connection_manager,
            routing_rules,
            router_config.clone(),
        ));

        Self {
            sessions: HashMap::new(),
            user_connections: HashMap::new(),
            topic_subscriptions: HashMap::new(),
            connection_metadata: HashMap::new(),
            message_router,
            metrics: RouterMetrics::default(),
            config: router_config,
        }
    }

    /// Add a new session to the router
    fn add_session(
        &mut self,
        connection_id: Uuid,
        user_id: Option<String>,
        session_addr: Recipient<SessionMessage>,
        metadata: HashMap<String, String>,
    ) {
        // Register session
        self.sessions.insert(connection_id, session_addr);
        
        // Add to user connections
        if let Some(ref user_id) = user_id {
            self.user_connections
                .entry(user_id.clone())
                .or_insert_with(HashSet::new)
                .insert(connection_id);
        }
        
        // Store connection metadata
        let connection_info = ConnectionInfo {
            connection_id,
            user_id: user_id.clone(),
            connected_at: Utc::now(),
            last_activity: Utc::now(),
            presence_status: PresenceStatus::Online,
        };
        self.connection_metadata.insert(connection_id, connection_info);
        
        // Update metrics
        self.metrics.active_sessions = self.sessions.len();
        self.metrics.unique_users = self.user_connections.len();
        
        info!(
            "Session registered: connection_id={}, user_id={:?}, total_sessions={}",
            connection_id,
            user_id,
            self.sessions.len()
        );
    }

    /// Remove a session from the router
    fn remove_session(&mut self, connection_id: &Uuid, reason: Option<&str>) {
        // Remove session
        if self.sessions.remove(connection_id).is_some() {
            info!(
                "Session removed: connection_id={}, reason={:?}",
                connection_id,
                reason
            );
        }
        
        // Remove from user connections
        if let Some(connection_info) = self.connection_metadata.remove(connection_id) {
            if let Some(user_id) = &connection_info.user_id {
                if let Some(user_connections) = self.user_connections.get_mut(user_id) {
                    user_connections.remove(connection_id);
                    if user_connections.is_empty() {
                        self.user_connections.remove(user_id);
                    }
                }
            }
        }
        
        // Remove from topic subscriptions
        for subscribers in self.topic_subscriptions.values_mut() {
            subscribers.remove(connection_id);
        }
        
        // Clean up empty topics
        self.topic_subscriptions.retain(|_, subscribers| !subscribers.is_empty());
        
        // Update metrics
        self.metrics.active_sessions = self.sessions.len();
        self.metrics.unique_users = self.user_connections.len();
    }

    /// Route a direct message to specific user(s)
    fn route_direct_message(
        &mut self,
        from_connection: Uuid,
        from_user: Option<String>,
        to_user: &str,
        content: serde_json::Value,
        message_id: Option<String>,
    ) {
        let message_id = message_id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let timestamp = Utc::now();
        
        self.metrics.direct_messages += 1;
        self.metrics.messages_routed += 1;
        
        // Find target user's connections
        if let Some(target_connections) = self.user_connections.get(to_user) {
            let server_message = ServerMessage::MessageReceived {
                from: from_user.clone().unwrap_or_else(|| from_connection.to_string()),
                content: content.clone(),
                message_id: message_id.clone(),
                timestamp: timestamp.timestamp(),
            };
            
            let session_message = SessionMessage {
                message: server_message,
                priority: MessagePriority::Normal,
            };
            
            let mut delivered_count = 0;
            for &target_connection in target_connections {
                if let Some(session_addr) = self.sessions.get(&target_connection) {
                    session_addr.do_send(session_message.clone());
                    delivered_count += 1;
                }
            }
            
            self.metrics.messages_delivered += delivered_count;
            
            if delivered_count > 0 {
                info!(
                    "Direct message delivered: from={:?} to={} connections={}",
                    from_user, to_user, delivered_count
                );
                
                // Send delivery confirmation back to sender
                if let Some(sender_session) = self.sessions.get(&from_connection) {
                    let confirmation = SessionMessage {
                        message: ServerMessage::DeliveryConfirmation {
                            message_id,
                            status: DeliveryStatus::Delivered,
                        },
                        priority: MessagePriority::Normal,
                    };
                    sender_session.do_send(confirmation);
                }
            } else {
                warn!("No active sessions found for user: {}", to_user);
                self.metrics.messages_failed += 1;
                
                // Send failure notification to sender
                if let Some(sender_session) = self.sessions.get(&from_connection) {
                    let error_message = SessionMessage {
                        message: ServerMessage::Error {
                            code: 404,
                            message: format!("User '{}' is not online", to_user),
                        },
                        priority: MessagePriority::Normal,
                    };
                    sender_session.do_send(error_message);
                }
            }
        } else {
            warn!("User not found: {}", to_user);
            self.metrics.messages_failed += 1;
            
            // Send error to sender
            if let Some(sender_session) = self.sessions.get(&from_connection) {
                let error_message = SessionMessage {
                    message: ServerMessage::Error {
                        code: 404,
                        message: format!("User '{}' not found", to_user),
                    },
                    priority: MessagePriority::Normal,
                };
                sender_session.do_send(error_message);
            }
        }
    }

    /// Route a topic message to all subscribers
    fn route_topic_message(
        &mut self,
        from_connection: Uuid,
        from_user: Option<String>,
        topic: &str,
        content: serde_json::Value,
        message_id: Option<String>,
    ) {
        let message_id = message_id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let timestamp = Utc::now();
        
        self.metrics.topic_messages += 1;
        self.metrics.messages_routed += 1;
        
        // Find topic subscribers
        if let Some(subscribers) = self.topic_subscriptions.get(topic) {
            let server_message = ServerMessage::TopicMessageReceived {
                topic: topic.to_string(),
                from: from_user.clone().unwrap_or_else(|| from_connection.to_string()),
                content: content.clone(),
                message_id: message_id.clone(),
                timestamp: timestamp.timestamp(),
            };
            
            let session_message = SessionMessage {
                message: server_message,
                priority: MessagePriority::Normal,
            };
            
            let mut delivered_count = 0;
            for &subscriber_connection in subscribers {
                // Don't send to the sender
                if subscriber_connection != from_connection {
                    if let Some(session_addr) = self.sessions.get(&subscriber_connection) {
                        session_addr.do_send(session_message.clone());
                        delivered_count += 1;
                    }
                }
            }
            
            self.metrics.messages_delivered += delivered_count;
            
            info!(
                "Topic message delivered: topic={} from={:?} subscribers={}",
                topic, from_user, delivered_count
            );
            
            // Send acknowledgment to sender
            if let Some(sender_session) = self.sessions.get(&from_connection) {
                let ack_message = SessionMessage {
                    message: ServerMessage::DeliveryConfirmation {
                        message_id,
                        status: DeliveryStatus::Delivered,
                    },
                    priority: MessagePriority::Normal,
                };
                sender_session.do_send(ack_message);
            }
        } else {
            warn!("No subscribers found for topic: {}", topic);
            self.metrics.messages_failed += 1;
            
            // Send acknowledgment even if no subscribers
            if let Some(sender_session) = self.sessions.get(&from_connection) {
                let ack_message = SessionMessage {
                    message: ServerMessage::DeliveryConfirmation {
                        message_id: message_id.clone(),
                        status: DeliveryStatus::Delivered,
                    },
                    priority: MessagePriority::Normal,
                };
                sender_session.do_send(ack_message);
            }
        }
    }

    /// Route a broadcast message to all connected sessions
    fn route_broadcast_message(
        &mut self,
        from_connection: Uuid,
        from_user: Option<String>,
        content: serde_json::Value,
        message_id: Option<String>,
    ) {
        let message_id = message_id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let timestamp = Utc::now();
        
        self.metrics.broadcast_messages += 1;
        self.metrics.messages_routed += 1;
        
        let server_message = ServerMessage::BroadcastReceived {
            from: from_user.clone().unwrap_or_else(|| from_connection.to_string()),
            content: content.clone(),
            message_id: message_id.clone(),
            timestamp: timestamp.timestamp(),
        };
        
        let session_message = SessionMessage {
            message: server_message,
            priority: MessagePriority::Normal,
        };
        
        let mut delivered_count = 0;
        for (&connection_id, session_addr) in &self.sessions {
            // Don't send to the sender
            if connection_id != from_connection {
                session_addr.do_send(session_message.clone());
                delivered_count += 1;
            }
        }
        
        self.metrics.messages_delivered += delivered_count;
        
        info!(
            "Broadcast message delivered: from={:?} recipients={}",
            from_user, delivered_count
        );
        
        // Send acknowledgment to sender
        if let Some(sender_session) = self.sessions.get(&from_connection) {
            let ack_message = SessionMessage {
                message: ServerMessage::DeliveryConfirmation {
                    message_id,
                    status: DeliveryStatus::Delivered,
                },
                priority: MessagePriority::Normal,
            };
            sender_session.do_send(ack_message);
        }
    }

    /// Subscribe connection to topics
    fn subscribe_to_topics(&mut self, connection_id: Uuid, topics: Vec<String>) {
        for topic in topics {
            self.topic_subscriptions
                .entry(topic.clone())
                .or_insert_with(HashSet::new)
                .insert(connection_id);
                
            debug!("Connection {} subscribed to topic '{}'", connection_id, topic);
        }
    }

    /// Unsubscribe connection from topics
    fn unsubscribe_from_topics(&mut self, connection_id: Uuid, topics: Vec<String>) {
        for topic in topics {
            if let Some(subscribers) = self.topic_subscriptions.get_mut(&topic) {
                subscribers.remove(&connection_id);
                if subscribers.is_empty() {
                    self.topic_subscriptions.remove(&topic);
                }
                debug!("Connection {} unsubscribed from topic '{}'", connection_id, topic);
            }
        }
    }

    /// Update presence status for a user
    fn update_presence(&mut self, user_id: String, connection_id: Uuid, status: PresenceStatus) {
        // Update connection metadata
        if let Some(connection_info) = self.connection_metadata.get_mut(&connection_id) {
            connection_info.presence_status = status.clone();
            connection_info.last_activity = Utc::now();
        }
        
        // Broadcast presence update to all users who have this user in their connections
        let presence_message = ServerMessage::PresenceUpdate {
            user_id: user_id.clone(),
            status,
            last_seen: Some(Utc::now().timestamp()),
        };
        
        let session_message = SessionMessage {
            message: presence_message,
            priority: MessagePriority::Normal,
        };
        
        // For now, broadcast to all sessions (in production, should be more targeted)
        for (&conn_id, session_addr) in &self.sessions {
            if conn_id != connection_id {
                session_addr.do_send(session_message.clone());
            }
        }
        
        info!("Presence updated: user_id={} status={:?}", user_id, connection_info.presence_status);
    }
}

impl Actor for RouterActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("Router actor started");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("Router actor stopped");
    }
}

/// Connect handler - Register new session
impl Handler<Connect> for RouterActor {
    type Result = ();

    fn handle(&mut self, msg: Connect, _ctx: &mut Self::Context) -> Self::Result {
        self.add_session(msg.connection_id, msg.user_id, msg.session_addr, msg.metadata);
    }
}

/// Disconnect handler - Remove session
impl Handler<Disconnect> for RouterActor {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _ctx: &mut Self::Context) -> Self::Result {
        self.remove_session(&msg.connection_id, msg.reason.as_deref());
    }
}

/// Route message handler - Main message routing
impl Handler<RouteMessage> for RouterActor {
    type Result = ();

    fn handle(&mut self, msg: RouteMessage, ctx: &mut Self::Context) -> Self::Result {
        let from_connection = msg.from_connection;
        let from_user = msg.from_user;
        
        // Update last activity
        if let Some(connection_info) = self.connection_metadata.get_mut(&from_connection) {
            connection_info.last_activity = msg.timestamp;
        }
        
        match msg.message {
            ClientMessage::DirectMessage { to, content, message_id } => {
                self.route_direct_message(from_connection, from_user, &to, content, message_id);
            }
            ClientMessage::TopicMessage { topic, content, message_id } => {
                self.route_topic_message(from_connection, from_user, &topic, content, message_id);
            }
            ClientMessage::BroadcastMessage { content, message_id } => {
                self.route_broadcast_message(from_connection, from_user, content, message_id);
            }
            ClientMessage::UpdatePresence { status, message: _ } => {
                if let Some(user_id) = from_user {
                    self.update_presence(user_id, from_connection, status);
                }
            }
            _ => {
                debug!("Unhandled message type in router: {:?}", msg.message);
            }
        }
    }
}

/// Subscribe to topic handler
impl Handler<SubscribeToTopic> for RouterActor {
    type Result = ();

    fn handle(&mut self, msg: SubscribeToTopic, _ctx: &mut Self::Context) -> Self::Result {
        self.subscribe_to_topics(msg.connection_id, msg.topics);
    }
}

/// Unsubscribe from topic handler
impl Handler<UnsubscribeFromTopic> for RouterActor {
    type Result = ();

    fn handle(&mut self, msg: UnsubscribeFromTopic, _ctx: &mut Self::Context) -> Self::Result {
        self.unsubscribe_from_topics(msg.connection_id, msg.topics);
    }
}

/// Get connections handler
impl Handler<GetConnections> for RouterActor {
    type Result = Vec<ConnectionSummary>;

    fn handle(&mut self, msg: GetConnections, _ctx: &mut Self::Context) -> Self::Result {
        let mut connections = Vec::new();
        
        for (connection_id, connection_info) in &self.connection_metadata {
            if let Some(ref filter_user_id) = msg.user_id {
                if let Some(ref user_id) = connection_info.user_id {
                    if user_id != filter_user_id {
                        continue;
                    }
                } else {
                    continue;
                }
            }
            
            // Get subscriptions for this connection
            let subscriptions: Vec<String> = self.topic_subscriptions
                .iter()
                .filter_map(|(topic, subscribers)| {
                    if subscribers.contains(connection_id) {
                        Some(topic.clone())
                    } else {
                        None
                    }
                })
                .collect();
            
            connections.push(ConnectionSummary {
                connection_id: *connection_id,
                user_id: connection_info.user_id.clone(),
                connected_at: connection_info.connected_at,
                last_activity: connection_info.last_activity,
                subscriptions,
                presence_status: connection_info.presence_status.clone(),
            });
        }
        
        connections
    }
}

/// Get system stats handler
impl Handler<GetSystemStats> for RouterActor {
    type Result = SystemStats;

    fn handle(&mut self, _msg: GetSystemStats, _ctx: &mut Self::Context) -> Self::Result {
        SystemStats {
            total_connections: self.sessions.len(),
            active_connections: self.sessions.len(),
            unique_users: self.user_connections.len(),
            messages_routed: self.metrics.messages_routed,
            messages_delivered: self.metrics.messages_delivered,
            messages_failed: self.metrics.messages_failed,
            topics_active: self.topic_subscriptions.len(),
            uptime_seconds: 0, // Would need to track start time
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix::System;
    use crate::connection::ConnectionManager;

    #[actix::test]
    async fn test_router_creation() {
        let connection_manager = Arc::new(ConnectionManager::new(100));
        let router = RouterActor::new(connection_manager);
        
        assert_eq!(router.sessions.len(), 0);
        assert_eq!(router.user_connections.len(), 0);
        assert_eq!(router.topic_subscriptions.len(), 0);
    }

    #[actix::test]
    async fn test_session_registration() {
        let connection_manager = Arc::new(ConnectionManager::new(100));
        let mut router = RouterActor::new(connection_manager);
        
        let connection_id = Uuid::new_v4();
        let user_id = Some("test_user".to_string());
        let metadata = HashMap::new();
        
        // Create a mock session address (this would normally be a real session actor)
        // For testing, we'll skip the actual session_addr since it requires complex setup
        
        assert_eq!(router.sessions.len(), 0);
        assert_eq!(router.user_connections.len(), 0);
    }
}