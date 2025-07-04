use async_trait::async_trait;
use serde_json;
use std::io;
use std::time::{Duration, Instant};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tokio::time::{sleep, timeout};
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};
use futures_util::{SinkExt, StreamExt};
use serde::{Serialize, Deserialize};

use crate::protocol::{McpMessage, McpRequest, McpResponse};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransportType {
    Stdio { 
        command: String, 
        args: Vec<String>,
        /// Restart the process if it exits unexpectedly
        auto_restart: bool,
        /// Maximum number of restart attempts
        max_restarts: u32,
    },
    WebSocket { 
        url: String,
        /// Heartbeat interval for keep-alive
        heartbeat_interval: Option<Duration>,
        /// Reconnection configuration
        reconnect_config: ReconnectConfig,
    },
    Http { 
        base_url: String,
        /// Connection pool configuration
        pool_config: HttpPoolConfig,
    },
}

/// WebSocket/HTTP reconnection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconnectConfig {
    /// Enable automatic reconnection
    pub enabled: bool,
    /// Maximum number of reconnection attempts
    pub max_attempts: u32,
    /// Initial delay between reconnection attempts
    pub initial_delay: Duration,
    /// Maximum delay between reconnection attempts
    pub max_delay: Duration,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_attempts: 5,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

/// HTTP connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpPoolConfig {
    /// Maximum number of connections per host
    pub max_connections_per_host: usize,
    /// Connection timeout
    pub connect_timeout: Duration,
    /// Request timeout
    pub request_timeout: Duration,
    /// Keep-alive timeout
    pub keep_alive_timeout: Duration,
}

impl Default for HttpPoolConfig {
    fn default() -> Self {
        Self {
            max_connections_per_host: 10,
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            keep_alive_timeout: Duration::from_secs(90),
        }
    }
}

#[async_trait]
pub trait McpTransport: Send + Sync {
    async fn connect(&mut self) -> Result<(), TransportError>;
    async fn send(&mut self, message: McpRequest) -> Result<(), TransportError>;
    async fn receive(&mut self) -> Result<McpResponse, TransportError>;
    async fn disconnect(&mut self) -> Result<(), TransportError>;
    
    /// Check if the transport is currently connected
    fn is_connected(&self) -> bool;
    
    /// Get transport-specific health information
    async fn health_check(&mut self) -> Result<TransportHealth, TransportError>;
    
    /// Send a ping/keep-alive message
    async fn ping(&mut self) -> Result<Duration, TransportError>;
    
    /// Get transport metrics
    fn get_metrics(&self) -> TransportMetrics;
    
    /// Force reconnection (if supported)
    async fn reconnect(&mut self) -> Result<(), TransportError>;
}

/// Transport health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportHealth {
    pub is_connected: bool,
    pub last_ping: Option<Duration>,
    pub connection_age: Duration,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub last_error: Option<String>,
}

/// Transport performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportMetrics {
    pub total_connections: u64,
    pub successful_connections: u64,
    pub failed_connections: u64,
    pub reconnection_attempts: u64,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub total_messages_sent: u64,
    pub total_messages_received: u64,
    pub average_latency: Duration,
    pub uptime: Duration,
}

/// Transport layer errors with rich context and proper error chaining
#[derive(thiserror::Error, Debug)]
pub enum TransportError {
    /// I/O operation failed
    #[error("I/O error during {operation}: {message}")]
    IoError {
        /// Description of the operation that failed
        message: String,
        /// Transport operation being performed
        operation: String,
        /// Underlying I/O error
        #[source]
        source: io::Error,
    },
    
    /// JSON serialization/deserialization failed
    #[error("Serialization error for {data_type} during {operation}: {message}")]
    SerializationError {
        /// Description of the serialization failure
        message: String,
        /// Type of data being serialized
        data_type: String,
        /// Transport operation being performed
        operation: String,
        /// Underlying serde error
        #[source]
        source: serde_json::Error,
    },
    
    /// WebSocket connection or communication error
    #[error("WebSocket error during {operation} to {endpoint}: {message}")]
    WebSocketError {
        /// Description of the WebSocket error
        message: String,
        /// WebSocket endpoint
        endpoint: String,
        /// Transport operation being performed
        operation: String,
        /// Underlying WebSocket error
        #[source]
        source: tokio_tungstenite::tungstenite::Error,
    },
    
    /// HTTP request/response error
    #[error("HTTP error during {operation} to {endpoint}{}: {message}", status_code.map(|c| format!(" (status {})", c)).unwrap_or_default())]
    HttpError {
        /// Description of the HTTP error
        message: String,
        /// HTTP endpoint
        endpoint: String,
        /// HTTP status code if available
        status_code: Option<u16>,
        /// Transport operation being performed
        operation: String,
        /// Underlying HTTP client error
        #[source]
        source: reqwest::Error,
    },
    
    /// Connection establishment or maintenance failed
    #[error("Connection error to {endpoint} via {transport_type}: {message}")]
    ConnectionError {
        /// Description of the connection failure
        message: String,
        /// Connection endpoint
        endpoint: String,
        /// Transport type (WebSocket, stdio, HTTP)
        transport_type: String,
        /// Number of retry attempts made
        retry_count: u32,
    },
    
    /// Protocol-level communication error
    #[error("Protocol error during {operation}: {message}. Expected: {expected}, Received: {received}")]
    ProtocolError {
        /// Description of the protocol violation
        message: String,
        /// Protocol operation being performed
        operation: String,
        /// What was expected according to protocol
        expected: String,
        /// What was actually received
        received: String,
    },
}

impl From<io::Error> for TransportError {
    fn from(err: io::Error) -> Self {
        TransportError::IoError {
            message: err.to_string(),
            operation: "unknown".to_string(),
            source: err,
        }
    }
}

