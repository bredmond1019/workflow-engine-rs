// File: tests/event_versioning_tests.rs
//
// Integration tests for event versioning and migration system
// Tests schema evolution and migration functionality

use backend::db::events::*;
use backend::db::events::types::*;
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[cfg(test)]
mod versioning_tests {
    use super::*;

    fn create_test_event(event_type: &str, version: i32, data: serde_json::Value) -> EventEnvelope {
        EventEnvelope {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            aggregate_type: "test_aggregate".to_string(),
            event_type: event_type.to_string(),
            aggregate_version: 1,
            event_data: data,
            metadata: EventMetadata::new(),
            occurred_at: Utc::now(),
            recorded_at: Utc::now(),
            schema_version: version,
            causation_id: None,
            correlation_id: None,
            checksum: None,
        }
    }

    #[tokio::test]
    async fn test_schema_version_registration_and_retrieval() {
        let config = VersioningConfig::default();
        let version_manager = EventVersionManager::new(config);

        // Register multiple schema versions
        let schema_v1 = SchemaVersion::new(
            "test_event".to_string(),
            1,
            "Initial version".to_string(),
        );
        
        let schema_v2 = SchemaVersion::new(
            "test_event".to_string(),
            2,
            "Added user context".to_string(),
        );
        
        let schema_v3 = SchemaVersion::new(
            "test_event".to_string(),
            3,
            "Enhanced metadata".to_string(),
        ).with_migration_notes("Requires user_id field".to_string());

        // Register versions
        version_manager.register_schema_version(schema_v1).await.unwrap();
        version_manager.register_schema_version(schema_v2).await.unwrap();
        version_manager.register_schema_version(schema_v3).await.unwrap();

        // Test retrieval
        let latest_version = version_manager.get_latest_version("test_event").await;
        assert_eq!(latest_version, Some(3));

        let all_versions = version_manager.get_schema_versions("test_event").await;
        assert_eq!(all_versions.len(), 3);
        assert_eq!(all_versions[0].version, 1);
        assert_eq!(all_versions[2].version, 3);
        assert!(all_versions[2].migration_notes.is_some());

        // Test version support checks
        assert!(version_manager.is_version_supported("test_event", 1).await);
        assert!(version_manager.is_version_supported("test_event", 2).await);
        assert!(version_manager.is_version_supported("test_event", 3).await);
        assert!(!version_manager.is_version_supported("test_event", 4).await);
        assert!(!version_manager.is_version_supported("nonexistent_event", 1).await);
    }

    #[tokio::test]
    async fn test_simple_event_migration() {
        let config = VersioningConfig::default();
        let version_manager = EventVersionManager::new(config);

        // Register schema versions
        let schema_v1 = SchemaVersion::new("test_event".to_string(), 1, "V1".to_string());
        let schema_v2 = SchemaVersion::new("test_event".to_string(), 2, "V2".to_string());
        
        version_manager.register_schema_version(schema_v1).await.unwrap();
        version_manager.register_schema_version(schema_v2).await.unwrap();

        // Register a simple migrator
        let migrator = FieldRenameMigration::new(
            "test_event".to_string(),
            1,
            2,
            {
                let mut renames = HashMap::new();
                renames.insert("old_field".to_string(), "new_field".to_string());
                renames
            },
        );
        version_manager.register_migrator(migrator).await.unwrap();

        // Create a v1 event
        let v1_event = create_test_event(
            "test_event",
            1,
            json!({"old_field": "test_value", "unchanged": "stays_same"}),
        );

        // Migrate to latest version
        let migrated_event = version_manager.migrate_to_latest(v1_event).await.unwrap();

        assert_eq!(migrated_event.schema_version, 2);
        assert_eq!(migrated_event.event_data["new_field"], "test_value");
        assert_eq!(migrated_event.event_data["unchanged"], "stays_same");
        assert!(migrated_event.event_data["old_field"].is_null());
    }

    #[tokio::test]
    async fn test_workflow_event_migrations() {
        let config = VersioningConfig::default();
        let version_manager = EventVersionManager::new(config);

        // Register workflow migrations
        let registry = MigrationRegistry::workflow_migrations();
        
        for schema_version in registry.schema_versions() {
            version_manager.register_schema_version(schema_version.clone()).await.unwrap();
        }
        
        for migration in registry.migrations() {
            // We can't move out of the Box, so we need to register each migration type manually
            // In practice, this would be done through a registration helper
        }

        // Register the specific migrations we need
        version_manager.register_migrator(WorkflowStartedV1ToV2Migration).await.unwrap();

        // Create a v1 workflow_started event
        let v1_workflow_event = create_test_event(
            "workflow_started",
            1,
            json!({
                "workflow_id": "wf_123",
                "workflow_type": "data_processing",
                "configuration": {
                    "timeout": 300,
                    "retries": 3
                },
                "input_data": {
                    "source": "file.txt",
                    "format": "csv"
                }
            }),
        );

        // Migrate to v2
        let migrated_event = version_manager.migrate_to_version(v1_workflow_event, 2).await.unwrap();

        assert_eq!(migrated_event.schema_version, 2);
        
        // Check that user_context was added
        assert!(migrated_event.event_data["user_context"].is_object());
        assert!(migrated_event.event_data["user_context"]["user_id"].is_null());
        
        // Check that metadata was enhanced
        assert!(migrated_event.event_data["metadata"].is_object());
        assert_eq!(migrated_event.event_data["metadata"]["priority"], "normal");
        assert_eq!(migrated_event.event_data["metadata"]["retry_config"]["max_retries"], 3);
        
        // Original data should be preserved
        assert_eq!(migrated_event.event_data["workflow_id"], "wf_123");
        assert_eq!(migrated_event.event_data["configuration"]["timeout"], 300);
    }

