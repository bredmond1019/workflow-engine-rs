/*!
# Service Registration Example

This example demonstrates how to use the AI Workflow System's service bootstrap functionality
to automatically register a service with the agent registry and maintain heartbeat.

## Usage

Run this example with:
```bash
cargo run --example service_registration_example
```

## Overview

This example shows:
1. Creating a service configuration
2. Bootstrapping the service with automatic registration
3. Handling heartbeat management
4. Graceful shutdown

*/

use workflow_engine_api::bootstrap::service::{bootstrap_service, ServiceConfig};
use workflow_engine_core::registry::agent_registry::{AgentRegistry, AgentRegistration};
use backend::db::agent::Agent;
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::signal;
use uuid::Uuid;

/// Example in-memory registry for demonstration
/// In production, you would use PostgresAgentRegistry with a real database
#[derive(Debug, Clone)]
pub struct ExampleRegistry {
    agents: Arc<Mutex<HashMap<Uuid, Agent>>>,
    agents_by_name: Arc<Mutex<HashMap<String, Uuid>>>,
}

impl ExampleRegistry {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(Mutex::new(HashMap::new())),
            agents_by_name: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub fn get_registered_services(&self) -> Vec<Agent> {
        self.agents.lock().unwrap().values().cloned().collect()
    }
}

