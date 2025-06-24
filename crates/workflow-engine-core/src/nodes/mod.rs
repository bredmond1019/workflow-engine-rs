//! # Workflow Node System
//!
//! This module defines the core node system for AI Architecture workflows. Nodes are
//! individual processing units that can be chained together to form complex AI-powered
//! workflows. Each node receives a [`TaskContext`], processes it, and returns an
//! updated context for the next node.
//!
//! ## Core Concepts
//!
//! ### Node Trait
//! The [`Node`] trait is the foundation of all workflow processing. Every workflow
//! component must implement this trait to participate in execution.
//!
//! ### Node Types
//! - **Processing Nodes**: Perform data transformation, analysis, or business logic
//! - **Router Nodes**: Make routing decisions based on context data
//! - **Agent Nodes**: Integrate with AI services for intelligent processing
//! - **External MCP Nodes**: Connect to external MCP servers for tool access
//!
//! ### Node Registry
//! The [`registry::NodeRegistry`] manages all registered nodes and provides
//! type-safe access during workflow execution.
//!
//! ## Usage Examples
//!
//! ### Basic Node Implementation
//!
//! ```rust
//! use ai_architecture_core::{
//!     nodes::Node,
//!     task::TaskContext,
//!     error::WorkflowError,
//! };
//! use serde_json::json;
//!
//! #[derive(Debug)]
//! struct TextProcessorNode {
//!     transform_type: String,
//! }
//!
//! impl TextProcessorNode {
//!     fn new(transform_type: String) -> Self {
//!         Self { transform_type }
//!     }
//! }
//!
//! impl Node for TextProcessorNode {
//!     fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
//!         // Extract input text from event data
//!         let input: serde_json::Value = context.get_event_data()?;
//!         let text = input.get("text")
//!             .and_then(|v| v.as_str())
//!             .ok_or_else(|| WorkflowError::ValidationError {
//!                 message: "Missing 'text' field in input".to_string()
//!             })?;
//!
//!         // Apply transformation based on type
//!         let transformed_text = match self.transform_type.as_str() {
//!             "uppercase" => text.to_uppercase(),
//!             "lowercase" => text.to_lowercase(),
//!             "word_count" => text.split_whitespace().count().to_string(),
//!             _ => text.to_string(),
//!         };
//!
//!         // Store result in context
//!         context.update_node(&format!("{}_result", self.transform_type), json!({
//!             "original_text": text,
//!             "transformed_text": transformed_text,
//!             "transformation": &self.transform_type
//!         }));
//!
//!         Ok(context)
//!     }
//! }
//!
//! // Usage in workflow
//! let processor = TextProcessorNode::new("uppercase".to_string());
//! let context = TaskContext::new(
//!     "text_processing".to_string(),
//!     json!({"text": "hello world"})
//! );
//! let result = processor.process(context)?;
//! ```
//!
//! ### Router Node Implementation
//!
//! ```rust
//! use ai_architecture_core::{
//!     nodes::{Node, Router},
//!     task::TaskContext,
//!     error::WorkflowError,
//! };
//! use serde_json::json;
//!
//! #[derive(Debug)]
//! struct PriorityRouter;
//!
//! impl Node for PriorityRouter {
//!     fn process(&self, context: TaskContext) -> Result<TaskContext, WorkflowError> {
//!         // Routers typically don't modify context, just determine routing
//!         Ok(context)
//!     }
//! }
//!
//! impl Router for PriorityRouter {
//!     fn route(&self, context: &TaskContext) -> Option<Box<dyn Node>> {
//!         let input: Result<serde_json::Value, _> = context.get_event_data();
//!         if let Ok(data) = input {
//!             if let Some(priority) = data.get("priority").and_then(|v| v.as_str()) {
//!                 return match priority {
//!                     "high" => Some(Box::new(HighPriorityProcessor)),
//!                     "medium" => Some(Box::new(MediumPriorityProcessor)),
//!                     "low" => Some(Box::new(LowPriorityProcessor)),
//!                     _ => Some(Box::new(DefaultProcessor)),
//!                 };
//!             }
//!         }
//!         Some(Box::new(DefaultProcessor))
//!     }
//! }
//! ```
//!
//! ### Parallel Processing Node
//!
//! ```rust
//! use ai_architecture_core::{
//!     nodes::{Node, ParallelNode},
//!     task::TaskContext,
//!     error::WorkflowError,
//! };
//! use serde_json::json;
//!
//! #[derive(Debug)]
//! struct DataAnalysisNode;
//!
//! impl Node for DataAnalysisNode {
//!     fn process(&self, context: TaskContext) -> Result<TaskContext, WorkflowError> {
//!         // This could be called in parallel with other analysis nodes
//!         self.execute_parallel(context)
//!             .map(|results| results.into_iter().next().unwrap_or_else(|| context.clone()))
//!     }
//! }
//!
//! impl ParallelNode for DataAnalysisNode {
//!     fn execute_parallel(
//!         &self,
//!         mut context: TaskContext,
//!     ) -> Result<Vec<TaskContext>, WorkflowError> {
//!         let input: serde_json::Value = context.get_event_data()?;
//!         let data = input.get("data").and_then(|v| v.as_array())
//!             .ok_or_else(|| WorkflowError::ValidationError {
//!                 message: "Expected 'data' array in input".to_string()
//!             })?;
//!
//!         // Perform different analyses on the data
//!         let total: f64 = data.iter()
//!             .filter_map(|v| v.as_f64())
//!             .sum();
//!         
//!         let count = data.len();
//!         let average = if count > 0 { total / count as f64 } else { 0.0 };
//!         
//!         let max = data.iter()
//!             .filter_map(|v| v.as_f64())
//!             .fold(f64::NEG_INFINITY, f64::max);
//!             
//!         let min = data.iter()
//!             .filter_map(|v| v.as_f64())
//!             .fold(f64::INFINITY, f64::min);
//!
//!         context.update_node("statistical_analysis", json!({
//!             "total": total,
//!             "count": count,
//!             "average": average,
//!             "max": max,
//!             "min": min
//!         }));
//!
//!         Ok(vec![context])
//!     }
//! }
//! ```
//!
//! ### Node Registry Usage
//!
//! ```rust
//! use ai_architecture_core::{
//!     nodes::registry::NodeRegistry,
//!     workflow::Workflow,
//! };
//! use std::any::TypeId;
//!
//! // Create registry and register nodes
//! let mut registry = NodeRegistry::new();
//! registry.register(TextProcessorNode::new("uppercase".to_string()));
//! registry.register(PriorityRouter);
//! registry.register(DataAnalysisNode);
//!
//! // Registry provides type-safe access
//! let text_processor_id = TypeId::of::<TextProcessorNode>();
//! if let Some(node) = registry.get(&text_processor_id) {
//!     let context = TaskContext::new(
//!         "test".to_string(),
//!         json!({"text": "hello"})
//!     );
//!     let result = node.process(context)?;
//!     println!("Processed: {:?}", result);
//! }
//! ```
//!
//! ### Agent Node Configuration
//!
//! ```rust
//! use ai_architecture_core::{
//!     nodes::agent::{AgentConfig, ModelProvider},
//!     ai_agents::anthropic::AnthropicAgentNode,
//! };
//!
//! // Configure AI agent node
//! let agent_config = AgentConfig {
//!     system_prompt: "You are a helpful AI assistant.".to_string(),
//!     model_provider: ModelProvider::Anthropic,
//!     model_name: "claude-3-sonnet-20240229".to_string(),
//!     mcp_server_uri: Some("ws://localhost:8080/mcp".to_string()),
//! };
//!
//! let agent_node = AnthropicAgentNode::new(agent_config);
//!
//! // Agent nodes implement the Node trait and can be used in workflows
//! ```
//!
//! ### External MCP Client Node
//!
//! ```rust
//! use ai_architecture_core::{
//!     nodes::external_mcp_client::ExternalMCPClientNode,
//!     mcp::transport::TransportType,
//! };
//!
//! // Create node that connects to external MCP server
//! let mcp_node = ExternalMCPClientNode::new(
//!     "external-tools".to_string(),
//!     TransportType::WebSocket {
//!         url: "ws://localhost:8080/external-mcp".to_string(),
//!     },
//!     vec!["analyze_data".to_string(), "generate_report".to_string()],
//! );
//!
//! // This node can call external MCP tools during processing
//! ```
//!
//! ## Node Configuration
//!
//! ### Basic Configuration
//!
//! ```rust
//! use ai_architecture_core::nodes::config::NodeConfig;
//! use serde_json::json;
//!
//! let node_config = NodeConfig {
//!     name: "data_validator".to_string(),
//!     description: "Validates input data against schema".to_string(),
//!     enabled: true,
//!     timeout_seconds: 30,
//!     retry_attempts: 3,
//!     config_data: json!({
//!         "schema_version": "1.0",
//!         "strict_validation": true,
//!         "required_fields": ["id", "name", "email"]
//!     }),
//! };
//! ```
//!
//! ### External Configuration
//!
//! ```rust
//! use ai_architecture_core::nodes::external_config::ExternalNodeConfig;
//! use std::collections::HashMap;
//!
//! let mut env_vars = HashMap::new();
//! env_vars.insert("API_KEY".to_string(), "secret-key".to_string());
//! env_vars.insert("BASE_URL".to_string(), "https://api.example.com".to_string());
//!
//! let external_config = ExternalNodeConfig {
//!     service_name: "external-api".to_string(),
//!     endpoint_url: "https://api.example.com/process".to_string(),
//!     authentication_method: "bearer".to_string(),
//!     timeout_ms: 5000,
//!     environment_variables: env_vars,
//!     headers: HashMap::new(),
//! };
//! ```
//!
//! ## Error Handling in Nodes
//!
//! ```rust
//! use ai_architecture_core::{
//!     nodes::Node,
//!     task::TaskContext,
//!     error::WorkflowError,
//! };
//!
//! #[derive(Debug)]
//! struct RobustProcessorNode;
//!
//! impl Node for RobustProcessorNode {
//!     fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
//!         // Validate input data
//!         let input = match context.get_event_data::<serde_json::Value>() {
//!             Ok(data) => data,
//!             Err(_) => {
//!                 return Err(WorkflowError::ValidationError {
//!                     message: "Invalid input data format".to_string()
//!                 });
//!             }
//!         };
//!
//!         // Check required fields
//!         let required_field = input.get("required_value")
//!             .ok_or_else(|| WorkflowError::ValidationError {
//!                 message: "Missing required_value field".to_string()
//!             })?;
//!
//!         // Process with error handling
//!         match self.perform_processing(required_field) {
//!             Ok(result) => {
//!                 context.update_node("processing_result", result);
//!                 context.set_metadata("processing_status", "success")?;
//!                 Ok(context)
//!             }
//!             Err(processing_error) => {
//!                 Err(WorkflowError::ProcessingError {
//!                     message: format!("Processing failed: {}", processing_error)
//!                 })
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! ## Best Practices
//!
//! ### Node Design
//! 1. **Single Responsibility**: Each node should have one clear purpose
//! 2. **Immutable Input**: Don't modify the original event data
//! 3. **Clear Naming**: Use descriptive names for node results
//! 4. **Error Handling**: Always handle potential failures gracefully
//! 5. **Validation**: Validate inputs and provide clear error messages
//!
//! ### Performance
//! 1. **Minimize Cloning**: Avoid unnecessary data cloning in processing
//! 2. **Async Operations**: Use async patterns for I/O operations when possible
//! 3. **Resource Cleanup**: Clean up resources in long-running nodes
//! 4. **Caching**: Cache expensive computations when appropriate
//!
//! ### Testing
//! 1. **Unit Tests**: Test nodes in isolation with mock data
//! 2. **Integration Tests**: Test nodes within workflow contexts
//! 3. **Error Cases**: Test error handling and edge cases
//! 4. **Performance Tests**: Validate performance with realistic data sizes
//!
//! ## Thread Safety
//!
//! All node implementations must be thread-safe:
//! - Nodes are `Send + Sync` and can be shared across threads
//! - Task contexts are cloned for parallel processing
//! - Registry access is protected by `RwLock`
//! - External resources should use appropriate synchronization
//!
//! ## Advanced Features
//!
//! ### Dynamic Node Loading
//! Nodes can be loaded dynamically based on configuration:
//!
//! ```rust
//! use ai_architecture_core::nodes::registry::NodeRegistry;
//!
//! // Register nodes based on runtime configuration
//! let mut registry = NodeRegistry::new();
//!
//! if config.enable_ai_processing {
//!     registry.register(AnthropicAgentNode::new(ai_config));
//! }
//!
//! if config.enable_external_tools {
//!     registry.register(ExternalMCPClientNode::new(
//!         "external-tools".to_string(),
//!         transport_config,
//!         tool_names,
//!     ));
//! }
//! ```
//!
//! ### Custom Node Metadata
//! Nodes can provide metadata for better workflow visualization:
//!
//! ```rust
//! impl Node for CustomNode {
//!     fn node_name(&self) -> String {
//!         "Custom Data Processor v2.1".to_string()
//!     }
//!
//!     fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
//!         // Add processing metadata
//!         context.set_metadata("node_version", "2.1.0")?;
//!         context.set_metadata("processing_start", chrono::Utc::now())?;
//!         
//!         // ... perform processing ...
//!         
//!         context.set_metadata("processing_duration_ms", duration.as_millis())?;
//!         Ok(context)
//!     }
//! }
//! ```

