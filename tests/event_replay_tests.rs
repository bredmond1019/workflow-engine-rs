// File: tests/event_replay_tests.rs
//
// Integration tests for event replay functionality
// Tests the replay engine with real PostgreSQL event store

use backend::db::events::*;
use backend::db::events::types::*;
use chrono::Utc;
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

#[cfg(test)]
mod replay_tests {
    use super::*;

    // Helper to create test database connection
    fn get_database_url() -> String {
        std::env::var("TEST_DATABASE_URL")
            .or_else(|_| std::env::var("DATABASE_URL"))
            .unwrap_or_else(|_| "postgresql://postgres:password@localhost/test_ai_workflow_db".to_string())
    }

    fn create_test_event_store() -> Option<Arc<PostgreSQLEventStore>> {
        let config = EventStoreConfig {
            database_url: get_database_url(),
            connection_pool_size: 5,
            batch_size: 100,
            snapshot_frequency: 10,
            enable_checksums: true,
        };
        
        PostgreSQLEventStore::new(config).ok().map(Arc::new)
    }

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
                .with_source("replay_test".to_string()),
            occurred_at: Utc::now(),
            recorded_at: Utc::now(),
            schema_version: 1,
            causation_id: None,
            correlation_id: Some(Uuid::new_v4()),
            checksum: None,
        }
    }

    // Test handler that counts processed events
    struct TestReplayHandler {
        name: String,
        events_processed: Arc<AtomicU64>,
        processed_events: Arc<Mutex<Vec<EventEnvelope>>>,
    }

    impl TestReplayHandler {
        fn new(name: String) -> Self {
            Self {
                name,
                events_processed: Arc::new(AtomicU64::new(0)),
                processed_events: Arc::new(Mutex::new(Vec::new())),
            }
        }

        async fn get_processed_count(&self) -> u64 {
            self.events_processed.load(Ordering::SeqCst)
        }

        async fn get_processed_events(&self) -> Vec<EventEnvelope> {
            self.processed_events.lock().await.clone()
        }
    }

    #[async_trait]
    impl ReplayHandler for TestReplayHandler {
        async fn handle_events(&mut self, events: &[EventEnvelope]) -> EventResult<()> {
            self.events_processed.fetch_add(events.len() as u64, Ordering::SeqCst);
            
            let mut processed = self.processed_events.lock().await;
            processed.extend_from_slice(events);
            
            // Simulate some processing time
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            
            Ok(())
        }

        async fn on_replay_start(&mut self, from_position: i64) -> EventResult<()> {
            println!("Starting replay for '{}' from position {}", self.name, from_position);
            Ok(())
        }

        async fn on_replay_complete(&mut self, events_processed: u64) -> EventResult<()> {
            println!("Completed replay for '{}': {} events processed", self.name, events_processed);
            Ok(())
        }

        fn consumer_name(&self) -> &str {
            &self.name
        }
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database
    async fn test_basic_event_replay() {
        let event_store = match create_test_event_store() {
            Some(store) => store,
            None => {
                println!("Skipping test - could not connect to PostgreSQL database");
                return;
            }
        };

        let aggregate_id = Uuid::new_v4();
        
        // Create test events
        let events: Vec<EventEnvelope> = (1..=20)
            .map(|i| create_test_event(
                aggregate_id,
                "replay_test_event",
                json!({"sequence": i, "data": format!("test_data_{}", i)}),
                i,
            ))
            .collect();

        // Store events
        for event in &events {
            event_store.append_event(event).await.expect("Failed to append event");
        }

        // Create replay engine
        let config = ReplayConfig {
            batch_size: 5,
            checkpoint_frequency: 3,
            ..Default::default()
        };
        
        let replay_engine = EventReplayEngine::new(event_store.clone(), config);
        
        // Create test handler
        let handler = TestReplayHandler::new("test_handler".to_string());
        let handler_arc = Arc::new(Mutex::new(handler));
        
        // Perform replay
        let result = replay_engine.replay_for_handler(
            handler_arc.clone(),
            Some(vec!["replay_test_event".to_string()]),
        ).await;
        
        assert!(result.is_ok(), "Replay should succeed: {:?}", result.err());
        
        let position = result.unwrap();
        assert_eq!(position.consumer_name, "test_handler");
        assert_eq!(position.events_processed, 20);
        
        // Verify all events were processed
        let handler_guard = handler_arc.lock().await;
        let processed_count = handler_guard.get_processed_count().await;
        assert_eq!(processed_count, 20);
        
        let processed_events = handler_guard.get_processed_events().await;
        assert_eq!(processed_events.len(), 20);
        
        // Verify event order
        for (i, event) in processed_events.iter().enumerate() {
            assert_eq!(event.aggregate_version, (i + 1) as i64);
            assert_eq!(event.event_data["sequence"], i + 1);
        }
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database
    async fn test_parallel_replay() {
        let event_store = match create_test_event_store() {
            Some(store) => store,
            None => {
                println!("Skipping test - could not connect to PostgreSQL database");
                return;
            }
        };

        // Create events across multiple aggregates
        for agg_num in 1..=3 {
            let aggregate_id = Uuid::new_v4();
            
            let events: Vec<EventEnvelope> = (1..=10)
                .map(|i| create_test_event(
                    aggregate_id,
                    "parallel_replay_test",
                    json!({"aggregate": agg_num, "sequence": i}),
                    i,
                ))
                .collect();

            for event in &events {
                event_store.append_event(event).await.expect("Failed to append event");
            }
        }

        // Create replay engine
        let config = ReplayConfig {
            batch_size: 5,
            parallelism: 3,
            ..Default::default()
        };
        
        let replay_engine = EventReplayEngine::new(event_store.clone(), config);
        
        // Create multiple handlers
        let handlers: Vec<Arc<Mutex<TestReplayHandler>>> = (1..=3)
            .map(|i| Arc::new(Mutex::new(TestReplayHandler::new(format!("handler_{}", i)))))
            .collect();
        
        // Perform parallel replay
        let result = replay_engine.replay_parallel(
            handlers.clone(),
            Some(vec!["parallel_replay_test".to_string()]),
        ).await;
        
        assert!(result.is_ok(), "Parallel replay should succeed: {:?}", result.err());
        
        let positions = result.unwrap();
        assert_eq!(positions.len(), 3);
        
        // Verify all handlers processed events
        let mut total_processed = 0u64;
        for (i, handler) in handlers.iter().enumerate() {
            let handler_guard = handler.lock().await;
            let processed_count = handler_guard.get_processed_count().await;
            total_processed += processed_count;
            
            assert_eq!(positions[i].consumer_name, format!("handler_{}", i + 1));
            println!("Handler {} processed {} events", i + 1, processed_count);
        }
        
        // All handlers should have processed all 30 events (3 aggregates × 10 events each)
        // Since each handler runs independently, they all process the same events
        assert_eq!(total_processed, 30 * 3); // 30 events × 3 handlers
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database
    async fn test_replay_with_snapshots() {
        let event_store = match create_test_event_store() {
            Some(store) => store,
            None => {
                println!("Skipping test - could not connect to PostgreSQL database");
                return;
            }
        };

        let aggregate_id = Uuid::new_v4();
        
        // Create events
        let events: Vec<EventEnvelope> = (1..=15)
            .map(|i| create_test_event(
                aggregate_id,
                "snapshot_replay_test",
                json!({"sequence": i, "accumulated_value": i * 10}),
                i,
            ))
            .collect();

        // Store events
        for event in &events {
            event_store.append_event(event).await.expect("Failed to append event");
        }

        // Create a snapshot at version 10
        let snapshot = AggregateSnapshot::new(
            aggregate_id,
            "test_aggregate".to_string(),
            10,
            json!({
                "state": "snapshot_state",
                "accumulated_value": 550, // Sum of 1*10 + 2*10 + ... + 10*10
                "last_sequence": 10
            }),
        );
        
        event_store.save_snapshot(&snapshot).await.expect("Failed to save snapshot");

        // Create replay engine with snapshots enabled
        let config = ReplayConfig {
            batch_size: 5,
            use_snapshots: true,
            ..Default::default()
        };
        
        let replay_engine = EventReplayEngine::new(event_store.clone(), config);
        
        // Test aggregate replay with snapshot
        let mut events_processed = 0;
        let result = replay_engine.replay_aggregate(
            aggregate_id,
            0, // Start from beginning, should use snapshot
            |event| {
                events_processed += 1;
                println!("Processing event version {} with sequence {}", 
                        event.aggregate_version, event.event_data["sequence"]);
                Ok(())
            },
        ).await;
        
        assert!(result.is_ok(), "Aggregate replay should succeed: {:?}", result.err());
        
        let final_version = result.unwrap();
        assert_eq!(final_version, 15);
        
        // Should only process events 11-15 (after snapshot)
        assert_eq!(events_processed, 5);
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database
    async fn test_replay_position_tracking() {
        let event_store = match create_test_event_store() {
            Some(store) => store,
            None => {
                println!("Skipping test - could not connect to PostgreSQL database");
                return;
            }
        };

        let config = ReplayConfig {
            batch_size: 3,
            checkpoint_frequency: 2,
            ..Default::default()
        };
        
        let replay_engine = EventReplayEngine::new(event_store.clone(), config);
        
        // Test position creation and tracking
        let status = replay_engine.get_replay_status().await;
        assert!(status.is_empty());
        
        // Reset position (should not fail even if doesn't exist)
        let reset_result = replay_engine.reset_position("test_consumer").await;
        assert!(reset_result.is_ok());
        
        // Create a test position
        let position = ReplayPosition::new("test_consumer".to_string());
        assert_eq!(position.consumer_name, "test_consumer");
        assert_eq!(position.position, 0);
        assert_eq!(position.events_processed, 0);
        assert!(position.last_event_id.is_none());
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database
    async fn test_batch_replay_processor() {
        let event_store = match create_test_event_store() {
            Some(store) => store,
            None => {
                println!("Skipping test - could not connect to PostgreSQL database");
                return;
            }
        };

        let aggregate_id = Uuid::new_v4();
        
        // Create test events
        let events: Vec<EventEnvelope> = (1..=25)
            .map(|i| create_test_event(
                aggregate_id,
                "batch_replay_test",
                json!({"sequence": i}),
                i,
            ))
            .collect();

        // Store events
        for event in &events {
            event_store.append_event(event).await.expect("Failed to append event");
        }

        // Create batch processor
        let config = ReplayConfig {
            batch_size: 5,
            ..Default::default()
        };
        
        let replay_engine = EventReplayEngine::new(event_store.clone(), config);
        let batch_processor = BatchReplayProcessor::new(replay_engine, 10, 1);
        
        // Process with buffer
        let mut processed_batches = 0;
        let mut total_events = 0;
        
        let result = batch_processor.process_with_buffer(
            0,
            Some(vec!["batch_replay_test".to_string()]),
            |batch| {
                processed_batches += 1;
                total_events += batch.len();
                println!("Processed batch {} with {} events", processed_batches, batch.len());
                Ok(())
            },
        ).await;
        
        assert!(result.is_ok(), "Batch processing should succeed: {:?}", result.err());
        assert_eq!(total_events, 25);
        assert!(processed_batches >= 2); // Should have multiple batches due to buffer size
    }
}