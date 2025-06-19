use std::time::Duration;
use tokio::time::sleep;
use std::sync::Arc;

use crate::registry::{AgentRegistry, AgentRegistryError};

/// Background task configuration
#[derive(Debug, Clone)]
pub struct BackgroundTaskConfig {
    /// How often to run the cleanup task (in seconds)
    pub cleanup_interval_seconds: u64,
    /// Consider agents inactive after this many minutes without heartbeat
    pub heartbeat_timeout_minutes: i64,
}

impl Default for BackgroundTaskConfig {
    fn default() -> Self {
        Self {
            cleanup_interval_seconds: 60, // Run every minute
            heartbeat_timeout_minutes: 5, // 5 minutes timeout
        }
    }
}

/// Background task manager for agent registry maintenance
pub struct RegistryBackgroundTasks<R: AgentRegistry> {
    registry: Arc<R>,
    config: BackgroundTaskConfig,
}

impl<R: AgentRegistry + 'static> RegistryBackgroundTasks<R> {
    pub fn new(registry: Arc<R>, config: BackgroundTaskConfig) -> Self {
        Self {
            registry,
            config,
        }
    }

    /// Start the background task to mark stale agents as inactive
    pub async fn start_cleanup_task(&self) {
        let registry = self.registry.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            loop {
                match registry.mark_inactive_stale(config.heartbeat_timeout_minutes).await {
                    Ok(count) => {
                        if count > 0 {
                            log::info!("Marked {} stale agents as inactive", count);
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to cleanup stale agents: {:?}", e);
                    }
                }
                
                sleep(Duration::from_secs(config.cleanup_interval_seconds)).await;
            }
        });
        
        log::info!(
            "Started agent cleanup task (interval: {}s, timeout: {}min)",
            self.config.cleanup_interval_seconds,
            self.config.heartbeat_timeout_minutes
        );
    }

    /// Run cleanup once (for testing or manual trigger)
    pub async fn run_cleanup_once(&self) -> Result<usize, AgentRegistryError> {
        self.registry.mark_inactive_stale(self.config.heartbeat_timeout_minutes).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = BackgroundTaskConfig::default();
        assert_eq!(config.cleanup_interval_seconds, 60);
        assert_eq!(config.heartbeat_timeout_minutes, 5);
    }

    #[test]
    fn test_custom_config() {
        let config = BackgroundTaskConfig {
            cleanup_interval_seconds: 30,
            heartbeat_timeout_minutes: 10,
        };
        assert_eq!(config.cleanup_interval_seconds, 30);
        assert_eq!(config.heartbeat_timeout_minutes, 10);
    }
}