//! Simple GraphQL Validation Test - RED Phase
//!
//! This test demonstrates the TDD RED phase by showing that
//! GraphQL query validation fails as expected.

use workflow_engine_core::graphql::{
    validation::{QueryValidator, ValidationConfig, ValidationError},
};
use async_graphql::Variables;
use std::time::Duration;

#[tokio::test]
async fn test_graphql_depth_validation_red_phase() {
    // RED: This test should fail initially because our depth calculation is too simple
    
    let config = ValidationConfig {
        max_depth: 5,
        max_complexity: 1000,
        timeout: Duration::from_secs(30),
        allow_introspection: false,
    };
    
    let validator = QueryValidator::new(config);
    
    // Create a deeply nested query that should fail
    let deep_query = r#"
        query DeepQuery {
            workflow {
                nodes {
                    connections {
                        target {
                            parameters {
                                metadata {
                                    tags
                                }
                            }
                        }
                    }
                }
            }
        }
    "#;
    
    let result = validator.validate_query(deep_query, Variables::default()).await;
    
    // This should fail due to depth > 5
    assert!(result.is_err());
    
    if let Err(ValidationError::QueryDepthExceeded { depth, limit }) = result {
        assert!(depth > 5);
        assert_eq!(limit, 5);
        println!("✅ RED Phase: Depth validation correctly failed - depth: {}, limit: {}", depth, limit);
    } else {
        panic!("Expected QueryDepthExceeded error, got: {:?}", result);
    }
}

#[tokio::test]
async fn test_graphql_introspection_validation_red_phase() {
    // RED: This test should fail because introspection is disabled by default
    
    let config = ValidationConfig {
        max_depth: 20,
        max_complexity: 1000,
        timeout: Duration::from_secs(30),
        allow_introspection: false, // Disabled
    };
    
    let validator = QueryValidator::new(config);
    
    let introspection_query = r#"
        query IntrospectionQuery {
            __schema {
                types {
                    name
                    fields {
                        name
                        type {
                            name
                        }
                    }
                }
            }
        }
    "#;
    
    let result = validator.validate_query(introspection_query, Variables::default()).await;
    
    // This should fail due to introspection being disabled
    assert!(result.is_err());
    
    if let Err(ValidationError::IntrospectionDisabled) = result {
        println!("✅ RED Phase: Introspection validation correctly failed");
    } else {
        panic!("Expected IntrospectionDisabled error, got: {:?}", result);
    }
}

#[tokio::test]
async fn test_graphql_complexity_validation_red_phase() {
    // RED: This test should fail due to high complexity
    
    let config = ValidationConfig {
        max_depth: 20,
        max_complexity: 50, // Low complexity limit
        timeout: Duration::from_secs(30),
        allow_introspection: false,
    };
    
    let validator = QueryValidator::new(config);
    
    // Create a complex query with many fields and operations
    let complex_query = r#"
        query ComplexQuery {
            workflows(limit: 1000) {
                id
                name
                description
                status
                createdAt
                updatedAt
                nodes {
                    id
                    type
                    config
                    parameters
                    connections {
                        id
                        source
                        target
                    }
                }
                executions {
                    id
                    status
                    startTime
                    endTime
                    logs {
                        level
                        message
                        timestamp
                    }
                }
            }
        }
    "#;
    
    let result = validator.validate_query(complex_query, Variables::default()).await;
    
    // This should fail due to high complexity
    assert!(result.is_err());
    
    if let Err(ValidationError::QueryComplexityExceeded { complexity, limit }) = result {
        assert!(complexity > 50);
        assert_eq!(limit, 50);
        println!("✅ RED Phase: Complexity validation correctly failed - complexity: {}, limit: {}", complexity, limit);
    } else {
        panic!("Expected QueryComplexityExceeded error, got: {:?}", result);
    }
}

#[tokio::test]
async fn test_graphql_mutation_security_warnings_red_phase() {
    // RED: This test should show security warnings for dangerous mutations
    
    let config = ValidationConfig::default();
    let validator = QueryValidator::new(config);
    
    let dangerous_mutation = r#"
        mutation DangerousMutation {
            deleteAllWorkflows {
                count
            }
            resetDatabase {
                success
            }
        }
    "#;
    
    let result = validator.validate_query(dangerous_mutation, Variables::default()).await;
    
    // This should succeed but have security warnings
    match result {
        Ok(validation_result) => {
            assert!(!validation_result.security_warnings.is_empty());
            assert!(validation_result.requires_rate_limiting);
            println!("✅ RED Phase: Mutation security warnings detected: {:?}", validation_result.security_warnings);
        },
        Err(e) => {
            // Also acceptable if the validator rejects dangerous mutations entirely
            println!("✅ RED Phase: Dangerous mutation rejected: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_graphql_valid_query_should_pass() {
    // This test should pass even in the RED phase (simple valid query)
    
    let config = ValidationConfig::default();
    let validator = QueryValidator::new(config);
    
    let simple_query = r#"
        query SimpleQuery {
            workflow {
                id
                name
                status
            }
        }
    "#;
    
    let result = validator.validate_query(simple_query, Variables::default()).await;
    
    // This should pass
    assert!(result.is_ok());
    
    let validation_result = result.unwrap();
    assert!(validation_result.depth <= 15); // Default limit
    assert_eq!(validation_result.errors.len(), 0);
    
    println!("✅ Simple valid query passed validation - depth: {}, complexity: {}", 
             validation_result.depth, validation_result.complexity);
}