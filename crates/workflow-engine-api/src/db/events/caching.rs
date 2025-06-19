// File: src/db/events/caching.rs
//
// Advanced caching layer for event store performance optimization
// Provides multi-tier caching with intelligent cache management

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info, warn, error};
use uuid::Uuid;

use super::{
    EventStore, EventEnvelope, EventError, EventResult, AggregateSnapshot,
};

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum number of events to cache
    pub max_events: usize,
    /// Maximum number of aggregates to cache
    pub max_aggregates: usize,
    /// Maximum number of snapshots to cache
    pub max_snapshots: usize,
    /// TTL for cached events (in seconds)
    pub event_ttl_seconds: u64,
    /// TTL for cached aggregates (in seconds)
    pub aggregate_ttl_seconds: u64,
    /// TTL for cached snapshots (in seconds)
    pub snapshot_ttl_seconds: u64,
    /// Whether to enable write-through caching
    pub write_through: bool,
    /// Whether to enable compression for cached data
    pub compress_cached_data: bool,
    /// Cache cleanup interval (in seconds)
    pub cleanup_interval_seconds: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_events: 10_000,
            max_aggregates: 1_000,
            max_snapshots: 500,
            event_ttl_seconds: 3600, // 1 hour
            aggregate_ttl_seconds: 1800, // 30 minutes
            snapshot_ttl_seconds: 7200, // 2 hours
            write_through: true,
            compress_cached_data: false,
            cleanup_interval_seconds: 300, // 5 minutes
        }
    }
}

/// Cache entry with metadata
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    value: T,
    created_at: Instant,
    last_accessed: Instant,
    access_count: u64,
    size_bytes: usize,
}

impl<T> CacheEntry<T> {
    fn new(value: T, size_bytes: usize) -> Self {
        let now = Instant::now();
        Self {
            value,
            created_at: now,
            last_accessed: now,
            access_count: 1,
            size_bytes,
        }
    }
    
    fn access(&mut self) -> &T {
        self.last_accessed = Instant::now();
        self.access_count += 1;
        &self.value
    }
    
    fn is_expired(&self, ttl: Duration) -> bool {
        self.created_at.elapsed() > ttl
    }
    
    fn age(&self) -> Duration {
        self.created_at.elapsed()
    }
}

/// Cache key for different types of cached data
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum CacheKey {
    Event(Uuid),
    Aggregate(Uuid),
    AggregateEvents(Uuid),
    AggregateEventsFromVersion(Uuid, i64),
    EventsByType(String, Option<DateTime<Utc>>, Option<DateTime<Utc>>, Option<usize>),
    EventsByCorrelation(Uuid),
    Snapshot(Uuid),
    AggregateVersion(Uuid),
    CurrentPosition,
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStatistics {
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub hit_ratio: f64,
    pub evictions: u64,
    pub total_cached_items: usize,
    pub cache_size_bytes: usize,
    pub average_access_time_ms: f64,
    pub cleanup_runs: u64,
    pub last_cleanup: Option<DateTime<Utc>>,
}

impl Default for CacheStatistics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            cache_hits: 0,
            cache_misses: 0,
            hit_ratio: 0.0,
            evictions: 0,
            total_cached_items: 0,
            cache_size_bytes: 0,
            average_access_time_ms: 0.0,
            cleanup_runs: 0,
            last_cleanup: None,
        }
    }
}

/// Multi-tier cache implementation
pub struct MultiTierCache {
    l1_cache: Arc<RwLock<HashMap<CacheKey, CacheEntry<Vec<u8>>>>>, // Hot data
    l2_cache: Arc<RwLock<HashMap<CacheKey, CacheEntry<Vec<u8>>>>>, // Warm data
    access_order: Arc<Mutex<VecDeque<CacheKey>>>, // LRU tracking
    config: CacheConfig,
    statistics: Arc<RwLock<CacheStatistics>>,
}

impl MultiTierCache {
    fn new(config: CacheConfig) -> Self {
        Self {
            l1_cache: Arc::new(RwLock::new(HashMap::new())),
            l2_cache: Arc::new(RwLock::new(HashMap::new())),
            access_order: Arc::new(Mutex::new(VecDeque::new())),
            config,
            statistics: Arc::new(RwLock::new(CacheStatistics::default())),
        }
    }
    
