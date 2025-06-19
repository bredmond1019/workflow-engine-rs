// Integration tests for service isolation and multi-tenancy

use ai_system_rust::db::service_isolation::*;
use ai_system_rust::db::tenant::*;
use ai_system_rust::db::connection_pool::*;
use ai_system_rust::core::error::WorkflowError;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::pg::PgConnection;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// Mock event bus for testing
struct MockEventBus {
    published_events: Arc<RwLock<Vec<CrossServiceEvent>>>,
}

#[async_trait::async_trait]
impl CrossServiceEventBus for MockEventBus {
    async fn publish(&self, event: &CrossServiceEvent) -> Result<(), ServiceIsolationError> {
        self.published_events.write().await.push(event.clone());
        Ok(())
    }
    
    async fn subscribe(&self, topics: Vec<String>) -> Result<Box<dyn CrossServiceEventStream>, ServiceIsolationError> {
        Ok(Box::new(MockEventStream::new()))
    }
    
    async fn get_event_history(&self, service: &str, limit: usize) -> Result<Vec<CrossServiceEvent>, ServiceIsolationError> {
        let events = self.published_events.read().await;
        Ok(events
            .iter()
            .filter(|e| e.source_service == service)
            .take(limit)
            .cloned()
            .collect())
    }
}

struct MockEventStream;

impl MockEventStream {
    fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl CrossServiceEventStream for MockEventStream {
    async fn next_event(&mut self) -> Option<Result<CrossServiceEvent, ServiceIsolationError>> {
        None
    }
    
    async fn close(&mut self) -> Result<(), ServiceIsolationError> {
        Ok(())
    }
}

// Mock event store for testing
struct MockEventStore;

#[async_trait::async_trait]
impl ai_system_rust::db::events::EventStore for MockEventStore {
    async fn append_events(
        &self,
        aggregate_id: Uuid,
        events: Vec<ai_system_rust::db::events::EventEnvelope>,
        expected_version: Option<i64>,
    ) -> Result<(), ai_system_rust::db::events::EventError> {
        Ok(())
    }

    async fn load_events(
        &self,
        aggregate_id: Uuid,
        from_version: Option<i64>,
        to_version: Option<i64>,
    ) -> Result<Vec<ai_system_rust::db::events::EventEnvelope>, ai_system_rust::db::events::EventError> {
        Ok(vec![])
    }

    async fn load_events_by_type(
        &self,
        event_type: &str,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<ai_system_rust::db::events::EventEnvelope>, ai_system_rust::db::events::EventError> {
        Ok(vec![])
    }

    async fn get_last_version(
        &self,
        aggregate_id: Uuid,
    ) -> Result<Option<i64>, ai_system_rust::db::events::EventError> {
        Ok(None)
    }

    async fn create_snapshot(
        &self,
        aggregate_id: Uuid,
        version: i64,
        data: serde_json::Value,
    ) -> Result<(), ai_system_rust::db::events::EventError> {
        Ok(())
    }

    async fn load_snapshot(
        &self,
        aggregate_id: Uuid,
    ) -> Result<Option<ai_system_rust::db::events::EventSnapshot>, ai_system_rust::db::events::EventError> {
        Ok(None)
    }

    async fn subscribe(
        &self,
        subscription_id: &str,
        event_types: Vec<String>,
        position: Option<i64>,
    ) -> ai_system_rust::db::events::EventResult<Box<dyn ai_system_rust::db::events::EventStream>> {
        // Mock implementation for testing - returns error indicating subscription not supported
        Err(ai_system_rust::db::events::EventError::ConfigurationError(
            format!("Mock event store does not support subscriptions for {}", subscription_id)
        ))
    }

    async fn acknowledge(
        &self,
        subscription_id: &str,
        position: i64,
    ) -> ai_system_rust::db::events::EventResult<()> {
        Ok(())
    }
}

fn setup_test_pool() -> Result<Arc<Pool<ConnectionManager<PgConnection>>>, WorkflowError> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost/ai_workflow_test".to_string());
    
    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    let pool = Pool::builder()
        .max_size(5)
        .build(manager)
        .map_err(|e| WorkflowError::Database(e.to_string()))?;
    
