// File: src/db/events/versioning.rs
//
// Event versioning and schema migration system
// Provides backward compatibility and schema evolution for events

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use uuid::Uuid;

use super::{
    EventEnvelope, EventError, EventResult, EventSerializable,
    replay::{ReplayHandler, EventReplayEngine, ReplayConfig},
};

/// Represents a schema version for an event type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchemaVersion {
    pub event_type: String,
    pub version: i32,
    pub description: String,
    pub introduced_at: DateTime<Utc>,
    pub deprecated_at: Option<DateTime<Utc>>,
    pub migration_notes: Option<String>,
}

impl SchemaVersion {
    pub fn new(event_type: String, version: i32, description: String) -> Self {
        Self {
            event_type,
            version,
            description,
            introduced_at: Utc::now(),
            deprecated_at: None,
            migration_notes: None,
        }
    }
    
    pub fn with_migration_notes(mut self, notes: String) -> Self {
        self.migration_notes = Some(notes);
        self
    }
    
    pub fn deprecate(mut self, notes: Option<String>) -> Self {
        self.deprecated_at = Some(Utc::now());
        if let Some(notes) = notes {
            self.migration_notes = Some(notes);
        }
        self
    }
    
    pub fn is_deprecated(&self) -> bool {
        self.deprecated_at.is_some()
    }
}

/// Trait for migrating events between schema versions
#[async_trait]
pub trait EventMigrator: Send + Sync {
    /// Get the event type this migrator handles
    fn event_type(&self) -> &str;
    
    /// Get the source version (what version this migrates from)
    fn from_version(&self) -> i32;
    
    /// Get the target version (what version this migrates to)
    fn to_version(&self) -> i32;
    
    /// Migrate event data from one version to another
    async fn migrate(&self, event_data: serde_json::Value) -> EventResult<serde_json::Value>;
    
    /// Check if the migration can be applied to the given data
    async fn can_migrate(&self, event_data: &serde_json::Value) -> bool {
        // Default implementation - always can migrate
        true
    }
    
    /// Get migration description
    fn description(&self) -> &str {
        "Event migration"
    }
}

/// Configuration for the versioning system
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VersioningConfig {
    /// Whether to allow automatic migrations during replay
    pub auto_migrate: bool,
    /// Whether to validate schema versions strictly
    pub strict_validation: bool,
    /// Maximum number of migration steps allowed in a chain
    pub max_migration_chain_length: usize,
    /// Whether to cache migration results
    pub cache_migrations: bool,
    /// Cache size for migration results
    pub migration_cache_size: usize,
}

impl Default for VersioningConfig {
    fn default() -> Self {
        Self {
            auto_migrate: true,
            strict_validation: false,
            max_migration_chain_length: 10,
            cache_migrations: true,
            migration_cache_size: 1000,
        }
    }
}

/// Statistics about versioning and migrations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersioningStatistics {
    pub total_migrations: u64,
    pub successful_migrations: u64,
    pub failed_migrations: u64,
    pub migrations_by_type: HashMap<String, u64>,
    pub migrations_by_version: HashMap<String, u64>, // "event_type:from->to"
    pub average_migration_time_ms: f64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

/// Manages event versioning and schema evolution
pub struct EventVersionManager {
    config: VersioningConfig,
    schema_versions: Arc<RwLock<HashMap<String, Vec<SchemaVersion>>>>,
    migrators: Arc<RwLock<HashMap<String, Box<dyn EventMigrator>>>>,
    migration_cache: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    statistics: Arc<RwLock<VersioningStatistics>>,
}

impl EventVersionManager {
    pub fn new(config: VersioningConfig) -> Self {
        let statistics = VersioningStatistics {
            total_migrations: 0,
            successful_migrations: 0,
            failed_migrations: 0,
            migrations_by_type: HashMap::new(),
            migrations_by_version: HashMap::new(),
            average_migration_time_ms: 0.0,
            cache_hits: 0,
            cache_misses: 0,
        };
        
        Self {
            config,
            schema_versions: Arc::new(RwLock::new(HashMap::new())),
            migrators: Arc::new(RwLock::new(HashMap::new())),
            migration_cache: Arc::new(RwLock::new(HashMap::new())),
            statistics: Arc::new(RwLock::new(statistics)),
        }
    }
    
