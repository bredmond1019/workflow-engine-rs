//! Federation Validation Example
//! 
//! This example validates the GraphQL Federation setup by:
//! 1. Checking all services are running
//! 2. Validating federation schemas
//! 3. Testing entity resolution
//! 4. Running sample federated queries
//! 
//! Run with: cargo run --example validate_federation

use std::collections::HashMap;
use workflow_engine_gateway::SubgraphConfig;

// Include test utilities (this would normally be a proper module import)
mod test_utils;
use test_utils::{FederationTestClient, FederationTestData, FederationTestQueries};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 GraphQL Federation Validation");
    println!("=================================\n");
    
    // Initialize test client
    let client = FederationTestClient::default();
    
    // Run comprehensive health check
    println!("📊 Running comprehensive health check...");
    let health_report = client.comprehensive_health_check().await;
    health_report.print_summary();
    
    if !health_report.all_services_healthy {
        println!("\n❌ Not all services are healthy. Please start all services before validation.");
        println!("   Use: ./scripts/test_federation.sh start");
        return Ok(());
    }
    
    println!("\n🧪 Running Federation Validation Tests");
    println!("======================================\n");
    
    // Test 1: Service Discovery and Schema Validation
    test_service_discovery(&client).await?;
    
    // Test 2: Entity Resolution
    test_entity_resolution(&client).await?;
    
    // Test 3: Cross-Service Queries
    test_cross_service_queries(&client).await?;
    
    // Test 4: Federation Directives
    test_federation_directives(&client).await?;
    
    // Test 5: Gateway Introspection
    test_gateway_introspection(&client).await?;
    
    println!("\n🎉 Federation validation completed!");
    println!("====================================");
    
    Ok(())
}

async fn test_service_discovery(client: &FederationTestClient) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Test 1: Service Discovery and Schema Validation");
    
    let subgraphs = vec!["workflow", "content_processing", "knowledge_graph", "realtime_communication"];
    
    for subgraph in subgraphs {
        println!("  📡 Testing subgraph: {}", subgraph);
        
        // Get SDL schema
        match client.get_subgraph_schema(subgraph).await {
            Ok(sdl) => {
                println!("    ✅ SDL retrieved ({} chars)", sdl.len());
                
                // Validate federation directives
                let validation = client.validate_federation_directives(&sdl);
                println!("    📋 Federation compliance: {:.1}%", validation.score() * 100.0);
                
                if validation.entity_count > 0 {
                    println!("    🏷️  Entities with @key directive: {}", validation.entity_count);
                }
                
                if validation.has_extends {
                    println!("    🔗 Has entity extensions");
                }
                
                if !validation.is_valid {
                    println!("    ⚠️  Missing some federation requirements");
                }
                
                // Show first few lines of SDL for debugging
                let first_lines: Vec<&str> = sdl.lines().take(5).collect();
                println!("    📄 SDL preview:");
                for line in first_lines {
                    if !line.trim().is_empty() {
                        println!("      {}", line);
                    }
                }
            }
            Err(e) => {
                println!("    ❌ Failed to get SDL: {}", e);
            }
        }
        
        println!();
    }
    
    Ok(())
}

async fn test_entity_resolution(client: &FederationTestClient) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Test 2: Entity Resolution");
    
    // Test with various entity types
    let test_cases = vec![
        ("User entities", FederationTestData::users()),
        ("Workflow entities", FederationTestData::workflows()),
        ("Content entities", FederationTestData::content()),
        ("Concept entities", FederationTestData::concepts()),
        ("Message entities", FederationTestData::messages()),
    ];
    
    for (test_name, entities) in test_cases {
        println!("  🧪 Testing: {}", test_name);
        
        // Take first 2 entities for testing
        let test_entities = entities.into_iter().take(2).collect();
        
        match client.test_entity_resolution(test_entities).await {
            Ok(result) => {
                if result.success {
                    println!("    ✅ Entity resolution successful ({:?})", result.execution_time);
                    
                    // Count resolved entities
                    if let Some(entities) = result.response
                        .get("data")
                        .and_then(|d| d.get("_entities"))
                        .and_then(|e| e.as_array()) {
                        
                        let resolved_count = entities.iter()
                            .filter(|e| !e.is_null())
                            .count();
                        
                        println!("    📊 Resolved: {}/{} entities", resolved_count, entities.len());
                    }
                } else {
                    println!("    ⚠️  Entity resolution had errors: {:?}", result.errors);
                }
            }
            Err(e) => {
                println!("    ❌ Entity resolution failed: {}", e);
            }
        }
        
        println!();
    }
    
    Ok(())
}

