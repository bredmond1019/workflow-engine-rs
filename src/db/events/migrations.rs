// File: src/db/events/migrations.rs
//
// Concrete event migrations and migration registry
// Provides common migration patterns and a registry system

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, error};

use super::{
    EventError, EventResult,
    versioning::{EventMigrator, SchemaVersion},
};

/// Registry for managing event migrations
pub struct MigrationRegistry {
    migrations: Vec<Box<dyn EventMigrator>>,
    schema_versions: Vec<SchemaVersion>,
}

impl MigrationRegistry {
    pub fn new() -> Self {
        Self {
            migrations: Vec::new(),
            schema_versions: Vec::new(),
        }
    }
    
    /// Add a migration to the registry
    pub fn add_migration<M: EventMigrator + 'static>(mut self, migration: M) -> Self {
        self.migrations.push(Box::new(migration));
        self
    }
    
    /// Add a schema version to the registry
    pub fn add_schema_version(mut self, schema_version: SchemaVersion) -> Self {
        self.schema_versions.push(schema_version);
        self
    }
    
    /// Get all migrations
    pub fn migrations(&self) -> &[Box<dyn EventMigrator>] {
        &self.migrations
    }
    
    /// Get all schema versions
    pub fn schema_versions(&self) -> &[SchemaVersion] {
        &self.schema_versions
    }
    
    /// Build a registry with common workflow event migrations
    pub fn workflow_migrations() -> Self {
        Self::new()
            // Workflow event schema versions
            .add_schema_version(SchemaVersion::new(
                "workflow_started".to_string(),
                1,
                "Initial workflow started event".to_string(),
            ))
            .add_schema_version(SchemaVersion::new(
                "workflow_started".to_string(),
                2,
                "Added user context and metadata".to_string(),
            ))
            .add_schema_version(SchemaVersion::new(
                "workflow_completed".to_string(),
                1,
                "Initial workflow completed event".to_string(),
            ))
            .add_schema_version(SchemaVersion::new(
                "workflow_completed".to_string(),
                2,
                "Added performance metrics and output validation".to_string(),
            ))
            
            // Migrations
            .add_migration(WorkflowStartedV1ToV2Migration)
            .add_migration(WorkflowCompletedV1ToV2Migration)
    }
    
    /// Build a registry with AI interaction event migrations
    pub fn ai_interaction_migrations() -> Self {
        Self::new()
            // AI interaction schema versions
            .add_schema_version(SchemaVersion::new(
                "prompt_sent".to_string(),
                1,
                "Initial prompt sent event".to_string(),
            ))
            .add_schema_version(SchemaVersion::new(
                "prompt_sent".to_string(),
                2,
                "Added token count estimation and model parameters".to_string(),
            ))
            .add_schema_version(SchemaVersion::new(
                "response_received".to_string(),
                1,
                "Initial response received event".to_string(),
            ))
            .add_schema_version(SchemaVersion::new(
                "response_received".to_string(),
                2,
                "Added detailed token usage and cost tracking".to_string(),
            ))
            
            // Migrations
            .add_migration(PromptSentV1ToV2Migration)
            .add_migration(ResponseReceivedV1ToV2Migration)
    }
}

impl Default for MigrationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Workflow Event Migrations

/// Migration for WorkflowStarted events from v1 to v2
pub struct WorkflowStartedV1ToV2Migration;

#[async_trait]
impl EventMigrator for WorkflowStartedV1ToV2Migration {
    fn event_type(&self) -> &str {
        "workflow_started"
    }
    
    fn from_version(&self) -> i32 {
        1
    }
    
    fn to_version(&self) -> i32 {
        2
    }
    
