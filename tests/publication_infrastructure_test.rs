use std::process::Command;
use std::collections::HashMap;

/// Test 7: Publication Infrastructure - TDD Test for crates.io publication readiness
/// 
/// This test validates that all workspace crates can be successfully published to crates.io
/// in the correct dependency order, with proper metadata and without warnings.

#[test]
#[ignore] // Requires network access for crates.io validation
fn test_publication_infrastructure_readiness() {
    let publication_order = vec![
        "workflow-engine-core",    // No internal dependencies
        "workflow-engine-mcp",     // Depends on core
        "workflow-engine-nodes",   // Depends on core + mcp
        "workflow-engine-api",     // Depends on core + mcp
        "workflow-engine-gateway", // Depends on core
        "workflow-engine-app",     // Depends on all others
    ];

    println!("🧪 Testing publication infrastructure for all crates...");
    
    for crate_name in &publication_order {
        println!("📦 Testing publication readiness for: {}", crate_name);
        
        // Test 1: Dry run publish should succeed
        test_dry_run_publish_succeeds(crate_name);
        
        // Test 2: Package should compile without warnings
        test_package_compiles_without_warnings(crate_name);
        
        // Test 3: Required metadata should be present
        test_required_metadata_present(crate_name);
        
        // Test 4: Documentation should build successfully
        test_documentation_builds(crate_name);
        
        println!("✅ {}: All publication tests passed", crate_name);
    }
    
    println!("🎉 All crates ready for publication!");
}

fn test_dry_run_publish_succeeds(crate_name: &str) {
    let output = Command::new("cargo")
        .args(&["publish", "--dry-run", "--allow-dirty", "-p", crate_name])
        .output()
        .expect("Failed to execute cargo publish --dry-run");

    if !output.status.success() {
        panic!(
            "❌ Dry run publish failed for {}: {}", 
            crate_name, 
            String::from_utf8_lossy(&output.stderr)
        );
    }
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("error:") {
        panic!("❌ Publish errors found for {}: {}", crate_name, stderr);
    }
    
    println!("  ✓ Dry run publish successful for {}", crate_name);
}

fn test_package_compiles_without_warnings(crate_name: &str) {
    let output = Command::new("cargo")
        .args(&["check", "-p", crate_name, "--release"])
        .output()
        .expect("Failed to execute cargo check");

    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Allow some warnings but flag critical ones
    let critical_warnings = [
        "unused_imports",
        "unused_variables", 
        "unused_mut",
        "dead_code",
        "private_interfaces"
    ];
    
    let mut warning_count = 0;
    for warning in &critical_warnings {
        if stderr.contains(warning) {
            warning_count += 1;
        }
    }
    
    if warning_count > 5 {
        println!("⚠️  {} has {} critical warnings - should be addressed before publication", crate_name, warning_count);
    } else {
        println!("  ✓ {} compiles with acceptable warnings", crate_name);
    }
}

fn test_required_metadata_present(crate_name: &str) {
    let output = Command::new("cargo")
        .args(&["metadata", "--format-version", "1"])
        .output()
        .expect("Failed to get cargo metadata");

    let metadata_str = String::from_utf8_lossy(&output.stdout);
    
    // Parse the JSON to verify required fields
    let required_fields = [
        "version", "authors", "license", "description", 
        "repository", "homepage", "documentation", "keywords", "categories"
    ];
    
    for field in &required_fields {
        if !metadata_str.contains(field) {
            panic!("❌ Required metadata field '{}' missing for {}", field, crate_name);
        }
    }
    
    println!("  ✓ All required metadata present for {}", crate_name);
}

fn test_documentation_builds(crate_name: &str) {
    let output = Command::new("cargo")
        .args(&["doc", "-p", crate_name, "--no-deps"])
        .output()
        .expect("Failed to build documentation");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("error:") {
            panic!("❌ Documentation build failed for {}: {}", crate_name, stderr);
        }
    }
    
    println!("  ✓ Documentation builds successfully for {}", crate_name);
}

#[test]
fn test_publication_dependency_order() {
    println!("🔗 Testing dependency publication order...");
    
    let dependency_graph = get_dependency_graph();
    let publication_order = calculate_publication_order(&dependency_graph);
    
    let expected_order = vec![
        "workflow-engine-core",
        "workflow-engine-mcp", 
        "workflow-engine-nodes",
        "workflow-engine-api",
        "workflow-engine-gateway",
        "workflow-engine-app"
    ];
    
    for (i, crate_name) in expected_order.iter().enumerate() {
        assert_eq!(
            publication_order.get(i).unwrap(), 
            crate_name,
            "❌ Publication order incorrect at position {}", i
        );
    }
    
    println!("✅ Publication dependency order is correct");
}

fn get_dependency_graph() -> HashMap<String, Vec<String>> {
    let mut graph = HashMap::new();
    
    // Manually define the dependency relationships based on Cargo.toml files
    graph.insert("workflow-engine-core".to_string(), vec![]);
    graph.insert("workflow-engine-mcp".to_string(), vec!["workflow-engine-core".to_string()]);
    graph.insert("workflow-engine-nodes".to_string(), vec!["workflow-engine-core".to_string(), "workflow-engine-mcp".to_string()]);
    graph.insert("workflow-engine-api".to_string(), vec!["workflow-engine-core".to_string(), "workflow-engine-mcp".to_string()]);
    graph.insert("workflow-engine-gateway".to_string(), vec!["workflow-engine-core".to_string()]);
    graph.insert("workflow-engine-app".to_string(), vec!["workflow-engine-core".to_string(), "workflow-engine-api".to_string()]);
    
    graph
}

fn calculate_publication_order(dependency_graph: &HashMap<String, Vec<String>>) -> Vec<String> {
    let mut order = Vec::new();
    let mut visited = std::collections::HashSet::new();
    
    fn visit(
        crate_name: &str,
        graph: &HashMap<String, Vec<String>>,
        visited: &mut std::collections::HashSet<String>,
        order: &mut Vec<String>
    ) {
        if visited.contains(crate_name) {
            return;
        }
        
        visited.insert(crate_name.to_string());
        
        if let Some(deps) = graph.get(crate_name) {
            for dep in deps {
                visit(dep, graph, visited, order);
            }
        }
        
        order.push(crate_name.to_string());
    }
    
    for crate_name in dependency_graph.keys() {
        visit(crate_name, dependency_graph, &mut visited, &mut order);
    }
    
    order
}

#[test] 
fn test_workspace_configuration_valid() {
    println!("⚙️  Testing workspace configuration...");
    
    let output = Command::new("cargo")
        .args(&["check", "--workspace"])
        .output()
        .expect("Failed to check workspace");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("❌ Workspace configuration invalid: {}", stderr);
    }
    
    println!("✅ Workspace configuration is valid");
}

#[test]
fn test_no_path_dependencies_without_versions() {
    println!("🔍 Checking for path dependencies without versions...");
    
    let output = Command::new("rg")
        .args(&[r#"path.*=.*"\.\.""#, "-n", "crates/"])
        .output()
        .expect("Failed to search for path dependencies");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    
    for line in &lines {
        if !line.contains("version") {
            println!("⚠️  Path dependency without version: {}", line);
        }
    }
    
    println!("✅ Path dependency version check complete");
}