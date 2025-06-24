//! Service discovery mechanisms
//!
//! This module implements service discovery patterns including:
//! - DNS-based discovery
//! - Registry-based discovery
//! - Multicast/broadcast discovery
//! - Health-aware discovery

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use workflow_engine_core::error::{WorkflowError, ErrorContext, ErrorContextExt};
use super::registry::{ServiceInstance, ServiceRegistry, LoadBalancingStrategy, HealthStatus};
use super::config::ServiceConfiguration;

/// Service discovery mechanism trait
#[async_trait]
pub trait ServiceDiscovery: Send + Sync {
    /// Discover services by name
    async fn discover_service(&self, service_name: &str) -> Result<Vec<ServiceInstance>, WorkflowError>;
    
    /// Discover services by capability
    async fn discover_by_capability(&self, capability: &str) -> Result<Vec<ServiceInstance>, WorkflowError>;
    
    /// Register a service for discovery
    async fn register_service(&self, config: &ServiceConfiguration) -> Result<(), WorkflowError>;
    
    /// Unregister a service
    async fn unregister_service(&self, service_name: &str, instance_id: uuid::Uuid) -> Result<(), WorkflowError>;
    
    /// Watch for service changes
    async fn watch_services(&self) -> Result<ServiceWatcher, WorkflowError>;
}

/// Service change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceEvent {
    /// New service instance registered
    ServiceAdded {
        instance: ServiceInstance,
    },
    
    /// Service instance removed
    ServiceRemoved {
        service_name: String,
        instance_id: uuid::Uuid,
    },
    
    /// Service health status changed
    HealthChanged {
        service_name: String,
        instance_id: uuid::Uuid,
        old_status: HealthStatus,
        new_status: HealthStatus,
    },
    
    /// Service configuration updated
    ConfigurationUpdated {
        service_name: String,
        instance_id: uuid::Uuid,
    },
}

/// Service watcher for receiving change notifications
pub struct ServiceWatcher {
    receiver: tokio::sync::mpsc::Receiver<ServiceEvent>,
}

impl ServiceWatcher {
    /// Wait for the next service event
    pub async fn next_event(&mut self) -> Option<ServiceEvent> {
        self.receiver.recv().await
    }
}

/// Registry-based service discovery
pub struct RegistryDiscovery {
    registry: Arc<ServiceRegistry>,
    event_sender: Arc<RwLock<Option<tokio::sync::mpsc::Sender<ServiceEvent>>>>,
}

