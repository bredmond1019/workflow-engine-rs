//! Basic workflow example demonstrating the core API usage
//! This example shows how to create and execute a simple knowledge base workflow

use backend::core::task::TaskContext;
use backend::workflows::knowledge_base_workflow::create_knowledge_base_workflow;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    // Create a knowledge base workflow
    let workflow = create_knowledge_base_workflow()?;
    
    // Create task context with query data
    let task_context = TaskContext::new(
        "knowledge_base".to_string(),
        json!({
            "query_id": "QUERY-001",
            "user_id": "USER-123",
            "user_query": "How do I configure SSL certificates?",
            "query_type": "technical",
            "sources": ["notion", "helpscout", "slack"]
        })
    );
    
    println!("🚀 Starting knowledge base workflow...");
    println!("📝 Query: How do I configure SSL certificates?");
    
    // Execute the workflow
    match workflow.run(task_context.event_data) {
        Ok(result) => {
            println!("✅ Workflow completed successfully!");
            
            // Check if we got search results
            if let Some(notion_results) = result.get_data::<serde_json::Value>("notion_search_results").ok().flatten() {
                println!("\n📚 Notion Results:");
                println!("{:#?}", notion_results);
            }
            
            if let Some(helpscout_results) = result.get_data::<serde_json::Value>("helpscout_search_results").ok().flatten() {
                println!("\n🎫 HelpScout Results:");
                println!("{:#?}", helpscout_results);
            }
            
            if let Some(slack_results) = result.get_data::<serde_json::Value>("slack_search_results").ok().flatten() {
                println!("\n💬 Slack Results:");
                println!("{:#?}", slack_results);
            }
        }
        Err(e) => {
            eprintln!("❌ Workflow failed: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}