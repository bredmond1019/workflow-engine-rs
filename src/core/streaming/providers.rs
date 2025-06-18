use async_trait::async_trait;
use bytes::Bytes;
use futures_util::{Stream, StreamExt, TryStreamExt};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;

use crate::core::error::WorkflowError;
use super::types::{StreamChunk, StreamConfig, StreamMetadata, StreamingProvider, StreamResponse};
use super::sse::SSEParser;
use super::backpressure::create_rate_limited_stream;

/// OpenAI streaming provider implementation
#[derive(Debug)]
pub struct OpenAIStreamingProvider {
    client: Arc<reqwest::Client>,
    model_name: String,
    system_prompt: String,
}

impl OpenAIStreamingProvider {
    pub fn new(client: Arc<reqwest::Client>, model_name: String, system_prompt: String) -> Self {
        Self {
            client,
            model_name,
            system_prompt,
        }
    }

    /// Create a streaming request to OpenAI API
    async fn create_streaming_request(
        &self,
        prompt: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>, WorkflowError> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| WorkflowError::ConfigurationError("OPENAI_API_KEY not set".to_string()))?;

        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&serde_json::json!({
                "model": &self.model_name,
                "messages": [
                    {
                        "role": "system",
                        "content": &self.system_prompt
                    },
                    {
                        "role": "user",
                        "content": prompt
                    }
                ],
                "stream": true
            }))
            .send()
            .await
            .map_err(|e| WorkflowError::ApiError {
                message: format!("OpenAI streaming API request failed: {}", e),
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            return Err(WorkflowError::ApiError {
                message: format!("OpenAI streaming API error: {} - {}", status, error_body),
            });
        }

        use futures_util::TryStreamExt;
        let byte_stream = response.bytes_stream();
        Ok(Box::pin(byte_stream))
    }
}

#[async_trait]
impl StreamingProvider for OpenAIStreamingProvider {
    fn stream_response(&self, prompt: &str, config: &StreamConfig) -> StreamResponse {
        let client = self.client.clone();
        let model_name = self.model_name.clone();
        let system_prompt = self.system_prompt.clone();
        let prompt = prompt.to_string();
        let config = config.clone();

        let stream = async_stream::stream! {
            let provider = OpenAIStreamingProvider::new(client, model_name.clone(), system_prompt);
            
            match provider.create_streaming_request(&prompt).await {
                Ok(byte_stream) => {
                    let parser = SSEParser::new("openai".to_string(), model_name.clone());
                    let mut chunk_stream = parser.parse_sse_stream(byte_stream);
                    
                    while let Some(chunk_result) = chunk_stream.next().await {
                        match chunk_result {
                            Ok(mut chunk) => {
                                // Add timing metadata if enabled
                                if config.include_metadata {
                                    if let Some(ref mut metadata) = chunk.metadata {
                                        metadata.processing_time_ms = Some(
                                            chunk.timestamp.timestamp_millis() as u64
                                        );
                                    }
                                }
                                yield Ok(chunk);
                            }
                            Err(e) => yield Err(e),
                        }
                    }
                }
                Err(e) => yield Err(e),
            }
        };

        if config.min_chunk_delay_ms.is_some() {
            create_rate_limited_stream(Box::pin(stream), config)
        } else {
            Box::pin(stream)
        }
    }

    fn provider_name(&self) -> &str {
        "openai"
    }

    fn supports_streaming(&self) -> bool {
        true
    }
}

/// Anthropic streaming provider implementation
#[derive(Debug)]
pub struct AnthropicStreamingProvider {
    client: Arc<reqwest::Client>,
    model_name: String,
    system_prompt: String,
}

impl AnthropicStreamingProvider {
    pub fn new(client: Arc<reqwest::Client>, model_name: String, system_prompt: String) -> Self {
        Self {
            client,
            model_name,
            system_prompt,
        }
    }