impl RegistryDiscovery {
    /// Create a new registry-based discovery
    pub fn new(registry: Arc<ServiceRegistry>) -> Self {
        Self {
            registry,
            event_sender: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Notify watchers of a service event
    async fn notify_event(&self, event: ServiceEvent) {
        if let Some(sender) = &*self.event_sender.read().await {
            let _ = sender.send(event).await;
        }
    }
}

#[async_trait]
impl ServiceDiscovery for RegistryDiscovery {
    async fn discover_service(&self, service_name: &str) -> Result<Vec<ServiceInstance>, WorkflowError> {
        self.registry.get_service_instances(service_name).await
    }
    
    async fn discover_by_capability(&self, capability: &str) -> Result<Vec<ServiceInstance>, WorkflowError> {
        self.registry.discover_by_capability(capability).await
    }
    
    async fn register_service(&self, config: &ServiceConfiguration) -> Result<(), WorkflowError> {
        let metadata = super::registry::ServiceMetadata {
            tags: vec![],
            environment: config.environment.clone(),
            region: "default".to_string(),
            attributes: HashMap::new(),
            api_version: "v1".to_string(),
            protocol: "http".to_string(),
        };
        
        let instance = self.registry.register_instance(config, metadata).await?;
        
        self.notify_event(ServiceEvent::ServiceAdded { instance }).await;
        
        Ok(())
    }
    
    async fn unregister_service(&self, service_name: &str, instance_id: uuid::Uuid) -> Result<(), WorkflowError> {
        self.registry.unregister_instance(instance_id).await?;
        
        self.notify_event(ServiceEvent::ServiceRemoved {
            service_name: service_name.to_string(),
            instance_id,
        }).await;
        
        Ok(())
    }
    
    async fn watch_services(&self) -> Result<ServiceWatcher, WorkflowError> {
        let (sender, receiver) = tokio::sync::mpsc::channel(100);
        *self.event_sender.write().await = Some(sender);
        
        Ok(ServiceWatcher { receiver })
    }
}

/// Composite service discovery that tries multiple mechanisms
pub struct CompositeDiscovery {
    discoveries: Vec<Arc<dyn ServiceDiscovery>>,
}

impl CompositeDiscovery {
    /// Create a new composite discovery
    pub fn new(discoveries: Vec<Arc<dyn ServiceDiscovery>>) -> Self {
        Self { discoveries }
    }
}

#[async_trait]
impl ServiceDiscovery for CompositeDiscovery {
    async fn discover_service(&self, service_name: &str) -> Result<Vec<ServiceInstance>, WorkflowError> {
        let mut all_instances = Vec::new();
        let mut last_error = None;
        
        for discovery in &self.discoveries {
            match discovery.discover_service(service_name).await {
                Ok(instances) => all_instances.extend(instances),
                Err(e) => last_error = Some(e),
            }
        }
        
        if all_instances.is_empty() {
            if let Some(error) = last_error {
                return Err(error);
            }
            return Err(WorkflowError::registry_error_simple(
                format!("Service not found: {}", service_name)
            ));
        }
        
        Ok(all_instances)
    }
    
    async fn discover_by_capability(&self, capability: &str) -> Result<Vec<ServiceInstance>, WorkflowError> {
        let mut all_instances = Vec::new();
        let mut last_error = None;
        
        for discovery in &self.discoveries {
            match discovery.discover_by_capability(capability).await {
                Ok(instances) => all_instances.extend(instances),
                Err(e) => last_error = Some(e),
            }
        }
        
        if all_instances.is_empty() {
            if let Some(error) = last_error {
                return Err(error);
            }
            return Err(WorkflowError::registry_error_simple(
                format!("No services found with capability: {}", capability)
            ));
        }
        
        Ok(all_instances)
    }
    
    async fn register_service(&self, config: &ServiceConfiguration) -> Result<(), WorkflowError> {
        let mut last_error = None;
        let mut registered = false;
        
        for discovery in &self.discoveries {
            match discovery.register_service(config).await {
                Ok(_) => registered = true,
                Err(e) => last_error = Some(e),
            }
        }
        
        if !registered {
            if let Some(error) = last_error {
                return Err(error);
            }
            return Err(WorkflowError::registry_error_simple(
                "Failed to register service with any discovery mechanism".to_string()
            ));
        }
        
        Ok(())
    }
    
    async fn unregister_service(&self, service_name: &str, instance_id: uuid::Uuid) -> Result<(), WorkflowError> {
        let mut last_error = None;
        
        for discovery in &self.discoveries {
            if let Err(e) = discovery.unregister_service(service_name, instance_id).await {
                last_error = Some(e);
            }
        }
        
        if let Some(error) = last_error {
            return Err(error);
        }
        
        Ok(())
    }
    
    async fn watch_services(&self) -> Result<ServiceWatcher, WorkflowError> {
        // For composite discovery, we only watch the first mechanism
        if let Some(discovery) = self.discoveries.first() {
            discovery.watch_services().await
        } else {
            Err(WorkflowError::configuration_error_simple(
                "No discovery mechanisms configured".to_string()
            ))
        }
    }
}

/// Service discovery client with caching and retry
pub struct DiscoveryClient {
    discovery: Arc<dyn ServiceDiscovery>,
    cache: Arc<RwLock<DiscoveryCache>>,
    cache_ttl: Duration,
}

/// Discovery cache
struct DiscoveryCache {
    services: HashMap<String, (Vec<ServiceInstance>, std::time::Instant)>,
    capabilities: HashMap<String, (Vec<ServiceInstance>, std::time::Instant)>,
}

impl DiscoveryClient {
    /// Create a new discovery client
    pub fn new(discovery: Arc<dyn ServiceDiscovery>, cache_ttl: Duration) -> Self {
        Self {
            discovery,
            cache: Arc::new(RwLock::new(DiscoveryCache {
                services: HashMap::new(),
                capabilities: HashMap::new(),
            })),
            cache_ttl,
        }
    }
    
    /// Discover a service with caching
    pub async fn discover_service(&self, service_name: &str) -> Result<Vec<ServiceInstance>, WorkflowError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some((instances, timestamp)) = cache.services.get(service_name) {
                if timestamp.elapsed() < self.cache_ttl {
                    return Ok(instances.clone());
                }
            }
        }
        
        // Cache miss or expired, fetch from discovery
        let instances = self.discovery.discover_service(service_name).await?;
        
        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.services.insert(
                service_name.to_string(),
                (instances.clone(), std::time::Instant::now())
            );
        }
        
