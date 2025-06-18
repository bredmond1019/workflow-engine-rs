use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Unified task structure for cross-service communication
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UnifiedTask {
    /// Unique task identifier
    pub id: Uuid,
    
    /// Type name of the task (e.g., "research", "analysis", "notification")
    pub type_name: String,
    
    /// Task input data as JSON value
    pub input: Value,
    
    /// Current status of the task
    pub status: TaskStatus,
    
    /// User or service that created the task (JWT subject)
    pub created_by: String,
    
    /// When the task was created
    pub created_at: DateTime<Utc>,
    
    /// When the task was last updated
    pub updated_at: DateTime<Utc>,
    
    /// Optional output data (populated when task completes)
    pub output: Option<Value>,
    
    /// Optional error information if task failed
    pub error: Option<String>,
    
    /// Optional metadata for additional context
    pub metadata: Option<Value>,
}

impl UnifiedTask {
    /// Create a new UnifiedTask
    pub fn new(type_name: String, input: Value, created_by: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            type_name,
            input,
            status: TaskStatus::Pending,
            created_by,
            created_at: now,
            updated_at: now,
            output: None,
            error: None,
            metadata: None,
        }
    }
    
    /// Mark task as running
    pub fn mark_running(&mut self) {
        self.status = TaskStatus::Running;
        self.updated_at = Utc::now();
    }
    
    /// Mark task as completed with output
    pub fn mark_completed(&mut self, output: Value) {
        self.status = TaskStatus::Completed;
        self.output = Some(output);
        self.updated_at = Utc::now();
    }
    
    /// Mark task as failed with error
    pub fn mark_failed(&mut self, error: String) {
        self.status = TaskStatus::Failed;
        self.error = Some(error);
        self.updated_at = Utc::now();
    }
    
    /// Set metadata
    pub fn set_metadata(&mut self, metadata: Value) {
        self.metadata = Some(metadata);
        self.updated_at = Utc::now();
    }
    
    /// Check if task is finished (completed or failed)
    pub fn is_finished(&self) -> bool {
        matches!(self.status, TaskStatus::Completed | TaskStatus::Failed)
    }
}

/// Task execution status
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum TaskStatus {
    /// Task is waiting to be processed
    Pending,
    
    /// Task is currently being processed
    Running,
    
    /// Task completed successfully
    Completed,
    
    /// Task failed with error
    Failed,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "pending"),
            TaskStatus::Running => write!(f, "running"),
            TaskStatus::Completed => write!(f, "completed"),
            TaskStatus::Failed => write!(f, "failed"),
        }
    }
}

/// Message structure for inter-service communication
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServiceMessage {
    /// Service that sent the message
    pub from_service: String,
    
    /// Service that should receive the message
    pub to_service: String,
    
    /// Correlation ID for tracking related messages
    pub correlation_id: Uuid,
    
    /// Message payload as JSON value
    pub payload: Value,
    
    /// When the message was created
    pub timestamp: DateTime<Utc>,
    
    /// Optional message type for routing
    pub message_type: Option<String>,
    
    /// Optional priority for message processing
    pub priority: Option<MessagePriority>,
    
    /// Optional TTL (time to live) in seconds
    pub ttl_seconds: Option<u64>,
}

impl ServiceMessage {
    /// Create a new ServiceMessage
    pub fn new(
        from_service: String,
        to_service: String,
        payload: Value,
    ) -> Self {
        Self {
            from_service,
            to_service,
            correlation_id: Uuid::new_v4(),
            payload,
            timestamp: Utc::now(),
            message_type: None,
            priority: None,
            ttl_seconds: None,
        }
    }
    
    /// Create a new ServiceMessage with correlation ID
    pub fn with_correlation_id(
        from_service: String,
        to_service: String,
        correlation_id: Uuid,
        payload: Value,
    ) -> Self {
        Self {
            from_service,
            to_service,
            correlation_id,
            payload,
            timestamp: Utc::now(),
            message_type: None,
            priority: None,
            ttl_seconds: None,
        }
    }
    
    /// Set message type
    pub fn with_type(mut self, message_type: String) -> Self {
        self.message_type = Some(message_type);
        self
    }
    
    /// Set message priority
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = Some(priority);
        self
    }
    
    /// Set TTL in seconds
    pub fn with_ttl(mut self, ttl_seconds: u64) -> Self {
        self.ttl_seconds = Some(ttl_seconds);
        self
    }
    
    /// Check if message has expired based on TTL
    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl_seconds {
            let elapsed = Utc::now().timestamp() as u64 - self.timestamp.timestamp() as u64;
            elapsed > ttl
        } else {
            false
        }
    }
}

/// Message priority levels
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

impl std::fmt::Display for MessagePriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessagePriority::Low => write!(f, "low"),
            MessagePriority::Normal => write!(f, "normal"),
            MessagePriority::High => write!(f, "high"),
            MessagePriority::Critical => write!(f, "critical"),
        }
    }
}

/// Request wrapper for unified service communication
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServiceRequest {
    /// The unified task to be processed
    pub task: UnifiedTask,
    
    /// Optional service message for routing
    pub message: Option<ServiceMessage>,
    
    /// Request metadata
    pub metadata: Option<Value>,
}

