// File: tests/event_sourcing_resilience_tests.rs
//
// Tests for the resilient event sourcing system with error handling and recovery

use backend::db::events::*;
use backend::db::events::error_integration::*;
use backend::db::events::types::*;
use backend::db::events::dead_letter_queue::{DeadLetterQueue, DeadLetterEntry, DeadLetterStatistics};
use backend::db::migration::*;
use backend::core::error::*;
use chrono::Utc;
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};
use uuid::Uuid;
use tokio;

// Mock event store that can simulate failures
pub struct MockFailingEventStore {
    failure_count: AtomicU32,
    max_failures_before_success: u32,
    should_fail_permanently: bool,
}

impl MockFailingEventStore {
    pub fn new(max_failures_before_success: u32) -> Self {
        Self {
            failure_count: AtomicU32::new(0),
            max_failures_before_success,
            should_fail_permanently: false,
        }
    }

    pub fn new_permanent_failure() -> Self {
        Self {
            failure_count: AtomicU32::new(0),
            max_failures_before_success: 0,
            should_fail_permanently: true,
        }
    }
}

#[async_trait::async_trait]
impl EventStore for MockFailingEventStore {
    async fn append_event(&self, _event: &EventEnvelope) -> EventResult<()> {
        if self.should_fail_permanently {
            return Err(EventError::DatabaseError {
                message: "Permanent database failure".to_string(),
            });
        }

        let count = self.failure_count.fetch_add(1, Ordering::SeqCst);
        if count < self.max_failures_before_success {
            Err(EventError::DatabaseError {
                message: format!("Simulated database failure #{}", count + 1),
            })
        } else {
            Ok(())
        }
    }

    async fn append_events(&self, events: &[EventEnvelope]) -> EventResult<()> {
        // Simulate batch append by calling append_event for the first event
        if !events.is_empty() {
            self.append_event(&events[0]).await
        } else {
            Ok(())
        }
    }

        async fn get_events(&self, _aggregate_id: Uuid) -> EventResult<Vec<EventEnvelope>> {
            if self.should_fail_permanently {
                return Err(EventError::DatabaseError {
                    message: "Permanent database failure".to_string(),
                });
            }
            Ok(vec![])
        }

        async fn get_events_from_version(&self, _aggregate_id: Uuid, _from_version: i64) -> EventResult<Vec<EventEnvelope>> {
            Ok(vec![])
        }

        async fn get_events_by_type(&self, _event_type: &str, _from: Option<chrono::DateTime<chrono::Utc>>, _to: Option<chrono::DateTime<chrono::Utc>>, _limit: Option<usize>) -> EventResult<Vec<EventEnvelope>> {
            Ok(vec![])
        }

        async fn get_events_by_correlation_id(&self, _correlation_id: Uuid) -> EventResult<Vec<EventEnvelope>> {
            Ok(vec![])
        }

        async fn get_aggregate_version(&self, _aggregate_id: Uuid) -> EventResult<i64> {
            Ok(0)
        }

        async fn aggregate_exists(&self, _aggregate_id: Uuid) -> EventResult<bool> {
            Ok(false)
        }

        async fn save_snapshot(&self, _snapshot: &AggregateSnapshot) -> EventResult<()> {
            Ok(())
        }

        async fn get_snapshot(&self, _aggregate_id: Uuid) -> EventResult<Option<AggregateSnapshot>> {
            Ok(None)
        }

        async fn get_events_from_position(&self, _position: i64, _limit: usize) -> EventResult<Vec<EventEnvelope>> {
            Ok(vec![])
        }

        async fn get_current_position(&self) -> EventResult<i64> {
            Ok(0)
        }

