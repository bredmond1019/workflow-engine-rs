// File: src/db/events/types.rs
//
// Event type definitions for the AI Workflow Orchestration System
// Provides typed events for different system domains

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::{EventError, EventResult, EventSerializable};

/// Metadata attached to all events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub correlation_id: Option<Uuid>,
    pub causation_id: Option<Uuid>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub source: Option<String>,
    pub tags: HashMap<String, String>,
    pub custom: HashMap<String, serde_json::Value>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for EventMetadata {
    fn default() -> Self {
        Self {
            correlation_id: None,
            causation_id: None,
            user_id: None,
            session_id: None,
            source: None,
            tags: HashMap::new(),
            custom: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }
}

impl EventMetadata {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_correlation_id(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }
    
    pub fn with_causation_id(mut self, causation_id: Uuid) -> Self {
        self.causation_id = Some(causation_id);
        self
    }
    
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }
    
    pub fn with_source(mut self, source: String) -> Self {
        self.source = Some(source);
        self
    }
    
    pub fn add_tag(mut self, key: String, value: String) -> Self {
        self.tags.insert(key, value);
        self
    }
    
    pub fn add_custom(mut self, key: String, value: serde_json::Value) -> Self {
        self.custom.insert(key, value);
        self
    }
}

/// Base trait for all events in the system
pub trait Event: EventSerializable + Clone + Send + Sync {
    fn aggregate_id(&self) -> Uuid;
    fn event_id(&self) -> Uuid;
    fn occurred_at(&self) -> DateTime<Utc>;
    fn metadata(&self) -> &EventMetadata;
}

/// Aggregate event wrapper for domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateEvent<T> {
    pub event_id: Uuid,
    pub aggregate_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub metadata: EventMetadata,
    pub payload: T,
}

