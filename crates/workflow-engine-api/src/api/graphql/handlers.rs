use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use actix_web::{web, HttpResponse};

use super::schema::WorkflowSchema;

/// GraphQL query/mutation handler
pub async fn graphql_handler(
    schema: web::Data<WorkflowSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let query = req.into_inner();
    let res = schema.execute(query).await;
    res.into()
}

/// GraphQL playground UI
pub async fn graphql_playground() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(GraphQLPlaygroundConfig::new("/api/v1/graphql")))
}