use std::fmt::Debug;
use async_trait::async_trait;

use super::error::WorkflowError;
use super::task::TaskContext;

pub mod agent;
pub mod config;
pub mod config_builder;
pub mod registry;
pub mod template_agent;
pub mod type_safe;
// pub mod research; // Temporarily disabled to test other modules
// External MCP modules moved to workflow-engine-nodes crate to avoid circular dependency

/// Base trait for all workflow nodes.
///
/// The `Node` trait is the foundation of the AI Architecture workflow system.
/// Every processing component must implement this trait to participate in
/// workflow execution. Nodes receive a [`TaskContext`], process it according
/// to their specific logic, and return an updated context.
///
/// # Requirements
///
/// Implementations must be:
/// - `Send + Sync`: Safe for concurrent access across threads
/// - `Debug`: Provide debug output for troubleshooting
/// - Stateless or internally synchronized for thread safety
///
/// # Examples
///
/// ```rust
/// use ai_architecture_core::{nodes::Node, task::TaskContext, error::WorkflowError};
/// use serde_json::json;
///
/// #[derive(Debug)]
/// struct GreetingNode {
///     greeting_prefix: String,
/// }
///
/// impl Node for GreetingNode {
///     fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
///         let input: serde_json::Value = context.get_event_data()?;
///         let name = input.get("name").and_then(|v| v.as_str()).unwrap_or("World");
///         
///         context.update_node("greeting", json!({
///             "message": format!("{} {}!", self.greeting_prefix, name)
///         }));
///         
///         Ok(context)
///     }
/// }
/// ```
pub trait Node: Send + Sync + Debug {
    /// Returns a human-readable name for this node.
    ///
    /// The default implementation extracts the type name, but nodes can
    /// override this to provide more descriptive names.
    ///
    /// # Examples
    ///
    /// ```rust
    /// impl Node for MyCustomNode {
    ///     fn node_name(&self) -> String {
    ///         "Custom Data Processor v1.2".to_string()
    ///     }
    ///     
    ///     // ... rest of implementation
    /// }
    /// ```
    fn node_name(&self) -> String {
        std::any::type_name::<Self>()
            .split("::")
            .last()
            .unwrap_or("UnknownNode")
            .to_string()
    }

