//! # AI Architecture Workflows Module
//!
//! This module contains pre-built workflow implementations and comprehensive demonstrations
//! of the AI Architecture system. It provides complete examples of customer support and
//! knowledge base workflows, along with interactive demos that showcase the system's
//! capabilities.
//!
//! ## Module Structure
//!
//! ### Core Workflows
//! - [`customer_support_workflow`] - Complete customer support automation workflow
//! - [`knowledge_base_workflow`] - Knowledge search and retrieval workflow
//!
//! ### Interactive Demos
//! - [`demos`] - Comprehensive demonstration suite with timing and visual feedback
//!
//! ### Workflow Runner
//! - [`WorkflowRunner`] - Database-integrated workflow execution engine
//!
//! ## Quick Start
//!
//! To run all demos and see the system in action:
//!
//! ```bash
//! # Clone the repository
//! git clone <repository-url>
//! cd ai-system-rust
//!
//! # Set up environment
//! cp .env.example .env
//! # Edit .env with your API keys and database settings
//!
//! # Run the demos
//! cargo run
//! ```
//!
//! ## Available Workflows
//!
//! ### Customer Support Workflow
//!
//! A complete customer support automation system featuring:
//! - Ticket validation and spam filtering
//! - Intent determination and routing
//! - AI-powered response generation
//! - Escalation handling
//! - Automatic ticket closure
//!
//! ```rust
//! use ai_architecture_workflows::customer_support_workflow::create_customer_support_workflow;
//! use serde_json::json;
//!
//! // Create the workflow
//! let workflow = create_customer_support_workflow()?;
//!
//! // Process a customer ticket
//! let result = workflow.run(json!({
//!     "ticket_id": "TICKET-123",
//!     "customer_message": "I need help with my order",
//!     "customer_email": "customer@example.com",
//!     "priority": "medium"
//! }))?;
//! ```
//!
//! ### Knowledge Base Workflow
//!
//! A knowledge search and retrieval system that:
//! - Validates and routes user queries
//! - Searches multiple knowledge sources (Notion, HelpScout, Slack)
//! - Generates comprehensive responses
//! - Provides source attribution
//!
//! ```rust
//! use ai_architecture_workflows::knowledge_base_workflow::create_knowledge_base_workflow;
//! use serde_json::json;
//!
//! // Create the workflow
//! let workflow = create_knowledge_base_workflow()?;
//!
//! // Search knowledge base
//! let result = workflow.run(json!({
//!     "query_id": "QUERY-456",
//!     "user_query": "How do I configure SSL certificates?",
//!     "user_id": "user789",
//!     "priority": "high"
//! }))?;
//! ```
//!
//! ## Running Demos
//!
//! The system includes comprehensive interactive demos that showcase all features:
//!
//! ### Method 1: Run All Demos (Recommended)
//! ```bash
//! cargo run
//! ```
//! This runs all demos in sequence with visual progress indicators and timing.
//!
//! ### Method 2: Run Specific Demo Categories
//! ```rust
//! use ai_architecture_workflows::demos;
//!
//! // Run only customer care demos
//! demos::run_all_customer_care_demos().await;
//!
//! // Run only knowledge base demos
//! demos::run_all_knowledge_base_demos().await;
//! ```
//!
//! ### Method 3: Run Individual Demos
//! ```rust
//! use ai_architecture_workflows::demos::*;
//!
//! // Individual demo functions
//! customer_care_workflow_demo().await;
//! customer_care_mcp_demo().await;
//! knowledge_base_workflow_demo().await;
//! knowledge_base_mcp_demo().await;
//! ```
//!
//! ## Database Integration
//!
//! ### Using WorkflowRunner
//!
//! The [`WorkflowRunner`] provides database-integrated workflow execution:
//!
//! ```rust
//! use ai_architecture_workflows::{WorkflowRunner, customer_support_workflow::create_customer_support_workflow};
//! use diesel::prelude::*;
//! use serde_json::json;
//!
//! // Set up workflow runner
//! let workflow = create_customer_support_workflow()?;
//! let runner = WorkflowRunner::new(workflow);
//!
//! // Process events from database
//! let mut conn = establish_connection();
//! let events = load_pending_events(&mut conn)?;
//!
//! for event in events {
//!     match runner.process_event(&event, &mut conn) {
//!         Ok(processed_event) => {
//!             println!("Processed event: {}", processed_event.id);
//!         }
//!         Err(e) => {
//!             eprintln!("Failed to process event: {}", e);
//!         }
//!     }
//! }
//! ```
//!
//! ### Creating and Processing New Events
//!
//! ```rust
//! // Create and immediately process a new event
//! let result = runner.create_and_process(
//!     json!({
//!         "ticket_id": "TICKET-789",
//!         "customer_message": "Urgent: Payment issue",
//!         "priority": "high"
//!     }),
//!     &mut conn
//! )?;
//!
//! println!("Event processed: {}", result.id);
//! ```
//!
//! ## Configuration
//!
//! ### Environment Variables
//!
//! Set up your `.env` file with the following variables:
//!
//! ```bash
//! # Database
//! DATABASE_URL=postgresql://username:password@localhost/ai_architecture
//!
//! # AI Providers
//! ANTHROPIC_API_KEY=your_anthropic_key_here
//! OPENAI_API_KEY=your_openai_key_here
//!
//! # MCP Servers (optional)
//! MCP_ENABLED=true
//! MCP_CLIENT_NAME=ai-architecture-client
//! MCP_CLIENT_VERSION=1.0.0
//!
//! # Server Configuration
//! HOST=127.0.0.1
//! PORT=8080
//! ```
//!
//! ### Database Setup
//!
//! ```bash
//! # Install diesel CLI
//! cargo install diesel_cli --no-default-features --features postgres
//!
//! # Set up database
//! diesel setup
//! diesel migration run
//! ```
//!
//! ## Demo Output Examples
//!
//! When you run the demos, you'll see detailed output like this:
//!
//! ```text
//! ╔═══════════════════════════════════════════════════════════╗
//! ║           Customer Care Workflow Demo                     ║
//! ╚═══════════════════════════════════════════════════════════╝
//!
//! 🚀 Starting workflow execution...
//! ┌─────────────────────────────────────────────────────────┐
//! │ Processing: validate_ticket                             │
//! │         📋 Validating ticket format and content...     │
//! │ ✅ Completed in 0.15s                                  │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Extending Workflows
//!
//! ### Adding Custom Nodes
//!
//! ```rust
//! use ai_architecture_core::{nodes::Node, task::TaskContext, error::WorkflowError};
//!
//! #[derive(Debug)]
//! struct CustomProcessingNode;
//!
//! impl Node for CustomProcessingNode {
//!     fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
//!         // Your custom processing logic
//!         context.update_node("custom_result", serde_json::json!({
//!             "processed": true,
//!             "timestamp": chrono::Utc::now()
//!         }));
//!         Ok(context)
//!     }
//! }
//! ```
//!
//! ### Modifying Existing Workflows
//!
//! ```rust
//! use ai_architecture_core::workflow::builder::WorkflowBuilder;
//!
//! // Extend customer support workflow
//! let enhanced_workflow = WorkflowBuilder::new("enhanced_customer_support")
//!     .start_with::<ValidateTicketNode>()
//!     .then::<CustomProcessingNode>()  // Add custom node
//!     .then::<DetermineIntentNode>()
//!     .then::<GenerateResponseNode>()
//!     .build()?;
//! ```
//!
//! ## Testing Workflows
//!
//! ### Unit Testing Individual Workflows
//!
//! ```bash
//! # Test customer support workflow
//! cargo test customer_support_workflow
//!
//! # Test knowledge base workflow  
//! cargo test knowledge_base_workflow
//!
//! # Test all workflow components
//! cargo test workflows
//! ```
//!
//! ### Integration Testing
//!
//! ```bash
//! # Run integration tests with database
//! cargo test --test workflow_integration_test
//!
//! # Run MCP integration tests
//! cargo test --test mcp_integration_test
//! ```
//!
//! ## Performance Monitoring
//!
//! The demos include built-in performance monitoring:
//!
//! ```text
//! ⏱️  Workflow Performance Summary:
//! ┌─────────────────────────────────────────────────────────┐
//! │ Total execution time: 2.34s                            │
//! │ Nodes processed: 8                                     │
//! │ Average per node: 0.29s                               │
//! │ Parallel nodes: 3                                      │
//! │ Database operations: 2                                 │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Troubleshooting
//!
//! ### Common Issues
//!
//! 1. **Missing API Keys**
//!    ```bash
//!    Error: Environment variable ANTHROPIC_API_KEY not found
//!    Solution: Set your API keys in .env file
//!    ```
//!
//! 2. **Database Connection Errors**
//!    ```bash
//!    Error: Connection to database failed
//!    Solution: Check DATABASE_URL and ensure PostgreSQL is running
//!    ```
//!
//! 3. **MCP Server Unavailable**
//!    ```bash
//!    Warning: MCP server connection failed, using fallback mode
//!    Solution: Start required MCP servers or disable MCP in config
//!    ```
//!
//! ### Debug Mode
//!
//! ```bash
//! # Run with debug logging
//! RUST_LOG=debug cargo run
//!
//! # Run with trace logging for detailed output
//! RUST_LOG=trace cargo run
//! ```
//!
//! ## Next Steps
//!
//! After running the demos:
//!
//! 1. **Explore the Code**: Review the workflow implementations in detail
//! 2. **Customize Workflows**: Modify existing workflows for your use case
//! 3. **Add New Nodes**: Create custom processing nodes
//! 4. **Integrate MCP**: Set up external MCP servers for enhanced capabilities
//! 5. **Deploy**: Use the workflows in production applications
//!
//! For more detailed information, see the individual module documentation and the
//! comprehensive examples in the [`demos`] module.