    /// Create a streaming request to Anthropic API
    async fn create_streaming_request(
        &self,
        prompt: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>, WorkflowError> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| WorkflowError::ConfigurationError("ANTHROPIC_API_KEY not set".to_string()))?;

        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&serde_json::json!({
                "model": &self.model_name,
                "max_tokens": 4096,
                "messages": [
                    {
                        "role": "user",
                        "content": prompt
                    }
                ],
                "system": &self.system_prompt,
                "stream": true
            }))
            .send()
            .await
            .map_err(|e| WorkflowError::ApiError {
                message: format!("Anthropic streaming API request failed: {}", e),
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            return Err(WorkflowError::ApiError {
                message: format!("Anthropic streaming API error: {} - {}", status, error_body),
            });
        }

        use futures_util::TryStreamExt;
        let byte_stream = response.bytes_stream();
        Ok(Box::pin(byte_stream))
    }
}

#[async_trait]
impl StreamingProvider for AnthropicStreamingProvider {
    fn stream_response(&self, prompt: &str, config: &StreamConfig) -> StreamResponse {
        let client = self.client.clone();
        let model_name = self.model_name.clone();
        let system_prompt = self.system_prompt.clone();
        let prompt = prompt.to_string();
        let config = config.clone();

        let stream = async_stream::stream! {
            let provider = AnthropicStreamingProvider::new(client, model_name.clone(), system_prompt);
            
            match provider.create_streaming_request(&prompt).await {
                Ok(byte_stream) => {
                    let parser = SSEParser::new("anthropic".to_string(), model_name.clone());
                    let mut chunk_stream = parser.parse_sse_stream(byte_stream);
                    
                    while let Some(chunk_result) = chunk_stream.next().await {
                        match chunk_result {
                            Ok(mut chunk) => {
                                // Add timing metadata if enabled
                                if config.include_metadata {
                                    if let Some(ref mut metadata) = chunk.metadata {
                                        metadata.processing_time_ms = Some(
                                            chunk.timestamp.timestamp_millis() as u64
                                        );
                                    }
                                }
                                yield Ok(chunk);
                            }
                            Err(e) => yield Err(e),
                        }
                    }
                }
                Err(e) => yield Err(e),
            }
        };

        if config.min_chunk_delay_ms.is_some() {
            create_rate_limited_stream(Box::pin(stream), config)
        } else {
            Box::pin(stream)
        }
    }

    fn provider_name(&self) -> &str {
        "anthropic"
    }

    fn supports_streaming(&self) -> bool {
        true
    }
}

/// Bedrock streaming provider implementation
#[derive(Debug)]
pub struct BedrockStreamingProvider {
    model_name: String,
    system_prompt: String,
}

impl BedrockStreamingProvider {
    pub fn new(model_name: String, system_prompt: String) -> Self {
        Self {
            model_name,
            system_prompt,
        }
    }

