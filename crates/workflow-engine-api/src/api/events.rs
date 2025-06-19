//! Endpoint implementations for the API.
//!
//! This module contains the actual implementations of the API endpoints,
//! including request handlers and their supporting functions.

use std::sync::Arc;

use actix_web::{Error, HttpResponse, post, web};
use serde_json::Value;
use uuid::Uuid;
use chrono::Utc;

use crate::db::{
    event::{Event, NewEvent},
    session::DbPool,
    events::{EventDispatcher, EventEnvelope, EventMetadata},
};

/// Creates a new event in the system.
///
/// This endpoint accepts event data via POST request and stores it in the database.
/// The event is associated with a workflow type (currently hardcoded to "customer_care")
/// and prepared for future processing.
///
/// # Arguments
///
/// * `db_pool` - Database connection pool wrapped in web::Data and Arc
/// * `event` - JSON payload containing the event data
///
/// # Examples
///
/// ```rust,no_run
/// use actix_web::web;
/// use serde_json::json;
/// use backend::db::event::Event;
///
/// async fn example_request(client: &awc::Client) {
///     let event = Event {
///         id: None, // ID will be generated
///         data: json!({
///             "customer_id": "12345",
///             "message": "Need assistance"
///         }),
///         created_at: None, // Will be set by database
///         workflow_type: None, // Will be set by handler
///         workflow_state: None, // Will be set by handler
///     };
///
///     let response = client
///         .post("http://localhost:8080/events")
///         .send_json(&event)
///         .await;
/// }
/// ```
///
/// # Returns
///
/// Returns an `HttpResponse` with:
/// * 200 OK on successful event creation
/// * 500 Internal Server Error if database operations fail
#[post("/events")]
pub async fn create_event(
    db_pool: web::Data<Arc<DbPool>>,
    event_dispatcher: Option<web::Data<Arc<EventDispatcher>>>,
    event: web::Json<Event>,
) -> Result<HttpResponse, Error> {
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            log::error!("Failed to get database connection: {}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database connection failed",
                "message": "Unable to connect to database"
            })));
        }
    };
    
    let data = event.into_inner().data.clone();
    let event = NewEvent::new(data.clone(), get_workflow_type(), Value::Null);

    // Store event in database - NewEvent::new returns an Event struct
    let mut stored_event = event;
    if let Err(e) = stored_event.store(&mut conn) {
        log::error!("Failed to store event in database: {}", e);
        return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Database storage failed",
            "message": format!("Failed to store event: {}", e)
        })));
    }

    // Queue event for processing via event dispatcher
    if let Some(dispatcher) = event_dispatcher {
        match queue_event_for_processing(stored_event.clone(), &dispatcher).await {
            Ok(event_id) => {
                log::info!("Event {} queued for processing successfully", event_id);
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "message": "Event stored and queued for processing",
                    "event_id": event_id,
                    "status": "queued"
                })))
            },
            Err(e) => {
                log::error!("Failed to queue event for processing: {}", e);
                // Event is stored but not queued - still return success but note the issue
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "message": "Event stored but failed to queue for processing",
                    "event_id": stored_event.id,
                    "status": "stored_not_queued",
                    "error": e.to_string()
                })))
            }
        }
    } else {
        // No event dispatcher available - this is okay for basic storage
        log::warn!("No event dispatcher available, event stored but not queued");
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "Event stored successfully (no dispatcher configured)",
            "event_id": stored_event.id,
            "status": "stored_only"
        })))
    }
}

/// Queue an event for asynchronous processing through the event dispatcher
async fn queue_event_for_processing(
    stored_event: crate::db::event::Event,
    dispatcher: &EventDispatcher,
) -> Result<Uuid, String> {
    // Convert the stored database event to an EventEnvelope for the event dispatcher
    let event_envelope = EventEnvelope {
        event_id: Uuid::new_v4(),
        aggregate_id: stored_event.id,
        aggregate_type: "api_event".to_string(),
        event_type: format!("{}_created", stored_event.workflow_type),
        aggregate_version: 1,
        event_data: stored_event.data,
        metadata: EventMetadata::new()
            .with_source("api_events".to_string())
            .with_correlation_id(Uuid::new_v4()),
        occurred_at: stored_event.created_at,
        recorded_at: Utc::now(),
        schema_version: 1,
        causation_id: None,
        correlation_id: Some(Uuid::new_v4()),
        checksum: None,
    };

    // Dispatch the event for processing
    match dispatcher.dispatch(&event_envelope).await {
        Ok(()) => {
            Ok(event_envelope.event_id)
        },
        Err(e) => {
            Err(format!("Failed to dispatch event: {}", e))
        }
    }
}

/// Returns the workflow type for new events.
///
/// Currently returns a hardcoded value of "customer_care".
/// This function is intended to be expanded in the future to determine
/// workflow type based on event characteristics or configuration.
///
/// # Returns
///
/// Returns a `String` containing the workflow type identifier.
fn get_workflow_type() -> String {
    "customer_care".to_string()
}
