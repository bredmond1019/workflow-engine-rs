#[cfg(feature = "graphql")]
pub mod schema;
#[cfg(feature = "graphql")]
pub mod handlers;

#[cfg(feature = "graphql")]
pub use schema::{create_schema, WorkflowSchema};
#[cfg(feature = "graphql")]
pub use handlers::{graphql_handler, graphql_playground};