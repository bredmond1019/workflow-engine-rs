//! MCP (Model Context Protocol) validation module
//! 
//! This module provides comprehensive validation for MCP protocol messages
//! to prevent security vulnerabilities, resource exhaustion, and protocol violations.

pub mod validation;

pub use validation::{
    McpMessageValidator,
    ValidationError,
    ValidationResult,
};