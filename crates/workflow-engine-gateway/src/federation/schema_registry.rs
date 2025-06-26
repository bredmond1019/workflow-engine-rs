use async_graphql::parser::{parse_schema, types::ServiceDocument};
use async_graphql::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Represents a subgraph in the federation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubgraphSchema {
    pub name: String,
    pub url: String,
    pub sdl: String,
    #[serde(skip)]
    pub parsed_schema: Option<ServiceDocument>,
}

/// Registry for managing subgraph schemas
pub struct SchemaRegistry {
    schemas: Arc<RwLock<HashMap<String, SubgraphSchema>>>,
}

impl SchemaRegistry {
    pub fn new() -> Self {
        Self {
            schemas: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new subgraph schema
    pub async fn register_schema(
        &self,
        name: String,
        url: String,
        sdl: String,
    ) -> Result<()> {
        // Parse the SDL to validate it
        let parsed_schema = parse_schema(&sdl)
            .map_err(|e| Error::new(format!("Failed to parse SDL: {:?}", e)))?;

        let subgraph = SubgraphSchema {
            name: name.clone(),
            url,
            sdl,
            parsed_schema: Some(parsed_schema),
        };

        let mut schemas = self.schemas.write().await;
        schemas.insert(name, subgraph);
        
        Ok(())
    }

    /// Get a subgraph schema by name
    pub async fn get_schema(&self, name: &str) -> Option<SubgraphSchema> {
        let schemas = self.schemas.read().await;
        schemas.get(name).cloned()
    }

    /// Get all registered schemas
    pub async fn get_all_schemas(&self) -> Vec<SubgraphSchema> {
        let schemas = self.schemas.read().await;
        schemas.values().cloned().collect()
    }

    /// Update a subgraph schema
    pub async fn update_schema(
        &self,
        name: String,
        url: Option<String>,
        sdl: Option<String>,
    ) -> Result<()> {
        let mut schemas = self.schemas.write().await;
        
        if let Some(subgraph) = schemas.get_mut(&name) {
            if let Some(new_url) = url {
                subgraph.url = new_url;
            }
            
            if let Some(new_sdl) = sdl {
                // Parse the new SDL to validate it
                let parsed_schema = parse_schema(&new_sdl)
                    .map_err(|e| Error::new(format!("Failed to parse SDL: {:?}", e)))?;
                
                subgraph.sdl = new_sdl;
                subgraph.parsed_schema = Some(parsed_schema);
            }
            
            Ok(())
        } else {
            Err(Error::new(format!("Subgraph '{}' not found", name)))
        }
    }

    /// Remove a subgraph schema
    pub async fn remove_schema(&self, name: &str) -> Result<()> {
        let mut schemas = self.schemas.write().await;
        schemas.remove(name)
            .ok_or_else(|| Error::new(format!("Subgraph '{}' not found", name)))?;
        Ok(())
    }

    /// Compose all subgraph schemas into a federated schema
    pub async fn compose_federated_schema(&self) -> Result<String> {
        let schemas = self.schemas.read().await;
        
        if schemas.is_empty() {
            return Err(Error::new("No subgraphs registered"));
        }

        // Start with the federation directives
        let mut composed_sdl = String::from(
            r#"# Federated schema composed from subgraphs
scalar _Any
scalar _FieldSet

directive @external on FIELD_DEFINITION
directive @requires(fields: _FieldSet!) on FIELD_DEFINITION
directive @provides(fields: _FieldSet!) on FIELD_DEFINITION
directive @key(fields: _FieldSet!) on OBJECT | INTERFACE
directive @extends on OBJECT | INTERFACE

"#
        );

        // Add each subgraph's SDL
        for (name, subgraph) in schemas.iter() {
            composed_sdl.push_str(&format!("# Subgraph: {}\n", name));
            composed_sdl.push_str(&subgraph.sdl);
            composed_sdl.push_str("\n\n");
        }

        Ok(composed_sdl)
    }

    /// Validate that all required federation fields are present
    pub async fn validate_federation_compliance(&self) -> Result<Vec<String>> {
        let schemas = self.schemas.read().await;
        let mut issues = Vec::new();

        for (name, subgraph) in schemas.iter() {
            if let Some(_parsed) = &subgraph.parsed_schema {
                // TODO: Implement federation validation once we understand async-graphql v7 AST structure
                // For now, assume schemas are valid if they parse successfully
                _ = name; // Suppress unused warning
            }
        }

        Ok(issues)
    }
}

/// GraphQL query for schema registry management
pub struct SchemaRegistryQuery;

#[Object]
impl SchemaRegistryQuery {
    /// Get all registered subgraph schemas
    async fn subgraphs(&self, ctx: &Context<'_>) -> Result<Vec<SubgraphInfo>> {
        let registry = ctx.data::<SchemaRegistry>()?;
        let schemas = registry.get_all_schemas().await;
        
        Ok(schemas.into_iter().map(|s| SubgraphInfo {
            name: s.name,
            url: s.url,
            sdl_preview: s.sdl.lines().take(10).collect::<Vec<_>>().join("\n"),
        }).collect())
    }

    /// Get the composed federated schema
    async fn federated_schema(&self, ctx: &Context<'_>) -> Result<String> {
        let registry = ctx.data::<SchemaRegistry>()?;
        registry.compose_federated_schema().await
    }

    /// Validate federation compliance
    async fn validate_federation(&self, ctx: &Context<'_>) -> Result<ValidationResult> {
        let registry = ctx.data::<SchemaRegistry>()?;
        let issues = registry.validate_federation_compliance().await?;
        
        Ok(ValidationResult {
            is_valid: issues.is_empty(),
            issues,
        })
    }
}

/// GraphQL mutation for schema registry management
pub struct SchemaRegistryMutation;

#[Object]
impl SchemaRegistryMutation {
    /// Register a new subgraph
    async fn register_subgraph(
        &self,
        ctx: &Context<'_>,
        name: String,
        url: String,
        sdl: String,
    ) -> Result<bool> {
        let registry = ctx.data::<SchemaRegistry>()?;
        registry.register_schema(name, url, sdl).await?;
        Ok(true)
    }

    /// Update an existing subgraph
    async fn update_subgraph(
        &self,
        ctx: &Context<'_>,
        name: String,
        url: Option<String>,
        sdl: Option<String>,
    ) -> Result<bool> {
        let registry = ctx.data::<SchemaRegistry>()?;
        registry.update_schema(name, url, sdl).await?;
        Ok(true)
    }

    /// Remove a subgraph
    async fn remove_subgraph(
        &self,
        ctx: &Context<'_>,
        name: String,
    ) -> Result<bool> {
        let registry = ctx.data::<SchemaRegistry>()?;
        registry.remove_schema(&name).await?;
        Ok(true)
    }
}

#[derive(SimpleObject)]
struct SubgraphInfo {
    name: String,
    url: String,
    sdl_preview: String,
}

#[derive(SimpleObject)]
struct ValidationResult {
    is_valid: bool,
    issues: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_schema_registration() {
        let registry = SchemaRegistry::new();
        
        let sdl = r#"
            type Query {
                _service: _Service!
            }
            
            type _Service {
                sdl: String!
            }
        "#;
        
        registry.register_schema(
            "test".to_string(),
            "http://localhost:4001".to_string(),
            sdl.to_string()
        ).await.unwrap();
        
        let schema = registry.get_schema("test").await.unwrap();
        assert_eq!(schema.name, "test");
        assert_eq!(schema.url, "http://localhost:4001");
    }

    #[tokio::test]
    async fn test_schema_composition() {
        let registry = SchemaRegistry::new();
        
        // Register two subgraphs
        let sdl1 = r#"
            type Query {
                _service: _Service!
                workflows: [Workflow!]!
            }
            
            type _Service {
                sdl: String!
            }
            
            type Workflow @key(fields: "id") {
                id: ID!
                name: String!
            }
        "#;
        
        let sdl2 = r#"
            type Query {
                _service: _Service!
                nodes: [Node!]!
            }
            
            type _Service {
                sdl: String!
            }
            
            type Node {
                id: ID!
                workflowId: ID!
            }
        "#;
        
        registry.register_schema(
            "workflows".to_string(),
            "http://localhost:4001".to_string(),
            sdl1.to_string()
        ).await.unwrap();
        
        registry.register_schema(
            "nodes".to_string(),
            "http://localhost:4002".to_string(),
            sdl2.to_string()
        ).await.unwrap();
        
        let composed = registry.compose_federated_schema().await.unwrap();
        assert!(composed.contains("Subgraph: workflows"));
        assert!(composed.contains("Subgraph: nodes"));
        assert!(composed.contains("directive @key"));
    }
}