impl ServiceRequest {
    /// Create a new ServiceRequest
    pub fn new(task: UnifiedTask) -> Self {
        Self {
            task,
            message: None,
            metadata: None,
        }
    }
    
    /// Create a ServiceRequest with message
    pub fn with_message(mut self, message: ServiceMessage) -> Self {
        self.message = Some(message);
        self
    }
    
    /// Create a ServiceRequest with metadata
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Response wrapper for unified service communication
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServiceResponse {
    /// The processed task with updated status
    pub task: UnifiedTask,
    
    /// Optional response message
    pub message: Option<ServiceMessage>,
    
    /// Response metadata
    pub metadata: Option<Value>,
    
    /// Whether the request was successful
    pub success: bool,
}

impl ServiceResponse {
    /// Create a successful ServiceResponse
    pub fn success(task: UnifiedTask) -> Self {
        Self {
            task,
            message: None,
            metadata: None,
            success: true,
        }
    }
    
    /// Create a failed ServiceResponse
    pub fn failure(task: UnifiedTask) -> Self {
        Self {
            task,
            message: None,
            metadata: None,
            success: false,
        }
    }
    
    /// Add response message
    pub fn with_message(mut self, message: ServiceMessage) -> Self {
        self.message = Some(message);
        self
    }
    
    /// Add response metadata
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_unified_task_creation() {
        let task = UnifiedTask::new(
            "test_task".to_string(),
            json!({"input": "test"}),
            "user123".to_string(),
        );
        
        assert_eq!(task.type_name, "test_task");
        assert_eq!(task.input, json!({"input": "test"}));
        assert_eq!(task.created_by, "user123");
        assert_eq!(task.status, TaskStatus::Pending);
        assert!(task.output.is_none());
        assert!(task.error.is_none());
    }

    #[test]
    fn test_task_status_transitions() {
        let mut task = UnifiedTask::new(
            "test_task".to_string(),
            json!({}),
            "user123".to_string(),
        );
        
        // Mark as running
        task.mark_running();
        assert_eq!(task.status, TaskStatus::Running);
        
        // Mark as completed
        let output = json!({"result": "success"});
        task.mark_completed(output.clone());
        assert_eq!(task.status, TaskStatus::Completed);
        assert_eq!(task.output, Some(output));
        assert!(task.is_finished());
    }

    #[test]
    fn test_task_failure() {
        let mut task = UnifiedTask::new(
            "test_task".to_string(),
            json!({}),
            "user123".to_string(),
        );
        
        task.mark_failed("Test error".to_string());
        assert_eq!(task.status, TaskStatus::Failed);
        assert_eq!(task.error, Some("Test error".to_string()));
        assert!(task.is_finished());
    }

    #[test]
    fn test_service_message_creation() {
        let message = ServiceMessage::new(
            "service_a".to_string(),
            "service_b".to_string(),
            json!({"data": "test"}),
        );
        
        assert_eq!(message.from_service, "service_a");
        assert_eq!(message.to_service, "service_b");
        assert_eq!(message.payload, json!({"data": "test"}));
        assert!(!message.is_expired()); // Should not be expired immediately
    }

    #[test]
    fn test_message_with_correlation_id() {
        let correlation_id = Uuid::new_v4();
        let message = ServiceMessage::with_correlation_id(
            "service_a".to_string(),
            "service_b".to_string(),
            correlation_id,
            json!({}),
        );
        
        assert_eq!(message.correlation_id, correlation_id);
    }

    #[test]
    fn test_message_priority() {
        let message = ServiceMessage::new(
            "service_a".to_string(),
            "service_b".to_string(),
            json!({}),
        ).with_priority(MessagePriority::High);
        
        assert_eq!(message.priority, Some(MessagePriority::High));
    }

    #[test]
    fn test_message_ttl() {
        let mut message = ServiceMessage::new(
            "service_a".to_string(),
            "service_b".to_string(),
            json!({}),
        ).with_ttl(0); // Immediate expiration
        
        // Manually set timestamp to past to test expiration
        message.timestamp = Utc::now() - chrono::Duration::seconds(1);
        assert!(message.is_expired());
    }

    #[test]
    fn test_service_request_response() {
        let task = UnifiedTask::new(
            "test_task".to_string(),
            json!({}),
            "user123".to_string(),
        );
        
        let request = ServiceRequest::new(task.clone());
        assert_eq!(request.task.id, task.id);
        
        let response = ServiceResponse::success(task.clone());
        assert!(response.success);
        assert_eq!(response.task.id, task.id);
    }

    #[test]
    fn test_task_status_display() {
        assert_eq!(TaskStatus::Pending.to_string(), "pending");
        assert_eq!(TaskStatus::Running.to_string(), "running");
        assert_eq!(TaskStatus::Completed.to_string(), "completed");
        assert_eq!(TaskStatus::Failed.to_string(), "failed");
    }

    #[test]
    fn test_message_priority_ordering() {
        assert!(MessagePriority::Critical > MessagePriority::High);
        assert!(MessagePriority::High > MessagePriority::Normal);
        assert!(MessagePriority::Normal > MessagePriority::Low);
    }
}