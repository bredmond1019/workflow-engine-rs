//! Test fixtures for GraphQL response parsing tests
//!
//! Contains realistic GraphQL response JSON samples for testing
//! various parsing scenarios including successful responses,
//! error responses, and edge cases.

use serde_json::{json, Value};
use uuid::Uuid;

/// Standard test UUID for consistent testing
pub const TEST_UUID_1: &str = "550e8400-e29b-41d4-a716-446655440000";
pub const TEST_UUID_2: &str = "550e8400-e29b-41d4-a716-446655440001";
pub const TEST_UUID_3: &str = "550e8400-e29b-41d4-a716-446655440002";
pub const TEST_UUID_4: &str = "550e8400-e29b-41d4-a716-446655440003";

/// Standard test timestamp
pub const TEST_TIMESTAMP: &str = "2024-01-01T00:00:00Z";
pub const TEST_TIMESTAMP_UPDATED: &str = "2024-01-02T00:00:00Z";

pub struct TestFixtures;

impl TestFixtures {
    /// Basic concept response
    pub fn simple_concept_response() -> Value {
        json!({
            "data": {
                "concept": {
                    "id": TEST_UUID_1,
                    "name": "Rust Programming",
                    "description": "A systems programming language that runs blazingly fast",
                    "difficulty": "intermediate",
                    "category": "Programming",
                    "subcategory": "Systems Programming",
                    "tags": ["rust", "systems", "programming", "memory-safe"],
                    "qualityScore": 0.85,
                    "estimatedTime": 120.5,
                    "embeddings": [0.1, 0.2, 0.3, 0.4, 0.5],
                    "createdAt": TEST_TIMESTAMP,
                    "updatedAt": TEST_TIMESTAMP,
                    "version": 1
                }
            }
        })
    }

