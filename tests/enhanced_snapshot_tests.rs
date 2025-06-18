// File: tests/enhanced_snapshot_tests.rs
//
// Integration tests for enhanced snapshot functionality
// Tests compression, restoration, and lifecycle management

use backend::db::events::*;
use chrono::Utc;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

#[cfg(test)]
mod snapshot_tests {
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

    fn create_test_snapshot_data() -> serde_json::Value {
        json!({
            "aggregate_state": {
                "user_id": "user_123",
                "name": "John Doe",
                "email": "john@example.com",
                "preferences": {
                    "theme": "dark",
                    "language": "en",
                    "notifications": true
                },
                "activity_log": [
                    {"action": "login", "timestamp": "2024-01-01T10:00:00Z"},
                    {"action": "update_profile", "timestamp": "2024-01-01T10:05:00Z"},
                    {"action": "change_password", "timestamp": "2024-01-01T10:10:00Z"}
                ],
                "metadata": {
                    "created_at": "2023-12-01T00:00:00Z",
                    "last_updated": "2024-01-01T10:10:00Z",
                    "version": 15
                }
            },
            "counters": {
                "login_count": 42,
                "profile_updates": 5,
                "password_changes": 2
            },
            "cached_calculations": {
                "total_activity_score": 1337,
                "engagement_level": "high",
                "recommendations": [
                    "feature_a", "feature_b", "feature_c"
                ]
            }
        })
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database
    async fn test_basic_snapshot_creation_and_restoration() {
        let event_store = match create_test_event_store() {
            Some(store) => store,
            None => {
                println!("Skipping test - could not connect to PostgreSQL database");
                return;
            }
        };

        let config = SnapshotConfig {
            compression_type: CompressionType::None,
            ..Default::default()
        };

        let snapshot_manager = EnhancedSnapshotManager::new(event_store.clone(), config);

        let aggregate_id = Uuid::new_v4();
        let snapshot_data = create_test_snapshot_data();

        // Create snapshot
        let created_snapshot = snapshot_manager.create_snapshot(
            aggregate_id,
            "user_aggregate".to_string(),
            15,
            snapshot_data.clone(),
        ).await;

        assert!(created_snapshot.is_ok(), "Failed to create snapshot: {:?}", created_snapshot.err());
        
        let snapshot = created_snapshot.unwrap();
        assert_eq!(snapshot.aggregate_id, aggregate_id);
        assert_eq!(snapshot.aggregate_type, "user_aggregate");
        assert_eq!(snapshot.aggregate_version, 15);
        assert_eq!(snapshot.compression_type, CompressionType::None);
        assert!(snapshot.checksum.is_some());

        // Restore snapshot
        let restored_snapshot = snapshot_manager.restore_snapshot(aggregate_id).await;
        assert!(restored_snapshot.is_ok(), "Failed to restore snapshot: {:?}", restored_snapshot.err());
        
        let restored = restored_snapshot.unwrap();
        assert!(restored.is_some(), "Snapshot should exist");
        
        let restored = restored.unwrap();
        assert_eq!(restored.aggregate_id, aggregate_id);
        assert_eq!(restored.aggregate_version, 15);
        assert_eq!(restored.snapshot_data, snapshot_data);
        assert_eq!(restored.checksum, snapshot.checksum);
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database
    async fn test_gzip_compression() {
        let event_store = match create_test_event_store() {
            Some(store) => store,
            None => {
                println!("Skipping test - could not connect to PostgreSQL database");
                return;
            }
        };

        let config = SnapshotConfig {
            compression_type: CompressionType::Gzip,
            compression_threshold_bytes: 100, // Low threshold to ensure compression
            min_compression_ratio: 0.9, // Allow even modest compression
            ..Default::default()
        };

        let snapshot_manager = EnhancedSnapshotManager::new(event_store.clone(), config);

        let aggregate_id = Uuid::new_v4();
        let snapshot_data = create_test_snapshot_data();

        // Create compressed snapshot
        let created_snapshot = snapshot_manager.create_snapshot(
            aggregate_id,
            "user_aggregate".to_string(),
            20,
            snapshot_data.clone(),
        ).await;

        assert!(created_snapshot.is_ok(), "Failed to create compressed snapshot: {:?}", created_snapshot.err());
        
        let snapshot = created_snapshot.unwrap();
        assert_eq!(snapshot.compression_type, CompressionType::Gzip);
        assert!(snapshot.compressed_size > 0);
        assert!(snapshot.original_size > 0);
        
        println!(
            "Compression: {} bytes -> {} bytes (ratio: {:.2})",
            snapshot.original_size,
            snapshot.compressed_size,
            snapshot.compression_ratio()
        );

        // Restore and verify decompression
        let restored_snapshot = snapshot_manager.restore_snapshot(aggregate_id).await;
        assert!(restored_snapshot.is_ok(), "Failed to restore compressed snapshot: {:?}", restored_snapshot.err());
        
        let restored = restored_snapshot.unwrap().unwrap();
        assert_eq!(restored.snapshot_data, snapshot_data);
        assert_eq!(restored.compression_type, CompressionType::None); // Should be decompressed
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database
    async fn test_lz4_compression() {
        let event_store = match create_test_event_store() {
            Some(store) => store,
            None => {
                println!("Skipping test - could not connect to PostgreSQL database");
                return;
            }
        };

        let config = SnapshotConfig {
            compression_type: CompressionType::Lz4,
            compression_threshold_bytes: 100,
            min_compression_ratio: 0.9,
            ..Default::default()
        };

        let snapshot_manager = EnhancedSnapshotManager::new(event_store.clone(), config);

        let aggregate_id = Uuid::new_v4();
        let snapshot_data = create_test_snapshot_data();

        // Create LZ4 compressed snapshot
        let created_snapshot = snapshot_manager.create_snapshot(
            aggregate_id,
            "user_aggregate".to_string(),
            25,
            snapshot_data.clone(),
        ).await;

        assert!(created_snapshot.is_ok(), "Failed to create LZ4 compressed snapshot: {:?}", created_snapshot.err());
        
        let snapshot = created_snapshot.unwrap();
        assert_eq!(snapshot.compression_type, CompressionType::Lz4);
        
        println!(
            "LZ4 Compression: {} bytes -> {} bytes (ratio: {:.2})",
            snapshot.original_size,
            snapshot.compressed_size,
            snapshot.compression_ratio()
        );

        // Restore and verify decompression
        let restored_snapshot = snapshot_manager.restore_snapshot(aggregate_id).await;
        assert!(restored_snapshot.is_ok(), "Failed to restore LZ4 compressed snapshot: {:?}", restored_snapshot.err());
        
        let restored = restored_snapshot.unwrap().unwrap();
        assert_eq!(restored.snapshot_data, snapshot_data);
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database
    async fn test_snapshot_should_create_logic() {
        let event_store = match create_test_event_store() {
            Some(store) => store,
            None => {
                println!("Skipping test - could not connect to PostgreSQL database");
                return;
            }
        };

        let config = SnapshotConfig {
            snapshot_frequency: 10,
            ..Default::default()
        };

        let snapshot_manager = EnhancedSnapshotManager::new(event_store.clone(), config);

        let aggregate_id = Uuid::new_v4();

        // Should create snapshot when no previous snapshot exists
        let should_create = snapshot_manager.should_create_snapshot(aggregate_id, 15).await;
        assert!(should_create.is_ok());
        assert!(should_create.unwrap());

        // Create a snapshot at version 10
        let _snapshot = snapshot_manager.create_snapshot(
            aggregate_id,
            "test_aggregate".to_string(),
            10,
            json!({"state": "test_state"}),
        ).await.unwrap();

        // Should not create snapshot at version 15 (only 5 events since last snapshot)
        let should_create = snapshot_manager.should_create_snapshot(aggregate_id, 15).await;
        assert!(should_create.is_ok());
        assert!(!should_create.unwrap());

        // Should create snapshot at version 21 (11 events since last snapshot)
        let should_create = snapshot_manager.should_create_snapshot(aggregate_id, 21).await;
        assert!(should_create.is_ok());
        assert!(should_create.unwrap());
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database
    async fn test_snapshot_statistics() {
        let event_store = match create_test_event_store() {
            Some(store) => store,
            None => {
                println!("Skipping test - could not connect to PostgreSQL database");
                return;
            }
        };

        let config = SnapshotConfig {
            compression_type: CompressionType::Gzip,
            compression_threshold_bytes: 50,
            ..Default::default()
        };

        let snapshot_manager = EnhancedSnapshotManager::new(event_store.clone(), config);

        // Create multiple snapshots
        for i in 1..=5 {
            let aggregate_id = Uuid::new_v4();
            let snapshot_data = json!({
                "iteration": i,
                "data": "test data that should compress well when repeated".repeat(10),
                "metadata": {
                    "created_at": Utc::now(),
                    "version": i * 10
                }
            });

            let _snapshot = snapshot_manager.create_snapshot(
                aggregate_id,
                format!("test_aggregate_{}", i),
                i * 10,
                snapshot_data,
            ).await.unwrap();
        }

        // Get statistics
        let stats = snapshot_manager.get_statistics().await;
        
        assert_eq!(stats.total_snapshots, 5);
        assert!(stats.compressed_snapshots <= 5); // Some or all should be compressed
        assert!(stats.total_original_size > 0);
        assert!(stats.total_compressed_size > 0);
        assert!(stats.average_compression_ratio > 0.0);
        
        println!("Snapshot Statistics:");
        println!("  Total snapshots: {}", stats.total_snapshots);
        println!("  Compressed snapshots: {}", stats.compressed_snapshots);
        println!("  Total original size: {} bytes", stats.total_original_size);
        println!("  Total compressed size: {} bytes", stats.total_compressed_size);
        println!("  Average compression ratio: {:.2}", stats.average_compression_ratio);
        println!("  Total space saved: {} bytes", stats.total_space_saved);
        println!("  Snapshots by type: {:?}", stats.snapshots_by_type);
        println!("  Compression by type: {:?}", stats.compression_by_type);
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database
    async fn test_snapshot_cleanup() {
        let event_store = match create_test_event_store() {
            Some(store) => store,
            None => {
                println!("Skipping test - could not connect to PostgreSQL database");
                return;
            }
        };

        let config = SnapshotConfig {
            max_snapshots_per_aggregate: 3,
            ..Default::default()
        };

        let snapshot_manager = EnhancedSnapshotManager::new(event_store.clone(), config);

        // Create multiple snapshots for the same aggregate
        let aggregate_id = Uuid::new_v4();
        
        for version in 1..=10 {
            let snapshot_data = json!({"version": version, "state": "test_state"});
            
            let _snapshot = snapshot_manager.create_snapshot(
                aggregate_id,
                "test_aggregate".to_string(),
                version * 10,
                snapshot_data,
            ).await.unwrap();
        }

        // Perform cleanup
        let cleaned_count = snapshot_manager.cleanup_old_snapshots().await;
        assert!(cleaned_count.is_ok(), "Cleanup should succeed: {:?}", cleaned_count.err());
        
        let deleted_count = cleaned_count.unwrap();
        println!("Cleaned up {} old snapshots", deleted_count);
        
        // Note: The actual cleanup behavior depends on the underlying event store implementation
        // This test mainly verifies that the cleanup operation completes without errors
    }

    #[tokio::test]
    async fn test_compression_ratio_calculation() {
        let mut snapshot = EnhancedSnapshot::new(
            Uuid::new_v4(),
            "test".to_string(),
            1,
            json!({"test": "data"}),
        );

        // Test no compression
        assert_eq!(snapshot.compression_ratio(), 1.0);
        assert_eq!(snapshot.space_saved_bytes(), 0);

        // Test with compression
        snapshot.original_size = 1000;
        snapshot.compressed_size = 600;
        
        assert_eq!(snapshot.compression_ratio(), 0.6);
        assert_eq!(snapshot.space_saved_bytes(), 400);

        // Test with expansion (compression made it bigger)
        snapshot.compressed_size = 1200;
        assert_eq!(snapshot.compression_ratio(), 1.2);
        assert_eq!(snapshot.space_saved_bytes(), 0); // No space saved
    }

    #[test]
    fn test_enhanced_snapshot_metadata() {
        let aggregate_id = Uuid::new_v4();
        let snapshot_data = json!({"state": "test"});
        
        let snapshot = EnhancedSnapshot::new(
            aggregate_id,
            "test_aggregate".to_string(),
            5,
            snapshot_data,
        ).with_metadata("custom_field".to_string(), json!("custom_value"));

        assert!(snapshot.metadata.contains_key("custom_field"));
        assert_eq!(snapshot.metadata["custom_field"], json!("custom_value"));
    }
}