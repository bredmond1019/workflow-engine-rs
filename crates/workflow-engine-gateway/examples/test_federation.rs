//! Test the federation capabilities
//! 
//! Run with: cargo run --example test_federation

use reqwest;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing GraphQL Federation");
    println!("=========================\n");
    
    let gateway_url = "http://localhost:4000/graphql";
    let workflow_url = "http://localhost:8080/api/v1/graphql";
    
    let client = reqwest::Client::new();
    
    // Test 1: Query _service from workflow subgraph
    println!("1. Testing _service query on workflow subgraph:");
    let service_query = json!({
        "query": "{ _service { sdl } }"
    });
    
    match client.post(workflow_url).json(&service_query).send().await {
        Ok(response) => {
            let result: serde_json::Value = response.json().await?;
            if let Some(sdl) = result["data"]["_service"]["sdl"].as_str() {
                println!("SDL received from workflow subgraph:");
                println!("{}\n", sdl.lines().take(10).collect::<Vec<_>>().join("\n"));
                println!("... (truncated)\n");
            }
        }
        Err(e) => {
            println!("Note: Workflow API needs to be running at {}", workflow_url);
            println!("Error: {}\n", e);
        }
    }
    
    // Test 2: Query _entities from workflow subgraph
    println!("2. Testing _entities query on workflow subgraph:");
    let entities_query = json!({
        "query": r#"
            query GetEntities($representations: [_Any!]!) {
                _entities(representations: $representations) {
                    ... on Workflow {
                        id
                        name
                        status
                    }
                }
            }
        "#,
        "variables": {
            "representations": [
                {
                    "__typename": "Workflow",
                    "id": "123"
                }
            ]
        }
    });
    
    match client.post(workflow_url).json(&entities_query).send().await {
        Ok(response) => {
            let result: serde_json::Value = response.json().await?;
            println!("Entities response: {}\n", serde_json::to_string_pretty(&result)?);
        }
        Err(e) => {
            println!("Error: {}\n", e);
        }
    }
    
    // Test 3: Federated query through gateway
    println!("3. Testing federated query through gateway:");
    let federated_query = json!({
        "query": r#"
            {
                workflow(id: "123") {
                    id
                    name
                    status
                }
                
                _service {
                    sdl
                }
            }
        "#
    });
    
    match client.post(gateway_url).json(&federated_query).send().await {
        Ok(response) => {
            let result: serde_json::Value = response.json().await?;
            println!("Federated query response: {}\n", serde_json::to_string_pretty(&result)?);
        }
        Err(e) => {
            println!("Note: Gateway needs to be running at {}", gateway_url);
            println!("Error: {}\n", e);
        }
    }
    
    println!("Federation test complete!");
    println!("\nTo test the full federation setup:");
    println!("1. Start the workflow API: cargo run --bin workflow-engine");
    println!("2. Start the gateway: cargo run --bin graphql-gateway");
    println!("3. Run this test: cargo run --example test_federation");
    
    Ok(())
}