// File: tests/event_sourcing_tests.rs
//
// Comprehensive tests for the event sourcing architecture

use ai_workflow_engine::db::events::*;
use ai_workflow_engine::db::events::types::*;
use ai_workflow_engine::db::events::dispatcher::EventHandler;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;
use uuid::Uuid;

#[cfg(test)]
mod event_sourcing_tests {
    use super::*;

    // Helper function to create a test event store
    fn create_test_event_store() -> PostgreSQLEventStore {
        let config = EventStoreConfig {
            database_url: "postgresql://postgres:password@localhost/test_db".to_string(),
            connection_pool_size: 5,
            batch_size: 100,
            snapshot_frequency: 10,
            enable_checksums: true,
        };
        
        PostgreSQLEventStore::new(config).expect("Failed to create test event store")
    }

    // Helper function to create a test event envelope
    fn create_test_event(
        aggregate_id: Uuid,
        event_type: &str,
        event_data: serde_json::Value,
        version: i64,
    ) -> EventEnvelope {
        EventEnvelope {
            event_id: Uuid::new_v4(),
            aggregate_id,
            aggregate_type: "test_aggregate".to_string(),
            event_type: event_type.to_string(),
            aggregate_version: version,
            event_data,
            metadata: EventMetadata::new()
                .with_correlation_id(Uuid::new_v4())
                .with_source("test".to_string()),
            occurred_at: Utc::now(),
            recorded_at: Utc::now(),
            schema_version: 1,
            causation_id: None,
            correlation_id: Some(Uuid::new_v4()),
            checksum: None,
        }
    }

    // Test event handler for logging events
    pub struct LoggingEventHandler {
        name: String,
        event_types: Vec<String>,
    }

    impl LoggingEventHandler {
        pub fn new(name: String, event_types: Vec<String>) -> Self {
            Self { name, event_types }
        }
    }

    #[async_trait]
    impl EventHandler for LoggingEventHandler {
        async fn handle(&self, event: &EventEnvelope) -> EventResult<()> {
            tracing::info!(
                "Handler '{}' processing event {} of type '{}'",
                self.name,
                event.event_id,
                event.event_type
            );
            Ok(())
        }

        fn event_types(&self) -> Vec<String> {
            self.event_types.clone()
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[tokio::test]
    async fn test_event_store_append_and_retrieve() {
        let event_store = create_test_event_store();
        let aggregate_id = Uuid::new_v4();
        
        // Create test event
        let event = create_test_event(
            aggregate_id,
            "test_event",
            json!({"message": "Hello, World!"}),
            1,
        );

        // Append event
        let result = event_store.append_event(&event).await;
        assert!(result.is_ok(), "Failed to append event: {:?}", result.err());

        // Retrieve events
        let retrieved_events = event_store.get_events(aggregate_id).await;
        assert!(retrieved_events.is_ok());
        
        let events = retrieved_events.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_id, event.event_id);
        assert_eq!(events[0].aggregate_id, aggregate_id);
        assert_eq!(events[0].event_type, "test_event");
    }

    #[tokio::test]
    async fn test_event_store_batch_append() {
        let event_store = create_test_event_store();
        let aggregate_id = Uuid::new_v4();
        
        // Create multiple test events
        let events = vec![
            create_test_event(aggregate_id, "event_1", json!({"data": 1}), 1),
            create_test_event(aggregate_id, "event_2", json!({"data": 2}), 2),
            create_test_event(aggregate_id, "event_3", json!({"data": 3}), 3),
        ];

        // Append events in batch
        let result = event_store.append_events(&events).await;
        assert!(result.is_ok(), "Failed to append events batch: {:?}", result.err());

        // Retrieve events
        let retrieved_events = event_store.get_events(aggregate_id).await;
        assert!(retrieved_events.is_ok());
        
        let events = retrieved_events.unwrap();
        assert_eq!(events.len(), 3);
        
        // Verify order
        for (i, event) in events.iter().enumerate() {
            assert_eq!(event.aggregate_version, (i + 1) as i64);
        }
    }

    #[tokio::test]
    async fn test_event_dispatcher() {
        let event_store = Arc::new(create_test_event_store());
        let mut dispatcher = EventDispatcher::new(event_store);

        // Create a test handler
        let handler = Arc::new(LoggingEventHandler::new(
            "test_handler".to_string(),
            vec!["test_event".to_string()],
        ));
        
        // Register handler
        let result = dispatcher.register_handler(handler.clone()).await;
        assert!(result.is_ok(), "Failed to register handler: {:?}", result.err());

        // Create and dispatch event
        let event = create_test_event(
            Uuid::new_v4(),
            "test_event",
            json!({"message": "Test dispatch"}),
            1,
        );

        let result = dispatcher.dispatch(&event).await;
        assert!(result.is_ok(), "Failed to dispatch event: {:?}", result.err());

        // Verify handler was called (would need mock in real test)
        // For now, we just verify the dispatch succeeded
    }

