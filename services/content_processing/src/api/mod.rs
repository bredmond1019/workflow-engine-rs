//! API module for content processing service

pub mod graphql;

// REST API module
pub mod rest {
    pub use crate::api_rest::*;
}