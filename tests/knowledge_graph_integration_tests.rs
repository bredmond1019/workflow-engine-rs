//! Knowledge Graph Integration Tests with Real Dgraph Instance
//! 
//! These tests verify end-to-end functionality against a real Dgraph database.
//! They require a running Dgraph instance and are marked with #[ignore] to run
//! only when explicitly requested with: cargo test --ignored
//! 
//! Setup:
//! 1. Start test Dgraph: cd services/knowledge_graph && docker-compose -f docker-compose.test.yml up -d
//! 2. Run tests: cargo test knowledge_graph_integration --ignored
//! 3. Cleanup: docker-compose -f docker-compose.test.yml down -v

use knowledge_graph::{
    client::{DgraphClient, ConnectionConfig},
    graph::{Concept, DgraphConfig, GraphDatabase},
    query::{QueryBuilder, QueryType, QueryParameters, QueryConstraints},
    service::KnowledgeGraphService,
    error::KnowledgeGraphError,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use uuid::Uuid;
use chrono::Utc;

/// Test configuration for Dgraph instance
const TEST_DGRAPH_HOST: &str = "localhost";
const TEST_DGRAPH_GRPC_PORT: u16 = 19080;
const TEST_DGRAPH_HTTP_PORT: u16 = 18080;
const TEST_TIMEOUT_SECS: u64 = 30;

/// Helper to create test Dgraph configuration
fn create_test_config() -> DgraphConfig {
    DgraphConfig {
        hosts: vec![format!("{}:{}", TEST_DGRAPH_HOST, TEST_DGRAPH_GRPC_PORT)],
        pool_size: 5,
        timeout_seconds: TEST_TIMEOUT_SECS,
        max_retries: 3,
        retry_delay_ms: 100,
        enable_tls: false,
        enable_compression: false,
        ..Default::default()
    }
}

/// Helper to wait for Dgraph to be ready
async fn wait_for_dgraph() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let health_url = format!("http://{}:{}/health", TEST_DGRAPH_HOST, TEST_DGRAPH_HTTP_PORT);
    
    for attempt in 1..=30 {
        match client.get(&health_url).send().await {
            Ok(response) if response.status().is_success() => {
                println!("Dgraph is ready after {} attempts", attempt);
                return Ok(());
            }
            _ => {
                if attempt == 30 {
                    return Err("Dgraph failed to become ready within timeout".into());
                }
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
    Ok(())
}

/// Helper to clean up test data
async fn cleanup_test_data(client: &DgraphClient) -> Result<(), KnowledgeGraphError> {
    let cleanup_mutation = r#"
    {
        delete {
            <*> * * .
        }
    }
    "#;
    
    // Note: In production, you'd want more targeted cleanup
    // This is acceptable for isolated test environments
    client.mutate(cleanup_mutation, true).await?;
    Ok(())
}

/// Helper to insert test concept
async fn insert_test_concept(
    client: &DgraphClient,
    name: &str,
    category: &str,
    difficulty: &str,
) -> Result<String, KnowledgeGraphError> {
    let mutation = json!({
        "set": [{
            "dgraph.type": "TestConcept",
            "name": name,
            "description": format!("Test concept: {}", name),
            "difficulty": difficulty,
            "category": category,
            "tags": ["test"],
            "qualityScore": 0.8,
            "estimatedTime": 60.0,
            "createdAt": Utc::now().to_rfc3339(),
            "updatedAt": Utc::now().to_rfc3339(),
            "version": 1
        }]
    });

    let result = client.mutate(&mutation.to_string(), true).await?;
    
    // Extract UID from result
    if let Some(uids) = result.get("uids") {
        if let Some(uid_map) = uids.as_object() {
            for (_, uid_value) in uid_map {
                if let Some(uid) = uid_value.as_str() {
                    return Ok(uid.to_string());
                }
            }
        }
    }
    
    Err(KnowledgeGraphError::QueryError("Failed to get UID from mutation result".to_string()))
}

#[tokio::test]
#[ignore]
async fn test_dgraph_connection_and_health() {
    // Wait for Dgraph to be ready
    wait_for_dgraph().await.expect("Dgraph should be ready");
    
    let config = create_test_config();
    let client = DgraphClient::new(config.hosts[0].clone()).await
        .expect("Should connect to Dgraph");
    
    // Test health check
    let health = client.health_check().await;
    assert!(health.is_ok(), "Health check should pass");
    assert!(health.unwrap(), "Dgraph should be healthy");
}

#[tokio::test]
#[ignore]
async fn test_graph_database_initialization() {
    wait_for_dgraph().await.expect("Dgraph should be ready");
    
    let config = create_test_config();
    let db = GraphDatabase::new(config).await
        .expect("Should create GraphDatabase");
    
    let is_healthy = db.health_check().await
        .expect("Health check should work");
    assert!(is_healthy, "Database should be healthy");
}

#[tokio::test]
#[ignore]
async fn test_schema_loading_and_validation() {
    wait_for_dgraph().await.expect("Dgraph should be ready");
    
    let config = create_test_config();
    let client = DgraphClient::new(config.hosts[0].clone()).await
        .expect("Should connect to Dgraph");
    
    // Query schema to verify it's loaded
    let schema_query = "schema {}";
    let result = client.query(schema_query).await
        .expect("Should query schema");
    
    let schema_text = result.to_string();
    assert!(schema_text.contains("TestConcept"), "Schema should contain TestConcept type");
    assert!(schema_text.contains("TestResource"), "Schema should contain TestResource type");
    assert!(schema_text.contains("TestLearningPath"), "Schema should contain TestLearningPath type");
}

#[tokio::test]
#[ignore]
async fn test_basic_query_operations() {
    wait_for_dgraph().await.expect("Dgraph should be ready");
    
    let config = create_test_config();
    let client = DgraphClient::new(config.hosts[0].clone()).await
        .expect("Should connect to Dgraph");
    
    // Query for test concepts
    let query = r#"
    {
        concepts(func: type(TestConcept)) @filter(eq(category, "programming")) {
            uid
            name
            difficulty
            category
            qualityScore
        }
    }
    "#;
    
    let result = client.query(query).await
        .expect("Should execute query successfully");
    
    println!("Query result: {}", serde_json::to_string_pretty(&result).unwrap());
    
    // Verify we got programming concepts
    if let Some(concepts) = result.get("concepts").and_then(|c| c.as_array()) {
        assert!(!concepts.is_empty(), "Should find programming concepts from sample data");
        
        for concept in concepts {
            assert!(concept.get("name").is_some(), "Concept should have name");
            assert!(concept.get("category").is_some(), "Concept should have category");
            assert_eq!(
                concept.get("category").and_then(|c| c.as_str()),
                Some("programming"),
                "All concepts should be programming category"
            );
        }
    } else {
        panic!("Expected concepts array in result");
    }
}

#[tokio::test]
#[ignore]
async fn test_relationship_queries() {
    wait_for_dgraph().await.expect("Dgraph should be ready");
    
    let config = create_test_config();
    let client = DgraphClient::new(config.hosts[0].clone()).await
        .expect("Should connect to Dgraph");
    
    // Query for concepts with relationships
    let query = r#"
    {
        concepts(func: type(TestConcept)) @filter(eq(name, "Functions")) {
            uid
            name
            prerequisites {
                uid
                name
            }
            enabledBy {
                uid
                name
            }
        }
    }
    "#;
    
    let result = client.query(query).await
        .expect("Should execute relationship query");
    
    if let Some(concepts) = result.get("concepts").and_then(|c| c.as_array()) {
        assert!(!concepts.is_empty(), "Should find Functions concept");
        
        let functions_concept = &concepts[0];
        assert_eq!(
            functions_concept.get("name").and_then(|n| n.as_str()),
            Some("Functions")
        );
        
        // Check prerequisites
        if let Some(prerequisites) = functions_concept.get("prerequisites").and_then(|p| p.as_array()) {
            assert!(!prerequisites.is_empty(), "Functions should have prerequisites");
            assert!(
                prerequisites.iter().any(|p| 
                    p.get("name").and_then(|n| n.as_str()) == Some("Variables")
                ),
                "Functions should have Variables as prerequisite"
            );
        }
    } else {
        panic!("Expected to find Functions concept");
    }
}

#[tokio::test]
#[ignore]
async fn test_search_functionality() {
    wait_for_dgraph().await.expect("Dgraph should be ready");
    
    let config = create_test_config();
    let client = DgraphClient::new(config.hosts[0].clone()).await
        .expect("Should connect to Dgraph");
    
    // Test fulltext search
    let search_query = r#"
    {
        concepts(func: alloftext(description, "data structure")) {
            uid
            name
            description
            category
        }
    }
    "#;
    
    let result = client.query(search_query).await
        .expect("Should execute search query");
    
    if let Some(concepts) = result.get("concepts").and_then(|c| c.as_array()) {
        // Should find concepts related to data structures
        let found_data_structures = concepts.iter().any(|c| {
            c.get("name").and_then(|n| n.as_str())
                .map(|name| name.contains("Data Structures") || name.contains("Arrays") || name.contains("Linked Lists"))
                .unwrap_or(false)
        });
        
        assert!(found_data_structures, "Should find data structure related concepts");
    }
    
    // Test exact search
    let exact_query = r#"
    {
        concepts(func: type(TestConcept)) @filter(eq(difficulty, "beginner")) {
            uid
            name
            difficulty
        }
    }
    "#;
    
    let result = client.query(exact_query).await
        .expect("Should execute exact search");
    
    if let Some(concepts) = result.get("concepts").and_then(|c| c.as_array()) {
        assert!(!concepts.is_empty(), "Should find beginner concepts");
        
        for concept in concepts {
            assert_eq!(
                concept.get("difficulty").and_then(|d| d.as_str()),
                Some("beginner"),
                "All concepts should be beginner difficulty"
            );
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_aggregation_queries() {
    wait_for_dgraph().await.expect("Dgraph should be ready");
    
    let config = create_test_config();
    let client = DgraphClient::new(config.hosts[0].clone()).await
        .expect("Should connect to Dgraph");
    
    // Test count aggregation
    let count_query = r#"
    {
        concept_count(func: type(TestConcept)) {
            count(uid)
        }
    }
    "#;
    
    let result = client.query(count_query).await
        .expect("Should execute count query");
    
    if let Some(count_result) = result.get("concept_count").and_then(|c| c.as_array()) {
        if let Some(count_obj) = count_result.first() {
            if let Some(count) = count_obj.get("count").and_then(|c| c.as_u64()) {
                assert!(count > 0, "Should have concepts in database");
                println!("Total concepts: {}", count);
            }
        }
    }
    
    // Test average quality score
    let avg_query = r#"
    {
        quality_stats(func: type(TestConcept)) {
            avg_quality: avg(qualityScore)
            max_quality: max(qualityScore)
            min_quality: min(qualityScore)
        }
    }
    "#;
    
    let result = client.query(avg_query).await
        .expect("Should execute aggregation query");
    
    if let Some(stats) = result.get("quality_stats").and_then(|s| s.as_array()) {
        if let Some(stat_obj) = stats.first() {
            assert!(stat_obj.get("avg_quality").is_some(), "Should have average quality");
            assert!(stat_obj.get("max_quality").is_some(), "Should have max quality");
            assert!(stat_obj.get("min_quality").is_some(), "Should have min quality");
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_complex_traversal_queries() {
    wait_for_dgraph().await.expect("Dgraph should be ready");
    
    let config = create_test_config();
    let client = DgraphClient::new(config.hosts[0].clone()).await
        .expect("Should connect to Dgraph");
    
    // Test multi-level traversal
    let traversal_query = r#"
    {
        start_concepts(func: type(TestConcept)) @filter(eq(name, "Variables")) {
            uid
            name
            enabledBy {
                uid
                name
                enabledBy {
                    uid
                    name
                }
            }
        }
    }
    "#;
    
    let result = client.query(traversal_query).await
        .expect("Should execute traversal query");
    
    if let Some(concepts) = result.get("start_concepts").and_then(|c| c.as_array()) {
        assert!(!concepts.is_empty(), "Should find Variables concept");
        
        let variables_concept = &concepts[0];
        assert_eq!(
            variables_concept.get("name").and_then(|n| n.as_str()),
            Some("Variables")
        );
        
        // Check if we can traverse to enabled concepts
        if let Some(enabled_by) = variables_concept.get("enabledBy").and_then(|e| e.as_array()) {
            assert!(!enabled_by.is_empty(), "Variables should enable other concepts");
            
            // Check for nested traversal
            for enabled_concept in enabled_by {
                println!("Variables enables: {}", 
                    enabled_concept.get("name").and_then(|n| n.as_str()).unwrap_or("unknown"));
                
                if let Some(nested_enabled) = enabled_concept.get("enabledBy").and_then(|e| e.as_array()) {
                    for nested in nested_enabled {
                        println!("  Which enables: {}", 
                            nested.get("name").and_then(|n| n.as_str()).unwrap_or("unknown"));
                    }
                }
            }
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_facet_queries() {
    wait_for_dgraph().await.expect("Dgraph should be ready");
    
    let config = create_test_config();
    let client = DgraphClient::new(config.hosts[0].clone()).await
        .expect("Should connect to Dgraph");
    
    // Test resource relationships with facets
    let facet_query = r#"
    {
        concepts_with_resources(func: type(TestConcept)) @filter(has(resources)) {
            uid
            name
            resources {
                uid
                title
                resourceType
                quality
            }
        }
    }
    "#;
    
    let result = client.query(facet_query).await
        .expect("Should execute facet query");
    
    if let Some(concepts) = result.get("concepts_with_resources").and_then(|c| c.as_array()) {
        for concept in concepts {
            if let Some(resources) = concept.get("resources").and_then(|r| r.as_array()) {
                for resource in resources {
                    assert!(resource.get("title").is_some(), "Resource should have title");
                    assert!(resource.get("resourceType").is_some(), "Resource should have type");
                    
                    if let Some(quality) = resource.get("quality").and_then(|q| q.as_f64()) {
                        assert!(quality >= 0.0 && quality <= 1.0, "Quality should be between 0 and 1");
                    }
                }
            }
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_recursive_queries() {
    wait_for_dgraph().await.expect("Dgraph should be ready");
    
    let config = create_test_config();
    let client = DgraphClient::new(config.hosts[0].clone()).await
        .expect("Should connect to Dgraph");
    
    // Test recursive prerequisite chain
    let recursive_query = r#"
    {
        concept_chains(func: type(TestConcept)) @filter(eq(name, "Object-Oriented Programming")) @recurse(depth: 10) {
            uid
            name
            prerequisites
        }
    }
    "#;
    
    let result = client.query(recursive_query).await
        .expect("Should execute recursive query");
    
    if let Some(chains) = result.get("concept_chains").and_then(|c| c.as_array()) {
        assert!(!chains.is_empty(), "Should find OOP concept and its prerequisite chain");
        
        // The recursive query should include the full prerequisite chain
        let total_nodes = count_recursive_nodes(&chains[0]);
        assert!(total_nodes > 1, "Should have prerequisite chain with multiple concepts");
        
        println!("Found {} concepts in prerequisite chain", total_nodes);
    }
}

/// Helper function to count nodes in recursive query result
fn count_recursive_nodes(node: &Value) -> usize {
    let mut count = 1; // Count current node
    
    if let Some(prerequisites) = node.get("prerequisites").and_then(|p| p.as_array()) {
        for prereq in prerequisites {
            count += count_recursive_nodes(prereq);
        }
    }
    
    count
}

#[tokio::test]
#[ignore]
async fn test_pagination_and_limits() {
    wait_for_dgraph().await.expect("Dgraph should be ready");
    
    let config = create_test_config();
    let client = DgraphClient::new(config.hosts[0].clone()).await
        .expect("Should connect to Dgraph");
    
    // Test pagination with first/offset
    let paginated_query = r#"
    {
        page1: concepts(func: type(TestConcept), first: 3, offset: 0) {
            uid
            name
        }
        page2: concepts(func: type(TestConcept), first: 3, offset: 3) {
            uid
            name
        }
    }
    "#;
    
    let result = client.query(paginated_query).await
        .expect("Should execute paginated query");
    
    let page1 = result.get("page1").and_then(|p| p.as_array()).unwrap_or(&vec![]);
    let page2 = result.get("page2").and_then(|p| p.as_array()).unwrap_or(&vec![]);
    
    assert!(page1.len() <= 3, "Page 1 should have at most 3 items");
    assert!(page2.len() <= 3, "Page 2 should have at most 3 items");
    
    // Verify no duplicate UIDs between pages
    let page1_uids: Vec<&str> = page1.iter()
        .filter_map(|c| c.get("uid").and_then(|u| u.as_str()))
        .collect();
    let page2_uids: Vec<&str> = page2.iter()
        .filter_map(|c| c.get("uid").and_then(|u| u.as_str()))
        .collect();
    
    for uid1 in &page1_uids {
        assert!(!page2_uids.contains(uid1), "Pages should not have overlapping UIDs");
    }
}

#[tokio::test]
#[ignore]
async fn test_sorting_and_ordering() {
    wait_for_dgraph().await.expect("Dgraph should be ready");
    
    let config = create_test_config();
    let client = DgraphClient::new(config.hosts[0].clone()).await
        .expect("Should connect to Dgraph");
    
    // Test sorting by quality score
    let sorted_query = r#"
    {
        concepts_by_quality(func: type(TestConcept), orderasc: qualityScore) {
            uid
            name
            qualityScore
        }
        concepts_by_quality_desc(func: type(TestConcept), orderdesc: qualityScore) {
            uid
            name
            qualityScore
        }
    }
    "#;
    
    let result = client.query(sorted_query).await
        .expect("Should execute sorted query");
    
    // Check ascending order
    if let Some(asc_concepts) = result.get("concepts_by_quality").and_then(|c| c.as_array()) {
        let mut prev_quality = 0.0;
        for concept in asc_concepts {
            if let Some(quality) = concept.get("qualityScore").and_then(|q| q.as_f64()) {
                assert!(quality >= prev_quality, "Concepts should be sorted by quality ascending");
                prev_quality = quality;
            }
        }
    }
    
    // Check descending order
    if let Some(desc_concepts) = result.get("concepts_by_quality_desc").and_then(|c| c.as_array()) {
        let mut prev_quality = 1.0;
        for concept in desc_concepts {
            if let Some(quality) = concept.get("qualityScore").and_then(|q| q.as_f64()) {
                assert!(quality <= prev_quality, "Concepts should be sorted by quality descending");
                prev_quality = quality;
            }
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_query_builder_integration() {
    wait_for_dgraph().await.expect("Dgraph should be ready");
    
    let config = create_test_config();
    let client = DgraphClient::new(config.hosts[0].clone()).await
        .expect("Should connect to Dgraph");
    
    let query_builder = QueryBuilder::new();
    
    // Test search query building and execution
    let params = QueryParameters {
        search_term: Some("programming".to_string()),
        limit: Some(5),
        offset: Some(0),
        constraints: Some(QueryConstraints {
            difficulty: vec!["beginner".to_string(), "intermediate".to_string()],
            min_quality: 0.8,
            categories: Some(vec!["programming".to_string()]),
            include_subtopics: Some(true),
        }),
        sort_by: Some("qualityScore".to_string()),
        sort_order: Some("desc".to_string()),
        ..Default::default()
    };
    
    let query = query_builder.build_query(QueryType::SearchConcepts, params)
        .expect("Should build search query");
    
    println!("Generated query: {}", query);
    
    // Execute the generated query
    let result = client.query(&query).await
        .expect("Should execute generated query");
    
    println!("Query result: {}", serde_json::to_string_pretty(&result).unwrap());
    
    // Verify results match constraints
    if let Some(concepts) = result.get("concepts").and_then(|c| c.as_array()) {
        for concept in concepts {
            // Check category constraint
            if let Some(category) = concept.get("category").and_then(|c| c.as_str()) {
                assert_eq!(category, "programming", "Should only return programming concepts");
            }
            
            // Check difficulty constraint
            if let Some(difficulty) = concept.get("difficulty").and_then(|d| d.as_str()) {
                assert!(
                    difficulty == "beginner" || difficulty == "intermediate",
                    "Should only return beginner or intermediate concepts"
                );
            }
            
            // Check quality constraint
            if let Some(quality) = concept.get("qualityScore").and_then(|q| q.as_f64()) {
                assert!(quality >= 0.8, "Should only return high quality concepts");
            }
        }
        
        assert!(concepts.len() <= 5, "Should respect limit constraint");
    }
}

#[tokio::test]
#[ignore]
async fn test_performance_with_large_queries() {
    wait_for_dgraph().await.expect("Dgraph should be ready");
    
    let config = create_test_config();
    let client = DgraphClient::new(config.hosts[0].clone()).await
        .expect("Should connect to Dgraph");
    
    // Test query performance
    let complex_query = r#"
    {
        all_concepts(func: type(TestConcept)) {
            uid
            name
            description
            category
            difficulty
            qualityScore
            estimatedTime
            prerequisites {
                uid
                name
                category
            }
            enabledBy {
                uid
                name
                category
            }
            resources {
                uid
                title
                resourceType
                quality
            }
        }
    }
    "#;
    
    let start = Instant::now();
    let result = client.query(complex_query).await
        .expect("Should execute complex query");
    let duration = start.elapsed();
    
    println!("Complex query took: {:?}", duration);
    
    // Verify we got comprehensive results
    if let Some(concepts) = result.get("all_concepts").and_then(|c| c.as_array()) {
        assert!(!concepts.is_empty(), "Should return concepts");
        
        for concept in concepts {
            assert!(concept.get("uid").is_some(), "Each concept should have UID");
            assert!(concept.get("name").is_some(), "Each concept should have name");
        }
    }
    
    // Performance should be reasonable (adjust threshold as needed)
    assert!(duration.as_millis() < 5000, "Complex query should complete within 5 seconds");
}

/// Integration test for KnowledgeGraphService
#[tokio::test]
#[ignore]
async fn test_knowledge_graph_service_integration() {
    wait_for_dgraph().await.expect("Dgraph should be ready");
    
    let config = create_test_config();
    let service = KnowledgeGraphService::new(config).await
        .expect("Should create KnowledgeGraphService");
    
    // Test service health
    let health = service.health_check().await
        .expect("Health check should work");
    assert!(health, "Service should be healthy");
    
    // Test would continue with service-specific operations...
    // Note: Actual service methods would need to be implemented and tested here
}