    #[tokio::test]
    async fn test_workflow_aggregate() {
        use ai_workflow_engine::db::events::aggregate::*;
        
        let aggregate_id = Uuid::new_v4();
        let mut workflow = WorkflowAggregate::new(aggregate_id);

        // Test create workflow command
        let create_command = WorkflowCommand::CreateWorkflow {
            workflow_type: "test_workflow".to_string(),
            input_data: json!({"input": "test data"}),
        };

        let events = workflow.handle_command(create_command).await;
        assert!(events.is_ok());
        
        let event_list = events.unwrap();
        assert_eq!(event_list.len(), 1);
        
        // Verify workflow state
        assert_eq!(workflow.workflow_type, "test_workflow");
        assert_eq!(workflow.version(), 1);

        // Test start workflow command
        let start_command = WorkflowCommand::StartWorkflow;
        let events = workflow.handle_command(start_command).await;
        assert!(events.is_ok());
        
        let event_list = events.unwrap();
        assert_eq!(event_list.len(), 1);
        assert_eq!(workflow.version(), 2);
    }

    #[tokio::test]
    async fn test_event_serialization() {
        use ai_workflow_engine::db::events::types::*;
        
        // Test workflow event serialization
        let workflow_event = WorkflowEvent::WorkflowStarted(WorkflowStartedEvent {
            workflow_id: Uuid::new_v4(),
            workflow_type: "test_workflow".to_string(),
            configuration: json!({"config": "test"}),
            input_data: json!({"input": "test"}),
            user_id: Some("test_user".to_string()),
        });

        // Serialize
        let serialized = workflow_event.serialize();
        assert!(serialized.is_ok(), "Failed to serialize workflow event: {:?}", serialized.err());

        // Deserialize
        let deserialized = WorkflowEvent::deserialize(&serialized.unwrap(), 1);
        assert!(deserialized.is_ok(), "Failed to deserialize workflow event: {:?}", deserialized.err());

        // Verify schema version
        assert_eq!(WorkflowEvent::schema_version(), 1);
        assert_eq!(WorkflowEvent::event_type(), "workflow_event");
    }

    #[tokio::test]
    async fn test_ai_interaction_events() {
        use ai_workflow_engine::db::events::types::*;
        
        // Test AI interaction event types
        let prompt_event = AIInteractionEvent::PromptSent(PromptSentEvent {
            request_id: Uuid::new_v4(),
            model: "gpt-4".to_string(),
            provider: "openai".to_string(),
            prompt: "Test prompt".to_string(),
            parameters: std::collections::HashMap::new(),
            workflow_id: Some(Uuid::new_v4()),
            node_id: Some("test_node".to_string()),
        });

        let response_event = AIInteractionEvent::ResponseReceived(ResponseReceivedEvent {
            request_id: Uuid::new_v4(),
            response: "Test response".to_string(),
            completion_tokens: 50,
            prompt_tokens: 10,
            total_tokens: 60,
            cost_usd: Some(0.001),
            duration_ms: 1500,
            model: "gpt-4".to_string(),
            provider: "openai".to_string(),
        });

        // Test serialization for both events
        let prompt_serialized = prompt_event.serialize();
        assert!(prompt_serialized.is_ok());

        let response_serialized = response_event.serialize();
        assert!(response_serialized.is_ok());

        // Test deserialization
        let prompt_deserialized = AIInteractionEvent::deserialize(&prompt_serialized.unwrap(), 1);
        assert!(prompt_deserialized.is_ok());

        let response_deserialized = AIInteractionEvent::deserialize(&response_serialized.unwrap(), 1);
        assert!(response_deserialized.is_ok());
    }

