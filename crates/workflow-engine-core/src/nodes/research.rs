//! # Research Node Implementation
//!
//! This module provides the ResearchNode which can discover AI Tutor services
//! through the system registry and perform research operations via cross-system
//! communication. This demonstrates the integration between the workflow system
//! and external services.

use super::{Node, Router};
use crate::error::WorkflowError;
use crate::task::TaskContext;
use crate::integrations::{CrossSystemClient};
use crate::integrations::cross_system::HttpCrossSystemClient;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

/// Configuration for the Research Node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchConfig {
    /// Registry endpoint for service discovery
    pub registry_endpoint: String,
    /// Authentication token for registry access
    pub auth_token: Option<String>,
    /// Preferred research service capability
    pub preferred_capability: String,
    /// Timeout for research requests in seconds
    pub timeout_seconds: u64,
}

impl Default for ResearchConfig {
    fn default() -> Self {
        Self {
            registry_endpoint: "http://localhost:8080".to_string(),
            auth_token: None,
            preferred_capability: "tutoring".to_string(),
            timeout_seconds: 30,
        }
    }
}

/// Research Node that discovers AI Tutor services and performs research
pub struct ResearchNode {
    config: ResearchConfig,
    cross_system_client: Arc<dyn CrossSystemClient>,
}

impl std::fmt::Debug for ResearchNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResearchNode")
            .field("config", &self.config)
            .field("cross_system_client", &"<cross_system_client>")
            .finish()
    }
}

/// Request structure for research operations
#[derive(Debug, Serialize, Deserialize)]
pub struct ResearchRequest {
    /// The research query or topic
    pub query: String,
    /// Research context or background information
    pub context: Option<Value>,
    /// Subject area for specialized research
    pub subject: Option<String>,
    /// Difficulty level for the research
    pub difficulty_level: Option<String>,
    /// Maximum number of services to try
    pub max_services: Option<usize>,
}

/// Response structure for research operations
#[derive(Debug, Serialize, Deserialize)]
pub struct ResearchResponse {
    /// The research query that was processed
    pub query: String,
    /// The research explanation or answer
    pub explanation: String,
    /// Confidence score of the research result
    pub confidence: f64,
    /// Source service that provided the research
    pub source_service: String,
    /// Additional examples or supporting information
    pub examples: Vec<String>,
    /// Follow-up questions for deeper research
    pub follow_up_questions: Vec<String>,
    /// Additional resources for further study
    pub resources: Vec<String>,
    /// Processing metadata
    pub metadata: Value,
}

impl ResearchNode {
    /// Creates a new Research Node with the given configuration
    pub fn new(config: ResearchConfig) -> Self {
        let client = HttpCrossSystemClient::new(config.registry_endpoint.clone());
        let client = if let Some(ref token) = config.auth_token {
            client.with_auth_token(token.clone())
        } else {
            client
        };

        Self {
            config,
            cross_system_client: Arc::new(client),
        }
    }

    /// Creates a new Research Node with a custom cross-system client
    pub fn with_client(
        config: ResearchConfig,
        client: Arc<dyn CrossSystemClient>,
    ) -> Self {
        Self {
            config,
            cross_system_client: client,
        }
    }

    /// Discovers available research services from the registry
    async fn discover_research_services(&self) -> Result<Vec<String>, WorkflowError> {
        self.cross_system_client
            .discover_services(&self.config.preferred_capability)
            .await
            .map_err(|e| WorkflowError::RegistryError {
                message: format!("Service discovery failed: {}", e),
            })
    }

    /// Performs research using the first available research service
    async fn perform_research(
        &self,
        request: &ResearchRequest,
    ) -> Result<ResearchResponse, WorkflowError> {
        // Discover available services
        let services = self.discover_research_services().await?;

        if services.is_empty() {
            return Err(WorkflowError::RegistryError {
                message: "No research services available".to_string(),
            });
        }

        let max_services = request.max_services.unwrap_or(services.len()).min(services.len());

        // Try services in order until one succeeds
        for service_name in services.iter().take(max_services) {
            match self.call_research_service(service_name, request).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    log::warn!("Research service '{}' failed: {}", service_name, e);
                    continue;
                }
            }
        }

        Err(WorkflowError::RegistryError {
            message: "All research services failed".to_string(),
        })
    }

    /// Calls a specific research service
    async fn call_research_service(
        &self,
        service_name: &str,
        request: &ResearchRequest,
    ) -> Result<ResearchResponse, WorkflowError> {
        log::info!("🔍 Calling research service: {}", service_name);

        // Prepare the request payload for the AI Tutor service
        let payload = json!({
            "concept": request.query,
            "context": request.context.clone().unwrap_or_else(|| json!({})),
            "subject": request.subject.as_deref().unwrap_or("general"),
            "difficulty_level": request.difficulty_level.as_deref().unwrap_or("intermediate")
        });

        // Call the service's explain endpoint
        let result = self
            .cross_system_client
            .call_service(service_name, "explain", payload)
            .await
            .map_err(|e| WorkflowError::ApiError {
                message: format!("Service call failed: {}", e),
            })?;

        // Parse the response into our ResearchResponse structure
        let explanation = result
            .get("explanation")
            .and_then(|v| v.as_str())
            .unwrap_or("No explanation provided")
            .to_string();

        let confidence = result
            .get("confidence")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.8);

        // Extract examples, follow-up questions, and resources if available
        let examples = result
            .get("examples")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_else(Vec::new);

        let follow_up_questions = result
            .get("follow_up_questions")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_else(Vec::new);

        let resources = result
            .get("resources")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_else(Vec::new);

        let metadata = result
            .get("metadata")
            .cloned()
            .unwrap_or_else(|| json!({}));

        Ok(ResearchResponse {
            query: request.query.clone(),
            explanation,
            confidence,
            source_service: service_name.to_string(),
            examples,
            follow_up_questions,
            resources,
            metadata,
        })
    }
}