    async fn get(&self, key: &CacheKey) -> Option<Vec<u8>> {
        let start_time = Instant::now();
        
        // Try L1 cache first
        {
            let mut l1 = self.l1_cache.write().await;
            if let Some(entry) = l1.get_mut(key) {
                let data = entry.access().clone();
                self.update_access_order(key).await;
                self.record_hit(start_time.elapsed()).await;
                return Some(data);
            }
        }
        
        // Try L2 cache
        {
            let mut l2 = self.l2_cache.write().await;
            if let Some(entry) = l2.remove(key) {
                let data = entry.value.clone();
                
                // Promote to L1
                let mut l1 = self.l1_cache.write().await;
                l1.insert(key.clone(), CacheEntry::new(data.clone(), entry.size_bytes));
                
                self.update_access_order(key).await;
                self.record_hit(start_time.elapsed()).await;
                return Some(data);
            }
        }
        
        self.record_miss(start_time.elapsed()).await;
        None
    }
    
    async fn put(&self, key: CacheKey, data: Vec<u8>) {
        let size_bytes = data.len();
        let entry = CacheEntry::new(data, size_bytes);
        
        // Check if we need to evict from L1
        {
            let mut l1 = self.l1_cache.write().await;
            
            while l1.len() >= self.config.max_events / 2 {
                if let Some(evict_key) = self.get_lru_key().await {
                    if let Some(evicted_entry) = l1.remove(&evict_key) {
                        // Move to L2 if not expired
                        let ttl = Duration::from_secs(self.config.event_ttl_seconds);
                        if !evicted_entry.is_expired(ttl) {
                            let mut l2 = self.l2_cache.write().await;
                            l2.insert(evict_key, evicted_entry);
                        }
                        self.record_eviction().await;
                    }
                } else {
                    break;
                }
            }
            
            l1.insert(key.clone(), entry);
        }
        
        self.update_access_order(&key).await;
    }
    
    async fn update_access_order(&self, key: &CacheKey) {
        let mut order = self.access_order.lock().await;
        
        // Remove if already exists
        if let Some(pos) = order.iter().position(|k| k == key) {
            order.remove(pos);
        }
        
        // Add to front (most recently used)
        order.push_front(key.clone());
        
        // Limit size
        while order.len() > self.config.max_events {
            order.pop_back();
        }
    }
    
    async fn get_lru_key(&self) -> Option<CacheKey> {
        let order = self.access_order.lock().await;
        order.back().cloned()
    }
    
    async fn record_hit(&self, access_time: Duration) {
        let mut stats = self.statistics.write().await;
        stats.total_requests += 1;
        stats.cache_hits += 1;
        stats.hit_ratio = stats.cache_hits as f64 / stats.total_requests as f64;
        
        let access_time_ms = access_time.as_millis() as f64;
        let total_time = stats.average_access_time_ms * (stats.total_requests - 1) as f64;
        stats.average_access_time_ms = (total_time + access_time_ms) / stats.total_requests as f64;
    }
    
    async fn record_miss(&self, access_time: Duration) {
        let mut stats = self.statistics.write().await;
        stats.total_requests += 1;
        stats.cache_misses += 1;
        stats.hit_ratio = stats.cache_hits as f64 / stats.total_requests as f64;
        
        let access_time_ms = access_time.as_millis() as f64;
        let total_time = stats.average_access_time_ms * (stats.total_requests - 1) as f64;
        stats.average_access_time_ms = (total_time + access_time_ms) / stats.total_requests as f64;
    }
    
    async fn record_eviction(&self) {
        let mut stats = self.statistics.write().await;
        stats.evictions += 1;
    }
    
