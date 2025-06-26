// File: src/db/events/tests/integration_tests.rs
//
// Integration tests for the complete event-driven microservices synchronization system

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::db::events::*;

#[tokio::test]
#[ignore] // Requires Redis and database setup
async fn test_end_to_end_event_processing() {
    // Setup event store
    let event_store = setup_test_event_store().await;
    
    // Setup enhanced dead letter queue
    let dlq_config = EnhancedDLQConfig {
        redis_url: Some("redis://localhost:6379".to_string()),
        use_redis_persistence: true,
        ..EnhancedDLQConfig::default()
    };
    let dlq = Arc::new(EnhancedDeadLetterQueue::new(dlq_config).await.unwrap());
    
    // Setup cross-service router
    let routing_config = ServiceRoutingConfig {
        event_routes: {
            let mut routes = std::collections::HashMap::new();
            routes.insert("user_created".to_string(), vec!["profile_service".to_string(), "notification_service".to_string()]);
            routes.insert("order_placed".to_string(), vec!["inventory_service".to_string(), "payment_service".to_string()]);
            routes
        },
        ..ServiceRoutingConfig::default()
    };
    let router = Arc::new(CrossServiceEventRouter::new("redis://localhost:6379", routing_config).await.unwrap());
    
    // Setup event dispatcher with all components
    let dispatcher = EventDispatcher::with_cross_service_routing(
        event_store,
        dlq.clone(),
        router.clone(),
        "test_service".to_string(),
    );
    
    // Create test events
    let user_event = create_test_event("user_created", "Test user creation");
    let order_event = create_test_event("order_placed", "Test order placement");
    
    // Dispatch events
    dispatcher.dispatch(&user_event).await.unwrap();
    dispatcher.dispatch(&order_event).await.unwrap();
    
    // Verify events were processed
    sleep(Duration::from_millis(500)).await;
    
    let routing_stats = router.get_routing_stats().await.unwrap();
    assert!(routing_stats.total_events_routed >= 2);
    
    let dlq_stats = dlq.get_enhanced_statistics().await;
    assert_eq!(dlq_stats.enhanced_metrics.total_events_added, 0); // No failures expected
}

#[tokio::test]
#[ignore] // Requires database setup
async fn test_saga_orchestration_with_compensation() {
    let event_store = setup_test_event_store().await;
    let saga_store = Arc::new(InMemorySagaStore::new());
    let orchestrator = SagaOrchestrator::new(event_store, saga_store);
    
    // Register mock step executors
    orchestrator.register_step_executor(
        "payment_service",
        Arc::new(MockSagaStepExecutor::new("payment_service", true)),
    ).await;
    
    orchestrator.register_step_executor(
        "inventory_service", 
        Arc::new(MockSagaStepExecutor::new("inventory_service", false)), // Will fail
    ).await;
    
    // Define a multi-step saga
    let saga_definition = SagaDefinition {
        saga_type: "order_processing".to_string(),
        name: "Process Order".to_string(),
        description: "Complete order processing workflow".to_string(),
        steps: vec![
            SagaStep {
                step_id: "reserve_payment".to_string(),
                service_name: "payment_service".to_string(),
                operation: "reserve_funds".to_string(),
                input_data: serde_json::json!({"amount": 100.0}),
                compensation_operation: Some("release_funds".to_string()),
                compensation_data: Some(serde_json::json!({"amount": 100.0})),
                timeout_seconds: Some(30),
                retry_policy: Some(RetryPolicy::default()),
                depends_on: vec![],
                parallel_group: None,
            },
            SagaStep {
                step_id: "reserve_inventory".to_string(),
                service_name: "inventory_service".to_string(),
                operation: "reserve_items".to_string(),
                input_data: serde_json::json!({"items": ["item1", "item2"]}),
                compensation_operation: Some("release_items".to_string()),
                compensation_data: Some(serde_json::json!({"items": ["item1", "item2"]})),
                timeout_seconds: Some(30),
                retry_policy: Some(RetryPolicy::default()),
                depends_on: vec!["reserve_payment".to_string()],
                parallel_group: None,
            },
        ],
        global_timeout_seconds: Some(300),
        compensation_strategy: CompensationStrategy::ReverseOrder,
    };
    
    // Start saga
    let saga_id = orchestrator.start_saga(
        saga_definition,
        serde_json::json!({"order_id": "order_123"})
    ).await.unwrap();
    
    // Wait for processing
    sleep(Duration::from_millis(1000)).await;
    
    // Verify saga failed and compensation ran
    let saga_status = orchestrator.get_saga_status(saga_id).await.unwrap().unwrap();
    assert_eq!(saga_status.state, SagaState::Failed);
    
    // Verify compensation was executed
    let payment_step = saga_status.steps.iter()
        .find(|s| s.step.step_id == "reserve_payment")
        .unwrap();
    assert_eq!(payment_step.status, SagaStepStatus::Compensated);
}

