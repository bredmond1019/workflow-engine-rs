//! Service health monitoring and automated recovery
//!
//! This module provides comprehensive health checking including:
//! - Active and passive health checks
//! - Circuit breaker integration
//! - Automated recovery procedures
//! - Health aggregation and reporting

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;

use workflow_engine_core::error::{WorkflowError, ErrorContext, ErrorContextExt, retry_with_policy, RetryPolicy};
use workflow_engine_core::error::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use super::registry::{ServiceRegistry, HealthStatus, ServiceInstance};
use super::config::HealthCheckConfig;

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Service instance ID
    pub instance_id: Uuid,
    
    /// Health status
    pub status: HealthStatus,
    
    /// Check timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Response time in milliseconds
    pub response_time_ms: u64,
    
    /// Additional health details
    pub details: HealthDetails,
    
    /// Error message if check failed
    pub error: Option<String>,
}

/// Detailed health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthDetails {
    /// Service version
    pub version: String,
    
    /// Uptime in seconds
    pub uptime_seconds: u64,
    
    /// Current load percentage
    pub load_percentage: f32,
    
    /// Available memory in MB
    pub available_memory_mb: u64,
    
    /// Active connections
    pub active_connections: u32,
    
    /// Custom health metrics
    pub custom_metrics: HashMap<String, serde_json::Value>,
}

/// Health check strategy
#[async_trait]
pub trait HealthCheckStrategy: Send + Sync {
    /// Perform health check
    async fn check_health(&self, instance: &ServiceInstance) -> Result<HealthCheckResult, WorkflowError>;
}

/// HTTP-based health check
pub struct HttpHealthCheck {
    client: reqwest::Client,
}

impl HttpHealthCheck {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap(),
        }
    }
}

#[async_trait]
impl HealthCheckStrategy for HttpHealthCheck {
    async fn check_health(&self, instance: &ServiceInstance) -> Result<HealthCheckResult, WorkflowError> {
        let health_url = format!("{}/health", instance.endpoint);
        let start_time = std::time::Instant::now();
        
        match self.client.get(&health_url).send().await {
            Ok(response) => {
                let response_time_ms = start_time.elapsed().as_millis() as u64;
                
                if response.status().is_success() {
                    // Try to parse health details from response
                    let details = if let Ok(body) = response.json::<HealthDetails>().await {
                        body
                    } else {
                        // Default health details if parsing fails
                        HealthDetails {
                            version: instance.version.clone(),
                            uptime_seconds: 0,
                            load_percentage: 0.0,
                            available_memory_mb: 0,
                            active_connections: 0,
                            custom_metrics: HashMap::new(),
                        }
                    };
                    
                    Ok(HealthCheckResult {
                        instance_id: instance.id,
                        status: HealthStatus::Healthy,
                        timestamp: Utc::now(),
                        response_time_ms,
                        details,
                        error: None,
                    })
                } else {
                    Ok(HealthCheckResult {
                        instance_id: instance.id,
                        status: HealthStatus::Unhealthy,
                        timestamp: Utc::now(),
                        response_time_ms,
                        details: HealthDetails {
                            version: instance.version.clone(),
                            uptime_seconds: 0,
                            load_percentage: 0.0,
                            available_memory_mb: 0,
                            active_connections: 0,
                            custom_metrics: HashMap::new(),
                        },
                        error: Some(format!("HTTP {}", response.status())),
                    })
                }
            }
            Err(e) => Ok(HealthCheckResult {
                instance_id: instance.id,
                status: HealthStatus::Unhealthy,
                timestamp: Utc::now(),
                response_time_ms: start_time.elapsed().as_millis() as u64,
                details: HealthDetails {
                    version: instance.version.clone(),
                    uptime_seconds: 0,
                    load_percentage: 0.0,
                    available_memory_mb: 0,
                    active_connections: 0,
                    custom_metrics: HashMap::new(),
                },
                error: Some(e.to_string()),
            }),
        }
    }
}

/// Health monitor that performs periodic health checks
pub struct HealthMonitor {
    registry: Arc<ServiceRegistry>,
    strategy: Arc<dyn HealthCheckStrategy>,
    circuit_breakers: Arc<RwLock<HashMap<Uuid, CircuitBreaker>>>,
    health_history: Arc<RwLock<HashMap<Uuid, Vec<HealthCheckResult>>>>,
    recovery_handlers: Arc<RwLock<HashMap<String, Arc<dyn RecoveryHandler>>>>,
}

/// Recovery handler for automated recovery procedures
#[async_trait]
pub trait RecoveryHandler: Send + Sync {
    /// Attempt to recover a service
    async fn recover(&self, instance: &ServiceInstance, failure_count: u32) -> Result<(), WorkflowError>;
}