async fn test_cross_service_queries(client: &FederationTestClient) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Test 3: Cross-Service Queries");
    
    let test_queries = vec![
        ("Basic cross-service query", FederationTestQueries::cross_service(), None),
        ("User with extensions", FederationTestQueries::user_with_extensions(), Some(serde_json::json!({"userId": "user_123"}))),
    ];
    
    for (test_name, query, variables) in test_queries {
        println!("  🧪 Testing: {}", test_name);
        
        match client.execute_gateway_query(query, variables).await {
            Ok(result) => {
                if result.success {
                    println!("    ✅ Query successful ({:?})", result.execution_time);
                    
                    // Analyze response structure
                    if let Some(data) = result.response.get("data") {
                        let field_count = data.as_object().map(|obj| obj.len()).unwrap_or(0);
                        println!("    📊 Response fields: {}", field_count);
                        
                        // Check for data from different services
                        let data_sources = analyze_response_sources(data);
                        if !data_sources.is_empty() {
                            println!("    🔗 Data from services: {}", data_sources.join(", "));
                        }
                    }
                } else {
                    println!("    ⚠️  Query had errors: {:?}", result.errors);
                }
            }
            Err(e) => {
                println!("    ❌ Query failed: {}", e);
            }
        }
        
        println!();
    }
    
    Ok(())
}

async fn test_federation_directives(client: &FederationTestClient) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Test 4: Federation Directives");
    
    let subgraphs = vec!["workflow", "content_processing", "knowledge_graph", "realtime_communication"];
    
    for subgraph in subgraphs {
        println!("  📡 Analyzing federation directives for: {}", subgraph);
        
        match client.get_subgraph_schema(subgraph).await {
            Ok(sdl) => {
                let directive_analysis = analyze_federation_directives(&sdl);
                
                println!("    🏷️  Federation Directives Found:");
                for (directive, count) in directive_analysis {
                    if count > 0 {
                        println!("      @{}: {} occurrences", directive, count);
                    }
                }
                
                // Check for proper entity definitions
                let entity_definitions = find_entity_definitions(&sdl);
                if !entity_definitions.is_empty() {
                    println!("    🎯 Entity Definitions:");
                    for entity in entity_definitions {
                        println!("      {}", entity);
                    }
                }
            }
            Err(e) => {
                println!("    ❌ Failed to analyze directives: {}", e);
            }
        }
        
        println!();
    }
    
    Ok(())
}

async fn test_gateway_introspection(client: &FederationTestClient) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Test 5: Gateway Introspection");
    
    match client.execute_gateway_query(FederationTestQueries::gateway_introspection(), None).await {
        Ok(result) => {
            if result.success {
                println!("  ✅ Gateway introspection successful ({:?})", result.execution_time);
                
                if let Some(schema) = result.response
                    .get("data")
                    .and_then(|d| d.get("__schema")) {
                    
                    // Analyze schema composition
                    analyze_composed_schema(schema);
                }
            } else {
                println!("  ⚠️  Gateway introspection had errors: {:?}", result.errors);
            }
        }
        Err(e) => {
            println!("  ❌ Gateway introspection failed: {}", e);
        }
    }
    
    println!();
    
    Ok(())
}

fn analyze_response_sources(data: &serde_json::Value) -> Vec<String> {
    let mut sources = Vec::new();
    
    if data.get("workflows").is_some() {
        sources.push("workflow_api".to_string());
    }
    
    if data.get("content").is_some() || data.get("searchContent").is_some() {
        sources.push("content_processing".to_string());
    }
    
    if data.get("concepts").is_some() || data.get("searchConcepts").is_some() {
        sources.push("knowledge_graph".to_string());
    }
    
    if data.get("conversations").is_some() || data.get("messages").is_some() {
        sources.push("realtime_communication".to_string());
    }
    
    sources
}

