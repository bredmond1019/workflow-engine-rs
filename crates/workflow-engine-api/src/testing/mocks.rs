use mockall::mock;
use workflow_engine_core::registry::{
    AgentRegistry, AgentRegistration, AgentRegistryError, Agent
};

// Generate the mock using mockall
mock! {
    pub AgentRegistry {}
    
    #[async_trait::async_trait]
    impl AgentRegistry for AgentRegistry {
        async fn register(&self, agent: AgentRegistration) -> Result<Agent, AgentRegistryError>;
        async fn discover(&self, capability: &str) -> Result<Vec<Agent>, AgentRegistryError>;
        async fn heartbeat(&self, agent_id: &uuid::Uuid) -> Result<(), AgentRegistryError>;
        async fn list_active(&self) -> Result<Vec<Agent>, AgentRegistryError>;
        async fn get_by_id(&self, agent_id: &uuid::Uuid) -> Result<Agent, AgentRegistryError>;
        async fn get_by_name(&self, name: &str) -> Result<Agent, AgentRegistryError>;
        async fn mark_inactive_stale(&self, threshold_minutes: i64) -> Result<usize, AgentRegistryError>;
        async fn unregister(&self, agent_id: &uuid::Uuid) -> Result<(), AgentRegistryError>;
    }
}