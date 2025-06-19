//! # Template-Based Workflow Example
//!
//! This example demonstrates how to use the template system
//! with AI agents in a workflow.

use workflow_engine_core::{
    ai::templates::{TemplateManager, Template, OutputFormat, VariableType},
    nodes::{
        agent::{AgentConfig, ModelProvider},
        template_agent::{TemplateAgentNode, TemplateAgentBuilder, register_default_agent_templates},
        config::NodeConfig,
    },
    task::TaskContext,
    workflow::builder::WorkflowBuilder,
    error::WorkflowError,
};
use serde_json::json;
use std::{collections::HashMap, sync::Arc};

#[tokio::main]
async fn main() -> Result<(), WorkflowError> {
    // Initialize template manager
    let mut template_manager = TemplateManager::new()?;
    
    // Register default agent templates
    register_default_agent_templates(&mut template_manager)?;
    
    // Register custom templates for customer support
    register_customer_support_templates(&mut template_manager)?;
    
    let template_manager = Arc::new(template_manager);
    
    // Create AI agent configuration
    let agent_config = AgentConfig {
        system_prompt: "Default system prompt".to_string(), // Will be overridden by template
        model_provider: ModelProvider::Anthropic,
        model_name: "claude-3-sonnet-20240229".to_string(),
        mcp_server_uri: Some("ws://localhost:8001/mcp".to_string()),
    };
    
    // Create template-enhanced agent configuration
    let template_config = TemplateAgentBuilder::new(agent_config)
        .with_system_prompt_template("customer_support_system")
        .with_user_message_template("customer_support_message")
        .with_template_var("department", json!("Customer Support"))
        .with_template_var("company", json!("AI Workflow Corp"))
        .with_template_based_tools()
        .build();
    
    // Create the template agent node
    let agent_node = TemplateAgentNode::new(template_config, template_manager.clone())?;
    
    // Create and configure workflow
    let workflow = WorkflowBuilder::new::<TemplateAgentNode>("customer_support_workflow".to_string())
        .description("Customer Support Workflow with Template Agent".to_string())
        .add_node(
            NodeConfig::new::<TemplateAgentNode>()
                .with_description("Template-enhanced customer support agent".to_string())
        )
        .build()?;
    
    // Register the template agent node
    workflow.register_node(agent_node);
    
    // Create test context for customer support ticket
    let task_context = TaskContext::new(
        "customer_support_ticket".to_string(),
        json!({
            "prompt": "Help me reset my password",
            "customer_id": "CUST-12345",
            "priority": "medium",
            "category": "account_access",
            "previous_interactions": [
                {"date": "2024-01-01", "type": "email", "summary": "Initial password issues"},
                {"date": "2024-01-02", "type": "chat", "summary": "Attempted self-service"}
            ]
        })
    );
    
    // Execute workflow
    println!("Executing customer support workflow...");
    let result = workflow.run(task_context.get_event_data()?)?;
    
    // Display results
    println!("Workflow completed successfully!");
    println!("Result: {}", serde_json::to_string_pretty(&result.get_all_data())?);
    
    // Demonstrate template metrics
    let metrics = template_manager.metrics();
    println!("\nTemplate Performance Metrics:");
    println!("Cache hit rate: {:.2}%", metrics.cache_hit_rate() * 100.0);
    
    // Show render times for different templates
    for template_id in ["customer_support_system", "customer_support_message"] {
        if let Some(avg_time) = metrics.average_render_time(template_id) {
            println!("Average render time for '{}': {:?}", template_id, avg_time);
        }
    }
    
    Ok(())
}

