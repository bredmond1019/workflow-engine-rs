//! DGraph client and connection management

use anyhow::{Context, Result};
use dgraph_tonic::{Client, Query};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

/// Configuration for DGraph connection
#[derive(Debug, Clone)]
pub struct DgraphConfig {
    pub host: String,
    pub grpc_port: u16,
    pub http_port: u16,
    pub max_connections: usize,
    pub query_timeout_ms: u64,
    pub mutation_timeout_ms: u64,
}

impl Default for DgraphConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            grpc_port: 9080,
            http_port: 8080,
            max_connections: 20,
            query_timeout_ms: 30_000,
            mutation_timeout_ms: 60_000,
        }
    }
}

/// DGraph client wrapper with connection pooling
pub struct GraphDatabase {
    client: Arc<Client>,
    config: DgraphConfig,
    stats: Arc<RwLock<ConnectionStats>>,
}

#[derive(Debug, Default)]
struct ConnectionStats {
    queries_executed: u64,
    mutations_executed: u64,
    errors: u64,
    avg_query_time_ms: f64,
}

impl GraphDatabase {
    /// Create a new connection to DGraph
    pub async fn new(config: DgraphConfig) -> Result<Self> {
        let endpoint = format!("{}:{}", config.host, config.grpc_port);
        
        info!("Connecting to DGraph at {}", endpoint);
        
        let client = Client::new(&endpoint)
            .context("Failed to create DGraph client")?;
        
        // Test the connection
        Self::test_connection(&client).await?;
        
        info!("Successfully connected to DGraph");
        
        Ok(Self {
            client: Arc::new(client),
            config,
            stats: Arc::new(RwLock::new(ConnectionStats::default())),
        })
    }
    
    /// Test the DGraph connection
    async fn test_connection(client: &Client) -> Result<()> {
        let query = "{ health() }";
        
        client
            .new_read_only_txn()
            .query(query)
            .await
            .context("DGraph health check failed")?;
        
        Ok(())
    }
    
    /// Execute a GraphQL query
    pub async fn query(&self, query: &str) -> Result<serde_json::Value> {
        let start = std::time::Instant::now();
        
        let mut txn = self.client.new_read_only_txn();
        let response = txn
            .query(query)
            .await
            .context("Failed to execute query")?;
        
        let duration = start.elapsed();
        self.update_stats(duration, false, true).await;
        
        let result: serde_json::Value = serde_json::from_slice(&response.json)
            .context("Failed to parse query response")?;
        
        Ok(result)
    }
    
    /// Execute a GraphQL query with variables
    pub async fn query_with_vars(
        &self,
        query: &str,
        vars: HashMap<String, String>,
    ) -> Result<serde_json::Value> {
        let start = std::time::Instant::now();
        
        let mut txn = self.client.new_read_only_txn();
        let response = txn
            .query_with_vars(query, vars)
            .await
            .context("Failed to execute query with variables")?;
        
        let duration = start.elapsed();
        self.update_stats(duration, false, true).await;
        
        let result: serde_json::Value = serde_json::from_slice(&response.json)
            .context("Failed to parse query response")?;
        
        Ok(result)
    }
    
