// File: src/db/events/handlers.rs
//
// Specific event handlers for different domains in the AI Workflow System

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{
    EventError, EventResult, EventEnvelope, EventHandler,
    types::{WorkflowEvent, AIInteractionEvent, ServiceCallEvent, SystemEvent}
};

/// Handler for workflow-related events
pub struct WorkflowEventHandler {
    name: String,
    metrics: Arc<RwLock<WorkflowMetrics>>,
}

#[derive(Debug, Default, Clone)]
pub struct WorkflowMetrics {
    pub workflows_started: u64,
    pub workflows_completed: u64,
    pub workflows_failed: u64,
    pub workflows_cancelled: u64,
    pub nodes_executed: u64,
    pub average_duration_ms: f64,
}

impl WorkflowEventHandler {
    pub fn new() -> Self {
        Self {
            name: "workflow_event_handler".to_string(),
            metrics: Arc::new(RwLock::new(WorkflowMetrics::default())),
        }
    }
    
    pub async fn get_metrics(&self) -> WorkflowMetrics {
        let metrics = self.metrics.read().await;
        (*metrics).clone()
    }
    
    async fn handle_workflow_event(&self, event: &WorkflowEvent) -> EventResult<()> {
        let mut metrics = self.metrics.write().await;
        
        match event {
            WorkflowEvent::WorkflowStarted(_) => {
                metrics.workflows_started += 1;
                tracing::info!("Workflow started event processed");
            }
            WorkflowEvent::WorkflowCompleted(completed_event) => {
                metrics.workflows_completed += 1;
                metrics.average_duration_ms = 
                    (metrics.average_duration_ms + completed_event.duration_ms as f64) / 2.0;
                tracing::info!("Workflow completed in {}ms", completed_event.duration_ms);
            }
            WorkflowEvent::WorkflowFailed(_) => {
                metrics.workflows_failed += 1;
                tracing::warn!("Workflow failed event processed");
            }
            WorkflowEvent::WorkflowCancelled(_) => {
                metrics.workflows_cancelled += 1;
                tracing::info!("Workflow cancelled event processed");
            }
            WorkflowEvent::NodeExecutionStarted(_) => {
                tracing::debug!("Node execution started");
            }
            WorkflowEvent::NodeExecutionCompleted(_) => {
                metrics.nodes_executed += 1;
                tracing::debug!("Node execution completed");
            }
            WorkflowEvent::NodeExecutionFailed(failed_event) => {
                tracing::error!(
                    "Node execution failed: {} - {}",
                    failed_event.node_id,
                    failed_event.error_message
                );
            }
        }
        
        Ok(())
    }
}

#[async_trait]
impl EventHandler for WorkflowEventHandler {
    async fn handle(&self, event: &EventEnvelope) -> EventResult<()> {
        // Deserialize workflow event from envelope
        let workflow_event: WorkflowEvent = serde_json::from_value(event.event_data.clone())
            .map_err(|e| EventError::SerializationError {
                message: format!("Failed to deserialize workflow event: {}", e),
            })?;
        
        self.handle_workflow_event(&workflow_event).await
    }
    