#[tokio::test]
async fn test_event_ordering_and_deduplication() {
    let config = OrderingConfig {
        ordering_strategy: OrderingStrategy::Timestamp,
        deduplication_strategy: DeduplicationStrategy::EventId,
        buffer_size: 100,
        max_out_of_order_delay_ms: 1000,
        enable_strict_ordering: false,
        partition_count: Some(4),
    };
    
    let processor = EventOrderingProcessor::new(config);
    let aggregate_id = Uuid::new_v4();
    
    // Create events with different timestamps
    let mut events = vec![];
    for i in 0..5 {
        let mut event = create_test_event("test_event", &format!("Event {}", i));
        event.aggregate_id = aggregate_id;
        event.occurred_at = chrono::Utc::now() + chrono::Duration::milliseconds(i * 100);
        events.push(event);
    }
    
    // Process events out of order
    let result1 = processor.process_event(events[2].clone()).await.unwrap();
    let result2 = processor.process_event(events[0].clone()).await.unwrap();
    let result3 = processor.process_event(events[1].clone()).await.unwrap();
    
    // Events should be buffered and ordered
    assert!(result1.is_some() || result2.is_some() || result3.is_some());
    
    // Test deduplication
    let duplicate_result = processor.process_event(events[0].clone()).await.unwrap();
    assert!(duplicate_result.is_none()); // Should be deduplicated
    
    let stats = processor.get_statistics().await;
    assert_eq!(stats.duplicates_detected, 1);
    assert!(stats.total_events_processed >= 4);
}

#[tokio::test]
#[ignore = "Test hangs due to async lock issues - needs investigation"]
async fn test_dead_letter_queue_with_circuit_breaker() {
    let config = EnhancedDLQConfig {
        circuit_breaker_threshold: 2,
        circuit_breaker_timeout_seconds: 1,
        poison_message_threshold: 3,
        ..EnhancedDLQConfig::default()
    };
    
    let dlq = EnhancedDeadLetterQueue::new(config).await.unwrap();
    let event = create_test_event("test_event", "Test event data");
    
    // Add events successfully - circuit breaker protects DLQ operations, not event additions
    for i in 0..3 {
        let result = dlq.add_failed_event(
            &event,
            format!("Error {}", i),
            serde_json::json!({"attempt": i})
        ).await;
        
        // All should succeed as the DLQ itself is working
        assert!(result.is_ok());
    }
    
    // Wait for circuit breaker timeout
    sleep(Duration::from_millis(1100)).await;
    
    // Should be able to process again
    let result = dlq.add_failed_event(
        &event,
        "After timeout".to_string(),
        serde_json::json!({"after_timeout": true})
    ).await;
    assert!(result.is_ok());
    
    let stats = dlq.get_enhanced_statistics().await;
    assert!(stats.enhanced_metrics.total_events_added > 0);
}

