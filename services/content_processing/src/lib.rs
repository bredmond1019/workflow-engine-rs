//! Content Processing Engine
//! 
//! High-performance content processing service for analyzing and extracting
//! information from various document formats.

pub mod models;
pub mod traits;
pub mod processor;
pub mod analysis;
pub mod parsers;
pub mod ai_integration;
// Content processing service implementation complete for basic functionality
// pub mod plugins;
// pub mod batch;
pub mod api;
pub mod api_rest;
pub mod db;

pub use models::*;
pub use traits::*;
pub use processor::*;

// Re-export key types for convenience
pub type Result<T> = std::result::Result<T, ProcessingError>;