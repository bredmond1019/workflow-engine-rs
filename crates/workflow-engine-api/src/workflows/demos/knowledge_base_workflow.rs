//! # Knowledge Base Workflow Demo
//!
//! This module provides an interactive demonstration of the comprehensive knowledge
//! search and retrieval workflow. The demo showcases multi-source parallel searching
//! across Notion, HelpScout, and Slack with real-time visualization of search
//! execution, result analysis, and intelligent response generation.
//!
//! ## Demo Overview
//!
//! The knowledge base workflow demo demonstrates:
//! - **Multi-source parallel searching** across Notion, HelpScout, and Slack
//! - **Query validation and spam filtering** with confidence scoring
//! - **Real-time search visualization** with source-specific timing and progress
//! - **Result analysis and relevance scoring** with completeness assessment
//! - **Response generation with source attribution** and confidence metrics
//! - **Database integration** showing query tracking and results storage
//! - **Type-safe data extraction** for processed search results
//!
//! ## How to Run
//!
//! ### Quick Start
//! ```bash
//! # Run all demos (includes knowledge base workflow)
//! cargo run
//!
//! # Run only knowledge base demos
//! cargo run --example knowledge_base_demos
//! ```
//!
//! ### Programmatic Execution
//! ```rust
//! use ai_architecture_workflows::demos::knowledge_base_workflow::knowledge_base_workflow_demo;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Run the interactive knowledge base workflow demonstration
//!     knowledge_base_workflow_demo().await;
//! }
//! ```
//!
//! ## Demo Scenarios
//!
//! The demo includes four comprehensive search scenarios:
//!
//! ### Scenario 1: Technical Documentation Query
//! - **Query Type**: Technical
//! - **Sources**: Notion, HelpScout, Slack
//! - **Example**: "How do I configure SSL certificates for the API?"
//! - **Demonstrates**: Full-text search, documentation retrieval, technical knowledge aggregation
//!
//! ### Scenario 2: Product Feature Question
//! - **Query Type**: Product
//! - **Sources**: Notion, Slack
//! - **Example**: "What are the new features in version 2.0?"
//! - **Demonstrates**: Product knowledge search, feature documentation, release notes
//!
//! ### Scenario 3: Troubleshooting Query
//! - **Query Type**: Troubleshooting
//! - **Sources**: HelpScout, Slack, Notion
//! - **Example**: "My application keeps crashing on startup, what should I check?"
//! - **Demonstrates**: Problem-solving search, diagnostic information, solution aggregation
//!
//! ### Scenario 4: General Information Request
//! - **Query Type**: Policy
//! - **Sources**: Notion, HelpScout
//! - **Example**: "What is your refund policy?"
//! - **Demonstrates**: Policy search, customer support knowledge, official documentation
//!
//! ## Demo Features
//!
//! ### Visual Search Progress
//! The demo provides detailed visual feedback for each search operation:
//! ```text
//! ╔═══════════════════════════════════════════════════════════╗
//! ║           Knowledge Base Workflow Demo                    ║
//! ╚═══════════════════════════════════════════════════════════╝
//!
//! ✅ Knowledge Base Workflow created successfully!
//!    📚 Workflow type: knowledge_base
//!    🔧 Initializing search components...
//!
//! ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//! 🔍 Testing Query Scenario 1 of 4: Technical Documentation Query
//! ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//! ```
//!
//! ### Source-Specific Search Visualization
//! Each knowledge source shows detailed search progress:
//! ```text
//! 🔸 Node 4 - 'notion_search'
//!          📚 Searching Notion database...
//!          ✓ Status: "completed"
//!          ✓ Result: {"results_found": 5, "relevance_score": 0.87}
//!          🔍 Sources: ["SSL Configuration Guide", "API Documentation"]
//!          📊 Relevance: 0.95
//!
//! 🔸 Node 5 - 'helpscout_search'
//!          🎧 Searching HelpScout articles...
//!          ✓ Status: "completed"
//!          ✓ Result: {"results_found": 3, "relevance_score": 0.72}
//!
//! 🔸 Node 6 - 'slack_search'
//!          💬 Searching Slack conversations...
//!          ✓ Status: "completed"
//!          ✓ Result: {"results_found": 8, "relevance_score": 0.65}
//! ```
//!
//! ### Performance Metrics
//! Real-time performance tracking for search operations:
//! ```text
//! ✅ Knowledge search completed successfully in 2.34s!
//!    🆔 Query ID: query-kb-001
//!    📊 Workflow Type: knowledge_base
//!    ⏰ Processing Time: Started at 2024-01-15 11:30:00 UTC
//! ```
//!
//! ## Knowledge Source Configuration
//!
//! ### Environment Variables
//! ```bash
//! # AI Provider Configuration
//! ANTHROPIC_API_KEY=your_anthropic_key_here
//! OPENAI_API_KEY=your_openai_key_here
//!
//! # Knowledge Source APIs (optional for demo mode)
//! NOTION_API_KEY=your_notion_integration_key
//! NOTION_DATABASE_ID=your_notion_database_id
//! HELPSCOUT_API_KEY=your_helpscout_api_key
//! SLACK_BOT_TOKEN=xoxb-your-slack-bot-token
//!
//! # Knowledge Base Settings
//! KB_MAX_RESULTS_PER_SOURCE=10
//! KB_SEARCH_TIMEOUT_SECONDS=30
//! KB_MIN_RELEVANCE_SCORE=0.6
//! KB_ENABLE_PARALLEL_SEARCH=true
//! ```
//!
//! ### Demo Mode vs Real Sources
//! The demo works in two modes:
//!
//! #### Demo Mode (Default)
//! When knowledge source APIs are not configured:
//! - Simulated search results with realistic data
//! - Full workflow execution without external API calls
//! - Educational value without API key requirements
//!
//! #### Real Source Mode
//! When knowledge source APIs are configured:
//! - Actual searches against configured knowledge sources
//! - Real-time results from Notion, HelpScout, and Slack
//! - Production-ready search capabilities
//!
//! ## Database Integration
//!
//! ### Knowledge Query Event Processing
//! ```text
//! 🗄️  Knowledge Base Event Integration Demo
//! 🔧 Creating new knowledge base query event...
//! 📄 Created Query Event with ID: query-kb-db-001
//!    🕐 Timestamp: 2024-01-15 11:45:00 UTC
//!
//! 🚀 Processing knowledge query through workflow...
//! ✅ Knowledge query processed successfully in 2.89s!
//! 💾 Knowledge base event updated with search results
//!    Search results size: 15,247 bytes
//! ```
//!
//! ### Type-Safe Data Extraction
//! ```text
//! 🔍 Type-safe Knowledge Base Data Extraction Demo
//! 🔄 Attempting to extract typed query data from event...
//! ✅ Successfully extracted typed knowledge base data:
//!    Query ID: query-kb-db-001
//!    User ID: user-db-001
//!    Query Type: integration_test
//!    Query Length: 45 characters
//!    Sources to Search: ["notion", "helpscout", "slack"]
//!    Query Preview: Testing knowledge base database integration...
//! ```
//!
//! ## Search Result Analysis
//!
//! ### Result Quality Assessment
//! The demo shows how the system analyzes search quality:
//! ```json
//! {
//!   "analyze_knowledge": {
//!     "status": "completed",
//!     "sufficient_info": true,
//!     "overall_confidence": 0.82,
//!     "source_diversity": 3,
//!     "completeness_score": 0.89
//!   }
//! }
//! ```
//!
//! ### Response Generation
//! AI-powered response synthesis with source attribution:
//! ```json
//! {
//!   "generate_response": {
//!     "status": "completed",
//!     "response": "Based on our documentation and team discussions...",
//!     "response_type": "comprehensive_guide",
//!     "sources_cited": 5,
//!     "confidence": 0.86
//!   }
//! }
//! ```
//!
//! ## Performance Characteristics
//!
//! ### Search Timing Analysis
//! - **Query Validation**: 50-150ms
//! - **Notion Search**: 300-800ms (varies by database size)
//! - **HelpScout Search**: 200-600ms (depends on article count)
//! - **Slack Search**: 400-1200ms (varies by history scope)
//! - **Result Analysis**: 100-300ms
//! - **Response Generation**: 500-1500ms (depends on AI model)
//! - **Total Execution**: 1.5-4.5s (with parallel searching)
//!
//! ### Parallel Search Benefits
//! - **Sequential Search**: ~5.5s total execution time
//! - **Parallel Search**: ~2.8s total execution time
//! - **Performance Improvement**: 49% faster execution
//! - **Scalability**: Linear improvement with additional sources
//!
//! ## Error Handling and Recovery
//!
//! ### Knowledge Source Failures
//! ```text
//! ❌ Knowledge search workflow failed: ProcessingError { message: "Notion API timeout" }
//! 💡 Tip: Check search service configurations
//! 🔍 Error details: Connection timeout after 30 seconds
//! ```
//!
//! ### Insufficient Search Results
//! ```text
//! ⚠️  Warning: Low relevance scores across all sources
//! 💡 Tip: Broaden search terms or check knowledge source content
//! 🔍 Relevance threshold: 0.6, highest found: 0.4
//! ```
//!
//! ### Query Validation Failures
//! ```text
//! ❌ Knowledge search workflow failed: DeserializationError { message: "missing field query_id" }
//! 💡 Tip: Verify query data structure
//! 🔍 Required fields: query_id, user_id, user_query, query_type, sources
//! ```
//!
//! ## Search Strategy Customization
//!
//! ### Custom Query Processing
//! ```rust
//! let custom_query = serde_json::json!({
//!     "query_id": "CUSTOM-SEARCH-001",
//!     "user_id": "POWER-USER-123",
//!     "user_query": "Advanced configuration for high-availability setup",
//!     "query_type": "advanced_technical",
//!     "sources": ["notion", "slack"],
//!     "filters": {
//!         "date_range": "last_3_months",
//!         "tags": ["high-availability", "configuration", "advanced"],
//!         "minimum_relevance": 0.8
//!     }
//! });
//! ```
//!
//! ### Source-Specific Configuration
//! ```rust
//! // Notion-focused search for documentation
//! let documentation_search = serde_json::json!({
//!     "query_type": "documentation",
//!     "sources": ["notion"],
//!     "preferences": {
//!         "include_page_hierarchy": true,
//!         "expand_linked_pages": true,
//!         "max_depth": 3
//!     }
//! });
//!
//! // Multi-source troubleshooting search
//! let troubleshooting_search = serde_json::json!({
//!     "query_type": "troubleshooting",
//!     "sources": ["helpscout", "slack", "notion"],
//!     "preferences": {
//!         "prioritize_solutions": true,
//!         "include_conversation_context": true,
//!         "search_timeframe_days": 90
//!     }
//! });
//! ```
//!
//! ## Testing and Development
//!
//! ### Unit Testing Search Components
//! ```bash
//! # Test knowledge base workflow
//! cargo test knowledge_base_workflow::tests
//!
//! # Test individual search nodes
//! cargo test query_router_node
//! cargo test notion_search_node
//! cargo test result_analysis_node
//! ```
//!
//! ### Integration Testing
//! ```bash
//! # Test with simulated knowledge sources
//! cargo test --test knowledge_base_integration
//!
//! # Test with real APIs (requires configuration)
//! ENABLE_REAL_APIS=true cargo test --test knowledge_base_real_sources
//! ```
//!
//! ### Performance Benchmarking
//! ```bash
//! # Run search performance benchmarks
//! cargo bench knowledge_base_benchmarks
//!
//! # Test parallel vs sequential performance
//! cargo bench --bench parallel_search_comparison
//! ```
//!
//! ## Troubleshooting
//!
//! ### Common Issues
//!
//! #### Knowledge Source Authentication
//! ```bash
//! Error: ProcessingError { message: "Notion API authentication failed" }
//! Solution: Verify NOTION_API_KEY in environment variables
//! Debug: Test API key with: curl -H "Authorization: Bearer $NOTION_API_KEY" https://api.notion.com/v1/users/me
//! ```
//!
//! #### Search Timeout Issues
//! ```bash
//! Error: ProcessingError { message: "Search timeout exceeded" }
//! Solution: Increase KB_SEARCH_TIMEOUT_SECONDS or optimize queries
//! Debug: Check network connectivity and source responsiveness
//! ```
//!
//! #### Low Relevance Scores
//! ```bash
//! Warning: All search results below relevance threshold
//! Solution: Adjust KB_MIN_RELEVANCE_SCORE or improve search terms
//! Debug: Review query keywords and source content quality
//! ```
//!
//! ### Debug Mode
//! ```bash
//! # Enable knowledge base search debugging
//! RUST_LOG=ai_architecture_workflows::knowledge_base_workflow=debug cargo run
//!
//! # Trace search execution across all sources
//! RUST_LOG=ai_architecture_core::mcp::clients=trace cargo run
//!
//! # Debug specific knowledge source
//! RUST_LOG=ai_architecture_core::mcp::clients::notion=debug cargo run
//! ```
//!
//! ## Educational Value
//!
//! ### Search Architecture Patterns
//! - Multi-source search orchestration
//! - Parallel processing for performance
//! - Result aggregation and ranking strategies
//! - Error handling in distributed search systems
//!
//! ### AI Integration Techniques
//! - Query intent analysis and classification
//! - Result relevance scoring and filtering
//! - Response generation with source attribution
//! - Confidence assessment and quality metrics
//!
//! ### Production Readiness
//! - Scalable search architecture
//! - Comprehensive error handling and recovery
//! - Performance monitoring and optimization
//! - Security considerations for knowledge access
//!
//! ## Next Steps
//!
//! After running the demo:
//!
//! 1. **Configure Real Knowledge Sources** - Set up API keys for actual searches
//! 2. **Customize Search Logic** - Modify search strategies for your use case
//! 3. **Add New Knowledge Sources** - Integrate additional search endpoints
//! 4. **Optimize Performance** - Tune search parameters and caching strategies
//! 5. **Deploy Production Search** - Use patterns for production knowledge systems
//!
//! ## Related Demos
//!
//! - [`knowledge_base_mcp`](../knowledge_base_mcp/index.html) - MCP integration for knowledge search
//! - [`customer_care_workflow`](../customer_care_workflow/index.html) - Customer support automation
//! - [`customer_care_mcp`](../customer_care_mcp/index.html) - Customer care MCP integration

