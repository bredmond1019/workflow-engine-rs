// File: src/db/events/snapshots.rs
//
// Enhanced snapshot management with compression and optimization
// Provides efficient snapshot creation, restoration, and lifecycle management

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use uuid::Uuid;

use super::{
    EventStore, EventEnvelope, EventError, EventResult, AggregateSnapshot,
};

/// Compression types supported for snapshots
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CompressionType {
    None,
    Gzip,
    Lz4,
    Zstd,
}

impl Default for CompressionType {
    fn default() -> Self {
        CompressionType::Gzip
    }
}

impl std::fmt::Display for CompressionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompressionType::None => write!(f, "none"),
            CompressionType::Gzip => write!(f, "gzip"),
            CompressionType::Lz4 => write!(f, "lz4"),
            CompressionType::Zstd => write!(f, "zstd"),
        }
    }
}

/// Configuration for snapshot management
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SnapshotConfig {
    /// Frequency of snapshot creation (every N events)
    pub snapshot_frequency: i64,
    /// Compression type to use
    pub compression_type: CompressionType,
    /// Minimum compression ratio to keep compressed version
    pub min_compression_ratio: f32,
    /// Maximum age of snapshots to keep (in days)
    pub max_age_days: i32,
    /// Maximum number of snapshots per aggregate
    pub max_snapshots_per_aggregate: usize,
    /// Minimum size threshold for compression (bytes)
    pub compression_threshold_bytes: usize,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            snapshot_frequency: 100,
            compression_type: CompressionType::Gzip,
            min_compression_ratio: 0.8,
            max_age_days: 90,
            max_snapshots_per_aggregate: 5,
            compression_threshold_bytes: 1024, // 1KB
        }
    }
}

/// Enhanced snapshot with compression metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedSnapshot {
    pub id: Uuid,
    pub aggregate_id: Uuid,
    pub aggregate_type: String,
    pub aggregate_version: i64,
    pub snapshot_data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub compression_type: CompressionType,
    pub original_size: usize,
    pub compressed_size: usize,
    pub checksum: Option<String>,
}

impl EnhancedSnapshot {
    pub fn new(
        aggregate_id: Uuid,
        aggregate_type: String,
        aggregate_version: i64,
        snapshot_data: serde_json::Value,
    ) -> Self {
        let data_str = snapshot_data.to_string();
        let original_size = data_str.len();
        
        Self {
            id: Uuid::new_v4(),
            aggregate_id,
            aggregate_type,
            aggregate_version,
            snapshot_data,
            created_at: Utc::now(),
            metadata: HashMap::new(),
            compression_type: CompressionType::None,
            original_size,
            compressed_size: original_size,
            checksum: None,
        }
    }
    
    /// Get compression ratio (compressed_size / original_size)
    pub fn compression_ratio(&self) -> f32 {
        if self.original_size == 0 {
            1.0
        } else {
            self.compressed_size as f32 / self.original_size as f32
        }
    }
    
    /// Get space saved by compression
    pub fn space_saved_bytes(&self) -> usize {
        if self.compressed_size < self.original_size {
            self.original_size - self.compressed_size
        } else {
            0
        }
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Statistics about snapshot operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotStatistics {
    pub total_snapshots: u64,
    pub compressed_snapshots: u64,
    pub total_original_size: u64,
    pub total_compressed_size: u64,
    pub average_compression_ratio: f32,
    pub total_space_saved: u64,
    pub snapshots_by_type: HashMap<String, u64>,
    pub compression_by_type: HashMap<CompressionType, u64>,
}

/// Trait for compressing and decompressing snapshot data
#[async_trait]
pub trait SnapshotCompressor: Send + Sync {
    /// Compress snapshot data
    async fn compress(&self, data: &[u8]) -> EventResult<Vec<u8>>;
    
    /// Decompress snapshot data
    async fn decompress(&self, compressed_data: &[u8]) -> EventResult<Vec<u8>>;
    
    /// Get compression type
    fn compression_type(&self) -> CompressionType;
}

/// Gzip compressor implementation
pub struct GzipCompressor;

#[async_trait]
impl SnapshotCompressor for GzipCompressor {
    async fn compress(&self, data: &[u8]) -> EventResult<Vec<u8>> {
        use flate2::{Compression, write::GzEncoder};
        use std::io::Write;
        
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data).map_err(|e| EventError::SerializationError {
            message: format!("Gzip compression failed: {}", e),
        })?;
        
        encoder.finish().map_err(|e| EventError::SerializationError {
            message: format!("Gzip compression finalization failed: {}", e),
        })
    }
    
    async fn decompress(&self, compressed_data: &[u8]) -> EventResult<Vec<u8>> {
        use flate2::read::GzDecoder;
        use std::io::Read;
        
        let mut decoder = GzDecoder::new(compressed_data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).map_err(|e| EventError::SerializationError {
            message: format!("Gzip decompression failed: {}", e),
        })?;
        
        Ok(decompressed)
    }
    
    fn compression_type(&self) -> CompressionType {
        CompressionType::Gzip
    }
}

/// LZ4 compressor implementation
pub struct Lz4Compressor;

#[async_trait]
impl SnapshotCompressor for Lz4Compressor {
    async fn compress(&self, data: &[u8]) -> EventResult<Vec<u8>> {
        Ok(lz4_flex::compress_prepend_size(data))
    }
    
