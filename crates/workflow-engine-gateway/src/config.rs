use serde::{Deserialize, Serialize};
use crate::subgraph::SubgraphConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub subgraphs: Vec<SubgraphConfig>,
    pub port: u16,
    pub host: String,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            subgraphs: vec![
                SubgraphConfig {
                    name: "workflow-api".to_string(),
                    url: "http://localhost:8080/api/v1/graphql".to_string(),
                    schema_url: None,
                },
                SubgraphConfig {
                    name: "content-processing".to_string(),
                    url: "http://localhost:8082/graphql".to_string(),
                    schema_url: None,
                },
                SubgraphConfig {
                    name: "knowledge-graph".to_string(),
                    url: "http://localhost:3002/graphql".to_string(),
                    schema_url: None,
                },
                SubgraphConfig {
                    name: "realtime-communication".to_string(),
                    url: "http://localhost:8081/graphql".to_string(),
                    schema_url: None,
                },
            ],
            port: 4000,
            host: "0.0.0.0".to_string(),
        }
    }
}

impl GatewayConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        // Override subgraph URLs from environment variables if present
        if let Ok(url) = std::env::var("WORKFLOW_API_URL") {
            if let Some(subgraph) = config.subgraphs.iter_mut().find(|s| s.name == "workflow-api") {
                subgraph.url = url;
            }
        }
        
        if let Ok(url) = std::env::var("CONTENT_PROCESSING_URL") {
            if let Some(subgraph) = config.subgraphs.iter_mut().find(|s| s.name == "content-processing") {
                subgraph.url = url;
            }
        }
        
        if let Ok(url) = std::env::var("KNOWLEDGE_GRAPH_URL") {
            if let Some(subgraph) = config.subgraphs.iter_mut().find(|s| s.name == "knowledge-graph") {
                subgraph.url = url;
            }
        }
        
        if let Ok(url) = std::env::var("REALTIME_COMM_URL") {
            if let Some(subgraph) = config.subgraphs.iter_mut().find(|s| s.name == "realtime-communication") {
                subgraph.url = url;
            }
        }
        
        // Override gateway port and host if specified
        if let Ok(port) = std::env::var("GATEWAY_PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                config.port = port_num;
            }
        }
        
        if let Ok(host) = std::env::var("GATEWAY_HOST") {
            config.host = host;
        }
        
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = GatewayConfig::default();
        assert_eq!(config.subgraphs.len(), 4);
        assert_eq!(config.port, 4000);
        assert_eq!(config.host, "0.0.0.0");
    }
    
    #[test]
    fn test_from_env() {
        std::env::set_var("WORKFLOW_API_URL", "http://test:8080/graphql");
        std::env::set_var("GATEWAY_PORT", "5000");
        
        let config = GatewayConfig::from_env();
        
        let api_subgraph = config.subgraphs.iter()
            .find(|s| s.name == "workflow-api")
            .unwrap();
        assert_eq!(api_subgraph.url, "http://test:8080/graphql");
        assert_eq!(config.port, 5000);
        
        // Clean up
        std::env::remove_var("WORKFLOW_API_URL");
        std::env::remove_var("GATEWAY_PORT");
    }
}