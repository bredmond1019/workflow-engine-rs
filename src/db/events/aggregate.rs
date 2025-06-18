// File: src/db/events/aggregate.rs
//
// Aggregate root pattern implementation for event sourcing

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::{
    EventError, EventResult, EventEnvelope, EventMetadata, EventStore, 
    AggregateSnapshot, EventSerializable
};

/// Version number for aggregate concurrency control
pub type AggregateVersion = i64;

/// Trait for aggregate roots that can be reconstructed from events
#[async_trait]
pub trait AggregateRoot: Send + Sync + Sized {
    type Event: EventSerializable + Clone + Send + Sync;
    type Command;
    type Error: From<EventError> + Send + Sync;
    
    /// Get the aggregate ID
    fn aggregate_id(&self) -> Uuid;
    
    /// Get the aggregate type name
    fn aggregate_type() -> &'static str;
    
    /// Get the current version of the aggregate
    fn version(&self) -> AggregateVersion;
    
    /// Apply an event to the aggregate state
    fn apply_event(&mut self, event: &Self::Event) -> Result<(), Self::Error>;
    
    /// Handle a command and return the resulting events
    async fn handle_command(&mut self, command: Self::Command) -> Result<Vec<Self::Event>, Self::Error>;
    
    /// Create a snapshot of the current aggregate state
    fn create_snapshot(&self) -> Result<serde_json::Value, Self::Error>;
    
    /// Restore aggregate state from a snapshot
    fn from_snapshot(
        aggregate_id: Uuid,
        version: AggregateVersion,
        snapshot_data: &serde_json::Value,
    ) -> Result<Self, Self::Error>;
    
    /// Create a new aggregate with the given ID
    fn new(aggregate_id: Uuid) -> Self;
    
    /// Get uncommitted events (for event sourcing repositories)
    fn get_uncommitted_events(&self) -> Vec<Self::Event> {
        // Default implementation returns empty - override if needed
        Vec::new()
    }
    
    /// Mark events as committed (for event sourcing repositories)
    fn mark_events_as_committed(&mut self) {
        // Default implementation does nothing - override if needed
    }
}

/// Repository for loading and saving aggregate roots using event sourcing
pub struct EventSourcingRepository<T: AggregateRoot> {
    event_store: Box<dyn EventStore>,
    snapshot_frequency: i64,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: AggregateRoot> EventSourcingRepository<T> {
    /// Create a new repository
    pub fn new(event_store: Box<dyn EventStore>, snapshot_frequency: i64) -> Self {
        Self {
            event_store,
            snapshot_frequency,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Load an aggregate from the event store
    pub async fn load(&self, aggregate_id: Uuid) -> Result<Option<T>, T::Error> {
        // Try to load from snapshot first
        let (mut aggregate, from_version) = match self.event_store.get_snapshot(aggregate_id).await? {
            Some(snapshot) => {
                let aggregate = T::from_snapshot(
                    aggregate_id,
                    snapshot.aggregate_version,
                    &snapshot.snapshot_data,
                )?;
                (aggregate, snapshot.aggregate_version)
            }
            None => {
                // No snapshot, start from scratch
                if !self.event_store.aggregate_exists(aggregate_id).await? {
                    return Ok(None);
                }
                (T::new(aggregate_id), 0)
            }
        };
        
        // Load and apply events since the snapshot
        let events = self.event_store
            .get_events_from_version(aggregate_id, from_version)
            .await?;
        
        for event_envelope in events {
            // Deserialize the event
            let event = T::Event::deserialize(&event_envelope.event_data, event_envelope.schema_version)?;
            aggregate.apply_event(&event)?;
        }
        
        Ok(Some(aggregate))
    }
    
    /// Save an aggregate to the event store
    pub async fn save(&self, aggregate: &mut T) -> Result<(), T::Error> {
        let uncommitted_events = aggregate.get_uncommitted_events();
        
        if uncommitted_events.is_empty() {
            return Ok(());
        }
        
        // Convert domain events to event envelopes
        let mut event_envelopes = Vec::new();
        let mut current_version = aggregate.version();
        
        for event in &uncommitted_events {
            current_version += 1;
            
            let event_envelope = EventEnvelope {
                event_id: Uuid::new_v4(),
                aggregate_id: aggregate.aggregate_id(),
                aggregate_type: T::aggregate_type().to_string(),
                event_type: T::Event::event_type().to_string(),
                aggregate_version: current_version,
                event_data: event.serialize()?,
                metadata: EventMetadata::default(),
                occurred_at: Utc::now(),
                recorded_at: Utc::now(),
                schema_version: T::Event::schema_version(),
                causation_id: None,
                correlation_id: None,
                checksum: None,
            };
            
            event_envelopes.push(event_envelope);
        }
        
        // Save events to store
        self.event_store.append_events(&event_envelopes).await?;
        
        // Mark events as committed
        aggregate.mark_events_as_committed();
        
        // Create snapshot if needed
        if current_version % self.snapshot_frequency == 0 {
            let snapshot_data = aggregate.create_snapshot()?;
            let snapshot = AggregateSnapshot::new(
                aggregate.aggregate_id(),
                T::aggregate_type().to_string(),
                current_version,
                snapshot_data,
            );
            
            self.event_store.save_snapshot(&snapshot).await?;
        }
        
        Ok(())
    }
    
    /// Create a new aggregate and save it
    pub async fn create(&self, aggregate: &mut T) -> Result<(), T::Error> {
        self.save(aggregate).await
    }
    
    /// Check if an aggregate exists
    pub async fn exists(&self, aggregate_id: Uuid) -> Result<bool, T::Error> {
        Ok(self.event_store.aggregate_exists(aggregate_id).await?)
    }
}

/// Base aggregate root implementation with event tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseAggregate<TEvent> {
    pub id: Uuid,
    pub version: AggregateVersion,
    pub uncommitted_events: Vec<TEvent>,
}

impl<TEvent> BaseAggregate<TEvent> {
    /// Create a new base aggregate
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            version: 0,
            uncommitted_events: Vec::new(),
        }
    }
    