impl<T> AggregateEvent<T> {
    pub fn new(aggregate_id: Uuid, payload: T) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            aggregate_id,
            occurred_at: Utc::now(),
            metadata: EventMetadata::default(),
            payload,
        }
    }
    
    pub fn with_metadata(mut self, metadata: EventMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

impl<T> Event for AggregateEvent<T>
where
    T: EventSerializable + Clone + Send + Sync + Serialize + for<'de> Deserialize<'de>,
{
    fn aggregate_id(&self) -> Uuid {
        self.aggregate_id
    }
    
    fn event_id(&self) -> Uuid {
        self.event_id
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
    
    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
}

impl<T> EventSerializable for AggregateEvent<T>
where
    T: EventSerializable + Clone + Send + Sync + Serialize + for<'de> Deserialize<'de>,
{
    fn serialize(&self) -> EventResult<serde_json::Value> {
        serde_json::to_value(self).map_err(|e| EventError::SerializationError {
            message: format!("Failed to serialize aggregate event: {}", e),
        })
    }
    
    fn deserialize(data: &serde_json::Value, _version: i32) -> EventResult<Self> {
        serde_json::from_value(data.clone()).map_err(|e| EventError::SerializationError {
            message: format!("Failed to deserialize aggregate event: {}", e),
        })
    }
    
    fn schema_version() -> i32 {
        1
    }
    
    fn event_type() -> &'static str {
        "aggregate_event"
    }
}

// ================================
// Workflow Events
// ================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowEvent {
    WorkflowStarted(WorkflowStartedEvent),
    WorkflowCompleted(WorkflowCompletedEvent),
    WorkflowFailed(WorkflowFailedEvent),
    WorkflowCancelled(WorkflowCancelledEvent),
    NodeExecutionStarted(NodeExecutionStartedEvent),
    NodeExecutionCompleted(NodeExecutionCompletedEvent),
    NodeExecutionFailed(NodeExecutionFailedEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStartedEvent {
    pub workflow_id: Uuid,
    pub workflow_type: String,
    pub configuration: serde_json::Value,
    pub input_data: serde_json::Value,
    pub user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowCompletedEvent {
    pub workflow_id: Uuid,
    pub output_data: serde_json::Value,
    pub duration_ms: i64,
    pub nodes_executed: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowFailedEvent {
    pub workflow_id: Uuid,
    pub error_message: String,
    pub error_details: serde_json::Value,
    pub failed_node: Option<String>,
    pub duration_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowCancelledEvent {
    pub workflow_id: Uuid,
    pub reason: String,
    pub cancelled_by: Option<String>,
    pub duration_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeExecutionStartedEvent {
    pub workflow_id: Uuid,
    pub node_id: String,
    pub node_type: String,
    pub input_data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeExecutionCompletedEvent {
    pub workflow_id: Uuid,
    pub node_id: String,
    pub output_data: serde_json::Value,
    pub duration_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeExecutionFailedEvent {
    pub workflow_id: Uuid,
    pub node_id: String,
    pub error_message: String,
    pub error_details: serde_json::Value,
    pub duration_ms: i64,
}

impl EventSerializable for WorkflowEvent {
    fn serialize(&self) -> EventResult<serde_json::Value> {
        serde_json::to_value(self).map_err(|e| EventError::SerializationError {
            message: format!("Failed to serialize workflow event: {}", e),
        })
    }
    
    fn deserialize(data: &serde_json::Value, _version: i32) -> EventResult<Self> {
        serde_json::from_value(data.clone()).map_err(|e| EventError::SerializationError {
            message: format!("Failed to deserialize workflow event: {}", e),
        })
    }
    
    fn schema_version() -> i32 {
        1
    }
    
    fn event_type() -> &'static str {
        "workflow_event"
    }
}

// ================================
// AI Interaction Events
// ================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIInteractionEvent {
    PromptSent(PromptSentEvent),
    ResponseReceived(ResponseReceivedEvent),
    TokensUsed(TokensUsedEvent),
    AIModelChanged(AIModelChangedEvent),
    RateLimitHit(RateLimitHitEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptSentEvent {
    pub request_id: Uuid,
    pub model: String,
    pub provider: String,
    pub prompt: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub workflow_id: Option<Uuid>,
    pub node_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseReceivedEvent {
    pub request_id: Uuid,
    pub response: String,
    pub completion_tokens: i32,
    pub prompt_tokens: i32,
    pub total_tokens: i32,
    pub cost_usd: Option<f64>,
    pub duration_ms: i64,
    pub model: String,
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokensUsedEvent {
    pub request_id: Uuid,
    pub model: String,
    pub provider: String,
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
    pub cost_usd: Option<f64>,
    pub workflow_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIModelChangedEvent {
    pub old_model: String,
    pub new_model: String,
    pub provider: String,
    pub reason: String,
    pub workflow_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitHitEvent {
    pub provider: String,
    pub model: String,
    pub limit_type: String,
    pub retry_after_seconds: Option<i64>,
    pub request_id: Uuid,
}

impl EventSerializable for AIInteractionEvent {
    fn serialize(&self) -> EventResult<serde_json::Value> {
        serde_json::to_value(self).map_err(|e| EventError::SerializationError {
            message: format!("Failed to serialize AI interaction event: {}", e),
        })
    }
    
    fn deserialize(data: &serde_json::Value, _version: i32) -> EventResult<Self> {
        serde_json::from_value(data.clone()).map_err(|e| EventError::SerializationError {
            message: format!("Failed to deserialize AI interaction event: {}", e),
        })
    }
    
    fn schema_version() -> i32 {
        1
    }
    
    fn event_type() -> &'static str {
        "ai_interaction_event"
    }
}

// ================================
// Service Call Events
// ================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceCallEvent {
    MCPCallStarted(MCPCallStartedEvent),
    MCPCallCompleted(MCPCallCompletedEvent),
    MCPCallFailed(MCPCallFailedEvent),
    ServiceRegistered(ServiceRegisteredEvent),
    ServiceUnregistered(ServiceUnregisteredEvent),
    ServiceHealthCheckFailed(ServiceHealthCheckFailedEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPCallStartedEvent {
    pub call_id: Uuid,
    pub service_name: String,
    pub tool_name: String,
    pub parameters: serde_json::Value,
    pub workflow_id: Option<Uuid>,
    pub node_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPCallCompletedEvent {
    pub call_id: Uuid,
    pub service_name: String,
    pub tool_name: String,
    pub result: serde_json::Value,
    pub duration_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPCallFailedEvent {
    pub call_id: Uuid,
    pub service_name: String,
    pub tool_name: String,
    pub error_message: String,
    pub error_details: serde_json::Value,
    pub duration_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegisteredEvent {
    pub service_name: String,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceUnregisteredEvent {
    pub service_name: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealthCheckFailedEvent {
    pub service_name: String,
    pub endpoint: String,
    pub error_message: String,
    pub consecutive_failures: i32,
}

impl EventSerializable for ServiceCallEvent {
    fn serialize(&self) -> EventResult<serde_json::Value> {
        serde_json::to_value(self).map_err(|e| EventError::SerializationError {
            message: format!("Failed to serialize service call event: {}", e),
        })
    }
    
    fn deserialize(data: &serde_json::Value, _version: i32) -> EventResult<Self> {
        serde_json::from_value(data.clone()).map_err(|e| EventError::SerializationError {
            message: format!("Failed to deserialize service call event: {}", e),
        })
    }
    
    fn schema_version() -> i32 {
        1
    }
    
    fn event_type() -> &'static str {
        "service_call_event"
    }
}

// ================================
// System Events
// ================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemEvent {
    SystemStarted(SystemStartedEvent),
    SystemShutdown(SystemShutdownEvent),
    ErrorOccurred(ErrorOccurredEvent),
    PerformanceMetric(PerformanceMetricEvent),
    ConfigurationChanged(ConfigurationChangedEvent),
    DatabaseConnectionFailed(DatabaseConnectionFailedEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStartedEvent {
    pub version: String,
    pub environment: String,
    pub configuration: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemShutdownEvent {
    pub reason: String,
    pub graceful: bool,
    pub uptime_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorOccurredEvent {
    pub error_type: String,
    pub error_message: String,
    pub error_details: serde_json::Value,
    pub component: String,
    pub severity: String,
    pub stack_trace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetricEvent {
    pub metric_name: String,
    pub metric_value: f64,
    pub metric_type: String, // counter, gauge, histogram
    pub labels: HashMap<String, String>,
    pub component: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationChangedEvent {
    pub component: String,
    pub old_config: serde_json::Value,
    pub new_config: serde_json::Value,
    pub changed_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConnectionFailedEvent {
    pub database_name: String,
    pub error_message: String,
    pub retry_count: i32,
    pub next_retry_at: DateTime<Utc>,
}

impl EventSerializable for SystemEvent {
    fn serialize(&self) -> EventResult<serde_json::Value> {
        serde_json::to_value(self).map_err(|e| EventError::SerializationError {
            message: format!("Failed to serialize system event: {}", e),
        })
    }
    
    fn deserialize(data: &serde_json::Value, _version: i32) -> EventResult<Self> {
        serde_json::from_value(data.clone()).map_err(|e| EventError::SerializationError {
            message: format!("Failed to deserialize system event: {}", e),
        })
    }
    
    fn schema_version() -> i32 {
        1
    }
    
    fn event_type() -> &'static str {
        "system_event"
    }
}