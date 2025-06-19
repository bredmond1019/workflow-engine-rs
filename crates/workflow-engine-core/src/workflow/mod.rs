//! # Workflow Execution Engine
//!
//! This module provides the core workflow execution engine for AI Architecture. Workflows
//! are directed graphs of processing nodes that define how data flows through AI-powered
//! tasks. The engine handles node execution, routing, parallel processing, and MCP
//! integration for building complex AI applications.
//!
//! ## Core Components
//!
//! ### Workflow
//! The [`Workflow`] struct is the main execution engine that orchestrates node processing
//! according to a defined schema. It manages node registration, execution flow, and
//! integration with external services.
//!
//! ### Schema
//! [`schema::WorkflowSchema`] defines the structure of a workflow - which nodes to execute,
//! how they connect, and any parallel processing configurations.
//!
//! ### Builder
//! [`builder::WorkflowBuilder`] provides a fluent interface for constructing workflows
//! with type safety and validation.
//!
//! ### Validator
//! [`validator::WorkflowValidator`] ensures workflow schemas are valid before execution,
//! checking for cycles, unreachable nodes, and proper routing configuration.
//!
//! ## Usage Examples
//!
//! ### Basic Workflow Creation
//!
//! ```rust
//! use ai_architecture_core::{
//!     workflow::{builder::WorkflowBuilder, Workflow},
//!     nodes::Node,
//!     task::TaskContext,
//!     error::WorkflowError,
//! };
//! use serde_json::json;
//!
//! // Define processing nodes
//! #[derive(Debug)]
//! struct ValidationNode;
//!
//! impl Node for ValidationNode {
//!     fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
//!         let input: serde_json::Value = context.get_event_data()?;
//!         let is_valid = input.get("data").is_some();
//!         
//!         context.update_node("validation", json!({
//!             "valid": is_valid,
//!             "checked_at": chrono::Utc::now()
//!         }));
//!         
//!         Ok(context)
//!     }
//! }
//!
//! #[derive(Debug)]
//! struct ProcessingNode;
//!
//! impl Node for ProcessingNode {
//!     fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
//!         let input: serde_json::Value = context.get_event_data()?;
//!         
//!         context.update_node("processing", json!({
//!             "result": "processed data",
//!             "timestamp": chrono::Utc::now()
//!         }));
//!         
//!         Ok(context)
//!     }
//! }
//!
//! // Build workflow
//! let workflow = WorkflowBuilder::new("data_processing_workflow")
//!     .start_with::<ValidationNode>()
//!     .then::<ProcessingNode>()
//!     .build()?;
//!
//! // Register nodes
//! workflow.register_node(ValidationNode);
//! workflow.register_node(ProcessingNode);
//!
//! // Execute workflow
//! let result = workflow.run(json!({
//!     "data": {"id": 1, "name": "test"},
//!     "user_id": "user123"
//! }))?;
//!
//! println!("Workflow completed: {:?}", result);
//! ```
//!
//! ### Parallel Processing Workflow
//!
//! ```rust
//! use ai_architecture_core::{
//!     workflow::builder::WorkflowBuilder,
//!     nodes::Node,
//! };
//! use std::any::TypeId;
//!
//! // Build workflow with parallel processing
//! let workflow = WorkflowBuilder::new("parallel_processing_workflow")
//!     .start_with::<DataIngestionNode>()
//!     .parallel(&[
//!         TypeId::of::<ValidationProcessor>(),
//!         TypeId::of::<TransformationProcessor>(),
//!         TypeId::of::<EnrichmentProcessor>(),
//!     ])
//!     .then::<AggregationNode>()
//!     .then::<OutputNode>()
//!     .build()?;
//!
//! // Register all nodes
//! workflow.register_node(DataIngestionNode);
//! workflow.register_node(ValidationProcessor);
//! workflow.register_node(TransformationProcessor);
//! workflow.register_node(EnrichmentProcessor);
//! workflow.register_node(AggregationNode);
//! workflow.register_node(OutputNode);
//!
//! // Execute with parallel processing
//! let result = workflow.run(json!({
//!     "input_data": [1, 2, 3, 4, 5],
//!     "processing_options": {"parallel": true}
//! }))?;
//! ```
//!
//! ### Router-based Conditional Workflows
//!
//! ```rust
//! use ai_architecture_core::{
//!     workflow::builder::WorkflowBuilder,
//!     nodes::{Node, Router},
//!     task::TaskContext,
//!     error::WorkflowError,
//! };
//!
//! #[derive(Debug)]
//! struct PriorityRouter;
//!
//! impl Node for PriorityRouter {
//!     fn process(&self, context: TaskContext) -> Result<TaskContext, WorkflowError> {
//!         Ok(context) // Routers don't modify context
//!     }
//! }
//!
//! impl Router for PriorityRouter {
//!     fn route(&self, context: &TaskContext) -> Option<Box<dyn Node>> {
//!         let input: Result<serde_json::Value, _> = context.get_event_data();
//!         if let Ok(data) = input {
//!             if let Some(priority) = data.get("priority").and_then(|v| v.as_str()) {
//!                 return match priority {
//!                     "high" => Some(Box::new(UrgentProcessor)),
//!                     "medium" => Some(Box::new(StandardProcessor)),
//!                     "low" => Some(Box::new(BatchProcessor)),
//!                     _ => Some(Box::new(DefaultProcessor)),
//!                 };
//!             }
//!         }
//!         None
//!     }
//! }
//!
//! // Build conditional workflow
//! let workflow = WorkflowBuilder::new("priority_routing_workflow")
//!     .start_with::<RequestValidator>()
//!     .route::<PriorityRouter>(&[
//!         TypeId::of::<UrgentProcessor>(),
//!         TypeId::of::<StandardProcessor>(),
//!         TypeId::of::<BatchProcessor>(),
//!     ])
//!     .then::<ResponseFormatter>()
//!     .build()?;
//! ```
//!
//! ### AI Agent Integration
//!
//! ```rust
//! use ai_architecture_core::{
//!     workflow::builder::WorkflowBuilder,
//!     ai_agents::anthropic::AnthropicAgentNode,
//!     nodes::agent::{AgentConfig, ModelProvider},
//! };
//!
//! // Configure AI agent
//! let agent_config = AgentConfig {
//!     system_prompt: "You are an AI assistant that analyzes customer feedback.".to_string(),
//!     model_provider: ModelProvider::Anthropic,
//!     model_name: "claude-3-sonnet-20240229".to_string(),
//!     mcp_server_uri: None,
//! };
//!
//! // Build AI-enhanced workflow
//! let workflow = WorkflowBuilder::new("ai_feedback_analysis")
//!     .start_with::<FeedbackValidator>()
//!     .then::<AnthropicAgentNode>()
//!     .then::<SentimentAggregator>()
//!     .then::<ReportGenerator>()
//!     .build()?;
//!
//! workflow.register_node(FeedbackValidator);
//! workflow.register_node(AnthropicAgentNode::new(agent_config));
//! workflow.register_node(SentimentAggregator);
//! workflow.register_node(ReportGenerator);
//!
//! // Process customer feedback
//! let result = workflow.run(json!({
//!     "feedback": "The product is amazing but delivery was slow",
//!     "customer_id": "CUST-123",
//!     "product_id": "PROD-456"
//! }))?;
//! ```
//!
//! ### MCP Server Integration
//!
//! ```rust
//! use ai_architecture_core::{
//!     workflow::Workflow,
//!     mcp::{transport::TransportType, server::MCPToolServer},
//! };
//!
//! #[tokio::main]
//! async fn main() -> Result<(), WorkflowError> {
//!     // Create workflow
//!     let workflow = create_data_processing_workflow()?;
//!
//!     // Expose workflow as MCP server
//!     let mcp_server = workflow
//!         .expose_as_mcp_server("data-processor", "1.0.0")
//!         .await?;
//!
//!     println!("Workflow exposed as MCP server with {} tools", 
//!         mcp_server.get_tool_count().await);
//!
//!     // Register external MCP servers for enhanced capabilities
//!     workflow.register_mcp_server(
//!         "ws://localhost:8080/external-tools",
//!         TransportType::WebSocket {
//!             url: "ws://localhost:8080/external-tools".to_string(),
//!         },
//!     ).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Database Event Processing
//!
//! ```rust
//! use ai_architecture_core::{
//!     workflow::Workflow,
//!     db::event::Event,
//!     workflows::WorkflowRunner,
//! };
//! use diesel::prelude::*;
//!
//! fn process_database_events(
//!     workflow: &Workflow,
//!     events: Vec<Event>,
//!     conn: &mut PgConnection,
//! ) -> Result<Vec<Event>, WorkflowError> {
//!     let runner = WorkflowRunner::new(workflow.clone());
//!     let mut results = Vec::new();
//!
//!     for event in events {
//!         match runner.process_event(&event, conn) {
//!             Ok(processed_event) => {
//!                 println!("Processed event: {}", processed_event.id);
//!                 results.push(processed_event);
//!             }
//!             Err(e) => {
//!                 eprintln!("Failed to process event {}: {}", event.id, e);
//!                 // Continue processing other events
//!             }
//!         }
//!     }
//!
//!     Ok(results)
//! }
//! ```
//!
//! ## Schema Definition
//!
//! ### Basic Schema
//!
//! ```rust
//! use ai_architecture_core::workflow::schema::{WorkflowSchema, NodeConfig};
//! use std::any::TypeId;
//!
//! let schema = WorkflowSchema {
//!     workflow_type: "content_moderation".to_string(),
//!     start: TypeId::of::<ContentValidator>(),
//!     nodes: vec![
//!         NodeConfig {
//!             node_type: TypeId::of::<ContentValidator>(),
//!             connections: vec![TypeId::of::<ModerationAgent>()],
//!             parallel_nodes: vec![],
//!             is_router: false,
//!         },
//!         NodeConfig {
//!             node_type: TypeId::of::<ModerationAgent>(),
//!             connections: vec![TypeId::of::<DecisionMaker>()],
//!             parallel_nodes: vec![],
//!             is_router: false,
//!         },
//!         NodeConfig {
//!             node_type: TypeId::of::<DecisionMaker>(),
//!             connections: vec![],
//!             parallel_nodes: vec![],
//!             is_router: false,
//!         },
//!     ],
//! };
//!
//! let workflow = Workflow::new(schema)?;
//! ```
//!
//! ### Advanced Schema with Routing
//!
//! ```rust
//! use ai_architecture_core::workflow::schema::{WorkflowSchema, NodeConfig};
//!
//! let schema = WorkflowSchema {
//!     workflow_type: "adaptive_processing".to_string(),
//!     start: TypeId::of::<InputAnalyzer>(),
//!     nodes: vec![
//!         NodeConfig {
//!             node_type: TypeId::of::<InputAnalyzer>(),
//!             connections: vec![TypeId::of::<AdaptiveRouter>()],
//!             parallel_nodes: vec![],
//!             is_router: false,
//!         },
//!         NodeConfig {
//!             node_type: TypeId::of::<AdaptiveRouter>(),
//!             connections: vec![
//!                 TypeId::of::<FastProcessor>(),
//!                 TypeId::of::<AccurateProcessor>(),
//!                 TypeId::of::<HybridProcessor>(),
//!             ],
//!             parallel_nodes: vec![],
//!             is_router: true, // Mark as router for multiple connections
//!         },
//!         // ... processor node configs
//!     ],
//! };
//! ```
//!
//! ## Validation
//!
//! ```rust
//! use ai_architecture_core::workflow::{
//!     schema::WorkflowSchema,
//!     validator::WorkflowValidator,
//! };
//!
//! // Validate workflow before execution
//! let validator = WorkflowValidator::new(&schema);
//!
//! match validator.validate() {
//!     Ok(_) => {
//!         println!("Workflow schema is valid");
//!         let workflow = Workflow::new(schema)?;
//!     }
//!     Err(WorkflowError::CycleDetected) => {
//!         eprintln!("Workflow contains cycles");
//!     }
//!     Err(WorkflowError::UnreachableNodes { nodes }) => {
//!         eprintln!("Unreachable nodes: {:?}", nodes);
//!     }
//!     Err(WorkflowError::InvalidRouter { node }) => {
//!         eprintln!("Invalid router configuration: {}", node);
//!     }
//!     Err(e) => eprintln!("Validation error: {}", e),
//! }
//! ```
//!
//! ## Error Handling
//!
//! ```rust
//! use ai_architecture_core::{workflow::Workflow, error::WorkflowError};
//!
//! fn robust_workflow_execution(
//!     workflow: &Workflow,
//!     input_data: serde_json::Value,
//! ) -> Result<serde_json::Value, WorkflowError> {
//!     match workflow.run(input_data) {
//!         Ok(context) => {
//!             // Extract final results
//!             Ok(serde_json::to_value(context.get_all_data())?)
//!         }
//!         Err(WorkflowError::NodeNotFound { node_type }) => {
//!             eprintln!("Missing node registration: {:?}", node_type);
//!             Err(WorkflowError::ProcessingError {
//!                 message: "Required node not registered".to_string()
//!             })
//!         }
//!         Err(WorkflowError::ValidationError { message }) => {
//!             eprintln!("Input validation failed: {}", message);
//!             Err(WorkflowError::ValidationError { message })
//!         }
//!         Err(e) => {
//!             eprintln!("Workflow execution failed: {}", e);
//!             Err(e)
//!         }
//!     }
//! }
//! ```
//!
//! ## Performance Considerations
//!
//! ### Parallel Execution
//! - Use parallel nodes for independent operations
//! - Monitor memory usage with large parallel contexts
//! - Consider CPU core count when designing parallel workflows
//!
//! ### Node Registry
//! - Registry uses `RwLock` for thread-safe concurrent access
//! - Reading is optimized for high concurrency
//! - Consider node registration order for better cache locality
//!
//! ### Memory Management
//! - Task contexts clone data for parallel processing
//! - Large datasets should use streaming patterns
//! - Clean up intermediate results when possible
//!
//! ## Thread Safety
//!
//! All workflow components are designed for concurrent execution:
//! - [`Workflow`] instances can be shared across threads
//! - Node registry protects concurrent access with `RwLock`
//! - Task contexts are cloned for parallel node execution
//! - External MCP connections handle concurrent requests
//!
//! ## Best Practices
//!
//! 1. **Workflow Design**: Keep workflows focused and avoid overly complex routing
//! 2. **Node Registration**: Register all nodes before workflow execution
//! 3. **Error Handling**: Implement proper error handling at each node
//! 4. **Testing**: Test workflows with realistic data and error conditions
//! 5. **Monitoring**: Add logging and metrics for production workflows
//! 6. **Validation**: Always validate workflow schemas before deployment

