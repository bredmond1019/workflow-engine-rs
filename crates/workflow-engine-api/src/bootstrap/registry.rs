//! Enhanced service registry with metadata and lifecycle management
//!
//! This module provides comprehensive service registry functionality including:
//! - Service registration and discovery
//! - Metadata management and versioning
//! - Load balancing support
//! - Service lifecycle tracking

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use workflow_engine_core::error::{WorkflowError, ErrorContext, ErrorContextExt};
use workflow_engine_core::registry::agent_registry::{AgentRegistry, AgentRegistration, AgentRegistryError};
use crate::db::agent::Agent;
use super::config::ServiceConfiguration;

/// Service instance representing a running service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInstance {
    /// Unique instance ID
    pub id: Uuid,
    
    /// Service name
    pub name: String,
    
    /// Service version
    pub version: String,
    
    /// Instance endpoint
    pub endpoint: String,
    
    /// Service capabilities
    pub capabilities: Vec<String>,
    
    /// Service metadata
    pub metadata: ServiceMetadata,
    
    /// Health status
    pub health_status: HealthStatus,
    
    /// Load metrics
    pub load_metrics: LoadMetrics,
    
    /// Registration time
    pub registered_at: DateTime<Utc>,
    
    /// Last seen time
    pub last_seen: DateTime<Utc>,
}

/// Service metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetadata {
    /// Service tags for grouping
    pub tags: Vec<String>,
    
    /// Service environment
    pub environment: String,
    
    /// Service region/zone
    pub region: String,
    
    /// Custom attributes
    pub attributes: HashMap<String, serde_json::Value>,
    
    /// API version
    pub api_version: String,
    
    /// Protocol supported (http, grpc, etc.)
    pub protocol: String,
}

/// Health status of a service
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    /// Service is healthy and accepting requests
    Healthy,
    
    /// Service is degraded but still operational
    Degraded,
    
    /// Service is unhealthy and not accepting requests
    Unhealthy,
    
    /// Service health is unknown
    Unknown,
}

/// Load metrics for a service instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadMetrics {
    /// Current number of active connections
    pub active_connections: u32,
    
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    
    /// CPU usage percentage (0-100)
    pub cpu_usage: f32,
    
    /// Memory usage percentage (0-100)
    pub memory_usage: f32,
    
    /// Request rate per second
    pub request_rate: f64,
    
    /// Error rate percentage (0-100)
    pub error_rate: f32,
}

impl Default for LoadMetrics {
    fn default() -> Self {
        Self {
            active_connections: 0,
            avg_response_time_ms: 0.0,
            cpu_usage: 0.0,
            memory_usage: 0.0,
            request_rate: 0.0,
            error_rate: 0.0,
        }
    }
}

/// Load balancing strategy
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoadBalancingStrategy {
    /// Round-robin selection
    RoundRobin,
    
    /// Least connections
    LeastConnections,
    
    /// Weighted round-robin based on capacity
    WeightedRoundRobin,
    
    /// Random selection
    Random,
    
    /// Based on response time
    ResponseTime,
}

/// Enhanced service registry
pub struct ServiceRegistry {
    /// Service instances indexed by service name
    instances: Arc<RwLock<HashMap<String, Vec<ServiceInstance>>>>,
    
    /// Service instances indexed by ID
    instances_by_id: Arc<RwLock<HashMap<Uuid, ServiceInstance>>>,
    
    /// Round-robin counters for load balancing
    round_robin_counters: Arc<RwLock<HashMap<String, usize>>>,
    
    /// Underlying agent registry for persistence
    agent_registry: Arc<dyn AgentRegistry>,
}

impl ServiceRegistry {
    /// Create a new service registry
    pub fn new(agent_registry: Arc<dyn AgentRegistry>) -> Self {
        Self {
            instances: Arc::new(RwLock::new(HashMap::new())),
            instances_by_id: Arc::new(RwLock::new(HashMap::new())),
            round_robin_counters: Arc::new(RwLock::new(HashMap::new())),
            agent_registry,
        }
    }
    
