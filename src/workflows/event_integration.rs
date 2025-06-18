// File: src/workflows/event_integration.rs
//
// Integration layer between workflow system and event sourcing

use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::core::error::WorkflowError;
use crate::db::events::{
    EventDispatcher, EventEnvelope, EventMetadata, EventStore, EventSerializable,
    types::{WorkflowEvent, SystemEvent, AIInteractionEvent, ServiceCallEvent, 
            WorkflowStartedEvent, WorkflowCompletedEvent, WorkflowFailedEvent}
};

/// Workflow event publisher for integrating with event sourcing
pub struct WorkflowEventPublisher {
    dispatcher: Arc<EventDispatcher>,
    source_name: String,
}

impl WorkflowEventPublisher {
    /// Create a new workflow event publisher
    pub fn new(dispatcher: Arc<EventDispatcher>, source_name: String) -> Self {
        Self {
            dispatcher,
            source_name,
        }
    }
    
    /// Publish workflow started event
    pub async fn publish_workflow_started(
        &self,
        workflow_id: Uuid,
        workflow_type: String,
        configuration: serde_json::Value,
        input_data: serde_json::Value,
        user_id: Option<String>,
        correlation_id: Option<Uuid>,
    ) -> Result<(), WorkflowError> {
        let event = WorkflowEvent::WorkflowStarted(WorkflowStartedEvent {
            workflow_id,
            workflow_type: workflow_type.clone(),
            configuration,
            input_data,
            user_id,
        });
        
        self.publish_workflow_event(workflow_id, event, correlation_id).await
    }
    
    /// Publish workflow completed event
    pub async fn publish_workflow_completed(
        &self,
        workflow_id: Uuid,
        output_data: serde_json::Value,
        duration_ms: i64,
        nodes_executed: i32,
        correlation_id: Option<Uuid>,
    ) -> Result<(), WorkflowError> {
        let event = WorkflowEvent::WorkflowCompleted(WorkflowCompletedEvent {
            workflow_id,
            output_data,
            duration_ms,
            nodes_executed,
        });
        
        self.publish_workflow_event(workflow_id, event, correlation_id).await
    }
    
    /// Publish workflow failed event
    pub async fn publish_workflow_failed(
        &self,
        workflow_id: Uuid,
        error_message: String,
        error_details: serde_json::Value,
        failed_node: Option<String>,
        duration_ms: i64,
        correlation_id: Option<Uuid>,
    ) -> Result<(), WorkflowError> {
        let event = WorkflowEvent::WorkflowFailed(WorkflowFailedEvent {
            workflow_id,
            error_message,
            error_details,
            failed_node,
            duration_ms,
        });
        
        self.publish_workflow_event(workflow_id, event, correlation_id).await
    }
    
    /// Publish AI interaction event for token tracking
    pub async fn publish_ai_tokens_used(
        &self,
        request_id: Uuid,
        model: String,
        provider: String,
        prompt_tokens: i32,
        completion_tokens: i32,
        total_tokens: i32,
        cost_usd: Option<f64>,
        workflow_id: Option<Uuid>,
        correlation_id: Option<Uuid>,
    ) -> Result<(), WorkflowError> {
        let event = AIInteractionEvent::TokensUsed(
            crate::db::events::types::TokensUsedEvent {
                request_id,
                model,
                provider,
                prompt_tokens,
                completion_tokens,
                total_tokens,
                cost_usd,
                workflow_id,
            }
        );
        
        self.publish_ai_event(workflow_id.unwrap_or(Uuid::new_v4()), event, correlation_id).await
    }
    
    /// Publish service call started event
    pub async fn publish_service_call_started(
        &self,
        call_id: Uuid,
        service_name: String,
        tool_name: String,
        parameters: serde_json::Value,
        workflow_id: Option<Uuid>,
        node_id: Option<String>,
        correlation_id: Option<Uuid>,
    ) -> Result<(), WorkflowError> {
        let event = ServiceCallEvent::MCPCallStarted(
            crate::db::events::types::MCPCallStartedEvent {
                call_id,
                service_name,
                tool_name,
                parameters,
                workflow_id,
                node_id,
            }
        );
        
        self.publish_service_event(workflow_id.unwrap_or(Uuid::new_v4()), event, correlation_id).await
    }
    
    /// Publish system error event
    pub async fn publish_system_error(
        &self,
        error_type: String,
        error_message: String,
        error_details: serde_json::Value,
        component: String,
        severity: String,
        stack_trace: Option<String>,
        correlation_id: Option<Uuid>,
    ) -> Result<(), WorkflowError> {
        let event = SystemEvent::ErrorOccurred(
            crate::db::events::types::ErrorOccurredEvent {
                error_type,
                error_message,
                error_details,
                component,
                severity,
                stack_trace,
            }
        );
        
        self.publish_system_event(event, correlation_id).await
    }
    
