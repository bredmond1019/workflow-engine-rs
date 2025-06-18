// File: src/db/migration.rs
//
// Database migration system for the AI Workflow Orchestration System
// Provides versioned migrations with rollback support and validation

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use uuid::Uuid;

use crate::db::events::{EventError, EventResult};

/// Migration metadata and tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Migration {
    pub version: String,
    pub name: String,
    pub content: String,
    pub checksum: String,
    pub applied_at: Option<DateTime<Utc>>,
    pub execution_time_ms: Option<i32>,
}

impl Migration {
    pub fn new(version: String, name: String, content: String) -> Self {
        let checksum = Self::calculate_checksum(&content);
        Self {
            version,
            name,
            content,
            checksum,
            applied_at: None,
            execution_time_ms: None,
        }
    }

    fn calculate_checksum(content: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

/// Migration status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MigrationStatus {
    Pending,
    Applied,
    Failed,
    ChecksumMismatch,
}

/// Migration result
#[derive(Debug, Clone)]
pub struct MigrationResult {
    pub migration: Migration,
    pub status: MigrationStatus,
    pub execution_time_ms: i32,
    pub error: Option<String>,
}

/// Database migration manager
#[async_trait]
pub trait MigrationManager: Send + Sync {
    /// Load migrations from directory
    async fn load_migrations(&self, directory: &Path) -> EventResult<Vec<Migration>>;
    
    /// Get applied migrations from database
    async fn get_applied_migrations(&self) -> EventResult<Vec<Migration>>;
    
    /// Check if migration is applied
    async fn is_migration_applied(&self, version: &str) -> EventResult<bool>;
    
    /// Apply a single migration
    async fn apply_migration(&self, migration: &Migration) -> EventResult<MigrationResult>;
    
    /// Apply all pending migrations
    async fn apply_pending_migrations(&self, directory: &Path) -> EventResult<Vec<MigrationResult>>;
    
    /// Rollback migration (if supported)
    async fn rollback_migration(&self, version: &str) -> EventResult<MigrationResult>;
    
    /// Validate migration checksums
    async fn validate_migration_checksums(&self, directory: &Path) -> EventResult<Vec<MigrationResult>>;
    
    /// Get migration status
    async fn get_migration_status(&self, directory: &Path) -> EventResult<HashMap<String, MigrationStatus>>;
}

/// PostgreSQL migration manager implementation
pub struct PostgreSQLMigrationManager {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl PostgreSQLMigrationManager {
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        Self { pool }
    }

    fn get_connection(&self) -> EventResult<diesel::r2d2::PooledConnection<ConnectionManager<PgConnection>>> {
        self.pool.get().map_err(|e| EventError::DatabaseError {
            message: format!("Failed to get database connection: {}", e),
        })
    }

    /// Parse migration filename to extract version and name
    pub fn parse_migration_filename(filename: &str) -> Option<(String, String)> {
        if !filename.ends_with(".sql") {
            return None;
        }

        let name_without_ext = &filename[..filename.len() - 4];
        
        // Expected format: YYYYMMDD_HHMMSS_migration_name.sql
        // or: 20241213_000001_create_event_store.sql
        let parts: Vec<&str> = name_without_ext.splitn(3, '_').collect();
        if parts.len() >= 3 {
            let version = format!("{}_{}", parts[0], parts[1]);
            let name = parts[2..].join("_");
            Some((version, name))
        } else {
            None
        }
    }

    /// Ensure migration tracking tables exist
    async fn ensure_migration_tables(&self) -> EventResult<()> {
        let mut conn = self.get_connection()?;
        
        let schema_sql = r#"
            CREATE TABLE IF NOT EXISTS schema_migrations (
                id SERIAL PRIMARY KEY,
                version VARCHAR(255) NOT NULL UNIQUE,
                name VARCHAR(255) NOT NULL,
                applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                checksum VARCHAR(64),
                execution_time_ms INTEGER
            );
            
            CREATE INDEX IF NOT EXISTS idx_schema_migrations_version ON schema_migrations(version);
        "#;

        diesel::sql_query(schema_sql)
            .execute(&mut conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to create migration tables: {}", e),
            })?;

        Ok(())
    }
}

#[async_trait]
impl MigrationManager for PostgreSQLMigrationManager {
    async fn load_migrations(&self, directory: &Path) -> EventResult<Vec<Migration>> {
        let mut migrations = Vec::new();

        if !directory.exists() {
            return Err(EventError::ConfigurationError {
                message: format!("Migration directory does not exist: {}", directory.display()),
            });
        }

        let entries = fs::read_dir(directory).map_err(|e| EventError::ConfigurationError {
            message: format!("Failed to read migration directory: {}", e),
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| EventError::ConfigurationError {
                message: format!("Failed to read directory entry: {}", e),
            })?;

            let path = entry.path();
            if !path.is_file() || !path.extension().map_or(false, |ext| ext == "sql") {
                continue;
            }

            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| EventError::ConfigurationError {
                    message: format!("Invalid filename: {}", path.display()),
                })?;

            if let Some((version, name)) = Self::parse_migration_filename(filename) {
                let content = fs::read_to_string(&path).map_err(|e| EventError::ConfigurationError {
                    message: format!("Failed to read migration file {}: {}", path.display(), e),
                })?;

                migrations.push(Migration::new(version, name, content));
            }
        }