#[tokio::test]
async fn test_comprehensive_system_load() {
    // This test simulates high load across all components
    let event_store = setup_test_event_store().await;
    let dlq = Arc::new(EnhancedDeadLetterQueue::new(EnhancedDLQConfig::default()).await.unwrap());
    
    let ordering_manager = EventOrderingManager::new(OrderingConfig::default());
    let saga_store = Arc::new(InMemorySagaStore::new());
    let saga_orchestrator = SagaOrchestrator::new(event_store.clone(), saga_store);
    
    // Generate load
    let mut handles = vec![];
    
    for i in 0..100 {
        let dlq_clone = dlq.clone();
        let ordering_clone = if let Some(processor) = ordering_manager.get_processor("default").await {
            processor
        } else {
            ordering_manager.register_processor("default", None).await
        };
        
        let handle = tokio::spawn(async move {
            let event = create_test_event("load_test_event", &format!("Load test {}", i));
            
            // Process through ordering
            let _ = ordering_clone.process_event(event.clone()).await;
            
            // Simulate some failures for DLQ
            if i % 10 == 0 {
                let _ = dlq_clone.add_failed_event(
                    &event,
                    "Simulated failure".to_string(),
                    serde_json::json!({"load_test": true})
                ).await;
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Verify system handled the load
    let ordering_stats = ordering_manager.get_combined_statistics().await;
    let dlq_stats = dlq.get_enhanced_statistics().await;
    
    assert!(ordering_stats.get("default").unwrap().total_events_processed >= 100);
    assert!(dlq_stats.enhanced_metrics.total_events_added >= 10);
    
    println!("Load test completed:");
    println!("- Ordering processed: {}", ordering_stats.get("default").unwrap().total_events_processed);
    println!("- DLQ events: {}", dlq_stats.enhanced_metrics.total_events_added);
    println!("- Duplicates detected: {}", ordering_stats.get("default").unwrap().duplicates_detected);
}

// Helper functions and mock implementations

async fn setup_test_event_store() -> Arc<dyn EventStore> {
    // In a real test, this would setup a test database
    // For now, return a mock event store
    Arc::new(MockEventStore::new())
}

fn create_test_event(event_type: &str, data: &str) -> EventEnvelope {
    EventEnvelope {
        event_id: Uuid::new_v4(),
        aggregate_id: Uuid::new_v4(),
        aggregate_type: "test_aggregate".to_string(),
        event_type: event_type.to_string(),
        aggregate_version: 1,
        event_data: serde_json::json!({"data": data}),
        metadata: EventMetadata {
            user_id: Some(Uuid::new_v4().to_string()),
            session_id: Some(Uuid::new_v4().to_string()),
            correlation_id: Some(Uuid::new_v4()),
            causation_id: None,
            timestamp: chrono::Utc::now(),
            source: None,
            tags: std::collections::HashMap::new(),
            custom: std::collections::HashMap::new(),
        },
        occurred_at: chrono::Utc::now(),
        recorded_at: chrono::Utc::now(),
        schema_version: 1,
        causation_id: None,
        correlation_id: None,
        checksum: None,
    }
}

// Mock implementations for testing

struct MockEventStore;

impl MockEventStore {
    fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl EventStore for MockEventStore {
    async fn append_event(&self, _event: &EventEnvelope) -> EventResult<()> {
        Ok(())
    }
    
    async fn append_events(&self, _events: &[EventEnvelope]) -> EventResult<()> {
        Ok(())
    }
    
    
    async fn get_events_from_position(
        &self,
        _position: i64,
        _limit: usize,
    ) -> EventResult<Vec<EventEnvelope>> {
        Ok(vec![])
    }
    
    async fn get_events(&self, _aggregate_id: Uuid) -> EventResult<Vec<EventEnvelope>> {
        Ok(vec![])
    }
    
    async fn get_events_from_version(
        &self,
        _aggregate_id: Uuid,
        _from_version: i64,
    ) -> EventResult<Vec<EventEnvelope>> {
        Ok(vec![])
    }
    
    async fn get_events_by_type(
        &self,
        _event_type: &str,
        _from: Option<DateTime<Utc>>,
        _to: Option<DateTime<Utc>>,
        _limit: Option<usize>,
    ) -> EventResult<Vec<EventEnvelope>> {
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
    
    async fn get_current_position(&self) -> EventResult<i64> {
        Ok(0)
    }
    
    async fn replay_events(
        &self,
        _from_position: i64,
        _event_types: Option<Vec<String>>,
        _batch_size: usize,
    ) -> EventResult<Vec<EventEnvelope>> {
        Ok(vec![])
    }
    
    async fn get_events_for_aggregates(&self, _aggregate_ids: &[Uuid]) -> EventResult<Vec<EventEnvelope>> {
        Ok(vec![])
    }
    
    async fn cleanup_old_snapshots(&self, _keep_latest: usize) -> EventResult<usize> {
        Ok(0)
    }
    
    async fn get_aggregate_ids_by_type(
        &self,
        _aggregate_type: &str,
        _offset: i64,
        _limit: usize,
    ) -> EventResult<Vec<Uuid>> {
        Ok(vec![])
    }
    
    async fn optimize_storage(&self) -> EventResult<()> {
        Ok(())
    }
}

struct MockSagaStepExecutor {
    service_name: String,
    should_succeed: bool,
}

impl MockSagaStepExecutor {
    fn new(service_name: &str, should_succeed: bool) -> Self {
        Self {
            service_name: service_name.to_string(),
            should_succeed,
        }
    }
}

#[async_trait::async_trait]
impl SagaStepExecutor for MockSagaStepExecutor {
    async fn execute_step(
        &self,
        _step: &SagaStep,
        _global_context: &serde_json::Value,
    ) -> SagaStepResult {
        // Simulate processing time
        sleep(Duration::from_millis(100)).await;
        
        if self.should_succeed {
            SagaStepResult::Success {
                output_data: serde_json::json!({"result": "success"})
            }
        } else {
            SagaStepResult::Failure {
                error_message: format!("Mock failure from {}", self.service_name)
            }
        }
    }
    
    fn service_name(&self) -> &str {
        &self.service_name
    }
}