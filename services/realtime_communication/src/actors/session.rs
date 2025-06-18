//! Session Actor
//! 
//! Individual WebSocket session actor that manages a single connection
//! with message buffering, heartbeat handling, and graceful reconnection.

use actix::{Actor, ActorContext, Addr, Context, Handler, AsyncContext, StreamHandler, Running};
use actix_web_actors::ws;
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::sync::Arc;

use super::messages::*;
use super::router::RouterActor;

/// Session Actor - Manages individual WebSocket connection
pub struct SessionActor {
    /// Unique connection identifier
    connection_id: Uuid,
    
    /// Associated user ID (if authenticated)
    user_id: Option<String>,
    
    /// Router actor address for message routing
    router_addr: Addr<RouterActor>,
    
    /// Connection state
    state: SessionState,
    
    /// Message buffer for delivery reliability
    message_buffer: VecDeque<BufferedMessage>,
    
    /// Heartbeat management
    heartbeat: HeartbeatManager,
    
    /// Session metrics
    metrics: SessionMetrics,
    
    /// Configuration
    config: SessionConfig,
    
    /// Redis client for session persistence
    redis_client: Option<Arc<redis::Client>>,
}

/// Session state enumeration
#[derive(Debug, Clone, PartialEq)]
enum SessionState {
    Connecting,
    Connected,
    Authenticated(String), // Contains user_id
    Disconnecting,
    Disconnected,
    Error(String),
}

/// Buffered message for reliability
#[derive(Debug, Clone)]
struct BufferedMessage {
    message: ServerMessage,
    priority: MessagePriority,
    timestamp: DateTime<Utc>,
    retry_count: u32,
    max_retries: u32,
}

/// Heartbeat manager
#[derive(Debug)]
struct HeartbeatManager {
    last_heartbeat: Instant,
    last_pong: Instant,
    missed_heartbeats: u32,
    max_missed_heartbeats: u32,
}

/// Session metrics
#[derive(Debug, Default)]
struct SessionMetrics {
    messages_sent: u64,
    messages_received: u64,
    messages_buffered: u64,
    bytes_sent: u64,
    bytes_received: u64,
    connection_duration: Duration,
    last_activity: Instant,
}

/// Session configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub heartbeat_interval: Duration,
    pub client_timeout: Duration,
    pub max_missed_heartbeats: u32,
    pub max_message_buffer_size: usize,
    pub message_retry_attempts: u32,
    pub enable_message_buffering: bool,
    pub enable_redis_persistence: bool,
    pub max_frame_size: usize,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval: Duration::from_secs(30),
            client_timeout: Duration::from_secs(60),
            max_missed_heartbeats: 3,
            max_message_buffer_size: 1000,
            message_retry_attempts: 3,
            enable_message_buffering: true,
            enable_redis_persistence: false,
            max_frame_size: 64 * 1024, // 64KB
        }
    }
}

impl SessionActor {
    pub fn new(
        connection_id: Uuid,
        router_addr: Addr<RouterActor>,
        config: SessionConfig,
        redis_client: Option<Arc<redis::Client>>,
    ) -> Self {
        let now = Instant::now();
        
        Self {
            connection_id,
            user_id: None,
            router_addr,
            state: SessionState::Connecting,
            message_buffer: VecDeque::new(),
            heartbeat: HeartbeatManager {
                last_heartbeat: now,
                last_pong: now,
                missed_heartbeats: 0,
                max_missed_heartbeats: config.max_missed_heartbeats,
            },
            metrics: SessionMetrics {
                last_activity: now,
                ..Default::default()
            },
            config,
            redis_client,
        }
    }

