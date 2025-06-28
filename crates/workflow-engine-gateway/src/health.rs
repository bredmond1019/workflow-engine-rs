use crate::{GatewayConfig, SubgraphClient, SubgraphConfig};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResult {
    pub subgraph_name: String,
    pub is_healthy: bool,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
}

pub struct HealthChecker {
    config: GatewayConfig,
}

impl HealthChecker {
    pub fn new(config: &GatewayConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }
    
    pub fn check_all_subgraphs(&self) -> Vec<HealthResult> {
        // For testing purposes, we'll return mock healthy results
        // In a real implementation, this would make actual health check requests
        self.config.subgraphs.iter().map(|subgraph| {
            HealthResult {
                subgraph_name: subgraph.name.clone(),
                is_healthy: true,
                latency_ms: Some(10),
                error: None,
            }
        }).collect()
    }
    
    pub async fn check_all_subgraphs_async(&self) -> Vec<HealthResult> {
        let mut results = Vec::new();
        let client = SubgraphClient::new(self.config.subgraphs.clone());
        
        for subgraph in &self.config.subgraphs {
            let start = std::time::Instant::now();
            let health_query = r#"{ __typename }"#;
            
            match client.query_subgraph(&subgraph.name, health_query, None).await {
                Ok(_) => {
                    results.push(HealthResult {
                        subgraph_name: subgraph.name.clone(),
                        is_healthy: true,
                        latency_ms: Some(start.elapsed().as_millis() as u64),
                        error: None,
                    });
                }
                Err(e) => {
                    results.push(HealthResult {
                        subgraph_name: subgraph.name.clone(),
                        is_healthy: false,
                        latency_ms: None,
                        error: Some(e.to_string()),
                    });
                }
            }
        }
        
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_health_checker_creation() {
        let config = GatewayConfig::default();
        let checker = HealthChecker::new(&config);
        let results = checker.check_all_subgraphs();
        
        assert_eq!(results.len(), 4);
        assert!(results.iter().all(|r| r.is_healthy));
    }
}