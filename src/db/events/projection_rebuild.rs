// File: src/db/events/projection_rebuild.rs
//
// Advanced projection rebuild functionality using the event replay engine
// Provides efficient rebuild of read models and projections

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{info, warn, error};
use uuid::Uuid;

use super::{
    EventStore, EventEnvelope, EventError, EventResult,
    replay::{EventReplayEngine, ReplayHandler, ReplayConfig, ReplayPosition},
    projections::{Projection, ProjectionState},
};

/// Configuration for projection rebuilds
#[derive(Debug, Clone)]
pub struct ProjectionRebuildConfig {
    /// Batch size for processing events
    pub batch_size: usize,
    /// Number of parallel rebuild tasks
    pub parallelism: usize,
    /// Whether to use incremental rebuilds when possible
    pub incremental_rebuild: bool,
    /// Maximum age for incremental rebuilds (in hours)
    pub max_incremental_age_hours: i64,
    /// Timeout for rebuild operations (in seconds)
    pub rebuild_timeout_seconds: u64,
}

impl Default for ProjectionRebuildConfig {
    fn default() -> Self {
        Self {
            batch_size: 1000,
            parallelism: 2,
            incremental_rebuild: true,
            max_incremental_age_hours: 24,
            rebuild_timeout_seconds: 3600, // 1 hour
        }
    }
}

/// Metadata about a projection rebuild
#[derive(Debug, Clone)]
pub struct RebuildMetadata {
    pub projection_name: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub events_processed: u64,
    pub from_position: i64,
    pub to_position: i64,
    pub is_incremental: bool,
    pub error: Option<String>,
}

/// Statistics for projection rebuilds
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RebuildStatistics {
    pub total_rebuilds: u64,
    pub successful_rebuilds: u64,
    pub failed_rebuilds: u64,
    pub average_rebuild_time_seconds: f64,
    pub total_events_processed: u64,
    pub average_throughput_events_per_second: f64,
}

/// Manager for rebuilding projections efficiently
pub struct ProjectionRebuildManager {
    event_store: Arc<dyn EventStore>,
    replay_engine: EventReplayEngine,
    config: ProjectionRebuildConfig,
    rebuild_history: Arc<RwLock<Vec<RebuildMetadata>>>,
    active_rebuilds: Arc<RwLock<HashMap<String, RebuildMetadata>>>,
}

