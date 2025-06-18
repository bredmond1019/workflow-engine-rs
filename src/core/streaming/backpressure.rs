use futures_util::{Stream, StreamExt};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::sleep;

use crate::core::error::WorkflowError;
use super::types::{StreamChunk, StreamConfig, StreamingError};

/// Backpressure handler for streaming responses
pub struct BackpressureHandler {
    config: StreamConfig,
    buffer: Vec<StreamChunk>,
    last_send_time: Option<Instant>,
    total_chunks_sent: u64,
}

impl BackpressureHandler {
    pub fn new(config: StreamConfig) -> Self {
        Self {
            config,
            buffer: Vec::new(),
            last_send_time: None,
            total_chunks_sent: 0,
        }
    }

    /// Add a chunk to the buffer
    pub fn add_chunk(&mut self, chunk: StreamChunk) -> Result<(), StreamingError> {
        if let Some(buffer_size) = self.config.buffer_size {
            if self.buffer.len() >= buffer_size {
                return Err(StreamingError::BufferOverflow {
                    message: format!("Buffer overflow: {} chunks", buffer_size),
                });
            }
        }
        
        self.buffer.push(chunk);
        Ok(())
    }

    /// Check if we should send the next chunk based on timing constraints
    pub fn should_send_chunk(&self) -> bool {
        if self.buffer.is_empty() {
            return false;
        }

        // Check minimum delay
        if let Some(min_delay) = self.config.min_chunk_delay_ms {
            if let Some(last_send) = self.last_send_time {
                let elapsed = last_send.elapsed();
                if elapsed < Duration::from_millis(min_delay) {
                    return false;
                }
            }
        }

        // Check maximum delay
        if let Some(max_delay) = self.config.max_chunk_delay_ms {
            if let Some(last_send) = self.last_send_time {
                let elapsed = last_send.elapsed();
                if elapsed >= Duration::from_millis(max_delay) {
                    return true;
                }
            }
        }

        // Check chunk size
        if let Some(max_chunk_size) = self.config.max_chunk_size {
            let total_buffered_size: usize = self.buffer.iter()
                .map(|chunk| chunk.content_length())
                .sum();
            
            if total_buffered_size >= max_chunk_size {
                return true;
            }
        }

        // If no conditions to send are met, don't send
        false
    }

    /// Get the next chunk to send
    pub fn get_next_chunk(&mut self) -> Option<StreamChunk> {
        if self.should_send_chunk() && !self.buffer.is_empty() {
            self.last_send_time = Some(Instant::now());
            self.total_chunks_sent += 1;
            Some(self.buffer.remove(0))
        } else {
            None
        }
    }

    /// Get all remaining chunks (for cleanup)
    pub fn drain_buffer(&mut self) -> Vec<StreamChunk> {
        self.buffer.drain(..).collect()
    }

    /// Get statistics
    pub fn get_stats(&self) -> BackpressureStats {
        BackpressureStats {
            buffered_chunks: self.buffer.len(),
            total_chunks_sent: self.total_chunks_sent,
            buffer_size_bytes: self.buffer.iter()
                .map(|chunk| chunk.content_length())
                .sum(),
        }
    }
}

/// Statistics for backpressure handling
#[derive(Debug, Clone)]
pub struct BackpressureStats {
    pub buffered_chunks: usize,
    pub total_chunks_sent: u64,
    pub buffer_size_bytes: usize,
}

/// A stream wrapper that applies backpressure control
pub struct BackpressureStream<S> {
    inner: S,
    handler: BackpressureHandler,
    sender: Option<mpsc::UnboundedSender<Result<StreamChunk, WorkflowError>>>,
    receiver: Option<mpsc::UnboundedReceiver<Result<StreamChunk, WorkflowError>>>,
}

