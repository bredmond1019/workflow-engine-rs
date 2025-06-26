//! REST API endpoints for content processing

use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::{ContentType, ProcessingOptions, ProcessingContext, DefaultContentProcessor, ProcessingPriority, ProcessingResult};
use crate::traits::ContentProcessor;

#[derive(Deserialize, serde::Serialize)]
pub struct ProcessRequest {
    pub content: String,
    pub content_type: ContentType,
    pub options: ProcessingOptions,
}

pub async fn process_content(
    req: HttpRequest,
    payload: web::Json<ProcessRequest>,
) -> actix_web::Result<HttpResponse> {
    // Create processor instance
    let processor = DefaultContentProcessor::new();
    
    // Validate input
    let content_bytes = payload.content.as_bytes();
    if let Err(validation_error) = processor.validate_input(content_bytes, &payload.content_type) {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "validation_failed",
            "message": validation_error.to_string()
        })));
    }
    
    // Extract user ID from auth headers
    let user_id = extract_user_id_from_auth(&req)
        .and_then(|id| Uuid::parse_str(&id).ok());
    
    // Create processing context
    let context = ProcessingContext {
        job_id: Uuid::new_v4(),
        correlation_id: Uuid::new_v4(),
        user_id,
        webhook_url: None,
        priority: ProcessingPriority::Normal,
        metadata: std::collections::HashMap::new(),
        session_id: None,
        processing_started_at: Some(chrono::Utc::now()),
        max_memory_mb: None,
        retry_count: 0,
        custom_data: None,
    };
    
    // Process content
    match processor.process(content_bytes, payload.content_type.clone(), payload.options.clone(), &context).await {
        Ok(ProcessingResult::Success(output)) => {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "status": "completed",
                "job_id": context.job_id,
                "result": output
            })))
        },
        Ok(ProcessingResult::Error(error)) => {
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "failed",
                "job_id": context.job_id,
                "error": error.to_string()
            })))
        },
        Ok(ProcessingResult::Partial(output, errors)) => {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "status": "partial",
                "job_id": context.job_id,
                "result": output,
                "errors": errors.iter().map(|e| e.to_string()).collect::<Vec<_>>()
            })))
        },
        Err(error) => {
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "failed",
                "job_id": context.job_id,
                "error": error.to_string()
            })))
        }
    }
}

/// Extract user ID from authentication headers
fn extract_user_id_from_auth(req: &HttpRequest) -> Option<String> {
    // Try to extract from Authorization header with Bearer token
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                
                // For a production system, you would decode and validate the JWT token here
                // For now, we'll do a simple extraction assuming a specific token format
                if let Some(user_id) = extract_user_id_from_jwt(token) {
                    return Some(user_id);
                }
            }
        }
    }
    
    // Try to extract from X-User-ID header as fallback
    if let Some(user_header) = req.headers().get("X-User-ID") {
        if let Ok(user_id) = user_header.to_str() {
            return Some(user_id.to_string());
        }
    }
    
    // Try to extract from custom auth header
    if let Some(custom_header) = req.headers().get("X-Auth-User") {
        if let Ok(user_id) = custom_header.to_str() {
            return Some(user_id.to_string());
        }
    }
    
    None
}