    async fn migrate(&self, mut event_data: serde_json::Value) -> EventResult<serde_json::Value> {
        if let Some(obj) = event_data.as_object_mut() {
            // Add user context if not present
            if !obj.contains_key("user_context") {
                obj.insert("user_context".to_string(), serde_json::json!({
                    "user_id": null,
                    "session_id": null,
                    "ip_address": null
                }));
            }
            
            // Add enhanced metadata
            if !obj.contains_key("metadata") {
                obj.insert("metadata".to_string(), serde_json::json!({}));
            }
            
            if let Some(metadata) = obj.get_mut("metadata").and_then(|m| m.as_object_mut()) {
                metadata.insert("schema_migrated_from".to_string(), serde_json::json!("v1"));
                metadata.insert("migration_timestamp".to_string(), serde_json::json!(Utc::now()));
                
                // Add workflow priority if not present
                if !metadata.contains_key("priority") {
                    metadata.insert("priority".to_string(), serde_json::json!("normal"));
                }
                
                // Add retry configuration
                if !metadata.contains_key("retry_config") {
                    metadata.insert("retry_config".to_string(), serde_json::json!({
                        "max_retries": 3,
                        "retry_delay_seconds": 60
                    }));
                }
            }
        }
        
        Ok(event_data)
    }
    
    async fn can_migrate(&self, event_data: &serde_json::Value) -> bool {
        // Check if this looks like a v1 workflow_started event
        if let Some(obj) = event_data.as_object() {
            obj.contains_key("workflow_id") && obj.contains_key("workflow_type")
        } else {
            false
        }
    }
    
    fn description(&self) -> &str {
        "Migrate WorkflowStarted from v1 to v2: add user context and enhanced metadata"
    }
}

/// Migration for WorkflowCompleted events from v1 to v2
pub struct WorkflowCompletedV1ToV2Migration;

#[async_trait]
impl EventMigrator for WorkflowCompletedV1ToV2Migration {
    fn event_type(&self) -> &str {
        "workflow_completed"
    }
    
    fn from_version(&self) -> i32 {
        1
    }
    
    fn to_version(&self) -> i32 {
        2
    }
    
    async fn migrate(&self, mut event_data: serde_json::Value) -> EventResult<serde_json::Value> {
        if let Some(obj) = event_data.as_object_mut() {
            // Add performance metrics if not present
            if !obj.contains_key("performance_metrics") {
                let duration_ms = obj.get("duration_ms")
                    .and_then(|d| d.as_i64())
                    .unwrap_or(0);
                
                obj.insert("performance_metrics".to_string(), serde_json::json!({
                    "total_duration_ms": duration_ms,
                    "node_execution_times": {},
                    "memory_usage_peak_mb": null,
                    "cpu_usage_average_percent": null,
                    "network_requests_count": 0,
                    "cache_hit_ratio": null
                }));
            }
            
            // Add output validation if not present
            if !obj.contains_key("output_validation") {
                obj.insert("output_validation".to_string(), serde_json::json!({
                    "is_valid": true,
                    "validation_errors": [],
                    "schema_version": "unknown",
                    "validated_at": Utc::now()
                }));
            }
            
            // Add quality metrics
            if !obj.contains_key("quality_metrics") {
                obj.insert("quality_metrics".to_string(), serde_json::json!({
                    "success_rate": 1.0,
                    "error_count": 0,
                    "warning_count": 0,
                    "completeness_score": 1.0
                }));
            }
        }
        
        Ok(event_data)
    }
    
    async fn can_migrate(&self, event_data: &serde_json::Value) -> bool {
        if let Some(obj) = event_data.as_object() {
            obj.contains_key("workflow_id") && obj.contains_key("output_data")
        } else {
            false
        }
    }
    
    fn description(&self) -> &str {
        "Migrate WorkflowCompleted from v1 to v2: add performance metrics and output validation"
    }
}

// AI Interaction Event Migrations

/// Migration for PromptSent events from v1 to v2
pub struct PromptSentV1ToV2Migration;

#[async_trait]
impl EventMigrator for PromptSentV1ToV2Migration {
    fn event_type(&self) -> &str {
        "prompt_sent"
    }
    
    fn from_version(&self) -> i32 {
        1
    }
    
