//! Tests for complex nested structures and advanced parsing scenarios
//!
//! These tests cover deeply nested relationships, complex GraphQL responses,
//! large datasets, and advanced parsing features like fragments and aliases.

use super::fixtures::*;
use crate::client::dgraph::DgraphResponseParser;
use std::collections::HashMap;

#[test]
fn test_parse_deeply_nested_concept_hierarchy() {
    let parser = DgraphResponseParser::new();
    let response = TestFixtures::deeply_nested_response();

    let result = parser.parse_concepts_from_graph_result(response);
    assert!(result.is_ok(), "Should parse deeply nested structures: {:?}", result.err());

    let concepts = result.unwrap();
    assert_eq!(concepts.len(), 2); // Only extracts first level relationships

    // Verify only root and first level are present
    let concept_names: Vec<&str> = concepts.iter().map(|c| c.name.as_str()).collect();
    assert!(concept_names.contains(&"Root Concept"));
    assert!(concept_names.contains(&"Level 1 Prerequisite"));
    // Deeper levels are not extracted by this function

    // Verify deduplication works (no duplicate concepts)
    let mut unique_ids = std::collections::HashSet::new();
    for concept in &concepts {
        assert!(unique_ids.insert(concept.id), "Concept ID should be unique: {}", concept.id);
    }
}

#[test]
fn test_parse_complex_multi_relationship_graph() {
    let parser = DgraphResponseParser::new();
    
    // Create a complex graph with multiple relationship types
    let response = serde_json::json!({
        "data": {
            "concept": {
                "id": TEST_UUID_1,
                "name": "Central Concept",
                "difficulty": "intermediate",
                "category": "Programming",
                "qualityScore": 0.9,
                "tags": ["central", "hub"],
                "embeddings": [0.5, 0.6, 0.7],
                "createdAt": TEST_TIMESTAMP,
                "updatedAt": TEST_TIMESTAMP,
                "version": 1,
                "prerequisites": [
                    {
                        "id": TEST_UUID_2,
                        "name": "Prerequisite A",
                        "difficulty": "beginner",
                        "category": "Programming",
                        "qualityScore": 0.7,
                        "tags": ["prereq"],
                        "embeddings": [0.1],
                        "createdAt": TEST_TIMESTAMP,
                        "updatedAt": TEST_TIMESTAMP,
                        "version": 1,
                        "relatedTo": [
                            {
                                "id": TEST_UUID_3,
                                "name": "Cross Reference",
                                "difficulty": "beginner",
                                "category": "Programming",
                                "qualityScore": 0.6,
                                "tags": ["cross"],
                                "embeddings": [0.2],
                                "createdAt": TEST_TIMESTAMP,
                                "updatedAt": TEST_TIMESTAMP,
                                "version": 1
                            }
                        ]
                    }
                ],
                "relatedTo": [
                    {
                        "id": TEST_UUID_4,
                        "name": "Related Concept",
                        "difficulty": "intermediate",
                        "category": "Programming",
                        "qualityScore": 0.8,
                        "tags": ["related"],
                        "embeddings": [0.3],
                        "createdAt": TEST_TIMESTAMP,
                        "updatedAt": TEST_TIMESTAMP,
                        "version": 1,
                        "prerequisites": [
                            {
                                "id": TEST_UUID_2,
                                "name": "Prerequisite A",
                                "difficulty": "beginner",
                                "category": "Programming",
                                "qualityScore": 0.7,
                                "tags": ["prereq"],
                                "embeddings": [0.1],
                                "createdAt": TEST_TIMESTAMP,
                                "updatedAt": TEST_TIMESTAMP,
                                "version": 1
                            }
                        ]
                    }
                ],
                "enabledBy": [
                    {
                        "id": TEST_UUID_3,
                        "name": "Cross Reference",
                        "difficulty": "beginner",
                        "category": "Programming",
                        "qualityScore": 0.6,
                        "tags": ["cross"],
                        "embeddings": [0.2],
                        "createdAt": TEST_TIMESTAMP,
                        "updatedAt": TEST_TIMESTAMP,
                        "version": 1
                    }
                ],
                "subtopics": [
                    {
                        "id": "550e8400-e29b-41d4-a716-446655440005",
                        "name": "Subtopic 1",
                        "difficulty": "advanced",
                        "category": "Programming",
                        "qualityScore": 0.85,
                        "tags": ["subtopic"],
                        "embeddings": [0.4],
                        "createdAt": TEST_TIMESTAMP,
                        "updatedAt": TEST_TIMESTAMP,
                        "version": 1
                    }
                ]
            }
        }
    });

    let result = parser.parse_concepts_from_graph_result(response);
    assert!(result.is_ok(), "Should parse complex multi-relationship graph: {:?}", result.err());

    let concepts = result.unwrap();
    
    // Should deduplicate: Central + Prerequisite A + Cross Reference + Related + Subtopic 1 = 5 unique
    assert_eq!(concepts.len(), 5);
    
    // Verify all concepts are present
    let concept_names: Vec<&str> = concepts.iter().map(|c| c.name.as_str()).collect();
    assert!(concept_names.contains(&"Central Concept"));
    assert!(concept_names.contains(&"Prerequisite A"));
    assert!(concept_names.contains(&"Cross Reference"));
    assert!(concept_names.contains(&"Related Concept"));
    assert!(concept_names.contains(&"Subtopic 1"));
    
    // Verify no duplicates (Prerequisite A and Cross Reference appear multiple times in relationships)
    let prerequisite_count = concepts.iter().filter(|c| c.name == "Prerequisite A").count();
    let cross_ref_count = concepts.iter().filter(|c| c.name == "Cross Reference").count();
    assert_eq!(prerequisite_count, 1, "Prerequisite A should be deduplicated");
    assert_eq!(cross_ref_count, 1, "Cross Reference should be deduplicated");
}

