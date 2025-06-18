//! Message Types and Definitions
//! 
//! Defines message types for different categories: Progress, Notification, Agent, Control

use uuid::Uuid;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Message priority levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MessagePriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

impl Default for MessagePriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Message delivery options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryOptions {
    pub priority: MessagePriority,
    pub ttl_seconds: Option<u64>,
    pub retry_count: u8,
    pub require_acknowledgment: bool,
    pub broadcast: bool,
    pub persist_offline: bool,
}

impl Default for DeliveryOptions {
    fn default() -> Self {
        Self {
            priority: MessagePriority::Normal,
            ttl_seconds: Some(3600), // 1 hour
            retry_count: 3,
            require_acknowledgment: false,
            broadcast: false,
            persist_offline: true,
        }
    }
}

/// Progress update messages for long-running operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressMessage {
    pub operation_id: String,
    pub progress_percent: f32,
    pub current_step: String,
    pub total_steps: Option<u32>,
    pub estimated_remaining_seconds: Option<u32>,
    pub status: ProgressStatus,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProgressStatus {
    Starting,
    InProgress,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

/// Notification messages for user alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMessage {
    pub title: String,
    pub message: String,
    pub level: NotificationLevel,
    pub category: String,
    pub action_url: Option<String>,
    pub action_text: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

/// Agent communication messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub agent_id: String,
    pub agent_type: String,
    pub content_type: AgentContentType,
    pub content: serde_json::Value,
    pub conversation_id: Option<String>,
    pub parent_message_id: Option<Uuid>,
    pub capabilities: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentContentType {
    Text,
    Json,
    Binary,
    Code,
    Markdown,
    Image,
    Audio,
    Video,
}

/// Control messages for system operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlMessage {
    pub command: ControlCommand,
    pub target: ControlTarget,
    pub parameters: HashMap<String, serde_json::Value>,
    pub requires_confirmation: bool,
    pub timeout_seconds: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlCommand {
    Start,
    Stop,
    Pause,
    Resume,
    Restart,
    Configure,
    Status,
    Reset,
    Shutdown,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlTarget {
    System,
    Service(String),
    Agent(String),
    Connection(Uuid),
    Topic(String),
    All,
}

/// Unified routing message envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingMessage {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub sender_id: Option<String>,
    pub sender_type: SenderType,
    pub routing_key: String,
    pub topic: String,
    pub target_users: Vec<String>,
    pub target_connections: Vec<Uuid>,
    pub message_type: MessageType,
    pub delivery_options: DeliveryOptions,
    pub correlation_id: Option<String>,
    pub trace_id: Option<String>,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SenderType {
    User,
    Agent,
    System,
    Service,
    External,
}

/// Message type variants
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum MessageType {
    Progress(ProgressMessage),
    Notification(NotificationMessage),
    Agent(AgentMessage),
    Control(ControlMessage),
    Heartbeat,
    Ack { message_id: Uuid },
    Error { code: u32, message: String, details: Option<serde_json::Value> },
    Custom { type_name: String, data: serde_json::Value },
}

impl RoutingMessage {
    /// Create a new routing message
    pub fn new(
        routing_key: String,
        topic: String,
        message_type: MessageType,
        sender_id: Option<String>,
        sender_type: SenderType,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            sender_id,
            sender_type,
            routing_key,
            topic,
            target_users: Vec::new(),
            target_connections: Vec::new(),
            message_type,
            delivery_options: DeliveryOptions::default(),
            correlation_id: None,
            trace_id: None,
            headers: HashMap::new(),
        }
    }

    /// Create a progress message
    pub fn progress(
        operation_id: String,
        progress_percent: f32,
        status: ProgressStatus,
        current_step: String,
    ) -> Self {
        let progress = ProgressMessage {
            operation_id: operation_id.clone(),
            progress_percent,
            current_step,
            total_steps: None,
            estimated_remaining_seconds: None,
            status,
            details: None,
        };

        Self::new(
            format!("progress.{}", operation_id),
            "progress".to_string(),
            MessageType::Progress(progress),
            None,
            SenderType::System,
        )
    }

    /// Create a notification message
    pub fn notification(
        title: String,
        message: String,
        level: NotificationLevel,
        target_users: Vec<String>,
    ) -> Self {
        let notification = NotificationMessage {
            title,
            message,
            level,
            category: "general".to_string(),
            action_url: None,
            action_text: None,
            expires_at: None,
            metadata: HashMap::new(),
        };

        let mut routing_msg = Self::new(
            "notification.general".to_string(),
            "notifications".to_string(),
            MessageType::Notification(notification),
            None,
            SenderType::System,
        );
        routing_msg.target_users = target_users;
        routing_msg
    }

    /// Create an agent message
    pub fn agent(
        agent_id: String,
        agent_type: String,
        content: serde_json::Value,
        content_type: AgentContentType,
    ) -> Self {
        let agent_msg = AgentMessage {
            agent_id: agent_id.clone(),
            agent_type: agent_type.clone(),
            content_type,
            content,
            conversation_id: None,
            parent_message_id: None,
            capabilities: Vec::new(),
            metadata: HashMap::new(),
        };

        Self::new(
            format!("agent.{}.{}", agent_type, agent_id),
            "agents".to_string(),
            MessageType::Agent(agent_msg),
            Some(agent_id),
            SenderType::Agent,
        )
    }

    /// Create a control message
    pub fn control(
        command: ControlCommand,
        target: ControlTarget,
        sender_id: Option<String>,
    ) -> Self {
        let control_msg = ControlMessage {
            command: command.clone(),
            target: target.clone(),
            parameters: HashMap::new(),
            requires_confirmation: false,
            timeout_seconds: Some(30),
        };

        let routing_key = match &target {
            ControlTarget::System => "control.system".to_string(),
            ControlTarget::Service(name) => format!("control.service.{}", name),
            ControlTarget::Agent(id) => format!("control.agent.{}", id),
            ControlTarget::Connection(id) => format!("control.connection.{}", id),
            ControlTarget::Topic(topic) => format!("control.topic.{}", topic),
            ControlTarget::All => "control.all".to_string(),
        };

        Self::new(
            routing_key,
            "control".to_string(),
            MessageType::Control(control_msg),
            sender_id,
            SenderType::System,
        )
    }

    /// Add target user
    pub fn with_target_user(mut self, user_id: String) -> Self {
        self.target_users.push(user_id);
        self
    }

    /// Add target connection
    pub fn with_target_connection(mut self, connection_id: Uuid) -> Self {
        self.target_connections.push(connection_id);
        self
    }

    /// Set delivery options
    pub fn with_delivery_options(mut self, options: DeliveryOptions) -> Self {
        self.delivery_options = options;
        self
    }

    /// Set correlation ID for request tracking
    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    /// Set trace ID for distributed tracing
    pub fn with_trace_id(mut self, trace_id: String) -> Self {
        self.trace_id = Some(trace_id);
        self
    }

    /// Add header
    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }

    /// Check if message has expired
    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.delivery_options.ttl_seconds {
            if ttl == 0 {
                return true; // TTL of 0 means immediately expired
            }
            let elapsed = Utc::now().signed_duration_since(self.created_at);
            elapsed.num_seconds() as u64 >= ttl
        } else {
            false
        }
    }

    /// Get message size estimate in bytes
    pub fn size_estimate(&self) -> usize {
        // Rough estimate of serialized size
        serde_json::to_string(self).map(|s| s.len()).unwrap_or(1024)
    }
}