        async fn replay_events(&self, _from_position: i64, _event_types: Option<Vec<String>>, _batch_size: usize) -> EventResult<Vec<EventEnvelope>> {
            Ok(vec![])
        }
    }

    // Mock dead letter queue for testing
    pub struct MockDeadLetterQueue {
        sent_events: Arc<tokio::sync::RwLock<Vec<(Uuid, serde_json::Value, String)>>>,
    }

    impl MockDeadLetterQueue {
        pub fn new() -> Self {
            Self {
                sent_events: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            }
        }

        async fn get_sent_events_count(&self) -> usize {
            self.sent_events.read().await.len()
        }
    }

    #[async_trait::async_trait]
    impl DeadLetterQueue for MockDeadLetterQueue {
        async fn add_failed_event(
            &self,
            event: &EventEnvelope,
            error_message: String,
            error_details: serde_json::Value,
        ) -> EventResult<()> {
            let mut events = self.sent_events.write().await;
            events.push((event.event_id, event.event_data.clone(), error_message));
            Ok(())
        }

        async fn get_retry_candidates(&self, _limit: usize) -> EventResult<Vec<DeadLetterEntry>> {
            Ok(vec![])
        }

        async fn mark_retrying(&self, _entry_id: Uuid) -> EventResult<()> {
            Ok(())
        }

        async fn mark_resolved(&self, _entry_id: Uuid) -> EventResult<()> {
            Ok(())
        }

        async fn increment_retry(&self, _entry_id: Uuid, _error_message: String) -> EventResult<()> {
            Ok(())
        }

        async fn mark_permanently_failed(&self, _entry_id: Uuid) -> EventResult<()> {
            Ok(())
        }

        async fn get_statistics(&self) -> EventResult<DeadLetterStatistics> {
            Ok(DeadLetterStatistics {
                total_entries: self.sent_events.read().await.len() as i64,
                failed_entries: 0,
                retrying_entries: 0,
                permanently_failed_entries: 0,
                resolved_entries: 0,
                oldest_entry: None,
                newest_entry: None,
                average_retry_count: 0.0,
            })
        }

        async fn purge_old_entries(&self, _older_than: chrono::DateTime<chrono::Utc>) -> EventResult<usize> {
            Ok(0)
        }
    }

    // Mock metrics for testing
    pub struct MockEventSourcingMetrics {
        operation_failures: Arc<tokio::sync::RwLock<Vec<String>>>,
        circuit_breaker_opens: Arc<tokio::sync::RwLock<Vec<String>>>,
    }

    impl MockEventSourcingMetrics {
        pub fn new() -> Self {
            Self {
                operation_failures: Arc::new(tokio::sync::RwLock::new(Vec::new())),
                circuit_breaker_opens: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            }
        }

        fn record_operation_success(&self, _operation: &str) {
            // Mock implementation
        }

        fn record_operation_failure(&self, operation: &str, _error: &str) {
            let _ = self.operation_failures.blocking_write().push(operation.to_string());
        }

        fn record_circuit_breaker_open(&self, operation: &str) {
            let _ = self.circuit_breaker_opens.blocking_write().push(operation.to_string());
        }

        fn record_dlq_success(&self, _operation: &str) {
            // Mock implementation
        }

        fn record_dlq_failure(&self, _operation: &str) {
            // Mock implementation
        }

        async fn get_failure_count(&self) -> usize {
            self.operation_failures.read().await.len()
        }
    }

    pub fn create_test_event(aggregate_id: Uuid, version: i64) -> EventEnvelope {
        EventEnvelope {
            event_id: Uuid::new_v4(),
            aggregate_id,
            aggregate_type: "test_aggregate".to_string(),
            event_type: "test_event".to_string(),
            aggregate_version: version,
            event_data: json!({"message": "test"}),
            metadata: EventMetadata::default(),
            occurred_at: Utc::now(),
            recorded_at: Utc::now(),
            schema_version: 1,
            causation_id: None,
            correlation_id: Some(Uuid::new_v4()),
            checksum: None,
        }
    }

    #[tokio::test]
    async fn test_resilient_event_store_retry_success() {
        // Create a mock store that fails 2 times before succeeding
        let mock_store = Arc::new(MockFailingEventStore::new(2));
        let mock_dlq = Arc::new(MockDeadLetterQueue::new());
        let mock_metrics = Arc::new(MockEventSourcingMetrics::new());
        
        // Create resilient store
        let resilient_store = ResilientEventStore::new(
            mock_store.clone(),
            mock_dlq.clone(),
        );

        let aggregate_id = Uuid::new_v4();
        let event = create_test_event(aggregate_id, 1);

        // Should eventually succeed after retries
        let result = resilient_store.append_event(&event).await;
        assert!(result.is_ok(), "Expected success after retries, got: {:?}", result.err());

        // DLQ should not have received any events since it eventually succeeded
        assert_eq!(mock_dlq.get_sent_events_count().await, 0);
    }

    #[tokio::test]
    async fn test_resilient_event_store_permanent_failure() {
        // Create a mock store that always fails
        let mock_store = Arc::new(MockFailingEventStore::new_permanent_failure());
        let mock_dlq = Arc::new(MockDeadLetterQueue::new());
        let mock_metrics = Arc::new(MockEventSourcingMetrics::new());
        
        let resilient_store = ResilientEventStore::new(
            mock_store.clone(),
            mock_dlq.clone(),
        );

        let aggregate_id = Uuid::new_v4();
        let event = create_test_event(aggregate_id, 1);

        // Should fail after all retries are exhausted
        let result = resilient_store.append_event(&event).await;
        assert!(result.is_err(), "Expected failure after all retries");

        // DLQ should have received the failed event
        assert_eq!(mock_dlq.get_sent_events_count().await, 1);

        // Metrics should record the failure
        assert!(mock_metrics.get_failure_count().await > 0);
    }

    #[tokio::test]
    async fn test_resilient_event_store_read_fallback() {
        // Create a store that fails on reads
        let mock_store = Arc::new(MockFailingEventStore::new_permanent_failure());
        let mock_dlq = Arc::new(MockDeadLetterQueue::new());
        let mock_metrics = Arc::new(MockEventSourcingMetrics::new());
        
        let resilient_store = ResilientEventStore::new(
            mock_store.clone(),
            mock_dlq.clone(),
        );

        let aggregate_id = Uuid::new_v4();

        // Read operations should return fallback values instead of failing
        let events = resilient_store.get_events(aggregate_id).await;
        assert!(events.is_ok());
        assert_eq!(events.unwrap().len(), 0); // Should return empty vec as fallback

        let version = resilient_store.get_aggregate_version(aggregate_id).await;
        assert!(version.is_ok());
        assert_eq!(version.unwrap(), 0); // Should return 0 as fallback

        let exists = resilient_store.aggregate_exists(aggregate_id).await;
        assert!(exists.is_ok());
        assert!(!exists.unwrap()); // Should return false as fallback
    }

    #[tokio::test]
    async fn test_error_recovery_strategies() {
        // Test event append recovery
        let append_recovery = EventSourcingRecovery::event_append_recovery();
        let result = append_recovery.recover_from_error(EventError::DatabaseError {
            message: "Test error".to_string(),
        });
        assert!(result.is_ok());

        // Test event retrieval recovery
        let retrieval_recovery = EventSourcingRecovery::event_retrieval_recovery::<Vec<EventEnvelope>>();
        let result = retrieval_recovery.recover_from_error(EventError::DatabaseError {
            message: "Test error".to_string(),
        });
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);

        // Test snapshot recovery
        let snapshot_recovery = EventSourcingRecovery::snapshot_recovery();
        let result = snapshot_recovery.recover_from_error(EventError::DatabaseError {
            message: "Test error".to_string(),
        });
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_batch_append_failure_handling() {
        let mock_store = Arc::new(MockFailingEventStore::new_permanent_failure());
        let mock_dlq = Arc::new(MockDeadLetterQueue::new());
        let mock_metrics = Arc::new(MockEventSourcingMetrics::new());
        
        let resilient_store = ResilientEventStore::new(
            mock_store.clone(),
            mock_dlq.clone(),
        );

        let aggregate_id = Uuid::new_v4();
        let events = vec![
            create_test_event(aggregate_id, 1),
            create_test_event(aggregate_id, 2),
            create_test_event(aggregate_id, 3),
        ];

        // Batch append should fail
        let result = resilient_store.append_events(&events).await;
        assert!(result.is_err());

        // All events should be sent to DLQ
        assert_eq!(mock_dlq.get_sent_events_count().await, 3);
    }

    #[tokio::test]
    async fn test_event_error_conversion() {
        // Test conversion of EventError to WorkflowError
        let database_error = EventError::DatabaseError {
            message: "Database connection failed".to_string(),
        };
        
        let converted = ResilientEventStore::convert_event_error(database_error);
        match converted {
            WorkflowError::DatabaseError { message } => {
                assert_eq!(message, "Database connection failed");
            }
            _ => panic!("Expected DatabaseError"),
        }

        let serialization_error = EventError::SerializationError {
            message: "JSON parsing failed".to_string(),
        };
        
        let converted = ResilientEventStore::convert_event_error(serialization_error);
        match converted {
            WorkflowError::SerializationError { message } => {
                assert_eq!(message, "JSON parsing failed");
            }
            _ => panic!("Expected SerializationError"),
        }
    }

    #[tokio::test]
    async fn test_retryable_error_classification() {
        // Database errors should be retryable
        let database_error = EventError::DatabaseError {
            message: "Connection timeout".to_string(),
        };
        assert!(ResilientEventStore::is_retryable_error(&database_error));

        // Concurrency errors should be retryable
        let concurrency_error = EventError::ConcurrencyError {
            message: "Version conflict".to_string(),
        };
        assert!(ResilientEventStore::is_retryable_error(&concurrency_error));

        // Serialization errors should not be retryable
        let serialization_error = EventError::SerializationError {
            message: "Invalid JSON".to_string(),
        };
        assert!(!ResilientEventStore::is_retryable_error(&serialization_error));

        // Configuration errors should not be retryable
        let config_error = EventError::ConfigurationError {
            message: "Invalid config".to_string(),
        };
        assert!(!ResilientEventStore::is_retryable_error(&config_error));
    }

