use actix_web::{web, HttpResponse, Result as ActixResult};
use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::sync::Arc;

use crate::core::error::WorkflowError;
use super::types::{StreamChunk, StreamConfig, StreamingProvider};
use super::providers::create_streaming_provider;
use super::sse::{create_sse_chunk_stream, SSEStreamManager};
use super::recovery::{RecoveryStreamingProvider, StreamingRecoveryConfig};

/// Request payload for streaming endpoints
#[derive(Debug, Deserialize)]
pub struct StreamingRequest {
    /// The AI provider to use
    pub provider: String,
    /// The model name
    pub model: String,
    /// The user prompt
    pub prompt: String,
    /// Optional system prompt
    pub system_prompt: Option<String>,
    /// Streaming configuration
    pub config: Option<StreamConfig>,
}

/// Response for non-streaming endpoints
#[derive(Debug, Serialize)]
pub struct StreamingResponse {
    /// The complete response
    pub response: String,
    /// Metadata about the response
    pub metadata: StreamingResponseMetadata,
}

/// Metadata about a streaming response
#[derive(Debug, Serialize)]
pub struct StreamingResponseMetadata {
    /// The provider used
    pub provider: String,
    /// The model used
    pub model: String,
    /// Total chunks received
    pub total_chunks: u32,
    /// Total processing time in milliseconds
    pub processing_time_ms: u64,
    /// Total tokens (if available)
    pub total_tokens: Option<u32>,
}

/// HTTP streaming endpoint (Server-Sent Events)
pub async fn stream_ai_response(
    req: web::Json<StreamingRequest>,
) -> ActixResult<HttpResponse> {
    let request = req.into_inner();
    
    // Create base streaming provider
    let client = Arc::new(reqwest::Client::new());
    let base_provider = match create_streaming_provider(
        &request.provider,
        request.model.clone(),
        request.system_prompt.unwrap_or_else(|| "You are a helpful assistant.".to_string()),
        Some(client),
    ) {
        Ok(provider) => provider,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Failed to create streaming provider: {}", e)
            })));
        }
    };

    let config = request.config.unwrap_or_default();
    
    // Wrap with recovery capabilities
    let recovery_config = StreamingRecoveryConfig::default();
    let recovery_provider = RecoveryStreamingProvider::new(Arc::from(base_provider), recovery_config);
    
    // Create the streaming response with recovery
    let chunk_stream = recovery_provider.stream_with_recovery(&request.prompt, &config).await;
    
    // Create SSE stream with heartbeat and connection management
    let sse_manager = SSEStreamManager::new(30); // 30 second heartbeat
    let sse_stream = sse_manager.create_managed_sse_stream(chunk_stream);
    
    Ok(HttpResponse::Ok()
        .content_type("text/event-stream")
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("Connection", "keep-alive"))
        .insert_header(("Access-Control-Allow-Origin", "*"))
        .insert_header(("Access-Control-Allow-Headers", "Content-Type"))
        .insert_header(("X-Accel-Buffering", "no")) // Disable nginx buffering
        .streaming(sse_stream.map(|result| {
            result.map_err(|e| actix_web::error::ErrorInternalServerError(e))
        })))
}

/// Non-streaming endpoint that collects all chunks
pub async fn complete_ai_response(
    req: web::Json<StreamingRequest>,
) -> ActixResult<HttpResponse> {
    let request = req.into_inner();
    
    // Create base streaming provider
    let client = Arc::new(reqwest::Client::new());
    let base_provider = match create_streaming_provider(
        &request.provider,
        request.model.clone(),
        request.system_prompt.unwrap_or_else(|| "You are a helpful assistant.".to_string()),
        Some(client),
    ) {
        Ok(provider) => provider,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Failed to create streaming provider: {}", e)
            })));
        }
    };

    let config = request.config.unwrap_or_default();
    let start_time = std::time::Instant::now();
    
    // Wrap with recovery capabilities
    let recovery_config = StreamingRecoveryConfig::default();
    let recovery_provider = RecoveryStreamingProvider::new(Arc::from(base_provider), recovery_config);
    
    // Collect all chunks with recovery
    let mut chunk_stream = recovery_provider.stream_with_recovery(&request.prompt, &config).await;
    let mut complete_response = String::new();
    let mut chunk_count = 0u32;
    let mut total_tokens = None;
    
    while let Some(chunk_result) = chunk_stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                complete_response.push_str(&chunk.content);
                chunk_count += 1;
                
                if let Some(metadata) = &chunk.metadata {
                    if let Some(tokens) = metadata.total_tokens {
                        total_tokens = Some(tokens);
                    }
                }
                
                if chunk.is_final {
                    break;
                }
            }
            Err(e) => {
                return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Streaming error: {}", e)
                })));
            }
        }
    }
    
    let processing_time = start_time.elapsed();
    
    let response = StreamingResponse {
        response: complete_response,
        metadata: StreamingResponseMetadata {
            provider: request.provider,
            model: request.model,
            total_chunks: chunk_count,
            processing_time_ms: processing_time.as_millis() as u64,
            total_tokens,
        },
    };
    
    Ok(HttpResponse::Ok().json(response))
}

/// Health check endpoint for streaming services
pub async fn streaming_health() -> ActixResult<HttpResponse> {
    let health_info = serde_json::json!({
        "status": "healthy",
        "service": "ai-streaming",
        "timestamp": chrono::Utc::now(),
        "supported_providers": ["openai", "anthropic", "bedrock"],
        "features": {
            "sse_streaming": true,
            "websocket_streaming": true,
            "backpressure_control": true,
            "rate_limiting": true
        }
    });
    
    Ok(HttpResponse::Ok().json(health_info))
}

