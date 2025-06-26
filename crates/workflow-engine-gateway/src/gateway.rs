use crate::error::{Result, GatewayError};
use crate::subgraph::{SubgraphClient, SubgraphConfig};
use crate::federation::{
    SchemaRegistry, QueryPlanner, EntityResolver, DefaultEntityResolver,
    EntitiesQuery, ServiceQuery, QueryPlanCache,
};
use async_graphql::*;
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::Extension,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use futures_util::{stream, Stream};

pub struct GraphQLGateway {
    schema: Schema<QueryRoot, MutationRoot, SubscriptionRoot>,
    subgraph_client: Arc<SubgraphClient>,
    schema_registry: Arc<SchemaRegistry>,
    query_planner: Arc<QueryPlanner>,
    entity_resolver: Arc<Box<dyn EntityResolver>>,
    query_cache: Arc<QueryPlanCache>,
}

#[derive(MergedObject, Default)]
pub struct QueryRoot(BaseQuery, EntitiesQuery, ServiceQuery);

#[derive(Default)]
pub struct BaseQuery;

#[Object]
impl BaseQuery {
    /// Simple health check query
    async fn health(&self) -> &'static str {
        "GraphQL Gateway is healthy!"
    }
    
    /// Example federated query - get workflow by ID
    async fn workflow(&self, ctx: &Context<'_>, id: ID) -> Result<serde_json::Value> {
        let client = ctx.data::<Arc<SubgraphClient>>()?;
        
        let query = r#"
            query GetWorkflow($id: ID!) {
                workflow(id: $id) {
                    id
                    name
                    status
                }
            }
        "#;
        
        let variables = serde_json::json!({ "id": id });
        let result = client.query_subgraph("workflow", query, Some(variables)).await?;
        
        Ok(result)
    }
    
    /// List workflows with pagination
    async fn workflows(&self, ctx: &Context<'_>, limit: Option<i32>, offset: Option<i32>) -> Result<serde_json::Value> {
        let client = ctx.data::<Arc<SubgraphClient>>()?;
        
        let query = r#"
            query ListWorkflows($limit: Int, $offset: Int) {
                workflows(limit: $limit, offset: $offset) {
                    items {
                        id
                        name
                        status
                        createdAt
                    }
                    totalCount
                }
            }
        "#;
        
        let variables = serde_json::json!({ 
            "limit": limit.unwrap_or(10),
            "offset": offset.unwrap_or(0)
        });
        
        let result = client.query_subgraph("workflow", query, Some(variables)).await?;
        Ok(result)
    }
    
    /// Execute a federated query using the query planner
    async fn federated_query(&self, ctx: &Context<'_>, query: String) -> Result<serde_json::Value> {
        let planner = ctx.data::<Arc<QueryPlanner>>()?;
        let cache = ctx.data::<Arc<QueryPlanCache>>()?;
        
        // Check cache first
        if let Some(plan) = cache.get(&query).await {
            let result = planner.execute_plan(plan, Variables::default(), ctx).await?;
            return Ok(result.into_json()?);
        }
        
        // Plan and execute the query
        let plan = planner.plan_query(&query).await?;
        let optimized_plan = planner.optimize_plan(plan.clone());
        
        // Cache the plan
        cache.insert(query, optimized_plan.clone()).await;
        
        // Execute the plan
        let result = planner.execute_plan(optimized_plan, Variables::default(), ctx).await?;
        Ok(result.into_json()?)
    }
}

#[derive(Default)]
pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Example mutation - create a new workflow
    async fn create_workflow(&self, ctx: &Context<'_>, name: String, description: Option<String>) -> Result<serde_json::Value> {
        let client = ctx.data::<Arc<SubgraphClient>>()?;
        
        let mutation = r#"
            mutation CreateWorkflow($name: String!, $description: String) {
                createWorkflow(input: { name: $name, description: $description }) {
                    id
                    name
                    description
                    status
                    createdAt
                }
            }
        "#;
        
        let variables = serde_json::json!({ 
            "name": name,
            "description": description
        });
        
        let result = client.query_subgraph("workflow", mutation, Some(variables)).await?;
        Ok(result)
    }
}

#[derive(Default)]
pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    /// Example subscription - watch workflow status changes
    async fn workflow_status_changed(&self, workflow_id: ID) -> impl Stream<Item = String> {
        // For POC, just return a simple stream
        // In production, this would connect to the subgraph's subscription endpoint
        stream::unfold(
            (workflow_id, 0),
            |(wf_id, state)| async move {
                match state {
                    0 => {
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        Some((format!("Watching workflow {:?}", wf_id), (wf_id, 1)))
                    }
                    1 => {
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        Some(("Status: Running".to_string(), (wf_id, 2)))
                    }
                    2 => {
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        Some(("Status: Completed".to_string(), (wf_id, 3)))
                    }
                    _ => None,
                }
            },
        )
    }
}

impl GraphQLGateway {
    pub fn new(subgraphs: Vec<SubgraphConfig>) -> Self {
        let subgraph_client = Arc::new(SubgraphClient::new(subgraphs));
        let schema_registry = Arc::new(SchemaRegistry::new());
        let query_planner = Arc::new(QueryPlanner::new(schema_registry.clone()));
        let entity_resolver: Arc<Box<dyn EntityResolver>> = Arc::new(Box::new(DefaultEntityResolver::new()));
        let query_cache = Arc::new(QueryPlanCache::new(std::time::Duration::from_secs(300))); // 5 minute cache
        
        let schema = Schema::build(QueryRoot::default(), MutationRoot, SubscriptionRoot)
            .data(subgraph_client.clone())
            .data(schema_registry.clone())
            .data(query_planner.clone())
            .data(entity_resolver.clone())
            .data(query_cache.clone())
            .enable_federation()
            .finish();
        
        Self {
            schema,
            subgraph_client,
            schema_registry,
            query_planner,
            entity_resolver,
            query_cache,
        }
    }
    
    /// Register a subgraph schema with the federation
    pub async fn register_subgraph(&self, name: String, url: String, sdl: String) -> Result<()> {
        self.schema_registry.register_schema(name, url, sdl).await
            .map_err(|e| GatewayError::GraphQLError(e.message))
    }
    
    /// Initialize federation by fetching schemas from all subgraphs
    pub async fn initialize_federation(&self) -> Result<()> {
        // For each configured subgraph, fetch its schema
        let subgraphs = self.subgraph_client.get_subgraphs();
        
        for subgraph in subgraphs {
            // Query the _service field to get the SDL
            let service_query = r#"
                query {
                    _service {
                        sdl
                    }
                }
            "#;
            
            match self.subgraph_client.query_subgraph(&subgraph.name, service_query, None).await {
                Ok(result) => {
                    if let Some(service) = result.get("_service") {
                        if let Some(sdl) = service.get("sdl").and_then(|v| v.as_str()) {
                            self.register_subgraph(
                                subgraph.name.clone(),
                                subgraph.url.clone(),
                                sdl.to_string()
                            ).await?;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to fetch schema from subgraph {}: {:?}", subgraph.name, e);
                }
            }
        }
        
        Ok(())
    }
    
    pub fn into_router(self) -> Router {
        Router::new()
            .route("/graphql", post(graphql_handler))
            .route("/graphql", get(graphql_playground))
            .layer(Extension(self.schema))
            .layer(CorsLayer::permissive())
    }
}

async fn graphql_handler(
    schema: Extension<Schema<QueryRoot, MutationRoot, SubscriptionRoot>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}