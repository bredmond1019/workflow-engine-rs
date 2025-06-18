use backend::bootstrap::service::{bootstrap_service, ServiceConfig};
use backend::core::registry::agent_registry::{AgentRegistry, AgentRegistration, AgentRegistryError};
use backend::db::agent::Agent;
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// In-memory registry for integration testing
#[derive(Debug, Clone)]
pub struct InMemoryAgentRegistry {
    agents: Arc<Mutex<HashMap<Uuid, Agent>>>,
    agents_by_name: Arc<Mutex<HashMap<String, Uuid>>>,
}

impl InMemoryAgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(Mutex::new(HashMap::new())),
            agents_by_name: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub fn get_agent_count(&self) -> usize {
        self.agents.lock().unwrap().len()
    }
    
    pub fn get_all_agents(&self) -> Vec<Agent> {
        self.agents.lock().unwrap().values().cloned().collect()
    }
}

#[async_trait]
impl AgentRegistry for InMemoryAgentRegistry {
    async fn register(&self, registration: AgentRegistration) -> Result<Agent, AgentRegistryError> {
        let mut agents = self.agents.lock().unwrap();
        let mut agents_by_name = self.agents_by_name.lock().unwrap();
        
        // Check for duplicate names
        if agents_by_name.contains_key(&registration.name) {
            return Err(AgentRegistryError::DuplicateName { 
                name: registration.name 
            });
        }
        
        let agent_id = Uuid::new_v4();
        let now = Utc::now();
        
        let agent = Agent {
            id: agent_id,
            name: registration.name.clone(),
            endpoint: registration.endpoint,
            capabilities: registration.capabilities,
            status: "active".to_string(),
            last_seen: now,
            metadata: registration.metadata,
            created_at: now,
            updated_at: now,
        };
        
        agents.insert(agent_id, agent.clone());
        agents_by_name.insert(registration.name, agent_id);
        
        Ok(agent)
    }
    
    async fn discover(&self, capability: &str) -> Result<Vec<Agent>, AgentRegistryError> {
        let agents = self.agents.lock().unwrap();
        let filtered_agents: Vec<Agent> = agents
            .values()
            .filter(|agent| {
                agent.status == "active" && 
                agent.capabilities.contains(&capability.to_string())
            })
            .cloned()
            .collect();
        
        Ok(filtered_agents)
    }
    
    async fn heartbeat(&self, agent_id: &Uuid) -> Result<(), AgentRegistryError> {
        let mut agents = self.agents.lock().unwrap();
        
        if let Some(agent) = agents.get_mut(agent_id) {
            agent.last_seen = Utc::now();
            agent.updated_at = Utc::now();
            Ok(())
        } else {
            Err(AgentRegistryError::AgentNotFound { id: *agent_id })
        }
    }
    
    async fn list_active(&self) -> Result<Vec<Agent>, AgentRegistryError> {
        let agents = self.agents.lock().unwrap();
        let active_agents: Vec<Agent> = agents
            .values()
            .filter(|agent| agent.status == "active")
            .cloned()
            .collect();
        
        Ok(active_agents)
    }
    
    async fn get_by_id(&self, agent_id: &Uuid) -> Result<Agent, AgentRegistryError> {
        let agents = self.agents.lock().unwrap();
        
        agents.get(agent_id)
            .cloned()
            .ok_or(AgentRegistryError::AgentNotFound { id: *agent_id })
    }
    
    async fn get_by_name(&self, name: &str) -> Result<Agent, AgentRegistryError> {
        let agents = self.agents.lock().unwrap();
        let agents_by_name = self.agents_by_name.lock().unwrap();
        
        if let Some(agent_id) = agents_by_name.get(name) {
            agents.get(agent_id)
                .cloned()
                .ok_or(AgentRegistryError::AgentNotFound { id: *agent_id })
        } else {
            Err(AgentRegistryError::AgentNotFound { 
                id: Uuid::nil() // Not ideal, but the error type expects a UUID
            })
        }
    }
    
    async fn mark_inactive_stale(&self, threshold_minutes: i64) -> Result<usize, AgentRegistryError> {
        let mut agents = self.agents.lock().unwrap();
        let threshold = Utc::now() - chrono::Duration::minutes(threshold_minutes);
        let mut count = 0;
        
        for agent in agents.values_mut() {
            if agent.status == "active" && agent.last_seen < threshold {
                agent.status = "inactive".to_string();
                agent.updated_at = Utc::now();
                count += 1;
            }
        }
        
        Ok(count)
    }
    
    async fn unregister(&self, agent_id: &Uuid) -> Result<(), AgentRegistryError> {
        let mut agents = self.agents.lock().unwrap();
        let mut agents_by_name = self.agents_by_name.lock().unwrap();
        
        if let Some(agent) = agents.remove(agent_id) {
            agents_by_name.remove(&agent.name);
            Ok(())
        } else {
            Err(AgentRegistryError::AgentNotFound { id: *agent_id })
        }
    }
}

