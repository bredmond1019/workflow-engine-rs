use workflow_engine_gateway::{GraphQLGateway, SubgraphConfig};
use tracing_subscriber;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Configure subgraphs
    let subgraphs = vec![
        SubgraphConfig {
            name: "workflow".to_string(),
            url: "http://localhost:8080/graphql".to_string(),
            schema_url: None,
        },
        SubgraphConfig {
            name: "knowledge_graph".to_string(),
            url: "http://localhost:3002/graphql".to_string(),
            schema_url: None,
        },
        SubgraphConfig {
            name: "content_processing".to_string(),
            url: "http://localhost:8001/graphql".to_string(),
            schema_url: None,
        },
        SubgraphConfig {
            name: "realtime_communication".to_string(),
            url: "http://localhost:8002/graphql".to_string(),
            schema_url: None,
        },
    ];
    
    // Create gateway
    let gateway = GraphQLGateway::new(subgraphs);
    let app = gateway.into_router();
    
    // Start server
    let addr = "127.0.0.1:4000";
    println!("GraphQL Gateway running at http://{}/graphql", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}