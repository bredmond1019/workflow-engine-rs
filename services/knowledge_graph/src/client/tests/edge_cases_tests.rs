//! Tests for edge cases in response parsing
//!
//! These tests cover unusual but valid scenarios like empty results,
//! boundary values, performance with large datasets, and various
//! edge cases that might occur in real-world usage.

use super::fixtures::*;
use crate::client::dgraph::DgraphResponseParser;
use std::collections::HashMap;

#[test]
fn test_empty_search_results() {
    let parser = DgraphResponseParser::new();
    let response = TestFixtures::empty_data_response();

    let result = parser.parse_concepts_from_search_result(response, None);
    assert!(result.is_ok(), "Should handle empty results gracefully: {:?}", result.err());

    let concepts = result.unwrap();
    assert_eq!(concepts.len(), 0);
}

#[test]
fn test_zero_limit_search() {
    let parser = DgraphResponseParser::new();
    let response = TestFixtures::search_concepts_response();

    let result = parser.parse_concepts_from_search_result(response, Some(0));
    assert!(result.is_ok(), "Should handle zero limit: {:?}", result.err());

    let concepts = result.unwrap();
    assert_eq!(concepts.len(), 0);
}

#[test]
fn test_limit_larger_than_results() {
    let parser = DgraphResponseParser::new();
    let response = TestFixtures::search_concepts_response();

    let result = parser.parse_concepts_from_search_result(response, Some(1000));
    assert!(result.is_ok(), "Should handle large limit: {:?}", result.err());

    let concepts = result.unwrap();
    assert_eq!(concepts.len(), 3); // Only 3 concepts in fixture
}