    fn event_types(&self) -> Vec<String> {
        vec!["workflow_event".to_string()]
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// Handler for AI interaction events (token counting, cost tracking)
pub struct AIEventHandler {
    name: String,
    metrics: Arc<RwLock<AIMetrics>>,
}

#[derive(Debug, Default, Clone)]
pub struct AIMetrics {
    pub total_requests: u64,
    pub total_tokens_used: u64,
    pub total_cost_usd: f64,
    pub requests_by_model: std::collections::HashMap<String, u64>,
    pub tokens_by_model: std::collections::HashMap<String, u64>,
    pub rate_limit_hits: u64,
    pub average_response_time_ms: f64,
}

impl AIEventHandler {
    pub fn new() -> Self {
        Self {
            name: "ai_event_handler".to_string(),
            metrics: Arc::new(RwLock::new(AIMetrics::default())),
        }
    }
    
    pub async fn get_metrics(&self) -> AIMetrics {
        let metrics = self.metrics.read().await;
        (*metrics).clone()
    }
    
    async fn handle_ai_event(&self, event: &AIInteractionEvent) -> EventResult<()> {
        let mut metrics = self.metrics.write().await;
        
        match event {
            AIInteractionEvent::PromptSent(prompt_event) => {
                metrics.total_requests += 1;
                *metrics.requests_by_model.entry(prompt_event.model.clone()).or_insert(0) += 1;
                tracing::debug!(
                    "AI prompt sent to {} model: {} chars",
                    prompt_event.model,
                    prompt_event.prompt.len()
                );
            }
            AIInteractionEvent::ResponseReceived(response_event) => {
                metrics.total_tokens_used += response_event.total_tokens as u64;
                *metrics.tokens_by_model.entry(response_event.model.clone()).or_insert(0) += 
                    response_event.total_tokens as u64;
                
                if let Some(cost) = response_event.cost_usd {
                    metrics.total_cost_usd += cost;
                }
                
                metrics.average_response_time_ms = 
                    (metrics.average_response_time_ms + response_event.duration_ms as f64) / 2.0;
                
                tracing::info!(
                    "AI response received: {} tokens, ${:.4} cost, {}ms duration",
                    response_event.total_tokens,
                    response_event.cost_usd.unwrap_or(0.0),
                    response_event.duration_ms
                );
            }
            AIInteractionEvent::TokensUsed(token_event) => {
                metrics.total_tokens_used += token_event.total_tokens as u64;
                if let Some(cost) = token_event.cost_usd {
                    metrics.total_cost_usd += cost;
                }
                tracing::debug!(
                    "Tokens used: {} (${:.4})",
                    token_event.total_tokens,
                    token_event.cost_usd.unwrap_or(0.0)
                );
            }
            AIInteractionEvent::AIModelChanged(model_event) => {
                tracing::info!(
                    "AI model changed from {} to {}: {}",
                    model_event.old_model,
                    model_event.new_model,
                    model_event.reason
                );
            }
            AIInteractionEvent::RateLimitHit(rate_limit_event) => {
                metrics.rate_limit_hits += 1;
                tracing::warn!(
                    "Rate limit hit for {} model: {}",
                    rate_limit_event.model,
                    rate_limit_event.limit_type
                );
            }
        }
        
        Ok(())
    }
}

#[async_trait]
impl EventHandler for AIEventHandler {
    async fn handle(&self, event: &EventEnvelope) -> EventResult<()> {
        let ai_event: AIInteractionEvent = serde_json::from_value(event.event_data.clone())
            .map_err(|e| EventError::SerializationError {
                message: format!("Failed to deserialize AI interaction event: {}", e),
            })?;
        
        self.handle_ai_event(&ai_event).await
    }
    
    fn event_types(&self) -> Vec<String> {
        vec!["ai_interaction_event".to_string()]
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// Handler for service call events (MCP interactions)
pub struct ServiceEventHandler {
    name: String,
    metrics: Arc<RwLock<ServiceMetrics>>,
}

#[derive(Debug, Default, Clone)]
pub struct ServiceMetrics {
    pub total_calls: u64,
    pub successful_calls: u64,
    pub failed_calls: u64,
    pub calls_by_service: std::collections::HashMap<String, u64>,
    pub average_call_duration_ms: f64,
    pub registered_services: std::collections::HashSet<String>,
}

impl ServiceEventHandler {
    pub fn new() -> Self {
        Self {
            name: "service_event_handler".to_string(),
            metrics: Arc::new(RwLock::new(ServiceMetrics::default())),
        }
    }
    
    pub async fn get_metrics(&self) -> ServiceMetrics {
        let metrics = self.metrics.read().await;
        (*metrics).clone()
    }
    
    async fn handle_service_event(&self, event: &ServiceCallEvent) -> EventResult<()> {
        let mut metrics = self.metrics.write().await;
        
        match event {
            ServiceCallEvent::MCPCallStarted(call_event) => {
                metrics.total_calls += 1;
                *metrics.calls_by_service.entry(call_event.service_name.clone()).or_insert(0) += 1;
                tracing::debug!(
                    "MCP call started: {} -> {}",
                    call_event.service_name,
                    call_event.tool_name
                );
            }
            ServiceCallEvent::MCPCallCompleted(completed_event) => {
                metrics.successful_calls += 1;
                metrics.average_call_duration_ms = 
                    (metrics.average_call_duration_ms + completed_event.duration_ms as f64) / 2.0;
                tracing::info!(
                    "MCP call completed: {} -> {} in {}ms",
                    completed_event.service_name,
                    completed_event.tool_name,
                    completed_event.duration_ms
                );
            }
            ServiceCallEvent::MCPCallFailed(failed_event) => {
                metrics.failed_calls += 1;
                tracing::error!(
                    "MCP call failed: {} -> {}: {}",
                    failed_event.service_name,
                    failed_event.tool_name,
                    failed_event.error_message
                );
            }
            ServiceCallEvent::ServiceRegistered(registered_event) => {
                metrics.registered_services.insert(registered_event.service_name.clone());
                tracing::info!(
                    "Service registered: {} at {}",
                    registered_event.service_name,
                    registered_event.endpoint
                );
            }
            ServiceCallEvent::ServiceUnregistered(unregistered_event) => {
                metrics.registered_services.remove(&unregistered_event.service_name);
                tracing::warn!(
                    "Service unregistered: {} ({})",
                    unregistered_event.service_name,
                    unregistered_event.reason
                );
            }
            ServiceCallEvent::ServiceHealthCheckFailed(health_event) => {
                tracing::error!(
                    "Service health check failed: {} ({} consecutive failures)",
                    health_event.service_name,
                    health_event.consecutive_failures
                );
            }
        }
        
        Ok(())
    }
}

#[async_trait]
impl EventHandler for ServiceEventHandler {
    async fn handle(&self, event: &EventEnvelope) -> EventResult<()> {
        let service_event: ServiceCallEvent = serde_json::from_value(event.event_data.clone())
            .map_err(|e| EventError::SerializationError {
                message: format!("Failed to deserialize service call event: {}", e),
            })?;
        
        self.handle_service_event(&service_event).await
    }
    
    fn event_types(&self) -> Vec<String> {
        vec!["service_call_event".to_string()]
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// Handler for system events (errors, performance metrics)
pub struct SystemEventHandler {
    name: String,
    metrics: Arc<RwLock<SystemMetrics>>,
}

#[derive(Debug, Default, Clone)]
pub struct SystemMetrics {
    pub total_errors: u64,
    pub errors_by_severity: std::collections::HashMap<String, u64>,
    pub performance_metrics: std::collections::HashMap<String, f64>,
    pub configuration_changes: u64,
    pub database_connection_failures: u64,
}

impl SystemEventHandler {
    pub fn new() -> Self {
        Self {
            name: "system_event_handler".to_string(),
            metrics: Arc::new(RwLock::new(SystemMetrics::default())),
        }
    }
    
    pub async fn get_metrics(&self) -> SystemMetrics {
        let metrics = self.metrics.read().await;
        (*metrics).clone()
    }
    
    async fn handle_system_event(&self, event: &SystemEvent) -> EventResult<()> {
        let mut metrics = self.metrics.write().await;
        
        match event {
            SystemEvent::SystemStarted(start_event) => {
                tracing::info!(
                    "System started: version {} in {} environment",
                    start_event.version,
                    start_event.environment
                );
            }
            SystemEvent::SystemShutdown(shutdown_event) => {
                tracing::info!(
                    "System shutdown: {} (graceful: {}, uptime: {}s)",
                    shutdown_event.reason,
                    shutdown_event.graceful,
                    shutdown_event.uptime_seconds
                );
            }
            SystemEvent::ErrorOccurred(error_event) => {
                metrics.total_errors += 1;
                *metrics.errors_by_severity.entry(error_event.severity.clone()).or_insert(0) += 1;
                tracing::error!(
                    "System error in {}: {} ({})",
                    error_event.component,
                    error_event.error_message,
                    error_event.severity
                );
            }
            SystemEvent::PerformanceMetric(perf_event) => {
                metrics.performance_metrics.insert(
                    format!("{}_{}", perf_event.component, perf_event.metric_name),
                    perf_event.metric_value,
                );
                tracing::debug!(
                    "Performance metric: {}.{} = {} ({})",
                    perf_event.component,
                    perf_event.metric_name,
                    perf_event.metric_value,
                    perf_event.metric_type
                );
            }
            SystemEvent::ConfigurationChanged(config_event) => {
                metrics.configuration_changes += 1;
                tracing::info!(
                    "Configuration changed in component: {}",
                    config_event.component
                );
            }
            SystemEvent::DatabaseConnectionFailed(db_event) => {
                metrics.database_connection_failures += 1;
                tracing::error!(
                    "Database connection failed: {} (retry #{} at {})",
                    db_event.database_name,
                    db_event.retry_count,
                    db_event.next_retry_at
                );
            }
        }
        
        Ok(())
    }
}

#[async_trait]
impl EventHandler for SystemEventHandler {
    async fn handle(&self, event: &EventEnvelope) -> EventResult<()> {
        let system_event: SystemEvent = serde_json::from_value(event.event_data.clone())
            .map_err(|e| EventError::SerializationError {
                message: format!("Failed to deserialize system event: {}", e),
            })?;
        
        self.handle_system_event(&system_event).await
    }
    
    fn event_types(&self) -> Vec<String> {
        vec!["system_event".to_string()]
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// Composite handler that aggregates metrics from all domains
pub struct MetricsAggregatorHandler {
    name: String,
    workflow_handler: Arc<WorkflowEventHandler>,
    ai_handler: Arc<AIEventHandler>,
    service_handler: Arc<ServiceEventHandler>,
    system_handler: Arc<SystemEventHandler>,
}

impl MetricsAggregatorHandler {
    pub fn new(
        workflow_handler: Arc<WorkflowEventHandler>,
        ai_handler: Arc<AIEventHandler>,
        service_handler: Arc<ServiceEventHandler>,
        system_handler: Arc<SystemEventHandler>,
    ) -> Self {
        Self {
            name: "metrics_aggregator_handler".to_string(),
            workflow_handler,
            ai_handler,
            service_handler,
            system_handler,
        }
    }
    
    pub async fn get_aggregate_metrics(&self) -> AggregateMetrics {
        let workflow_metrics = self.workflow_handler.get_metrics().await;
        let ai_metrics = self.ai_handler.get_metrics().await;
        let service_metrics = self.service_handler.get_metrics().await;
        let system_metrics = self.system_handler.get_metrics().await;
        
        AggregateMetrics {
            workflow: workflow_metrics,
            ai: ai_metrics,
            service: service_metrics,
            system: system_metrics,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AggregateMetrics {
    pub workflow: WorkflowMetrics,
    pub ai: AIMetrics,
    pub service: ServiceMetrics,
    pub system: SystemMetrics,
}

#[async_trait]
impl EventHandler for MetricsAggregatorHandler {
    async fn handle(&self, _event: &EventEnvelope) -> EventResult<()> {
        // This handler doesn't process individual events, 
        // it just provides aggregated metrics on demand
        Ok(())
    }
    
    fn event_types(&self) -> Vec<String> {
        // Don't handle any events directly
        vec![]
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn should_handle(&self, _event: &EventEnvelope) -> bool {
        // Never handle events directly
        false
    }
}