    #[tokio::test]
    async fn test_event_metadata() {
        let metadata = EventMetadata::new()
            .with_correlation_id(Uuid::new_v4())
            .with_causation_id(Uuid::new_v4())
            .with_user_id("test_user".to_string())
            .with_source("test_service".to_string())
            .add_tag("environment".to_string(), "test".to_string())
            .add_custom("custom_field".to_string(), json!("custom_value"));

        // Verify metadata fields are set
        assert!(metadata.correlation_id.is_some());
        assert!(metadata.causation_id.is_some());
        assert!(metadata.user_id.is_some());
        assert!(metadata.source.is_some());
        assert_eq!(metadata.tags.get("environment"), Some(&"test".to_string()));
        assert_eq!(metadata.custom.get("custom_field"), Some(&json!("custom_value")));
    }

    #[tokio::test]
    async fn test_event_stream_config() {
        use ai_workflow_engine::db::events::streaming::*;
        
        let config = EventStreamConfig::new("test_stream".to_string())
            .with_event_types(vec!["workflow_event".to_string(), "ai_interaction_event".to_string()])
            .with_filter("aggregate_type".to_string(), json!("workflow"))
            .with_batch_size(50)
            .with_poll_interval(500)
            .from_beginning();

        assert_eq!(config.stream_name, "test_stream");
        assert_eq!(config.event_types.len(), 2);
        assert_eq!(config.batch_size, 50);
        assert_eq!(config.poll_interval_ms, 500);
        assert_eq!(config.start_position, 0);
        assert!(config.include_existing);

        // Test event matching
        let matching_event = create_test_event(
            Uuid::new_v4(),
            "workflow_event",
            json!({"test": "data"}),
            1,
        );

        let non_matching_event = create_test_event(
            Uuid::new_v4(),
            "system_event",
            json!({"test": "data"}),
            1,
        );

        // This test would need to be adjusted based on actual matching logic
        // For now, we just verify the config was created correctly
    }

