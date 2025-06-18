//! Knowledge Graph Query Engine
//! 
//! High-performance graph database service for concept relationships,
//! learning paths, and knowledge exploration.

#![recursion_limit = "256"]

pub mod error;
pub mod graceful_degradation;
pub mod client;
pub mod graph;
pub mod query;
pub mod algorithms;
pub mod api;
pub mod service;

pub use error::{KnowledgeGraphError, Result, ErrorContext, CircuitBreaker, RetryPolicy, RetryExecutor, ResultExt};
pub use graceful_degradation::{GracefulDegradationHandler, DegradationStrategy, create_degradation_handler};
pub use client::*;
pub use graph::*;
pub use query::*;
pub use algorithms::*;
pub use service::*;