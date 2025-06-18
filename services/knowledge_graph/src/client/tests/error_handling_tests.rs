//! Tests for error handling in response parsing
//!
//! These tests cover GraphQL error responses, malformed data,
//! missing fields, and various error conditions to ensure
//! robust error handling and meaningful error messages.

use super::fixtures::*;
use crate::client::dgraph::DgraphResponseParser;
use crate::error::KnowledgeGraphError;

#[test]
fn test_parse_graphql_errors() {
    let parser = DgraphResponseParser::new();
    let response = TestFixtures::graphql_errors_response();

    let result = parser.parse_concept_from_result(response);
    assert!(result.is_err(), "Should fail with GraphQL errors");

    match result.unwrap_err() {
        KnowledgeGraphError::GraphQLError { message, errors, .. } => {
            assert!(message.contains("Variable $conceptId is not defined"));
            assert!(message.contains("Field 'invalidField' doesn't exist"));
            assert_eq!(errors.len(), 2);
            
            // Check first error details
            assert_eq!(errors[0].message, "Variable $conceptId is not defined");
            assert_eq!(errors[0].path, Some(vec!["getConcept".to_string()]));
            assert!(errors[0].extensions.is_some());
            
            // Check second error details
            assert_eq!(errors[1].message, "Field 'invalidField' doesn't exist on type 'Concept'");
            assert_eq!(errors[1].path, Some(vec!["getConcept".to_string(), "invalidField".to_string()]));
        }
        _ => panic!("Expected GraphQLError, got different error type"),
    }
}

#[test]
fn test_parse_missing_data_field() {
    let parser = DgraphResponseParser::new();
    let response = TestFixtures::missing_data_field();

    let result = parser.parse_concept_from_result(response);
    assert!(result.is_err(), "Should fail with missing data field");

    match result.unwrap_err() {
        KnowledgeGraphError::ParseError { message, field, .. } => {
            assert!(message.contains("Missing 'data' field"));
            assert_eq!(field, Some("data".to_string()));
        }
        _ => panic!("Expected ParseError, got different error type"),
    }
}

#[test]
fn test_parse_malformed_concept_data() {
    let parser = DgraphResponseParser::new();
    let response = TestFixtures::malformed_data_response();

    let result = parser.parse_concept_from_result(response);
    assert!(result.is_err(), "Should fail with malformed data");

    // Should fail because UUID is invalid
    match result.unwrap_err() {
        KnowledgeGraphError::ParseError { message, .. } => {
            assert!(message.contains("Failed to parse UUID") || message.contains("Missing or invalid field"));
        }
        _ => panic!("Expected ParseError, got different error type"),
    }
}

#[test]
fn test_parse_invalid_uuid() {
    let parser = DgraphResponseParser::new();
    
    let response = serde_json::json!({
        "data": {
            "concept": {
                "id": "not-a-valid-uuid",
                "name": "Test Concept",
                "difficulty": "beginner",
                "category": "Test",
                "qualityScore": 0.5,
                "tags": [],
                "embeddings": [],
                "createdAt": TEST_TIMESTAMP,
                "updatedAt": TEST_TIMESTAMP,
                "version": 1
            }
        }
    });

    let result = parser.parse_concept_from_result(response);
    assert!(result.is_err(), "Should fail with invalid UUID");

    match result.unwrap_err() {
        KnowledgeGraphError::ParseError { message, field, raw_data, .. } => {
            assert!(message.contains("Failed to parse UUID"));
            assert_eq!(field, Some("id".to_string()));
            assert_eq!(raw_data, Some("not-a-valid-uuid".to_string()));
        }
        _ => panic!("Expected ParseError with UUID details, got different error type"),
    }
}