    /// Execute a GraphQL mutation
    pub async fn mutate(&self, mutation: &str) -> Result<serde_json::Value> {
        let start = std::time::Instant::now();
        
        // For GraphQL mutations, we use the HTTP endpoint
        let endpoint = format!("http://{}:{}/graphql", self.config.host, self.config.http_port);
        let client = reqwest::Client::new();
        
        let response = client.post(&endpoint)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "query": mutation
            }))
            .send()
            .await
            .context("Failed to send mutation request")?;

        let result: serde_json::Value = response.json().await
            .context("Failed to parse mutation response")?;
        
        let duration = start.elapsed();
        self.update_stats(duration, true, true).await;
        
        Ok(result)
    }
    
    /// Execute a transaction with multiple operations
    pub async fn transaction<F, T>(&self, operations: F) -> Result<T>
    where
        F: FnOnce(&Client) -> Result<T>,
    {
        let start = std::time::Instant::now();
        
        // TODO: Implement proper transaction with dgraph-tonic 0.11 API
        let result = operations(&self.client)?;
        
        let duration = start.elapsed();
        self.update_stats(duration, true, true).await;
        Ok(result)
    }
    
    /// Update connection statistics
    async fn update_stats(&self, duration: std::time::Duration, is_mutation: bool, success: bool) {
        let mut stats = self.stats.write().await;
        
        let duration_ms = duration.as_millis() as f64;
        
        if is_mutation {
            stats.mutations_executed += 1;
        } else {
            stats.queries_executed += 1;
        }
        
        if !success {
            stats.errors += 1;
        }
        
        // Update running average
        let total_ops = stats.queries_executed + stats.mutations_executed;
        stats.avg_query_time_ms = 
            (stats.avg_query_time_ms * (total_ops - 1) as f64 + duration_ms) / total_ops as f64;
    }
    
    /// Get connection statistics
    pub async fn get_stats(&self) -> ConnectionStats {
        self.stats.read().await.clone()
    }
    
    /// Get the underlying DGraph client
    pub fn client(&self) -> Arc<Client> {
        Arc::clone(&self.client)
    }
    
    /// Check if the connection is healthy
    pub async fn health_check(&self) -> Result<bool> {
        match Self::test_connection(&self.client).await {
            Ok(_) => Ok(true),
            Err(e) => {
                warn!("DGraph health check failed: {}", e);
                Ok(false)
            }
        }
    }
}

impl Clone for ConnectionStats {
    fn clone(&self) -> Self {
        Self {
            queries_executed: self.queries_executed,
            mutations_executed: self.mutations_executed,
            errors: self.errors,
            avg_query_time_ms: self.avg_query_time_ms,
        }
    }
}

// Data models for the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concept {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub difficulty: String,
    pub category: String,
    pub subcategory: Option<String>,
    pub tags: Vec<String>,
    pub quality_score: f32,
    pub estimated_time: Option<f32>,
    pub embeddings: Vec<f32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningResource {
    pub id: Uuid,
    pub url: String,
    pub title: String,
    pub resource_type: String,
    pub format: Option<String>,
    pub source: Option<String>,
    pub quality: Option<f32>,
    pub difficulty: Option<String>,
    pub duration: Option<i32>,
    pub language: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningPath {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub target_audience: Option<String>,
    pub estimated_time: Option<f32>,
    pub difficulty_progression: Option<String>,
    pub learning_outcomes: Vec<String>,
    pub creator: Option<String>,
    pub is_custom: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathConcept {
    pub concept_id: Uuid,
    pub order: i32,
    pub is_optional: bool,
    pub alternative_concepts: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProgress {
    pub id: Uuid,
    pub user_id: String,
    pub concept_id: Uuid,
    pub status: String, // not_started, in_progress, completed, mastered
    pub percent_complete: Option<f32>,
    pub time_spent: Option<i32>,
    pub resources_completed: Option<i32>,
    pub difficulty_rating: Option<f32>,
    pub notes: Option<String>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_accessed_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    #[ignore] // Requires DGraph instance
    async fn test_dgraph_connection() {
        let config = DgraphConfig::default();
        let db = GraphDatabase::new(config).await;
        assert!(db.is_ok());
    }
    
    #[tokio::test]
    #[ignore] // Requires DGraph instance
    async fn test_health_check() {
        let config = DgraphConfig::default();
        let db = GraphDatabase::new(config).await.unwrap();
        let is_healthy = db.health_check().await.unwrap();
        assert!(is_healthy);
    }
    
    #[tokio::test]
    #[ignore] // Requires DGraph instance
    async fn test_simple_query() {
        let config = DgraphConfig::default();
        let db = GraphDatabase::new(config).await.unwrap();
        
        let query = r#"
            {
                q(func: has(dgraph.type)) {
                    count(uid)
                }
            }
        "#;
        
        let result = db.query(query).await;
        assert!(result.is_ok());
    }
}