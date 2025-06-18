//! Tests for mutation response parsing
//!
//! These tests cover parsing of various mutation results including
//! add, update, delete operations, bulk operations, and custom mutations.

use super::fixtures::*;
use crate::client::dgraph::{DgraphResponseParser, MutationOperationType};

#[test]
fn test_parse_add_concept_mutation() {
    let parser = DgraphResponseParser::new();
    let response = TestFixtures::add_concept_mutation_success();

    let result = parser.parse_mutation_result(response);
    assert!(result.is_ok(), "Failed to parse add mutation: {:?}", result.err());

    let mutation_result = result.unwrap();
    assert!(mutation_result.success);
    assert_eq!(mutation_result.operation_type, MutationOperationType::Add);
    assert_eq!(mutation_result.affected_count, 1);
    assert_eq!(mutation_result.affected_ids.len(), 1);
    assert_eq!(mutation_result.affected_ids[0].to_string(), TEST_UUID_1);
    assert_eq!(mutation_result.uids, vec!["0x123abc"]);
    assert!(mutation_result.message.as_ref().unwrap().contains("created successfully"));
    assert!(mutation_result.data.is_some());
    assert_eq!(mutation_result.conflicts.len(), 0);
}

#[test]
fn test_parse_update_concept_mutation_with_conflicts() {
    let parser = DgraphResponseParser::new();
    let response = TestFixtures::update_concept_mutation_with_conflicts();

    let result = parser.parse_mutation_result(response);
    assert!(result.is_ok(), "Failed to parse update mutation with conflicts: {:?}", result.err());

    let mutation_result = result.unwrap();
    assert!(mutation_result.success);
    assert_eq!(mutation_result.operation_type, MutationOperationType::Update);
    assert_eq!(mutation_result.affected_count, 1);
    assert_eq!(mutation_result.affected_ids.len(), 1);
    assert!(mutation_result.message.as_ref().unwrap().contains("updated successfully"));
    
    // Verify conflicts
    assert_eq!(mutation_result.conflicts.len(), 2);
    
    let version_conflict = &mutation_result.conflicts[0];
    assert_eq!(version_conflict.field, "version");
    assert_eq!(version_conflict.existing_value, Some(serde_json::json!(1)));
    assert_eq!(version_conflict.attempted_value, Some(serde_json::json!(1)));
    assert_eq!(version_conflict.resolution, Some("incremented".to_string()));
    
    let quality_conflict = &mutation_result.conflicts[1];
    assert_eq!(quality_conflict.field, "qualityScore");
    assert_eq!(quality_conflict.existing_value, Some(serde_json::json!(0.7)));
    assert_eq!(quality_conflict.attempted_value, Some(serde_json::json!(0.9)));
    assert_eq!(quality_conflict.resolution, Some("overwritten".to_string()));
}

#[test]
fn test_parse_delete_concept_mutation() {
    let parser = DgraphResponseParser::new();
    let response = TestFixtures::delete_concept_mutation_success();

    let result = parser.parse_mutation_result(response);
    assert!(result.is_ok(), "Failed to parse delete mutation: {:?}", result.err());

    let mutation_result = result.unwrap();
    assert!(mutation_result.success);
    assert_eq!(mutation_result.operation_type, MutationOperationType::Delete);
    assert_eq!(mutation_result.affected_count, 1);
    assert_eq!(mutation_result.affected_ids.len(), 1);
    assert_eq!(mutation_result.affected_ids[0].to_string(), TEST_UUID_1);
    assert_eq!(mutation_result.uids, vec!["0x123abc"]);
    assert!(mutation_result.message.as_ref().unwrap().contains("Successfully deleted concept"));
    assert!(mutation_result.data.is_none()); // Delete operations don't return data
}

