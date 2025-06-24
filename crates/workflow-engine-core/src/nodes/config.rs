use std::any::TypeId;
use std::time::Duration;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use super::Node;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub node_type: TypeId,
    pub connections: Vec<TypeId>,
    pub is_router: bool,
    pub description: Option<String>,
    pub parallel_nodes: Vec<TypeId>,
    pub timeout: Option<Duration>,
    pub retry_attempts: Option<u32>,
    pub retry_delay: Option<Duration>,
    pub required_inputs: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub max_concurrent_executions: Option<usize>,
    pub priority: Option<u8>,
    pub tags: Vec<String>,
}

impl NodeConfig {
    pub fn new<T: Node + 'static>() -> Self {
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
            metadata: HashMap::new(),
            max_concurrent_executions: None,
            priority: None,
            tags: Vec::new(),
        }
    }

    pub fn with_connections(mut self, connections: Vec<TypeId>) -> Self {
        self.connections = connections;
        self
    }

    pub fn with_router(mut self, is_router: bool) -> Self {
        self.is_router = is_router;
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_parallel_nodes(mut self, parallel_nodes: Vec<TypeId>) -> Self {
        self.parallel_nodes = parallel_nodes;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn with_retry(mut self, attempts: u32, delay: Duration) -> Self {
        self.retry_attempts = Some(attempts);
        self.retry_delay = Some(delay);
        self
    }

    pub fn with_required_inputs(mut self, inputs: Vec<String>) -> Self {
        self.required_inputs = inputs;
        self
    }

    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = Some(priority);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_max_concurrent_executions(mut self, max: usize) -> Self {
        self.max_concurrent_executions = Some(max);
        self
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), crate::error::WorkflowError> {
        // Validate router configuration
        if !self.is_router && self.connections.len() > 1 {
            return Err(crate::error::WorkflowError::InvalidRouter {
                node: format!("{:?}", self.node_type),
            });
        }

        // Validate timeout
        if let Some(timeout) = self.timeout {
            if timeout.as_secs() == 0 {
                return Err(crate::error::WorkflowError::ConfigurationError(
                    "Timeout must be greater than 0".to_string()
                ));
            }
        }

        // Validate retry configuration
        if let Some(attempts) = self.retry_attempts {
            if attempts == 0 {
                return Err(crate::error::WorkflowError::ConfigurationError(
                    "Retry attempts must be greater than 0".to_string()
                ));
            }
            if self.retry_delay.is_none() {
                return Err(crate::error::WorkflowError::ConfigurationError(
                    "Retry delay must be specified when retry attempts are set".to_string()
                ));
            }
        }

        // Validate priority
        if let Some(priority) = self.priority {
            if priority == 0 {
                return Err(crate::error::WorkflowError::ConfigurationError(
                    "Priority must be greater than 0".to_string()
                ));
            }
        }

        // Validate max concurrent executions
        if let Some(max) = self.max_concurrent_executions {
            if max == 0 {
                return Err(crate::error::WorkflowError::ConfigurationError(
                    "Max concurrent executions must be greater than 0".to_string()
                ));
            }
        }

        Ok(())
    }
}