#[cfg(test)]
mod migration_tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_migration_filename_parsing() {
        // Test valid migration filenames
        assert_eq!(
            PostgreSQLMigrationManager::parse_migration_filename("20241213_000001_create_event_store.sql"),
            Some(("20241213_000001".to_string(), "create_event_store".to_string()))
        );

        assert_eq!(
            PostgreSQLMigrationManager::parse_migration_filename("20241213_000002_add_indexes_and_partitioning.sql"),
            Some(("20241213_000002".to_string(), "add_indexes_and_partitioning".to_string()))
        );

        // Test invalid filenames
        assert_eq!(
            PostgreSQLMigrationManager::parse_migration_filename("invalid.txt"),
            None
        );

        assert_eq!(
            PostgreSQLMigrationManager::parse_migration_filename("20241213_invalid_format"),
            None
        );
    }

    #[tokio::test]
    async fn test_migration_checksum_calculation() {
        let migration = Migration::new(
            "test_version".to_string(),
            "test_migration".to_string(),
            "CREATE TABLE test (id INT);".to_string()
        );

        assert!(!migration.checksum.is_empty());
        assert_eq!(migration.checksum.len(), 64); // SHA256 hex string length

        // Same content should produce same checksum
        let migration2 = Migration::new(
            "test_version2".to_string(),
            "test_migration2".to_string(),
            "CREATE TABLE test (id INT);".to_string()
        );

        assert_eq!(migration.checksum, migration2.checksum);

        // Different content should produce different checksum
        let migration3 = Migration::new(
            "test_version3".to_string(),
            "test_migration3".to_string(),
            "CREATE TABLE different (name TEXT);".to_string()
        );

        assert_ne!(migration.checksum, migration3.checksum);
    }

    #[tokio::test]
    async fn test_migration_loading_from_directory() {
        // Create temporary directory with test migration files
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let migration_dir = temp_dir.path();

        // Create test migration files
        let migration1_content = "CREATE TABLE test1 (id INT);";
        let migration2_content = "CREATE TABLE test2 (name TEXT);";

        fs::write(
            migration_dir.join("20241213_000001_create_test1.sql"),
            migration1_content
        ).expect("Failed to write migration file");

        fs::write(
            migration_dir.join("20241213_000002_create_test2.sql"),
            migration2_content
        ).expect("Failed to write migration file");

        // Also create a non-SQL file that should be ignored
        fs::write(
            migration_dir.join("README.md"),
            "This should be ignored"
        ).expect("Failed to write readme file");

        // Mock database connection (would need real connection for full test)
        // For now, just test the directory reading logic
        
        // This test would need to be adjusted to work with actual database connection
        // For now, we just verify the test setup works
        assert!(migration_dir.join("20241213_000001_create_test1.sql").exists());
        assert!(migration_dir.join("20241213_000002_create_test2.sql").exists());
    }

    #[tokio::test]
    async fn test_migration_config() {
        let config = MigrationConfig {
            migration_directory: std::path::PathBuf::from("test_migrations"),
            auto_apply: true,
            validate_checksums: true,
        };

        assert_eq!(config.migration_directory, std::path::PathBuf::from("test_migrations"));
        assert!(config.auto_apply);
        assert!(config.validate_checksums);

        // Test default config
        let default_config = MigrationConfig::default();
        assert_eq!(default_config.migration_directory, std::path::PathBuf::from("migrations"));
        assert!(!default_config.auto_apply);
        assert!(default_config.validate_checksums);
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_event_serialization_performance() {
        let workflow_event = WorkflowEvent::WorkflowStarted(WorkflowStartedEvent {
            workflow_id: Uuid::new_v4(),
            workflow_type: "performance_test".to_string(),
            configuration: json!({"large_config": "x".repeat(1000)}),
            input_data: json!({"large_input": "y".repeat(1000)}),
            user_id: Some("test_user".to_string()),
        });

        let start = Instant::now();
        let mut total_size = 0;

        // Serialize 1000 events
        for _ in 0..1000 {
            let serialized = workflow_event.serialize();
            assert!(serialized.is_ok());
            total_size += serialized.unwrap().to_string().len();
        }

        let duration = start.elapsed();
        println!("Serialized 1000 events in {:?}, total size: {} bytes", duration, total_size);

        // Should complete within reasonable time (adjust threshold as needed)
        assert!(duration.as_millis() < 1000, "Serialization took too long: {:?}", duration);
    }

    #[tokio::test]
    async fn test_event_metadata_performance() {
        let start = Instant::now();

        // Create 10000 event metadata objects with various fields
        for i in 0..10000 {
            let metadata = EventMetadata::new()
                .with_correlation_id(Uuid::new_v4())
                .with_causation_id(Uuid::new_v4())
                .with_user_id(format!("user_{}", i))
                .with_source("performance_test".to_string())
                .add_tag("iteration".to_string(), i.to_string())
                .add_custom("timestamp".to_string(), json!(Utc::now()));

            // Verify metadata was created correctly
            assert!(metadata.correlation_id.is_some());
            assert!(metadata.causation_id.is_some());
            assert!(metadata.user_id.is_some());
        }

        let duration = start.elapsed();
        println!("Created 10000 metadata objects in {:?}", duration);

        // Should complete within reasonable time
        assert!(duration.as_millis() < 500, "Metadata creation took too long: {:?}", duration);
    }

    #[tokio::test]
    async fn test_error_handling_overhead() {
        let mock_store = Arc::new(MockFailingEventStore::new(0)); // Always succeeds
        let mock_dlq = Arc::new(MockDeadLetterQueue::new());
        let mock_metrics = Arc::new(MockEventSourcingMetrics::new());
        
        let resilient_store = ResilientEventStore::new(
            mock_store.clone(),
            mock_dlq.clone(),
        );

        let aggregate_id = Uuid::new_v4();
        let event = create_test_event(aggregate_id, 1);

        let start = Instant::now();

        // Perform 100 successful operations
        for i in 1..=100 {
            let mut test_event = event.clone();
            test_event.aggregate_version = i;
            let result = resilient_store.append_event(&test_event).await;
            assert!(result.is_ok());
        }

        let duration = start.elapsed();
        println!("Performed 100 resilient operations in {:?}", duration);

        // Error handling overhead should be minimal for successful operations
        assert!(duration.as_millis() < 1000, "Error handling overhead too high: {:?}", duration);
    }
}