    /// Create a real streaming response using Bedrock's invoke_model_with_response_stream
    async fn create_streaming_request(
        &self,
        prompt: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, WorkflowError>> + Send>>, WorkflowError> {
        use aws_sdk_bedrockruntime::{primitives::Blob, Client};
        use futures_util::TryStreamExt;
        
        // Initialize AWS SDK
        let config = aws_config::load_from_env().await;
        let client = Client::new(&config);
        
        // Prepare the request body based on the model
        let request_body = if self.model_name.starts_with("anthropic.claude") {
            serde_json::json!({
                "anthropic_version": "bedrock-2023-05-31",
                "max_tokens": 4096,
                "messages": [{
                    "role": "user",
                    "content": prompt
                }],
                "system": &self.system_prompt
            })
        } else if self.model_name.starts_with("amazon.titan") {
            serde_json::json!({
                "inputText": format!("{}\n\n{}", self.system_prompt, prompt),
                "textGenerationConfig": {
                    "maxTokenCount": 4096,
                    "temperature": 0.7,
                    "topP": 0.9
                }
            })
        } else if self.model_name.starts_with("cohere.command") {
            serde_json::json!({
                "message": prompt,
                "chat_history": [],
                "max_tokens": 4096,
                "temperature": 0.7,
                "p": 0.9
            })
        } else if self.model_name.starts_with("ai21.j2") {
            serde_json::json!({
                "prompt": format!("{}\n\n{}", self.system_prompt, prompt),
                "maxTokens": 4096,
                "temperature": 0.7,
                "topP": 0.9
            })
        } else {
            return Err(WorkflowError::ConfigurationError(
                format!("Unsupported Bedrock model: {}", self.model_name)
            ));
        };
        
        let body = Blob::new(serde_json::to_vec(&request_body).map_err(|e| {
            WorkflowError::SerializationError {
                message: format!("Failed to serialize request body: {}", e),
            }
        })?);
        
        // Use streaming API
        let response_stream = client
            .invoke_model_with_response_stream()
            .model_id(&self.model_name)
            .content_type("application/json")
            .body(body)
            .send()
            .await
            .map_err(|e| WorkflowError::ApiError {
                message: format!("Bedrock streaming API request failed: {}", e),
            })?;

        let model_name = self.model_name.clone();
        
        // Convert the response stream to our chunk stream
        let mut complete_text = String::new();
        let mut event_receiver = response_stream.body;
        
        while let Some(event) = event_receiver.recv().await.map_err(|e| {
            WorkflowError::ProcessingError {
                message: format!("Failed to receive stream event: {}", e),
            }
        })? {
            match event {
                    aws_sdk_bedrockruntime::types::ResponseStream::Chunk(chunk_data) => {
                        let bytes = chunk_data.bytes().ok_or_else(|| {
                            crate::core::error::WorkflowError::ProcessingError {
                                message: "No bytes in chunk".to_string()
                            }
                        })?;
                        let chunk_str = String::from_utf8_lossy(bytes.as_ref());
                        
                        // Parse the chunk based on model type
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&chunk_str) {
                            let text_content = Self::extract_text_from_chunk(&json, &model_name)?;
                            if !text_content.is_empty() {
                                complete_text.push_str(&text_content);
                            }
                        }
                    }
                _ => {
                    // Handle any other variants including Unknown
                }
            }
        }
        
        // Now create a stream from the complete text
        let chunks = Self::chunk_text_for_streaming(&complete_text, &model_name);
        let chunk_stream = futures_util::stream::iter(chunks.into_iter().map(Ok));

        Ok(Box::pin(chunk_stream))
    }

    /// Extract text content from a streaming chunk based on model type
    fn extract_text_from_chunk(json: &serde_json::Value, model_name: &str) -> Result<String, WorkflowError> {
        if model_name.starts_with("anthropic.claude") {
            // Claude streaming format
            if let Some(delta) = json.get("delta") {
                if let Some(text) = delta.get("text").and_then(|t| t.as_str()) {
                    return Ok(text.to_string());
                }
            }
        } else if model_name.starts_with("amazon.titan") {
            // Titan streaming format
            if let Some(output_text) = json.get("outputText").and_then(|t| t.as_str()) {
                return Ok(output_text.to_string());
            }
        } else if model_name.starts_with("cohere.command") {
            // Cohere streaming format
            if let Some(text) = json.get("text").and_then(|t| t.as_str()) {
                return Ok(text.to_string());
            }
        } else if model_name.starts_with("ai21.j2") {
            // AI21 streaming format
            if let Some(completions) = json.get("completions").and_then(|c| c.as_array()) {
                if let Some(completion) = completions.first() {
                    if let Some(data) = completion.get("data").and_then(|d| d.get("text")).and_then(|t| t.as_str()) {
                        return Ok(data.to_string());
                    }
                }
            }
        }
        
        Ok(String::new())
    }