    fn to_version(&self) -> i32 {
        2
    }
    
    async fn migrate(&self, mut event_data: serde_json::Value) -> EventResult<serde_json::Value> {
        if let Some(obj) = event_data.as_object_mut() {
            // Add token count estimation if not present
            if !obj.contains_key("estimated_tokens") {
                let prompt = obj.get("prompt")
                    .and_then(|p| p.as_str())
                    .unwrap_or("");
                
                // Simple estimation: roughly 4 characters per token
                let estimated_tokens = prompt.len() / 4;
                
                obj.insert("estimated_tokens".to_string(), serde_json::json!({
                    "prompt_tokens": estimated_tokens,
                    "estimation_method": "simple_character_count",
                    "estimated_at": Utc::now()
                }));
            }
            
            // Enhance model parameters
            if !obj.contains_key("model_parameters") {
                obj.insert("model_parameters".to_string(), serde_json::json!({}));
            }
            
            if let Some(params) = obj.get_mut("model_parameters").and_then(|p| p.as_object_mut()) {
                // Add default parameters if not present
                if !params.contains_key("temperature") {
                    params.insert("temperature".to_string(), serde_json::json!(0.7));
                }
                if !params.contains_key("max_tokens") {
                    params.insert("max_tokens".to_string(), serde_json::json!(null));
                }
                if !params.contains_key("top_p") {
                    params.insert("top_p".to_string(), serde_json::json!(1.0));
                }
                if !params.contains_key("frequency_penalty") {
                    params.insert("frequency_penalty".to_string(), serde_json::json!(0.0));
                }
                if !params.contains_key("presence_penalty") {
                    params.insert("presence_penalty".to_string(), serde_json::json!(0.0));
                }
            }
            
            // Add cost estimation
            if !obj.contains_key("cost_estimation") {
                obj.insert("cost_estimation".to_string(), serde_json::json!({
                    "estimated_cost_usd": null,
                    "cost_model": "unknown",
                    "estimation_timestamp": Utc::now()
                }));
            }
        }
        
        Ok(event_data)
    }
    
    async fn can_migrate(&self, event_data: &serde_json::Value) -> bool {
        if let Some(obj) = event_data.as_object() {
            obj.contains_key("request_id") && obj.contains_key("prompt")
        } else {
            false
        }
    }
    
    fn description(&self) -> &str {
        "Migrate PromptSent from v1 to v2: add token estimation and enhanced model parameters"
    }
}

/// Migration for ResponseReceived events from v1 to v2
pub struct ResponseReceivedV1ToV2Migration;

#[async_trait]
impl EventMigrator for ResponseReceivedV1ToV2Migration {
    fn event_type(&self) -> &str {
        "response_received"
    }
    
    fn from_version(&self) -> i32 {
        1
    }
    
    fn to_version(&self) -> i32 {
        2
    }
    
