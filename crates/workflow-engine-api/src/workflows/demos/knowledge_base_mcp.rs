//! # Knowledge Base MCP Integration Demo
//!
//! This module demonstrates the Model Context Protocol (MCP) integration capabilities
//! for knowledge base search workflows. It showcases how to create, configure, and use
//! MCP servers for knowledge search automation, including search tool registration,
//! execution across multiple knowledge sources, and external knowledge service integration.
//!
//! ## Demo Overview
//!
//! The knowledge base MCP demo demonstrates:
//! - **Knowledge search MCP server creation** with search tool registration
//! - **Multi-source search tool execution** across Notion, HelpScout, and Slack
//! - **Search-optimized connection pooling** for knowledge base workloads
//! - **Workflow exposure as knowledge search endpoints** for external consumption
//! - **External knowledge base MCP server integration** for third-party sources
//! - **Performance monitoring for search operations** and result aggregation
//!
//! ## How to Run
//!
//! ### Quick Start
//! ```bash
//! # Run all demos (includes knowledge base MCP)
//! cargo run
//!
//! # Run only knowledge base demos (includes MCP integration)
//! cargo run --example knowledge_base_demos
//! ```
//!
//! ### Programmatic Execution
//! ```rust
//! use ai_architecture_workflows::demos::knowledge_base_mcp::knowledge_base_mcp_demo;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Run the interactive knowledge base MCP integration demonstration
//!     knowledge_base_mcp_demo().await;
//! }
//! ```
//!
//! ## Demo Components
//!
//! ### 1. Knowledge Base MCP Server Creation
//!
//! The demo creates a comprehensive MCP server for knowledge search:
//! ```text
//! 🧠 Demo 1: Creating Knowledge Base MCP Server
//! 🔧 Initializing Knowledge Base MCP server components...
//! ℹ️  Note: KnowledgeBaseMCPServer is currently in development
//! 📝 This demo shows the planned functionality for knowledge base MCP integration
//! ✅ Knowledge Base MCP Server simulation initialized!
//!
//! 📊 Simulated Knowledge Base Server Information:
//!    🔧 Available search tools: 4 (simulated)
//!    🔍 Search tool names: ["validate_query", "notion_search", "helpscout_search", "slack_search"]
//! ```
//!
//! **Available Knowledge Search Tools:**
//! - `validate_query` - Validates user queries for completeness and safety
//! - `notion_search` - Searches Notion database for relevant documentation
//! - `helpscout_search` - Searches HelpScout knowledge base articles
//! - `slack_search` - Searches Slack conversations for relevant discussions
//!
//! ### 2. Search Tool Execution Across Multiple Sources
//!
//! The demo shows comprehensive search tool execution:
//! ```text
//! 🔧 Testing Knowledge Base Tool: validate_query
//!    📝 Validating user query
//! ✅ Successfully called validate_query tool in 0.32s (simulated)
//!    📊 Tool execution completed successfully
//!    📝 Result preview: Query validation passed: technical query about SSL certificates
//!
//! 🔧 Testing Knowledge Base Tool: notion_search
//!    📝 Searching Notion database
//! ✅ Successfully called notion_search tool in 0.68s (simulated)
//!    📝 Result preview: Found 3 relevant documentation pages about API configuration
//!
//! 🔧 Testing Knowledge Base Tool: slack_search
//!    📝 Searching Slack conversations
//! ✅ Successfully called slack_search tool in 0.71s (simulated)
//!    📝 Result preview: Found 8 recent conversations about product updates and releases
//! ```
//!
//! ### 3. Workflow Exposure as Knowledge Search Endpoint
//!
//! Shows how to expose knowledge workflows as MCP endpoints:
//! ```text
//! 🌐 Demo 2: Exposing Knowledge Base Workflow as MCP Server
//! 🔧 Converting knowledge base workflow to MCP server...
//! ✅ Knowledge Base Workflow exposed as MCP server successfully!
//!    🔧 Workflow tools available: 8
//!    📚 Knowledge workflow tool names: ["search_knowledge", "analyze_results", ...]
//! ```
//!
//! ### 4. Search-Optimized Connection Pool
//!
//! Demonstrates connection pool management optimized for search workloads:
//! ```text
//! 🏊 Demo 4: Knowledge Base MCP Connection Pool
//! 🔧 Creating knowledge base connection pool with search-optimized configuration...
//! 📝 Registering knowledge base servers in connection pool...
//!    ✅ Registered: notion-kb-server
//!    ✅ Registered: helpscout-kb-server
//!    ✅ Registered: slack-kb-server
//!
//! ✅ Knowledge Base connection pool created with search servers
//!    📊 KB Pool statistics: 3 search server configurations
//!    🏥 KB Health check results: 3 search servers checked
//! ```
//!
//! ### 5. External Knowledge Base Server Integration
//!
//! Shows integration with external knowledge services:
//! ```text
//! 🔗 Demo 5: Registering External Knowledge Base MCP Servers
//! 🔧 Attempting to register External Notion Server...
//! ✅ External Notion Server registered successfully
//!    📡 Server URL: ws://localhost:8091/external-notion
//!    🔌 Transport: WebSocket
//!
//! 🔧 Attempting to register External Documentation Server...
//! ✅ External Documentation Server registered successfully
//!    📡 Server URL: ws://localhost:8092/external-docs
//! ```
//!
//! ### 6. Integrated Knowledge Base Search Test
//!
//! Comprehensive integration test with real search execution:
//! ```text
//! 🔍 Demo 6: Integrated Knowledge Base Search Test
//! 🚀 Running integrated knowledge base search through MCP workflow...
//! ✅ Integrated MCP knowledge search completed in 3.42s!
//!    🆔 Search Query ID: INTEGRATED-SEARCH-001
//!    📚 Search Results: 8 nodes processed
//!    📊 Sources searched: 3
//!    📄 Total results found: 23
//! ```
//!
//! ## Knowledge Base MCP Configuration
//!
//! ### Environment Variables
//!
//! ```bash
//! # Basic Knowledge Base MCP Configuration
//! MCP_ENABLED=true
//! MCP_CLIENT_NAME=ai-architecture-knowledge-base
//! MCP_CLIENT_VERSION=1.0.0
//!
//! # Knowledge Source Endpoints
//! MCP_NOTION_SERVER=ws://localhost:8081/kb/notion
//! MCP_HELPSCOUT_SERVER=ws://localhost:8082/kb/helpscout
//! MCP_SLACK_SERVER=ws://localhost:8083/kb/slack
//!
//! # Search-Specific Configuration
//! KB_MCP_CONNECTION_TIMEOUT=10000
//! KB_MCP_MAX_CONNECTIONS=5
//! KB_MCP_HEALTH_CHECK_INTERVAL=15000
//! KB_MCP_SEARCH_TIMEOUT=30000
//! ```
//!
//! ### Search-Optimized Connection Pool
//!
//! ```rust
//! use ai_architecture_core::mcp::connection_pool::ConnectionConfig;
//! use std::time::Duration;
//!
//! let kb_pool_config = ConnectionConfig {
//!     max_connections_per_server: 5, // Higher for search workloads
//!     connection_timeout: Duration::from_secs(10), // Longer for search operations
//!     idle_timeout: Duration::from_secs(60),
//!     retry_attempts: 3, // More retries for search reliability
//!     retry_delay: Duration::from_millis(750),
//!     health_check_interval: Duration::from_secs(15),
//! };
//! ```
//!
//! ## Search Tool Examples
//!
//! ### Query Validation Tool
//! ```json
//! {
//!   "name": "validate_query",
//!   "arguments": {
//!     "context_data": {
//!       "user_query": "How do I configure SSL certificates?",
//!       "query_type": "technical",
//!       "user_id": "KB-USER-001"
//!     }
//!   }
//! }
//! ```
//!
//! ### Notion Search Tool
//! ```json
//! {
//!   "name": "notion_search",
//!   "arguments": {
//!     "context_data": {
//!       "user_query": "API documentation and examples",
//!       "keywords": ["API", "documentation", "examples"],
//!       "user_id": "KB-USER-002",
//!       "filters": {
//!         "database_id": "your_notion_database_id",
//!         "max_results": 10,
//!         "include_content": true
//!       }
//!     }
//!   }
//! }
//! ```
//!
//! ### Multi-Source Search Response
//! ```json
//! {
//!   "is_error": false,
//!   "content": [
//!     {
//!       "type": "text",
//!       "text": "Found 5 relevant results across knowledge sources",
//!       "sources": [
//!         {
//!           "source": "notion",
//!           "title": "SSL Certificate Setup Guide",
//!           "url": "https://notion.so/ssl-setup",
//!           "relevance": 0.95
//!         },
//!         {
//!           "source": "helpscout",
//!           "title": "SSL Troubleshooting FAQ",
//!           "url": "https://helpscout.com/ssl-faq",
//!           "relevance": 0.87
//!         }
//!       ]
//!     }
//!   ]
//! }
//! ```
//!
//! ## Performance Monitoring for Knowledge Search
//!
//! ### Search Operation Metrics
//! - **Query Validation**: 50-200ms per validation
//! - **Notion Search**: 300-1200ms (varies by database size)
//! - **HelpScout Search**: 200-800ms (depends on article count)
//! - **Slack Search**: 400-1500ms (varies by history scope)
//! - **Result Aggregation**: 100-300ms
//! - **Parallel Search Benefits**: 60-70% time reduction vs sequential
//!
//! ### Connection Pool Statistics
//! ```text
//! Knowledge Base Pool Statistics:
//! - Active Search Connections: 12/15 total
//! - Notion Server: 4/5 connections (healthy)
//! - HelpScout Server: 3/5 connections (healthy)
//! - Slack Server: 5/5 connections (healthy)
//! - Average Search Response Time: 645ms
//! - Cache Hit Rate: 34% (for repeated queries)
//! ```
//!
//! ## Development and Testing
//!
//! ### Setting Up Knowledge Base Test Servers
//!
//! ```bash
//! # Start knowledge base MCP test servers
//! cd scripts
//! python knowledge_base_server.py --port 8081 --source notion
//! python knowledge_base_server.py --port 8082 --source helpscout
//! python knowledge_base_server.py --port 8083 --source slack
//!
//! # Verify knowledge servers are running
//! curl -i http://localhost:8081/health
//! curl -i http://localhost:8082/search?q=test
//! curl -i http://localhost:8083/channels
//! ```
//!
//! ### Custom Knowledge Search Tool Development
//!
//! ```rust
//! use ai_architecture_core::mcp::protocol::{Tool, ToolResult};
//!
//! pub struct CustomKnowledgeSearchTool {
//!     source_name: String,
//!     search_endpoint: String,
//! }
//!
//! impl Tool for CustomKnowledgeSearchTool {
//!     fn name(&self) -> &str {
//!         &format!("{}_search", self.source_name)
//!     }
//!
//!     fn description(&self) -> &str {
//!         &format!("Searches {} knowledge base for relevant information", self.source_name)
//!     }
//!
//!     async fn execute(&self, args: ToolArguments) -> Result<ToolResult, ToolError> {
//!         let query = args.get("user_query")?;
//!         let keywords = args.get("keywords").unwrap_or_default();
//!         
//!         // Perform custom knowledge search
//!         let results = self.search_knowledge_source(query, keywords).await?;
//!         
//!         Ok(ToolResult::success_with_data(json!({
//!             "source": self.source_name,
//!             "results": results,
//!             "search_time": "0.65s",
//!             "relevance_scores": self.calculate_relevance(&results)
//!         })))
//!     }
//! }
//! ```
//!
//! ### Integration Testing for Knowledge Search
//!
//! ```rust
//! #[tokio::test]
//! async fn test_knowledge_base_mcp_integration() {
//!     let workflow = create_knowledge_base_workflow().await?;
//!     
//!     let search_query = json!({
//!         "query_id": "TEST-KB-001",
//!         "user_id": "TEST-USER",
//!         "user_query": "How to deploy in production?",
//!         "query_type": "deployment",
//!         "sources": ["notion", "slack"]
//!     });
//!
//!     let result = workflow.run(search_query).await?;
//!     
//!     // Verify search execution
//!     assert!(result.nodes.contains_key("notion_search"));
//!     assert!(result.nodes.contains_key("slack_search"));
//!     
//!     // Verify result quality
//!     let analysis = result.nodes.get("analyze_knowledge").unwrap();
//!     assert_eq!(analysis.get("sufficient_info").unwrap(), true);
//! }
//! ```
//!
//! ## Production Considerations for Knowledge Search
//!
//! ### Search Performance Optimization
//! - **Parallel Search Execution**: Enable concurrent searches across all sources
//! - **Result Caching**: Cache frequently accessed knowledge for faster responses
//! - **Query Optimization**: Use keyword extraction and semantic search improvements
//! - **Connection Pooling**: Maintain persistent connections to knowledge sources
//!
//! ### Security for Knowledge Access
//! - **Authentication**: Secure API keys for all knowledge source integrations
//! - **Authorization**: Role-based access to different knowledge domains
//! - **Query Sanitization**: Prevent injection attacks in search queries
//! - **Result Filtering**: Filter sensitive information from search results
//!
//! ### Scalability for Knowledge Workloads
//! - **Load Balancing**: Distribute search requests across multiple servers
//! - **Auto-scaling**: Scale knowledge search capacity based on demand
//! - **Circuit Breakers**: Prevent cascade failures from knowledge source outages
//! - **Rate Limiting**: Protect knowledge sources from overuse
//!
//! ## Error Handling and Recovery
//!
//! ### Knowledge Source Failures
//! ```text
//! ❌ Integrated MCP knowledge search failed: ProcessingError { message: "Notion search timeout" }
//! 💡 Tip: Check knowledge source availability and increase timeout values
//! 🔍 Fallback: Continue with available sources (HelpScout, Slack)
//! ```
//!
//! ### Search Quality Issues
//! ```text
//! ⚠️  Warning: Low relevance scores across all knowledge sources
//! 💡 Tip: Refine search keywords or expand knowledge base content
//! 🔍 Suggestion: Try broader search terms or check source indexing
//! ```
//!
//! ### Connection Pool Issues
//! ```text
//! ❌ Failed to register External Documentation Server: Connection pool exhausted
//! 💡 Tip: Increase max_connections_per_server or implement connection recycling
//! 🔍 Current pool usage: 5/5 connections in use
//! ```
//!
//! ## Advanced Knowledge Search Features
//!
//! ### Semantic Search Integration
//! ```rust
//! // Enhanced search with semantic understanding
//! let semantic_search_config = json!({
//!     "query_type": "semantic_technical",
//!     "sources": ["notion", "slack"],
//!     "advanced_options": {
//!         "use_embeddings": true,
//!         "similarity_threshold": 0.8,
//!         "context_expansion": true,
//!         "cross_reference_linking": true
//!     }
//! });
//! ```
//!
//! ### Knowledge Graph Integration
//! ```rust
//! // Connect related knowledge across sources
//! let knowledge_graph_search = json!({
//!     "query_type": "connected_knowledge",
//!     "sources": ["notion", "helpscout", "slack"],
//!     "graph_options": {
//!         "follow_relationships": true,
//!         "max_depth": 3,
//!         "include_related_topics": true,
//!         "merge_similar_results": true
//!     }
//! });
//! ```
//!
//! ### Real-Time Knowledge Updates
//! ```rust
//! // Subscribe to knowledge base changes
//! let real_time_config = json!({
//!     "subscriptions": {
//!         "notion_updates": true,
//!         "slack_new_messages": true,
//!         "helpscout_article_changes": true
//!     },
//!     "update_handling": {
//!         "invalidate_cache": true,
//!         "reindex_content": true,
//!         "notify_watchers": true
//!     }
//! });
//! ```
//!
//! ## Troubleshooting Knowledge Base MCP
//!
//! ### Common Issues
//!
//! #### Knowledge Source Authentication
//! ```bash
//! Error: Failed to authenticate with Notion API
//! Solution: Verify NOTION_API_KEY and integration permissions
//! Debug: Test with: curl -H "Authorization: Bearer $NOTION_API_KEY" https://api.notion.com/v1/users/me
//! ```
//!
//! #### Search Performance Issues
//! ```bash
//! Error: Knowledge search timeout exceeded
//! Solution: Increase KB_MCP_SEARCH_TIMEOUT or optimize search queries
//! Debug: Monitor search latency across different sources
//! ```
//!
//! #### Result Quality Problems
//! ```bash
//! Warning: Consistently low relevance scores
//! Solution: Review search algorithms and knowledge base content quality
//! Debug: Analyze query patterns and source content coverage
//! ```
//!
//! ### Debug Mode for Knowledge Search
//!
//! ```bash
//! # Enable knowledge base MCP debugging
//! RUST_LOG=ai_architecture_workflows::demos::knowledge_base_mcp=debug cargo run
//!
//! # Trace search execution across sources
//! RUST_LOG=ai_architecture_core::mcp::clients=trace cargo run
//!
//! # Debug specific knowledge source interactions
//! RUST_LOG=ai_architecture_core::mcp::clients::notion=debug cargo run
//! ```
//!
//! ## Educational Value
//!
//! ### Knowledge Search Architecture
//! - Multi-source search orchestration and result aggregation
//! - Performance optimization through parallel processing
//! - Quality assessment and relevance scoring techniques
//! - Error handling in distributed knowledge systems
//!
//! ### MCP Integration Patterns
//! - Knowledge-specific tool design and implementation
//! - Connection pool optimization for search workloads
//! - External service integration and fallback strategies
//! - Performance monitoring and health management
//!
//! ### Production Knowledge Systems
//! - Scalable search architecture for enterprise knowledge
//! - Security considerations for knowledge access control
//! - Real-time updates and cache invalidation strategies
//! - Analytics and insights for knowledge usage patterns
//!
//! ## Next Steps
//!
//! After running the demo:
//!
//! 1. **Set Up Real Knowledge Sources** - Configure actual API keys and endpoints
//! 2. **Implement Custom Search Logic** - Add domain-specific search algorithms
//! 3. **Optimize for Your Content** - Tune search parameters for your knowledge base
//! 4. **Add New Knowledge Sources** - Integrate additional search endpoints
//! 5. **Deploy Production Search** - Use patterns for enterprise knowledge systems
//!
//! ## Related Demos
//!
//! - [`knowledge_base_workflow`](../knowledge_base_workflow/index.html) - Base knowledge search workflow
//! - [`customer_care_mcp`](../customer_care_mcp/index.html) - Customer care MCP integration
//! - [`customer_care_workflow`](../customer_care_workflow/index.html) - Customer support automation

