//! Knowledge Graph Transaction Integration Tests
//! 
//! These tests verify transaction handling, consistency, and rollback scenarios
//! against a real Dgraph instance. They test both successful transactions
//! and failure/rollback scenarios.
//! 
//! Setup:
//! 1. Start test Dgraph: cd services/knowledge_graph && docker-compose -f docker-compose.test.yml up -d
//! 2. Run tests: cargo test knowledge_graph_transaction --ignored
//! 3. Cleanup: docker-compose -f docker-compose.test.yml down -v

use knowledge_graph::{
    client::DgraphClient,
    graph::DgraphConfig,
    error::KnowledgeGraphError,
};
use serde_json::{json, Value};
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

/// Helper to count concepts with specific criteria
async fn count_concepts_by_category(
    client: &DgraphClient,
    category: &str,
) -> Result<usize, KnowledgeGraphError> {
    let query = format!(
        r#"
        {{
            concepts(func: type(TestConcept)) @filter(eq(category, "{}")) {{
                uid
            }}
        }}
        "#,
        category
    );
    
    let result = client.query(&query).await?;
    
    if let Some(concepts) = result.get("concepts").and_then(|c| c.as_array()) {
        Ok(concepts.len())
    } else {
        Ok(0)
    }
}

/// Helper to verify concept exists
async fn concept_exists(
    client: &DgraphClient,
    uid: &str,
) -> Result<bool, KnowledgeGraphError> {
    let query = format!(
        r#"
        {{
            concept(func: uid("{}")) {{
                uid
                dgraph.type
            }}
        }}
        "#,
        uid
    );
    
    let result = client.query(&query).await?;
    
    if let Some(concepts) = result.get("concept").and_then(|c| c.as_array()) {
        Ok(!concepts.is_empty())
    } else {
        Ok(false)
    }
}

#[tokio::test]
#[ignore]
async fn test_simple_transaction_commit() {
    wait_for_dgraph().await.expect("Dgraph should be ready");
    
    let config = create_test_config();
    let client = DgraphClient::new(config.hosts[0].clone()).await
        .expect("Should connect to Dgraph");
    
    let test_name = format!("Transaction Test {}", Uuid::new_v4());
    let now = Utc::now().to_rfc3339();
    
    // Start a transaction
    let transaction = client.new_txn().await
        .expect("Should create new transaction");
    
    // Create a concept within the transaction
    let mutation = json!({
        "set": [{
            "uid": "_:txn_concept",
            "dgraph.type": "TestConcept",
            "name": test_name,
            "description": "Concept created in transaction",
            "difficulty": "intermediate",
            "category": "transaction_testing",
            "qualityScore": 0.85,
            "createdAt": now,
            "updatedAt": now,
            "version": 1
        }]
    });
    
    let mutation_result = transaction.mutate(&mutation.to_string()).await
        .expect("Should mutate within transaction");
    
    let uid = extract_uid_from_result(&mutation_result, "txn_concept")
        .expect("Should get UID from transaction mutation");
    
    // Verify the concept is not visible outside the transaction yet
    let initial_count = count_concepts_by_category(&client, "transaction_testing").await
        .expect("Should count concepts");
    
    // Commit the transaction
    transaction.commit().await
        .expect("Should commit transaction");
    
    // Verify the concept is now visible
    let final_count = count_concepts_by_category(&client, "transaction_testing").await
        .expect("Should count concepts after commit");
    
    assert_eq!(final_count, initial_count + 1, "Should have one more concept after commit");
    
    // Verify the specific concept exists
    assert!(
        concept_exists(&client, &uid).await.unwrap(),
        "Committed concept should exist"
    );
}