#[test]
fn test_parse_missing_required_fields() {
    let parser = DgraphResponseParser::new();
    
    // Missing required name field
    let response = serde_json::json!({
        "data": {
            "concept": {
                "id": TEST_UUID_1,
                "difficulty": "beginner",
                "category": "Test",
                "qualityScore": 0.5,
                "tags": [],
                "embeddings": [],
                "createdAt": TEST_TIMESTAMP,
                "updatedAt": TEST_TIMESTAMP,
                "version": 1
            }
        }
    });

    let result = parser.parse_concept_from_result(response);
    assert!(result.is_err(), "Should fail with missing required field");

    match result.unwrap_err() {
        KnowledgeGraphError::ParseError { message, field, .. } => {
            assert!(message.contains("Missing or invalid field: name"));
            assert_eq!(field, Some("name".to_string()));
        }
        _ => panic!("Expected ParseError for missing field, got different error type"),
    }
}

#[test]
fn test_parse_invalid_timestamps() {
    let parser = DgraphResponseParser::new();
    
    let response = serde_json::json!({
        "data": {
            "concept": {
                "id": TEST_UUID_1,
                "name": "Test Concept",
                "difficulty": "beginner",
                "category": "Test",
                "qualityScore": 0.5,
                "tags": [],
                "embeddings": [],
                "createdAt": "not-a-valid-timestamp",
                "updatedAt": TEST_TIMESTAMP,
                "version": 1
            }
        }
    });

    let result = parser.parse_concept_from_result(response);
    assert!(result.is_err(), "Should fail with invalid timestamp");

    match result.unwrap_err() {
        KnowledgeGraphError::ParseError { message, field, .. } => {
            assert!(message.contains("Invalid datetime format"));
            assert_eq!(field, Some("datetime".to_string()));
        }
        _ => panic!("Expected ParseError for invalid timestamp, got different error type"),
    }
}

#[test]
fn test_parse_search_results_missing_concepts_field() {
    let parser = DgraphResponseParser::new();
    
    let response = serde_json::json!({
        "data": {
            "other_field": "some_value"
        }
    });

    let result = parser.parse_concepts_from_search_result(response, None);
    assert!(result.is_err(), "Should fail with missing concepts field");

    match result.unwrap_err() {
        KnowledgeGraphError::ParseError { message, field, .. } => {
            assert!(message.contains("No concepts field found"));
            assert_eq!(field, Some("concepts".to_string()));
        }
        _ => panic!("Expected ParseError for missing concepts field, got different error type"),
    }
}

#[test]
fn test_parse_search_results_concepts_not_array() {
    let parser = DgraphResponseParser::new();
    
    let response = serde_json::json!({
        "data": {
            "concepts": "not-an-array"
        }
    });

    let result = parser.parse_concepts_from_search_result(response, None);
    assert!(result.is_err(), "Should fail when concepts is not an array");

    match result.unwrap_err() {
        KnowledgeGraphError::ParseError { message, field, .. } => {
            assert!(message.contains("Concepts field is not an array"));
            assert_eq!(field, Some("concepts".to_string()));
        }
        _ => panic!("Expected ParseError for non-array concepts, got different error type"),
    }
}

#[test]
fn test_parse_learning_resources_missing_field() {
    let parser = DgraphResponseParser::new();
    
    let response = serde_json::json!({
        "data": {
            "other_field": []
        }
    });

    let result = parser.parse_learning_resources(response);
    assert!(result.is_err(), "Should fail with missing resources field");

    match result.unwrap_err() {
        KnowledgeGraphError::ParseError { message, field, .. } => {
            assert!(message.contains("No resources field found"));
            assert_eq!(field, Some("resources".to_string()));
        }
        _ => panic!("Expected ParseError for missing resources field, got different error type"),
    }
}

#[test]
fn test_parse_learning_path_missing_field() {
    let parser = DgraphResponseParser::new();
    
    let response = serde_json::json!({
        "data": {
            "other_field": {}
        }
    });

    let result = parser.parse_learning_path(response);
    assert!(result.is_err(), "Should fail with missing learning path field");

    match result.unwrap_err() {
        KnowledgeGraphError::ParseError { message, field, .. } => {
            assert!(message.contains("No learning path field found"));
            assert_eq!(field, Some("learningPath".to_string()));
        }
        _ => panic!("Expected ParseError for missing learning path field, got different error type"),
    }
}