    /// Register a service instance
    pub async fn register_instance(
        &self,
        config: &ServiceConfiguration,
        metadata: ServiceMetadata,
    ) -> Result<ServiceInstance, WorkflowError> {
        // Create agent registration for persistence
        let agent_reg = AgentRegistration {
            name: config.name.clone(),
            endpoint: config.endpoint.clone(),
            capabilities: config.capabilities.clone(),
            metadata: serde_json::to_value(&metadata)
                .map_err(|e| WorkflowError::serialization_error_simple(
                    format!("Failed to serialize service metadata: {}", e)
                ))?,
        };
        
        // Register with underlying agent registry
        let agent = self.agent_registry.register(agent_reg).await
            .map_err(|e| WorkflowError::registry_error_simple(
                format!("Failed to register service: {}", e)
            ))?;
            
        // Create service instance
        let instance = ServiceInstance {
            id: agent.id,
            name: config.name.clone(),
            version: config.version.clone(),
            endpoint: config.endpoint.clone(),
            capabilities: config.capabilities.clone(),
            metadata,
            health_status: HealthStatus::Unknown,
            load_metrics: LoadMetrics::default(),
            registered_at: agent.created_at,
            last_seen: agent.last_seen,
        };
        
        // Store in local cache
        let mut instances = self.instances.write().await;
        let mut instances_by_id = self.instances_by_id.write().await;
        
        instances.entry(config.name.clone())
            .or_insert_with(Vec::new)
            .push(instance.clone());
            
        instances_by_id.insert(instance.id, instance.clone());
        
        Ok(instance)
    }
    
    /// Unregister a service instance
    pub async fn unregister_instance(&self, instance_id: Uuid) -> Result<(), WorkflowError> {
        // Remove from underlying registry
        self.agent_registry.unregister(&instance_id).await
            .map_err(|e| WorkflowError::registry_error_simple(
                format!("Failed to unregister service: {}", e)
            ))?;
            
        // Remove from local cache
        let mut instances = self.instances.write().await;
        let mut instances_by_id = self.instances_by_id.write().await;
        
        if let Some(instance) = instances_by_id.remove(&instance_id) {
            if let Some(service_instances) = instances.get_mut(&instance.name) {
                service_instances.retain(|i| i.id != instance_id);
                if service_instances.is_empty() {
                    instances.remove(&instance.name);
                }
            }
        }
        
        Ok(())
    }
    
    /// Update service health status
    pub async fn update_health_status(
        &self,
        instance_id: Uuid,
        status: HealthStatus,
    ) -> Result<(), WorkflowError> {
        let mut instances_by_id = self.instances_by_id.write().await;
        let mut instances = self.instances.write().await;
        
        if let Some(instance) = instances_by_id.get_mut(&instance_id) {
            instance.health_status = status.clone();
            instance.last_seen = Utc::now();
            
            // Update in the name-indexed map
            if let Some(service_instances) = instances.get_mut(&instance.name) {
                if let Some(idx) = service_instances.iter().position(|i| i.id == instance_id) {
                    service_instances[idx].health_status = status;
                    service_instances[idx].last_seen = Utc::now();
                }
            }
            
            // Send heartbeat to underlying registry
            self.agent_registry.heartbeat(&instance_id).await
                .map_err(|e| WorkflowError::registry_error_simple(
                    format!("Failed to send heartbeat: {}", e)
                ))?;
                
            Ok(())
        } else {
            Err(WorkflowError::registry_error_simple(
                format!("Service instance not found: {}", instance_id)
            ))
        }
    }
    
    /// Update service load metrics
    pub async fn update_load_metrics(
        &self,
        instance_id: Uuid,
        metrics: LoadMetrics,
    ) -> Result<(), WorkflowError> {
        let mut instances_by_id = self.instances_by_id.write().await;
        let mut instances = self.instances.write().await;
        
        if let Some(instance) = instances_by_id.get_mut(&instance_id) {
            instance.load_metrics = metrics.clone();
            
            // Update in the name-indexed map
            if let Some(service_instances) = instances.get_mut(&instance.name) {
                if let Some(idx) = service_instances.iter().position(|i| i.id == instance_id) {
                    service_instances[idx].load_metrics = metrics;
                }
            }
            
            Ok(())
        } else {
            Err(WorkflowError::registry_error_simple(
                format!("Service instance not found: {}", instance_id)
            ))
        }
    }
    
