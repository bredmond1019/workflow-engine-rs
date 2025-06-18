use backend::core::streaming::{
    types::*,
    providers::*,
    sse::*,
    backpressure::*,
};
use backend::core::error::WorkflowError;
use futures_util::StreamExt;
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_stream_chunk_creation() {
    let chunk = StreamChunk::new("Hello world".to_string(), false);
    
    assert_eq!(chunk.content, "Hello world");
    assert!(!chunk.is_final);
    assert!(chunk.metadata.is_none());
    assert_eq!(chunk.content_length(), 11);
    assert!(!chunk.is_empty());
}

#[tokio::test]
async fn test_stream_chunk_with_metadata() {
    let metadata = StreamMetadata::new("gpt-4".to_string(), "openai".to_string())
        .with_token_count(5)
        .with_total_tokens(100)
        .with_processing_time(250);
    
    let chunk = StreamChunk::with_metadata("Hello".to_string(), true, metadata);
    
    assert_eq!(chunk.content, "Hello");
    assert!(chunk.is_final);
    assert!(chunk.metadata.is_some());
    
    let metadata = chunk.metadata.unwrap();
    assert_eq!(metadata.model, "gpt-4");
    assert_eq!(metadata.provider, "openai");
    assert_eq!(metadata.token_count, Some(5));
    assert_eq!(metadata.total_tokens, Some(100));
    assert_eq!(metadata.processing_time_ms, Some(250));
}

#[tokio::test]
async fn test_stream_config_default() {
    let config = StreamConfig::default();
    
    assert!(config.enabled);
    assert_eq!(config.max_chunk_size, Some(1024));
    assert_eq!(config.min_chunk_delay_ms, Some(10));
    assert_eq!(config.max_chunk_delay_ms, Some(100));
    assert_eq!(config.buffer_size, Some(1000));
    assert!(config.include_metadata);
}

#[tokio::test]
async fn test_streaming_provider_creation() {
    let client = Arc::new(reqwest::Client::new());
    
    // Test OpenAI provider creation
    let openai_provider = create_streaming_provider(
        "openai",
        "gpt-4".to_string(),
        "You are helpful".to_string(),
        Some(client.clone()),
    );
    assert!(openai_provider.is_ok());
    let provider = openai_provider.unwrap();
    assert_eq!(provider.provider_name(), "openai");
    assert!(provider.supports_streaming());
    
    // Test Anthropic provider creation
    let anthropic_provider = create_streaming_provider(
        "anthropic",
        "claude-3-sonnet-20240229".to_string(),
        "You are helpful".to_string(),
        Some(client.clone()),
    );
    assert!(anthropic_provider.is_ok());
    let provider = anthropic_provider.unwrap();
    assert_eq!(provider.provider_name(), "anthropic");
    assert!(provider.supports_streaming());
    
    // Test Bedrock provider creation
    let bedrock_provider = create_streaming_provider(
        "bedrock",
        "anthropic.claude-3-sonnet-20240229-v1:0".to_string(),
        "You are helpful".to_string(),
        None,
    );
    assert!(bedrock_provider.is_ok());
    let provider = bedrock_provider.unwrap();
    assert_eq!(provider.provider_name(), "bedrock");
    assert!(provider.supports_streaming());
    
    // Test unsupported provider
    let unsupported = create_streaming_provider(
        "unsupported",
        "model".to_string(),
        "prompt".to_string(),
        None,
    );
    assert!(unsupported.is_err());
}

#[tokio::test]
async fn test_backpressure_handler() {
    let config = StreamConfig {
        enabled: true,
        max_chunk_size: Some(20), // 20 characters
        min_chunk_delay_ms: Some(100),
        max_chunk_delay_ms: Some(1000),
        buffer_size: Some(5),
        include_metadata: true,
    };
    
    let mut handler = BackpressureHandler::new(config);
    
    // Add small chunks
    let chunk1 = StreamChunk::new("Hello".to_string(), false);
    let chunk2 = StreamChunk::new(" world".to_string(), false);
    
    assert!(handler.add_chunk(chunk1).is_ok());
    assert!(handler.add_chunk(chunk2).is_ok());
    
    // Should have buffered content
    let stats = handler.get_stats();
    assert_eq!(stats.buffered_chunks, 2);
    assert_eq!(stats.buffer_size_bytes, 11); // "Hello" + " world"
    
    // Should send due to chunk size limit being reached
    assert!(handler.should_send_chunk());
    
    let sent_chunk = handler.get_next_chunk();
    assert!(sent_chunk.is_some());
    assert_eq!(sent_chunk.unwrap().content, "Hello");
    
    // Stats should be updated
    let stats = handler.get_stats();
    assert_eq!(stats.buffered_chunks, 1);
    assert_eq!(stats.total_chunks_sent, 1);
}