use diesel::PgConnection;
use serde_json::Value;

use workflow_engine_core::{error::WorkflowError, workflow::Workflow};
use crate::db::event::{Event, NewEvent};

// Import extension traits
use self::event_integration::{WorkflowEventExt, TaskContextEventExt};

pub mod customer_support_workflow;
pub mod demos;
pub mod executor;
pub mod knowledge_base_workflow;
pub mod nodes;
pub mod parser;
pub mod registry;
pub mod schema;
pub mod event_integration;

pub struct WorkflowRunner {
    workflow: Workflow,
}

impl WorkflowRunner {
    pub fn new(workflow: Workflow) -> Self {
        Self { workflow }
    }

    /// Process an event from the database
    pub fn process_event(
        &self,
        event: &Event,
        _conn: &mut PgConnection,
    ) -> Result<Event, WorkflowError> {
        // Run the workflow
        let task_context = self.workflow.run_from_event(event)?;

        // Convert task context to JSON for storage
        let task_context_json = task_context.to_event()?;
        
        // Create updated event with the task context
        let updated_event = Event {
            id: event.id,
            workflow_type: event.workflow_type.clone(),
            data: event.data.clone(),
            task_context: task_context_json,
            created_at: event.created_at,
            updated_at: chrono::Utc::now(),
        };

        // Event persistence not implemented - requires Event::store method implementation
        // Currently returns in-memory event without database persistence
        // updated_event.store(conn)?;

        Ok(updated_event)
    }

    /// Create and process a new event
    pub fn create_and_process(
        &self,
        event_data: Value,
        conn: &mut PgConnection,
    ) -> Result<Event, WorkflowError> {
        // Create initial event
        let event = NewEvent::new(
            event_data,
            self.workflow.workflow_type().to_string(),
            Value::Null,
        );
        
        // Store the event
        event.store(conn).map_err(|e| WorkflowError::DatabaseError {
            message: format!("Failed to store event: {}", e),
        })?;

        // Process it
        self.process_event(&event, conn)
    }
}
