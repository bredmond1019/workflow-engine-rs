//! Test utilities for GraphQL Federation testing
//! 
//! This module provides helper functions and utilities for testing
//! the GraphQL Federation implementation.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use reqwest::Client;
use serde_json::{json, Value};
use tokio::time::timeout;

/// Test configuration for federation services
#[derive(Debug, Clone)]
pub struct FederationTestConfig {
    pub gateway_url: String,
    pub subgraph_urls: HashMap<String, String>,
    pub timeout_duration: Duration,
}

impl Default for FederationTestConfig {
    fn default() -> Self {
        let mut subgraph_urls = HashMap::new();
        subgraph_urls.insert("workflow".to_string(), "http://localhost:8080/api/v1/graphql".to_string());
        subgraph_urls.insert("content_processing".to_string(), "http://localhost:3001/graphql".to_string());
        subgraph_urls.insert("knowledge_graph".to_string(), "http://localhost:3002/graphql".to_string());
        subgraph_urls.insert("realtime_communication".to_string(), "http://localhost:3003/graphql".to_string());
        
        Self {
            gateway_url: "http://localhost:4000/graphql".to_string(),
            subgraph_urls,
            timeout_duration: Duration::from_secs(10),
        }
    }
}

/// Result of a GraphQL query execution
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub response: Value,
    pub execution_time: Duration,
    pub success: bool,
    pub errors: Vec<String>,
}

/// Federation test client for executing queries and validations
pub struct FederationTestClient {
    client: Client,
    config: FederationTestConfig,
}

impl FederationTestClient {
    pub fn new(config: FederationTestConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }
    
    pub fn default() -> Self {
        Self::new(FederationTestConfig::default())
    }
    
    /// Execute a GraphQL query against the gateway
    pub async fn execute_gateway_query(&self, query: &str, variables: Option<Value>) -> Result<QueryResult, Box<dyn std::error::Error>> {
        self.execute_query(&self.config.gateway_url, query, variables).await
    }
    
    /// Execute a GraphQL query against a specific subgraph
    pub async fn execute_subgraph_query(&self, subgraph: &str, query: &str, variables: Option<Value>) -> Result<QueryResult, Box<dyn std::error::Error>> {
        let url = self.config.subgraph_urls.get(subgraph)
            .ok_or_else(|| format!("Unknown subgraph: {}", subgraph))?;
        
        self.execute_query(url, query, variables).await
    }
    
    /// Execute a GraphQL query against any URL
    pub async fn execute_query(&self, url: &str, query: &str, variables: Option<Value>) -> Result<QueryResult, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        
        let payload = json!({
            "query": query,
            "variables": variables.unwrap_or(json!({}))
        });
        
        let response = timeout(
            self.config.timeout_duration,
            self.client.post(url)
                .json(&payload)
                .send()
        ).await??;
        
        let response_json: Value = response.json().await?;
        let execution_time = start_time.elapsed();
        
