/*!
# Correlation ID Management

This module provides correlation ID generation and propagation for
tracking requests across system boundaries.

Task 3.3: Add correlation ID propagation in Workflow System (Rust)
*/

use actix_web::{
    Error, HttpMessage, Result,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::header::HeaderValue,
};
use futures_util::future::{Ready, ok};
use std::cell::RefCell;
use std::rc::Rc;
use std::task::{Context, Poll};
use uuid::Uuid;

const CORRELATION_ID_HEADER: &str = "X-Correlation-ID";

/// Correlation ID middleware that ensures every request has a correlation ID
pub struct CorrelationIdMiddleware;

impl<S, B> Transform<S, ServiceRequest> for CorrelationIdMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = CorrelationIdService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CorrelationIdService {
            service: Rc::new(RefCell::new(service)),
        })
    }
}

pub struct CorrelationIdService<S> {
    service: Rc<RefCell<S>>,
}

impl<S, B> Service<ServiceRequest> for CorrelationIdService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();

        Box::pin(async move {
            // Get or generate correlation ID
            let correlation_id = req
                .headers()
                .get(CORRELATION_ID_HEADER)
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string())
                .unwrap_or_else(|| Uuid::new_v4().to_string());

            // Store correlation ID in request extensions for later use
            req.extensions_mut()
                .insert(CorrelationId(correlation_id.clone()));

            // Call the next service
            let mut res = service.borrow_mut().call(req).await?;

            // Add correlation ID to response headers
            res.headers_mut().insert(
                actix_web::http::header::HeaderName::from_static("x-correlation-id"),
                HeaderValue::from_str(&correlation_id).unwrap(),
            );

            Ok(res)
        })
    }
}

use std::pin::Pin;

/// Wrapper for correlation ID
#[derive(Debug, Clone)]
pub struct CorrelationId(pub String);

impl CorrelationId {
    /// Get the correlation ID value
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the correlation ID value as owned string
    pub fn into_string(self) -> String {
        self.0
    }
}

/// Helper function to extract correlation ID from Actix request
pub fn extract_correlation_id(req: &ServiceRequest) -> Option<String> {
    req.extensions().get::<CorrelationId>().map(|c| c.0.clone())
}

/// Structured logging helper that includes correlation ID
pub struct CorrelatedLogger;

impl CorrelatedLogger {
    /// Log with correlation ID context
    pub fn log(
        level: log::Level,
        correlation_id: Option<&str>,
        message: &str,
        module: &str,
        line: u32,
    ) {
        if let Some(id) = correlation_id {
            log::logger().log(
                &log::Record::builder()
                    .args(format_args!("[{}] {}", id, message))
                    .level(level)
                    .target(module)
                    .file(Some(file!()))
                    .line(Some(line))
                    .module_path(Some(module))
                    .build(),
            );
        } else {
            log::logger().log(
                &log::Record::builder()
                    .args(format_args!("{}", message))
                    .level(level)
                    .target(module)
                    .file(Some(file!()))
                    .line(Some(line))
                    .module_path(Some(module))
                    .build(),
            );
        }
    }

    /// Log info with correlation ID
    pub fn info(correlation_id: Option<&str>, message: &str) {
        Self::log(
            log::Level::Info,
            correlation_id,
            message,
            module_path!(),
            line!(),
        );
    }

    /// Log error with correlation ID
    pub fn error(correlation_id: Option<&str>, message: &str) {
        Self::log(
            log::Level::Error,
            correlation_id,
            message,
            module_path!(),
            line!(),
        );
    }

    /// Log debug with correlation ID
    pub fn debug(correlation_id: Option<&str>, message: &str) {
        Self::log(
            log::Level::Debug,
            correlation_id,
            message,
            module_path!(),
            line!(),
        );
    }

    /// Log warn with correlation ID
    pub fn warn(correlation_id: Option<&str>, message: &str) {
        Self::log(
            log::Level::Warn,
            correlation_id,
            message,
            module_path!(),
            line!(),
        );
    }
}