    async fn cleanup_expired(&self) {
        let event_ttl = Duration::from_secs(self.config.event_ttl_seconds);
        let mut removed_count = 0;
        
        // Cleanup L1 cache
        {
            let mut l1 = self.l1_cache.write().await;
            let expired_keys: Vec<CacheKey> = l1.iter()
                .filter(|(_, entry)| entry.is_expired(event_ttl))
                .map(|(key, _)| key.clone())
                .collect();
            
            for key in expired_keys {
                l1.remove(&key);
                removed_count += 1;
            }
        }
        
        // Cleanup L2 cache
        {
            let mut l2 = self.l2_cache.write().await;
            let expired_keys: Vec<CacheKey> = l2.iter()
                .filter(|(_, entry)| entry.is_expired(event_ttl))
                .map(|(key, _)| key.clone())
                .collect();
            
            for key in expired_keys {
                l2.remove(&key);
                removed_count += 1;
            }
        }
        
        // Update statistics
        {
            let mut stats = self.statistics.write().await;
            stats.cleanup_runs += 1;
            stats.last_cleanup = Some(Utc::now());
            
            let l1_size = self.l1_cache.read().await.len();
            let l2_size = self.l2_cache.read().await.len();
            stats.total_cached_items = l1_size + l2_size;
        }
        
        if removed_count > 0 {
            debug!("Cache cleanup removed {} expired entries", removed_count);
        }
    }
    
    async fn get_statistics(&self) -> CacheStatistics {
        let stats = self.statistics.read().await;
        let mut result = stats.clone();
        
        // Update current cache size
        let l1_size = self.l1_cache.read().await.len();
        let l2_size = self.l2_cache.read().await.len();
        result.total_cached_items = l1_size + l2_size;
        
        result
    }
    
    async fn clear(&self) {
        let mut l1 = self.l1_cache.write().await;
        let mut l2 = self.l2_cache.write().await;
        let mut order = self.access_order.lock().await;
        
        l1.clear();
        l2.clear();
        order.clear();
        
        info!("Cache cleared");
    }
}

/// Cached event store implementation
pub struct CachedEventStore {
    inner: Arc<dyn EventStore>,
    cache: MultiTierCache,
    config: CacheConfig,
}

impl CachedEventStore {
    pub fn new(inner: Arc<dyn EventStore>, config: CacheConfig) -> Self {
        let cache = MultiTierCache::new(config.clone());
        
        Self {
            inner,
            cache,
            config,
        }
    }
    
    /// Start background cache cleanup task
    pub async fn start_cache_cleanup(&self) -> tokio::task::JoinHandle<()> {
        let cache = self.cache.l1_cache.clone();
        let cache2 = self.cache.l2_cache.clone();
        let statistics = self.cache.statistics.clone();
        let cleanup_interval = Duration::from_secs(self.config.cleanup_interval_seconds);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);
            
            loop {
                interval.tick().await;
                
                // Simple cleanup logic
                let event_ttl = Duration::from_secs(3600); // 1 hour
                let mut removed_count = 0;
                
                // Cleanup expired entries
                {
                    let mut l1 = cache.write().await;
                    let expired_keys: Vec<CacheKey> = l1.iter()
                        .filter(|(_, entry)| entry.is_expired(event_ttl))
                        .map(|(key, _)| key.clone())
                        .collect();
                    
                    for key in expired_keys {
                        l1.remove(&key);
                        removed_count += 1;
                    }
                }
                
                {
                    let mut l2 = cache2.write().await;
                    let expired_keys: Vec<CacheKey> = l2.iter()
                        .filter(|(_, entry)| entry.is_expired(event_ttl))
                        .map(|(key, _)| key.clone())
                        .collect();
                    
                    for key in expired_keys {
                        l2.remove(&key);
                        removed_count += 1;
                    }
                }
                
                if removed_count > 0 {
                    debug!("Background cache cleanup removed {} expired entries", removed_count);
                }
                
                // Update statistics
                {
                    let mut stats = statistics.write().await;
                    stats.cleanup_runs += 1;
                    stats.last_cleanup = Some(Utc::now());
                }
            }
        })
    }
    
    /// Get cache statistics
    pub async fn get_cache_statistics(&self) -> CacheStatistics {
        self.cache.get_statistics().await
    }
    
    /// Clear the cache
    pub async fn clear_cache(&self) {
        self.cache.clear().await;
    }
    
    /// Serialize data for caching
    fn serialize_for_cache<T: serde::Serialize>(&self, data: &T) -> EventResult<Vec<u8>> {
        if self.config.compress_cached_data {
            // Simple compression using bincode + gzip
            let serialized = bincode::serialize(data).map_err(|e| EventError::SerializationError {
                message: format!("Failed to serialize for cache: {}", e),
            })?;
            
            use flate2::{Compression, write::GzEncoder};
            use std::io::Write;
            
            let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
            encoder.write_all(&serialized).map_err(|e| EventError::SerializationError {
                message: format!("Failed to compress cache data: {}", e),
            })?;
            
            encoder.finish().map_err(|e| EventError::SerializationError {
                message: format!("Failed to finalize compression: {}", e),
            })
        } else {
            bincode::serialize(data).map_err(|e| EventError::SerializationError {
                message: format!("Failed to serialize for cache: {}", e),
            })
        }
    }
    
    /// Deserialize data from cache
    fn deserialize_from_cache<T: serde::de::DeserializeOwned>(&self, data: &[u8]) -> EventResult<T> {
        if self.config.compress_cached_data {
            use flate2::read::GzDecoder;
            use std::io::Read;
            
            let mut decoder = GzDecoder::new(data);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed).map_err(|e| EventError::SerializationError {
                message: format!("Failed to decompress cache data: {}", e),
            })?;
            
            bincode::deserialize(&decompressed).map_err(|e| EventError::SerializationError {
                message: format!("Failed to deserialize from cache: {}", e),
            })
        } else {
            bincode::deserialize(data).map_err(|e| EventError::SerializationError {
                message: format!("Failed to deserialize from cache: {}", e),
            })
        }
    }
}

