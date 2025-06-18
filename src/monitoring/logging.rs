/*!
# Structured Logging with Correlation IDs

This module provides structured logging capabilities with correlation ID support
for the AI Workflow System, ensuring consistent log formatting and request tracking.

Task 3.4: Configure structured logging with correlation IDs
*/

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{Level, Metadata, Subscriber};
use tracing_subscriber::{
    EnvFilter, Layer,
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use uuid::Uuid;

/// Structured log entry format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredLogEntry {
    /// Timestamp of the log entry
    pub timestamp: DateTime<Utc>,

    /// Log level (INFO, WARN, ERROR, etc.)
    pub level: String,

    /// Correlation ID for request tracking
    pub correlation_id: Option<String>,

    /// Service name
    pub service: String,

    /// Module path where the log originated
    pub module: String,

    /// Log message
    pub message: String,

    /// Additional structured fields
    pub fields: HashMap<String, serde_json::Value>,

    /// Span ID if part of a trace
    pub span_id: Option<String>,

    /// Trace ID if part of a distributed trace
    pub trace_id: Option<String>,
}

impl StructuredLogEntry {
    /// Create a new structured log entry
    pub fn new(level: Level, message: String, module: String) -> Self {
        Self {
            timestamp: Utc::now(),
            level: level.to_string(),
            correlation_id: crate::monitoring::correlation::get_correlation_id(),
            service: "ai-workflow-system".to_string(),
            module,
            message,
            fields: HashMap::new(),
            span_id: None,
            trace_id: None,
        }
    }

    /// Add a field to the log entry
    pub fn with_field(mut self, key: String, value: serde_json::Value) -> Self {
        self.fields.insert(key, value);
        self
    }

    /// Set span and trace IDs
    pub fn with_trace_info(mut self, span_id: String, trace_id: String) -> Self {
        self.span_id = Some(span_id);
        self.trace_id = Some(trace_id);
        self
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> String {
        serde_json::to_string(self)
            .unwrap_or_else(|_| format!("[{}] {} - {}", self.level, self.module, self.message))
    }
}

/// Custom log formatter that includes correlation IDs
pub struct CorrelationIdFormatter;

impl<S> Layer<S> for CorrelationIdFormatter
where
    S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_event(&self, event: &tracing::Event<'_>, ctx: tracing_subscriber::layer::Context<'_, S>) {
        let metadata = event.metadata();
        let level = metadata.level();
        let module = metadata.module_path().unwrap_or("unknown");

        // Extract message from event
        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);
        let message = visitor.message.unwrap_or_else(|| "".to_string());

        // Create structured log entry
        let mut log_entry = StructuredLogEntry::new(level.clone(), message, module.to_string());

        // Add fields from event
        let mut field_visitor = FieldVisitor::default();
        event.record(&mut field_visitor);
        for (key, value) in field_visitor.fields {
            log_entry.fields.insert(key, value);
        }

        // Add span information if available
        if let Some(scope) = ctx.event_scope(event) {
            if let Some(span) = scope.from_root().next() {
                let extensions = span.extensions();
                if let Some(trace_id) = extensions.get::<String>() {
                    log_entry.trace_id = Some(trace_id.clone());
                }
                log_entry.span_id = Some(span.id().into_u64().to_string());
            }
        }

        // Output the structured log
        println!("{}", log_entry.to_json());
    }
}

/// Visitor to extract message from event
#[derive(Default)]
struct MessageVisitor {
    message: Option<String>,
}

impl tracing::field::Visit for MessageVisitor {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = Some(value.to_string());
        }
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = Some(format!("{:?}", value));
        }
    }
}

/// Visitor to extract all fields from event
#[derive(Default)]
struct FieldVisitor {
    fields: HashMap<String, serde_json::Value>,
}

impl tracing::field::Visit for FieldVisitor {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() != "message" {
            self.fields.insert(
                field.name().to_string(),
                serde_json::Value::String(value.to_string()),
            );
        }
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.fields.insert(
            field.name().to_string(),
            serde_json::Value::Number(value.into()),
        );
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.fields.insert(
            field.name().to_string(),
            serde_json::Value::Number(value.into()),
        );
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.fields
            .insert(field.name().to_string(), serde_json::Value::Bool(value));
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() != "message" {
            self.fields.insert(
                field.name().to_string(),
                serde_json::Value::String(format!("{:?}", value)),
            );
        }
    }
}

/// Initialize structured logging with correlation ID support
pub fn init_structured_logging() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let formatting_layer = fmt::layer()
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_span_events(FmtSpan::CLOSE)
        .json();

    let correlation_layer = CorrelationIdFormatter;

    tracing_subscriber::registry()
        .with(env_filter)
        .with(formatting_layer)
        .with(correlation_layer)
        .init();
}