        let success = !response_json.get("errors").map_or(false, |e| !e.as_array().unwrap_or(&vec![]).is_empty());
        let errors = response_json.get("errors")
            .and_then(|e| e.as_array())
            .map(|errors| {
                errors.iter()
                    .filter_map(|err| err.get("message").and_then(|m| m.as_str()))
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();
        
        Ok(QueryResult {
            response: response_json,
            execution_time,
            success,
            errors,
        })
    }
    
    /// Check if a service is healthy
    pub async fn check_service_health(&self, url: &str) -> bool {
        let health_query = r#"{ __schema { queryType { name } } }"#;
        
        match self.execute_query(url, health_query, None).await {
            Ok(result) => result.success,
            Err(_) => false,
        }
    }
    
    /// Check if all configured services are healthy
    pub async fn check_all_services_health(&self) -> HashMap<String, bool> {
        let mut health_status = HashMap::new();
        
        // Check gateway
        let gateway_healthy = self.check_service_health(&self.config.gateway_url).await;
        health_status.insert("gateway".to_string(), gateway_healthy);
        
        // Check all subgraphs
        for (name, url) in &self.config.subgraph_urls {
            let healthy = self.check_service_health(url).await;
            health_status.insert(name.clone(), healthy);
        }
        
        health_status
    }
    
    /// Get the SDL schema from a subgraph
    pub async fn get_subgraph_schema(&self, subgraph: &str) -> Result<String, Box<dyn std::error::Error>> {
        let service_query = r#"{ _service { sdl } }"#;
        
        let result = self.execute_subgraph_query(subgraph, service_query, None).await?;
        
        if !result.success {
            return Err(format!("Failed to get schema from {}: {:?}", subgraph, result.errors).into());
        }
        
        let sdl = result.response
            .get("data")
            .and_then(|d| d.get("_service"))
            .and_then(|s| s.get("sdl"))
            .and_then(|sdl| sdl.as_str())
            .ok_or("No SDL found in response")?;
        
        Ok(sdl.to_string())
    }
    
    /// Test entity resolution for specific entities
    pub async fn test_entity_resolution(&self, representations: Vec<Value>) -> Result<QueryResult, Box<dyn std::error::Error>> {
        let query = r#"
            query EntityResolution($representations: [_Any!]!) {
                _entities(representations: $representations) {
                    __typename
                }
            }
        "#;
        
        let variables = json!({
            "representations": representations
        });
        
        self.execute_gateway_query(query, Some(variables)).await
    }
    
    /// Validate federation directives in a schema
    pub fn validate_federation_directives(&self, sdl: &str) -> FederationDirectiveValidation {
        let mut validation = FederationDirectiveValidation::default();
        
        // Check for federation directives
        validation.has_key = sdl.contains("@key");
        validation.has_extends = sdl.contains("@extends");
        validation.has_external = sdl.contains("@external");
        validation.has_provides = sdl.contains("@provides");
        validation.has_requires = sdl.contains("@requires");
        
        // Check for federation types
        validation.has_service_type = sdl.contains("_Service");
        validation.has_entities_field = sdl.contains("_entities");
        validation.has_any_scalar = sdl.contains("_Any");
        
        // Check for entity types (types with @key directive)
        let entity_count = sdl.matches("@key").count();
        validation.entity_count = entity_count;
        
        validation.is_valid = validation.has_service_type && validation.has_entities_field;
        
        validation
    }
    
    /// Run a comprehensive federation health check
    pub async fn comprehensive_health_check(&self) -> FederationHealthReport {
        let mut report = FederationHealthReport::default();
        
        // Check service health
        report.service_health = self.check_all_services_health().await;
        report.all_services_healthy = report.service_health.values().all(|&healthy| healthy);
        
        // If services are healthy, check federation features
        if report.all_services_healthy {
            // Test gateway introspection
            if let Ok(result) = self.execute_gateway_query(r#"{ __schema { queryType { name } } }"#, None).await {
                report.gateway_introspection_working = result.success;
            }
            
            // Test subgraph schemas
            for subgraph in self.config.subgraph_urls.keys() {
                if let Ok(sdl) = self.get_subgraph_schema(subgraph).await {
                    let validation = self.validate_federation_directives(&sdl);
                    report.subgraph_schemas.insert(subgraph.clone(), validation);
                }
            }
            
            // Test basic entity resolution
            let test_representations = vec![
                json!({"__typename": "User", "id": "test_user"}),
                json!({"__typename": "Workflow", "id": "test_workflow"}),
            ];
            
            if let Ok(result) = self.test_entity_resolution(test_representations).await {
                report.entity_resolution_working = result.success;
            }
        }
        
        report
    }
}

/// Validation result for federation directives in a schema
#[derive(Debug, Clone, Default)]
pub struct FederationDirectiveValidation {
    pub has_key: bool,
    pub has_extends: bool,
    pub has_external: bool,
    pub has_provides: bool,
    pub has_requires: bool,
    pub has_service_type: bool,
    pub has_entities_field: bool,
    pub has_any_scalar: bool,
    pub entity_count: usize,
    pub is_valid: bool,
}

impl FederationDirectiveValidation {
    pub fn score(&self) -> f64 {
        let mut score = 0.0;
        let total_checks = 8.0;
        
        if self.has_key { score += 1.0; }
        if self.has_extends { score += 1.0; }
        if self.has_external { score += 1.0; }
        if self.has_provides { score += 0.5; }
        if self.has_requires { score += 0.5; }
        if self.has_service_type { score += 1.0; }
        if self.has_entities_field { score += 1.0; }
        if self.has_any_scalar { score += 1.0; }
        if self.entity_count > 0 { score += 1.0; }
        
        score / total_checks
    }
}

/// Comprehensive health report for the federation system
#[derive(Debug, Clone, Default)]
pub struct FederationHealthReport {
    pub service_health: HashMap<String, bool>,
    pub all_services_healthy: bool,
    pub gateway_introspection_working: bool,
    pub entity_resolution_working: bool,
    pub subgraph_schemas: HashMap<String, FederationDirectiveValidation>,
}

impl FederationHealthReport {
    pub fn overall_health_score(&self) -> f64 {
        if !self.all_services_healthy {
            return 0.0;
        }
        
        let mut score = 0.0;
        let mut components = 0.0;
        
        // Service health (25%)
        score += 0.25;
        components += 1.0;
        
        // Gateway introspection (25%)
        if self.gateway_introspection_working {
            score += 0.25;
        }
        components += 1.0;
        
        // Entity resolution (25%)
        if self.entity_resolution_working {
            score += 0.25;
        }
        components += 1.0;
        
        // Schema validation (25%)
        if !self.subgraph_schemas.is_empty() {
            let schema_scores: Vec<f64> = self.subgraph_schemas.values()
                .map(|v| v.score())
                .collect();
            let avg_schema_score = schema_scores.iter().sum::<f64>() / schema_scores.len() as f64;
            score += 0.25 * avg_schema_score;
        }
        components += 1.0;
        
        score
    }
    