/// Message validation result
#[derive(Debug, Clone)]
pub enum ValidationResult {
    Valid,
    Invalid(Vec<ValidationError>),
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl RoutingMessage {
    /// Validate message content
    pub fn validate(&self) -> ValidationResult {
        let mut errors = Vec::new();

        // Check routing key format
        if self.routing_key.is_empty() {
            errors.push(ValidationError {
                field: "routing_key".to_string(),
                message: "Routing key cannot be empty".to_string(),
            });
        }

        // Check topic format
        if self.topic.is_empty() {
            errors.push(ValidationError {
                field: "topic".to_string(),
                message: "Topic cannot be empty".to_string(),
            });
        }

        // Validate specific message types
        match &self.message_type {
            MessageType::Progress(progress) => {
                if progress.progress_percent < 0.0 || progress.progress_percent > 100.0 {
                    errors.push(ValidationError {
                        field: "progress_percent".to_string(),
                        message: "Progress percent must be between 0.0 and 100.0".to_string(),
                    });
                }
            }
            MessageType::Notification(notification) => {
                if notification.title.is_empty() {
                    errors.push(ValidationError {
                        field: "title".to_string(),
                        message: "Notification title cannot be empty".to_string(),
                    });
                }
                if notification.message.is_empty() {
                    errors.push(ValidationError {
                        field: "message".to_string(),
                        message: "Notification message cannot be empty".to_string(),
                    });
                }
            }
            MessageType::Agent(agent) => {
                if agent.agent_id.is_empty() {
                    errors.push(ValidationError {
                        field: "agent_id".to_string(),
                        message: "Agent ID cannot be empty".to_string(),
                    });
                }
                if agent.agent_type.is_empty() {
                    errors.push(ValidationError {
                        field: "agent_type".to_string(),
                        message: "Agent type cannot be empty".to_string(),
                    });
                }
            }
            _ => {} // Other types are self-validating
        }

        if errors.is_empty() {
            ValidationResult::Valid
        } else {
            ValidationResult::Invalid(errors)
        }
    }

    /// Sanitize message content for security
    pub fn sanitize(&mut self) {
        // Remove potentially dangerous headers
        self.headers.remove("authorization");
        self.headers.remove("cookie");
        self.headers.remove("x-api-key");

        // Sanitize based on message type
        match &mut self.message_type {
            MessageType::Notification(notification) => {
                // Basic HTML sanitization (in a real implementation, use a proper library)
                notification.title = sanitize_html_basic(&notification.title);
                notification.message = sanitize_html_basic(&notification.message);
            }
            MessageType::Agent(agent) => {
                // Ensure agent content doesn't contain executable code for certain types
                if matches!(agent.content_type, AgentContentType::Text | AgentContentType::Markdown) {
                    if let Some(content_str) = agent.content.as_str() {
                        let sanitized = sanitize_html_basic(content_str);
                        agent.content = serde_json::Value::String(sanitized);
                    }
                }
            }
            _ => {} // Other types don't need sanitization
        }
    }
}

/// Basic HTML sanitization (placeholder - use a proper library in production)
fn sanitize_html_basic(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_routing_message_creation() {
        let msg = RoutingMessage::progress(
            "test_op".to_string(),
            50.0,
            ProgressStatus::InProgress,
            "Processing data".to_string(),
        );

        assert_eq!(msg.routing_key, "progress.test_op");
        assert_eq!(msg.topic, "progress");
        
        if let MessageType::Progress(progress) = &msg.message_type {
            assert_eq!(progress.progress_percent, 50.0);
            assert_eq!(progress.current_step, "Processing data");
        } else {
            panic!("Wrong message type");
        }
    }

    #[test]
    fn test_message_validation() {
        let mut msg = RoutingMessage::new(
            "test.key".to_string(),
            "test".to_string(),
            MessageType::Progress(ProgressMessage {
                operation_id: "test".to_string(),
                progress_percent: 150.0, // Invalid
                current_step: "test".to_string(),
                total_steps: None,
                estimated_remaining_seconds: None,
                status: ProgressStatus::InProgress,
                details: None,
            }),
            None,
            SenderType::System,
        );

        match msg.validate() {
            ValidationResult::Invalid(errors) => {
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0].field, "progress_percent");
            }
            ValidationResult::Valid => panic!("Should be invalid"),
        }
    }

    #[test]
    fn test_message_sanitization() {
        let mut msg = RoutingMessage::notification(
            "<script>alert('xss')</script>".to_string(),
            "Hello <b>world</b>".to_string(),
            NotificationLevel::Info,
            vec!["user1".to_string()],
        );

        msg.sanitize();

        if let MessageType::Notification(notification) = &msg.message_type {
            assert!(notification.title.contains("&lt;script&gt;"));
            assert!(notification.title.contains("&gt;"));
            assert!(notification.message.contains("&lt;b&gt;"));
        } else {
            panic!("Wrong message type");
        }
    }

    #[test]
    fn test_message_expiration() {
        let mut msg = RoutingMessage::progress(
            "test".to_string(),
            0.0,
            ProgressStatus::Starting,
            "Starting".to_string(),
        );

        // Set TTL of 0 which means immediately expired
        msg.delivery_options.ttl_seconds = Some(0);
        
        // Should be expired immediately
        assert!(msg.is_expired());
    }

    #[test]
    fn test_message_builder_pattern() {
        let msg = RoutingMessage::agent(
            "agent1".to_string(),
            "chat".to_string(),
            serde_json::json!({"text": "Hello"}),
            AgentContentType::Json,
        )
        .with_target_user("user1".to_string())
        .with_correlation_id("corr123".to_string())
        .with_header("x-source".to_string(), "web".to_string());

        assert_eq!(msg.target_users.len(), 1);
        assert_eq!(msg.correlation_id, Some("corr123".to_string()));
        assert_eq!(msg.headers.get("x-source"), Some(&"web".to_string()));
    }
}