    async fn decompress(&self, compressed_data: &[u8]) -> EventResult<Vec<u8>> {
        lz4_flex::decompress_size_prepended(compressed_data).map_err(|e| EventError::SerializationError {
            message: format!("LZ4 decompression failed: {}", e),
        })
    }
    
    fn compression_type(&self) -> CompressionType {
        CompressionType::Lz4
    }
}

/// Manager for enhanced snapshot operations
pub struct EnhancedSnapshotManager {
    event_store: Arc<dyn EventStore>,
    config: SnapshotConfig,
    compressors: HashMap<CompressionType, Box<dyn SnapshotCompressor>>,
    statistics: Arc<RwLock<SnapshotStatistics>>,
}

impl Clone for EnhancedSnapshotManager {
    fn clone(&self) -> Self {
        let mut compressors: HashMap<CompressionType, Box<dyn SnapshotCompressor>> = HashMap::new();
        compressors.insert(CompressionType::Gzip, Box::new(GzipCompressor));
        compressors.insert(CompressionType::Lz4, Box::new(Lz4Compressor));
        
        Self {
            event_store: Arc::clone(&self.event_store),
            config: self.config.clone(),
            compressors,
            statistics: Arc::clone(&self.statistics),
        }
    }
}

impl EnhancedSnapshotManager {
    pub fn new(event_store: Arc<dyn EventStore>, config: SnapshotConfig) -> Self {
        let mut compressors: HashMap<CompressionType, Box<dyn SnapshotCompressor>> = HashMap::new();
        compressors.insert(CompressionType::Gzip, Box::new(GzipCompressor));
        compressors.insert(CompressionType::Lz4, Box::new(Lz4Compressor));
        
        let statistics = SnapshotStatistics {
            total_snapshots: 0,
            compressed_snapshots: 0,
            total_original_size: 0,
            total_compressed_size: 0,
            average_compression_ratio: 1.0,
            total_space_saved: 0,
            snapshots_by_type: HashMap::new(),
            compression_by_type: HashMap::new(),
        };
        
        Self {
            event_store,
            config,
            compressors,
            statistics: Arc::new(RwLock::new(statistics)),
        }
    }
    
    /// Create a snapshot with automatic compression
    pub async fn create_snapshot(
        &self,
        aggregate_id: Uuid,
        aggregate_type: String,
        aggregate_version: i64,
        snapshot_data: serde_json::Value,
    ) -> EventResult<EnhancedSnapshot> {
        let mut snapshot = EnhancedSnapshot::new(
            aggregate_id,
            aggregate_type.clone(),
            aggregate_version,
            snapshot_data,
        );
        
        // Calculate checksum
        snapshot.checksum = Some(self.calculate_checksum(&snapshot.snapshot_data));
        
        // Apply compression if configured and data is large enough
        if self.config.compression_type != CompressionType::None 
            && snapshot.original_size >= self.config.compression_threshold_bytes {
            
            snapshot = self.compress_snapshot(snapshot).await?;
        }
        
        // Convert to basic snapshot for storage
        let basic_snapshot = self.to_basic_snapshot(&snapshot);
        
        // Store in event store
        self.event_store.save_snapshot(&basic_snapshot).await?;
        
        // Update statistics
        self.update_statistics(&snapshot).await;
        
        info!(
            "Created snapshot for aggregate {} at version {} (compression: {}, ratio: {:.2})",
            aggregate_id,
            aggregate_version,
            snapshot.compression_type,
            snapshot.compression_ratio()
        );
        
        Ok(snapshot)
    }
    