    pub fn print_summary(&self) {
        println!("Federation Health Report");
        println!("=======================");
        println!("Overall Health Score: {:.2}%", self.overall_health_score() * 100.0);
        println!();
        
        println!("Service Health:");
        for (service, healthy) in &self.service_health {
            let status = if *healthy { "✅ Healthy" } else { "❌ Unhealthy" };
            println!("  {}: {}", service, status);
        }
        println!();
        
        println!("Federation Features:");
        println!("  Gateway Introspection: {}", if self.gateway_introspection_working { "✅ Working" } else { "❌ Not Working" });
        println!("  Entity Resolution: {}", if self.entity_resolution_working { "✅ Working" } else { "❌ Not Working" });
        println!();
        
        println!("Subgraph Schema Validation:");
        for (subgraph, validation) in &self.subgraph_schemas {
            println!("  {}: {:.2}% compliant", subgraph, validation.score() * 100.0);
            if validation.entity_count > 0 {
                println!("    - {} entities with @key directive", validation.entity_count);
            }
            if !validation.is_valid {
                println!("    - ⚠️  Missing required federation types");
            }
        }
    }
}

/// Performance metrics for query execution
#[derive(Debug, Clone)]
pub struct QueryPerformanceMetrics {
    pub total_time: Duration,
    pub planning_time: Option<Duration>,
    pub execution_time: Option<Duration>,
    pub subgraph_calls: HashMap<String, Duration>,
    pub cache_hits: usize,
    pub cache_misses: usize,
}

impl QueryPerformanceMetrics {
    pub fn efficiency_score(&self) -> f64 {
        // Simple efficiency score based on execution time and cache performance
        let time_score = if self.total_time < Duration::from_millis(100) {
            1.0
        } else if self.total_time < Duration::from_millis(500) {
            0.8
        } else if self.total_time < Duration::from_secs(1) {
            0.6
        } else if self.total_time < Duration::from_secs(2) {
            0.4
        } else {
            0.2
        };
        
        let cache_score = if self.cache_hits + self.cache_misses == 0 {
            1.0 // No cache operations
        } else {
            self.cache_hits as f64 / (self.cache_hits + self.cache_misses) as f64
        };
        
        (time_score * 0.7) + (cache_score * 0.3)
    }
}

/// Test data generator for federation tests
pub struct FederationTestData;

impl FederationTestData {
    /// Generate test user entities
    pub fn users() -> Vec<Value> {
        vec![
            json!({"__typename": "User", "id": "user_1", "name": "Alice Smith"}),
            json!({"__typename": "User", "id": "user_2", "name": "Bob Johnson"}),
            json!({"__typename": "User", "id": "user_3", "name": "Charlie Brown"}),
        ]
    }
    
