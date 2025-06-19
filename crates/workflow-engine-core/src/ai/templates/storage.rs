//! # Template Storage
//!
//! This module provides storage backends for template persistence,
//! including versioning support and multiple storage options.

use super::{
    Template, TemplateId, TemplateVersion, TemplateMetadata,
    TemplateError,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Storage backend trait for templates
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Save a template
    fn save(&self, template: &Template) -> Result<(), StorageError>;
    
    /// Get a template by ID and optional version
    fn get(&self, id: &str, version: Option<u32>) -> Result<Template, StorageError>;
    
    /// List all templates
    fn list(&self) -> Result<Vec<TemplateMetadata>, StorageError>;
    
    /// Get template history
    fn get_history(&self, id: &str) -> Result<Vec<TemplateVersion>, StorageError>;
    
    /// Delete a template
    fn delete(&self, id: &str) -> Result<(), StorageError>;
    
    /// Search templates by criteria
    fn search(&self, criteria: &SearchCriteria) -> Result<Vec<TemplateMetadata>, StorageError>;
}

/// Storage-specific errors
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Template not found: {id}")]
    NotFound { id: String },
    
    #[error("Version not found: {id} v{version}")]
    VersionNotFound { id: String, version: u32 },
    
    #[error("Storage backend error: {message}")]
    BackendError { message: String },
    
    #[error("Serialization error: {message}")]
    SerializationError { message: String },
    
    #[error("IO error: {message}")]
    IoError { message: String },
}

impl From<StorageError> for TemplateError {
    fn from(error: StorageError) -> Self {
        TemplateError::StorageError {
            message: error.to_string(),
        }
    }
}

/// Search criteria for templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchCriteria {
    pub name_contains: Option<String>,
    pub tags: Option<Vec<String>>,
    pub context: Option<String>,
    pub author: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

