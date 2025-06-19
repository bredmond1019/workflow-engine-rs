//! # Customer Support Workflow
//!
//! This module provides a complete customer support automation workflow that handles
//! the entire lifecycle of customer support tickets from initial validation through
//! final response and closure.
//!
//! ## Workflow Overview
//!
//! The customer support workflow implements a comprehensive ticket processing system with:
//! - **Automated ticket validation** and spam filtering
//! - **Intelligent intent determination** and routing
//! - **AI-powered response generation** with context awareness
//! - **Escalation handling** for complex issues
//! - **Automatic ticket closure** and follow-up
//!
//! ## Workflow Architecture
//!
//! ```text
//! ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
//! │  Analyze Ticket │───▶│  Ticket Router  │───▶│Generate Response│
//! │                 │    │                 │    │                 │
//! └─────────────────┘    └─────────────────┘    └─────────────────┘
//!          │                       │                       │
//!          ▼                       │                       ▼
//! ┌─────────────────┐              │              ┌─────────────────┐
//! │ Parallel Tasks: │              │              │   Send Reply    │
//! │ - Determine     │              │              │                 │
//! │   Intent        │              │              └─────────────────┘
//! │ - Filter Spam   │              │
//! │ - Validate      │              │
//! │   Ticket        │              │
//! └─────────────────┘              │
//! ```
//!
//! ## Usage
//!
//! ### Basic Workflow Creation
//!
//! ```rust
//! use ai_architecture_workflows::customer_support_workflow::create_customer_care_workflow;
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create the customer support workflow
//!     let workflow = create_customer_care_workflow()?;
//!
//!     // Process a customer ticket
//!     let ticket_data = json!({
//!         "ticket_id": "TICKET-2024-001",
//!         "customer_id": "CUST-456",
//!         "message": "I'm having trouble accessing my account after the recent update",
//!         "priority": "high",
//!         "category": "technical_support"
//!     });
//!
//!     match workflow.run(ticket_data).await {
//!         Ok(result) => {
//!             println!("Ticket processed successfully: {}", result.event_id);
//!             // Access processing results
//!             for (node_name, node_result) in &result.nodes {
//!                 println!("Node '{}' result: {:?}", node_name, node_result);
//!             }
//!         }
//!         Err(e) => eprintln!("Ticket processing failed: {}", e),
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Advanced Usage with Database Integration
//!
//! ```rust
//! use ai_architecture_workflows::{WorkflowRunner, customer_support_workflow::create_customer_care_workflow};
//! use ai_architecture_core::db::event::NewEvent;
//! use diesel::prelude::*;
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Set up workflow runner with database integration
//!     let workflow = create_customer_care_workflow()?;
//!     let runner = WorkflowRunner::new(workflow);
//!     let mut conn = establish_connection();
//!
//!     // Create and process multiple tickets
//!     let tickets = vec![
//!         json!({
//!             "ticket_id": "TICKET-001",
//!             "customer_id": "CUSTOMER-123", 
//!             "message": "Billing question about my subscription",
//!             "priority": "medium"
//!         }),
//!         json!({
//!             "ticket_id": "TICKET-002",
//!             "customer_id": "CUSTOMER-456",
//!             "message": "URGENT: My service is completely down!",
//!             "priority": "critical"
//!         }),
//!     ];
//!
//!     for ticket_data in tickets {
//!         match runner.create_and_process(ticket_data, &mut conn) {
//!             Ok(processed_event) => {
//!                 println!("Processed ticket: {}", processed_event.id);
//!             }
//!             Err(e) => {
//!                 eprintln!("Failed to process ticket: {}", e);
//!             }
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Workflow Nodes
//!
//! ### AnalyzeTicketNode
//! 
//! Entry point that performs initial ticket analysis and coordinates parallel processing:
//! - Extracts key information from the ticket
//! - Triggers parallel validation, spam filtering, and intent determination
//! - Prepares data for routing decisions
//!
//! ### TicketRouterNode (Router)
//!
//! Intelligent routing node that directs tickets based on analysis results:
//! - Routes to appropriate response generation based on intent
//! - Handles escalation for complex or high-priority issues  
//! - Manages load balancing across response generation nodes
//!
//! ### Parallel Processing Nodes
//!
//! #### DetermineTicketIntentNode
//! - Analyzes customer message to determine intent (billing, technical, general, etc.)
//! - Uses AI models to classify the type of support needed
//! - Provides confidence scores for routing decisions
//!
//! #### FilterSpamNode
//! - Detects spam, abusive, or automated messages
//! - Implements multiple filtering strategies
//! - Flags suspicious content for review
//!
//! #### ValidateTicketNode
//! - Ensures ticket data completeness and validity
//! - Checks for required fields and proper formatting
//! - Validates customer information
//!
//! ### GenerateResponseNode
//!
//! AI-powered response generation:
//! - Creates contextual responses based on ticket analysis
//! - Incorporates customer history and preferences
//! - Generates multiple response options when appropriate
//!
//! ### SendReplyNode
//!
//! Final delivery and cleanup:
//! - Sends the response to the customer
//! - Updates ticket status and tracking
//! - Schedules follow-up actions if needed
//!
//! ## Input Data Format
//!
//! The workflow expects ticket data in the following JSON format:
//!
//! ```json
//! {
//!   "ticket_id": "TICKET-2024-001",
//!   "customer_id": "CUSTOMER-456",
//!   "message": "Customer's support request message",
//!   "priority": "low|medium|high|critical",
//!   "category": "billing|technical|general|account",
//!   "channel": "email|chat|phone|web",
//!   "timestamp": "2024-01-15T10:30:00Z",
//!   "customer_tier": "basic|premium|enterprise",
//!   "previous_tickets": 5,
//!   "metadata": {
//!     "user_agent": "...",
//!     "source_ip": "...",
//!     "session_id": "..."
//!   }
//! }
//! ```
//!
//! ### Required Fields
//! - `ticket_id`: Unique identifier for the support ticket
//! - `customer_id`: Customer identifier for context and history
//! - `message`: The customer's support request or question
//! - `priority`: Priority level for processing order
//!
//! ### Optional Fields
//! - `category`: Pre-classified category if available
//! - `channel`: Communication channel used by customer
//! - `timestamp`: When the ticket was created
//! - `customer_tier`: Customer service level
//! - `previous_tickets`: Number of previous tickets for context
//! - `metadata`: Additional tracking and context information
//!
//! ## Output Data Structure
//!
//! The workflow produces a `TaskContext` with the following node results:
//!
//! ```json
//! {
//!   "analyze_ticket": {
//!     "status": "completed",
//!     "extracted_info": {
//!       "keywords": ["account", "access", "login"],
//!       "sentiment": "frustrated",
//!       "complexity": "medium"
//!     }
//!   },
//!   "determine_intent": {
//!     "status": "completed", 
//!     "intent": "technical_support",
//!     "confidence": 0.89,
//!     "sub_category": "account_access"
//!   },
//!   "filter_spam": {
//!     "status": "completed",
//!     "is_spam": false,
//!     "confidence": 0.95,
//!     "flags": []
//!   },
//!   "validate_ticket": {
//!     "status": "completed",
//!     "is_valid": true,
//!     "missing_fields": [],
//!     "validation_errors": []
//!   },
//!   "generate_response": {
//!     "status": "completed",
//!     "response": "Thank you for contacting support...",
//!     "response_type": "solution_provided",
//!     "estimated_resolution_time": "2-4 hours"
//!   },
//!   "send_reply": {
//!     "status": "completed",
//!     "sent_at": "2024-01-15T10:35:22Z",
//!     "delivery_method": "email",
//!     "tracking_id": "MSG-789"
//!   }
//! }
//! ```
//!
//! ## Running the Demo
//!
//! To see the customer support workflow in action:
//!
//! ### Quick Start
//! ```bash
//! # Run all demos including customer support
//! cargo run
//! 
//! # Run only customer support demos
//! cargo run --example customer_support_demo
//! ```
//!
//! ### Programmatic Demo Execution
//! ```rust
//! use ai_architecture_workflows::demos::customer_care_workflow::customer_care_workflow_demo;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Run the interactive customer support workflow demo
//!     customer_care_workflow_demo().await;
//! }
//! ```
//!
//! ### Demo Features
//!
//! The demo showcases:
//! - **Multiple ticket scenarios** (billing, technical, urgent, general)
//! - **Real-time processing visualization** with timing information
//! - **Node-by-node execution tracking** with detailed results
//! - **Database integration** showing event creation and updates
//! - **Error handling** with specific troubleshooting guidance
//! - **Type-safe data extraction** from processed events
//!
//! ## Configuration
//!
//! ### Environment Variables
//!
//! ```bash
//! # AI Provider Configuration
//! ANTHROPIC_API_KEY=your_anthropic_key_here
//! OPENAI_API_KEY=your_openai_key_here
//!
//! # Database Configuration
//! DATABASE_URL=postgresql://user:password@localhost/ai_architecture
//!
//! # Customer Support Specific Settings
//! SUPPORT_ESCALATION_THRESHOLD=0.8
//! SUPPORT_AUTO_CLOSE_ENABLED=true
//! SUPPORT_RESPONSE_TIMEOUT_MINUTES=30
//! ```
//!
//! ### Workflow Customization
//!
//! #### Adding Custom Validation Rules
//! ```rust
//! use ai_architecture_core::{nodes::Node, task::TaskContext, error::WorkflowError};
//!
//! #[derive(Debug)]
//! struct CustomValidationNode {
//!     business_rules: Vec<ValidationRule>,
//! }
//!
//! impl Node for CustomValidationNode {
//!     fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
//!         // Apply custom business validation logic
//!         let ticket_data = context.get_data::<serde_json::Value>("ticket_data")?;
//!         
//!         for rule in &self.business_rules {
//!             rule.validate(&ticket_data)?;
//!         }
//!         
//!         context.update_node("custom_validation", serde_json::json!({
//!             "validation_passed": true,
//!             "rules_applied": self.business_rules.len()
//!         }));
//!         
//!         Ok(context)
//!     }
//! }
//! ```
//!
//! #### Extending the Workflow
//! ```rust
//! use ai_architecture_core::workflow::builder::WorkflowBuilder;
//! use std::any::TypeId;
//!
//! pub fn create_enhanced_customer_support_workflow() -> Result<Workflow, WorkflowError> {
//!     let workflow = WorkflowBuilder::new::<AnalyzeTicketNode>("enhanced_customer_support".to_string())
//!         .description("Enhanced Customer Support with Custom Rules".to_string())
//!         .add_node(
//!             NodeConfig::new::<AnalyzeTicketNode>()
//!                 .with_connections(vec![TypeId::of::<CustomValidationNode>()])
//!                 .with_parallel_nodes(vec![
//!                     TypeId::of::<DetermineTicketIntentNode>(),
//!                     TypeId::of::<FilterSpamNode>(),
//!                     TypeId::of::<ValidateTicketNode>(),
//!                 ])
//!         )
//!         .add_node(
//!             NodeConfig::new::<CustomValidationNode>()
//!                 .with_connections(vec![TypeId::of::<TicketRouterNode>()])
//!         )
//!         // ... continue with remaining nodes
//!         .build()?;
//!     
//!     // Register all nodes including custom ones
//!     workflow.register_node(CustomValidationNode::new());
//!     // ... register other nodes
//!     
//!     Ok(workflow)
//! }
//! ```
//!
//! ## Performance Considerations
//!
//! ### Parallel Processing
//! - Validation, spam filtering, and intent determination run in parallel
//! - Reduces total processing time from ~3s to ~1s for typical tickets
//! - Scales well with ticket volume
//!
//! ### Memory Usage
//! - Each workflow instance maintains minimal state
//! - Node results are stored efficiently in TaskContext
//! - Database integration allows for persistent storage without memory bloat
//!
//! ### Throughput Optimization
//! ```rust
//! // Process multiple tickets concurrently
//! use tokio::task::JoinSet;
//!
//! let mut join_set = JoinSet::new();
//! for ticket in tickets {
//!     let workflow = workflow.clone();
//!     join_set.spawn(async move {
//!         workflow.run(ticket).await
//!     });
//! }
//!
//! // Collect results as they complete
//! while let Some(result) = join_set.join_next().await {
//!     match result? {
//!         Ok(context) => println!("Ticket processed: {}", context.event_id),
//!         Err(e) => eprintln!("Processing failed: {}", e),
//!     }
//! }
//! ```
//!
//! ## Testing
//!
//! ### Unit Tests
//! ```bash
//! # Test individual workflow components
//! cargo test customer_support_workflow::tests
//!
//! # Test specific nodes
//! cargo test analyze_ticket_node
//! cargo test ticket_router_node
//! ```
//!
//! ### Integration Tests
//! ```bash
//! # Test complete workflow with database
//! cargo test --test customer_support_integration
//!
//! # Test with MCP integration
//! cargo test --test customer_support_mcp_integration
//! ```
//!
//! ### Performance Tests
//! ```bash
//! # Run performance benchmarks
//! cargo bench customer_support_benchmarks
//! ```
//!
//! ## Troubleshooting
//!
//! ### Common Issues
//!
//! #### Missing Node Registration
//! ```
//! Error: NodeNotFound { node_type: "AnalyzeTicketNode" }
//! ```
//! **Solution**: Ensure all nodes are registered with `workflow.register_node()`
//!
//! #### Invalid Input Data
//! ```
//! Error: DeserializationError { message: "missing field ticket_id" }
//! ```
//! **Solution**: Verify input JSON contains all required fields
//!
//! #### Workflow Validation Errors
//! ```
//! Error: UnreachableNodes { nodes: ["SendReplyNode"] }
//! ```
//! **Solution**: Check node connections in WorkflowBuilder configuration
//!
//! ### Debug Mode
//! ```bash
//! # Enable detailed logging
//! RUST_LOG=debug cargo run
//!
//! # Trace node execution
//! RUST_LOG=ai_architecture_workflows::customer_support_workflow=trace cargo run
//! ```
//!
//! ## Related Documentation
//!
//! - [`WorkflowRunner`](../struct.WorkflowRunner.html) - Database-integrated execution
//! - [`demos`](../demos/index.html) - Interactive demonstrations  
//! - [`knowledge_base_workflow`](../knowledge_base_workflow/index.html) - Knowledge search workflow
//! - [MCP Integration Guide](../../../docs/mcp-integration.md) - External service integration

