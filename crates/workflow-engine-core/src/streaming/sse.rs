use bytes::Bytes;
use futures_util::{Stream, StreamExt, TryStreamExt};
use serde_json::Value;
use std::pin::Pin;
use std::time::Duration;

use crate::error::WorkflowError;
use super::types::{StreamChunk, StreamMetadata, StreamingError};

/// SSE (Server-Sent Events) parser for streaming AI responses
pub struct SSEParser {
    provider: String,
    model: String,
}

impl SSEParser {
    pub fn new(provider: String, model: String) -> Self {
        Self { provider, model }
    }

    /// Parse SSE stream from bytes into StreamChunks
    pub fn parse_sse_stream(
        &self,
        byte_stream: Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>,
    ) -> Pin<Box<dyn Stream<Item = Result<StreamChunk, WorkflowError>> + Send>> {
        let provider = self.provider.clone();
        let model = self.model.clone();
        
        let stream = byte_stream
            .map_err(|e| WorkflowError::ApiError {
                message: format!("Stream error: {}", e),
            })
            .flat_map(|bytes_result| {
                match bytes_result {
                    Ok(bytes) => {
                        let string_data = String::from_utf8_lossy(&bytes);
                        futures_util::stream::iter(
                            string_data
                                .lines()
                                .filter_map(|line| {
                                    if line.starts_with("data: ") {
                                        Some(Ok(line[6..].to_string()))
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<_>>()
                        )
                    }
                    Err(e) => futures_util::stream::iter(vec![Err(e)]),
                }
            })
            .filter_map(move |line_result| {
                let provider_clone = provider.clone();
                let model_clone = model.clone();
                
                async move {
                    match line_result {
                        Ok(line) => {
                            if line == "[DONE]" {
                                Some(Ok(StreamChunk::with_metadata(
                                    String::new(),
                                    true,
                                    StreamMetadata::new(model_clone, provider_clone),
                                )))
                            } else {
                                match serde_json::from_str::<Value>(&line) {
                                    Ok(json) => {
                                        match SSEParser::extract_content_from_json(&json, &provider_clone) {
                                            Ok(Some(content)) => {
                                                Some(Ok(StreamChunk::with_metadata(
                                                    content,
                                                    false,
                                                    StreamMetadata::new(model_clone, provider_clone),
                                                )))
                                            }
                                            Ok(None) => None, // No content in this chunk
                                            Err(e) => Some(Err(e)),
                                        }
                                    }
                                    Err(e) => Some(Err(WorkflowError::DeserializationError {
                                        message: format!("Failed to parse SSE JSON: {}", e),
                                    })),
                                }
                            }
                        }
                        Err(e) => Some(Err(e)),
                    }
                }
            });

        Box::pin(stream)
    }

    /// Extract content from JSON based on provider format
    fn extract_content_from_json(
        json: &Value,
        provider: &str,
    ) -> Result<Option<String>, WorkflowError> {
        match provider.to_lowercase().as_str() {
            "openai" | "azureopenai" => {
                // OpenAI format: {"choices": [{"delta": {"content": "text"}}]}
                if let Some(choices) = json.get("choices").and_then(|c| c.as_array()) {
                    if let Some(choice) = choices.first() {
                        if let Some(delta) = choice.get("delta") {
                            if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                                return Ok(Some(content.to_string()));
                            }
                        }
                    }
                }
                Ok(None)
            }
            "anthropic" => {
                // Anthropic format: {"type": "content_block_delta", "delta": {"text": "text"}}
                if json.get("type").and_then(|t| t.as_str()) == Some("content_block_delta") {
                    if let Some(delta) = json.get("delta") {
                        if let Some(text) = delta.get("text").and_then(|t| t.as_str()) {
                            return Ok(Some(text.to_string()));
                        }
                    }
                }
                Ok(None)
            }
            _ => Err(WorkflowError::ConfigurationError(
                format!("Unsupported provider for SSE parsing: {}", provider),
            )),
        }
    }
}

/// Create an SSE response stream from a string stream
pub fn create_sse_response_stream(
    content_stream: Pin<Box<dyn Stream<Item = Result<String, WorkflowError>> + Send>>,
) -> Pin<Box<dyn Stream<Item = Result<Bytes, WorkflowError>> + Send>> {
    let stream = content_stream.map(|result| {
        match result {
            Ok(content) => {
                // Escape content for SSE format
                let escaped_content = content.replace("\n", "\\n").replace("\r", "\\r");
                let sse_data = format!("data: {}\n\n", escaped_content);
                Ok(Bytes::from(sse_data))
            }
            Err(e) => {
                let error_data = format!("event: error\ndata: {{\"error\": \"{}\"}}\n\n", e);
                Ok(Bytes::from(error_data))
            }
        }
    });
    
    Box::pin(stream)
}

/// Create an SSE stream for StreamChunk objects
pub fn create_sse_chunk_stream(
    chunk_stream: Pin<Box<dyn Stream<Item = Result<StreamChunk, WorkflowError>> + Send>>,
) -> Pin<Box<dyn Stream<Item = Result<Bytes, WorkflowError>> + Send>> {
    let stream = chunk_stream.enumerate().map(|(index, result)| {
        match result {
            Ok(chunk) => {
                match serde_json::to_string(&chunk) {
                    Ok(json) => {
                        let event_name = if chunk.is_final { "complete" } else { "chunk" };
                        let sse_data = format!(
                            "id: {}\nevent: {}\ndata: {}\n\n{}",
                            index,
                            event_name,
                            json,
                            if chunk.is_final { "event: done\ndata: [DONE]\n\n" } else { "" }
                        );
                        Ok(Bytes::from(sse_data))
                    }
                    Err(e) => {
                        let error_data = format!(
                            "id: {}\nevent: error\ndata: {{\"error\": \"Serialization error: {}\"}}\n\n",
                            index, e
                        );
                        Ok(Bytes::from(error_data))
                    }
                }
            }
            Err(e) => {
                let error_data = format!(
                    "id: {}\nevent: error\ndata: {{\"error\": \"{}\"}}\n\n",
                    index, e
                );
                Ok(Bytes::from(error_data))
            }
        }
    });
    
    Box::pin(stream)
}

/// Enhanced SSE streaming with heartbeat and reconnection support
pub struct SSEStreamManager {
    heartbeat_interval: Duration,
    last_heartbeat: std::time::Instant,
    connection_id: String,
}

impl SSEStreamManager {
    pub fn new(heartbeat_interval_secs: u64) -> Self {
        Self {
            heartbeat_interval: Duration::from_secs(heartbeat_interval_secs),
            last_heartbeat: std::time::Instant::now(),
            connection_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// Create an SSE stream with heartbeat and connection management
    pub fn create_managed_sse_stream<S>(
        &self,
        chunk_stream: S,
    ) -> Pin<Box<dyn Stream<Item = Result<Bytes, WorkflowError>> + Send>>
    where
        S: Stream<Item = Result<StreamChunk, WorkflowError>> + Send + 'static,
    {
        let heartbeat_interval = self.heartbeat_interval;
        let connection_id = self.connection_id.clone();
        
        let stream = async_stream::stream! {
            let mut chunk_stream = Box::pin(chunk_stream);
            let mut last_heartbeat = std::time::Instant::now();
            let mut message_id = 0u64;
            
            // Send initial connection event
            let connection_event = format!(
                "id: {}\nevent: connected\ndata: {{\"connection_id\": \"{}\", \"timestamp\": \"{}\"}}\n\n",
                message_id,
                connection_id,
                chrono::Utc::now().to_rfc3339()
            );
            yield Ok(Bytes::from(connection_event));
            message_id += 1;
            
            loop {
                // Check if we need to send a heartbeat
                if last_heartbeat.elapsed() >= heartbeat_interval {
                    let heartbeat_event = format!(
                        "id: {}\nevent: heartbeat\ndata: {{\"timestamp\": \"{}\", \"connection_id\": \"{}\"}}\n\n",
                        message_id,
                        chrono::Utc::now().to_rfc3339(),
                        connection_id
                    );
                    yield Ok(Bytes::from(heartbeat_event));
                    last_heartbeat = std::time::Instant::now();
                    message_id += 1;
                }
                
                // Use a timeout to regularly check for heartbeat
                let timeout_duration = Duration::from_millis(1000);
                match tokio::time::timeout(timeout_duration, chunk_stream.next()).await {
                    Ok(Some(chunk_result)) => {
                        match chunk_result {
                            Ok(chunk) => {
                                match serde_json::to_string(&chunk) {
                                    Ok(json) => {
                                        let event_name = if chunk.is_final { "complete" } else { "chunk" };
                                        let sse_data = format!(
                                            "id: {}\nevent: {}\ndata: {}\n\n",
                                            message_id,
                                            event_name,
                                            json
                                        );
                                        yield Ok(Bytes::from(sse_data));
                                        message_id += 1;
                                        
                                        if chunk.is_final {
                                            let done_event = format!(
                                                "id: {}\nevent: done\ndata: [DONE]\n\n",
                                                message_id
                                            );
                                            yield Ok(Bytes::from(done_event));
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                        let error_data = format!(
                                            "id: {}\nevent: error\ndata: {{\"error\": \"Serialization error: {}\"}}\n\n",
                                            message_id, e
                                        );
                                        yield Ok(Bytes::from(error_data));
                                        message_id += 1;
                                    }
                                }
                            }
                            Err(e) => {
                                let error_data = format!(
                                    "id: {}\nevent: error\ndata: {{\"error\": \"{}\"}}\n\n",
                                    message_id, e
                                );
                                yield Ok(Bytes::from(error_data));
                                message_id += 1;
                                break;
                            }
                        }
                    }
                    Ok(None) => {
                        // Stream ended normally
                        let done_event = format!(
                            "id: {}\nevent: done\ndata: [DONE]\n\n",
                            message_id
                        );
                        yield Ok(Bytes::from(done_event));
                        break;
                    }
                    Err(_) => {
                        // Timeout occurred, continue to check heartbeat
                        continue;
                    }
                }
            }
        };
        
        Box::pin(stream)
    }

    /// Get connection ID
    pub fn connection_id(&self) -> &str {
        &self.connection_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;

    #[tokio::test]
    async fn test_openai_sse_parsing() {
        let parser = SSEParser::new("openai".to_string(), "gpt-4".to_string());
        
        let test_data = r#"data: {"choices": [{"delta": {"content": "Hello"}}]}
data: {"choices": [{"delta": {"content": " world"}}]}
data: [DONE]"#;
        
        let byte_stream = futures_util::stream::once(async move {
            Ok(Bytes::from(test_data))
        });
        
        let mut chunk_stream = parser.parse_sse_stream(Box::pin(byte_stream));
        
        let mut chunks = Vec::new();
        while let Some(chunk_result) = chunk_stream.next().await {
            chunks.push(chunk_result.unwrap());
        }
        
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].content, "Hello");
        assert_eq!(chunks[1].content, " world");
        assert!(chunks[2].is_final);
    }

    #[tokio::test]
    async fn test_anthropic_sse_parsing() {
        let parser = SSEParser::new("anthropic".to_string(), "claude-3".to_string());
        
        let test_data = r#"data: {"type": "content_block_delta", "delta": {"text": "Hello"}}
data: {"type": "content_block_delta", "delta": {"text": " world"}}
data: [DONE]"#;
        
        let byte_stream = futures_util::stream::once(async move {
            Ok(Bytes::from(test_data))
        });
        
        let mut chunk_stream = parser.parse_sse_stream(Box::pin(byte_stream));
        
        let mut chunks = Vec::new();
        while let Some(chunk_result) = chunk_stream.next().await {
            chunks.push(chunk_result.unwrap());
        }
        
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].content, "Hello");
        assert_eq!(chunks[1].content, " world");
        assert!(chunks[2].is_final);
    }
}