/// Log with correlation ID
#[macro_export]
macro_rules! log_with_correlation {
    ($level:expr, $($arg:tt)*) => {
        if let Some(correlation_id) = $crate::monitoring::correlation::get_correlation_id() {
            match $level {
                tracing::Level::ERROR => tracing::error!(correlation_id = %correlation_id, $($arg)*),
                tracing::Level::WARN => tracing::warn!(correlation_id = %correlation_id, $($arg)*),
                tracing::Level::INFO => tracing::info!(correlation_id = %correlation_id, $($arg)*),
                tracing::Level::DEBUG => tracing::debug!(correlation_id = %correlation_id, $($arg)*),
                tracing::Level::TRACE => tracing::trace!(correlation_id = %correlation_id, $($arg)*),
            }
        } else {
            match $level {
                tracing::Level::ERROR => tracing::error!($($arg)*),
                tracing::Level::WARN => tracing::warn!($($arg)*),
                tracing::Level::INFO => tracing::info!($($arg)*),
                tracing::Level::DEBUG => tracing::debug!($($arg)*),
                tracing::Level::TRACE => tracing::trace!($($arg)*),
            }
        }
    };
}

/// Convenience macros for different log levels
#[macro_export]
macro_rules! error_with_correlation {
    ($($arg:tt)*) => {
        $crate::log_with_correlation!(tracing::Level::ERROR, $($arg)*)
    };
}

#[macro_export]
macro_rules! warn_with_correlation {
    ($($arg:tt)*) => {
        $crate::log_with_correlation!(tracing::Level::WARN, $($arg)*)
    };
}

#[macro_export]
macro_rules! info_with_correlation {
    ($($arg:tt)*) => {
        $crate::log_with_correlation!(tracing::Level::INFO, $($arg)*)
    };
}

#[macro_export]
macro_rules! debug_with_correlation {
    ($($arg:tt)*) => {
        $crate::log_with_correlation!(tracing::Level::DEBUG, $($arg)*)
    };
}

#[macro_export]
macro_rules! trace_with_correlation {
    ($($arg:tt)*) => {
        $crate::log_with_correlation!(tracing::Level::TRACE, $($arg)*)
    };
}

/// Helper function to create structured log fields
pub fn log_fields(pairs: &[(&str, &dyn std::fmt::Display)]) -> HashMap<String, serde_json::Value> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), serde_json::Value::String(v.to_string())))
        .collect()
}

/// Log HTTP request details with correlation ID
pub fn log_http_request(
    method: &str,
    path: &str,
    status: u16,
    duration_ms: u64,
    correlation_id: Option<&str>,
) {
    let fields = log_fields(&[
        ("method", &method),
        ("path", &path),
        ("status", &status.to_string()),
        ("duration_ms", &duration_ms.to_string()),
    ]);

    if let Some(id) = correlation_id {
        tracing::info!(
            correlation_id = %id,
            method = %method,
            path = %path,
            status = %status,
            duration_ms = %duration_ms,
            "HTTP request completed"
        );
    } else {
        tracing::info!(
            method = %method,
            path = %path,
            status = %status,
            duration_ms = %duration_ms,
            "HTTP request completed"
        );
    }
}

/// Log workflow execution details
pub fn log_workflow_event(
    workflow_id: &str,
    event_type: &str,
    node_id: Option<&str>,
    details: HashMap<String, String>,
    correlation_id: Option<&str>,
) {
    let mut fields = details;
    fields.insert("workflow_id".to_string(), workflow_id.to_string());
    fields.insert("event_type".to_string(), event_type.to_string());

    if let Some(node) = node_id {
        fields.insert("node_id".to_string(), node.to_string());
    }

    if let Some(id) = correlation_id {
        fields.insert("correlation_id".to_string(), id.to_string());
    }

    match event_type {
        "error" => tracing::error!(?fields, "Workflow error"),
        "warning" => tracing::warn!(?fields, "Workflow warning"),
        _ => tracing::info!(?fields, "Workflow event"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_structured_log_entry() {
        let entry = StructuredLogEntry::new(
            Level::INFO,
            "Test message".to_string(),
            "test_module".to_string(),
        );

        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.message, "Test message");
        assert_eq!(entry.module, "test_module");
        assert_eq!(entry.service, "ai-workflow-system");
    }

    #[test]
    fn test_log_entry_with_fields() {
        let entry = StructuredLogEntry::new(
            Level::INFO,
            "Test message".to_string(),
            "test_module".to_string(),
        )
        .with_field("user_id".to_string(), serde_json::json!("12345"))
        .with_field("action".to_string(), serde_json::json!("login"));

        assert_eq!(entry.fields.len(), 2);
        assert_eq!(
            entry.fields.get("user_id"),
            Some(&serde_json::json!("12345"))
        );
        assert_eq!(
            entry.fields.get("action"),
            Some(&serde_json::json!("login"))
        );
    }

    #[test]
    fn test_log_entry_json_serialization() {
        let mut entry = StructuredLogEntry::new(
            Level::ERROR,
            "Test error".to_string(),
            "test_module".to_string(),
        );
        entry.correlation_id = Some("test-correlation-123".to_string());

        let json = entry.to_json();
        assert!(json.contains("\"level\":\"ERROR\""));
        assert!(json.contains("\"message\":\"Test error\""));
        assert!(json.contains("\"correlation_id\":\"test-correlation-123\""));
    }

    #[test]
    fn test_log_fields_helper() {
        let fields = log_fields(&[("key1", &"value1"), ("key2", &42)]);

        assert_eq!(fields.len(), 2);
        assert_eq!(fields.get("key1"), Some(&serde_json::json!("value1")));
        assert_eq!(fields.get("key2"), Some(&serde_json::json!("42")));
    }
}