impl From<serde_json::Error> for TransportError {
    fn from(err: serde_json::Error) -> Self {
        TransportError::SerializationError {
            message: err.to_string(),
            data_type: "unknown".to_string(),
            operation: "unknown".to_string(),
            source: err,
        }
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for TransportError {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        TransportError::WebSocketError {
            message: err.to_string(),
            endpoint: "unknown".to_string(),
            operation: "unknown".to_string(),
            source: err,
        }
    }
}

impl From<reqwest::Error> for TransportError {
    fn from(err: reqwest::Error) -> Self {
        let status_code = err.status().map(|s| s.as_u16());
        let endpoint = err.url().map(|u| u.to_string()).unwrap_or_else(|| "unknown".to_string());
        
        TransportError::HttpError {
            message: err.to_string(),
            endpoint,
            status_code,
            operation: "unknown".to_string(),
            source: err,
        }
    }
}

// Display and Error traits are now automatically implemented by thiserror

impl TransportError {
    /// Create an I/O error with context
    pub fn io_error(
        message: impl Into<String>,
        operation: impl Into<String>,
        source: io::Error,
    ) -> Self {
        Self::IoError {
            message: message.into(),
            operation: operation.into(),
            source,
        }
    }

    /// Create a serialization error with context
    pub fn serialization_error(
        message: impl Into<String>,
        data_type: impl Into<String>,
        operation: impl Into<String>,
        source: serde_json::Error,
    ) -> Self {
        Self::SerializationError {
            message: message.into(),
            data_type: data_type.into(),
            operation: operation.into(),
            source,
        }
    }

    /// Create a WebSocket error with context
    pub fn websocket_error(
        message: impl Into<String>,
        endpoint: impl Into<String>,
        operation: impl Into<String>,
        source: tokio_tungstenite::tungstenite::Error,
    ) -> Self {
        Self::WebSocketError {
            message: message.into(),
            endpoint: endpoint.into(),
            operation: operation.into(),
            source,
        }
    }

    /// Create an HTTP error with context
    pub fn http_error(
        message: impl Into<String>,
        endpoint: impl Into<String>,
        operation: impl Into<String>,
        status_code: Option<u16>,
        source: reqwest::Error,
    ) -> Self {
        Self::HttpError {
            message: message.into(),
            endpoint: endpoint.into(),
            status_code,
            operation: operation.into(),
            source,
        }
    }

    /// Create a connection error
    pub fn connection_error(
        message: impl Into<String>,
        endpoint: impl Into<String>,
        transport_type: impl Into<String>,
        retry_count: u32,
    ) -> Self {
        Self::ConnectionError {
            message: message.into(),
            endpoint: endpoint.into(),
            transport_type: transport_type.into(),
            retry_count,
        }
    }

    /// Create a protocol error
    pub fn protocol_error(
        message: impl Into<String>,
        operation: impl Into<String>,
        expected: impl Into<String>,
        received: impl Into<String>,
    ) -> Self {
        Self::ProtocolError {
            message: message.into(),
            operation: operation.into(),
            expected: expected.into(),
            received: received.into(),
        }
    }
}

// Implement error categorization for TransportError
impl workflow_engine_core::error::ErrorExt for TransportError {
    fn category(&self) -> workflow_engine_core::error::ErrorCategory {
        use workflow_engine_core::error::ErrorCategory;
        match self {
            // Transient errors that may succeed on retry
            Self::IoError { .. } |
            Self::WebSocketError { .. } |
            Self::HttpError { .. } |
            Self::ConnectionError { .. } => ErrorCategory::Transient,
            
            // System errors (infrastructure, dependencies)
            Self::SerializationError { .. } => ErrorCategory::System,
            
            // User/business errors 
            Self::ProtocolError { .. } => ErrorCategory::Business,
        }
    }
    
    fn severity(&self) -> workflow_engine_core::error::ErrorSeverity {
        use workflow_engine_core::error::ErrorSeverity;
        match self {
            // Critical - connection failures
            Self::ConnectionError { .. } => ErrorSeverity::Critical,
            
            // Error - transport failures
            Self::IoError { .. } |
            Self::WebSocketError { .. } |
            Self::HttpError { .. } => ErrorSeverity::Error,
            
            // Warning - protocol and serialization issues
            Self::SerializationError { .. } |
            Self::ProtocolError { .. } => ErrorSeverity::Warning,
        }
    }
    
    fn error_code(&self) -> &'static str {
        match self {
            Self::IoError { .. } => "MCP_TRANSPORT_IO_ERROR",
            Self::SerializationError { .. } => "MCP_TRANSPORT_SERIALIZATION_ERROR",
            Self::WebSocketError { .. } => "MCP_TRANSPORT_WEBSOCKET_ERROR",
            Self::HttpError { .. } => "MCP_TRANSPORT_HTTP_ERROR",
            Self::ConnectionError { .. } => "MCP_TRANSPORT_CONNECTION_ERROR",
            Self::ProtocolError { .. } => "MCP_TRANSPORT_PROTOCOL_ERROR",
        }
    }
}

// Convert TransportError to WorkflowError
impl From<TransportError> for workflow_engine_core::error::WorkflowError {
    fn from(err: TransportError) -> Self {
        match &err {
            TransportError::IoError { message, operation, .. } => {
                workflow_engine_core::error::WorkflowError::MCPTransportError {
                    message: format!("I/O error during {}: {}", operation, message),
                    server_name: "unknown".to_string(),
                    transport_type: "stdio".to_string(),
                    operation: operation.clone(),
                    source: Some(Box::new(err)),
                }
            },
            TransportError::SerializationError { message, data_type, operation, .. } => {
                workflow_engine_core::error::WorkflowError::SerializationError {
                    message: format!("MCP serialization error for {}: {}", data_type, message),
                    type_name: data_type.clone(),
                    context: format!("during MCP {}", operation),
                    source: None, // Can't extract serde_json::Error due to Box constraints
                }
            },
            TransportError::WebSocketError { message, endpoint, operation, .. } => {
                workflow_engine_core::error::WorkflowError::MCPConnectionError {
                    message: format!("WebSocket error during {}: {}", operation, message),
                    server_name: "unknown".to_string(),
                    transport_type: "WebSocket".to_string(),
                    endpoint: endpoint.clone(),
                    retry_count: 0,
                    source: Some(Box::new(err)),
                }
            },
            TransportError::HttpError { message, endpoint, status_code, operation, .. } => {
                workflow_engine_core::error::WorkflowError::ApiError {
                    message: format!("HTTP transport error during {}: {}", operation, message),
                    service: "mcp_server".to_string(),
                    endpoint: endpoint.clone(),
                    status_code: *status_code,
                    retry_count: 0,
                    source: Some(Box::new(err)),
                }
            },
            TransportError::ConnectionError { message, endpoint, transport_type, retry_count } => {
                workflow_engine_core::error::WorkflowError::MCPConnectionError {
                    message: message.clone(),
                    server_name: "unknown".to_string(),
                    transport_type: transport_type.clone(),
                    endpoint: endpoint.clone(),
                    retry_count: *retry_count,
                    source: Some(Box::new(err)),
                }
            },
            TransportError::ProtocolError { message, operation, expected, received } => {
                workflow_engine_core::error::WorkflowError::MCPProtocolError {
                    message: message.clone(),
                    server_name: "unknown".to_string(),
                    expected: expected.clone(),
                    received: received.clone(),
                    message_type: operation.clone(),
                    source: Some(Box::new(err)),
                }
            },
        }
    }
}

