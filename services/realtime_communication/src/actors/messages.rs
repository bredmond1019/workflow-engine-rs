//! Actor Messages
//! 
//! Message types for inter-actor communication in the WebSocket system.

use actix::{Message, Recipient};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// WebSocket message types for client communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ClientMessage {
    // Connection lifecycle
    Connect { user_id: Option<String>, metadata: HashMap<String, String> },
    Disconnect { reason: Option<String> },
    
    // Subscription management
    Subscribe { topics: Vec<String> },
    Unsubscribe { topics: Vec<String> },
    
    // Messaging
    DirectMessage { to: String, content: serde_json::Value, message_id: Option<String> },
    TopicMessage { topic: String, content: serde_json::Value, message_id: Option<String> },
    BroadcastMessage { content: serde_json::Value, message_id: Option<String> },
    
    // Presence
    UpdatePresence { status: PresenceStatus, message: Option<String> },
    TypingStart { conversation_id: String },
    TypingEnd { conversation_id: String },
    
    // Control
    Ping { timestamp: u64 },
    Pong { timestamp: u64 },
    Ack { message_id: String },
}

/// Server-to-client message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ServerMessage {
    // Connection status
    Connected { connection_id: String, server_time: i64 },
    Disconnected { reason: String },
    
    // Message delivery
    MessageReceived { from: String, content: serde_json::Value, message_id: String, timestamp: i64 },
    TopicMessageReceived { topic: String, from: String, content: serde_json::Value, message_id: String, timestamp: i64 },
    BroadcastReceived { from: String, content: serde_json::Value, message_id: String, timestamp: i64 },
    
    // Notifications
    Notification { level: NotificationLevel, title: String, message: String, timestamp: i64 },
    
    // Presence updates
    PresenceUpdate { user_id: String, status: PresenceStatus, last_seen: Option<i64> },
    TypingIndicator { user_id: String, conversation_id: String, is_typing: bool },
    
    // System messages
    Error { code: u32, message: String },
    SystemMessage { message: String, level: String },
    DeliveryConfirmation { message_id: String, status: DeliveryStatus },
    
    // Control
    Ping { timestamp: u64 },
    Pong { timestamp: u64 },
}

/// Presence status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PresenceStatus {
    Online,
    Away,
    Busy,
    Offline,
}

/// Notification levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
    Success,
}

/// Message delivery status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeliveryStatus {
    Sent,
    Delivered,
    Read,
    Failed,
}

/// Internal actor messages
#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct Connect {
    pub connection_id: Uuid,
    pub user_id: Option<String>,
    pub session_addr: Recipient<SessionMessage>,
    pub metadata: HashMap<String, String>,
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub connection_id: Uuid,
    pub reason: Option<String>,
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct RouteMessage {
    pub from_connection: Uuid,
    pub from_user: Option<String>,
    pub message: ClientMessage,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct DeliverMessage {
    pub to_connection: Uuid,
    pub message: ServerMessage,
    pub priority: MessagePriority,
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct SessionMessage {
    pub message: ServerMessage,
    pub priority: MessagePriority,
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct UpdatePresence {
    pub user_id: String,
    pub connection_id: Uuid,
    pub status: PresenceStatus,
    pub message: Option<String>,
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct SubscribeToTopic {
    pub connection_id: Uuid,
    pub topics: Vec<String>,
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct UnsubscribeFromTopic {
    pub connection_id: Uuid,
    pub topics: Vec<String>,
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "Vec<ConnectionSummary>")]
pub struct GetConnections {
    pub user_id: Option<String>,
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "SystemStats")]
pub struct GetSystemStats;

/// Message priority levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Connection summary for status queries
#[derive(Debug, Clone, Serialize)]
pub struct ConnectionSummary {
    pub connection_id: Uuid,
    pub user_id: Option<String>,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub subscriptions: Vec<String>,
    pub presence_status: PresenceStatus,
}

/// System statistics
#[derive(Debug, Clone, Serialize)]
pub struct SystemStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub unique_users: usize,
    pub messages_routed: u64,
    pub messages_delivered: u64,
    pub messages_failed: u64,
    pub topics_active: usize,
    pub uptime_seconds: u64,
}

/// Message persistence request
#[derive(Message, Debug, Clone)]
#[rtype(result = "Result<(), String>")]
pub struct PersistMessage {
    pub message_id: String,
    pub from_user: Option<String>,
    pub to_user: Option<String>,
    pub topic: Option<String>,
    pub content: serde_json::Value,
    pub message_type: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub delivery_status: DeliveryStatus,
    pub metadata: HashMap<String, String>,
}

/// Message history request
#[derive(Message, Debug, Clone)]
#[rtype(result = "Result<Vec<PersistedMessage>, String>")]
pub struct GetMessageHistory {
    pub conversation_id: String,
    pub user_id: String,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub before_timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

/// Persisted message representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedMessage {
    pub id: String,
    pub from_user: Option<String>,
    pub to_user: Option<String>,
    pub topic: Option<String>,
    pub content: serde_json::Value,
    pub message_type: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub delivery_status: DeliveryStatus,
    pub metadata: HashMap<String, String>,
}

/// Typing indicator message
#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct TypingIndicator {
    pub user_id: String,
    pub conversation_id: String,
    pub is_typing: bool,
    pub connection_id: Uuid,
}

/// Heartbeat message for keeping connections alive
#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct Heartbeat {
    pub connection_id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Session cleanup message
#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct CleanupSession {
    pub connection_id: Uuid,
    pub reason: String,
}

/// Notification delivery request
#[derive(Message, Debug, Clone)]
#[rtype(result = "Result<(), String>")]
pub struct DeliverNotification {
    pub user_id: String,
    pub notification: ServerMessage,
    pub persistence_required: bool,
    pub retry_count: u32,
}

impl Default for MessagePriority {
    fn default() -> Self {
        MessagePriority::Normal
    }
}

impl From<&str> for MessagePriority {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "low" => MessagePriority::Low,
            "high" => MessagePriority::High,
            "critical" => MessagePriority::Critical,
            _ => MessagePriority::Normal,
        }
    }
}