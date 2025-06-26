//! REST API endpoints for graph queries

pub mod graphql;

use actix_web::{web, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::client::DgraphClient;
use crate::query::{QueryType, QueryParameters, QueryBuilder, QueryConstraints};
use crate::service::{KnowledgeGraphService, RelationshipDiscoveryRequest, PathFindingRequest};

#[derive(Deserialize)]
pub struct GraphQueryRequest {
    pub query_type: QueryType,
    pub parameters: QueryParameters,
}

#[derive(Serialize)]
pub struct GraphQueryResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time_ms: u128,
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub execution_time_ms: u128,
}

/// Generic query endpoint (legacy support)
pub async fn query_graph(
    payload: web::Json<GraphQueryRequest>,
    client: web::Data<Arc<DgraphClient>>,
) -> ActixResult<HttpResponse> {
    let start_time = std::time::Instant::now();
    
    // Build the appropriate query based on the request type
    let query_builder = QueryBuilder::new();
    
    match query_builder.build_query(payload.query_type.clone(), payload.parameters.clone()) {
        Ok(query) => {
            // Execute the query against Dgraph
            match client.query(&query).await {
                Ok(result) => {
                    let execution_time = start_time.elapsed().as_millis();
                    
                    Ok(HttpResponse::Ok().json(GraphQueryResponse {
                        success: true,
                        data: Some(result),
                        error: None,
                        execution_time_ms: execution_time,
                    }))
                }
                Err(e) => {
                    let execution_time = start_time.elapsed().as_millis();
                    
                    Ok(HttpResponse::InternalServerError().json(GraphQueryResponse {
                        success: false,
                        data: None,
                        error: Some(format!("Query execution failed: {}", e)),
                        execution_time_ms: execution_time,
                    }))
                }
            }
        }
        Err(e) => {
            let execution_time = start_time.elapsed().as_millis();
            
            Ok(HttpResponse::BadRequest().json(GraphQueryResponse {
                success: false,
                data: None,
                error: Some(format!("Query building failed: {}", e)),
                execution_time_ms: execution_time,
            }))
        }
    }
}

/// Discover relationships for a concept
pub async fn discover_relationships(
    payload: web::Json<RelationshipDiscoveryRequest>,
    service: web::Data<Arc<KnowledgeGraphService>>,
) -> ActixResult<HttpResponse> {
    let start_time = std::time::Instant::now();
    
    match service.discover_relationships(payload.into_inner()).await {
        Ok(result) => {
            let execution_time = start_time.elapsed().as_millis();
            Ok(HttpResponse::Ok().json(ApiResponse {
                success: true,
                data: Some(result),
                error: None,
                execution_time_ms: execution_time,
            }))
        }
        Err(e) => {
            let execution_time = start_time.elapsed().as_millis();
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                data: None,
                error: Some(format!("Relationship discovery failed: {}", e)),
                execution_time_ms: execution_time,
            }))
        }
    }
}

/// Find path between two concepts
pub async fn find_path(
    payload: web::Json<PathFindingRequest>,
    service: web::Data<Arc<KnowledgeGraphService>>,
) -> ActixResult<HttpResponse> {
    let start_time = std::time::Instant::now();
    
    match service.find_path(payload.into_inner()).await {
        Ok(result) => {
            let execution_time = start_time.elapsed().as_millis();
            Ok(HttpResponse::Ok().json(ApiResponse {
                success: true,
                data: Some(result),
                error: None,
                execution_time_ms: execution_time,
            }))
        }
        Err(e) => {
            let execution_time = start_time.elapsed().as_millis();
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                data: None,
                error: Some(format!("Path finding failed: {}", e)),
                execution_time_ms: execution_time,
            }))
        }
    }
}

/// Calculate concept similarity
pub async fn calculate_similarity(
    path: web::Path<String>,
    query: web::Query<SimilarityQuery>,
    service: web::Data<Arc<KnowledgeGraphService>>,
) -> ActixResult<HttpResponse> {
    let start_time = std::time::Instant::now();
    let concept_id = path.into_inner();
    
    match service.calculate_similarity(&concept_id, query.limit).await {
        Ok(result) => {
            let execution_time = start_time.elapsed().as_millis();
            Ok(HttpResponse::Ok().json(ApiResponse {
                success: true,
                data: Some(result),
                error: None,
                execution_time_ms: execution_time,
            }))
        }
        Err(e) => {
            let execution_time = start_time.elapsed().as_millis();
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                data: None,
                error: Some(format!("Similarity calculation failed: {}", e)),
                execution_time_ms: execution_time,
            }))
        }
    }
}