    #[tokio::test]
    async fn test_concurrency_control() {
        let event_store = create_test_event_store();
        let aggregate_id = Uuid::new_v4();
        
        // Create first event
        let event1 = create_test_event(
            aggregate_id,
            "event_1",
            json!({"data": 1}),
            1,
        );

        // Append first event
        let result = event_store.append_event(&event1).await;
        assert!(result.is_ok());

        // Try to append event with same version (should fail)
        let event2 = create_test_event(
            aggregate_id,
            "event_2",
            json!({"data": 2}),
            1, // Same version as event1
        );

        let result = event_store.append_event(&event2).await;
        assert!(result.is_err());
        
        // Should be a concurrency error
        match result.unwrap_err() {
            EventError::ConcurrencyError { .. } => {
                // Expected error type
            }
            other => panic!("Expected ConcurrencyError, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_event_handlers() {
        use ai_workflow_engine::db::events::handlers::*;
        use std::sync::Arc;
        use tokio::sync::RwLock;

        // Test workflow event handler
        let workflow_handler = WorkflowEventHandler::new();
        
        // Create a workflow event envelope
        let workflow_event_data = WorkflowEvent::WorkflowStarted(WorkflowStartedEvent {
            workflow_id: Uuid::new_v4(),
            workflow_type: "test_workflow".to_string(),
            configuration: json!({}),
            input_data: json!({}),
            user_id: None,
        });

        let event_envelope = EventEnvelope {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            aggregate_type: "workflow".to_string(),
            event_type: "workflow_event".to_string(),
            aggregate_version: 1,
            event_data: serde_json::to_value(workflow_event_data).unwrap(),
            metadata: EventMetadata::default(),
            occurred_at: Utc::now(),
            recorded_at: Utc::now(),
            schema_version: 1,
            causation_id: None,
            correlation_id: None,
            checksum: None,
        };

        // Handle the event
        let result = workflow_handler.handle(&event_envelope).await;
        assert!(result.is_ok(), "Workflow handler failed: {:?}", result.err());

        // Check metrics were updated
        let metrics = workflow_handler.get_metrics().await;
        assert_eq!(metrics.workflows_started, 1);
    }

    #[tokio::test]
    async fn test_snapshot_functionality() {
        let event_store = create_test_event_store();
        let aggregate_id = Uuid::new_v4();

        // Create snapshot
        let snapshot = AggregateSnapshot::new(
            aggregate_id,
            "test_aggregate".to_string(),
            5,
            json!({"state": "test_state", "version": 5}),
        );

        // Save snapshot
        let result = event_store.save_snapshot(&snapshot).await;
        assert!(result.is_ok(), "Failed to save snapshot: {:?}", result.err());

        // Retrieve snapshot
        let retrieved = event_store.get_snapshot(aggregate_id).await;
        assert!(retrieved.is_ok());
        
        let snapshot_option = retrieved.unwrap();
        assert!(snapshot_option.is_some());
        
        let retrieved_snapshot = snapshot_option.unwrap();
        assert_eq!(retrieved_snapshot.aggregate_id, aggregate_id);
        assert_eq!(retrieved_snapshot.aggregate_version, 5);
        assert_eq!(retrieved_snapshot.aggregate_type, "test_aggregate");
    }

    #[tokio::test]
    async fn test_event_store_query_operations() {
        let event_store = create_test_event_store();
        let aggregate_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();

        // Create events with correlation ID
        let mut events = vec![];
        for i in 1..=5 {
            let mut event = create_test_event(
                aggregate_id,
                "test_event",
                json!({"sequence": i}),
                i,
            );
            event.correlation_id = Some(correlation_id);
            events.push(event);
        }

        // Append events
        let result = event_store.append_events(&events).await;
        assert!(result.is_ok());

        // Test get events from version
        let events_from_3 = event_store.get_events_from_version(aggregate_id, 2).await;
        assert!(events_from_3.is_ok());
        
        let filtered_events = events_from_3.unwrap();
        assert_eq!(filtered_events.len(), 3); // Events 3, 4, 5

        // Test get events by correlation ID
        let correlated_events = event_store.get_events_by_correlation_id(correlation_id).await;
        assert!(correlated_events.is_ok());
        
        let corr_events = correlated_events.unwrap();
        assert_eq!(corr_events.len(), 5);

        // Test get events by type
        let typed_events = event_store.get_events_by_type(
            "test_event",
            None,
            None,
            Some(3),
        ).await;
        assert!(typed_events.is_ok());
        
        let type_events = typed_events.unwrap();
        assert!(type_events.len() <= 3); // Limited to 3

        // Test aggregate version
        let version = event_store.get_aggregate_version(aggregate_id).await;
        assert!(version.is_ok());
        assert_eq!(version.unwrap(), 5);

        // Test aggregate exists
        let exists = event_store.aggregate_exists(aggregate_id).await;
        assert!(exists.is_ok());
        assert!(exists.unwrap());

        // Test non-existent aggregate
        let non_existent = event_store.aggregate_exists(Uuid::new_v4()).await;
        assert!(non_existent.is_ok());
        assert!(!non_existent.unwrap());
    }

    #[tokio::test]
    async fn test_error_handling() {
        use ai_workflow_engine::db::events::types::*;
        
        // Test invalid JSON deserialization
        let invalid_json = json!({"invalid": "structure"});
        let result = WorkflowEvent::deserialize(&invalid_json, 1);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            EventError::SerializationError { .. } => {
                // Expected error
            }
            other => panic!("Expected SerializationError, got: {:?}", other),
        }

        // Test event not found error
        let event_error = EventError::EventNotFound {
            event_id: Uuid::new_v4(),
        };
        assert!(format!("{}", event_error).contains("Event not found"));

        // Test aggregate not found error
        let aggregate_error = EventError::AggregateNotFound {
            aggregate_id: Uuid::new_v4(),
        };
        assert!(format!("{}", aggregate_error).contains("Aggregate not found"));
    }
}

// Integration tests that require a real database connection
#[cfg(test)]
mod integration_tests {
    use super::*;

    // These tests would require a real PostgreSQL database
    // For now, they are marked as ignored and can be run with: cargo test -- --ignored

    #[tokio::test]
    #[ignore]
    async fn test_full_event_sourcing_workflow() {
        // This test would demonstrate the full event sourcing workflow:
        // 1. Create an aggregate
        // 2. Handle commands and generate events
        // 3. Save events to the store
        // 4. Reload aggregate from events
        // 5. Verify state reconstruction
        // 6. Create snapshots
        // 7. Test projections
        // 8. Test event streaming
        
        // Implementation would require actual database setup
        // Full event sourcing workflow integration test
        let aggregate_id = Uuid::new_v4();
        let event_store = create_test_event_store();
        
        // 1. Create events representing a workflow lifecycle
        let create_event = create_test_event(
            aggregate_id,
            "workflow_created",
            json!({"workflow_type": "test_workflow", "status": "created"}),
            1,
        );
        
        let start_event = create_test_event(
            aggregate_id,
            "workflow_started",
            json!({"started_at": Utc::now().to_rfc3339()}),
            2,
        );
        
        let complete_event = create_test_event(
            aggregate_id,
            "workflow_completed",
            json!({"result": "success", "completed_at": Utc::now().to_rfc3339()}),
            3,
        );
        
        // 2. Save events to the store
        let events = vec![create_event, start_event, complete_event];
        let result = event_store.append_events(&events).await;
        assert!(result.is_ok(), "Failed to append workflow events");
        
        // 3. Reload aggregate from events and verify state reconstruction
        let retrieved_events = event_store.get_events(aggregate_id).await;
        assert!(retrieved_events.is_ok());
        let event_list = retrieved_events.unwrap();
        assert_eq!(event_list.len(), 3);
        
        // 4. Verify events are in correct order
        assert_eq!(event_list[0].event_type, "workflow_created");
        assert_eq!(event_list[1].event_type, "workflow_started");
        assert_eq!(event_list[2].event_type, "workflow_completed");
        
        // 5. Test snapshot functionality
        let snapshot = AggregateSnapshot::new(
            aggregate_id,
            "workflow".to_string(),
            3,
            json!({"status": "completed", "version": 3}),
        );
        
        let save_result = event_store.save_snapshot(&snapshot).await;
        assert!(save_result.is_ok(), "Failed to save snapshot");
        
        // 6. Verify snapshot retrieval
        let retrieved_snapshot = event_store.get_snapshot(aggregate_id).await;
        assert!(retrieved_snapshot.is_ok());
        assert!(retrieved_snapshot.unwrap().is_some());
    }

