//! GraphQL Query Validation and Security Module
//!
//! This module provides comprehensive GraphQL query validation and security features
//! for the AI Workflow Engine, including:
//!
//! - Query depth and complexity analysis
//! - Resource usage estimation and limits
//! - Security threat detection
//! - Syntax and semantic validation
//! - Introspection policy enforcement
//! - Mutation and subscription security

pub mod validation;
pub mod security;
pub mod limits;

pub use validation::{
    QueryValidator, ValidationConfig, ValidationResult, ValidationError,
    QueryDepth, QueryComplexity
};

pub use security::{
    QuerySecurityAnalyzer, SecurityLevel, ThreatAnalysis
};

pub use limits::{
    ResourceLimits, ExecutionLimits, TimeoutPolicy
};