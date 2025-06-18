use crate::core::registry::agent_registry::{AgentRegistration, AgentRegistryError};
use crate::monitoring::metrics::{CrossSystemMetrics, DiscoveryMetrics};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur during cross-system operations
#[derive(Error, Debug)]
pub enum CrossSystemError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Registry error: {0}")]
    RegistryError(#[from] AgentRegistryError),
    
    #[error("Service not found: {service_name}")]
    ServiceNotFound { service_name: String },
    
    #[error("Authentication failed: {reason}")]
    AuthenticationFailed { reason: String },
    
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },
}

/// Configuration for external services that need to register with the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalServiceConfig {
    /// Service name (must be unique)
    pub name: String,
    
    /// Service endpoint URL
    pub endpoint: String,
    
    /// List of capabilities this service provides
    pub capabilities: Vec<String>,
    
    /// Registry endpoint to register with
    pub registry_endpoint: String,
    
    /// Authentication token for registry
    pub auth_token: Option<String>,
    
    /// Heartbeat interval in seconds
    pub heartbeat_interval: u64,
    
    /// Service metadata
    pub metadata: serde_json::Value,
    
    /// Service type (e.g., "python", "rust", "node")
    pub service_type: String,
}

impl ExternalServiceConfig {
    pub fn new(
        name: String,
        endpoint: String,
        capabilities: Vec<String>,
        registry_endpoint: String,
    ) -> Self {
        Self {
            name,
            endpoint,
            capabilities,
            registry_endpoint,
            auth_token: None,
            heartbeat_interval: 60,
            metadata: serde_json::json!({}),
            service_type: "external".to_string(),
        }
    }
    
    pub fn with_auth_token(mut self, token: String) -> Self {
        self.auth_token = Some(token);
        self
    }
    
    pub fn with_heartbeat_interval(mut self, interval: u64) -> Self {
        self.heartbeat_interval = interval;
        self
    }
    
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
    
    pub fn with_service_type(mut self, service_type: String) -> Self {
        self.service_type = service_type;
        self
    }
}

/// Registration response from the registry
#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrationResponse {
    pub agent_id: Uuid,
    pub status: String,
    pub message: String,
    pub heartbeat_url: String,
}

/// Heartbeat request payload
#[derive(Debug, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub status: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Client for handling service registration with the registry
pub struct ServiceRegistrationClient {
    client: Client,
    config: ExternalServiceConfig,
    agent_id: Option<Uuid>,
}

impl ServiceRegistrationClient {
    pub fn new(config: ExternalServiceConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());
            
        Self {
            client,
            config,
            agent_id: None,
        }
    }
    
    /// Register the service with the registry
    pub async fn register(&mut self) -> Result<RegistrationResponse, CrossSystemError> {
        let registration = AgentRegistration {
            name: self.config.name.clone(),
            endpoint: self.config.endpoint.clone(),
            capabilities: self.config.capabilities.clone(),
            metadata: self.config.metadata.clone(),
        };
        
        let url = format!("{}/registry/agents", self.config.registry_endpoint);
        let mut request = self.client.post(&url).json(&registration);
        
        // Add authentication if available
        if let Some(ref token) = self.config.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }
        
        let response = request.send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(CrossSystemError::ConfigurationError {
                message: format!("HTTP error: {} - {}", status, text),
            });
        }
        
        let agent: crate::db::agent::Agent = response.json().await?;
        self.agent_id = Some(agent.id);
        
        let registration_response = RegistrationResponse {
            agent_id: agent.id,
            status: "registered".to_string(),
            message: format!("Service '{}' registered successfully", self.config.name),
            heartbeat_url: format!("{}/registry/agents/{}/heartbeat", 
                                 self.config.registry_endpoint, agent.id),
        };
        
        println!("✅ Service '{}' registered with ID: {}", self.config.name, agent.id);
        
        Ok(registration_response)
    }
    
    /// Send a heartbeat to the registry
    pub async fn send_heartbeat(&self) -> Result<(), CrossSystemError> {
        let agent_id = self.agent_id.ok_or_else(|| {
            CrossSystemError::ConfigurationError {
                message: "Service not registered yet".to_string(),
            }
        })?;
        
        let heartbeat = HeartbeatRequest {
            timestamp: chrono::Utc::now(),
            status: Some("active".to_string()),
            metadata: Some(self.config.metadata.clone()),
        };
        
        let url = format!("{}/registry/agents/{}/heartbeat", 
                         self.config.registry_endpoint, agent_id);
        let mut request = self.client.post(&url).json(&heartbeat);
        
        // Add authentication if available
        if let Some(ref token) = self.config.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }
        
        let response = request.send().await?;
        
        if response.status().is_success() {
            println!("💓 Heartbeat sent for service '{}'", self.config.name);
            Ok(())
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(CrossSystemError::ConfigurationError {
                message: format!("Heartbeat failed: {} - {}", status, text),
            })
        }
    }
    
    /// Start automatic heartbeat loop
    pub async fn start_heartbeat_loop(&self) -> tokio::task::JoinHandle<()> {
        let client = self.client.clone();
        let config = self.config.clone();
        let agent_id = self.agent_id;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(config.heartbeat_interval));
            
            loop {
                interval.tick().await;
                
                if let Some(agent_id) = agent_id {
                    let heartbeat = HeartbeatRequest {
                        timestamp: chrono::Utc::now(),
                        status: Some("active".to_string()),
                        metadata: Some(config.metadata.clone()),
                    };
                    
                    let url = format!("{}/registry/agents/{}/heartbeat", 
                                     config.registry_endpoint, agent_id);
                    let mut request = client.post(&url).json(&heartbeat);
                    
                    if let Some(ref token) = config.auth_token {
                        request = request.header("Authorization", format!("Bearer {}", token));
                    }
                    
                    match request.send().await {
                        Ok(response) => {
                            if response.status().is_success() {
                                println!("💓 Heartbeat sent for service '{}'", config.name);
                            } else {
                                eprintln!("❌ Heartbeat failed for service '{}': HTTP {}", 
                                         config.name, response.status());
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Heartbeat error for service '{}': {}", config.name, e);
                        }
                    }
                }
            }
        })
    }
    
    /// Get the agent ID if registered
    pub fn get_agent_id(&self) -> Option<Uuid> {
        self.agent_id
    }
    
    /// Check if the service is registered
    pub fn is_registered(&self) -> bool {
        self.agent_id.is_some()
    }
}