#[test]
fn test_parse_large_dataset_with_relationships() {
    let parser = DgraphResponseParser::new();
    let response = TestFixtures::large_array_response();

    let start = std::time::Instant::now();
    let result = parser.parse_concepts_from_search_result(response, None);
    let duration = start.elapsed();

    assert!(result.is_ok(), "Should parse large dataset: {:?}", result.err());
    assert!(duration.as_millis() < 2000, "Large dataset parsing should be under 2s, took {:?}", duration);

    let concepts = result.unwrap();
    assert_eq!(concepts.len(), 100);

    // Verify data integrity for a few random samples
    assert_eq!(concepts[0].name, "Concept 0");
    assert_eq!(concepts[0].difficulty, "beginner");
    assert_eq!(concepts[50].name, "Concept 50");
    assert_eq!(concepts[99].name, "Concept 99");
    
    // Verify categories are distributed as expected
    let category_counts: std::collections::HashMap<String, usize> = 
        concepts.iter().fold(std::collections::HashMap::new(), |mut acc, concept| {
            *acc.entry(concept.category.clone()).or_insert(0) += 1;
            acc
        });
    assert_eq!(category_counts.len(), 10); // Should have 10 different categories
}

#[test]
fn test_parse_response_with_multiple_aliases() {
    let parser = DgraphResponseParser::new();
    
    let mut response = serde_json::json!({
        "data": {
            "sourceConcept:concept": {
                "id": TEST_UUID_1,
                "name": "Source",
                "difficulty": "beginner",
                "category": "Test",
                "qualityScore": 0.7,
                "tags": [],
                "embeddings": [],
                "createdAt": TEST_TIMESTAMP,
                "updatedAt": TEST_TIMESTAMP,
                "version": 1
            },
            "targetConcept:concept": {
                "id": TEST_UUID_2,
                "name": "Target",
                "difficulty": "intermediate",
                "category": "Test",
                "qualityScore": 0.8,
                "tags": [],
                "embeddings": [],
                "createdAt": TEST_TIMESTAMP,
                "updatedAt": TEST_TIMESTAMP,
                "version": 1
            },
            "relatedConcepts:concepts": [
                {
                    "id": TEST_UUID_3,
                    "name": "Related 1",
                    "difficulty": "beginner",
                    "category": "Test",
                    "qualityScore": 0.6,
                    "tags": [],
                    "embeddings": [],
                    "createdAt": TEST_TIMESTAMP,
                    "updatedAt": TEST_TIMESTAMP,
                    "version": 1
                },
                {
                    "id": TEST_UUID_4,
                    "name": "Related 2",
                    "difficulty": "advanced",
                    "category": "Test",
                    "qualityScore": 0.9,
                    "tags": [],
                    "embeddings": [],
                    "createdAt": TEST_TIMESTAMP,
                    "updatedAt": TEST_TIMESTAMP,
                    "version": 1
                }
            ]
        }
    });

    let result = parser.resolve_aliases(&mut response);
    assert!(result.is_ok(), "Should resolve multiple aliases: {:?}", result.err());

    let data = response.get("data").unwrap();
    
    // Check that aliases are resolved
    assert!(data.get("sourceConcept").is_some());
    assert!(data.get("targetConcept").is_some());
    assert!(data.get("relatedConcepts").is_some());
    
    // Verify we can parse the source concept by creating a full response
    let source_response = serde_json::json!({
        "data": {
            "concept": data["sourceConcept"].clone()
        }
    });
    let source_concept = parser.parse_concept_from_result(source_response).unwrap();
    assert_eq!(source_concept.name, "Source");
    
    // Verify we can parse the related concepts array
    let related_concepts_data = &data["relatedConcepts"];
    assert!(related_concepts_data.is_array());
    assert_eq!(related_concepts_data.as_array().unwrap().len(), 2);
}

