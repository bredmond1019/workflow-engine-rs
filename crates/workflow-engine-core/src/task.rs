//! # Task Context Management
//!
//! This module provides the [`TaskContext`] type, which serves as the primary data container
//! that flows through workflow execution. The task context carries event data, node results,
//! metadata, and workflow state as processing moves from node to node.
//!
//! ## Core Concepts
//!
//! ### Task Context
//! The [`TaskContext`] is the fundamental data structure that:
//! - Carries input data through the workflow
//! - Stores results from each processing node
//! - Maintains metadata about execution state
//! - Provides type-safe data access methods
//! - Tracks timing and workflow information
//!
//! ### Data Flow
//! As a workflow executes:
//! 1. Initial event data is loaded into the context
//! 2. Each node processes the context and adds its results
//! 3. Node results are stored with unique keys
//! 4. The updated context flows to the next node
//! 5. Final context contains all processing results
//!
//! ## Usage Examples
//!
//! ### Creating a Task Context
//!
//! ```rust
//! use workflow_engine_core::task::TaskContext;
//! use serde_json::json;
//!
//! // Create from workflow type and event data
//! let context = TaskContext::new(
//!     "customer_support_workflow".to_string(),
//!     json!({
//!         "ticket_id": "TICKET-123",
//!         "customer_message": "My order is delayed",
//!         "priority": "high"
//!     })
//! );
//!
//! println!("Workflow: {}", context.workflow_type);
//! println!("Event ID: {}", context.event_id);
//! ```
//!
//! ### Extracting Event Data
//!
//! ```rust
//! use serde::{Deserialize, Serialize};
//! use workflow_engine_core::{task::TaskContext, error::WorkflowError};
//!
//! #[derive(Debug, Deserialize, Serialize)]
//! struct TicketData {
//!     ticket_id: String,
//!     customer_message: String,
//!     priority: String,
//! }
//!
//! fn process_ticket_data(context: &TaskContext) -> Result<TicketData, WorkflowError> {
//!     // Type-safe extraction of event data
//!     let ticket_data: TicketData = context.get_event_data()?;
//!     println!("Processing ticket: {}", ticket_data.ticket_id);
//!     Ok(ticket_data)
//! }
//! ```
//!
//! ### Storing Node Results
//!
//! ```rust,ignore
//! // This example shows node result storage patterns
//! use workflow_engine_core::{nodes::Node, task::TaskContext, error::WorkflowError};
//! use serde_json::json;
//!
//! #[derive(Debug)]
//! struct AnalysisNode;
//!
//! impl Node for AnalysisNode {
//!     fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
//!         // Extract input data
//!         let ticket_data: TicketData = context.get_event_data()?;
//!         
//!         // Perform analysis
//!         let sentiment_score = analyze_sentiment(&ticket_data.customer_message);
//!         let urgency_level = determine_urgency(&ticket_data.priority);
//!         
//!         // Store results in context
//!         context.update_node("sentiment_analysis", json!({
//!             "score": sentiment_score,
//!             "label": if sentiment_score > 0.5 { "positive" } else { "negative" }
//!         }));
//!         
//!         context.update_node("urgency_analysis", json!({
//!             "level": urgency_level,
//!             "requires_escalation": urgency_level > 8
//!         }));
//!         
//!         // Store metadata
//!         context.set_metadata("processing_node", "analysis")?;
//!         context.set_metadata("analysis_timestamp", chrono::Utc::now())?;
//!         
//!         Ok(context)
//!     }
//! }
//! ```
//!
//! ### Retrieving Node Results
//!
//! ```rust
//! use serde::{Deserialize, Serialize};
//! use workflow_engine_core::{task::TaskContext, error::WorkflowError};
//!
//! #[derive(Debug, Deserialize)]
//! struct SentimentResult {
//!     score: f64,
//!     label: String,
//! }
//!
//! #[derive(Debug, Deserialize)]
//! struct UrgencyResult {
//!     level: u8,
//!     requires_escalation: bool,
//! }
//!
//! fn use_analysis_results(context: &TaskContext) -> Result<(), WorkflowError> {
//!     // Retrieve typed results from previous nodes
//!     if let Some(sentiment) = context.get_node_data::<SentimentResult>("sentiment_analysis")? {
//!         println!("Sentiment: {} (score: {})", sentiment.label, sentiment.score);
//!     }
//!     
//!     if let Some(urgency) = context.get_node_data::<UrgencyResult>("urgency_analysis")? {
//!         if urgency.requires_escalation {
//!             println!("ALERT: Ticket requires escalation (level {})", urgency.level);
//!         }
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Working with Metadata
//!
//! ```rust
//! use workflow_engine_core::{task::TaskContext, error::WorkflowError};
//! use chrono::{DateTime, Utc};
//!
//! fn track_processing_metadata(
//!     mut context: TaskContext,
//!     node_name: &str
//! ) -> Result<TaskContext, WorkflowError> {
//!     // Set processing metadata
//!     context.set_metadata("current_node", node_name)?;
//!     context.set_metadata("processing_start", Utc::now())?;
//!     context.set_metadata("attempt_count", 1u32)?;
//!     
//!     // Later: retrieve metadata
//!     if let Some(start_time) = context.get_metadata::<DateTime<Utc>>("processing_start")? {
//!         let duration = Utc::now().signed_duration_since(start_time);
//!         println!("Processing time: {} ms", duration.num_milliseconds());
//!     }
//!     
//!     if let Some(attempt) = context.get_metadata::<u32>("attempt_count")? {
//!         context.set_metadata("attempt_count", attempt + 1)?;
//!     }
//!     
//!     Ok(context)
//! }
//! ```
//!
//! ### Error Handling Patterns
//!
//! ```rust
//! use workflow_engine_core::{
//!     nodes::Node,
//!     task::TaskContext,
//!     error::WorkflowError,
//! };
//! use serde_json::json;
//!
//! #[derive(Debug)]
//! struct RobustProcessingNode;
//!
//! impl Node for RobustProcessingNode {
//!     fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
//!         // Attempt to extract data with fallback
//!         let input_data = match context.get_event_data::<serde_json::Value>() {
//!             Ok(data) => data,
//!             Err(_) => {
//!                 // Store error information and continue with default
//!                 context.set_metadata("input_error", "Failed to parse event data")?;
//!                 json!({"fallback": true})
//!             }
//!         };
//!         
//!         // Store processing result
//!         context.update_node("processed_data", json!({
//!             "input": input_data,
//!             "status": "completed",
//!             "timestamp": chrono::Utc::now()
//!         }));
//!         
//!         Ok(context)
//!     }
//! }
//! ```
//!
//! ### Converting to Database Events
//!
//! ```rust,ignore
//! // This example requires database features
//! use workflow_engine_core::{task::TaskContext, db::event::Event, error::WorkflowError};
//! use diesel::prelude::*;
//!
//! fn persist_workflow_result(
//!     context: TaskContext,
//!     conn: &mut PgConnection
//! ) -> Result<Event, WorkflowError> {
//!     // Convert context back to database event
//!     let event = context.to_event()?;
//!     
//!     // Store in database
//!     event.store(conn)?;
//!     
//!     println!("Stored workflow result for event: {}", event.id);
//!     Ok(event)
//! }
//! ```
//!
//! ## Performance Considerations
//!
//! ### Memory Usage
//! - Task contexts can grow large with accumulated node data
//! - Consider removing intermediate results that won't be needed
//! - Use streaming for large data processing when possible
//!
//! ### Cloning Efficiency
//! - Task contexts are cloneable for parallel processing
//! - Cloning copies all data - be mindful of large datasets
//! - Consider using references where full context isn't needed
//!
//! ### Serialization Overhead
//! - All data is JSON-serialized for storage and transmission
//! - Complex types may have serialization overhead
//! - Consider using efficient serialization formats for large data
//!
//! ## Thread Safety
//!
//! [`TaskContext`] is designed for single-threaded workflow execution but supports:
//! - Cloning for parallel node execution
//! - Serialization for cross-thread communication
//! - Immutable access patterns for shared reading
//!
//! ## Best Practices
//!
//! 1. **Use Descriptive Keys**: Name node results clearly (`"email_analysis"` vs `"result1"`)
//! 2. **Structure Node Data**: Use well-defined types rather than generic JSON
//! 3. **Handle Missing Data**: Always check for `None` when retrieving optional node data
//! 4. **Clean Up Intermediate Data**: Remove large temporary data that's no longer needed
//! 5. **Use Metadata**: Store processing information, timestamps, and debug data in metadata

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

