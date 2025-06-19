//! Service configuration management
//!
//! This module provides configuration management for services including:
//! - Service-specific configuration loading
//! - Configuration hot-reload capabilities
//! - Environment-based configuration
//! - Configuration validation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use workflow_engine_core::error::{WorkflowError, ErrorContext, ErrorContextExt};

/// Service configuration with hot-reload support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfiguration {
    /// Service name
    pub name: String,
    
    /// Service version
    pub version: String,
    
    /// Service endpoint
    pub endpoint: String,
    
    /// Service capabilities
    pub capabilities: Vec<String>,
    
    /// Service dependencies
    pub dependencies: Vec<ServiceDependency>,
    
    /// Health check configuration
    pub health_check: HealthCheckConfig,
    
    /// Retry configuration
    pub retry_config: RetryConfig,
    
    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,
    
    /// Custom configuration values
    pub custom_config: HashMap<String, serde_json::Value>,
    
    /// Environment-specific overrides
    pub environment: String,
}

/// Service dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependency {
    /// Name of the dependent service
    pub service_name: String,
    
    /// Required version (semantic versioning)
    pub version_requirement: String,
    
    /// Whether this dependency is optional
    pub optional: bool,
    
    /// Required capabilities from the dependency
    pub required_capabilities: Vec<String>,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Health check endpoint path
    pub endpoint: String,
    
    /// Health check interval in seconds
    pub interval_seconds: u64,
    
    /// Health check timeout in seconds
    pub timeout_seconds: u64,
    
    /// Number of consecutive failures before marking unhealthy
    pub failure_threshold: u32,
    
    /// Number of consecutive successes before marking healthy
    pub success_threshold: u32,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    
    /// Initial retry delay in milliseconds
    pub initial_delay_ms: u64,
    
    /// Maximum retry delay in milliseconds
    pub max_delay_ms: u64,
    
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
    
    /// Whether to add jitter to retry delays
    pub jitter: bool,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures to open circuit
    pub failure_threshold: u32,
    
    /// Time window for failure counting in seconds
    pub failure_window_seconds: u64,
    
    /// Time to wait before attempting recovery in seconds
    pub recovery_timeout_seconds: u64,
    
    /// Number of successful calls to close circuit
    pub success_threshold: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            endpoint: "/health".to_string(),
            interval_seconds: 30,
            timeout_seconds: 5,
            failure_threshold: 3,
            success_threshold: 2,
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            failure_window_seconds: 60,
            recovery_timeout_seconds: 30,
            success_threshold: 3,
        }
    }
}

impl CircuitBreakerConfig {
    /// Set failure threshold
    pub fn with_failure_threshold(mut self, threshold: u32) -> Self {
        self.failure_threshold = threshold;
        self
    }
    
    /// Set recovery timeout
    pub fn with_recovery_timeout(mut self, seconds: u64) -> Self {
        self.recovery_timeout_seconds = seconds;
        self
    }
}

/// Configuration manager with hot-reload support
pub struct ConfigurationManager {
    /// Configuration storage
    configurations: Arc<RwLock<HashMap<String, ServiceConfiguration>>>,
    
    /// Configuration file path
    config_path: Option<String>,
    
    /// Environment
    environment: String,
}

impl ConfigurationManager {
    /// Create a new configuration manager
    pub fn new(environment: String) -> Self {
        Self {
            configurations: Arc::new(RwLock::new(HashMap::new())),
            config_path: None,
            environment,
        }
    }
    
    /// Create configuration manager with file path for hot-reload
    pub fn with_config_file(environment: String, config_path: String) -> Self {
        Self {
            configurations: Arc::new(RwLock::new(HashMap::new())),
            config_path: Some(config_path),
            environment,
        }
    }
    