/// Default recovery handler that logs and notifies
pub struct DefaultRecoveryHandler;

#[async_trait]
impl RecoveryHandler for DefaultRecoveryHandler {
    async fn recover(&self, instance: &ServiceInstance, failure_count: u32) -> Result<(), WorkflowError> {
        tracing::warn!(
            "Service {} (ID: {}) has failed {} times. Manual intervention may be required.",
            instance.name,
            instance.id,
            failure_count
        );
        
        // In a real implementation, this could:
        // - Send alerts to monitoring systems
        // - Attempt to restart the service
        // - Trigger failover procedures
        // - Scale up healthy instances
        
        Ok(())
    }
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new(
        registry: Arc<ServiceRegistry>,
        strategy: Arc<dyn HealthCheckStrategy>,
    ) -> Self {
        Self {
            registry,
            strategy,
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
            health_history: Arc::new(RwLock::new(HashMap::new())),
            recovery_handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a recovery handler for a service
    pub async fn register_recovery_handler(
        &self,
        service_name: String,
        handler: Arc<dyn RecoveryHandler>,
    ) {
        let mut handlers = self.recovery_handlers.write().await;
        handlers.insert(service_name, handler);
    }
    
    /// Start monitoring all registered services
    pub async fn start_monitoring(&self, default_interval: Duration) {
        let monitor = self.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(default_interval);
            
            loop {
                interval.tick().await;
                
                if let Err(e) = monitor.check_all_services().await {
                    tracing::error!("Error during health monitoring: {}", e);
                }
            }
        });
        
        tracing::info!("Started health monitoring with interval: {:?}", default_interval);
    }
    