impl ProjectionRebuildManager {
    pub fn new(
        event_store: Arc<dyn EventStore>,
        config: ProjectionRebuildConfig,
    ) -> Self {
        let replay_config = ReplayConfig {
            batch_size: config.batch_size,
            parallelism: config.parallelism,
            ..Default::default()
        };
        
        let replay_engine = EventReplayEngine::new(event_store.clone(), replay_config);
        
        Self {
            event_store,
            replay_engine,
            config,
            rebuild_history: Arc::new(RwLock::new(Vec::new())),
            active_rebuilds: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Rebuild a single projection
    pub async fn rebuild_projection<P>(
        &self,
        projection: Arc<Mutex<P>>,
        event_types: Option<Vec<String>>,
        force_full_rebuild: bool,
    ) -> EventResult<RebuildMetadata>
    where
        P: Projection + ReplayHandler + Send + Sync + 'static,
    {
        let projection_name = {
            let p = projection.lock().await;
            p.name().to_string()
        };
        
        info!("Starting rebuild for projection '{}'", projection_name);
        
        // Check if already rebuilding
        {
            let active = self.active_rebuilds.read().await;
            if active.contains_key(&projection_name) {
                return Err(EventError::HandlerError {
                    message: format!("Projection '{}' is already being rebuilt", projection_name),
                });
            }
        }
        
        let started_at = Utc::now();
        let mut metadata = RebuildMetadata {
            projection_name: projection_name.clone(),
            started_at,
            completed_at: None,
            events_processed: 0,
            from_position: 0,
            to_position: 0,
            is_incremental: false,
            error: None,
        };
        
        // Mark as active
        {
            let mut active = self.active_rebuilds.write().await;
            active.insert(projection_name.clone(), metadata.clone());
        }
        
        let result = self.execute_rebuild(
            projection,
            event_types,
            force_full_rebuild,
            &mut metadata,
        ).await;
        
        // Remove from active and add to history
        {
            let mut active = self.active_rebuilds.write().await;
            active.remove(&projection_name);
        }
        
        metadata.completed_at = Some(Utc::now());
        
        match &result {
            Ok(position) => {
                metadata.to_position = position.position;
                metadata.events_processed = position.events_processed;
                
                let duration = (metadata.completed_at.unwrap() - metadata.started_at).num_seconds();
                info!(
                    "Completed rebuild for projection '{}' in {} seconds: {} events processed",
                    projection_name, duration, metadata.events_processed
                );
            }
            Err(error) => {
                metadata.error = Some(error.to_string());
                error!(
                    "Failed to rebuild projection '{}': {}",
                    projection_name, error
                );
            }
        }
        
        // Add to history
        {
            let mut history = self.rebuild_history.write().await;
            history.push(metadata.clone());
            
            // Keep only last 100 rebuilds
            if history.len() > 100 {
                history.remove(0);
            }
        }
        
        result.map(|_| metadata)
    }
    
    /// Rebuild multiple projections in parallel
    pub async fn rebuild_projections_parallel<P>(
        &self,
        projections: Vec<(Arc<Mutex<P>>, Option<Vec<String>>)>,
        force_full_rebuild: bool,
    ) -> EventResult<Vec<RebuildMetadata>>
    where
        P: Projection + ReplayHandler + Send + Sync + 'static,
    {
        let mut tasks = Vec::new();
        
        for (projection, event_types) in projections {
            let manager = self.clone();
            
            let task = tokio::spawn(async move {
                manager.rebuild_projection(projection, event_types, force_full_rebuild).await
            });
            
            tasks.push(task);
        }
        
        let mut results = Vec::new();
        for task in tasks {
            match task.await {
                Ok(Ok(metadata)) => results.push(metadata),
                Ok(Err(e)) => return Err(e),
                Err(e) => return Err(EventError::HandlerError {
                    message: format!("Task join error: {}", e),
                }),
            }
        }
        
        Ok(results)
    }
    
    /// Check if a projection needs rebuilding
    pub async fn needs_rebuild<P>(
        &self,
        projection: &Arc<Mutex<P>>,
    ) -> EventResult<bool>
    where
        P: Projection + Send + Sync,
    {
        let projection_guard = projection.lock().await;
        let state = projection_guard.state();
        
        match state {
            ProjectionState::Building | ProjectionState::Rebuilding => Ok(false), // Already building
            ProjectionState::Failed => Ok(true), // Failed, needs rebuild
            ProjectionState::Ready => {
                // Check if it's too old for incremental update
                if let Some(last_updated) = projection_guard.last_updated() {
                    let age_hours = (Utc::now() - last_updated).num_hours();
                    Ok(age_hours > self.config.max_incremental_age_hours)
                } else {
                    Ok(true) // Never updated
                }
            }
            ProjectionState::Active => {
                // Check if it's too old for incremental update
                if let Some(last_updated) = projection_guard.last_updated() {
                    let age_hours = (Utc::now() - last_updated).num_hours();
                    Ok(age_hours > self.config.max_incremental_age_hours)
                } else {
                    Ok(true) // Never updated
                }
            }
            ProjectionState::Stopped => Ok(true), // Stopped, may need rebuild
        }
    }
    
    /// Get current rebuild statistics
    pub async fn get_statistics(&self) -> RebuildStatistics {
        let history = self.rebuild_history.read().await;
        
        let total_rebuilds = history.len() as u64;
        let successful_rebuilds = history.iter()
            .filter(|m| m.error.is_none())
            .count() as u64;
        let failed_rebuilds = total_rebuilds - successful_rebuilds;
        
        let total_events_processed: u64 = history.iter()
            .map(|m| m.events_processed)
            .sum();
        
        let total_duration_seconds: f64 = history.iter()
            .filter_map(|m| m.completed_at.map(|end| (end - m.started_at).num_seconds()))
            .map(|s| s as f64)
            .sum();
        
        let average_rebuild_time_seconds = if total_rebuilds > 0 {
            total_duration_seconds / total_rebuilds as f64
        } else {
            0.0
        };
        
        let average_throughput_events_per_second = if total_duration_seconds > 0.0 {
            total_events_processed as f64 / total_duration_seconds
        } else {
            0.0
        };
        
        RebuildStatistics {
            total_rebuilds,
            successful_rebuilds,
            failed_rebuilds,
            average_rebuild_time_seconds,
            total_events_processed,
            average_throughput_events_per_second,
        }
    }
    
    /// Get active rebuilds
    pub async fn get_active_rebuilds(&self) -> Vec<RebuildMetadata> {
        self.active_rebuilds.read().await.values().cloned().collect()
    }
    
    /// Get rebuild history
    pub async fn get_rebuild_history(&self) -> Vec<RebuildMetadata> {
        self.rebuild_history.read().await.clone()
    }
    
    /// Cancel an active rebuild
    pub async fn cancel_rebuild(&self, projection_name: &str) -> EventResult<()> {
        let mut active = self.active_rebuilds.write().await;
        
        if let Some(mut metadata) = active.remove(projection_name) {
            metadata.error = Some("Cancelled by user".to_string());
            metadata.completed_at = Some(Utc::now());
            
            let mut history = self.rebuild_history.write().await;
            history.push(metadata);
            
            info!("Cancelled rebuild for projection '{}'", projection_name);
            Ok(())
        } else {
            Err(EventError::HandlerError {
                message: format!("No active rebuild found for projection '{}'", projection_name),
            })
        }
    }
    
    /// Execute the actual rebuild
    async fn execute_rebuild<P>(
        &self,
        projection: Arc<Mutex<P>>,
        event_types: Option<Vec<String>>,
        force_full_rebuild: bool,
        metadata: &mut RebuildMetadata,
    ) -> EventResult<ReplayPosition>
    where
        P: Projection + ReplayHandler + Send + Sync,
    {
        // Determine starting position
        let from_position = if force_full_rebuild || !self.config.incremental_rebuild {
            // Full rebuild from the beginning
            {
                let mut p = projection.lock().await;
                p.reset().await?;
            }
            0
        } else {
            // Try incremental rebuild
            let p = projection.lock().await;
            if let Some(last_position) = p.last_position() {
                metadata.is_incremental = true;
                last_position
            } else {
                // No position available, do full rebuild
                drop(p);
                let mut p = projection.lock().await;
                p.reset().await?;
                0
            }
        };
        
        metadata.from_position = from_position;
        
        info!(
            "Rebuilding projection '{}' from position {} ({})",
            metadata.projection_name,
            from_position,
            if metadata.is_incremental { "incremental" } else { "full" }
        );
        
        // Set projection state to rebuilding
        {
            let mut p = projection.lock().await;
            p.set_state(ProjectionState::Rebuilding).await?;
        }
        
        // Perform the replay with timeout
        let rebuild_future = self.replay_engine.replay_for_handler(
            projection.clone(),
            event_types,
        );
        
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(self.config.rebuild_timeout_seconds),
            rebuild_future,
        ).await;
        
        match result {
            Ok(Ok(position)) => {
                // Mark projection as ready
                let mut p = projection.lock().await;
                p.set_state(ProjectionState::Ready).await?;
                Ok(position)
            }
            Ok(Err(e)) => {
                // Mark projection as failed
                let mut p = projection.lock().await;
                p.set_state(ProjectionState::Failed).await?;
                Err(e)
            }
            Err(_) => {
                // Timeout
                let mut p = projection.lock().await;
                p.set_state(ProjectionState::Failed).await?;
                Err(EventError::HandlerError {
                    message: format!(
                        "Rebuild timeout after {} seconds",
                        self.config.rebuild_timeout_seconds
                    ),
                })
            }
        }
    }
}

impl Clone for ProjectionRebuildManager {
    fn clone(&self) -> Self {
        Self {
            event_store: self.event_store.clone(),
            replay_engine: self.replay_engine.clone(),
            config: self.config.clone(),
            rebuild_history: self.rebuild_history.clone(),
            active_rebuilds: self.active_rebuilds.clone(),
        }
    }
}

/// Utility for batch rebuilding all projections
pub struct BatchProjectionRebuilder {
    manager: ProjectionRebuildManager,
}

impl BatchProjectionRebuilder {
    pub fn new(manager: ProjectionRebuildManager) -> Self {
        Self { manager }
    }
    
