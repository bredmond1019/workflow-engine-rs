// File: src/db/event_driven_sync.rs
//
// Event-driven synchronization patterns for microservices communication
// Provides eventual consistency and coordination across service boundaries

use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

use crate::db::events::{EventStore, EventEnvelope, EventResult, EventError};
use crate::db::service_isolation::{CrossServiceEvent, ServiceIsolationError};

/// Saga pattern implementation for distributed transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Saga {
    pub saga_id: Uuid,
    pub saga_type: String,
    pub status: SagaStatus,
    pub current_step: usize,
    pub steps: Vec<SagaStep>,
    pub compensation_steps: Vec<SagaStep>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub timeout_at: Option<DateTime<Utc>>,
    pub correlation_id: Option<Uuid>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SagaStatus {
    Started,
    InProgress,
    Completed,
    Failed,
    Compensating,
    Compensated,
    TimedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaStep {
    pub step_id: Uuid,
    pub service_name: String,
    pub operation: String,
    pub payload: serde_json::Value,
    pub compensation_operation: Option<String>,
    pub compensation_payload: Option<serde_json::Value>,
    pub timeout_seconds: u32,
    pub retry_count: u32,
    pub max_retries: u32,
    pub status: SagaStepStatus,
    pub executed_at: Option<DateTime<Utc>>,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SagaStepStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
    Compensated,
}

impl Saga {
    pub fn new(saga_type: String, timeout_minutes: Option<i64>) -> Self {
        let now = Utc::now();
        Self {
            saga_id: Uuid::new_v4(),
            saga_type,
            status: SagaStatus::Started,
            current_step: 0,
            steps: Vec::new(),
            compensation_steps: Vec::new(),
            created_at: now,
            updated_at: now,
            timeout_at: timeout_minutes.map(|m| now + Duration::minutes(m)),
            correlation_id: None,
            metadata: HashMap::new(),
        }
    }

    pub fn add_step(&mut self, step: SagaStep) {
        self.steps.push(step);
        self.updated_at = Utc::now();
    }

    pub fn current_step(&self) -> Option<&SagaStep> {
        self.steps.get(self.current_step)
    }

    pub fn is_timed_out(&self) -> bool {
        if let Some(timeout) = self.timeout_at {
            Utc::now() > timeout
        } else {
            false
        }
    }

    pub fn can_proceed(&self) -> bool {
        matches!(self.status, SagaStatus::Started | SagaStatus::InProgress) && 
        !self.is_timed_out()
    }

    pub fn mark_step_completed(&mut self, step_result: serde_json::Value) {
        if let Some(step) = self.steps.get_mut(self.current_step) {
            step.status = SagaStepStatus::Completed;
            step.executed_at = Some(Utc::now());
            step.result = Some(step_result);
        }
        self.current_step += 1;
        self.updated_at = Utc::now();

        if self.current_step >= self.steps.len() {
            self.status = SagaStatus::Completed;
        }
    }

    pub fn mark_step_failed(&mut self, error: String) {
        if let Some(step) = self.steps.get_mut(self.current_step) {
            step.status = SagaStepStatus::Failed;
            step.executed_at = Some(Utc::now());
            step.error = Some(error);
        }
        self.status = SagaStatus::Failed;
        self.updated_at = Utc::now();
    }

    pub fn start_compensation(&mut self) {
        self.status = SagaStatus::Compensating;
        self.updated_at = Utc::now();
        
        // Build compensation steps from completed forward steps
        for (i, step) in self.steps.iter().enumerate() {
            if step.status == SagaStepStatus::Completed && 
               step.compensation_operation.is_some() {
                let compensation_step = SagaStep {
                    step_id: Uuid::new_v4(),
                    service_name: step.service_name.clone(),
                    operation: step.compensation_operation.clone().unwrap(),
                    payload: step.compensation_payload.clone().unwrap_or_default(),
                    compensation_operation: None,
                    compensation_payload: None,
                    timeout_seconds: step.timeout_seconds,
                    retry_count: 0,
                    max_retries: step.max_retries,
                    status: SagaStepStatus::Pending,
                    executed_at: None,
                    result: None,
                    error: None,
                };
                self.compensation_steps.push(compensation_step);
            }
        }
        
        // Reverse compensation steps to undo in reverse order
        self.compensation_steps.reverse();
    }
}

impl SagaStep {
    pub fn new(
        service_name: String,
        operation: String,
        payload: serde_json::Value,
        timeout_seconds: u32,
    ) -> Self {
        Self {
            step_id: Uuid::new_v4(),
            service_name,
            operation,
            payload,
            compensation_operation: None,
            compensation_payload: None,
            timeout_seconds,
            retry_count: 0,
            max_retries: 3,
            status: SagaStepStatus::Pending,
            executed_at: None,
            result: None,
            error: None,
        }
    }

    pub fn with_compensation(
        mut self,
        compensation_operation: String,
        compensation_payload: serde_json::Value,
    ) -> Self {
        self.compensation_operation = Some(compensation_operation);
        self.compensation_payload = Some(compensation_payload);
        self
    }

    pub fn with_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }

    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }
}

