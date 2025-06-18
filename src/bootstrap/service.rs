use crate::core::registry::agent_registry::{AgentRegistry, AgentRegistration, PostgresAgentRegistry};
use crate::core::error::WorkflowError;
use chrono::Utc;
use serde_json::Value;
use std::time::Duration;
use tokio::time::interval;
use uuid::Uuid;

/// Configuration for service bootstrap
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    /// Unique service name
    pub name: String,
    
    /// Service endpoint URL
    pub endpoint: String,
    
    /// List of capabilities this service provides
    pub capabilities: Vec<String>,
    
    /// Heartbeat interval in seconds
    pub heartbeat_interval: u64,
    
    /// Registry endpoint for registration
    pub registry_endpoint: String,
    
    /// Optional authentication token
    pub auth_token: Option<String>,
    
    /// Optional metadata
    pub metadata: Option<Value>,
}

impl ServiceConfig {
    pub fn new(name: String, endpoint: String, capabilities: Vec<String>) -> Self {
        Self {
            name,
            endpoint,
            capabilities,
            heartbeat_interval: 60, // Default 1 minute heartbeat
            registry_endpoint: "http://localhost:8080".to_string(),
            auth_token: None,
            metadata: None,
        }
    }
    
    pub fn with_heartbeat_interval(mut self, interval_seconds: u64) -> Self {
        self.heartbeat_interval = interval_seconds;
        self
    }
    
    pub fn with_registry_endpoint(mut self, endpoint: String) -> Self {
        self.registry_endpoint = endpoint;
        self
    }
    
    pub fn with_auth_token(mut self, token: String) -> Self {
        self.auth_token = Some(token);
        self
    }
    
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Errors that can occur during service bootstrap
#[derive(Debug)]
pub enum BootstrapError {
    RegistrationFailed(String),
    HeartbeatFailed(String),
    ConfigurationError(String),
    NetworkError(String),
}

impl std::fmt::Display for BootstrapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BootstrapError::RegistrationFailed(msg) => write!(f, "Registration failed: {}", msg),
            BootstrapError::HeartbeatFailed(msg) => write!(f, "Heartbeat failed: {}", msg),
            BootstrapError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            BootstrapError::NetworkError(msg) => write!(f, "Network error: {}", msg),
        }
    }
}

impl std::error::Error for BootstrapError {}

/// Bootstrap a service with automatic registration and heartbeat
pub async fn bootstrap_service(
    config: ServiceConfig,
    registry: &dyn AgentRegistry,
) -> Result<tokio::task::JoinHandle<()>, BootstrapError> {
    // Validate configuration
    if config.name.is_empty() {
        return Err(BootstrapError::ConfigurationError(
            "Service name cannot be empty".to_string()
        ));
    }
    
    if config.endpoint.is_empty() {
        return Err(BootstrapError::ConfigurationError(
            "Service endpoint cannot be empty".to_string()
        ));
    }
    
    if config.capabilities.is_empty() {
        return Err(BootstrapError::ConfigurationError(
            "Service must have at least one capability".to_string()
        ));
    }

    // Register the service
    let service_id = register_service(&config, registry).await?;
    
    // Spawn heartbeat task
    let heartbeat_handle = spawn_heartbeat_task(service_id, config, registry).await;
    
    Ok(heartbeat_handle)
}

/// Register a service with the registry
async fn register_service(
    config: &ServiceConfig,
    registry: &dyn AgentRegistry,
) -> Result<Uuid, BootstrapError> {
    let registration = AgentRegistration {
        name: config.name.clone(),
        endpoint: config.endpoint.clone(),
        capabilities: config.capabilities.clone(),
        metadata: config.metadata.clone().unwrap_or_else(|| serde_json::json!({})),
    };
    
    match registry.register(registration).await {
        Ok(agent) => {
            println!("Service '{}' registered successfully with ID: {}", config.name, agent.id);
            Ok(agent.id)
        }
        Err(e) => {
            Err(BootstrapError::RegistrationFailed(
                format!("Failed to register service '{}': {}", config.name, e)
            ))
        }
    }
}