    Ok(Arc::new(pool))
}

#[tokio::test]
#[ignore] // Requires database
async fn test_tenant_creation_and_isolation() {
    let pool = setup_test_pool().expect("Failed to create pool");
    let tenant_manager = Arc::new(TenantManager::new(pool.clone()));
    
    // Create a new tenant
    let new_tenant = NewTenant {
        name: "Test Company".to_string(),
        isolation_mode: TenantIsolationMode::RowLevel,
        settings: Some(serde_json::json!({
            "max_users": 100,
            "features": ["analytics", "api_access"]
        })),
    };
    
    let tenant = tenant_manager.create_tenant(new_tenant).await
        .expect("Failed to create tenant");
    
    assert_eq!(tenant.name, "Test Company");
    assert_eq!(tenant.isolation_mode, "row_level");
    assert!(tenant.is_active);
    
    // Get tenant by ID
    let retrieved_tenant = tenant_manager.get_tenant(tenant.id).await
        .expect("Failed to get tenant");
    
    assert_eq!(retrieved_tenant.id, tenant.id);
    assert_eq!(retrieved_tenant.name, tenant.name);
    
    // Test tenant connection
    let tenant_conn = tenant_manager.get_tenant_connection(tenant.id).await
        .expect("Failed to get tenant connection");
    
    assert_eq!(tenant_conn.context.tenant_id, tenant.id);
}

#[tokio::test]
#[ignore] // Requires database
async fn test_service_boundary_registration() {
    let pool = setup_test_pool().expect("Failed to create pool");
    let tenant_manager = Arc::new(TenantManager::new(pool.clone()));
    let connection_pool_manager = Arc::new(ConnectionPoolManager::new(
        tenant_manager.clone(),
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgresql://localhost/test".to_string()),
    ));
    
    let event_bus = Arc::new(MockEventBus {
        published_events: Arc::new(RwLock::new(Vec::new())),
    });
    let event_store = Arc::new(MockEventStore);
    
    let mut isolation_manager = ServiceIsolationManager::new(
        event_store,
        event_bus.clone(),
        tenant_manager,
        connection_pool_manager,
    );
    
    // Create service boundary
    let boundary = ServiceBoundary {
        service_name: "content_processing".to_string(),
        database_url: std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/content_processing".to_string()),
        database_type: DatabaseType::PostgreSQL {
            schema: "content".to_string(),
            connection_pool_size: 20,
        },
        allowed_operations: vec![
            ServiceOperation::Read,
            ServiceOperation::Write,
            ServiceOperation::Monitor,
        ],
        event_topics: vec!["content.created".to_string(), "content.updated".to_string()],
        resource_limits: ResourceLimits {
            max_connections: 20,
            max_query_time_seconds: 30,
            max_result_size_mb: 100,
            max_concurrent_operations: 50,
            rate_limit_per_second: 100,
        },
        isolation_level: IsolationLevel::Schema,
    };
    
    isolation_manager.register_service(boundary.clone()).await
        .expect("Failed to register service");
    
    // Verify service was registered
    let retrieved_boundary = isolation_manager.get_service_boundary("content_processing")
        .expect("Service not found");
    
    assert_eq!(retrieved_boundary.service_name, "content_processing");
    assert_eq!(retrieved_boundary.isolation_level, IsolationLevel::Schema);
    
    // Check that registration event was published
    let events = event_bus.published_events.read().await;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "service_registered");
}

#[tokio::test]
#[ignore] // Requires database
async fn test_service_connection_pooling() {
    let pool = setup_test_pool().expect("Failed to create pool");
    let tenant_manager = Arc::new(TenantManager::new(pool.clone()));
    let connection_pool_manager = Arc::new(ConnectionPoolManager::new(
        tenant_manager.clone(),
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgresql://localhost/test".to_string()),
    ));
    
    // Register a service pool
    let config = ServicePoolConfig::new(
        "test_service".to_string(),
        std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/test".to_string()),
    )
    .with_pool_size(5, 20);
    
    connection_pool_manager.register_service_pool(config).await
        .expect("Failed to register service pool");
    
    // Get connections
    let conn1 = connection_pool_manager.get_service_connection("test_service").await
        .expect("Failed to get connection 1");
    let conn2 = connection_pool_manager.get_service_connection("test_service").await
        .expect("Failed to get connection 2");
    
    // Test connections
    diesel::sql_query("SELECT 1")
        .execute(&*conn1)
        .expect("Failed to execute query on conn1");
    
    diesel::sql_query("SELECT 1")
        .execute(&*conn2)
        .expect("Failed to execute query on conn2");
    
    // Get metrics
    let metrics = connection_pool_manager.get_all_metrics().await;
    let service_metrics = metrics.get("test_service").expect("No metrics for service");
    
    assert!(service_metrics.total_connections > 0);
}