/// Trait for cross-system communication clients
#[async_trait]
pub trait CrossSystemClient: Send + Sync {
    /// Make a cross-system call to another service
    async fn call_service(
        &self,
        service_name: &str,
        method: &str,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, CrossSystemError>;
    
    /// Discover services by capability
    async fn discover_services(&self, capability: &str) -> Result<Vec<String>, CrossSystemError>;
    
    /// Get service endpoint by name
    async fn get_service_endpoint(&self, service_name: &str) -> Result<String, CrossSystemError>;
}

/// HTTP-based cross-system client implementation
pub struct HttpCrossSystemClient {
    client: Client,
    registry_endpoint: String,
    auth_token: Option<String>,
}

impl HttpCrossSystemClient {
    pub fn new(registry_endpoint: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());
            
        Self {
            client,
            registry_endpoint,
            auth_token: None,
        }
    }
    
    pub fn with_auth_token(mut self, token: String) -> Self {
        self.auth_token = Some(token);
        self
    }
}

#[async_trait]
impl CrossSystemClient for HttpCrossSystemClient {
    async fn call_service(
        &self,
        service_name: &str,
        method: &str,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, CrossSystemError> {
        // Start metrics timer
        let timer = CrossSystemMetrics::record_call_start(service_name, method);
        
        // First, get the service endpoint
        let endpoint = match self.get_service_endpoint(service_name).await {
            Ok(endpoint) => endpoint,
            Err(e) => {
                timer.failure("service_discovery");
                return Err(e);
            }
        };
        
        // Make the actual service call
        let url = format!("{}/{}", endpoint, method);
        let mut request = self.client.post(&url).json(&payload);
        
        // Add authentication if available
        if let Some(ref token) = self.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }
        
        let response = match request.send().await {
            Ok(response) => response,
            Err(e) => {
                timer.failure("network_error");
                return Err(CrossSystemError::HttpError(e));
            }
        };
        
        if response.status().is_success() {
            match response.json().await {
                Ok(result) => {
                    timer.success();
                    Ok(result)
                }
                Err(e) => {
                    timer.failure("response_parsing");
                    Err(CrossSystemError::ConfigurationError {
                        message: format!("Failed to parse response: {}", e),
                    })
                }
            }
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            let error_type = match status.as_u16() {
                400..=499 => "client_error",
                500..=599 => "server_error",
                _ => "unknown_error",
            };
            timer.failure(error_type);
            Err(CrossSystemError::ConfigurationError {
                message: format!("Service call failed: {} - {}", status, text),
            })
        }
    }
    