    /// Start heartbeat process
    fn start_heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(self.config.heartbeat_interval, |act, ctx| {
            act.check_heartbeat(ctx);
        });
    }

    /// Check heartbeat and send ping if needed
    fn check_heartbeat(&mut self, ctx: &mut ws::WebsocketContext<Self>) {
        let now = Instant::now();
        
        // Check if client has timed out
        if now.duration_since(self.heartbeat.last_pong) > self.config.client_timeout {
            warn!("Connection {} timed out", self.connection_id);
            self.disconnect(ctx, Some("Heartbeat timeout".to_string()));
            return;
        }
        
        // Send ping if it's time
        if now.duration_since(self.heartbeat.last_heartbeat) >= self.config.heartbeat_interval {
            let ping_message = ServerMessage::Ping {
                timestamp: Utc::now().timestamp() as u64,
            };
            
            if let Err(e) = self.send_message_internal(ctx, ping_message) {
                error!("Failed to send heartbeat ping: {}", e);
                self.disconnect(ctx, Some("Heartbeat send failed".to_string()));
            } else {
                self.heartbeat.last_heartbeat = now;
                self.heartbeat.missed_heartbeats += 1;
                
                if self.heartbeat.missed_heartbeats > self.heartbeat.max_missed_heartbeats {
                    warn!("Too many missed heartbeats for connection {}", self.connection_id);
                    self.disconnect(ctx, Some("Too many missed heartbeats".to_string()));
                }
            }
        }
    }

    /// Send message to client
    fn send_message_internal(
        &mut self,
        ctx: &mut ws::WebsocketContext<Self>,
        message: ServerMessage,
    ) -> Result<(), String> {
        let json = serde_json::to_string(&message)
            .map_err(|e| format!("Serialization error: {}", e))?;
        
        ctx.text(json.clone());
        
        // Update metrics
        self.metrics.messages_sent += 1;
        self.metrics.bytes_sent += json.len() as u64;
        self.metrics.last_activity = Instant::now();
        
        debug!("Sent message to connection {}: {:?}", self.connection_id, message);
        Ok(())
    }

    /// Buffer message for later delivery
    fn buffer_message(&mut self, message: ServerMessage, priority: MessagePriority) {
        if !self.config.enable_message_buffering {
            return;
        }
        
        // Check buffer size limit
        if self.message_buffer.len() >= self.config.max_message_buffer_size {
            // Remove oldest low-priority message
            if let Some(index) = self.message_buffer
                .iter()
                .position(|msg| msg.priority == MessagePriority::Low)
            {
                self.message_buffer.remove(index);
            } else {
                // Remove oldest message if no low-priority messages
                self.message_buffer.pop_front();
            }
        }
        
        let buffered_message = BufferedMessage {
            message,
            priority,
            timestamp: Utc::now(),
            retry_count: 0,
            max_retries: self.config.message_retry_attempts,
        };
        
        // Insert based on priority
        let insert_index = self.message_buffer
            .iter()
            .position(|msg| msg.priority < priority)
            .unwrap_or(self.message_buffer.len());
        
        self.message_buffer.insert(insert_index, buffered_message);
        self.metrics.messages_buffered += 1;
        
        debug!("Buffered message for connection {}, buffer size: {}", 
               self.connection_id, self.message_buffer.len());
    }

    /// Flush message buffer
    fn flush_message_buffer(&mut self, ctx: &mut ws::WebsocketContext<Self>) {
        let mut to_remove = Vec::new();
        
        for (index, buffered_message) in self.message_buffer.iter_mut().enumerate() {
            match self.send_message_internal(ctx, buffered_message.message.clone()) {
                Ok(()) => {
                    to_remove.push(index);
                }
                Err(e) => {
                    buffered_message.retry_count += 1;
                    warn!("Failed to send buffered message (attempt {}): {}", 
                          buffered_message.retry_count, e);
                    
                    if buffered_message.retry_count >= buffered_message.max_retries {
                        error!("Max retries reached for buffered message, dropping");
                        to_remove.push(index);
                    }
                }
            }
        }
        
        // Remove successfully sent or failed messages (in reverse order to maintain indices)
        for &index in to_remove.iter().rev() {
            self.message_buffer.remove(index);
        }
        
        if !to_remove.is_empty() {
            debug!("Flushed {} messages from buffer, {} remaining", 
                   to_remove.len(), self.message_buffer.len());
        }
    }

    /// Handle client message
    fn handle_client_message(&mut self, ctx: &mut ws::WebsocketContext<Self>, text: &str) {
        self.metrics.messages_received += 1;
        self.metrics.bytes_received += text.len() as u64;
        self.metrics.last_activity = Instant::now();
        
        let client_message: ClientMessage = match serde_json::from_str(text) {
            Ok(msg) => msg,
            Err(e) => {
                warn!("Failed to parse client message: {}", e);
                let error_message = ServerMessage::Error {
                    code: 400,
                    message: "Invalid message format".to_string(),
                };
                if let Err(send_err) = self.send_message_internal(ctx, error_message) {
                    error!("Failed to send error response: {}", send_err);
                }
                return;
            }
        };
        
        debug!("Received client message: {:?}", client_message);
        
        match &client_message {
            ClientMessage::Connect { user_id, metadata } => {
                self.handle_connect(ctx, user_id.clone(), metadata.clone());
            }
            ClientMessage::Disconnect { reason } => {
                self.disconnect(ctx, reason.clone());
            }
            ClientMessage::Subscribe { topics } => {
                self.handle_subscribe(topics.clone());
            }
            ClientMessage::Unsubscribe { topics } => {
                self.handle_unsubscribe(topics.clone());
            }
            ClientMessage::Pong { timestamp: _ } => {
                self.heartbeat.last_pong = Instant::now();
                self.heartbeat.missed_heartbeats = 0;
                debug!("Received pong from connection {}", self.connection_id);
            }
            _ => {
                // Route message through router
                let route_message = RouteMessage {
                    from_connection: self.connection_id,
                    from_user: self.user_id.clone(),
                    message: client_message,
                    timestamp: Utc::now(),
                };
                self.router_addr.do_send(route_message);
            }
        }
    }

    /// Handle connect message
    fn handle_connect(
        &mut self,
        ctx: &mut ws::WebsocketContext<Self>,
        user_id: Option<String>,
        metadata: std::collections::HashMap<String, String>,
    ) {
        self.user_id = user_id.clone();
        self.state = match user_id.clone() {
            Some(uid) => SessionState::Authenticated(uid),
            None => SessionState::Connected,
        };
        
        // Register with router
        let connect_message = Connect {
            connection_id: self.connection_id,
            user_id: user_id.clone(),
            session_addr: ctx.address().recipient(),
            metadata,
        };
        self.router_addr.do_send(connect_message);
        
        // Send connection confirmation
        let connected_message = ServerMessage::Connected {
            connection_id: self.connection_id.to_string(),
            server_time: Utc::now().timestamp(),
        };
        
        if let Err(e) = self.send_message_internal(ctx, connected_message) {
            error!("Failed to send connection confirmation: {}", e);
        }
        
        // Flush any buffered messages
        self.flush_message_buffer(ctx);
        
        info!("Session connected: connection_id={}, user_id={:?}", 
              self.connection_id, user_id);
    }

    /// Handle subscribe message
    fn handle_subscribe(&mut self, topics: Vec<String>) {
        let subscribe_message = SubscribeToTopic {
            connection_id: self.connection_id,
            topics: topics.clone(),
        };
        self.router_addr.do_send(subscribe_message);
        
        info!("Connection {} subscribed to topics: {:?}", self.connection_id, topics);
    }

    /// Handle unsubscribe message
    fn handle_unsubscribe(&mut self, topics: Vec<String>) {
        let unsubscribe_message = UnsubscribeFromTopic {
            connection_id: self.connection_id,
            topics: topics.clone(),
        };
        self.router_addr.do_send(unsubscribe_message);
        
        info!("Connection {} unsubscribed from topics: {:?}", self.connection_id, topics);
    }

    /// Disconnect session
    fn disconnect(&mut self, ctx: &mut ws::WebsocketContext<Self>, reason: Option<String>) {
        self.state = SessionState::Disconnecting;
        
        // Notify router
        let disconnect_message = Disconnect {
            connection_id: self.connection_id,
            reason: reason.clone(),
        };
        self.router_addr.do_send(disconnect_message);
        
        // Send disconnection message to client
        if let Some(reason) = reason.clone() {
            let disconnect_msg = ServerMessage::Disconnected { reason };
            let _ = self.send_message_internal(ctx, disconnect_msg);
        }
        
        info!("Session disconnecting: connection_id={}, reason={:?}", 
              self.connection_id, reason);
        
        // Stop the actor
        ctx.stop();
    }

    /// Persist session state to Redis
    async fn persist_session_state(&self) -> Result<(), String> {
        if !self.config.enable_redis_persistence {
            return Ok(());
        }
        
        if let Some(redis_client) = &self.redis_client {
            // Implementation would go here
            // For now, just log that we would persist
            debug!("Would persist session state for connection {}", self.connection_id);
        }
        
        Ok(())
    }
}