impl Default for TransportMetrics {
    fn default() -> Self {
        Self {
            total_connections: 0,
            successful_connections: 0,
            failed_connections: 0,
            reconnection_attempts: 0,
            total_bytes_sent: 0,
            total_bytes_received: 0,
            total_messages_sent: 0,
            total_messages_received: 0,
            average_latency: Duration::from_millis(0),
            uptime: Duration::from_secs(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::Duration;

    // Task 5.2.1: Create tests for HTTP transport in `src/core/mcp/transport.rs`
    
    #[test]
    fn test_http_transport_creation() {
        let transport = HttpTransport::new("http://localhost:8080".to_string());
        assert!(transport.is_connected()); // HTTP is always "connected"
    }

    #[test]
    fn test_http_transport_with_auth() {
        let transport = HttpTransport::new("http://localhost:8080".to_string())
            .with_auth_token("test-token".to_string());
        
        // Verify transport is created with auth token
        assert!(transport.auth_token.is_some());
        assert_eq!(transport.auth_token.unwrap(), "test-token");
    }

    #[tokio::test]
    async fn test_http_transport_connect() {
        let mut transport = HttpTransport::new("http://localhost:8080".to_string());
        let result = transport.connect().await;
        assert!(result.is_ok()); // HTTP connect is always successful
    }

    #[tokio::test]
    async fn test_http_transport_disconnect() {
        let mut transport = HttpTransport::new("http://localhost:8080".to_string());
        let result = transport.disconnect().await;
        assert!(result.is_ok()); // HTTP disconnect is always successful
    }

    #[tokio::test]
    async fn test_http_transport_receive_error() {
        let mut transport = HttpTransport::new("http://localhost:8080".to_string());
        let result = transport.receive().await;
        
        // HTTP transport doesn't support receive
        assert!(result.is_err());
        match result.unwrap_err() {
            TransportError::ProtocolError { message, .. } => {
                assert!(message.contains("HTTP transport does not support receive"));
            }
            _ => panic!("Expected ProtocolError"),
        }
    }

    #[tokio::test]
    async fn test_http_transport_health_check() {
        let mut transport = HttpTransport::new("http://localhost:8080".to_string());
        let health = transport.health_check().await.unwrap();
        
        assert!(health.is_connected);
        assert_eq!(health.connection_age, Duration::from_secs(0));
        assert!(health.last_error.is_none());
    }

    #[tokio::test]
    async fn test_http_transport_metrics() {
        let transport = HttpTransport::new("http://localhost:8080".to_string());
        let metrics = transport.get_metrics();
        
        // Default metrics should be zero
        assert_eq!(metrics.total_connections, 0);
        assert_eq!(metrics.successful_connections, 0);
        assert_eq!(metrics.failed_connections, 0);
        assert_eq!(metrics.total_messages_sent, 0);
        assert_eq!(metrics.total_messages_received, 0);
    }

    #[tokio::test]
    async fn test_http_transport_reconnect() {
        let mut transport = HttpTransport::new("http://localhost:8080".to_string());
        let result = transport.reconnect().await;
        assert!(result.is_ok()); // HTTP reconnect is always successful
    }

    // Task 5.2.2: Add WebSocket transport tests
    
    #[test]
    fn test_websocket_transport_creation() {
        let transport = WebSocketTransport::new("ws://localhost:8080".to_string());
        assert!(!transport.is_connected()); // WebSocket starts disconnected
    }

    #[test]
    fn test_websocket_transport_with_config() {
        let reconnect_config = ReconnectConfig {
            enabled: true,
            max_attempts: 10,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 3.0,
        };
        
        let transport = WebSocketTransport::new("ws://localhost:8080".to_string())
            .with_reconnect_config(reconnect_config.clone())
            .with_heartbeat_interval(Some(Duration::from_secs(15)));
        
        // Verify configuration is applied
        assert_eq!(transport.reconnect_config.enabled, true);
        assert_eq!(transport.reconnect_config.max_attempts, 10);
        assert_eq!(transport.reconnect_config.backoff_multiplier, 3.0);
        assert_eq!(transport.heartbeat_interval, Some(Duration::from_secs(15)));
    }

    #[tokio::test]
    async fn test_websocket_transport_health_check_disconnected() {
        let mut transport = WebSocketTransport::new("ws://localhost:8080".to_string());
        let health = transport.health_check().await.unwrap();
        
        assert!(!health.is_connected);
        assert!(health.last_ping.is_none());
        assert_eq!(health.connection_age, Duration::from_secs(0));
    }

    #[tokio::test]
    async fn test_websocket_transport_metrics() {
        let transport = WebSocketTransport::new("ws://localhost:8080".to_string());
        let metrics = transport.get_metrics();
        
        // Default metrics should be zero
        assert_eq!(metrics.total_connections, 0);
        assert_eq!(metrics.successful_connections, 0);
        assert_eq!(metrics.failed_connections, 0);
        assert_eq!(metrics.uptime, Duration::from_secs(0));
    }

    #[tokio::test]
    async fn test_websocket_transport_send_when_disconnected() {
        let mut transport = WebSocketTransport::new("ws://localhost:8080".to_string());
        let request = McpRequest::ListTools {
            id: "test-1".to_string(),
        };
        
        let result = transport.send(request).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TransportError::ConnectionError { message, .. } => {
                assert_eq!(message, "Not connected");
            }
            _ => panic!("Expected ConnectionError"),
        }
    }

    #[tokio::test]
    async fn test_websocket_transport_receive_when_disconnected() {
        let mut transport = WebSocketTransport::new("ws://localhost:8080".to_string());
        
        let result = transport.receive().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TransportError::ConnectionError { message, .. } => {
                assert_eq!(message, "Not connected");
            }
            _ => panic!("Expected ConnectionError"),
        }
    }

    #[tokio::test]
    async fn test_websocket_transport_ping_when_disconnected() {
        let mut transport = WebSocketTransport::new("ws://localhost:8080".to_string());
        
        let result = transport.ping().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TransportError::ConnectionError { message, .. } => {
                assert_eq!(message, "Not connected");
            }
            _ => panic!("Expected ConnectionError"),
        }
    }