    async fn migrate(&self, mut event_data: serde_json::Value) -> EventResult<serde_json::Value> {
        if let Some(obj) = event_data.as_object_mut() {
            // Add detailed token usage if not present
            if !obj.contains_key("detailed_usage") {
                let prompt_tokens = obj.get("prompt_tokens").and_then(|t| t.as_i64()).unwrap_or(0);
                let completion_tokens = obj.get("completion_tokens").and_then(|t| t.as_i64()).unwrap_or(0);
                let total_tokens = obj.get("total_tokens").and_then(|t| t.as_i64()).unwrap_or(prompt_tokens + completion_tokens);
                
                obj.insert("detailed_usage".to_string(), serde_json::json!({
                    "prompt_tokens": prompt_tokens,
                    "completion_tokens": completion_tokens,
                    "total_tokens": total_tokens,
                    "cached_tokens": 0,
                    "reasoning_tokens": 0,
                    "tool_use_tokens": 0
                }));
            }
            
            // Add cost breakdown if not present
            if !obj.contains_key("cost_breakdown") {
                let cost_usd = obj.get("cost_usd").and_then(|c| c.as_f64());
                
                obj.insert("cost_breakdown".to_string(), serde_json::json!({
                    "total_cost_usd": cost_usd,
                    "prompt_cost_usd": null,
                    "completion_cost_usd": null,
                    "additional_fees_usd": 0.0,
                    "currency": "USD",
                    "rate_timestamp": Utc::now()
                }));
            }
            
            // Add response quality metrics
            if !obj.contains_key("quality_metrics") {
                let response = obj.get("response").and_then(|r| r.as_str()).unwrap_or("");
                
                obj.insert("quality_metrics".to_string(), serde_json::json!({
                    "response_length": response.len(),
                    "estimated_readability": null,
                    "language_detected": null,
                    "contains_code": response.contains("```"),
                    "contains_links": response.contains("http"),
                    "sentiment_score": null
                }));
            }
            
            // Add performance metrics
            if !obj.contains_key("performance_metrics") {
                let duration_ms = obj.get("duration_ms").and_then(|d| d.as_i64()).unwrap_or(0);
                
                obj.insert("performance_metrics".to_string(), serde_json::json!({
                    "total_duration_ms": duration_ms,
                    "time_to_first_token_ms": null,
                    "tokens_per_second": null,
                    "model_processing_time_ms": null,
                    "network_latency_ms": null
                }));
            }
        }
        
        Ok(event_data)
    }
    
    async fn can_migrate(&self, event_data: &serde_json::Value) -> bool {
        if let Some(obj) = event_data.as_object() {
            obj.contains_key("request_id") && obj.contains_key("response")
        } else {
            false
        }
    }
    
    fn description(&self) -> &str {
        "Migrate ResponseReceived from v1 to v2: add detailed usage, cost breakdown, and quality metrics"
    }
}

/// Generic field renaming migration
pub struct FieldRenameMigration {
    event_type: String,
    from_version: i32,
    to_version: i32,
    field_renames: HashMap<String, String>,
    description: String,
}

impl FieldRenameMigration {
    pub fn new(
        event_type: String,
        from_version: i32,
        to_version: i32,
        field_renames: HashMap<String, String>,
    ) -> Self {
        let description = format!(
            "Rename fields for {} v{} to v{}: {:?}",
            event_type, from_version, to_version, field_renames
        );
        
        Self {
            event_type,
            from_version,
            to_version,
            field_renames,
            description,
        }
    }
}

#[async_trait]
impl EventMigrator for FieldRenameMigration {
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
        if let Some(obj) = event_data.as_object_mut() {
            for (old_name, new_name) in &self.field_renames {
                if let Some(value) = obj.remove(old_name) {
                    obj.insert(new_name.clone(), value);
                }
            }
        }
        
        Ok(event_data)
    }
    
    fn description(&self) -> &str {
        &self.description
    }
}

/// Generic field removal migration
pub struct FieldRemovalMigration {
    event_type: String,
    from_version: i32,
    to_version: i32,
    fields_to_remove: Vec<String>,
    description: String,
}

impl FieldRemovalMigration {
    pub fn new(
        event_type: String,
        from_version: i32,
        to_version: i32,
        fields_to_remove: Vec<String>,
    ) -> Self {
        let description = format!(
            "Remove fields for {} v{} to v{}: {:?}",
            event_type, from_version, to_version, fields_to_remove
        );
        
        Self {
            event_type,
            from_version,
            to_version,
            fields_to_remove,
            description,
        }
    }
}

#[async_trait]
impl EventMigrator for FieldRemovalMigration {
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
        if let Some(obj) = event_data.as_object_mut() {
            for field_name in &self.fields_to_remove {
                obj.remove(field_name);
            }
        }
        
