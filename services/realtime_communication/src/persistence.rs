//! Message Persistence Layer
//! 
//! PostgreSQL-backed message storage with delivery tracking, history retrieval,
//! and archival capabilities for the real-time communication system.

use sqlx::{PgPool, Row};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tracing::{info, warn, error, debug};
use serde_json;

use crate::actors::messages::{DeliveryStatus, PersistedMessage, PersistMessage, GetMessageHistory};

/// Message persistence manager
pub struct MessagePersistence {
    pool: PgPool,
    config: PersistenceConfig,
}

/// Persistence configuration
#[derive(Debug, Clone)]
pub struct PersistenceConfig {
    pub retention_days: u32,
    pub archive_threshold_days: u32,
    pub batch_size: usize,
    pub enable_full_text_search: bool,
    pub max_message_size_bytes: usize,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            retention_days: 365,      // Keep messages for 1 year
            archive_threshold_days: 90, // Archive after 90 days
            batch_size: 1000,
            enable_full_text_search: true,
            max_message_size_bytes: 1024 * 1024, // 1MB
        }
    }
}

/// Message statistics
#[derive(Debug, Clone)]
pub struct MessageStats {
    pub total_messages: i64,
    pub messages_today: i64,
    pub messages_this_week: i64,
    pub average_message_size: f64,
    pub delivery_success_rate: f64,
    pub storage_size_bytes: i64,
}

impl MessagePersistence {
    /// Create new message persistence manager
    pub fn new(pool: PgPool, config: PersistenceConfig) -> Self {
        Self { pool, config }
    }

    /// Initialize database schema
    pub async fn initialize_schema(&self) -> Result<(), sqlx::Error> {
        info!("Initializing message persistence schema");

        // Create messages table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS messages (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                message_id VARCHAR(255) UNIQUE NOT NULL,
                from_user VARCHAR(255),
                to_user VARCHAR(255),
                topic VARCHAR(255),
                conversation_id VARCHAR(255),
                content JSONB NOT NULL,
                message_type VARCHAR(100) NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                delivery_status VARCHAR(50) NOT NULL DEFAULT 'sent',
                metadata JSONB DEFAULT '{}',
                size_bytes INTEGER NOT NULL DEFAULT 0,
                search_vector tsvector,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
        "#)
        .execute(&self.pool)
        .await?;

        // Create delivery tracking table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS message_deliveries (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                message_id VARCHAR(255) NOT NULL,
                connection_id UUID NOT NULL,
                user_id VARCHAR(255),
                delivery_status VARCHAR(50) NOT NULL,
                attempted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                delivered_at TIMESTAMPTZ,
                failed_reason TEXT,
                retry_count INTEGER NOT NULL DEFAULT 0,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
        "#)
        .execute(&self.pool)
        .await?;

        // Create conversation tracking table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS conversations (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                conversation_id VARCHAR(255) UNIQUE NOT NULL,
                participants TEXT[] NOT NULL,
                conversation_type VARCHAR(50) NOT NULL DEFAULT 'direct',
                created_by VARCHAR(255),
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                metadata JSONB DEFAULT '{}'
            )
        "#)
        .execute(&self.pool)
        .await?;

        // Create indexes for performance
        self.create_indexes().await?;

        // Create search configuration if enabled
        if self.config.enable_full_text_search {
            self.setup_full_text_search().await?;
        }