    // Task 5.2.3: Test stdio transport implementation
    
    #[test]
    fn test_stdio_transport_creation() {
        let transport = StdioTransport::new(
            "python".to_string(),
            vec!["-m".to_string(), "mcp_server".to_string()],
        );
        assert!(!transport.is_connected()); // Stdio starts disconnected
    }

    #[test]
    fn test_stdio_transport_with_restart_config() {
        let transport = StdioTransport::new(
            "python".to_string(),
            vec!["-m".to_string(), "mcp_server".to_string()],
        ).with_restart_config(true, 5);
        
        // Verify restart configuration
        assert_eq!(transport.auto_restart, true);
        assert_eq!(transport.max_restarts, 5);
    }

    #[tokio::test]
    async fn test_stdio_transport_send_when_disconnected() {
        let mut transport = StdioTransport::new(
            "python".to_string(),
            vec!["-m".to_string(), "mcp_server".to_string()],
        );
        
        let request = McpRequest::ListTools {
            id: "test-1".to_string(),
        };
        
        let result = transport.send(request).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TransportError::ConnectionError { message, .. } => {
                assert_eq!(message, "Not connected");
            }
            _ => panic!("Expected ConnectionError"),
        }
    }

    #[tokio::test]
    async fn test_stdio_transport_receive_when_disconnected() {
        let mut transport = StdioTransport::new(
            "python".to_string(),
            vec!["-m".to_string(), "mcp_server".to_string()],
        );
        
        let result = transport.receive().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TransportError::ConnectionError { message, .. } => {
                assert_eq!(message, "Not connected");
            }
            _ => panic!("Expected ConnectionError"),
        }
    }

    #[tokio::test]
    async fn test_stdio_transport_health_check_disconnected() {
        let mut transport = StdioTransport::new(
            "python".to_string(),
            vec!["-m".to_string(), "mcp_server".to_string()],
        );
        
        let health = transport.health_check().await.unwrap();
        assert!(!health.is_connected);
        assert!(health.last_ping.is_none()); // Stdio doesn't support ping
        assert_eq!(health.connection_age, Duration::from_secs(0));
        assert!(health.last_error.is_none());
    }

    #[tokio::test]
    async fn test_stdio_transport_ping() {
        let mut transport = StdioTransport::new(
            "python".to_string(),
            vec!["-m".to_string(), "mcp_server".to_string()],
        );
        
        // Ping when disconnected should fail
        let result = transport.ping().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TransportError::ConnectionError { message, .. } => {
                assert_eq!(message, "Not connected");
            }
            _ => panic!("Expected ConnectionError"),
        }
    }

    #[tokio::test]
    async fn test_stdio_transport_metrics() {
        let transport = StdioTransport::new(
            "python".to_string(),
            vec!["-m".to_string(), "mcp_server".to_string()],
        );
        
        let metrics = transport.get_metrics();
        
        // Default metrics should be zero
        assert_eq!(metrics.total_connections, 0);
        assert_eq!(metrics.successful_connections, 0);
        assert_eq!(metrics.failed_connections, 0);
        assert_eq!(metrics.reconnection_attempts, 0);
        assert_eq!(metrics.uptime, Duration::from_secs(0));
    }

    // Task 5.2.4: Test transport connection lifecycle
    
    #[tokio::test]
    async fn test_transport_lifecycle_metrics_update() {
        let mut transport = HttpTransport::new("http://localhost:8080".to_string());
        
        // Connect should not fail for HTTP
        let connect_result = transport.connect().await;
        assert!(connect_result.is_ok());
        
        // Disconnect should not fail for HTTP
        let disconnect_result = transport.disconnect().await;
        assert!(disconnect_result.is_ok());
    }

    #[test]
    fn test_reconnect_config_default() {
        let config = ReconnectConfig::default();
        
        assert_eq!(config.enabled, true);
        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.initial_delay, Duration::from_millis(500));
        assert_eq!(config.max_delay, Duration::from_secs(30));
        assert_eq!(config.backoff_multiplier, 2.0);
    }

    #[test]
    fn test_http_pool_config_default() {
        let config = HttpPoolConfig::default();
        
        assert_eq!(config.max_connections_per_host, 10);
        assert_eq!(config.connect_timeout, Duration::from_secs(10));
        assert_eq!(config.request_timeout, Duration::from_secs(30));
        assert_eq!(config.keep_alive_timeout, Duration::from_secs(90));
    }

    #[test]
    fn test_transport_metrics_default() {
        let metrics = TransportMetrics::default();
        
        assert_eq!(metrics.total_connections, 0);
        assert_eq!(metrics.successful_connections, 0);
        assert_eq!(metrics.failed_connections, 0);
        assert_eq!(metrics.reconnection_attempts, 0);
        assert_eq!(metrics.total_bytes_sent, 0);
        assert_eq!(metrics.total_bytes_received, 0);
        assert_eq!(metrics.total_messages_sent, 0);
        assert_eq!(metrics.total_messages_received, 0);
        assert_eq!(metrics.average_latency, Duration::from_millis(0));
        assert_eq!(metrics.uptime, Duration::from_secs(0));
    }

    // Task 5.2.5: Add tests for transport error handling
    
    #[test]
    fn test_transport_error_display() {
        let io_error = TransportError::IoError {
            message: "Connection refused".to_string(),
            operation: "connect".to_string(),
            source: std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                "Connection refused",
            ),
        };
        assert!(format!("{}", io_error).contains("I/O error"));

        let serialization_error = TransportError::SerializationError {
            message: "Invalid JSON".to_string(),
            data_type: "McpMessage".to_string(),
            operation: "deserialize".to_string(),
            source: serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err(),
        };
        assert!(format!("{}", serialization_error).contains("Serialization error"));

        let connection_error = TransportError::connection_error(
            "Test connection error",
            "test-endpoint",
            "test",
            0
        );
        assert!(format!("{}", connection_error).contains("Test connection error"));

        let protocol_error = TransportError::protocol_error(
            "Test protocol error",
            "test-operation",
            "expected",
            "received"
        );
        assert!(format!("{}", protocol_error).contains("Test protocol error"));
    }

    #[test]
    fn test_transport_error_from_io() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let transport_error: TransportError = io_error.into();
        
        match transport_error {
            TransportError::IoError { .. } => {},
            _ => panic!("Expected IoError variant"),
        }
    }

    #[test]
    fn test_transport_error_from_serde() {
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let transport_error: TransportError = json_error.into();
        
        match transport_error {
            TransportError::SerializationError { .. } => {},
            _ => panic!("Expected SerializationError variant"),
        }
    }

    // Task 5.2.6: Test message framing and buffering
    
    #[test]
    fn test_transport_type_serialization() {
        let stdio_transport = TransportType::Stdio {
            command: "python".to_string(),
            args: vec!["-m".to_string(), "server".to_string()],
            auto_restart: true,
            max_restarts: 3,
        };
        
        let json = serde_json::to_value(&stdio_transport).unwrap();
        // For tagged enums, the variant is the top-level key
        assert_eq!(json["Stdio"]["command"], "python");
        assert_eq!(json["Stdio"]["args"][0], "-m");
        assert_eq!(json["Stdio"]["args"][1], "server");
        assert_eq!(json["Stdio"]["auto_restart"], true);
        assert_eq!(json["Stdio"]["max_restarts"], 3);

        let ws_transport = TransportType::WebSocket {
            url: "ws://localhost:8080".to_string(),
            heartbeat_interval: Some(Duration::from_secs(30)),
            reconnect_config: ReconnectConfig::default(),
        };
        
        let json = serde_json::to_value(&ws_transport).unwrap();
        assert_eq!(json["WebSocket"]["url"], "ws://localhost:8080");
        assert!(json["WebSocket"]["heartbeat_interval"].is_object());
        assert!(json["WebSocket"]["reconnect_config"].is_object());

        let http_transport = TransportType::Http {
            base_url: "http://localhost:8080".to_string(),
            pool_config: HttpPoolConfig::default(),
        };
        
        let json = serde_json::to_value(&http_transport).unwrap();
        assert_eq!(json["Http"]["base_url"], "http://localhost:8080");
        assert!(json["Http"]["pool_config"].is_object());
    }

    #[test]
    fn test_transport_health_serialization() {
        let health = TransportHealth {
            is_connected: true,
            last_ping: Some(Duration::from_millis(42)),
            connection_age: Duration::from_secs(300),
            bytes_sent: 1024,
            bytes_received: 2048,
            messages_sent: 10,
            messages_received: 15,
            last_error: Some("Test error".to_string()),
        };
        
        let json = serde_json::to_value(&health).unwrap();
        assert_eq!(json["is_connected"], true);
        assert!(json["last_ping"].is_object());
        assert!(json["connection_age"].is_object());
        assert_eq!(json["bytes_sent"], 1024);
        assert_eq!(json["bytes_received"], 2048);
        assert_eq!(json["messages_sent"], 10);
        assert_eq!(json["messages_received"], 15);
        assert_eq!(json["last_error"], "Test error");
    }

    #[test]
    fn test_transport_metrics_serialization() {
        let metrics = TransportMetrics {
            total_connections: 100,
            successful_connections: 95,
            failed_connections: 5,
            reconnection_attempts: 3,
            total_bytes_sent: 1_000_000,
            total_bytes_received: 2_000_000,
            total_messages_sent: 1000,
            total_messages_received: 1500,
            average_latency: Duration::from_millis(50),
            uptime: Duration::from_secs(3600),
        };
        
        let json = serde_json::to_value(&metrics).unwrap();
        assert_eq!(json["total_connections"], 100);
        assert_eq!(json["successful_connections"], 95);
        assert_eq!(json["failed_connections"], 5);
        assert_eq!(json["reconnection_attempts"], 3);
        assert_eq!(json["total_bytes_sent"], 1_000_000);
        assert_eq!(json["total_bytes_received"], 2_000_000);
        assert_eq!(json["total_messages_sent"], 1000);
        assert_eq!(json["total_messages_received"], 1500);
        assert!(json["average_latency"].is_object());
        assert!(json["uptime"].is_object());
    }

    #[tokio::test]
    async fn test_stdio_transport_attempt_restart_disabled() {
        let mut transport = StdioTransport::new(
            "python".to_string(),
            vec!["-m".to_string(), "server".to_string()],
        ).with_restart_config(false, 3);
        
        // Restart should fail when disabled
        let result = transport.attempt_restart().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TransportError::ConnectionError { message, .. } => {
                assert_eq!(message, "Process restart limit reached");
            }
            _ => panic!("Expected ConnectionError"),
        }
    }

    #[tokio::test]
    async fn test_websocket_transport_attempt_reconnect_disabled() {
        let mut config = ReconnectConfig::default();
        config.enabled = false;
        
        let mut transport = WebSocketTransport::new("ws://localhost:8080".to_string())
            .with_reconnect_config(config);
        
        // Reconnect should fail when disabled
        let result = transport.attempt_reconnect().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TransportError::ConnectionError { message, .. } => {
                assert_eq!(message, "Reconnection limit reached");
            }
            _ => panic!("Expected ConnectionError"),
        }
    }
}

