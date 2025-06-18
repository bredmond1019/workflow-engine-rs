//! Tests for successful response parsing scenarios
//!
//! These tests cover the main parsing methods for successful GraphQL responses,
//! ensuring that valid data is correctly parsed into domain objects.

use super::fixtures::*;
use crate::client::dgraph::{DgraphResponseParser, MutationOperationType};
use crate::graph::{Concept, LearningResource, LearningPath, UserProgress};

#[test]
fn test_parse_simple_concept_response() {
    let parser = DgraphResponseParser::new();
    let response = TestFixtures::simple_concept_response();

    let result = parser.parse_concept_from_result(response);
    assert!(result.is_ok(), "Failed to parse simple concept: {:?}", result.err());

    let concept = result.unwrap();
    assert_eq!(concept.id.to_string(), TEST_UUID_1);
    assert_eq!(concept.name, "Rust Programming");
    assert_eq!(concept.description, Some("A systems programming language that runs blazingly fast".to_string()));
    assert_eq!(concept.difficulty, "intermediate");
    assert_eq!(concept.category, "Programming");
    assert_eq!(concept.subcategory, Some("Systems Programming".to_string()));
    assert_eq!(concept.tags, vec!["rust", "systems", "programming", "memory-safe"]);
    assert_eq!(concept.quality_score, 0.85);
    assert_eq!(concept.estimated_time, Some(120.5));
    assert_eq!(concept.embeddings, vec![0.1, 0.2, 0.3, 0.4, 0.5]);
    assert_eq!(concept.version, 1);
}

#[test]
fn test_parse_minimal_concept_response() {
    let parser = DgraphResponseParser::new();
    let response = TestFixtures::minimal_concept_response();

    let result = parser.parse_concept_from_result(response);
    assert!(result.is_ok(), "Failed to parse minimal concept: {:?}", result.err());

    let concept = result.unwrap();
    assert_eq!(concept.id.to_string(), TEST_UUID_1);
    assert_eq!(concept.name, "Minimal Concept");
    assert_eq!(concept.description, None);
    assert_eq!(concept.difficulty, "beginner");
    assert_eq!(concept.category, "Test");
    assert_eq!(concept.subcategory, None);
    assert_eq!(concept.tags, Vec::<String>::new());
    assert_eq!(concept.quality_score, 0.5);
    assert_eq!(concept.estimated_time, None);
    assert_eq!(concept.embeddings, Vec::<f32>::new());
    assert_eq!(concept.version, 1);
}

#[test]
fn test_parse_concept_with_nulls() {
    let parser = DgraphResponseParser::new();
    let response = TestFixtures::concept_with_nulls_response();

    let result = parser.parse_concept_from_result(response);
    assert!(result.is_ok(), "Failed to parse concept with nulls: {:?}", result.err());

    let concept = result.unwrap();
    assert_eq!(concept.name, "Concept with Nulls");
    assert_eq!(concept.description, None);
    assert_eq!(concept.subcategory, None);
    assert_eq!(concept.estimated_time, None);
}

#[test]
fn test_parse_search_results() {
    let parser = DgraphResponseParser::new();
    let response = TestFixtures::search_concepts_response();

    let result = parser.parse_concepts_from_search_result(response, None);
    assert!(result.is_ok(), "Failed to parse search results: {:?}", result.err());

    let concepts = result.unwrap();
    assert_eq!(concepts.len(), 3);

    // Verify first concept
    assert_eq!(concepts[0].name, "First Concept");
    assert_eq!(concepts[0].difficulty, "beginner");
    assert_eq!(concepts[0].tags, vec!["basic", "intro"]);

    // Verify second concept
    assert_eq!(concepts[1].name, "Second Concept");
    assert_eq!(concepts[1].difficulty, "intermediate");
    assert_eq!(concepts[1].subcategory, Some("Web Development".to_string()));

    // Verify third concept
    assert_eq!(concepts[2].name, "Third Concept");
    assert_eq!(concepts[2].difficulty, "advanced");
    assert_eq!(concepts[2].quality_score, 0.95);
    assert_eq!(concepts[2].estimated_time, Some(240.0));
    assert_eq!(concepts[2].version, 2);
}

#[test]
fn test_parse_search_with_limit() {
    let parser = DgraphResponseParser::new();
    let response = TestFixtures::search_concepts_response();

    let result = parser.parse_concepts_from_search_result(response, Some(2));
    assert!(result.is_ok(), "Failed to parse limited search results: {:?}", result.err());

    let concepts = result.unwrap();
    assert_eq!(concepts.len(), 2);
    assert_eq!(concepts[0].name, "First Concept");
    assert_eq!(concepts[1].name, "Second Concept");
}

