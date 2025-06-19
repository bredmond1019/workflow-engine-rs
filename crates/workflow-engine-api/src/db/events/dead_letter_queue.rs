// File: src/db/events/dead_letter_queue.rs
//
// Dead letter queue implementation for handling failed event processing

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::time::interval;
use uuid::Uuid;

use crate::db::schema::event_dead_letter_queue;
use super::{EventError, EventResult, EventEnvelope};

/// Configuration for dead letter queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterConfig {
    /// Maximum number of retry attempts
    pub max_retries: i32,
    
    /// Base retry delay in seconds
    pub base_retry_delay_seconds: u64,
    
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
    
    /// Maximum retry delay in seconds
    pub max_retry_delay_seconds: u64,
    
    /// Whether to enable dead letter queue
    pub enabled: bool,
    
    /// Retry processing interval in seconds
    pub processing_interval_seconds: u64,
}

impl Default for DeadLetterConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_retry_delay_seconds: 60,
            backoff_multiplier: 2.0,
            max_retry_delay_seconds: 3600, // 1 hour max
            enabled: true,
            processing_interval_seconds: 300, // 5 minutes
        }
    }
}

/// Dead letter queue entry status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeadLetterStatus {
    Failed,
    Retrying,
    MaxRetriesExceeded,
    Resolved,
}

impl From<String> for DeadLetterStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "failed" => DeadLetterStatus::Failed,
            "retrying" => DeadLetterStatus::Retrying,
            "max_retries_exceeded" => DeadLetterStatus::MaxRetriesExceeded,
            "resolved" => DeadLetterStatus::Resolved,
            _ => DeadLetterStatus::Failed,
        }
    }
}

impl From<DeadLetterStatus> for String {
    fn from(status: DeadLetterStatus) -> Self {
        match status {
            DeadLetterStatus::Failed => "failed".to_string(),
            DeadLetterStatus::Retrying => "retrying".to_string(),
            DeadLetterStatus::MaxRetriesExceeded => "max_retries_exceeded".to_string(),
            DeadLetterStatus::Resolved => "resolved".to_string(),
        }
    }
}

/// Dead letter queue entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterEntry {
    pub id: Uuid,
    pub original_event_id: Uuid,
    pub event_data: serde_json::Value,
    pub error_message: String,
    pub error_details: serde_json::Value,
    pub retry_count: i32,
    pub max_retries: i32,
    pub status: DeadLetterStatus,
    pub created_at: DateTime<Utc>,
    pub last_retry_at: Option<DateTime<Utc>>,
    pub next_retry_at: Option<DateTime<Utc>>,
}

/// Database model for dead letter queue entries
#[derive(Debug, Clone, Queryable, Insertable, Selectable)]
#[diesel(table_name = event_dead_letter_queue)]
struct DeadLetterRecord {
    pub id: Uuid,
    pub original_event_id: Uuid,
    pub event_data: serde_json::Value,
    pub error_message: String,
    pub error_details: serde_json::Value,
    pub retry_count: Option<i32>,
    pub max_retries: Option<i32>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub last_retry_at: Option<DateTime<Utc>>,
    pub next_retry_at: Option<DateTime<Utc>>,
}

impl From<DeadLetterRecord> for DeadLetterEntry {
    fn from(record: DeadLetterRecord) -> Self {
        Self {
            id: record.id,
            original_event_id: record.original_event_id,
            event_data: record.event_data,
            error_message: record.error_message,
            error_details: record.error_details,
            retry_count: record.retry_count.unwrap_or(0),
            max_retries: record.max_retries.unwrap_or(3),
            status: DeadLetterStatus::from(record.status),
            created_at: record.created_at,
            last_retry_at: record.last_retry_at,
            next_retry_at: record.next_retry_at,
        }
    }
}