impl Actor for SessionActor {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Session actor started: connection_id={}", self.connection_id);
        self.start_heartbeat(ctx);
        self.metrics.connection_duration = Duration::from_secs(0);
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        info!("Session actor stopping: connection_id={}", self.connection_id);
        self.state = SessionState::Disconnected;
        Running::Stop
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("Session actor stopped: connection_id={}", self.connection_id);
        self.metrics.connection_duration = self.metrics.last_activity.elapsed();
        
        // Log final metrics
        debug!("Session metrics - Messages sent: {}, received: {}, buffered: {}, duration: {:?}",
               self.metrics.messages_sent,
               self.metrics.messages_received,
               self.metrics.messages_buffered,
               self.metrics.connection_duration);
    }
}

/// WebSocket message handler
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for SessionActor {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                self.handle_client_message(ctx, &text);
            }
            Ok(ws::Message::Binary(bin)) => {
                debug!("Received binary message ({} bytes) from connection {}", 
                       bin.len(), self.connection_id);
                // For now, just acknowledge binary messages
                let response = ServerMessage::SystemMessage {
                    message: format!("Binary message received ({} bytes)", bin.len()),
                    level: "info".to_string(),
                };
                if let Err(e) = self.send_message_internal(ctx, response) {
                    error!("Failed to send binary acknowledgment: {}", e);
                }
            }
            Ok(ws::Message::Ping(msg)) => {
                debug!("Received ping from connection {}", self.connection_id);
                ctx.pong(&msg);
                self.heartbeat.last_pong = Instant::now();
            }
            Ok(ws::Message::Pong(_)) => {
                debug!("Received pong from connection {}", self.connection_id);
                self.heartbeat.last_pong = Instant::now();
                self.heartbeat.missed_heartbeats = 0;
            }
            Ok(ws::Message::Close(reason)) => {
                info!("WebSocket close received for connection {}: {:?}", 
                      self.connection_id, reason);
                self.disconnect(ctx, reason.map(|r| r.description().unwrap_or("Unknown").to_string()));
            }
            Err(e) => {
                error!("WebSocket protocol error for connection {}: {}", 
                       self.connection_id, e);
                self.disconnect(ctx, Some(format!("Protocol error: {}", e)));
            }
            _ => {
                debug!("Unhandled WebSocket message type for connection {}", 
                       self.connection_id);
            }
        }
    }
}

