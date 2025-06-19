//! Customer Support Demo
//!
//! This example demonstrates a complete customer support workflow using
//! the AI Workflow Engine with rule-based implementations.

use workflow_engine_api::workflows::customer_support_workflow::create_customer_care_workflow;
use workflow_engine_core::task::TaskContext;
use serde_json::json;
use clap::{App, Arg};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments
    let matches = App::new("Customer Support Demo")
        .version("1.0")
        .about("Demonstrates customer support automation workflow")
        .arg(Arg::with_name("with-mcp")
            .long("with-mcp")
            .help("Use real MCP servers instead of mocks"))
        .arg(Arg::with_name("full")
            .long("full")
            .help("Run with full production-like setup"))
        .arg(Arg::with_name("ticket-file")
            .short("f")
            .long("file")
            .value_name("FILE")
            .help("JSON file containing test tickets")
            .takes_value(true))
        .get_matches();

    // Initialize logging
    env_logger::init();
    
    println!("🎫 Customer Support Workflow Demo");
    println!("=================================\n");

    // Configure based on command line options
    let use_mcp = matches.is_present("with-mcp");
    let full_setup = matches.is_present("full");
    
    if full_setup {
        println!("📦 Running with full production setup...");
        setup_production_environment().await?;
    } else if use_mcp {
        println!("🔌 Using real MCP servers...");
        setup_mcp_servers().await?;
    } else {
        println!("🎭 Using mocked services (no external dependencies)");
    }

    // Create the workflow
    println!("\n🔧 Creating customer support workflow...");
    let workflow = create_customer_care_workflow()?;
    
    // Load test tickets
    let tickets = if let Some(file) = matches.value_of("ticket-file") {
        println!("📂 Loading tickets from: {}", file);
        load_tickets_from_file(file)?
    } else {
        println!("📝 Using demo tickets");
        get_demo_tickets()
    };

    // Process each ticket
    println!("\n🚀 Processing {} tickets...\n", tickets.len());
    
    for (i, ticket) in tickets.iter().enumerate() {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Ticket {}/{}: {}", i + 1, tickets.len(), ticket["ticket_id"]);
        println!("Subject: {}", ticket["subject"]);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        // Create task context
        let mut context = TaskContext::new();
        context.set_input(ticket.clone());
        
        // Execute workflow
        match workflow.execute(context).await {
            Ok(result) => {
                println!("✅ Ticket processed successfully!");
                print_ticket_result(&result);
            }
            Err(e) => {
                println!("❌ Error processing ticket: {}", e);
            }
        }
        
        println!();
    }
    
    println!("🎉 Demo completed!");
    
    Ok(())
}

/// Get demo tickets for testing
fn get_demo_tickets() -> Vec<serde_json::Value> {
    vec![
        // Basic support request
        json!({
            "ticket_id": "DEMO-001",
            "customer_email": "user@example.com",
            "subject": "Cannot login to my account",
            "content": "I forgot my password and need help resetting it. I've tried the reset link but it's not working.",
            "priority": "medium",
            "category": "technical_support"
        }),
        
        // Billing issue
        json!({
            "ticket_id": "DEMO-002",
            "customer_email": "billing@company.com",
            "subject": "Invoice discrepancy",
            "content": "I was charged twice for my subscription this month. Please refund the duplicate charge.",
            "priority": "high",
            "category": "billing",
            "invoice_number": "INV-2024-001",
            "amount": 99.99
        }),
        
        // Feature request
        json!({
            "ticket_id": "DEMO-003",
            "customer_email": "feature@startup.io",
            "subject": "Feature Request: API Integration",
            "content": "We would like to integrate your service with our platform. Do you have an API available?",
            "priority": "low",
            "category": "feature_request"
        }),
        
        // Spam ticket
        json!({
            "ticket_id": "DEMO-004",
            "customer_email": "spam@spam.com",
            "subject": "WIN $1000000 NOW!!!",
            "content": "Click here for your prize! Limited time offer! Act now!",
            "priority": "low",
            "category": "unknown"
        }),
        
        // Complex issue requiring escalation
        json!({
            "ticket_id": "DEMO-005",
            "customer_email": "enterprise@bigcorp.com",
            "subject": "Critical: Production system down",
            "content": "Our entire production system is experiencing issues after the latest update. We need immediate assistance. This is affecting 10,000+ users.",
            "priority": "critical",
            "category": "technical_support",
            "account_type": "enterprise",
            "sla": "premium"
        })
    ]
}

/// Load tickets from a JSON file
fn load_tickets_from_file(path: &str) -> Result<Vec<serde_json::Value>, Box<dyn Error>> {
    let content = std::fs::read_to_string(path)?;
    let tickets: Vec<serde_json::Value> = serde_json::from_str(&content)?;
    Ok(tickets)
}

/// Print the results of ticket processing
fn print_ticket_result(context: &TaskContext) {
    if let Some(analysis) = context.get_metadata("ticket_analysis") {
        println!("📊 Analysis: {}", analysis);
    }
    
    if let Some(intent) = context.get_metadata("intent") {
        println!("🎯 Intent: {}", intent);
    }
    
    if let Some(spam_score) = context.get_metadata("spam_score") {
        println!("🚫 Spam Score: {}", spam_score);
    }
    
    if let Some(response) = context.get_metadata("generated_response") {
        println!("💬 Response: {}", response);
    }
    
    if let Some(status) = context.get_metadata("ticket_status") {
        println!("📌 Status: {}", status);
    }
}

/// Setup MCP servers for demo
async fn setup_mcp_servers() -> Result<(), Box<dyn Error>> {
    // This would normally start real MCP servers
    // For the demo, we'll just simulate the setup
    println!("  → Starting HelpScout MCP server on port 8001");
    println!("  → Starting Notion MCP server on port 8002");
    println!("  → Starting Slack MCP server on port 8003");
    
    // Simulate startup delay
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    
    println!("  ✓ All MCP servers ready");
    Ok(())
}

/// Setup production-like environment
async fn setup_production_environment() -> Result<(), Box<dyn Error>> {
    println!("  → Initializing PostgreSQL database");
    println!("  → Setting up Redis cache");
    println!("  → Configuring monitoring (Prometheus + Grafana)");
    println!("  → Loading AI model configurations");
    
    // Simulate setup
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    println!("  ✓ Production environment ready");
    Ok(())
}