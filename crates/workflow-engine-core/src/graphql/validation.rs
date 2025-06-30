//! GraphQL Query Validation
//!
//! Provides comprehensive validation for GraphQL queries including:
//! - Syntax validation
//! - Depth and complexity analysis
//! - Security policy enforcement
//! - Resource limit validation

use async_graphql::{Variables, parser, Value};
use std::time::{Duration, Instant};
use thiserror::Error;

/// Configuration for GraphQL query validation
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Maximum allowed query depth
    pub max_depth: u32,
    /// Maximum allowed query complexity score
    pub max_complexity: u32,
    /// Maximum time allowed for validation
    pub timeout: Duration,
    /// Whether introspection queries are allowed
    pub allow_introspection: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_depth: 15,
            max_complexity: 1000,
            timeout: Duration::from_secs(30),
            allow_introspection: false, // Secure by default
        }
    }
}

/// Result of query validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Calculated query depth
    pub depth: u32,
    /// Calculated complexity score
    pub complexity: u32,
    /// Estimated memory usage in bytes
    pub estimated_memory: u64,
    /// Validation errors found
    pub errors: Vec<String>,
    /// Security warnings
    pub security_warnings: Vec<String>,
    /// Whether the query requires rate limiting
    pub requires_rate_limiting: bool,
    /// Whether the query requires authorization
    pub requires_authorization: bool,
    /// Validation duration
    pub validation_duration: Duration,
}

/// Validation errors that can occur during GraphQL query validation
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Query depth {depth} exceeds limit {limit}")]
    QueryDepthExceeded { depth: u32, limit: u32 },
    
    #[error("Query complexity {complexity} exceeds limit {limit}")]
    QueryComplexityExceeded { complexity: u32, limit: u32 },
    
    #[error("GraphQL syntax error: {message} at line {line}, column {column}")]
    SyntaxError { message: String, line: u32, column: u32 },
    
    #[error("Invalid GraphQL operation: {message}")]
    InvalidOperation { message: String },
    
    #[error("Introspection queries are disabled")]
    IntrospectionDisabled,
    
    #[error("Too many aliases: {count} exceeds limit {limit}")]
    TooManyAliases { count: u32, limit: u32 },
    
    #[error("Too many selections: {count} exceeds limit {limit}")]
    TooManySelections { count: u32, limit: u32 },
    
    #[error("Variable complexity exceeds limit")]
    VariableComplexityExceeded { depth: u32, limit: u32 },
    
    #[error("Unsafe mutation detected: {reason}")]
    UnsafeMutation { reason: String },
    
    #[error("Subscription limits exceeded: {reason}")]
    SubscriptionLimitsExceeded { reason: String },
    
    #[error("Unauthorized subscription: {field}")]
    UnauthorizedSubscription { field: String },
    
    #[error("Validation timeout: took {duration:?}, limit {limit:?}")]
    ValidationTimeout { duration: Duration, limit: Duration },
    
    #[error("Estimated memory usage {estimated} bytes exceeds limit {limit} bytes")]
    EstimatedMemoryExceeded { estimated: u64, limit: u64 },
}

/// Query depth analyzer
#[derive(Debug, Clone)]
pub struct QueryDepth;

impl QueryDepth {
    pub fn calculate_depth(query: &str) -> u32 {
        // Simplified depth calculation using brace counting
        // This is a basic approach that works without parsing the full AST
        let mut depth: u32 = 0;
        let mut max_depth: u32 = 0;
        
        for char in query.chars() {
            match char {
                '{' => {
                    depth += 1;
                    max_depth = max_depth.max(depth);
                },
                '}' => {
                    depth = depth.saturating_sub(1);
                },
                _ => {},
            }
        }
        
        max_depth
    }
}

/// Query complexity analyzer
#[derive(Debug, Clone)]
pub struct QueryComplexity;

impl QueryComplexity {
    pub fn calculate_complexity(query: &str, _variables: &Variables) -> u32 {
        // Simplified complexity calculation
        let mut complexity = 0;
        
        // Count selections (fields)
        complexity += query.matches('\n').count() as u32;
        
        // Increase complexity for list operations
        if query.contains("limit:") || query.contains("first:") {
            complexity += 10;
        }
        
        // Increase complexity for nested operations
        complexity += query.matches('{').count() as u32 * 2;
        
        // Increase complexity for aliases
        complexity += query.matches(':').count() as u32;
        
        complexity
    }
}

