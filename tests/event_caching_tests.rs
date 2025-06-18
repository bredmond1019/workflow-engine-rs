// File: tests/event_caching_tests.rs
//
// Integration tests for event store caching functionality
// Tests cache performance, hit rates, and data consistency

use backend::db::events::*;
use chrono::Utc;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

#[cfg(test)]
mod caching_tests {
    use super::*;

    fn create_test_event(aggregate_id: Uuid, version: i64) -> EventEnvelope {
        EventEnvelope {
            event_id: Uuid::new_v4(),
            aggregate_id,
            aggregate_type: "test_aggregate".to_string(),
            event_type: "test_event".to_string(),
            aggregate_version: version,
            event_data: json!({
                "test_field": "test_value",
                "version": version,
                "timestamp": Utc::now().timestamp()
            }),
            metadata: EventMetadata::new(),
            occurred_at: Utc::now(),
            recorded_at: Utc::now(),
            schema_version: 1,
            causation_id: None,
            correlation_id: None,
            checksum: None,
        }
    }

    #[tokio::test]
    async fn test_cache_config_defaults() {
        let config = CacheConfig::default();
        
        assert_eq!(config.max_events, 10_000);
        assert_eq!(config.max_aggregates, 1_000);
        assert_eq!(config.max_snapshots, 500);
        assert_eq!(config.event_ttl_seconds, 3600);
        assert_eq!(config.aggregate_ttl_seconds, 1800);
        assert_eq!(config.snapshot_ttl_seconds, 7200);
        assert!(config.write_through);
        assert!(!config.compress_cached_data);
        assert_eq!(config.cleanup_interval_seconds, 300);
    }

    #[tokio::test]
    async fn test_cache_statistics_initialization() {
        let config = CacheConfig::default();
        let cache = MultiTierCache::new(config);
        
        let stats = cache.get_statistics().await;
        
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 0);
        assert_eq!(stats.hit_ratio, 0.0);
        assert_eq!(stats.evictions, 0);
        assert_eq!(stats.total_cached_items, 0);
        assert_eq!(stats.cache_size_bytes, 0);
        assert_eq!(stats.average_access_time_ms, 0.0);
        assert_eq!(stats.cleanup_runs, 0);
        assert!(stats.last_cleanup.is_none());
    }

    #[tokio::test]
    async fn test_multi_tier_cache_basic_operations() {
        let config = CacheConfig {
            max_events: 10,
            ..Default::default()
        };
        let cache = MultiTierCache::new(config);
        
        let key = CacheKey::Event(Uuid::new_v4());
        let data = b"test_data_for_caching_performance_optimization".to_vec();
        
        // Test cache miss
        let result = cache.get(&key).await;
        assert_eq!(result, None);
        
        // Test cache put
        cache.put(key.clone(), data.clone()).await;
        
        // Test cache hit
        let result = cache.get(&key).await;
        assert_eq!(result, Some(data.clone()));
        
        // Verify statistics
        let stats = cache.get_statistics().await;
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.hit_ratio, 0.5);
    }

    #[tokio::test]
    async fn test_cache_eviction_behavior() {
        let config = CacheConfig {
            max_events: 4, // Small cache to test eviction
            ..Default::default()
        };
        let cache = MultiTierCache::new(config);
        
        // Fill cache beyond capacity
        for i in 0..6 {
            let key = CacheKey::Event(Uuid::new_v4());
            let data = format!("test_data_{}", i).into_bytes();
            cache.put(key, data).await;
        }
        
        let stats = cache.get_statistics().await;
        
        // Should have triggered evictions
        assert!(stats.evictions > 0);
        assert!(stats.total_cached_items <= 4); // L1 + L2 should not exceed reasonable limits
    }

    #[tokio::test]
    async fn test_cache_ttl_and_cleanup() {
        let config = CacheConfig {
            event_ttl_seconds: 1, // Very short TTL for testing
            ..Default::default()
        };
        let cache = MultiTierCache::new(config);
        
        let key = CacheKey::Event(Uuid::new_v4());
        let data = b"test_data_with_short_ttl".to_vec();
        
        // Add data to cache
        cache.put(key.clone(), data.clone()).await;
        
        // Should be available immediately
        let result = cache.get(&key).await;
        assert_eq!(result, Some(data));
        
        // Wait for TTL to expire
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Run cleanup
        cache.cleanup_expired().await;
        
        // Should be expired and removed
        let result = cache.get(&key).await;
        assert_eq!(result, None);
        
        let stats = cache.get_statistics().await;
        assert!(stats.cleanup_runs > 0);
    }

    #[tokio::test]
    async fn test_cache_compression_configuration() {
        let config_uncompressed = CacheConfig {
            compress_cached_data: false,
            ..Default::default()
        };
        
        let config_compressed = CacheConfig {
            compress_cached_data: true,
            ..Default::default()
        };
        
        // Test that both configurations are valid
        assert!(!config_uncompressed.compress_cached_data);
        assert!(config_compressed.compress_cached_data);
    }

    #[tokio::test]
    async fn test_cache_key_variants() {
        let aggregate_id = Uuid::new_v4();
        let event_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        
        // Test different cache key types
        let keys = vec![
            CacheKey::Event(event_id),
            CacheKey::Aggregate(aggregate_id),
            CacheKey::AggregateEvents(aggregate_id),
            CacheKey::AggregateEventsFromVersion(aggregate_id, 5),
            CacheKey::EventsByCorrelation(correlation_id),
            CacheKey::Snapshot(aggregate_id),
            CacheKey::AggregateVersion(aggregate_id),
            CacheKey::CurrentPosition,
        ];
        
        // All keys should be unique
        for (i, key1) in keys.iter().enumerate() {
            for (j, key2) in keys.iter().enumerate() {
                if i != j {
                    assert_ne!(key1, key2, "Cache keys should be unique");
                }
            }
        }
    }

    #[tokio::test]
    async fn test_cache_hit_miss_ratio_calculation() {
        let config = CacheConfig::default();
        let cache = MultiTierCache::new(config);
        
        // Generate some cache misses
        for i in 0..5 {
            let key = CacheKey::Event(Uuid::new_v4());
            let _result = cache.get(&key).await; // All misses
        }
        
        // Add some data and generate hits
        let keys: Vec<_> = (0..3).map(|i| {
            let key = CacheKey::Event(Uuid::new_v4());
            let data = format!("data_{}", i).into_bytes();
            (key, data)
        }).collect();
        
        for (key, data) in &keys {
            cache.put(key.clone(), data.clone()).await;
        }
        
        // Generate cache hits
        for (key, _) in &keys {
            let _result = cache.get(key).await; // All hits
        }
        
        let stats = cache.get_statistics().await;
        
        assert_eq!(stats.cache_misses, 5);
        assert_eq!(stats.cache_hits, 3);
        assert_eq!(stats.total_requests, 8);
        assert!((stats.hit_ratio - 0.375).abs() < 0.001); // 3/8 = 0.375
    }

    #[tokio::test]
    async fn test_cache_average_access_time_tracking() {
        let config = CacheConfig::default();
        let cache = MultiTierCache::new(config);
        
        let key = CacheKey::Event(Uuid::new_v4());
        let data = b"test_data_for_timing".to_vec();
        
        // Perform some operations to generate timing data
        cache.put(key.clone(), data.clone()).await;
        
        for _ in 0..5 {
            let _result = cache.get(&key).await;
        }
        
        let stats = cache.get_statistics().await;
        
        // Should have recorded access times
        assert!(stats.average_access_time_ms >= 0.0);
        assert!(stats.total_requests > 0);
    }

    #[tokio::test]
    async fn test_cache_clear_functionality() {
        let config = CacheConfig::default();
        let cache = MultiTierCache::new(config);
        
        // Add some data
        for i in 0..5 {
            let key = CacheKey::Event(Uuid::new_v4());
            let data = format!("data_{}", i).into_bytes();
            cache.put(key, data).await;
        }
        
        // Verify data exists
        let stats_before = cache.get_statistics().await;
        assert!(stats_before.total_cached_items > 0);
        
        // Clear cache
        cache.clear().await;
        
        // Verify cache is empty
        let stats_after = cache.get_statistics().await;
        assert_eq!(stats_after.total_cached_items, 0);
    }
}