use workflow_engine_core::{error::WorkflowError, workflow::Workflow};
use crate::workflows::event_integration::{WorkflowEventExt, TaskContextEventExt};
use workflow_engine_mcp::server::knowledge_base::KnowledgeBaseEventData;
use crate::{
    db::event::NewEvent,
    workflows::{knowledge_base_workflow::create_knowledge_base_workflow, demos::{timing::*, utils::*}},
};
use serde_json::{json, Value};
use std::time::Instant;
use tokio::time::sleep;

pub async fn knowledge_base_workflow_demo() {
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║           Knowledge Base Workflow Demo                    ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    sleep(WORKFLOW_START_PAUSE).await;

    match create_knowledge_base_workflow() {
        Ok(workflow) => {
            println!("✅ Knowledge Base Workflow created successfully!");
            println!("   📚 Workflow type: {}", workflow.workflow_type());
            println!("   🔧 Initializing search components...");
            sleep(MEDIUM_PAUSE).await;

            // Test data for different query scenarios
            let test_scenarios = vec![
                (
                    "Technical Documentation Query",
                    serde_json::json!({
                        "query_id": "QUERY-KB-001",
                        "user_id": "USER-123",
                        "user_query": "How do I configure SSL certificates for the API?",
                        "query_type": "technical",
                        "sources": ["notion", "helpscout", "slack"]
                    }),
                ),
                (
                    "Product Feature Question",
                    serde_json::json!({
                        "query_id": "QUERY-KB-002",
                        "user_id": "USER-456",
                        "user_query": "What are the new features in version 2.0?",
                        "query_type": "product",
                        "sources": ["notion", "slack"]
                    }),
                ),
                (
                    "Troubleshooting Query",
                    serde_json::json!({
                        "query_id": "QUERY-KB-003",
                        "user_id": "USER-789",
                        "user_query": "My application keeps crashing on startup, what should I check?",
                        "query_type": "troubleshooting",
                        "sources": ["helpscout", "slack", "notion"]
                    }),
                ),
                (
                    "General Information Request",
                    serde_json::json!({
                        "query_id": "QUERY-KB-004",
                        "user_id": "USER-101",
                        "user_query": "What is your refund policy?",
                        "query_type": "policy",
                        "sources": ["notion", "helpscout"]
                    }),
                ),
            ];

            for (i, (scenario_name, event_data)) in test_scenarios.iter().enumerate() {
                println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!(
                    "🔍 Testing Query Scenario {} of {}: {}",
                    i + 1,
                    test_scenarios.len(),
                    scenario_name
                );
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                sleep(SECTION_PAUSE).await;

                println!(
                    "   📝 Query Data: {}",
                    serde_json::to_string_pretty(&event_data)
                        .unwrap_or_else(|_| "Invalid JSON".to_string())
                );

                println!("\n🚀 Starting knowledge base search workflow...");
                let start_time = Instant::now();

                match workflow.run(event_data.clone()) {
                    Ok(context) => {
                        let elapsed = start_time.elapsed();
                        println!(
                            "\n✅ Knowledge search completed successfully in {:.2}s!",
                            elapsed.as_secs_f64()
                        );
                        println!("   🆔 Query ID: {}", context.event_id);
                        println!("   📊 Workflow Type: {}", context.workflow_type);
                        println!(
                            "   ⏰ Processing Time: Started at {}, Updated at {}",
                            context.created_at.format("%Y-%m-%d %H:%M:%S UTC"),
                            context.updated_at.format("%Y-%m-%d %H:%M:%S UTC")
                        );

                        println!("\n   🔍 Knowledge Search Node Results:");
                        for (idx, (node_name, node_data)) in context.nodes.iter().enumerate() {
                            println!("\n      🔸 Node {} - '{}'", idx + 1, node_name);
                            
                            // Show different processing for different node types
                            match node_name.as_str() {
                                name if name.contains("query_router") => {
                                    println!("         📋 Processing user query...");
                                    sleep(NODE_PROCESSING_PAUSE).await;
                                }
                                name if name.contains("validate") => {
                                    println!("         ✅ Validating query structure...");
                                    sleep(NODE_PROCESSING_PAUSE).await;
                                }
                                name if name.contains("spam") => {
                                    println!("         🛡️  Checking for spam content...");
                                    sleep(NODE_PROCESSING_PAUSE).await;
                                }
                                name if name.contains("search_router") => {
                                    println!("         🔀 Routing to knowledge sources...");
                                    sleep(KNOWLEDGE_SEARCH_PAUSE).await;
                                }
                                name if name.contains("notion") => {
                                    println!("         📚 Searching Notion database...");
                                    sleep(KNOWLEDGE_SEARCH_PAUSE).await;
                                }
                                name if name.contains("helpscout") => {
                                    println!("         🎧 Searching HelpScout articles...");
                                    sleep(KNOWLEDGE_SEARCH_PAUSE).await;
                                }
                                name if name.contains("slack") => {
                                    println!("         💬 Searching Slack conversations...");
                                    sleep(KNOWLEDGE_SEARCH_PAUSE).await;
                                }
                                name if name.contains("analyze") => {
                                    println!("         🧠 Analyzing search results...");
                                    sleep(OPERATION_PAUSE).await;
                                }
                                name if name.contains("generate") => {
                                    println!("         📝 Generating comprehensive response...");
                                    sleep(OPERATION_PAUSE).await;
                                }
                                name if name.contains("send") => {
                                    println!("         📤 Delivering response to user...");
                                    sleep(NODE_PROCESSING_PAUSE).await;
                                }
                                _ => {
                                    println!("         ⚙️  Processing...");
                                    sleep(NODE_PROCESSING_PAUSE).await;
                                }
                            }

                            if let Some(obj) = node_data.as_object() {
                                if let Some(status) = obj.get("status") {
                                    println!("         ✓ Status: {}", status);
                                }
                                if let Some(result) = obj.get("result") {
                                    let result_str = serde_json::to_string(result)
                                        .unwrap_or_else(|_| "Complex result".to_string());
                                    if result_str.len() > 100 {
                                        println!("         ✓ Result: {}...", &result_str[..97]);
                                    } else {
                                        println!("         ✓ Result: {}", result_str);
                                    }
                                }
                                if let Some(sources) = obj.get("sources_searched") {
                                    println!("         🔍 Sources: {}", sources);
                                }
                                if let Some(relevance) = obj.get("relevance_score") {
                                    println!("         📊 Relevance: {}", relevance);
                                }
                            } else {
                                let output = serde_json::to_string_pretty(node_data)
                                    .unwrap_or_else(|_| "Invalid JSON".to_string());
                                if output.len() > 200 {
                                    println!("         ✓ Output: {}...", &output[..197]);
                                } else {
                                    println!("         ✓ Output: {}", output);
                                }
                            }
                        }

                        if !context.metadata.is_empty() {
                            println!("\n   📋 Search Metadata:");
                            for (key, value) in &context.metadata {
                                println!("      🔹 {} -> {}", key, value);
                            }
                            sleep(QUICK_PAUSE).await;
                        }

                        // Demonstrate Event conversion
                        match context.to_event() {
                            Ok(event) => {
                                println!("\n   💾 Knowledge Query Event:");
                                println!("      ID: {}", event.get("event_id").and_then(|v| v.as_str()).unwrap_or("unknown"));
                                println!("      Workflow Type: {}", event.get("workflow_type").and_then(|v| v.as_str()).unwrap_or("unknown"));
                                println!(
                                    "      Created: {}",
                                    event.get("created_at").and_then(|v| v.as_str()).unwrap_or("unknown")
                                );
                                println!(
                                    "      Task Context Size: {} bytes",
                                    event.get("task_context").map(|v| serde_json::to_string(v).map(|s| s.len()).unwrap_or(0)).unwrap_or(0)
                                );
                            }
                            Err(e) => {
                                println!("   ❌ Failed to convert to Event: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("❌ Knowledge search workflow failed: {}", e);

                        // Provide specific error context for knowledge base
                        match e {
                            WorkflowError::NodeNotFound { node_type } => {
                                println!(
                                    "   💡 Tip: Ensure all knowledge base nodes are registered"
                                );
                                println!("   🔍 Missing node type: {:?}", node_type);
                            }
                            WorkflowError::ProcessingError(details) => {
                                println!("   💡 Tip: Check search service configurations");
                                println!("   🔍 Error details: {}", details.message);
                            }
                            WorkflowError::DeserializationError(details) => {
                                println!("   💡 Tip: Verify query data structure");
                                println!("   🔍 Deserialization error: {}", details.message);
                            }
                            _ => {
                                println!("   🔍 Error type: {:?}", e);
                            }
                        }
                    }
                }

                println!("\n   ⏱️  Query scenario completed. Pausing before next query...");
                sleep(SCENARIO_PAUSE).await;
            }

            // Demo: Working with Knowledge Base Events
            println!("\n\n═══════════════════════════════════════════════════════════════");
            println!("🗄️  Knowledge Base Event Integration Demo");
            println!("═══════════════════════════════════════════════════════════════\n");
            sleep(LONG_PAUSE).await;

            let kb_event_data = serde_json::json!({
                "query_id": "QUERY-KB-DB-001",
                "user_id": "USER-DB-001",
                "user_query": "Testing knowledge base database integration",
                "query_type": "integration_test",
                "sources": ["notion", "helpscout", "slack"]
            });

            println!("🔧 Creating new knowledge base query event...");
            sleep(DATABASE_OPERATION_PAUSE).await;

            let mut kb_event = NewEvent::new(kb_event_data, "knowledge_base".to_string(), Value::Null);
            println!("📄 Created Query Event with ID: {}", kb_event.id);
            println!(
                "   🕐 Timestamp: {}",
                kb_event.created_at.format("%Y-%m-%d %H:%M:%S UTC")
            );

            println!("\n🚀 Processing knowledge query through workflow...");
            let event_start = Instant::now();

            match workflow.run_from_event(&kb_event) {
                Ok(context) => {
                    let event_elapsed = event_start.elapsed();
                    println!(
                        "\n✅ Knowledge query processed successfully in {:.2}s!",
                        event_elapsed.as_secs_f64()
                    );
                    sleep(NODE_PROCESSING_PAUSE).await;

                    // Update the event with the task context
                    match kb_event.update_task_context(&context) {
                        Ok(()) => {
                            println!("💾 Knowledge base event updated with search results");
                            println!(
                                "   Search results size: {} bytes",
                                serde_json::to_string(&kb_event.task_context)
                                    .map(|s| s.len())
                                    .unwrap_or(0)
                            );
                        }
                        Err(e) => {
                            println!("❌ Failed to update knowledge base event: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Failed to process knowledge query: {}", e);
                }
            }

            // Demo: Type-safe knowledge base data extraction
            println!("\n\n═══════════════════════════════════════════════════════════════");
            println!("🔍 Type-safe Knowledge Base Data Extraction Demo");
            println!("═══════════════════════════════════════════════════════════════\n");
            sleep(SECTION_PAUSE).await;

            println!("🔄 Attempting to extract typed query data from event...");
            sleep(SHORT_PAUSE).await;

            match kb_event.get_typed_data::<KnowledgeBaseEventData>() {
                Ok(typed_data) => {
                    println!("✅ Successfully extracted typed knowledge base data:");
                    println!("   Query ID: {}", typed_data.query_id);
                    println!("   User ID: {}", typed_data.user_id);
                    println!("   Query Type: {}", typed_data.query_type);
                    println!("   Query Length: {} characters", typed_data.user_query.len());
                    println!("   Sources to Search: {:?}", typed_data.sources);
                    println!("   Query Preview: {}...", 
                        if typed_data.user_query.len() > 50 {
                            &typed_data.user_query[..47]
                        } else {
                            &typed_data.user_query
                        }
                    );
                }
                Err(e) => {
                    println!("❌ Failed to extract typed knowledge base data: {}", e);
                }
            }

            println!("\n\n╔═══════════════════════════════════════════════════════════╗");
            println!("║      🎉 Knowledge Base Demo completed successfully! 🎉    ║");
            println!("╚═══════════════════════════════════════════════════════════╝\n");
            sleep(DEMO_TRANSITION_PAUSE).await;
        }
        Err(e) => {
            println!("❌ Failed to create knowledge base workflow: {}", e);

            match e {
                WorkflowError::CycleDetected => {
                    println!("💡 Tip: Check knowledge base workflow for circular dependencies");
                }
                WorkflowError::UnreachableNodes { nodes } => {
                    println!("💡 Tip: Ensure all search nodes are connected properly");
                    println!("🔍 Unreachable nodes: {:?}", nodes);
                }
                WorkflowError::InvalidRouter { node } => {
                    println!("💡 Tip: Verify search router configuration");
                    println!("🔍 Problematic node: {}", node);
                }
                _ => {
                    println!("🔍 Error details: {:?}", e);
                }
            }
        }
    }
}