pub struct StdioTransport {
    command: String,
    args: Vec<String>,
    auto_restart: bool,
    max_restarts: u32,
    process: Option<Child>,
    reader: Option<BufReader<tokio::process::ChildStdout>>,
    writer: Option<tokio::process::ChildStdin>,
    restart_count: u32,
    connected_at: Option<Instant>,
    metrics: TransportMetrics,
    last_error: Option<String>,
}

impl StdioTransport {
    pub fn new(command: String, args: Vec<String>) -> Self {
        Self {
            command,
            args,
            auto_restart: true,
            max_restarts: 3,
            process: None,
            reader: None,
            writer: None,
            restart_count: 0,
            connected_at: None,
            metrics: TransportMetrics::default(),
            last_error: None,
        }
    }
    
    pub fn with_restart_config(mut self, auto_restart: bool, max_restarts: u32) -> Self {
        self.auto_restart = auto_restart;
        self.max_restarts = max_restarts;
        self
    }
    
    async fn attempt_restart(&mut self) -> Result<(), TransportError> {
        if !self.auto_restart || self.restart_count >= self.max_restarts {
            return Err(TransportError::ConnectionError {
                message: "Process restart limit reached".to_string(),
                endpoint: "stdio".to_string(),
                transport_type: "stdio".to_string(),
                retry_count: self.restart_count,
            });
        }
        
        log::warn!(
            "Attempting to restart stdio process (attempt {}/{})", 
            self.restart_count + 1, 
            self.max_restarts
        );
        
        self.restart_count += 1;
        self.metrics.reconnection_attempts += 1;
        
        // Wait a bit before restarting
        let delay = Duration::from_millis(500 * (self.restart_count as u64));
        sleep(delay).await;
        
        self.connect().await
    }
}