// Event integration is in the API crate
// use crate::db::event::Event;

use super::error::WorkflowError;

/// The primary data container that flows through workflow execution.
///
/// `TaskContext` carries all necessary information for workflow processing:
/// - Original event data that triggered the workflow
/// - Results from each processing node
/// - Metadata about execution state and timing
/// - Workflow identification and tracking information
///
/// # Data Structure
///
/// ## Core Fields
/// - `event_id`: Unique identifier for this workflow execution
/// - `workflow_type`: Type of workflow being executed
/// - `event_data`: Original input data that triggered the workflow
/// - `nodes`: Results from each processing node, keyed by node name
/// - `metadata`: Additional execution metadata and debugging information
/// - `created_at`: When this context was originally created
/// - `updated_at`: When this context was last modified
///
/// # Thread Safety
///
/// `TaskContext` is `Clone` and can be safely passed between threads for
/// parallel processing. All data is internally managed through JSON values
/// for consistent serialization.
///
/// # Examples
///
/// ```rust
/// use workflow_engine_core::task::TaskContext;
/// use serde_json::json;
/// use uuid::Uuid;
///
/// // Create new context
/// let mut context = TaskContext::new(
///     "order_processing".to_string(),
///     json!({"order_id": "ORD-123", "amount": 99.99})
/// );
///
/// // Add node result
/// context.update_node("validation", json!({
///     "valid": true,
///     "checks_passed": ["amount", "customer", "inventory"]
/// }));
///
/// // Add metadata
/// context.set_metadata("processing_start", chrono::Utc::now()).unwrap();
///
/// // Retrieve data
/// let order_data: serde_json::Value = context.get_event_data().unwrap();
/// println!("Processing order: {}", order_data["order_id"]);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    /// Unique identifier for this workflow execution instance
    pub event_id: Uuid,
    
    /// Type identifier for the workflow being executed
    pub workflow_type: String,
    
    /// Original event data that triggered this workflow
    pub event_data: Value,
    
    /// Results from each processing node, keyed by node name
    pub nodes: HashMap<String, Value>,
    
    /// Additional metadata about execution state and debugging information
    pub metadata: HashMap<String, Value>,
    
    /// Timestamp when this context was originally created
    pub created_at: DateTime<Utc>,
    
    /// Timestamp when this context was last updated
    pub updated_at: DateTime<Utc>,
}