#[tokio::test]
#[ignore] // Requires database
async fn test_tenant_data_isolation() {
    let pool = setup_test_pool().expect("Failed to create pool");
    let tenant_manager = Arc::new(TenantManager::new(pool.clone()));
    let connection_pool_manager = Arc::new(ConnectionPoolManager::new(
        tenant_manager.clone(),
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgresql://localhost/test".to_string()),
    ));
    
    // Create two tenants
    let tenant1 = tenant_manager.create_tenant(NewTenant {
        name: "Tenant 1".to_string(),
        isolation_mode: TenantIsolationMode::RowLevel,
        settings: None,
    }).await.expect("Failed to create tenant 1");
    
    let tenant2 = tenant_manager.create_tenant(NewTenant {
        name: "Tenant 2".to_string(),
        isolation_mode: TenantIsolationMode::RowLevel,
        settings: None,
    }).await.expect("Failed to create tenant 2");
    
    // Register service pool
    let config = ServicePoolConfig::new(
        "isolated_service".to_string(),
        std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/test".to_string()),
    );
    
    connection_pool_manager.register_service_pool(config).await
        .expect("Failed to register service pool");
    
    // Get tenant-specific connections
    let mut conn1 = connection_pool_manager
        .get_tenant_service_connection("isolated_service", tenant1.id)
        .await
        .expect("Failed to get tenant 1 connection");
    
    let mut conn2 = connection_pool_manager
        .get_tenant_service_connection("isolated_service", tenant2.id)
        .await
        .expect("Failed to get tenant 2 connection");
    
    // Verify isolation contexts
    assert_eq!(conn1.context.tenant_id, tenant1.id);
    assert_eq!(conn2.context.tenant_id, tenant2.id);
    
    // Test that tenant context is applied
    let result1 = conn1.with_tenant_context(|conn, ctx| {
        assert_eq!(ctx.tenant_id, tenant1.id);
        Ok(())
    });
    assert!(result1.is_ok());
    
    let result2 = conn2.with_tenant_context(|conn, ctx| {
        assert_eq!(ctx.tenant_id, tenant2.id);
        Ok(())
    });
    assert!(result2.is_ok());
}

#[tokio::test]
async fn test_cross_service_event_creation() {
    let event = CrossServiceEvent::new(
        "service_a".to_string(),
        "data_processed".to_string(),
        "processing".to_string(),
        serde_json::json!({"record_id": "123", "status": "completed"}),
    )
    .with_target("service_b".to_string())
    .with_correlation_id(Uuid::new_v4())
    .add_metadata("priority".to_string(), "high".to_string());
    
    assert_eq!(event.source_service, "service_a");
    assert_eq!(event.event_type, "data_processed");
    assert_eq!(event.topic, "processing");
    assert_eq!(event.target_service, Some("service_b".to_string()));
    assert!(event.correlation_id.is_some());
    assert_eq!(event.metadata.get("priority"), Some(&"high".to_string()));
}

#[tokio::test]
async fn test_resource_limits() {
    let limits = ResourceLimits {
        max_connections: 100,
        max_query_time_seconds: 60,
        max_result_size_mb: 500,
        max_concurrent_operations: 200,
        rate_limit_per_second: 1000,
    };
    
    assert_eq!(limits.max_connections, 100);
    assert_eq!(limits.max_query_time_seconds, 60);
    assert_eq!(limits.max_result_size_mb, 500);
    assert_eq!(limits.max_concurrent_operations, 200);
    assert_eq!(limits.rate_limit_per_second, 1000);
    
    let default_limits = ResourceLimits::default();
    assert_eq!(default_limits.max_connections, 20);
    assert_eq!(default_limits.max_query_time_seconds, 30);
}