/// Get streaming configuration
pub async fn get_streaming_config() -> ActixResult<HttpResponse> {
    let default_config = StreamConfig::default();
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "default_config": default_config,
        "limits": {
            "max_chunk_size": 2048,
            "max_buffer_size": 10000,
            "max_delay_ms": 5000,
            "min_delay_ms": 1
        },
        "providers": {
            "openai": {
                "supports_streaming": true,
                "models": ["gpt-4", "gpt-3.5-turbo", "gpt-4-turbo-preview"]
            },
            "anthropic": {
                "supports_streaming": true,
                "models": ["claude-3-opus-20240229", "claude-3-sonnet-20240229", "claude-3-haiku-20240307"]
            },
            "bedrock": {
                "supports_streaming": true,
                "note": "Simulated streaming by chunking complete responses",
                "models": ["anthropic.claude-3-sonnet-20240229-v1:0", "amazon.titan-text-express-v1"]
            }
        }
    })))
}

/// Stream status endpoint for monitoring active streams
pub async fn stream_status() -> ActixResult<HttpResponse> {
    // In a real implementation, you'd track active streams
    let status = serde_json::json!({
        "active_streams": 0,
        "total_streams_started": 0,
        "total_streams_completed": 0,
        "total_streams_errored": 0,
        "average_stream_duration_ms": 0,
        "uptime_seconds": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });
    
    Ok(HttpResponse::Ok().json(status))
}

/// Example usage endpoint
pub async fn streaming_examples() -> ActixResult<HttpResponse> {
    let examples = serde_json::json!({
        "sse_streaming": {
            "endpoint": "/api/stream",
            "method": "POST",
            "content_type": "application/json",
            "example_request": {
                "provider": "openai",
                "model": "gpt-4",
                "prompt": "Explain quantum computing in simple terms",
                "system_prompt": "You are a helpful science tutor",
                "config": {
                    "enabled": true,
                    "max_chunk_size": 100,
                    "min_chunk_delay_ms": 50,
                    "include_metadata": true
                }
            },
            "response_format": "text/event-stream",
            "example_chunk": {
                "content": "Quantum computing is a revolutionary approach...",
                "is_final": false,
                "metadata": {
                    "model": "gpt-4",
                    "provider": "openai",
                    "token_count": 15,
                    "processing_time_ms": 120
                },
                "timestamp": "2024-01-01T00:00:00Z"
            }
        },
        "websocket_streaming": {
            "endpoint": "/ws/stream",
            "protocol": "websocket",
            "connect": "ws://localhost:8080/ws/stream",
            "example_start_message": {
                "type": "StartStream",
                "stream_id": "unique-stream-id",
                "provider": "anthropic",
                "model": "claude-3-sonnet-20240229",
                "prompt": "Write a short story about AI",
                "system_prompt": "You are a creative writer",
                "config": {
                    "enabled": true,
                    "min_chunk_delay_ms": 100
                }
            },
            "example_response": {
                "stream_id": "unique-stream-id",
                "event": {
                    "type": "Chunk",
                    "data": {
                        "content": "In a world where artificial intelligence...",
                        "is_final": false,
                        "metadata": {
                            "model": "claude-3-sonnet-20240229",
                            "provider": "anthropic"
                        }
                    }
                },
                "sequence": 1
            }
        },
        "complete_response": {
            "endpoint": "/api/complete",
            "method": "POST",
            "description": "Get complete response (collects all streaming chunks)",
            "example_request": {
                "provider": "bedrock",
                "model": "anthropic.claude-3-sonnet-20240229-v1:0",
                "prompt": "Summarize the benefits of renewable energy"
            },
            "example_response": {
                "response": "Renewable energy offers numerous benefits including...",
                "metadata": {
                    "provider": "bedrock",
                    "model": "anthropic.claude-3-sonnet-20240229-v1:0",
                    "total_chunks": 5,
                    "processing_time_ms": 2500,
                    "total_tokens": 150
                }
            }
        }
    });
    
    Ok(HttpResponse::Ok().json(examples))
}

/// Configure streaming routes
pub fn configure_streaming_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/streaming")
            .route("/stream", web::post().to(stream_ai_response))
            .route("/complete", web::post().to(complete_ai_response))
            .route("/health", web::get().to(streaming_health))
            .route("/config", web::get().to(get_streaming_config))
            .route("/status", web::get().to(stream_status))
            .route("/examples", web::get().to(streaming_examples))
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_streaming_health() {
        let app = test::init_service(
            App::new().route("/health", web::get().to(streaming_health))
        ).await;
        
        let req = test::TestRequest::get()
            .uri("/health")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_get_streaming_config() {
        let app = test::init_service(
            App::new().route("/config", web::get().to(get_streaming_config))
        ).await;
        
        let req = test::TestRequest::get()
            .uri("/config")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[tokio::test]
    async fn test_streaming_request_deserialization() {
        let json = r#"{
            "provider": "openai",
            "model": "gpt-4",
            "prompt": "Hello world",
            "system_prompt": "You are helpful",
            "config": {
                "enabled": true,
                "max_chunk_size": 100,
                "include_metadata": true
            }
        }"#;
        
        let request: StreamingRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.provider, "openai");
        assert_eq!(request.model, "gpt-4");
        assert_eq!(request.prompt, "Hello world");
    }
}