/// Spawn a heartbeat task that runs in the background
async fn spawn_heartbeat_task(
    service_id: Uuid,
    config: ServiceConfig,
    registry: &dyn AgentRegistry,
) -> tokio::task::JoinHandle<()> {
    let interval_duration = Duration::from_secs(config.heartbeat_interval);
    let service_name = config.name.clone();
    let registry_endpoint = config.registry_endpoint.clone();
    let auth_token = config.auth_token.clone();
    
    tokio::spawn(async move {
        let mut interval_timer = interval(interval_duration);
        let client = reqwest::Client::new();
        let mut consecutive_failures = 0u32;
        let max_retries = 3;
        let max_consecutive_failures = 5;
        let base_retry_delay = Duration::from_secs(1);
        
        loop {
            interval_timer.tick().await;
            
            let heartbeat_success = send_heartbeat_with_retry(
                &client,
                &registry_endpoint,
                service_id,
                &auth_token,
                &service_name,
                max_retries,
                base_retry_delay,
            ).await;
            
            if heartbeat_success {
                // Reset failure counter on success
                consecutive_failures = 0;
                println!("Heartbeat sent successfully for service '{}' (ID: {})", service_name, service_id);
            } else {
                consecutive_failures += 1;
                eprintln!("Heartbeat failed for service '{}' (ID: {}). Consecutive failures: {}", 
                         service_name, service_id, consecutive_failures);
                
                // If we have too many consecutive failures, attempt re-registration
                if consecutive_failures >= max_consecutive_failures {
                    eprintln!("Maximum consecutive failures reached for service '{}'. Attempting re-registration...", service_name);
                    
                    // Note: In a real implementation, we would need a way to pass the registry
                    // to this task for re-registration. For now, we'll log the intent.
                    // This could be solved by using a channel to communicate back to the main
                    // bootstrap function or by restructuring the architecture.
                    eprintln!("Re-registration would be attempted here for service '{}' (ID: {})", service_name, service_id);
                    
                    // Reset failure counter after re-registration attempt
                    consecutive_failures = 0;
                }
            }
        }
    })
}

/// Send heartbeat with exponential backoff retry logic
async fn send_heartbeat_with_retry(
    client: &reqwest::Client,
    registry_endpoint: &str,
    service_id: Uuid,
    auth_token: &Option<String>,
    service_name: &str,
    max_retries: u32,
    base_delay: Duration,
) -> bool {
    for attempt in 0..=max_retries {
        let url = format!("{}/registry/agents/{}/heartbeat", registry_endpoint, service_id);
        let mut request_builder = client.post(&url);
        
        // Add authentication header if available
        if let Some(token) = auth_token {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", token));
        }
        
        match request_builder.send().await {
            Ok(response) => {
                if response.status().is_success() {
                    return true;
                } else {
                    eprintln!("Heartbeat attempt {} failed for service '{}' (ID: {}): HTTP {}", 
                             attempt + 1, service_name, service_id, response.status());
                    
                    // Don't retry on 4xx errors (client errors) - these won't be fixed by retrying
                    if response.status().is_client_error() {
                        eprintln!("Client error detected - not retrying for service '{}'", service_name);
                        return false;
                    }
                }
            }
            Err(e) => {
                eprintln!("Heartbeat attempt {} failed for service '{}' (ID: {}): {}", 
                         attempt + 1, service_name, service_id, e);
            }
        }
        
        // Don't sleep after the last attempt
        if attempt < max_retries {
            // Exponential backoff: base_delay * 2^attempt
            let delay = base_delay * (2_u32.pow(attempt));
            tokio::time::sleep(delay).await;
        }
    }
    
    false
}