impl TaskContext {
    // Event integration methods are disabled in core crate
    /*
    pub fn from_event(event: &Event) -> Self {
        Self {
            event_id: event.id,
            workflow_type: event.workflow_type.clone(),
            event_data: event.data.clone(),
            nodes: HashMap::new(),
            metadata: HashMap::new(),
            created_at: event.created_at,
            updated_at: Utc::now(),
        }
    }
    */

    pub fn new(workflow_type: String, event_data: Value) -> Self {
        let now = Utc::now();
        Self {
            event_id: Uuid::new_v4(),
            workflow_type,
            event_data,
            nodes: HashMap::new(),
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_node<T: Serialize>(&mut self, node_name: &str, data: T) {
        if let Ok(value) = serde_json::to_value(data) {
            self.nodes.insert(node_name.to_string(), value);
            self.updated_at = Utc::now();
        }
    }

    pub fn get_event_data<T: for<'de> Deserialize<'de>>(&self) -> Result<T, WorkflowError> {
        serde_json::from_value(self.event_data.clone()).map_err(|e| {
            WorkflowError::DeserializationError {
                message: format!("Failed to deserialize event data: {}", e),
                expected_type: std::any::type_name::<T>().to_string(),
                context: "from event data".to_string(),
                raw_data: Some(self.event_data.to_string()),
                source: Some(e),
            }
        })
    }

    pub fn get_node_data<T: for<'de> Deserialize<'de>>(
        &self,
        node_name: &str,
    ) -> Result<Option<T>, WorkflowError> {
        match self.nodes.get(node_name) {
            Some(value) => {
                let data = serde_json::from_value(value.clone()).map_err(|e| {
                    WorkflowError::DeserializationError {
                        message: format!(
                            "Failed to deserialize node data for {}: {}",
                            node_name, e
                        ),
                        expected_type: std::any::type_name::<T>().to_string(),
                        context: format!("from node '{}' data", node_name),
                        raw_data: Some(value.to_string()),
                        source: Some(e),
                    }
                })?;
                Ok(Some(data))
            }
            None => Ok(None),
        }
    }

    /*
    pub fn to_event(&self) -> Result<Event, WorkflowError> {
        let task_context_value =
            serde_json::to_value(self).map_err(|e| WorkflowError::SerializationError {
                message: format!("Failed to serialize task context: {}", e),
            })?;

        Ok(Event {
            id: self.event_id,
            workflow_type: self.workflow_type.clone(),
            data: self.event_data.clone(),
            task_context: task_context_value,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
    */

    // Additional methods that are referenced in the codebase
    pub fn set_data<T: Serialize>(&mut self, key: &str, data: T) -> Result<(), WorkflowError> {
        let value = serde_json::to_value(data).map_err(|e| WorkflowError::SerializationError {
            message: format!("Failed to serialize data for key {}: {}", key, e),
            type_name: std::any::type_name::<T>().to_string(),
            context: format!("for key '{}'", key),
            source: Some(e),
        })?;
        self.nodes.insert(key.to_string(), value);
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn get_data<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>, WorkflowError> {
        self.get_node_data(key)
    }

    pub fn get_all_data(&self) -> &HashMap<String, Value> {
        &self.nodes
    }

    pub fn add_data<T: Serialize>(&mut self, key: &str, data: T) -> Result<(), WorkflowError> {
        self.set_data(key, data)
    }

    pub fn set_metadata<T: Serialize>(&mut self, key: &str, value: T) -> Result<(), WorkflowError> {
        let serialized_value = serde_json::to_value(value).map_err(|e| WorkflowError::SerializationError {
            message: format!("Failed to serialize metadata for key {}: {}", key, e),
            type_name: std::any::type_name::<T>().to_string(),
            context: format!("for metadata key '{}'", key),
            source: Some(e),
        })?;
        self.metadata.insert(key.to_string(), serialized_value);
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn get_metadata<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>, WorkflowError> {
        match self.metadata.get(key) {
            Some(value) => {
                let data = serde_json::from_value(value.clone()).map_err(|e| {
                    WorkflowError::DeserializationError {
                        message: format!("Failed to deserialize metadata for {}: {}", key, e),
                        expected_type: std::any::type_name::<T>().to_string(),
                        context: format!("from metadata key '{}'", key),
                        raw_data: Some(value.to_string()),
                        source: Some(e),
                    }
                })?;
                Ok(Some(data))
            }
            None => Ok(None),
        }
    }

    pub fn get_all_metadata(&self) -> &HashMap<String, Value> {
        &self.metadata
    }
}
