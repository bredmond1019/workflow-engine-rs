//! # Model Context Protocol (MCP) Implementation
//!
//! This module provides a comprehensive implementation of the Model Context Protocol (MCP),
//! enabling AI agents to communicate with external tools and services in a standardized way.
//! MCP allows AI models to access real-time data, perform actions, and integrate with
//! external systems safely and efficiently.
//!
//! ## Architecture Overview
//!
//! The MCP implementation consists of several key components:
//!
//! ### Transport Layer
//! - **WebSocket Transport**: Real-time bidirectional communication
//! - **Stdio Transport**: Process-based communication via stdin/stdout
//! - **Transport Abstraction**: Unified interface for different transport mechanisms
//!
//! ### Protocol Layer
//! - **Message Types**: Requests, responses, notifications, and errors
//! - **Tool Definitions**: Standardized tool metadata and schemas
//! - **Call Handling**: Tool invocation and result processing
//!
//! ### Client/Server Components
//! - **MCP Clients**: Connect to external MCP servers
//! - **MCP Servers**: Expose workflow nodes as MCP tools
//! - **Connection Management**: Pooling, health checks, and retry logic
//!
//! ### Configuration
//! - **Server Configuration**: Connection settings and capabilities
//! - **Pool Configuration**: Connection limits and timeouts
//! - **Environment Integration**: Configuration from environment variables
//!
//! ## Usage Examples
//!
//! ### Basic MCP Client
//!
//! ```rust
//! use ai_architecture_core::mcp::{
//!     clients::websocket::WebSocketMcpClient,
//!     protocol::{ToolCall, ToolDefinition},
//!     transport::TransportType,
//! };
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Connect to MCP server
//!     let client = WebSocketMcpClient::new("ws://localhost:8080/mcp".to_string());
//!     client.connect().await?;
//!
//!     // List available tools
//!     let tools = client.list_tools().await?;
//!     println!("Available tools: {:?}", tools);
//!
//!     // Call a specific tool
//!     let result = client.call_tool(
//!         "analyze_sentiment",
//!         json!({
//!             "text": "I love this product!",
//!             "language": "en"
//!         })
//!     ).await?;
//!
//!     println!("Analysis result: {:?}", result);
//!     Ok(())
//! }
//! ```
//!
//! ### MCP Server Setup
//!
//! ```rust
//! use ai_architecture_core::{
//!     mcp::server::MCPToolServer,
//!     nodes::Node,
//!     task::TaskContext,
//!     error::WorkflowError,
//! };
//! use serde_json::json;
//!
//! // Define a custom node to expose as MCP tool
//! #[derive(Debug)]
//! struct TextAnalysisNode;
//!
//! impl Node for TextAnalysisNode {
//!     fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
//!         let input: serde_json::Value = context.get_event_data()?;
//!         let text = input.get("text").and_then(|v| v.as_str()).unwrap_or("");
//!         
//!         // Perform analysis
//!         let word_count = text.split_whitespace().count();
//!         let sentiment = if text.contains("love") || text.contains("great") {
//!             "positive"
//!         } else if text.contains("hate") || text.contains("terrible") {
//!             "negative"
//!         } else {
//!             "neutral"
//!         };
//!         
//!         context.update_node("analysis_result", json!({
//!             "word_count": word_count,
//!             "sentiment": sentiment,
//!             "processed_text": text
//!         }));
//!         
//!         Ok(context)
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), WorkflowError> {
//!     // Create MCP server
//!     let server = MCPToolServer::new(
//!         "text-analysis-server".to_string(),
//!         "1.0.0".to_string()
//!     );
//!
//!     // Register node as MCP tool
//!     server.register_node_with_metadata(
//!         Box::new(TextAnalysisNode),
//!         "analyze_text",
//!         "Analyzes text for word count and sentiment",
//!         json!({
//!             "type": "object",
//!             "properties": {
//!                 "text": {
//!                     "type": "string",
//!                     "description": "Text to analyze"
//!                 }
//!             },
//!             "required": ["text"]
//!         })
//!     ).await?;
//!
//!     // Start server (this would typically run indefinitely)
//!     println!("MCP server ready with text analysis tools");
//!     Ok(())
//! }
//! ```
//!
//! ### Connection Pool Management
//!
//! ```rust
//! use ai_architecture_core::mcp::{
//!     connection_pool::{MCPConnectionPool, ConnectionConfig},
//!     transport::TransportType,
//! };
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Configure connection pool
//!     let pool_config = ConnectionConfig {
//!         max_connections_per_server: 10,
//!         connection_timeout: Duration::from_secs(30),
//!         idle_timeout: Duration::from_secs(300),
//!         retry_attempts: 3,
//!         retry_delay: Duration::from_millis(1000),
//!         health_check_interval: Duration::from_secs(60),
//!     };
//!
//!     let pool = MCPConnectionPool::new(pool_config);
//!
//!     // Register multiple MCP servers
//!     pool.register_server(
//!         "customer-support".to_string(),
//!         TransportType::WebSocket {
//!             url: "ws://localhost:8080/customer-mcp".to_string(),
//!         },
//!         "ai-client".to_string(),
//!         "1.0.0".to_string(),
//!     ).await;
//!
//!     pool.register_server(
//!         "knowledge-base".to_string(),
//!         TransportType::WebSocket {
//!             url: "ws://localhost:8081/knowledge-mcp".to_string(),
//!         },
//!         "ai-client".to_string(),
//!         "1.0.0".to_string(),
//!     ).await;
//!
//!     // Use pooled connections
//!     let client = pool.get_client("customer-support").await?;
//!     let tools = client.list_tools().await?;
//!     println!("Customer support tools: {:?}", tools);
//!
//!     // Pool automatically manages connections, health checks, and retries
//!     Ok(())
//! }
//! ```
//!
//! ### Workflow Integration
//!
//! ```rust
//! use ai_architecture_core::{
//!     workflow::{builder::WorkflowBuilder, Workflow},
//!     ai_agents::anthropic::AnthropicAgentNode,
//!     nodes::agent::AgentConfig,
//!     mcp::clients::websocket::WebSocketMcpClient,
//! };
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Build workflow with MCP-enabled AI agent
//!     let workflow = WorkflowBuilder::new("mcp_enhanced_workflow")
//!         .start_with::<DataValidationNode>()
//!         .then::<AnthropicAgentNode>()
//!         .then::<ResultFormatterNode>()
//!         .build()?;
//!
//!     // Configure agent with MCP capabilities
//!     let agent_config = AgentConfig {
//!         system_prompt: "You are an AI assistant with access to specialized tools via MCP.".to_string(),
//!         model_provider: ModelProvider::Anthropic,
//!         model_name: "claude-3-sonnet-20240229".to_string(),
//!         mcp_server_uri: Some("ws://localhost:8080/tools".to_string()),
//!     };
//!
//!     let mut agent = AnthropicAgentNode::new(agent_config);
//!
//!     // Connect MCP client
//!     let mcp_client = WebSocketMcpClient::new("ws://localhost:8080/tools".to_string());
//!     mcp_client.connect().await?;
//!     agent.set_mcp_client(Box::new(mcp_client));
//!
//!     workflow.register_node(agent);
//!
//!     // Expose workflow as MCP server
//!     let mcp_server = workflow
//!         .expose_as_mcp_server("workflow-server", "1.0.0")
//!         .await?;
//!
//!     println!("Workflow exposed as MCP server with tool capabilities");
//!     Ok(())
//! }
//! ```
//!
//! ### Error Handling
//!
//! ```rust
//! use ai_architecture_core::{
//!     mcp::clients::websocket::WebSocketMcpClient,
//!     error::WorkflowError,
//! };
//! use serde_json::json;
//!
//! async fn robust_mcp_call() -> Result<serde_json::Value, WorkflowError> {
//!     let client = WebSocketMcpClient::new("ws://localhost:8080/mcp".to_string());
//!     
//!     match client.connect().await {
//!         Ok(_) => {
//!             match client.call_tool("analyze_data", json!({"data": [1, 2, 3]})).await {
//!                 Ok(result) => Ok(result),
//!                 Err(e) => {
//!                     eprintln!("MCP tool call failed: {}", e);
//!                     Err(WorkflowError::MCPError {
//!                         message: format!("Tool call failed: {}", e)
//!                     })
//!                 }
//!             }
//!         }
//!         Err(e) => {
//!             eprintln!("MCP connection failed: {}", e);
//!             Err(WorkflowError::MCPConnectionError {
//!                 message: format!("Connection failed: {}", e)
//!             })
//!         }
//!     }
//! }
//! ```
//!
//! ## Protocol Details
//!
//! ### Message Types
//!
//! MCP defines several message types for communication:
//!
//! ```rust
//! use ai_architecture_core::mcp::protocol::{McpMessage, McpRequest, McpResponse};
//!
//! // Request to list available tools
//! let list_tools_request = McpRequest::ListTools;
//!
//! // Response with tool definitions
//! let tools_response = McpResponse::Tools {
//!     tools: vec![/* tool definitions */]
//! };
//!
//! // Tool call request
//! let call_request = McpRequest::CallTool {
//!     name: "analyze_sentiment".to_string(),
//!     arguments: serde_json::json!({"text": "Great product!"})
//! };
//! ```
//!
//! ### Tool Definitions
//!
//! Tools are defined with JSON Schema for type safety:
//!
//! ```rust
//! use ai_architecture_core::mcp::protocol::ToolDefinition;
//! use serde_json::json;
//!
//! let tool_definition = ToolDefinition {
//!     name: "calculate_statistics".to_string(),
//!     description: Some("Calculate statistical measures for a dataset".to_string()),
//!     input_schema: json!({
//!         "type": "object",
//!         "properties": {
//!             "data": {
//!                 "type": "array",
//!                 "items": {"type": "number"},
//!                 "description": "Array of numbers to analyze"
//!             },
//!             "measures": {
//!                 "type": "array",
//!                 "items": {"enum": ["mean", "median", "mode", "std_dev"]},
//!                 "description": "Statistical measures to calculate"
//!             }
//!         },
//!         "required": ["data"]
//!     })
//! };
//! ```
//!
//! ## Transport Options
//!
//! ### WebSocket Transport
//! - Real-time bidirectional communication
//! - Suitable for interactive applications
//! - Support for connection management and reconnection
//!
//! ### Stdio Transport
//! - Process-based communication
//! - Suitable for standalone tools and scripts
//! - Lower overhead for simple request/response patterns
//!
//! ## Configuration
//!
//! ### Environment-based Configuration
//!
//! ```rust
//! use ai_architecture_core::mcp::config::McpConfig;
//!
//! // Load from environment variables
//! let config = McpConfig::from_env().unwrap_or_default();
//!
//! println!("MCP enabled: {}", config.enabled);
//! println!("Client name: {}", config.client_name);
//! println!("Configured servers: {}", config.servers.len());
//! ```
//!
//! ### Server-specific Configuration
//!
//! ```rust
//! use ai_architecture_core::mcp::config::McpServerConfig;
//!
//! let server_config = McpServerConfig {
//!     enabled: true,
//!     auto_connect: true,
//!     connection_timeout: std::time::Duration::from_secs(30),
//!     retry_attempts: 3,
//!     health_check_interval: std::time::Duration::from_secs(60),
//! };
//! ```
//!
//! ## Best Practices
//!
//! 1. **Connection Management**: Use connection pools for multiple servers
//! 2. **Error Handling**: Implement robust retry and fallback mechanisms
//! 3. **Schema Validation**: Define clear input/output schemas for tools
//! 4. **Security**: Validate all inputs and implement proper authentication
//! 5. **Performance**: Cache tool definitions and reuse connections
//! 6. **Monitoring**: Implement health checks and connection monitoring
//!
//! ## Performance Considerations
//!
//! - **Connection Pooling**: Reduces connection overhead for frequent calls
//! - **Message Batching**: Group related tool calls when possible
//! - **Caching**: Cache tool definitions and results when appropriate
//! - **Timeout Management**: Set appropriate timeouts for different tool types
//!
//! ## Security Considerations
//!
//! - **Input Validation**: Always validate tool arguments against schemas
//! - **Output Sanitization**: Sanitize tool results before use
//! - **Authentication**: Implement proper authentication for MCP connections
//! - **Authorization**: Control which tools are accessible to which clients
//! - **Rate Limiting**: Implement rate limiting to prevent abuse

pub mod clients;
pub mod config;
pub mod connection_pool;
pub mod health;
pub mod load_balancer;
pub mod metrics;
pub mod protocol;
pub mod server;
pub mod transport;

pub use clients::{HttpMcpClient, McpClient, McpConnection, StdioMcpClient, WebSocketMcpClient};
pub use config::{McpConfig, McpServerConfig};
pub use connection_pool::{BorrowedConnection, ConnectionConfig, McpConnectionPool, PoolStats, DetailedHealthInfo, ServerHealthInfo, LoadBalancingStrategy, BackoffConfig};
pub use health::{ConnectionHealthMonitor, HealthConfig, HealthStatus, HealthMetrics};
pub use load_balancer::{MCPLoadBalancer, AdvancedMCPLoadBalancer, ConnectionInfo, LoadBalancingMetrics};
pub use metrics::{MCPMetricsCollector, MCPMetricsManager};
pub use protocol::{McpMessage, McpRequest, McpResponse, ToolCall, ToolDefinition};
pub use server::{MCPToolServer, ToolMetadata};
pub use transport::{MCPTransport, TransportType};