impl Node for ResearchNode {
    fn node_name(&self) -> String {
        "Research Node".to_string()
    }

    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // For now, create a blocking wrapper around the async function
        // In a real implementation, this would be properly integrated with async workflows
        let runtime = tokio::runtime::Runtime::new().map_err(|e| {
            WorkflowError::ProcessingError {
                message: format!("Failed to create async runtime: {}", e),
            }
        })?;

        let result = runtime.block_on(async {
            // Extract research request from the task context
            let event_data: Value = context.get_event_data()?;

            let research_request = ResearchRequest {
                query: event_data
                    .get("query")
                    .or_else(|| event_data.get("concept"))
                    .or_else(|| event_data.get("topic"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("research query")
                    .to_string(),
                context: event_data.get("context").cloned(),
                subject: event_data
                    .get("subject")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                difficulty_level: event_data
                    .get("difficulty_level")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                max_services: event_data
                    .get("max_services")
                    .and_then(|v| v.as_u64())
                    .map(|n| n as usize),
            };

            // Perform the research
            self.perform_research(&research_request).await
        })?;

        // Store the research results in the task context
        context.update_node(
            "research_result",
            json!({
                "query": result.query,
                "explanation": result.explanation,
                "confidence": result.confidence,
                "source_service": result.source_service,
                "examples": result.examples,
                "follow_up_questions": result.follow_up_questions,
                "resources": result.resources,
                "metadata": result.metadata
            }),
        );

        // Add metadata about the research operation
        context
            .set_metadata("research_node_service", &result.source_service)
            .map_err(|e| WorkflowError::ProcessingError {
                message: format!("Failed to set metadata: {}", e),
            })?;

        context
            .set_metadata("research_node_confidence", result.confidence)
            .map_err(|e| WorkflowError::ProcessingError {
                message: format!("Failed to set metadata: {}", e),
            })?;

        log::info!(
            "✅ Research completed for query: '{}' using service: '{}'",
            result.query,
            result.source_service
        );

        Ok(context)
    }
}

/// Router implementation for the Research Node
/// 
/// This allows the Research Node to route to different specialized research nodes
/// based on the research subject or complexity
impl Router for ResearchNode {
    fn route(&self, context: &TaskContext) -> Option<Box<dyn Node>> {
        // For this implementation, we don't route to other nodes
        // The ResearchNode handles all research internally
        // In a more complex setup, this could route to specialized research nodes
        // based on subject matter or complexity

        let _ = context; // Suppress unused parameter warning
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::TaskContext;
    use serde_json::json;

    #[test]
    fn test_research_config_default() {
        let config = ResearchConfig::default();
        assert_eq!(config.registry_endpoint, "http://localhost:8080");
        assert_eq!(config.preferred_capability, "tutoring");
        assert_eq!(config.timeout_seconds, 30);
        assert!(config.auth_token.is_none());
    }

    #[test]
    fn test_research_request_serialization() {
        let request = ResearchRequest {
            query: "What is machine learning?".to_string(),
            context: Some(json!({"background": "computer science"})),
            subject: Some("programming".to_string()),
            difficulty_level: Some("beginner".to_string()),
            max_services: Some(3),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: ResearchRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(request.query, deserialized.query);
        assert_eq!(request.subject, deserialized.subject);
        assert_eq!(request.difficulty_level, deserialized.difficulty_level);
        assert_eq!(request.max_services, deserialized.max_services);
    }

    #[test]
    fn test_research_node_creation() {
        let config = ResearchConfig::default();
        let node = ResearchNode::new(config);

        assert_eq!(node.node_name(), "Research Node");
    }

    #[test]
    fn test_research_node_name() {
        let config = ResearchConfig::default();
        let node = ResearchNode::new(config);
        assert_eq!(node.node_name(), "Research Node");
    }

    #[tokio::test]
    async fn test_research_node_process_basic() {
        let config = ResearchConfig::default();
        let node = ResearchNode::new(config);

        let context = TaskContext::new(
            "test_research".to_string(),
            json!({
                "query": "What is artificial intelligence?",
                "subject": "computer_science"
            }),
        );

        // Note: This test will fail without a running registry and AI Tutor service
        // In production, you would use mock services for unit testing
        let result = node.process(context);
        
        // For now, we expect this to fail since we don't have services running
        // In a real test environment, you would mock the cross-system client
        assert!(result.is_err());
    }
}