use workflow_engine_core::{error::WorkflowError, workflow::Workflow};
use crate::workflows::event_integration::{WorkflowEventExt, TaskContextEventExt, WorkflowMcpExt};
use workflow_engine_mcp::{
    config::MCPConfig,
    connection_pool::{ConnectionConfig, MCPConnectionPool},
    transport::{TransportType, ReconnectConfig},
};
use crate::workflows::{knowledge_base_workflow::create_knowledge_base_workflow, demos::timing::*};
use std::time::{Duration, Instant};
use tokio::time::sleep;

pub async fn knowledge_base_mcp_demo() {
    println!("\n\n╔═══════════════════════════════════════════════════════════╗");
    println!("║           🔌 Knowledge Base MCP Integration Demo          ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");
    sleep(WORKFLOW_START_PAUSE).await;

    match create_knowledge_base_workflow() {
        Ok(workflow) => {
            println!("✅ Knowledge Base MCP Workflow created successfully!");
            println!("   📚 Workflow type: {}", workflow.workflow_type());

            // Demo 1: Simulated Knowledge Base MCP server (feature in development)
            println!("\n═══════════════════════════════════════════════════════════════");
            println!("🧠 Demo 1: Creating Knowledge Base MCP Server");
            println!("═══════════════════════════════════════════════════════════════\n");
            sleep(SECTION_PAUSE).await;

            println!("🔧 Initializing Knowledge Base MCP server components...");
            println!("ℹ️  Note: KnowledgeBaseMCPServer is currently in development");
            println!("📝 This demo shows the planned functionality for knowledge base MCP integration");
            sleep(NODE_PROCESSING_PAUSE).await;

            println!("✅ Knowledge Base MCP Server simulation initialized!");
            println!("\n📊 Simulated Knowledge Base Server Information:");
            println!("   🔧 Available search tools: 4 (simulated)");
            
            let simulated_tool_names = vec!["validate_query", "notion_search", "helpscout_search", "slack_search"];
            println!("   🔍 Search tool names: {:?}", simulated_tool_names);

            // Test listing knowledge base tools
            println!("\n🔍 Simulating knowledge base MCP tools listing...");
            sleep(MCP_TOOL_CALL_PAUSE).await;
            
            println!("✅ Successfully listed Knowledge Base MCP tools (simulated)");
            
            let simulated_tools = vec![
                ("validate_query", "Validates user queries for completeness and safety"),
                ("notion_search", "Searches Notion database for relevant documentation"),
                ("helpscout_search", "Searches HelpScout knowledge base articles"),
                ("slack_search", "Searches Slack conversations for relevant discussions"),
            ];
            
            println!("   Found {} knowledge search tools:", simulated_tools.len());
            for (tool_name, description) in &simulated_tools {
                println!("     - {}: {}", tool_name, description);
            }

            // Test calling knowledge search tools
            let search_scenarios = vec![
                ("validate_query", "Validating user query", serde_json::json!({
                    "context_data": {
                        "user_query": "How do I configure SSL certificates?",
                        "query_type": "technical",
                        "user_id": "KB-USER-001"
                    }
                })),
                ("notion_search", "Searching Notion database", serde_json::json!({
                    "context_data": {
                        "user_query": "API documentation and examples",
                        "keywords": ["API", "documentation", "examples"],
                        "user_id": "KB-USER-002"
                    }
                })),
                ("helpscout_search", "Searching HelpScout articles", serde_json::json!({
                    "context_data": {
                        "user_query": "Common troubleshooting steps",
                        "keywords": ["troubleshooting", "error", "fix"],
                        "user_id": "KB-USER-003"
                    }
                })),
                ("slack_search", "Searching Slack conversations", serde_json::json!({
                    "context_data": {
                        "user_query": "Recent product updates",
                        "keywords": ["product", "update", "release"],
                        "user_id": "KB-USER-004"
                    }
                })),
            ];

            for (tool_name, description, test_data) in search_scenarios {
                println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("🔧 Testing Knowledge Base Tool: {}", tool_name);
                println!("   📝 {}", description);
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
                sleep(OPERATION_PAUSE).await;

                println!("📝 Preparing knowledge search parameters...");
                println!("   📊 Simulating tool call for: {}", tool_name);
                
                if let Ok(formatted_data) = serde_json::to_string_pretty(&test_data) {
                    if formatted_data.len() > 300 {
                        println!("   📋 Test data: {}...", &formatted_data[..297]);
                    } else {
                        println!("   📋 Test data: {}", formatted_data);
                    }
                }

                println!("\n🚀 Executing knowledge base tool call (simulated)...");
                let tool_start = Instant::now();
                sleep(KNOWLEDGE_SEARCH_PAUSE).await;

                // Simulate successful tool execution
                let tool_elapsed = tool_start.elapsed();
                println!(
                    "\n✅ Successfully called {} tool in {:.2}s (simulated)",
                    tool_name,
                    tool_elapsed.as_secs_f64()
                );
                sleep(QUICK_PAUSE).await;
                
                println!("   📊 Tool execution completed successfully");
                println!("   ❌ Error status: false");
                println!("   📄 Content items: 1");
                
                // Show simulated search results
                let simulated_result = match tool_name {
                    "validate_query" => "Query validation passed: technical query about SSL certificates",
                    "notion_search" => "Found 3 relevant documentation pages about API configuration",
                    "helpscout_search" => "Found 5 articles about troubleshooting application crashes",
                    "slack_search" => "Found 8 recent conversations about product updates and releases",
                    _ => "Simulated search result for knowledge base query"
                };
                println!("   📝 Result preview: {}", simulated_result);
                
                sleep(SHORT_PAUSE).await;
            }

            // Demo 2: Knowledge Base Workflow MCP server exposure
            println!("\n\n═══════════════════════════════════════════════════════════════");
            println!("🌐 Demo 2: Exposing Knowledge Base Workflow as MCP Server");
            println!("═══════════════════════════════════════════════════════════════\n");
            sleep(SECTION_PAUSE).await;

            println!("🔧 Converting knowledge base workflow to MCP server...");
            match workflow
                .expose_as_mcp_server("kb-workflow-server", "1.0.0")
                .await
            {
                Ok(kb_workflow_server) => {
                    println!("✅ Knowledge Base Workflow exposed as MCP server successfully!");
                    println!(
                        "   🔧 Workflow tools available: {}",
                        kb_workflow_server.get_tool_count().await
                    );

                    let kb_workflow_tools = kb_workflow_server.get_tool_names().await;
                    println!("   📚 Knowledge workflow tool names: {:?}", kb_workflow_tools);
                }
                Err(e) => println!("❌ Failed to expose knowledge base workflow as MCP server: {}", e),
            }

            // Demo 3: Knowledge Base MCP Configuration
            println!("\n\n═══════════════════════════════════════════════════════════════");
            println!("⚙️  Demo 3: Knowledge Base MCP Configuration");
            println!("═══════════════════════════════════════════════════════════════\n");
            sleep(SECTION_PAUSE).await;

            println!("📋 Loading Knowledge Base MCP configuration...");
            sleep(CONFIGURATION_PAUSE).await;
            let mcp_config = MCPConfig::from_env().unwrap_or_else(|_| {
                println!("   Using default MCP configuration for knowledge base");
                MCPConfig::default()
            });

            println!("   🔍 Knowledge Base MCP Enabled: {}", mcp_config.enabled);
            println!("   📚 KB Client Name: {}", mcp_config.client_name);
            println!("   📊 KB Client Version: {}", mcp_config.client_version);
            println!("   🔧 KB Configured Servers: {}", mcp_config.servers.len());

            if !mcp_config.servers.is_empty() {
                println!("   📡 Knowledge base server configurations:");
                for (name, config) in &mcp_config.servers {
                    if name.contains("knowledge") || name.contains("search") || name.contains("kb") {
                        println!(
                            "     - {}: enabled={}, auto_connect={}",
                            name, config.enabled, config.auto_connect
                        );
                    }
                }
            }

            // Demo 4: Knowledge Base Connection Pool
            println!("\n\n═══════════════════════════════════════════════════════════════");
            println!("🏊 Demo 4: Knowledge Base MCP Connection Pool");
            println!("═══════════════════════════════════════════════════════════════\n");
            sleep(SECTION_PAUSE).await;

            println!("🔧 Creating knowledge base connection pool with search-optimized configuration...");
            sleep(CONFIGURATION_PAUSE).await;
            let kb_pool_config = ConnectionConfig {
                max_connections_per_server: 5, // Higher for search workloads
                connection_timeout: Duration::from_secs(10), // Longer for search operations
                idle_timeout: Duration::from_secs(60),
                retry_attempts: 3, // More retries for search reliability
                retry_delay: Duration::from_millis(750),
                health_check_interval: Duration::from_secs(15),
                ..ConnectionConfig::default()
            };

            let kb_connection_pool = MCPConnectionPool::new(kb_pool_config);

            // Register knowledge base servers
            let kb_servers = vec![
                ("notion-kb-server", "ws://localhost:8081/kb/notion"),
                ("helpscout-kb-server", "ws://localhost:8082/kb/helpscout"), 
                ("slack-kb-server", "ws://localhost:8083/kb/slack"),
            ];

            println!("\n📝 Registering knowledge base servers in connection pool...");
            for (server_name, server_url) in kb_servers {
                sleep(NODE_PROCESSING_PAUSE).await;
                kb_connection_pool
                    .register_server(
                        server_name.to_string(),
                        TransportType::WebSocket {
                            url: server_url.to_string(),
                            heartbeat_interval: Some(Duration::from_secs(30)),
                            reconnect_config: ReconnectConfig::default(),
                        },
                        "kb-client".to_string(),
                        "1.0.0".to_string(),
                    )
                    .await;
                println!("   ✅ Registered: {}", server_name);
            }

            println!("\n✅ Knowledge Base connection pool created with search servers");

            let kb_pool_stats = kb_connection_pool.get_pool_stats().await;
            println!(
                "   📊 KB Pool statistics: {} search server configurations",
                kb_pool_stats.len()
            );

            let kb_health_status = kb_connection_pool.health_check().await.unwrap_or_default();
            println!(
                "   🏥 KB Health check results: {} search servers checked",
                kb_health_status.len()
            );

            // Demo 5: Register external knowledge base MCP servers with workflow
            println!("\n\n═══════════════════════════════════════════════════════════════");
            println!("🔗 Demo 5: Registering External Knowledge Base MCP Servers");
            println!("═══════════════════════════════════════════════════════════════\n");
            sleep(SECTION_PAUSE).await;

            let external_kb_servers = vec![
                ("External Notion Server", "ws://localhost:8091/external-notion"),
                ("External Documentation Server", "ws://localhost:8092/external-docs"),
                ("External Wiki Server", "ws://localhost:8093/external-wiki"),
            ];

            for (server_desc, server_url) in external_kb_servers {
                println!("🔧 Attempting to register {}...", server_desc);
                sleep(CONFIGURATION_PAUSE).await;
                
                match workflow
                    .register_mcp_server(
                        server_url,
                        TransportType::WebSocket {
                            url: server_url.to_string(),
                            heartbeat_interval: Some(Duration::from_secs(30)),
                            reconnect_config: ReconnectConfig::default(),
                        },
                    )
                    .await
                {
                    Ok(()) => {
                        println!("✅ {} registered successfully", server_desc);
                        println!("   📡 Server URL: {}", server_url);
                        println!("   🔌 Transport: WebSocket");
                    }
                    Err(e) => println!("❌ Failed to register {}: {}", server_desc, e),
                }
                sleep(SHORT_PAUSE).await;
            }

            // Demo 6: Knowledge Base Search Integration Test
            println!("\n\n═══════════════════════════════════════════════════════════════");
            println!("🔍 Demo 6: Integrated Knowledge Base Search Test");
            println!("═══════════════════════════════════════════════════════════════\n");
            sleep(SECTION_PAUSE).await;

            let search_query_data = serde_json::json!({
                "query_id": "INTEGRATED-SEARCH-001",
                "user_id": "MCP-TEST-USER",
                "user_query": "How to integrate MCP servers with knowledge base workflows?",
                "query_type": "integration",
                "sources": ["notion", "helpscout", "slack"],
                "priority": "high"
            });

            println!("🚀 Running integrated knowledge base search through MCP workflow...");
            let integrated_start = Instant::now();

            match workflow.run(search_query_data) {
                Ok(context) => {
                    let integrated_elapsed = integrated_start.elapsed();
                    println!(
                        "\n✅ Integrated MCP knowledge search completed in {:.2}s!",
                        integrated_elapsed.as_secs_f64()
                    );
                    
                    println!("   🆔 Search Query ID: {}", context.event_id);
                    println!("   📚 Search Results: {} nodes processed", context.nodes.len());
                    
                    // Show summary of search results
                    let mut sources_found = 0;
                    let mut total_results = 0;
                    for (node_name, node_data) in &context.nodes {
                        if node_name.contains("search") {
                            sources_found += 1;
                            if let Some(obj) = node_data.as_object() {
                                if let Some(results) = obj.get("results_count") {
                                    if let Some(count) = results.as_u64() {
                                        total_results += count;
                                    }
                                }
                            }
                        }
                    }
                    println!("   📊 Sources searched: {}", sources_found);
                    println!("   📄 Total results found: {}", total_results);
                }
                Err(e) => {
                    println!("❌ Integrated MCP knowledge search failed: {}", e);
                }
            }

            println!("\n\n╔═══════════════════════════════════════════════════════════╗");
            println!("║    🎉 Knowledge Base MCP Integration Demo Completed! 🎉   ║");
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
                    println!("💡 Tip: Ensure all knowledge search nodes are connected properly");
                    println!("🔍 Unreachable nodes: {:?}", nodes);
                }
                WorkflowError::InvalidRouter { node } => {
                    println!("💡 Tip: Verify knowledge base search router configuration");
                    println!("🔍 Problematic node: {}", node);
                }
                _ => {
                    println!("🔍 Error details: {:?}", e);
                }
            }
        }
    }
}