    /// Generate test workflow entities
    pub fn workflows() -> Vec<Value> {
        vec![
            json!({"__typename": "Workflow", "id": "wf_1", "name": "Data Processing", "status": "Active"}),
            json!({"__typename": "Workflow", "id": "wf_2", "name": "ML Training", "status": "Running"}),
            json!({"__typename": "Workflow", "id": "wf_3", "name": "Report Generation", "status": "Completed"}),
        ]
    }
    
    /// Generate test content entities
    pub fn content() -> Vec<Value> {
        vec![
            json!({"__typename": "ContentMetadata", "id": "content_1", "title": "Rust Guide", "contentType": "Markdown"}),
            json!({"__typename": "ContentMetadata", "id": "content_2", "title": "GraphQL Tutorial", "contentType": "Html"}),
            json!({"__typename": "ContentMetadata", "id": "content_3", "title": "API Documentation", "contentType": "Json"}),
        ]
    }
    
    /// Generate test concept entities
    pub fn concepts() -> Vec<Value> {
        vec![
            json!({"__typename": "Concept", "id": "concept_1", "name": "Rust Programming", "difficulty": "Intermediate"}),
            json!({"__typename": "Concept", "id": "concept_2", "name": "GraphQL Federation", "difficulty": "Advanced"}),
            json!({"__typename": "Concept", "id": "concept_3", "name": "Microservices", "difficulty": "Intermediate"}),
        ]
    }
    
    /// Generate test message entities
    pub fn messages() -> Vec<Value> {
        vec![
            json!({"__typename": "Message", "id": "msg_1", "content": "Hello World", "senderId": "user_1"}),
            json!({"__typename": "Message", "id": "msg_2", "content": "How are you?", "senderId": "user_2"}),
            json!({"__typename": "Message", "id": "msg_3", "content": "Great, thanks!", "senderId": "user_1"}),
        ]
    }
    
    /// Generate entity representations for testing entity resolution
    pub fn entity_representations() -> Vec<Value> {
        let mut representations = Vec::new();
        representations.extend(Self::users());
        representations.extend(Self::workflows());
        representations.extend(Self::content());
        representations.extend(Self::concepts());
        representations.extend(Self::messages());
        representations
    }
}

/// Common GraphQL queries for federation testing
pub struct FederationTestQueries;

impl FederationTestQueries {
    /// Basic health check query
    pub fn health_check() -> &'static str {
        r#"{ __schema { queryType { name } } }"#
    }
    
    /// Service SDL query
    pub fn service_sdl() -> &'static str {
        r#"{ _service { sdl } }"#
    }
    
    /// Entity resolution query
    pub fn entity_resolution() -> &'static str {
        r#"
            query EntityResolution($representations: [_Any!]!) {
                _entities(representations: $representations) {
                    __typename
                    ... on User { id }
                    ... on Workflow { id name status }
                    ... on ContentMetadata { id title contentType }
                    ... on Concept { id name difficulty }
                    ... on Message { id content }
                }
            }
        "#
    }
    
    /// Cross-service query
    pub fn cross_service() -> &'static str {
        r#"
            query CrossServiceQuery {
                workflows(limit: 2) { id name status }
                searchContent(limit: 2) { content { id title } }
                searchConcepts(query: "test", limit: 2) { concepts { id name } }
                conversations(limit: 2) { id name type }
            }
        "#
    }
    
    /// User with extensions from all services
    pub fn user_with_extensions() -> &'static str {
        r#"
            query UserWithExtensions($userId: ID!) {
                user(id: $userId) {
                    id
                    workflows { id name status }
                    processedContent { id title contentType }
                    completedConcepts { id name difficulty }
                    conversations { id name type }
                }
            }
        "#
    }
    
    /// Gateway introspection
    pub fn gateway_introspection() -> &'static str {
        r#"
            query GatewayIntrospection {
                __schema {
                    queryType { name }
                    mutationType { name }
                    subscriptionType { name }
                    directives { name locations }
                    types { name kind }
                }
            }
        "#
    }
}