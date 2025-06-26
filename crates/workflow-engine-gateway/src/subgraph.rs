use crate::error::{GatewayError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use async_graphql::{Variables, Response as GraphQLResponse};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubgraphConfig {
    pub name: String,
    pub url: String,
    pub schema_url: Option<String>,
}

pub struct SubgraphClient {
    client: Client,
    subgraphs: HashMap<String, SubgraphConfig>,
}

impl SubgraphClient {
    pub fn new(subgraphs: Vec<SubgraphConfig>) -> Self {
        let client = Client::new();
        let subgraph_map = subgraphs
            .into_iter()
            .map(|s| (s.name.clone(), s))
            .collect();
        
        Self {
            client,
            subgraphs: subgraph_map,
        }
    }
    
    pub async fn query_subgraph(
        &self,
        subgraph_name: &str,
        query: &str,
        variables: Option<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let subgraph = self.subgraphs.get(subgraph_name)
            .ok_or_else(|| GatewayError::SubgraphError(
                format!("Subgraph '{}' not found", subgraph_name)
            ))?;
        
        let body = serde_json::json!({
            "query": query,
            "variables": variables.unwrap_or(serde_json::json!({}))
        });
        
        let response = self.client
            .post(&subgraph.url)
            .json(&body)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(GatewayError::SubgraphError(
                format!("Subgraph request failed: {}", response.status())
            ));
        }
        
        let result: serde_json::Value = response.json().await?;
        Ok(result)
    }
    
    pub async fn introspect_subgraph(&self, subgraph_name: &str) -> Result<String> {
        let introspection_query = r#"
            query IntrospectionQuery {
                __schema {
                    types {
                        name
                        kind
                        fields {
                            name
                            type {
                                name
                                kind
                            }
                        }
                    }
                }
            }
        "#;
        
        let result = self.query_subgraph(subgraph_name, introspection_query, None).await?;
        Ok(serde_json::to_string_pretty(&result)?)
    }
    
    /// Get all configured subgraphs
    pub fn get_subgraphs(&self) -> Vec<SubgraphConfig> {
        self.subgraphs.values().cloned().collect()
    }
    
    /// Query multiple subgraphs in parallel
    pub async fn query(&self, url: &str, query: &str, variables: Variables) -> Result<GraphQLResponse> {
        let body = serde_json::json!({
            "query": query,
            "variables": variables
        });
        
        let response = self.client
            .post(url)
            .json(&body)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(GatewayError::SubgraphError(
                format!("Subgraph request failed: {}", response.status())
            ));
        }
        
        let result: serde_json::Value = response.json().await?;
        
        // Convert to GraphQL response
        let data = result.get("data")
            .map(|d| async_graphql::Value::from_json(d.clone()).unwrap_or(async_graphql::Value::Null))
            .unwrap_or(async_graphql::Value::Null);
        let errors = result.get("errors")
            .and_then(|e| e.as_array())
            .map(|errors| {
                errors.iter()
                    .filter_map(|e| {
                        e.get("message")
                            .and_then(|m| m.as_str())
                            .map(|msg| async_graphql::ServerError::new(msg, None))
                    })
                    .collect()
            })
            .unwrap_or_default();
        
        let mut response = GraphQLResponse::new(data);
        response.errors = errors;
        Ok(response)
    }
}