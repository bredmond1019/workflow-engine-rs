//! Knowledge Graph Mutation Integration Tests
//! 
//! These tests verify mutation operations (create, update, delete) against a real Dgraph instance.
//! They test the complete mutation lifecycle and data consistency.
//! 
//! Setup:
//! 1. Start test Dgraph: cd services/knowledge_graph && docker-compose -f docker-compose.test.yml up -d
//! 2. Run tests: cargo test knowledge_graph_mutation --ignored
//! 3. Cleanup: docker-compose -f docker-compose.test.yml down -v

use knowledge_graph::{
    client::DgraphClient,
    graph::DgraphConfig,
    query::{QueryBuilder, QueryType, QueryParameters},
    error::KnowledgeGraphError,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;
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

/// Helper to extract UID from mutation result
fn extract_uid_from_result(result: &Value, blank_node: &str) -> Option<String> {
    result.get("uids")
        .and_then(|uids| uids.as_object())
        .and_then(|uid_map| uid_map.get(blank_node))
        .and_then(|uid| uid.as_str())
        .map(|s| s.to_string())
}

/// Helper to verify concept exists with expected data
async fn verify_concept_exists(
    client: &DgraphClient,
    uid: &str,
    expected_name: &str,
) -> Result<bool, KnowledgeGraphError> {
    let query = format!(
        r#"
        {{
            concept(func: uid("{}")) {{
                uid
                name
                dgraph.type
            }}
        }}
        "#,
        uid
    );
    
    let result = client.query(&query).await?;
    
    if let Some(concepts) = result.get("concept").and_then(|c| c.as_array()) {
        if let Some(concept) = concepts.first() {
            let name = concept.get("name").and_then(|n| n.as_str());
            return Ok(name == Some(expected_name));
        }
    }
    
    Ok(false)
}

/// Helper to count concepts matching criteria
async fn count_concepts_with_name(
    client: &DgraphClient,
    name: &str,
) -> Result<usize, KnowledgeGraphError> {
    let query = format!(
        r#"
        {{
            concepts(func: type(TestConcept)) @filter(eq(name, "{}")) {{
                uid
            }}
        }}
        "#,
        name
    );
    
    let result = client.query(&query).await?;
    
    if let Some(concepts) = result.get("concepts").and_then(|c| c.as_array()) {
        Ok(concepts.len())
    } else {
        Ok(0)
    }
}

#[tokio::test]
#[ignore]
async fn test_create_concept_mutation() {
    wait_for_dgraph().await.expect("Dgraph should be ready");
    
    let config = create_test_config();
    let client = DgraphClient::new(config.hosts[0].clone()).await
        .expect("Should connect to Dgraph");
    
    let test_name = format!("Test Concept {}", Uuid::new_v4());
    let now = Utc::now().to_rfc3339();
    
    // Create a new concept
    let mutation = json!({
        "set": [{
            "uid": "_:newconcept",
            "dgraph.type": "TestConcept",
            "name": test_name,
            "description": "A test concept created by integration test",
            "difficulty": "intermediate",
            "category": "testing",
            "tags": ["test", "integration", "mutation"],
            "qualityScore": 0.85,
            "estimatedTime": 75.0,
            "createdAt": now,
            "updatedAt": now,
            "version": 1
        }]
    });
    
    let result = client.mutate(&mutation.to_string(), true).await
        .expect("Should create concept successfully");
    
    // Extract the UID of the created concept
    let uid = extract_uid_from_result(&result, "newconcept")
        .expect("Should get UID from mutation result");
    
    println!("Created concept with UID: {}", uid);
    
    // Verify the concept was created correctly
    assert!(
        verify_concept_exists(&client, &uid, &test_name).await.unwrap(),
        "Created concept should exist with correct name"
    );
    
    // Query the full concept to verify all fields
    let query = format!(
        r#"
        {{
            concept(func: uid("{}")) {{
                uid
                name
                description
                difficulty
                category
                tags
                qualityScore
                estimatedTime
                version
                createdAt
                updatedAt
            }}
        }}
        "#,
        uid
    );
    
    let result = client.query(&query).await
        .expect("Should query created concept");
    
    if let Some(concepts) = result.get("concept").and_then(|c| c.as_array()) {
        let concept = &concepts[0];
        
        assert_eq!(concept.get("name").and_then(|n| n.as_str()), Some(test_name.as_str()));
        assert_eq!(concept.get("difficulty").and_then(|d| d.as_str()), Some("intermediate"));
        assert_eq!(concept.get("category").and_then(|c| c.as_str()), Some("testing"));
        assert_eq!(concept.get("qualityScore").and_then(|q| q.as_f64()), Some(0.85));
        assert_eq!(concept.get("estimatedTime").and_then(|t| t.as_f64()), Some(75.0));
        assert_eq!(concept.get("version").and_then(|v| v.as_i64()), Some(1));
        
        if let Some(tags) = concept.get("tags").and_then(|t| t.as_array()) {
            let tag_strings: Vec<&str> = tags.iter()
                .filter_map(|t| t.as_str())
                .collect();
            assert!(tag_strings.contains(&"test"));
            assert!(tag_strings.contains(&"integration"));
            assert!(tag_strings.contains(&"mutation"));
        }
    } else {
        panic!("Should find the created concept");
    }
}

#[tokio::test]
#[ignore]
async fn test_update_concept_mutation() {
    wait_for_dgraph().await.expect("Dgraph should be ready");
    
    let config = create_test_config();
    let client = DgraphClient::new(config.hosts[0].clone()).await
        .expect("Should connect to Dgraph");
    
    let original_name = format!("Original Concept {}", Uuid::new_v4());
    let updated_name = format!("Updated Concept {}", Uuid::new_v4());
    let now = Utc::now().to_rfc3339();
    
    // First, create a concept
    let create_mutation = json!({
        "set": [{
            "uid": "_:testconcept",
            "dgraph.type": "TestConcept",
            "name": original_name,
            "description": "Original description",
            "difficulty": "beginner",
            "category": "testing",
            "qualityScore": 0.7,
            "estimatedTime": 60.0,
            "createdAt": now,
            "updatedAt": now,
            "version": 1
        }]
    });
    
    let create_result = client.mutate(&create_mutation.to_string(), true).await
        .expect("Should create concept for update test");
    
    let uid = extract_uid_from_result(&create_result, "testconcept")
        .expect("Should get UID from creation");
    
    // Now update the concept
    let update_time = Utc::now().to_rfc3339();
    let update_mutation = json!({
        "set": [{
            "uid": uid,
            "name": updated_name,
            "description": "Updated description with more details",
            "difficulty": "intermediate",
            "qualityScore": 0.9,
            "estimatedTime": 90.0,
            "updatedAt": update_time,
            "version": 2
        }]
    });
    
    let update_result = client.mutate(&update_mutation.to_string(), true).await
        .expect("Should update concept successfully");
    
    println!("Updated concept result: {:?}", update_result);
    
    // Verify the concept was updated
    let query = format!(
        r#"
        {{
            concept(func: uid("{}")) {{
                uid
                name
                description
                difficulty
                category
                qualityScore
                estimatedTime
                version
                createdAt
                updatedAt
            }}
        }}
        "#,
        uid
    );
    
    let result = client.query(&query).await
        .expect("Should query updated concept");
    
    if let Some(concepts) = result.get("concept").and_then(|c| c.as_array()) {
        let concept = &concepts[0];
        
        // Verify updated fields
        assert_eq!(concept.get("name").and_then(|n| n.as_str()), Some(updated_name.as_str()));
        assert_eq!(concept.get("description").and_then(|d| d.as_str()), Some("Updated description with more details"));
        assert_eq!(concept.get("difficulty").and_then(|d| d.as_str()), Some("intermediate"));
        assert_eq!(concept.get("qualityScore").and_then(|q| q.as_f64()), Some(0.9));
        assert_eq!(concept.get("estimatedTime").and_then(|t| t.as_f64()), Some(90.0));
        assert_eq!(concept.get("version").and_then(|v| v.as_i64()), Some(2));
        
        // Verify unchanged fields
        assert_eq!(concept.get("category").and_then(|c| c.as_str()), Some("testing"));
        assert!(concept.get("createdAt").is_some());
        assert!(concept.get("updatedAt").is_some());
        
        // Verify updatedAt is more recent than createdAt
        let created_at = concept.get("createdAt").and_then(|c| c.as_str()).unwrap();
        let updated_at = concept.get("updatedAt").and_then(|u| u.as_str()).unwrap();
        assert_ne!(created_at, updated_at, "UpdatedAt should be different from createdAt");
    } else {
        panic!("Should find the updated concept");
    }
}

#[tokio::test]
#[ignore]
async fn test_delete_concept_mutation() {
    wait_for_dgraph().await.expect("Dgraph should be ready");
    
    let config = create_test_config();
    let client = DgraphClient::new(config.hosts[0].clone()).await
        .expect("Should connect to Dgraph");
    
    let test_name = format!("Concept to Delete {}", Uuid::new_v4());
    let now = Utc::now().to_rfc3339();
    
    // First, create a concept to delete
    let create_mutation = json!({
        "set": [{
            "uid": "_:deleteconcept",
            "dgraph.type": "TestConcept",
            "name": test_name,
            "description": "This concept will be deleted",
            "difficulty": "beginner",
            "category": "testing",
            "qualityScore": 0.8,
            "createdAt": now,
            "updatedAt": now,
            "version": 1
        }]
    });
    
    let create_result = client.mutate(&create_mutation.to_string(), true).await
        .expect("Should create concept for deletion test");
    
    let uid = extract_uid_from_result(&create_result, "deleteconcept")
        .expect("Should get UID from creation");
    
    // Verify the concept exists before deletion
    assert!(
        verify_concept_exists(&client, &uid, &test_name).await.unwrap(),
        "Concept should exist before deletion"
    );
    
    // Delete the concept
    let delete_mutation = json!({
        "delete": [{
            "uid": uid
        }]
    });
    
    let delete_result = client.mutate(&delete_mutation.to_string(), true).await
        .expect("Should delete concept successfully");
    
    println!("Delete result: {:?}", delete_result);
    
    // Verify the concept no longer exists
    let query = format!(
        r#"
        {{
            concept(func: uid("{}")) {{
                uid
                name
            }}
        }}
        "#,
        uid
    );
    
    let result = client.query(&query).await
        .expect("Should query for deleted concept");
    
    if let Some(concepts) = result.get("concept").and_then(|c| c.as_array()) {
        assert!(concepts.is_empty(), "Deleted concept should not be found");
    }
    
    // Also verify by name
    let count = count_concepts_with_name(&client, &test_name).await.unwrap();
    assert_eq!(count, 0, "Should not find any concepts with the deleted name");
}

#[tokio::test]
#[ignore]
async fn test_create_concept_with_relationships() {
    wait_for_dgraph().await.expect("Dgraph should be ready");
    
    let config = create_test_config();
    let client = DgraphClient::new(config.hosts[0].clone()).await
        .expect("Should connect to Dgraph");
    
    let prereq_name = format!("Prerequisite Concept {}", Uuid::new_v4());
    let advanced_name = format!("Advanced Concept {}", Uuid::new_v4());
    let now = Utc::now().to_rfc3339();
    
    // Create prerequisite concept first
    let prereq_mutation = json!({
        "set": [{
            "uid": "_:prereq",
            "dgraph.type": "TestConcept",
            "name": prereq_name,
            "description": "Prerequisite concept",
            "difficulty": "beginner",
            "category": "testing",
            "qualityScore": 0.8,
            "createdAt": now,
            "updatedAt": now,
            "version": 1
        }]
    });
    
    let prereq_result = client.mutate(&prereq_mutation.to_string(), true).await
        .expect("Should create prerequisite concept");
    
    let prereq_uid = extract_uid_from_result(&prereq_result, "prereq")
        .expect("Should get prerequisite UID");
    
    // Create advanced concept with relationship to prerequisite
    let advanced_mutation = json!({
        "set": [{
            "uid": "_:advanced",
            "dgraph.type": "TestConcept",
            "name": advanced_name,
            "description": "Advanced concept with prerequisites",
            "difficulty": "advanced",
            "category": "testing",
            "qualityScore": 0.9,
            "prerequisites": [{"uid": prereq_uid}],
            "createdAt": now,
            "updatedAt": now,
            "version": 1
        }]
    });
    
    let advanced_result = client.mutate(&advanced_mutation.to_string(), true).await
        .expect("Should create advanced concept with relationship");
    
    let advanced_uid = extract_uid_from_result(&advanced_result, "advanced")
        .expect("Should get advanced concept UID");
    
    // Verify the relationship was created correctly
    let query = format!(
        r#"
        {{
            advanced(func: uid("{}")) {{
                uid
                name
                prerequisites {{
                    uid
                    name
                }}
            }}
            prereq(func: uid("{}")) {{
                uid
                name
                enabledBy {{
                    uid
                    name
                }}
            }}
        }}
        "#,
        advanced_uid, prereq_uid
    );
    
    let result = client.query(&query).await
        .expect("Should query relationship");
    
    // Verify forward relationship
    if let Some(advanced_concepts) = result.get("advanced").and_then(|a| a.as_array()) {
        let advanced_concept = &advanced_concepts[0];
        assert_eq!(
            advanced_concept.get("name").and_then(|n| n.as_str()),
            Some(advanced_name.as_str())
        );
        
        if let Some(prerequisites) = advanced_concept.get("prerequisites").and_then(|p| p.as_array()) {
            assert_eq!(prerequisites.len(), 1, "Should have one prerequisite");
            assert_eq!(
                prerequisites[0].get("name").and_then(|n| n.as_str()),
                Some(prereq_name.as_str())
            );
        } else {
            panic!("Advanced concept should have prerequisites");
        }
    } else {
        panic!("Should find advanced concept");
    }
    
    // Verify reverse relationship (enabledBy)
    if let Some(prereq_concepts) = result.get("prereq").and_then(|p| p.as_array()) {
        let prereq_concept = &prereq_concepts[0];
        assert_eq!(
            prereq_concept.get("name").and_then(|n| n.as_str()),
            Some(prereq_name.as_str())
        );
        
        if let Some(enabled_by) = prereq_concept.get("enabledBy").and_then(|e| e.as_array()) {
            assert_eq!(enabled_by.len(), 1, "Should enable one concept");
            assert_eq!(
                enabled_by[0].get("name").and_then(|n| n.as_str()),
                Some(advanced_name.as_str())
            );
        } else {
            panic!("Prerequisite should enable other concepts");
        }
    } else {
        panic!("Should find prerequisite concept");
    }
}

#[tokio::test]
#[ignore]
async fn test_update_relationships() {
    wait_for_dgraph().await.expect("Dgraph should be ready");
    
    let config = create_test_config();
    let client = DgraphClient::new(config.hosts[0].clone()).await
        .expect("Should connect to Dgraph");
    
    let concept1_name = format!("Concept 1 {}", Uuid::new_v4());
    let concept2_name = format!("Concept 2 {}", Uuid::new_v4());
    let concept3_name = format!("Concept 3 {}", Uuid::new_v4());
    let now = Utc::now().to_rfc3339();
    
    // Create three concepts
    let create_mutation = json!({
        "set": [
            {
                "uid": "_:concept1",
                "dgraph.type": "TestConcept",
                "name": concept1_name,
                "description": "First concept",
                "difficulty": "beginner",
                "category": "testing",
                "qualityScore": 0.8,
                "createdAt": now,
                "updatedAt": now,
                "version": 1
            },
            {
                "uid": "_:concept2",
                "dgraph.type": "TestConcept",
                "name": concept2_name,
                "description": "Second concept",
                "difficulty": "intermediate",
                "category": "testing",
                "qualityScore": 0.8,
                "createdAt": now,
                "updatedAt": now,
                "version": 1
            },
            {
                "uid": "_:concept3",
                "dgraph.type": "TestConcept",
                "name": concept3_name,
                "description": "Third concept",
                "difficulty": "advanced",
                "category": "testing",
                "qualityScore": 0.8,
                "createdAt": now,
                "updatedAt": now,
                "version": 1
            }
        ]
    });
    
    let create_result = client.mutate(&create_mutation.to_string(), true).await
        .expect("Should create concepts");
    
    let uid1 = extract_uid_from_result(&create_result, "concept1").expect("Should get UID 1");
    let uid2 = extract_uid_from_result(&create_result, "concept2").expect("Should get UID 2");
    let uid3 = extract_uid_from_result(&create_result, "concept3").expect("Should get UID 3");
    
    // Add prerequisite relationship: concept3 requires concept1
    let add_relationship_mutation = json!({
        "set": [{
            "uid": uid3,
            "prerequisites": [{"uid": uid1}]
        }]
    });
    
    client.mutate(&add_relationship_mutation.to_string(), true).await
        .expect("Should add relationship");
    
    // Update relationship: concept3 now requires concept2 instead of concept1
    let update_relationship_mutation = json!({
        "delete": [{
            "uid": uid3,
            "prerequisites": [{"uid": uid1}]
        }],
        "set": [{
            "uid": uid3,
            "prerequisites": [{"uid": uid2}]
        }]
    });
    
    client.mutate(&update_relationship_mutation.to_string(), true).await
        .expect("Should update relationship");
    
    // Verify the relationship was updated correctly
    let query = format!(
        r#"
        {{
            concept3(func: uid("{}")) {{
                uid
                name
                prerequisites {{
                    uid
                    name
                }}
            }}
        }}
        "#,
        uid3
    );
    
    let result = client.query(&query).await
        .expect("Should query updated relationship");
    
    if let Some(concepts) = result.get("concept3").and_then(|c| c.as_array()) {
        let concept = &concepts[0];
        
        if let Some(prerequisites) = concept.get("prerequisites").and_then(|p| p.as_array()) {
            assert_eq!(prerequisites.len(), 1, "Should have exactly one prerequisite");
            assert_eq!(
                prerequisites[0].get("name").and_then(|n| n.as_str()),
                Some(concept2_name.as_str()),
                "Should require concept2, not concept1"
            );
        } else {
            panic!("Concept3 should have prerequisites");
        }
    } else {
        panic!("Should find concept3");
    }
}

#[tokio::test]
#[ignore]
async fn test_delete_with_relationships() {
    wait_for_dgraph().await.expect("Dgraph should be ready");
    
    let config = create_test_config();
    let client = DgraphClient::new(config.hosts[0].clone()).await
        .expect("Should connect to Dgraph");
    
    let parent_name = format!("Parent Concept {}", Uuid::new_v4());
    let child_name = format!("Child Concept {}", Uuid::new_v4());
    let now = Utc::now().to_rfc3339();
    
    // Create parent and child concepts with relationship
    let create_mutation = json!({
        "set": [
            {
                "uid": "_:parent",
                "dgraph.type": "TestConcept",
                "name": parent_name,
                "description": "Parent concept",
                "difficulty": "intermediate",
                "category": "testing",
                "qualityScore": 0.8,
                "createdAt": now,
                "updatedAt": now,
                "version": 1
            },
            {
                "uid": "_:child",
                "dgraph.type": "TestConcept",
                "name": child_name,
                "description": "Child concept",
                "difficulty": "advanced",
                "category": "testing",
                "qualityScore": 0.8,
                "prerequisites": [{"uid": "_:parent"}],
                "createdAt": now,
                "updatedAt": now,
                "version": 1
            }
        ]
    });
    
    let create_result = client.mutate(&create_mutation.to_string(), true).await
        .expect("Should create parent and child concepts");
    
    let parent_uid = extract_uid_from_result(&create_result, "parent").expect("Should get parent UID");
    let child_uid = extract_uid_from_result(&create_result, "child").expect("Should get child UID");
    
    // Verify relationship exists
    let verify_query = format!(
        r#"
        {{
            child(func: uid("{}")) {{
                uid
                name
                prerequisites {{
                    uid
                    name
                }}
            }}
        }}
        "#,
        child_uid
    );
    
    let verify_result = client.query(&verify_query).await
        .expect("Should verify initial relationship");
    
    // Ensure relationship exists before deletion
    if let Some(children) = verify_result.get("child").and_then(|c| c.as_array()) {
        let child = &children[0];
        if let Some(prerequisites) = child.get("prerequisites").and_then(|p| p.as_array()) {
            assert_eq!(prerequisites.len(), 1, "Should have prerequisite relationship");
        } else {
            panic!("Child should have prerequisite before deletion");
        }
    }
    
    // Delete parent concept
    let delete_mutation = json!({
        "delete": [{
            "uid": parent_uid
        }]
    });
    
    client.mutate(&delete_mutation.to_string(), true).await
        .expect("Should delete parent concept");
    
    // Verify parent is deleted and relationship is cleaned up
    let check_query = format!(
        r#"
        {{
            parent(func: uid("{}")) {{
                uid
                name
            }}
            child(func: uid("{}")) {{
                uid
                name
                prerequisites {{
                    uid
                    name
                }}
            }}
        }}
        "#,
        parent_uid, child_uid
    );
    
    let check_result = client.query(&check_query).await
        .expect("Should check after deletion");
    
    // Parent should be gone
    if let Some(parents) = check_result.get("parent").and_then(|p| p.as_array()) {
        assert!(parents.is_empty(), "Parent should be deleted");
    }
    
    // Child should still exist but without the prerequisite relationship
    if let Some(children) = check_result.get("child").and_then(|c| c.as_array()) {
        assert_eq!(children.len(), 1, "Child should still exist");
        
        let child = &children[0];
        assert_eq!(
            child.get("name").and_then(|n| n.as_str()),
            Some(child_name.as_str())
        );
        
        // Prerequisites should be empty or non-existent
        if let Some(prerequisites) = child.get("prerequisites") {
            if let Some(prereq_array) = prerequisites.as_array() {
                assert!(prereq_array.is_empty(), "Prerequisites should be empty after parent deletion");
            }
        }
        // If prerequisites field doesn't exist, that's also acceptable
    } else {
        panic!("Child concept should still exist");
    }
}

#[tokio::test]
#[ignore]
async fn test_batch_mutations() {
    wait_for_dgraph().await.expect("Dgraph should be ready");
    
    let config = create_test_config();
    let client = DgraphClient::new(config.hosts[0].clone()).await
        .expect("Should connect to Dgraph");
    
    let batch_prefix = format!("Batch Test {}", Uuid::new_v4());
    let now = Utc::now().to_rfc3339();
    
    // Create multiple concepts in a single mutation
    let batch_mutation = json!({
        "set": [
            {
                "uid": "_:batch1",
                "dgraph.type": "TestConcept",
                "name": format!("{} Concept 1", batch_prefix),
                "description": "First batch concept",
                "difficulty": "beginner",
                "category": "batch_testing",
                "qualityScore": 0.7,
                "createdAt": now,
                "updatedAt": now,
                "version": 1
            },
            {
                "uid": "_:batch2",
                "dgraph.type": "TestConcept",
                "name": format!("{} Concept 2", batch_prefix),
                "description": "Second batch concept",
                "difficulty": "intermediate",
                "category": "batch_testing",
                "qualityScore": 0.8,
                "prerequisites": [{"uid": "_:batch1"}],
                "createdAt": now,
                "updatedAt": now,
                "version": 1
            },
            {
                "uid": "_:batch3",
                "dgraph.type": "TestConcept",
                "name": format!("{} Concept 3", batch_prefix),
                "description": "Third batch concept",
                "difficulty": "advanced",
                "category": "batch_testing",
                "qualityScore": 0.9,
                "prerequisites": [{"uid": "_:batch2"}],
                "relatedTo": [{"uid": "_:batch1"}],
                "createdAt": now,
                "updatedAt": now,
                "version": 1
            }
        ]
    });
    
    let batch_result = client.mutate(&batch_mutation.to_string(), true).await
        .expect("Should create batch concepts");
    
    let uid1 = extract_uid_from_result(&batch_result, "batch1").expect("Should get batch1 UID");
    let uid2 = extract_uid_from_result(&batch_result, "batch2").expect("Should get batch2 UID");
    let uid3 = extract_uid_from_result(&batch_result, "batch3").expect("Should get batch3 UID");
    
    // Verify all concepts were created with correct relationships
    let verify_query = r#"
    {
        batch_concepts(func: type(TestConcept)) @filter(eq(category, "batch_testing")) {
            uid
            name
            difficulty
            prerequisites {
                uid
                name
            }
            relatedTo {
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
    
    let verify_result = client.query(verify_query).await
        .expect("Should verify batch concepts");
    
    if let Some(concepts) = verify_result.get("batch_concepts").and_then(|c| c.as_array()) {
        assert_eq!(concepts.len(), 3, "Should have created 3 concepts");
        
        // Find each concept and verify relationships
        let concept1 = concepts.iter().find(|c| 
            c.get("name").and_then(|n| n.as_str()).unwrap_or("").contains("Concept 1")
        ).expect("Should find concept 1");
        
        let concept2 = concepts.iter().find(|c| 
            c.get("name").and_then(|n| n.as_str()).unwrap_or("").contains("Concept 2")
        ).expect("Should find concept 2");
        
        let concept3 = concepts.iter().find(|c| 
            c.get("name").and_then(|n| n.as_str()).unwrap_or("").contains("Concept 3")
        ).expect("Should find concept 3");
        
        // Verify concept 1 has no prerequisites but enables concept 2
        if let Some(enabled_by) = concept1.get("enabledBy").and_then(|e| e.as_array()) {
            assert!(enabled_by.len() > 0, "Concept 1 should enable other concepts");
        }
        
        // Verify concept 2 has concept 1 as prerequisite and enables concept 3
        if let Some(prerequisites) = concept2.get("prerequisites").and_then(|p| p.as_array()) {
            assert_eq!(prerequisites.len(), 1, "Concept 2 should have 1 prerequisite");
        }
        
        // Verify concept 3 has concept 2 as prerequisite and concept 1 as related
        if let Some(prerequisites) = concept3.get("prerequisites").and_then(|p| p.as_array()) {
            assert_eq!(prerequisites.len(), 1, "Concept 3 should have 1 prerequisite");
        }
        
        if let Some(related_to) = concept3.get("relatedTo").and_then(|r| r.as_array()) {
            assert_eq!(related_to.len(), 1, "Concept 3 should have 1 related concept");
        }
    } else {
        panic!("Should find batch concepts");
    }
}

#[tokio::test]
#[ignore]
async fn test_mutation_with_resources() {
    wait_for_dgraph().await.expect("Dgraph should be ready");
    
    let config = create_test_config();
    let client = DgraphClient::new(config.hosts[0].clone()).await
        .expect("Should connect to Dgraph");
    
    let concept_name = format!("Concept with Resources {}", Uuid::new_v4());
    let resource_title = format!("Resource for Concept {}", Uuid::new_v4());
    let now = Utc::now().to_rfc3339();
    
    // Create concept with associated resource
    let mutation = json!({
        "set": [
            {
                "uid": "_:concept",
                "dgraph.type": "TestConcept",
                "name": concept_name,
                "description": "Concept with learning resources",
                "difficulty": "intermediate",
                "category": "testing",
                "qualityScore": 0.85,
                "resources": [{"uid": "_:resource"}],
                "createdAt": now,
                "updatedAt": now,
                "version": 1
            },
            {
                "uid": "_:resource",
                "dgraph.type": "TestResource",
                "url": "https://test.example.com/resource",
                "title": resource_title,
                "resourceType": "tutorial",
                "quality": 0.9,
                "difficulty": "intermediate",
                "duration": 45,
                "language": "en",
                "concepts": [{"uid": "_:concept"}],
                "createdAt": now,
                "updatedAt": now
            }
        ]
    });
    
    let result = client.mutate(&mutation.to_string(), true).await
        .expect("Should create concept with resource");
    
    let concept_uid = extract_uid_from_result(&result, "concept").expect("Should get concept UID");
    let resource_uid = extract_uid_from_result(&result, "resource").expect("Should get resource UID");
    
    // Verify the concept-resource relationship
    let query = format!(
        r#"
        {{
            concept(func: uid("{}")) {{
                uid
                name
                resources {{
                    uid
                    title
                    resourceType
                    quality
                }}
            }}
            resource(func: uid("{}")) {{
                uid
                title
                concepts {{
                    uid
                    name
                }}
            }}
        }}
        "#,
        concept_uid, resource_uid
    );
    
    let query_result = client.query(&query).await
        .expect("Should query concept-resource relationship");
    
    // Verify forward relationship (concept -> resources)
    if let Some(concepts) = query_result.get("concept").and_then(|c| c.as_array()) {
        let concept = &concepts[0];
        assert_eq!(
            concept.get("name").and_then(|n| n.as_str()),
            Some(concept_name.as_str())
        );
        
        if let Some(resources) = concept.get("resources").and_then(|r| r.as_array()) {
            assert_eq!(resources.len(), 1, "Should have one resource");
            assert_eq!(
                resources[0].get("title").and_then(|t| t.as_str()),
                Some(resource_title.as_str())
            );
            assert_eq!(
                resources[0].get("resourceType").and_then(|rt| rt.as_str()),
                Some("tutorial")
            );
        } else {
            panic!("Concept should have resources");
        }
    } else {
        panic!("Should find concept");
    }
    
    // Verify reverse relationship (resource -> concepts)
    if let Some(resources) = query_result.get("resource").and_then(|r| r.as_array()) {
        let resource = &resources[0];
        assert_eq!(
            resource.get("title").and_then(|t| t.as_str()),
            Some(resource_title.as_str())
        );
        
        if let Some(concepts) = resource.get("concepts").and_then(|c| c.as_array()) {
            assert_eq!(concepts.len(), 1, "Resource should be associated with one concept");
            assert_eq!(
                concepts[0].get("name").and_then(|n| n.as_str()),
                Some(concept_name.as_str())
            );
        } else {
            panic!("Resource should be associated with concepts");
        }
    } else {
        panic!("Should find resource");
    }
}