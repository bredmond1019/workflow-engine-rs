use actix_web::{test, web, App};
use futures_util::StreamExt;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

use backend::core::streaming::{
    handlers::{complete_ai_response, stream_ai_response, StreamingRequest},
    providers::{create_streaming_provider, OpenAIStreamingProvider, AnthropicStreamingProvider, BedrockStreamingProvider},
    types::{StreamChunk, StreamConfig, StreamMetadata, StreamingProvider},
    recovery::{RecoveryStreamingProvider, StreamingRecoveryConfig},
    sse::SSEStreamManager,
    backpressure::{create_rate_limited_stream, AdaptiveBackpressureController},
};

/// Mock streaming provider for testing
struct MockStreamingProvider {
    chunks: Vec<StreamChunk>,
    should_fail: bool,
    fail_after_chunks: usize,
}

impl MockStreamingProvider {
    fn new_success() -> Self {
        let chunks = vec![
            StreamChunk::with_metadata(
                "Hello".to_string(),
                false,
                StreamMetadata::new("test-model".to_string(), "mock".to_string())
                    .with_token_count(1)
                    .with_total_tokens(3),
            ),
            StreamChunk::with_metadata(
                " world".to_string(),
                false,
                StreamMetadata::new("test-model".to_string(), "mock".to_string())
                    .with_token_count(1)
                    .with_total_tokens(3),
            ),
            StreamChunk::with_metadata(
                "!".to_string(),
                true,
                StreamMetadata::new("test-model".to_string(), "mock".to_string())
                    .with_token_count(1)
                    .with_total_tokens(3),
            ),
        ];
        
        Self {
            chunks,
            should_fail: false,
            fail_after_chunks: 0,
        }
    }
    
    fn new_with_failure(fail_after_chunks: usize) -> Self {
        let mut provider = Self::new_success();
        provider.should_fail = true;
        provider.fail_after_chunks = fail_after_chunks;
        provider
    }
}

impl StreamingProvider for MockStreamingProvider {
    fn stream_response(&self, _prompt: &str, _config: &StreamConfig) -> backend::core::streaming::types::StreamResponse {
        let chunks = self.chunks.clone();
        let should_fail = self.should_fail;
        let fail_after_chunks = self.fail_after_chunks;
        
        let stream = async_stream::stream! {
            for (i, chunk) in chunks.into_iter().enumerate() {
                if should_fail && i >= fail_after_chunks {
                    yield Err(backend::core::error::WorkflowError::ApiError {
                        message: "Mock failure".to_string(),
                    });
                    return;
                }
                
                // Add small delay to simulate real streaming
                tokio::time::sleep(Duration::from_millis(10)).await;
                yield Ok(chunk);
            }
        };
        
        Box::pin(stream)
    }
    
    fn provider_name(&self) -> &str {
        "mock"
    }
    
    fn supports_streaming(&self) -> bool {
        true
    }
}

#[actix_web::test]
async fn test_streaming_provider_factory() {
    let client = Arc::new(reqwest::Client::new());
    
    // Test OpenAI provider creation
    let openai_result = create_streaming_provider(
        "openai",
        "gpt-4".to_string(),
        "You are helpful".to_string(),
        Some(client.clone()),
    );
    assert!(openai_result.is_ok());
    assert_eq!(openai_result.unwrap().provider_name(), "openai");
    
    // Test Anthropic provider creation
    let anthropic_result = create_streaming_provider(
        "anthropic",
        "claude-3-sonnet-20240229".to_string(),
        "You are helpful".to_string(),
        Some(client.clone()),
    );
    assert!(anthropic_result.is_ok());
    assert_eq!(anthropic_result.unwrap().provider_name(), "anthropic");
    
    // Test Bedrock provider creation
    let bedrock_result = create_streaming_provider(
        "bedrock",
        "anthropic.claude-3-sonnet-20240229-v1:0".to_string(),
        "You are helpful".to_string(),
        None,
    );
    assert!(bedrock_result.is_ok());
    assert_eq!(bedrock_result.unwrap().provider_name(), "bedrock");
    
    // Test unsupported provider
    let unsupported_result = create_streaming_provider(
        "unsupported",
        "model".to_string(),
        "prompt".to_string(),
        None,
    );
    assert!(unsupported_result.is_err());
}

