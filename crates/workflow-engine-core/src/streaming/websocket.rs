#[cfg(feature = "streaming")]
use actix::{Actor, ActorContext, AsyncContext, Handler, Message, StreamHandler};
#[cfg(feature = "streaming")]
use actix_web::{web, HttpRequest, HttpResponse, Result as ActixResult};
#[cfg(feature = "streaming")]
use actix_web_actors::ws;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::error::WorkflowError;
use super::types::{StreamChunk, StreamConfig, StreamEvent, StreamMessage, StreamingProvider, StreamResponse, StreamMetadata};

#[cfg(feature = "streaming")]
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

// When streaming feature is enabled, include full WebSocket implementation
#[cfg(feature = "streaming")]
pub mod websocket_impl {
    use super::*;

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
        pub code: Option<u32>,
    }

    impl StreamingWebSocket {
        /// Creates a new WebSocket actor
        pub fn new() -> Self {
            Self {
                session_id: Uuid::new_v4().to_string(),
                hb: Instant::now(),
                streams: HashMap::new(),
                default_config: StreamConfig::default(),
            }
        }

        /// Start heartbeat process for this connection
        fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
            ctx.run_interval(Duration::from_secs(30), |act, ctx| {
                if Instant::now().duration_since(act.hb) > Duration::from_secs(60) {
                    println!("WebSocket heartbeat failed, disconnecting!");
                    ctx.stop();
                    return;
                }
                ctx.ping(b"");
            });
        }
    }

    impl Actor for StreamingWebSocket {
        type Context = ws::WebsocketContext<Self>;

        fn started(&mut self, ctx: &mut Self::Context) {
            self.hb(ctx);
        }
    }

    // WebSocket message handler
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
                    let msg = text.trim();
                    if let Ok(ws_msg) = serde_json::from_str::<WebSocketMessage>(msg) {
                        self.handle_websocket_message(ws_msg, ctx);
                    }
                }
                Ok(ws::Message::Binary(bin)) => {
                    ctx.binary(bin);
                }
                Ok(ws::Message::Close(reason)) => {
                    ctx.close(reason);
                    ctx.stop();
                }
                _ => ctx.stop(),
            }
        }
    }

    impl StreamingWebSocket {
        fn handle_websocket_message(&mut self, msg: WebSocketMessage, ctx: &mut ws::WebsocketContext<Self>) {
            match msg {
                WebSocketMessage::StartStream { 
                    stream_id, 
                    provider, 
                    model, 
                    prompt, 
                    system_prompt, 
                    config 
                } => {
                    println!("Starting stream: {}", stream_id);
                    
                    let stream_config = config.unwrap_or_else(|| self.default_config.clone());
                    
                    // Create streaming provider
                    let client = Arc::new(reqwest::Client::new());
                    match create_streaming_provider(&provider, model.clone(), system_prompt.unwrap_or_else(|| "You are a helpful assistant.".to_string()), Some(client)) {
                        Ok(provider) => {
                            let provider_name = provider.provider_name().to_string();
                            
                            let session = StreamingSession {
                                stream_id: stream_id.clone(),
                                provider,
                                config: stream_config.clone(),
                                sequence: 0,
                                started_at: Instant::now(),
                            };
                            
                            self.streams.insert(stream_id.clone(), session);
                            
                            // Start streaming (simplified)
                            let response = StreamMessage {
                                stream_id: stream_id.clone(),
                                event: StreamEvent::Started {
                                    stream_id: stream_id.clone(),
                                    metadata: StreamMetadata::new(model, provider_name),
                                },
                                sequence: 0,
                            };
                            
                            ctx.text(serde_json::to_string(&response).unwrap_or_default());
                        }
                        Err(e) => {
                            let error_response = StreamMessage {
                                stream_id: stream_id.clone(),
                                event: StreamEvent::Error {
                                    stream_id: stream_id.clone(),
                                    error: e.to_string(),
                                },
                                sequence: 0,
                            };
                            ctx.text(serde_json::to_string(&error_response).unwrap_or_default());
                        }
                    }
                }
                WebSocketMessage::StopStream { stream_id } => {
                    if let Some(_session) = self.streams.remove(&stream_id) {
                        let response = StreamMessage {
                            stream_id: stream_id.clone(),
                            event: StreamEvent::Completed {
                                stream_id: stream_id.clone(),
                                total_chunks: 0,
                                total_tokens: None,
                                duration_ms: 0,
                            },
                            sequence: 0,
                        };
                        ctx.text(serde_json::to_string(&response).unwrap_or_default());
                    }
                }
                WebSocketMessage::Ping { timestamp: _ } => {
                    let pong = WebSocketMessage::Pong {
                        timestamp: chrono::Utc::now(),
                    };
                    ctx.text(serde_json::to_string(&pong).unwrap_or_default());
                }
                WebSocketMessage::Pong { timestamp: _ } => {
                    // Handle pong if needed
                }
            }
        }
    }

    // Actor message handlers
    impl Handler<StreamChunkMessage> for StreamingWebSocket {
        type Result = ();

        fn handle(&mut self, msg: StreamChunkMessage, ctx: &mut Self::Context) -> Self::Result {
            if let Some(session) = self.streams.get_mut(&msg.stream_id) {
                session.sequence = msg.sequence;
                
                let stream_msg = StreamMessage {
                    stream_id: msg.stream_id,
                    event: StreamEvent::Chunk {
                        data: msg.chunk,
                    },
                    sequence: msg.sequence,
                };
                
                ctx.text(serde_json::to_string(&stream_msg).unwrap_or_default());
            }
        }
    }

    impl Handler<StreamCompleteMessage> for StreamingWebSocket {
        type Result = ();

        fn handle(&mut self, msg: StreamCompleteMessage, ctx: &mut Self::Context) -> Self::Result {
            if let Some(_) = self.streams.remove(&msg.stream_id) {
                let stream_msg = StreamMessage {
                    stream_id: msg.stream_id.clone(),
                    event: StreamEvent::Completed {
                        stream_id: msg.stream_id,
                        total_chunks: msg.total_chunks,
                        total_tokens: msg.total_tokens,
                        duration_ms: msg.duration_ms,
                    },
                    sequence: 0,
                };
                
                ctx.text(serde_json::to_string(&stream_msg).unwrap_or_default());
            }
        }
    }

    impl Handler<StreamErrorMessage> for StreamingWebSocket {
        type Result = ();

        fn handle(&mut self, msg: StreamErrorMessage, ctx: &mut Self::Context) -> Self::Result {
            self.streams.remove(&msg.stream_id);
            
            let stream_msg = StreamMessage {
                stream_id: msg.stream_id.clone(),
                event: StreamEvent::Error {
                    stream_id: msg.stream_id,
                    error: msg.error,
                },
                sequence: 0,
            };
            
            ctx.text(serde_json::to_string(&stream_msg).unwrap_or_default());
        }
    }

    /// Create new WebSocket endpoint
    pub async fn websocket_endpoint(
        req: HttpRequest,
        stream: web::Payload,
    ) -> ActixResult<HttpResponse> {
        let ws = StreamingWebSocket::new();
        let resp = ws::start(ws, &req, stream);
        println!("WebSocket connection established");
        resp
    }
}

// Re-export types when streaming is enabled
#[cfg(feature = "streaming")]
pub use websocket_impl::*;

// When streaming feature is disabled, provide stub implementations
#[cfg(not(feature = "streaming"))]
pub mod websocket_stub {
    use super::*;
    
    /// Stub implementation when streaming is disabled
    pub struct StreamingWebSocket;
    
    impl StreamingWebSocket {
        pub fn new() -> Self {
            Self
        }
    }
    
    pub struct StreamChunkMessage;
    pub struct StreamCompleteMessage;
    pub struct StreamErrorMessage;
    
    /// Stub function that returns an error when streaming is disabled
    pub async fn websocket_endpoint() -> Result<(), WorkflowError> {
        Err(WorkflowError::UnsupportedOperation { 
            operation: "WebSocket streaming requires 'streaming' feature".to_string() 
        })
    }
}

#[cfg(not(feature = "streaming"))]
pub use websocket_stub::*;