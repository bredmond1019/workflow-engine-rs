use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::schema::agents;

/// Agent database model
#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = agents)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Agent {
    pub id: Uuid,
    pub name: String,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub status: String,
    pub last_seen: DateTime<Utc>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Agent creation model
#[derive(Insertable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = agents)]
pub struct NewAgent {
    pub id: Uuid,
    pub name: String,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub status: String,
    pub last_seen: DateTime<Utc>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Agent update model
#[derive(AsChangeset, Serialize, Deserialize, Debug)]
#[diesel(table_name = agents)]
pub struct UpdateAgent {
    pub endpoint: Option<String>,
    pub capabilities: Option<Vec<String>>,
    pub status: Option<String>,
    pub last_seen: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub updated_at: DateTime<Utc>,
}

impl NewAgent {
    pub fn new(name: String, endpoint: String, capabilities: Vec<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            endpoint,
            capabilities,
            status: "active".to_string(),
            last_seen: now,
            metadata: serde_json::json!({}),
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

impl UpdateAgent {
    pub fn heartbeat() -> Self {
        Self {
            endpoint: None,
            capabilities: None,
            status: Some("active".to_string()),
            last_seen: Some(Utc::now()),
            metadata: None,
            updated_at: Utc::now(),
        }
    }
    
    pub fn set_inactive() -> Self {
        Self {
            endpoint: None,
            capabilities: None,
            status: Some("inactive".to_string()),
            last_seen: None,
            metadata: None,
            updated_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_agent_creation() {
        let agent = NewAgent::new(
            "test-agent".to_string(),
            "http://localhost:8080".to_string(),
            vec!["capability1".to_string(), "capability2".to_string()],
        );
        
        assert_eq!(agent.name, "test-agent");
        assert_eq!(agent.endpoint, "http://localhost:8080");
        assert_eq!(agent.capabilities, vec!["capability1", "capability2"]);
        assert_eq!(agent.status, "active");
        assert_eq!(agent.metadata, serde_json::json!({}));
    }

    #[test]
    fn test_agent_with_metadata() {
        let metadata = serde_json::json!({"version": "1.0", "region": "us-west"});
        let agent = NewAgent::new(
            "test-agent".to_string(),
            "http://localhost:8080".to_string(),
            vec!["capability1".to_string()],
        ).with_metadata(metadata.clone());
        
        assert_eq!(agent.metadata, metadata);
    }

    #[test]
    fn test_update_agent_heartbeat() {
        let update = UpdateAgent::heartbeat();
        assert_eq!(update.status, Some("active".to_string()));
        assert!(update.last_seen.is_some());
    }

    #[test]
    fn test_update_agent_set_inactive() {
        let update = UpdateAgent::set_inactive();
        assert_eq!(update.status, Some("inactive".to_string()));
        assert!(update.last_seen.is_none());
    }
}