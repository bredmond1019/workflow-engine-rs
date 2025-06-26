use async_graphql::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Represents a query plan for executing a federated query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPlan {
    pub steps: Vec<QueryStep>,
}

/// A single step in the query execution plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStep {
    pub subgraph: String,
    pub query: String,
    pub depends_on: Vec<usize>, // Indices of steps this depends on
    pub provides: HashSet<String>, // Fields this step provides
    pub requires: HashSet<String>, // Fields this step requires
}

/// Result from executing a query step
#[derive(Debug, Clone)]
pub struct StepResult {
    pub data: Value,
    pub errors: Vec<Error>,
}

/// Query planner that optimizes federated query execution
pub struct QueryPlanner {
    schema_registry: Arc<crate::federation::schema_registry::SchemaRegistry>,
}

impl QueryPlanner {
    pub fn new(schema_registry: Arc<crate::federation::schema_registry::SchemaRegistry>) -> Self {
        Self { schema_registry }
    }

    /// Plan the execution of a GraphQL query across subgraphs
    pub async fn plan_query(&self, query: &str) -> Result<QueryPlan> {
        // Parse the query
        let parsed_query = async_graphql::parser::parse_query(query)
            .map_err(|e| Error::new(format!("Failed to parse query: {:?}", e)))?;

        // Get all available schemas
        let schemas = self.schema_registry.get_all_schemas().await;
        
        // Analyze which fields are needed from which subgraphs
        let field_locations = self.analyze_field_locations(&parsed_query, &schemas).await?;
        
        // Build the query plan
        let plan = self.build_query_plan(field_locations, &parsed_query).await?;
        
        Ok(plan)
    }

    /// Analyze which subgraphs provide which fields
    async fn analyze_field_locations(
        &self,
        query: &async_graphql::parser::types::ExecutableDocument,
        schemas: &[crate::federation::schema_registry::SubgraphSchema],
    ) -> Result<HashMap<String, String>> {
        let mut field_locations = HashMap::new();
        
        // For each field in the query, determine which subgraph provides it
        // This is a simplified implementation - a real one would parse the schemas
        // and build a complete field map
        
        for schema in schemas {
            if schema.sdl.contains("workflows") {
                field_locations.insert("workflows".to_string(), schema.name.clone());
            }
            if schema.sdl.contains("nodes") {
                field_locations.insert("nodes".to_string(), schema.name.clone());
            }
            if schema.sdl.contains("executions") {
                field_locations.insert("executions".to_string(), schema.name.clone());
            }
        }
        
        Ok(field_locations)
    }

    /// Build an optimized query plan
    async fn build_query_plan(
        &self,
        field_locations: HashMap<String, String>,
        query: &async_graphql::parser::types::ExecutableDocument,
    ) -> Result<QueryPlan> {
        let mut steps = Vec::new();
        let mut step_index = 0;
        
        // Group fields by subgraph
        let mut subgraph_fields: HashMap<String, Vec<String>> = HashMap::new();
        for (field, subgraph) in field_locations {
            subgraph_fields.entry(subgraph).or_default().push(field);
        }
        
        // Create a step for each subgraph
        for (subgraph, fields) in subgraph_fields {
            let step = QueryStep {
                subgraph: subgraph.clone(),
                query: self.build_subgraph_query(&fields, query)?,
                depends_on: vec![], // Will be filled in based on dependencies
                provides: fields.iter().cloned().collect(),
                requires: HashSet::new(), // Will be analyzed based on @requires directives
            };
            
            steps.push(step);
            step_index += 1;
        }
        
        // Analyze dependencies between steps
        self.analyze_dependencies(&mut steps)?;
        
        Ok(QueryPlan { steps })
    }

    /// Build a query for a specific subgraph
    fn build_subgraph_query(
        &self,
        fields: &[String],
        _original_query: &async_graphql::parser::types::ExecutableDocument,
    ) -> Result<String> {
        // This is a simplified implementation
        // A real implementation would properly transform the query
        let mut query = String::from("query {\n");
        for field in fields {
            query.push_str(&format!("  {}\n", field));
        }
        query.push_str("}");
        Ok(query)
    }

    /// Analyze dependencies between query steps
    fn analyze_dependencies(&self, steps: &mut Vec<QueryStep>) -> Result<()> {
        // Analyze @requires directives to determine dependencies
        // This is a simplified implementation
        
        for i in 0..steps.len() {
            for j in 0..i {
                // Check if step i requires any fields provided by step j
                let requires = &steps[i].requires;
                let provides = &steps[j].provides;
                
                if requires.intersection(provides).count() > 0 {
                    steps[i].depends_on.push(j);
                }
            }
        }
        
        Ok(())
    }

    /// Execute a query plan
    pub async fn execute_plan(
        &self,
        plan: QueryPlan,
        variables: Variables,
        ctx: &Context<'_>,
    ) -> Result<Value> {
        let mut results: HashMap<usize, StepResult> = HashMap::new();
        let mut final_data = serde_json::Map::new();
        
        // Execute steps in dependency order
        for (index, step) in plan.steps.iter().enumerate() {
            // Wait for dependencies
            for dep_index in &step.depends_on {
                if !results.contains_key(dep_index) {
                    return Err(Error::new(format!(
                        "Dependency {} not satisfied for step {}",
                        dep_index, index
                    )));
                }
            }
            
            // Build context with results from dependencies
            let mut step_variables = variables.clone();
            for dep_index in &step.depends_on {
                if let Some(dep_result) = results.get(dep_index) {
                    // Merge dependency results into variables
                    if let Value::Object(obj) = &dep_result.data {
                        for (key, value) in obj {
                            step_variables.insert(key.clone(), value.clone());
                        }
                    }
                }
            }
            
            // Execute the step
            let result = self.execute_step(&step, step_variables, ctx).await?;
            
            // Merge results
            if let Value::Object(obj) = &result.data {
                for (key, value) in obj {
                    final_data.insert(key.to_string(), value.clone().into_json()?);
                }
            }
            
            results.insert(index, result);
        }
        
        Ok(Value::from_json(serde_json::Value::Object(final_data))?)
    }

