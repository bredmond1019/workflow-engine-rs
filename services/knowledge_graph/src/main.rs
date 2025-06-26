use actix_web::{web, App, HttpServer, middleware};
use std::sync::Arc;
use knowledge_graph::{
    KnowledgeGraphService,
    graph::DgraphConfig,
    api,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load configuration
    let dgraph_host = std::env::var("DGRAPH_HOST").unwrap_or_else(|_| "localhost".to_string());
    let dgraph_grpc_port = std::env::var("DGRAPH_GRPC_PORT")
        .unwrap_or_else(|_| "9080".to_string())
        .parse::<u16>()
        .expect("Invalid DGRAPH_GRPC_PORT");
    let dgraph_http_port = std::env::var("DGRAPH_HTTP_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("Invalid DGRAPH_HTTP_PORT");
    let service_port = std::env::var("SERVICE_PORT")
        .unwrap_or_else(|_| "3002".to_string())
        .parse::<u16>()
        .expect("Invalid SERVICE_PORT");

    // Create Dgraph config
    let dgraph_config = DgraphConfig {
        host: dgraph_host,
        grpc_port: dgraph_grpc_port,
        http_port: dgraph_http_port,
        max_connections: 20,
        query_timeout_ms: 30_000,
        mutation_timeout_ms: 60_000,
    };

    // Create service
    let service = Arc::new(KnowledgeGraphService::new(dgraph_config).await.map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to create service: {}", e))
    })?);

    // Create GraphQL schema
    let graphql_schema = api::graphql::schema::create_schema(service.clone());

    println!("Starting Knowledge Graph Service on port {}", service_port);

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            // Add logging middleware
            .wrap(middleware::Logger::default())
            // Add GraphQL schema data
            .app_data(web::Data::new(graphql_schema.clone()))
            // Add service data for REST endpoints
            .app_data(web::Data::new(service.clone()))
            // Configure GraphQL routes
            .configure(api::graphql::handlers::configure_graphql_routes)
            // Configure REST routes
            .configure(api::configure_routes)
    })
    .bind(("0.0.0.0", service_port))?
    .run()
    .await
}