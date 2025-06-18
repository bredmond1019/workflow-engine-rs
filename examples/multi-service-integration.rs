//! Multi-service integration example
//! This example shows how to use the customer support workflow with MCP integrations

use backend::workflows::customer_support_workflow::create_customer_care_workflow;
use backend::core::task::TaskContext;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    println!("🎫 Customer Support Workflow with Multi-Service Integration");
    println!("{}", "=".repeat(60));
    
    // Create a customer support workflow with MCP integrations
    let workflow = create_customer_care_workflow()?;
    
    // Create task context with ticket data
    let task_context = TaskContext::new(
        "customer_support".to_string(),
        json!({
            "ticket_id": "TICKET-12345",
            "customer_email": "user@example.com",
            "subject": "Password Reset Request",
            "description": "I forgot my password and need help resetting it. I've tried the reset link but it's not working.",
            "priority": "normal",
            "category": "account_access",
            "customer_name": "John Doe",
            "account_id": "ACC-98765",
            "created_at": "2024-01-15T10:30:00Z"
        })
    );
    
    println!("📧 Processing ticket: TICKET-12345");
    println!("👤 Customer: John Doe (user@example.com)");
    println!("📝 Subject: Password Reset Request");
    println!("🏷️ Category: Account Access");
    println!("⚡ Priority: Normal");
    println!("\n🚀 Starting workflow execution...\n");
    
    // Execute the workflow
    match workflow.run(task_context.event_data) {
        Ok(result) => {
            println!("✅ Workflow completed successfully!\n");
            
            // Check ticket analysis results
            if let Some(analysis) = result.get_data::<serde_json::Value>("ticket_analysis").ok().flatten() {
                println!("🔍 Ticket Analysis:");
                println!("{:#?}", analysis);
            }
            
            // Check intent determination
            if let Some(intent) = result.get_data::<String>("ticket_intent").ok().flatten() {
                println!("\n🎯 Detected Intent: {}", intent);
            }
            
            // Check spam status
            if let Some(is_spam) = result.get_data::<bool>("is_spam").ok().flatten() {
                println!("🚫 Spam Status: {}", if is_spam { "SPAM DETECTED" } else { "Not spam" });
            }
            
            // Check routing decision
            if let Some(routing) = result.get_data::<serde_json::Value>("routing_decision").ok().flatten() {
                println!("\n📍 Routing Decision:");
                println!("{:#?}", routing);
            }
            
            // Check if ticket was processed
            if let Some(ticket_processed) = result.get_data::<bool>("ticket_processed").ok().flatten() {
                println!("\n✅ Ticket Processing Status: {}", 
                    if ticket_processed { "Successfully processed" } else { "Processing failed" }
                );
            }
            
            // Check for automated response
            if let Some(response) = result.get_data::<String>("automated_response").ok().flatten() {
                println!("\n💬 Automated Response Generated:");
                println!("{}", response);
            }
            
            // Check for knowledge base articles
            if let Some(kb_articles) = result.get_data::<serde_json::Value>("knowledge_base_articles").ok().flatten() {
                println!("\n📚 Related Knowledge Base Articles:");
                println!("{:#?}", kb_articles);
            }
        }
        Err(e) => {
            eprintln!("❌ Workflow execution failed: {}", e);
            eprintln!("💡 Tip: Make sure MCP servers are running:");
            eprintln!("   - HelpScout MCP: ws://localhost:8003");
            eprintln!("   - Notion MCP: ws://localhost:8001");
            eprintln!("   - Slack MCP: ws://localhost:8002");
            return Err(e.into());
        }
    }
    
    Ok(())
}