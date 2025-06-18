//! WebSocket Actor Implementation
//! 
//! Actor-based WebSocket connection handler providing message processing,
//! heartbeat management, and graceful connection lifecycle.

use actix_ws::{Message, MessageStream, Session};
use uuid::Uuid;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio_stream::StreamExt;
use tracing::{info, warn, error, debug};
use serde::{Deserialize, Serialize};

use crate::connection::{ConnectionManager, ConnectionInfo};
use crate::server::{ServerConfig, ServerMetrics};

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    // Control messages
    Ping { timestamp: u64 },
    Pong { timestamp: u64 },
    Subscribe { topics: Vec<String> },
    Unsubscribe { topics: Vec<String> },
    
    // Data messages
    Broadcast { topic: String, payload: serde_json::Value },
    DirectMessage { target_user: String, payload: serde_json::Value },
    
    // Status messages
    Error { code: u32, message: String },
    Ack { message_id: String },
    Status { status: String, details: Option<serde_json::Value> },
}

/// WebSocket actor for handling individual connections
pub struct WebSocketActor {
    connection_id: Uuid,
    session: Session,
    connection_manager: Arc<ConnectionManager>,
    metrics: Arc<ServerMetrics>,
    config: ServerConfig,
    last_heartbeat: Instant,
}

impl WebSocketActor {
    pub fn new(
        connection_id: Uuid,
        session: Session,
        connection_manager: Arc<ConnectionManager>,
        metrics: Arc<ServerMetrics>,
        config: ServerConfig,
    ) -> Self {
        Self {
            connection_id,
            session,
            connection_manager,
            metrics,
            config,
            last_heartbeat: Instant::now(),
        }
    }

    /// Run the WebSocket actor
    pub async fn run(mut self, mut stream: MessageStream) -> Result<(), ActorError> {
        info!("WebSocket actor started for connection {}", self.connection_id);

        // Create connection info and add to manager
        let connection_info = ConnectionInfo::new(self.connection_id);
        if let Err(e) = self.connection_manager.add_connection(connection_info).await {
            error!("Failed to add connection {}: {}", self.connection_id, e);
            return Err(ActorError::ConnectionSetupFailed(e.to_string()));
        }

        // Send welcome message
        if let Err(e) = self.send_welcome().await {
            warn!("Failed to send welcome message: {}", e);
        }

        // Start heartbeat task
        let heartbeat_connection_id = self.connection_id;
        let heartbeat_session = self.session.clone();
        let heartbeat_interval = self.config.heartbeat_interval;
        
        let heartbeat_handle = tokio::spawn(async move {
            Self::heartbeat_task(heartbeat_connection_id, heartbeat_session, heartbeat_interval).await;
        });

        // Main message processing loop
        let result = self.message_loop(&mut stream).await;

        // Cleanup
        heartbeat_handle.abort();
        self.cleanup().await;

        result
    }

    /// Main message processing loop
    async fn message_loop(&mut self, stream: &mut MessageStream) -> Result<(), ActorError> {
        while let Some(msg_result) = stream.next().await {
            match msg_result {
                Ok(msg) => {
                    self.last_heartbeat = Instant::now();
                    self.connection_manager.update_activity(&self.connection_id).await;
                    self.metrics.increment_messages_received().await;

                    if let Err(e) = self.handle_message(msg).await {
                        error!("Error handling message for connection {}: {}", self.connection_id, e);
                        self.metrics.increment_errors().await;
                        
                        // Send error response
                        let error_msg = WsMessage::Error {
                            code: 500,
                            message: "Internal server error".to_string(),
                        };
                        if let Err(send_err) = self.send_message(&error_msg).await {
                            error!("Failed to send error response: {}", send_err);
                            break;
                        }
                    }
                }
                Err(e) => {
                    error!("WebSocket stream error for connection {}: {}", self.connection_id, e);
                    self.metrics.increment_errors().await;
                    break;
                }
            }

            // Check for timeout
            if self.last_heartbeat.elapsed() > self.config.client_timeout {
                warn!("Connection {} timed out", self.connection_id);
                break;
            }
        }

        Ok(())
    }