/// Macro for easier correlated logging
#[macro_export]
macro_rules! log_with_correlation_id {
    ($level:ident, $correlation_id:expr, $($arg:tt)*) => {
        $crate::monitoring::correlation::CorrelatedLogger::$level(
            $correlation_id.as_ref().map(|s| s.as_str()),
            &format!($($arg)*)
        )
    };
}

/// Context holder for correlation ID in async contexts
#[derive(Debug, Clone)]
pub struct CorrelationContext {
    correlation_id: String,
}

impl CorrelationContext {
    /// Create new correlation context
    pub fn new() -> Self {
        Self {
            correlation_id: Uuid::new_v4().to_string(),
        }
    }

    /// Create correlation context with specific ID
    pub fn with_id(correlation_id: String) -> Self {
        Self { correlation_id }
    }

    /// Get correlation ID
    pub fn correlation_id(&self) -> &str {
        &self.correlation_id
    }

    /// Create HTTP headers with correlation ID
    pub fn to_headers(&self) -> std::collections::HashMap<String, String> {
        let mut headers = std::collections::HashMap::new();
        headers.insert(
            CORRELATION_ID_HEADER.to_string(),
            self.correlation_id.clone(),
        );
        headers
    }

    /// Add correlation ID to reqwest RequestBuilder
    pub fn add_to_request(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        request.header(CORRELATION_ID_HEADER, &self.correlation_id)
    }
}

impl Default for CorrelationContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract correlation ID from HTTP headers
pub fn extract_correlation_id_from_headers(headers: &reqwest::header::HeaderMap) -> Option<String> {
    headers
        .get(CORRELATION_ID_HEADER)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
}

/// Helper trait for adding correlation context to cross-system calls
pub trait WithCorrelation {
    fn with_correlation(self, context: &CorrelationContext) -> Self;
}

impl WithCorrelation for reqwest::RequestBuilder {
    fn with_correlation(self, context: &CorrelationContext) -> Self {
        context.add_to_request(self)
    }
}

use std::sync::Arc;
use std::sync::RwLock;

// Global correlation ID storage
static GLOBAL_CORRELATION_ID: RwLock<Option<String>> = RwLock::new(None);

/// Set the correlation ID for the current task
pub fn set_correlation_id(correlation_id: Option<String>) {
    if let Ok(mut guard) = GLOBAL_CORRELATION_ID.write() {
        *guard = correlation_id;
    }
}

/// Get the correlation ID for the current task
pub fn get_correlation_id() -> Option<String> {
    GLOBAL_CORRELATION_ID
        .read()
        .ok()
        .and_then(|guard| guard.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, HttpResponse, test, web};

    async fn test_handler(req: actix_web::HttpRequest) -> HttpResponse {
        let correlation_id = req
            .extensions()
            .get::<CorrelationId>()
            .map(|c| c.0.clone())
            .unwrap_or_else(|| "none".to_string());

        HttpResponse::Ok().json(serde_json::json!({
            "correlation_id": correlation_id
        }))
    }

    #[actix_web::test]
    async fn test_correlation_id_middleware() {
        let app = test::init_service(
            App::new()
                .wrap(CorrelationIdMiddleware)
                .route("/test", web::get().to(test_handler)),
        )
        .await;

        // Test without correlation ID header
        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        let correlation_header = resp.headers().get("x-correlation-id");
        assert!(correlation_header.is_some());

        // Test with existing correlation ID header
        let test_correlation_id = "test-correlation-123";
        let req = test::TestRequest::get()
            .uri("/test")
            .insert_header((CORRELATION_ID_HEADER, test_correlation_id))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        let correlation_header = resp.headers().get("x-correlation-id");
        assert_eq!(
            correlation_header.unwrap().to_str().unwrap(),
            test_correlation_id
        );
    }

    #[test]
    async fn test_correlation_context() {
        let context = CorrelationContext::new();
        assert!(!context.correlation_id().is_empty());

        let headers = context.to_headers();
        assert!(headers.contains_key(CORRELATION_ID_HEADER));
        assert_eq!(headers[CORRELATION_ID_HEADER], context.correlation_id());
    }

    #[test]
    async fn test_correlation_context_with_id() {
        let test_id = "test-123";
        let context = CorrelationContext::with_id(test_id.to_string());
        assert_eq!(context.correlation_id(), test_id);
    }
}