    /// Rebuild all projections that need it
    pub async fn rebuild_all_needed<P>(
        &self,
        projections: Vec<(Arc<Mutex<P>>, Option<Vec<String>>)>,
    ) -> EventResult<Vec<RebuildMetadata>>
    where
        P: Projection + ReplayHandler + Send + Sync + 'static,
    {
        let mut to_rebuild = Vec::new();
        
        // Check which projections need rebuilding
        for (projection, event_types) in projections {
            if self.manager.needs_rebuild(&projection).await? {
                to_rebuild.push((projection, event_types));
            }
        }
        
        if to_rebuild.is_empty() {
            info!("No projections need rebuilding");
            return Ok(Vec::new());
        }
        
        info!("Rebuilding {} projections that need updating", to_rebuild.len());
        
        // Rebuild in parallel
        self.manager.rebuild_projections_parallel(to_rebuild, false).await
    }
    
    /// Force rebuild of all projections
    pub async fn rebuild_all_force<P>(
        &self,
        projections: Vec<(Arc<Mutex<P>>, Option<Vec<String>>)>,
    ) -> EventResult<Vec<RebuildMetadata>>
    where
        P: Projection + ReplayHandler + Send + Sync + 'static,
    {
        info!("Force rebuilding {} projections", projections.len());
        self.manager.rebuild_projections_parallel(projections, true).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    
    struct TestProjection {
        name: String,
        state: ProjectionState,
        events_processed: Arc<AtomicU64>,
        last_position: Option<i64>,
        reset_called: Arc<AtomicBool>,
        initialized: Arc<AtomicBool>,
    }
    
    impl TestProjection {
        fn new(name: String) -> Self {
            Self {
                name,
                state: ProjectionState::Ready,
                events_processed: Arc::new(AtomicU64::new(0)),
                last_position: None,
                reset_called: Arc::new(AtomicBool::new(false)),
                initialized: Arc::new(AtomicBool::new(false)),
            }
        }
    }
    
    #[async_trait]
    impl Projection for TestProjection {
        fn name(&self) -> &str {
            &self.name
        }
        
        fn state(&self) -> ProjectionState {
            self.state.clone()
        }
        
        async fn set_state(&mut self, state: ProjectionState) -> EventResult<()> {
            self.state = state;
            Ok(())
        }
        
        fn last_position(&self) -> Option<i64> {
            self.last_position
        }
        
        fn last_updated(&self) -> Option<DateTime<Utc>> {
            Some(Utc::now() - chrono::Duration::hours(1))
        }
        
        async fn reset(&mut self) -> EventResult<()> {
            self.reset_called.store(true, Ordering::SeqCst);
            self.events_processed.store(0, Ordering::SeqCst);
            self.last_position = None;
            Ok(())
        }
        
        fn event_types(&self) -> Vec<String> {
            vec!["test_event".to_string()]
        }
        
        async fn handle_event(&mut self, _event: &crate::db::events::EventEnvelope) -> EventResult<()> {
            self.events_processed.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
        
        async fn initialize(&mut self) -> EventResult<()> {
            self.initialized.store(true, Ordering::SeqCst);
            Ok(())
        }
    }
    
    #[async_trait]
    impl ReplayHandler for TestProjection {
        async fn handle_events(&mut self, events: &[EventEnvelope]) -> EventResult<()> {
            self.events_processed.fetch_add(events.len() as u64, Ordering::SeqCst);
            
            if let Some(last_event) = events.last() {
                self.last_position = Some(last_event.recorded_at.timestamp_millis());
            }
            
            Ok(())
        }
        
        fn consumer_name(&self) -> &str {
            &self.name
        }
    }
    
    #[test]
    fn test_rebuild_config_defaults() {
        let config = ProjectionRebuildConfig::default();
        assert_eq!(config.batch_size, 1000);
        assert_eq!(config.parallelism, 2);
        assert!(config.incremental_rebuild);
        assert_eq!(config.max_incremental_age_hours, 24);
        assert_eq!(config.rebuild_timeout_seconds, 3600);
    }
    
    #[test]
    fn test_rebuild_metadata() {
        let metadata = RebuildMetadata {
            projection_name: "test_projection".to_string(),
            started_at: Utc::now(),
            completed_at: None,
            events_processed: 0,
            from_position: 0,
            to_position: 0,
            is_incremental: false,
            error: None,
        };
        
        assert_eq!(metadata.projection_name, "test_projection");
        assert!(!metadata.is_incremental);
        assert!(metadata.error.is_none());
    }
}