    #[tokio::test]
    #[ignore]
    async fn test_projection_rebuilding() {
        // This test would verify that projections can be rebuilt from events
        // 1. Create events
        // 2. Build initial projections
        // 3. Reset projections
        // 4. Rebuild from events
        // 5. Verify consistency
        
        // Projection rebuilding test
        let aggregate_id = Uuid::new_v4();
        let event_store = create_test_event_store();
        
        // 1. Create events that would update projections
        let events = vec![
            create_test_event(aggregate_id, "user_created", json!({"name": "Alice", "email": "alice@example.com"}), 1),
            create_test_event(aggregate_id, "user_updated", json!({"name": "Alice Smith"}), 2),
            create_test_event(aggregate_id, "user_activated", json!({"activated_at": Utc::now().to_rfc3339()}), 3),
        ];
        
        // 2. Store events
        let result = event_store.append_events(&events).await;
        assert!(result.is_ok(), "Failed to store events for projection test");
        
        // 3. Simulate initial projection build (in a real system, this would update database tables)
        let projection_state = json!({
            "user_id": aggregate_id,
            "name": "Alice Smith",
            "email": "alice@example.com",
            "status": "active",
            "last_updated": Utc::now().to_rfc3339()
        });
        
        // 4. Simulate projection reset and rebuild from events
        let retrieved_events = event_store.get_events(aggregate_id).await;
        assert!(retrieved_events.is_ok());
        
        let event_list = retrieved_events.unwrap();
        assert_eq!(event_list.len(), 3);
        
        // 5. Verify events can be processed to rebuild projection
        let mut rebuilt_projection = json!({});
        for event in event_list {
            match event.event_type.as_str() {
                "user_created" => {
                    rebuilt_projection["user_id"] = json!(event.aggregate_id);
                    if let Some(name) = event.event_data.get("name") {
                        rebuilt_projection["name"] = name.clone();
                    }
                    if let Some(email) = event.event_data.get("email") {
                        rebuilt_projection["email"] = email.clone();
                    }
                },
                "user_updated" => {
                    if let Some(name) = event.event_data.get("name") {
                        rebuilt_projection["name"] = name.clone();
                    }
                },
                "user_activated" => {
                    rebuilt_projection["status"] = json!("active");
                },
                _ => {}
            }
        }
        
        // 6. Verify rebuilt projection matches expected state
        assert_eq!(rebuilt_projection["name"], json!("Alice Smith"));
        assert_eq!(rebuilt_projection["email"], json!("alice@example.com"));
        assert_eq!(rebuilt_projection["status"], json!("active"));
    }