use std::{
    any::TypeId,
    sync::{Arc, RwLock},
    thread,
};

use serde_json::Value;

use schema::WorkflowSchema;
use validator::WorkflowValidator;

// use crate::db::event::Event;  // Commented out - db moved to API crate

use super::{
    error::WorkflowError,
    // mcp::server::MCPToolServer,  // Commented out to avoid circular dependency
    nodes::{Node, registry::NodeRegistry},
    task::TaskContext,
};

pub mod builder;
pub mod schema;
pub mod validator;
pub mod workflow_builder;

/// Represents a workflow with its schema and node registry.
pub struct Workflow {
    schema: WorkflowSchema,
    registry: Arc<RwLock<NodeRegistry>>,
}

impl Workflow {
    /// Creates a new `Workflow` instance from a given schema.
    ///
    /// # Arguments
    ///
    /// * `schema` - The `WorkflowSchema` defining the structure of the workflow.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the new `Workflow` instance if successful,
    /// or a `WorkflowError` if validation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_crate::{Workflow, WorkflowSchema};
    ///
    /// let schema = WorkflowSchema::new("example_workflow".to_string(), std::any::TypeId::of::<StartNode>());
    /// let workflow = Workflow::new(schema).expect("Failed to create workflow");
    /// ```
    pub fn new(schema: WorkflowSchema) -> Result<Self, WorkflowError> {
        let validator = WorkflowValidator::new(&schema);
        validator.validate()?;

        Ok(Self {
            schema,
            registry: Arc::new(RwLock::new(NodeRegistry::new())),
        })
    }