#[test]
fn test_parse_search_with_alternative_field_names() {
    let parser = DgraphResponseParser::new();
    let response = TestFixtures::search_with_alternative_fields();

    let result = parser.parse_concepts_from_search_result(response, None);
    assert!(result.is_ok(), "Failed to parse alternative field names: {:?}", result.err());

    let concepts = result.unwrap();
    assert_eq!(concepts.len(), 1);
    assert_eq!(concepts[0].name, "Query Concept");
}

#[test]
fn test_parse_concept_with_relationships() {
    let parser = DgraphResponseParser::new();
    let response = TestFixtures::concept_with_relationships();

    let result = parser.parse_concepts_from_graph_result(response);
    assert!(result.is_ok(), "Failed to parse concept with relationships: {:?}", result.err());

    let concepts = result.unwrap();
    
    // Should include main concept plus prerequisites and related concepts
    assert_eq!(concepts.len(), 4); // main + 2 prerequisites + 1 related
    
    // Check that main concept is included
    let main_concept = concepts.iter().find(|c| c.name == "Main Concept").unwrap();
    assert_eq!(main_concept.tags, vec!["main", "relationships"]);
    
    // Check that prerequisites are included
    assert!(concepts.iter().any(|c| c.name == "Prerequisite 1"));
    assert!(concepts.iter().any(|c| c.name == "Prerequisite 2"));
    assert!(concepts.iter().any(|c| c.name == "Related Concept"));
}

#[test]
fn test_parse_learning_resources() {
    let parser = DgraphResponseParser::new();
    let response = TestFixtures::learning_resource_response();

    let result = parser.parse_learning_resources(response);
    assert!(result.is_ok(), "Failed to parse learning resources: {:?}", result.err());

    let resources = result.unwrap();
    assert_eq!(resources.len(), 2);

    // Verify first resource
    let first_resource = &resources[0];
    assert_eq!(first_resource.url, "https://example.com/rust-tutorial");
    assert_eq!(first_resource.title, "Complete Rust Tutorial");
    assert_eq!(first_resource.resource_type, "tutorial");
    assert_eq!(first_resource.format, Some("video".to_string()));
    assert_eq!(first_resource.source, Some("YouTube".to_string()));
    assert_eq!(first_resource.quality, Some(0.9));
    assert_eq!(first_resource.difficulty, Some("beginner".to_string()));
    assert_eq!(first_resource.duration, Some(3600));
    assert_eq!(first_resource.language, Some("English".to_string()));

    // Verify second resource
    let second_resource = &resources[1];
    assert_eq!(second_resource.url, "https://doc.rust-lang.org/book/");
    assert_eq!(second_resource.title, "The Rust Programming Language");
    assert_eq!(second_resource.resource_type, "documentation");
    assert_eq!(second_resource.duration, None);
}

#[test]
fn test_parse_learning_path() {
    let parser = DgraphResponseParser::new();
    let response = TestFixtures::learning_path_response();

    let result = parser.parse_learning_path(response);
    assert!(result.is_ok(), "Failed to parse learning path: {:?}", result.err());

    let path = result.unwrap();
    assert_eq!(path.id.to_string(), TEST_UUID_1);
    assert_eq!(path.name, "Rust Mastery Path");
    assert_eq!(path.description, Some("Complete path to master Rust programming".to_string()));
    assert_eq!(path.target_audience, Some("Intermediate developers".to_string()));
    assert_eq!(path.estimated_time, Some(480.0));
    assert_eq!(path.difficulty_progression, Some("beginner -> intermediate -> advanced".to_string()));
    assert_eq!(path.learning_outcomes, vec![
        "Understand ownership and borrowing",
        "Write safe concurrent code",
        "Build web applications with Rust"
    ]);
    assert_eq!(path.creator, Some("Rust Expert".to_string()));
    assert_eq!(path.is_custom, false);
}