    #[tokio::test]
    async fn test_ai_interaction_event_migrations() {
        let config = VersioningConfig::default();
        let version_manager = EventVersionManager::new(config);

        // Register AI interaction migrations
        let registry = MigrationRegistry::ai_interaction_migrations();
        
        for schema_version in registry.schema_versions() {
            version_manager.register_schema_version(schema_version.clone()).await.unwrap();
        }

        version_manager.register_migrator(PromptSentV1ToV2Migration).await.unwrap();
        version_manager.register_migrator(ResponseReceivedV1ToV2Migration).await.unwrap();

        // Test PromptSent v1 to v2 migration
        let v1_prompt_event = create_test_event(
            "prompt_sent",
            1,
            json!({
                "request_id": "req_456",
                "model": "gpt-4",
                "provider": "openai",
                "prompt": "Write a comprehensive test plan for an AI system. Include unit tests, integration tests, and performance benchmarks.",
                "parameters": {
                    "temperature": 0.7,
                    "max_tokens": 2000
                }
            }),
        );

        let migrated_prompt = version_manager.migrate_to_latest(v1_prompt_event).await.unwrap();
        
        assert_eq!(migrated_prompt.schema_version, 2);
        assert!(migrated_prompt.event_data["estimated_tokens"].is_object());
        assert!(migrated_prompt.event_data["estimated_tokens"]["prompt_tokens"].is_number());
        assert!(migrated_prompt.event_data["model_parameters"]["temperature"].is_number());
        assert!(migrated_prompt.event_data["cost_estimation"].is_object());

        // Test ResponseReceived v1 to v2 migration
        let v1_response_event = create_test_event(
            "response_received",
            1,
            json!({
                "request_id": "req_456",
                "response": "Here's a comprehensive test plan:\n\n1. Unit Tests...",
                "completion_tokens": 1500,
                "prompt_tokens": 50,
                "total_tokens": 1550,
                "cost_usd": 0.062,
                "duration_ms": 2500,
                "model": "gpt-4",
                "provider": "openai"
            }),
        );

        let migrated_response = version_manager.migrate_to_latest(v1_response_event).await.unwrap();
        
        assert_eq!(migrated_response.schema_version, 2);
        assert!(migrated_response.event_data["detailed_usage"].is_object());
        assert_eq!(migrated_response.event_data["detailed_usage"]["prompt_tokens"], 50);
        assert_eq!(migrated_response.event_data["detailed_usage"]["completion_tokens"], 1500);
        assert!(migrated_response.event_data["cost_breakdown"].is_object());
        assert_eq!(migrated_response.event_data["cost_breakdown"]["total_cost_usd"], 0.062);
        assert!(migrated_response.event_data["quality_metrics"].is_object());
        assert!(migrated_response.event_data["performance_metrics"].is_object());
    }

    #[tokio::test]
    async fn test_migration_chain() {
        let config = VersioningConfig::default();
        let version_manager = EventVersionManager::new(config);

        // Register schema versions 1, 2, 3
        for version in 1..=3 {
            let schema = SchemaVersion::new(
                "chain_event".to_string(),
                version,
                format!("Version {}", version),
            );
            version_manager.register_schema_version(schema).await.unwrap();
        }

        // Register migrations 1->2 and 2->3
        let migrator_1_2 = FieldRenameMigration::new(
            "chain_event".to_string(),
            1,
            2,
            {
                let mut renames = HashMap::new();
                renames.insert("v1_field".to_string(), "v2_field".to_string());
                renames
            },
        );

        let migrator_2_3 = FieldRenameMigration::new(
            "chain_event".to_string(),
            2,
            3,
            {
                let mut renames = HashMap::new();
                renames.insert("v2_field".to_string(), "v3_field".to_string());
                renames
            },
        );

        version_manager.register_migrator(migrator_1_2).await.unwrap();
        version_manager.register_migrator(migrator_2_3).await.unwrap();

        // Create v1 event
        let v1_event = create_test_event(
            "chain_event",
            1,
            json!({"v1_field": "test_value", "stable_field": "unchanged"}),
        );

        // Migrate to v3 (should go through v2)
        let migrated_event = version_manager.migrate_to_version(v1_event, 3).await.unwrap();

        assert_eq!(migrated_event.schema_version, 3);
        assert_eq!(migrated_event.event_data["v3_field"], "test_value");
        assert_eq!(migrated_event.event_data["stable_field"], "unchanged");
        assert!(migrated_event.event_data["v1_field"].is_null());
        assert!(migrated_event.event_data["v2_field"].is_null());
    }