    /// Execute a single query step
    async fn execute_step(
        &self,
        step: &QueryStep,
        variables: Variables,
        ctx: &Context<'_>,
    ) -> Result<StepResult> {
        // Get the subgraph client
        let subgraph_client = ctx.data::<crate::subgraph::SubgraphClient>()?;
        
        // Get the subgraph schema
        let schema = self.schema_registry.get_schema(&step.subgraph).await
            .ok_or_else(|| Error::new(format!("Subgraph '{}' not found", step.subgraph)))?;
        
        // Execute the query against the subgraph
        let response = subgraph_client
            .query(&schema.url, &step.query, variables)
            .await?;
        
        Ok(StepResult {
            data: response.data,
            errors: response.errors.into_iter().map(|e| Error::new(e.message)).collect(),
        })
    }

    /// Optimize a query plan by merging compatible steps
    pub fn optimize_plan(&self, plan: QueryPlan) -> QueryPlan {
        // Merge steps that can be executed in parallel
        let mut optimized_steps = Vec::new();
        let mut merged_indices = HashSet::new();
        
        for (i, step) in plan.steps.iter().enumerate() {
            if merged_indices.contains(&i) {
                continue;
            }
            
            let mut merged_step = step.clone();
            
            // Find other steps that can be merged
            for (j, other_step) in plan.steps.iter().enumerate().skip(i + 1) {
                if merged_indices.contains(&j) {
                    continue;
                }
                
                // Can merge if:
                // 1. Same subgraph
                // 2. No dependencies between them
                // 3. Compatible requirements
                if step.subgraph == other_step.subgraph
                    && !step.depends_on.contains(&j)
                    && !other_step.depends_on.contains(&i)
                {
                    // Merge the queries
                    merged_step.query = self.merge_queries(&merged_step.query, &other_step.query);
                    merged_step.provides.extend(other_step.provides.clone());
                    merged_step.requires.extend(other_step.requires.clone());
                    merged_indices.insert(j);
                }
            }
            
            optimized_steps.push(merged_step);
        }
        
        QueryPlan { steps: optimized_steps }
    }

    /// Merge two GraphQL queries
    fn merge_queries(&self, query1: &str, query2: &str) -> String {
        // This is a simplified implementation
        // A real implementation would properly parse and merge the queries
        format!("{}\n{}", query1, query2)
    }
}

/// Query plan cache for performance optimization
pub struct QueryPlanCache {
    cache: Arc<RwLock<HashMap<String, (QueryPlan, std::time::Instant)>>>,
    ttl: std::time::Duration,
}

impl QueryPlanCache {
    pub fn new(ttl: std::time::Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }

    pub async fn get(&self, query: &str) -> Option<QueryPlan> {
        let cache = self.cache.read().await;
        if let Some((plan, timestamp)) = cache.get(query) {
            if timestamp.elapsed() < self.ttl {
                return Some(plan.clone());
            }
        }
        None
    }

    pub async fn insert(&self, query: String, plan: QueryPlan) {
        let mut cache = self.cache.write().await;
        cache.insert(query, (plan, std::time::Instant::now()));
    }

    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_query_step_creation() {
        let step = QueryStep {
            subgraph: "workflows".to_string(),
            query: "{ workflows { id name } }".to_string(),
            depends_on: vec![],
            provides: ["workflows".to_string()].into_iter().collect(),
            requires: HashSet::new(),
        };

        assert_eq!(step.subgraph, "workflows");
        assert!(step.provides.contains("workflows"));
        assert!(step.depends_on.is_empty());
    }

    #[tokio::test]
    async fn test_query_plan_creation() {
        let step1 = QueryStep {
            subgraph: "workflows".to_string(),
            query: "{ workflows { id name } }".to_string(),
            depends_on: vec![],
            provides: ["workflows".to_string()].into_iter().collect(),
            requires: HashSet::new(),
        };

        let step2 = QueryStep {
            subgraph: "nodes".to_string(),
            query: "{ nodes { id workflowId } }".to_string(),
            depends_on: vec![0],
            provides: ["nodes".to_string()].into_iter().collect(),
            requires: ["workflows".to_string()].into_iter().collect(),
        };

        let plan = QueryPlan {
            steps: vec![step1, step2],
        };

        assert_eq!(plan.steps.len(), 2);
        assert_eq!(plan.steps[1].depends_on, vec![0]);
    }

    #[tokio::test]
    async fn test_query_plan_cache() {
        let cache = QueryPlanCache::new(std::time::Duration::from_secs(60));
        
        let plan = QueryPlan {
            steps: vec![QueryStep {
                subgraph: "test".to_string(),
                query: "{ test }".to_string(),
                depends_on: vec![],
                provides: HashSet::new(),
                requires: HashSet::new(),
            }],
        };

        cache.insert("test_query".to_string(), plan.clone()).await;
        
        let cached = cache.get("test_query").await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().steps.len(), 1);
    }
}