#[tokio::test]
async fn test_rate_limited_stream() {
    use futures_util::stream;
    
    let chunks = vec![
        Ok(StreamChunk::new("chunk1".to_string(), false)),
        Ok(StreamChunk::new("chunk2".to_string(), false)),
        Ok(StreamChunk::new("chunk3".to_string(), true)),
    ];
    
    let stream = stream::iter(chunks);
    let config = StreamConfig {
        min_chunk_delay_ms: Some(50), // 50ms delay
        ..Default::default()
    };
    
    let start = std::time::Instant::now();
    let mut rate_limited = create_rate_limited_stream(stream, config);
    
    let mut count = 0;
    while let Some(result) = rate_limited.next().await {
        assert!(result.is_ok());
        count += 1;
    }
    
    let elapsed = start.elapsed();
    assert_eq!(count, 3);
    // Should take at least 150ms due to rate limiting (3 chunks * 50ms)
    assert!(elapsed >= Duration::from_millis(150));
}

#[tokio::test]
async fn test_sse_parser_openai() {
    use bytes::Bytes;
    use futures_util::stream;
    
    let parser = SSEParser::new("openai".to_string(), "gpt-4".to_string());
    
    let test_data = r#"data: {"choices": [{"delta": {"content": "Hello"}}]}

data: {"choices": [{"delta": {"content": " world"}}]}

data: [DONE]

"#;
    
    let byte_stream = stream::once(async move {
        Ok(Bytes::from(test_data))
    });
    
    let mut chunk_stream = parser.parse_sse_stream(Box::pin(byte_stream));
    
    let mut chunks = Vec::new();
    while let Some(chunk_result) = chunk_stream.next().await {
        chunks.push(chunk_result.unwrap());
    }
    
    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0].content, "Hello");
    assert!(!chunks[0].is_final);
    assert_eq!(chunks[1].content, " world");
    assert!(!chunks[1].is_final);
    assert_eq!(chunks[2].content, ""); // Final chunk
    assert!(chunks[2].is_final);
}

#[tokio::test]
async fn test_sse_parser_anthropic() {
    use bytes::Bytes;
    use futures_util::stream;
    
    let parser = SSEParser::new("anthropic".to_string(), "claude-3".to_string());
    
    let test_data = r#"data: {"type": "content_block_start", "index": 0, "content_block": {"type": "text", "text": ""}}

data: {"type": "content_block_delta", "index": 0, "delta": {"type": "text_delta", "text": "Hello"}}

data: {"type": "content_block_delta", "index": 0, "delta": {"type": "text_delta", "text": " world"}}

data: {"type": "content_block_stop", "index": 0}

data: [DONE]

"#;
    
    let byte_stream = stream::once(async move {
        Ok(Bytes::from(test_data))
    });
    
    let mut chunk_stream = parser.parse_sse_stream(Box::pin(byte_stream));
    
    let mut chunks = Vec::new();
    while let Some(chunk_result) = chunk_stream.next().await {
        chunks.push(chunk_result.unwrap());
    }
    
    // Should get chunks for the content_block_delta events plus final chunk
    assert!(!chunks.is_empty());
    
    // Find the content chunks
    let content_chunks: Vec<&StreamChunk> = chunks.iter()
        .filter(|chunk| !chunk.content.is_empty())
        .collect();
    
    assert!(content_chunks.len() >= 2);
    assert_eq!(content_chunks[0].content, "Hello");
    assert_eq!(content_chunks[1].content, " world");
    
    // Last chunk should be final
    assert!(chunks.last().unwrap().is_final);
}