    /// Processes the task context and returns an updated context.
    ///
    /// This is the core method that defines what the node does. It receives
    /// the current workflow state via [`TaskContext`] and must return an
    /// updated context with any results or modifications.
    ///
    /// # Arguments
    ///
    /// * `task_context` - The current workflow state containing event data,
    ///   previous node results, and metadata
    ///
    /// # Returns
    ///
    /// * `Ok(TaskContext)` - Updated context with this node's results
    /// * `Err(WorkflowError)` - Processing error with details
    ///
    /// # Examples
    ///
    /// ```rust
    /// fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
    ///     // Extract input data
    ///     let input: MyInputType = context.get_event_data()?;
    ///     
    ///     // Perform processing
    ///     let result = self.perform_analysis(&input)?;
    ///     
    ///     // Store results
    ///     context.update_node("analysis_result", result);
    ///     context.set_metadata("processing_time", start_time.elapsed())?;
    ///     
    ///     Ok(context)
    /// }
    /// ```
    fn process(&self, task_context: TaskContext) -> Result<TaskContext, WorkflowError>;
}

/// Trait for nodes that determine routing in workflows.
///
/// This trait is used by workflow engines to determine which node
/// should be executed next based on the current task context.
/// 
/// Note: This trait is being phased out in favor of the [`Router`] trait.
pub trait RouterNode: Send + Sync + Debug {
    /// Determines the next node to execute based on task context.
    ///
    /// # Arguments
    ///
    /// * `task_context` - Current workflow state
    ///
    /// # Returns
    ///
    /// * `Some(Box<dyn Node>)` - Next node to execute
    /// * `None` - No next node (end of workflow)
    fn determine_next_node(&self, task_context: &TaskContext) -> Option<Box<dyn Node>>;
}