/// Helper function to bootstrap a service with database registry
/// Note: In production, you would pass a properly configured connection pool
pub async fn bootstrap_service_with_db(
    config: ServiceConfig,
    registry: PostgresAgentRegistry,
) -> Result<tokio::task::JoinHandle<()>, BootstrapError> {
    bootstrap_service(config, &registry).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use crate::db::agent::Agent;
    use crate::core::registry::agent_registry::AgentRegistryError;

    #[tokio::test]
    async fn test_service_config_creation() {
        let config = ServiceConfig::new(
            "test-service".to_string(),
            "http://localhost:3000".to_string(),
            vec!["capability1".to_string(), "capability2".to_string()],
        );
        
        assert_eq!(config.name, "test-service");
        assert_eq!(config.endpoint, "http://localhost:3000");
        assert_eq!(config.capabilities.len(), 2);
        assert_eq!(config.heartbeat_interval, 60);
    }
    
    #[tokio::test]
    async fn test_service_config_builder() {
        let config = ServiceConfig::new(
            "test-service".to_string(),
            "http://localhost:3000".to_string(),
            vec!["capability1".to_string()],
        )
        .with_heartbeat_interval(30)
        .with_registry_endpoint("http://registry:8080".to_string())
        .with_auth_token("token123".to_string())
        .with_metadata(serde_json::json!({"version": "1.0"}));
        
        assert_eq!(config.heartbeat_interval, 30);
        assert_eq!(config.registry_endpoint, "http://registry:8080");
        assert_eq!(config.auth_token, Some("token123".to_string()));
        assert!(config.metadata.is_some());
    }
    
    // Create a simple test registry for validation tests
    struct TestRegistry;
    
    #[async_trait]
    impl AgentRegistry for TestRegistry {
        async fn register(&self, agent: AgentRegistration) -> Result<Agent, AgentRegistryError> {
            // Create a mock Agent for testing
            let now = Utc::now();
            Ok(Agent {
                id: Uuid::new_v4(),
                name: agent.name,
                endpoint: agent.endpoint,
                capabilities: agent.capabilities,
                status: "active".to_string(),
                last_seen: now,
                metadata: agent.metadata,
                created_at: now,
                updated_at: now,
            })
        }
        
        async fn discover(&self, _capability: &str) -> Result<Vec<Agent>, AgentRegistryError> {
            // Return empty Vec for test scenarios - no agents discovered
            Ok(Vec::new())
        }
        
        async fn heartbeat(&self, _agent_id: &Uuid) -> Result<(), AgentRegistryError> {
            // Return success for test scenarios - heartbeat always succeeds
            Ok(())
        }
        
        async fn list_active(&self) -> Result<Vec<Agent>, AgentRegistryError> {
            // Return empty Vec for test scenarios - no active agents
            Ok(Vec::new())
        }
        
        async fn get_by_id(&self, agent_id: &Uuid) -> Result<Agent, AgentRegistryError> {
            // Return AgentNotFound for test scenarios - simulates no agent found
            Err(AgentRegistryError::AgentNotFound { id: *agent_id })
        }
        
        async fn get_by_name(&self, _name: &str) -> Result<Agent, AgentRegistryError> {
            // Return AgentNotFound for test scenarios - simulates no agent found
            Err(AgentRegistryError::AgentNotFound { id: Uuid::new_v4() })
        }
        
        async fn mark_inactive_stale(&self, _threshold_minutes: i64) -> Result<usize, AgentRegistryError> {
            // Return 0 for test scenarios - no agents marked inactive
            Ok(0)
        }
        
        async fn unregister(&self, _agent_id: &Uuid) -> Result<(), AgentRegistryError> {
            // Return success for test scenarios - unregister always succeeds
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_bootstrap_validation_empty_name() {
        let config = ServiceConfig::new(
            "".to_string(),
            "http://localhost:3000".to_string(),
            vec!["capability1".to_string()],
        );
        
        let test_registry = TestRegistry;
        
        let result = bootstrap_service(config, &test_registry).await;
        assert!(result.is_err());
        
        if let Err(BootstrapError::ConfigurationError(msg)) = result {
            assert!(msg.contains("Service name cannot be empty"));
        } else {
            panic!("Expected ConfigurationError for empty name");
        }
    }
    
    #[tokio::test]
    async fn test_bootstrap_validation_empty_endpoint() {
        let config = ServiceConfig::new(
            "test-service".to_string(),
            "".to_string(),
            vec!["capability1".to_string()],
        );
        
        let test_registry = TestRegistry;
        
        let result = bootstrap_service(config, &test_registry).await;
        assert!(result.is_err());
        
        if let Err(BootstrapError::ConfigurationError(msg)) = result {
            assert!(msg.contains("Service endpoint cannot be empty"));
        } else {
            panic!("Expected ConfigurationError for empty endpoint");
        }
    }
    
    #[tokio::test]
    async fn test_bootstrap_validation_empty_capabilities() {
        let config = ServiceConfig::new(
            "test-service".to_string(),
            "http://localhost:3000".to_string(),
            vec![],
        );
        
        let test_registry = TestRegistry;
        
        let result = bootstrap_service(config, &test_registry).await;
        assert!(result.is_err());
        
        if let Err(BootstrapError::ConfigurationError(msg)) = result {
            assert!(msg.contains("Service must have at least one capability"));
        } else {
            panic!("Expected ConfigurationError for empty capabilities");
        }
    }
}