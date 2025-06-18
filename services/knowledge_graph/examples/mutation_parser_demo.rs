//! Demonstration of the enhanced mutation parsing capabilities
//! 
//! This example shows how to use the DgraphResponseParser to handle
//! various mutation response formats from Dgraph.

use knowledge_graph::client::{DgraphResponseParser, MutationOperationType};
use serde_json::json;

fn main() {
    let parser = DgraphResponseParser::new();
    
    println!("=== Dgraph Mutation Response Parser Demo ===\n");
    
    // Example 1: Simple add mutation
    println!("1. Parsing simple add mutation:");
    let add_response = json!({
        "data": {
            "addConcept": {
                "concept": {
                    "id": "550e8400-e29b-41d4-a716-446655440000",
                    "name": "Rust Programming",
                    "difficulty": "intermediate",
                    "category": "Programming",
                    "qualityScore": 0.85,
                    "tags": ["rust", "systems", "programming"],
                    "createdAt": "2024-01-01T00:00:00Z",
                    "updatedAt": "2024-01-01T00:00:00Z",
                    "version": 1
                },
                "numUids": 1,
                "uid": "0x123"
            }
        }
    });
    
    match parser.parse_mutation_result(add_response) {
        Ok(result) => {
            println!("  ✓ Success: {}", result.success);
            println!("  ✓ Operation: {:?}", result.operation_type);
            println!("  ✓ Message: {}", result.message.unwrap_or_default());
            println!("  ✓ Affected IDs: {:?}", result.affected_ids);
            println!("  ✓ UIDs: {:?}", result.uids);
            println!("  ✓ Count: {}\n", result.affected_count);
        }
        Err(e) => println!("  ✗ Error: {}\n", e),
    }
    
    // Example 2: Bulk mutation
    println!("2. Parsing bulk mutation:");
    let bulk_response = json!({
        "data": {
            "addConcepts": {
                "concepts": [
                    {
                        "id": "550e8400-e29b-41d4-a716-446655440001",
                        "name": "Concept 1",
                        "difficulty": "beginner",
                        "category": "Test",
                        "qualityScore": 0.7,
                        "tags": [],
                        "createdAt": "2024-01-01T00:00:00Z",
                        "updatedAt": "2024-01-01T00:00:00Z",
                        "version": 1
                    },
                    {
                        "id": "550e8400-e29b-41d4-a716-446655440002",
                        "name": "Concept 2",
                        "difficulty": "intermediate",
                        "category": "Test",
                        "qualityScore": 0.8,
                        "tags": [],
                        "createdAt": "2024-01-01T00:00:00Z",
                        "updatedAt": "2024-01-01T00:00:00Z",
                        "version": 1
                    }
                ],
                "numUids": 2
            }
        }
    });
    
    match parser.parse_mutation_result(bulk_response) {
        Ok(result) => {
            println!("  ✓ Success: {}", result.success);
            println!("  ✓ Operation: {:?}", result.operation_type);
            println!("  ✓ Message: {}", result.message.unwrap_or_default());
            println!("  ✓ Affected count: {}\n", result.affected_count);
        }
        Err(e) => println!("  ✗ Error: {}\n", e),
    }
    
    // Example 3: Update with conflicts
    println!("3. Parsing update mutation with conflicts:");
    let update_response = json!({
        "data": {
            "updateConcept": {
                "concept": {
                    "id": "550e8400-e29b-41d4-a716-446655440000",
                    "name": "Updated Concept",
                    "difficulty": "advanced",
                    "category": "Test",
                    "qualityScore": 0.9,
                    "tags": ["updated"],
                    "createdAt": "2024-01-01T00:00:00Z",
                    "updatedAt": "2024-01-02T00:00:00Z",
                    "version": 2
                },
                "conflicts": [
                    {
                        "field": "version",
                        "existingValue": 1,
                        "attemptedValue": 2,
                        "resolution": "overwritten"
                    }
                ]
            }
        }
    });
    
    match parser.parse_mutation_result(update_response) {
        Ok(result) => {
            println!("  ✓ Success: {}", result.success);
            println!("  ✓ Operation: {:?}", result.operation_type);
            println!("  ✓ Conflicts detected: {}", result.conflicts.len());
            for conflict in &result.conflicts {
                println!("    - Field: {}, Resolution: {}", 
                    conflict.field, 
                    conflict.resolution.as_ref().unwrap_or(&"none".to_string())
                );
            }
            println!();
        }
        Err(e) => println!("  ✗ Error: {}\n", e),
    }
    
    // Example 4: Raw Dgraph mutation
    println!("4. Parsing raw Dgraph mutation:");
    let raw_response = json!({
        "data": {
            "mutate": {
                "code": "Success",
                "message": "Mutation applied successfully",
                "uids": {
                    "concept1": "0x123",
                    "concept2": "0x124",
                    "concept3": "0x125"
                },
                "queries": {
                    "q": [{
                        "count": 3
                    }]
                }
            }
        }
    });
    
    match parser.parse_mutation_result(raw_response) {
        Ok(result) => {
            println!("  ✓ Success: {}", result.success);
            println!("  ✓ UIDs extracted: {:?}", result.uids);
            println!("  ✓ Total affected: {}\n", result.affected_count);
        }
        Err(e) => println!("  ✗ Error: {}\n", e),
    }
    
    // Example 5: Error handling
    println!("5. Handling mutation errors:");
    let error_response = json!({
        "errors": [{
            "message": "Conflict: concept with this name already exists",
            "extensions": {
                "code": "CONFLICT"
            }
        }]
    });
    
    match parser.parse_mutation_result(error_response) {
        Ok(_) => println!("  ✗ Unexpected success\n"),
        Err(e) => println!("  ✓ Error correctly detected: {}\n", e),
    }
    
    // Example 6: Generic mutation result
    println!("6. Parsing generic mutation result:");
    let generic_response = json!({
        "data": {
            "customMutation": {
                "success": true,
                "message": "Custom operation to delete items completed",
                "numUids": 5
            }
        }
    });
    
    match parser.parse_mutation_result(generic_response) {
        Ok(result) => {
            println!("  ✓ Success: {}", result.success);
            println!("  ✓ Operation: {:?}", result.operation_type);
            println!("  ✓ Message: {}", result.message.unwrap_or_default());
            println!("  ✓ Items affected: {}\n", result.affected_count);
        }
        Err(e) => println!("  ✗ Error: {}\n", e),
    }
    
    println!("=== Demo Complete ===");
}