#[test]
fn test_parse_bulk_add_concepts() {
    let parser = DgraphResponseParser::new();
    let response = TestFixtures::bulk_add_concepts_success();

    let result = parser.parse_mutation_result(response);
    assert!(result.is_ok(), "Failed to parse bulk add mutation: {:?}", result.err());

    let mutation_result = result.unwrap();
    assert!(mutation_result.success);
    assert_eq!(mutation_result.operation_type, MutationOperationType::Bulk);
    assert_eq!(mutation_result.affected_count, 2);
    assert_eq!(mutation_result.affected_ids.len(), 2);
    assert!(mutation_result.message.as_ref().unwrap().contains("Created 2 concepts"));
    
    // Verify that data contains the created concepts
    assert!(mutation_result.data.is_some());
    let data = mutation_result.data.unwrap();
    assert!(data.is_array());
    assert_eq!(data.as_array().unwrap().len(), 2);
}

#[test]
fn test_parse_raw_dgraph_mutation() {
    let parser = DgraphResponseParser::new();
    let response = TestFixtures::raw_dgraph_mutation_success();

    let result = parser.parse_mutation_result(response);
    assert!(result.is_ok(), "Failed to parse raw DGraph mutation: {:?}", result.err());

    let mutation_result = result.unwrap();
    assert!(mutation_result.success);
    assert_eq!(mutation_result.operation_type, MutationOperationType::Add);
    assert_eq!(mutation_result.affected_count, 3); // 3 UIDs returned
    assert_eq!(mutation_result.uids.len(), 3);
    assert!(mutation_result.uids.contains(&"0x123".to_string()));
    assert!(mutation_result.uids.contains(&"0x124".to_string()));
    assert!(mutation_result.uids.contains(&"0x125".to_string()));
    
    // Verify query data is preserved
    assert!(mutation_result.data.is_some());
    let data = mutation_result.data.unwrap();
    assert!(data.get("q").is_some());
}

#[test]
fn test_parse_generic_custom_mutation() {
    let parser = DgraphResponseParser::new();
    let response = TestFixtures::generic_mutation_result();

    let result = parser.parse_mutation_result(response);
    assert!(result.is_ok(), "Failed to parse generic mutation: {:?}", result.err());

    let mutation_result = result.unwrap();
    assert!(mutation_result.success);
    assert_eq!(mutation_result.operation_type, MutationOperationType::Upsert); // Default for generic
    assert_eq!(mutation_result.affected_count, 1);
    assert_eq!(mutation_result.affected_ids.len(), 1);
    assert_eq!(mutation_result.affected_ids[0].to_string(), TEST_UUID_1);
    assert_eq!(mutation_result.uids, vec!["0x999"]);
    assert!(mutation_result.message.as_ref().unwrap().contains("Custom operation completed"));
    
    // Verify metadata is preserved
    assert!(mutation_result.data.is_some());
    let data = mutation_result.data.unwrap();
    assert!(data.get("metadata").is_some());
}

#[test]
fn test_parse_learning_resource_mutation() {
    let parser = DgraphResponseParser::new();
    
    // Create a learning resource mutation response
    let response = serde_json::json!({
        "data": {
            "addLearningResource": {
                "learningResource": {
                    "id": TEST_UUID_1,
                    "url": "https://example.com/new-resource",
                    "title": "New Learning Resource",
                    "resourceType": "video",
                    "format": "mp4",
                    "source": "Internal",
                    "quality": 0.9,
                    "difficulty": "intermediate",
                    "duration": 1800,
                    "language": "English",
                    "createdAt": TEST_TIMESTAMP,
                    "updatedAt": TEST_TIMESTAMP
                }
            }
        }
    });

    let result = parser.parse_mutation_result(response);
    assert!(result.is_ok(), "Failed to parse learning resource mutation: {:?}", result.err());

    let mutation_result = result.unwrap();
    assert!(mutation_result.success);
    assert_eq!(mutation_result.operation_type, MutationOperationType::Add);
    assert_eq!(mutation_result.affected_count, 1);
    assert!(mutation_result.message.as_ref().unwrap().contains("Learning resource created"));
    
    // Verify the resource data is preserved
    assert!(mutation_result.data.is_some());
    let data = mutation_result.data.unwrap();
    assert_eq!(data["url"], "https://example.com/new-resource");
    assert_eq!(data["title"], "New Learning Resource");
}

