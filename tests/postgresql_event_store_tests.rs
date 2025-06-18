// File: tests/postgresql_event_store_tests.rs
//
// Tests specific to PostgreSQL event store implementation
// These tests require a real PostgreSQL database connection

use backend::db::events::*;
use backend::db::events::types::*;
use backend::db::migration::*;
use chrono::Utc;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use serde_json::json;
use std::env;
use std::sync::Arc;
use uuid::Uuid;

#[cfg(test)]
mod postgresql_tests {
    use super::*;

    // Helper to create test database connection
    pub fn create_test_connection_pool() -> Option<Pool<ConnectionManager<PgConnection>>> {
        let database_url = env::var("TEST_DATABASE_URL")
            .or_else(|_| env::var("DATABASE_URL"))
            .unwrap_or_else(|_| "postgresql://postgres:password@localhost/test_ai_workflow_db".to_string());

        let manager = ConnectionManager::<PgConnection>::new(&database_url);
        Pool::builder()
            .max_size(5)
            .build(manager)
            .ok()
    }

    pub fn create_test_event_store() -> Option<PostgreSQLEventStore> {
        let config = EventStoreConfig {
            database_url: env::var("TEST_DATABASE_URL")
                .or_else(|_| env::var("DATABASE_URL"))
                .unwrap_or_else(|_| "postgresql://postgres:password@localhost/test_ai_workflow_db".to_string()),
            connection_pool_size: 5,
            batch_size: 100,
            snapshot_frequency: 10,
            enable_checksums: true,
        };
        
        PostgreSQLEventStore::new(config).ok()
    }

    pub fn create_test_event(
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
                .with_source("postgresql_test".to_string()),
            occurred_at: Utc::now(),
            recorded_at: Utc::now(),
            schema_version: 1,
            causation_id: None,
            correlation_id: Some(Uuid::new_v4()),
            checksum: None,
        }
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database
    async fn test_postgresql_event_store_basic_operations() {
        let event_store = match create_test_event_store() {
            Some(store) => store,
            None => {
                println!("Skipping test - could not connect to PostgreSQL database");
                return;
            }
        };

        let aggregate_id = Uuid::new_v4();
        
        // Create test event
        let event = create_test_event(
            aggregate_id,
            "test_event",
            json!({"message": "Hello PostgreSQL!", "data": {"count": 42}}),
            1,
        );

        // Test append
        let result = event_store.append_event(&event).await;
        assert!(result.is_ok(), "Failed to append event: {:?}", result.err());

        // Test retrieve
        let retrieved_events = event_store.get_events(aggregate_id).await;
        assert!(retrieved_events.is_ok(), "Failed to retrieve events: {:?}", retrieved_events.err());
        
        let events = retrieved_events.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_id, event.event_id);
        assert_eq!(events[0].aggregate_id, aggregate_id);
        assert_eq!(events[0].event_type, "test_event");
        assert_eq!(events[0].event_data["message"], "Hello PostgreSQL!");
        assert_eq!(events[0].event_data["data"]["count"], 42);