/// In-memory storage backend
pub struct MemoryStorage {
    templates: Arc<RwLock<HashMap<TemplateId, Vec<StoredTemplate>>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredTemplate {
    template: Template,
    version: TemplateVersion,
}

impl MemoryStorage {
    /// Create new memory storage
    pub fn new() -> Self {
        Self {
            templates: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl StorageBackend for MemoryStorage {
    fn save(&self, template: &Template) -> Result<(), StorageError> {
        let mut templates = self.templates.write()
            .map_err(|_| StorageError::BackendError {
                message: "Failed to acquire write lock".to_string(),
            })?;
        
        let stored = StoredTemplate {
            template: template.clone(),
            version: TemplateVersion {
                version: template.metadata.version,
                created_at: template.metadata.updated_at,
                created_by: template.metadata.author.clone(),
                comment: None,
                content_hash: calculate_hash(&template.content),
            },
        };
        
        templates.entry(template.id.clone())
            .or_insert_with(Vec::new)
            .push(stored);
        
        Ok(())
    }
    
    fn get(&self, id: &str, version: Option<u32>) -> Result<Template, StorageError> {
        let templates = self.templates.read()
            .map_err(|_| StorageError::BackendError {
                message: "Failed to acquire read lock".to_string(),
            })?;
        
        let template_id = TemplateId::from(id);
        let versions = templates.get(&template_id)
            .ok_or_else(|| StorageError::NotFound {
                id: id.to_string(),
            })?;
        
        if let Some(v) = version {
            versions.iter()
                .find(|t| t.version.version == v)
                .map(|t| t.template.clone())
                .ok_or_else(|| StorageError::VersionNotFound {
                    id: id.to_string(),
                    version: v,
                })
        } else {
            // Get latest version
            versions.last()
                .map(|t| t.template.clone())
                .ok_or_else(|| StorageError::NotFound {
                    id: id.to_string(),
                })
        }
    }
    
    fn list(&self) -> Result<Vec<TemplateMetadata>, StorageError> {
        let templates = self.templates.read()
            .map_err(|_| StorageError::BackendError {
                message: "Failed to acquire read lock".to_string(),
            })?;
        
        Ok(templates.values()
            .filter_map(|versions| versions.last())
            .map(|t| t.template.metadata.clone())
            .collect())
    }
    
    fn get_history(&self, id: &str) -> Result<Vec<TemplateVersion>, StorageError> {
        let templates = self.templates.read()
            .map_err(|_| StorageError::BackendError {
                message: "Failed to acquire read lock".to_string(),
            })?;
        
        let template_id = TemplateId::from(id);
        let versions = templates.get(&template_id)
            .ok_or_else(|| StorageError::NotFound {
                id: id.to_string(),
            })?;
        
        Ok(versions.iter()
            .map(|t| t.version.clone())
            .collect())
    }
    
    fn delete(&self, id: &str) -> Result<(), StorageError> {
        let mut templates = self.templates.write()
            .map_err(|_| StorageError::BackendError {
                message: "Failed to acquire write lock".to_string(),
            })?;
        
        let template_id = TemplateId::from(id);
        templates.remove(&template_id)
            .ok_or_else(|| StorageError::NotFound {
                id: id.to_string(),
            })?;
        
        Ok(())
    }
    
    fn search(&self, criteria: &SearchCriteria) -> Result<Vec<TemplateMetadata>, StorageError> {
        let templates = self.templates.read()
            .map_err(|_| StorageError::BackendError {
                message: "Failed to acquire read lock".to_string(),
            })?;
        
        let results: Vec<TemplateMetadata> = templates.values()
            .filter_map(|versions| versions.last())
            .filter(|t| {
                let metadata = &t.template.metadata;
                
                // Check name
                if let Some(name_contains) = &criteria.name_contains {
                    if !metadata.name.contains(name_contains) {
                        return false;
                    }
                }
                
                // Check tags
                if let Some(required_tags) = &criteria.tags {
                    if !required_tags.iter().all(|tag| metadata.tags.contains(tag)) {
                        return false;
                    }
                }
                
                // Check context
                if let Some(context) = &criteria.context {
                    if metadata.context.as_ref() != Some(context) {
                        return false;
                    }
                }
                
                // Check author
                if let Some(author) = &criteria.author {
                    if metadata.author.as_ref() != Some(author) {
                        return false;
                    }
                }
                
                // Check date range
                if let Some(after) = criteria.created_after {
                    if metadata.created_at < after {
                        return false;
                    }
                }
                
                if let Some(before) = criteria.created_before {
                    if metadata.created_at > before {
                        return false;
                    }
                }
                
                true
            })
            .map(|t| t.template.metadata.clone())
            .collect();
        
        Ok(results)
    }
}

/// File-based storage backend
pub struct FileStorage {
    base_path: std::path::PathBuf,
}

impl FileStorage {
    /// Create new file storage
    pub fn new(base_path: impl Into<std::path::PathBuf>) -> Result<Self, StorageError> {
        let base_path = base_path.into();
        
        // Create directory if it doesn't exist
        std::fs::create_dir_all(&base_path)
            .map_err(|e| StorageError::IoError {
                message: format!("Failed to create storage directory: {}", e),
            })?;
        
        Ok(Self { base_path })
    }
    
    fn template_path(&self, id: &str, version: Option<u32>) -> std::path::PathBuf {
        let mut path = self.base_path.join(id);
        if let Some(v) = version {
            path.push(format!("v{}.json", v));
        } else {
            path.push("latest.json");
        }
        path
    }
    
    fn metadata_path(&self, id: &str) -> std::path::PathBuf {
        self.base_path.join(id).join("metadata.json")
    }
}

#[async_trait]
impl StorageBackend for FileStorage {
    fn save(&self, template: &Template) -> Result<(), StorageError> {
        let template_dir = self.base_path.join(&template.id.0);
        std::fs::create_dir_all(&template_dir)
            .map_err(|e| StorageError::IoError {
                message: format!("Failed to create template directory: {}", e),
            })?;
        
        // Save versioned template
        let version_path = self.template_path(&template.id.0, Some(template.metadata.version));
        let json = serde_json::to_string_pretty(template)
            .map_err(|e| StorageError::SerializationError {
                message: e.to_string(),
            })?;
        
        std::fs::write(&version_path, json)
            .map_err(|e| StorageError::IoError {
                message: format!("Failed to write template file: {}", e),
            })?;
        
        // Update latest symlink or copy
        let latest_path = self.template_path(&template.id.0, None);
        #[cfg(unix)]
        {
            if latest_path.exists() {
                std::fs::remove_file(&latest_path).ok();
            }
            std::os::unix::fs::symlink(&version_path, &latest_path)
                .map_err(|e| StorageError::IoError {
                    message: format!("Failed to create symlink: {}", e),
                })?;
        }
        #[cfg(not(unix))]
        {
            std::fs::copy(&version_path, &latest_path)
                .map_err(|e| StorageError::IoError {
                    message: format!("Failed to copy latest version: {}", e),
                })?;
        }
        
        // Update metadata
        let metadata_path = self.metadata_path(&template.id.0);
        let metadata_json = serde_json::to_string_pretty(&template.metadata)
            .map_err(|e| StorageError::SerializationError {
                message: e.to_string(),
            })?;
        
        std::fs::write(metadata_path, metadata_json)
            .map_err(|e| StorageError::IoError {
                message: format!("Failed to write metadata: {}", e),
            })?;
        
        Ok(())
    }
    
    fn get(&self, id: &str, version: Option<u32>) -> Result<Template, StorageError> {
        let path = self.template_path(id, version);
        
        let json = std::fs::read_to_string(&path)
            .map_err(|_| StorageError::NotFound {
                id: id.to_string(),
            })?;
        
        serde_json::from_str(&json)
            .map_err(|e| StorageError::SerializationError {
                message: e.to_string(),
            })
    }
    
    fn list(&self) -> Result<Vec<TemplateMetadata>, StorageError> {
        let mut metadata_list = Vec::new();
        
        let entries = std::fs::read_dir(&self.base_path)
            .map_err(|e| StorageError::IoError {
                message: format!("Failed to read storage directory: {}", e),
            })?;
        
        for entry in entries {
            let entry = entry.map_err(|e| StorageError::IoError {
                message: format!("Failed to read directory entry: {}", e),
            })?;
            
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let metadata_path = entry.path().join("metadata.json");
                if metadata_path.exists() {
                    let json = std::fs::read_to_string(&metadata_path)
                        .map_err(|e| StorageError::IoError {
                            message: format!("Failed to read metadata: {}", e),
                        })?;
                    
                    let metadata: TemplateMetadata = serde_json::from_str(&json)
                        .map_err(|e| StorageError::SerializationError {
                            message: e.to_string(),
                        })?;
                    
                    metadata_list.push(metadata);
                }
            }
        }
        
        Ok(metadata_list)
    }
    
    fn get_history(&self, id: &str) -> Result<Vec<TemplateVersion>, StorageError> {
        let template_dir = self.base_path.join(id);
        
        if !template_dir.exists() {
            return Err(StorageError::NotFound {
                id: id.to_string(),
            });
        }
        
        let mut versions = Vec::new();
        
        let entries = std::fs::read_dir(&template_dir)
            .map_err(|e| StorageError::IoError {
                message: format!("Failed to read template directory: {}", e),
            })?;
        
        for entry in entries {
            let entry = entry.map_err(|e| StorageError::IoError {
                message: format!("Failed to read directory entry: {}", e),
            })?;
            
            let filename = entry.file_name();
            let filename_str = filename.to_string_lossy();
            
            // Parse version files (v1.json, v2.json, etc.)
            if filename_str.starts_with('v') && filename_str.ends_with(".json") {
                if let Ok(template) = self.get(id, extract_version(&filename_str)) {
                    versions.push(TemplateVersion {
                        version: template.metadata.version,
                        created_at: template.metadata.updated_at,
                        created_by: template.metadata.author.clone(),
                        comment: None,
                        content_hash: calculate_hash(&template.content),
                    });
                }
            }
        }
        
        // Sort by version number
        versions.sort_by_key(|v| v.version);
        
        Ok(versions)
    }
    
    fn delete(&self, id: &str) -> Result<(), StorageError> {
        let template_dir = self.base_path.join(id);
        
        if !template_dir.exists() {
            return Err(StorageError::NotFound {
                id: id.to_string(),
            });
        }
        
        std::fs::remove_dir_all(template_dir)
            .map_err(|e| StorageError::IoError {
                message: format!("Failed to delete template directory: {}", e),
            })?;
        
        Ok(())
    }
    
    fn search(&self, criteria: &SearchCriteria) -> Result<Vec<TemplateMetadata>, StorageError> {
        // For file storage, we need to load all metadata and filter
        let all_metadata = self.list()?;
        
        Ok(all_metadata.into_iter()
            .filter(|metadata| {
                // Apply same filtering logic as memory storage
                if let Some(name_contains) = &criteria.name_contains {
                    if !metadata.name.contains(name_contains) {
                        return false;
                    }
                }
                
                if let Some(required_tags) = &criteria.tags {
                    if !required_tags.iter().all(|tag| metadata.tags.contains(tag)) {
                        return false;
                    }
                }
                
                if let Some(context) = &criteria.context {
                    if metadata.context.as_ref() != Some(context) {
                        return false;
                    }
                }
                
                if let Some(author) = &criteria.author {
                    if metadata.author.as_ref() != Some(author) {
                        return false;
                    }
                }
                
                if let Some(after) = criteria.created_after {
                    if metadata.created_at < after {
                        return false;
                    }
                }
                
                if let Some(before) = criteria.created_before {
                    if metadata.created_at > before {
                        return false;
                    }
                }
                
                true
            })
            .collect())
    }
}

/// Database storage backend wrapper
#[cfg(feature = "database")]
pub struct DatabaseStorage {
    pool: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
}

#[cfg(feature = "database")]
impl DatabaseStorage {
    /// Create new database storage
    pub fn new(database_url: &str) -> Result<Self, StorageError> {
        use diesel::r2d2::{ConnectionManager, Pool};
        
        let manager = ConnectionManager::<diesel::PgConnection>::new(database_url);
        let pool = Pool::builder()
            .build(manager)
            .map_err(|e| StorageError::BackendError {
                message: format!("Failed to create connection pool: {}", e),
            })?;
        
        Ok(Self { pool })
    }
}

// Note: Full database implementation would require schema migrations and diesel models

/// Main template storage manager
pub struct TemplateStorage {
    backend: Box<dyn StorageBackend>,
}

impl TemplateStorage {
    /// Create with memory backend
    pub fn memory() -> Self {
        Self {
            backend: Box::new(MemoryStorage::new()),
        }
    }
    
    /// Create with file backend
    pub fn file(path: impl Into<std::path::PathBuf>) -> Result<Self, StorageError> {
        Ok(Self {
            backend: Box::new(FileStorage::new(path)?),
        })
    }
    
    /// Create with custom backend
    pub fn with_backend(backend: Box<dyn StorageBackend>) -> Self {
        Self { backend }
    }
    
    /// Save a template
    pub fn save(&self, template: &Template) -> Result<(), StorageError> {
        self.backend.save(template)
    }
    
    /// Get a template
    pub fn get(&self, id: &str, version: Option<u32>) -> Result<Template, StorageError> {
        self.backend.get(id, version)
    }
    
    /// List all templates
    pub fn list(&self) -> Result<Vec<TemplateMetadata>, StorageError> {
        self.backend.list()
    }
    
    /// Get template history
    pub fn get_history(&self, id: &str) -> Result<Vec<TemplateVersion>, StorageError> {
        self.backend.get_history(id)
    }
    
    /// Delete a template
    pub fn delete(&self, id: &str) -> Result<(), StorageError> {
        self.backend.delete(id)
    }
    
    /// Search templates
    pub fn search(&self, criteria: &SearchCriteria) -> Result<Vec<TemplateMetadata>, StorageError> {
        self.backend.search(criteria)
    }
}

/// Calculate content hash
fn calculate_hash(content: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Extract version number from filename
fn extract_version(filename: &str) -> Option<u32> {
    filename.strip_prefix('v')
        .and_then(|s| s.strip_suffix(".json"))
        .and_then(|s| s.parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_storage() {
        let storage = TemplateStorage::memory();
        
        // Create and save template
        let template = Template::new("test", "Hello {{name}}!").unwrap();
        storage.save(&template).unwrap();
        
        // Retrieve template
        let retrieved = storage.get("test", None).unwrap();
        assert_eq!(retrieved.id, template.id);
        assert_eq!(retrieved.content, template.content);
        
        // List templates
        let list = storage.list().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id.0, "test");
        
        // Update template
        let updated = template.with_content("Hi {{name}}!");
        storage.save(&updated).unwrap();
        
        // Check history
        let history = storage.get_history("test").unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].version, 1);
        assert_eq!(history[1].version, 2);
        
        // Delete template
        storage.delete("test").unwrap();
        let list = storage.list().unwrap();
        assert_eq!(list.len(), 0);
    }
    
    #[test]
    fn test_search() {
        let storage = TemplateStorage::memory();
        
        // Create templates with different attributes
        let template1 = Template::new("greeting", "Hello!")
            .unwrap()
            .with_tags(vec!["greeting".to_string(), "casual".to_string()]);
        
        let template2 = Template::new("farewell", "Goodbye!")
            .unwrap()
            .with_tags(vec!["farewell".to_string(), "formal".to_string()]);
        
        storage.save(&template1).unwrap();
        storage.save(&template2).unwrap();
        
        // Search by tag
        let criteria = SearchCriteria {
            tags: Some(vec!["greeting".to_string()]),
            ..Default::default()
        };
        
        let results = storage.search(&criteria).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id.0, "greeting");
        
        // Search by name
        let criteria = SearchCriteria {
            name_contains: Some("fare".to_string()),
            ..Default::default()
        };
        
        let results = storage.search(&criteria).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id.0, "farewell");
    }
}

impl Default for SearchCriteria {
    fn default() -> Self {
        Self {
            name_contains: None,
            tags: None,
            context: None,
            author: None,
            created_after: None,
            created_before: None,
        }
    }
}