    /// Restore a snapshot with automatic decompression
    pub async fn restore_snapshot(&self, aggregate_id: Uuid) -> EventResult<Option<EnhancedSnapshot>> {
        // Get basic snapshot from event store
        let basic_snapshot = match self.event_store.get_snapshot(aggregate_id).await? {
            Some(snapshot) => snapshot,
            None => return Ok(None),
        };
        
        // Convert to enhanced snapshot
        let mut enhanced = self.from_basic_snapshot(&basic_snapshot);
        
        // Decompress if needed
        if enhanced.compression_type != CompressionType::None {
            enhanced = self.decompress_snapshot(enhanced).await?;
        }
        
        // Verify checksum if available
        if let Some(expected_checksum) = &enhanced.checksum {
            let actual_checksum = self.calculate_checksum(&enhanced.snapshot_data);
            if &actual_checksum != expected_checksum {
                return Err(EventError::SerializationError {
                    message: format!("Snapshot checksum mismatch for aggregate {}", aggregate_id),
                });
            }
        }
        
        info!(
            "Restored snapshot for aggregate {} at version {} (compression: {})",
            aggregate_id,
            enhanced.aggregate_version,
            enhanced.compression_type
        );
        
        Ok(Some(enhanced))
    }
    
    /// Check if an aggregate should create a snapshot
    pub async fn should_create_snapshot(
        &self,
        aggregate_id: Uuid,
        current_version: i64,
    ) -> EventResult<bool> {
        // Get latest snapshot version
        let latest_snapshot_version = if let Some(snapshot) = self.event_store.get_snapshot(aggregate_id).await? {
            snapshot.aggregate_version
        } else {
            0
        };
        
        // Check if enough events have passed since last snapshot
        let events_since_snapshot = current_version - latest_snapshot_version;
        Ok(events_since_snapshot >= self.config.snapshot_frequency)
    }
    
    /// Cleanup old snapshots based on configuration
    pub async fn cleanup_old_snapshots(&self) -> EventResult<usize> {
        info!("Starting snapshot cleanup process");
        
        // For now, use the basic cleanup from event store
        // In a more advanced implementation, we would:
        // 1. Get all snapshots grouped by aggregate
        // 2. Keep only the latest N per aggregate
        // 3. Remove snapshots older than max_age_days
        
        let deleted_count = self.event_store
            .cleanup_old_snapshots(self.config.max_snapshots_per_aggregate)
            .await?;
        
        info!("Cleaned up {} old snapshots", deleted_count);
        Ok(deleted_count)
    }
    
    /// Get snapshot statistics
    pub async fn get_statistics(&self) -> SnapshotStatistics {
        self.statistics.read().await.clone()
    }
    
    /// Recompress existing snapshots with a different algorithm
    pub async fn recompress_snapshots(
        &self,
        new_compression_type: CompressionType,
    ) -> EventResult<u64> {
        // This would be implemented to:
        // 1. Get all snapshots
        // 2. Decompress them
        // 3. Recompress with new algorithm
        // 4. Save back to store
        // For now, return a placeholder
        
        info!(
            "Recompressing snapshots with {} compression",
            new_compression_type
        );
        
        // Implementation would go here
        Ok(0)
    }
    