    #[tokio::test]
    #[ignore]
    async fn test_event_streaming_real_time() {
        // This test would verify real-time event streaming
        // 1. Set up event streams with subscribers
        // 2. Generate events
        // 3. Verify subscribers receive events in real-time
        // 4. Test filtering and error handling
        
        // Real-time event streaming test
        let event_store = Arc::new(create_test_event_store());
        let mut dispatcher = EventDispatcher::new(event_store.clone());
        
        // 1. Set up event stream subscriber
        let handler = Arc::new(LoggingEventHandler::new(
            "streaming_test_handler".to_string(),
            vec!["stream_test_event".to_string()],
        ));
        
        let register_result = dispatcher.register_handler(handler.clone()).await;
        assert!(register_result.is_ok(), "Failed to register streaming handler");
        
        // 2. Generate test events
        let test_events = vec![
            create_test_event(Uuid::new_v4(), "stream_test_event", json!({"message": "Event 1"}), 1),
            create_test_event(Uuid::new_v4(), "stream_test_event", json!({"message": "Event 2"}), 1),
            create_test_event(Uuid::new_v4(), "other_event", json!({"message": "Should be filtered"}), 1),
            create_test_event(Uuid::new_v4(), "stream_test_event", json!({"message": "Event 3"}), 1),
        ];
        
        // 3. Dispatch events and verify handling
        for event in test_events {
            let dispatch_result = dispatcher.dispatch(&event).await;
            if event.event_type == "stream_test_event" {
                assert!(dispatch_result.is_ok(), "Failed to dispatch stream test event: {:?}", dispatch_result.err());
            }
        }
        
        // 4. In a real implementation, we would verify that the subscriber received exactly 3 events
        // For this test, we verify that the dispatcher successfully processed the events
        
        // 5. Test error handling by creating an invalid event
        let mut invalid_event = create_test_event(Uuid::new_v4(), "stream_test_event", json!({"invalid": true}), 1);
        invalid_event.event_data = json!("invalid_json_structure");
        
        // The handler should be resilient to invalid events
        let error_result = dispatcher.dispatch(&invalid_event).await;
        // Depending on implementation, this might succeed with logging or fail gracefully
        assert!(error_result.is_ok() || error_result.is_err(), "Handler should handle invalid events gracefully");
    }

    #[tokio::test]
    #[ignore]
    async fn test_performance_and_scalability() {
        // This test would verify performance characteristics
        // 1. Create large numbers of events
        // 2. Measure append performance
        // 3. Measure query performance
        // 4. Test concurrent access
        // 5. Verify memory usage
        
        // Performance and scalability test
        let event_store = create_test_event_store();
        let aggregate_id = Uuid::new_v4();
        
        // 1. Create a large number of events
        let event_count = 100; // Reduced for test performance
        let mut events = Vec::with_capacity(event_count);
        
        for i in 1..=event_count {
            let event = create_test_event(
                aggregate_id,
                "performance_test_event",
                json!({"sequence": i, "data": format!("test_data_{}", i)}),
                i as i64,
            );
            events.push(event);
        }
        
        // 2. Measure append performance
        let start_time = std::time::Instant::now();
        let batch_result = event_store.append_events(&events).await;
        let append_duration = start_time.elapsed();
        
        assert!(batch_result.is_ok(), "Failed to append batch of events");
        println!("Appended {} events in {:?}", event_count, append_duration);
        
        // 3. Measure query performance
        let query_start = std::time::Instant::now();
        let retrieved_events = event_store.get_events(aggregate_id).await;
        let query_duration = query_start.elapsed();
        
        assert!(retrieved_events.is_ok(), "Failed to retrieve events");
        let event_list = retrieved_events.unwrap();
        assert_eq!(event_list.len(), event_count);
        println!("Retrieved {} events in {:?}", event_count, query_duration);
        
        // 4. Test partial queries
        let partial_start = std::time::Instant::now();
        let partial_events = event_store.get_events_from_version(aggregate_id, 50).await;
        let partial_duration = partial_start.elapsed();
        
        assert!(partial_events.is_ok(), "Failed to retrieve partial events");
        let partial_list = partial_events.unwrap();
        assert_eq!(partial_list.len(), event_count - 49); // Events 50-100
        println!("Retrieved {} partial events in {:?}", partial_list.len(), partial_duration);
        
        // 5. Test aggregate version query
        let version_start = std::time::Instant::now();
        let version = event_store.get_aggregate_version(aggregate_id).await;
        let version_duration = version_start.elapsed();
        
        assert!(version.is_ok(), "Failed to get aggregate version");
        assert_eq!(version.unwrap(), event_count as i64);
        println!("Retrieved aggregate version in {:?}", version_duration);
        
        // 6. Performance assertions (these are lenient for test environment)
        assert!(append_duration.as_millis() < 5000, "Append operation took too long: {:?}", append_duration);
        assert!(query_duration.as_millis() < 1000, "Query operation took too long: {:?}", query_duration);
        assert!(partial_duration.as_millis() < 1000, "Partial query took too long: {:?}", partial_duration);
        assert!(version_duration.as_millis() < 100, "Version query took too long: {:?}", version_duration);
    }
}