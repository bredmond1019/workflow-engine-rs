use actix_web::{web, HttpResponse, Result as ActixResult};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use std::sync::Arc;

use super::schema::KnowledgeGraphSchema;

/// GraphQL endpoint handler
pub async fn graphql_handler(
    schema: web::Data<KnowledgeGraphSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

/// GraphQL playground UI handler
pub async fn graphql_playground() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(
            GraphQLPlaygroundConfig::new("/api/v1/graphql").subscription_endpoint("/api/v1/graphql"),
        )))
}

/// Configure GraphQL routes
pub fn configure_graphql_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/graphql", web::post().to(graphql_handler))
            .route("/graphql", web::get().to(graphql_playground))
    );
}