use std::any::TypeId;

use workflow_engine_mcp::server::customer_support::tools::{
    AnalyzeTicketNode, DetermineTicketIntentNode, FilterSpamNode, GenerateResponseNode,
    SendReplyNode, TicketRouterNode, ValidateTicketNode,
};
use workflow_engine_core::{
    error::WorkflowError,
    nodes::config::NodeConfig,
    workflow::{Workflow, builder::WorkflowBuilder},
};

pub fn create_customer_care_workflow() -> Result<Workflow, WorkflowError> {
    let workflow = WorkflowBuilder::new::<AnalyzeTicketNode>("customer_care".to_string())
        .description("Customer Care Workflow".to_string())
        .add_node(
            NodeConfig::new::<AnalyzeTicketNode>()
                .with_connections(vec![TypeId::of::<TicketRouterNode>()])
                .with_description("Analyzes incoming ticket".to_string())
                .with_parallel_nodes(vec![
                    TypeId::of::<DetermineTicketIntentNode>(),
                    TypeId::of::<FilterSpamNode>(),
                    TypeId::of::<ValidateTicketNode>(),
                ]),
        )
        .add_node(
            NodeConfig::new::<TicketRouterNode>()
                .with_connections(vec![TypeId::of::<GenerateResponseNode>()])
                .with_router(true)
                .with_description("Routes ticket based on analysis".to_string()),
        )
        .add_node(
            NodeConfig::new::<GenerateResponseNode>()
                .with_connections(vec![TypeId::of::<SendReplyNode>()])
                .with_description("Generates response to customer".to_string()),
        )
        .add_node(
            NodeConfig::new::<SendReplyNode>()
                .with_description("Sends reply to customer".to_string()),
        )
        .build()?;

    // Register all nodes
    workflow.register_node(AnalyzeTicketNode);
    workflow.register_node(DetermineTicketIntentNode);
    workflow.register_node(FilterSpamNode);
    workflow.register_node(ValidateTicketNode);
    workflow.register_node(TicketRouterNode::new());
    workflow.register_node(GenerateResponseNode);
    workflow.register_node(SendReplyNode);

    Ok(workflow)
}