    /// Publish generic workflow event
    async fn publish_workflow_event(
        &self,
        workflow_id: Uuid,
        event: WorkflowEvent,
        correlation_id: Option<Uuid>,
    ) -> Result<(), WorkflowError> {
        let event_envelope = EventEnvelope {
            event_id: Uuid::new_v4(),
            aggregate_id: workflow_id,
            aggregate_type: "workflow".to_string(),
            event_type: "workflow_event".to_string(),
            aggregate_version: 1, // Will be properly set by event store
            event_data: event.serialize().map_err(|e| WorkflowError::SerializationError {
                message: format!("Failed to serialize workflow event: {}", e),
            })?,
            metadata: EventMetadata::new()
                .with_source(self.source_name.clone())
                .with_correlation_id(correlation_id.unwrap_or_else(Uuid::new_v4)),
            occurred_at: Utc::now(),
            recorded_at: Utc::now(),
            schema_version: 1,
            causation_id: None,
            correlation_id,
            checksum: None,
        };
        
        self.dispatcher.dispatch(&event_envelope).await.map_err(|e| {
            WorkflowError::SerializationError {
                message: format!("Failed to dispatch workflow event: {}", e),
            }
        })
    }
    
    /// Publish AI interaction event
    async fn publish_ai_event(
        &self,
        aggregate_id: Uuid,
        event: AIInteractionEvent,
        correlation_id: Option<Uuid>,
    ) -> Result<(), WorkflowError> {
        let event_envelope = EventEnvelope {
            event_id: Uuid::new_v4(),
            aggregate_id,
            aggregate_type: "ai_interaction".to_string(),
            event_type: "ai_interaction_event".to_string(),
            aggregate_version: 1,
            event_data: event.serialize().map_err(|e| WorkflowError::SerializationError {
                message: format!("Failed to serialize AI event: {}", e),
            })?,
            metadata: EventMetadata::new()
                .with_source(self.source_name.clone())
                .with_correlation_id(correlation_id.unwrap_or_else(Uuid::new_v4)),
            occurred_at: Utc::now(),
            recorded_at: Utc::now(),
            schema_version: 1,
            causation_id: None,
            correlation_id,
            checksum: None,
        };
        
        self.dispatcher.dispatch(&event_envelope).await.map_err(|e| {
            WorkflowError::SerializationError {
                message: format!("Failed to dispatch AI event: {}", e),
            }
        })
    }
    
    /// Publish service call event
    async fn publish_service_event(
        &self,
        aggregate_id: Uuid,
        event: ServiceCallEvent,
        correlation_id: Option<Uuid>,
    ) -> Result<(), WorkflowError> {
        let event_envelope = EventEnvelope {
            event_id: Uuid::new_v4(),
            aggregate_id,
            aggregate_type: "service_call".to_string(),
            event_type: "service_call_event".to_string(),
            aggregate_version: 1,
            event_data: event.serialize().map_err(|e| WorkflowError::SerializationError {
                message: format!("Failed to serialize service event: {}", e),
            })?,
            metadata: EventMetadata::new()
                .with_source(self.source_name.clone())
                .with_correlation_id(correlation_id.unwrap_or_else(Uuid::new_v4)),
            occurred_at: Utc::now(),
            recorded_at: Utc::now(),
            schema_version: 1,
            causation_id: None,
            correlation_id,
            checksum: None,
        };
        
        self.dispatcher.dispatch(&event_envelope).await.map_err(|e| {
            WorkflowError::SerializationError {
                message: format!("Failed to dispatch service event: {}", e),
            }
        })
    }
    
    /// Publish system event
    async fn publish_system_event(
        &self,
        event: SystemEvent,
        correlation_id: Option<Uuid>,
    ) -> Result<(), WorkflowError> {
        let event_envelope = EventEnvelope {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(), // System events use generated aggregate ID
            aggregate_type: "system".to_string(),
            event_type: "system_event".to_string(),
            aggregate_version: 1,
            event_data: event.serialize().map_err(|e| WorkflowError::SerializationError {
                message: format!("Failed to serialize system event: {}", e),
            })?,
            metadata: EventMetadata::new()
                .with_source(self.source_name.clone())
                .with_correlation_id(correlation_id.unwrap_or_else(Uuid::new_v4)),
            occurred_at: Utc::now(),
            recorded_at: Utc::now(),
            schema_version: 1,
            causation_id: None,
            correlation_id,
            checksum: None,
        };
        
        self.dispatcher.dispatch(&event_envelope).await.map_err(|e| {
            WorkflowError::SerializationError {
                message: format!("Failed to dispatch system event: {}", e),
            }
        })
    }
}

/// Helper trait to add event publishing to workflow components
#[async_trait]
pub trait WorkflowEventSource {
    /// Get the event publisher for this component
    fn event_publisher(&self) -> Option<&WorkflowEventPublisher>;
    