    /// Chunk text for streaming output
    fn chunk_text_for_streaming(text: &str, model_name: &str) -> Vec<StreamChunk> {
        let chunk_size = 50; // words per chunk
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut chunks = Vec::new();
        
        if words.is_empty() {
            // Empty response
            chunks.push(StreamChunk::with_metadata(
                String::new(),
                true,
                StreamMetadata::new(model_name.to_string(), "bedrock".to_string()),
            ));
            return chunks;
        }
        
        for (i, word_chunk) in words.chunks(chunk_size).enumerate() {
            let chunk_content = word_chunk.join(" ");
            let is_final = i == (words.len() + chunk_size - 1) / chunk_size - 1;
            
            let metadata = StreamMetadata::new(
                model_name.to_string(),
                "bedrock".to_string(),
            );
            
            chunks.push(StreamChunk::with_metadata(
                chunk_content,
                is_final,
                metadata,
            ));
        }
        
        chunks
    }

    /// Fallback to non-streaming for unsupported models
    async fn create_chunked_response(
        &self,
        prompt: &str,
    ) -> Result<Vec<StreamChunk>, WorkflowError> {
        use aws_sdk_bedrockruntime::{primitives::Blob, Client};
        
        // Initialize AWS SDK
        let config = aws_config::load_from_env().await;
        let client = Client::new(&config);
        
        // Prepare the request body based on the model
        let request_body = if self.model_name.starts_with("anthropic.claude") {
            serde_json::json!({
                "anthropic_version": "bedrock-2023-05-31",
                "max_tokens": 4096,
                "messages": [{
                    "role": "user",
                    "content": prompt
                }],
                "system": &self.system_prompt
            })
        } else if self.model_name.starts_with("amazon.titan") {
            serde_json::json!({
                "inputText": format!("{}\n\n{}", self.system_prompt, prompt),
                "textGenerationConfig": {
                    "maxTokenCount": 4096,
                    "temperature": 0.7,
                    "topP": 0.9
                }
            })
        } else {
            return Err(WorkflowError::ConfigurationError(
                format!("Unsupported Bedrock model: {}", self.model_name)
            ));
        };
        
        let body = Blob::new(serde_json::to_vec(&request_body).map_err(|e| {
            WorkflowError::SerializationError {
                message: format!("Failed to serialize request body: {}", e),
            }
        })?);
        
        let response = client
            .invoke_model()
            .model_id(&self.model_name)
            .content_type("application/json")
            .accept("application/json")
            .body(body)
            .send()
            .await
            .map_err(|e| WorkflowError::ApiError {
                message: format!("Bedrock API request failed: {}", e),
            })?;
        
        let response_body = response.body().as_ref();
        let response_json: serde_json::Value = serde_json::from_slice(response_body)
            .map_err(|e| WorkflowError::DeserializationError {
                message: format!("Failed to parse Bedrock response: {}", e),
            })?;

        // Extract the text content based on model type
        let content = if self.model_name.starts_with("anthropic.claude") {
            response_json["content"][0]["text"]
                .as_str()
                .ok_or_else(|| WorkflowError::ApiError {
                    message: "Invalid response structure from Bedrock Claude".to_string(),
                })?
        } else if self.model_name.starts_with("amazon.titan") {
            response_json["results"][0]["outputText"]
                .as_str()
                .ok_or_else(|| WorkflowError::ApiError {
                    message: "Invalid response structure from Bedrock Titan".to_string(),
                })?
        } else {
            return Err(WorkflowError::ConfigurationError(
                format!("Unsupported Bedrock model for content extraction: {}", self.model_name)
            ));
        };

        Ok(Self::chunk_text_for_streaming(content, &self.model_name))
    }
}

