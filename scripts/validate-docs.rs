#!/usr/bin/env rust-script
//! This script validates that all code examples in documentation compile correctly.
//! 
//! Usage: cargo script scripts/validate-docs.rs

use std::fs;
use std::io::Write;
use std::process::Command;
use std::path::Path;
use regex::Regex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Validating documentation code examples...\n");

    // Read README.md
    let readme_content = fs::read_to_string("README.md")?;
    
    // Extract Rust code blocks
    let code_block_regex = Regex::new(r"```rust\n((?:[^`]|\n)+?)```")?;
    let mut examples = Vec::new();
    
    for (i, captures) in code_block_regex.captures_iter(&readme_content).enumerate() {
        if let Some(code) = captures.get(1) {
            examples.push((i + 1, code.as_str()));
        }
    }
    
    println!("Found {} Rust code examples in README.md", examples.len());
    
    // Create temporary directory for testing
    let temp_dir = std::env::temp_dir().join("ai_workflow_doc_tests");
    fs::create_dir_all(&temp_dir)?;
    
    let mut passed = 0;
    let mut failed = 0;
    
    for (example_num, code) in examples {
        print!("Testing example {}... ", example_num);
        std::io::stdout().flush()?;
        
        // Skip dependency examples or incomplete snippets
        if code.trim().starts_with("[dependencies]") {
            println!("⏭️  Skipped (dependency declaration)");
            continue;
        }
        
        // Create a test file with proper imports and main function
        let test_file_path = temp_dir.join(format!("test_example_{}.rs", example_num));
        let mut test_code = String::new();
        
        // Add necessary imports and crate references
        test_code.push_str("extern crate backend;\n");
        test_code.push_str("use std::error::Error;\n\n");
        
        // Check if the code already has a main function
        if !code.contains("fn main()") && !code.contains("async fn") {
            // Wrap in a main function
            test_code.push_str("#[tokio::main]\n");
            test_code.push_str("async fn main() -> Result<(), Box<dyn Error>> {\n");
            test_code.push_str(code);
            test_code.push_str("\n    Ok(())\n}\n");
        } else {
            test_code.push_str(code);
        }
        
        // Write test file
        fs::write(&test_file_path, test_code)?;
        
        // Try to compile the example
        let output = Command::new("rustc")
            .arg("--edition=2021")
            .arg("--crate-type=bin")
            .arg("--extern")
            .arg("backend=target/debug/libbackend.rlib")
            .arg("--extern")
            .arg("tokio=target/debug/deps/libtokio-*.rlib")
            .arg("--extern")
            .arg("serde_json=target/debug/deps/libserde_json-*.rlib")
            .arg("-L")
            .arg("target/debug/deps")
            .arg(&test_file_path)
            .output()?;
        
        if output.status.success() {
            println!("✅ Passed");
            passed += 1;
        } else {
            println!("❌ Failed");
            failed += 1;
            
            // Print error details
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("Error details:");
            println!("{}", stderr);
            println!("Test code:");
            println!("{}", test_code);
            println!("-".repeat(80));
        }
        
        // Clean up
        let _ = fs::remove_file(&test_file_path);
    }
    
    // Summary
    println!("\n📊 Summary:");
    println!("  ✅ Passed: {}", passed);
    println!("  ❌ Failed: {}", failed);
    println!("  Total: {}", passed + failed);
    
    if failed > 0 {
        std::process::exit(1);
    }
    
    Ok(())
}