        // Sort migrations by version
        migrations.sort_by(|a, b| a.version.cmp(&b.version));

        Ok(migrations)
    }

    async fn get_applied_migrations(&self) -> EventResult<Vec<Migration>> {
        self.ensure_migration_tables().await?;
        
        let mut conn = self.get_connection()?;

        #[derive(Queryable, Debug)]
        #[derive(diesel::QueryableByName)]
        struct AppliedMigration {
            #[diesel(sql_type = diesel::sql_types::Text)]
            version: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            name: String,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            applied_at: DateTime<Utc>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            checksum: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Integer>)]
            execution_time_ms: Option<i32>,
        }

        let applied: Vec<AppliedMigration> = diesel::sql_query(
            "SELECT version, name, applied_at, checksum, execution_time_ms 
             FROM schema_migrations 
             ORDER BY version"
        )
        .load(&mut conn)
        .map_err(|e| EventError::DatabaseError {
            message: format!("Failed to load applied migrations: {}", e),
        })?;

        let migrations = applied
            .into_iter()
            .map(|a| Migration {
                version: a.version,
                name: a.name,
                content: String::new(), // Content not stored in DB
                checksum: a.checksum.unwrap_or_default(),
                applied_at: Some(a.applied_at),
                execution_time_ms: a.execution_time_ms,
            })
            .collect();

        Ok(migrations)
    }

    async fn is_migration_applied(&self, version: &str) -> EventResult<bool> {
        self.ensure_migration_tables().await?;
        
        let mut conn = self.get_connection()?;

        #[derive(diesel::QueryableByName)]
        struct CountResult {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            count: i64,
        }

        let result: CountResult = diesel::sql_query(
            "SELECT COUNT(*) as count FROM schema_migrations WHERE version = $1"
        )
        .bind::<diesel::sql_types::Text, _>(version)
        .get_result(&mut conn)
        .map_err(|e| EventError::DatabaseError {
            message: format!("Failed to check migration status: {}", e),
        })?;

        let count = result.count;

        Ok(count > 0)
    }

    async fn apply_migration(&self, migration: &Migration) -> EventResult<MigrationResult> {
        self.ensure_migration_tables().await?;
        
        let start_time = Instant::now();
        let mut conn = self.get_connection()?;

        // Check if already applied
        if self.is_migration_applied(&migration.version).await? {
            return Ok(MigrationResult {
                migration: migration.clone(),
                status: MigrationStatus::Applied,
                execution_time_ms: 0,
                error: Some("Migration already applied".to_string()),
            });
        }

        // Execute migration in transaction
        let result = conn.transaction::<_, EventError, _>(|conn| {
            // Execute migration SQL
            diesel::sql_query(&migration.content)
                .execute(conn)
                .map_err(|e| EventError::DatabaseError {
                    message: format!("Migration execution failed: {}", e),
                })?;

            let execution_time = start_time.elapsed().as_millis() as i32;

            // Record migration
            diesel::sql_query(
                "INSERT INTO schema_migrations (version, name, checksum, execution_time_ms) 
                 VALUES ($1, $2, $3, $4)"
            )
            .bind::<diesel::sql_types::Text, _>(&migration.version)
            .bind::<diesel::sql_types::Text, _>(&migration.name)
            .bind::<diesel::sql_types::Text, _>(&migration.checksum)
            .bind::<diesel::sql_types::Integer, _>(execution_time)
            .execute(conn)
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to record migration: {}", e),
            })?;

            Ok(execution_time)
        });

        match result {
            Ok(execution_time) => Ok(MigrationResult {
                migration: migration.clone(),
                status: MigrationStatus::Applied,
                execution_time_ms: execution_time,
                error: None,
            }),
            Err(e) => Ok(MigrationResult {
                migration: migration.clone(),
                status: MigrationStatus::Failed,
                execution_time_ms: start_time.elapsed().as_millis() as i32,
                error: Some(e.to_string()),
            }),
        }
    }

    async fn apply_pending_migrations(&self, directory: &Path) -> EventResult<Vec<MigrationResult>> {
        let all_migrations = self.load_migrations(directory).await?;
        let applied_migrations = self.get_applied_migrations().await?;
        
        let applied_versions: std::collections::HashSet<String> = 
            applied_migrations.iter().map(|m| m.version.clone()).collect();

        let mut results = Vec::new();

        for migration in all_migrations {
            if !applied_versions.contains(&migration.version) {
                let result = self.apply_migration(&migration).await?;
                results.push(result);
            }
        }

        Ok(results)
    }

    async fn rollback_migration(&self, _version: &str) -> EventResult<MigrationResult> {
        // Rollback not implemented for PostgreSQL migrations
        // This would require storing rollback SQL
        Err(EventError::ConfigurationError {
            message: "Rollback not supported for PostgreSQL migrations".to_string(),
        })
    }

    async fn validate_migration_checksums(&self, directory: &Path) -> EventResult<Vec<MigrationResult>> {
        let file_migrations = self.load_migrations(directory).await?;
        let applied_migrations = self.get_applied_migrations().await?;

        let applied_map: HashMap<String, Migration> = 
            applied_migrations.into_iter().map(|m| (m.version.clone(), m)).collect();

        let mut results = Vec::new();

        for file_migration in file_migrations {
            if let Some(applied_migration) = applied_map.get(&file_migration.version) {
                let status = if file_migration.checksum == applied_migration.checksum {
                    MigrationStatus::Applied
                } else {
                    MigrationStatus::ChecksumMismatch
                };

                results.push(MigrationResult {
                    migration: file_migration,
                    status,
                    execution_time_ms: applied_migration.execution_time_ms.unwrap_or(0),
                    error: if status == MigrationStatus::ChecksumMismatch {
                        Some("Checksum mismatch - migration file may have been modified".to_string())
                    } else {
                        None
                    },
                });
            } else {
                results.push(MigrationResult {
                    migration: file_migration,
                    status: MigrationStatus::Pending,
                    execution_time_ms: 0,
                    error: None,
                });
            }
        }

        Ok(results)
    }

    async fn get_migration_status(&self, directory: &Path) -> EventResult<HashMap<String, MigrationStatus>> {
        let validation_results = self.validate_migration_checksums(directory).await?;
        
        let status_map = validation_results
            .into_iter()
            .map(|result| (result.migration.version, result.status))
            .collect();

        Ok(status_map)
    }
}

