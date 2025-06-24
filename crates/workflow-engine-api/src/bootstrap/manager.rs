//! Comprehensive service bootstrap manager
//!
//! This module provides a unified interface for service management including:
//! - Service registration and discovery
//! - Health monitoring and recovery
//! - Lifecycle management
//! - Configuration hot-reload

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;

use workflow_engine_core::error::{WorkflowError, ErrorContext, ErrorContextExt};
use workflow_engine_core::registry::agent_registry::AgentRegistry;
use super::{
    config::{ConfigurationManager, ServiceConfiguration},
    discovery::{ServiceDiscovery, DiscoveryClient, RegistryDiscovery},
    health::{HealthMonitor, HttpHealthCheck, ServiceHealthStats},
    lifecycle::{ServiceLifecycleManager, ServiceState, ServiceLifecycleHooks},
    registry::{ServiceRegistry, ServiceInstance, LoadBalancingStrategy},
};

/// Comprehensive service bootstrap manager
pub struct ServiceBootstrapManager {
    /// Service registry
    registry: Arc<ServiceRegistry>,
    
    /// Configuration manager
    config_manager: Arc<ConfigurationManager>,
    
    /// Service discovery
    discovery: Arc<dyn ServiceDiscovery>,
    
    /// Discovery client with caching
    discovery_client: Arc<DiscoveryClient>,
    
    /// Health monitor
    health_monitor: Arc<HealthMonitor>,
    
    /// Lifecycle manager
    lifecycle_manager: Arc<ServiceLifecycleManager>,
}

impl ServiceBootstrapManager {
    /// Create a new service bootstrap manager
    pub fn new(
        agent_registry: Arc<dyn AgentRegistry>,
        environment: String,
    ) -> Self {
        // Create core components
        let registry = Arc::new(ServiceRegistry::new(agent_registry));
        let config_manager = Arc::new(ConfigurationManager::new(environment));
        let discovery = Arc::new(RegistryDiscovery::new(registry.clone()));
        let discovery_client = Arc::new(DiscoveryClient::new(
            discovery.clone(),
            Duration::from_secs(30), // 30 second cache TTL
        ));
        let health_monitor = Arc::new(HealthMonitor::new(
            registry.clone(),
            Arc::new(HttpHealthCheck::new()),
        ));
        let lifecycle_manager = Arc::new(ServiceLifecycleManager::new(
            registry.clone(),
            config_manager.clone(),
            discovery.clone(),
        ));
        
        Self {
            registry,
            config_manager,
            discovery,
            discovery_client,
            health_monitor,
            lifecycle_manager,
        }
    }
    
    /// Initialize the manager with configuration file
    pub async fn initialize(&self, config_path: Option<&str>) -> Result<(), WorkflowError> {
        // Load configuration if provided
        if let Some(path) = config_path {
            self.config_manager.load_from_file(path).await?;
            
            // Start configuration hot-reload
            self.config_manager.start_hot_reload(60).await; // Check every 60 seconds
        }
        
        // Start health monitoring
        self.health_monitor.start_monitoring(Duration::from_secs(30)).await;
        
        // Setup graceful shutdown
        self.lifecycle_manager.setup_graceful_shutdown().await;
        
        tracing::info!("Service bootstrap manager initialized");
        
        Ok(())
    }
    
    /// Register a service
    pub async fn register_service(
        &self,
        config: ServiceConfiguration,
        hooks: Option<Arc<dyn ServiceLifecycleHooks>>,
    ) -> Result<(), WorkflowError> {
        // Add configuration
        self.config_manager.upsert_configuration(config.clone()).await?;
        
        // Register with lifecycle manager
        self.lifecycle_manager.register_service(config, hooks).await?;
        
        Ok(())
    }
    
    /// Start a service
    pub async fn start_service(&self, service_name: &str) -> Result<(), WorkflowError> {
        self.lifecycle_manager.start_service(service_name).await
    }
    
    /// Stop a service
    pub async fn stop_service(&self, service_name: &str) -> Result<(), WorkflowError> {
        self.lifecycle_manager.stop_service(service_name).await
    }
    
    /// Restart a service
    pub async fn restart_service(&self, service_name: &str) -> Result<(), WorkflowError> {
        self.lifecycle_manager.restart_service(service_name).await
    }
    
    /// Start all registered services
    pub async fn start_all_services(&self) -> Result<(), WorkflowError> {
        self.lifecycle_manager.start_all().await
    }
    
    /// Stop all services
    pub async fn stop_all_services(&self) -> Result<(), WorkflowError> {
        self.lifecycle_manager.stop_all().await
    }
    
    /// Get service by name
    pub async fn get_service(&self, service_name: &str) -> Result<Vec<ServiceInstance>, WorkflowError> {
        self.discovery_client.discover_service(service_name).await
    }
    
    /// Get services by capability
    pub async fn get_services_by_capability(&self, capability: &str) -> Result<Vec<ServiceInstance>, WorkflowError> {
        self.discovery_client.discover_by_capability(capability).await
    }
    
    /// Select a service instance with load balancing
    pub async fn select_service_instance(
        &self,
        service_name: &str,
        strategy: LoadBalancingStrategy,
    ) -> Result<ServiceInstance, WorkflowError> {
        self.registry.select_instance(service_name, strategy).await
    }
    
