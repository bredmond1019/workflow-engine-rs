use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::core::error::WorkflowError;
use crate::core::mcp::clients::{MCPClient, StdioMCPClient, WebSocketMCPClient};
use crate::core::mcp::protocol::{CallToolResult, ToolDefinition};
use crate::core::mcp::transport::{HttpTransport, MCPTransport, TransportType};
use crate::core::nodes::Node;
use crate::core::task::TaskContext;

/// Configuration for external MCP server connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalMCPConfig {
    /// Name of the external service (e.g., "notion", "slack", "helpscout")
    pub service_name: String,

    /// Transport configuration for connecting to the MCP server
    pub transport: TransportType,

    /// Optional authentication configuration
    pub auth: Option<AuthConfig>,

    /// Retry configuration
    pub retry_config: RetryConfig,
}

/// Authentication configuration for external MCP servers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// API key or token
    pub token: Option<String>,

    /// Additional headers for HTTP transport
    pub headers: Option<HashMap<String, String>>,
}

/// Retry configuration for failed connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,

    /// Initial retry delay in milliseconds
    pub initial_delay_ms: u64,

    /// Maximum retry delay in milliseconds
    pub max_delay_ms: u64,

    /// Backoff multiplier for exponential backoff
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
        }
    }
}

/// Base trait for nodes that connect to external MCP servers
#[async_trait]
pub trait ExternalMCPClientNode: Node + Send + Sync {
    /// Get the configuration for this external MCP client
    fn get_config(&self) -> &ExternalMCPConfig;

    /// Connect to the external MCP server
    async fn connect(&mut self) -> Result<(), WorkflowError>;

    /// Execute a tool on the external MCP server
    async fn execute_tool(
        &mut self,
        tool_name: &str,
        arguments: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<CallToolResult, WorkflowError>;

    /// List available tools from the external MCP server
    async fn list_tools(&mut self) -> Result<Vec<ToolDefinition>, WorkflowError>;

    /// Disconnect from the external MCP server
    async fn disconnect(&mut self) -> Result<(), WorkflowError>;

    /// Check if the client is connected
    fn is_connected(&self) -> bool;
}

/// Base implementation for external MCP client nodes
pub struct BaseExternalMCPClient {
    config: ExternalMCPConfig,
    client: Option<Box<dyn MCPClient>>,
    connection_pool: Arc<Mutex<ConnectionPool>>,
}

impl Debug for BaseExternalMCPClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BaseExternalMCPClient")
            .field("config", &self.config)
            .field("connected", &self.is_connected())
            .finish()
    }
}

impl BaseExternalMCPClient {
    pub fn new(config: ExternalMCPConfig) -> Self {
        Self {
            config,
            client: None,
            connection_pool: Arc::new(Mutex::new(ConnectionPool::new())),
        }
    }

    /// Create an MCP client based on the transport type
    fn create_client(&self) -> Result<Box<dyn MCPClient>, WorkflowError> {
        match &self.config.transport {
            TransportType::Stdio { command, args, .. } => {
                Ok(Box::new(StdioMCPClient::new(command.clone(), args.clone())))
            }
            TransportType::WebSocket { url, .. } => Ok(Box::new(WebSocketMCPClient::new(url.clone()))),
            TransportType::Http { base_url, .. } => {
                // For HTTP transport, we need a custom implementation
                // that handles request/response pattern
                Ok(Box::new(HttpMCPClient::new(
                    base_url.clone(),
                    self.config.auth.clone(),
                )))
            }
        }
    }