fn analyze_federation_directives(sdl: &str) -> HashMap<String, usize> {
    let mut directives = HashMap::new();
    
    directives.insert("key".to_string(), sdl.matches("@key").count());
    directives.insert("extends".to_string(), sdl.matches("@extends").count());
    directives.insert("external".to_string(), sdl.matches("@external").count());
    directives.insert("provides".to_string(), sdl.matches("@provides").count());
    directives.insert("requires".to_string(), sdl.matches("@requires").count());
    
    directives
}

fn find_entity_definitions(sdl: &str) -> Vec<String> {
    let mut entities = Vec::new();
    
    // Simple regex-like search for entity definitions
    // This is a basic implementation - a real parser would be more robust
    for line in sdl.lines() {
        if line.contains("@key") && (line.contains("type ") || line.contains("extend type ")) {
            // Extract type name
            if let Some(type_start) = line.find("type ") {
                let after_type = &line[type_start + 5..];
                if let Some(space_pos) = after_type.find(' ') {
                    let type_name = &after_type[..space_pos];
                    entities.push(format!("type {} (entity)", type_name));
                }
            }
        }
    }
    
    entities
}

fn analyze_composed_schema(schema: &serde_json::Value) {
    // Check root types
    if let Some(query_type) = schema.get("queryType") {
        if let Some(name) = query_type.get("name").and_then(|n| n.as_str()) {
            println!("  📋 Query root type: {}", name);
        }
    }
    
    if let Some(mutation_type) = schema.get("mutationType") {
        if let Some(name) = mutation_type.get("name").and_then(|n| n.as_str()) {
            println!("  🔄 Mutation root type: {}", name);
        }
    }
    
    if let Some(subscription_type) = schema.get("subscriptionType") {
        if let Some(name) = subscription_type.get("name").and_then(|n| n.as_str()) {
            println!("  📡 Subscription root type: {}", name);
        }
    }
    
    // Check types
    if let Some(types) = schema.get("types").and_then(|t| t.as_array()) {
        let type_counts = count_types_by_kind(types);
        
        println!("  📊 Composed Schema Statistics:");
        for (kind, count) in type_counts {
            println!("    {}: {}", kind, count);
        }
        
        // Check for federation-specific types
        let federation_types = count_federation_types(types);
        if !federation_types.is_empty() {
            println!("  🔗 Federation Types:");
            for (type_name, kind) in federation_types {
                println!("    {}: {}", type_name, kind);
            }
        }
    }
    
    // Check directives
    if let Some(directives) = schema.get("directives").and_then(|d| d.as_array()) {
        let federation_directives: Vec<_> = directives.iter()
            .filter_map(|d| d.get("name").and_then(|n| n.as_str()))
            .filter(|name| name.starts_with("key") || name.starts_with("extends") || name.starts_with("external"))
            .collect();
        
        if !federation_directives.is_empty() {
            println!("  🏷️  Federation Directives: {}", federation_directives.join(", "));
        }
    }
}

fn count_types_by_kind(types: &[serde_json::Value]) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    
    for type_def in types {
        if let Some(kind) = type_def.get("kind").and_then(|k| k.as_str()) {
            *counts.entry(kind.to_string()).or_insert(0) += 1;
        }
    }
    
    counts
}

fn count_federation_types(types: &[serde_json::Value]) -> Vec<(String, String)> {
    let mut federation_types = Vec::new();
    
    for type_def in types {
        if let Some(name) = type_def.get("name").and_then(|n| n.as_str()) {
            if let Some(kind) = type_def.get("kind").and_then(|k| k.as_str()) {
                if name.starts_with("_") || name.contains("Service") || name.contains("Entity") {
                    federation_types.push((name.to_string(), kind.to_string()));
                }
            }
        }
    }
    
    federation_types
}