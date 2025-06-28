#![cfg(feature = "integration")]

use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
#[ignore]
async fn test_federated_query_across_services() {
    // Start all services and gateway
    // This assumes services are running via docker-compose
    
    let client = Client::new();
    let gateway_url = "http://localhost:4000/graphql";
    
    // Wait for services to be ready
    wait_for_services(&client).await;
    
    // Execute a federated query that spans multiple services
    let query = r#"
        query FederatedWorkflowQuery($workflowId: ID!) {
            workflow(id: $workflowId) {
                id
                name
                status
                nodes {
                    id
                    type
                    config
                }
                # From content-processing service
                processedContent {
                    id
                    summary
                    extractedEntities
                    metadata {
                        processedAt
                        processingTime
                    }
                }
                # From knowledge-graph service
                relatedKnowledge {
                    concepts
                    relationships {
                        from
                        to
                        type
                    }
                }
                # From realtime-communication service
                activeCollaborators {
                    userId
                    presence
                    lastActivity
                }
            }
        }
    "#;
    
    let variables = json!({
        "workflowId": "test-workflow-123"
    });
    
    let response = client
        .post(gateway_url)
        .json(&json!({
            "query": query,
            "variables": variables
        }))
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    
    // Verify data from all services is present
    assert!(body["data"]["workflow"].is_object());
    assert!(body["data"]["workflow"]["processedContent"].is_object());
    assert!(body["data"]["workflow"]["relatedKnowledge"].is_object());
    assert!(body["data"]["workflow"]["activeCollaborators"].is_array());
}

#[tokio::test]
#[ignore]
async fn test_federation_entity_resolution() {
    let client = Client::new();
    let gateway_url = "http://localhost:4000/graphql";
    
    wait_for_services(&client).await;
    
    // Test entity resolution across services
    let query = r#"
        query ResolveEntities($representations: [_Any!]!) {
            _entities(representations: $representations) {
                ... on Workflow {
                    id
                    name
                    status
                    nodes {
                        id
                        type
                    }
                }
                ... on ProcessedContent {
                    id
                    workflowId
                    summary
                    extractedEntities
                }
                ... on KnowledgeNode {
                    id
                    concept
                    relatedWorkflows
                }
            }
        }
    "#;
    
    let representations = json!([
        {
            "__typename": "Workflow",
            "id": "workflow-1"
        },
        {
            "__typename": "ProcessedContent",
            "id": "content-1"
        },
        {
            "__typename": "KnowledgeNode",
            "id": "knowledge-1"
        }
    ]);
    
    let response = client
        .post(gateway_url)
        .json(&json!({
            "query": query,
            "variables": { "representations": representations }
        }))
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    
    // Verify entities from different services are resolved
    let entities = &body["data"]["_entities"];
    assert!(entities.is_array());
    assert_eq!(entities.as_array().unwrap().len(), 3);
}

#[tokio::test]
#[ignore]
async fn test_federation_partial_failure_handling() {
    let client = Client::new();
    let gateway_url = "http://localhost:4000/graphql";
    
    wait_for_services(&client).await;
    
    // Simulate a query where one service might fail
    // but others should still return data
    let query = r#"
        query ResilientWorkflowQuery($workflowId: ID!) {
            workflow(id: $workflowId) {
                id
                name
                # This should always work (from main API)
                nodes {
                    id
                    type
                }
                # These might fail if services are down
                processedContent {
                    summary
                }
                relatedKnowledge {
                    concepts
                }
            }
        }
    "#;
    
    let response = client
        .post(gateway_url)
        .json(&json!({
            "query": query,
            "variables": { "workflowId": "test-123" }
        }))
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    
    // Should always have workflow data even if federated fields fail
    assert!(body["data"]["workflow"]["id"].is_string());
    assert!(body["data"]["workflow"]["nodes"].is_array());
    
    // Check if there are any errors for failed services
    if body["errors"].is_array() {
        let errors = body["errors"].as_array().unwrap();
        for error in errors {
            // Verify error structure for service failures
            assert!(error["extensions"]["service"].is_string());
            assert!(error["extensions"]["code"].is_string());
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_federation_subscription_support() {
    use futures_util::StreamExt;
    use tokio_tungstenite::connect_async;
    
    let ws_url = "ws://localhost:4000/graphql";
    
    // Connect to WebSocket for subscriptions
    let (ws_stream, _) = connect_async(ws_url)
        .await
        .expect("Failed to connect to WebSocket");
    
    let (mut write, mut read) = ws_stream.split();
    
    // Send subscription
    let subscription = json!({
        "id": "sub-1",
        "type": "start",
        "payload": {
            "query": r#"
                subscription OnWorkflowUpdate($workflowId: ID!) {
                    workflowUpdated(id: $workflowId) {
                        id
                        status
                        lastModified
                        # Real-time data from realtime-communication service
                        activeUsers {
                            userId
                            presence
                        }
                    }
                }
            "#,
            "variables": { "workflowId": "workflow-123" }
        }
    });
    
    use tokio_tungstenite::tungstenite::Message;
    write.send(Message::Text(subscription.to_string()))
        .await
        .expect("Failed to send subscription");
    
    // Wait for subscription acknowledgment
    if let Some(Ok(Message::Text(msg))) = read.next().await {
        let response: serde_json::Value = serde_json::from_str(&msg)
            .expect("Failed to parse subscription response");
        
        assert_eq!(response["type"], "ack");
        assert_eq!(response["id"], "sub-1");
    }
    
    // Simulate waiting for updates
    sleep(Duration::from_secs(1)).await;
    
    // Clean up subscription
    let stop = json!({
        "id": "sub-1",
        "type": "stop"
    });
    
    write.send(Message::Text(stop.to_string()))
        .await
        .expect("Failed to stop subscription");
}

async fn wait_for_services(client: &Client) {
    println!("Waiting for services to be ready...");
    
    let health_endpoints = vec![
        ("Gateway", "http://localhost:4000/health"),
        ("Workflow API", "http://localhost:8080/health"),
        ("Content Processing", "http://localhost:8082/health"),
        ("Knowledge Graph", "http://localhost:3002/health"),
        ("Realtime Communication", "http://localhost:8081/health"),
    ];
    
    for (name, url) in health_endpoints {
        let mut retries = 30; // 30 seconds timeout
        loop {
            match client.get(url).send().await {
                Ok(response) if response.status().is_success() => {
                    println!("✓ {} is ready", name);
                    break;
                }
                _ => {
                    if retries == 0 {
                        panic!("{} failed to start", name);
                    }
                    retries -= 1;
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }
    
    println!("All services are ready!");
}