impl From<DeadLetterEntry> for DeadLetterRecord {
    fn from(entry: DeadLetterEntry) -> Self {
        Self {
            id: entry.id,
            original_event_id: entry.original_event_id,
            event_data: entry.event_data,
            error_message: entry.error_message,
            error_details: entry.error_details,
            retry_count: Some(entry.retry_count),
            max_retries: Some(entry.max_retries),
            status: String::from(entry.status),
            created_at: entry.created_at,
            last_retry_at: entry.last_retry_at,
            next_retry_at: entry.next_retry_at,
        }
    }
}

/// Dead letter queue interface
#[async_trait]
pub trait DeadLetterQueue: Send + Sync {
    /// Add a failed event to the dead letter queue
    async fn add_failed_event(
        &self,
        event: &EventEnvelope,
        error_message: String,
        error_details: serde_json::Value,
    ) -> EventResult<()>;
    
    /// Get events ready for retry
    async fn get_retry_candidates(&self, limit: usize) -> EventResult<Vec<DeadLetterEntry>>;
    
    /// Mark an event as retrying
    async fn mark_retrying(&self, entry_id: Uuid) -> EventResult<()>;
    
    /// Mark an event as resolved (successfully processed)
    async fn mark_resolved(&self, entry_id: Uuid) -> EventResult<()>;
    
    /// Increment retry count and schedule next retry
    async fn increment_retry(&self, entry_id: Uuid, error_message: String) -> EventResult<()>;
    
    /// Mark an event as permanently failed (max retries exceeded)
    async fn mark_permanently_failed(&self, entry_id: Uuid) -> EventResult<()>;
    
    /// Get dead letter queue statistics
    async fn get_statistics(&self) -> EventResult<DeadLetterStatistics>;
    
    /// Purge old entries
    async fn purge_old_entries(&self, older_than: DateTime<Utc>) -> EventResult<usize>;
}

/// Dead letter queue statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterStatistics {
    pub total_entries: i64,
    pub failed_entries: i64,
    pub retrying_entries: i64,
    pub permanently_failed_entries: i64,
    pub resolved_entries: i64,
    pub oldest_entry: Option<DateTime<Utc>>,
    pub newest_entry: Option<DateTime<Utc>>,
    pub average_retry_count: f64,
}

/// PostgreSQL implementation of dead letter queue
pub struct PostgreSQLDeadLetterQueue {
    db_pool: Arc<Pool<ConnectionManager<PgConnection>>>,
    config: DeadLetterConfig,
}

impl PostgreSQLDeadLetterQueue {
    /// Create a new PostgreSQL dead letter queue
    pub fn new(
        db_pool: Arc<Pool<ConnectionManager<PgConnection>>>,
        config: DeadLetterConfig,
    ) -> Self {
        Self { db_pool, config }
    }
    
    /// Get database connection
    fn get_connection(&self) -> EventResult<diesel::r2d2::PooledConnection<ConnectionManager<PgConnection>>> {
        self.db_pool.get().map_err(|e| EventError::DatabaseError {
            message: format!("Failed to get database connection: {}", e),
        })
    }
    
    /// Calculate next retry time with exponential backoff
    fn calculate_next_retry_time(&self, retry_count: i32) -> DateTime<Utc> {
        let delay_seconds = (self.config.base_retry_delay_seconds as f64 
            * self.config.backoff_multiplier.powi(retry_count))
            .min(self.config.max_retry_delay_seconds as f64) as i64;
        
        Utc::now() + Duration::seconds(delay_seconds)
    }
}

