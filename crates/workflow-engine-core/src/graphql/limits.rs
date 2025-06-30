//! GraphQL Resource Limits and Policies
//!
//! Defines resource limits and execution policies for GraphQL queries

use std::time::Duration;
use serde::{Serialize, Deserialize};

/// Resource limits for GraphQL query execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in bytes
    pub max_memory_bytes: u64,
    /// Maximum CPU time in milliseconds
    pub max_cpu_time_ms: u64,
    /// Maximum number of database queries
    pub max_db_queries: u32,
    /// Maximum result set size
    pub max_result_size: u64,
    /// Maximum number of concurrent operations
    pub max_concurrent_ops: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 100 * 1024 * 1024, // 100MB
            max_cpu_time_ms: 30_000, // 30 seconds
            max_db_queries: 100,
            max_result_size: 10 * 1024 * 1024, // 10MB
            max_concurrent_ops: 10,
        }
    }
}

/// Execution limits for different operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLimits {
    /// Limits for query operations
    pub query_limits: ResourceLimits,
    /// Limits for mutation operations
    pub mutation_limits: ResourceLimits,
    /// Limits for subscription operations
    pub subscription_limits: ResourceLimits,
}

impl Default for ExecutionLimits {
    fn default() -> Self {
        Self {
            query_limits: ResourceLimits::default(),
            mutation_limits: ResourceLimits {
                max_memory_bytes: 50 * 1024 * 1024, // 50MB for mutations
                max_cpu_time_ms: 60_000, // 60 seconds for mutations
                max_db_queries: 50,
                max_result_size: 5 * 1024 * 1024, // 5MB
                max_concurrent_ops: 5,
            },
            subscription_limits: ResourceLimits {
                max_memory_bytes: 20 * 1024 * 1024, // 20MB for subscriptions
                max_cpu_time_ms: 5_000, // 5 seconds for subscriptions
                max_db_queries: 10,
                max_result_size: 1024 * 1024, // 1MB
                max_concurrent_ops: 3,
            },
        }
    }
}

/// Timeout policies for GraphQL operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutPolicy {
    /// Default query timeout
    pub query_timeout: Duration,
    /// Default mutation timeout
    pub mutation_timeout: Duration,
    /// Default subscription timeout
    pub subscription_timeout: Duration,
    /// Validation timeout
    pub validation_timeout: Duration,
    /// Network timeout for federated queries
    pub network_timeout: Duration,
}

impl Default for TimeoutPolicy {
    fn default() -> Self {
        Self {
            query_timeout: Duration::from_secs(30),
            mutation_timeout: Duration::from_secs(60),
            subscription_timeout: Duration::from_secs(300), // 5 minutes
            validation_timeout: Duration::from_secs(5),
            network_timeout: Duration::from_secs(10),
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Requests per minute per user
    pub requests_per_minute: u32,
    /// Queries per minute per user
    pub queries_per_minute: u32,
    /// Mutations per minute per user
    pub mutations_per_minute: u32,
    /// Subscriptions per user
    pub max_subscriptions_per_user: u32,
    /// Complexity points per minute per user
    pub complexity_points_per_minute: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 100,
            queries_per_minute: 80,
            mutations_per_minute: 20,
            max_subscriptions_per_user: 5,
            complexity_points_per_minute: 10000,
        }
    }
}

/// Security policies for GraphQL operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    /// Whether to allow introspection queries
    pub allow_introspection: bool,
    /// Whether to allow file uploads
    pub allow_file_uploads: bool,
    /// Maximum upload file size in bytes
    pub max_upload_size: u64,
    /// Allowed file types for uploads
    pub allowed_file_types: Vec<String>,
    /// Whether to log all queries for audit
    pub log_all_queries: bool,
    /// Whether to block queries with potential injection
    pub block_injection_attempts: bool,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            allow_introspection: false, // Secure by default
            allow_file_uploads: false,
            max_upload_size: 10 * 1024 * 1024, // 10MB
            allowed_file_types: vec![
                "image/jpeg".to_string(),
                "image/png".to_string(),
                "application/pdf".to_string(),
            ],
            log_all_queries: true,
            block_injection_attempts: true,
        }
    }
}

/// Combined policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLPolicy {
    pub resource_limits: ExecutionLimits,
    pub timeout_policy: TimeoutPolicy,
    pub rate_limit_config: RateLimitConfig,
    pub security_policy: SecurityPolicy,
}