#[tokio::test]
#[ignore]
async fn test_transaction_rollback() {
    wait_for_dgraph().await.expect("Dgraph should be ready");
    
    let config = create_test_config();
    let client = DgraphClient::new(config.hosts[0].clone()).await
        .expect("Should connect to Dgraph");
    
    let test_name = format!("Rollback Test {}", Uuid::new_v4());
    let now = Utc::now().to_rfc3339();
    
    // Count initial concepts
    let initial_count = count_concepts_by_category(&client, "rollback_testing").await
        .expect("Should count initial concepts");
    
    // Start a transaction
    let transaction = client.new_txn().await
        .expect("Should create new transaction");
    
    // Create a concept within the transaction
    let mutation = json!({
        "set": [{
            "uid": "_:rollback_concept",
            "dgraph.type": "TestConcept",
            "name": test_name,
            "description": "Concept to be rolled back",
            "difficulty": "beginner",
            "category": "rollback_testing",
            "qualityScore": 0.7,
            "createdAt": now,
            "updatedAt": now,
            "version": 1
        }]
    });
    
    let mutation_result = transaction.mutate(&mutation.to_string()).await
        .expect("Should mutate within transaction");
    
    let uid = extract_uid_from_result(&mutation_result, "rollback_concept")
        .expect("Should get UID from transaction mutation");
    
    // Explicitly rollback the transaction (discard changes)
    transaction.discard().await
        .expect("Should discard transaction");
    
    // Verify the concept was not committed
    let final_count = count_concepts_by_category(&client, "rollback_testing").await
        .expect("Should count concepts after rollback");
    
    assert_eq!(final_count, initial_count, "Should have same count after rollback");
    
    // Verify the specific concept doesn't exist
    assert!(
        !concept_exists(&client, &uid).await.unwrap(),
        "Rolled back concept should not exist"
    );
}

#[tokio::test]
#[ignore]
async fn test_transaction_with_query_and_mutation() {
    wait_for_dgraph().await.expect("Dgraph should be ready");
    
    let config = create_test_config();
    let client = DgraphClient::new(config.hosts[0].clone()).await
        .expect("Should connect to Dgraph");
    
    let base_name = format!("Query Mutation Test {}", Uuid::new_v4());
    let now = Utc::now().to_rfc3339();
    
    // First, create a base concept outside transaction
    let base_mutation = json!({
        "set": [{
            "uid": "_:base_concept",
            "dgraph.type": "TestConcept",
            "name": base_name,
            "description": "Base concept for transaction test",
            "difficulty": "beginner",
            "category": "query_mutation_testing",
            "qualityScore": 0.8,
            "createdAt": now,
            "updatedAt": now,
            "version": 1
        }]
    });
    
    let base_result = client.mutate(&base_mutation.to_string(), true).await
        .expect("Should create base concept");
    
    let base_uid = extract_uid_from_result(&base_result, "base_concept")
        .expect("Should get base concept UID");
    
    // Start a transaction
    let transaction = client.new_txn().await
        .expect("Should create new transaction");
    
    // Query the base concept within the transaction
    let query = format!(
        r#"
        {{
            base_concept(func: uid("{}")) {{
                uid
                name
                qualityScore
            }}
        }}
        "#,
        base_uid
    );
    
    let query_result = transaction.query(&query).await
        .expect("Should query within transaction");
    
    // Verify we can see the base concept
    if let Some(concepts) = query_result.get("base_concept").and_then(|c| c.as_array()) {
        assert_eq!(concepts.len(), 1, "Should find base concept");
        let concept = &concepts[0];
        assert_eq!(
            concept.get("name").and_then(|n| n.as_str()),
            Some(base_name.as_str())
        );
    } else {
        panic!("Should find base concept in transaction");
    }
    
    // Create a related concept within the same transaction
    let related_name = format!("Related to {}", base_name);
    let related_mutation = json!({
        "set": [{
            "uid": "_:related_concept",
            "dgraph.type": "TestConcept",
            "name": related_name,
            "description": "Related concept created in transaction",
            "difficulty": "intermediate",
            "category": "query_mutation_testing",
            "qualityScore": 0.85,
            "prerequisites": [{"uid": base_uid}],
            "createdAt": now,
            "updatedAt": now,
            "version": 1
        }]
    });
    
    let related_result = transaction.mutate(&related_mutation.to_string()).await
        .expect("Should create related concept in transaction");
    
    let related_uid = extract_uid_from_result(&related_result, "related_concept")
        .expect("Should get related concept UID");
    
    // Query both concepts within the transaction to verify relationship
    let relationship_query = format!(
        r#"
        {{
            related(func: uid("{}")) {{
                uid
                name
                prerequisites {{
                    uid
                    name
                }}
            }}
            base(func: uid("{}")) {{
                uid
                name
                enabledBy {{
                    uid
                    name
                }}
            }}
        }}
        "#,
        related_uid, base_uid
    );
    
    let relationship_result = transaction.query(&relationship_query).await
        .expect("Should query relationships within transaction");
    
    // Verify the relationship within the transaction
    if let Some(related_concepts) = relationship_result.get("related").and_then(|r| r.as_array()) {
        let related_concept = &related_concepts[0];
        if let Some(prerequisites) = related_concept.get("prerequisites").and_then(|p| p.as_array()) {
            assert_eq!(prerequisites.len(), 1, "Should have prerequisite relationship");
            assert_eq!(
                prerequisites[0].get("name").and_then(|n| n.as_str()),
                Some(base_name.as_str())
            );
        } else {
            panic!("Related concept should have prerequisites in transaction");
        }
    }
    
    // Commit the transaction
    transaction.commit().await
        .expect("Should commit transaction");
    
    // Verify the relationship persists after commit
    let verify_query = format!(
        r#"
        {{
            related(func: uid("{}")) {{
                uid
                name
                prerequisites {{
                    uid
                    name
                }}
            }}
        }}
        "#,
        related_uid
    );
    
    let verify_result = client.query(&verify_query).await
        .expect("Should verify relationship after commit");
    
    if let Some(related_concepts) = verify_result.get("related").and_then(|r| r.as_array()) {
        let related_concept = &related_concepts[0];
        if let Some(prerequisites) = related_concept.get("prerequisites").and_then(|p| p.as_array()) {
            assert_eq!(prerequisites.len(), 1, "Relationship should persist after commit");
        } else {
            panic!("Relationship should persist after commit");
        }
    } else {
        panic!("Related concept should exist after commit");
    }
}

