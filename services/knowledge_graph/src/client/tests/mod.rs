//! Comprehensive test suite for GraphQL response parsing
//!
//! This module contains extensive tests for all response parsing methods,
//! including fixtures for realistic GraphQL responses, edge cases,
//! error handling, and complex nested structures.

mod fixtures;
mod response_parsing_tests;
mod edge_cases_tests;
mod error_handling_tests;
mod mutation_parsing_tests;
mod complex_structure_tests;

pub use fixtures::*;