#[test]
fn test_parse_learning_path_mutation() {
    let parser = DgraphResponseParser::new();
    
    // Create a learning path mutation response
    let response = serde_json::json!({
        "data": {
            "addLearningPath": {
                "learningPath": {
                    "id": TEST_UUID_1,
                    "name": "New Learning Path",
                    "description": "A comprehensive learning path",
                    "targetAudience": "Beginners",
                    "estimatedTime": 600.0,
                    "difficultyProgression": "beginner -> advanced",
                    "learningOutcomes": ["Master basics", "Build projects"],
                    "creator": "Expert Teacher",
                    "isCustom": true,
                    "createdAt": TEST_TIMESTAMP,
                    "updatedAt": TEST_TIMESTAMP
                }
            }
        }
    });

    let result = parser.parse_mutation_result(response);
    assert!(result.is_ok(), "Failed to parse learning path mutation: {:?}", result.err());

    let mutation_result = result.unwrap();
    assert!(mutation_result.success);
    assert_eq!(mutation_result.operation_type, MutationOperationType::Add);
    assert!(mutation_result.message.as_ref().unwrap().contains("Learning path created"));
}

#[test]
fn test_parse_user_progress_mutation() {
    let parser = DgraphResponseParser::new();
    
    // Create a user progress mutation response
    let response = serde_json::json!({
        "data": {
            "addUserProgress": {
                "userProgress": {
                    "id": TEST_UUID_1,
                    "userId": "user456",
                    "conceptId": TEST_UUID_2,
                    "status": "started",
                    "percentComplete": 0.0,
                    "timeSpent": 0,
                    "resourcesCompleted": 0,
                    "difficultyRating": null,
                    "notes": "Just getting started",
                    "startedAt": TEST_TIMESTAMP,
                    "completedAt": null,
                    "lastAccessedAt": TEST_TIMESTAMP
                }
            }
        }
    });

    let result = parser.parse_mutation_result(response);
    assert!(result.is_ok(), "Failed to parse user progress mutation: {:?}", result.err());

    let mutation_result = result.unwrap();
    assert!(mutation_result.success);
    assert_eq!(mutation_result.operation_type, MutationOperationType::Add);
    assert!(mutation_result.message.as_ref().unwrap().contains("User progress created"));
}

#[test]
fn test_parse_bulk_update_mutation() {
    let parser = DgraphResponseParser::new();
    
    // Create a bulk update mutation response
    let response = serde_json::json!({
        "data": {
            "updateConcepts": {
                "concepts": [
                    {
                        "id": TEST_UUID_1,
                        "name": "Updated Concept 1",
                        "difficulty": "intermediate",
                        "category": "Programming",
                        "qualityScore": 0.85,
                        "tags": ["updated"],
                        "embeddings": [0.1],
                        "createdAt": TEST_TIMESTAMP,
                        "updatedAt": TEST_TIMESTAMP_UPDATED,
                        "version": 2
                    },
                    {
                        "id": TEST_UUID_2,
                        "name": "Updated Concept 2",
                        "difficulty": "advanced",
                        "category": "Programming",
                        "qualityScore": 0.9,
                        "tags": ["updated", "advanced"],
                        "embeddings": [0.2],
                        "createdAt": TEST_TIMESTAMP,
                        "updatedAt": TEST_TIMESTAMP_UPDATED,
                        "version": 3
                    }
                ],
                "numUids": 2,
                "conflicts": [
                    {
                        "field": "version",
                        "existingValue": 1,
                        "attemptedValue": 2,
                        "resolution": "incremented"
                    }
                ]
            }
        }
    });

    let result = parser.parse_mutation_result(response);
    assert!(result.is_ok(), "Failed to parse bulk update mutation: {:?}", result.err());

    let mutation_result = result.unwrap();
    assert!(mutation_result.success);
    assert_eq!(mutation_result.operation_type, MutationOperationType::Bulk);
    assert_eq!(mutation_result.affected_count, 2);
    assert_eq!(mutation_result.affected_ids.len(), 2);
    assert!(mutation_result.message.as_ref().unwrap().contains("Updated 2 concepts"));
    assert_eq!(mutation_result.conflicts.len(), 1);
}

