pub mod schema;
pub mod handlers;
pub mod resolvers;
pub mod federation;

pub use schema::create_schema;

// Always expose test schema for integration tests
pub use schema::create_test_schema;