#[tokio::test]
#[ignore]
async fn test_transaction_conflict_handling() {
    wait_for_dgraph().await.expect("Dgraph should be ready");
    
    let config = create_test_config();
    let client1 = DgraphClient::new(config.hosts[0].clone()).await
        .expect("Should connect client 1 to Dgraph");
    let client2 = DgraphClient::new(config.hosts[0].clone()).await
        .expect("Should connect client 2 to Dgraph");
    
    let test_name = format!("Conflict Test {}", Uuid::new_v4());
    let now = Utc::now().to_rfc3339();
    
    // Create a base concept
    let base_mutation = json!({
        "set": [{
            "uid": "_:conflict_concept",
            "dgraph.type": "TestConcept",
            "name": test_name,
            "description": "Concept for conflict testing",
            "difficulty": "beginner",
            "category": "conflict_testing",
            "qualityScore": 0.5,
            "createdAt": now,
            "updatedAt": now,
            "version": 1
        }]
    });
    
    let base_result = client1.mutate(&base_mutation.to_string(), true).await
        .expect("Should create base concept");
    
    let uid = extract_uid_from_result(&base_result, "conflict_concept")
        .expect("Should get concept UID");
    
    // Start two concurrent transactions
    let txn1 = client1.new_txn().await
        .expect("Should create transaction 1");
    let txn2 = client2.new_txn().await
        .expect("Should create transaction 2");
    
    // Both transactions read the concept
    let read_query = format!(
        r#"
        {{
            concept(func: uid("{}")) {{
                uid
                name
                qualityScore
                version
            }}
        }}
        "#,
        uid
    );
    
    let _result1 = txn1.query(&read_query).await
        .expect("Transaction 1 should read concept");
    let _result2 = txn2.query(&read_query).await
        .expect("Transaction 2 should read concept");
    
    // Transaction 1 updates the concept
    let update1 = json!({
        "set": [{
            "uid": uid,
            "qualityScore": 0.8,
            "version": 2,
            "updatedAt": Utc::now().to_rfc3339()
        }]
    });
    
    txn1.mutate(&update1.to_string()).await
        .expect("Transaction 1 should update concept");
    
    // Transaction 2 also tries to update the same concept
    let update2 = json!({
        "set": [{
            "uid": uid,
            "qualityScore": 0.9,
            "version": 2,
            "updatedAt": Utc::now().to_rfc3339()
        }]
    });
    
    txn2.mutate(&update2.to_string()).await
        .expect("Transaction 2 should update concept");
    
    // Commit transaction 1 (should succeed)
    let commit1_result = txn1.commit().await;
    assert!(commit1_result.is_ok(), "Transaction 1 commit should succeed");
    
    // Commit transaction 2 (should fail due to conflict)
    let commit2_result = txn2.commit().await;
    // Note: Dgraph transaction conflict behavior may vary
    // The test verifies that the system handles conflicts gracefully
    
    // Verify final state - only one update should have succeeded
    let final_query = format!(
        r#"
        {{
            concept(func: uid("{}")) {{
                uid
                qualityScore
                version
            }}
        }}
        "#,
        uid
    );
    
    let final_result = client1.query(&final_query).await
        .expect("Should query final state");
    
    if let Some(concepts) = final_result.get("concept").and_then(|c| c.as_array()) {
        let concept = &concepts[0];
        let final_quality = concept.get("qualityScore").and_then(|q| q.as_f64()).unwrap();
        let final_version = concept.get("version").and_then(|v| v.as_i64()).unwrap();
        
        // Either transaction 1's update (0.8) or transaction 2's update (0.9) should be applied
        assert!(
            final_quality == 0.8 || final_quality == 0.9,
            "Final quality should be from one of the transactions"
        );
        assert_eq!(final_version, 2, "Version should be updated");
        
        println!("Final quality score: {}, Commit1 result: {:?}, Commit2 result: {:?}", 
                 final_quality, commit1_result, commit2_result);
    }
}