/// Get recommendations for a user
pub async fn get_recommendations(
    path: web::Path<String>,
    query: web::Query<RecommendationQuery>,
    service: web::Data<Arc<KnowledgeGraphService>>,
) -> ActixResult<HttpResponse> {
    let start_time = std::time::Instant::now();
    let user_id = path.into_inner();
    
    match service.get_recommendations(&user_id, query.limit).await {
        Ok(result) => {
            let execution_time = start_time.elapsed().as_millis();
            Ok(HttpResponse::Ok().json(ApiResponse {
                success: true,
                data: Some(result),
                error: None,
                execution_time_ms: execution_time,
            }))
        }
        Err(e) => {
            let execution_time = start_time.elapsed().as_millis();
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                data: None,
                error: Some(format!("Recommendation generation failed: {}", e)),
                execution_time_ms: execution_time,
            }))
        }
    }
}

/// Create a new concept
pub async fn create_concept(
    payload: web::Json<serde_json::Value>,
    client: web::Data<Arc<DgraphClient>>,
) -> ActixResult<HttpResponse> {
    let start_time = std::time::Instant::now();
    
    let params = QueryParameters {
        concept_data: Some(payload.into_inner()),
        ..Default::default()
    };
    
    let query_builder = QueryBuilder::new();
    
    match query_builder.build_query(QueryType::CreateConcept, params) {
        Ok(mutation) => {
            match client.mutate(&mutation).await {
                Ok(result) => {
                    let execution_time = start_time.elapsed().as_millis();
                    Ok(HttpResponse::Created().json(ApiResponse {
                        success: true,
                        data: Some(result),
                        error: None,
                        execution_time_ms: execution_time,
                    }))
                }
                Err(e) => {
                    let execution_time = start_time.elapsed().as_millis();
                    Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                        success: false,
                        data: None,
                        error: Some(format!("Concept creation failed: {}", e)),
                        execution_time_ms: execution_time,
                    }))
                }
            }
        }
        Err(e) => {
            let execution_time = start_time.elapsed().as_millis();
            Ok(HttpResponse::BadRequest().json(ApiResponse::<()> {
                success: false,
                data: None,
                error: Some(format!("Invalid concept data: {}", e)),
                execution_time_ms: execution_time,
            }))
        }
    }
}

/// Update an existing concept
pub async fn update_concept(
    path: web::Path<String>,
    payload: web::Json<serde_json::Value>,
    client: web::Data<Arc<DgraphClient>>,
) -> ActixResult<HttpResponse> {
    let start_time = std::time::Instant::now();
    let concept_id = path.into_inner();
    
    let params = QueryParameters {
        concept_id: Some(concept_id),
        concept_data: Some(payload.into_inner()),
        ..Default::default()
    };
    
    let query_builder = QueryBuilder::new();
    
    match query_builder.build_query(QueryType::UpdateConcept, params) {
        Ok(mutation) => {
            match client.mutate(&mutation).await {
                Ok(result) => {
                    let execution_time = start_time.elapsed().as_millis();
                    Ok(HttpResponse::Ok().json(ApiResponse {
                        success: true,
                        data: Some(result),
                        error: None,
                        execution_time_ms: execution_time,
                    }))
                }
                Err(e) => {
                    let execution_time = start_time.elapsed().as_millis();
                    Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                        success: false,
                        data: None,
                        error: Some(format!("Concept update failed: {}", e)),
                        execution_time_ms: execution_time,
                    }))
                }
            }
        }
        Err(e) => {
            let execution_time = start_time.elapsed().as_millis();
            Ok(HttpResponse::BadRequest().json(ApiResponse::<()> {
                success: false,
                data: None,
                error: Some(format!("Invalid update data: {}", e)),
                execution_time_ms: execution_time,
            }))
        }
    }
}

/// Delete a concept
pub async fn delete_concept(
    path: web::Path<String>,
    client: web::Data<Arc<DgraphClient>>,
) -> ActixResult<HttpResponse> {
    let start_time = std::time::Instant::now();
    let concept_id = path.into_inner();
    
    let params = QueryParameters {
        concept_id: Some(concept_id),
        ..Default::default()
    };
    
    let query_builder = QueryBuilder::new();
    
    match query_builder.build_query(QueryType::DeleteConcept, params) {
        Ok(mutation) => {
            match client.mutate(&mutation).await {
                Ok(result) => {
                    let execution_time = start_time.elapsed().as_millis();
                    Ok(HttpResponse::Ok().json(ApiResponse {
                        success: true,
                        data: Some(result),
                        error: None,
                        execution_time_ms: execution_time,
                    }))
                }
                Err(e) => {
                    let execution_time = start_time.elapsed().as_millis();
                    Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                        success: false,
                        data: None,
                        error: Some(format!("Concept deletion failed: {}", e)),
                        execution_time_ms: execution_time,
                    }))
                }
            }
        }
        Err(e) => {
            let execution_time = start_time.elapsed().as_millis();
            Ok(HttpResponse::BadRequest().json(ApiResponse::<()> {
                success: false,
                data: None,
                error: Some(format!("Invalid concept ID: {}", e)),
                execution_time_ms: execution_time,
            }))
        }
    }
}

