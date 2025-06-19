/// Mock implementations for testing without external dependencies

use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use crate::registry::agent_registry::{Agent, AgentRegistration, AgentRegistry, AgentRegistryError};

/// In-memory mock implementation of AgentRegistry
pub struct MockAgentRegistryImpl {
    agents: std::sync::Mutex<Vec<Agent>>,
}

impl MockAgentRegistryImpl {
    pub fn new() -> Self {
        Self {
            agents: std::sync::Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl AgentRegistry for MockAgentRegistryImpl {
    async fn register(&self, registration: AgentRegistration) -> Result<Agent, AgentRegistryError> {
        let mut agents = self.agents.lock().unwrap();
        
        // Check for duplicate names
        if agents.iter().any(|a| a.name == registration.name) {
            return Err(AgentRegistryError::DuplicateName { 
                name: registration.name 
            });
        }
        
        let agent = Agent {
            id: Uuid::new_v4(),
            name: registration.name,
            endpoint: registration.endpoint,
            capabilities: registration.capabilities,
            status: "active".to_string(),
            last_seen: Utc::now(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: registration.metadata,
        };
        
        agents.push(agent.clone());
        Ok(agent)
    }
    
    async fn discover(&self, capability: &str) -> Result<Vec<Agent>, AgentRegistryError> {
        let agents = self.agents.lock().unwrap();
        Ok(agents
            .iter()
            .filter(|a| a.capabilities.contains(&capability.to_string()) && a.status == "active")
            .cloned()
            .collect())
    }
    
    async fn heartbeat(&self, agent_id: &Uuid) -> Result<(), AgentRegistryError> {
        let mut agents = self.agents.lock().unwrap();
        
        if let Some(agent) = agents.iter_mut().find(|a| &a.id == agent_id) {
            agent.last_seen = Utc::now();
            Ok(())
        } else {
            Err(AgentRegistryError::AgentNotFound { id: *agent_id })
        }
    }
    
    async fn list_active(&self) -> Result<Vec<Agent>, AgentRegistryError> {
        let agents = self.agents.lock().unwrap();
        Ok(agents
            .iter()
            .filter(|a| a.status == "active")
            .cloned()
            .collect())
    }
    
    async fn get_by_id(&self, agent_id: &Uuid) -> Result<Agent, AgentRegistryError> {
        let agents = self.agents.lock().unwrap();
        agents
            .iter()
            .find(|a| &a.id == agent_id)
            .cloned()
            .ok_or_else(|| AgentRegistryError::AgentNotFound { id: *agent_id })
    }
    
    async fn get_by_name(&self, name: &str) -> Result<Agent, AgentRegistryError> {
        let agents = self.agents.lock().unwrap();
        agents
            .iter()
            .find(|a| a.name == name)
            .cloned()
            .ok_or_else(|| AgentRegistryError::OperationFailed { 
                message: format!("Agent with name '{}' not found", name) 
            })
    }
    
    async fn mark_inactive_stale(&self, threshold_minutes: i64) -> Result<usize, AgentRegistryError> {
        let mut agents = self.agents.lock().unwrap();
        let threshold = Utc::now() - chrono::Duration::minutes(threshold_minutes);
        let mut count = 0;
        
        for agent in agents.iter_mut() {
            if agent.last_seen < threshold && agent.status == "active" {
                agent.status = "inactive".to_string();
                count += 1;
            }
        }
        
        Ok(count)
    }
    
    async fn unregister(&self, agent_id: &Uuid) -> Result<(), AgentRegistryError> {
        let mut agents = self.agents.lock().unwrap();
        
        if let Some(pos) = agents.iter().position(|a| &a.id == agent_id) {
            agents.remove(pos);
            Ok(())
        } else {
            Err(AgentRegistryError::AgentNotFound { id: *agent_id })
        }
    }
}