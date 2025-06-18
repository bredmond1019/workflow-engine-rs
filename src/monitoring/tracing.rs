/*!
# Distributed Tracing

This module provides distributed tracing capabilities using OpenTelemetry
for tracking requests across system boundaries.

Task 3.5: Set up Jaeger for distributed tracing
*/

use std::collections::HashMap;
use uuid::Uuid;

/// Span represents a unit of work in distributed tracing
#[derive(Debug, Clone)]
pub struct Span {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub operation_name: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub tags: HashMap<String, String>,
    pub logs: Vec<LogEntry>,
    pub status: SpanStatus,
}

/// Log entry within a span
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, String>,
}

/// Span status
#[derive(Debug, Clone, PartialEq)]
pub enum SpanStatus {
    Ok,
    Error,
    Timeout,
    Cancelled,
}

impl Span {
    /// Create a new root span
    pub fn new_root(operation_name: String) -> Self {
        Self {
            trace_id: Uuid::new_v4().to_string(),
            span_id: Uuid::new_v4().to_string(),
            parent_span_id: None,
            operation_name,
            start_time: chrono::Utc::now(),
            end_time: None,
            tags: HashMap::new(),
            logs: Vec::new(),
            status: SpanStatus::Ok,
        }
    }

    /// Create a child span
    pub fn new_child(&self, operation_name: String) -> Self {
        Self {
            trace_id: self.trace_id.clone(),
            span_id: Uuid::new_v4().to_string(),
            parent_span_id: Some(self.span_id.clone()),
            operation_name,
            start_time: chrono::Utc::now(),
            end_time: None,
            tags: HashMap::new(),
            logs: Vec::new(),
            status: SpanStatus::Ok,
        }
    }

    /// Add a tag to the span
    pub fn set_tag(&mut self, key: String, value: String) {
        self.tags.insert(key, value);
    }

    /// Add a log entry to the span
    pub fn log(&mut self, level: String, message: String) {
        let log_entry = LogEntry {
            timestamp: chrono::Utc::now(),
            level,
            message,
            fields: HashMap::new(),
        };
        self.logs.push(log_entry);
    }

    /// Add a log entry with additional fields
    pub fn log_with_fields(&mut self, level: String, message: String, fields: HashMap<String, String>) {
        let log_entry = LogEntry {
            timestamp: chrono::Utc::now(),
            level,
            message,
            fields,
        };
        self.logs.push(log_entry);
    }

    /// Finish the span
    pub fn finish(&mut self) {
        self.end_time = Some(chrono::Utc::now());
    }

    /// Finish the span with error status
    pub fn finish_with_error(&mut self, error_message: String) {
        self.status = SpanStatus::Error;
        self.set_tag("error".to_string(), "true".to_string());
        self.set_tag("error.message".to_string(), error_message);
        self.finish();
    }

    /// Get span duration in milliseconds
    pub fn duration_ms(&self) -> Option<i64> {
        self.end_time.map(|end| {
            (end - self.start_time).num_milliseconds()
        })
    }

    /// Convert span to Jaeger-compatible format
    pub fn to_jaeger_format(&self) -> serde_json::Value {
        let mut jaeger_span = serde_json::json!({
            "traceID": self.trace_id,
            "spanID": self.span_id,
            "operationName": self.operation_name,
            "startTime": self.start_time.timestamp_micros(),
            "tags": self.tags.iter().map(|(k, v)| {
                serde_json::json!({
                    "key": k,
                    "vStr": v,
                    "vType": "string"
                })
            }).collect::<Vec<_>>(),
            "logs": self.logs.iter().map(|log| {
                serde_json::json!({
                    "timestamp": log.timestamp.timestamp_micros(),
                    "fields": [{
                        "key": "level",
                        "vStr": log.level,
                        "vType": "string"
                    }, {
                        "key": "message",
                        "vStr": log.message,
                        "vType": "string"
                    }]
                })
            }).collect::<Vec<_>>()
        });

        if let Some(parent_id) = &self.parent_span_id {
            jaeger_span["parentSpanID"] = serde_json::Value::String(parent_id.clone());
        }

        if let Some(end_time) = self.end_time {
            jaeger_span["duration"] = serde_json::Value::Number(
                serde_json::Number::from((end_time - self.start_time).num_microseconds().unwrap_or(0))
            );
        }

        jaeger_span
    }
}

/// Tracer for managing spans and traces
pub struct Tracer {
    service_name: String,
    jaeger_endpoint: Option<String>,
    active_spans: HashMap<String, Span>,
}

impl Tracer {
    /// Create a new tracer
    pub fn new(service_name: String) -> Self {
        Self {
            service_name,
            jaeger_endpoint: std::env::var("JAEGER_ENDPOINT").ok(),
            active_spans: HashMap::new(),
        }
    }

    /// Start a new root span
    pub fn start_span(&mut self, operation_name: String) -> String {
        let mut span = Span::new_root(operation_name);
        span.set_tag("service.name".to_string(), self.service_name.clone());
        
        let span_id = span.span_id.clone();
        self.active_spans.insert(span_id.clone(), span);
        span_id
    }

    /// Start a child span
    pub fn start_child_span(&mut self, parent_span_id: &str, operation_name: String) -> Option<String> {
        if let Some(parent_span) = self.active_spans.get(parent_span_id) {
            let mut child_span = parent_span.new_child(operation_name);
            child_span.set_tag("service.name".to_string(), self.service_name.clone());
            
            let span_id = child_span.span_id.clone();
            self.active_spans.insert(span_id.clone(), child_span);
            Some(span_id)
        } else {
            None
        }
    }