#[test]
fn test_parse_user_progress_missing_field() {
    let parser = DgraphResponseParser::new();
    
    let response = serde_json::json!({
        "data": {
            "other_field": []
        }
    });

    let result = parser.parse_user_progress(response);
    assert!(result.is_err(), "Should fail with missing progress field");

    match result.unwrap_err() {
        KnowledgeGraphError::ParseError { message, field, .. } => {
            assert!(message.contains("No progress field found"));
            assert_eq!(field, Some("progress".to_string()));
        }
        _ => panic!("Expected ParseError for missing progress field, got different error type"),
    }
}

#[test]
fn test_parse_mutation_with_errors() {
    let parser = DgraphResponseParser::new();
    
    let response = serde_json::json!({
        "errors": [
            {
                "message": "Conflict: concept already exists",
                "path": ["addConcept"],
                "extensions": {
                    "code": "CONFLICT",
                    "conflictingId": TEST_UUID_1
                }
            }
        ]
    });

    let result = parser.parse_mutation_result(response);
    assert!(result.is_err(), "Should fail with mutation errors");

    match result.unwrap_err() {
        KnowledgeGraphError::GraphQLError { message, errors, .. } => {
            assert!(message.contains("Conflict: concept already exists"));
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].message, "Conflict: concept already exists");
        }
        _ => panic!("Expected GraphQLError for mutation errors, got different error type"),
    }
}

#[test]
fn test_parse_mutation_missing_concept_field() {
    let parser = DgraphResponseParser::new();
    
    let response = serde_json::json!({
        "data": {
            "addConcept": {
                "numUids": 1,
                "uid": "0x123"
                // Missing concept field
            }
        }
    });

    let result = parser.parse_mutation_result(response);
    assert!(result.is_err(), "Should fail with missing concept in mutation result");

    match result.unwrap_err() {
        KnowledgeGraphError::ParseError { message, field, .. } => {
            assert!(message.contains("Missing concept in add mutation result"));
            assert_eq!(field, Some("concept".to_string()));
        }
        _ => panic!("Expected ParseError for missing concept in mutation, got different error type"),
    }
}

#[test]
fn test_parse_partial_success_response() {
    let parser = DgraphResponseParser::new();
    let response = TestFixtures::partial_success_response();

    // The current implementation treats responses with errors as failed, even if there's valid data
    let result = parser.parse_concepts_from_search_result(response, None);
    
    // This should fail because the parser checks for errors first
    assert!(result.is_err(), "Should fail due to GraphQL errors in response");
    
    match result.unwrap_err() {
        KnowledgeGraphError::GraphQLError { message, .. } => {
            assert!(message.contains("Some concepts could not be retrieved"));
        }
        _ => panic!("Expected GraphQLError"),
    }
}

#[test]
fn test_parse_completely_invalid_json_structure() {
    let parser = DgraphResponseParser::new();
    
    // Not even a valid GraphQL response structure
    let response = serde_json::json!("just a string");

    let result = parser.parse_concept_from_result(response);
    assert!(result.is_err(), "Should fail with invalid JSON structure");

    match result.unwrap_err() {
        KnowledgeGraphError::ParseError { message, .. } => {
            assert!(message.contains("Missing 'data' field"));
        }
        _ => panic!("Expected ParseError for invalid JSON structure, got different error type"),
    }
}