/// Router trait for nodes that make routing decisions.
///
/// Router nodes implement conditional logic to determine the next node
/// in a workflow based on the current task context. This enables
/// dynamic workflow paths based on data or processing results.
///
/// # Examples
///
/// ```rust
/// use ai_architecture_core::{nodes::{Node, Router}, task::TaskContext, error::WorkflowError};
///
/// #[derive(Debug)]
/// struct ConditionalRouter;
///
/// impl Node for ConditionalRouter {
///     fn process(&self, context: TaskContext) -> Result<TaskContext, WorkflowError> {
///         // Routers typically don't modify the context
///         Ok(context)
///     }
/// }
///
/// impl Router for ConditionalRouter {
///     fn route(&self, context: &TaskContext) -> Option<Box<dyn Node>> {
///         let input: Result<serde_json::Value, _> = context.get_event_data();
///         if let Ok(data) = input {
///             if let Some(condition) = data.get("route").and_then(|v| v.as_str()) {
///                 return match condition {
///                     "path_a" => Some(Box::new(ProcessorA)),
///                     "path_b" => Some(Box::new(ProcessorB)),
///                     _ => Some(Box::new(DefaultProcessor)),
///                 };
///             }
///         }
///         None
///     }
/// }
/// ```
pub trait Router: Node {
    /// Determines the next node based on routing logic.
    ///
    /// # Arguments
    ///
    /// * `task_context` - Current workflow state to base routing decision on
    ///
    /// # Returns
    ///
    /// * `Some(Box<dyn Node>)` - Next node to execute
    /// * `None` - No next node (end of workflow path)
    fn route(&self, task_context: &TaskContext) -> Option<Box<dyn Node>>;
}