/// Configuration for migration manager
#[derive(Debug, Clone)]
pub struct MigrationConfig {
    pub migration_directory: PathBuf,
    pub auto_apply: bool,
    pub validate_checksums: bool,
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            migration_directory: PathBuf::from("migrations"),
            auto_apply: false,
            validate_checksums: true,
        }
    }
}

/// Migration service for application startup
pub struct MigrationService {
    manager: Box<dyn MigrationManager>,
    config: MigrationConfig,
}

impl MigrationService {
    pub fn new(manager: Box<dyn MigrationManager>, config: MigrationConfig) -> Self {
        Self { manager, config }
    }

    /// Initialize migrations on application startup
    pub async fn initialize(&self) -> EventResult<()> {
        if self.config.validate_checksums {
            let validation_results = self.manager
                .validate_migration_checksums(&self.config.migration_directory)
                .await?;

            for result in &validation_results {
                if result.status == MigrationStatus::ChecksumMismatch {
                    return Err(EventError::ConfigurationError {
                        message: format!(
                            "Migration checksum mismatch for version {}: {}",
                            result.migration.version,
                            result.error.as_ref().unwrap_or(&"Unknown error".to_string())
                        ),
                    });
                }
            }
        }

        if self.config.auto_apply {
            let results = self.manager
                .apply_pending_migrations(&self.config.migration_directory)
                .await?;

            for result in results {
                match result.status {
                    MigrationStatus::Applied => {
                        tracing::info!(
                            "Applied migration {} '{}' in {}ms",
                            result.migration.version,
                            result.migration.name,
                            result.execution_time_ms
                        );
                    }
                    MigrationStatus::Failed => {
                        return Err(EventError::DatabaseError {
                            message: format!(
                                "Migration {} failed: {}",
                                result.migration.version,
                                result.error.unwrap_or("Unknown error".to_string())
                            ),
                        });
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    /// Get current migration status
    pub async fn status(&self) -> EventResult<HashMap<String, MigrationStatus>> {
        self.manager
            .get_migration_status(&self.config.migration_directory)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_parse_migration_filename() {
        assert_eq!(
            PostgreSQLMigrationManager::parse_migration_filename("20241213_000001_create_event_store.sql"),
            Some(("20241213_000001".to_string(), "create_event_store".to_string()))
        );

        assert_eq!(
            PostgreSQLMigrationManager::parse_migration_filename("20241213_000002_add_indexes_and_partitioning.sql"),
            Some(("20241213_000002".to_string(), "add_indexes_and_partitioning".to_string()))
        );

        assert_eq!(
            PostgreSQLMigrationManager::parse_migration_filename("invalid.txt"),
            None
        );

        assert_eq!(
            PostgreSQLMigrationManager::parse_migration_filename("20241213_invalid_format"),
            None
        );
    }

    #[test]
    fn test_migration_checksum() {
        let migration = Migration::new(
            "test_version".to_string(),
            "test_migration".to_string(),
            "CREATE TABLE test (id INT);".to_string()
        );

        assert!(!migration.checksum.is_empty());
        assert_eq!(migration.checksum.len(), 64); // SHA256 hex string length
    }
}