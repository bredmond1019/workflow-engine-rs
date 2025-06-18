// =============================================================================
// Knowledge Base Workflow Example
// =============================================================================

use backend::{
    core::{task::TaskContext, workflow::Workflow},
    workflows::knowledge_base_workflow::create_knowledge_base_workflow,
};
use serde_json::{Value, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create the knowledge base workflow
    let workflow = create_knowledge_base_workflow()?;

    // Simulate a user query
    let user_query = "How do I reset my password?";

    // Create task context with the user query
    let mut task_context = TaskContext::new(
        "knowledge_base".to_string(),
        json!({
            "user_query": user_query,
            "user_id": "user_123",
            "timestamp": "2024-01-15T10:30:00Z"
        }),
    );

    // Add the user query to the context
    task_context.set_data("user_query", user_query)?;

    println!("🔍 Processing knowledge base query: {}", user_query);
    println!("📋 Starting workflow...\n");

    // Run the workflow
    match workflow.run(task_context.event_data) {
        Ok(result_context) => {
            println!("✅ Workflow completed successfully!\n");

            // Display results
            if let Some(Value::Bool(true)) = result_context
                .get_data::<Value>("reply_sent")
                .unwrap_or(None)
            {
                if let Some(response) = result_context
                    .get_data::<String>("reply_content")
                    .unwrap_or(None)
                {
                    println!("📝 Generated Response:");
                    println!("{}", response);
                    println!();
                }

                if let Some(sources) = result_context
                    .get_data::<Value>("response_sources")
                    .unwrap_or(None)
                {
                    println!("📚 Sources Found:");
                    if let Some(sources_array) = sources.as_array() {
                        for (i, source) in sources_array.iter().enumerate() {
                            if let Some(source_obj) = source.as_object() {
                                let source_type = source_obj
                                    .get("type")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("unknown");
                                let title = source_obj
                                    .get("title")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("No title");
                                let url = source_obj
                                    .get("url")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("No URL");
                                println!("  {}. {} ({}): {}", i + 1, title, source_type, url);
                            }
                        }
                    }
                    println!();
                }

                // Show search statistics
                if let Some(total_results) = result_context
                    .get_data::<Value>("total_results_found")
                    .unwrap_or(None)
                {
                    println!("📊 Search Statistics:");
                    println!(
                        "  - Total results found: {}",
                        total_results.as_u64().unwrap_or(0)
                    );

                    if let Some(high_relevance) = result_context
                        .get_data::<Value>("high_relevance_count")
                        .unwrap_or(None)
                    {
                        println!(
                            "  - High relevance results: {}",
                            high_relevance.as_u64().unwrap_or(0)
                        );
                    }

                    if let Some(sources_searched) = result_context
                        .get_data::<Value>("sources_searched")
                        .unwrap_or(None)
                    {
                        if let Some(sources_array) = sources_searched.as_array() {
                            let source_names: Vec<String> = sources_array
                                .iter()
                                .filter_map(|v| v.as_str())
                                .map(|s| s.to_string())
                                .collect();
                            println!("  - Sources searched: {}", source_names.join(", "));
                        }
                    }
                    println!();
                }
            } else {
                println!("❌ Failed to generate response");
            }
        }
        Err(e) => {
            println!("❌ Workflow failed: {}", e);
            return Err(e.into());
        }
    }

    println!("🏁 Knowledge base workflow example completed!");

    Ok(())
}