    /// Get service state
    pub async fn get_service_state(&self, service_name: &str) -> Result<ServiceState, WorkflowError> {
        self.lifecycle_manager.get_service_state(service_name).await
    }
    
    /// Get all service states
    pub async fn get_all_service_states(&self) -> std::collections::HashMap<String, ServiceState> {
        self.lifecycle_manager.get_all_states().await
    }
    
    /// Get service health statistics
    pub async fn get_service_health_stats(&self, service_name: &str) -> Result<ServiceHealthStats, WorkflowError> {
        self.health_monitor.get_service_health_stats(service_name).await
    }
    
    /// Update service health status
    pub async fn update_health_status(
        &self,
        instance_id: Uuid,
        status: super::registry::HealthStatus,
    ) -> Result<(), WorkflowError> {
        self.registry.update_health_status(instance_id, status).await
    }
    
    /// Update service load metrics
    pub async fn update_load_metrics(
        &self,
        instance_id: Uuid,
        metrics: super::registry::LoadMetrics,
    ) -> Result<(), WorkflowError> {
        self.registry.update_load_metrics(instance_id, metrics).await
    }
    
    /// Trigger graceful shutdown
    pub async fn shutdown(&self) -> Result<(), WorkflowError> {
        tracing::info!("Initiating service bootstrap manager shutdown");
        
        // Stop all services
        self.lifecycle_manager.trigger_shutdown().await;
        
        // Wait a bit for cleanup
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        tracing::info!("Service bootstrap manager shutdown complete");
        
        Ok(())
    }
    
    /// Get service configuration
    pub async fn get_service_configuration(&self, service_name: &str) -> Result<ServiceConfiguration, WorkflowError> {
        self.config_manager.get_configuration(service_name).await
    }
    
    /// Validate service dependencies
    pub async fn validate_dependencies(&self, service_name: &str) -> Result<Vec<String>, WorkflowError> {
        self.config_manager.validate_dependencies(service_name).await
    }
    
    /// Health check a specific service instance
    pub async fn health_check_instance(&self, instance: &ServiceInstance) -> Result<(), WorkflowError> {
        self.health_monitor.check_instance_health(instance).await
    }
    
    /// Clear discovery cache
    pub async fn clear_discovery_cache(&self) {
        self.discovery_client.clear_cache().await
    }
}

/// Builder for ServiceBootstrapManager
pub struct ServiceBootstrapManagerBuilder {
    agent_registry: Option<Arc<dyn AgentRegistry>>,
    environment: String,
    config_path: Option<String>,
    health_check_interval: Duration,
    cache_ttl: Duration,
}

impl ServiceBootstrapManagerBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            agent_registry: None,
            environment: "development".to_string(),
            config_path: None,
            health_check_interval: Duration::from_secs(30),
            cache_ttl: Duration::from_secs(30),
        }
    }
    
    /// Set the agent registry
    pub fn with_agent_registry(mut self, registry: Arc<dyn AgentRegistry>) -> Self {
        self.agent_registry = Some(registry);
        self
    }
    
    /// Set the environment
    pub fn with_environment(mut self, environment: String) -> Self {
        self.environment = environment;
        self
    }
    
    /// Set the configuration file path
    pub fn with_config_file(mut self, path: String) -> Self {
        self.config_path = Some(path);
        self
    }
    
    /// Set the health check interval
    pub fn with_health_check_interval(mut self, interval: Duration) -> Self {
        self.health_check_interval = interval;
        self
    }
    
    /// Set the cache TTL
    pub fn with_cache_ttl(mut self, ttl: Duration) -> Self {
        self.cache_ttl = ttl;
        self
    }
    
    /// Build the manager
    pub async fn build(self) -> Result<ServiceBootstrapManager, WorkflowError> {
        let registry = self.agent_registry
            .ok_or_else(|| WorkflowError::configuration_error_simple(
                "Agent registry is required".to_string()
            ))?;
            
        let manager = ServiceBootstrapManager::new(registry, self.environment);
        
        // Initialize with config if provided
        manager.initialize(self.config_path.as_deref()).await?;
        
        Ok(manager)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::mocks::MockAgentRegistry;
    
    #[tokio::test]
    async fn test_bootstrap_manager_creation() {
        let mock_registry = MockAgentRegistry::new();
        let manager = ServiceBootstrapManager::new(
            Arc::new(mock_registry),
            "test".to_string(),
        );
        
        // Test initialization
        manager.initialize(None).await.unwrap();
        
        // Test service states
        let states = manager.get_all_service_states().await;
        assert!(states.is_empty());
    }
    
    #[tokio::test]
    async fn test_builder_pattern() {
        let mock_registry = MockAgentRegistry::new();
        
        let manager = ServiceBootstrapManagerBuilder::new()
            .with_agent_registry(Arc::new(mock_registry))
            .with_environment("production".to_string())
            .with_health_check_interval(Duration::from_secs(60))
            .with_cache_ttl(Duration::from_secs(120))
            .build()
            .await
            .unwrap();
            
        // Test that manager is properly configured
        let states = manager.get_all_service_states().await;
        assert!(states.is_empty());
    }
}