#[async_trait]
impl AgentRegistry for ExampleRegistry {
    async fn register(&self, registration: AgentRegistration) -> Result<Agent, backend::core::registry::agent_registry::AgentRegistryError> {
        let mut agents = self.agents.lock().unwrap();
        let mut agents_by_name = self.agents_by_name.lock().unwrap();
        
        // Check for duplicate names
        if agents_by_name.contains_key(&registration.name) {
            return Err(backend::core::registry::agent_registry::AgentRegistryError::DuplicateName { 
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
        
        println!("✅ Registered service: {} (ID: {})", agent.name, agent.id);
        
        Ok(agent)
    }
    
    async fn discover(&self, capability: &str) -> Result<Vec<Agent>, backend::core::registry::agent_registry::AgentRegistryError> {
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
    
    async fn heartbeat(&self, agent_id: &Uuid) -> Result<(), backend::core::registry::agent_registry::AgentRegistryError> {
        let mut agents = self.agents.lock().unwrap();
        
        if let Some(agent) = agents.get_mut(agent_id) {
            agent.last_seen = Utc::now();
            agent.updated_at = Utc::now();
            println!("💓 Heartbeat received for: {} (ID: {})", agent.name, agent.id);
            Ok(())
        } else {
            Err(backend::core::registry::agent_registry::AgentRegistryError::AgentNotFound { id: *agent_id })
        }
    }
    
    async fn list_active(&self) -> Result<Vec<Agent>, backend::core::registry::agent_registry::AgentRegistryError> {
        let agents = self.agents.lock().unwrap();
        let active_agents: Vec<Agent> = agents
            .values()
            .filter(|agent| agent.status == "active")
            .cloned()
            .collect();
        
        Ok(active_agents)
    }
    
    async fn get_by_id(&self, agent_id: &Uuid) -> Result<Agent, backend::core::registry::agent_registry::AgentRegistryError> {
        let agents = self.agents.lock().unwrap();
        
        agents.get(agent_id)
            .cloned()
            .ok_or(backend::core::registry::agent_registry::AgentRegistryError::AgentNotFound { id: *agent_id })
    }
    
    async fn get_by_name(&self, name: &str) -> Result<Agent, backend::core::registry::agent_registry::AgentRegistryError> {
        let agents = self.agents.lock().unwrap();
        let agents_by_name = self.agents_by_name.lock().unwrap();
        
        if let Some(agent_id) = agents_by_name.get(name) {
            agents.get(agent_id)
                .cloned()
                .ok_or(backend::core::registry::agent_registry::AgentRegistryError::AgentNotFound { id: *agent_id })
        } else {
            Err(backend::core::registry::agent_registry::AgentRegistryError::AgentNotFound { 
                id: Uuid::nil()
            })
        }
    }
    
    async fn mark_inactive_stale(&self, threshold_minutes: i64) -> Result<usize, backend::core::registry::agent_registry::AgentRegistryError> {
        let mut agents = self.agents.lock().unwrap();
        let threshold = Utc::now() - chrono::Duration::minutes(threshold_minutes);
        let mut count = 0;
        
        for agent in agents.values_mut() {
            if agent.status == "active" && agent.last_seen < threshold {
                agent.status = "inactive".to_string();
                agent.updated_at = Utc::now();
                count += 1;
                println!("⚠️ Marked agent as inactive: {} (ID: {})", agent.name, agent.id);
            }
        }
        
        Ok(count)
    }
    
    async fn unregister(&self, agent_id: &Uuid) -> Result<(), backend::core::registry::agent_registry::AgentRegistryError> {
        let mut agents = self.agents.lock().unwrap();
        let mut agents_by_name = self.agents_by_name.lock().unwrap();
        
        if let Some(agent) = agents.remove(agent_id) {
            agents_by_name.remove(&agent.name);
            println!("🗑️ Unregistered service: {} (ID: {})", agent.name, agent.id);
            Ok(())
        } else {
            Err(backend::core::registry::agent_registry::AgentRegistryError::AgentNotFound { id: *agent_id })
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 AI Workflow System - Service Registration Example");
    println!("====================================================");
    
    // Create an example registry
    let registry = ExampleRegistry::new();
    
    // Example 1: Basic service registration
    println!("\n📝 Example 1: Basic Service Registration");
    println!("----------------------------------------");
    
    let basic_config = ServiceConfig::new(
        "ai-tutor-service".to_string(),
        "http://localhost:3001".to_string(),
        vec!["tutoring".to_string(), "education".to_string()],
    );
    
    let _basic_handle = bootstrap_service(basic_config, &registry).await?;
    
    // Example 2: Advanced service configuration
    println!("\n📝 Example 2: Advanced Service Configuration");
    println!("--------------------------------------------");
    
    let advanced_config = ServiceConfig::new(
        "workflow-engine".to_string(),
        "http://localhost:3002".to_string(),
        vec!["orchestration".to_string(), "workflow".to_string(), "automation".to_string()],
    )
    .with_heartbeat_interval(30) // 30 seconds
    .with_registry_endpoint("http://localhost:8080".to_string())
    .with_auth_token("example-jwt-token-123".to_string())
    .with_metadata(serde_json::json!({
        "version": "2.1.0",
        "environment": "production",
        "region": "us-west-2",
        "features": ["parallel_execution", "error_recovery", "monitoring"],
        "max_concurrent_workflows": 100
    }));
    
    let _advanced_handle = bootstrap_service(advanced_config, &registry).await?;
    
    // Example 3: Multiple services with different capabilities
    println!("\n📝 Example 3: Multiple Specialized Services");
    println!("-------------------------------------------");
    
    // Data processing service
    let data_config = ServiceConfig::new(
        "data-processor".to_string(),
        "http://localhost:3003".to_string(),
        vec!["data_processing".to_string(), "analytics".to_string(), "etl".to_string()],
    )
    .with_heartbeat_interval(45)
    .with_metadata(serde_json::json!({
        "version": "1.5.2",
        "supported_formats": ["json", "csv", "parquet", "avro"],
        "max_file_size_mb": 500
    }));
    
    let _data_handle = bootstrap_service(data_config, &registry).await?;
    
    // Notification service  
    let notification_config = ServiceConfig::new(
        "notification-service".to_string(),
        "http://localhost:3004".to_string(),
        vec!["notifications".to_string(), "messaging".to_string(), "alerts".to_string()],
    )
    .with_heartbeat_interval(60)
    .with_metadata(serde_json::json!({
        "version": "1.0.8",
        "channels": ["email", "slack", "webhook", "sms"],
        "rate_limit_per_minute": 1000
    }));
    
    let _notification_handle = bootstrap_service(notification_config, &registry).await?;
    
    // Example 4: Service discovery
    println!("\n📝 Example 4: Service Discovery");
    println!("-------------------------------");
    
    // Wait a moment for registration to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Discover all tutoring services
    let tutoring_services = registry.discover("tutoring").await?;
    println!("🔍 Found {} tutoring service(s):", tutoring_services.len());
    for service in &tutoring_services {
        println!("  - {} at {}", service.name, service.endpoint);
    }
    
    // Discover all workflow services
    let workflow_services = registry.discover("workflow").await?;
    println!("🔍 Found {} workflow service(s):", workflow_services.len());
    for service in &workflow_services {
        println!("  - {} at {}", service.name, service.endpoint);
    }
    
    // Discover all notification services
    let notification_services = registry.discover("notifications").await?;
    println!("🔍 Found {} notification service(s):", notification_services.len());
    for service in &notification_services {
        println!("  - {} at {}", service.name, service.endpoint);
    }
    
    // Example 5: List all registered services
    println!("\n📝 Example 5: All Registered Services");
    println!("------------------------------------");
    
    let all_services = registry.list_active().await?;
    println!("📋 Total active services: {}", all_services.len());
    for (index, service) in all_services.iter().enumerate() {
        println!("{}. {} ({})", 
            index + 1, 
            service.name, 
            service.capabilities.join(", ")
        );
        println!("   Endpoint: {}", service.endpoint);
        println!("   Status: {} | Last seen: {}", service.status, service.last_seen);
        if let Some(version) = service.metadata.get("version") {
            println!("   Version: {}", version);
        }
        println!();
    }
    
    // Example 6: Demonstrate graceful shutdown
    println!("📝 Example 6: Graceful Shutdown");
    println!("-------------------------------");
    println!("Press Ctrl+C to demonstrate graceful shutdown...");
    
    // Wait for Ctrl+C
    signal::ctrl_c().await?;
    
    println!("\n🛑 Shutdown signal received. Cleaning up...");
    
    // In a real application, you would:
    // 1. Stop accepting new work
    // 2. Complete ongoing operations
    // 3. Unregister from the service registry
    // 4. Clean up resources
    
    // For this example, let's unregister all services
    let registered_services = registry.get_registered_services();
    for service in registered_services {
        registry.unregister(&service.id).await?;
    }
    
    println!("✅ All services unregistered. Shutdown complete.");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_service_registration_example() {
        let registry = ExampleRegistry::new();
        
        // Test basic service registration
        let config = ServiceConfig::new(
            "test-service".to_string(),
            "http://localhost:9999".to_string(),
            vec!["testing".to_string()],
        );
        
        let handle = bootstrap_service(config, &registry).await
            .expect("Failed to bootstrap test service");
        
        // Verify service was registered
        let services = registry.list_active().await
            .expect("Failed to list services");
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "test-service");
        
        // Test service discovery
        let test_services = registry.discover("testing").await
            .expect("Failed to discover services");
        assert_eq!(test_services.len(), 1);
        assert_eq!(test_services[0].name, "test-service");
        
        // Clean up
        handle.abort();
    }
    
    #[tokio::test]
    async fn test_multiple_service_registration() {
        let registry = ExampleRegistry::new();
        
        // Register multiple services
        let configs = vec![
            ServiceConfig::new("service-1".to_string(), "http://localhost:8001".to_string(), vec!["capability-a".to_string()]),
            ServiceConfig::new("service-2".to_string(), "http://localhost:8002".to_string(), vec!["capability-b".to_string()]),
            ServiceConfig::new("service-3".to_string(), "http://localhost:8003".to_string(), vec!["capability-a".to_string(), "capability-c".to_string()]),
        ];
        
        let mut handles = Vec::new();
        for config in configs {
            let handle = bootstrap_service(config, &registry).await
                .expect("Failed to bootstrap service");
            handles.push(handle);
        }
        
        // Verify all services were registered
        let all_services = registry.list_active().await
            .expect("Failed to list services");
        assert_eq!(all_services.len(), 3);
        
        // Test capability-based discovery
        let capability_a_services = registry.discover("capability-a").await
            .expect("Failed to discover capability-a services");
        assert_eq!(capability_a_services.len(), 2); // service-1 and service-3
        
        let capability_b_services = registry.discover("capability-b").await
            .expect("Failed to discover capability-b services");
        assert_eq!(capability_b_services.len(), 1); // service-2
        
        let capability_c_services = registry.discover("capability-c").await
            .expect("Failed to discover capability-c services");
        assert_eq!(capability_c_services.len(), 1); // service-3
        
        // Clean up
        for handle in handles {
            handle.abort();
        }
    }
}