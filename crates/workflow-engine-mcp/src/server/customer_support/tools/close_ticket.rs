use crate::server::ToolMetadata;
use workflow_engine_core::{error::WorkflowError, nodes::Node, task::TaskContext};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::super::server::CustomerSupportMcpServer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketClosureResult {
    pub ticket_id: String,
    pub resolution: String,
    pub closure_reason: String,
    pub customer_satisfaction: Option<f32>,
    pub follow_up_required: bool,
    pub follow_up_date: Option<String>,
    pub closure_type: String,
    pub time_to_resolution: Option<String>,
    pub tags: Vec<String>,
    pub knowledge_base_updated: bool,
}

#[derive(Debug, Clone)]
pub struct CloseTicketNode;

impl CloseTicketNode {
    pub fn new() -> Self {
        Self
    }
    
    fn validate_closure_request(
        &self,
        ticket_id: &str,
        resolution: &str,
    ) -> Result<(), WorkflowError> {
        if resolution.trim().is_empty() {
            return Err(WorkflowError::ValidationError {
                message: "Resolution cannot be empty".to_string(),
            });
        }
        
        if resolution.len() < 10 {
            return Err(WorkflowError::ValidationError {
                message: "Resolution must be at least 10 characters long".to_string(),
            });
        }
        
        // Check if ticket exists and is eligible for closure
        self.check_closure_eligibility(ticket_id)?;
        
        Ok(())
    }
    
    fn check_closure_eligibility(&self, ticket_id: &str) -> Result<(), WorkflowError> {
        // In a real implementation, this would check the ticket database
        log::info!("Checking closure eligibility for ticket {}", ticket_id);
        
        // Simulate eligibility check (always passes in this simulation)
        Ok(())
    }
    
    fn execute_ticket_closure(
        &self,
        ticket_id: &str,
        resolution: &str,
        customer_satisfaction: Option<f32>,
        follow_up_required: bool,
    ) -> Result<TicketClosureResult, WorkflowError> {
        // Determine closure type based on resolution content
        let closure_type = self.determine_closure_type(resolution);
        
        // Calculate time to resolution (simulated)
        let time_to_resolution = self.calculate_time_to_resolution(ticket_id)?;
        
        // Generate tags based on resolution
        let tags = self.generate_closure_tags(resolution, &closure_type);
        
        // Determine follow-up requirements
        let (follow_up_required, follow_up_date) = self.determine_follow_up(
            follow_up_required,
            &closure_type,
            customer_satisfaction,
        )?;
        
        // Update knowledge base if applicable
        let knowledge_base_updated = self.update_knowledge_base_if_needed(
            ticket_id,
            resolution,
            &closure_type,
        )?;
        
        // Send closure notifications
        self.send_closure_notifications(
            ticket_id,
            &closure_type,
            customer_satisfaction,
        )?;
        
        Ok(TicketClosureResult {
            ticket_id: ticket_id.to_string(),
            resolution: resolution.to_string(),
            closure_reason: self.generate_closure_reason(&closure_type),
            customer_satisfaction,
            follow_up_required,
            follow_up_date,
            closure_type,
            time_to_resolution,
            tags,
            knowledge_base_updated,
        })
    }
    
    fn determine_closure_type(&self, resolution: &str) -> String {
        let resolution_lower = resolution.to_lowercase();
        
        if resolution_lower.contains("resolved") || resolution_lower.contains("fixed") {
            "resolved".to_string()
        } else if resolution_lower.contains("duplicate") {
            "duplicate".to_string()
        } else if resolution_lower.contains("spam") || resolution_lower.contains("invalid") {
            "invalid".to_string()
        } else if resolution_lower.contains("no response") || resolution_lower.contains("timeout") {
            "no_response".to_string()
        } else if resolution_lower.contains("escalated") {
            "escalated".to_string()
        } else {
            "completed".to_string()
        }
    }
    
    fn calculate_time_to_resolution(&self, ticket_id: &str) -> Result<Option<String>, WorkflowError> {
        // In a real implementation, this would calculate actual time from ticket creation
        log::info!("Calculating time to resolution for ticket {}", ticket_id);
        
        // Simulate time calculation
        Ok(Some("2 hours 30 minutes".to_string()))
    }
    
    fn generate_closure_tags(&self, resolution: &str, closure_type: &str) -> Vec<String> {
        let mut tags = vec![closure_type.to_string()];
        let resolution_lower = resolution.to_lowercase();
        
        // Add tags based on resolution content
        if resolution_lower.contains("billing") {
            tags.push("billing".to_string());
        }
        if resolution_lower.contains("technical") {
            tags.push("technical".to_string());
        }
        if resolution_lower.contains("product") {
            tags.push("product".to_string());
        }
        if resolution_lower.contains("refund") {
            tags.push("refund".to_string());
        }
        if resolution_lower.contains("password") || resolution_lower.contains("login") {
            tags.push("authentication".to_string());
        }
        if resolution_lower.contains("bug") || resolution_lower.contains("error") {
            tags.push("bug_report".to_string());
        }
        
        tags
    }
    