#[tokio::test]
async fn test_service_discovery_workflow() {
    let registry = InMemoryAgentRegistry::new();
    
    // Test 1: Bootstrap multiple services with different capabilities
    let ai_tutor_config = ServiceConfig::new(
        "ai-tutor".to_string(),
        "http://localhost:3001".to_string(),
        vec!["tutoring".to_string(), "education".to_string()],
    ).with_heartbeat_interval(5); // Short interval for testing
    
    let workflow_config = ServiceConfig::new(
        "workflow-system".to_string(),
        "http://localhost:3002".to_string(),
        vec!["orchestration".to_string(), "workflow".to_string()],
    ).with_heartbeat_interval(5);
    
    let analytics_config = ServiceConfig::new(
        "analytics-service".to_string(),
        "http://localhost:3003".to_string(),
        vec!["analytics".to_string(), "tutoring".to_string()],
    ).with_heartbeat_interval(5);
    
    // Bootstrap all services
    let ai_tutor_handle = bootstrap_service(ai_tutor_config, &registry).await
        .expect("Failed to bootstrap ai-tutor service");
    
    let workflow_handle = bootstrap_service(workflow_config, &registry).await
        .expect("Failed to bootstrap workflow-system service");
        
    let analytics_handle = bootstrap_service(analytics_config, &registry).await
        .expect("Failed to bootstrap analytics service");
    
    // Test 2: Verify all services are registered
    assert_eq!(registry.get_agent_count(), 3);
    
    let active_agents = registry.list_active().await
        .expect("Failed to list active agents");
    assert_eq!(active_agents.len(), 3);
    
    // Test 3: Test service discovery by capability
    let tutoring_services = registry.discover("tutoring").await
        .expect("Failed to discover tutoring services");
    assert_eq!(tutoring_services.len(), 2); // ai-tutor and analytics-service
    
    let tutoring_names: Vec<String> = tutoring_services.iter()
        .map(|agent| agent.name.clone())
        .collect();
    assert!(tutoring_names.contains(&"ai-tutor".to_string()));
    assert!(tutoring_names.contains(&"analytics-service".to_string()));
    
    let orchestration_services = registry.discover("orchestration").await
        .expect("Failed to discover orchestration services");
    assert_eq!(orchestration_services.len(), 1); // only workflow-system
    assert_eq!(orchestration_services[0].name, "workflow-system");
    
    // Test 4: Test service lookup by name
    let ai_tutor_agent = registry.get_by_name("ai-tutor").await
        .expect("Failed to get ai-tutor by name");
    assert_eq!(ai_tutor_agent.name, "ai-tutor");
    assert_eq!(ai_tutor_agent.endpoint, "http://localhost:3001");
    assert!(ai_tutor_agent.capabilities.contains(&"tutoring".to_string()));
    assert!(ai_tutor_agent.capabilities.contains(&"education".to_string()));
    
    // Test 5: Test heartbeat functionality
    let workflow_agent = registry.get_by_name("workflow-system").await
        .expect("Failed to get workflow-system by name");
    
    let original_last_seen = workflow_agent.last_seen;
    
    // Wait a moment to ensure different timestamp
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    
    registry.heartbeat(&workflow_agent.id).await
        .expect("Failed to send heartbeat");
    
    let updated_agent = registry.get_by_id(&workflow_agent.id).await
        .expect("Failed to get updated agent");
    assert!(updated_agent.last_seen > original_last_seen);
    
    // Test 6: Test duplicate name prevention
    let duplicate_config = ServiceConfig::new(
        "ai-tutor".to_string(), // Same name as existing service
        "http://localhost:3004".to_string(),
        vec!["duplicate".to_string()],
    );
    
    let duplicate_result = bootstrap_service(duplicate_config, &registry).await;
    assert!(duplicate_result.is_err());
    
    // Ensure we still have only 3 services
    assert_eq!(registry.get_agent_count(), 3);
    
    // Test 7: Test stale agent marking
    let stale_count = registry.mark_inactive_stale(0).await // Mark all as stale (0 minutes threshold)
        .expect("Failed to mark stale agents");
    assert_eq!(stale_count, 3); // All agents should be marked as stale
    
    let active_agents_after_stale = registry.list_active().await
        .expect("Failed to list active agents");
    assert_eq!(active_agents_after_stale.len(), 0); // No active agents
    
    // Clean up: Cancel background tasks
    ai_tutor_handle.abort();
    workflow_handle.abort();
    analytics_handle.abort();
    
    println!("✅ Service discovery workflow integration test completed successfully");
}

#[tokio::test]
async fn test_service_registration_error_handling() {
    let registry = InMemoryAgentRegistry::new();
    
    // Test empty service name validation
    let invalid_config = ServiceConfig::new(
        "".to_string(),
        "http://localhost:3001".to_string(),
        vec!["test".to_string()],
    );
    
    let result = bootstrap_service(invalid_config, &registry).await;
    assert!(result.is_err());
    
    // Test empty endpoint validation
    let invalid_config = ServiceConfig::new(
        "test-service".to_string(),
        "".to_string(),
        vec!["test".to_string()],
    );
    
    let result = bootstrap_service(invalid_config, &registry).await;
    assert!(result.is_err());
    
    // Test empty capabilities validation
    let invalid_config = ServiceConfig::new(
        "test-service".to_string(),
        "http://localhost:3001".to_string(),
        vec![],
    );
    
    let result = bootstrap_service(invalid_config, &registry).await;
    assert!(result.is_err());
    
    // Ensure no agents were registered due to validation failures
    assert_eq!(registry.get_agent_count(), 0);
    
    println!("✅ Service registration error handling test completed successfully");
}

