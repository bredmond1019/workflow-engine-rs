use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::core::error::WorkflowError;
use crate::db::{agent::{Agent, NewAgent, UpdateAgent}, schema::agents};

/// Agent registry errors
#[derive(Error, Debug)]
pub enum AgentRegistryError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] diesel::result::Error),
    
    #[error("Connection pool error: {0}")]
    PoolError(#[from] diesel::r2d2::PoolError),
    
    #[error("Agent not found: {id}")]
    AgentNotFound { id: Uuid },
    
    #[error("Agent name already exists: {name}")]
    DuplicateName { name: String },
    
    #[error("Invalid agent status: {status}")]
    InvalidStatus { status: String },
}

impl From<AgentRegistryError> for WorkflowError {
    fn from(err: AgentRegistryError) -> Self {
        WorkflowError::RegistryError { message: err.to_string() }
    }
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

/// PostgreSQL implementation of AgentRegistry
pub struct PostgresAgentRegistry {
    pub pool: Pool<ConnectionManager<PgConnection>>,
}

impl PostgresAgentRegistry {
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        Self { pool }
    }
    
    /// Check if agent name already exists
    async fn name_exists(&self, name: &str) -> Result<bool, AgentRegistryError> {
        use crate::db::schema::agents::dsl;
        
        let mut conn = self.pool.get()?;
        let name_owned = name.to_string();
        let count: i64 = tokio::task::spawn_blocking(move || {
            dsl::agents
                .filter(dsl::name.eq(name_owned))
                .count()
                .get_result(&mut conn)
        }).await.map_err(|e| AgentRegistryError::DatabaseError(
            diesel::result::Error::RollbackTransaction
        ))??;
        
        Ok(count > 0)
    }
}

#[async_trait]
impl AgentRegistry for PostgresAgentRegistry {
    async fn register(&self, registration: AgentRegistration) -> Result<Agent, AgentRegistryError> {
        // Check for duplicate names
        if self.name_exists(&registration.name).await? {
            return Err(AgentRegistryError::DuplicateName { 
                name: registration.name 
            });
        }
        
        let new_agent = NewAgent::new(
            registration.name,
            registration.endpoint,
            registration.capabilities,
        ).with_metadata(registration.metadata);
        
        let mut conn = self.pool.get()?;
        let agent_clone = new_agent.clone();
        
        let agent = tokio::task::spawn_blocking(move || {
            diesel::insert_into(agents::table)
                .values(&agent_clone)
                .returning(Agent::as_returning())
                .get_result(&mut conn)
        }).await.map_err(|e| AgentRegistryError::DatabaseError(
            diesel::result::Error::RollbackTransaction
        ))??;
        
        Ok(agent)
    }
    
    async fn discover(&self, capability: &str) -> Result<Vec<Agent>, AgentRegistryError> {
        use crate::db::schema::agents::dsl;
        
        let mut conn = self.pool.get()?;
        let capability_owned = capability.to_string();
        
        let agents = tokio::task::spawn_blocking(move || {
            dsl::agents
                .filter(dsl::status.eq("active"))
                .filter(dsl::capabilities.contains(vec![capability_owned]))
                .order(dsl::last_seen.desc())
                .load::<Agent>(&mut conn)
        }).await.map_err(|e| AgentRegistryError::DatabaseError(
            diesel::result::Error::RollbackTransaction
        ))??;
        
        Ok(agents)
    }
    
    async fn heartbeat(&self, agent_id: &Uuid) -> Result<(), AgentRegistryError> {
        use crate::db::schema::agents::dsl;
        
        let mut conn = self.pool.get()?;
        let agent_id_owned = *agent_id;
        
        let updated_rows = tokio::task::spawn_blocking(move || {
            diesel::update(dsl::agents.filter(dsl::id.eq(agent_id_owned)))
                .set(&UpdateAgent::heartbeat())
                .execute(&mut conn)
        }).await.map_err(|e| AgentRegistryError::DatabaseError(
            diesel::result::Error::RollbackTransaction
        ))??;
        
        if updated_rows == 0 {
            return Err(AgentRegistryError::AgentNotFound { id: *agent_id });
        }
        
        Ok(())
    }
    
