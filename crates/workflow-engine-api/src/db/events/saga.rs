// File: src/db/events/saga.rs
//
// Saga pattern implementation for distributed transaction coordination

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{EventEnvelope, EventError, EventResult, EventStore};

/// Saga transaction state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SagaState {
    /// Saga is being orchestrated and steps are executing
    Running,
    /// All saga steps completed successfully
    Completed,
    /// Saga failed and compensations are running
    Compensating,
    /// Saga failed and all compensations completed
    Failed,
    /// Saga was aborted before completion
    Aborted,
}

/// Saga step execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SagaStepStatus {
    /// Step is waiting to be executed
    Pending,
    /// Step is currently executing
    Running,
    /// Step completed successfully
    Completed,
    /// Step failed and needs compensation
    Failed,
    /// Step compensation is running
    Compensating,
    /// Step compensation completed
    Compensated,
    /// Step was skipped
    Skipped,
}

/// Saga step definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaStep {
    pub step_id: String,
    pub service_name: String,
    pub operation: String,
    pub input_data: serde_json::Value,
    pub compensation_operation: Option<String>,
    pub compensation_data: Option<serde_json::Value>,
    pub timeout_seconds: Option<u64>,
    pub retry_policy: Option<RetryPolicy>,
    pub depends_on: Vec<String>,
    pub parallel_group: Option<String>,
}

/// Retry policy for saga steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub exponential_backoff: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 1000,
            max_delay_ms: 30000,
            exponential_backoff: true,
        }
    }
}

/// Saga execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaExecution {
    pub saga_id: Uuid,
    pub saga_type: String,
    pub state: SagaState,
    pub steps: Vec<SagaStepExecution>,
    pub global_context: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub total_timeout_seconds: Option<u64>,
}

/// Saga step execution state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaStepExecution {
    pub step: SagaStep,
    pub status: SagaStepStatus,
    pub attempt_count: u32,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub output_data: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub compensation_started_at: Option<DateTime<Utc>>,
    pub compensation_completed_at: Option<DateTime<Utc>>,
}

impl SagaStepExecution {
    fn new(step: SagaStep) -> Self {
        Self {
            step,
            status: SagaStepStatus::Pending,
            attempt_count: 0,
            started_at: None,
            completed_at: None,
            output_data: None,
            error_message: None,
            compensation_started_at: None,
            compensation_completed_at: None,
        }
    }
}

/// Saga definition template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaDefinition {
    pub saga_type: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<SagaStep>,
    pub global_timeout_seconds: Option<u64>,
    pub compensation_strategy: CompensationStrategy,
}

/// Compensation strategy for failed sagas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompensationStrategy {
    /// Compensate in reverse order of execution
    ReverseOrder,
    /// Compensate in parallel where possible
    Parallel,
    /// Custom compensation order
    Custom(Vec<String>),
}

/// Saga orchestrator for managing distributed transactions
pub struct SagaOrchestrator {
    event_store: Arc<dyn EventStore>,
    saga_store: Arc<dyn SagaStore>,
    step_executors: Arc<RwLock<HashMap<String, Arc<dyn SagaStepExecutor>>>>,
    running_sagas: Arc<RwLock<HashMap<Uuid, SagaExecution>>>,
}

