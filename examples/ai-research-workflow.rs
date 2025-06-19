//! AI-powered research workflow example
//! This example demonstrates how to build a custom workflow using the WorkflowBuilder API

use workflow_engine_core::workflow::builder::WorkflowBuilder;
use workflow_engine_core::nodes::{config::NodeConfig, agent::BaseAgentNode};
use workflow_engine_core::mcp::clients::notion::NotionClientNode;
use serde_json::json;
use std::any::TypeId;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    println!("🔬 Building AI-powered research workflow...");
    
    // Build a research workflow using Notion as knowledge source and AI for analysis
    let workflow = WorkflowBuilder::new::<NotionClientNode>("ai_research".to_string())
        .description("AI-powered research and analysis workflow".to_string())
        .add_node(
            NodeConfig::new::<NotionClientNode>()
                .with_description("Search Notion for information".to_string())
                .with_connections(vec![TypeId::of::<BaseAgentNode>()])
        )
        .add_node(
            NodeConfig::new::<BaseAgentNode>()
                .with_description("Analyze and summarize findings".to_string())
        )
        .build()?;
    
    // Create execution context
    let context = json!({
        "query": "Latest developments in Rust async programming",
        "sources": ["docs.rs", "github.com", "blog.rust-lang.org"],
        "model": "gpt-4",
        "max_results": 10,
        "include_code_examples": true
    });
    
    println!("🔍 Research query: Latest developments in Rust async programming");
    println!("📚 Sources: docs.rs, github.com, blog.rust-lang.org");
    println!("🤖 AI Model: GPT-4");
    println!("\n🚀 Executing research workflow...");
    
    // Execute workflow
    match workflow.run(context) {
        Ok(result) => {
            println!("✅ Research workflow completed successfully!");
            
            // Extract research results
            if let Some(research_data) = result.get_data::<serde_json::Value>("research_results").ok().flatten() {
                println!("\n📊 Research Results:");
                println!("{:#?}", research_data);
            }
            
            // Extract AI analysis
            if let Some(analysis) = result.get_data::<String>("ai_analysis").ok().flatten() {
                println!("\n🧠 AI Analysis:");
                println!("{}", analysis);
            }
            
            // Extract summary
            if let Some(summary) = result.get_data::<String>("summary").ok().flatten() {
                println!("\n📝 Summary:");
                println!("{}", summary);
            }
        }
        Err(e) => {
            eprintln!("❌ Research workflow failed: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}