        // Verify checksum was calculated
        assert!(events[0].checksum.is_some());
        assert!(!events[0].checksum.as_ref().unwrap().is_empty());
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database
    async fn test_postgresql_batch_operations() {
        let event_store = match create_test_event_store() {
            Some(store) => store,
            None => {
                println!("Skipping test - could not connect to PostgreSQL database");
                return;
            }
        };

        let aggregate_id = Uuid::new_v4();
        
        // Create batch of events
        let events: Vec<EventEnvelope> = (1..=10)
            .map(|i| create_test_event(
                aggregate_id,
                "batch_event",
                json!({"sequence": i, "batch_id": "test_batch"}),
                i,
            ))
            .collect();

        // Test batch append
        let result = event_store.append_events(&events).await;
        assert!(result.is_ok(), "Failed to append event batch: {:?}", result.err());

        // Test retrieve all
        let retrieved_events = event_store.get_events(aggregate_id).await;
        assert!(retrieved_events.is_ok(), "Failed to retrieve events: {:?}", retrieved_events.err());
        
        let events = retrieved_events.unwrap();
        assert_eq!(events.len(), 10);

        // Verify order and content
        for (i, event) in events.iter().enumerate() {
            assert_eq!(event.aggregate_version, (i + 1) as i64);
            assert_eq!(event.event_data["sequence"], i + 1);
            assert_eq!(event.event_data["batch_id"], "test_batch");
        }
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database
    async fn test_postgresql_concurrency_control() {
        let event_store = match create_test_event_store() {
            Some(store) => store,
            None => {
                println!("Skipping test - could not connect to PostgreSQL database");
                return;
            }
        };

        let aggregate_id = Uuid::new_v4();
        
        // Create first event
        let event1 = create_test_event(
            aggregate_id,
            "concurrency_test",
            json!({"step": 1}),
            1,
        );

        // Append first event
        let result = event_store.append_event(&event1).await;
        assert!(result.is_ok(), "Failed to append first event: {:?}", result.err());

        // Try to append event with same version (should fail due to unique constraint)
        let event2 = create_test_event(
            aggregate_id,
            "concurrency_test",
            json!({"step": 2}),
            1, // Same version as event1
        );

        let result = event_store.append_event(&event2).await;
        assert!(result.is_err(), "Expected concurrency error");
        
        // Verify it's a concurrency error
        match result.unwrap_err() {
            EventError::ConcurrencyError { .. } => {
                // Expected error type
            }
            other => panic!("Expected ConcurrencyError, got: {:?}", other),
        }

        // Append with correct version should succeed
        let event3 = create_test_event(
            aggregate_id,
            "concurrency_test",
            json!({"step": 3}),
            2, // Correct next version
        );

        let result = event_store.append_event(&event3).await;
        assert!(result.is_ok(), "Failed to append event with correct version: {:?}", result.err());
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database
    async fn test_postgresql_query_operations() {
        let event_store = match create_test_event_store() {
            Some(store) => store,
            None => {
                println!("Skipping test - could not connect to PostgreSQL database");
                return;
            }
        };

        let aggregate_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        
        // Create events with specific characteristics for querying
        let mut events = Vec::new();
        for i in 1..=5 {
            let mut event = create_test_event(
                aggregate_id,
                if i % 2 == 0 { "even_event" } else { "odd_event" },
                json!({"value": i, "category": if i <= 3 { "early" } else { "late" }}),
                i,
            );
            event.correlation_id = Some(correlation_id);
            events.push(event);
        }

        // Append events
        let result = event_store.append_events(&events).await;
        assert!(result.is_ok(), "Failed to append events: {:?}", result.err());

        // Test get events from version
        let events_from_3 = event_store.get_events_from_version(aggregate_id, 2).await;
        assert!(events_from_3.is_ok(), "Failed to get events from version: {:?}", events_from_3.err());
        
        let filtered_events = events_from_3.unwrap();
        assert_eq!(filtered_events.len(), 3); // Events 3, 4, 5
        assert_eq!(filtered_events[0].aggregate_version, 3);
        assert_eq!(filtered_events[2].aggregate_version, 5);

        // Test get events by correlation ID
        let correlated_events = event_store.get_events_by_correlation_id(correlation_id).await;
        assert!(correlated_events.is_ok(), "Failed to get correlated events: {:?}", correlated_events.err());
        
        let corr_events = correlated_events.unwrap();
        assert_eq!(corr_events.len(), 5);

        // Test get events by type
        let even_events = event_store.get_events_by_type(
            "even_event",
            None,
            None,
            None,
        ).await;
        assert!(even_events.is_ok(), "Failed to get events by type: {:?}", even_events.err());
        
        let type_events = even_events.unwrap();
        assert_eq!(type_events.len(), 2); // Events 2 and 4

        // Test aggregate version
        let version = event_store.get_aggregate_version(aggregate_id).await;
        assert!(version.is_ok(), "Failed to get aggregate version: {:?}", version.err());
        assert_eq!(version.unwrap(), 5);

        // Test aggregate exists
        let exists = event_store.aggregate_exists(aggregate_id).await;
        assert!(exists.is_ok(), "Failed to check aggregate existence: {:?}", exists.err());
        assert!(exists.unwrap());

        // Test non-existent aggregate
        let non_existent = event_store.aggregate_exists(Uuid::new_v4()).await;
        assert!(non_existent.is_ok(), "Failed to check non-existent aggregate: {:?}", non_existent.err());
        assert!(!non_existent.unwrap());
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database
    async fn test_postgresql_snapshot_operations() {
        let event_store = match create_test_event_store() {
            Some(store) => store,
            None => {
                println!("Skipping test - could not connect to PostgreSQL database");
                return;
            }
        };

        let aggregate_id = Uuid::new_v4();

        // Create and save snapshot
        let snapshot_data = json!({
            "state": "test_state",
            "accumulated_value": 150,
            "last_update": Utc::now(),
            "metadata": {
                "version": 5,
                "checkpoints": [1, 3, 5]
            }
        });

        let snapshot = AggregateSnapshot::new(
            aggregate_id,
            "test_aggregate".to_string(),
            5,
            snapshot_data.clone(),
        );

        // Save snapshot
        let result = event_store.save_snapshot(&snapshot).await;
        assert!(result.is_ok(), "Failed to save snapshot: {:?}", result.err());

        // Retrieve snapshot
        let retrieved = event_store.get_snapshot(aggregate_id).await;
        assert!(retrieved.is_ok(), "Failed to retrieve snapshot: {:?}", retrieved.err());
        
        let snapshot_option = retrieved.unwrap();
        assert!(snapshot_option.is_some(), "Snapshot should exist");
        
        let retrieved_snapshot = snapshot_option.unwrap();
        assert_eq!(retrieved_snapshot.aggregate_id, aggregate_id);
        assert_eq!(retrieved_snapshot.aggregate_version, 5);
        assert_eq!(retrieved_snapshot.aggregate_type, "test_aggregate");
        assert_eq!(retrieved_snapshot.snapshot_data, snapshot_data);

        // Update snapshot with newer version
        let updated_snapshot_data = json!({
            "state": "updated_state",
            "accumulated_value": 200,
            "metadata": {
                "version": 7,
                "checkpoints": [1, 3, 5, 7]
            }
        });

        let updated_snapshot = AggregateSnapshot::new(
            aggregate_id,
            "test_aggregate".to_string(),
            7,
            updated_snapshot_data.clone(),
        );

        let result = event_store.save_snapshot(&updated_snapshot).await;
        assert!(result.is_ok(), "Failed to update snapshot: {:?}", result.err());

        // Retrieve updated snapshot
        let retrieved = event_store.get_snapshot(aggregate_id).await;
        assert!(retrieved.is_ok(), "Failed to retrieve updated snapshot: {:?}", retrieved.err());
        
        let snapshot_option = retrieved.unwrap();
        assert!(snapshot_option.is_some(), "Updated snapshot should exist");
        
        let retrieved_snapshot = snapshot_option.unwrap();
        assert_eq!(retrieved_snapshot.aggregate_version, 7);
        assert_eq!(retrieved_snapshot.snapshot_data, updated_snapshot_data);

        // Test non-existent snapshot
        let non_existent = event_store.get_snapshot(Uuid::new_v4()).await;
        assert!(non_existent.is_ok(), "Failed to check non-existent snapshot: {:?}", non_existent.err());
        assert!(non_existent.unwrap().is_none());
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database
    async fn test_postgresql_event_streaming() {
        let event_store = match create_test_event_store() {
            Some(store) => store,
            None => {
                println!("Skipping test - could not connect to PostgreSQL database");
                return;
            }
        };

        // Create events in multiple aggregates
        let mut all_events = Vec::new();
        for agg in 1..=3 {
            let aggregate_id = Uuid::new_v4();
            for i in 1..=5 {
                let event = create_test_event(
                    aggregate_id,
                    "streaming_test",
                    json!({"aggregate": agg, "sequence": i}),
                    i,
                );
                all_events.push(event);
            }
        }

        // Append all events
        let result = event_store.append_events(&all_events).await;
        assert!(result.is_ok(), "Failed to append streaming test events: {:?}", result.err());

        // Test get current position
        let current_position = event_store.get_current_position().await;
        assert!(current_position.is_ok(), "Failed to get current position: {:?}", current_position.err());
        let position = current_position.unwrap();
        assert!(position > 0, "Current position should be greater than 0");

        // Test get events from position (start from beginning)
        let events_from_start = event_store.get_events_from_position(0, 10).await;
        assert!(events_from_start.is_ok(), "Failed to get events from position: {:?}", events_from_start.err());
        
        let streamed_events = events_from_start.unwrap();
        assert!(streamed_events.len() > 0, "Should have streamed some events");
        assert!(streamed_events.len() <= 10, "Should respect limit");

        // Test replay events with type filter
        let replayed_events = event_store.replay_events(
            0,
            Some(vec!["streaming_test".to_string()]),
            5,
        ).await;
        assert!(replayed_events.is_ok(), "Failed to replay events: {:?}", replayed_events.err());
        
        let replay_events = replayed_events.unwrap();
        assert!(replay_events.len() > 0, "Should have replayed some events");
        assert!(replay_events.len() <= 5, "Should respect batch size");
        
        // All replayed events should be of the filtered type
        for event in replay_events {
            assert_eq!(event.event_type, "streaming_test");
        }
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database
    async fn test_postgresql_large_event_data() {
        let event_store = match create_test_event_store() {
            Some(store) => store,
            None => {
                println!("Skipping test - could not connect to PostgreSQL database");
                return;
            }
        };

        let aggregate_id = Uuid::new_v4();
        
        // Create event with large JSON payload
        let large_data = json!({
            "large_text": "x".repeat(10000),
            "large_array": (0..1000).collect::<Vec<i32>>(),
            "nested_object": {
                "level1": {
                    "level2": {
                        "level3": {
                            "data": "y".repeat(5000)
                        }
                    }
                }
            },
            "metadata": {
                "processing_info": "z".repeat(2000),
                "timestamps": (0..100).map(|i| format!("timestamp_{}", i)).collect::<Vec<String>>()
            }
        });

        let event = create_test_event(
            aggregate_id,
            "large_event",
            large_data.clone(),
            1,
        );

        // Test append large event
        let result = event_store.append_event(&event).await;
        assert!(result.is_ok(), "Failed to append large event: {:?}", result.err());

        // Test retrieve large event
        let retrieved_events = event_store.get_events(aggregate_id).await;
        assert!(retrieved_events.is_ok(), "Failed to retrieve large event: {:?}", retrieved_events.err());
        
        let events = retrieved_events.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_data, large_data);
        
        // Verify checksum integrity
        assert!(events[0].checksum.is_some());
        assert!(!events[0].checksum.as_ref().unwrap().is_empty());
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database
    async fn test_migration_system() {
        let pool = match create_test_connection_pool() {
            Some(pool) => pool,
            None => {
                println!("Skipping test - could not connect to PostgreSQL database");
                return;
            }
        };

        let manager = PostgreSQLMigrationManager::new(pool);

        // Test that we can check if a migration is applied
        let is_applied = manager.is_migration_applied("20241213_000001").await;
        assert!(is_applied.is_ok(), "Failed to check migration status: {:?}", is_applied.err());

        // Test getting applied migrations
        let applied_migrations = manager.get_applied_migrations().await;
        assert!(applied_migrations.is_ok(), "Failed to get applied migrations: {:?}", applied_migrations.err());

        println!("Applied migrations: {}", applied_migrations.unwrap().len());
    }
}

#[cfg(test)]
mod database_specific_tests {
    use super::*;
    use super::postgresql_tests::*;

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database with specific setup
    async fn test_postgresql_jsonb_queries() {
        let event_store = match create_test_event_store() {
            Some(store) => store,
            None => {
                println!("Skipping test - could not connect to PostgreSQL database");
                return;
            }
        };

        let aggregate_id = Uuid::new_v4();
        
        // Create events with structured JSON data for JSONB testing
        let events = vec![
            create_test_event(
                aggregate_id,
                "user_action",
                json!({
                    "action": "login",
                    "user": {"id": 123, "name": "Alice"},
                    "metadata": {"ip": "192.168.1.1", "user_agent": "Chrome"}
                }),
                1,
            ),
            create_test_event(
                aggregate_id,
                "user_action",
                json!({
                    "action": "purchase",
                    "user": {"id": 123, "name": "Alice"},
                    "product": {"id": 456, "name": "Widget", "price": 29.99},
                    "metadata": {"payment_method": "credit_card"}
                }),
                2,
            ),
            create_test_event(
                aggregate_id,
                "user_action",
                json!({
                    "action": "logout",
                    "user": {"id": 123, "name": "Alice"},
                    "metadata": {"session_duration": 1800}
                }),
                3,
            ),
        ];

        // Append events
        let result = event_store.append_events(&events).await;
        assert!(result.is_ok(), "Failed to append JSONB test events: {:?}", result.err());

        // Retrieve and verify JSONB data structure is preserved
        let retrieved_events = event_store.get_events(aggregate_id).await;
        assert!(retrieved_events.is_ok(), "Failed to retrieve JSONB events: {:?}", retrieved_events.err());
        
        let events = retrieved_events.unwrap();
        assert_eq!(events.len(), 3);

        // Verify JSON structure preservation
        assert_eq!(events[0].event_data["action"], "login");
        assert_eq!(events[0].event_data["user"]["id"], 123);
        assert_eq!(events[0].event_data["user"]["name"], "Alice");

        assert_eq!(events[1].event_data["action"], "purchase");
        assert_eq!(events[1].event_data["product"]["price"], 29.99);

        assert_eq!(events[2].event_data["action"], "logout");
        assert_eq!(events[2].event_data["metadata"]["session_duration"], 1800);
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database
    async fn test_postgresql_transaction_isolation() {
        let event_store = match create_test_event_store() {
            Some(store) => store,
            None => {
                println!("Skipping test - could not connect to PostgreSQL database");
                return;
            }
        };

        let aggregate_id = Uuid::new_v4();
        
        // Create multiple events to test transaction boundaries
        let events = vec![
            create_test_event(aggregate_id, "tx_test_1", json!({"step": 1}), 1),
            create_test_event(aggregate_id, "tx_test_2", json!({"step": 2}), 2),
            create_test_event(aggregate_id, "tx_test_3", json!({"step": 3}), 3),
        ];

        // Test that batch append is atomic (all or nothing)
        let result = event_store.append_events(&events).await;
        assert!(result.is_ok(), "Batch append should succeed: {:?}", result.err());

        // Verify all events were stored
        let retrieved_events = event_store.get_events(aggregate_id).await;
        assert!(retrieved_events.is_ok(), "Failed to retrieve events: {:?}", retrieved_events.err());
        
        let stored_events = retrieved_events.unwrap();
        assert_eq!(stored_events.len(), 3, "All events should be stored atomically");

        // Test that individual checksum validation works
        for event in &stored_events {
            assert!(event.checksum.is_some(), "Checksum should be calculated");
            assert!(!event.checksum.as_ref().unwrap().is_empty(), "Checksum should not be empty");
        }
    }
}