    async fn discover_services(&self, capability: &str) -> Result<Vec<String>, CrossSystemError> {
        let url = format!("{}/registry/agents/discover?capability={}", 
                         self.registry_endpoint, capability);
        let mut request = self.client.get(&url);
        
        // Add authentication if available
        if let Some(ref token) = self.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }
        
        let response = request.send().await;
        
        match response {
            Ok(response) if response.status().is_success() => {
                match response.json::<serde_json::Value>().await {
                    Ok(discovery_response) => {
                        if let Some(agents) = discovery_response.get("agents").and_then(|v| v.as_array()) {
                            let service_names: Vec<String> = agents
                                .iter()
                                .filter_map(|agent| agent.get("name").and_then(|v| v.as_str()))
                                .map(|s| s.to_string())
                                .collect();
                            
                            // Record successful discovery with service count
                            DiscoveryMetrics::record_discovery(capability, "success");
                            DiscoveryMetrics::update_registered_services(capability, service_names.len() as i64);
                            
                            Ok(service_names)
                        } else {
                            DiscoveryMetrics::record_discovery(capability, "success");
                            DiscoveryMetrics::update_registered_services(capability, 0);
                            Ok(Vec::new())
                        }
                    }
                    Err(e) => {
                        DiscoveryMetrics::record_discovery(capability, "parse_error");
                        Err(CrossSystemError::ConfigurationError {
                            message: format!("Failed to parse discovery response: {}", e),
                        })
                    }
                }
            }
            Ok(response) => {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                DiscoveryMetrics::record_discovery(capability, "http_error");
                Err(CrossSystemError::ConfigurationError {
                    message: format!("Service discovery failed: {} - {}", status, text),
                })
            }
            Err(e) => {
                DiscoveryMetrics::record_discovery(capability, "network_error");
                Err(CrossSystemError::HttpError(e))
            }
        }
    }
    
    async fn get_service_endpoint(&self, service_name: &str) -> Result<String, CrossSystemError> {
        let url = format!("{}/registry/agents", self.registry_endpoint);
        let mut request = self.client.get(&url);
        
        // Add authentication if available
        if let Some(ref token) = self.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }
        
        let response = request.send().await?;
        
        if response.status().is_success() {
            let agents_response: serde_json::Value = response.json().await?;
            
            if let Some(agents) = agents_response.get("agents").and_then(|v| v.as_array()) {
                for agent in agents {
                    if let (Some(name), Some(endpoint)) = (
                        agent.get("name").and_then(|v| v.as_str()),
                        agent.get("endpoint").and_then(|v| v.as_str()),
                    ) {
                        if name == service_name {
                            return Ok(endpoint.to_string());
                        }
                    }
                }
            }
            
            Err(CrossSystemError::ServiceNotFound {
                service_name: service_name.to_string(),
            })
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(CrossSystemError::ConfigurationError {
                message: format!("Service discovery failed: {} - {}", status, text),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_external_service_config_creation() {
        let config = ExternalServiceConfig::new(
            "test-service".to_string(),
            "http://localhost:3000".to_string(),
            vec!["testing".to_string(), "example".to_string()],
            "http://localhost:8080".to_string(),
        );
        
        assert_eq!(config.name, "test-service");
        assert_eq!(config.endpoint, "http://localhost:3000");
        assert_eq!(config.capabilities.len(), 2);
        assert_eq!(config.registry_endpoint, "http://localhost:8080");
        assert_eq!(config.heartbeat_interval, 60);
        assert!(config.auth_token.is_none());
    }
    
    #[test]
    fn test_external_service_config_builder() {
        let config = ExternalServiceConfig::new(
            "test-service".to_string(),
            "http://localhost:3000".to_string(),
            vec!["testing".to_string()],
            "http://localhost:8080".to_string(),
        )
        .with_auth_token("token123".to_string())
        .with_heartbeat_interval(30)
        .with_service_type("python".to_string())
        .with_metadata(serde_json::json!({"version": "1.0.0"}));
        
        assert_eq!(config.auth_token, Some("token123".to_string()));
        assert_eq!(config.heartbeat_interval, 30);
        assert_eq!(config.service_type, "python");
        assert!(config.metadata.get("version").is_some());
    }
    
    #[tokio::test]
    async fn test_http_cross_system_client_creation() {
        let client = HttpCrossSystemClient::new("http://localhost:8080".to_string())
            .with_auth_token("test-token".to_string());
        
        assert_eq!(client.registry_endpoint, "http://localhost:8080");
        assert_eq!(client.auth_token, Some("test-token".to_string()));
    }
}