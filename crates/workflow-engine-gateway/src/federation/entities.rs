use async_graphql::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use futures::future::BoxFuture;

/// Represents an entity reference in the federation
#[derive(Debug, Clone, Serialize, Deserialize, InputObject)]
pub struct EntityReference {
    #[serde(rename = "__typename")]
    #[graphql(name = "__typename")]
    pub typename: String,
    #[serde(flatten)]
    #[graphql(skip)]
    pub keys: HashMap<String, serde_json::Value>,
}

/// Trait for resolving entity references
#[async_trait::async_trait]
pub trait EntityResolver: Send + Sync {
    /// Resolve entity references to their full representations
    async fn resolve_entities(&self, references: Vec<EntityReference>) -> Result<Vec<Value>>;
}

/// Default entity resolver that handles basic entity resolution
pub struct DefaultEntityResolver {
    resolvers: HashMap<String, Box<dyn Fn(EntityReference) -> BoxFuture<'static, Result<Value>> + Send + Sync>>,
}

impl DefaultEntityResolver {
    pub fn new() -> Self {
        Self {
            resolvers: HashMap::new(),
        }
    }

    /// Register an entity resolver for a specific type
    pub fn register_resolver<F, Fut>(&mut self, typename: &str, resolver: F)
    where
        F: Fn(EntityReference) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Value>> + Send + 'static,
    {
        self.resolvers.insert(
            typename.to_string(),
            Box::new(move |reference| Box::pin(resolver(reference))),
        );
    }
}

#[async_trait::async_trait]
impl EntityResolver for DefaultEntityResolver {
    async fn resolve_entities(&self, references: Vec<EntityReference>) -> Result<Vec<Value>> {
        let mut results = Vec::new();
        
        for reference in references {
            if let Some(resolver) = self.resolvers.get(&reference.typename) {
                let result = resolver(reference.clone()).await?;
                results.push(result);
            } else {
                return Err(Error::new(format!(
                    "No resolver found for entity type: {}",
                    reference.typename
                )));
            }
        }
        
        Ok(results)
    }
}

/// Entity union type for federation
#[derive(Union)]
pub enum Entity {
    Workflow(WorkflowEntity),
    Node(NodeEntity),
    Execution(ExecutionEntity),
}

/// Workflow entity
#[derive(SimpleObject, Serialize, Deserialize)]
pub struct WorkflowEntity {
    pub id: ID,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
}

/// Node entity
#[derive(SimpleObject)]
pub struct NodeEntity {
    pub id: ID,
    pub workflow_id: ID,
    pub name: String,
    pub node_type: String,
}

/// Execution entity
#[derive(SimpleObject)]
pub struct ExecutionEntity {
    pub id: ID,
    pub workflow_id: ID,
    pub status: String,
    pub started_at: String,
    pub completed_at: Option<String>,
}

/// Entity resolution query for federation
#[derive(Default)]
pub struct EntitiesQuery;

#[Object]
impl EntitiesQuery {
    /// Resolve entity references
    async fn entities(
        &self,
        ctx: &Context<'_>,
        representations: Vec<EntityReference>,
    ) -> Result<Vec<Entity>> {
        let resolver = ctx.data::<Box<dyn EntityResolver>>()?;
        let values = resolver.resolve_entities(representations).await?;
        
        // Convert Values to Entity enum
        let entities = values
            .into_iter()
            .map(|value| {
                // Parse the value to determine entity type and convert
                // This is a simplified version - in production you'd have more robust parsing
                if let Ok(workflow) = serde_json::from_value::<WorkflowEntity>(value.into_json()?) {
                    Ok(Entity::Workflow(workflow))
                } else {
                    Err(Error::new("Failed to parse entity"))
                }
            })
            .collect::<Result<Vec<_>>>()?;
        
        Ok(entities)
    }
}

/// Service definition query for federation
#[derive(Default)]
pub struct ServiceQuery;

#[Object]
impl ServiceQuery {
    /// Return the service's SDL
    async fn service(&self, ctx: &Context<'_>) -> Result<Service> {
        let sdl = ctx.data::<String>()
            .map(|s| s.clone())
            .unwrap_or_else(|_| String::from("# Service SDL not configured"));
        
        Ok(Service { sdl })
    }
}

/// Service definition
#[derive(SimpleObject)]
pub struct Service {
    pub sdl: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_entity_reference_serialization() {
        let reference = EntityReference {
            typename: "Workflow".to_string(),
            keys: {
                let mut map = HashMap::new();
                map.insert("id".to_string(), serde_json::Value::from("workflow-123"));
                map
            },
        };

        let json = serde_json::to_string(&reference).unwrap();
        assert!(json.contains("__typename"));
        assert!(json.contains("Workflow"));
        assert!(json.contains("workflow-123"));
    }

    #[tokio::test]
    async fn test_default_entity_resolver() {
        let mut resolver = DefaultEntityResolver::new();
        
        // Register a simple resolver
        resolver.register_resolver("Workflow", |reference| async move {
            Ok(Value::from_json(serde_json::json!({
                "id": reference.keys.get("id").cloned().unwrap_or(serde_json::Value::Null),
                "name": "Test Workflow",
                "description": null,
                "status": "active"
            })).unwrap())
        });

        let reference = EntityReference {
            typename: "Workflow".to_string(),
            keys: {
                let mut map = HashMap::new();
                map.insert("id".to_string(), serde_json::Value::from("workflow-123"));
                map
            },
        };

        let results = resolver.resolve_entities(vec![reference]).await.unwrap();
        assert_eq!(results.len(), 1);
    }
}