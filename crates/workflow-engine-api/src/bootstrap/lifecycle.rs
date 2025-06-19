//! Service lifecycle management
//!
//! This module provides service lifecycle management including:
//! - Startup sequencing with dependency resolution
//! - Graceful shutdown procedures
//! - State transitions and hooks
//! - Resource cleanup

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use tokio::task::JoinHandle;
use uuid::Uuid;

use workflow_engine_core::error::{WorkflowError, ErrorContext, ErrorContextExt};
use super::config::{ServiceConfiguration, ConfigurationManager};
use super::registry::ServiceRegistry;
use super::discovery::ServiceDiscovery;

/// Service lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceState {
    /// Service is not yet started
    Uninitialized,
    
    /// Service is starting up
    Starting,
    
    /// Service is running and healthy
    Running,
    
    /// Service is stopping
    Stopping,
    
    /// Service has stopped
    Stopped,
    
    /// Service failed to start or crashed
    Failed,
}

/// Service lifecycle hooks
#[async_trait]
pub trait ServiceLifecycleHooks: Send + Sync {
    /// Called before service starts
    async fn pre_start(&self, config: &ServiceConfiguration) -> Result<(), WorkflowError>;
    
    /// Called after service starts successfully
    async fn post_start(&self, config: &ServiceConfiguration) -> Result<(), WorkflowError>;
    
    /// Called before service stops
    async fn pre_stop(&self, config: &ServiceConfiguration) -> Result<(), WorkflowError>;
    
    /// Called after service stops
    async fn post_stop(&self, config: &ServiceConfiguration) -> Result<(), WorkflowError>;
    
    /// Called when service fails
    async fn on_failure(&self, config: &ServiceConfiguration, error: &WorkflowError) -> Result<(), WorkflowError>;
}

/// Service instance managed by lifecycle manager
pub struct ManagedService {
    /// Service configuration
    pub config: ServiceConfiguration,
    
    /// Current state
    pub state: ServiceState,
    
    /// Service process handle (if applicable)
    pub handle: Option<JoinHandle<()>>,
    
    /// Lifecycle hooks
    pub hooks: Option<Arc<dyn ServiceLifecycleHooks>>,
    
    /// Instance ID
    pub instance_id: Option<Uuid>,
}

/// Service lifecycle manager
pub struct ServiceLifecycleManager {
    /// Managed services
    services: Arc<RwLock<HashMap<String, ManagedService>>>,
    
    /// Service registry
    registry: Arc<ServiceRegistry>,
    
    /// Configuration manager
    config_manager: Arc<ConfigurationManager>,
    
    /// Service discovery
    discovery: Arc<dyn ServiceDiscovery>,
    
    /// Shutdown signal
    shutdown_signal: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
}