    /// Register a schema version for an event type
    pub async fn register_schema_version(&self, schema_version: SchemaVersion) -> EventResult<()> {
        let mut versions = self.schema_versions.write().await;
        let event_versions = versions.entry(schema_version.event_type.clone()).or_insert_with(Vec::new);
        
        // Check if version already exists
        if event_versions.iter().any(|v| v.version == schema_version.version) {
            return Err(EventError::ConfigurationError {
                message: format!(
                    "Schema version {} already exists for event type '{}'",
                    schema_version.version, schema_version.event_type
                ),
            });
        }
        
        event_versions.push(schema_version.clone());
        
        // Sort by version number
        event_versions.sort_by_key(|v| v.version);
        
        info!(
            "Registered schema version {} for event type '{}': {}",
            schema_version.version, schema_version.event_type, schema_version.description
        );
        
        Ok(())
    }
    
    /// Register a migrator for transforming events between versions
    pub async fn register_migrator<M: EventMigrator + 'static>(
        &self,
        migrator: M,
    ) -> EventResult<()> {
        let migrator_key = format!(
            "{}:{}->{}",
            migrator.event_type(),
            migrator.from_version(),
            migrator.to_version()
        );
        
        let mut migrators = self.migrators.write().await;
        
        if migrators.contains_key(&migrator_key) {
            return Err(EventError::ConfigurationError {
                message: format!("Migrator already registered for {}", migrator_key),
            });
        }
        
        migrators.insert(migrator_key.clone(), Box::new(migrator));
        
        info!("Registered migrator: {}", migrator_key);
        
        Ok(())
    }
    
    /// Get the latest schema version for an event type
    pub async fn get_latest_version(&self, event_type: &str) -> Option<i32> {
        let versions = self.schema_versions.read().await;
        versions.get(event_type)?.last().map(|v| v.version)
    }
    
    /// Get all schema versions for an event type
    pub async fn get_schema_versions(&self, event_type: &str) -> Vec<SchemaVersion> {
        let versions = self.schema_versions.read().await;
        versions.get(event_type).cloned().unwrap_or_default()
    }
    
    /// Check if a schema version is supported
    pub async fn is_version_supported(&self, event_type: &str, version: i32) -> bool {
        let versions = self.schema_versions.read().await;
        if let Some(event_versions) = versions.get(event_type) {
            event_versions.iter().any(|v| v.version == version)
        } else {
            false
        }
    }
    
    /// Migrate an event to the latest version
    pub async fn migrate_to_latest(
        &self,
        mut event: EventEnvelope,
    ) -> EventResult<EventEnvelope> {
        let latest_version = match self.get_latest_version(&event.event_type).await {
            Some(version) => version,
            None => {
                if self.config.strict_validation {
                    return Err(EventError::ConfigurationError {
                        message: format!("No schema versions registered for event type '{}'", event.event_type),
                    });
                } else {
                    // If not strict, return the event as-is
                    return Ok(event);
                }
            }
        };
        
        if event.schema_version == latest_version {
            // Already at latest version
            return Ok(event);
        }
        
        // Migrate through version chain
        event = self.migrate_event(event, latest_version).await?;
        
        Ok(event)
    }
    
    /// Migrate an event to a specific version
    pub async fn migrate_to_version(
        &self,
        event: EventEnvelope,
        target_version: i32,
    ) -> EventResult<EventEnvelope> {
        if event.schema_version == target_version {
            return Ok(event);
        }
        
        self.migrate_event(event, target_version).await
    }
    
    /// Internal method to migrate an event through the version chain
    async fn migrate_event(
        &self,
        mut event: EventEnvelope,
        target_version: i32,
    ) -> EventResult<EventEnvelope> {
        let start_time = std::time::Instant::now();
        let original_version = event.schema_version;
        
        // Check cache first
        let cache_key = format!(
            "{}:{}:{}->{}",
            event.event_type,
            event.event_id,
            original_version,
            target_version
        );
        
        if self.config.cache_migrations {
            let cache = self.migration_cache.read().await;
            if let Some(cached_data) = cache.get(&cache_key) {
                event.event_data = cached_data.clone();
                event.schema_version = target_version;
                
                // Update cache hit statistics
                {
                    let mut stats = self.statistics.write().await;
                    stats.cache_hits += 1;
                }
                
                debug!("Cache hit for migration: {}", cache_key);
                return Ok(event);
            }
        }
        
        // Find migration path
        let migration_path = self.find_migration_path(&event.event_type, original_version, target_version).await?;
        
        if migration_path.is_empty() {
            return Err(EventError::ConfigurationError {
                message: format!(
                    "No migration path found from version {} to {} for event type '{}'",
                    original_version, target_version, event.event_type
                ),
            });
        }
        
        if migration_path.len() > self.config.max_migration_chain_length {
            return Err(EventError::ConfigurationError {
                message: format!(
                    "Migration chain too long: {} steps (max: {})",
                    migration_path.len(), self.config.max_migration_chain_length
                ),
            });
        }
        
        // Execute migration chain
        let mut current_data = event.event_data.clone();
        
        for (from_version, to_version) in migration_path {
            let migrator_key = format!("{}:{}->{}",
                                     event.event_type, from_version, to_version);
            
            let migrators = self.migrators.read().await;
            let migrator = migrators.get(&migrator_key)
                .ok_or_else(|| EventError::ConfigurationError {
                    message: format!("No migrator found for {}", migrator_key),
                })?;
            
            if !migrator.can_migrate(&current_data).await {
                return Err(EventError::SerializationError {
                    message: format!(
                        "Migration {} cannot be applied to event data",
                        migrator_key
                    ),
                });
            }
            
            current_data = migrator.migrate(current_data).await?;
            
            debug!(
                "Migrated event {} from version {} to {} using {}",
                event.event_id, from_version, to_version, migrator_key
            );
        }
        
        // Update event with migrated data
        event.event_data = current_data.clone();
        event.schema_version = target_version;
        
        // Cache the result
        if self.config.cache_migrations {
            let mut cache = self.migration_cache.write().await;
            
            // Evict old entries if cache is full
            if cache.len() >= self.config.migration_cache_size {
                // Simple LRU: remove first entry
                if let Some(first_key) = cache.keys().next().cloned() {
                    cache.remove(&first_key);
                }
            }
            
            cache.insert(cache_key, current_data);
        }
        
        // Update statistics
        self.update_migration_statistics(&event.event_type, original_version, target_version, start_time.elapsed().as_millis() as f64).await;
        
        info!(
            "Successfully migrated event {} from version {} to {}",
            event.event_id, original_version, target_version
        );
        
        Ok(event)
    }
    
    /// Find the shortest migration path between two versions
    async fn find_migration_path(
        &self,
        event_type: &str,
        from_version: i32,
        to_version: i32,
    ) -> EventResult<Vec<(i32, i32)>> {
        if from_version == to_version {
            return Ok(Vec::new());
        }
        
        let migrators = self.migrators.read().await;
        let mut available_migrations = Vec::new();
        
        // Find all available migrations for this event type
        for (key, _) in migrators.iter() {
            if let Some((event_type_from_key, version_part)) = key.split_once(':') {
                if event_type_from_key == event_type {
                    if let Some((from_str, to_str)) = version_part.split_once("->") {
                        if let (Ok(from), Ok(to)) = (from_str.parse::<i32>(), to_str.parse::<i32>()) {
                            available_migrations.push((from, to));
                        }
                    }
                }
            }
        }
        
        // Use Dijkstra's algorithm to find shortest path
        let path = self.find_shortest_path(available_migrations, from_version, to_version);
        
        if path.is_empty() && from_version != to_version {
            return Err(EventError::ConfigurationError {
                message: format!(
                    "No migration path found from version {} to {} for event type '{}'",
                    from_version, to_version, event_type
                ),
            });
        }
        
        Ok(path)
    }
    
    /// Find shortest path using a simple BFS approach
    fn find_shortest_path(
        &self,
        migrations: Vec<(i32, i32)>,
        start: i32,
        end: i32,
    ) -> Vec<(i32, i32)> {
        use std::collections::{HashMap, VecDeque};
        
        if start == end {
            return Vec::new();
        }
        
        // Build adjacency list
        let mut graph: HashMap<i32, Vec<i32>> = HashMap::new();
        for (from, to) in &migrations {
            graph.entry(*from).or_insert_with(Vec::new).push(*to);
        }
        
        // BFS to find shortest path
        let mut queue = VecDeque::new();
        let mut visited = HashMap::new();
        let mut parent: HashMap<i32, Option<i32>> = HashMap::new();
        
        queue.push_back(start);
        visited.insert(start, true);
        parent.insert(start, None);
        
        while let Some(current) = queue.pop_front() {
            if current == end {
                break;
            }
            
            if let Some(neighbors) = graph.get(&current) {
                for &neighbor in neighbors {
                    if !visited.contains_key(&neighbor) {
                        visited.insert(neighbor, true);
                        parent.insert(neighbor, Some(current));
                        queue.push_back(neighbor);
                    }
                }
            }
        }
        
        // Reconstruct path
        if !parent.contains_key(&end) {
            return Vec::new(); // No path found
        }
        
        let mut path = Vec::new();
        let mut current = end;
        
        while let Some(Some(prev)) = parent.get(&current) {
            path.push((*prev, current));
            current = *prev;
        }
        
        path.reverse();
        path
    }
    
    /// Update migration statistics
    async fn update_migration_statistics(
        &self,
        event_type: &str,
        from_version: i32,
        to_version: i32,
        migration_time_ms: f64,
    ) {
        let mut stats = self.statistics.write().await;
        
        stats.total_migrations += 1;
        stats.successful_migrations += 1;
        stats.cache_misses += 1;
        
        *stats.migrations_by_type.entry(event_type.to_string()).or_insert(0) += 1;
        
        let version_key = format!("{}:{}->{}",  event_type, from_version, to_version);
        *stats.migrations_by_version.entry(version_key).or_insert(0) += 1;
        
        // Update average migration time
        let total_time = stats.average_migration_time_ms * (stats.total_migrations - 1) as f64;
        stats.average_migration_time_ms = (total_time + migration_time_ms) / stats.total_migrations as f64;
    }
    
    /// Get migration statistics
    pub async fn get_statistics(&self) -> VersioningStatistics {
        self.statistics.read().await.clone()
    }
    
    /// Clear migration cache
    pub async fn clear_cache(&self) {
        let mut cache = self.migration_cache.write().await;
        cache.clear();
        
        info!("Migration cache cleared");
    }
    
    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> (usize, usize) {
        let cache = self.migration_cache.read().await;
        let stats = self.statistics.read().await;
        (cache.len(), stats.cache_hits as usize + stats.cache_misses as usize)
    }
}