    /// Load configuration from file
    pub async fn load_from_file(&self, path: &str) -> Result<(), WorkflowError> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| WorkflowError::ConfigurationError(
                format!("Failed to read config file {}: {}", path, e)
            ))?;
            
        let configs: Vec<ServiceConfiguration> = serde_json::from_str(&content)
            .map_err(|e| WorkflowError::ConfigurationError(
                format!("Failed to parse config JSON: {}", e)
            ))?;
            
        let mut configurations = self.configurations.write().await;
        for config in configs {
            if config.environment == self.environment || config.environment == "all" {
                configurations.insert(config.name.clone(), config);
            }
        }
        
        Ok(())
    }
    
    /// Add or update a service configuration
    pub async fn upsert_configuration(&self, config: ServiceConfiguration) -> Result<(), WorkflowError> {
        let mut configurations = self.configurations.write().await;
        configurations.insert(config.name.clone(), config);
        Ok(())
    }
    
    /// Get configuration for a service
    pub async fn get_configuration(&self, service_name: &str) -> Result<ServiceConfiguration, WorkflowError> {
        let configurations = self.configurations.read().await;
        configurations
            .get(service_name)
            .cloned()
            .ok_or_else(|| WorkflowError::ConfigurationError(
                format!("Configuration not found for service: {}", service_name)
            ))
    }
    
    /// Get all configurations
    pub async fn get_all_configurations(&self) -> Vec<ServiceConfiguration> {
        let configurations = self.configurations.read().await;
        configurations.values().cloned().collect()
    }
    
    /// Validate service dependencies
    pub async fn validate_dependencies(&self, service_name: &str) -> Result<Vec<String>, WorkflowError> {
        let configurations = self.configurations.read().await;
        
        let config = configurations
            .get(service_name)
            .ok_or_else(|| WorkflowError::ConfigurationError(
                format!("Service configuration not found: {}", service_name)
            ))?;
            
        let mut missing_deps = Vec::new();
        
        for dep in &config.dependencies {
            if !dep.optional && !configurations.contains_key(&dep.service_name) {
                missing_deps.push(dep.service_name.clone());
            }
        }
        
        Ok(missing_deps)
    }
    
    /// Start configuration hot-reload monitoring
    pub async fn start_hot_reload(&self, interval_seconds: u64) {
        if let Some(config_path) = &self.config_path {
            let manager = self.clone();
            let path = config_path.clone();
            
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(
                    std::time::Duration::from_secs(interval_seconds)
                );
                
                loop {
                    interval.tick().await;
                    
                    if let Err(e) = manager.reload_configuration(&path).await {
                        tracing::error!("Failed to reload configuration: {}", e);
                    } else {
                        tracing::info!("Configuration reloaded successfully");
                    }
                }
            });
        }
    }
    
    /// Reload configuration from file
    async fn reload_configuration(&self, path: &str) -> Result<(), WorkflowError> {
        self.load_from_file(path).await?;
        tracing::info!("Configuration reloaded from: {}", path);
        Ok(())
    }
}

impl Clone for ConfigurationManager {
    fn clone(&self) -> Self {
        Self {
            configurations: self.configurations.clone(),
            config_path: self.config_path.clone(),
            environment: self.environment.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_configuration_manager() {
        let manager = ConfigurationManager::new("test".to_string());
        
        let config = ServiceConfiguration {
            name: "test-service".to_string(),
            version: "1.0.0".to_string(),
            endpoint: "http://localhost:8080".to_string(),
            capabilities: vec!["test".to_string()],
            dependencies: vec![],
            health_check: HealthCheckConfig::default(),
            retry_config: RetryConfig::default(),
            circuit_breaker: CircuitBreakerConfig::default(),
            custom_config: HashMap::new(),
            environment: "test".to_string(),
        };
        
        manager.upsert_configuration(config.clone()).await.unwrap();
        
        let retrieved = manager.get_configuration("test-service").await.unwrap();
        assert_eq!(retrieved.name, "test-service");
        assert_eq!(retrieved.version, "1.0.0");
    }
    
    #[tokio::test]
    async fn test_dependency_validation() {
        let manager = ConfigurationManager::new("test".to_string());
        
        let service1 = ServiceConfiguration {
            name: "service1".to_string(),
            version: "1.0.0".to_string(),
            endpoint: "http://localhost:8081".to_string(),
            capabilities: vec!["capability1".to_string()],
            dependencies: vec![
                ServiceDependency {
                    service_name: "service2".to_string(),
                    version_requirement: "^1.0.0".to_string(),
                    optional: false,
                    required_capabilities: vec!["capability2".to_string()],
                }
            ],
            health_check: HealthCheckConfig::default(),
            retry_config: RetryConfig::default(),
            circuit_breaker: CircuitBreakerConfig::default(),
            custom_config: HashMap::new(),
            environment: "test".to_string(),
        };
        
        manager.upsert_configuration(service1).await.unwrap();
        
        let missing = manager.validate_dependencies("service1").await.unwrap();
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0], "service2");
    }
}