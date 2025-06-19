use actix::{Actor, ActorContext, AsyncContext, Handler, Message, StreamHandler};
use actix_web::{web, HttpRequest, HttpResponse, Result as ActixResult};
use actix_web_actors::ws;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::core::error::WorkflowError;
use super::types::{StreamChunk, StreamConfig, StreamEvent, StreamMessage, StreamingProvider, StreamResponse};
use super::providers::create_streaming_provider;

/// Dummy provider for session tracking (real provider is moved to async task)
struct DummyProvider;

impl StreamingProvider for DummyProvider {
    fn stream_response(&self, _prompt: &str, _config: &StreamConfig) -> StreamResponse {
        use futures_util::stream;
        Box::pin(stream::empty())
    }
    
    fn provider_name(&self) -> &str {
        "dummy"
    }
    
    fn supports_streaming(&self) -> bool {
        false
    }
}

/// WebSocket actor for streaming AI responses
pub struct StreamingWebSocket {
    /// Unique session ID
    session_id: String,
    /// Heartbeat interval
    hb: Instant,
    /// Active streams
    streams: HashMap<String, StreamingSession>,
    /// Default streaming configuration
    default_config: StreamConfig,
}

/// Information about a streaming session
struct StreamingSession {
    stream_id: String,
    provider: Box<dyn StreamingProvider + Send + Sync>,
    config: StreamConfig,
    sequence: u64,
    started_at: Instant,
}