    /// Handle individual WebSocket messages
    async fn handle_message(&mut self, msg: Message) -> Result<(), ActorError> {
        match msg {
            Message::Text(text) => {
                debug!("Received text message from {}: {}", self.connection_id, text);
                self.handle_text_message(&text).await
            }
            Message::Binary(bytes) => {
                debug!("Received binary message from {} ({} bytes)", self.connection_id, bytes.len());
                self.handle_binary_message(&bytes).await
            }
            Message::Ping(bytes) => {
                debug!("Received ping from {}", self.connection_id);
                self.session.pong(&bytes).await.map_err(|e| ActorError::SendFailed(e.to_string()))?;
                Ok(())
            }
            Message::Pong(_) => {
                debug!("Received pong from {}", self.connection_id);
                Ok(())
            }
            Message::Close(reason) => {
                info!("Connection {} closed: {:?}", self.connection_id, reason);
                Err(ActorError::ConnectionClosed)
            }
            _ => {
                warn!("Received unexpected message type from {}", self.connection_id);
                Ok(())
            }
        }
    }

    /// Handle text messages (JSON protocol)
    async fn handle_text_message(&mut self, text: &str) -> Result<(), ActorError> {
        let ws_message: WsMessage = serde_json::from_str(text)
            .map_err(|e| ActorError::InvalidMessage(format!("Failed to parse JSON: {}", e)))?;

        match ws_message {
            WsMessage::Ping { timestamp } => {
                let pong = WsMessage::Pong { timestamp };
                self.send_message(&pong).await?;
            }
            WsMessage::Subscribe { topics } => {
                for topic in topics {
                    if let Err(e) = self.connection_manager
                        .subscribe_to_topic(&self.connection_id, topic.clone()).await {
                        warn!("Failed to subscribe to topic {}: {}", topic, e);
                    } else {
                        info!("Connection {} subscribed to topic {}", self.connection_id, topic);
                    }
                }
                
                let ack = WsMessage::Status {
                    status: "subscribed".to_string(),
                    details: None,
                };
                self.send_message(&ack).await?;
            }
            WsMessage::Unsubscribe { topics } => {
                for topic in topics {
                    if let Err(e) = self.connection_manager
                        .unsubscribe_from_topic(&self.connection_id, &topic).await {
                        warn!("Failed to unsubscribe from topic {}: {}", topic, e);
                    } else {
                        info!("Connection {} unsubscribed from topic {}", self.connection_id, topic);
                    }
                }
                
                let ack = WsMessage::Status {
                    status: "unsubscribed".to_string(),
                    details: None,
                };
                self.send_message(&ack).await?;
            }
            WsMessage::Broadcast { topic, payload } => {
                // Handle broadcast messages (relay to other subscribers)
                self.handle_broadcast(&topic, &payload).await?;
            }
            WsMessage::DirectMessage { target_user, payload } => {
                // Handle direct messages
                self.handle_direct_message(&target_user, &payload).await?;
            }
            _ => {
                warn!("Received unexpected message type from {}: {:?}", self.connection_id, ws_message);
            }
        }

        Ok(())
    }

    /// Handle binary messages
    async fn handle_binary_message(&mut self, _bytes: &[u8]) -> Result<(), ActorError> {
        // For now, we'll just acknowledge binary messages
        // In the future, this could handle binary protocols or file transfers
        let status = WsMessage::Status {
            status: "binary_received".to_string(),
            details: Some(serde_json::json!({"size": _bytes.len()})),
        };
        self.send_message(&status).await
    }

    /// Handle broadcast messages
    async fn handle_broadcast(&mut self, topic: &str, payload: &serde_json::Value) -> Result<(), ActorError> {
        info!("Broadcasting message to topic {} from connection {}", topic, self.connection_id);
        
        // Get all subscribers to this topic
        let subscribers = self.connection_manager.get_topic_subscribers(topic).await;
        
        let _broadcast_msg = WsMessage::Broadcast {
            topic: topic.to_string(),
            payload: payload.clone(),
        };
        
        // Send to all subscribers (except sender)
        for subscriber_id in subscribers {
            if subscriber_id != self.connection_id {
                // In a real implementation, we'd need a way to send messages to other connections
                // For now, we'll just log that we would send the message
                debug!("Would send broadcast to connection {}", subscriber_id);
            }
        }
        
        // Send acknowledgment
        let ack = WsMessage::Ack {
            message_id: format!("broadcast_{}", uuid::Uuid::new_v4()),
        };
        self.send_message(&ack).await
    }

    /// Handle direct messages
    async fn handle_direct_message(&mut self, target_user: &str, _payload: &serde_json::Value) -> Result<(), ActorError> {
        info!("Sending direct message to user {} from connection {}", target_user, self.connection_id);
        
        // Get target user's connections
        let target_connections = self.connection_manager.get_user_connections(target_user).await;
        
        if target_connections.is_empty() {
            let error = WsMessage::Error {
                code: 404,
                message: format!("User {} not found or not connected", target_user),
            };
            return self.send_message(&error).await;
        }
        
        // Send to all of target user's connections
        for connection_id in target_connections {
            // In a real implementation, we'd need a way to send messages to other connections
            debug!("Would send direct message to connection {}", connection_id);
        }
        
        // Send acknowledgment
        let ack = WsMessage::Ack {
            message_id: format!("direct_{}", uuid::Uuid::new_v4()),
        };
        self.send_message(&ack).await
    }