#[test]
fn test_parse_user_progress() {
    let parser = DgraphResponseParser::new();
    let response = TestFixtures::user_progress_response();

    let result = parser.parse_user_progress(response);
    assert!(result.is_ok(), "Failed to parse user progress: {:?}", result.err());

    let progress_items = result.unwrap();
    assert_eq!(progress_items.len(), 2);

    // Verify first progress item (in progress)
    let first_progress = &progress_items[0];
    assert_eq!(first_progress.id.to_string(), TEST_UUID_1);
    assert_eq!(first_progress.user_id, "user123");
    assert_eq!(first_progress.concept_id.to_string(), TEST_UUID_2);
    assert_eq!(first_progress.status, "in_progress");
    assert_eq!(first_progress.percent_complete, Some(65.5));
    assert_eq!(first_progress.time_spent, Some(1800));
    assert_eq!(first_progress.resources_completed, Some(3));
    assert_eq!(first_progress.difficulty_rating, Some(3.5));
    assert_eq!(first_progress.notes, Some("Making good progress, need to review ownership concepts".to_string()));
    assert!(first_progress.started_at.is_some());
    assert!(first_progress.completed_at.is_none());

    // Verify second progress item (completed)
    let second_progress = &progress_items[1];
    assert_eq!(second_progress.id.to_string(), TEST_UUID_3);
    assert_eq!(second_progress.status, "completed");
    assert_eq!(second_progress.percent_complete, Some(100.0));
    assert!(second_progress.completed_at.is_some());
}

#[test]
fn test_parse_query_result_generic() {
    let parser = DgraphResponseParser::new();
    let response = TestFixtures::simple_concept_response();

    // Test generic parsing with explicit type
    let result: Result<serde_json::Value, _> = parser.parse_query_result(response);
    assert!(result.is_ok(), "Failed to parse query result generically: {:?}", result.err());

    let data = result.unwrap();
    assert!(data.get("concept").is_some());
    assert_eq!(data["concept"]["name"], "Rust Programming");
}

#[test]
fn test_resolve_aliases() {
    let parser = DgraphResponseParser::new();
    let mut response = TestFixtures::aliased_response();

    let result = parser.resolve_aliases(&mut response);
    assert!(result.is_ok(), "Failed to resolve aliases: {:?}", result.err());

    let data = response.get("data").unwrap();
    assert!(data.get("fromConcept").is_some());
    assert!(data.get("toConcept").is_some());
    
    // Original aliased fields should still exist
    assert!(data.get("fromConcept:concept").is_some());
    assert!(data.get("toConcept:concept").is_some());
}

#[test]
fn test_parse_empty_search_results() {
    let parser = DgraphResponseParser::new();
    let response = TestFixtures::empty_data_response();

    let result = parser.parse_concepts_from_search_result(response, None);
    assert!(result.is_ok(), "Failed to parse empty search results: {:?}", result.err());

    let concepts = result.unwrap();
    assert_eq!(concepts.len(), 0);
}

#[test]
fn test_parse_large_array_performance() {
    let parser = DgraphResponseParser::new();
    let response = TestFixtures::large_array_response();

    let start = std::time::Instant::now();
    let result = parser.parse_concepts_from_search_result(response, None);
    let duration = start.elapsed();
    
    assert!(result.is_ok(), "Failed to parse large array: {:?}", result.err());
    assert!(duration.as_millis() < 1000, "Parsing took too long: {:?}", duration);

    let concepts = result.unwrap();
    assert_eq!(concepts.len(), 100);
    
    // Verify a few random concepts
    assert_eq!(concepts[0].name, "Concept 0");
    assert_eq!(concepts[50].name, "Concept 50");
    assert_eq!(concepts[99].name, "Concept 99");
}

#[test]
fn test_parse_deeply_nested_structures() {
    let parser = DgraphResponseParser::new();
    let response = TestFixtures::deeply_nested_response();

    let result = parser.parse_concepts_from_graph_result(response);
    assert!(result.is_ok(), "Failed to parse deeply nested structures: {:?}", result.err());

    let concepts = result.unwrap();
    assert_eq!(concepts.len(), 2); // Root + 1 level of prerequisites (function doesn't recurse)
    
    // Verify root and first level are included
    assert!(concepts.iter().any(|c| c.name == "Root Concept"));
    assert!(concepts.iter().any(|c| c.name == "Level 1 Prerequisite"));
    
    // Note: Deeper levels are not extracted by parse_concepts_from_graph_result
    // as it only processes the first level of relationships
}

#[test]
fn test_parse_timestamp_variations() {
    let parser = DgraphResponseParser::new();
    let response = TestFixtures::timestamp_variations_response();

    let result = parser.parse_concepts_from_search_result(response, None);
    assert!(result.is_ok(), "Failed to parse timestamp variations: {:?}", result.err());

    let concepts = result.unwrap();
    assert_eq!(concepts.len(), 2);

    // Both concepts should parse successfully despite different timestamp formats
    assert_eq!(concepts[0].name, "ISO 8601 Timestamp");
    assert_eq!(concepts[1].name, "Unix Timestamp");
    
    // Verify timestamps are valid
    assert!(concepts[0].created_at.timestamp() > 0);
    assert!(concepts[1].created_at.timestamp() > 0);
}