#[async_trait]
impl DeadLetterQueue for PostgreSQLDeadLetterQueue {
    async fn add_failed_event(
        &self,
        event: &EventEnvelope,
        error_message: String,
        error_details: serde_json::Value,
    ) -> EventResult<()> {
        if !self.config.enabled {
            return Ok(());
        }
        
        let mut conn = self.get_connection()?;
        
        let entry = DeadLetterEntry {
            id: Uuid::new_v4(),
            original_event_id: event.event_id,
            event_data: event.event_data.clone(),
            error_message,
            error_details,
            retry_count: 0,
            max_retries: self.config.max_retries,
            status: DeadLetterStatus::Failed,
            created_at: Utc::now(),
            last_retry_at: None,
            next_retry_at: Some(self.calculate_next_retry_time(0)),
        };
        
        let record = DeadLetterRecord::from(entry);
        
        diesel::insert_into(event_dead_letter_queue::table)
            .values(&record)
            .execute(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to insert dead letter entry: {}", e),
            })?;
        
        tracing::warn!(
            "Event {} added to dead letter queue: {}",
            event.event_id,
            record.error_message
        );
        
        Ok(())
    }
    
    async fn get_retry_candidates(&self, limit: usize) -> EventResult<Vec<DeadLetterEntry>> {
        let mut conn = self.get_connection()?;
        
        let records: Vec<DeadLetterRecord> = event_dead_letter_queue::table
            .filter(event_dead_letter_queue::status.eq("failed"))
            .filter(event_dead_letter_queue::next_retry_at.le(Utc::now()))
            .filter(event_dead_letter_queue::retry_count.lt(self.config.max_retries))
            .order(event_dead_letter_queue::next_retry_at.asc())
            .limit(limit as i64)
            .load(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to get retry candidates: {}", e),
            })?;
        
        Ok(records.into_iter().map(DeadLetterEntry::from).collect())
    }
    
    async fn mark_retrying(&self, entry_id: Uuid) -> EventResult<()> {
        let mut conn = self.get_connection()?;
        
        diesel::update(event_dead_letter_queue::table)
            .filter(event_dead_letter_queue::id.eq(entry_id))
            .set((
                event_dead_letter_queue::status.eq("retrying"),
                event_dead_letter_queue::last_retry_at.eq(Some(Utc::now())),
            ))
            .execute(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to mark entry as retrying: {}", e),
            })?;
        
        Ok(())
    }
    
    async fn mark_resolved(&self, entry_id: Uuid) -> EventResult<()> {
        let mut conn = self.get_connection()?;
        
        diesel::update(event_dead_letter_queue::table)
            .filter(event_dead_letter_queue::id.eq(entry_id))
            .set(event_dead_letter_queue::status.eq("resolved"))
            .execute(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to mark entry as resolved: {}", e),
            })?;
        
        tracing::info!("Dead letter entry {} resolved", entry_id);
        Ok(())
    }
    
    async fn increment_retry(&self, entry_id: Uuid, error_message: String) -> EventResult<()> {
        let mut conn = self.get_connection()?;
        
        // First, get the current retry count
        let current_entry: DeadLetterRecord = event_dead_letter_queue::table
            .filter(event_dead_letter_queue::id.eq(entry_id))
            .first(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to get current entry: {}", e),
            })?;
        
        let new_retry_count = current_entry.retry_count.unwrap_or(0) + 1;
        let max_retries = current_entry.max_retries.unwrap_or(self.config.max_retries);
        
        if new_retry_count >= max_retries {
            // Mark as permanently failed
            diesel::update(event_dead_letter_queue::table)
                .filter(event_dead_letter_queue::id.eq(entry_id))
                .set((
                    event_dead_letter_queue::status.eq("max_retries_exceeded"),
                    event_dead_letter_queue::retry_count.eq(Some(new_retry_count)),
                    event_dead_letter_queue::error_message.eq(error_message.clone()),
                ))
                .execute(&mut conn)
                .map_err(|e| EventError::DatabaseError {
                    message: format!("Failed to mark as permanently failed: {}", e),
                })?;
            
            tracing::error!(
                "Dead letter entry {} exceeded max retries ({}/{}): {}",
                entry_id,
                new_retry_count,
                max_retries,
                error_message
            );
        } else {
            // Schedule next retry
            let next_retry_at = self.calculate_next_retry_time(new_retry_count);
            
            diesel::update(event_dead_letter_queue::table)
                .filter(event_dead_letter_queue::id.eq(entry_id))
                .set((
                    event_dead_letter_queue::status.eq("failed"),
                    event_dead_letter_queue::retry_count.eq(Some(new_retry_count)),
                    event_dead_letter_queue::error_message.eq(error_message.clone()),
                    event_dead_letter_queue::next_retry_at.eq(Some(next_retry_at)),
                ))
                .execute(&mut conn)
                .map_err(|e| EventError::DatabaseError {
                    message: format!("Failed to increment retry: {}", e),
                })?;
            
            tracing::warn!(
                "Dead letter entry {} retry {}/{} scheduled for {}: {}",
                entry_id,
                new_retry_count,
                max_retries,
                next_retry_at.format("%Y-%m-%d %H:%M:%S UTC"),
                error_message
            );
        }
        
        Ok(())
    }
    
    async fn mark_permanently_failed(&self, entry_id: Uuid) -> EventResult<()> {
        let mut conn = self.get_connection()?;
        
        diesel::update(event_dead_letter_queue::table)
            .filter(event_dead_letter_queue::id.eq(entry_id))
            .set(event_dead_letter_queue::status.eq("max_retries_exceeded"))
            .execute(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to mark as permanently failed: {}", e),
            })?;
        
        tracing::error!("Dead letter entry {} marked as permanently failed", entry_id);
        Ok(())
    }
    
    async fn get_statistics(&self) -> EventResult<DeadLetterStatistics> {
        let mut conn = self.get_connection()?;
        
        // Get total count
        let total_entries: i64 = event_dead_letter_queue::table
            .count()
            .get_result(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to get total count: {}", e),
            })?;
        
        // Get counts by status
        let failed_entries: i64 = event_dead_letter_queue::table
            .filter(event_dead_letter_queue::status.eq("failed"))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);
        
        let retrying_entries: i64 = event_dead_letter_queue::table
            .filter(event_dead_letter_queue::status.eq("retrying"))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);
        
        let permanently_failed_entries: i64 = event_dead_letter_queue::table
            .filter(event_dead_letter_queue::status.eq("max_retries_exceeded"))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);
        
        let resolved_entries: i64 = event_dead_letter_queue::table
            .filter(event_dead_letter_queue::status.eq("resolved"))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);
        
        // Get oldest and newest entries
        let oldest_entry: Option<DateTime<Utc>> = event_dead_letter_queue::table
            .select(diesel::dsl::min(event_dead_letter_queue::created_at))
            .first(&mut conn)
            .unwrap_or(None);
        
        let newest_entry: Option<DateTime<Utc>> = event_dead_letter_queue::table
            .select(diesel::dsl::max(event_dead_letter_queue::created_at))
            .first(&mut conn)
            .unwrap_or(None);
        
        // Calculate average retry count - use a simpler approach to avoid type issues
        let retry_counts: Vec<Option<i32>> = event_dead_letter_queue::table
            .select(event_dead_letter_queue::retry_count)
            .load(&mut conn)
            .unwrap_or_default();
        
        let avg_retry_count = if retry_counts.is_empty() {
            0.0
        } else {
            let sum: i32 = retry_counts.iter().filter_map(|&x| x).sum();
            let count = retry_counts.iter().filter(|x| x.is_some()).count();
            if count > 0 {
                sum as f64 / count as f64
            } else {
                0.0
            }
        };
        
        Ok(DeadLetterStatistics {
            total_entries,
            failed_entries,
            retrying_entries,
            permanently_failed_entries,
            resolved_entries,
            oldest_entry,
            newest_entry,
            average_retry_count: avg_retry_count,
        })
    }
    
    async fn purge_old_entries(&self, older_than: DateTime<Utc>) -> EventResult<usize> {
        let mut conn = self.get_connection()?;
        
        let deleted_count = diesel::delete(event_dead_letter_queue::table)
            .filter(event_dead_letter_queue::created_at.lt(older_than))
            .filter(event_dead_letter_queue::status.eq_any(vec!["resolved", "max_retries_exceeded"]))
            .execute(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to purge old entries: {}", e),
            })?;
        
        tracing::info!(
            "Purged {} old dead letter entries older than {}",
            deleted_count,
            older_than.format("%Y-%m-%d %H:%M:%S UTC")
        );
        
        Ok(deleted_count)
    }
}