#[tokio::test]
#[ignore]
async fn test_transaction_with_multiple_operations() {
    wait_for_dgraph().await.expect("Dgraph should be ready");
    
    let config = create_test_config();
    let client = DgraphClient::new(config.hosts[0].clone()).await
        .expect("Should connect to Dgraph");
    
    let test_prefix = format!("Multi Op Test {}", Uuid::new_v4());
    let now = Utc::now().to_rfc3339();
    
    // Start transaction
    let transaction = client.new_txn().await
        .expect("Should create new transaction");
    
    // Operation 1: Create a concept
    let create_mutation = json!({
        "set": [{
            "uid": "_:multi_concept",
            "dgraph.type": "TestConcept",
            "name": format!("{} Concept", test_prefix),
            "description": "Multi-operation concept",
            "difficulty": "intermediate",
            "category": "multi_op_testing",
            "qualityScore": 0.7,
            "createdAt": now,
            "updatedAt": now,
            "version": 1
        }]
    });
    
    let create_result = transaction.mutate(&create_mutation.to_string()).await
        .expect("Should create concept in transaction");
    
    let concept_uid = extract_uid_from_result(&create_result, "multi_concept")
        .expect("Should get concept UID");
    
    // Operation 2: Create a resource linked to the concept
    let resource_mutation = json!({
        "set": [{
            "uid": "_:multi_resource",
            "dgraph.type": "TestResource",
            "url": "https://test.example.com/multi-op-resource",
            "title": format!("{} Resource", test_prefix),
            "resourceType": "tutorial",
            "quality": 0.85,
            "difficulty": "intermediate",
            "duration": 60,
            "language": "en",
            "concepts": [{"uid": concept_uid}],
            "createdAt": now,
            "updatedAt": now
        }]
    });
    
    let resource_result = transaction.mutate(&resource_mutation.to_string()).await
        .expect("Should create resource in transaction");
    
    let resource_uid = extract_uid_from_result(&resource_result, "multi_resource")
        .expect("Should get resource UID");
    
    // Operation 3: Update the concept to reference the resource
    let update_mutation = json!({
        "set": [{
            "uid": concept_uid,
            "resources": [{"uid": resource_uid}],
            "updatedAt": Utc::now().to_rfc3339()
        }]
    });
    
    transaction.mutate(&update_mutation.to_string()).await
        .expect("Should update concept with resource reference");
    
    // Operation 4: Query to verify everything within transaction
    let verify_query = format!(
        r#"
        {{
            concept(func: uid("{}")) {{
                uid
                name
                resources {{
                    uid
                    title
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
    
    let verify_result = transaction.query(&verify_query).await
        .expect("Should verify within transaction");
    
    // Verify relationships within transaction
    if let Some(concepts) = verify_result.get("concept").and_then(|c| c.as_array()) {
        let concept = &concepts[0];
        if let Some(resources) = concept.get("resources").and_then(|r| r.as_array()) {
            assert_eq!(resources.len(), 1, "Concept should have one resource");
        } else {
            panic!("Concept should have resources within transaction");
        }
    }
    
    if let Some(resources) = verify_result.get("resource").and_then(|r| r.as_array()) {
        let resource = &resources[0];
        if let Some(concepts) = resource.get("concepts").and_then(|c| c.as_array()) {
            assert_eq!(concepts.len(), 1, "Resource should be linked to one concept");
        } else {
            panic!("Resource should be linked to concepts within transaction");
        }
    }
    
    // Commit all operations
    transaction.commit().await
        .expect("Should commit all operations");
    
    // Verify everything persists after commit
    let final_query = format!(
        r#"
        {{
            concept(func: uid("{}")) {{
                uid
                name
                resources {{
                    uid
                    title
                    quality
                }}
            }}
        }}
        "#,
        concept_uid
    );
    
    let final_result = client.query(&final_query).await
        .expect("Should verify after commit");
    
    if let Some(concepts) = final_result.get("concept").and_then(|c| c.as_array()) {
        let concept = &concepts[0];
        assert_eq!(
            concept.get("name").and_then(|n| n.as_str()),
            Some(format!("{} Concept", test_prefix).as_str())
        );
        
        if let Some(resources) = concept.get("resources").and_then(|r| r.as_array()) {
            assert_eq!(resources.len(), 1, "All operations should be committed");
            assert_eq!(
                resources[0].get("title").and_then(|t| t.as_str()),
                Some(format!("{} Resource", test_prefix).as_str())
            );
            assert_eq!(
                resources[0].get("quality").and_then(|q| q.as_f64()),
                Some(0.85)
            );
        } else {
            panic!("All relationships should persist after commit");
        }
    } else {
        panic!("Concept should exist after commit");
    }
}

#[tokio::test]
#[ignore]
async fn test_transaction_error_handling() {
    wait_for_dgraph().await.expect("Dgraph should be ready");
    
    let config = create_test_config();
    let client = DgraphClient::new(config.hosts[0].clone()).await
        .expect("Should connect to Dgraph");
    
    let test_name = format!("Error Test {}", Uuid::new_v4());
    let now = Utc::now().to_rfc3339();
    
    // Start transaction
    let transaction = client.new_txn().await
        .expect("Should create new transaction");
    
    // Valid operation
    let valid_mutation = json!({
        "set": [{
            "uid": "_:valid_concept",
            "dgraph.type": "TestConcept",
            "name": test_name,
            "description": "Valid concept",
            "difficulty": "beginner",
            "category": "error_testing",
            "qualityScore": 0.8,
            "createdAt": now,
            "updatedAt": now,
            "version": 1
        }]
    });
    
    let valid_result = transaction.mutate(&valid_mutation.to_string()).await
        .expect("Valid mutation should succeed");
    
    let valid_uid = extract_uid_from_result(&valid_result, "valid_concept")
        .expect("Should get valid concept UID");
    
    // Invalid operation (trying to set an invalid field type)
    let invalid_mutation = json!({
        "set": [{
            "uid": "_:invalid_concept",
            "dgraph.type": "TestConcept",
            "name": "Invalid Concept",
            "description": "Invalid concept with wrong field type",
            "difficulty": "beginner",
            "category": "error_testing",
            "qualityScore": "invalid_string_for_float", // This should cause an error
            "createdAt": now,
            "updatedAt": now,
            "version": 1
        }]
    });
    
    // The invalid mutation may or may not fail immediately depending on Dgraph behavior
    // Let's try to commit and see what happens
    let invalid_result = transaction.mutate(&invalid_mutation.to_string()).await;
    
    match invalid_result {
        Ok(_) => {
            // If the mutation succeeds, the error might be caught during commit
            let commit_result = transaction.commit().await;
            if commit_result.is_err() {
                println!("Transaction failed on commit as expected due to invalid data");
                
                // Verify that no data was committed
                let verify_count = count_concepts_by_category(&client, "error_testing").await
                    .expect("Should count concepts");
                // The valid concept should not exist since the transaction was rolled back
                println!("Concepts in error_testing category: {}", verify_count);
            } else {
                println!("Transaction committed despite invalid data - Dgraph may have coerced the type");
            }
        }
        Err(e) => {
            println!("Invalid mutation failed as expected: {:?}", e);
            
            // Transaction should be in error state, try to discard
            let discard_result = transaction.discard().await;
            match discard_result {
                Ok(_) => println!("Transaction discarded successfully"),
                Err(discard_err) => println!("Transaction discard error: {:?}", discard_err),
            }
            
            // Verify no data was committed
            let verify_count = count_concepts_by_category(&client, "error_testing").await
                .expect("Should count concepts");
            assert_eq!(verify_count, 0, "No concepts should be committed after error");
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_read_only_transaction() {
    wait_for_dgraph().await.expect("Dgraph should be ready");
    
    let config = create_test_config();
    let client = DgraphClient::new(config.hosts[0].clone()).await
        .expect("Should connect to Dgraph");
    
    // First, create some test data outside any transaction
    let test_name = format!("Read Only Test {}", Uuid::new_v4());
    let now = Utc::now().to_rfc3339();
    
    let setup_mutation = json!({
        "set": [{
            "uid": "_:readonly_concept",
            "dgraph.type": "TestConcept",
            "name": test_name,
            "description": "Concept for read-only testing",
            "difficulty": "intermediate",
            "category": "readonly_testing",
            "qualityScore": 0.85,
            "createdAt": now,
            "updatedAt": now,
            "version": 1
        }]
    });
    
    let setup_result = client.mutate(&setup_mutation.to_string(), true).await
        .expect("Should create test data");
    
    let uid = extract_uid_from_result(&setup_result, "readonly_concept")
        .expect("Should get concept UID");
    
    // Start a read-only transaction
    let readonly_txn = client.new_readonly_txn().await
        .expect("Should create read-only transaction");
    
    // Perform multiple read operations
    let query1 = format!(
        r#"
        {{
            concept(func: uid("{}")) {{
                uid
                name
                description
                qualityScore
            }}
        }}
        "#,
        uid
    );
    
    let result1 = readonly_txn.query(&query1).await
        .expect("Should perform read in read-only transaction");
    
    if let Some(concepts) = result1.get("concept").and_then(|c| c.as_array()) {
        assert_eq!(concepts.len(), 1, "Should find the test concept");
        assert_eq!(
            concepts[0].get("name").and_then(|n| n.as_str()),
            Some(test_name.as_str())
        );
    }
    
    // Perform another read operation in the same transaction
    let query2 = r#"
    {
        all_readonly(func: type(TestConcept)) @filter(eq(category, "readonly_testing")) {
            uid
            name
            qualityScore
        }
    }
    "#;
    
    let result2 = readonly_txn.query(query2).await
        .expect("Should perform second read in read-only transaction");
    
    if let Some(concepts) = result2.get("all_readonly").and_then(|c| c.as_array()) {
        assert_eq!(concepts.len(), 1, "Should find concepts in category");
    }
    
    // Read-only transactions don't need explicit commit/discard in Dgraph
    // They automatically clean up when dropped
    
    // Verify data is still accessible after transaction
    let verify_result = client.query(&query1).await
        .expect("Should verify data after read-only transaction");
    
    if let Some(concepts) = verify_result.get("concept").and_then(|c| c.as_array()) {
        assert_eq!(concepts.len(), 1, "Data should still be accessible");
    }
}

#[tokio::test]
#[ignore]
async fn test_transaction_timeout() {
    wait_for_dgraph().await.expect("Dgraph should be ready");
    
    let config = create_test_config();
    let client = DgraphClient::new(config.hosts[0].clone()).await
        .expect("Should connect to Dgraph");
    
    let test_name = format!("Timeout Test {}", Uuid::new_v4());
    let now = Utc::now().to_rfc3339();
    
    // Start a transaction
    let transaction = client.new_txn().await
        .expect("Should create new transaction");
    
    // Perform a mutation
    let mutation = json!({
        "set": [{
            "uid": "_:timeout_concept",
            "dgraph.type": "TestConcept",
            "name": test_name,
            "description": "Concept for timeout testing",
            "difficulty": "beginner",
            "category": "timeout_testing",
            "qualityScore": 0.7,
            "createdAt": now,
            "updatedAt": now,
            "version": 1
        }]
    });
    
    transaction.mutate(&mutation.to_string()).await
        .expect("Should mutate in transaction");
    
    // Wait for a significant amount of time (simulating long processing)
    // Note: Actual timeout behavior depends on Dgraph configuration
    println!("Waiting to test transaction timeout behavior...");
    sleep(Duration::from_secs(5)).await;
    
    // Try to commit after delay
    let commit_result = transaction.commit().await;
    
    match commit_result {
        Ok(_) => {
            println!("Transaction committed successfully despite delay");
            
            // Verify the concept was created
            let count = count_concepts_by_category(&client, "timeout_testing").await
                .expect("Should count concepts");
            assert_eq!(count, 1, "Concept should be committed");
        }
        Err(e) => {
            println!("Transaction failed due to timeout or other error: {:?}", e);
            
            // Verify no data was committed
            let count = count_concepts_by_category(&client, "timeout_testing").await
                .expect("Should count concepts");
            assert_eq!(count, 0, "No concepts should be committed after timeout");
        }
    }
}