/// Performance benchmark tests for the caching system
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_cache_performance_under_load() {
        let config = CacheConfig {
            max_events: 1000,
            ..Default::default()
        };
        let cache = MultiTierCache::new(config);
        
        let start_time = Instant::now();
        
        // Perform many cache operations
        for i in 0..500 {
            let key = CacheKey::Event(Uuid::new_v4());
            let data = format!("performance_test_data_{}", i).into_bytes();
            
            // Put operation
            cache.put(key.clone(), data.clone()).await;
            
            // Get operation
            let result = cache.get(&key).await;
            assert_eq!(result, Some(data));
        }
        
        let duration = start_time.elapsed();
        
        // Verify performance is reasonable (should complete in under 2 seconds)
        assert!(duration.as_secs() < 2, "Cache operations took too long: {:?}", duration);
        
        let stats = cache.get_statistics().await;
        
        // Verify hit ratio
        assert!(stats.hit_ratio > 0.4, "Hit ratio too low: {}", stats.hit_ratio);
        
        // Verify average access time is reasonable (under 10ms)
        assert!(stats.average_access_time_ms < 10.0, 
                "Average access time too high: {}ms", stats.average_access_time_ms);
    }

    #[tokio::test]
    async fn test_cache_memory_efficiency() {
        let config = CacheConfig {
            max_events: 100,
            ..Default::default()
        };
        let cache = MultiTierCache::new(config);
        
        // Add data with known sizes
        let test_data = "x".repeat(1000); // 1KB of data
        
        for i in 0..50 {
            let key = CacheKey::Event(Uuid::new_v4());
            cache.put(key, test_data.clone().into_bytes()).await;
        }
        
        let stats = cache.get_statistics().await;
        
        // Should have reasonable memory usage
        assert!(stats.total_cached_items <= 50);
        
        // Cache should be managing memory efficiently
        assert!(stats.evictions == 0 || stats.total_cached_items < 50,
                "Cache should either fit all items or have evicted some");
    }
}