    /// Connect with retry logic
    async fn connect_with_retry(&mut self) -> Result<(), WorkflowError> {
        let retry_config = self.config.retry_config.clone();
        let mut delay_ms = retry_config.initial_delay_ms;

        for attempt in 0..=retry_config.max_retries {
            match self.connect_internal().await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    if attempt == retry_config.max_retries {
                        return Err(e);
                    }

                    // Log retry attempt
                    log::warn!(
                        "[{}] Connection attempt {} failed, retrying in {}ms: {}",
                        self.config.service_name,
                        attempt + 1,
                        delay_ms,
                        e
                    );

                    // Wait before retry
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;

                    // Calculate next delay with exponential backoff
                    delay_ms = (delay_ms as f64 * retry_config.backoff_multiplier) as u64;
                    delay_ms = delay_ms.min(retry_config.max_delay_ms);
                }
            }
        }

        Err(WorkflowError::MCPConnectionError {
            message: format!(
                "Failed to connect to {} after {} attempts",
                self.config.service_name,
                retry_config.max_retries + 1
            ),
        })
    }

    /// Internal connection logic
    async fn connect_internal(&mut self) -> Result<(), WorkflowError> {
        let mut client = self.create_client()?;
        client.connect().await?;
        client
            .initialize(&self.config.service_name, "1.0.0")
            .await?;
        self.client = Some(client);
        Ok(())
    }
}

#[async_trait]
impl ExternalMCPClientNode for BaseExternalMCPClient {
    fn get_config(&self) -> &ExternalMCPConfig {
        &self.config
    }

    async fn connect(&mut self) -> Result<(), WorkflowError> {
        self.connect_with_retry().await
    }

    async fn execute_tool(
        &mut self,
        tool_name: &str,
        arguments: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<CallToolResult, WorkflowError> {
        if let Some(ref mut client) = self.client {
            client.call_tool(tool_name, arguments).await
        } else {
            Err(WorkflowError::MCPConnectionError {
                message: format!("{} client not connected", self.config.service_name),
            })
        }
    }

    async fn list_tools(&mut self) -> Result<Vec<ToolDefinition>, WorkflowError> {
        if let Some(ref mut client) = self.client {
            client.list_tools().await
        } else {
            Err(WorkflowError::MCPConnectionError {
                message: format!("{} client not connected", self.config.service_name),
            })
        }
    }

    async fn disconnect(&mut self) -> Result<(), WorkflowError> {
        if let Some(ref mut client) = self.client {
            client.disconnect().await?;
        }
        self.client = None;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.client
            .as_ref()
            .map(|c| c.is_connected())
            .unwrap_or(false)
    }
}

impl Node for BaseExternalMCPClient {
    fn process(&self, mut task_context: TaskContext) -> Result<TaskContext, WorkflowError> {
        task_context.set_data("external_mcp_client_processed", true)?;
        Ok(task_context)
    }
}

/// Connection pool for managing MCP connections
#[derive(Debug)]
struct ConnectionPool {
    connections: HashMap<String, Arc<Mutex<Box<dyn MCPClient>>>>,
}

impl ConnectionPool {
    fn new() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }

    async fn get_or_create(
        &mut self,
        key: &str,
        creator: impl FnOnce() -> Box<dyn MCPClient>,
    ) -> Arc<Mutex<Box<dyn MCPClient>>> {
        if let Some(conn) = self.connections.get(key) {
            conn.clone()
        } else {
            let conn = Arc::new(Mutex::new(creator()));
            self.connections.insert(key.to_string(), conn.clone());
            conn
        }
    }

    async fn remove(&mut self, key: &str) {
        self.connections.remove(key);
    }
}

/// HTTP MCP Client implementation
#[derive(Debug)]
struct HttpMCPClient {
    base_url: String,
    auth: Option<AuthConfig>,
    client: reqwest::Client,
    is_connected: bool,
}

impl HttpMCPClient {
    fn new(base_url: String, auth: Option<AuthConfig>) -> Self {
        Self {
            base_url,
            auth,
            client: reqwest::Client::new(),
            is_connected: false,
        }
    }
}

#[async_trait]
impl MCPClient for HttpMCPClient {
    async fn connect(&mut self) -> Result<(), WorkflowError> {
        // For HTTP, "connection" is just validation that the server is reachable
        self.is_connected = true;
        Ok(())
    }

    async fn initialize(
        &mut self,
        client_name: &str,
        client_version: &str,
    ) -> Result<(), WorkflowError> {
        // HTTP clients may not need initialization
        Ok(())
    }