    /// Check health of all registered services
    pub async fn check_all_services(&self) -> Result<(), WorkflowError> {
        let services = self.registry.list_services().await;
        
        for service_name in services {
            if let Ok(instances) = self.registry.get_service_instances(&service_name).await {
                for instance in instances {
                    if let Err(e) = self.check_instance_health(&instance).await {
                        tracing::error!(
                            "Failed to check health for {} ({}): {}",
                            instance.name,
                            instance.id,
                            e
                        );
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Check health of a specific instance
    pub async fn check_instance_health(&self, instance: &ServiceInstance) -> Result<(), WorkflowError> {
        // Check if we have a circuit breaker for this instance
        let has_breaker = {
            let breakers = self.circuit_breakers.read().await;
            breakers.contains_key(&instance.id)
        };
        
        // Create circuit breaker if it doesn't exist
        if !has_breaker {
            let mut breakers = self.circuit_breakers.write().await;
            breakers.insert(
                instance.id,
                CircuitBreaker::new(CircuitBreakerConfig {
                    failure_threshold: 5,
                    success_threshold: 3,
                    timeout: Duration::from_secs(60),
                    window: Duration::from_secs(60),
                    on_state_change: None,
                })
            );
        }
        
        // Now use the circuit breaker
        let strategy = self.strategy.clone();
        let instance_clone = instance.clone();
        
        let result = {
            let breakers = self.circuit_breakers.read().await;
            if let Some(circuit_breaker) = breakers.get(&instance.id) {
                circuit_breaker.call(|| async move {
                    strategy.check_health(&instance_clone).await
                }).await
            } else {
                return Err(WorkflowError::RuntimeError {
                    message: "Circuit breaker not found".to_string()
                });
            }
        };
        
        // Check if circuit breaker allows the call
        match result {
            Ok(result) => {
                // Update registry with health status
                self.registry.update_health_status(instance.id, result.status.clone()).await?;
                
                // Store health history
                {
                    let mut history = self.health_history.write().await;
                    let instance_history = history.entry(instance.id).or_insert_with(Vec::new);
                    instance_history.push(result.clone());
                    
                    // Keep only last 100 results
                    if instance_history.len() > 100 {
                        instance_history.remove(0);
                    }
                }
                
                // Handle unhealthy status
                if result.status == HealthStatus::Unhealthy {
                    self.handle_unhealthy_instance(instance).await?;
                }
                
                Ok(())
            }
            Err(e) => {
                // Circuit breaker is open or call failed
                tracing::warn!(
                    "Health check blocked by circuit breaker for {} ({}): {}",
                    instance.name,
                    instance.id,
                    e
                );
                
                // Mark as unhealthy
                self.registry.update_health_status(instance.id, HealthStatus::Unhealthy).await?;
                self.handle_unhealthy_instance(instance).await?;
                
                Err(e)
            }
        }
    }
    
    /// Handle an unhealthy instance
    async fn handle_unhealthy_instance(&self, instance: &ServiceInstance) -> Result<(), WorkflowError> {
        // Count consecutive failures
        let failure_count = {
            let history = self.health_history.read().await;
            if let Some(instance_history) = history.get(&instance.id) {
                instance_history.iter()
                    .rev()
                    .take_while(|r| r.status == HealthStatus::Unhealthy)
                    .count() as u32
            } else {
                1
            }
        };
        
        // Trigger recovery if threshold exceeded
        if failure_count >= 3 {
            let handlers = self.recovery_handlers.read().await;
            let handler = handlers.get(&instance.name)
                .cloned()
                .unwrap_or_else(|| Arc::new(DefaultRecoveryHandler));
                
            handler.recover(instance, failure_count).await?;
        }
        
        Ok(())
    }
    
    /// Get health history for an instance
    pub async fn get_health_history(&self, instance_id: Uuid) -> Vec<HealthCheckResult> {
        let history = self.health_history.read().await;
        history.get(&instance_id).cloned().unwrap_or_default()
    }
    
    /// Get aggregated health statistics for a service
    pub async fn get_service_health_stats(&self, service_name: &str) -> Result<ServiceHealthStats, WorkflowError> {
        let instances = self.registry.get_service_instances(service_name).await?;
        let history = self.health_history.read().await;
        
        let mut total_checks = 0;
        let mut successful_checks = 0;
        let mut total_response_time = 0u64;
        let mut status_counts: HashMap<String, usize> = HashMap::new();
        
        for instance in &instances {
            if let Some(instance_history) = history.get(&instance.id) {
                for result in instance_history {
                    total_checks += 1;
                    if result.status == HealthStatus::Healthy {
                        successful_checks += 1;
                    }
                    total_response_time += result.response_time_ms;
                    
                    let status_str = format!("{:?}", result.status);
                    *status_counts.entry(status_str).or_insert(0) += 1;
                }
            }
        }
        
        let availability = if total_checks > 0 {
            (successful_checks as f64 / total_checks as f64) * 100.0
        } else {
            0.0
        };
        
        let avg_response_time = if total_checks > 0 {
            total_response_time / total_checks as u64
        } else {
            0
        };
        
        Ok(ServiceHealthStats {
            service_name: service_name.to_string(),
            total_instances: instances.len(),
            healthy_instances: instances.iter().filter(|i| i.health_status == HealthStatus::Healthy).count(),
            availability_percentage: availability,
            avg_response_time_ms: avg_response_time,
            status_distribution: status_counts,
        })
    }
}

impl Clone for HealthMonitor {
    fn clone(&self) -> Self {
        Self {
            registry: self.registry.clone(),
            strategy: self.strategy.clone(),
            circuit_breakers: self.circuit_breakers.clone(),
            health_history: self.health_history.clone(),
            recovery_handlers: self.recovery_handlers.clone(),
        }
    }
}

/// Service health statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealthStats {
    pub service_name: String,
    pub total_instances: usize,
    pub healthy_instances: usize,
    pub availability_percentage: f64,
    pub avg_response_time_ms: u64,
    pub status_distribution: HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use workflow_engine_core::registry::agent_registry::MockAgentRegistry;
    
    #[tokio::test]
    async fn test_http_health_check() {
        let health_check = HttpHealthCheck::new();
        
        let instance = ServiceInstance {
            id: Uuid::new_v4(),
            name: "test-service".to_string(),
            version: "1.0.0".to_string(),
            endpoint: "http://localhost:8080".to_string(),
            capabilities: vec![],
            metadata: super::super::registry::ServiceMetadata {
                tags: vec![],
                environment: "test".to_string(),
                region: "us-east-1".to_string(),
                attributes: HashMap::new(),
                api_version: "v1".to_string(),
                protocol: "http".to_string(),
            },
            health_status: HealthStatus::Unknown,
            load_metrics: Default::default(),
            registered_at: Utc::now(),
            last_seen: Utc::now(),
        };
        
        // This will fail since no server is running, but we're testing the structure
        let result = health_check.check_health(&instance).await.unwrap();
        assert_eq!(result.instance_id, instance.id);
        assert!(result.error.is_some()); // Expected to fail
    }
    
    #[tokio::test]
    async fn test_health_monitor_creation() {
        let mock_registry = MockAgentRegistry::new();
        let service_registry = Arc::new(ServiceRegistry::new(Arc::new(mock_registry)));
        let health_check = Arc::new(HttpHealthCheck::new());
        
        let monitor = HealthMonitor::new(service_registry, health_check);
        
        // Register a recovery handler
        monitor.register_recovery_handler(
            "test-service".to_string(),
            Arc::new(DefaultRecoveryHandler)
        ).await;
        
        // Test structure is created properly
        assert_eq!(monitor.get_health_history(Uuid::new_v4()).await.len(), 0);
    }
}