impl<S> BackpressureStream<S>
where
    S: Stream<Item = Result<StreamChunk, WorkflowError>> + Unpin,
{
    pub fn new(inner: S, config: StreamConfig) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        
        Self {
            inner,
            handler: BackpressureHandler::new(config),
            sender: Some(sender),
            receiver: Some(receiver),
        }
    }

    /// Start the backpressure processing task
    pub async fn start_processing(&mut self) {
        let sender = self.sender.take().unwrap();
        let mut handler = BackpressureHandler::new(self.handler.config.clone());
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(10));
            
            loop {
                interval.tick().await;
                
                // Send any chunks that are ready
                while let Some(chunk) = handler.get_next_chunk() {
                    if sender.send(Ok(chunk)).is_err() {
                        return; // Receiver dropped
                    }
                }
                
                // Check if we should continue
                if handler.buffer.is_empty() {
                    // Small delay to prevent busy waiting
                    sleep(Duration::from_millis(1)).await;
                }
            }
        });
    }
}

impl<S> Stream for BackpressureStream<S>
where
    S: Stream<Item = Result<StreamChunk, WorkflowError>> + Unpin,
{
    type Item = Result<StreamChunk, WorkflowError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Poll the receiver first
        if let Some(receiver) = &mut self.receiver {
            match receiver.poll_recv(cx) {
                Poll::Ready(Some(item)) => return Poll::Ready(Some(item)),
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => {}
            }
        }

        // Poll the inner stream and add to buffer
        match self.inner.poll_next_unpin(cx) {
            Poll::Ready(Some(Ok(chunk))) => {
                if let Err(e) = self.handler.add_chunk(chunk) {
                    return Poll::Ready(Some(Err(WorkflowError::ProcessingError {
                        message: e.to_string(),
                    })));
                }
                Poll::Pending
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => {
                // Stream ended, drain any remaining chunks
                let remaining = self.handler.drain_buffer();
                if remaining.is_empty() {
                    Poll::Ready(None)
                } else {
                    // Return the first remaining chunk
                    let chunk = remaining.into_iter().next().unwrap();
                    Poll::Ready(Some(Ok(chunk)))
                }
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Create a rate-limited stream
pub fn create_rate_limited_stream<S>(
    stream: S,
    config: StreamConfig,
) -> Pin<Box<dyn Stream<Item = Result<StreamChunk, WorkflowError>> + Send>>
where
    S: Stream<Item = Result<StreamChunk, WorkflowError>> + Send + 'static,
{
    let rate_limited = stream.then(move |chunk_result| {
        let min_delay = config.min_chunk_delay_ms.unwrap_or(0);
        
        async move {
            if min_delay > 0 {
                sleep(Duration::from_millis(min_delay)).await;
            }
            chunk_result
        }
    });
    
    Box::pin(rate_limited)
}

/// Create a buffered stream with backpressure control
pub fn create_buffered_stream<S>(
    stream: S,
    config: StreamConfig,
) -> Pin<Box<dyn Stream<Item = Result<StreamChunk, WorkflowError>> + Send>>
where
    S: Stream<Item = Result<StreamChunk, WorkflowError>> + Send + Unpin + 'static,
{
    let buffer_size = config.buffer_size.unwrap_or(1000);
    
    let buffered_stream = BufferedBackpressureStream::new(stream, config);
    Box::pin(buffered_stream)
}

/// Advanced buffered stream with backpressure control
pub struct BufferedBackpressureStream<S> {
    inner: S,
    buffer: Vec<StreamChunk>,
    config: StreamConfig,
    last_emit_time: Option<std::time::Instant>,
    total_buffered_size: usize,
    is_exhausted: bool,
}

impl<S> BufferedBackpressureStream<S>
where
    S: Stream<Item = Result<StreamChunk, WorkflowError>> + Unpin,
{
    pub fn new(inner: S, config: StreamConfig) -> Self {
        Self {
            inner,
            buffer: Vec::new(),
            config,
            last_emit_time: None,
            total_buffered_size: 0,
            is_exhausted: false,
        }
    }

    fn should_emit_chunk(&self) -> bool {
        if self.buffer.is_empty() {
            return false;
        }

        // Check minimum delay constraint
        if let Some(min_delay) = self.config.min_chunk_delay_ms {
            if let Some(last_emit) = self.last_emit_time {
                if last_emit.elapsed() < Duration::from_millis(min_delay) {
                    return false;
                }
            }
        }

        // Check buffer size constraints
        if let Some(max_chunk_size) = self.config.max_chunk_size {
            if self.total_buffered_size >= max_chunk_size {
                return true;
            }
        }

        // Check maximum delay constraint
        if let Some(max_delay) = self.config.max_chunk_delay_ms {
            if let Some(last_emit) = self.last_emit_time {
                if last_emit.elapsed() >= Duration::from_millis(max_delay) {
                    return true;
                }
            } else {
                // First chunk, emit immediately
                return true;
            }
        }

        // Check buffer count limit
        if let Some(buffer_size) = self.config.buffer_size {
            if self.buffer.len() >= buffer_size {
                return true;
            }
        }

        // Default: emit if we have something and no constraints prevent it
        !self.buffer.is_empty()
    }

    fn emit_next_chunk(&mut self) -> Option<StreamChunk> {
        if self.should_emit_chunk() && !self.buffer.is_empty() {
            let chunk = self.buffer.remove(0);
            self.total_buffered_size = self.total_buffered_size.saturating_sub(chunk.content_length());
            self.last_emit_time = Some(std::time::Instant::now());
            Some(chunk)
        } else {
            None
        }
    }

    fn add_to_buffer(&mut self, chunk: StreamChunk) -> Result<(), StreamingError> {
        // Check if buffer would overflow
        if let Some(buffer_size) = self.config.buffer_size {
            if self.buffer.len() >= buffer_size {
                return Err(StreamingError::BufferOverflow {
                    message: format!("Buffer overflow: {} chunks", buffer_size),
                });
            }
        }

        self.total_buffered_size += chunk.content_length();
        self.buffer.push(chunk);
        Ok(())
    }
}

impl<S> Stream for BufferedBackpressureStream<S>
where
    S: Stream<Item = Result<StreamChunk, WorkflowError>> + Unpin,
{
    type Item = Result<StreamChunk, WorkflowError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        loop {
            // Try to emit a buffered chunk first
            if let Some(chunk) = self.emit_next_chunk() {
                return std::task::Poll::Ready(Some(Ok(chunk)));
            }

            // If stream is exhausted and no more chunks to emit, we're done
            if self.is_exhausted && self.buffer.is_empty() {
                return std::task::Poll::Ready(None);
            }

            // Poll the inner stream for more chunks
            match self.inner.poll_next_unpin(cx) {
                std::task::Poll::Ready(Some(Ok(chunk))) => {
                    match self.add_to_buffer(chunk) {
                        Ok(()) => {
                            // Continue loop to check if we should emit
                            continue;
                        }
                        Err(e) => {
                            return std::task::Poll::Ready(Some(Err(WorkflowError::ProcessingError {
                                message: e.to_string(),
                            })));
                        }
                    }
                }
                std::task::Poll::Ready(Some(Err(e))) => {
                    return std::task::Poll::Ready(Some(Err(e)));
                }
                std::task::Poll::Ready(None) => {
                    self.is_exhausted = true;
                    // Continue loop to emit any remaining chunks
                    if self.buffer.is_empty() {
                        return std::task::Poll::Ready(None);
                    }
                    continue;
                }
                std::task::Poll::Pending => {
                    // No more data available right now
                    return std::task::Poll::Pending;
                }
            }
        }
    }
}

/// Adaptive backpressure controller that adjusts based on system load
pub struct AdaptiveBackpressureController {
    config: StreamConfig,
    current_delay: u64,
    load_factor: f64,
    last_adjustment: std::time::Instant,
    throughput_samples: Vec<(std::time::Instant, u64)>, // (timestamp, chunks_processed)
}

impl AdaptiveBackpressureController {
    pub fn new(config: StreamConfig) -> Self {
        Self {
            current_delay: config.min_chunk_delay_ms.unwrap_or(10),
            config,
            load_factor: 1.0,
            last_adjustment: std::time::Instant::now(),
            throughput_samples: Vec::new(),
        }
    }

    /// Update controller with current system metrics
    pub fn update_metrics(&mut self, chunks_processed: u64, processing_time_ms: u64) {
        let now = std::time::Instant::now();
        self.throughput_samples.push((now, chunks_processed));

        // Keep only recent samples (last 10 seconds)
        self.throughput_samples.retain(|(timestamp, _)| {
            now.duration_since(*timestamp).as_secs() <= 10
        });

        // Calculate current throughput
        let total_chunks: u64 = self.throughput_samples.iter().map(|(_, count)| count).sum();
        let time_span = self.throughput_samples.first().map(|(first, _)| {
            now.duration_since(*first).as_secs_f64()
        }).unwrap_or(1.0);

        let current_throughput = total_chunks as f64 / time_span;

        // Adjust load factor based on processing time
        if processing_time_ms > 1000 {
            // High processing time indicates system stress
            self.load_factor = (self.load_factor * 1.2).min(5.0);
        } else if processing_time_ms < 100 {
            // Low processing time indicates system has capacity
            self.load_factor = (self.load_factor * 0.9).max(0.1);
        }

        // Adjust delay based on load factor
        if now.duration_since(self.last_adjustment).as_secs() >= 1 {
            self.adjust_delay();
            self.last_adjustment = now;
        }
    }

    fn adjust_delay(&mut self) {
        let base_delay = self.config.min_chunk_delay_ms.unwrap_or(10) as f64;
        let max_delay = self.config.max_chunk_delay_ms.unwrap_or(1000) as f64;

        self.current_delay = (base_delay * self.load_factor).min(max_delay) as u64;
    }

    /// Get the current adaptive delay
    pub fn get_current_delay(&self) -> Duration {
        Duration::from_millis(self.current_delay)
    }

    /// Get current load factor
    pub fn get_load_factor(&self) -> f64 {
        self.load_factor
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::stream;

    #[tokio::test]
    async fn test_backpressure_handler() {
        let config = StreamConfig {
            enabled: true,
            max_chunk_size: Some(10),
            min_chunk_delay_ms: Some(50),
            ..Default::default()
        };
        
        let mut handler = BackpressureHandler::new(config);
        
        // Add a small chunk
        let chunk1 = StreamChunk::new("Hello".to_string(), false);
        handler.add_chunk(chunk1).unwrap();
        
        // Should not send immediately due to size limit
        assert!(!handler.should_send_chunk());
        
        // Add another chunk to exceed size limit
        let chunk2 = StreamChunk::new(" World!".to_string(), false);
        handler.add_chunk(chunk2).unwrap();
        
        // Should send now due to size
        assert!(handler.should_send_chunk());
        
        let sent_chunk = handler.get_next_chunk().unwrap();
        assert_eq!(sent_chunk.content, "Hello");
    }

    #[tokio::test]
    async fn test_rate_limited_stream() {
        let chunks = vec![
            Ok(StreamChunk::new("chunk1".to_string(), false)),
            Ok(StreamChunk::new("chunk2".to_string(), false)),
            Ok(StreamChunk::new("chunk3".to_string(), true)),
        ];
        
        let stream = stream::iter(chunks);
        let config = StreamConfig {
            min_chunk_delay_ms: Some(10),
            ..Default::default()
        };
        
        let mut rate_limited = create_rate_limited_stream(stream, config);
        
        let start = Instant::now();
        let mut count = 0;
        
        while let Some(_) = rate_limited.next().await {
            count += 1;
        }
        
        let elapsed = start.elapsed();
        assert_eq!(count, 3);
        assert!(elapsed >= Duration::from_millis(30)); // 3 chunks with 10ms delay each
    }
}