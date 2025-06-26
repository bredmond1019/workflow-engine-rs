//! Content Processing Engine

#![allow(clippy::manual_clamp, clippy::unnecessary_map_or, clippy::manual_range_contains, 
         clippy::manual_strip, clippy::only_used_in_recursion, clippy::needless_borrows_for_generic_args,
         clippy::unwrap_or_default, clippy::manual_flatten, clippy::needless_borrow)]
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