#[async_trait]
impl McpTransport for StdioTransport {
    async fn connect(&mut self) -> Result<(), TransportError> {
        self.metrics.total_connections += 1;
        
        let result = async {
            let mut child = Command::new(&self.command)
                .args(&self.args)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()?;

            let stdout = child.stdout.take()
                .ok_or_else(|| TransportError::ConnectionError {
                    message: "Failed to get stdout".to_string(),
                    endpoint: "stdio".to_string(),
                    transport_type: "stdio".to_string(),
                    retry_count: 0,
                })?;
            let stdin = child.stdin.take()
                .ok_or_else(|| TransportError::ConnectionError {
                    message: "Failed to get stdin".to_string(),
                    endpoint: "stdio".to_string(),
                    transport_type: "stdio".to_string(),
                    retry_count: 0,
                })?;

            self.reader = Some(BufReader::new(stdout));
            self.writer = Some(stdin);
            self.process = Some(child);
            self.connected_at = Some(Instant::now());
            
            Ok::<(), TransportError>(())
        }.await;
        
        match result {
            Ok(_) => {
                self.metrics.successful_connections += 1;
                self.last_error = None;
                log::debug!("Successfully connected stdio transport to {}", self.command);
                Ok(())
            }
            Err(e) => {
                self.metrics.failed_connections += 1;
                self.last_error = Some(e.to_string());
                Err(e)
            }
        }
    }

    async fn send(&mut self, message: McpRequest) -> Result<(), TransportError> {
        let writer = self.writer.as_mut()
            .ok_or_else(|| TransportError::ConnectionError {
                message: "Not connected".to_string(),
                endpoint: "stdio".to_string(),
                transport_type: "stdio".to_string(),
                retry_count: 0,
            })?;

        let json = serde_json::to_string(&message)?;
        let data = format!("{}\n", json);
        
        writer.write_all(data.as_bytes()).await?;
        writer.flush().await?;
        
        self.metrics.total_messages_sent += 1;
        self.metrics.total_bytes_sent += data.len() as u64;

        Ok(())
    }

    async fn receive(&mut self) -> Result<McpResponse, TransportError> {
        let reader = self.reader.as_mut()
            .ok_or_else(|| TransportError::ConnectionError {
                message: "Not connected".to_string(),
                endpoint: "stdio".to_string(),
                transport_type: "stdio".to_string(),
                retry_count: 0,
            })?;

        let mut line = String::new();
        reader.read_line(&mut line).await?;

        if line.is_empty() {
            return Err(TransportError::ConnectionError {
                message: "Connection closed".to_string(),
                endpoint: "stdio".to_string(),
                transport_type: "stdio".to_string(),
                retry_count: 0,
            });
        }

        let response: McpResponse = serde_json::from_str(&line)?;
        Ok(response)
    }

    async fn disconnect(&mut self) -> Result<(), TransportError> {
        if let Some(mut process) = self.process.take() {
            process.kill().await?;
        }
        self.reader = None;
        self.writer = None;
        self.connected_at = None;
        Ok(())
    }
    
    fn is_connected(&self) -> bool {
        self.process.is_some() && self.reader.is_some() && self.writer.is_some()
    }
    