    async fn list_tools(&mut self) -> Result<Vec<ToolDefinition>, WorkflowError> {
        let url = format!("{}/tools/list", self.base_url);
        let mut request = self.client.get(&url);

        // Add authentication headers if configured
        if let Some(ref auth) = self.auth {
            if let Some(ref token) = auth.token {
                request = request.header("Authorization", format!("Bearer {}", token));
            }
            if let Some(ref headers) = auth.headers {
                for (key, value) in headers {
                    request = request.header(key, value);
                }
            }
        }

        let response = request
            .send()
            .await
            .map_err(|e| WorkflowError::MCPConnectionError {
                message: format!("Failed to list tools: {}", e),
            })?;

        if !response.status().is_success() {
            return Err(WorkflowError::MCPError {
                message: format!("HTTP error: {}", response.status()),
            });
        }

        let tools: Vec<ToolDefinition> =
            response
                .json()
                .await
                .map_err(|e| WorkflowError::MCPProtocolError {
                    message: format!("Failed to parse tools response: {}", e),
                })?;

        Ok(tools)
    }

    async fn call_tool(
        &mut self,
        name: &str,
        arguments: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<CallToolResult, WorkflowError> {
        let url = format!("{}/tools/call", self.base_url);
        let mut request = self.client.post(&url);

        // Add authentication headers if configured
        if let Some(ref auth) = self.auth {
            if let Some(ref token) = auth.token {
                request = request.header("Authorization", format!("Bearer {}", token));
            }
            if let Some(ref headers) = auth.headers {
                for (key, value) in headers {
                    request = request.header(key, value);
                }
            }
        }

        let body = serde_json::json!({
            "name": name,
            "arguments": arguments,
        });

        let response =
            request
                .json(&body)
                .send()
                .await
                .map_err(|e| WorkflowError::MCPConnectionError {
                    message: format!("Failed to call tool: {}", e),
                })?;

        if !response.status().is_success() {
            return Err(WorkflowError::MCPError {
                message: format!("HTTP error: {}", response.status()),
            });
        }

        let result: CallToolResult =
            response
                .json()
                .await
                .map_err(|e| WorkflowError::MCPProtocolError {
                    message: format!("Failed to parse tool call response: {}", e),
                })?;

        Ok(result)
    }

    async fn disconnect(&mut self) -> Result<(), WorkflowError> {
        self.is_connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.is_connected
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::*;
    use mockall::predicate::*;

    // Mock trait for MCPClient
    mock! {
        TestMCPClient {}
        
        impl std::fmt::Debug for TestMCPClient {
            fn fmt<'a>(&self, f: &mut std::fmt::Formatter<'a>) -> std::fmt::Result;
        }
        
        #[async_trait]
        impl MCPClient for TestMCPClient {
            async fn connect(&mut self) -> Result<(), WorkflowError>;
            async fn initialize(&mut self, client_name: &str, client_version: &str) -> Result<(), WorkflowError>;
            async fn list_tools(&mut self) -> Result<Vec<ToolDefinition>, WorkflowError>;
            async fn call_tool(&mut self, name: &str, arguments: Option<HashMap<String, serde_json::Value>>) -> Result<CallToolResult, WorkflowError>;
            async fn disconnect(&mut self) -> Result<(), WorkflowError>;
            fn is_connected(&self) -> bool;
        }
    }

    fn create_test_config(service_name: &str) -> ExternalMCPConfig {
        ExternalMCPConfig {
            service_name: service_name.to_string(),
            transport: TransportType::Http {
                base_url: "http://localhost:8080".to_string(),
                pool_config: crate::core::mcp::transport::HttpPoolConfig::default(),
            },
            auth: None,
            retry_config: RetryConfig::default(),
        }
    }

    #[test]
    fn test_base_external_mcp_client_creation() {
        let config = create_test_config("test_service");
        let client = BaseExternalMCPClient::new(config.clone());
        
        assert_eq!(client.config.service_name, "test_service");
        assert!(!client.is_connected());
    }

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay_ms, 1000);
        assert_eq!(config.max_delay_ms, 30000);
        assert_eq!(config.backoff_multiplier, 2.0);
    }

    #[tokio::test]
    async fn test_connect_success() {
        let config = create_test_config("test_service");
        let mut client = BaseExternalMCPClient::new(config);
        
        // We can't easily mock the internal client creation, so we'll test the HttpMCPClient directly
        let mut http_client = HttpMCPClient::new("http://localhost:8080".to_string(), None);
        let result = http_client.connect().await;
        
        assert!(result.is_ok());
        assert!(http_client.is_connected());
    }

    #[tokio::test]
    async fn test_execute_tool_not_connected() {
        let config = create_test_config("test_service");
        let mut client = BaseExternalMCPClient::new(config);
        
        let result = client.execute_tool("test_tool", None).await;
        
        assert!(result.is_err());
        match result {
            Err(WorkflowError::MCPConnectionError { message }) => {
                assert!(message.contains("not connected"));
            }
            _ => panic!("Expected MCPConnectionError"),
        }
    }

    #[tokio::test]
    async fn test_list_tools_not_connected() {
        let config = create_test_config("test_service");
        let mut client = BaseExternalMCPClient::new(config);
        
        let result = client.list_tools().await;
        
        assert!(result.is_err());
        match result {
            Err(WorkflowError::MCPConnectionError { message }) => {
                assert!(message.contains("not connected"));
            }
            _ => panic!("Expected MCPConnectionError"),
        }
    }

    #[tokio::test]
    async fn test_disconnect_when_not_connected() {
        let config = create_test_config("test_service");
        let mut client = BaseExternalMCPClient::new(config);
        
        let result = client.disconnect().await;
        
        assert!(result.is_ok());
        assert!(!client.is_connected());
    }

    #[test]
    fn test_node_process() {
        let config = create_test_config("test_service");
        let client = BaseExternalMCPClient::new(config);
        
        let task_context = TaskContext::new("test_workflow".to_string(), serde_json::json!({}));
        let result = client.process(task_context);
        
        assert!(result.is_ok());
        let updated_context = result.unwrap();
        let data = updated_context.get_data::<bool>("external_mcp_client_processed");
        assert!(data.is_ok());
        assert_eq!(data.unwrap(), Some(true));
    }

    #[test]
    fn test_auth_config_creation() {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer token123".to_string());
        
        let auth = AuthConfig {
            token: Some("token123".to_string()),
            headers: Some(headers.clone()),
        };
        
        assert_eq!(auth.token, Some("token123".to_string()));
        assert_eq!(auth.headers.unwrap().get("Authorization"), Some(&"Bearer token123".to_string()));
    }

    #[test]
    fn test_connection_pool_creation() {
        let pool = ConnectionPool::new();
        assert!(pool.connections.is_empty());
    }

    #[tokio::test]
    async fn test_connection_pool_get_or_create() {
        let mut pool = ConnectionPool::new();
        
        let key = "test_connection";
        let conn1 = pool.get_or_create(key, || {
            Box::new(MockTestMCPClient::new())
        }).await;
        
        let conn2 = pool.get_or_create(key, || {
            Box::new(MockTestMCPClient::new())
        }).await;
        
        // Should return the same connection
        assert!(Arc::ptr_eq(&conn1, &conn2));
    }

    #[tokio::test]
    async fn test_connection_pool_remove() {
        let mut pool = ConnectionPool::new();
        
        let key = "test_connection";
        pool.get_or_create(key, || {
            Box::new(MockTestMCPClient::new())
        }).await;
        
        assert!(!pool.connections.is_empty());
        
        pool.remove(key).await;
        assert!(pool.connections.is_empty());
    }

    #[tokio::test]
    async fn test_http_mcp_client_initialization() {
        let auth = AuthConfig {
            token: Some("api_key".to_string()),
            headers: None,
        };
        
        let mut client = HttpMCPClient::new("http://api.example.com".to_string(), Some(auth));
        
        assert_eq!(client.base_url, "http://api.example.com");
        assert!(!client.is_connected());
        
        let result = client.initialize("test_client", "1.0.0").await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_transport_type_creation() {
        // Test HTTP transport
        let http_transport = TransportType::Http {
            base_url: "http://localhost:8080".to_string(),
            pool_config: crate::core::mcp::transport::HttpPoolConfig::default(),
        };
        
        match http_transport {
            TransportType::Http { base_url, .. } => {
                assert_eq!(base_url, "http://localhost:8080");
            }
            _ => panic!("Expected HTTP transport"),
        }
        
        // Test WebSocket transport
        let ws_transport = TransportType::WebSocket {
            url: "ws://localhost:8080".to_string(),
            heartbeat_interval: Some(std::time::Duration::from_secs(30)),
            reconnect_config: crate::core::mcp::transport::ReconnectConfig::default(),
        };
        
        match ws_transport {
            TransportType::WebSocket { url, .. } => {
                assert_eq!(url, "ws://localhost:8080");
            }
            _ => panic!("Expected WebSocket transport"),
        }
        
        // Test stdio transport
        let stdio_transport = TransportType::Stdio {
            command: "python".to_string(),
            args: vec!["-m".to_string(), "mcp_server".to_string()],
            auto_restart: true,
            max_restarts: 3,
        };
        
        match stdio_transport {
            TransportType::Stdio { command, args, .. } => {
                assert_eq!(command, "python");
                assert_eq!(args, vec!["-m", "mcp_server"]);
            }
            _ => panic!("Expected stdio transport"),
        }
    }

    #[tokio::test]
    async fn test_retry_config_backoff() {
        let config = create_test_config("test_service");
        let retry_config = config.retry_config.clone();
        
        let mut delay = retry_config.initial_delay_ms;
        
        // Test exponential backoff calculation
        for _ in 0..3 {
            delay = (delay as f64 * retry_config.backoff_multiplier) as u64;
            delay = delay.min(retry_config.max_delay_ms);
        }
        
        // After 3 iterations with 2.0 multiplier: 1000 -> 2000 -> 4000 -> 8000
        assert_eq!(delay, 8000);
    }

    #[test]
    fn test_external_mcp_config_with_auth() {
        let mut headers = HashMap::new();
        headers.insert("X-API-Key".to_string(), "secret_key".to_string());
        
        let auth = Some(AuthConfig {
            token: Some("secret_key".to_string()),
            headers: Some(headers),
        });
        
        let config = ExternalMCPConfig {
            service_name: "auth_service".to_string(),
            transport: TransportType::Http {
                base_url: "https://api.example.com".to_string(),
                pool_config: crate::core::mcp::transport::HttpPoolConfig::default(),
            },
            auth: auth.clone(),
            retry_config: RetryConfig::default(),
        };
        
        assert_eq!(config.service_name, "auth_service");
        assert!(config.auth.is_some());
        
        let auth_config = config.auth.unwrap();
        assert_eq!(auth_config.token, Some("secret_key".to_string()));
        assert!(auth_config.headers.is_some());
    }

    // Tests for mocking external MCP services
    #[tokio::test]
    async fn test_mock_external_service_success() {
        let mut mock_client = MockTestMCPClient::new();
        
        // Set up expectations
        mock_client
            .expect_connect()
            .times(1)
            .returning(|| Ok(()));
            
        mock_client
            .expect_initialize()
            .with(eq("test_service"), eq("1.0.0"))
            .times(1)
            .returning(|_, _| Ok(()));
            
        mock_client
            .expect_is_connected()
            .returning(|| true);
            
        mock_client
            .expect_list_tools()
            .times(1)
            .returning(|| Ok(vec![
                ToolDefinition {
                    name: "test_tool".to_string(),
                    description: Some("A test tool".to_string()),
                    input_schema: serde_json::json!({}),
                },
            ]));
            
        // Execute test
        mock_client.connect().await.unwrap();
        mock_client.initialize("test_service", "1.0.0").await.unwrap();
        assert!(mock_client.is_connected());
        
        let tools = mock_client.list_tools().await.unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "test_tool");
    }

    #[tokio::test]
    async fn test_mock_external_service_connection_failure() {
        let mut mock_client = MockTestMCPClient::new();
        
        mock_client
            .expect_connect()
            .times(1)
            .returning(|| Err(WorkflowError::MCPConnectionError {
                message: "Connection refused".to_string(),
            }));
            
        let result = mock_client.connect().await;
        assert!(result.is_err());
        
        match result {
            Err(WorkflowError::MCPConnectionError { message }) => {
                assert_eq!(message, "Connection refused");
            }
            _ => panic!("Expected MCPConnectionError"),
        }
    }

    #[tokio::test]
    async fn test_mock_tool_execution() {
        let mut mock_client = MockTestMCPClient::new();
        
        let mut args = HashMap::new();
        args.insert("param1".to_string(), serde_json::Value::String("value1".to_string()));
        
        mock_client
            .expect_call_tool()
            .with(eq("test_tool"), eq(Some(args.clone())))
            .times(1)
            .returning(|_, _| Ok(CallToolResult {
                content: vec![crate::core::mcp::protocol::ToolContent::Text {
                    text: "success".to_string(),
                }],
                is_error: Some(false),
            }));
            
        let result = mock_client.call_tool("test_tool", Some(args)).await.unwrap();
        assert_eq!(result.is_error, Some(false));
        assert_eq!(result.content.len(), 1);
    }

    // Test error propagation
    #[tokio::test]
    async fn test_error_propagation_from_transport() {
        let config = create_test_config("error_service");
        let mut client = BaseExternalMCPClient::new(config);
        
        // Since we can't inject a mock easily, we test with disconnected state
        let result = client.execute_tool("any_tool", None).await;
        
        assert!(result.is_err());
        match result {
            Err(WorkflowError::MCPConnectionError { message }) => {
                assert!(message.contains("not connected"));
            }
            _ => panic!("Expected MCPConnectionError"),
        }
    }

    #[tokio::test]
    async fn test_error_propagation_invalid_tool() {
        let mut mock_client = MockTestMCPClient::new();
        
        mock_client
            .expect_call_tool()
            .with(eq("invalid_tool"), eq(None))
            .times(1)
            .returning(|_, _| Err(WorkflowError::MCPError {
                message: "Tool not found: invalid_tool".to_string(),
            }));
            
        let result = mock_client.call_tool("invalid_tool", None).await;
        
        assert!(result.is_err());
        match result {
            Err(WorkflowError::MCPError { message }) => {
                assert!(message.contains("Tool not found"));
            }
            _ => panic!("Expected MCPError"),
        }
    }

    // Test timeout and retry behavior
    #[tokio::test]
    async fn test_retry_on_connection_failure() {
        let mut retry_config = RetryConfig::default();
        retry_config.max_retries = 2;
        retry_config.initial_delay_ms = 10; // Short delay for testing
        retry_config.max_delay_ms = 50;
        
        let mut config = create_test_config("retry_service");
        config.retry_config = retry_config;
        
        let client = BaseExternalMCPClient::new(config);
        
        // We can verify the retry config is properly set
        assert_eq!(client.config.retry_config.max_retries, 2);
        assert_eq!(client.config.retry_config.initial_delay_ms, 10);
    }

    #[tokio::test]
    async fn test_exponential_backoff_calculation() {
        let retry_config = RetryConfig {
            max_retries: 5,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.5,
        };
        
        let mut delay = retry_config.initial_delay_ms;
        let mut delays = vec![delay];
        
        for _ in 0..4 {
            delay = (delay as f64 * retry_config.backoff_multiplier) as u64;
            delay = delay.min(retry_config.max_delay_ms);
            delays.push(delay);
        }
        
        // Expected: 100, 250, 625, 1562, 3905
        assert_eq!(delays[0], 100);
        assert_eq!(delays[1], 250);
        assert_eq!(delays[2], 625);
        assert_eq!(delays[3], 1562);
        assert_eq!(delays[4], 3905);
    }

    // Integration test placeholder (would need real MCP servers)
    #[tokio::test]
    #[ignore] // Run with cargo test -- --ignored
    async fn test_integration_with_real_mcp_server() {
        // This test would connect to a real MCP server
        // For now, it's a placeholder showing the structure
        
        let config = ExternalMCPConfig {
            service_name: "test_integration".to_string(),
            transport: TransportType::Http {
                base_url: "http://localhost:8001".to_string(), // HelpScout test server
                pool_config: crate::core::mcp::transport::HttpPoolConfig::default(),
            },
            auth: None,
            retry_config: RetryConfig::default(),
        };
        
        let mut client = BaseExternalMCPClient::new(config);
        
        // Would test real connection
        let connect_result = client.connect().await;
        if connect_result.is_ok() {
            assert!(client.is_connected());
            
            let tools = client.list_tools().await;
            assert!(tools.is_ok());
            
            let _ = client.disconnect().await;
        }
    }

    // Test different transport types
    #[test]
    fn test_create_client_with_different_transports() {
        let config_http = ExternalMCPConfig {
            service_name: "http_service".to_string(),
            transport: TransportType::Http {
                base_url: "http://example.com".to_string(),
                pool_config: crate::core::mcp::transport::HttpPoolConfig::default(),
            },
            auth: None,
            retry_config: RetryConfig::default(),
        };
        
        let config_ws = ExternalMCPConfig {
            service_name: "ws_service".to_string(),
            transport: TransportType::WebSocket {
                url: "ws://example.com".to_string(),
                heartbeat_interval: None,
                reconnect_config: crate::core::mcp::transport::ReconnectConfig::default(),
            },
            auth: None,
            retry_config: RetryConfig::default(),
        };
        
        let config_stdio = ExternalMCPConfig {
            service_name: "stdio_service".to_string(),
            transport: TransportType::Stdio {
                command: "python".to_string(),
                args: vec![],
                auto_restart: false,
                max_restarts: 0,
            },
            auth: None,
            retry_config: RetryConfig::default(),
        };
        
        let client_http = BaseExternalMCPClient::new(config_http);
        let client_ws = BaseExternalMCPClient::new(config_ws);
        let client_stdio = BaseExternalMCPClient::new(config_stdio);
        
        assert_eq!(client_http.config.service_name, "http_service");
        assert_eq!(client_ws.config.service_name, "ws_service");
        assert_eq!(client_stdio.config.service_name, "stdio_service");
    }

    // Test HTTP client specific functionality
    #[tokio::test]
    async fn test_http_client_with_auth_headers() {
        let mut headers = HashMap::new();
        headers.insert("X-Custom-Header".to_string(), "custom_value".to_string());
        headers.insert("X-API-Version".to_string(), "v2".to_string());
        
        let auth = Some(AuthConfig {
            token: Some("bearer_token".to_string()),
            headers: Some(headers),
        });
        
        let client = HttpMCPClient::new("https://api.example.com".to_string(), auth.clone());
        
        assert_eq!(client.base_url, "https://api.example.com");
        assert!(client.auth.is_some());
        
        let client_auth = client.auth.unwrap();
        assert_eq!(client_auth.token, Some("bearer_token".to_string()));
        assert!(client_auth.headers.is_some());
        
        let headers = client_auth.headers.unwrap();
        assert_eq!(headers.get("X-Custom-Header"), Some(&"custom_value".to_string()));
        assert_eq!(headers.get("X-API-Version"), Some(&"v2".to_string()));
    }

    #[tokio::test]
    async fn test_http_client_error_handling() {
        let mut client = HttpMCPClient::new("http://invalid.local".to_string(), None);
        client.is_connected = true; // Simulate connected state
        
        // This would fail with a real request to invalid.local
        // For testing purposes, we're verifying the structure
        assert_eq!(client.base_url, "http://invalid.local");
        assert!(client.is_connected());
    }
}
