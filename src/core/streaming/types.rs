use serde::{Deserialize, Serialize};
use std::pin::Pin;
use futures_util::stream::Stream;
use crate::core::error::WorkflowError;

/// Represents a chunk of streaming data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    /// The content of this chunk
    pub content: String,
    /// Whether this is the final chunk in the stream
    pub is_final: bool,
    /// Optional metadata about the chunk
    pub metadata: Option<StreamMetadata>,
    /// Timestamp when this chunk was created
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Metadata associated with a streaming chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMetadata {
    /// The model that generated this chunk
    pub model: String,
    /// The provider that generated this chunk
    pub provider: String,
    /// Token count for this chunk (if available)
    pub token_count: Option<u32>,
    /// Total tokens used so far
    pub total_tokens: Option<u32>,
    /// Processing time for this chunk in milliseconds
    pub processing_time_ms: Option<u64>,
}

/// Configuration for streaming responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamConfig {
    /// Whether streaming is enabled
    pub enabled: bool,
    /// Maximum chunk size in characters
    pub max_chunk_size: Option<usize>,
    /// Minimum delay between chunks in milliseconds
    pub min_chunk_delay_ms: Option<u64>,
    /// Maximum delay before sending a chunk in milliseconds
    pub max_chunk_delay_ms: Option<u64>,
    /// Buffer size for backpressure handling
    pub buffer_size: Option<usize>,
    /// Whether to include metadata in chunks
    pub include_metadata: bool,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_chunk_size: Some(1024),
            min_chunk_delay_ms: Some(10),
            max_chunk_delay_ms: Some(100),
            buffer_size: Some(1000),
            include_metadata: true,
        }
    }
}

/// Type alias for a streaming response
pub type StreamResponse = Pin<Box<dyn Stream<Item = Result<StreamChunk, WorkflowError>> + Send>>;

/// Trait for streaming providers
pub trait StreamingProvider: Send + Sync {
    /// Create a streaming response from a prompt
    fn stream_response(&self, prompt: &str, config: &StreamConfig) -> StreamResponse;
    
    /// Get the provider name
    fn provider_name(&self) -> &str;
    
    /// Check if streaming is supported for this provider
    fn supports_streaming(&self) -> bool;
}

/// Error types specific to streaming
#[derive(Debug, thiserror::Error)]
pub enum StreamingError {
    #[error("Streaming not supported for provider: {provider}")]
    NotSupported { provider: String },
    
    #[error("Stream parsing error: {message}")]
    ParseError { message: String },
    
    #[error("Stream connection error: {message}")]
    ConnectionError { message: String },
    
    #[error("Stream buffer overflow: {message}")]
    BufferOverflow { message: String },
    
    #[error("Stream timeout: {message}")]
    Timeout { message: String },
}

/// Streaming events for WebSocket connections
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamEvent {
    /// A new chunk of data
    Chunk {
        data: StreamChunk,
    },
    /// Stream has started
    Started {
        stream_id: String,
        metadata: StreamMetadata,
    },
    /// Stream has ended successfully
    Completed {
        stream_id: String,
        total_chunks: u32,
        total_tokens: Option<u32>,
        duration_ms: u64,
    },
    /// Stream encountered an error
    Error {
        stream_id: String,
        error: String,
    },
    /// Heartbeat to keep connection alive
    Heartbeat {
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// WebSocket message wrapper for streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMessage {
    /// Unique identifier for this stream
    pub stream_id: String,
    /// The streaming event
    pub event: StreamEvent,
    /// Sequence number for ordering
    pub sequence: u64,
}

impl StreamChunk {
    /// Create a new streaming chunk
    pub fn new(content: String, is_final: bool) -> Self {
        Self {
            content,
            is_final,
            metadata: None,
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// Create a new streaming chunk with metadata
    pub fn with_metadata(content: String, is_final: bool, metadata: StreamMetadata) -> Self {
        Self {
            content,
            is_final,
            metadata: Some(metadata),
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// Get the content length
    pub fn content_length(&self) -> usize {
        self.content.len()
    }
    
    /// Check if this chunk is empty
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }
}

impl StreamMetadata {
    /// Create new metadata
    pub fn new(model: String, provider: String) -> Self {
        Self {
            model,
            provider,
            token_count: None,
            total_tokens: None,
            processing_time_ms: None,
        }
    }
    
    /// Set token count
    pub fn with_token_count(mut self, token_count: u32) -> Self {
        self.token_count = Some(token_count);
        self
    }
    
    /// Set total tokens
    pub fn with_total_tokens(mut self, total_tokens: u32) -> Self {
        self.total_tokens = Some(total_tokens);
        self
    }
    
    /// Set processing time
    pub fn with_processing_time(mut self, processing_time_ms: u64) -> Self {
        self.processing_time_ms = Some(processing_time_ms);
        self
    }
}