/// Session message handler from router
impl Handler<SessionMessage> for SessionActor {
    type Result = ();

    fn handle(&mut self, msg: SessionMessage, ctx: &mut Self::Context) -> Self::Result {
        // If connected, send immediately; otherwise buffer
        match self.state {
            SessionState::Connected | SessionState::Authenticated(_) => {
                if let Err(e) = self.send_message_internal(ctx, msg.message) {
                    error!("Failed to send message to connection {}: {}", 
                           self.connection_id, e);
                    // Buffer the message for retry
                    self.buffer_message(msg.message, msg.priority);
                }
            }
            _ => {
                // Buffer message for later delivery
                self.buffer_message(msg.message, msg.priority);
                debug!("Buffered message for disconnected session {}", self.connection_id);
            }
        }
    }
}

/// Heartbeat message handler
impl Handler<Heartbeat> for SessionActor {
    type Result = ();

    fn handle(&mut self, _msg: Heartbeat, ctx: &mut Self::Context) -> Self::Result {
        self.check_heartbeat(ctx);
    }
}

/// Cleanup session handler
impl Handler<CleanupSession> for SessionActor {
    type Result = ();

    fn handle(&mut self, msg: CleanupSession, ctx: &mut Self::Context) -> Self::Result {
        warn!("Session cleanup requested: connection_id={}, reason={}", 
              self.connection_id, msg.reason);
        self.disconnect(ctx, Some(msg.reason));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix::System;

    #[test]
    fn test_session_config_default() {
        let config = SessionConfig::default();
        assert_eq!(config.heartbeat_interval, Duration::from_secs(30));
        assert_eq!(config.client_timeout, Duration::from_secs(60));
        assert_eq!(config.max_missed_heartbeats, 3);
        assert_eq!(config.max_message_buffer_size, 1000);
        assert_eq!(config.message_retry_attempts, 3);
        assert!(config.enable_message_buffering);
        assert!(!config.enable_redis_persistence);
    }

    #[test]
    fn test_buffered_message() {
        let message = ServerMessage::SystemMessage {
            message: "test".to_string(),
            level: "info".to_string(),
        };
        
        let buffered = BufferedMessage {
            message: message.clone(),
            priority: MessagePriority::Normal,
            timestamp: Utc::now(),
            retry_count: 0,
            max_retries: 3,
        };
        
        assert_eq!(buffered.retry_count, 0);
        assert_eq!(buffered.max_retries, 3);
        assert_eq!(buffered.priority, MessagePriority::Normal);
    }

    #[test]
    fn test_heartbeat_manager() {
        let now = Instant::now();
        let heartbeat = HeartbeatManager {
            last_heartbeat: now,
            last_pong: now,
            missed_heartbeats: 0,
            max_missed_heartbeats: 3,
        };
        
        assert_eq!(heartbeat.missed_heartbeats, 0);
        assert_eq!(heartbeat.max_missed_heartbeats, 3);
    }
}