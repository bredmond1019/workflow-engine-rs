use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::error::WorkflowError;

/// Agent registry errors
#[derive(Error, Debug)]
pub enum AgentRegistryError {
    #[error("Agent not found: {id}")]
    AgentNotFound { id: Uuid },
    
    #[error("Agent name already exists: {name}")]
    DuplicateName { name: String },
    
    #[error("Invalid agent status: {status}")]
    InvalidStatus { status: String },
    
    #[error("Registry operation failed: {message}")]
    OperationFailed { message: String },
}

impl From<AgentRegistryError> for WorkflowError {
    fn from(err: AgentRegistryError) -> Self {
        WorkflowError::RegistryError { message: err.to_string() }
    }
}

/// Agent information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Agent {
    pub id: Uuid,
    pub name: String,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub status: String,
    pub last_seen: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Simple registration model
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentRegistration {
    pub name: String,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Core registry operations trait
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AgentRegistry: Send + Sync {
    async fn register(&self, agent: AgentRegistration) -> Result<Agent, AgentRegistryError>;
    async fn discover(&self, capability: &str) -> Result<Vec<Agent>, AgentRegistryError>;
    async fn heartbeat(&self, agent_id: &Uuid) -> Result<(), AgentRegistryError>;
    async fn list_active(&self) -> Result<Vec<Agent>, AgentRegistryError>;
    async fn get_by_id(&self, agent_id: &Uuid) -> Result<Agent, AgentRegistryError>;
    async fn get_by_name(&self, name: &str) -> Result<Agent, AgentRegistryError>;
    async fn mark_inactive_stale(&self, threshold_minutes: i64) -> Result<usize, AgentRegistryError>;
    async fn unregister(&self, agent_id: &Uuid) -> Result<(), AgentRegistryError>;
}

// Note: Concrete implementations of AgentRegistry (like PostgresAgentRegistry)
// should be provided by the application layer or a separate persistence crate

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_registration_creation() {
        let registration = AgentRegistration {
            name: "test-agent".to_string(),
            endpoint: "http://localhost:8080".to_string(),
            capabilities: vec!["test".to_string(), "demo".to_string()],
            metadata: serde_json::json!({"version": "1.0"}),
        };
        
        assert_eq!(registration.name, "test-agent");
        assert_eq!(registration.endpoint, "http://localhost:8080");
        assert_eq!(registration.capabilities.len(), 2);
        assert_eq!(registration.metadata["version"], "1.0");
    }
    
    #[test]
    fn test_agent_registration_serialization() {
        let registration = AgentRegistration {
            name: "test-agent".to_string(),
            endpoint: "http://localhost:8080".to_string(),
            capabilities: vec!["test".to_string()],
            metadata: serde_json::json!({}),
        };
        
        let json = serde_json::to_string(&registration).unwrap();
        let deserialized: AgentRegistration = serde_json::from_str(&json).unwrap();
        
        assert_eq!(registration.name, deserialized.name);
        assert_eq!(registration.endpoint, deserialized.endpoint);
        assert_eq!(registration.capabilities, deserialized.capabilities);
    }
}