    async fn health_check(&mut self) -> Result<TransportHealth, TransportError> {
        let is_connected = self.is_connected();
        let connection_age = self.connected_at
            .map(|t| t.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
            
        Ok(TransportHealth {
            is_connected,
            last_ping: None, // Stdio doesn't support ping
            connection_age,
            bytes_sent: self.metrics.total_bytes_sent,
            bytes_received: self.metrics.total_bytes_received,
            messages_sent: self.metrics.total_messages_sent,
            messages_received: self.metrics.total_messages_received,
            last_error: self.last_error.clone(),
        })
    }
    
    async fn ping(&mut self) -> Result<Duration, TransportError> {
        // Stdio transport doesn't support ping, return a synthetic response
        if !self.is_connected() {
            return Err(TransportError::ConnectionError {
                message: "Not connected".to_string(),
                endpoint: "unknown".to_string(),
                transport_type: "unknown".to_string(),
                retry_count: 0,
            });
        }
        Ok(Duration::from_millis(0))
    }
    
    fn get_metrics(&self) -> TransportMetrics {
        let mut metrics = self.metrics.clone();
        if let Some(connected_at) = self.connected_at {
            metrics.uptime = connected_at.elapsed();
        }
        metrics
    }
    
    async fn reconnect(&mut self) -> Result<(), TransportError> {
        self.disconnect().await?;
        self.attempt_restart().await
    }
}

pub struct WebSocketTransport {
    url: String,
    stream: Option<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
    reconnect_config: ReconnectConfig,
    heartbeat_interval: Option<Duration>,
    connected_at: Option<Instant>,
    metrics: TransportMetrics,
    last_error: Option<String>,
    reconnect_attempts: u32,
}

impl WebSocketTransport {
    pub fn new(url: String) -> Self {
        Self {
            url,
            stream: None,
            reconnect_config: ReconnectConfig::default(),
            heartbeat_interval: Some(Duration::from_secs(30)),
            connected_at: None,
            metrics: TransportMetrics::default(),
            last_error: None,
            reconnect_attempts: 0,
        }
    }
    
    pub fn with_reconnect_config(mut self, config: ReconnectConfig) -> Self {
        self.reconnect_config = config;
        self
    }
    
    pub fn with_heartbeat_interval(mut self, interval: Option<Duration>) -> Self {
        self.heartbeat_interval = interval;
        self
    }
    
    async fn attempt_reconnect(&mut self) -> Result<(), TransportError> {
        if !self.reconnect_config.enabled || self.reconnect_attempts >= self.reconnect_config.max_attempts {
            return Err(TransportError::ConnectionError {
                message: "Reconnection limit reached".to_string(),
                endpoint: self.url.clone(),
                transport_type: "WebSocket".to_string(),
                retry_count: self.reconnect_attempts,
            });
        }
        
        log::warn!(
            "Attempting to reconnect WebSocket (attempt {}/{})", 
            self.reconnect_attempts + 1, 
            self.reconnect_config.max_attempts
        );
        
        self.reconnect_attempts += 1;
        self.metrics.reconnection_attempts += 1;
        
        // Calculate delay with exponential backoff
        let delay_ms = self.reconnect_config.initial_delay.as_millis() as f64 * 
                      self.reconnect_config.backoff_multiplier.powi(self.reconnect_attempts as i32);
        let delay = Duration::from_millis(delay_ms.min(self.reconnect_config.max_delay.as_millis() as f64) as u64);
        
        sleep(delay).await;
        
        self.connect().await
    }
}

#[async_trait]
impl McpTransport for WebSocketTransport {
    async fn connect(&mut self) -> Result<(), TransportError> {
        self.metrics.total_connections += 1;
        
        let result = connect_async(&self.url).await;
        match result {
            Ok((ws_stream, _)) => {
                self.stream = Some(ws_stream);
                self.connected_at = Some(Instant::now());
                self.reconnect_attempts = 0; // Reset on successful connection
                self.metrics.successful_connections += 1;
                self.last_error = None;
                log::debug!("Successfully connected WebSocket to {}", self.url);
                Ok(())
            }
            Err(e) => {
                self.metrics.failed_connections += 1;
                let error = TransportError::websocket_error(
                    "Failed to connect",
                    self.url.clone(),
                    "connect",
                    e
                );
                self.last_error = Some(error.to_string());
                Err(error)
            }
        }
    }

    async fn send(&mut self, message: McpRequest) -> Result<(), TransportError> {
        let stream = self.stream.as_mut()
            .ok_or_else(|| TransportError::ConnectionError {
                message: "Not connected".to_string(),
                endpoint: "stdio".to_string(),
                transport_type: "stdio".to_string(),
                retry_count: 0,
            })?;

        let json = serde_json::to_string(&message)?;
        stream.send(Message::Text(json.clone())).await?;
        
        self.metrics.total_messages_sent += 1;
        self.metrics.total_bytes_sent += json.len() as u64;
        
        Ok(())
    }

    async fn receive(&mut self) -> Result<McpResponse, TransportError> {
        let stream = self.stream.as_mut()
            .ok_or_else(|| TransportError::ConnectionError {
                message: "Not connected".to_string(),
                endpoint: "stdio".to_string(),
                transport_type: "stdio".to_string(),
                retry_count: 0,
            })?;

        if let Some(msg) = stream.next().await {
            let msg = msg?;
            match msg {
                Message::Text(text) => {
                    self.metrics.total_messages_received += 1;
                    self.metrics.total_bytes_received += text.len() as u64;
                    
                    let response: McpResponse = serde_json::from_str(&text)?;
                    Ok(response)
                }
                Message::Close(_) => {
                    Err(TransportError::connection_error(
                        "Connection closed",
                        self.url.clone(),
                        "websocket",
                        self.reconnect_attempts
                    ))
                }
                Message::Ping(_) => {
                    // Handle ping frames automatically
                    self.receive().await
                }
                Message::Pong(_) => {
                    // Handle pong frames automatically
                    self.receive().await
                }
                _ => Err(TransportError::protocol_error(
                    "Unexpected message type",
                    "receive",
                    "Text message",
                    format!("{:?}", msg)
                )),
            }
        } else {
            Err(TransportError::connection_error(
                "Connection closed",
                self.url.clone(),
                "websocket",
                self.reconnect_attempts
            ))
        }
    }

    async fn disconnect(&mut self) -> Result<(), TransportError> {
        if let Some(mut stream) = self.stream.take() {
            stream.close(None).await?;
        }
        self.connected_at = None;
        Ok(())
    }
    
    fn is_connected(&self) -> bool {
        self.stream.is_some()
    }
    