/// Register customer support specific templates
fn register_customer_support_templates(
    template_manager: &mut TemplateManager,
) -> Result<(), WorkflowError> {
    // Customer support system prompt
    let system_prompt = Template::new(
        "customer_support_system",
        r#"You are an AI customer support assistant for {{company}} in the {{department}} department.

Your role is to:
- Provide helpful and accurate information
- Resolve customer issues efficiently
- Escalate complex problems when necessary
- Maintain a professional and empathetic tone

{{#if tools_available}}
You have access to the following tools:
{{#each available_tools}}
- {{this.name}}: {{this.description}}
{{/each}}

Use these tools when appropriate to provide better assistance.
{{/if}}

Always prioritize customer satisfaction while following company policies."#
    )?
    .with_variable("company", VariableType::String)
    .with_variable("department", VariableType::String)
    .with_variable("tools_available", VariableType::Boolean)
    .with_variable("available_tools", VariableType::Array(Box::new(VariableType::Object(
        HashMap::from([
            ("name".to_string(), VariableType::String),
            ("description".to_string(), VariableType::String),
        ])
    ))));
    
    template_manager.register(system_prompt)?;
    
    // Customer support message template
    let message_template = Template::new(
        "customer_support_message",
        r#"Customer Support Request

Customer ID: {{customer_id}}
Priority: {{priority}}
Category: {{category}}

{{#if previous_interactions}}
Previous Interactions:
{{#each previous_interactions}}
- {{this.date}} ({{this.type}}): {{this.summary}}
{{/each}}

{{/if}}
Current Request: {{prompt}}

{{#if workflow_context}}
Additional Context:
{{json workflow_context}}
{{/if}}

Please provide assistance for this customer request."#
    )?
    .with_variable("customer_id", VariableType::String)
    .with_variable("priority", VariableType::String)
    .with_variable("category", VariableType::String)
    .with_variable("prompt", VariableType::String)
    .with_variable("previous_interactions", VariableType::Array(Box::new(VariableType::Object(
        HashMap::from([
            ("date".to_string(), VariableType::String),
            ("type".to_string(), VariableType::String),
            ("summary".to_string(), VariableType::String),
        ])
    ))))
    .with_variable("workflow_context", VariableType::Any);
    
    template_manager.register(message_template)?;
    
    // Response formatting template
    let response_template = Template::new(
        "customer_support_response",
        r#"{
    "ticket_id": "{{ticket_id}}",
    "status": "{{status}}",
    "response": "{{response}}",
    "next_steps": [
        {{#each next_steps}}
        "{{this}}"{{#unless @last}},{{/unless}}
        {{/each}}
    ],
    "escalation_needed": {{escalation_needed}},
    "estimated_resolution_time": "{{resolution_time}}",
    "follow_up_required": {{follow_up_required}}
}"#
    )?
    .with_output_format(OutputFormat::Json)
    .with_variable("ticket_id", VariableType::String)
    .with_variable("status", VariableType::String)
    .with_variable("response", VariableType::String)
    .with_variable("next_steps", VariableType::Array(Box::new(VariableType::String)))
    .with_variable("escalation_needed", VariableType::Boolean)
    .with_variable("resolution_time", VariableType::String)
    .with_variable("follow_up_required", VariableType::Boolean);
    
    template_manager.register(response_template)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_template_workflow_setup() {
        let mut template_manager = TemplateManager::new().unwrap();
        
        // Register templates
        register_default_agent_templates(&mut template_manager).unwrap();
        register_customer_support_templates(&mut template_manager).unwrap();
        
        // Verify templates are registered
        let templates = template_manager.list_templates().unwrap();
        assert!(templates.iter().any(|t| t.id.0 == "customer_support_system"));
        assert!(templates.iter().any(|t| t.id.0 == "customer_support_message"));
        assert!(templates.iter().any(|t| t.id.0 == "customer_support_response"));
    }
    
    #[test]
    fn test_template_rendering() {
        let mut template_manager = TemplateManager::new().unwrap();
        register_customer_support_templates(&mut template_manager).unwrap();
        
        let mut vars = HashMap::new();
        vars.insert("company".to_string(), json!("Test Corp"));
        vars.insert("department".to_string(), json!("Support"));
        vars.insert("tools_available".to_string(), json!(false));
        
        let result = template_manager.render("customer_support_system", &vars).unwrap();
        assert!(result.contains("Test Corp"));
        assert!(result.contains("Support"));
    }
}