#[tokio::test]
async fn test_stream_events_serialization() {
    let chunk = StreamChunk::new("Hello".to_string(), false);
    let event = StreamEvent::Chunk { data: chunk };
    let msg = StreamMessage {
        stream_id: "test-123".to_string(),
        event,
        sequence: 1,
    };
    
    let json = serde_json::to_string(&msg).unwrap();
    let parsed: StreamMessage = serde_json::from_str(&json).unwrap();
    
    assert_eq!(parsed.stream_id, "test-123");
    assert_eq!(parsed.sequence, 1);
    
    match parsed.event {
        StreamEvent::Chunk { data } => {
            assert_eq!(data.content, "Hello");
            assert!(!data.is_final);
        }
        _ => panic!("Wrong event type"),
    }
}

#[tokio::test]
async fn test_buffer_overflow_protection() {
    let config = StreamConfig {
        buffer_size: Some(2), // Very small buffer
        ..Default::default()
    };
    
    let mut handler = BackpressureHandler::new(config);
    
    // Add chunks until buffer is full
    assert!(handler.add_chunk(StreamChunk::new("chunk1".to_string(), false)).is_ok());
    assert!(handler.add_chunk(StreamChunk::new("chunk2".to_string(), false)).is_ok());
    
    // Adding one more should fail
    let result = handler.add_chunk(StreamChunk::new("chunk3".to_string(), false));
    assert!(result.is_err());
    
    match result.unwrap_err() {
        backend::core::streaming::types::StreamingError::BufferOverflow { .. } => {
            // Expected error type
        }
        _ => panic!("Expected BufferOverflow error"),
    }
}

// Integration tests (marked as ignored - require actual API keys)
#[tokio::test]
#[ignore]
async fn test_openai_streaming_integration() {
    use std::env;
    
    // Skip if no API key
    if env::var("OPENAI_API_KEY").is_err() {
        return;
    }
    
    let client = Arc::new(reqwest::Client::new());
    let provider = OpenAIStreamingProvider::new(
        client,
        "gpt-3.5-turbo".to_string(),
        "You are a helpful assistant.".to_string(),
    );
    
    let config = StreamConfig::default();
    let mut stream = provider.stream_response("Say hello in 3 words", &config);
    
    let mut chunks = Vec::new();
    let mut total_content = String::new();
    
    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                total_content.push_str(&chunk.content);
                chunks.push(chunk.clone());
                
                if chunk.is_final {
                    break;
                }
            }
            Err(e) => {
                println!("Stream error: {}", e);
                break;
            }
        }
    }
    
    assert!(!chunks.is_empty());
    assert!(!total_content.is_empty());
    println!("Received {} chunks with total content: '{}'", chunks.len(), total_content);
}

#[tokio::test]
#[ignore]
async fn test_anthropic_streaming_integration() {
    use std::env;
    
    // Skip if no API key
    if env::var("ANTHROPIC_API_KEY").is_err() {
        return;
    }
    
    let client = Arc::new(reqwest::Client::new());
    let provider = AnthropicStreamingProvider::new(
        client,
        "claude-3-haiku-20240307".to_string(),
        "You are a helpful assistant.".to_string(),
    );
    
    let config = StreamConfig::default();
    let mut stream = provider.stream_response("Count from 1 to 5", &config);
    
    let mut chunks = Vec::new();
    let mut total_content = String::new();
    
    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                total_content.push_str(&chunk.content);
                chunks.push(chunk.clone());
                
                if chunk.is_final {
                    break;
                }
            }
            Err(e) => {
                println!("Stream error: {}", e);
                break;
            }
        }
    }
    
    assert!(!chunks.is_empty());
    assert!(!total_content.is_empty());
    println!("Received {} chunks with total content: '{}'", chunks.len(), total_content);
}

#[test]
fn test_streaming_error_types() {
    use backend::core::streaming::types::StreamingError;
    
    let not_supported = StreamingError::NotSupported {
        provider: "test".to_string(),
    };
    assert!(not_supported.to_string().contains("Streaming not supported"));
    
    let parse_error = StreamingError::ParseError {
        message: "Invalid JSON".to_string(),
    };
    assert!(parse_error.to_string().contains("Stream parsing error"));
    
    let connection_error = StreamingError::ConnectionError {
        message: "Network error".to_string(),
    };
    assert!(connection_error.to_string().contains("Stream connection error"));
    
    let buffer_overflow = StreamingError::BufferOverflow {
        message: "Too many chunks".to_string(),
    };
    assert!(buffer_overflow.to_string().contains("Stream buffer overflow"));
    
    let timeout = StreamingError::Timeout {
        message: "Request timeout".to_string(),
    };
    assert!(timeout.to_string().contains("Stream timeout"));
}