#[async_trait]
impl StreamingProvider for BedrockStreamingProvider {
    fn stream_response(&self, prompt: &str, config: &StreamConfig) -> StreamResponse {
        let model_name = self.model_name.clone();
        let system_prompt = self.system_prompt.clone();
        let prompt = prompt.to_string();
        let config = config.clone();

        let stream = async_stream::stream! {
            let provider = BedrockStreamingProvider::new(model_name, system_prompt);
            
            // Try to use real streaming first
            match provider.create_streaming_request(&prompt).await {
                Ok(mut chunk_stream) => {
                    while let Some(chunk_result) = chunk_stream.next().await {
                        match chunk_result {
                            Ok(mut chunk) => {
                                // Add timing metadata if enabled
                                if config.include_metadata {
                                    if let Some(ref mut metadata) = chunk.metadata {
                                        metadata.processing_time_ms = Some(
                                            chunk.timestamp.timestamp_millis() as u64
                                        );
                                    }
                                }
                                yield Ok(chunk);
                            }
                            Err(e) => yield Err(e),
                        }
                    }
                }
                Err(streaming_error) => {
                    // Fall back to chunked response if streaming fails
                    tracing::warn!(
                        error = %streaming_error,
                        "Bedrock streaming failed, falling back to chunked response"
                    );
                    
                    match provider.create_chunked_response(&prompt).await {
                        Ok(chunks) => {
                            for chunk in chunks {
                                // Add artificial delay to simulate streaming
                                if let Some(delay) = config.min_chunk_delay_ms {
                                    tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                                }
                                yield Ok(chunk);
                            }
                        }
                        Err(e) => yield Err(e),
                    }
                }
            }
        };

        if config.min_chunk_delay_ms.is_some() {
            create_rate_limited_stream(Box::pin(stream), config)
        } else {
            Box::pin(stream)
        }
    }

    fn provider_name(&self) -> &str {
        "bedrock"
    }

    fn supports_streaming(&self) -> bool {
        true // Real streaming with fallback
    }
}

/// Factory function to create streaming providers
pub fn create_streaming_provider(
    provider_name: &str,
    model_name: String,
    system_prompt: String,
    client: Option<Arc<reqwest::Client>>,
) -> Result<Box<dyn StreamingProvider + Send + Sync>, WorkflowError> {
    match provider_name.to_lowercase().as_str() {
        "openai" | "azureopenai" => {
            let client = client.ok_or_else(|| WorkflowError::ConfigurationError(
                "HTTP client required for OpenAI streaming".to_string(),
            ))?;
            Ok(Box::new(OpenAIStreamingProvider::new(client, model_name, system_prompt)))
        }
        "anthropic" => {
            let client = client.ok_or_else(|| WorkflowError::ConfigurationError(
                "HTTP client required for Anthropic streaming".to_string(),
            ))?;
            Ok(Box::new(AnthropicStreamingProvider::new(client, model_name, system_prompt)))
        }
        "bedrock" => {
            Ok(Box::new(BedrockStreamingProvider::new(model_name, system_prompt)))
        }
        _ => Err(WorkflowError::ConfigurationError(
            format!("Unsupported streaming provider: {}", provider_name),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_create_streaming_provider() {
        let client = Arc::new(reqwest::Client::new());
        
        // Test OpenAI provider creation
        let openai_provider = create_streaming_provider(
            "openai",
            "gpt-4".to_string(),
            "You are a helpful assistant".to_string(),
            Some(client.clone()),
        );
        assert!(openai_provider.is_ok());
        assert_eq!(openai_provider.unwrap().provider_name(), "openai");
        
        // Test Anthropic provider creation
        let anthropic_provider = create_streaming_provider(
            "anthropic",
            "claude-3-sonnet-20240229".to_string(),
            "You are a helpful assistant".to_string(),
            Some(client.clone()),
        );
        assert!(anthropic_provider.is_ok());
        assert_eq!(anthropic_provider.unwrap().provider_name(), "anthropic");
        
        // Test Bedrock provider creation
        let bedrock_provider = create_streaming_provider(
            "bedrock",
            "anthropic.claude-3-sonnet-20240229-v1:0".to_string(),
            "You are a helpful assistant".to_string(),
            None,
        );
        assert!(bedrock_provider.is_ok());
        assert_eq!(bedrock_provider.unwrap().provider_name(), "bedrock");
        
        // Test unsupported provider
        let unsupported = create_streaming_provider(
            "unsupported",
            "model".to_string(),
            "prompt".to_string(),
            None,
        );
        assert!(unsupported.is_err());
    }
}