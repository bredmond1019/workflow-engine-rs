//! Example demonstrating GraphQL response parsing functionality

use knowledge_graph::client::DgraphResponseParser;
use serde_json::json;

fn main() {
    // Initialize the parser
    let parser = DgraphResponseParser::new();
    
    println!("=== GraphQL Response Parser Demo ===\n");
    
    // Example 1: Parse a simple concept query response
    println!("1. Parsing a simple concept query response:");
    let simple_response = json!({
        "data": {
            "concept": {
                "id": "550e8400-e29b-41d4-a716-446655440000",
                "name": "Rust Programming",
                "description": "A systems programming language focused on safety and performance",
                "difficulty": "intermediate",
                "category": "Programming",
                "subcategory": "Systems Programming",
                "tags": ["rust", "systems", "memory-safety"],
                "qualityScore": 0.85,
                "estimatedTime": 120.0,
                "embeddings": [0.1, 0.2, 0.3],
                "createdAt": "2024-01-01T00:00:00Z",
                "updatedAt": "2024-01-15T00:00:00Z",
                "version": 1
            }
        }
    });
    
    match parser.parse_concept_from_result(simple_response) {
        Ok(concept) => {
            println!("  ✓ Successfully parsed concept: {}", concept.name);
            println!("    - Difficulty: {}", concept.difficulty);
            println!("    - Category: {}", concept.category);
            println!("    - Quality Score: {}", concept.quality_score);
        }
        Err(e) => println!("  ✗ Error parsing concept: {}", e),
    }
    
    // Example 2: Parse search results with multiple concepts
    println!("\n2. Parsing search results with multiple concepts:");
    let search_response = json!({
        "data": {
            "concepts": [
                {
                    "id": "550e8400-e29b-41d4-a716-446655440001",
                    "name": "Async Programming",
                    "difficulty": "advanced",
                    "category": "Programming",
                    "qualityScore": 0.9,
                    "tags": ["async", "concurrency"],
                    "createdAt": "2024-01-01T00:00:00Z",
                    "updatedAt": "2024-01-01T00:00:00Z",
                    "version": 1
                },
                {
                    "id": "550e8400-e29b-41d4-a716-446655440002",
                    "name": "Error Handling",
                    "difficulty": "intermediate",
                    "category": "Programming",
                    "qualityScore": 0.8,
                    "tags": ["errors", "result-type"],
                    "createdAt": "2024-01-01T00:00:00Z",
                    "updatedAt": "2024-01-01T00:00:00Z",
                    "version": 1
                }
            ]
        }
    });
    
    match parser.parse_concepts_from_search_result(search_response, None) {
        Ok(concepts) => {
            println!("  ✓ Successfully parsed {} concepts:", concepts.len());
            for concept in concepts {
                println!("    - {} ({})", concept.name, concept.difficulty);
            }
        }
        Err(e) => println!("  ✗ Error parsing search results: {}", e),
    }
    
    // Example 3: Handle GraphQL aliases
    println!("\n3. Handling GraphQL aliases:");
    let mut aliased_response = json!({
        "data": {
            "fromConcept:concept": {
                "id": "550e8400-e29b-41d4-a716-446655440003",
                "name": "Source Concept"
            },
            "toConcept:concept": {
                "id": "550e8400-e29b-41d4-a716-446655440004",
                "name": "Target Concept"
            }
        }
    });
    
    match parser.resolve_aliases(&mut aliased_response) {
        Ok(_) => {
            println!("  ✓ Successfully resolved aliases");
            if let Some(data) = aliased_response.get("data") {
                if data.get("fromConcept").is_some() && data.get("toConcept").is_some() {
                    println!("    - Both aliased fields are now accessible");
                }
            }
        }
        Err(e) => println!("  ✗ Error resolving aliases: {}", e),
    }
    
    // Example 4: Handle nested relationships
    println!("\n4. Parsing nested relationships:");
    let nested_response = json!({
        "data": {
            "concept": {
                "id": "550e8400-e29b-41d4-a716-446655440005",
                "name": "Main Topic",
                "difficulty": "intermediate",
                "category": "Programming",
                "qualityScore": 0.85,
                "tags": ["main"],
                "createdAt": "2024-01-01T00:00:00Z",
                "updatedAt": "2024-01-01T00:00:00Z",
                "version": 1,
                "prerequisites": [
                    {
                        "id": "550e8400-e29b-41d4-a716-446655440006",
                        "name": "Prerequisite 1",
                        "difficulty": "beginner",
                        "category": "Programming",
                        "qualityScore": 0.7,
                        "tags": ["basic"],
                        "createdAt": "2024-01-01T00:00:00Z",
                        "updatedAt": "2024-01-01T00:00:00Z",
                        "version": 1
                    }
                ],
                "relatedTo": [
                    {
                        "id": "550e8400-e29b-41d4-a716-446655440007",
                        "name": "Related Topic",
                        "difficulty": "intermediate",
                        "category": "Programming",
                        "qualityScore": 0.8,
                        "tags": ["related"],
                        "createdAt": "2024-01-01T00:00:00Z",
                        "updatedAt": "2024-01-01T00:00:00Z",
                        "version": 1
                    }
                ]
            }
        }
    });
    
    match parser.parse_concepts_from_graph_result(nested_response) {
        Ok(concepts) => {
            println!("  ✓ Successfully parsed graph with {} concepts:", concepts.len());
            for concept in concepts {
                println!("    - {} ({})", concept.name, concept.difficulty);
            }
        }
        Err(e) => println!("  ✗ Error parsing graph result: {}", e),
    }
    
    // Example 5: Handle GraphQL errors
    println!("\n5. Handling GraphQL errors:");
    let error_response = json!({
        "errors": [
            {
                "message": "Variable $conceptId is not defined",
                "extensions": {
                    "code": "GRAPHQL_VALIDATION_FAILED"
                }
            }
        ]
    });
    
    match parser.parse_concept_from_result(error_response) {
        Ok(_) => println!("  ✗ Unexpected success"),
        Err(e) => {
            println!("  ✓ Correctly caught GraphQL error:");
            println!("    - {}", e);
        }
    }
    
    // Example 6: Parse mutation results
    println!("\n6. Parsing mutation results:");
    let mutation_response = json!({
        "data": {
            "addConcept": {
                "concept": {
                    "id": "550e8400-e29b-41d4-a716-446655440008",
                    "name": "New Concept",
                    "difficulty": "beginner",
                    "category": "Programming",
                    "qualityScore": 0.5,
                    "createdAt": "2024-01-15T00:00:00Z"
                }
            }
        }
    });
    
    match parser.parse_mutation_result(mutation_response) {
        Ok(result) => {
            println!("  ✓ Successfully parsed mutation result:");
            println!("    - Success: {}", result.success);
            if let Some(msg) = result.message {
                println!("    - Message: {}", msg);
            }
            println!("    - Affected IDs: {}", result.affected_ids.len());
        }
        Err(e) => println!("  ✗ Error parsing mutation result: {}", e),
    }
    
    println!("\n=== Demo Complete ===");
}