/// WebSocket messages
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    /// Start a new streaming session
    StartStream {
        stream_id: String,
        provider: String,
        model: String,
        prompt: String,
        system_prompt: Option<String>,
        config: Option<StreamConfig>,
    },
    /// Stop a streaming session
    StopStream {
        stream_id: String,
    },
    /// Ping message for keepalive
    Ping {
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Pong response
    Pong {
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// Actor messages
#[derive(Message)]
#[rtype(result = "()")]
pub struct StreamChunkMessage {
    pub stream_id: String,
    pub chunk: StreamChunk,
    pub sequence: u64,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct StreamCompleteMessage {
    pub stream_id: String,
    pub total_chunks: u32,
    pub duration_ms: u64,
    pub total_tokens: Option<u32>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct StreamErrorMessage {
    pub stream_id: String,
    pub error: String,
}

impl StreamingWebSocket {
    pub fn new() -> Self {
        Self {
            session_id: Uuid::new_v4().to_string(),
            hb: Instant::now(),
            streams: HashMap::new(),
            default_config: StreamConfig::default(),
        }
    }

    /// Start heartbeat process
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        let hb_interval = Duration::from_secs(5);
        let client_timeout = Duration::from_secs(10);
        
        ctx.run_interval(hb_interval, move |act, ctx| {
            if Instant::now().duration_since(act.hb) > client_timeout {
                println!("WebSocket client heartbeat failed, disconnecting!");
                ctx.stop();
                return;
            }
            
            ctx.ping(b"");
        });
    }

    /// Handle start stream message
    async fn handle_start_stream(
        &mut self,
        stream_id: String,
        provider: String,
        model: String,
        prompt: String,
        system_prompt: Option<String>,
        config: Option<StreamConfig>,
        ctx: &mut ws::WebsocketContext<Self>,
    ) {
        // Check if we already have this stream ID
        if self.streams.contains_key(&stream_id) {
            let error_msg = StreamMessage {
                stream_id: stream_id.clone(),
                event: StreamEvent::Error {
                    stream_id: stream_id.clone(),
                    error: "Stream with this ID already exists".to_string(),
                },
                sequence: 0,
            };
            
            if let Ok(json) = serde_json::to_string(&error_msg) {
                ctx.text(json);
            }
            return;
        }

        let system_prompt = system_prompt.unwrap_or_else(|| "You are a helpful assistant.".to_string());
        let config = config.unwrap_or_else(|| self.default_config.clone());
        
        // Create streaming provider with recovery
        let client = Arc::new(reqwest::Client::new());
        let base_provider = match create_streaming_provider(
            &provider,
            model.clone(),
            system_prompt.clone(),
            Some(client),
        ) {
            Ok(provider) => provider,
            Err(e) => {
                let error_msg = StreamMessage {
                    stream_id: stream_id.clone(),
                    event: StreamEvent::Error {
                        stream_id: stream_id.clone(),
                        error: format!("Failed to create provider: {}", e),
                    },
                    sequence: 0,
                };
                
                if let Ok(json) = serde_json::to_string(&error_msg) {
                    ctx.text(json);
                }
                return;
            }
        };

        // Wrap with recovery capabilities
        let recovery_config = super::recovery::StreamingRecoveryConfig::default();
        let recovery_provider = Arc::new(super::recovery::RecoveryStreamingProvider::new(
            Arc::from(base_provider),
            recovery_config,
        ));

        // Send stream started event
        let started_event = StreamMessage {
            stream_id: stream_id.clone(),
            event: StreamEvent::Started {
                stream_id: stream_id.clone(),
                metadata: super::types::StreamMetadata::new(model.clone(), provider.clone()),
            },
            sequence: 0,
        };
        
        if let Ok(json) = serde_json::to_string(&started_event) {
            ctx.text(json);
        }

        // Start streaming with recovery
        let addr = ctx.address();
        let stream_id_clone = stream_id.clone();
        let prompt_clone = prompt.clone();
        let started_at = Instant::now();
        let config_clone = config.clone();
        
        actix::spawn(async move {
            let mut chunk_stream = recovery_provider.stream_with_recovery(&prompt_clone, &config_clone).await;
            let mut chunk_count = 0u32;
            let mut sequence = 1u64;
            let mut total_tokens = None;
            
            while let Some(chunk_result) = chunk_stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        chunk_count += 1;
                        let is_final = chunk.is_final;
                        
                        // Track token usage
                        if let Some(metadata) = &chunk.metadata {
                            if let Some(tokens) = metadata.total_tokens {
                                total_tokens = Some(tokens);
                            }
                        }
                        
                        addr.do_send(StreamChunkMessage {
                            stream_id: stream_id_clone.clone(),
                            chunk,
                            sequence,
                        });
                        sequence += 1;
                        
                        if is_final {
                            let duration = started_at.elapsed();
                            addr.do_send(StreamCompleteMessage {
                                stream_id: stream_id_clone.clone(),
                                total_chunks: chunk_count,
                                duration_ms: duration.as_millis() as u64,
                                total_tokens,
                            });
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            stream_id = %stream_id_clone,
                            error = %e,
                            "WebSocket streaming error"
                        );
                        
                        addr.do_send(StreamErrorMessage {
                            stream_id: stream_id_clone.clone(),
                            error: format!("Stream error: {}", e),
                        });
                        break;
                    }
                }
            }
        });
        
        // Create streaming session for tracking
        let session = StreamingSession {
            stream_id: stream_id.clone(),
            provider: Box::new(DummyProvider), // Placeholder since we moved the real provider
            config,
            sequence: 1,
            started_at,
        };
        
        self.streams.insert(stream_id, session);
    }

    /// Handle stop stream message
    fn handle_stop_stream(&mut self, stream_id: String, ctx: &mut ws::WebsocketContext<Self>) {
        if let Some(session) = self.streams.remove(&stream_id) {
            let duration = session.started_at.elapsed();
            let completed_event = StreamMessage {
                stream_id: stream_id.clone(),
                event: StreamEvent::Completed {
                    stream_id,
                    total_chunks: 0, // We don't track this for stopped streams
                    total_tokens: None,
                    duration_ms: duration.as_millis() as u64,
                },
                sequence: session.sequence,
            };
            
            if let Ok(json) = serde_json::to_string(&completed_event) {
                ctx.text(json);
            }
        }
    }
}

