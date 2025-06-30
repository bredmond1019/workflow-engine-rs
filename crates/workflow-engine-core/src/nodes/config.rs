use std::any::TypeId;
use std::time::Duration;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use super::Node;

#[derive(Debug, Clone)]
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
        self.validate_router_configuration()?;
        self.validate_timeouts_and_retries()?;
        self.validate_resource_limits()?;
        self.validate_metadata_security()?;
        self.validate_tags_security()?;
        Ok(())
    }

    fn validate_router_configuration(&self) -> Result<(), crate::error::WorkflowError> {
        // Validate router configuration
        if !self.is_router && self.connections.len() > 1 {
            return Err(crate::error::WorkflowError::InvalidRouter {
                node: format!("{:?}", self.node_type),
            });
        }
        Ok(())
    }

    fn validate_timeouts_and_retries(&self) -> Result<(), crate::error::WorkflowError> {
        // Configuration constants for validation
        const MAX_TIMEOUT_SECONDS: u64 = 3600; // 1 hour max timeout
        const MAX_RETRY_ATTEMPTS: u32 = 100;   // Reasonable retry limit
        
        // Validate timeout configuration
        if let Some(timeout) = self.timeout {
            if timeout.as_secs() == 0 {
                return Err(crate::error::WorkflowError::configuration_error(
                    "Timeout must be greater than 0",
                    "timeout",
                    "node_configuration",
                    "positive duration",
                    Some("0_seconds".to_string()),
                ));
            }
            
            if timeout.as_secs() > MAX_TIMEOUT_SECONDS {
                return Err(crate::error::WorkflowError::configuration_error(
                    format!("Timeout exceeds maximum of {} seconds", MAX_TIMEOUT_SECONDS),
                    "timeout",
                    "node_configuration",
                    format!("timeout <= {} seconds", MAX_TIMEOUT_SECONDS),
                    Some(format!("{}_seconds", timeout.as_secs())),
                ));
            }
        }

        // Validate retry configuration
        if let Some(attempts) = self.retry_attempts {
            if attempts == 0 {
                return Err(crate::error::WorkflowError::configuration_error(
                    "Retry attempts must be greater than 0",
                    "retry_attempts",
                    "node_configuration",
                    "positive number",
                    Some("0".to_string()),
                ));
            }
            
            if attempts > MAX_RETRY_ATTEMPTS {
                return Err(crate::error::WorkflowError::configuration_error(
                    format!("Retry attempts exceed maximum of {}", MAX_RETRY_ATTEMPTS),
                    "retry_attempts",
                    "node_configuration",
                    format!("retry attempts <= {}", MAX_RETRY_ATTEMPTS),
                    Some(attempts.to_string()),
                ));
            }
            
            if self.retry_delay.is_none() {
                return Err(crate::error::WorkflowError::configuration_error(
                    "Retry delay must be specified when retry attempts are set",
                    "retry_delay",
                    "node_configuration",
                    "positive duration",
                    Some("missing".to_string()),
                ));
            }
        }

        Ok(())
    }

    fn validate_resource_limits(&self) -> Result<(), crate::error::WorkflowError> {
        // Validate priority
        if let Some(priority) = self.priority {
            if priority == 0 {
                return Err(crate::error::WorkflowError::configuration_error(
                    "Priority must be greater than 0",
                    "priority",
                    "node_configuration",
                    "positive number (1-255)",
                    Some("0".to_string()),
                ));
            }
        }

        // Validate max concurrent executions
        if let Some(max) = self.max_concurrent_executions {
            if max == 0 {
                return Err(crate::error::WorkflowError::configuration_error(
                    "Max concurrent executions must be greater than 0",
                    "max_concurrent_executions",
                    "node_configuration",
                    "positive number",
                    Some("0".to_string()),
                ));
            }
            
            // Check for excessive concurrent executions
            const MAX_CONCURRENT_EXECUTIONS: usize = 10000;
            if max > MAX_CONCURRENT_EXECUTIONS {
                return Err(crate::error::WorkflowError::configuration_error(
                    format!("Max concurrent executions exceed limit of {}", MAX_CONCURRENT_EXECUTIONS),
                    "max_concurrent_executions",
                    "node_configuration",
                    format!("concurrent executions <= {}", MAX_CONCURRENT_EXECUTIONS),
                    Some(max.to_string()),
                ));
            }
        }

        Ok(())
    }

    fn validate_metadata_security(&self) -> Result<(), crate::error::WorkflowError> {
        // Security configuration constants
        const MAX_METADATA_SIZE: usize = 1_000_000; // 1MB total metadata size limit
        const RESERVED_KEYS: &[&str] = &[
            "__system__", "_internal_", "NODE_TYPE", "_secret_", "__admin__",
            "password", "token", "key", "credential"
        ];

        // Check total metadata size
        let total_size: usize = self.metadata.iter()
            .map(|(k, v)| k.len() + v.to_string().len())
            .sum();
        
        if total_size > MAX_METADATA_SIZE {
            return Err(crate::error::WorkflowError::configuration_error(
                format!("Metadata size exceeds maximum of {} bytes", MAX_METADATA_SIZE),
                "metadata",
                "node_configuration",
                format!("total size <= {} bytes", MAX_METADATA_SIZE),
                Some(format!("{}_bytes", total_size)),
            ));
        }

        // Check for reserved keys and malicious content
        for (key, value) in &self.metadata {
            // Check reserved keys
            if RESERVED_KEYS.iter().any(|&reserved| key.to_lowercase().contains(reserved)) {
                return Err(crate::error::WorkflowError::configuration_error(
                    format!("Metadata key '{}' contains reserved keyword", key),
                    "metadata",
                    "node_configuration",
                    "keys without reserved keywords",
                    Some(key.clone()),
                ));
            }

            // Check for potentially malicious content
            let value_str = value.to_string();
            if self.contains_malicious_content(&value_str) {
                return Err(crate::error::WorkflowError::configuration_error(
                    format!("Metadata value contains potentially malicious content"),
                    "metadata",
                    "node_configuration",
                    "safe content without scripts or injection attempts",
                    Some("malicious_content_detected".to_string()),
                ));
            }
        }

        Ok(())
    }

    fn validate_tags_security(&self) -> Result<(), crate::error::WorkflowError> {
        const MAX_TAG_LENGTH: usize = 100;
        const MAX_TAGS_COUNT: usize = 50;

        if self.tags.len() > MAX_TAGS_COUNT {
            return Err(crate::error::WorkflowError::configuration_error(
                format!("Too many tags (max: {})", MAX_TAGS_COUNT),
                "tags",
                "node_configuration",
                format!("tag count <= {}", MAX_TAGS_COUNT),
                Some(self.tags.len().to_string()),
            ));
        }

        for tag in &self.tags {
            if tag.len() > MAX_TAG_LENGTH {
                return Err(crate::error::WorkflowError::configuration_error(
                    format!("Tag '{}' exceeds maximum length of {}", tag, MAX_TAG_LENGTH),
                    "tags",
                    "node_configuration",
                    format!("tag length <= {}", MAX_TAG_LENGTH),
                    Some(format!("length_{}", tag.len())),
                ));
            }

            if self.contains_malicious_content(tag) {
                return Err(crate::error::WorkflowError::configuration_error(
                    "Tag contains potentially malicious content",
                    "tags",
                    "node_configuration",
                    "safe tag content",
                    Some("malicious_tag_detected".to_string()),
                ));
            }

            // Check for control characters
            if tag.chars().any(|c| c.is_control()) {
                return Err(crate::error::WorkflowError::configuration_error(
                    "Tag contains invalid control characters",
                    "tags",
                    "node_configuration",
                    "tags without control characters",
                    Some("control_characters_detected".to_string()),
                ));
            }
        }

        Ok(())
    }

    fn contains_malicious_content(&self, content: &str) -> bool {
        let content_lower = content.to_lowercase();
        
        // Check for script injection patterns
        let malicious_patterns = [
            "<script", "javascript:", "vbscript:",
            "onload=", "onerror=", "onclick=",
            "'; drop table", "; drop table", "union select",
            "rm -rf", "del /", "format c:",
            "../../", "../", "..\\",
            "eval(", "exec(", "system(",
            "__import__", "subprocess",
        ];

        malicious_patterns.iter().any(|&pattern| content_lower.contains(pattern))
    }
}