/// Introspection policy enforcer
#[derive(Debug, Clone)]
pub struct IntrospectionPolicy {
    allowed: bool,
}

impl IntrospectionPolicy {
    pub fn new(allowed: bool) -> Self {
        Self { allowed }
    }
    
    pub fn validate(&self, query: &str) -> Result<(), ValidationError> {
        if self.allowed {
            return Ok(());
        }
        
        // Check for introspection patterns
        if query.contains("__schema") || 
           query.contains("__type") || 
           query.contains("__Field") ||
           query.contains("__typename") {
            return Err(ValidationError::IntrospectionDisabled);
        }
        
        Ok(())
    }
}

/// Mutation policy enforcer
#[derive(Debug, Clone)]
pub struct MutationPolicy;

impl MutationPolicy {
    pub fn validate_mutations(query: &str) -> Vec<String> {
        let mut warnings = Vec::new();
        
        // Check for dangerous mutation patterns
        if query.contains("deleteAll") || query.contains("delete_all") {
            warnings.push("Potentially dangerous bulk deletion detected".to_string());
        }
        
        if query.contains("reset") || query.contains("Reset") {
            warnings.push("Administrative operation detected".to_string());
        }
        
        if query.contains("bulk") || query.contains("mass") || query.contains("batch") {
            warnings.push("Bulk operation requires rate limiting".to_string());
        }
        
        // Check for SQL injection patterns
        let injection_patterns = ["DROP", "DELETE", "UPDATE", "--", "UNION SELECT"];
        for pattern in &injection_patterns {
            if query.to_uppercase().contains(pattern) {
                warnings.push(format!("Potential SQL injection pattern detected: {}", pattern));
            }
        }
        
        warnings
    }
}

/// Subscription policy enforcer
#[derive(Debug, Clone)]
pub struct SubscriptionPolicy;

impl SubscriptionPolicy {
    pub fn validate_subscriptions(query: &str) -> Vec<String> {
        let mut warnings = Vec::new();
        
        // Check for resource-intensive subscriptions
        if query.contains("all") || query.contains("All") {
            warnings.push("Resource-intensive subscription detected".to_string());
        }
        
        // Check for authorization-required subscriptions
        if query.contains("user") || query.contains("private") || query.contains("system") {
            warnings.push("Authorization required for subscription".to_string());
        }
        
        // Check for high-frequency polling
        if query.contains("pollInterval") && (query.contains("10") || query.contains("100")) {
            warnings.push("High-frequency subscription detected".to_string());
        }
        
        // Count subscriptions
        let subscription_count = query.matches("subscription").count();
        if subscription_count > 3 {
            warnings.push(format!("Too many subscriptions in single operation: {}", subscription_count));
        }
        
        warnings
    }
}

/// Main GraphQL query validator
pub struct QueryValidator {
    config: ValidationConfig,
    introspection_policy: IntrospectionPolicy,
}

impl QueryValidator {
    pub fn new(config: ValidationConfig) -> Self {
        let introspection_policy = IntrospectionPolicy::new(config.allow_introspection);
        
        Self {
            config,
            introspection_policy,
        }
    }
    