/// Extract user ID from JWT token (simplified implementation)
fn extract_user_id_from_jwt(token: &str) -> Option<String> {
    // In a real implementation, you would:
    // 1. Validate the JWT signature
    // 2. Check expiration
    // 3. Decode the payload
    // 4. Extract the user ID from claims
    
    // For now, we'll do a simple base64 decode assuming the payload is in the middle part
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    
    // Try to decode the payload (second part)
    use base64::{Engine as _, engine::general_purpose};
    if let Ok(payload_bytes) = general_purpose::URL_SAFE_NO_PAD.decode(parts[1]) {
        if let Ok(payload_str) = String::from_utf8(payload_bytes) {
            if let Ok(payload_json) = serde_json::from_str::<serde_json::Value>(&payload_str) {
                // Look for common user ID fields
                if let Some(user_id) = payload_json.get("sub").and_then(|v| v.as_str()) {
                    return Some(user_id.to_string());
                }
                if let Some(user_id) = payload_json.get("user_id").and_then(|v| v.as_str()) {
                    return Some(user_id.to_string());
                }
                if let Some(user_id) = payload_json.get("uid").and_then(|v| v.as_str()) {
                    return Some(user_id.to_string());
                }
            }
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};
    use std::collections::HashMap;
    
    #[actix_web::test]
    async fn test_process_content_endpoint() {
        let app = test::init_service(
            App::new()
                .route("/process", web::post().to(process_content))
        ).await;
        
        let request_body = ProcessRequest {
            content: "Test content".to_string(),
            content_type: ContentType::PlainText,
            options: ProcessingOptions::default(),
        };
        
        let req = test::TestRequest::post()
            .uri("/process")
            .set_json(&request_body)
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["status"], "completed");
    }
    
    #[actix_web::test]
    async fn test_process_content_with_html() {
        let app = test::init_service(
            App::new()
                .route("/process", web::post().to(process_content))
        ).await;
        
        let request_body = ProcessRequest {
            content: "<html><body>Test HTML</body></html>".to_string(),
            content_type: ContentType::Html,
            options: ProcessingOptions {
                extract_concepts: true,
                assess_quality: false,
                analyze_difficulty: true,
                extract_objectives: false,
                generate_summary: true,
                extract_keywords: true,
                detect_language: true,
                plugins: vec![],
                timeout_seconds: Some(30),
                plugin_params: HashMap::new(),
                verbose_logging: false,
            },
        };
        
        let req = test::TestRequest::post()
            .uri("/process")
            .set_json(&request_body)
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
    
    #[actix_web::test]
    async fn test_process_content_with_markdown() {
        let app = test::init_service(
            App::new()
                .route("/process", web::post().to(process_content))
        ).await;
        
        let request_body = ProcessRequest {
            content: "# Heading\n\nThis is **markdown** content.".to_string(),
            content_type: ContentType::Markdown,
            options: ProcessingOptions::default(),
        };
        
        let req = test::TestRequest::post()
            .uri("/process")
            .set_json(&request_body)
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }
    
    #[actix_web::test]
    async fn test_process_content_with_plugins() {
        let app = test::init_service(
            App::new()
                .route("/process", web::post().to(process_content))
        ).await;
        
        let mut plugin_params = HashMap::new();
        plugin_params.insert("sentiment_plugin".to_string(), serde_json::json!({
            "model": "bert-base",
            "threshold": 0.8
        }));
        
        let request_body = ProcessRequest {
            content: "This is amazing content!".to_string(),
            content_type: ContentType::PlainText,
            options: ProcessingOptions {
                extract_concepts: true,
                assess_quality: true,
                analyze_difficulty: true,
                extract_objectives: true,
                generate_summary: true,
                extract_keywords: true,
                detect_language: true,
                plugins: vec!["sentiment_plugin".to_string()],
                timeout_seconds: Some(60),
                plugin_params,
                verbose_logging: true,
            },
        };
        
        let req = test::TestRequest::post()
            .uri("/process")
            .set_json(&request_body)
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
    
    #[tokio::test]
    async fn test_process_request_deserialization() {
        let json = r#"{
            "content": "Test content",
            "content_type": "PlainText",
            "options": {
                "extract_concepts": true,
                "assess_quality": false,
                "analyze_difficulty": true,
                "extract_objectives": false,
                "generate_summary": true,
                "extract_keywords": true,
                "detect_language": true,
                "plugins": ["test_plugin"],
                "timeout_seconds": 30,
                "plugin_params": {},
                "verbose_logging": false
            }
        }"#;
        
        let request: Result<ProcessRequest, _> = serde_json::from_str(json);
        assert!(request.is_ok());
        
        let req = request.unwrap();
        assert_eq!(req.content, "Test content");
        assert_eq!(req.content_type, ContentType::PlainText);
        assert!(req.options.extract_concepts);
        assert!(!req.options.assess_quality);
        assert_eq!(req.options.plugins.len(), 1);
        assert_eq!(req.options.plugins[0], "test_plugin");
    }
    
    #[tokio::test]
    async fn test_content_type_serialization() {
        let test_cases = vec![
            (ContentType::Html, "\"Html\""),
            (ContentType::Pdf, "\"Pdf\""),
            (ContentType::Markdown, "\"Markdown\""),
            (ContentType::Video, "\"Video\""),
            (ContentType::Code, "\"Code\""),
            (ContentType::PlainText, "\"PlainText\""),
            (ContentType::Json, "\"Json\""),
            (ContentType::Xml, "\"Xml\""),
        ];
        
        for (content_type, expected) in test_cases {
            let serialized = serde_json::to_string(&content_type).unwrap();
            assert_eq!(serialized, expected);
        }
    }
}