    /// Compress a snapshot
    async fn compress_snapshot(&self, mut snapshot: EnhancedSnapshot) -> EventResult<EnhancedSnapshot> {
        let compressor = self.compressors.get(&self.config.compression_type)
            .ok_or_else(|| EventError::ConfigurationError {
                message: format!("Compressor not available for type: {}", self.config.compression_type),
            })?;
        
        let data_bytes = snapshot.snapshot_data.to_string().into_bytes();
        let compressed_data = compressor.compress(&data_bytes).await?;
        
        // Check compression ratio
        let compression_ratio = compressed_data.len() as f32 / data_bytes.len() as f32;
        
        if compression_ratio <= self.config.min_compression_ratio {
            // Compression was effective, keep it
            snapshot.snapshot_data = serde_json::Value::String(
                base64::encode(&compressed_data)
            );
            snapshot.compression_type = compressor.compression_type();
            snapshot.compressed_size = compressed_data.len();
        } else {
            // Compression not effective, keep original
            warn!(
                "Compression ratio {} too high for aggregate {}, keeping uncompressed",
                compression_ratio,
                snapshot.aggregate_id
            );
        }
        
        Ok(snapshot)
    }
    
    /// Decompress a snapshot
    async fn decompress_snapshot(&self, mut snapshot: EnhancedSnapshot) -> EventResult<EnhancedSnapshot> {
        let compressor = self.compressors.get(&snapshot.compression_type)
            .ok_or_else(|| EventError::ConfigurationError {
                message: format!("Compressor not available for type: {}", snapshot.compression_type),
            })?;
        
        // Extract compressed data from JSON
        let compressed_data = match &snapshot.snapshot_data {
            serde_json::Value::String(encoded) => base64::decode(encoded).map_err(|e| {
                EventError::SerializationError {
                    message: format!("Failed to decode base64 snapshot data: {}", e),
                }
            })?,
            _ => return Err(EventError::SerializationError {
                message: "Compressed snapshot data should be base64 encoded string".to_string(),
            }),
        };
        
        // Decompress
        let decompressed_data = compressor.decompress(&compressed_data).await?;
        
        // Parse back to JSON
        let decompressed_str = String::from_utf8(decompressed_data).map_err(|e| {
            EventError::SerializationError {
                message: format!("Decompressed data is not valid UTF-8: {}", e),
            }
        })?;
        
        snapshot.snapshot_data = serde_json::from_str(&decompressed_str).map_err(|e| {
            EventError::SerializationError {
                message: format!("Failed to parse decompressed JSON: {}", e),
            }
        })?;
        
        // Reset compression info
        snapshot.compression_type = CompressionType::None;
        snapshot.compressed_size = snapshot.original_size;
        
        Ok(snapshot)
    }
    
    /// Convert enhanced snapshot to basic snapshot for storage
    fn to_basic_snapshot(&self, enhanced: &EnhancedSnapshot) -> AggregateSnapshot {
        let mut metadata = enhanced.metadata.clone();
        metadata.insert("compression_type".to_string(), 
                       serde_json::Value::String(enhanced.compression_type.to_string()));
        metadata.insert("original_size".to_string(), 
                       serde_json::Value::Number(enhanced.original_size.into()));
        metadata.insert("compressed_size".to_string(), 
                       serde_json::Value::Number(enhanced.compressed_size.into()));
        if let Some(checksum) = &enhanced.checksum {
            metadata.insert("checksum".to_string(), 
                           serde_json::Value::String(checksum.clone()));
        }
        
        AggregateSnapshot {
            id: enhanced.id,
            aggregate_id: enhanced.aggregate_id,
            aggregate_type: enhanced.aggregate_type.clone(),
            aggregate_version: enhanced.aggregate_version,
            snapshot_data: enhanced.snapshot_data.clone(),
            created_at: enhanced.created_at,
            metadata,
        }
    }
    
    /// Convert basic snapshot to enhanced snapshot
    fn from_basic_snapshot(&self, basic: &AggregateSnapshot) -> EnhancedSnapshot {
        let compression_type = basic.metadata.get("compression_type")
            .and_then(|v| v.as_str())
            .and_then(|s| match s {
                "gzip" => Some(CompressionType::Gzip),
                "lz4" => Some(CompressionType::Lz4),
                "zstd" => Some(CompressionType::Zstd),
                _ => Some(CompressionType::None),
            })
            .unwrap_or(CompressionType::None);
        
        let original_size = basic.metadata.get("original_size")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
        
        let compressed_size = basic.metadata.get("compressed_size")
            .and_then(|v| v.as_u64())
            .unwrap_or(original_size as u64) as usize;
        
        let checksum = basic.metadata.get("checksum")
            .and_then(|v| v.as_str())
            .map(String::from);
        
        EnhancedSnapshot {
            id: basic.id,
            aggregate_id: basic.aggregate_id,
            aggregate_type: basic.aggregate_type.clone(),
            aggregate_version: basic.aggregate_version,
            snapshot_data: basic.snapshot_data.clone(),
            created_at: basic.created_at,
            metadata: basic.metadata.clone(),
            compression_type,
            original_size,
            compressed_size,
            checksum,
        }
    }
    