        info!("Message persistence schema initialized successfully");
        Ok(())
    }

    /// Create database indexes
    async fn create_indexes(&self) -> Result<(), sqlx::Error> {
        let indexes = vec![
            "CREATE INDEX IF NOT EXISTS idx_messages_timestamp ON messages(timestamp)",
            "CREATE INDEX IF NOT EXISTS idx_messages_from_user ON messages(from_user)",
            "CREATE INDEX IF NOT EXISTS idx_messages_to_user ON messages(to_user)",
            "CREATE INDEX IF NOT EXISTS idx_messages_topic ON messages(topic)",
            "CREATE INDEX IF NOT EXISTS idx_messages_conversation ON messages(conversation_id)",
            "CREATE INDEX IF NOT EXISTS idx_messages_type ON messages(message_type)",
            "CREATE INDEX IF NOT EXISTS idx_messages_delivery_status ON messages(delivery_status)",
            "CREATE INDEX IF NOT EXISTS idx_deliveries_message_id ON message_deliveries(message_id)",
            "CREATE INDEX IF NOT EXISTS idx_deliveries_connection_id ON message_deliveries(connection_id)",
            "CREATE INDEX IF NOT EXISTS idx_deliveries_user_id ON message_deliveries(user_id)",
            "CREATE INDEX IF NOT EXISTS idx_deliveries_status ON message_deliveries(delivery_status)",
            "CREATE INDEX IF NOT EXISTS idx_conversations_participants ON conversations USING GIN(participants)",
        ];

        for index_sql in indexes {
            sqlx::query(index_sql).execute(&self.pool).await?;
        }

        debug!("Database indexes created");
        Ok(())
    }

    /// Setup full-text search
    async fn setup_full_text_search(&self) -> Result<(), sqlx::Error> {
        // Create search vector index
        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_messages_search 
            ON messages USING GIN(search_vector)
        "#)
        .execute(&self.pool)
        .await?;

        // Create trigger to update search vector
        sqlx::query(r#"
            CREATE OR REPLACE FUNCTION update_message_search_vector()
            RETURNS TRIGGER AS $$
            BEGIN
                NEW.search_vector := to_tsvector('english', 
                    COALESCE(NEW.content->>'text', '') || ' ' ||
                    COALESCE(NEW.topic, '') || ' ' ||
                    COALESCE(NEW.from_user, '') || ' ' ||
                    COALESCE(NEW.to_user, '')
                );
                RETURN NEW;
            END;
            $$ LANGUAGE plpgsql;
        "#)
        .execute(&self.pool)
        .await?;

        sqlx::query(r#"
            DROP TRIGGER IF EXISTS trigger_update_search_vector ON messages;
            CREATE TRIGGER trigger_update_search_vector
                BEFORE INSERT OR UPDATE ON messages
                FOR EACH ROW EXECUTE FUNCTION update_message_search_vector();
        "#)
        .execute(&self.pool)
        .await?;

        debug!("Full-text search setup completed");
        Ok(())
    }

    /// Persist a message
    pub async fn persist_message(&self, message: PersistMessage) -> Result<String, String> {
        // Validate message size
        let content_size = serde_json::to_string(&message.content)
            .map_err(|e| format!("Content serialization failed: {}", e))?
            .len();

        if content_size > self.config.max_message_size_bytes {
            return Err(format!("Message size {} exceeds limit {}", 
                              content_size, self.config.max_message_size_bytes));
        }

        // Insert message
        let result = sqlx::query(r#"
            INSERT INTO messages (
                message_id, from_user, to_user, topic, content, 
                message_type, timestamp, delivery_status, metadata, size_bytes
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id
        "#)
        .bind(&message.message_id)
        .bind(&message.from_user)
        .bind(&message.to_user)
        .bind(&message.topic)
        .bind(&message.content)
        .bind(&message.message_type)
        .bind(message.timestamp)
        .bind(delivery_status_to_string(&message.delivery_status))
        .bind(serde_json::to_value(&message.metadata).unwrap_or(serde_json::Value::Object(serde_json::Map::new())))
        .bind(content_size as i32)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database insert failed: {}", e))?;

        let id: Uuid = result.get("id");

        // Create conversation if needed
        if let (Some(from_user), Some(to_user)) = (&message.from_user, &message.to_user) {
            self.ensure_conversation_exists(from_user, to_user).await?;
        }

        info!("Message persisted: id={}, message_id={}", id, message.message_id);
        Ok(id.to_string())
    }

    /// Record message delivery
    pub async fn record_delivery(
        &self,
        message_id: &str,
        connection_id: Uuid,
        user_id: Option<&str>,
        status: DeliveryStatus,
        failed_reason: Option<&str>,
    ) -> Result<(), String> {
        sqlx::query(r#"
            INSERT INTO message_deliveries (
                message_id, connection_id, user_id, delivery_status, 
                delivered_at, failed_reason, retry_count
            ) VALUES ($1, $2, $3, $4, $5, $6, 0)
        "#)
        .bind(message_id)
        .bind(connection_id)
        .bind(user_id)
        .bind(delivery_status_to_string(&status))
        .bind(match status {
            DeliveryStatus::Delivered | DeliveryStatus::Read => Some(Utc::now()),
            _ => None,
        })
        .bind(failed_reason)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Delivery recording failed: {}", e))?;

        debug!("Delivery recorded: message_id={}, status={:?}", message_id, status);
        Ok(())
    }

    /// Get message history
    pub async fn get_message_history(
        &self,
        request: GetMessageHistory,
    ) -> Result<Vec<PersistedMessage>, String> {
        let limit = request.limit.unwrap_or(50).min(1000); // Cap at 1000
        let offset = request.offset.unwrap_or(0);

        let query = if let Some(before_timestamp) = request.before_timestamp {
            sqlx::query(r#"
                SELECT message_id, from_user, to_user, topic, content, 
                       message_type, timestamp, delivery_status, metadata
                FROM messages 
                WHERE (conversation_id = $1 OR 
                       (from_user = $2 AND to_user = $3) OR 
                       (from_user = $3 AND to_user = $2))
                  AND timestamp < $4
                ORDER BY timestamp DESC 
                LIMIT $5 OFFSET $6
            "#)
            .bind(&request.conversation_id)
            .bind(&request.user_id)
            .bind(&request.conversation_id) // Assuming conversation_id might be the other user
            .bind(before_timestamp)
            .bind(limit as i64)
            .bind(offset as i64)
        } else {
            sqlx::query(r#"
                SELECT message_id, from_user, to_user, topic, content, 
                       message_type, timestamp, delivery_status, metadata
                FROM messages 
                WHERE (conversation_id = $1 OR 
                       (from_user = $2 AND to_user = $3) OR 
                       (from_user = $3 AND to_user = $2))
                ORDER BY timestamp DESC 
                LIMIT $4 OFFSET $5
            "#)
            .bind(&request.conversation_id)
            .bind(&request.user_id)
            .bind(&request.conversation_id) // Assuming conversation_id might be the other user
            .bind(limit as i64)
            .bind(offset as i64)
        };

        let rows = query.fetch_all(&self.pool).await
            .map_err(|e| format!("Message history query failed: {}", e))?;

        let mut messages = Vec::new();
        for row in rows {
            let delivery_status = string_to_delivery_status(
                &row.get::<String, _>("delivery_status")
            );

            let message = PersistedMessage {
                id: row.get("message_id"),
                from_user: row.get("from_user"),
                to_user: row.get("to_user"),
                topic: row.get("topic"),
                content: row.get("content"),
                message_type: row.get("message_type"),
                timestamp: row.get("timestamp"),
                delivery_status,
                metadata: {
                    let json_value: serde_json::Value = row.get("metadata");
                    serde_json::from_value(json_value).unwrap_or_default()
                },
            };
            messages.push(message);
        }

        debug!("Retrieved {} messages for conversation {}", messages.len(), request.conversation_id);
        Ok(messages)
    }

    /// Search messages by content
    pub async fn search_messages(
        &self,
        query: &str,
        user_id: &str,
        limit: Option<usize>,
    ) -> Result<Vec<PersistedMessage>, String> {
        if !self.config.enable_full_text_search {
            return Err("Full-text search is not enabled".to_string());
        }

        let limit = limit.unwrap_or(50).min(1000);

        let rows = sqlx::query(r#"
            SELECT message_id, from_user, to_user, topic, content, 
                   message_type, timestamp, delivery_status, metadata,
                   ts_rank(search_vector, plainto_tsquery('english', $1)) as rank
            FROM messages 
            WHERE (from_user = $2 OR to_user = $2)
              AND search_vector @@ plainto_tsquery('english', $1)
            ORDER BY rank DESC, timestamp DESC
            LIMIT $3
        "#)
        .bind(query)
        .bind(user_id)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Message search failed: {}", e))?;

        let mut messages = Vec::new();
        for row in rows {
            let delivery_status = string_to_delivery_status(
                &row.get::<String, _>("delivery_status")
            );

            let message = PersistedMessage {
                id: row.get("message_id"),
                from_user: row.get("from_user"),
                to_user: row.get("to_user"),
                topic: row.get("topic"),
                content: row.get("content"),
                message_type: row.get("message_type"),
                timestamp: row.get("timestamp"),
                delivery_status,
                metadata: {
                    let json_value: serde_json::Value = row.get("metadata");
                    serde_json::from_value(json_value).unwrap_or_default()
                },
            };
            messages.push(message);
        }

        info!("Search returned {} messages for query: '{}'", messages.len(), query);
        Ok(messages)
    }

    /// Get message statistics
    pub async fn get_message_stats(&self) -> Result<MessageStats, String> {
        let stats_row = sqlx::query(r#"
            SELECT 
                COUNT(*) as total_messages,
                COUNT(CASE WHEN timestamp >= CURRENT_DATE THEN 1 END) as messages_today,
                COUNT(CASE WHEN timestamp >= CURRENT_DATE - INTERVAL '7 days' THEN 1 END) as messages_this_week,
                AVG(size_bytes) as average_message_size,
                COUNT(CASE WHEN delivery_status = 'delivered' THEN 1 END)::float / COUNT(*)::float * 100 as delivery_success_rate,
                SUM(size_bytes) as storage_size_bytes
            FROM messages
        "#)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Stats query failed: {}", e))?;

        let stats = MessageStats {
            total_messages: stats_row.get("total_messages"),
            messages_today: stats_row.get("messages_today"),
            messages_this_week: stats_row.get("messages_this_week"),
            average_message_size: stats_row.get::<Option<f64>, _>("average_message_size").unwrap_or(0.0),
            delivery_success_rate: stats_row.get::<Option<f64>, _>("delivery_success_rate").unwrap_or(0.0),
            storage_size_bytes: stats_row.get::<Option<i64>, _>("storage_size_bytes").unwrap_or(0),
        };

        Ok(stats)
    }

    /// Archive old messages
    pub async fn archive_messages(&self) -> Result<usize, String> {
        let archive_date = Utc::now() - chrono::Duration::days(self.config.archive_threshold_days as i64);

        // Move old messages to archive table (create if not exists)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS messages_archive (LIKE messages INCLUDING ALL)
        "#)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Archive table creation failed: {}", e))?;

        let result = sqlx::query(r#"
            WITH archived AS (
                INSERT INTO messages_archive 
                SELECT * FROM messages 
                WHERE timestamp < $1
                RETURNING id
            )
            DELETE FROM messages 
            WHERE timestamp < $1
        "#)
        .bind(archive_date)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Archive operation failed: {}", e))?;

        let archived_count = result.rows_affected() as usize;
        info!("Archived {} messages older than {}", archived_count, archive_date);
        Ok(archived_count)
    }

    /// Ensure conversation exists
    async fn ensure_conversation_exists(&self, user1: &str, user2: &str) -> Result<(), String> {
        let mut participants = vec![user1.to_string(), user2.to_string()];
        participants.sort(); // Ensure consistent ordering

        let conversation_id = format!("dm_{}_{}", participants[0], participants[1]);

        sqlx::query(r#"
            INSERT INTO conversations (conversation_id, participants, conversation_type, created_by)
            VALUES ($1, $2, 'direct', $3)
            ON CONFLICT (conversation_id) DO NOTHING
        "#)
        .bind(&conversation_id)
        .bind(&participants)
        .bind(user1)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Conversation creation failed: {}", e))?;

        Ok(())
    }

    /// Clean up old data
    pub async fn cleanup_old_data(&self) -> Result<usize, String> {
        let retention_date = Utc::now() - chrono::Duration::days(self.config.retention_days as i64);

        let result = sqlx::query(r#"
            DELETE FROM messages WHERE timestamp < $1
        "#)
        .bind(retention_date)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Cleanup failed: {}", e))?;

        let deleted_count = result.rows_affected() as usize;
        info!("Cleaned up {} messages older than retention period", deleted_count);
        Ok(deleted_count)
    }
}

/// Convert delivery status to string
fn delivery_status_to_string(status: &DeliveryStatus) -> String {
    match status {
        DeliveryStatus::Sent => "sent".to_string(),
        DeliveryStatus::Delivered => "delivered".to_string(),
        DeliveryStatus::Read => "read".to_string(),
        DeliveryStatus::Failed => "failed".to_string(),
    }
}

/// Convert string to delivery status
fn string_to_delivery_status(status: &str) -> DeliveryStatus {
    match status {
        "sent" => DeliveryStatus::Sent,
        "delivered" => DeliveryStatus::Delivered,
        "read" => DeliveryStatus::Read,
        "failed" => DeliveryStatus::Failed,
        _ => DeliveryStatus::Sent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delivery_status_conversion() {
        assert_eq!(delivery_status_to_string(&DeliveryStatus::Sent), "sent");
        assert_eq!(delivery_status_to_string(&DeliveryStatus::Delivered), "delivered");
        assert_eq!(delivery_status_to_string(&DeliveryStatus::Read), "read");
        assert_eq!(delivery_status_to_string(&DeliveryStatus::Failed), "failed");

        assert_eq!(string_to_delivery_status("sent"), DeliveryStatus::Sent);
        assert_eq!(string_to_delivery_status("delivered"), DeliveryStatus::Delivered);
        assert_eq!(string_to_delivery_status("read"), DeliveryStatus::Read);
        assert_eq!(string_to_delivery_status("failed"), DeliveryStatus::Failed);
        assert_eq!(string_to_delivery_status("unknown"), DeliveryStatus::Sent);
    }

    #[test]
    fn test_persistence_config_default() {
        let config = PersistenceConfig::default();
        assert_eq!(config.retention_days, 365);
        assert_eq!(config.archive_threshold_days, 90);
        assert_eq!(config.batch_size, 1000);
        assert!(config.enable_full_text_search);
        assert_eq!(config.max_message_size_bytes, 1024 * 1024);
    }
}