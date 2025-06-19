pub mod agent_registry;
pub mod background_tasks;

pub use agent_registry::{AgentRegistry, AgentRegistration, AgentRegistryError, Agent};
pub use background_tasks::{BackgroundTaskConfig, RegistryBackgroundTasks};

#[cfg(test)]
pub use agent_registry::MockAgentRegistry;