    /// Concept response with minimal required fields only
    pub fn minimal_concept_response() -> Value {
        json!({
            "data": {
                "concept": {
                    "id": TEST_UUID_1,
                    "name": "Minimal Concept",
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
        })
    }

    /// Concept response with null values for optional fields
    pub fn concept_with_nulls_response() -> Value {
        json!({
            "data": {
                "concept": {
                    "id": TEST_UUID_1,
                    "name": "Concept with Nulls",
                    "description": null,
                    "difficulty": "beginner",
                    "category": "Test",
                    "subcategory": null,
                    "tags": [],
                    "qualityScore": 0.5,
                    "estimatedTime": null,
                    "embeddings": [],
                    "createdAt": TEST_TIMESTAMP,
                    "updatedAt": TEST_TIMESTAMP,
                    "version": 1
                }
            }
        })
    }

    /// Multiple concepts in search result
    pub fn search_concepts_response() -> Value {
        json!({
            "data": {
                "concepts": [
                    {
                        "id": TEST_UUID_1,
                        "name": "First Concept",
                        "difficulty": "beginner",
                        "category": "Programming",
                        "qualityScore": 0.7,
                        "tags": ["basic", "intro"],
                        "embeddings": [0.1, 0.2],
                        "createdAt": TEST_TIMESTAMP,
                        "updatedAt": TEST_TIMESTAMP,
                        "version": 1
                    },
                    {
                        "id": TEST_UUID_2,
                        "name": "Second Concept",
                        "difficulty": "intermediate",
                        "category": "Programming",
                        "subcategory": "Web Development",
                        "qualityScore": 0.8,
                        "tags": ["web", "frontend"],
                        "embeddings": [0.3, 0.4],
                        "createdAt": TEST_TIMESTAMP,
                        "updatedAt": TEST_TIMESTAMP,
                        "version": 1
                    },
                    {
                        "id": TEST_UUID_3,
                        "name": "Third Concept",
                        "description": "Advanced concept with full metadata",
                        "difficulty": "advanced",
                        "category": "Data Science",
                        "subcategory": "Machine Learning",
                        "qualityScore": 0.95,
                        "estimatedTime": 240.0,
                        "tags": ["ml", "algorithms", "advanced"],
                        "embeddings": [0.5, 0.6, 0.7, 0.8],
                        "createdAt": TEST_TIMESTAMP,
                        "updatedAt": TEST_TIMESTAMP_UPDATED,
                        "version": 2
                    }
                ]
            }
        })
    }

    /// Search result with alternative field names
    pub fn search_with_alternative_fields() -> Value {
        json!({
            "data": {
                "queryConcept": [
                    {
                        "id": TEST_UUID_1,
                        "name": "Query Concept",
                        "difficulty": "beginner",
                        "category": "Test",
                        "qualityScore": 0.6,
                        "tags": [],
                        "embeddings": [],
                        "createdAt": TEST_TIMESTAMP,
                        "updatedAt": TEST_TIMESTAMP,
                        "version": 1
                    }
                ]
            }
        })
    }

    /// Concept with complex nested relationships
    pub fn concept_with_relationships() -> Value {
        json!({
            "data": {
                "concept": {
                    "id": TEST_UUID_1,
                    "name": "Main Concept",
                    "difficulty": "intermediate",
                    "category": "Programming",
                    "qualityScore": 0.8,
                    "tags": ["main", "relationships"],
                    "embeddings": [0.1, 0.2, 0.3],
                    "createdAt": TEST_TIMESTAMP,
                    "updatedAt": TEST_TIMESTAMP,
                    "version": 1,
                    "prerequisites": [
                        {
                            "id": TEST_UUID_2,
                            "name": "Prerequisite 1",
                            "difficulty": "beginner",
                            "category": "Programming",
                            "qualityScore": 0.7,
                            "tags": ["prerequisite"],
                            "embeddings": [0.1],
                            "createdAt": TEST_TIMESTAMP,
                            "updatedAt": TEST_TIMESTAMP,
                            "version": 1
                        },
                        {
                            "id": TEST_UUID_3,
                            "name": "Prerequisite 2",
                            "difficulty": "beginner",
                            "category": "Programming",
                            "qualityScore": 0.75,
                            "tags": ["prerequisite", "foundation"],
                            "embeddings": [0.2],
                            "createdAt": TEST_TIMESTAMP,
                            "updatedAt": TEST_TIMESTAMP,
                            "version": 1
                        }
                    ],
                    "relatedTo": [
                        {
                            "id": TEST_UUID_4,
                            "name": "Related Concept",
                            "difficulty": "intermediate",
                            "category": "Programming",
                            "qualityScore": 0.82,
                            "tags": ["related"],
                            "embeddings": [0.3],
                            "createdAt": TEST_TIMESTAMP,
                            "updatedAt": TEST_TIMESTAMP,
                            "version": 1
                        }
                    ],
                    "subtopics": [],
                    "enabledBy": []
                }
            }
        })
    }

    /// Learning resource response
    pub fn learning_resource_response() -> Value {
        json!({
            "data": {
                "resources": [
                    {
                        "id": TEST_UUID_1,
                        "url": "https://example.com/rust-tutorial",
                        "title": "Complete Rust Tutorial",
                        "resourceType": "tutorial",
                        "format": "video",
                        "source": "YouTube",
                        "quality": 0.9,
                        "difficulty": "beginner",
                        "duration": 3600,
                        "language": "English",
                        "createdAt": TEST_TIMESTAMP,
                        "updatedAt": TEST_TIMESTAMP
                    },
                    {
                        "id": TEST_UUID_2,
                        "url": "https://doc.rust-lang.org/book/",
                        "title": "The Rust Programming Language",
                        "resourceType": "documentation",
                        "format": "text",
                        "source": "Official Docs",
                        "quality": 0.95,
                        "difficulty": "intermediate",
                        "duration": null,
                        "language": "English",
                        "createdAt": TEST_TIMESTAMP,
                        "updatedAt": TEST_TIMESTAMP
                    }
                ]
            }
        })
    }

    /// Learning path response
    pub fn learning_path_response() -> Value {
        json!({
            "data": {
                "learningPath": {
                    "id": TEST_UUID_1,
                    "name": "Rust Mastery Path",
                    "description": "Complete path to master Rust programming",
                    "targetAudience": "Intermediate developers",
                    "estimatedTime": 480.0,
                    "difficultyProgression": "beginner -> intermediate -> advanced",
                    "learningOutcomes": [
                        "Understand ownership and borrowing",
                        "Write safe concurrent code",
                        "Build web applications with Rust"
                    ],
                    "creator": "Rust Expert",
                    "isCustom": false,
                    "createdAt": TEST_TIMESTAMP,
                    "updatedAt": TEST_TIMESTAMP
                }
            }
        })
    }

    /// User progress response
    pub fn user_progress_response() -> Value {
        json!({
            "data": {
                "progress": [
                    {
                        "id": TEST_UUID_1,
                        "userId": "user123",
                        "conceptId": TEST_UUID_2,
                        "status": "in_progress",
                        "percentComplete": 65.5,
                        "timeSpent": 1800,
                        "resourcesCompleted": 3,
                        "difficultyRating": 3.5,
                        "notes": "Making good progress, need to review ownership concepts",
                        "startedAt": TEST_TIMESTAMP,
                        "completedAt": null,
                        "lastAccessedAt": TEST_TIMESTAMP_UPDATED
                    },
                    {
                        "id": TEST_UUID_3,
                        "userId": "user123",
                        "conceptId": TEST_UUID_4,
                        "status": "completed",
                        "percentComplete": 100.0,
                        "timeSpent": 2400,
                        "resourcesCompleted": 5,
                        "difficultyRating": 2.0,
                        "notes": "Completed successfully",
                        "startedAt": TEST_TIMESTAMP,
                        "completedAt": TEST_TIMESTAMP_UPDATED,
                        "lastAccessedAt": TEST_TIMESTAMP_UPDATED
                    }
                ]
            }
        })
    }

    /// Successful add mutation result
    pub fn add_concept_mutation_success() -> Value {
        json!({
            "data": {
                "addConcept": {
                    "concept": {
                        "id": TEST_UUID_1,
                        "name": "New Concept",
                        "difficulty": "beginner",
                        "category": "Programming",
                        "qualityScore": 0.8,
                        "tags": ["new", "test"],
                        "embeddings": [0.1, 0.2],
                        "createdAt": TEST_TIMESTAMP,
                        "updatedAt": TEST_TIMESTAMP,
                        "version": 1
                    },
                    "numUids": 1,
                    "uid": "0x123abc"
                }
            }
        })
    }

    /// Successful update mutation with conflicts
    pub fn update_concept_mutation_with_conflicts() -> Value {
        json!({
            "data": {
                "updateConcept": {
                    "concept": {
                        "id": TEST_UUID_1,
                        "name": "Updated Concept",
                        "difficulty": "intermediate",
                        "category": "Programming",
                        "qualityScore": 0.9,
                        "tags": ["updated", "enhanced"],
                        "embeddings": [0.2, 0.3, 0.4],
                        "createdAt": TEST_TIMESTAMP,
                        "updatedAt": TEST_TIMESTAMP_UPDATED,
                        "version": 2
                    },
                    "numUids": 1,
                    "conflicts": [
                        {
                            "field": "version",
                            "existingValue": 1,
                            "attemptedValue": 1,
                            "resolution": "incremented"
                        },
                        {
                            "field": "qualityScore",
                            "existingValue": 0.7,
                            "attemptedValue": 0.9,
                            "resolution": "overwritten"
                        }
                    ]
                }
            }
        })
    }

    /// Successful delete mutation result
    pub fn delete_concept_mutation_success() -> Value {
        json!({
            "data": {
                "deleteConcept": {
                    "msg": "Successfully deleted concept",
                    "numUids": 1,
                    "deletedUids": ["0x123abc"],
                    "deletedIds": [TEST_UUID_1]
                }
            }
        })
    }

    /// Bulk add mutation result
    pub fn bulk_add_concepts_success() -> Value {
        json!({
            "data": {
                "addConcepts": {
                    "concepts": [
                        {
                            "id": TEST_UUID_1,
                            "name": "Bulk Concept 1",
                            "difficulty": "beginner",
                            "category": "Test",
                            "qualityScore": 0.7,
                            "tags": ["bulk", "test"],
                            "embeddings": [],
                            "createdAt": TEST_TIMESTAMP,
                            "updatedAt": TEST_TIMESTAMP,
                            "version": 1
                        },
                        {
                            "id": TEST_UUID_2,
                            "name": "Bulk Concept 2",
                            "difficulty": "intermediate",
                            "category": "Test",
                            "qualityScore": 0.8,
                            "tags": ["bulk", "test", "advanced"],
                            "embeddings": [0.1],
                            "createdAt": TEST_TIMESTAMP,
                            "updatedAt": TEST_TIMESTAMP,
                            "version": 1
                        }
                    ],
                    "numUids": 2
                }
            }
        })
    }

    /// Raw DGraph mutation result
    pub fn raw_dgraph_mutation_success() -> Value {
        json!({
            "data": {
                "mutate": {
                    "code": "Success",
                    "message": "Mutation applied successfully",
                    "uids": {
                        "concept1": "0x123",
                        "concept2": "0x124",
                        "resource1": "0x125"
                    },
                    "queries": {
                        "q": [
                            { "count": 3 }
                        ]
                    }
                }
            }
        })
    }

    /// Generic custom mutation result
    pub fn generic_mutation_result() -> Value {
        json!({
            "data": {
                "customOperation": {
                    "success": true,
                    "message": "Custom operation completed successfully",
                    "id": TEST_UUID_1,
                    "uid": "0x999",
                    "numUids": 1,
                    "metadata": {
                        "operation": "custom_update",
                        "timestamp": TEST_TIMESTAMP_UPDATED
                    }
                }
            }
        })
    }

    /// Response with GraphQL errors
    pub fn graphql_errors_response() -> Value {
        json!({
            "errors": [
                {
                    "message": "Variable $conceptId is not defined",
                    "path": ["getConcept"],
                    "extensions": {
                        "code": "GRAPHQL_VALIDATION_FAILED",
                        "line": 2,
                        "column": 15
                    }
                },
                {
                    "message": "Field 'invalidField' doesn't exist on type 'Concept'",
                    "path": ["getConcept", "invalidField"],
                    "extensions": {
                        "code": "GRAPHQL_FIELD_ERROR"
                    }
                }
            ]
        })
    }

    /// Response with both data and errors (partial success)
    pub fn partial_success_response() -> Value {
        json!({
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
                    }
                ]
            },
            "errors": [
                {
                    "message": "Some concepts could not be retrieved",
                    "path": ["concepts", 1],
                    "extensions": {
                        "code": "PARTIAL_FAILURE"
                    }
                }
            ]
        })
    }

    /// Empty data response
    pub fn empty_data_response() -> Value {
        json!({
            "data": {
                "concepts": []
            }
        })
    }

    /// Missing data field
    pub fn missing_data_field() -> Value {
        json!({
            "extensions": {
                "tracing": {
                    "version": 1
                }
            }
        })
    }

    /// Malformed data structure
    pub fn malformed_data_response() -> Value {
        json!({
            "data": {
                "concept": {
                    "id": "not-a-uuid",
                    "name": 123,
                    "difficulty": null,
                    "category": "",
                    "qualityScore": "not-a-number",
                    "tags": "not-an-array",
                    "embeddings": [1, 2, "not-a-number"],
                    "createdAt": "invalid-date",
                    "updatedAt": "2024-13-45T99:99:99Z",
                    "version": "not-an-integer"
                }
            }
        })
    }

    /// Response with GraphQL aliases
    pub fn aliased_response() -> Value {
        json!({
            "data": {
                "fromConcept:concept": {
                    "id": TEST_UUID_1,
                    "name": "From Concept",
                    "difficulty": "intermediate",
                    "category": "Test",
                    "qualityScore": 0.8,
                    "tags": [],
                    "embeddings": [],
                    "createdAt": TEST_TIMESTAMP,
                    "updatedAt": TEST_TIMESTAMP,
                    "version": 1
                },
                "toConcept:concept": {
                    "id": TEST_UUID_2,
                    "name": "To Concept",
                    "difficulty": "advanced",
                    "category": "Test",
                    "qualityScore": 0.9,
                    "tags": [],
                    "embeddings": [],
                    "createdAt": TEST_TIMESTAMP,
                    "updatedAt": TEST_TIMESTAMP,
                    "version": 1
                }
            }
        })
    }

    /// Response with very large arrays for performance testing
    pub fn large_array_response() -> Value {
        let mut concepts = Vec::new();
        for i in 0..100 { // Reduced from 1000 to 100 to avoid recursion limit
            concepts.push(json!({
                "id": format!("550e8400-e29b-41d4-a716-{:012}", i),
                "name": format!("Concept {}", i),
                "difficulty": if i % 3 == 0 { "beginner" } else if i % 3 == 1 { "intermediate" } else { "advanced" },
                "category": format!("Category {}", i % 10),
                "subcategory": if i % 2 == 0 { serde_json::Value::String(format!("Subcategory {}", i % 5)) } else { serde_json::Value::Null },
                "qualityScore": (i % 100) as f32 / 100.0,
                "tags": (0..std::cmp::min(i%5, 3)).map(|j| format!("tag{}", j)).collect::<Vec<_>>(),
                "embeddings": (0..std::cmp::min(i%10, 5)).map(|j| (j as f32) / 10.0).collect::<Vec<_>>(),
                "createdAt": TEST_TIMESTAMP,
                "updatedAt": TEST_TIMESTAMP,
                "version": 1
            }));
        }

        json!({
            "data": {
                "concepts": concepts
            }
        })
    }

    /// Response with deeply nested structures
    pub fn deeply_nested_response() -> Value {
        // Create nested prerequisites step by step to avoid recursion limit
        let level3 = json!({
            "id": TEST_UUID_4,
            "name": "Level 3 Prerequisite",
            "difficulty": "beginner",
            "category": "Programming",
            "qualityScore": 0.5,
            "tags": ["level3"],
            "embeddings": [0.4],
            "createdAt": TEST_TIMESTAMP,
            "updatedAt": TEST_TIMESTAMP,
            "version": 1
        });

        let level2 = json!({
            "id": TEST_UUID_3,
            "name": "Level 2 Prerequisite",
            "difficulty": "beginner",
            "category": "Programming",
            "qualityScore": 0.6,
            "tags": ["level2"],
            "embeddings": [0.3],
            "createdAt": TEST_TIMESTAMP,
            "updatedAt": TEST_TIMESTAMP,
            "version": 1,
            "prerequisites": [level3]
        });

        let level1 = json!({
            "id": TEST_UUID_2,
            "name": "Level 1 Prerequisite",
            "difficulty": "beginner",
            "category": "Programming",
            "qualityScore": 0.7,
            "tags": ["level1"],
            "embeddings": [0.2],
            "createdAt": TEST_TIMESTAMP,
            "updatedAt": TEST_TIMESTAMP,
            "version": 1,
            "prerequisites": [level2]
        });

        json!({
            "data": {
                "concept": {
                    "id": TEST_UUID_1,
                    "name": "Root Concept",
                    "difficulty": "intermediate",
                    "category": "Programming",
                    "qualityScore": 0.8,
                    "tags": ["root"],
                    "embeddings": [0.1],
                    "createdAt": TEST_TIMESTAMP,
                    "updatedAt": TEST_TIMESTAMP,
                    "version": 1,
                    "prerequisites": [level1]
                }
            }
        })
    }

    /// Response with timestamp variations (different formats)
    pub fn timestamp_variations_response() -> Value {
        json!({
            "data": {
                "concepts": [
                    {
                        "id": TEST_UUID_1,
                        "name": "ISO 8601 Timestamp",
                        "difficulty": "beginner",
                        "category": "Test",
                        "qualityScore": 0.7,
                        "tags": [],
                        "embeddings": [],
                        "createdAt": "2024-01-01T00:00:00Z",
                        "updatedAt": "2024-01-01T12:30:45.123Z",
                        "version": 1
                    },
                    {
                        "id": TEST_UUID_2,
                        "name": "Unix Timestamp",
                        "difficulty": "beginner",
                        "category": "Test",
                        "qualityScore": 0.7,
                        "tags": [],
                        "embeddings": [],
                        "createdAt": "1704067200",
                        "updatedAt": "1704067200",
                        "version": 1
                    }
                ]
            }
        })
    }

    /// Generate a UUID from a test index for consistent testing
    pub fn test_uuid(index: u32) -> Uuid {
        Uuid::parse_str(&format!("550e8400-e29b-41d4-a716-{:012}", index)).unwrap()
    }
}