impl Actor for StreamingWebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for StreamingWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                let text = text.trim();
                
                match serde_json::from_str::<WebSocketMessage>(text) {
                    Ok(WebSocketMessage::StartStream {
                        stream_id,
                        provider,
                        model,
                        prompt,
                        system_prompt,
                        config,
                    }) => {
                        // Handle start stream in a simplified way for now
                        println!("Starting stream: {} with provider: {}", stream_id, provider);
                        
                        // In a full implementation, you would start the actual streaming here
                        // For now, just send a simple response
                        let response_msg = serde_json::json!({
                            "stream_id": stream_id,
                            "event": {
                                "type": "Started",
                                "stream_id": stream_id,
                                "metadata": {
                                    "model": model,
                                    "provider": provider
                                }
                            },
                            "sequence": 0
                        });
                        
                        if let Ok(json) = serde_json::to_string(&response_msg) {
                            ctx.text(json);
                        }
                    }
                    Ok(WebSocketMessage::StopStream { stream_id }) => {
                        self.handle_stop_stream(stream_id, ctx);
                    }
                    Ok(WebSocketMessage::Ping { timestamp: _ }) => {
                        let pong_msg = WebSocketMessage::Pong {
                            timestamp: chrono::Utc::now(),
                        };
                        if let Ok(json) = serde_json::to_string(&pong_msg) {
                            ctx.text(json);
                        }
                    }
                    Ok(WebSocketMessage::Pong { timestamp: _ }) => {
                        // Handle pong
                    }
                    Err(e) => {
                        println!("Failed to parse WebSocket message: {}", e);
                    }
                }
            }
            Ok(ws::Message::Binary(_)) => {
                println!("Unexpected binary message");
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

impl Handler<StreamChunkMessage> for StreamingWebSocket {
    type Result = ();

    fn handle(&mut self, msg: StreamChunkMessage, ctx: &mut Self::Context) {
        let stream_event = StreamMessage {
            stream_id: msg.stream_id.clone(),
            event: StreamEvent::Chunk {
                data: msg.chunk,
            },
            sequence: msg.sequence,
        };
        
        if let Ok(json) = serde_json::to_string(&stream_event) {
            ctx.text(json);
        }
    }
}

impl Handler<StreamCompleteMessage> for StreamingWebSocket {
    type Result = ();

    fn handle(&mut self, msg: StreamCompleteMessage, ctx: &mut Self::Context) {
        // Remove the completed stream
        self.streams.remove(&msg.stream_id);
        
        let completed_event = StreamMessage {
            stream_id: msg.stream_id.clone(),
            event: StreamEvent::Completed {
                stream_id: msg.stream_id,
                total_chunks: msg.total_chunks,
                total_tokens: msg.total_tokens,
                duration_ms: msg.duration_ms,
            },
            sequence: 0, // Final message
        };
        
        if let Ok(json) = serde_json::to_string(&completed_event) {
            ctx.text(json);
        }
    }
}

impl Handler<StreamErrorMessage> for StreamingWebSocket {
    type Result = ();

    fn handle(&mut self, msg: StreamErrorMessage, ctx: &mut Self::Context) {
        // Remove the errored stream
        self.streams.remove(&msg.stream_id);
        
        let error_event = StreamMessage {
            stream_id: msg.stream_id.clone(),
            event: StreamEvent::Error {
                stream_id: msg.stream_id,
                error: msg.error,
            },
            sequence: 0, // Error message
        };
        
        if let Ok(json) = serde_json::to_string(&error_event) {
            ctx.text(json);
        }
    }
}

/// WebSocket endpoint handler
pub async fn websocket_streaming_handler(
    req: HttpRequest,
    stream: web::Payload,
) -> ActixResult<HttpResponse> {
    ws::start(StreamingWebSocket::new(), &req, stream)
}

/// Configuration for WebSocket streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketStreamingConfig {
    /// Maximum number of concurrent streams per connection
    pub max_concurrent_streams: usize,
    /// Heartbeat interval in seconds
    pub heartbeat_interval_secs: u64,
    /// Client timeout in seconds
    pub client_timeout_secs: u64,
    /// Maximum message size in bytes
    pub max_message_size: usize,
}

impl Default for WebSocketStreamingConfig {
    fn default() -> Self {
        Self {
            max_concurrent_streams: 5,
            heartbeat_interval_secs: 5,
            client_timeout_secs: 10,
            max_message_size: 64 * 1024, // 64KB
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_message_serialization() {
        let start_msg = WebSocketMessage::StartStream {
            stream_id: "test-123".to_string(),
            provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            prompt: "Hello world".to_string(),
            system_prompt: Some("You are helpful".to_string()),
            config: None,
        };
        
        let json = serde_json::to_string(&start_msg).unwrap();
        let parsed: WebSocketMessage = serde_json::from_str(&json).unwrap();
        
        if let WebSocketMessage::StartStream { stream_id, .. } = parsed {
            assert_eq!(stream_id, "test-123");
        } else {
            assert!(false, "Expected StartStream message type, got: {:?}", parsed);
        }
    }

    #[test]
    fn test_stream_event_serialization() {
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
    }
}