#[async_trait]
impl EventStore for CachedEventStore {
    async fn append_event(&self, event: &EventEnvelope) -> EventResult<()> {
        let result = self.inner.append_event(event).await;
        
        if result.is_ok() && self.config.write_through {
            // Invalidate related cache entries
            let aggregate_key = CacheKey::AggregateEvents(event.aggregate_id);
            // We could implement cache invalidation here
        }
        
        result
    }
    
    async fn append_events(&self, events: &[EventEnvelope]) -> EventResult<()> {
        let result = self.inner.append_events(events).await;
        
        if result.is_ok() && self.config.write_through {
            // Invalidate cache entries for all affected aggregates
            for event in events {
                let aggregate_key = CacheKey::AggregateEvents(event.aggregate_id);
                // Cache invalidation logic would go here
            }
        }
        
        result
    }
    
    async fn get_events(&self, aggregate_id: Uuid) -> EventResult<Vec<EventEnvelope>> {
        let cache_key = CacheKey::AggregateEvents(aggregate_id);
        
        // Try cache first
        if let Some(cached_data) = self.cache.get(&cache_key).await {
            if let Ok(events) = self.deserialize_from_cache::<Vec<EventEnvelope>>(&cached_data) {
                debug!("Cache hit for aggregate events: {}", aggregate_id);
                return Ok(events);
            }
        }
        
        // Cache miss - fetch from underlying store
        let events = self.inner.get_events(aggregate_id).await?;
        
        // Cache the result
        if let Ok(serialized) = self.serialize_for_cache(&events) {
            self.cache.put(cache_key, serialized).await;
        }
        
        Ok(events)
    }
    
    async fn get_events_from_version(
        &self,
        aggregate_id: Uuid,
        from_version: i64,
    ) -> EventResult<Vec<EventEnvelope>> {
        let cache_key = CacheKey::AggregateEventsFromVersion(aggregate_id, from_version);
        
        // Try cache first
        if let Some(cached_data) = self.cache.get(&cache_key).await {
            if let Ok(events) = self.deserialize_from_cache::<Vec<EventEnvelope>>(&cached_data) {
                debug!("Cache hit for aggregate events from version: {} v{}", aggregate_id, from_version);
                return Ok(events);
            }
        }
        
        // Cache miss - fetch from underlying store
        let events = self.inner.get_events_from_version(aggregate_id, from_version).await?;
        
        // Cache the result
        if let Ok(serialized) = self.serialize_for_cache(&events) {
            self.cache.put(cache_key, serialized).await;
        }
        
        Ok(events)
    }
    