/// Saga orchestrator manages saga execution across services
#[async_trait]
pub trait SagaOrchestrator: Send + Sync {
    /// Start a new saga
    async fn start_saga(&self, saga: Saga) -> Result<Uuid, SyncError>;
    
    /// Execute the next step in a saga
    async fn execute_next_step(&self, saga_id: Uuid) -> Result<SagaStepResult, SyncError>;
    
    /// Handle saga step completion
    async fn handle_step_completion(
        &self,
        saga_id: Uuid,
        step_result: serde_json::Value,
    ) -> Result<(), SyncError>;
    
    /// Handle saga step failure
    async fn handle_step_failure(
        &self,
        saga_id: Uuid,
        error: String,
    ) -> Result<(), SyncError>;
    
    /// Get saga status
    async fn get_saga_status(&self, saga_id: Uuid) -> Result<SagaStatus, SyncError>;
    
    /// List active sagas
    async fn list_active_sagas(&self) -> Result<Vec<Saga>, SyncError>;
    
    /// Cancel a saga (if possible)
    async fn cancel_saga(&self, saga_id: Uuid) -> Result<(), SyncError>;
    
    /// Process timed out sagas
    async fn process_timeouts(&self) -> Result<Vec<Uuid>, SyncError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaStepResult {
    pub step_id: Uuid,
    pub status: SagaStepStatus,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub next_action: SagaNextAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SagaNextAction {
    ProceedToNext,
    Retry,
    Compensate,
    Complete,
    Fail,
}

/// Event-driven saga orchestrator implementation
pub struct EventDrivenSagaOrchestrator {
    event_store: Arc<dyn EventStore>,
    sagas: Arc<RwLock<HashMap<Uuid, Saga>>>,
    service_clients: Arc<RwLock<HashMap<String, Arc<dyn ServiceClient>>>>,
}

impl EventDrivenSagaOrchestrator {
    pub fn new(event_store: Arc<dyn EventStore>) -> Self {
        Self {
            event_store,
            sagas: Arc::new(RwLock::new(HashMap::new())),
            service_clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_service_client(
        &self,
        service_name: String,
        client: Arc<dyn ServiceClient>,
    ) {
        let mut clients = self.service_clients.write().await;
        clients.insert(service_name, client);
    }

    async fn get_service_client(&self, service_name: &str) -> Option<Arc<dyn ServiceClient>> {
        let clients = self.service_clients.read().await;
        clients.get(service_name).cloned()
    }

    async fn save_saga(&self, saga: &Saga) -> Result<(), SyncError> {
        // Store saga as an event
        let saga_event = EventEnvelope {
            event_id: Uuid::new_v4(),
            aggregate_id: saga.saga_id,
            aggregate_type: "saga".to_string(),
            event_type: "saga_updated".to_string(),
            aggregate_version: 1, // Would need to track actual version
            event_data: serde_json::to_value(saga).map_err(|e| SyncError::SerializationError {
                message: e.to_string(),
            })?,
            metadata: crate::db::events::EventMetadata::default(),
            occurred_at: Utc::now(),
            recorded_at: Utc::now(),
            schema_version: 1,
            causation_id: None,
            correlation_id: saga.correlation_id,
            checksum: None,
        };

        self.event_store.append_event(&saga_event).await.map_err(|e| SyncError::StorageError {
            message: e.to_string(),
        })?;

        // Also store in memory for quick access
        let mut sagas = self.sagas.write().await;
        sagas.insert(saga.saga_id, saga.clone());

        Ok(())
    }
}

#[async_trait]
impl SagaOrchestrator for EventDrivenSagaOrchestrator {
    async fn start_saga(&self, saga: Saga) -> Result<Uuid, SyncError> {
        let saga_id = saga.saga_id;
        self.save_saga(&saga).await?;

        // Publish saga started event
        let saga_started_event = CrossServiceEvent::new(
            "saga_orchestrator".to_string(),
            "saga_started".to_string(),
            "saga.lifecycle".to_string(),
            serde_json::json!({
                "saga_id": saga_id,
                "saga_type": saga.saga_type,
                "step_count": saga.steps.len()
            }),
        )
        .with_correlation_id(saga.correlation_id.unwrap_or(saga_id));

        // Would publish to event bus here

        Ok(saga_id)
    }

    async fn execute_next_step(&self, saga_id: Uuid) -> Result<SagaStepResult, SyncError> {
        let mut sagas = self.sagas.write().await;
        let saga = sagas.get_mut(&saga_id)
            .ok_or_else(|| SyncError::SagaNotFound { saga_id })?;

        if !saga.can_proceed() {
            return Err(SyncError::SagaNotExecutable {
                saga_id,
                status: saga.status.clone(),
            });
        }

        let current_step = saga.current_step();
        if let Some(step) = current_step {
            let service_client = self.get_service_client(&step.service_name).await
                .ok_or_else(|| SyncError::ServiceNotAvailable {
                    service: step.service_name.clone(),
                })?;

            // Execute the step
            match service_client.execute_operation(&step.operation, &step.payload).await {
                Ok(result) => {
                    Ok(SagaStepResult {
                        step_id: step.step_id,
                        status: SagaStepStatus::Completed,
                        result: Some(result),
                        error: None,
                        next_action: if saga.current_step + 1 >= saga.steps.len() {
                            SagaNextAction::Complete
                        } else {
                            SagaNextAction::ProceedToNext
                        },
                    })
                }
                Err(e) => {
                    Ok(SagaStepResult {
                        step_id: step.step_id,
                        status: SagaStepStatus::Failed,
                        result: None,
                        error: Some(e.to_string()),
                        next_action: if step.can_retry() {
                            SagaNextAction::Retry
                        } else {
                            SagaNextAction::Compensate
                        },
                    })
                }
            }
        } else {
            Err(SyncError::InvalidSagaState {
                saga_id,
                message: "No current step to execute".to_string(),
            })
        }
    }

    async fn handle_step_completion(
        &self,
        saga_id: Uuid,
        step_result: serde_json::Value,
    ) -> Result<(), SyncError> {
        let saga_clone = {
            let mut sagas = self.sagas.write().await;
            let saga = sagas.get_mut(&saga_id)
                .ok_or_else(|| SyncError::SagaNotFound { saga_id })?;

            saga.mark_step_completed(step_result);
            saga.clone()
        }; // Lock is automatically dropped here
        
        // Save updated saga
        self.save_saga(&saga_clone).await?;

        Ok(())
    }

    async fn handle_step_failure(
        &self,
        saga_id: Uuid,
        error: String,
    ) -> Result<(), SyncError> {
        let saga_clone = {
            let mut sagas = self.sagas.write().await;
            let saga = sagas.get_mut(&saga_id)
                .ok_or_else(|| SyncError::SagaNotFound { saga_id })?;

            saga.mark_step_failed(error);
            saga.start_compensation();
            saga.clone()
        }; // Lock is automatically dropped here
        
        // Save updated saga
        self.save_saga(&saga_clone).await?;

        Ok(())
    }

    async fn get_saga_status(&self, saga_id: Uuid) -> Result<SagaStatus, SyncError> {
        let sagas = self.sagas.read().await;
        let saga = sagas.get(&saga_id)
            .ok_or_else(|| SyncError::SagaNotFound { saga_id })?;

        Ok(saga.status.clone())
    }

    async fn list_active_sagas(&self) -> Result<Vec<Saga>, SyncError> {
        let sagas = self.sagas.read().await;
        let active_sagas = sagas
            .values()
            .filter(|saga| matches!(saga.status, SagaStatus::Started | SagaStatus::InProgress))
            .cloned()
            .collect();

        Ok(active_sagas)
    }

    async fn cancel_saga(&self, saga_id: Uuid) -> Result<(), SyncError> {
        let saga_clone = {
            let mut sagas = self.sagas.write().await;
            let saga = sagas.get_mut(&saga_id)
                .ok_or_else(|| SyncError::SagaNotFound { saga_id })?;

            if matches!(saga.status, SagaStatus::Started | SagaStatus::InProgress) {
                saga.status = SagaStatus::Failed;
                saga.start_compensation();
                Some(saga.clone())
            } else {
                None
            }
        }; // Lock is automatically dropped here
        
        if let Some(saga) = saga_clone {
            // Save updated saga
            self.save_saga(&saga).await?;
        }

        Ok(())
    }

    async fn process_timeouts(&self) -> Result<Vec<Uuid>, SyncError> {
        let (timed_out_sagas, saga_clones) = {
            let mut sagas = self.sagas.write().await;
            let mut timed_out_sagas = Vec::new();
            let mut saga_clones = Vec::new();

            for (saga_id, saga) in sagas.iter_mut() {
                if saga.is_timed_out() && matches!(saga.status, SagaStatus::Started | SagaStatus::InProgress) {
                    saga.status = SagaStatus::TimedOut;
                    saga.start_compensation();
                    timed_out_sagas.push(*saga_id);
                    saga_clones.push(saga.clone());
                }
            }
            
            (timed_out_sagas, saga_clones)
        }; // Lock is automatically dropped here

        // Save all updated sagas
        for saga in saga_clones {
            self.save_saga(&saga).await?;
        }

        Ok(timed_out_sagas)
    }
}

/// Service client interface for saga operations
#[async_trait]
pub trait ServiceClient: Send + Sync {
    async fn execute_operation(
        &self,
        operation: &str,
        payload: &serde_json::Value,
    ) -> Result<serde_json::Value, ServiceClientError>;

    async fn health_check(&self) -> Result<bool, ServiceClientError>;
}

/// Errors related to synchronization patterns
#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("Saga {saga_id} not found")]
    SagaNotFound { saga_id: Uuid },
    
    #[error("Saga {saga_id} is not executable, status: {status:?}")]
    SagaNotExecutable { saga_id: Uuid, status: SagaStatus },
    
    #[error("Invalid saga state for {saga_id}: {message}")]
    InvalidSagaState { saga_id: Uuid, message: String },
    
    #[error("Service {service} not available")]
    ServiceNotAvailable { service: String },
    
    #[error("Storage error: {message}")]
    StorageError { message: String },
    
    #[error("Serialization error: {message}")]
    SerializationError { message: String },
    
    #[error("Timeout error: {message}")]
    TimeoutError { message: String },
    
    #[error("Communication error: {message}")]
    CommunicationError { message: String },
}

#[derive(Debug, thiserror::Error)]
pub enum ServiceClientError {
    #[error("Network error: {message}")]
    NetworkError { message: String },
    
    #[error("Service error: {message}")]
    ServiceError { message: String },
    
    #[error("Timeout error")]
    TimeoutError,
    
    #[error("Serialization error: {message}")]
    SerializationError { message: String },
}

/// Event-driven consistency patterns
pub struct EventualConsistencyManager {
    event_store: Arc<dyn EventStore>,
    consistency_policies: Arc<RwLock<HashMap<String, ConsistencyPolicy>>>,
    pending_reconciliations: Arc<Mutex<VecDeque<ReconciliationTask>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyPolicy {
    pub entity_type: String,
    pub max_staleness_seconds: u32,
    pub reconciliation_strategy: ReconciliationStrategy,
    pub conflict_resolution: ConflictResolution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReconciliationStrategy {
    Immediate,
    Scheduled { interval_seconds: u32 },
    OnAccess,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    LastWriteWins,
    FirstWriteWins,
    MergeValues,
    CustomFunction { function_name: String },
}

#[derive(Debug, Clone)]
pub struct ReconciliationTask {
    pub task_id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub priority: ReconciliationPriority,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ReconciliationPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl EventualConsistencyManager {
    pub fn new(event_store: Arc<dyn EventStore>) -> Self {
        Self {
            event_store,
            consistency_policies: Arc::new(RwLock::new(HashMap::new())),
            pending_reconciliations: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub async fn register_consistency_policy(&self, policy: ConsistencyPolicy) {
        let mut policies = self.consistency_policies.write().await;
        policies.insert(policy.entity_type.clone(), policy);
    }

    pub async fn schedule_reconciliation(
        &self,
        entity_type: String,
        entity_id: Uuid,
        priority: ReconciliationPriority,
    ) {
        let task = ReconciliationTask {
            task_id: Uuid::new_v4(),
            entity_type,
            entity_id,
            created_at: Utc::now(),
            priority,
        };

        let mut queue = self.pending_reconciliations.lock().await;
        queue.push_back(task);
    }

    pub async fn process_reconciliation_queue(&self) -> Result<usize, SyncError> {
        let mut queue = self.pending_reconciliations.lock().await;
        let processed_count = queue.len();

        while let Some(task) = queue.pop_front() {
            // Process reconciliation task
            self.process_reconciliation_task(&task).await?;
        }

        Ok(processed_count)
    }

    async fn process_reconciliation_task(&self, task: &ReconciliationTask) -> Result<(), SyncError> {
        // Implementation would reconcile state across services
        // For now, just log the task
        tracing::info!("Processing reconciliation task: {:?}", task);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_saga_creation() {
        let mut saga = Saga::new("order_processing".to_string(), Some(60));
        
        let step1 = SagaStep::new(
            "inventory".to_string(),
            "reserve_items".to_string(),
            serde_json::json!({"items": ["item1", "item2"]}),
            30,
        ).with_compensation(
            "release_items".to_string(),
            serde_json::json!({"items": ["item1", "item2"]}),
        );

        let step2 = SagaStep::new(
            "payment".to_string(),
            "charge_card".to_string(),
            serde_json::json!({"amount": 100.0, "card": "xxxx-1234"}),
            30,
        ).with_compensation(
            "refund_charge".to_string(),
            serde_json::json!({"amount": 100.0}),
        );

        saga.add_step(step1);
        saga.add_step(step2);

        assert_eq!(saga.saga_type, "order_processing");
        assert_eq!(saga.status, SagaStatus::Started);
        assert_eq!(saga.steps.len(), 2);
        assert_eq!(saga.current_step, 0);
        assert!(saga.can_proceed());
    }

    #[test]
    fn test_saga_step_progression() {
        let mut saga = Saga::new("test_saga".to_string(), None);
        
        let step = SagaStep::new(
            "test_service".to_string(),
            "test_operation".to_string(),
            serde_json::json!({}),
            30,
        );

        saga.add_step(step);

        // Test step completion
        saga.mark_step_completed(serde_json::json!({"result": "success"}));
        assert_eq!(saga.current_step, 1);
        assert_eq!(saga.status, SagaStatus::Completed);
    }

    #[test]
    fn test_saga_failure_and_compensation() {
        let mut saga = Saga::new("test_saga".to_string(), None);
        
        // Add first step that will complete successfully
        let step1 = SagaStep::new(
            "test_service".to_string(),
            "create_resource".to_string(),
            serde_json::json!({}),
            30,
        ).with_compensation(
            "delete_resource".to_string(),
            serde_json::json!({}),
        );

        // Add second step that will fail
        let step2 = SagaStep::new(
            "test_service".to_string(),
            "update_resource".to_string(),
            serde_json::json!({}),
            30,
        );

        saga.add_step(step1);
        saga.add_step(step2);

        // Complete first step
        saga.mark_step_completed(serde_json::json!({"result": "created"}));
        assert_eq!(saga.current_step, 1);

        // Fail second step
        saga.mark_step_failed("Simulated failure".to_string());
        assert_eq!(saga.status, SagaStatus::Failed);

        // Test compensation start
        saga.start_compensation();
        assert_eq!(saga.status, SagaStatus::Compensating);
        assert!(!saga.compensation_steps.is_empty());
    }

    #[test]
    fn test_consistency_policy() {
        let policy = ConsistencyPolicy {
            entity_type: "user_profile".to_string(),
            max_staleness_seconds: 300,
            reconciliation_strategy: ReconciliationStrategy::Scheduled { interval_seconds: 60 },
            conflict_resolution: ConflictResolution::LastWriteWins,
        };

        assert_eq!(policy.entity_type, "user_profile");
        assert_eq!(policy.max_staleness_seconds, 300);
        assert!(matches!(
            policy.reconciliation_strategy,
            ReconciliationStrategy::Scheduled { interval_seconds: 60 }
        ));
    }

    #[test]
    fn test_cross_service_event_flow() {
        let event = CrossServiceEvent::new(
            "inventory".to_string(),
            "items_reserved".to_string(),
            "inventory.lifecycle".to_string(),
            serde_json::json!({"items": ["item1", "item2"], "reservation_id": "res123"}),
        )
        .with_target("order_service".to_string())
        .with_correlation_id(Uuid::new_v4())
        .add_metadata("priority".to_string(), "high".to_string());

        assert_eq!(event.source_service, "inventory");
        assert_eq!(event.event_type, "items_reserved");
        assert_eq!(event.target_service, Some("order_service".to_string()));
        assert!(event.correlation_id.is_some());
    }
}