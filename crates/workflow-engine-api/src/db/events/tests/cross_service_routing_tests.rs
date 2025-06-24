// File: src/db/events/tests/cross_service_routing_tests.rs
//
// Unit tests for cross-service event routing

use super::*;
use crate::db::events::cross_service_routing::*;
use crate::db::events::{EventEnvelope, EventMetadata};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Clone)]
struct MockCrossServiceEventHandler {
    service_name: String,
    received_events: Arc<RwLock<Vec<RoutedEvent>>>,
}

impl MockCrossServiceEventHandler {
    fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
            received_events: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl CrossServiceEventHandler for MockCrossServiceEventHandler {
    async fn handle_service_event(&self, event: &RoutedEvent) -> EventResult<()> {
        let mut events = self.received_events.write().await;
        events.push(event.clone());
        Ok(())
    }
    
    fn service_name(&self) -> &str {
        &self.service_name
    }
}

fn create_test_event(event_type: &str) -> EventEnvelope {
    EventEnvelope {
        event_id: Uuid::new_v4(),
        aggregate_id: Uuid::new_v4(),
        aggregate_type: "test_aggregate".to_string(),
        event_type: event_type.to_string(),
        aggregate_version: 1,
        event_data: serde_json::json!({"test": "data"}),
        metadata: EventMetadata {
            user_id: Some(Uuid::new_v4().to_string()),
            session_id: Some(Uuid::new_v4().to_string()),
            correlation_id: Some(Uuid::new_v4()),
            causation_id: Some(Uuid::new_v4()),
            source: Some("test".to_string()),
            tags: HashMap::new(),
            timestamp: chrono::Utc::now(),
            custom: Default::default(),
        },
        occurred_at: chrono::Utc::now(),
        recorded_at: chrono::Utc::now(),
        schema_version: 1,
        causation_id: None,
        correlation_id: None,
        checksum: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_routing_priority_ordering() {
        assert!(RoutingPriority::Critical > RoutingPriority::High);
        assert!(RoutingPriority::High > RoutingPriority::Normal);
        assert!(RoutingPriority::Normal > RoutingPriority::Low);
    }
    
    #[test]
    fn test_service_routing_config_defaults() {
        let config = ServiceRoutingConfig::default();
        
        assert_eq!(config.broadcast_channel, "events:broadcast");
        assert!(config.enable_deduplication);
        assert_eq!(config.deduplication_window_seconds, 300);
        assert_eq!(config.max_delivery_attempts, 3);
        assert!(config.enable_ordering);
    }
    
    #[tokio::test]
    async fn test_routing_metadata_creation() {
        let event = create_test_event("test_event");
        
        let metadata = RoutingMetadata {
            source_service: "service_a".to_string(),
            target_services: vec!["service_b".to_string(), "service_c".to_string()],
            routing_key: "test_aggregate:test_event".to_string(),
            priority: RoutingPriority::Normal,
            delivery_attempts: 0,
            sequence_number: Some(1),
            partition_key: Some(event.aggregate_id.to_string()),
        };
        
        assert_eq!(metadata.source_service, "service_a");
        assert_eq!(metadata.target_services.len(), 2);
        assert_eq!(metadata.priority, RoutingPriority::Normal);
    }
    
    #[test]
    fn test_routed_event_serialization() {
        let event = create_test_event("test_event");
        let metadata = RoutingMetadata {
            source_service: "service_a".to_string(),
            target_services: vec!["service_b".to_string()],
            routing_key: "test_key".to_string(),
            priority: RoutingPriority::High,
            delivery_attempts: 1,
            sequence_number: Some(42),
            partition_key: None,
        };
        
        let routed_event = RoutedEvent {
            event: event.clone(),
            routing_metadata: metadata,
            routed_at: chrono::Utc::now(),
        };
        
        // Test serialization
        let serialized = serde_json::to_string(&routed_event).unwrap();
        let deserialized: RoutedEvent = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(deserialized.event.event_id, event.event_id);
        assert_eq!(deserialized.routing_metadata.source_service, "service_a");
        assert_eq!(deserialized.routing_metadata.sequence_number, Some(42));
    }
    
    #[test]
    fn test_broadcast_event_creation() {
        let event_id = Uuid::new_v4();
        let broadcast_event = BroadcastEvent {
            event_id,
            event_type: "user_created".to_string(),
            source_service: "auth_service".to_string(),
            target_services: vec!["profile_service".to_string(), "notification_service".to_string()],
            timestamp: chrono::Utc::now(),
        };
        
        assert_eq!(broadcast_event.event_type, "user_created");
        assert_eq!(broadcast_event.target_services.len(), 2);
    }
    
    #[test]
    fn test_routing_statistics() {
        let stats = RoutingStatistics {
            total_events_routed: 1000,
            duplicate_events_detected: 50,
            failed_deliveries: 5,
            active_routes: 10,
            current_sequence_number: 1050,
            active_services: 4,
        };
        
        assert_eq!(stats.total_events_routed, 1000);
        assert_eq!(stats.duplicate_events_detected, 50);
        assert_eq!(stats.failed_deliveries, 5);
    }
    
    #[tokio::test]
    async fn test_mock_handler() {
        let handler = MockCrossServiceEventHandler::new("test_service");
        let event = create_test_event("test_event");
        let routed_event = RoutedEvent {
            event,
            routing_metadata: RoutingMetadata {
                source_service: "source".to_string(),
                target_services: vec!["test_service".to_string()],
                routing_key: "test_key".to_string(),
                priority: RoutingPriority::Normal,
                delivery_attempts: 0,
                sequence_number: None,
                partition_key: None,
            },
            routed_at: chrono::Utc::now(),
        };
        
        // Handle the event
        handler.handle_service_event(&routed_event).await.unwrap();
        
        // Verify it was received
        let received = handler.received_events.read().await;
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].event.event_id, routed_event.event.event_id);
    }
}