    /// Get a mutable reference to a span
    pub fn get_span_mut(&mut self, span_id: &str) -> Option<&mut Span> {
        self.active_spans.get_mut(span_id)
    }

    /// Finish a span and optionally send to Jaeger
    pub async fn finish_span(&mut self, span_id: &str) {
        if let Some(mut span) = self.active_spans.remove(span_id) {
            span.finish();
            
            // Send to Jaeger if endpoint is configured
            if let Some(ref endpoint) = self.jaeger_endpoint {
                self.send_to_jaeger(endpoint, &span).await;
            }
            
            // Log span completion
            log::debug!(
                "Span completed: {} (trace: {}, duration: {}ms)",
                span.operation_name,
                span.trace_id,
                span.duration_ms().unwrap_or(0)
            );
        }
    }

    /// Send span to Jaeger
    async fn send_to_jaeger(&self, endpoint: &str, span: &Span) {
        let jaeger_batch = serde_json::json!({
            "spans": [span.to_jaeger_format()],
            "process": {
                "serviceName": self.service_name,
                "tags": []
            }
        });

        let client = reqwest::Client::new();
        let url = format!("{}/api/traces", endpoint);
        
        match client.post(&url).json(&jaeger_batch).send().await {
            Ok(response) if response.status().is_success() => {
                log::debug!("Successfully sent span to Jaeger: {}", span.span_id);
            }
            Ok(response) => {
                log::warn!("Failed to send span to Jaeger: {}", response.status());
            }
            Err(e) => {
                log::warn!("Error sending span to Jaeger: {}", e);
            }
        }
    }
}

/// Global tracer instance
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;

lazy_static! {
    static ref GLOBAL_TRACER: Arc<Mutex<Tracer>> = Arc::new(Mutex::new(
        Tracer::new("ai-workflow-system".to_string())
    ));
}

/// Trace a function call
pub async fn trace_async<F, Fut, T>(operation_name: &str, f: F) -> T
where
    F: FnOnce(String) -> Fut,
    Fut: std::future::Future<Output = T>,
{
    let span_id = {
        let mut tracer = GLOBAL_TRACER.lock().unwrap();
        tracer.start_span(operation_name.to_string())
    };

    let result = f(span_id.clone()).await;

    let mut tracer = GLOBAL_TRACER.lock().unwrap();
    tracer.finish_span(&span_id).await;

    result
}

/// Add a tag to an active span
pub fn add_span_tag(span_id: &str, key: String, value: String) {
    let mut tracer = GLOBAL_TRACER.lock().unwrap();
    if let Some(span) = tracer.get_span_mut(span_id) {
        span.set_tag(key, value);
    }
}

/// Add a log to an active span
pub fn add_span_log(span_id: &str, level: String, message: String) {
    let mut tracer = GLOBAL_TRACER.lock().unwrap();
    if let Some(span) = tracer.get_span_mut(span_id) {
        span.log(level, message);
    }
}

/// Mark a span as error
pub fn mark_span_error(span_id: &str, error_message: String) {
    let mut tracer = GLOBAL_TRACER.lock().unwrap();
    if let Some(span) = tracer.get_span_mut(span_id) {
        span.set_tag("error".to_string(), "true".to_string());
        span.set_tag("error.message".to_string(), error_message.clone());
        span.log("error".to_string(), error_message);
    }
}

/// Macro for easy tracing
#[macro_export]
macro_rules! trace {
    ($operation:expr, $code:block) => {{
        $crate::monitoring::tracing::trace_async($operation, |_span_id| async move $code).await
    }};
}

/// Macro for tracing with span access
#[macro_export]
macro_rules! trace_with_span {
    ($operation:expr, |$span_id:ident| $code:block) => {{
        $crate::monitoring::tracing::trace_async($operation, |$span_id| async move $code).await
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_creation() {
        let span = Span::new_root("test_operation".to_string());
        assert!(!span.trace_id.is_empty());
        assert!(!span.span_id.is_empty());
        assert!(span.parent_span_id.is_none());
        assert_eq!(span.operation_name, "test_operation");
        assert_eq!(span.status, SpanStatus::Ok);
    }

    #[test]
    fn test_child_span() {
        let parent = Span::new_root("parent_operation".to_string());
        let child = parent.new_child("child_operation".to_string());
        
        assert_eq!(child.trace_id, parent.trace_id);
        assert_ne!(child.span_id, parent.span_id);
        assert_eq!(child.parent_span_id, Some(parent.span_id));
    }

    #[test]
    fn test_span_tags_and_logs() {
        let mut span = Span::new_root("test_operation".to_string());
        
        span.set_tag("key1".to_string(), "value1".to_string());
        span.log("info".to_string(), "Test log message".to_string());
        
        assert_eq!(span.tags.get("key1"), Some(&"value1".to_string()));
        assert_eq!(span.logs.len(), 1);
        assert_eq!(span.logs[0].message, "Test log message");
    }

    #[test]
    fn test_span_error() {
        let mut span = Span::new_root("test_operation".to_string());
        span.finish_with_error("Test error".to_string());
        
        assert_eq!(span.status, SpanStatus::Error);
        assert_eq!(span.tags.get("error"), Some(&"true".to_string()));
        assert_eq!(span.tags.get("error.message"), Some(&"Test error".to_string()));
        assert!(span.end_time.is_some());
    }

    #[tokio::test]
    async fn test_trace_macro() {
        let result = trace!("test_operation", {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            42
        });
        
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_trace_with_span_macro() {
        let result = trace_with_span!("test_operation", |span_id| {
            add_span_tag(&span_id, "test_tag".to_string(), "test_value".to_string());
            add_span_log(&span_id, "info".to_string(), "Test log from macro".to_string());
            "success"
        });
        
        assert_eq!(result, "success");
    }
}