//! Builder pattern for NodeConfig
//!
//! This module provides a fluent builder interface for creating
//! NodeConfig instances with proper validation and type safety.

use std::any::TypeId;
use std::time::Duration;
use std::marker::PhantomData;
use std::collections::HashMap;

use crate::error::WorkflowError;
use crate::nodes::{Node, config::NodeConfig};

/// Builder for creating NodeConfig with fluent interface
pub struct NodeConfigBuilder<T: Node> {
    node_type: TypeId,
    connections: Vec<TypeId>,
    is_router: bool,
    description: Option<String>,
    parallel_nodes: Vec<TypeId>,
    timeout: Option<Duration>,
    retry_attempts: Option<u32>,
    retry_delay: Option<Duration>,
    required_inputs: Vec<String>,
    metadata: std::collections::HashMap<String, serde_json::Value>,
    max_concurrent_executions: Option<usize>,
    priority: Option<u8>,
    tags: Vec<String>,
    _phantom: PhantomData<T>,
}

impl<T: Node + 'static> NodeConfigBuilder<T> {
    /// Create a new builder for the specified node type
    pub fn new() -> Self {
        Self {
            node_type: TypeId::of::<T>(),
            connections: Vec::new(),
            is_router: false,
            description: None,
            parallel_nodes: Vec::new(),
            timeout: None,
            retry_attempts: None,
            retry_delay: None,
            required_inputs: Vec::new(),
            metadata: std::collections::HashMap::new(),
            max_concurrent_executions: None,
            priority: None,
            tags: Vec::new(),
            _phantom: PhantomData,
        }
    }

    /// Set the node description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a connection to another node
    pub fn connect_to<N: Node + 'static>(mut self) -> Self {
        self.connections.push(TypeId::of::<N>());
        self
    }

    /// Add multiple connections
    pub fn connect_to_many(mut self, connections: Vec<TypeId>) -> Self {
        self.connections.extend(connections);
        self
    }

    /// Mark this node as a router
    pub fn as_router(mut self) -> Self {
        self.is_router = true;
        self
    }

    /// Add a parallel node
    pub fn parallel_with<N: Node + 'static>(mut self) -> Self {
        self.parallel_nodes.push(TypeId::of::<N>());
        self
    }

    /// Set execution timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set retry configuration
    pub fn retry(mut self, attempts: u32, delay: Duration) -> Self {
        self.retry_attempts = Some(attempts);
        self.retry_delay = Some(delay);
        self
    }

    /// Add required input field
    pub fn require_input(mut self, field: impl Into<String>) -> Self {
        self.required_inputs.push(field.into());
        self
    }

    /// Add multiple required input fields
    pub fn require_inputs(mut self, fields: Vec<String>) -> Self {
        self.required_inputs.extend(fields);
        self
    }

    /// Add metadata to the node configuration
    pub fn metadata(mut self, key: impl Into<String>, value: impl serde::Serialize) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.metadata.insert(key.into(), json_value);
        }
        self
    }

    /// Set the maximum number of concurrent executions for this node
    pub fn max_concurrent_executions(mut self, max: usize) -> Self {
        self.max_concurrent_executions = Some(max);
        self
    }

    /// Set node priority (1-255, higher numbers = higher priority)
    pub fn priority(mut self, priority: u8) -> Self {
        self.priority = Some(priority);
        self
    }

    /// Add tags to the node for categorization and filtering
    pub fn tags(mut self, tags: Vec<impl Into<String>>) -> Self {
        self.tags = tags.into_iter().map(|t| t.into()).collect();
        self
    }

    /// Add a single tag
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Build the NodeConfig with validation
    pub fn build(self) -> Result<NodeConfig, WorkflowError> {
        // Validate router configuration
        if !self.is_router && self.connections.len() > 1 {
            return Err(WorkflowError::InvalidRouter {
                node: std::any::type_name::<T>().to_string(),
            });
        }

        // Validate timeout
        if let Some(timeout) = self.timeout {
            if timeout.as_secs() == 0 {
                return Err(WorkflowError::configuration_error_simple(
                    "Timeout must be greater than 0"
                ));
            }
        }

        // Validate retry configuration
        if let Some(attempts) = self.retry_attempts {
            if attempts == 0 {
                return Err(WorkflowError::configuration_error_simple(
                    "Retry attempts must be greater than 0"
                ));
            }
            if self.retry_delay.is_none() {
                return Err(WorkflowError::configuration_error_simple(
                    "Retry delay must be specified when retry attempts are set"
                ));
            }
        }

        // Validate priority
        if let Some(priority) = self.priority {
            if priority == 0 {
                return Err(WorkflowError::configuration_error_simple(
                    "Priority must be greater than 0"
                ));
            }
        }

        // Validate max concurrent executions
        if let Some(max) = self.max_concurrent_executions {
            if max == 0 {
                return Err(WorkflowError::configuration_error_simple(
                    "Max concurrent executions must be greater than 0"
                ));
            }
        }

        // Create the full config with all fields
        let config = NodeConfig {
            node_type: self.node_type,
            connections: self.connections,
            is_router: self.is_router,
            description: self.description,
            parallel_nodes: self.parallel_nodes,
            timeout: self.timeout,
            retry_attempts: self.retry_attempts,
            retry_delay: self.retry_delay,
            required_inputs: self.required_inputs,
            metadata: self.metadata,
            max_concurrent_executions: self.max_concurrent_executions,
            priority: self.priority,
            tags: self.tags,
        };

        // Run final validation
        config.validate()?;

        Ok(config)
    }
}