#[test]
fn test_parse_numeric_string_fields() {
    let parser = DgraphResponseParser::new();
    
    // Test with numeric values in string fields (should fail)
    let response = serde_json::json!({
        "data": {
            "concept": {
                "id": TEST_UUID_1,
                "name": 12345, // Number instead of string
                "difficulty": "beginner",
                "category": "Test",
                "qualityScore": 0.5,
                "tags": [],
                "embeddings": [],
                "createdAt": TEST_TIMESTAMP,
                "updatedAt": TEST_TIMESTAMP,
                "version": 1
            }
        }
    });

    let result = parser.parse_concept_from_result(response);
    assert!(result.is_err(), "Should fail with numeric value in string field");

    match result.unwrap_err() {
        KnowledgeGraphError::ParseError { message, field, .. } => {
            assert!(message.contains("Missing or invalid field: name"));
            assert_eq!(field, Some("name".to_string()));
        }
        _ => panic!("Expected ParseError for invalid field type, got different error type"),
    }
}

#[test]
fn test_graceful_degradation_with_mixed_valid_invalid_concepts() {
    let parser = DgraphResponseParser::new();
    
    // Mix of valid and invalid concepts in search results
    let response = serde_json::json!({
        "data": {
            "concepts": [
                {
                    "id": TEST_UUID_1,
                    "name": "Valid Concept",
                    "difficulty": "beginner",
                    "category": "Test",
                    "qualityScore": 0.7,
                    "tags": [],
                    "embeddings": [],
                    "createdAt": TEST_TIMESTAMP,
                    "updatedAt": TEST_TIMESTAMP,
                    "version": 1
                },
                {
                    "id": "invalid-uuid",
                    "name": "Invalid Concept",
                    "difficulty": "beginner",
                    "category": "Test",
                    "qualityScore": 0.7,
                    "tags": [],
                    "embeddings": [],
                    "createdAt": TEST_TIMESTAMP,
                    "updatedAt": TEST_TIMESTAMP,
                    "version": 1
                },
                {
                    "id": TEST_UUID_2,
                    "name": "Another Valid Concept",
                    "difficulty": "intermediate",
                    "category": "Test",
                    "qualityScore": 0.8,
                    "tags": [],
                    "embeddings": [],
                    "createdAt": TEST_TIMESTAMP,
                    "updatedAt": TEST_TIMESTAMP,
                    "version": 1
                }
            ]
        }
    });

    let result = parser.parse_concepts_from_search_result(response, None);
    
    // Should succeed with partial results (graceful degradation)
    assert!(result.is_ok(), "Should succeed with graceful degradation: {:?}", result.err());
    
    let concepts = result.unwrap();
    assert_eq!(concepts.len(), 2); // Only valid concepts should be returned
    assert_eq!(concepts[0].name, "Valid Concept");
    assert_eq!(concepts[1].name, "Another Valid Concept");
}

#[test]
fn test_all_concepts_invalid_should_fail() {
    let parser = DgraphResponseParser::new();
    
    // All concepts are invalid
    let response = serde_json::json!({
        "data": {
            "concepts": [
                {
                    "id": "invalid-uuid-1",
                    "name": "Invalid Concept 1",
                    "difficulty": "beginner",
                    "category": "Test",
                    "qualityScore": 0.7,
                    "tags": [],
                    "embeddings": [],
                    "createdAt": TEST_TIMESTAMP,
                    "updatedAt": TEST_TIMESTAMP,
                    "version": 1
                },
                {
                    "id": "invalid-uuid-2",
                    "name": "Invalid Concept 2",
                    "difficulty": "beginner",
                    "category": "Test",
                    "qualityScore": 0.7,
                    "tags": [],
                    "embeddings": [],
                    "createdAt": TEST_TIMESTAMP,
                    "updatedAt": TEST_TIMESTAMP,
                    "version": 1
                }
            ]
        }
    });

    let result = parser.parse_concepts_from_search_result(response, None);
    
    // Should fail when all concepts are invalid
    assert!(result.is_err(), "Should fail when all concepts are invalid");
    
    // Should be a PartialResultError since parsing failed for all items
    match result.unwrap_err() {
        KnowledgeGraphError::PartialResultError { message, .. } => {
            assert!(message.contains("Failed to parse any concepts"));
        }
        _ => panic!("Expected PartialResultError when all parsing fails"),
    }
}