    /// Discover services by capability
    pub async fn discover_by_capability(&self, capability: &str) -> Result<Vec<ServiceInstance>, WorkflowError> {
        let instances = self.instances.read().await;
        let mut result = Vec::new();
        
        for service_instances in instances.values() {
            for instance in service_instances {
                if instance.capabilities.contains(&capability.to_string()) 
                    && instance.health_status == HealthStatus::Healthy {
                    result.push(instance.clone());
                }
            }
        }
        
        Ok(result)
    }
    
    /// Get all instances of a service
    pub async fn get_service_instances(&self, service_name: &str) -> Result<Vec<ServiceInstance>, WorkflowError> {
        let instances = self.instances.read().await;
        
        Ok(instances
            .get(service_name)
            .cloned()
            .unwrap_or_default())
    }
    
    /// Get a specific service instance
    pub async fn get_instance(&self, instance_id: Uuid) -> Result<ServiceInstance, WorkflowError> {
        let instances_by_id = self.instances_by_id.read().await;
        
        instances_by_id
            .get(&instance_id)
            .cloned()
            .ok_or_else(|| WorkflowError::registry_error_simple(
                format!("Service instance not found: {}", instance_id)
            ))
    }
    
    /// Select a service instance using load balancing
    pub async fn select_instance(
        &self,
        service_name: &str,
        strategy: LoadBalancingStrategy,
    ) -> Result<ServiceInstance, WorkflowError> {
        let instances = self.instances.read().await;
        
        let service_instances = instances
            .get(service_name)
            .ok_or_else(|| WorkflowError::registry_error_simple(
                format!("Service not found: {}", service_name)
            ))?;
            
        let healthy_instances: Vec<&ServiceInstance> = service_instances
            .iter()
            .filter(|i| i.health_status == HealthStatus::Healthy)
            .collect();
            
        if healthy_instances.is_empty() {
            return Err(WorkflowError::registry_error_simple(
                format!("No healthy instances for service: {}", service_name)
            ));
        }
        
        let selected = match strategy {
            LoadBalancingStrategy::RoundRobin => {
                let mut counters = self.round_robin_counters.write().await;
                let counter = counters.entry(service_name.to_string()).or_insert(0);
                let instance = &healthy_instances[*counter % healthy_instances.len()];
                *counter = (*counter + 1) % healthy_instances.len();
                instance
            }
            
            LoadBalancingStrategy::LeastConnections => {
                healthy_instances
                    .iter()
                    .min_by_key(|i| i.load_metrics.active_connections)
                    .unwrap()
            }
            
            LoadBalancingStrategy::Random => {
                use rand::Rng;
                let idx = rand::thread_rng().gen_range(0..healthy_instances.len());
                &healthy_instances[idx]
            }
            
            LoadBalancingStrategy::ResponseTime => {
                healthy_instances
                    .iter()
                    .min_by(|a, b| {
                        a.load_metrics.avg_response_time_ms
                            .partial_cmp(&b.load_metrics.avg_response_time_ms)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .unwrap()
            }
            
            LoadBalancingStrategy::WeightedRoundRobin => {
                // Simple implementation based on available capacity
                healthy_instances
                    .iter()
                    .min_by(|a, b| {
                        let a_score = a.load_metrics.cpu_usage + a.load_metrics.memory_usage;
                        let b_score = b.load_metrics.cpu_usage + b.load_metrics.memory_usage;
                        a_score.partial_cmp(&b_score).unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .unwrap()
            }
        };
        
        Ok((*selected).clone())
    }
    
    /// Get all registered services
    pub async fn list_services(&self) -> Vec<String> {
        let instances = self.instances.read().await;
        instances.keys().cloned().collect()
    }
    
    /// Get service statistics
    pub async fn get_service_stats(&self, service_name: &str) -> Result<ServiceStats, WorkflowError> {
        let instances = self.instances.read().await;
        
        let service_instances = instances
            .get(service_name)
            .ok_or_else(|| WorkflowError::registry_error_simple(
                format!("Service not found: {}", service_name)
            ))?;
            
        let healthy_count = service_instances
            .iter()
            .filter(|i| i.health_status == HealthStatus::Healthy)
            .count();
            
        let total_connections: u32 = service_instances
            .iter()
            .map(|i| i.load_metrics.active_connections)
            .sum();
            
        let avg_response_time = if !service_instances.is_empty() {
            service_instances
                .iter()
                .map(|i| i.load_metrics.avg_response_time_ms)
                .sum::<f64>() / service_instances.len() as f64
        } else {
            0.0
        };
        
        Ok(ServiceStats {
            service_name: service_name.to_string(),
            total_instances: service_instances.len(),
            healthy_instances: healthy_count,
            total_connections,
            avg_response_time_ms: avg_response_time,
        })
    }
}

/// Service statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStats {
    pub service_name: String,
    pub total_instances: usize,
    pub healthy_instances: usize,
    pub total_connections: u32,
    pub avg_response_time_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::mocks::MockAgentRegistry;
    use workflow_engine_core::registry::Agent;
    
    #[tokio::test]
    async fn test_service_registration() {
        let mut mock_registry = MockAgentRegistry::new();
        mock_registry.expect_register()
            .returning(|_| Ok(Agent {
                id: Uuid::new_v4(),
                name: "test-service".to_string(),
                endpoint: "http://localhost:8080".to_string(),
                capabilities: vec!["test".to_string()],
                status: "active".to_string(),
                last_seen: Utc::now(),
                metadata: serde_json::json!({}),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }));
            
        let registry = ServiceRegistry::new(Arc::new(mock_registry));
        
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
        
        let metadata = ServiceMetadata {
            tags: vec!["test".to_string()],
            environment: "test".to_string(),
            region: "us-east-1".to_string(),
            attributes: HashMap::new(),
            api_version: "v1".to_string(),
            protocol: "http".to_string(),
        };
        
        let instance = registry.register_instance(&config, metadata).await.unwrap();
        assert_eq!(instance.name, "test-service");
        assert_eq!(instance.version, "1.0.0");
    }
    
    #[tokio::test]
    async fn test_load_balancing() {
        let mut mock_registry = MockAgentRegistry::new();
        mock_registry.expect_register()
            .returning(|_| Ok(Agent {
                id: Uuid::new_v4(),
                name: "test-service".to_string(),
                endpoint: "http://localhost:8080".to_string(),
                capabilities: vec!["test".to_string()],
                status: "active".to_string(),
                last_seen: Utc::now(),
                metadata: serde_json::json!({}),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }));
        
        // Add expectation for heartbeat calls (called by update_health_status)
        mock_registry.expect_heartbeat()
            .returning(|_| Ok(()));
            
        let registry = ServiceRegistry::new(Arc::new(mock_registry));
        
        // Register multiple instances
        for i in 0..3 {
            let config = ServiceConfiguration {
                name: "test-service".to_string(),
                version: "1.0.0".to_string(),
                endpoint: format!("http://localhost:808{}", i),
                capabilities: vec!["test".to_string()],
                dependencies: vec![],
                health_check: Default::default(),
                retry_config: Default::default(),
                circuit_breaker: Default::default(),
                custom_config: HashMap::new(),
                environment: "test".to_string(),
            };
            
            let metadata = ServiceMetadata {
                tags: vec!["test".to_string()],
                environment: "test".to_string(),
                region: "us-east-1".to_string(),
                attributes: HashMap::new(),
                api_version: "v1".to_string(),
                protocol: "http".to_string(),
            };
            
            let instance = registry.register_instance(&config, metadata).await.unwrap();
            registry.update_health_status(instance.id, HealthStatus::Healthy).await.unwrap();
        }
        
        // Test round-robin selection
        let selected1 = registry.select_instance("test-service", LoadBalancingStrategy::RoundRobin).await.unwrap();
        let selected2 = registry.select_instance("test-service", LoadBalancingStrategy::RoundRobin).await.unwrap();
        assert_ne!(selected1.endpoint, selected2.endpoint);
    }
}