    /// Apply an event and increment version
    pub fn apply_event(&mut self, event: TEvent) 
    where 
        TEvent: Clone 
    {
        self.version += 1;
        self.uncommitted_events.push(event);
    }
    
    /// Get uncommitted events
    pub fn get_uncommitted_events(&self) -> Vec<TEvent> 
    where 
        TEvent: Clone 
    {
        self.uncommitted_events.clone()
    }
    
    /// Mark events as committed
    pub fn mark_events_as_committed(&mut self) {
        self.uncommitted_events.clear();
    }
    
    /// Apply event without adding to uncommitted (for replay)
    pub fn replay_event(&mut self, _event: &TEvent) {
        self.version += 1;
    }
}

/// Example workflow aggregate implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowAggregate {
    base: BaseAggregate<WorkflowEvent>,
    pub workflow_type: String,
    pub status: WorkflowStatus,
    pub input_data: serde_json::Value,
    pub output_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowStatus {
    Created,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowEvent {
    WorkflowCreated {
        workflow_type: String,
        input_data: serde_json::Value,
    },
    WorkflowStarted,
    WorkflowCompleted {
        output_data: serde_json::Value,
    },
    WorkflowFailed {
        error_message: String,
    },
    WorkflowCancelled {
        reason: String,
    },
}

#[derive(Debug, Clone)]
pub enum WorkflowCommand {
    CreateWorkflow {
        workflow_type: String,
        input_data: serde_json::Value,
    },
    StartWorkflow,
    CompleteWorkflow {
        output_data: serde_json::Value,
    },
    FailWorkflow {
        error_message: String,
    },
    CancelWorkflow {
        reason: String,
    },
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

#[async_trait]
impl AggregateRoot for WorkflowAggregate {
    type Event = WorkflowEvent;
    type Command = WorkflowCommand;
    type Error = EventError;
    
    fn aggregate_id(&self) -> Uuid {
        self.base.id
    }
    
    fn aggregate_type() -> &'static str {
        "workflow"
    }
    
    fn version(&self) -> AggregateVersion {
        self.base.version
    }
    
    fn apply_event(&mut self, event: &Self::Event) -> Result<(), Self::Error> {
        match event {
            WorkflowEvent::WorkflowCreated { workflow_type, input_data } => {
                self.workflow_type = workflow_type.clone();
                self.input_data = input_data.clone();
                self.status = WorkflowStatus::Created;
                self.created_at = Utc::now();
            }
            WorkflowEvent::WorkflowStarted => {
                self.status = WorkflowStatus::Running;
            }
            WorkflowEvent::WorkflowCompleted { output_data } => {
                self.status = WorkflowStatus::Completed;
                self.output_data = Some(output_data.clone());
                self.completed_at = Some(Utc::now());
            }
            WorkflowEvent::WorkflowFailed { error_message } => {
                self.status = WorkflowStatus::Failed;
                self.error_message = Some(error_message.clone());
                self.completed_at = Some(Utc::now());
            }
            WorkflowEvent::WorkflowCancelled { reason: _ } => {
                self.status = WorkflowStatus::Cancelled;
                self.completed_at = Some(Utc::now());
            }
        }
        
        self.base.replay_event(event);
        Ok(())
    }
    
    async fn handle_command(&mut self, command: Self::Command) -> Result<Vec<Self::Event>, Self::Error> {
        let events = match command {
            WorkflowCommand::CreateWorkflow { workflow_type, input_data } => {
                if self.base.version > 0 {
                    return Err(EventError::ConfigurationError {
                        message: "Workflow already exists".to_string(),
                    });
                }
                vec![WorkflowEvent::WorkflowCreated { workflow_type, input_data }]
            }
            WorkflowCommand::StartWorkflow => {
                match self.status {
                    WorkflowStatus::Created => vec![WorkflowEvent::WorkflowStarted],
                    _ => return Err(EventError::ConfigurationError {
                        message: "Workflow cannot be started in current state".to_string(),
                    }),
                }
            }
            WorkflowCommand::CompleteWorkflow { output_data } => {
                match self.status {
                    WorkflowStatus::Running => vec![WorkflowEvent::WorkflowCompleted { output_data }],
                    _ => return Err(EventError::ConfigurationError {
                        message: "Workflow cannot be completed in current state".to_string(),
                    }),
                }
            }
            WorkflowCommand::FailWorkflow { error_message } => {
                match self.status {
                    WorkflowStatus::Running => vec![WorkflowEvent::WorkflowFailed { error_message }],
                    _ => return Err(EventError::ConfigurationError {
                        message: "Workflow cannot be failed in current state".to_string(),
                    }),
                }
            }
            WorkflowCommand::CancelWorkflow { reason } => {
                match self.status {
                    WorkflowStatus::Created | WorkflowStatus::Running => {
                        vec![WorkflowEvent::WorkflowCancelled { reason }]
                    }
                    _ => return Err(EventError::ConfigurationError {
                        message: "Workflow cannot be cancelled in current state".to_string(),
                    }),
                }
            }
        };
        
        // Apply events to aggregate
        for event in &events {
            self.apply_event(event)?;
            self.base.apply_event(event.clone());
        }
        
        Ok(events)
    }
    
    fn create_snapshot(&self) -> Result<serde_json::Value, Self::Error> {
        serde_json::to_value(self).map_err(|e| EventError::SerializationError {
            message: format!("Failed to create snapshot: {}", e),
        })
    }
    
    fn from_snapshot(
        aggregate_id: Uuid,
        version: AggregateVersion,
        snapshot_data: &serde_json::Value,
    ) -> Result<Self, Self::Error> {
        let mut aggregate: WorkflowAggregate = serde_json::from_value(snapshot_data.clone())
            .map_err(|e| EventError::SerializationError {
                message: format!("Failed to deserialize snapshot: {}", e),
            })?;
        
        aggregate.base.id = aggregate_id;
        aggregate.base.version = version;
        aggregate.base.uncommitted_events.clear();
        
        Ok(aggregate)
    }
    
    fn new(aggregate_id: Uuid) -> Self {
        Self {
            base: BaseAggregate::new(aggregate_id),
            workflow_type: String::new(),
            status: WorkflowStatus::Created,
            input_data: serde_json::Value::Null,
            output_data: None,
            created_at: Utc::now(),
            completed_at: None,
            error_message: None,
        }
    }
    
    fn get_uncommitted_events(&self) -> Vec<Self::Event> {
        self.base.get_uncommitted_events()
    }
    
    fn mark_events_as_committed(&mut self) {
        self.base.mark_events_as_committed();
    }
}