    #[tokio::test]
    async fn test_migration_caching() {
        let config = VersioningConfig {
            cache_migrations: true,
            migration_cache_size: 10,
            ..Default::default()
        };
        let version_manager = EventVersionManager::new(config);

        // Register schema and migration
        let schema_v1 = SchemaVersion::new("cached_event".to_string(), 1, "V1".to_string());
        let schema_v2 = SchemaVersion::new("cached_event".to_string(), 2, "V2".to_string());
        
        version_manager.register_schema_version(schema_v1).await.unwrap();
        version_manager.register_schema_version(schema_v2).await.unwrap();

        let migrator = FieldRenameMigration::new(
            "cached_event".to_string(),
            1,
            2,
            {
                let mut renames = HashMap::new();
                renames.insert("old".to_string(), "new".to_string());
                renames
            },
        );
        version_manager.register_migrator(migrator).await.unwrap();

        // Create event with specific ID for caching
        let event_id = Uuid::new_v4();
        let mut event = create_test_event("cached_event", 1, json!({"old": "value"}));
        event.event_id = event_id;

        // First migration (cache miss)
        let migrated1 = version_manager.migrate_to_latest(event.clone()).await.unwrap();
        assert_eq!(migrated1.event_data["new"], "value");

        // Second migration of same event (cache hit)
        let migrated2 = version_manager.migrate_to_latest(event.clone()).await.unwrap();
        assert_eq!(migrated2.event_data["new"], "value");

        // Check cache statistics
        let stats = version_manager.get_statistics().await;
        assert!(stats.cache_hits > 0);
        assert!(stats.cache_misses > 0);
    }

    #[tokio::test]
    async fn test_migration_error_handling() {
        let config = VersioningConfig {
            strict_validation: true,
            ..Default::default()
        };
        let version_manager = EventVersionManager::new(config);

        // Try to migrate event with no registered schema
        let event = create_test_event("unknown_event", 1, json!({"data": "test"}));
        let result = version_manager.migrate_to_latest(event).await;
        assert!(result.is_err());

        // Register only v1 schema (no migration path to v2)
        let schema_v1 = SchemaVersion::new("orphan_event".to_string(), 1, "V1".to_string());
        version_manager.register_schema_version(schema_v1).await.unwrap();

        let orphan_event = create_test_event("orphan_event", 1, json!({"data": "test"}));
        
        // Should succeed (already at latest version)
        let result = version_manager.migrate_to_latest(orphan_event).await;
        assert!(result.is_ok());

        // Try to migrate to non-existent version
        let event = create_test_event("orphan_event", 1, json!({"data": "test"}));
        let result = version_manager.migrate_to_version(event, 99).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_versioning_statistics() {
        let config = VersioningConfig::default();
        let version_manager = EventVersionManager::new(config);

        // Register schema and migration
        let schema_v1 = SchemaVersion::new("stats_event".to_string(), 1, "V1".to_string());
        let schema_v2 = SchemaVersion::new("stats_event".to_string(), 2, "V2".to_string());
        
        version_manager.register_schema_version(schema_v1).await.unwrap();
        version_manager.register_schema_version(schema_v2).await.unwrap();

        let migrator = FieldRenameMigration::new(
            "stats_event".to_string(),
            1,
            2,
            {
                let mut renames = HashMap::new();
                renames.insert("old".to_string(), "new".to_string());
                renames
            },
        );
        version_manager.register_migrator(migrator).await.unwrap();

        // Perform several migrations
        for i in 0..5 {
            let event = create_test_event(
                "stats_event",
                1,
                json!({"old": format!("value_{}", i)}),
            );
            let _migrated = version_manager.migrate_to_latest(event).await.unwrap();
        }

        // Check statistics
        let stats = version_manager.get_statistics().await;
        assert_eq!(stats.successful_migrations, 5);
        assert_eq!(stats.failed_migrations, 0);
        assert!(stats.migrations_by_type.contains_key("stats_event"));
        assert_eq!(stats.migrations_by_type["stats_event"], 5);
        assert!(stats.average_migration_time_ms > 0.0);
    }

    #[test]
    fn test_schema_version_deprecation() {
        let schema = SchemaVersion::new(
            "deprecated_event".to_string(),
            1,
            "Old version".to_string(),
        ).deprecate(Some("Use version 2 instead".to_string()));

        assert!(schema.is_deprecated());
        assert!(schema.deprecated_at.is_some());
        assert_eq!(schema.migration_notes, Some("Use version 2 instead".to_string()));
    }

    #[test]
    fn test_migration_registry_creation() {
        let registry = MigrationRegistry::workflow_migrations();
        let migrations = registry.migrations();
        let schemas = registry.schema_versions();

        assert!(!migrations.is_empty());
        assert!(!schemas.is_empty());

        // Check that we have expected workflow schemas
        let has_workflow_started = schemas.iter().any(|s| s.event_type == "workflow_started");
        let has_workflow_completed = schemas.iter().any(|s| s.event_type == "workflow_completed");
        
        assert!(has_workflow_started);
        assert!(has_workflow_completed);
    }
}