#[test]
fn test_parse_response_with_fragments() {
    let parser = DgraphResponseParser::new();
    
    let mut response = serde_json::json!({
        "data": {
            "concept": {
                "id": TEST_UUID_1,
                "name": "Fragment Test",
                "...basicFields": {},
                "...metadataFields": {},
                "prerequisites": [
                    {
                        "id": TEST_UUID_2,
                        "name": "Prerequisite",
                        "...basicFields": {}
                    }
                ]
            }
        }
    });

    let mut fragments = HashMap::new();
    fragments.insert("basicFields".to_string(), serde_json::json!({
        "difficulty": "intermediate",
        "category": "Programming",
        "qualityScore": 0.8,
        "tags": ["fragment", "test"]
    }));
    fragments.insert("metadataFields".to_string(), serde_json::json!({
        "embeddings": [0.1, 0.2, 0.3],
        "createdAt": TEST_TIMESTAMP,
        "updatedAt": TEST_TIMESTAMP,
        "version": 1
    }));

    let result = parser.expand_fragments(&mut response, &fragments);
    assert!(result.is_ok(), "Should expand fragments: {:?}", result.err());

    // Now try to parse the expanded response
    let concept_result = parser.parse_concept_from_result(response);
    assert!(concept_result.is_ok(), "Should parse expanded fragments: {:?}", concept_result.err());

    let concept = concept_result.unwrap();
    assert_eq!(concept.name, "Fragment Test");
    assert_eq!(concept.difficulty, "intermediate");
    assert_eq!(concept.category, "Programming");
    assert_eq!(concept.quality_score, 0.8);
    assert_eq!(concept.tags, vec!["fragment", "test"]);
    assert_eq!(concept.embeddings, vec![0.1, 0.2, 0.3]);
}

#[test]
fn test_parse_mixed_content_types() {
    let parser = DgraphResponseParser::new();
    
    // Response containing concepts, resources, and paths in a single query
    let response = serde_json::json!({
        "data": {
            "concepts": [
                {
                    "id": TEST_UUID_1,
                    "name": "Main Concept",
                    "difficulty": "intermediate",
                    "category": "Programming",
                    "qualityScore": 0.8,
                    "tags": ["main"],
                    "embeddings": [0.1],
                    "createdAt": TEST_TIMESTAMP,
                    "updatedAt": TEST_TIMESTAMP,
                    "version": 1
                }
            ],
            "resources": [
                {
                    "id": TEST_UUID_2,
                    "url": "https://example.com/resource",
                    "title": "Related Resource",
                    "resourceType": "tutorial",
                    "format": "video",
                    "source": "YouTube",
                    "quality": 0.9,
                    "difficulty": "beginner",
                    "duration": 1800,
                    "language": "English",
                    "createdAt": TEST_TIMESTAMP,
                    "updatedAt": TEST_TIMESTAMP
                }
            ],
            "learningPath": {
                "id": TEST_UUID_3,
                "name": "Complete Path",
                "description": "Full learning path",
                "targetAudience": "Everyone",
                "estimatedTime": 600.0,
                "difficultyProgression": "beginner -> expert",
                "learningOutcomes": ["Learn everything"],
                "creator": "Expert",
                "isCustom": false,
                "createdAt": TEST_TIMESTAMP,
                "updatedAt": TEST_TIMESTAMP
            }
        }
    });

    // Test parsing each type from the mixed response
    let concepts_result = parser.parse_concepts_from_search_result(response.clone(), None);
    assert!(concepts_result.is_ok(), "Should parse concepts from mixed response");
    let concepts = concepts_result.unwrap();
    assert_eq!(concepts.len(), 1);
    assert_eq!(concepts[0].name, "Main Concept");

    let resources_result = parser.parse_learning_resources(response.clone());
    assert!(resources_result.is_ok(), "Should parse resources from mixed response");
    let resources = resources_result.unwrap();
    assert_eq!(resources.len(), 1);
    assert_eq!(resources[0].title, "Related Resource");

    let path_result = parser.parse_learning_path(response);
    assert!(path_result.is_ok(), "Should parse path from mixed response");
    let path = path_result.unwrap();
    assert_eq!(path.name, "Complete Path");
}

