//! Example demonstrating federated GraphQL queries through the gateway
//! 
//! Run with: cargo run --example federated_query --bin workflow-engine-gateway

use reqwest;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("GraphQL Federation Example");
    println!("==========================\n");
    
    // Gateway URL
    let gateway_url = "http://localhost:4000/graphql";
    
    // Example 1: Simple health check
    println!("1. Health Check Query:");
    let health_query = json!({
        "query": "{ health }"
    });
    
    let client = reqwest::Client::new();
    let response = client
        .post(gateway_url)
        .json(&health_query)
        .send()
        .await?;
    
    let result: serde_json::Value = response.json().await?;
    println!("Response: {}\n", serde_json::to_string_pretty(&result)?);
    
    // Example 2: Federated workflow query
    println!("2. Federated Workflow Query:");
    let workflow_query = json!({
        "query": r#"
            query GetWorkflowDetails {
                workflow(id: "123") {
                    id
                    name
                    status
                }
                
                workflows(limit: 5) {
                    items {
                        id
                        name
                        status
                        createdAt
                    }
                    totalCount
                }
            }
        "#
    });
    
    match client.post(gateway_url).json(&workflow_query).send().await {
        Ok(response) => {
            let result: serde_json::Value = response.json().await?;
            println!("Response: {}\n", serde_json::to_string_pretty(&result)?);
        }
        Err(e) => {
            println!("Note: This requires the workflow subgraph to be running at http://localhost:8080/graphql");
            println!("Error: {}\n", e);
        }
    }
    
    // Example 3: Mutation
    println!("3. Create Workflow Mutation:");
    let mutation = json!({
        "query": r#"
            mutation CreateNewWorkflow {
                createWorkflow(name: "Test Workflow", description: "Created via GraphQL") {
                    id
                    name
                    status
                    createdAt
                }
            }
        "#
    });
    
    match client.post(gateway_url).json(&mutation).send().await {
        Ok(response) => {
            let result: serde_json::Value = response.json().await?;
            println!("Response: {}\n", serde_json::to_string_pretty(&result)?);
        }
        Err(e) => {
            println!("Error: {}\n", e);
        }
    }
    
    // Example 4: Subscription (simulated)
    println!("4. Subscription Example (using regular query for demo):");
    let subscription_query = json!({
        "query": r#"
            subscription WatchWorkflow {
                workflowStatusChanged(workflowId: "123")
            }
        "#
    });
    
    println!("Note: Subscriptions require WebSocket connection.");
    println!("Query that would be used: {}", serde_json::to_string_pretty(&subscription_query)?);
    
    Ok(())
}