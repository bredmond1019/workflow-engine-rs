#![allow(warnings)]

use backend::workflows::demos;
use std::env;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Please specify which demo to run:");
        println!("Usage: cargo run --bin demo <demo_name>");
        println!("\nAvailable demos:");
        println!("  customer_care        - Run the Customer Care Workflow demo");
        println!("  customer_care_mcp    - Run the Customer Care MCP Integration demo");
        println!("  knowledge_base       - Run the Knowledge Base Workflow demo");
        println!("  knowledge_base_mcp   - Run the Knowledge Base MCP Integration demo");
        return;
    }

    match args[1].as_str() {
        "customer_care" => {
            println!("Running Customer Care Workflow demo...");
            demos::customer_care_workflow::customer_care_workflow_demo().await;
        }
        "customer_care_mcp" => {
            println!("Running Customer Care MCP Integration demo...");
            demos::customer_care_mcp::customer_care_mcp_demo().await;
        }
        "knowledge_base" => {
            println!("Running Knowledge Base Workflow demo...");
            demos::knowledge_base_workflow::knowledge_base_workflow_demo().await;
        }
        "knowledge_base_mcp" => {
            println!("Running Knowledge Base MCP Integration demo...");
            demos::knowledge_base_mcp::knowledge_base_mcp_demo().await;
        }
        _ => {
            println!("Unknown demo: {}", args[1]);
            println!("Available demos:");
            println!("  customer_care        - Run the Customer Care Workflow demo");
            println!("  customer_care_mcp    - Run the Customer Care MCP Integration demo");
            println!("  knowledge_base       - Run the Knowledge Base Workflow demo");
            println!("  knowledge_base_mcp   - Run the Knowledge Base MCP Integration demo");
        }
    }
}