impl<T: Node + 'static> Default for NodeConfigBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait for NodeConfig to provide builder
pub trait NodeConfigExt {
    /// Create a builder for this node type
    fn builder<T: Node + 'static>() -> NodeConfigBuilder<T>;
}

impl NodeConfigExt for NodeConfig {
    fn builder<T: Node + 'static>() -> NodeConfigBuilder<T> {
        NodeConfigBuilder::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::agent::BaseAgentNode;

    #[derive(Debug)]
    struct TestNode;
    impl Node for TestNode {
        fn process(&self, context: crate::task::TaskContext) -> Result<crate::task::TaskContext, WorkflowError> {
            Ok(context)
        }
    }

    #[derive(Debug)]
    struct RouterNode;
    impl Node for RouterNode {
        fn process(&self, context: crate::task::TaskContext) -> Result<crate::task::TaskContext, WorkflowError> {
            Ok(context)
        }
    }

    #[test]
    fn test_basic_builder() {
        let config = NodeConfigBuilder::<TestNode>::new()
            .description("Test node")
            .build()
            .unwrap();

        assert!(config.description.is_some());
        assert_eq!(config.description.unwrap(), "Test node");
        assert!(!config.is_router);
        assert!(config.connections.is_empty());
    }

    #[test]
    fn test_router_builder() {
        let config = NodeConfigBuilder::<RouterNode>::new()
            .description("Router node")
            .as_router()
            .connect_to::<TestNode>()
            .connect_to::<BaseAgentNode>()
            .build()
            .unwrap();

        assert!(config.is_router);
        assert_eq!(config.connections.len(), 2);
    }

    #[test]
    fn test_invalid_router() {
        let result = NodeConfigBuilder::<TestNode>::new()
            .connect_to::<RouterNode>()
            .connect_to::<BaseAgentNode>()
            .build();

        assert!(result.is_err());
        match result {
            Err(WorkflowError::InvalidRouter { .. }) => {},
            _ => panic!("Expected InvalidRouter error"),
        }
    }

    #[test]
    fn test_parallel_nodes() {
        let config = NodeConfigBuilder::<TestNode>::new()
            .parallel_with::<RouterNode>()
            .parallel_with::<BaseAgentNode>()
            .build()
            .unwrap();

        assert_eq!(config.parallel_nodes.len(), 2);
    }

    #[test]
    fn test_configuration_with_retry() {
        let config = NodeConfigBuilder::<TestNode>::new()
            .timeout(Duration::from_secs(30))
            .retry(3, Duration::from_millis(100))
            .require_input("user_id")
            .require_inputs(vec!["session_id".to_string(), "context".to_string()])
            .build()
            .unwrap();

        assert!(config.node_type == TypeId::of::<TestNode>());
        assert_eq!(config.timeout, Some(Duration::from_secs(30)));
        assert_eq!(config.retry_attempts, Some(3));
        assert_eq!(config.retry_delay, Some(Duration::from_millis(100)));
        assert_eq!(config.required_inputs.len(), 3);
        assert!(config.required_inputs.contains(&"user_id".to_string()));
    }

    #[test]
    fn test_enhanced_configuration() {
        let config = NodeConfigBuilder::<TestNode>::new()
            .description("Enhanced test node")
            .timeout(Duration::from_secs(60))
            .priority(10)
            .max_concurrent_executions(5)
            .metadata("version", "1.0.0")
            .metadata("environment", "test")
            .tags(vec!["processing", "test", "enhanced"])
            .tag("additional")
            .build()
            .unwrap();

        assert_eq!(config.description, Some("Enhanced test node".to_string()));
        assert_eq!(config.timeout, Some(Duration::from_secs(60)));
        assert_eq!(config.priority, Some(10));
        assert_eq!(config.max_concurrent_executions, Some(5));
        assert_eq!(config.metadata.len(), 2);
        assert!(config.metadata.contains_key("version"));
        assert!(config.metadata.contains_key("environment"));
        assert_eq!(config.tags.len(), 4);
        assert!(config.tags.contains(&"processing".to_string()));
        assert!(config.tags.contains(&"additional".to_string()));
    }

    #[test]
    fn test_validation_errors() {
        // Invalid priority
        let result = NodeConfigBuilder::<TestNode>::new()
            .priority(0)
            .build();
        assert!(result.is_err());

        // Invalid max concurrent executions
        let result = NodeConfigBuilder::<TestNode>::new()
            .max_concurrent_executions(0)
            .build();
        assert!(result.is_err());

        // Invalid timeout
        let result = NodeConfigBuilder::<TestNode>::new()
            .timeout(Duration::from_secs(0))
            .build();
        assert!(result.is_err());

        // Invalid retry configuration (attempts without delay)
        let result = NodeConfigBuilder::<TestNode>::new()
            .retry(3, Duration::from_millis(0))
            .build();
        assert!(result.is_err());
    }
}