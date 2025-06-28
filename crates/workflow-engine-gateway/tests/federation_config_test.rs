use workflow_engine_gateway::config::GatewayConfig;

#[test]
fn test_gateway_uses_correct_service_ports() {
    // Test that gateway configuration matches actual service ports
    let config = GatewayConfig::default();
    
    // Verify main API GraphQL endpoint
    let api_subgraph = config.subgraphs.iter()
        .find(|s| s.name == "workflow-api")
        .expect("workflow-api subgraph should exist");
    assert_eq!(api_subgraph.url, "http://localhost:8080/api/v1/graphql");
    
    // Verify Content Processing service port
    let content_subgraph = config.subgraphs.iter()
        .find(|s| s.name == "content-processing")
        .expect("content-processing subgraph should exist");
    assert_eq!(content_subgraph.url, "http://localhost:8082/graphql");
    
    // Verify Knowledge Graph service port
    let knowledge_subgraph = config.subgraphs.iter()
        .find(|s| s.name == "knowledge-graph")
        .expect("knowledge-graph subgraph should exist");
    assert_eq!(knowledge_subgraph.url, "http://localhost:3002/graphql");
    
    // Verify Realtime Communication service port
    let realtime_subgraph = config.subgraphs.iter()
        .find(|s| s.name == "realtime-communication")
        .expect("realtime-communication subgraph should exist");
    assert_eq!(realtime_subgraph.url, "http://localhost:8081/graphql");
}

#[test]
fn test_gateway_handles_service_discovery_from_env() {
    // Test that gateway can discover services from environment variables
    std::env::set_var("WORKFLOW_API_URL", "http://api:8080/api/v1/graphql");
    std::env::set_var("CONTENT_PROCESSING_URL", "http://content:8082/graphql");
    std::env::set_var("KNOWLEDGE_GRAPH_URL", "http://knowledge:3002/graphql");
    std::env::set_var("REALTIME_COMM_URL", "http://realtime:8081/graphql");
    
    let config = GatewayConfig::from_env();
    
    assert_eq!(config.subgraphs.len(), 4);
    assert!(config.subgraphs.iter().any(|s| s.url.contains("api:8080")));
    assert!(config.subgraphs.iter().any(|s| s.url.contains("content:8082")));
    assert!(config.subgraphs.iter().any(|s| s.url.contains("knowledge:3002")));
    assert!(config.subgraphs.iter().any(|s| s.url.contains("realtime:8081")));
}

#[test]
fn test_gateway_validates_subgraph_health_on_startup() {
    // Test that gateway checks if subgraphs are healthy before starting
    let config = GatewayConfig::default();
    let health_checker = workflow_engine_gateway::health::HealthChecker::new(&config);
    
    let health_results = health_checker.check_all_subgraphs();
    
    // All subgraphs should be reachable
    assert!(health_results.iter().all(|r| r.is_healthy));
    assert_eq!(health_results.len(), config.subgraphs.len());
}