    /// Registers a node with the workflow.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to register, which must implement the `Node` trait.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_crate::{Workflow, ExampleNode};
    ///
    /// let workflow = Workflow::new(schema).expect("Failed to create workflow");
    /// workflow.register_node(ExampleNode);
    /// ```
    pub fn register_node<T: Node + 'static>(&self, node: T) {
        if let Ok(mut registry) = self.registry.write() {
            registry.register(node);
        }
    }

    // Event integration methods moved to workflow-engine-api crate to avoid circular dependency
    // Event type is defined in the API crate and should not be referenced here

    /// Runs the workflow with new data, creating a new Event.
    ///
    /// # Arguments
    ///
    /// * `event_data` - The data to create a new Event from.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `TaskContext` if successful,
    /// or a `WorkflowError` if execution fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_crate::Workflow;
    /// use serde_json::json;
    ///
    /// let workflow = Workflow::new(schema).expect("Failed to create workflow");
    /// let result = workflow.run(json!({"key": "value"}));
    /// ```
    pub fn run(&self, event_data: Value) -> Result<TaskContext, WorkflowError> {
        let mut task_context = TaskContext::new(self.schema.workflow_type.clone(), event_data);
        self.execute_workflow(&mut task_context)
    }

    /// Core workflow execution logic.
    ///
    /// This method is private and used internally by `run` and `run_from_event`.
    fn execute_workflow(
        &self,
        task_context: &mut TaskContext,
    ) -> Result<TaskContext, WorkflowError> {
        let mut current_node_type = Some(self.schema.start);

        while let Some(node_type) = current_node_type {
            let node_name = {
                let registry = self.registry.read().unwrap();
                let node = registry
                    .get(&node_type)
                    .ok_or(WorkflowError::NodeNotFound { node_type })?;
                node.node_name()
            };

            println!("Processing node: {}", node_name);

            // Process parallel nodes if any
            if let Some(node_config) = self
                .schema
                .nodes
                .iter()
                .find(|nc| nc.node_type == node_type)
            {
                if !node_config.parallel_nodes.is_empty() {
                    self.execute_parallel_nodes(&node_config.parallel_nodes, task_context)?;
                }
            }

            // Actually process the node
            *task_context = {
                let registry = self.registry.read().unwrap();
                let node = registry
                    .get(&node_type)
                    .ok_or(WorkflowError::NodeNotFound { node_type })?;
                node.process(task_context.clone())?
            };

            // Get next node
            current_node_type = self.get_next_node_type(node_type, task_context)?;
        }

        Ok(task_context.clone())
    }

    /// Executes parallel nodes in the workflow.
    ///
    /// This method is private and used internally by `execute_workflow`.
    fn execute_parallel_nodes(
        &self,
        parallel_nodes: &[TypeId],
        task_context: &mut TaskContext,
    ) -> Result<(), WorkflowError> {
        let mut handles = Vec::new();

        for &node_type in parallel_nodes {
            let context_clone = task_context.clone();
            let registry_clone = self.registry.clone();

            let handle = thread::spawn(move || -> Result<TaskContext, WorkflowError> {
                let registry = registry_clone.read().unwrap();
                let node = registry
                    .get(&node_type)
                    .ok_or(WorkflowError::NodeNotFound { node_type })?;

                println!("Processing parallel node: {}", node.node_name());
                node.process(context_clone)
            });
            handles.push(handle);
        }

        let results: Result<Vec<TaskContext>, WorkflowError> =
            handles.into_iter().map(|h| h.join().unwrap()).collect();

        let parallel_results = results?;

        // Merge results back into main context
        for result in parallel_results {
            for (key, value) in result.nodes {
                task_context.nodes.insert(key, value);
            }
            // Merge metadata as well
            for (key, value) in result.metadata {
                task_context.metadata.insert(key, value);
            }
        }

        Ok(())
    }

    /// Determines the next node type in the workflow.
    ///
    /// This method is private and used internally by `execute_workflow`.
    fn get_next_node_type(
        &self,
        current_node_type: TypeId,
        task_context: &TaskContext,
    ) -> Result<Option<TypeId>, WorkflowError> {
        let node_config = self
            .schema
            .nodes
            .iter()
            .find(|nc| nc.node_type == current_node_type);

        match node_config {
            Some(config) if config.connections.is_empty() => Ok(None),
            Some(config) if config.is_router => {
                // Get the router and call its route method
                let registry = self.registry.read().unwrap();
                let node = registry
                    .get(&current_node_type)
                    .ok_or(WorkflowError::NodeNotFound {
                        node_type: current_node_type,
                    })?;

                // Try to downcast to Router trait
                // Note: This is a simplified approach. In a real implementation,
                // you might want to use a different pattern for router identification
                Ok(config.connections.first().copied())
            }
            Some(config) => Ok(config.connections.first().copied()),
            None => Ok(None),
        }
    }

    /// Returns the workflow type as a string slice.
    ///
    /// # Returns
    ///
    /// A string slice containing the workflow type.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_crate::Workflow;
    ///
    /// let workflow = Workflow::new(schema).expect("Failed to create workflow");
    /// assert_eq!(workflow.get_workflow_type(), "example_workflow");
    ///
    pub fn workflow_type(&self) -> &str {
        &self.schema.workflow_type
    }

    // MCP server methods removed - use workflow-engine-mcp crate directly for MCP server functionality


    /// Gets the node registry for direct access
    ///
    /// # Returns
    ///
    /// Returns an Arc reference to the node registry
    pub fn get_registry(&self) -> Arc<RwLock<NodeRegistry>> {
        self.registry.clone()
    }
}

/// Wrapper to make existing nodes compatible with MCP server registration
#[derive(Debug)]
struct NodeWrapper {
    node_name: String,
    node_type: TypeId,
    registry: Arc<RwLock<NodeRegistry>>,
}

impl NodeWrapper {
    fn new(node: &dyn Node, node_type: TypeId, registry: Arc<RwLock<NodeRegistry>>) -> Self {
        Self {
            node_name: node.node_name(),
            node_type,
            registry,
        }
    }
}

impl Node for NodeWrapper {
    fn node_name(&self) -> String {
        self.node_name.clone()
    }

    fn process(&self, task_context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let registry = self.registry.read().unwrap();
        if let Some(node) = registry.get(&self.node_type) {
            node.process(task_context)
        } else {
            Err(WorkflowError::NodeNotFound {
                node_type: self.node_type,
            })
        }
    }
}