impl ServiceLifecycleManager {
    /// Create a new lifecycle manager
    pub fn new(
        registry: Arc<ServiceRegistry>,
        config_manager: Arc<ConfigurationManager>,
        discovery: Arc<dyn ServiceDiscovery>,
    ) -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            registry,
            config_manager,
            discovery,
            shutdown_signal: Arc::new(Mutex::new(None)),
        }
    }
    
    /// Register a service with the lifecycle manager
    pub async fn register_service(
        &self,
        config: ServiceConfiguration,
        hooks: Option<Arc<dyn ServiceLifecycleHooks>>,
    ) -> Result<(), WorkflowError> {
        let mut services = self.services.write().await;
        
        if services.contains_key(&config.name) {
            return Err(WorkflowError::ConfigurationError(
                format!("Service {} is already registered", config.name)
            ));
        }
        
        services.insert(config.name.clone(), ManagedService {
            config,
            state: ServiceState::Uninitialized,
            handle: None,
            hooks,
            instance_id: None,
        });
        
        Ok(())
    }
    
    /// Start all registered services in dependency order
    pub async fn start_all(&self) -> Result<(), WorkflowError> {
        let start_order = self.calculate_start_order().await?;
        
        for service_name in start_order {
            if let Err(e) = self.start_service(&service_name).await {
                tracing::error!("Failed to start service {}: {}", service_name, e);
                // Continue starting other services
            }
        }
        
        Ok(())
    }
    
    /// Start a specific service
    pub async fn start_service(&self, service_name: &str) -> Result<(), WorkflowError> {
        let mut services = self.services.write().await;
        
        let service = services.get_mut(service_name)
            .ok_or_else(|| WorkflowError::ConfigurationError(
                format!("Service {} not registered", service_name)
            ))?;
            
        if service.state == ServiceState::Running {
            return Ok(()); // Already running
        }
        
        // Update state
        service.state = ServiceState::Starting;
        let config = service.config.clone();
        let hooks = service.hooks.clone();
        
        // Call pre-start hook
        if let Some(ref hooks) = hooks {
            hooks.pre_start(&config).await?;
        }
        
        // Check dependencies
        self.check_dependencies(&config).await?;
        
        // Register with service discovery
        self.discovery.register_service(&config).await?;
        
        // Register with service registry
        let metadata = super::registry::ServiceMetadata {
            tags: vec![],
            environment: config.environment.clone(),
            region: "default".to_string(),
            attributes: HashMap::new(),
            api_version: "v1".to_string(),
            protocol: "http".to_string(),
        };
        
        let instance = self.registry.register_instance(&config, metadata).await?;
        service.instance_id = Some(instance.id);
        
        // Update state to running
        service.state = ServiceState::Running;
        
        // Call post-start hook
        if let Some(ref hooks) = hooks {
            hooks.post_start(&config).await?;
        }
        
        tracing::info!("Started service: {}", service_name);
        
        Ok(())
    }
    
    /// Stop all services in reverse dependency order
    pub async fn stop_all(&self) -> Result<(), WorkflowError> {
        let mut start_order = self.calculate_start_order().await?;
        start_order.reverse(); // Stop in reverse order
        
        for service_name in start_order {
            if let Err(e) = self.stop_service(&service_name).await {
                tracing::error!("Failed to stop service {}: {}", service_name, e);
                // Continue stopping other services
            }
        }
        
        Ok(())
    }
    
    /// Stop a specific service
    pub async fn stop_service(&self, service_name: &str) -> Result<(), WorkflowError> {
        let mut services = self.services.write().await;
        
        let service = services.get_mut(service_name)
            .ok_or_else(|| WorkflowError::ConfigurationError(
                format!("Service {} not registered", service_name)
            ))?;
            
        if service.state == ServiceState::Stopped {
            return Ok(()); // Already stopped
        }
        
        // Update state
        service.state = ServiceState::Stopping;
        let config = service.config.clone();
        let hooks = service.hooks.clone();
        let instance_id = service.instance_id;
        
        // Call pre-stop hook
        if let Some(ref hooks) = hooks {
            hooks.pre_stop(&config).await?;
        }
        
        // Unregister from service registry
        if let Some(id) = instance_id {
            self.registry.unregister_instance(id).await?;
            self.discovery.unregister_service(&config.name, id).await?;
        }
        
        // Cancel any running task
        if let Some(handle) = service.handle.take() {
            handle.abort();
        }
        
        // Update state
        service.state = ServiceState::Stopped;
        service.instance_id = None;
        
        // Call post-stop hook
        if let Some(ref hooks) = hooks {
            hooks.post_stop(&config).await?;
        }
        
        tracing::info!("Stopped service: {}", service_name);
        
        Ok(())
    }
    
    /// Restart a service
    pub async fn restart_service(&self, service_name: &str) -> Result<(), WorkflowError> {
        self.stop_service(service_name).await?;
        self.start_service(service_name).await?;
        Ok(())
    }
    
    /// Get service state
    pub async fn get_service_state(&self, service_name: &str) -> Result<ServiceState, WorkflowError> {
        let services = self.services.read().await;
        
        services.get(service_name)
            .map(|s| s.state)
            .ok_or_else(|| WorkflowError::ConfigurationError(
                format!("Service {} not registered", service_name)
            ))
    }
    
    /// Get all service states
    pub async fn get_all_states(&self) -> HashMap<String, ServiceState> {
        let services = self.services.read().await;
        
        services.iter()
            .map(|(name, service)| (name.clone(), service.state))
            .collect()
    }
    
    /// Calculate service startup order based on dependencies
    async fn calculate_start_order(&self) -> Result<Vec<String>, WorkflowError> {
        let services = self.services.read().await;
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        
        // Build dependency graph
        for (name, service) in services.iter() {
            in_degree.insert(name.clone(), 0);
            graph.insert(name.clone(), Vec::new());
        }
        
        for (name, service) in services.iter() {
            for dep in &service.config.dependencies {
                if !dep.optional {
                    if let Some(deps) = graph.get_mut(&dep.service_name) {
                        deps.push(name.clone());
                        *in_degree.get_mut(name).unwrap() += 1;
                    }
                }
            }
        }
        
        // Topological sort using Kahn's algorithm
        let mut queue: Vec<String> = in_degree.iter()
            .filter(|(_, degree)| **degree == 0)
            .map(|(name, _)| name.clone())
            .collect();
            
        let mut result = Vec::new();
        
        while let Some(current) = queue.pop() {
            result.push(current.clone());
            
            if let Some(deps) = graph.get(&current) {
                for dep in deps.clone() {
                    let degree = in_degree.get_mut(&dep).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push(dep);
                    }
                }
            }
        }
        
        // Check for cycles
        if result.len() != services.len() {
            return Err(WorkflowError::ConfigurationError(
                "Circular dependency detected in service configuration".to_string()
            ));
        }
        
        Ok(result)
    }
    
    /// Check if all dependencies are satisfied
    async fn check_dependencies(&self, config: &ServiceConfiguration) -> Result<(), WorkflowError> {
        for dep in &config.dependencies {
            if !dep.optional {
                // Check if dependency is running
                let state = self.get_service_state(&dep.service_name).await?;
                if state != ServiceState::Running {
                    return Err(WorkflowError::ConfigurationError(
                        format!("Dependency {} is not running", dep.service_name)
                    ));
                }
                
                // Check if dependency provides required capabilities
                let instances = self.discovery.discover_service(&dep.service_name).await?;
                if instances.is_empty() {
                    return Err(WorkflowError::ConfigurationError(
                        format!("No instances found for dependency {}", dep.service_name)
                    ));
                }
                
                // Verify capabilities
                for required_cap in &dep.required_capabilities {
                    let has_capability = instances.iter()
                        .any(|i| i.capabilities.contains(required_cap));
                    if !has_capability {
                        return Err(WorkflowError::ConfigurationError(
                            format!(
                                "Dependency {} does not provide required capability {}",
                                dep.service_name, required_cap
                            )
                        ));
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Setup graceful shutdown
    pub async fn setup_graceful_shutdown(&self) {
        let manager = self.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        
        *self.shutdown_signal.lock().await = Some(tx);
        
        tokio::spawn(async move {
            // Wait for shutdown signal
            let _ = rx.await;
            
            tracing::info!("Initiating graceful shutdown...");
            
            if let Err(e) = manager.stop_all().await {
                tracing::error!("Error during graceful shutdown: {}", e);
            }
            
            tracing::info!("Graceful shutdown complete");
        });
    }
    
    /// Trigger graceful shutdown
    pub async fn trigger_shutdown(&self) {
        if let Some(tx) = self.shutdown_signal.lock().await.take() {
            let _ = tx.send(());
        }
    }
}

impl Clone for ServiceLifecycleManager {
    fn clone(&self) -> Self {
        Self {
            services: self.services.clone(),
            registry: self.registry.clone(),
            config_manager: self.config_manager.clone(),
            discovery: self.discovery.clone(),
            shutdown_signal: self.shutdown_signal.clone(),
        }
    }
}

/// Default lifecycle hooks implementation
pub struct DefaultLifecycleHooks;

#[async_trait]
impl ServiceLifecycleHooks for DefaultLifecycleHooks {
    async fn pre_start(&self, config: &ServiceConfiguration) -> Result<(), WorkflowError> {
        tracing::info!("Pre-start hook for service: {}", config.name);
        Ok(())
    }
    
    async fn post_start(&self, config: &ServiceConfiguration) -> Result<(), WorkflowError> {
        tracing::info!("Post-start hook for service: {}", config.name);
        Ok(())
    }
    
    async fn pre_stop(&self, config: &ServiceConfiguration) -> Result<(), WorkflowError> {
        tracing::info!("Pre-stop hook for service: {}", config.name);
        Ok(())
    }
    
    async fn post_stop(&self, config: &ServiceConfiguration) -> Result<(), WorkflowError> {
        tracing::info!("Post-stop hook for service: {}", config.name);
        Ok(())
    }
    
    async fn on_failure(&self, config: &ServiceConfiguration, error: &WorkflowError) -> Result<(), WorkflowError> {
        tracing::error!("Service {} failed: {}", config.name, error);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use workflow_engine_core::registry::agent_registry::MockAgentRegistry;
    use super::super::discovery::RegistryDiscovery;
    
    #[tokio::test]
    async fn test_lifecycle_manager() {
        let mock_registry = MockAgentRegistry::new();
        let service_registry = Arc::new(ServiceRegistry::new(Arc::new(mock_registry)));
        let config_manager = Arc::new(ConfigurationManager::new("test".to_string()));
        let discovery = Arc::new(RegistryDiscovery::new(service_registry.clone()));
        
        let manager = ServiceLifecycleManager::new(
            service_registry,
            config_manager,
            discovery,
        );
        
        let config = ServiceConfiguration {
            name: "test-service".to_string(),
            version: "1.0.0".to_string(),
            endpoint: "http://localhost:8080".to_string(),
            capabilities: vec!["test".to_string()],
            dependencies: vec![],
            health_check: Default::default(),
            retry_config: Default::default(),
            circuit_breaker: Default::default(),
            custom_config: HashMap::new(),
            environment: "test".to_string(),
        };
        
        manager.register_service(config, Some(Arc::new(DefaultLifecycleHooks))).await.unwrap();
        
        let state = manager.get_service_state("test-service").await.unwrap();
        assert_eq!(state, ServiceState::Uninitialized);
    }
    
    #[tokio::test]
    async fn test_dependency_ordering() {
        let mock_registry = MockAgentRegistry::new();
        let service_registry = Arc::new(ServiceRegistry::new(Arc::new(mock_registry)));
        let config_manager = Arc::new(ConfigurationManager::new("test".to_string()));
        let discovery = Arc::new(RegistryDiscovery::new(service_registry.clone()));
        
        let manager = ServiceLifecycleManager::new(
            service_registry,
            config_manager,
            discovery,
        );
        
        // Register services with dependencies
        let service1 = ServiceConfiguration {
            name: "service1".to_string(),
            version: "1.0.0".to_string(),
            endpoint: "http://localhost:8081".to_string(),
            capabilities: vec!["cap1".to_string()],
            dependencies: vec![],
            health_check: Default::default(),
            retry_config: Default::default(),
            circuit_breaker: Default::default(),
            custom_config: HashMap::new(),
            environment: "test".to_string(),
        };
        
        let service2 = ServiceConfiguration {
            name: "service2".to_string(),
            version: "1.0.0".to_string(),
            endpoint: "http://localhost:8082".to_string(),
            capabilities: vec!["cap2".to_string()],
            dependencies: vec![
                super::super::config::ServiceDependency {
                    service_name: "service1".to_string(),
                    version_requirement: "^1.0.0".to_string(),
                    optional: false,
                    required_capabilities: vec!["cap1".to_string()],
                }
            ],
            health_check: Default::default(),
            retry_config: Default::default(),
            circuit_breaker: Default::default(),
            custom_config: HashMap::new(),
            environment: "test".to_string(),
        };
        
        manager.register_service(service1, None).await.unwrap();
        manager.register_service(service2, None).await.unwrap();
        
        let order = manager.calculate_start_order().await.unwrap();
        assert_eq!(order, vec!["service1".to_string(), "service2".to_string()]);
    }
}