/// Parallel execution trait for nodes that support concurrent processing.
///
/// Nodes implementing this trait can be executed in parallel with other
/// nodes, potentially improving workflow performance for independent
/// operations.
///
/// # Examples
///
/// ```rust
/// use ai_architecture_core::{
///     nodes::{Node, ParallelNode},
///     task::TaskContext,
///     error::WorkflowError,
/// };
///
/// #[derive(Debug)]
/// struct DataAnalysisNode;
///
/// impl Node for DataAnalysisNode {
///     fn process(&self, context: TaskContext) -> Result<TaskContext, WorkflowError> {
///         // Delegate to parallel execution
///         self.execute_parallel(context)
///             .map(|results| results.into_iter().next().unwrap_or(context))
///     }
/// }
///
/// impl ParallelNode for DataAnalysisNode {
///     fn execute_parallel(
///         &self,
///         mut context: TaskContext,
///     ) -> Result<Vec<TaskContext>, WorkflowError> {
///         // Perform analysis that could be parallelized
///         let input: serde_json::Value = context.get_event_data()?;
///         
///         // Process data in parallel chunks
///         let results = self.analyze_data_chunks(&input)?;
///         
///         context.update_node("parallel_results", results);
///         Ok(vec![context])
///     }
/// }
/// ```
pub trait ParallelNode: Node {
    /// Executes the node logic with support for parallel processing.
    ///
    /// This method can create multiple task contexts for parallel processing
    /// or return a single context if parallel execution isn't beneficial.
    ///
    /// # Arguments
    ///
    /// * `task_context` - Input context for processing
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<TaskContext>)` - One or more result contexts
    /// * `Err(WorkflowError)` - Processing error
    fn execute_parallel(
        &self,
        task_context: TaskContext,
    ) -> Result<Vec<TaskContext>, WorkflowError>;
}

/// Async node trait for asynchronous workflow processing.
///
/// This trait represents the future direction of the workflow engine.
/// All nodes should implement this trait for better performance and
/// scalability. The synchronous [`Node`] trait will be maintained for
/// backward compatibility.
///
/// # Requirements
///
/// Implementations must be:
/// - `Send + Sync`: Safe for concurrent access across threads
/// - `Debug`: Provide debug output for troubleshooting
/// - Stateless or internally synchronized for thread safety
///
/// # Examples
///
/// ```rust
/// use workflow_engine_core::{
///     nodes::AsyncNode, 
///     task::TaskContext, 
///     error::WorkflowError
/// };
/// use async_trait::async_trait;
/// use serde_json::json;
///
/// #[derive(Debug)]
/// struct AsyncGreetingNode {
///     greeting_prefix: String,
/// }
///
/// #[async_trait]
/// impl AsyncNode for AsyncGreetingNode {
///     async fn process_async(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
///         let input: serde_json::Value = context.get_event_data()?;
///         let name = input.get("name").and_then(|v| v.as_str()).unwrap_or("World");
///         
///         // Simulate async work (e.g., API call)
///         tokio::time::sleep(std::time::Duration::from_millis(10)).await;
///         
///         context.update_node("greeting", json!({
///             "message": format!("{} {}!", self.greeting_prefix, name)
///         }));
///         
///         Ok(context)
///     }
/// }
/// ```
#[async_trait]
pub trait AsyncNode: Send + Sync + Debug {
    /// Returns a human-readable name for this node.
    ///
    /// The default implementation extracts the type name, but nodes can
    /// override this to provide more descriptive names.
    fn node_name(&self) -> String {
        std::any::type_name::<Self>()
            .split("::")
            .last()
            .unwrap_or("UnknownAsyncNode")
            .to_string()
    }