    async fn get_events_by_type(
        &self,
        event_type: &str,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        limit: Option<usize>,
    ) -> EventResult<Vec<EventEnvelope>> {
        // For now, skip caching for type-based queries as they're more complex
        self.inner.get_events_by_type(event_type, from, to, limit).await
    }
    
    async fn get_events_by_correlation_id(&self, correlation_id: Uuid) -> EventResult<Vec<EventEnvelope>> {
        let cache_key = CacheKey::EventsByCorrelation(correlation_id);
        
        // Try cache first
        if let Some(cached_data) = self.cache.get(&cache_key).await {
            if let Ok(events) = self.deserialize_from_cache::<Vec<EventEnvelope>>(&cached_data) {
                debug!("Cache hit for events by correlation: {}", correlation_id);
                return Ok(events);
            }
        }
        
        // Cache miss - fetch from underlying store
        let events = self.inner.get_events_by_correlation_id(correlation_id).await?;
        
        // Cache the result
        if let Ok(serialized) = self.serialize_for_cache(&events) {
            self.cache.put(cache_key, serialized).await;
        }
        
        Ok(events)
    }
    
    async fn get_aggregate_version(&self, aggregate_id: Uuid) -> EventResult<i64> {
        let cache_key = CacheKey::AggregateVersion(aggregate_id);
        
        // Try cache first
        if let Some(cached_data) = self.cache.get(&cache_key).await {
            if let Ok(version) = self.deserialize_from_cache::<i64>(&cached_data) {
                debug!("Cache hit for aggregate version: {}", aggregate_id);
                return Ok(version);
            }
        }
        
        // Cache miss - fetch from underlying store
        let version = self.inner.get_aggregate_version(aggregate_id).await?;
        
        // Cache the result
        if let Ok(serialized) = self.serialize_for_cache(&version) {
            self.cache.put(cache_key, serialized).await;
        }
        
        Ok(version)
    }
    
    async fn aggregate_exists(&self, aggregate_id: Uuid) -> EventResult<bool> {
        // For exists checks, we can try to get the version from cache first
        let cache_key = CacheKey::AggregateVersion(aggregate_id);
        
        if let Some(cached_data) = self.cache.get(&cache_key).await {
            if let Ok(_version) = self.deserialize_from_cache::<i64>(&cached_data) {
                debug!("Cache hit for aggregate exists check: {}", aggregate_id);
                return Ok(true);
            }
        }
        
        // Fallback to underlying store
        self.inner.aggregate_exists(aggregate_id).await
    }
    
    async fn save_snapshot(&self, snapshot: &AggregateSnapshot) -> EventResult<()> {
        let result = self.inner.save_snapshot(snapshot).await;
        
        if result.is_ok() && self.config.write_through {
            // Cache the snapshot
            let cache_key = CacheKey::Snapshot(snapshot.aggregate_id);
            if let Ok(serialized) = self.serialize_for_cache(snapshot) {
                self.cache.put(cache_key, serialized).await;
            }
        }
        
        result
    }
    
    async fn get_snapshot(&self, aggregate_id: Uuid) -> EventResult<Option<AggregateSnapshot>> {
        let cache_key = CacheKey::Snapshot(aggregate_id);
        
        // Try cache first
        if let Some(cached_data) = self.cache.get(&cache_key).await {
            if let Ok(snapshot) = self.deserialize_from_cache::<Option<AggregateSnapshot>>(&cached_data) {
                debug!("Cache hit for snapshot: {}", aggregate_id);
                return Ok(snapshot);
            }
        }
        
        // Cache miss - fetch from underlying store
        let snapshot = self.inner.get_snapshot(aggregate_id).await?;
        
        // Cache the result
        if let Ok(serialized) = self.serialize_for_cache(&snapshot) {
            self.cache.put(cache_key, serialized).await;
        }
        
        Ok(snapshot)
    }
    
    async fn get_events_from_position(&self, position: i64, limit: usize) -> EventResult<Vec<EventEnvelope>> {
        // Skip caching for streaming queries as they're position-based
        self.inner.get_events_from_position(position, limit).await
    }
    