// Manual Serialize implementation to handle TypeId
impl Serialize for NodeConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        
        let mut state = serializer.serialize_struct("NodeConfig", 11)?;
        state.serialize_field("node_type", &format!("{:?}", self.node_type))?;
        state.serialize_field("connections", &self.connections.iter().map(|id| format!("{:?}", id)).collect::<Vec<_>>())?;
        state.serialize_field("is_router", &self.is_router)?;
        state.serialize_field("description", &self.description)?;
        state.serialize_field("parallel_nodes", &self.parallel_nodes.iter().map(|id| format!("{:?}", id)).collect::<Vec<_>>())?;
        state.serialize_field("timeout", &self.timeout)?;
        state.serialize_field("retry_attempts", &self.retry_attempts)?;
        state.serialize_field("retry_delay", &self.retry_delay)?;
        state.serialize_field("required_inputs", &self.required_inputs)?;
        state.serialize_field("metadata", &self.metadata)?;
        state.serialize_field("max_concurrent_executions", &self.max_concurrent_executions)?;
        state.serialize_field("priority", &self.priority)?;
        state.serialize_field("tags", &self.tags)?;
        state.end()
    }
}

// Manual Deserialize implementation to handle TypeId (simplified version for compatibility)
impl<'de> Deserialize<'de> for NodeConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{MapAccess, Visitor};
        use std::fmt;
        
        struct NodeConfigVisitor;
        
        impl<'de> Visitor<'de> for NodeConfigVisitor {
            type Value = NodeConfig;
            
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct NodeConfig")
            }
            
            fn visit_map<V>(self, mut map: V) -> Result<NodeConfig, V::Error>
            where
                V: MapAccess<'de>,
            {
                // For deserialization, we'll create a basic config with default TypeId
                // This is a fallback since TypeId can't be properly deserialized
                let mut config = NodeConfig {
                    node_type: TypeId::of::<()>(), // Default/placeholder TypeId
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
                };
                
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "is_router" => config.is_router = map.next_value()?,
                        "description" => config.description = map.next_value()?,
                        "timeout" => config.timeout = map.next_value()?,
                        "retry_attempts" => config.retry_attempts = map.next_value()?,
                        "retry_delay" => config.retry_delay = map.next_value()?,
                        "required_inputs" => config.required_inputs = map.next_value()?,
                        "metadata" => config.metadata = map.next_value()?,
                        "max_concurrent_executions" => config.max_concurrent_executions = map.next_value()?,
                        "priority" => config.priority = map.next_value()?,
                        "tags" => config.tags = map.next_value()?,
                        _ => { let _: serde_json::Value = map.next_value()?; } // Ignore TypeId fields
                    }
                }
                
                Ok(config)
            }
        }
        
        deserializer.deserialize_struct("NodeConfig", &[], NodeConfigVisitor)
    }
}