#[test]
fn test_parse_bulk_delete_mutation() {
    let parser = DgraphResponseParser::new();
    
    // Create a bulk delete mutation response
    let response = serde_json::json!({
        "data": {
            "deleteConcepts": {
                "msg": "Bulk deletion completed successfully",
                "numUids": 3,
                "deletedUids": ["0x123", "0x124", "0x125"],
                "deletedIds": [TEST_UUID_1, TEST_UUID_2, TEST_UUID_3]
            }
        }
    });

    let result = parser.parse_mutation_result(response);
    assert!(result.is_ok(), "Failed to parse bulk delete mutation: {:?}", result.err());

    let mutation_result = result.unwrap();
    assert!(mutation_result.success);
    assert_eq!(mutation_result.operation_type, MutationOperationType::Bulk);
    assert_eq!(mutation_result.affected_count, 3);
    assert_eq!(mutation_result.affected_ids.len(), 3);
    assert_eq!(mutation_result.uids.len(), 3);
    assert!(mutation_result.message.as_ref().unwrap().contains("deleted 3 items"));
}

#[test]
fn test_parse_failed_delete_mutation() {
    let parser = DgraphResponseParser::new();
    
    // Create a failed delete mutation response
    let response = serde_json::json!({
        "data": {
            "deleteConcept": {
                "msg": "No matching concepts found for deletion",
                "numUids": 0,
                "deletedUids": [],
                "deletedIds": []
            }
        }
    });

    let result = parser.parse_mutation_result(response);
    assert!(result.is_ok(), "Failed to parse failed delete mutation: {:?}", result.err());

    let mutation_result = result.unwrap();
    assert!(!mutation_result.success); // Should be false when numUids is 0
    assert_eq!(mutation_result.operation_type, MutationOperationType::Delete);
    assert_eq!(mutation_result.affected_count, 0);
    assert_eq!(mutation_result.affected_ids.len(), 0);
    assert_eq!(mutation_result.uids.len(), 0);
}

#[test]
fn test_operation_type_detection_from_message() {
    let parser = DgraphResponseParser::new();

    // Test add operation detection
    let add_response = serde_json::json!({
        "data": {
            "customMutation": {
                "success": true,
                "message": "Successfully added new item",
                "numUids": 1
            }
        }
    });

    let result = parser.parse_mutation_result(add_response);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().operation_type, MutationOperationType::Add);

    // Test update operation detection
    let update_response = serde_json::json!({
        "data": {
            "customMutation": {
                "success": true,
                "message": "Item updated successfully",
                "numUids": 1
            }
        }
    });

    let result = parser.parse_mutation_result(update_response);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().operation_type, MutationOperationType::Update);

    // Test delete operation detection
    let delete_response = serde_json::json!({
        "data": {
            "customMutation": {
                "success": true,
                "message": "Item deleted successfully",
                "numUids": 1
            }
        }
    });

    let result = parser.parse_mutation_result(delete_response);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().operation_type, MutationOperationType::Delete);
}

#[test]
fn test_parse_mutation_with_complex_data() {
    let parser = DgraphResponseParser::new();
    
    // Create a mutation with complex nested data
    let response = serde_json::json!({
        "data": {
            "complexMutation": {
                "success": true,
                "message": "Complex operation completed",
                "id": TEST_UUID_1,
                "uid": "0x999",
                "numUids": 1,
                "additionalData": {
                    "relationships": ["prerequisite", "related"],
                    "metadata": {
                        "score": 0.95,
                        "validated": true,
                        "tags": ["complex", "nested"]
                    },
                    "timestamps": {
                        "started": TEST_TIMESTAMP,
                        "completed": TEST_TIMESTAMP_UPDATED
                    }
                }
            }
        }
    });

    let result = parser.parse_mutation_result(response);
    assert!(result.is_ok(), "Failed to parse complex mutation: {:?}", result.err());

    let mutation_result = result.unwrap();
    assert!(mutation_result.success);
    assert_eq!(mutation_result.affected_ids.len(), 1);
    assert_eq!(mutation_result.uids, vec!["0x999"]);
    
    // Verify complex data is preserved
    assert!(mutation_result.data.is_some());
    let data = mutation_result.data.unwrap();
    assert!(data.get("additionalData").is_some());
    assert!(data["additionalData"].get("metadata").is_some());
    assert_eq!(data["additionalData"]["metadata"]["score"], 0.95);
}