    async fn list_active(&self) -> Result<Vec<Agent>, AgentRegistryError> {
        use crate::db::schema::agents::dsl;
        
        let mut conn = self.pool.get()?;
        
        let agents = tokio::task::spawn_blocking(move || {
            dsl::agents
                .filter(dsl::status.eq("active"))
                .order(dsl::last_seen.desc())
                .load::<Agent>(&mut conn)
        }).await.map_err(|e| AgentRegistryError::DatabaseError(
            diesel::result::Error::RollbackTransaction
        ))??;
        
        Ok(agents)
    }
    
    async fn get_by_id(&self, agent_id: &Uuid) -> Result<Agent, AgentRegistryError> {
        use crate::db::schema::agents::dsl;
        
        let mut conn = self.pool.get()?;
        let agent_id_owned = *agent_id;
        
        let agent = tokio::task::spawn_blocking(move || {
            dsl::agents
                .filter(dsl::id.eq(agent_id_owned))
                .first::<Agent>(&mut conn)
        }).await.map_err(|e| AgentRegistryError::DatabaseError(
            diesel::result::Error::RollbackTransaction
        ))?;
        
        match agent {
            Ok(agent) => Ok(agent),
            Err(diesel::NotFound) => Err(AgentRegistryError::AgentNotFound { id: *agent_id }),
            Err(e) => Err(AgentRegistryError::DatabaseError(e)),
        }
    }
    
    async fn get_by_name(&self, name: &str) -> Result<Agent, AgentRegistryError> {
        use crate::db::schema::agents::dsl;
        
        let mut conn = self.pool.get()?;
        let name_owned = name.to_string();
        
        let agent = tokio::task::spawn_blocking(move || {
            dsl::agents
                .filter(dsl::name.eq(name_owned))
                .first::<Agent>(&mut conn)
        }).await.map_err(|e| AgentRegistryError::DatabaseError(
            diesel::result::Error::RollbackTransaction
        ))?;
        
        match agent {
            Ok(agent) => Ok(agent),
            Err(diesel::NotFound) => Err(AgentRegistryError::AgentNotFound { 
                id: Uuid::new_v4() // placeholder since we don't have the ID
            }),
            Err(e) => Err(AgentRegistryError::DatabaseError(e)),
        }
    }
    
    async fn mark_inactive_stale(&self, threshold_minutes: i64) -> Result<usize, AgentRegistryError> {
        use crate::db::schema::agents::dsl;
        
        let cutoff_time = Utc::now() - Duration::minutes(threshold_minutes);
        let mut conn = self.pool.get()?;
        
        let updated_rows = tokio::task::spawn_blocking(move || {
            diesel::update(
                dsl::agents.filter(
                    dsl::last_seen.lt(cutoff_time)
                        .and(dsl::status.eq("active"))
                )
            )
            .set(&UpdateAgent::set_inactive())
            .execute(&mut conn)
        }).await.map_err(|e| AgentRegistryError::DatabaseError(
            diesel::result::Error::RollbackTransaction
        ))??;
        
        Ok(updated_rows)
    }
    
    async fn unregister(&self, agent_id: &Uuid) -> Result<(), AgentRegistryError> {
        use crate::db::schema::agents::dsl;
        
        let mut conn = self.pool.get()?;
        let agent_id_owned = *agent_id;
        
        let deleted_rows = tokio::task::spawn_blocking(move || {
            diesel::delete(dsl::agents.filter(dsl::id.eq(agent_id_owned)))
                .execute(&mut conn)
        }).await.map_err(|e| AgentRegistryError::DatabaseError(
            diesel::result::Error::RollbackTransaction
        ))??;
        
        if deleted_rows == 0 {
            return Err(AgentRegistryError::AgentNotFound { id: *agent_id });
        }
        
        Ok(())
    }
}

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