/// Health check endpoint
pub async fn health_check(
    client: web::Data<Arc<DgraphClient>>,
) -> ActixResult<HttpResponse> {
    let start_time = std::time::Instant::now();
    
    match client.health_check().await {
        Ok(is_healthy) => {
            let execution_time = start_time.elapsed().as_millis();
            let status = if is_healthy { "healthy" } else { "unhealthy" };
            
            Ok(HttpResponse::Ok().json(ApiResponse {
                success: is_healthy,
                data: Some(serde_json::json!({
                    "status": status,
                    "dgraph_connected": is_healthy
                })),
                error: if is_healthy { None } else { Some("Dgraph connection failed".to_string()) },
                execution_time_ms: execution_time,
            }))
        }
        Err(e) => {
            let execution_time = start_time.elapsed().as_millis();
            Ok(HttpResponse::ServiceUnavailable().json(ApiResponse::<()> {
                success: false,
                data: None,
                error: Some(format!("Health check failed: {}", e)),
                execution_time_ms: execution_time,
            }))
        }
    }
}

#[derive(Deserialize)]
pub struct SimilarityQuery {
    pub limit: Option<u32>,
}

#[derive(Deserialize)]
pub struct RecommendationQuery {
    pub limit: Option<u32>,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub category: Option<String>,
    pub difficulty: Option<String>,
    pub min_quality: Option<f32>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Serialize)]
pub struct PaginatedResponse<T> {
    pub success: bool,
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
    pub error: Option<String>,
    pub execution_time_ms: u128,
}

#[derive(Serialize)]
pub struct PaginationMeta {
    pub total_count: usize,
    pub limit: u32,
    pub offset: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

/// Search concepts with pagination and filtering
pub async fn search_concepts(
    query: web::Query<SearchQuery>,
    client: web::Data<Arc<DgraphClient>>,
) -> ActixResult<HttpResponse> {
    let start_time = std::time::Instant::now();
    let search_params = query.into_inner();
    
    let params = QueryParameters {
        search_term: Some(search_params.q),
        limit: search_params.limit,
        offset: search_params.offset,
        sort_by: search_params.sort_by,
        sort_order: search_params.sort_order,
        constraints: Some(QueryConstraints {
            categories: search_params.category.map(|c| vec![c]),
            difficulty: search_params.difficulty.map(|d| vec![d]).unwrap_or_default(),
            min_quality: search_params.min_quality.unwrap_or(0.0),
            include_subtopics: None,
        }),
        ..Default::default()
    };
    
    let query_builder = QueryBuilder::new();
    
    match query_builder.build_query(QueryType::SearchConcepts, params.clone()) {
        Ok(query) => {
            match client.query(&query).await {
                Ok(result) => {
                    let execution_time = start_time.elapsed().as_millis();
                    
                    // Parse the result (simplified - would need proper parsing)
                    let total_count = 0; // Would extract from result.conceptsAggregate.count
                    let limit = params.limit.unwrap_or(20);
                    let offset = params.offset.unwrap_or(0);
                    
                    let pagination = PaginationMeta {
                        total_count,
                        limit,
                        offset,
                        has_next: (offset + limit) < total_count as u32,
                        has_prev: offset > 0,
                    };
                    
                    Ok(HttpResponse::Ok().json(PaginatedResponse::<serde_json::Value> {
                        success: true,
                        data: vec![], // Would contain parsed concepts
                        pagination,
                        error: None,
                        execution_time_ms: execution_time,
                    }))
                }
                Err(e) => {
                    let execution_time = start_time.elapsed().as_millis();
                    Ok(HttpResponse::InternalServerError().json(PaginatedResponse::<serde_json::Value> {
                        success: false,
                        data: vec![],
                        pagination: PaginationMeta {
                            total_count: 0,
                            limit: 0,
                            offset: 0,
                            has_next: false,
                            has_prev: false,
                        },
                        error: Some(format!("Search failed: {}", e)),
                        execution_time_ms: execution_time,
                    }))
                }
            }
        }
        Err(e) => {
            let execution_time = start_time.elapsed().as_millis();
            Ok(HttpResponse::BadRequest().json(PaginatedResponse::<serde_json::Value> {
                success: false,
                data: vec![],
                pagination: PaginationMeta {
                    total_count: 0,
                    limit: 0,
                    offset: 0,
                    has_next: false,
                    has_prev: false,
                },
                error: Some(format!("Invalid search parameters: {}", e)),
                execution_time_ms: execution_time,
            }))
        }
    }
}

/// Configure API routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/query", web::post().to(query_graph))
            .route("/search", web::get().to(search_concepts))
            .route("/relationships", web::post().to(discover_relationships))
            .route("/path", web::post().to(find_path))
            .route("/similarity/{concept_id}", web::get().to(calculate_similarity))
            .route("/recommendations/{user_id}", web::get().to(get_recommendations))
            .route("/concepts", web::post().to(create_concept))
            .route("/concepts/{concept_id}", web::put().to(update_concept))
            .route("/concepts/{concept_id}", web::delete().to(delete_concept))
            .route("/health", web::get().to(health_check))
    );
}