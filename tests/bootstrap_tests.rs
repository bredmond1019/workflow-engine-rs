//! Comprehensive tests for service bootstrap functionality

use backend::bootstrap::{
    ServiceBootstrapManager, ServiceBootstrapManagerBuilder,
    ServiceConfiguration, ServiceDependency, ServiceState,
    LoadBalancingStrategy, ServiceLifecycleHooks,
};
use backend::core::registry::agent_registry::{AgentRegistry, AgentRegistration, AgentRegistryError};
use backend::db::agent::Agent;
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;

/// Mock agent registry for testing
#[derive(Clone)]
struct MockAgentRegistry {
    agents: Arc<Mutex<HashMap<Uuid, Agent>>>,
}

impl MockAgentRegistry {
    fn new() -> Self {
        Self {
            agents: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl AgentRegistry for MockAgentRegistry {
    async fn register(&self, registration: AgentRegistration) -> Result<Agent, AgentRegistryError> {
        let agent = Agent {
            id: Uuid::new_v4(),
            name: registration.name,
            endpoint: registration.endpoint,
            capabilities: registration.capabilities,
            status: "active".to_string(),
            last_seen: Utc::now(),
            metadata: registration.metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        self.agents.lock().unwrap().insert(agent.id, agent.clone());
        Ok(agent)
    }
    
    async fn discover(&self, capability: &str) -> Result<Vec<Agent>, AgentRegistryError> {
        Ok(self.agents.lock().unwrap()
            .values()
            .filter(|a| a.capabilities.contains(&capability.to_string()))
            .cloned()
            .collect())
    }
    
    async fn heartbeat(&self, agent_id: &Uuid) -> Result<(), AgentRegistryError> {
        if let Some(agent) = self.agents.lock().unwrap().get_mut(agent_id) {
            agent.last_seen = Utc::now();
            Ok(())
        } else {
            Err(AgentRegistryError::AgentNotFound { id: *agent_id })
        }
    }
    
    async fn list_active(&self) -> Result<Vec<Agent>, AgentRegistryError> {
        Ok(self.agents.lock().unwrap()
            .values()
            .filter(|a| a.status == "active")
            .cloned()
            .collect())
    }
    
    async fn get_by_id(&self, agent_id: &Uuid) -> Result<Agent, AgentRegistryError> {
        self.agents.lock().unwrap()
            .get(agent_id)
            .cloned()
            .ok_or(AgentRegistryError::AgentNotFound { id: *agent_id })
    }
    
    async fn get_by_name(&self, name: &str) -> Result<Agent, AgentRegistryError> {
        self.agents.lock().unwrap()
            .values()
            .find(|a| a.name == name)
            .cloned()
            .ok_or(AgentRegistryError::AgentNotFound { id: Uuid::nil() })
    }
    
    async fn mark_inactive_stale(&self, threshold_minutes: i64) -> Result<usize, AgentRegistryError> {
        let threshold = Utc::now() - chrono::Duration::minutes(threshold_minutes);
        let mut count = 0;
        
        for agent in self.agents.lock().unwrap().values_mut() {
            if agent.last_seen < threshold && agent.status == "active" {
                agent.status = "inactive".to_string();
                count += 1;
            }
        }
        
        Ok(count)
    }
    
    async fn unregister(&self, agent_id: &Uuid) -> Result<(), AgentRegistryError> {
        if self.agents.lock().unwrap().remove(agent_id).is_some() {
            Ok(())
        } else {
            Err(AgentRegistryError::AgentNotFound { id: *agent_id })
        }
    }
}

#[tokio::test]
async fn test_service_bootstrap_manager_creation() {
    let registry = Arc::new(MockAgentRegistry::new());
    let manager = ServiceBootstrapManager::new(registry, "test".to_string());
    
    manager.initialize(None).await.unwrap();
    
    let states = manager.get_all_service_states().await;
    assert!(states.is_empty());
}

#[tokio::test]
async fn test_service_registration_and_startup() {
    let registry = Arc::new(MockAgentRegistry::new());
    let manager = ServiceBootstrapManager::new(registry, "test".to_string());
    
    manager.initialize(None).await.unwrap();
    
    let config = ServiceConfiguration {
        name: "test-service".to_string(),
        version: "1.0.0".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        capabilities: vec!["test".to_string(), "demo".to_string()],
        dependencies: vec![],
        health_check: Default::default(),
        retry_config: Default::default(),
        circuit_breaker: Default::default(),
        custom_config: HashMap::new(),
        environment: "test".to_string(),
    };
    
    // Register service
    manager.register_service(config.clone(), None).await.unwrap();
    
    // Check initial state
    let state = manager.get_service_state("test-service").await.unwrap();
    assert_eq!(state, ServiceState::Uninitialized);
    
    // Start service
    manager.start_service("test-service").await.unwrap();
    
    // Check running state
    let state = manager.get_service_state("test-service").await.unwrap();
    assert_eq!(state, ServiceState::Running);
    
    // Verify service is discoverable
    let instances = manager.get_service("test-service").await.unwrap();
    assert_eq!(instances.len(), 1);
    assert_eq!(instances[0].name, "test-service");
    
    // Test capability discovery
    let test_services = manager.get_services_by_capability("test").await.unwrap();
    assert_eq!(test_services.len(), 1);
    
    let demo_services = manager.get_services_by_capability("demo").await.unwrap();
    assert_eq!(demo_services.len(), 1);
}

#[tokio::test]
async fn test_service_dependencies() {
    let registry = Arc::new(MockAgentRegistry::new());
    let manager = ServiceBootstrapManager::new(registry, "test".to_string());
    
    manager.initialize(None).await.unwrap();
    
    // Register base service
    let base_config = ServiceConfiguration {
        name: "base-service".to_string(),
        version: "1.0.0".to_string(),
        endpoint: "http://localhost:8081".to_string(),
        capabilities: vec!["base".to_string()],
        dependencies: vec![],
        health_check: Default::default(),
        retry_config: Default::default(),
        circuit_breaker: Default::default(),
        custom_config: HashMap::new(),
        environment: "test".to_string(),
    };
    
    manager.register_service(base_config, None).await.unwrap();
    
    // Register dependent service
    let dependent_config = ServiceConfiguration {
        name: "dependent-service".to_string(),
        version: "1.0.0".to_string(),
        endpoint: "http://localhost:8082".to_string(),
        capabilities: vec!["dependent".to_string()],
        dependencies: vec![
            ServiceDependency {
                service_name: "base-service".to_string(),
                version_requirement: "^1.0.0".to_string(),
                optional: false,
                required_capabilities: vec!["base".to_string()],
            }
        ],
        health_check: Default::default(),
        retry_config: Default::default(),
        circuit_breaker: Default::default(),
        custom_config: HashMap::new(),
        environment: "test".to_string(),
    };
    
    manager.register_service(dependent_config, None).await.unwrap();
    
    // Try to start dependent service without base service running
    let result = manager.start_service("dependent-service").await;
    assert!(result.is_err());
    
    // Start base service
    manager.start_service("base-service").await.unwrap();
    
    // Now dependent service should start
    manager.start_service("dependent-service").await.unwrap();
    
    let state = manager.get_service_state("dependent-service").await.unwrap();
    assert_eq!(state, ServiceState::Running);
}

#[tokio::test]
async fn test_service_lifecycle() {
    let registry = Arc::new(MockAgentRegistry::new());
    let manager = ServiceBootstrapManager::new(registry, "test".to_string());
    
    manager.initialize(None).await.unwrap();
    
    let config = ServiceConfiguration {
        name: "lifecycle-test".to_string(),
        version: "1.0.0".to_string(),
        endpoint: "http://localhost:8083".to_string(),
        capabilities: vec!["lifecycle".to_string()],
        dependencies: vec![],
        health_check: Default::default(),
        retry_config: Default::default(),
        circuit_breaker: Default::default(),
        custom_config: HashMap::new(),
        environment: "test".to_string(),
    };
    
    // Track lifecycle events
    let events = Arc::new(Mutex::new(Vec::new()));
    
    struct TestLifecycleHooks {
        events: Arc<Mutex<Vec<String>>>,
    }
    
    #[async_trait]
    impl ServiceLifecycleHooks for TestLifecycleHooks {
        async fn pre_start(&self, config: &ServiceConfiguration) -> Result<(), backend::core::error::WorkflowError> {
            self.events.lock().unwrap().push(format!("pre_start: {}", config.name));
            Ok(())
        }
        
        async fn post_start(&self, config: &ServiceConfiguration) -> Result<(), backend::core::error::WorkflowError> {
            self.events.lock().unwrap().push(format!("post_start: {}", config.name));
            Ok(())
        }
        
        async fn pre_stop(&self, config: &ServiceConfiguration) -> Result<(), backend::core::error::WorkflowError> {
            self.events.lock().unwrap().push(format!("pre_stop: {}", config.name));
            Ok(())
        }
        
        async fn post_stop(&self, config: &ServiceConfiguration) -> Result<(), backend::core::error::WorkflowError> {
            self.events.lock().unwrap().push(format!("post_stop: {}", config.name));
            Ok(())
        }
        
        async fn on_failure(&self, config: &ServiceConfiguration, _error: &backend::core::error::WorkflowError) -> Result<(), backend::core::error::WorkflowError> {
            self.events.lock().unwrap().push(format!("on_failure: {}", config.name));
            Ok(())
        }
    }
    
    let hooks = Arc::new(TestLifecycleHooks { events: events.clone() });
    
    // Register with lifecycle hooks
    manager.register_service(config, Some(hooks)).await.unwrap();
    
    // Start service
    manager.start_service("lifecycle-test").await.unwrap();
    
    // Stop service
    manager.stop_service("lifecycle-test").await.unwrap();
    
    // Check lifecycle events
    let recorded_events = events.lock().unwrap().clone();
    assert_eq!(recorded_events.len(), 4);
    assert_eq!(recorded_events[0], "pre_start: lifecycle-test");
    assert_eq!(recorded_events[1], "post_start: lifecycle-test");
    assert_eq!(recorded_events[2], "pre_stop: lifecycle-test");
    assert_eq!(recorded_events[3], "post_stop: lifecycle-test");
}

#[tokio::test]
async fn test_load_balancing() {
    let registry = Arc::new(MockAgentRegistry::new());
    let manager = ServiceBootstrapManager::new(registry, "test".to_string());
    
    manager.initialize(None).await.unwrap();
    
    // For load balancing test, register multiple instances with unique names
    // but same capabilities
    let mut instance_ids = Vec::new();
    for i in 0..3 {
        let config = ServiceConfiguration {
            name: format!("lb-service-{}", i),
            version: "1.0.0".to_string(),
            endpoint: format!("http://localhost:808{}", i),
            capabilities: vec!["loadbalanced".to_string()],
            dependencies: vec![],
            health_check: Default::default(),
            retry_config: Default::default(),
            circuit_breaker: Default::default(),
            custom_config: HashMap::new(),
            environment: "test".to_string(),
        };
        
        manager.register_service(config, None).await.unwrap();
        manager.start_service(&format!("lb-service-{}", i)).await.unwrap();
        
        // Get the instance ID
        let instances = manager.get_service(&format!("lb-service-{}", i)).await.unwrap();
        instance_ids.push(instances[0].id);
    }
    
    // Update health status for all instances
    for id in &instance_ids {
        manager.update_health_status(*id, backend::bootstrap::registry::HealthStatus::Healthy).await.unwrap();
    }
    
    // Test capability-based discovery
    let loadbalanced_services = manager.get_services_by_capability("loadbalanced").await.unwrap();
    assert_eq!(loadbalanced_services.len(), 3);
    
    // Verify different endpoints
    let endpoints: std::collections::HashSet<_> = loadbalanced_services
        .iter()
        .map(|s| s.endpoint.clone())
        .collect();
    assert_eq!(endpoints.len(), 3);
}

#[tokio::test]
async fn test_graceful_shutdown() {
    let registry = Arc::new(MockAgentRegistry::new());
    let manager = ServiceBootstrapManager::new(registry, "test".to_string());
    
    manager.initialize(None).await.unwrap();
    
    // Register and start multiple services
    for i in 0..3 {
        let config = ServiceConfiguration {
            name: format!("service-{}", i),
            version: "1.0.0".to_string(),
            endpoint: format!("http://localhost:808{}", i),
            capabilities: vec!["shutdown-test".to_string()],
            dependencies: vec![],
            health_check: Default::default(),
            retry_config: Default::default(),
            circuit_breaker: Default::default(),
            custom_config: HashMap::new(),
            environment: "test".to_string(),
        };
        
        manager.register_service(config, None).await.unwrap();
        manager.start_service(&format!("service-{}", i)).await.unwrap();
    }
    
    // Verify all services are running
    let states = manager.get_all_service_states().await;
    assert_eq!(states.len(), 3);
    for (_, state) in states {
        assert_eq!(state, ServiceState::Running);
    }
    
    // Trigger shutdown
    manager.shutdown().await.unwrap();
    
    // Shutdown is synchronous, so services should be stopped immediately
    let states = manager.get_all_service_states().await;
    for (_, state) in states {
        assert_eq!(state, ServiceState::Stopped);
    }
}

#[tokio::test]
async fn test_configuration_management() {
    let registry = Arc::new(MockAgentRegistry::new());
    let manager = ServiceBootstrapManager::new(registry, "test".to_string());
    
    manager.initialize(None).await.unwrap();
    
    let mut custom_config = HashMap::new();
    custom_config.insert("feature_flag".to_string(), serde_json::json!(true));
    custom_config.insert("max_connections".to_string(), serde_json::json!(100));
    
    let config = ServiceConfiguration {
        name: "config-test".to_string(),
        version: "1.0.0".to_string(),
        endpoint: "http://localhost:8090".to_string(),
        capabilities: vec!["configurable".to_string()],
        dependencies: vec![],
        health_check: Default::default(),
        retry_config: Default::default(),
        circuit_breaker: Default::default(),
        custom_config,
        environment: "test".to_string(),
    };
    
    manager.register_service(config, None).await.unwrap();
    
    // Retrieve configuration
    let retrieved_config = manager.get_service_configuration("config-test").await.unwrap();
    assert_eq!(retrieved_config.name, "config-test");
    assert_eq!(retrieved_config.custom_config.get("feature_flag").unwrap(), &serde_json::json!(true));
    assert_eq!(retrieved_config.custom_config.get("max_connections").unwrap(), &serde_json::json!(100));
}

#[tokio::test]
async fn test_builder_pattern() {
    let registry = Arc::new(MockAgentRegistry::new());
    
    let manager = ServiceBootstrapManagerBuilder::new()
        .with_agent_registry(registry)
        .with_environment("production".to_string())
        .with_health_check_interval(Duration::from_secs(60))
        .with_cache_ttl(Duration::from_secs(120))
        .build()
        .await
        .unwrap();
    
    // Manager should be properly initialized
    let states = manager.get_all_service_states().await;
    assert!(states.is_empty());
}

/// Test comprehensive service management workflow
#[tokio::test]
async fn test_comprehensive_workflow() {
    let registry = Arc::new(MockAgentRegistry::new());
    let manager = ServiceBootstrapManager::new(registry, "test".to_string());
    
    manager.initialize(None).await.unwrap();
    
    // 1. Register a chain of dependent services
    let database_config = ServiceConfiguration {
        name: "database".to_string(),
        version: "1.0.0".to_string(),
        endpoint: "postgresql://localhost:5432".to_string(),
        capabilities: vec!["storage".to_string(), "persistence".to_string()],
        dependencies: vec![],
        health_check: Default::default(),
        retry_config: Default::default(),
        circuit_breaker: Default::default(),
        custom_config: HashMap::new(),
        environment: "test".to_string(),
    };
    
    let api_config = ServiceConfiguration {
        name: "api".to_string(),
        version: "1.0.0".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        capabilities: vec!["rest".to_string(), "graphql".to_string()],
        dependencies: vec![
            ServiceDependency {
                service_name: "database".to_string(),
                version_requirement: "^1.0.0".to_string(),
                optional: false,
                required_capabilities: vec!["storage".to_string()],
            }
        ],
        health_check: Default::default(),
        retry_config: Default::default(),
        circuit_breaker: Default::default(),
        custom_config: HashMap::new(),
        environment: "test".to_string(),
    };
    
    let frontend_config = ServiceConfiguration {
        name: "frontend".to_string(),
        version: "1.0.0".to_string(),
        endpoint: "http://localhost:3000".to_string(),
        capabilities: vec!["ui".to_string(), "web".to_string()],
        dependencies: vec![
            ServiceDependency {
                service_name: "api".to_string(),
                version_requirement: "^1.0.0".to_string(),
                optional: false,
                required_capabilities: vec!["rest".to_string()],
            }
        ],
        health_check: Default::default(),
        retry_config: Default::default(),
        circuit_breaker: Default::default(),
        custom_config: HashMap::new(),
        environment: "test".to_string(),
    };
    
    // 2. Register all services
    manager.register_service(database_config, None).await.unwrap();
    manager.register_service(api_config, None).await.unwrap();
    manager.register_service(frontend_config, None).await.unwrap();
    
    // 3. Start all services (should start in dependency order)
    manager.start_all_services().await.unwrap();
    
    // 4. Verify all services are running
    let states = manager.get_all_service_states().await;
    assert_eq!(states.len(), 3);
    for (_, state) in &states {
        assert_eq!(*state, ServiceState::Running);
    }
    
    // 5. Test service discovery
    let storage_services = manager.get_services_by_capability("storage").await.unwrap();
    assert_eq!(storage_services.len(), 1);
    assert_eq!(storage_services[0].name, "database");
    
    let rest_services = manager.get_services_by_capability("rest").await.unwrap();
    assert_eq!(rest_services.len(), 1);
    assert_eq!(rest_services[0].name, "api");
    
    // 6. Test graceful shutdown (should stop in reverse dependency order)
    manager.stop_all_services().await.unwrap();
    
    // 7. Verify all services are stopped
    let states = manager.get_all_service_states().await;
    for (_, state) in states {
        assert_eq!(state, ServiceState::Stopped);
    }
}