    pub async fn validate_query(
        &self,
        query: &str,
        variables: Variables
    ) -> Result<ValidationResult, ValidationError> {
        let start_time = Instant::now();
        
        // Basic syntax validation
        if let Err(e) = parser::parse_query(query) {
            return Err(ValidationError::SyntaxError {
                message: format!("Parse error: {:?}", e),
                line: 0,
                column: 0,
            });
        }
        
        // Check timeout during validation
        if start_time.elapsed() > self.config.timeout {
            return Err(ValidationError::ValidationTimeout {
                duration: start_time.elapsed(),
                limit: self.config.timeout,
            });
        }
        
        // Validate introspection policy
        self.introspection_policy.validate(query)?;
        
        // Calculate query depth
        let depth = QueryDepth::calculate_depth(query);
        if depth > self.config.max_depth {
            return Err(ValidationError::QueryDepthExceeded {
                depth,
                limit: self.config.max_depth,
            });
        }
        
        // Calculate query complexity
        let complexity = QueryComplexity::calculate_complexity(query, &variables);
        if complexity > self.config.max_complexity {
            return Err(ValidationError::QueryComplexityExceeded {
                complexity,
                limit: self.config.max_complexity,
            });
        }
        
        // Validate variables complexity
        self.validate_variable_complexity(&variables)?;
        
        // Check for excessive aliases and selections
        self.validate_query_structure(query)?;
        
        // Analyze security issues
        let mut security_warnings = Vec::new();
        security_warnings.extend(MutationPolicy::validate_mutations(query));
        security_warnings.extend(SubscriptionPolicy::validate_subscriptions(query));
        
        // Estimate memory usage
        let estimated_memory = self.estimate_memory_usage(query, &variables);
        
        // Check memory limits (example: 100MB limit)
        const MAX_MEMORY_BYTES: u64 = 100 * 1024 * 1024;
        if estimated_memory > MAX_MEMORY_BYTES {
            return Err(ValidationError::EstimatedMemoryExceeded {
                estimated: estimated_memory,
                limit: MAX_MEMORY_BYTES,
            });
        }
        
        // Determine if rate limiting is required
        let requires_rate_limiting = self.requires_rate_limiting(query);
        
        // Determine if authorization is required
        let requires_authorization = self.requires_authorization(query);
        
        let validation_duration = start_time.elapsed();
        
        Ok(ValidationResult {
            depth,
            complexity,
            estimated_memory,
            errors: Vec::new(), // No errors if we got here
            security_warnings,
            requires_rate_limiting,
            requires_authorization,
            validation_duration,
        })
    }
    
    fn validate_variable_complexity(&self, variables: &Variables) -> Result<(), ValidationError> {
        // Simple depth check for variables
        let max_depth = self.calculate_variable_depth(variables);
        if max_depth > 10 { // Reasonable limit for variable nesting
            return Err(ValidationError::VariableComplexityExceeded {
                depth: max_depth,
                limit: 10,
            });
        }
        Ok(())
    }
    
    fn calculate_variable_depth(&self, variables: &Variables) -> u32 {
        let mut max_depth = 0;
        
        for (_, value) in variables.iter() {
            let depth = self.calculate_value_depth(value, 0);
            max_depth = max_depth.max(depth);
        }
        
        max_depth
    }
    
    fn calculate_value_depth(&self, value: &Value, current_depth: u32) -> u32 {
        match value {
            Value::Object(obj) => {
                let mut max_depth = current_depth;
                for (_, nested_value) in obj {
                    let depth = self.calculate_value_depth(nested_value, current_depth + 1);
                    max_depth = max_depth.max(depth);
                }
                max_depth
            },
            Value::List(list) => {
                let mut max_depth = current_depth;
                for item in list {
                    let depth = self.calculate_value_depth(item, current_depth + 1);
                    max_depth = max_depth.max(depth);
                }
                max_depth
            },
            _ => current_depth,
        }
    }
    
    fn validate_query_structure(&self, query: &str) -> Result<(), ValidationError> {
        // Count aliases (DoS protection)
        let alias_count = query.matches(':').count() as u32;
        if alias_count > 100 {
            return Err(ValidationError::TooManyAliases {
                count: alias_count,
                limit: 100,
            });
        }
        
        // Count selections
        let selection_count = query.lines().count() as u32;
        if selection_count > 200 {
            return Err(ValidationError::TooManySelections {
                count: selection_count,
                limit: 200,
            });
        }
        
        Ok(())
    }
    
    fn estimate_memory_usage(&self, query: &str, _variables: &Variables) -> u64 {
        // Simplified memory estimation based on query length and complexity
        let base_size = query.len() as u64;
        let field_count = query.lines().count() as u64;
        
        // Estimate: base query size + field count * average field size
        base_size + (field_count * 100) // 100 bytes per field estimate
    }
    
    fn requires_rate_limiting(&self, query: &str) -> bool {
        query.contains("bulk") || 
        query.contains("mass") || 
        query.contains("batch") ||
        query.contains("mutation")
    }
    
    fn requires_authorization(&self, query: &str) -> bool {
        query.contains("user") || 
        query.contains("private") || 
        query.contains("system") ||
        query.contains("admin") ||
        query.contains("subscription")
    }
}