/// Dead letter queue processor for handling retry logic
pub struct DeadLetterProcessor {
    queue: Arc<dyn DeadLetterQueue>,
    config: DeadLetterConfig,
    is_running: Arc<tokio::sync::RwLock<bool>>,
}

impl DeadLetterProcessor {
    /// Create a new dead letter processor
    pub fn new(queue: Arc<dyn DeadLetterQueue>, config: DeadLetterConfig) -> Self {
        Self {
            queue,
            config,
            is_running: Arc::new(tokio::sync::RwLock::new(false)),
        }
    }
    
    /// Start the background processing loop
    pub async fn start(&self) -> EventResult<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Ok(()); // Already running
        }
        *is_running = true;
        drop(is_running);
        
        let queue = Arc::clone(&self.queue);
        let processing_interval = self.config.processing_interval_seconds;
        let is_running = Arc::clone(&self.is_running);
        
        tokio::spawn(async move {
            let mut interval = interval(tokio::time::Duration::from_secs(processing_interval));
            
            loop {
                interval.tick().await;
                
                // Check if we should continue running
                let running = *is_running.read().await;
                if !running {
                    break;
                }
                
                // Process retry candidates
                match queue.get_retry_candidates(50).await {
                    Ok(candidates) => {
                        for entry in candidates {
                            // Mark as retrying
                            if let Err(e) = queue.mark_retrying(entry.id).await {
                                tracing::error!("Failed to mark entry as retrying: {}", e);
                                continue;
                            }
                            
                            // Here you would re-process the event
                            // For now, we'll just simulate processing
                            let success = Self::simulate_event_processing(&entry).await;
                            
                            if success {
                                if let Err(e) = queue.mark_resolved(entry.id).await {
                                    tracing::error!("Failed to mark entry as resolved: {}", e);
                                }
                            } else {
                                if let Err(e) = queue.increment_retry(
                                    entry.id,
                                    "Retry failed".to_string(),
                                ).await {
                                    tracing::error!("Failed to increment retry: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to get retry candidates: {}", e);
                    }
                }
            }
            
            tracing::info!("Dead letter processor stopped");
        });
        
        tracing::info!("Dead letter processor started");
        Ok(())
    }
    
    /// Stop the background processing
    pub async fn stop(&self) {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        tracing::info!("Dead letter processor stopping");
    }
    
    /// Simulate event processing (placeholder)
    async fn simulate_event_processing(_entry: &DeadLetterEntry) -> bool {
        // Simulate processing time
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // Simulate success/failure (70% success rate)
        rand::random::<f64>() < 0.7
    }
    
    /// Get processor statistics
    pub async fn get_statistics(&self) -> EventResult<DeadLetterStatistics> {
        self.queue.get_statistics().await
    }
    
    /// Manually trigger retry processing
    pub async fn process_retries(&self) -> EventResult<usize> {
        let candidates = self.queue.get_retry_candidates(100).await?;
        let count = candidates.len();
        
        for entry in candidates {
            // Mark as retrying
            self.queue.mark_retrying(entry.id).await?;
            
            // Process the event
            let success = Self::simulate_event_processing(&entry).await;
            
            if success {
                self.queue.mark_resolved(entry.id).await?;
            } else {
                self.queue.increment_retry(entry.id, "Manual retry failed".to_string()).await?;
            }
        }
        
        Ok(count)
    }
}