    fn determine_follow_up(
        &self,
        follow_up_required: bool,
        closure_type: &str,
        customer_satisfaction: Option<f32>,
    ) -> Result<(bool, Option<String>), WorkflowError> {
        let mut requires_follow_up = follow_up_required;
        
        // Automatically require follow-up for certain closure types
        if closure_type == "escalated" || closure_type == "no_response" {
            requires_follow_up = true;
        }
        
        // Require follow-up for low satisfaction scores
        if let Some(satisfaction) = customer_satisfaction {
            if satisfaction < 3.0 {
                requires_follow_up = true;
            }
        }
        
        let follow_up_date = if requires_follow_up {
            let follow_up_days = match closure_type {
                "escalated" => 1,
                "no_response" => 3,
                _ => if customer_satisfaction.map_or(false, |s| s < 3.0) { 2 } else { 7 },
            };
            
            let follow_up_date = chrono::Utc::now() + chrono::Duration::days(follow_up_days);
            Some(follow_up_date.format("%Y-%m-%d").to_string())
        } else {
            None
        };
        
        Ok((requires_follow_up, follow_up_date))
    }
    
    fn update_knowledge_base_if_needed(
        &self,
        ticket_id: &str,
        resolution: &str,
        closure_type: &str,
    ) -> Result<bool, WorkflowError> {
        // Update knowledge base for resolved issues that might help other customers
        let should_update = closure_type == "resolved" && 
                           resolution.len() > 50 && 
                           !resolution.to_lowercase().contains("customer-specific");
        
        if should_update {
            log::info!("Updating knowledge base with resolution from ticket {}", ticket_id);
            // In a real implementation, this would add the resolution to a knowledge base
        }
        
        Ok(should_update)
    }
    
    fn send_closure_notifications(
        &self,
        ticket_id: &str,
        closure_type: &str,
        customer_satisfaction: Option<f32>,
    ) -> Result<(), WorkflowError> {
        // Send notification to customer
        log::info!("Sending closure notification to customer for ticket {}", ticket_id);
        
        // Send internal notifications for specific closure types
        match closure_type {
            "escalated" => {
                log::info!("Sending escalation closure notification for ticket {}", ticket_id);
            }
            "no_response" => {
                log::info!("Sending no-response closure notification for ticket {}", ticket_id);
            }
            _ => {}
        }
        
        // Send satisfaction survey if no rating provided
        if customer_satisfaction.is_none() {
            log::info!("Sending customer satisfaction survey for ticket {}", ticket_id);
        }
        
        Ok(())
    }
    
    fn generate_closure_reason(&self, closure_type: &str) -> String {
        match closure_type {
            "resolved" => "Issue successfully resolved".to_string(),
            "duplicate" => "Duplicate of existing ticket".to_string(),
            "invalid" => "Invalid or spam request".to_string(),
            "no_response" => "Closed due to no customer response".to_string(),
            "escalated" => "Escalated to appropriate team".to_string(),
            "completed" => "Request completed successfully".to_string(),
            _ => "Ticket processed and closed".to_string(),
        }
    }

    pub async fn register(server: &mut CustomerSupportMcpServer) -> Result<(), WorkflowError> {
        let node = Arc::new(Self::new());
        let metadata = ToolMetadata::new(
            "close_ticket".to_string(),
            "Closes resolved customer support ticket with appropriate resolution summary"
                .to_string(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "context_data": {
                        "type": "object",
                        "properties": {
                            "ticket_id": {"type": "string", "description": "Ticket to close"},
                            "resolution": {"type": "string", "description": "Resolution summary"},
                            "customer_satisfaction": {"type": "number", "minimum": 1, "maximum": 5, "description": "Customer satisfaction rating (optional)"},
                            "follow_up_required": {"type": "boolean", "description": "Whether follow-up is needed"}
                        },
                        "required": ["ticket_id", "resolution"]
                    }
                },
                "required": ["context_data"]
            }),
            std::any::TypeId::of::<Self>(),
        );
        server
            .get_server()
            .register_node_as_tool(node, metadata)
            .await?;
        Ok(())
    }
}

impl Node for CloseTicketNode {
    fn process(&self, mut task_context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Extract closure parameters from context
        let context_data = task_context.get_data::<serde_json::Value>("context_data")?;
        
        let ticket_id = context_data
            .as_ref()
            .and_then(|v| v["ticket_id"].as_str())
            .ok_or_else(|| WorkflowError::ValidationError {
                message: "Missing required field: ticket_id".to_string(),
            })?.to_string();
            
        let resolution = context_data
            .as_ref()  
            .and_then(|v| v["resolution"].as_str())
            .ok_or_else(|| WorkflowError::ValidationError {
                message: "Missing required field: resolution".to_string(),
            })?.to_string();
        
        let customer_satisfaction = context_data
            .as_ref()
            .and_then(|v| v["customer_satisfaction"].as_f64())
            .map(|f| f as f32);
            
        let follow_up_required = context_data
            .as_ref()
            .and_then(|v| v["follow_up_required"].as_bool())
            .unwrap_or(false);

        // Validate closure request
        self.validate_closure_request(&ticket_id, &resolution)?;
        
        // Process ticket closure
        let closure_result = self.execute_ticket_closure(
            &ticket_id,
            &resolution,
            customer_satisfaction,
            follow_up_required,
        )?;

        // Log the ticket closure
        log::info!(
            "Closed ticket {} with resolution: {} (Satisfaction: {:?})",
            ticket_id,
            resolution,
            customer_satisfaction
        );

        // Update task context with closure results
        task_context.update_node(&self.node_name(), serde_json::json!({
            "closure_result": closure_result,
            "closed_at": chrono::Utc::now()
        }));
        
        // Update ticket status for backward compatibility
        task_context.update_node("ticket_status", &"closed");
        
        Ok(task_context)
    }
}
