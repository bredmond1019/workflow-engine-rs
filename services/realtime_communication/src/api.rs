//! WebSocket API endpoints and server management

pub mod graphql;

use actix_web::{web, HttpResponse, App};
use async_graphql::{http::GraphiQLSource, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};

use crate::server::{WebSocketServer, ServerConfig, websocket_handler, health_handler, metrics_handler};
use self::graphql::schema::{QueryRoot, MutationRoot, SubscriptionRoot, RealtimeCommunicationSchema};

/// Start the WebSocket server with the given configuration
pub async fn start_server(config: ServerConfig) -> std::io::Result<()> {
    let server = WebSocketServer::new(config);
    server.start().await
}

/// GraphQL request handler
pub async fn graphql_handler(
    schema: web::Data<RealtimeCommunicationSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

/// GraphiQL playground handler
pub async fn graphiql() -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(GraphiQLSource::build().endpoint("/graphql").finish()))
}

/// Create the Actix web application with WebSocket routes
pub fn create_app(
    server_state: web::Data<crate::server::ServerState>,
    graphql_schema: web::Data<RealtimeCommunicationSchema>,
) -> App<
    impl actix_web::dev::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    App::new()
        .app_data(server_state)
        .app_data(graphql_schema)
        .route("/ws", web::get().to(websocket_handler))
        .route("/health", web::get().to(health_handler))
        .route("/metrics", web::get().to(metrics_handler))
        .route("/info", web::get().to(server_info_handler))
        .route("/graphql", web::post().to(graphql_handler))
        .route("/graphql", web::get().to(graphql_handler))
        .route("/graphiql", web::get().to(graphiql))
}

/// Server information endpoint
pub async fn server_info_handler(
    state: web::Data<crate::server::ServerState>,
) -> HttpResponse {
    let stats = state.metrics.get_stats().await;
    
    HttpResponse::Ok().json(serde_json::json!({
        "server": {
            "name": "WebSocket Real-time Communication Server",
            "version": "1.0.0",
            "config": {
                "host": state.config.host,
                "port": state.config.port,
                "max_connections": state.config.max_connections,
                "heartbeat_interval_secs": state.config.heartbeat_interval.as_secs(),
                "client_timeout_secs": state.config.client_timeout.as_secs(),
                "max_frame_size": state.config.max_frame_size
            }
        },
        "stats": stats,
        "features": [
            "websocket",
            "heartbeat",
            "topic_subscription",
            "broadcast_messaging", 
            "direct_messaging",
            "connection_management",
            "metrics"
        ]
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};
    use crate::server::{ServerState, ServerMetrics};
    use crate::connection::ConnectionManager;
    use std::sync::Arc;

    #[actix_web::test]
    async fn test_server_info_endpoint() {
        let config = ServerConfig::default();
        let connection_manager = Arc::new(ConnectionManager::new(config.max_connections));
        let metrics = Arc::new(ServerMetrics::default());
        
        let state = ServerState {
            connection_manager,
            config: config.clone(),
            metrics,
        };

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state))
                .route("/info", web::get().to(server_info_handler))
        ).await;

        let req = test::TestRequest::get()
            .uri("/info")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}