    /// Send a message to the WebSocket client
    async fn send_message(&mut self, message: &WsMessage) -> Result<(), ActorError> {
        let json = serde_json::to_string(message)
            .map_err(|e| ActorError::SerializationFailed(e.to_string()))?;
        
        self.session.text(json).await
            .map_err(|e| ActorError::SendFailed(e.to_string()))?;
        
        self.metrics.increment_messages_sent().await;
        Ok(())
    }

    /// Send welcome message to newly connected client
    async fn send_welcome(&mut self) -> Result<(), ActorError> {
        let welcome = WsMessage::Status {
            status: "connected".to_string(),
            details: Some(serde_json::json!({
                "connection_id": self.connection_id,
                "server_time": chrono::Utc::now().timestamp(),
                "protocol_version": "1.0"
            })),
        };
        self.send_message(&welcome).await
    }

    /// Heartbeat task to send periodic pings
    async fn heartbeat_task(connection_id: Uuid, mut session: Session, interval: Duration) {
        let mut heartbeat_timer = interval_at(tokio::time::Instant::now(), interval);
        
        loop {
            heartbeat_timer.tick().await;
            
            let ping_msg = WsMessage::Ping {
                timestamp: chrono::Utc::now().timestamp() as u64,
            };
            
            if let Ok(json) = serde_json::to_string(&ping_msg) {
                if let Err(e) = session.text(json).await {
                    error!("Failed to send heartbeat for connection {}: {}", connection_id, e);
                    break;
                }
            }
        }
    }

    /// Cleanup connection resources
    async fn cleanup(&self) {
        info!("Cleaning up connection {}", self.connection_id);
        
        // Remove connection from manager
        if let Some(connection_info) = self.connection_manager.remove_connection(&self.connection_id).await {
            info!("Connection {} cleaned up: {:?}", self.connection_id, connection_info.state);
        }
        
        // Update metrics
        self.metrics.decrement_connections().await;
    }
}

/// Actor-specific errors
#[derive(Debug, thiserror::Error)]
pub enum ActorError {
    #[error("Connection setup failed: {0}")]
    ConnectionSetupFailed(String),
    #[error("Invalid message: {0}")]
    InvalidMessage(String),
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),
    #[error("Send failed: {0}")]
    SendFailed(String),
    #[error("Connection closed")]
    ConnectionClosed,
    #[error("Connection timeout")]
    ConnectionTimeout,
    #[error("Actor error: {0}")]
    Generic(String),
}

// Helper function for interval_at (until tokio::time::interval_at is stable)
fn interval_at(_start: tokio::time::Instant, period: Duration) -> tokio::time::Interval {
    let mut interval = tokio::time::interval(period);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    interval
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ws_message_serialization() {
        let ping = WsMessage::Ping { timestamp: 1234567890 };
        let json = serde_json::to_string(&ping).unwrap();
        let deserialized: WsMessage = serde_json::from_str(&json).unwrap();
        
        match deserialized {
            WsMessage::Ping { timestamp } => assert_eq!(timestamp, 1234567890),
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_ws_message_subscribe() {
        let subscribe = WsMessage::Subscribe {
            topics: vec!["topic1".to_string(), "topic2".to_string()],
        };
        
        let json = serde_json::to_string(&subscribe).unwrap();
        let deserialized: WsMessage = serde_json::from_str(&json).unwrap();
        
        match deserialized {
            WsMessage::Subscribe { topics } => {
                assert_eq!(topics.len(), 2);
                assert!(topics.contains(&"topic1".to_string()));
                assert!(topics.contains(&"topic2".to_string()));
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_ws_message_broadcast() {
        let payload = serde_json::json!({"message": "Hello, world!"});
        let broadcast = WsMessage::Broadcast {
            topic: "general".to_string(),
            payload: payload.clone(),
        };
        
        let json = serde_json::to_string(&broadcast).unwrap();
        let deserialized: WsMessage = serde_json::from_str(&json).unwrap();
        
        match deserialized {
            WsMessage::Broadcast { topic, payload: deserialized_payload } => {
                assert_eq!(topic, "general");
                assert_eq!(deserialized_payload, payload);
            }
            _ => panic!("Wrong message type"),
        }
    }
}