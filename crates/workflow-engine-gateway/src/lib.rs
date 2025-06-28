pub mod config;
pub mod error;
pub mod federation;
pub mod gateway;
pub mod health;
pub mod subgraph;

pub use config::GatewayConfig;
pub use error::{GatewayError, Result};
pub use gateway::{GraphQLGateway, HealthResponse, SubgraphHealth};
pub use subgraph::{SubgraphClient, SubgraphConfig};

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_health_endpoint() {
        let subgraphs = vec![
            SubgraphConfig {
                name: "test".to_string(),
                url: "http://localhost:8080/graphql".to_string(),
                schema_url: None,
            }
        ];
        
        let gateway = Arc::new(GraphQLGateway::new(subgraphs));
        let health = gateway.get_health().await;
        
        assert!(health.status == "healthy" || health.status == "degraded");
        assert!(!health.version.is_empty());
        assert!(health.uptime_seconds >= 0);
        assert!(health.subgraphs.contains_key("test"));
    }

    #[tokio::test]
    async fn test_subgraph_health_check() {
        let subgraphs = vec![
            SubgraphConfig {
                name: "test".to_string(),
                url: "http://localhost:8080/graphql".to_string(),
                schema_url: None,
            }
        ];
        
        let gateway = GraphQLGateway::new(subgraphs);
        let subgraph_health = gateway.check_subgraphs_health().await;
        
        assert!(subgraph_health.contains_key("test"));
        let test_health = &subgraph_health["test"];
        assert!(test_health.status == "healthy" || test_health.status == "unhealthy");
        assert_eq!(test_health.url, "http://localhost:8080/graphql");
    }
}