impl SagaOrchestrator {
    /// Create a new saga orchestrator
    pub fn new(
        event_store: Arc<dyn EventStore>,
        saga_store: Arc<dyn SagaStore>,
    ) -> Self {
        Self {
            event_store,
            saga_store,
            step_executors: Arc::new(RwLock::new(HashMap::new())),
            running_sagas: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a step executor for a service
    pub async fn register_step_executor(
        &self,
        service_name: &str,
        executor: Arc<dyn SagaStepExecutor>,
    ) {
        let mut executors = self.step_executors.write().await;
        executors.insert(service_name.to_string(), executor);
    }
    
    /// Start a new saga execution
    pub async fn start_saga(
        &self,
        definition: SagaDefinition,
        input_context: serde_json::Value,
    ) -> EventResult<Uuid> {
        let saga_id = Uuid::new_v4();
        
        let execution = SagaExecution {
            saga_id,
            saga_type: definition.saga_type.clone(),
            state: SagaState::Running,
            steps: definition.steps.iter().map(|s| SagaStepExecution::new(s.clone())).collect(),
            global_context: input_context,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: None,
            error_message: None,
            total_timeout_seconds: definition.global_timeout_seconds,
        };
        
        // Store the saga
        self.saga_store.save_saga(&execution).await?;
        
        // Add to running sagas
        let mut running = self.running_sagas.write().await;
        running.insert(saga_id, execution.clone());
        drop(running);
        
        // Emit saga started event
        self.emit_saga_event(saga_id, "saga_started", serde_json::json!({
            "saga_type": definition.saga_type,
            "step_count": definition.steps.len()
        })).await?;
        
        // Start executing the saga
        self.execute_next_steps(saga_id).await?;
        
        Ok(saga_id)
    }
    
    /// Continue saga execution after a step completes
    pub async fn handle_step_completion(
        &self,
        saga_id: Uuid,
        step_id: &str,
        result: SagaStepResult,
    ) -> EventResult<()> {
        let mut running = self.running_sagas.write().await;
        let mut execution = running.get_mut(&saga_id)
            .ok_or_else(|| EventError::HandlerError {
                message: format!("Saga {} not found in running sagas", saga_id),
            })?
            .clone();
        drop(running);
        
        // Update step status
        if let Some(step_exec) = execution.steps.iter_mut().find(|s| s.step.step_id == step_id) {
            match result {
                SagaStepResult::Success { output_data } => {
                    step_exec.status = SagaStepStatus::Completed;
                    step_exec.completed_at = Some(Utc::now());
                    step_exec.output_data = Some(output_data);
                    
                    self.emit_saga_event(saga_id, "step_completed", serde_json::json!({
                        "step_id": step_id,
                        "service": step_exec.step.service_name
                    })).await?;
                }
                SagaStepResult::Failure { error_message } => {
                    step_exec.status = SagaStepStatus::Failed;
                    step_exec.completed_at = Some(Utc::now());
                    step_exec.error_message = Some(error_message.clone());
                    
                    self.emit_saga_event(saga_id, "step_failed", serde_json::json!({
                        "step_id": step_id,
                        "service": step_exec.step.service_name,
                        "error": error_message
                    })).await?;
                    
                    // Start compensation
                    execution.state = SagaState::Compensating;
                    execution.error_message = Some(error_message);
                    self.start_compensation(&mut execution).await?;
                }
            }
            
            execution.updated_at = Utc::now();
        }
        
        // Update running saga
        let mut running = self.running_sagas.write().await;
        running.insert(saga_id, execution.clone());
        drop(running);
        
        // Save to store
        self.saga_store.save_saga(&execution).await?;
        
        // Continue execution if still running
        if execution.state == SagaState::Running {
            Box::pin(self.execute_next_steps(saga_id)).await?;
        }
        
        Ok(())
    }
    
    /// Execute the next available steps in the saga
    async fn execute_next_steps(&self, saga_id: Uuid) -> EventResult<()> {
        let execution = {
            let running = self.running_sagas.read().await;
            running.get(&saga_id).cloned()
                .ok_or_else(|| EventError::HandlerError {
                    message: format!("Saga {} not found", saga_id),
                })?
        };
        
        if execution.state != SagaState::Running {
            return Ok(());
        }
        
        // Find steps that can be executed
        let executable_steps = self.find_executable_steps(&execution);
        
        if executable_steps.is_empty() {
            // Check if saga is complete
            let all_completed = execution.steps.iter()
                .all(|s| s.status == SagaStepStatus::Completed || s.status == SagaStepStatus::Skipped);
            
            if all_completed {
                self.complete_saga(saga_id).await?;
            }
            
            return Ok(());
        }
        
        // Execute the steps
        for step_id in executable_steps {
            Box::pin(self.execute_step(saga_id, &step_id)).await?;
        }
        
        Ok(())
    }
    
    /// Find steps that are ready to execute
    fn find_executable_steps(&self, execution: &SagaExecution) -> Vec<String> {
        let mut executable = Vec::new();
        
        for step_exec in &execution.steps {
            if step_exec.status != SagaStepStatus::Pending {
                continue;
            }
            
            // Check dependencies
            let dependencies_met = step_exec.step.depends_on.iter().all(|dep| {
                execution.steps.iter().any(|s| 
                    s.step.step_id == *dep && s.status == SagaStepStatus::Completed
                )
            });
            
            if dependencies_met {
                executable.push(step_exec.step.step_id.clone());
            }
        }
        
        executable
    }
    
    /// Execute a single saga step
    async fn execute_step(&self, saga_id: Uuid, step_id: &str) -> EventResult<()> {
        let (step, global_context) = {
            let mut running = self.running_sagas.write().await;
            let execution = running.get_mut(&saga_id)
                .ok_or_else(|| EventError::HandlerError {
                    message: format!("Saga {} not found", saga_id),
                })?;
            
            let step_exec = execution.steps.iter_mut()
                .find(|s| s.step.step_id == step_id)
                .ok_or_else(|| EventError::HandlerError {
                    message: format!("Step {} not found in saga {}", step_id, saga_id),
                })?;
            
            step_exec.status = SagaStepStatus::Running;
            step_exec.started_at = Some(Utc::now());
            step_exec.attempt_count += 1;
            
            (step_exec.step.clone(), execution.global_context.clone())
        };
        
        // Get the executor for this service
        let executors = self.step_executors.read().await;
        let executor = executors.get(&step.service_name)
            .ok_or_else(|| EventError::HandlerError {
                message: format!("No executor found for service {}", step.service_name),
            })?
            .clone();
        drop(executors);
        
        // Execute the step and handle completion
        let result = executor.execute_step(&step, &global_context).await;
        Box::pin(self.handle_step_completion(saga_id, step_id, result)).await?;
        
        Ok(())
    }
    
    /// Start compensation for a failed saga
    async fn start_compensation(&self, execution: &mut SagaExecution) -> EventResult<()> {
        self.emit_saga_event(execution.saga_id, "compensation_started", serde_json::json!({
            "error": execution.error_message
        })).await?;
        
        // Find steps that need compensation (completed steps in reverse order)
        let mut compensatable_steps: Vec<_> = execution.steps.iter_mut()
            .filter(|s| s.status == SagaStepStatus::Completed && s.step.compensation_operation.is_some())
            .collect();
        
        compensatable_steps.reverse();
        
        for step_exec in compensatable_steps {
            if let Some(_) = &step_exec.step.compensation_operation {
                step_exec.status = SagaStepStatus::Compensating;
                step_exec.compensation_started_at = Some(Utc::now());
                
                // Execute compensation
                self.execute_compensation(execution.saga_id, &step_exec.step.step_id).await?;
            }
        }
        
        Ok(())
    }
    
    /// Execute compensation for a step
    async fn execute_compensation(&self, saga_id: Uuid, step_id: &str) -> EventResult<()> {
        // This would execute the compensation operation
        // For now, we'll mark it as compensated
        let mut running = self.running_sagas.write().await;
        if let Some(execution) = running.get_mut(&saga_id) {
            if let Some(step_exec) = execution.steps.iter_mut().find(|s| s.step.step_id == step_id) {
                step_exec.status = SagaStepStatus::Compensated;
                step_exec.compensation_completed_at = Some(Utc::now());
            }
            
            // Check if all compensations are done
            let all_compensated = execution.steps.iter()
                .filter(|s| s.step.compensation_operation.is_some())
                .all(|s| s.status == SagaStepStatus::Compensated);
            
            if all_compensated {
                execution.state = SagaState::Failed;
                execution.completed_at = Some(Utc::now());
                
                self.emit_saga_event(saga_id, "saga_failed", serde_json::json!({
                    "error": execution.error_message
                })).await?;
            }
        }
        
        Ok(())
    }
    
    /// Complete a successful saga
    async fn complete_saga(&self, saga_id: Uuid) -> EventResult<()> {
        let mut running = self.running_sagas.write().await;
        if let Some(execution) = running.get_mut(&saga_id) {
            execution.state = SagaState::Completed;
            execution.completed_at = Some(Utc::now());
            execution.updated_at = Utc::now();
            
            self.saga_store.save_saga(execution).await?;
            
            self.emit_saga_event(saga_id, "saga_completed", serde_json::json!({
                "duration_ms": execution.completed_at.unwrap()
                    .signed_duration_since(execution.created_at)
                    .num_milliseconds()
            })).await?;
        }
        
        Ok(())
    }
    
    /// Emit a saga-related event
    async fn emit_saga_event(
        &self,
        saga_id: Uuid,
        event_type: &str,
        event_data: serde_json::Value,
    ) -> EventResult<()> {
        let event = EventEnvelope {
            event_id: Uuid::new_v4(),
            aggregate_id: saga_id,
            aggregate_type: "saga".to_string(),
            event_type: event_type.to_string(),
            aggregate_version: 1,
            event_data,
            metadata: super::EventMetadata {
                user_id: None,
                session_id: None,
                correlation_id: Some(saga_id),
                causation_id: None,
                timestamp: Utc::now(),
                source: None,
                tags: Default::default(),
                custom: Default::default(),
            },
            occurred_at: Utc::now(),
            recorded_at: Utc::now(),
            schema_version: 1,
            causation_id: None,
            correlation_id: Some(saga_id),
            checksum: None,
        };
        
        self.event_store.append_event(&event).await
    }
    
    /// Clone for async operations
    fn clone_for_async(&self) -> Self {
        Self {
            event_store: Arc::clone(&self.event_store),
            saga_store: Arc::clone(&self.saga_store),
            step_executors: Arc::clone(&self.step_executors),
            running_sagas: Arc::clone(&self.running_sagas),
        }
    }
    
    /// Get saga execution status
    pub async fn get_saga_status(&self, saga_id: Uuid) -> EventResult<Option<SagaExecution>> {
        let running = self.running_sagas.read().await;
        if let Some(execution) = running.get(&saga_id) {
            Ok(Some(execution.clone()))
        } else {
            self.saga_store.get_saga(saga_id).await
        }
    }
    
    /// List all running sagas
    pub async fn list_running_sagas(&self) -> Vec<Uuid> {
        let running = self.running_sagas.read().await;
        running.keys().cloned().collect()
    }
}

/// Result of a saga step execution
#[derive(Debug, Clone)]
pub enum SagaStepResult {
    Success { output_data: serde_json::Value },
    Failure { error_message: String },
}

/// Trait for executing saga steps
#[async_trait]
pub trait SagaStepExecutor: Send + Sync {
    /// Execute a saga step
    async fn execute_step(
        &self,
        step: &SagaStep,
        global_context: &serde_json::Value,
    ) -> SagaStepResult;
    
    /// Get the service name this executor handles
    fn service_name(&self) -> &str;
}

/// Trait for persisting saga state
#[async_trait]
pub trait SagaStore: Send + Sync {
    /// Save saga execution state
    async fn save_saga(&self, execution: &SagaExecution) -> EventResult<()>;
    
    /// Get saga execution by ID
    async fn get_saga(&self, saga_id: Uuid) -> EventResult<Option<SagaExecution>>;
    
    /// List sagas by state
    async fn list_sagas_by_state(&self, state: SagaState) -> EventResult<Vec<SagaExecution>>;
    
    /// Delete old completed sagas
    async fn cleanup_old_sagas(&self, before: DateTime<Utc>) -> EventResult<usize>;
}

/// In-memory saga store for testing
pub struct InMemorySagaStore {
    sagas: Arc<RwLock<HashMap<Uuid, SagaExecution>>>,
}

impl InMemorySagaStore {
    pub fn new() -> Self {
        Self {
            sagas: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl SagaStore for InMemorySagaStore {
    async fn save_saga(&self, execution: &SagaExecution) -> EventResult<()> {
        let mut sagas = self.sagas.write().await;
        sagas.insert(execution.saga_id, execution.clone());
        Ok(())
    }
    
    async fn get_saga(&self, saga_id: Uuid) -> EventResult<Option<SagaExecution>> {
        let sagas = self.sagas.read().await;
        Ok(sagas.get(&saga_id).cloned())
    }
    
    async fn list_sagas_by_state(&self, state: SagaState) -> EventResult<Vec<SagaExecution>> {
        let sagas = self.sagas.read().await;
        Ok(sagas.values()
            .filter(|s| s.state == state)
            .cloned()
            .collect())
    }
    
    async fn cleanup_old_sagas(&self, before: DateTime<Utc>) -> EventResult<usize> {
        let mut sagas = self.sagas.write().await;
        let initial_count = sagas.len();
        
        sagas.retain(|_, saga| {
            saga.completed_at.map_or(true, |completed| completed > before)
        });
        
        Ok(initial_count - sagas.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_saga_state_transitions() {
        assert_eq!(SagaState::Running, SagaState::Running);
        assert_ne!(SagaState::Running, SagaState::Completed);
    }
    
    #[test]
    fn test_step_status_transitions() {
        assert_eq!(SagaStepStatus::Pending, SagaStepStatus::Pending);
        assert_ne!(SagaStepStatus::Pending, SagaStepStatus::Running);
    }
    
    #[test]
    fn test_retry_policy_defaults() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_attempts, 3);
        assert_eq!(policy.base_delay_ms, 1000);
        assert!(policy.exponential_backoff);
    }
    
    #[tokio::test]
    async fn test_in_memory_saga_store() {
        let store = InMemorySagaStore::new();
        let saga_id = Uuid::new_v4();
        
        let execution = SagaExecution {
            saga_id,
            saga_type: "test_saga".to_string(),
            state: SagaState::Running,
            steps: vec![],
            global_context: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: None,
            error_message: None,
            total_timeout_seconds: None,
        };
        
        // Save saga
        store.save_saga(&execution).await.unwrap();
        
        // Retrieve saga
        let retrieved = store.get_saga(saga_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().saga_id, saga_id);
        
        // List by state
        let running_sagas = store.list_sagas_by_state(SagaState::Running).await.unwrap();
        assert_eq!(running_sagas.len(), 1);
    }
}