impl Default for GraphQLPolicy {
    fn default() -> Self {
        Self {
            resource_limits: ExecutionLimits::default(),
            timeout_policy: TimeoutPolicy::default(),
            rate_limit_config: RateLimitConfig::default(),
            security_policy: SecurityPolicy::default(),
        }
    }
}

impl GraphQLPolicy {
    /// Create a strict policy for production environments
    pub fn strict() -> Self {
        Self {
            resource_limits: ExecutionLimits {
                query_limits: ResourceLimits {
                    max_memory_bytes: 50 * 1024 * 1024, // 50MB
                    max_cpu_time_ms: 15_000, // 15 seconds
                    max_db_queries: 50,
                    max_result_size: 5 * 1024 * 1024, // 5MB
                    max_concurrent_ops: 5,
                },
                mutation_limits: ResourceLimits {
                    max_memory_bytes: 25 * 1024 * 1024, // 25MB
                    max_cpu_time_ms: 30_000, // 30 seconds
                    max_db_queries: 25,
                    max_result_size: 2 * 1024 * 1024, // 2MB
                    max_concurrent_ops: 3,
                },
                subscription_limits: ResourceLimits {
                    max_memory_bytes: 10 * 1024 * 1024, // 10MB
                    max_cpu_time_ms: 2_000, // 2 seconds
                    max_db_queries: 5,
                    max_result_size: 512 * 1024, // 512KB
                    max_concurrent_ops: 2,
                },
            },
            timeout_policy: TimeoutPolicy {
                query_timeout: Duration::from_secs(15),
                mutation_timeout: Duration::from_secs(30),
                subscription_timeout: Duration::from_secs(120), // 2 minutes
                validation_timeout: Duration::from_secs(2),
                network_timeout: Duration::from_secs(5),
            },
            rate_limit_config: RateLimitConfig {
                requests_per_minute: 60,
                queries_per_minute: 50,
                mutations_per_minute: 10,
                max_subscriptions_per_user: 3,
                complexity_points_per_minute: 5000,
            },
            security_policy: SecurityPolicy {
                allow_introspection: false,
                allow_file_uploads: false,
                max_upload_size: 5 * 1024 * 1024, // 5MB
                allowed_file_types: vec!["image/jpeg".to_string(), "image/png".to_string()],
                log_all_queries: true,
                block_injection_attempts: true,
            },
        }
    }
    
    /// Create a permissive policy for development environments
    pub fn permissive() -> Self {
        Self {
            resource_limits: ExecutionLimits {
                query_limits: ResourceLimits {
                    max_memory_bytes: 500 * 1024 * 1024, // 500MB
                    max_cpu_time_ms: 120_000, // 2 minutes
                    max_db_queries: 500,
                    max_result_size: 50 * 1024 * 1024, // 50MB
                    max_concurrent_ops: 50,
                },
                mutation_limits: ResourceLimits {
                    max_memory_bytes: 200 * 1024 * 1024, // 200MB
                    max_cpu_time_ms: 180_000, // 3 minutes
                    max_db_queries: 200,
                    max_result_size: 20 * 1024 * 1024, // 20MB
                    max_concurrent_ops: 20,
                },
                subscription_limits: ResourceLimits {
                    max_memory_bytes: 100 * 1024 * 1024, // 100MB
                    max_cpu_time_ms: 30_000, // 30 seconds
                    max_db_queries: 50,
                    max_result_size: 10 * 1024 * 1024, // 10MB
                    max_concurrent_ops: 20,
                },
            },
            timeout_policy: TimeoutPolicy {
                query_timeout: Duration::from_secs(120),
                mutation_timeout: Duration::from_secs(300),
                subscription_timeout: Duration::from_secs(3600), // 1 hour
                validation_timeout: Duration::from_secs(30),
                network_timeout: Duration::from_secs(60),
            },
            rate_limit_config: RateLimitConfig {
                requests_per_minute: 1000,
                queries_per_minute: 800,
                mutations_per_minute: 200,
                max_subscriptions_per_user: 50,
                complexity_points_per_minute: 100000,
            },
            security_policy: SecurityPolicy {
                allow_introspection: true,
                allow_file_uploads: true,
                max_upload_size: 100 * 1024 * 1024, // 100MB
                allowed_file_types: vec![
                    "image/jpeg".to_string(),
                    "image/png".to_string(),
                    "image/gif".to_string(),
                    "application/pdf".to_string(),
                    "text/plain".to_string(),
                    "application/json".to_string(),
                ],
                log_all_queries: false,
                block_injection_attempts: false,
            },
        }
    }
}