    /// Publish a workflow started event
    async fn publish_workflow_started(
        &self,
        workflow_id: Uuid,
        workflow_type: String,
        configuration: serde_json::Value,
        input_data: serde_json::Value,
        user_id: Option<String>,
        correlation_id: Option<Uuid>,
    ) -> Result<(), WorkflowError> {
        if let Some(publisher) = self.event_publisher() {
            publisher.publish_workflow_started(
                workflow_id,
                workflow_type,
                configuration,
                input_data,
                user_id,
                correlation_id,
            ).await
        } else {
            Ok(()) // No publisher configured
        }
    }
    
    /// Publish a workflow error event
    async fn publish_workflow_error(
        &self,
        workflow_id: Uuid,
        error_message: String,
        error_details: serde_json::Value,
        failed_node: Option<String>,
        duration_ms: i64,
        correlation_id: Option<Uuid>,
    ) -> Result<(), WorkflowError> {
        if let Some(publisher) = self.event_publisher() {
            publisher.publish_workflow_failed(
                workflow_id,
                error_message,
                error_details,
                failed_node,
                duration_ms,
                correlation_id,
            ).await
        } else {
            Ok(())
        }
    }
}

/// Event-driven workflow executor that publishes events during execution
pub struct EventDrivenWorkflowExecutor {
    publisher: Option<WorkflowEventPublisher>,
}

impl EventDrivenWorkflowExecutor {
    /// Create a new event-driven workflow executor
    pub fn new(publisher: Option<WorkflowEventPublisher>) -> Self {
        Self { publisher }
    }
    
    /// Execute a workflow with event publishing
    pub async fn execute_workflow(
        &self,
        workflow_id: Uuid,
        workflow_type: String,
        input_data: serde_json::Value,
        correlation_id: Option<Uuid>,
    ) -> Result<serde_json::Value, WorkflowError> {
        let start_time = std::time::Instant::now();
        
        // Publish workflow started event
        if let Some(publisher) = &self.publisher {
            publisher.publish_workflow_started(
                workflow_id,
                workflow_type.clone(),
                json!({}), // Empty configuration for now
                input_data.clone(),
                None, // No user ID for now
                correlation_id,
            ).await?;
        }
        
        // Execute workflow logic (placeholder)
        let result = self.execute_workflow_logic(workflow_id, input_data).await;
        
        let duration_ms = start_time.elapsed().as_millis() as i64;
        
        match result {
            Ok(output) => {
                // Publish success event
                if let Some(publisher) = &self.publisher {
                    publisher.publish_workflow_completed(
                        workflow_id,
                        output.clone(),
                        duration_ms,
                        1, // Number of nodes executed (placeholder)
                        correlation_id,
                    ).await?;
                }
                Ok(output)
            }
            Err(error) => {
                // Publish failure event
                if let Some(publisher) = &self.publisher {
                    publisher.publish_workflow_failed(
                        workflow_id,
                        error.to_string(),
                        json!({"error_type": "execution_error"}),
                        None, // Failed node (would be determined in real implementation)
                        duration_ms,
                        correlation_id,
                    ).await?;
                }
                Err(error)
            }
        }
    }
    
    /// Execute the actual workflow logic (placeholder)
    async fn execute_workflow_logic(
        &self,
        _workflow_id: Uuid,
        input_data: serde_json::Value,
    ) -> Result<serde_json::Value, WorkflowError> {
        // Placeholder implementation
        // In a real system, this would execute the workflow nodes
        
        // Simulate some processing time
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // Return processed data
        Ok(json!({
            "result": "processed",
            "input": input_data,
            "timestamp": Utc::now().to_rfc3339()
        }))
    }
}

#[async_trait]
impl WorkflowEventSource for EventDrivenWorkflowExecutor {
    fn event_publisher(&self) -> Option<&WorkflowEventPublisher> {
        self.publisher.as_ref()
    }
}

/// Factory for creating event-aware workflow components
pub struct EventAwareWorkflowFactory {
    event_dispatcher: Arc<EventDispatcher>,
}

impl EventAwareWorkflowFactory {
    /// Create a new factory
    pub fn new(event_dispatcher: Arc<EventDispatcher>) -> Self {
        Self { event_dispatcher }
    }
    
    /// Create an event-driven workflow executor
    pub fn create_workflow_executor(&self, source_name: String) -> EventDrivenWorkflowExecutor {
        let publisher = WorkflowEventPublisher::new(
            Arc::clone(&self.event_dispatcher),
            source_name,
        );
        
        EventDrivenWorkflowExecutor::new(Some(publisher))
    }
    
    /// Create a standalone event publisher
    pub fn create_event_publisher(&self, source_name: String) -> WorkflowEventPublisher {
        WorkflowEventPublisher::new(
            Arc::clone(&self.event_dispatcher),
            source_name,
        )
    }
}