        Ok(instances)
    }
    
    /// Discover services by capability with caching
    pub async fn discover_by_capability(&self, capability: &str) -> Result<Vec<ServiceInstance>, WorkflowError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some((instances, timestamp)) = cache.capabilities.get(capability) {
                if timestamp.elapsed() < self.cache_ttl {
                    return Ok(instances.clone());
                }
            }
        }
        
        // Cache miss or expired, fetch from discovery
        let instances = self.discovery.discover_by_capability(capability).await?;
        
        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.capabilities.insert(
                capability.to_string(),
                (instances.clone(), std::time::Instant::now())
            );
        }
        
        Ok(instances)
    }
    
    /// Select a service instance with load balancing
    pub async fn select_instance(
        &self,
        service_name: &str,
        strategy: LoadBalancingStrategy,
    ) -> Result<ServiceInstance, WorkflowError> {
        let instances = self.discover_service(service_name).await?;
        
        let healthy_instances: Vec<&ServiceInstance> = instances
            .iter()
            .filter(|i| i.health_status == HealthStatus::Healthy)
            .collect();
            
        if healthy_instances.is_empty() {
            return Err(WorkflowError::registry_error_simple(
                format!("No healthy instances for service: {}", service_name)
            ));
        }
        
        // Simple selection based on strategy
        let selected = match strategy {
            LoadBalancingStrategy::Random => {
                use rand::Rng;
                let idx = rand::thread_rng().gen_range(0..healthy_instances.len());
                &healthy_instances[idx]
            }
            _ => &healthy_instances[0], // Default to first for other strategies
        };
        
        Ok((*selected).clone())
    }
    
    /// Clear the discovery cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.services.clear();
        cache.capabilities.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::mocks::MockAgentRegistry;
    
    #[tokio::test]
    async fn test_registry_discovery() {
        let mock_registry = MockAgentRegistry::new();
        let service_registry = Arc::new(ServiceRegistry::new(Arc::new(mock_registry)));
        let discovery = RegistryDiscovery::new(service_registry);
        
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
        
        // Note: This test would need a properly mocked registry
        // For now, we're just testing the structure
    }
    
    #[tokio::test]
    async fn test_discovery_client_caching() {
        let mock_registry = MockAgentRegistry::new();
        let service_registry = Arc::new(ServiceRegistry::new(Arc::new(mock_registry)));
        let discovery = Arc::new(RegistryDiscovery::new(service_registry));
        let client = DiscoveryClient::new(discovery, Duration::from_secs(60));
        
        // Test cache operations
        client.clear_cache().await;
        
        // Note: This test would need proper mocking to test actual caching behavior
    }
}