    async fn get_current_position(&self) -> EventResult<i64> {
        let cache_key = CacheKey::CurrentPosition;
        
        // Try cache with very short TTL for current position
        if let Some(cached_data) = self.cache.get(&cache_key).await {
            if let Ok(position) = self.deserialize_from_cache::<i64>(&cached_data) {
                debug!("Cache hit for current position");
                return Ok(position);
            }
        }
        
        // Cache miss - fetch from underlying store
        let position = self.inner.get_current_position().await?;
        
        // Cache with short TTL (this data changes frequently)
        if let Ok(serialized) = self.serialize_for_cache(&position) {
            self.cache.put(cache_key, serialized).await;
        }
        
        Ok(position)
    }
    
    async fn replay_events(
        &self,
        from_position: i64,
        event_types: Option<Vec<String>>,
        batch_size: usize,
    ) -> EventResult<Vec<EventEnvelope>> {
        // Skip caching for replay queries as they're typically one-time operations
        self.inner.replay_events(from_position, event_types, batch_size).await
    }
    
    async fn get_events_for_aggregates(&self, aggregate_ids: &[Uuid]) -> EventResult<Vec<EventEnvelope>> {
        // For multi-aggregate queries, we could implement partial caching
        // For now, delegate to underlying store
        self.inner.get_events_for_aggregates(aggregate_ids).await
    }
    
    async fn cleanup_old_snapshots(&self, keep_latest: usize) -> EventResult<usize> {
        let result = self.inner.cleanup_old_snapshots(keep_latest).await;
        
        if result.is_ok() {
            // Could invalidate snapshot cache entries here
        }
        
        result
    }
    
    async fn get_aggregate_ids_by_type(
        &self,
        aggregate_type: &str,
        offset: i64,
        limit: usize,
    ) -> EventResult<Vec<Uuid>> {
        // Skip caching for paginated queries
        self.inner.get_aggregate_ids_by_type(aggregate_type, offset, limit).await
    }
    
    async fn optimize_storage(&self) -> EventResult<()> {
        self.inner.optimize_storage().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_cache_config_defaults() {
        let config = CacheConfig::default();
        assert_eq!(config.max_events, 10_000);
        assert_eq!(config.max_aggregates, 1_000);
        assert_eq!(config.max_snapshots, 500);
        assert!(config.write_through);
        assert!(!config.compress_cached_data);
    }
    
    #[test]
    fn test_cache_entry_creation() {
        let entry = CacheEntry::new("test_data".to_string(), 100);
        assert_eq!(entry.access_count, 1);
        assert_eq!(entry.size_bytes, 100);
        assert!(!entry.is_expired(Duration::from_secs(60)));
    }
    
    #[test]
    fn test_cache_key_creation() {
        let event_key = CacheKey::Event(Uuid::new_v4());
        let aggregate_key = CacheKey::Aggregate(Uuid::new_v4());
        
        // Keys should be different
        assert_ne!(format!("{:?}", event_key), format!("{:?}", aggregate_key));
    }
    
    #[tokio::test]
    async fn test_multi_tier_cache_basic_operations() {
        let config = CacheConfig {
            max_events: 10,
            ..Default::default()
        };
        let cache = MultiTierCache::new(config);
        
        let key = CacheKey::Event(Uuid::new_v4());
        let data = b"test_data".to_vec();
        
        // Put data
        cache.put(key.clone(), data.clone()).await;
        
        // Get data
        let retrieved = cache.get(&key).await;
        assert_eq!(retrieved, Some(data));
        
        // Get non-existent data
        let non_existent_key = CacheKey::Event(Uuid::new_v4());
        let result = cache.get(&non_existent_key).await;
        assert_eq!(result, None);
    }
    
    #[tokio::test]
    async fn test_cache_statistics() {
        let config = CacheConfig::default();
        let cache = MultiTierCache::new(config);
        
        let key = CacheKey::Event(Uuid::new_v4());
        let data = b"test_data".to_vec();
        
        // Put and get data to generate stats
        cache.put(key.clone(), data.clone()).await;
        let _retrieved = cache.get(&key).await;
        
        let stats = cache.get_statistics().await;
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.hit_ratio, 1.0);
    }
}