    /// Calculate checksum for snapshot data
    fn calculate_checksum(&self, data: &serde_json::Value) -> String {
        use sha2::{Sha256, Digest};
        let data_str = data.to_string();
        let mut hasher = Sha256::new();
        hasher.update(data_str.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    /// Update statistics
    async fn update_statistics(&self, snapshot: &EnhancedSnapshot) {
        let mut stats = self.statistics.write().await;
        
        stats.total_snapshots += 1;
        stats.total_original_size += snapshot.original_size as u64;
        stats.total_compressed_size += snapshot.compressed_size as u64;
        
        if snapshot.compression_type != CompressionType::None {
            stats.compressed_snapshots += 1;
            *stats.compression_by_type.entry(snapshot.compression_type).or_insert(0) += 1;
        }
        
        stats.total_space_saved += snapshot.space_saved_bytes() as u64;
        
        *stats.snapshots_by_type.entry(snapshot.aggregate_type.clone()).or_insert(0) += 1;
        
        // Recalculate average compression ratio
        if stats.total_original_size > 0 {
            stats.average_compression_ratio = 
                stats.total_compressed_size as f32 / stats.total_original_size as f32;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_enhanced_snapshot_creation() {
        let aggregate_id = Uuid::new_v4();
        let snapshot_data = json!({"state": "test", "count": 42});
        
        let snapshot = EnhancedSnapshot::new(
            aggregate_id,
            "test_aggregate".to_string(),
            10,
            snapshot_data.clone(),
        );
        
        assert_eq!(snapshot.aggregate_id, aggregate_id);
        assert_eq!(snapshot.aggregate_type, "test_aggregate");
        assert_eq!(snapshot.aggregate_version, 10);
        assert_eq!(snapshot.snapshot_data, snapshot_data);
        assert_eq!(snapshot.compression_type, CompressionType::None);
        assert_eq!(snapshot.compression_ratio(), 1.0);
        assert_eq!(snapshot.space_saved_bytes(), 0);
    }
    
    #[test]
    fn test_compression_type_display() {
        assert_eq!(CompressionType::None.to_string(), "none");
        assert_eq!(CompressionType::Gzip.to_string(), "gzip");
        assert_eq!(CompressionType::Lz4.to_string(), "lz4");
        assert_eq!(CompressionType::Zstd.to_string(), "zstd");
    }
    
    #[test]
    fn test_snapshot_config_defaults() {
        let config = SnapshotConfig::default();
        assert_eq!(config.snapshot_frequency, 100);
        assert_eq!(config.compression_type, CompressionType::Gzip);
        assert_eq!(config.min_compression_ratio, 0.8);
        assert_eq!(config.max_age_days, 90);
        assert_eq!(config.max_snapshots_per_aggregate, 5);
        assert_eq!(config.compression_threshold_bytes, 1024);
    }
    
    #[tokio::test]
    async fn test_gzip_compressor() {
        let compressor = GzipCompressor;
        let test_data = b"Hello, world! This is a test string that should compress well with repetition. Hello, world! This is a test string that should compress well with repetition.";
        
        let compressed = compressor.compress(test_data).await.unwrap();
        assert!(compressed.len() < test_data.len()); // Should be smaller
        
        let decompressed = compressor.decompress(&compressed).await.unwrap();
        assert_eq!(decompressed, test_data);
    }
    
    #[tokio::test]
    async fn test_lz4_compressor() {
        let compressor = Lz4Compressor;
        let test_data = b"Hello, world! This is a test string for LZ4 compression. Hello, world! This is a test string for LZ4 compression.";
        
        let compressed = compressor.compress(test_data).await.unwrap();
        let decompressed = compressor.decompress(&compressed).await.unwrap();
        assert_eq!(decompressed, test_data);
    }
}