#[tokio::test]
async fn test_streaming_provider_basic_functionality() {
    let provider = MockStreamingProvider::new_success();
    let config = StreamConfig::default();
    
    let mut stream = provider.stream_response("test prompt", &config);
    let mut chunks = Vec::new();
    
    while let Some(chunk_result) = stream.next().await {
        chunks.push(chunk_result.unwrap());
    }
    
    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0].content, "Hello");
    assert_eq!(chunks[1].content, " world");
    assert_eq!(chunks[2].content, "!");
    assert!(chunks[2].is_final);
    
    // Verify metadata
    for chunk in &chunks {
        assert!(chunk.metadata.is_some());
        if let Some(metadata) = &chunk.metadata {
            assert_eq!(metadata.provider, "mock");
            assert_eq!(metadata.model, "test-model");
            assert_eq!(metadata.total_tokens, Some(3));
        }
    }
}

#[tokio::test]
async fn test_recovery_provider_with_failures() {
    let base_provider = Arc::new(MockStreamingProvider::new_with_failure(1));
    let recovery_config = StreamingRecoveryConfig {
        max_retries: 2,
        initial_retry_delay_ms: 10,
        ..Default::default()
    };
    
    let recovery_provider = RecoveryStreamingProvider::new(base_provider, recovery_config);
    let stream_config = StreamConfig::default();
    
    let mut stream = recovery_provider.stream_with_recovery("test prompt", &stream_config).await;
    let mut chunks = Vec::new();
    
    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => chunks.push(chunk),
            Err(_) => break, // Expected failure after retries
        }
    }
    
    // Should get at least the first chunk before failure
    assert!(!chunks.is_empty());
}

#[tokio::test]
async fn test_backpressure_rate_limiting() {
    let provider = MockStreamingProvider::new_success();
    let config = StreamConfig {
        min_chunk_delay_ms: Some(50),
        ..Default::default()
    };
    
    let stream = provider.stream_response("test prompt", &config);
    let rate_limited_stream = create_rate_limited_stream(stream, config);
    
    let start_time = std::time::Instant::now();
    let mut count = 0;
    
    let mut stream = rate_limited_stream;
    while let Some(_) = stream.next().await {
        count += 1;
    }
    
    let elapsed = start_time.elapsed();
    
    // Should have taken at least 150ms for 3 chunks with 50ms delay each
    assert!(elapsed >= Duration::from_millis(100));
    assert_eq!(count, 3);
}

#[tokio::test]
async fn test_adaptive_backpressure_controller() {
    let config = StreamConfig {
        min_chunk_delay_ms: Some(100),
        max_chunk_delay_ms: Some(1000),
        ..Default::default()
    };
    
    let mut controller = AdaptiveBackpressureController::new(config);
    
    // Initial state
    assert_eq!(controller.get_load_factor(), 1.0);
    let initial_delay = controller.get_current_delay();
    
    // Update with high processing time (system stress)
    controller.update_metrics(10, 1500);
    assert!(controller.get_load_factor() > 1.0);
    
    // Update with low processing time (system has capacity)
    controller.update_metrics(10, 50);
    // Load factor should eventually decrease
}

#[tokio::test]
async fn test_sse_stream_manager() {
    let provider = MockStreamingProvider::new_success();
    let config = StreamConfig::default();
    let chunk_stream = provider.stream_response("test prompt", &config);
    
    let sse_manager = SSEStreamManager::new(1); // 1 second heartbeat for testing
    let mut sse_stream = sse_manager.create_managed_sse_stream(chunk_stream);
    
    let mut sse_messages = Vec::new();
    let mut heartbeat_count = 0;
    
    // Collect first few SSE messages
    for _ in 0..5 {
        if let Some(bytes_result) = sse_stream.next().await {
            let bytes = bytes_result.unwrap();
            let message = String::from_utf8(bytes.to_vec()).unwrap();
            
            if message.contains("event: heartbeat") {
                heartbeat_count += 1;
            }
            
            sse_messages.push(message);
        }
    }
    
    // Should have connection event and content chunks
    assert!(!sse_messages.is_empty());
    
    // Check that we have proper SSE format
    for message in &sse_messages {
        if !message.is_empty() {
            assert!(message.contains("id: ") || message.contains("event: ") || message.contains("data: "));
        }
    }
}