    /// Asynchronously processes the task context and returns an updated context.
    ///
    /// This is the core method that defines what the async node does. It receives
    /// the current workflow state via [`TaskContext`] and must return an
    /// updated context with any results or modifications.
    ///
    /// # Arguments
    ///
    /// * `task_context` - The current workflow state containing event data,
    ///   previous node results, and metadata
    ///
    /// # Returns
    ///
    /// * `Ok(TaskContext)` - Updated context with this node's results
    /// * `Err(WorkflowError)` - Processing error with details
    ///
    /// # Examples
    ///
    /// ```rust
    /// async fn process_async(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
    ///     // Extract input data
    ///     let input: MyInputType = context.get_event_data()?;
    ///     
    ///     // Perform async processing (e.g., API call, database query)
    ///     let result = self.perform_async_analysis(&input).await?;
    ///     
    ///     // Store results
    ///     context.update_node("async_analysis_result", result);
    ///     context.set_metadata("processing_time", start_time.elapsed())?;
    ///     
    ///     Ok(context)
    /// }
    /// ```
    async fn process_async(&self, task_context: TaskContext) -> Result<TaskContext, WorkflowError>;
}

/// Adapter to make synchronous nodes work in async contexts
///
/// This provides backward compatibility by wrapping synchronous [`Node`]
/// implementations to work with async workflow executors.
///
/// # Examples
///
/// ```rust
/// use workflow_engine_core::nodes::{Node, AsyncNodeAdapter};
///
/// // Existing sync node
/// #[derive(Debug)]
/// struct SyncNode;
/// 
/// impl Node for SyncNode {
///     fn process(&self, context: TaskContext) -> Result<TaskContext, WorkflowError> {
///         // Synchronous processing
///         Ok(context)
///     }
/// }
///
/// // Use in async context
/// let sync_node = SyncNode;
/// let async_adapter = AsyncNodeAdapter::new(sync_node);
/// ```
#[derive(Debug)]
pub struct AsyncNodeAdapter<N: Node + Clone + 'static> {
    inner: N,
}

impl<N: Node + Clone + 'static> AsyncNodeAdapter<N> {
    /// Creates a new adapter wrapping a synchronous node
    pub fn new(node: N) -> Self {
        Self { inner: node }
    }
    
    /// Get a reference to the inner node
    pub fn inner(&self) -> &N {
        &self.inner
    }
    
    /// Consume the adapter and return the inner node
    pub fn into_inner(self) -> N {
        self.inner
    }
}

#[async_trait]
impl<N: Node + Clone + 'static> AsyncNode for AsyncNodeAdapter<N> {
    fn node_name(&self) -> String {
        format!("AsyncAdapter({})", self.inner.node_name())
    }

    async fn process_async(&self, task_context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Run the synchronous process method in a blocking task
        let inner = self.inner.clone();
        tokio::task::spawn_blocking(move || inner.process(task_context))
            .await
            .map_err(|e| WorkflowError::processing_error(
                format!("Async adapter task failed: {}", e),
                "async_adapter"
            ))?
    }
}