#[test]
fn test_concept_with_empty_arrays() {
    let parser = DgraphResponseParser::new();
    
    let response = serde_json::json!({
        "data": {
            "concept": {
                "id": TEST_UUID_1,
                "name": "Empty Arrays Concept",
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
    assert!(result.is_ok(), "Should handle empty arrays: {:?}", result.err());

    let concept = result.unwrap();
    assert_eq!(concept.tags.len(), 0);
    assert_eq!(concept.embeddings.len(), 0);
}

#[test]
fn test_concept_with_very_long_arrays() {
    let parser = DgraphResponseParser::new();
    
    // Generate large arrays
    let large_tags: Vec<String> = (0..1000).map(|i| format!("tag{}", i)).collect();
    let large_embeddings: Vec<f32> = (0..10000).map(|i| i as f32 / 1000.0).collect();
    
    let response = serde_json::json!({
        "data": {
            "concept": {
                "id": TEST_UUID_1,
                "name": "Large Arrays Concept",
                "difficulty": "beginner",
                "category": "Test",
                "qualityScore": 0.5,
                "tags": large_tags,
                "embeddings": large_embeddings,
                "createdAt": TEST_TIMESTAMP,
                "updatedAt": TEST_TIMESTAMP,
                "version": 1
            }
        }
    });

    let result = parser.parse_concept_from_result(response);
    assert!(result.is_ok(), "Should handle large arrays: {:?}", result.err());

    let concept = result.unwrap();
    assert_eq!(concept.tags.len(), 1000);
    assert_eq!(concept.embeddings.len(), 10000);
}

#[test]
fn test_concept_with_very_long_strings() {
    let parser = DgraphResponseParser::new();
    
    let very_long_string = "a".repeat(100000);
    
    let response = serde_json::json!({
        "data": {
            "concept": {
                "id": TEST_UUID_1,
                "name": very_long_string,
                "description": very_long_string,
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
    assert!(result.is_ok(), "Should handle very long strings: {:?}", result.err());

    let concept = result.unwrap();
    assert_eq!(concept.name.len(), 100000);
    assert_eq!(concept.description.as_ref().unwrap().len(), 100000);
}

#[test]
fn test_concept_with_boundary_numeric_values() {
    let parser = DgraphResponseParser::new();
    
    let response = serde_json::json!({
        "data": {
            "concept": {
                "id": TEST_UUID_1,
                "name": "Boundary Values",
                "difficulty": "beginner",
                "category": "Test",
                "qualityScore": 0.0, // Minimum quality score
                "estimatedTime": f32::MAX, // Maximum float value
                "tags": [],
                "embeddings": [f32::MIN, f32::MAX, 0.0, -0.0, 1.0, -1.0],
                "createdAt": TEST_TIMESTAMP,
                "updatedAt": TEST_TIMESTAMP,
                "version": i32::MAX // Maximum version
            }
        }
    });

    let result = parser.parse_concept_from_result(response);
    assert!(result.is_ok(), "Should handle boundary numeric values: {:?}", result.err());

    let concept = result.unwrap();
    assert_eq!(concept.quality_score, 0.0);
    assert_eq!(concept.estimated_time, Some(f32::MAX));
    assert_eq!(concept.version, i32::MAX);
    assert_eq!(concept.embeddings.len(), 6);
}

#[test]
fn test_concept_with_unicode_strings() {
    let parser = DgraphResponseParser::new();
    
    let response = serde_json::json!({
        "data": {
            "concept": {
                "id": TEST_UUID_1,
                "name": "🦀 Rust Programming 中文 العربية 🚀",
                "description": "Unicode test: Ψώδαξ λόγους άλλα έτερη 🌍",
                "difficulty": "beginner",
                "category": "Programming",
                "subcategory": "系统编程",
                "qualityScore": 0.8,
                "tags": ["🦀", "rust", "中文", "العربية", "Русский"],
                "embeddings": [0.1, 0.2],
                "createdAt": TEST_TIMESTAMP,
                "updatedAt": TEST_TIMESTAMP,
                "version": 1
            }
        }
    });

    let result = parser.parse_concept_from_result(response);
    assert!(result.is_ok(), "Should handle Unicode strings: {:?}", result.err());

    let concept = result.unwrap();
    assert_eq!(concept.name, "🦀 Rust Programming 中文 العربية 🚀");
    assert!(concept.description.as_ref().unwrap().contains("🌍"));
    assert_eq!(concept.subcategory, Some("系统编程".to_string()));
    assert!(concept.tags.contains(&"🦀".to_string()));
}

#[test]
fn test_concept_with_special_characters() {
    let parser = DgraphResponseParser::new();
    
    let response = serde_json::json!({
        "data": {
            "concept": {
                "id": TEST_UUID_1,
                "name": "Special chars: @#$%^&*()_+-=[]{}|;':\",./<>?",
                "description": "Quotes: \"single\" and 'double', backslashes: \\n\\t\\r",
                "difficulty": "beginner",
                "category": "Test",
                "qualityScore": 0.5,
                "tags": ["@symbol", "#hash", "$dollar", "%percent"],
                "embeddings": [],
                "createdAt": TEST_TIMESTAMP,
                "updatedAt": TEST_TIMESTAMP,
                "version": 1
            }
        }
    });

    let result = parser.parse_concept_from_result(response);
    assert!(result.is_ok(), "Should handle special characters: {:?}", result.err());

    let concept = result.unwrap();
    assert!(concept.name.contains("@#$%^&*"));
    assert!(concept.description.as_ref().unwrap().contains("\\n"));
    assert!(concept.tags.contains(&"@symbol".to_string()));
}

#[test]
fn test_timestamp_edge_cases() {
    let parser = DgraphResponseParser::new();
    
    // Test with various timestamp formats
    let test_cases = vec![
        "1970-01-01T00:00:00Z", // Unix epoch
        "2038-01-19T03:14:07Z", // 32-bit timestamp limit
        "2024-02-29T23:59:59.999Z", // Leap year, end of day with milliseconds
        "2024-12-31T23:59:59.999999Z", // End of year with microseconds
    ];

    for timestamp in test_cases {
        let response = serde_json::json!({
            "data": {
                "concept": {
                    "id": TEST_UUID_1,
                    "name": format!("Timestamp Test: {}", timestamp),
                    "difficulty": "beginner",
                    "category": "Test",
                    "qualityScore": 0.5,
                    "tags": [],
                    "embeddings": [],
                    "createdAt": timestamp,
                    "updatedAt": timestamp,
                    "version": 1
                }
            }
        });

        let result = parser.parse_concept_from_result(response);
        assert!(result.is_ok(), "Should handle timestamp {}: {:?}", timestamp, result.err());
    }
}

#[test]
fn test_complex_alias_resolution() {
    let parser = DgraphResponseParser::new();
    
    let mut response = serde_json::json!({
        "data": {
            "first:concept": { "name": "First" },
            "second:concept": { "name": "Second" },
            "nested:field:concept": { "name": "Nested" },
            "simple": { "name": "No Alias" }
        }
    });

    let result = parser.resolve_aliases(&mut response);
    assert!(result.is_ok(), "Should resolve complex aliases: {:?}", result.err());

    let data = response.get("data").unwrap();
    assert!(data.get("first").is_some());
    assert!(data.get("second").is_some());
    assert!(data.get("nested").is_some());
    assert!(data.get("simple").is_some());
}

#[test]
fn test_fragment_expansion() {
    let parser = DgraphResponseParser::new();
    
    let mut response = serde_json::json!({
        "data": {
            "concept": {
                "id": TEST_UUID_1,
                "name": "Test",
                "...basicInfo": {},
                "...extendedInfo": {}
            }
        }
    });

    let mut fragments = HashMap::new();
    fragments.insert("basicInfo".to_string(), serde_json::json!({
        "difficulty": "beginner",
        "category": "Test"
    }));
    fragments.insert("extendedInfo".to_string(), serde_json::json!({
        "qualityScore": 0.8,
        "tags": ["fragment", "test"]
    }));

    let result = parser.expand_fragments(&mut response, &fragments);
    assert!(result.is_ok(), "Should expand fragments: {:?}", result.err());

    let concept = response["data"]["concept"].clone();
    assert_eq!(concept["difficulty"], "beginner");
    assert_eq!(concept["category"], "Test");
    assert_eq!(concept["qualityScore"], 0.8);
}

#[test]
fn test_nested_relationship_deduplication() {
    let parser = DgraphResponseParser::new();
    
    // Concept that appears multiple times in different relationships
    let duplicate_concept = serde_json::json!({
        "id": TEST_UUID_2,
        "name": "Duplicate Concept",
        "difficulty": "beginner",
        "category": "Test",
        "qualityScore": 0.7,
        "tags": [],
        "embeddings": [],
        "createdAt": TEST_TIMESTAMP,
        "updatedAt": TEST_TIMESTAMP,
        "version": 1
    });

    let response = serde_json::json!({
        "data": {
            "concept": {
                "id": TEST_UUID_1,
                "name": "Main Concept",
                "difficulty": "intermediate",
                "category": "Test",
                "qualityScore": 0.8,
                "tags": [],
                "embeddings": [],
                "createdAt": TEST_TIMESTAMP,
                "updatedAt": TEST_TIMESTAMP,
                "version": 1,
                "prerequisites": [duplicate_concept.clone()],
                "relatedTo": [duplicate_concept.clone()],
                "enabledBy": [duplicate_concept.clone()]
            }
        }
    });

    let result = parser.parse_concepts_from_graph_result(response);
    assert!(result.is_ok(), "Should handle duplicate concepts: {:?}", result.err());

    let concepts = result.unwrap();
    // Should be deduplicated: main concept + duplicate concept = 2 total
    assert_eq!(concepts.len(), 2);
    
    let duplicate_count = concepts.iter()
        .filter(|c| c.name == "Duplicate Concept")
        .count();
    assert_eq!(duplicate_count, 1, "Duplicate concept should appear only once");
}

#[test]
fn test_mutation_with_zero_affected_items() {
    let parser = DgraphResponseParser::new();
    
    let response = serde_json::json!({
        "data": {
            "updateConcept": {
                "concept": {
                    "id": TEST_UUID_1,
                    "name": "Unchanged",
                    "difficulty": "beginner",
                    "category": "Test",
                    "qualityScore": 0.5,
                    "tags": [],
                    "embeddings": [],
                    "createdAt": TEST_TIMESTAMP,
                    "updatedAt": TEST_TIMESTAMP,
                    "version": 1
                },
                "numUids": 0 // No actual changes made
            }
        }
    });

    let result = parser.parse_mutation_result(response);
    assert!(result.is_ok(), "Should handle zero affected items: {:?}", result.err());

    let mutation_result = result.unwrap();
    assert!(mutation_result.success); // Should still be considered successful
    assert_eq!(mutation_result.affected_count, 0);
}

#[test]
fn test_large_performance_benchmark() {
    let parser = DgraphResponseParser::new();
    let response = TestFixtures::large_array_response();

    // Test parsing performance with large datasets
    let iterations = 10;
    let mut total_duration = std::time::Duration::new(0, 0);
    
    for _ in 0..iterations {
        let start = std::time::Instant::now();
        let result = parser.parse_concepts_from_search_result(response.clone(), None);
        let duration = start.elapsed();
        
        assert!(result.is_ok(), "Large dataset parsing should succeed");
        total_duration += duration;
    }
    
    let avg_duration = total_duration / iterations;
    assert!(avg_duration.as_millis() < 500, "Average parsing time should be under 500ms, got {:?}", avg_duration);
}

#[test]
fn test_concurrent_parsing() {
    use std::sync::Arc;
    use std::thread;
    
    let parser = Arc::new(DgraphResponseParser::new());
    let response = Arc::new(TestFixtures::search_concepts_response());
    
    let mut handles = vec![];
    
    // Spawn multiple threads to test concurrent parsing
    for i in 0..10 {
        let parser_clone = Arc::clone(&parser);
        let response_clone = Arc::clone(&response);
        
        let handle = thread::spawn(move || {
            let result = parser_clone.parse_concepts_from_search_result((*response_clone).clone(), Some(i % 3 + 1));
            assert!(result.is_ok(), "Concurrent parsing should succeed");
            result.unwrap().len()
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        let count = handle.join().unwrap();
        assert!(count <= 3, "Should not exceed maximum concepts in fixture");
    }
}

#[test]
fn test_deeply_nested_null_handling() {
    let parser = DgraphResponseParser::new();
    
    let response = serde_json::json!({
        "data": {
            "concept": {
                "id": TEST_UUID_1,
                "name": "Null Test",
                "description": null,
                "difficulty": "beginner",
                "category": "Test",
                "subcategory": null,
                "qualityScore": 0.5,
                "estimatedTime": null,
                "tags": [],
                "embeddings": [],
                "createdAt": TEST_TIMESTAMP,
                "updatedAt": TEST_TIMESTAMP,
                "version": 1,
                "prerequisites": null,
                "relatedTo": null
            }
        }
    });

    let result = parser.parse_concept_from_result(response);
    assert!(result.is_ok(), "Should handle nested nulls: {:?}", result.err());

    let concept = result.unwrap();
    assert_eq!(concept.description, None);
    assert_eq!(concept.subcategory, None);
    assert_eq!(concept.estimated_time, None);
}

#[test]
fn test_mutation_result_with_empty_conflicts() {
    let parser = DgraphResponseParser::new();
    
    let response = serde_json::json!({
        "data": {
            "updateConcept": {
                "concept": {
                    "id": TEST_UUID_1,
                    "name": "No Conflicts",
                    "difficulty": "beginner",
                    "category": "Test",
                    "qualityScore": 0.5,
                    "tags": [],
                    "embeddings": [],
                    "createdAt": TEST_TIMESTAMP,
                    "updatedAt": TEST_TIMESTAMP,
                    "version": 1
                },
                "numUids": 1,
                "conflicts": []
            }
        }
    });

    let result = parser.parse_mutation_result(response);
    assert!(result.is_ok(), "Should handle empty conflicts: {:?}", result.err());

    let mutation_result = result.unwrap();
    assert_eq!(mutation_result.conflicts.len(), 0);
}

#[test] 
fn test_parse_with_unexpected_additional_fields() {
    let parser = DgraphResponseParser::new();
    
    // Response with extra fields that aren't part of the schema
    let response = serde_json::json!({
        "data": {
            "concept": {
                "id": TEST_UUID_1,
                "name": "Extra Fields",
                "difficulty": "beginner",
                "category": "Test",
                "qualityScore": 0.5,
                "tags": [],
                "embeddings": [],
                "createdAt": TEST_TIMESTAMP,
                "updatedAt": TEST_TIMESTAMP,
                "version": 1,
                "unexpectedField": "this should be ignored",
                "anotherExtra": 42,
                "nestedExtra": {
                    "field": "value"
                }
            }
        }
    });

    let result = parser.parse_concept_from_result(response);
    assert!(result.is_ok(), "Should ignore unexpected fields: {:?}", result.err());

    let concept = result.unwrap();
    assert_eq!(concept.name, "Extra Fields");
    // Extra fields should be ignored during deserialization
}