    async fn health_check(&mut self) -> Result<TransportHealth, TransportError> {
        let is_connected = self.is_connected();
        let connection_age = self.connected_at
            .map(|t| t.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
            
        // Try to ping to get actual latency
        let last_ping = if is_connected {
            self.ping().await.ok()
        } else {
            None
        };
        
        Ok(TransportHealth {
            is_connected,
            last_ping,
            connection_age,
            bytes_sent: self.metrics.total_bytes_sent,
            bytes_received: self.metrics.total_bytes_received,
            messages_sent: self.metrics.total_messages_sent,
            messages_received: self.metrics.total_messages_received,
            last_error: self.last_error.clone(),
        })
    }
    
    async fn ping(&mut self) -> Result<Duration, TransportError> {
        let stream = self.stream.as_mut()
            .ok_or_else(|| TransportError::ConnectionError {
                message: "Not connected".to_string(),
                endpoint: "stdio".to_string(),
                transport_type: "stdio".to_string(),
                retry_count: 0,
            })?;
            
        let start = Instant::now();
        stream.send(Message::Ping(vec![])).await?;
        
        // Wait for pong response (with timeout)
        let result = timeout(Duration::from_secs(5), async {
            while let Some(msg) = stream.next().await {
                match msg? {
                    Message::Pong(_) => return Ok::<(), TransportError>(()),
                    Message::Text(text) => {
                        // Got a regular message, handle it
                        self.metrics.total_messages_received += 1;
                        self.metrics.total_bytes_received += text.len() as u64;
                        continue;
                    }
                    Message::Close(_) => {
                        return Err(TransportError::ConnectionError {
                message: "Connection closed".to_string(),
                endpoint: "stdio".to_string(),
                transport_type: "stdio".to_string(),
                retry_count: 0,
            });
                    }
                    _ => continue,
                }
            }
            Err(TransportError::connection_error(
                "Connection closed during ping",
                self.url.clone(),
                "websocket",
                self.reconnect_attempts
            ))
        }).await;
        
        match result {
            Ok(_) => Ok(start.elapsed()),
            Err(_) => Err(TransportError::connection_error(
                "Ping timeout",
                self.url.clone(),
                "websocket",
                self.reconnect_attempts
            )),
        }
    }
    
    fn get_metrics(&self) -> TransportMetrics {
        let mut metrics = self.metrics.clone();
        if let Some(connected_at) = self.connected_at {
            metrics.uptime = connected_at.elapsed();
        }
        metrics
    }
    
    async fn reconnect(&mut self) -> Result<(), TransportError> {
        self.disconnect().await?;
        self.attempt_reconnect().await
    }
}

#[derive(Debug)]
pub struct HttpTransport {
    base_url: String,
    client: reqwest::Client,
    auth_token: Option<String>,
}

impl HttpTransport {
    pub fn new(base_url: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30)) // 30-second default timeout
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
            
        Self {
            base_url,
            client,
            auth_token: None,
        }
    }
    
    pub fn with_auth_token(mut self, token: String) -> Self {
        self.auth_token = Some(token);
        self
    }
    
    pub fn set_auth_token(&mut self, token: Option<String>) {
        self.auth_token = token;
    }
    
    /// Send a request and wait for response (for HTTP-based MCP communication)
    pub async fn send_request(&self, request: McpRequest) -> Result<McpResponse, TransportError> {
        let mut request_builder = self.client
            .post(&format!("{}/mcp", self.base_url))
            .json(&request);
        
        // Add authentication header if available
        if let Some(ref token) = self.auth_token {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", token));
        }
        
        let response = request_builder.send().await?;

        if !response.status().is_success() {
            return Err(TransportError::connection_error(
                format!("HTTP error: {}", response.status()),
                self.base_url.clone(),
                "http",
                0
            ));
        }

        let mcp_response: McpResponse = response.json().await?;

        Ok(mcp_response)
    }
}

#[async_trait]
impl McpTransport for HttpTransport {
    async fn connect(&mut self) -> Result<(), TransportError> {
        Ok(())
    }

    async fn send(&mut self, message: McpRequest) -> Result<(), TransportError> {
        let mut request_builder = self.client
            .post(&format!("{}/mcp", self.base_url))
            .json(&message);
        
        // Add authentication header if available
        if let Some(ref token) = self.auth_token {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", token));
        }
        
        let response = request_builder.send().await?;

        if !response.status().is_success() {
            return Err(TransportError::connection_error(
                format!("HTTP error: {}", response.status()),
                self.base_url.clone(),
                "http",
                0
            ));
        }

        Ok(())
    }

    async fn receive(&mut self) -> Result<McpResponse, TransportError> {
        Err(TransportError::protocol_error(
            "HTTP transport does not support receive - use request/response pattern",
            "receive",
            "response",
            "not supported"
        ))
    }

    async fn disconnect(&mut self) -> Result<(), TransportError> {
        Ok(())
    }
    
    fn is_connected(&self) -> bool {
        true // HTTP client is always "connected"
    }
    
    async fn health_check(&mut self) -> Result<TransportHealth, TransportError> {
        Ok(TransportHealth {
            is_connected: true,
            last_ping: None,
            connection_age: Duration::from_secs(0),
            bytes_sent: 0,
            bytes_received: 0,
            messages_sent: 0,
            messages_received: 0,
            last_error: None,
        })
    }
    
    async fn ping(&mut self) -> Result<Duration, TransportError> {
        let start = Instant::now();
        // Perform a simple HTTP GET to check connectivity
        let response = self.client.get(&format!("{}/health", self.base_url)).send().await;
        match response {
            Ok(_) => Ok(start.elapsed()),
            Err(e) => Err(TransportError::http_error(
                "Health check failed",
                self.base_url.clone(),
                "ping",
                None,
                e
            )),
        }
    }
    
    fn get_metrics(&self) -> TransportMetrics {
        TransportMetrics::default()
    }
    
    async fn reconnect(&mut self) -> Result<(), TransportError> {
        // HTTP doesn't need reconnection
        Ok(())
    }
}