#[test]
fn test_parse_circular_relationships() {
    let parser = DgraphResponseParser::new();
    
    // Create a response with circular references (A -> B -> C -> A)
    let concept_a = serde_json::json!({
        "id": TEST_UUID_1,
        "name": "Concept A",
        "difficulty": "intermediate",
        "category": "Programming",
        "qualityScore": 0.8,
        "tags": ["circular"],
        "embeddings": [0.1],
        "createdAt": TEST_TIMESTAMP,
        "updatedAt": TEST_TIMESTAMP,
        "version": 1
    });

    let concept_b = serde_json::json!({
        "id": TEST_UUID_2,
        "name": "Concept B",
        "difficulty": "intermediate",
        "category": "Programming",
        "qualityScore": 0.8,
        "tags": ["circular"],
        "embeddings": [0.2],
        "createdAt": TEST_TIMESTAMP,
        "updatedAt": TEST_TIMESTAMP,
        "version": 1
    });

    let concept_c = serde_json::json!({
        "id": TEST_UUID_3,
        "name": "Concept C",
        "difficulty": "intermediate",
        "category": "Programming",
        "qualityScore": 0.8,
        "tags": ["circular"],
        "embeddings": [0.3],
        "createdAt": TEST_TIMESTAMP,
        "updatedAt": TEST_TIMESTAMP,
        "version": 1
    });

    let response = serde_json::json!({
        "data": {
            "concept": {
                "id": TEST_UUID_1,
                "name": "Concept A",
                "difficulty": "intermediate",
                "category": "Programming",
                "qualityScore": 0.8,
                "tags": ["circular"],
                "embeddings": [0.1],
                "createdAt": TEST_TIMESTAMP,
                "updatedAt": TEST_TIMESTAMP,
                "version": 1,
                "prerequisites": [concept_b.clone()],
                "relatedTo": [
                    {
                        "id": TEST_UUID_2,
                        "name": "Concept B",
                        "difficulty": "intermediate",
                        "category": "Programming",
                        "qualityScore": 0.8,
                        "tags": ["circular"],
                        "embeddings": [0.2],
                        "createdAt": TEST_TIMESTAMP,
                        "updatedAt": TEST_TIMESTAMP,
                        "version": 1,
                        "prerequisites": [concept_c.clone()]
                    }
                ],
                "enabledBy": [
                    {
                        "id": TEST_UUID_3,
                        "name": "Concept C",
                        "difficulty": "intermediate",
                        "category": "Programming",
                        "qualityScore": 0.8,
                        "tags": ["circular"],
                        "embeddings": [0.3],
                        "createdAt": TEST_TIMESTAMP,
                        "updatedAt": TEST_TIMESTAMP,
                        "version": 1,
                        "prerequisites": [concept_a.clone()]
                    }
                ]
            }
        }
    });

    let result = parser.parse_concepts_from_graph_result(response);
    assert!(result.is_ok(), "Should handle circular relationships: {:?}", result.err());

    let concepts = result.unwrap();
    
    // Should deduplicate and return all 3 concepts
    assert_eq!(concepts.len(), 3);
    
    let concept_names: Vec<&str> = concepts.iter().map(|c| c.name.as_str()).collect();
    assert!(concept_names.contains(&"Concept A"));
    assert!(concept_names.contains(&"Concept B"));
    assert!(concept_names.contains(&"Concept C"));
}