        Ok(event_data)
    }
    
    fn description(&self) -> &str {
        &self.description
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[tokio::test]
    async fn test_workflow_started_v1_to_v2_migration() {
        let migration = WorkflowStartedV1ToV2Migration;
        
        let v1_data = json!({
            "workflow_id": "wf_123",
            "workflow_type": "data_processing",
            "configuration": {
                "timeout": 300
            }
        });
        
        let migrated = migration.migrate(v1_data).await.unwrap();
        
        // Check that user_context was added
        assert!(migrated["user_context"].is_object());
        assert!(migrated["user_context"]["user_id"].is_null());
        
        // Check that metadata was enhanced
        assert!(migrated["metadata"].is_object());
        assert_eq!(migrated["metadata"]["priority"], "normal");
        assert!(migrated["metadata"]["retry_config"].is_object());
        assert_eq!(migrated["metadata"]["retry_config"]["max_retries"], 3);
    }
    
    #[tokio::test]
    async fn test_prompt_sent_v1_to_v2_migration() {
        let migration = PromptSentV1ToV2Migration;
        
        let v1_data = json!({
            "request_id": "req_123",
            "prompt": "Hello, world! This is a test prompt that should be estimated for tokens.",
            "model": "gpt-4"
        });
        
        let migrated = migration.migrate(v1_data).await.unwrap();
        
        // Check that token estimation was added
        assert!(migrated["estimated_tokens"].is_object());
        assert!(migrated["estimated_tokens"]["prompt_tokens"].is_number());
        
        // Check that model parameters were enhanced
        assert!(migrated["model_parameters"].is_object());
        assert_eq!(migrated["model_parameters"]["temperature"], 0.7);
        assert_eq!(migrated["model_parameters"]["top_p"], 1.0);
        
        // Check cost estimation
        assert!(migrated["cost_estimation"].is_object());
    }
    
    #[tokio::test]
    async fn test_field_rename_migration() {
        let mut renames = HashMap::new();
        renames.insert("old_field".to_string(), "new_field".to_string());
        renames.insert("another_old".to_string(), "another_new".to_string());
        
        let migration = FieldRenameMigration::new(
            "test_event".to_string(),
            1,
            2,
            renames,
        );
        
        let test_data = json!({
            "old_field": "value1",
            "another_old": "value2",
            "unchanged_field": "value3"
        });
        
        let migrated = migration.migrate(test_data).await.unwrap();
        
        // Check that fields were renamed
        assert_eq!(migrated["new_field"], "value1");
        assert_eq!(migrated["another_new"], "value2");
        assert_eq!(migrated["unchanged_field"], "value3");
        
        // Check that old fields were removed
        assert!(migrated["old_field"].is_null());
        assert!(migrated["another_old"].is_null());
    }
    
    #[tokio::test]
    async fn test_field_removal_migration() {
        let fields_to_remove = vec![
            "deprecated_field".to_string(),
            "unused_field".to_string(),
        ];
        
        let migration = FieldRemovalMigration::new(
            "test_event".to_string(),
            1,
            2,
            fields_to_remove,
        );
        
        let test_data = json!({
            "deprecated_field": "value1",
            "unused_field": "value2",
            "important_field": "value3"
        });
        
        let migrated = migration.migrate(test_data).await.unwrap();
        
        // Check that deprecated fields were removed
        assert!(migrated["deprecated_field"].is_null());
        assert!(migrated["unused_field"].is_null());
        
        // Check that important field was preserved
        assert_eq!(migrated["important_field"], "value3");
    }
    
    #[test]
    fn test_migration_registry_workflow() {
        let registry = MigrationRegistry::workflow_migrations();
        
        let migrations = registry.migrations();
        let schema_versions = registry.schema_versions();
        
        // Should have migrations for workflow events
        assert!(!migrations.is_empty());
        assert!(!schema_versions.is_empty());
        
        // Check that we have the expected migrations
        let has_workflow_started_migration = migrations.iter().any(|m| {
            m.event_type() == "workflow_started" && m.from_version() == 1 && m.to_version() == 2
        });
        assert!(has_workflow_started_migration);
    }
}