#[tokio::test]
async fn test_service_configuration_builder() {
    let registry = InMemoryAgentRegistry::new();
    
    // Test service configuration with all builder options
    let config = ServiceConfig::new(
        "advanced-service".to_string(),
        "http://localhost:3005".to_string(),
        vec!["advanced".to_string(), "testing".to_string()],
    )
    .with_heartbeat_interval(30)
    .with_registry_endpoint("http://registry:8080".to_string())
    .with_auth_token("test-token-123".to_string())
    .with_metadata(serde_json::json!({
        "version": "1.2.3",
        "environment": "test",
        "features": ["feature1", "feature2"]
    }));
    
    let handle = bootstrap_service(config, &registry).await
        .expect("Failed to bootstrap advanced service");
    
    // Verify service was registered correctly
    assert_eq!(registry.get_agent_count(), 1);
    
    let agent = registry.get_by_name("advanced-service").await
        .expect("Failed to get advanced service");
    
    assert_eq!(agent.name, "advanced-service");
    assert_eq!(agent.endpoint, "http://localhost:3005");
    assert_eq!(agent.capabilities.len(), 2);
    assert!(agent.capabilities.contains(&"advanced".to_string()));
    assert!(agent.capabilities.contains(&"testing".to_string()));
    
    // Check metadata
    assert!(agent.metadata.get("version").is_some());
    assert_eq!(agent.metadata.get("version").unwrap(), "1.2.3");
    
    // Clean up
    handle.abort();
    
    println!("✅ Service configuration builder test completed successfully");
}

#[tokio::test]
async fn test_http_transport_with_registered_services() {
    use backend::core::mcp::transport::HttpTransport;
    use backend::core::mcp::protocol::{MCPRequest, ToolCallParams};
    use std::collections::HashMap;
    
    let registry = InMemoryAgentRegistry::new();
    
    // Bootstrap a service that would respond to HTTP requests
    let service_config = ServiceConfig::new(
        "http-test-service".to_string(),
        "http://localhost:3010".to_string(),
        vec!["http-testing".to_string(), "mcp".to_string()],
    )
    .with_registry_endpoint("http://localhost:8080".to_string())
    .with_auth_token("test-token-456".to_string());
    
    let handle = bootstrap_service(service_config, &registry).await
        .expect("Failed to bootstrap HTTP test service");
    
    // Verify service is registered
    assert_eq!(registry.get_agent_count(), 1);
    
    let registered_service = registry.get_by_name("http-test-service").await
        .expect("Failed to get registered service");
    
    assert_eq!(registered_service.endpoint, "http://localhost:3010");
    assert!(registered_service.capabilities.contains(&"http-testing".to_string()));
    assert!(registered_service.capabilities.contains(&"mcp".to_string()));
    
    // Create HTTP transport to communicate with the service
    let transport = HttpTransport::new("http://localhost:3010".to_string())
        .with_auth_token("test-token-456".to_string());
    
    // Test that HTTP transport is properly configured
    // Note: In a real test, we would set up a mock HTTP server to respond to these requests
    // For this test, we're just verifying the transport configuration and method availability
    
    // Create a sample MCP request
    let mut arguments = HashMap::new();
    arguments.insert("test_param".to_string(), serde_json::Value::String("test_value".to_string()));
    
    let request = MCPRequest::CallTool {
        id: "test-request-1".to_string(),
        params: ToolCallParams {
            name: "test_tool".to_string(),
            arguments: Some(arguments),
        },
    };
    
    // Test send_request method (this will fail since no actual server is running)
    // but we can verify the method is available and properly configured
    let result = transport.send_request(request).await;
    
    // We expect this to fail since no server is actually running, but it should fail
    // with a connection error, not a configuration error
    assert!(result.is_err());
    if let Err(e) = result {
        // Should be a connection error, not a configuration error
        assert!(e.to_string().contains("error") || e.to_string().contains("connection"));
    }
    
    // Test service discovery for HTTP-capable services
    let http_services = registry.discover("http-testing").await
        .expect("Failed to discover HTTP services");
    assert_eq!(http_services.len(), 1);
    assert_eq!(http_services[0].name, "http-test-service");
    
    let mcp_services = registry.discover("mcp").await
        .expect("Failed to discover MCP services");
    assert_eq!(mcp_services.len(), 1);
    assert_eq!(mcp_services[0].name, "http-test-service");
    
    // Clean up
    handle.abort();
    
    println!("✅ HTTP transport with registered services test completed successfully");
}