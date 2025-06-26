use thiserror::Error;

#[derive(Error, Debug)]
pub enum GatewayError {
    #[error("Subgraph error: {0}")]
    SubgraphError(String),
    
    #[error("Schema composition error: {0}")]
    SchemaCompositionError(String),
    
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("GraphQL error: {0}")]
    GraphQLError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

impl From<async_graphql::Error> for GatewayError {
    fn from(err: async_graphql::Error) -> Self {
        GatewayError::GraphQLError(err.message)
    }
}

pub type Result<T> = std::result::Result<T, GatewayError>;