impl Clone for EventVersionManager {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            schema_versions: self.schema_versions.clone(),
            migrators: self.migrators.clone(),
            migration_cache: self.migration_cache.clone(),
            statistics: self.statistics.clone(),
        }
    }
}

/// Replay handler that automatically migrates events during replay
pub struct MigratingReplayHandler<H> {
    inner_handler: H,
    version_manager: EventVersionManager,
    target_version: Option<i32>,
}

impl<H> MigratingReplayHandler<H> {
    pub fn new(inner_handler: H, version_manager: EventVersionManager) -> Self {
        Self {
            inner_handler,
            version_manager,
            target_version: None,
        }
    }
    
    pub fn with_target_version(mut self, version: i32) -> Self {
        self.target_version = Some(version);
        self
    }
}

#[async_trait]
impl<H> ReplayHandler for MigratingReplayHandler<H>
where
    H: ReplayHandler + Send + Sync,
{
    async fn handle_events(&mut self, events: &[EventEnvelope]) -> EventResult<()> {
        let mut migrated_events = Vec::new();
        
        for event in events {
            let migrated_event = if let Some(target_version) = self.target_version {
                self.version_manager.migrate_to_version(event.clone(), target_version).await?
            } else {
                self.version_manager.migrate_to_latest(event.clone()).await?
            };
            
            migrated_events.push(migrated_event);
        }
        
        self.inner_handler.handle_events(&migrated_events).await
    }
    
    async fn on_replay_start(&mut self, from_position: i64) -> EventResult<()> {
        self.inner_handler.on_replay_start(from_position).await
    }
    
    async fn on_replay_complete(&mut self, events_processed: u64) -> EventResult<()> {
        self.inner_handler.on_replay_complete(events_processed).await
    }
    
    fn consumer_name(&self) -> &str {
        self.inner_handler.consumer_name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    struct TestMigrator {
        event_type: String,
        from_version: i32,
        to_version: i32,
    }
    
    impl TestMigrator {
        fn new(event_type: String, from_version: i32, to_version: i32) -> Self {
            Self {
                event_type,
                from_version,
                to_version,
            }
        }
    }
    
    #[async_trait]
    impl EventMigrator for TestMigrator {
        fn event_type(&self) -> &str {
            &self.event_type
        }
        
        fn from_version(&self) -> i32 {
            self.from_version
        }
        
        fn to_version(&self) -> i32 {
            self.to_version
        }
        
        async fn migrate(&self, mut event_data: serde_json::Value) -> EventResult<serde_json::Value> {
            // Simple test migration: add a "migrated_from" field
            if let Some(obj) = event_data.as_object_mut() {
                obj.insert(
                    "migrated_from".to_string(),
                    json!(format!("v{}", self.from_version))
                );
                obj.insert(
                    "migrated_to".to_string(),
                    json!(format!("v{}", self.to_version))
                );
            }
            Ok(event_data)
        }
        
        fn description(&self) -> &str {
            "Test migrator"
        }
    }
    
    #[tokio::test]
    async fn test_schema_version_registration() {
        let config = VersioningConfig::default();
        let version_manager = EventVersionManager::new(config);
        
        let schema_v1 = SchemaVersion::new(
            "test_event".to_string(),
            1,
            "Initial version".to_string(),
        );
        
        let result = version_manager.register_schema_version(schema_v1).await;
        assert!(result.is_ok());
        
        let latest_version = version_manager.get_latest_version("test_event").await;
        assert_eq!(latest_version, Some(1));
        
        let is_supported = version_manager.is_version_supported("test_event", 1).await;
        assert!(is_supported);
        
        let is_unsupported = version_manager.is_version_supported("test_event", 2).await;
        assert!(!is_unsupported);
    }
    
    #[tokio::test]
    async fn test_migrator_registration_and_simple_migration() {
        let config = VersioningConfig::default();
        let version_manager = EventVersionManager::new(config);
        
        // Register schema versions
        let schema_v1 = SchemaVersion::new("test_event".to_string(), 1, "V1".to_string());
        let schema_v2 = SchemaVersion::new("test_event".to_string(), 2, "V2".to_string());
        
        version_manager.register_schema_version(schema_v1).await.unwrap();
        version_manager.register_schema_version(schema_v2).await.unwrap();
        
        // Register migrator
        let migrator = TestMigrator::new("test_event".to_string(), 1, 2);
        let result = version_manager.register_migrator(migrator).await;
        assert!(result.is_ok());
        
        // Create test event
        let mut event = EventEnvelope {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            aggregate_type: "test_aggregate".to_string(),
            event_type: "test_event".to_string(),
            aggregate_version: 1,
            event_data: json!({"original_data": "test"}),
            metadata: Default::default(),
            occurred_at: Utc::now(),
            recorded_at: Utc::now(),
            schema_version: 1,
            causation_id: None,
            correlation_id: None,
            checksum: None,
        };
        
        // Migrate to latest version
        let migrated = version_manager.migrate_to_latest(event.clone()).await;
        assert!(migrated.is_ok());
        
        let migrated_event = migrated.unwrap();
        assert_eq!(migrated_event.schema_version, 2);
        assert_eq!(migrated_event.event_data["original_data"], "test");
        assert_eq!(migrated_event.event_data["migrated_from"], "v1");
        assert_eq!(migrated_event.event_data["migrated_to"], "v2");
    }
    
    #[tokio::test]
    async fn test_migration_chain() {
        let config = VersioningConfig::default();
        let version_manager = EventVersionManager::new(config);
        
        // Register schema versions 1, 2, 3
        for version in 1..=3 {
            let schema = SchemaVersion::new(
                "test_event".to_string(),
                version,
                format!("Version {}", version),
            );
            version_manager.register_schema_version(schema).await.unwrap();
        }
        
        // Register migrators 1->2 and 2->3
        let migrator_1_2 = TestMigrator::new("test_event".to_string(), 1, 2);
        let migrator_2_3 = TestMigrator::new("test_event".to_string(), 2, 3);
        
        version_manager.register_migrator(migrator_1_2).await.unwrap();
        version_manager.register_migrator(migrator_2_3).await.unwrap();
        
        // Create event at version 1
        let event = EventEnvelope {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            aggregate_type: "test_aggregate".to_string(),
            event_type: "test_event".to_string(),
            aggregate_version: 1,
            event_data: json!({"original": "data"}),
            metadata: Default::default(),
            occurred_at: Utc::now(),
            recorded_at: Utc::now(),
            schema_version: 1,
            causation_id: None,
            correlation_id: None,
            checksum: None,
        };
        
        // Migrate to version 3 (should go through version 2)
        let migrated = version_manager.migrate_to_version(event, 3).await;
        assert!(migrated.is_ok());
        
        let migrated_event = migrated.unwrap();
        assert_eq!(migrated_event.schema_version, 3);
        
        // Should have migration markers from both steps
        assert!(migrated_event.event_data["migrated_from"].is_string());
        assert!(migrated_event.event_data["migrated_to"].is_string());
    }
    
    #[test]
    fn test_schema_version_deprecation() {
        let mut schema = SchemaVersion::new(
            "test_event".to_string(),
            1,
            "Test version".to_string(),
        );
        
        assert!(!schema.is_deprecated());
        
        schema = schema.deprecate(Some("Use version 2 instead".to_string()));
        assert!(schema.is_deprecated());
        assert!(schema.deprecated_at.is_some());
        assert_eq!(schema.migration_notes, Some("Use version 2 instead".to_string()));
    }
    
    #[test]
    fn test_versioning_config_defaults() {
        let config = VersioningConfig::default();
        assert!(config.auto_migrate);
        assert!(!config.strict_validation);
        assert_eq!(config.max_migration_chain_length, 10);
        assert!(config.cache_migrations);
        assert_eq!(config.migration_cache_size, 1000);
    }
}