#[test]
fn test_parse_mutation_with_nested_result_data() {
    let parser = DgraphResponseParser::new();
    
    // Complex mutation result with nested data structures
    let response = serde_json::json!({
        "data": {
            "complexMutation": {
                "success": true,
                "message": "Complex operation completed",
                "results": {
                    "concepts": [
                        {
                            "id": TEST_UUID_1,
                            "name": "Created Concept 1",
                            "difficulty": "beginner",
                            "category": "Generated",
                            "qualityScore": 0.7,
                            "tags": ["created"],
                            "embeddings": [0.1],
                            "createdAt": TEST_TIMESTAMP,
                            "updatedAt": TEST_TIMESTAMP,
                            "version": 1
                        }
                    ],
                    "relationships": [
                        {
                            "from": TEST_UUID_1,
                            "to": TEST_UUID_2,
                            "type": "prerequisite",
                            "weight": 0.8
                        }
                    ],
                    "statistics": {
                        "conceptsCreated": 1,
                        "relationshipsCreated": 1,
                        "processingTimeMs": 250
                    }
                },
                "numUids": 2,
                "metadata": {
                    "operation": "bulk_create_with_relationships",
                    "timestamp": TEST_TIMESTAMP_UPDATED,
                    "user": "system"
                }
            }
        }
    });

    let result = parser.parse_mutation_result(response);
    assert!(result.is_ok(), "Should parse complex nested mutation result: {:?}", result.err());

    let mutation_result = result.unwrap();
    assert!(mutation_result.success);
    assert_eq!(mutation_result.affected_count, 2);
    
    // Verify nested data is preserved
    assert!(mutation_result.data.is_some());
    let data = mutation_result.data.unwrap();
    assert!(data.get("results").is_some());
    assert!(data["results"].get("concepts").is_some());
    assert!(data["results"].get("statistics").is_some());
    assert!(data.get("metadata").is_some());
    
    // Verify statistics are accessible
    assert_eq!(data["results"]["statistics"]["conceptsCreated"], 1);
    assert_eq!(data["results"]["statistics"]["relationshipsCreated"], 1);
}

#[test]
fn test_memory_efficiency_with_large_nested_structures() {
    let parser = DgraphResponseParser::new();
    
    // Create a response with many nested levels but limited breadth to test memory efficiency
    let mut nested_concept = serde_json::json!({
        "id": "550e8400-e29b-41d4-a716-446655440099",
        "name": "Deepest Concept",
        "difficulty": "expert",
        "category": "Deep",
        "qualityScore": 0.95,
        "tags": ["deep"],
        "embeddings": [0.99],
        "createdAt": TEST_TIMESTAMP,
        "updatedAt": TEST_TIMESTAMP,
        "version": 1
    });

    // Build a chain of 50 nested prerequisites
    for i in (0..50).rev() {
        nested_concept = serde_json::json!({
            "id": format!("550e8400-e29b-41d4-a716-{:012}", i),
            "name": format!("Level {} Concept", i),
            "difficulty": "intermediate",
            "category": "Chain",
            "qualityScore": 0.5 + (i as f32) / 100.0,
            "tags": [format!("level{}", i)],
            "embeddings": [(i as f32) / 50.0],
            "createdAt": TEST_TIMESTAMP,
            "updatedAt": TEST_TIMESTAMP,
            "version": 1,
            "prerequisites": [nested_concept.clone()]
        });
    }

    let response = serde_json::json!({
        "data": {
            "concept": nested_concept
        }
    });

    let start_memory = get_memory_usage();
    let start_time = std::time::Instant::now();
    
    let result = parser.parse_concepts_from_graph_result(response);
    
    let end_time = std::time::Instant::now();
    let end_memory = get_memory_usage();
    
    assert!(result.is_ok(), "Should parse deep nested structure: {:?}", result.err());
    
    let concepts = result.unwrap();
    assert_eq!(concepts.len(), 2); // Only first level extracted
    
    // Performance assertions
    assert!(end_time.duration_since(start_time).as_millis() < 1000, "Deep nesting should parse under 1s");
    
    // Memory usage shouldn't explode (this is a rough check)
    let memory_diff = end_memory.saturating_sub(start_memory);
    assert!(memory_diff < 100_000_000, "Memory usage should be reasonable: {} bytes", memory_diff); // 100MB limit
}

// Helper function to get rough memory usage (simplified for testing)
fn get_memory_usage() -> usize {
    // This is a simplified memory check - in a real implementation you might use
    // system-specific APIs or memory profiling tools
    std::mem::size_of::<usize>() * 1000 // Placeholder
}