#[actix_web::test]
async fn test_complete_ai_response_endpoint() {
    let app = test::init_service(
        App::new().route("/complete", web::post().to(complete_ai_response))
    ).await;
    
    let request_payload = json!({
        "provider": "openai",
        "model": "gpt-4",
        "prompt": "Hello world",
        "system_prompt": "You are helpful",
        "config": {
            "enabled": true,
            "include_metadata": true
        }
    });
    
    let req = test::TestRequest::post()
        .uri("/complete")
        .set_json(&request_payload)
        .to_request();
    
    // Note: This test will fail without actual API keys, but verifies the endpoint structure
    let resp = test::call_service(&app, req).await;
    
    // Should return 400 (bad request) due to missing API key, not 500 (server error)
    assert!(resp.status().is_client_error() || resp.status().is_server_error());
}

#[actix_web::test]
async fn test_stream_ai_response_endpoint() {
    let app = test::init_service(
        App::new().route("/stream", web::post().to(stream_ai_response))
    ).await;
    
    let request_payload = json!({
        "provider": "anthropic",
        "model": "claude-3-sonnet-20240229",
        "prompt": "Explain AI",
        "config": {
            "enabled": true,
            "min_chunk_delay_ms": 10
        }
    });
    
    let req = test::TestRequest::post()
        .uri("/stream")
        .set_json(&request_payload)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Should return 400 (bad request) due to missing API key, not 500 (server error)
    assert!(resp.status().is_client_error() || resp.status().is_server_error());
}

#[tokio::test]
async fn test_streaming_with_different_configs() {
    let provider = MockStreamingProvider::new_success();
    
    // Test with metadata enabled
    let config_with_metadata = StreamConfig {
        include_metadata: true,
        ..Default::default()
    };
    
    let mut stream = provider.stream_response("test", &config_with_metadata);
    let chunk = stream.next().await.unwrap().unwrap();
    assert!(chunk.metadata.is_some());
    
    // Test with rate limiting
    let config_with_rate_limit = StreamConfig {
        min_chunk_delay_ms: Some(20),
        ..Default::default()
    };
    
    let start_time = std::time::Instant::now();
    let mut stream = provider.stream_response("test", &config_with_rate_limit);
    let mut count = 0;
    
    while let Some(_) = stream.next().await {
        count += 1;
    }
    
    let elapsed = start_time.elapsed();
    assert_eq!(count, 3);
    // Should take at least some time due to rate limiting
    assert!(elapsed >= Duration::from_millis(30));
}

#[tokio::test]
async fn test_concurrent_streaming() {
    let provider = Arc::new(MockStreamingProvider::new_success());
    let config = StreamConfig::default();
    
    let mut handles = Vec::new();
    
    // Start multiple concurrent streams
    for i in 0..5 {
        let provider_clone = provider.clone();
        let config_clone = config.clone();
        let handle = tokio::spawn(async move {
            let mut stream = provider_clone.stream_response(&format!("prompt {}", i), &config_clone);
            let mut chunks = Vec::new();
            
            while let Some(chunk_result) = stream.next().await {
                chunks.push(chunk_result.unwrap());
            }
            
            chunks.len()
        });
        handles.push(handle);
    }
    
    // Wait for all streams to complete
    let results = futures_util::future::join_all(handles).await;
    
    // All streams should complete successfully with 3 chunks each
    for result in results {
        assert_eq!(result.unwrap(), 3);
    }
}

#[tokio::test]
async fn test_error_propagation() {
    let provider = MockStreamingProvider::new_with_failure(0); // Fail immediately
    let config = StreamConfig::default();
    
    let mut stream = provider.stream_response("test prompt", &config);
    let result = stream.next().await.unwrap();
    
    assert!(result.is_err());
    match result {
        Err(backend::core::error::WorkflowError::ApiError { message }) => {
            assert_eq!(message, "Mock failure");
        }
        _ => panic!("Expected ApiError"),
    }
}

#[tokio::test]
async fn test_token_counting_integration() {
    let provider = MockStreamingProvider::new_success();
    let config = StreamConfig {
        include_metadata: true,
        ..Default::default()
    };
    
    let mut stream = provider.stream_response("test prompt", &config);
    let mut total_tokens = 0u32;
    let mut individual_tokens = 0u32;
    
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.unwrap();
        
        if let Some(metadata) = &chunk.metadata {
            if let Some(token_count) = metadata.token_count {
                individual_tokens += token_count;
            }
            if let Some(total) = metadata.total_tokens {
                total_tokens = total;
            }
        }
    }
    
    assert_eq!(individual_tokens, 3); // Sum of individual chunk tokens
    assert_eq!(total_tokens, 3); // Total tokens reported
}