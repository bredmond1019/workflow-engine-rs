use crate::server::ToolMetadata;
use workflow_engine_core::{error::WorkflowError, nodes::Node, task::TaskContext};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::super::server::CustomerSupportMcpServer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub target_department: String,
    pub assigned_agent_id: Option<String>,
    pub priority_level: String,
    pub routing_reason: String,
    pub estimated_resolution_time: Option<String>,
    pub required_skills: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TicketRouterNode;

impl TicketRouterNode {
    pub fn new() -> Self {
        Self
    }
    
    fn determine_routing(
        &self,
        ticket_id: &str,
        department: &str,
        priority: &str,
        routing_reason: &str,
        agent_id: Option<&str>,
    ) -> Result<RoutingDecision, WorkflowError> {
        // Validate department
        let validated_department = self.validate_department(department)?;
        
        // Determine priority level and escalation
        let priority_level = self.normalize_priority(priority);
        
        // Assign agent based on department and availability
        let assigned_agent = self.assign_agent(&validated_department, &priority_level, agent_id)?;
        
        // Estimate resolution time based on department and priority
        let estimated_resolution_time = self.estimate_resolution_time(&validated_department, &priority_level);
        
        // Determine required skills for this ticket
        let required_skills = self.determine_required_skills(&validated_department, routing_reason);
        
        Ok(RoutingDecision {
            target_department: validated_department,
            assigned_agent_id: assigned_agent,
            priority_level,
            routing_reason: routing_reason.to_string(),
            estimated_resolution_time,
            required_skills,
        })
    }
    
    fn validate_department(&self, department: &str) -> Result<String, WorkflowError> {
        match department.to_lowercase().as_str() {
            "billing" | "finance" | "payments" => Ok("billing".to_string()),
            "technical" | "tech" | "engineering" | "support" => Ok("technical".to_string()),
            "sales" | "business" | "commercial" => Ok("sales".to_string()),
            "general" | "customer_service" | "cs" => Ok("general".to_string()),
            _ => {
                log::warn!("Unknown department '{}', routing to general", department);
                Ok("general".to_string())
            }
        }
    }
    
    fn normalize_priority(&self, priority: &str) -> String {
        match priority.to_lowercase().as_str() {
            "critical" | "urgent" | "high" => "high".to_string(),
            "normal" | "medium" | "standard" => "normal".to_string(),
            "low" | "minor" => "low".to_string(),
            _ => "normal".to_string(),
        }
    }
    
    fn assign_agent(
        &self,
        department: &str,
        priority: &str,
        preferred_agent: Option<&str>,
    ) -> Result<Option<String>, WorkflowError> {
        // If a specific agent is requested, validate and assign
        if let Some(agent_id) = preferred_agent {
            if self.validate_agent_for_department(agent_id, department)? {
                return Ok(Some(agent_id.to_string()));
            }
        }
        
        // Otherwise, assign based on department and priority
        let agent_id = match (department, priority) {
            ("billing", "high") => Some("billing_senior_001".to_string()),
            ("billing", _) => Some("billing_agent_001".to_string()),
            ("technical", "high") => Some("tech_lead_001".to_string()),
            ("technical", _) => Some("tech_support_001".to_string()),
            ("sales", "high") => Some("sales_manager_001".to_string()),
            ("sales", _) => Some("sales_agent_001".to_string()),
            ("general", "high") => Some("supervisor_001".to_string()),
            ("general", _) => Some("cs_agent_001".to_string()),
            _ => None,
        };
        
        Ok(agent_id)
    }
    
    fn validate_agent_for_department(
        &self,
        agent_id: &str,
        department: &str,
    ) -> Result<bool, WorkflowError> {
        // In a real implementation, this would check against a database
        // For now, we'll do basic validation based on agent ID patterns
        let is_valid = match department {
            "billing" => agent_id.contains("billing") || agent_id.contains("finance"),
            "technical" => agent_id.contains("tech") || agent_id.contains("eng"),
            "sales" => agent_id.contains("sales") || agent_id.contains("business"),
            "general" => true, // General agents can handle any department
            _ => false,
        };
        
        if !is_valid {
            log::warn!(
                "Agent {} is not qualified for department {}",
                agent_id,
                department
            );
        }
        
        Ok(is_valid)
    }
    
    fn estimate_resolution_time(&self, department: &str, priority: &str) -> Option<String> {
        match (department, priority) {
            ("billing", "high") => Some("2 hours".to_string()),
            ("billing", "normal") => Some("4 hours".to_string()),
            ("billing", "low") => Some("1 day".to_string()),
            ("technical", "high") => Some("4 hours".to_string()),
            ("technical", "normal") => Some("1 day".to_string()),
            ("technical", "low") => Some("2 days".to_string()),
            ("sales", "high") => Some("1 hour".to_string()),
            ("sales", "normal") => Some("2 hours".to_string()),
            ("sales", "low") => Some("4 hours".to_string()),
            ("general", "high") => Some("2 hours".to_string()),
            ("general", "normal") => Some("4 hours".to_string()),
            ("general", "low") => Some("8 hours".to_string()),
            _ => Some("1 day".to_string()),
        }
    }
    
    fn determine_required_skills(&self, department: &str, routing_reason: &str) -> Vec<String> {
        let mut skills = match department {
            "billing" => vec!["billing_systems".to_string(), "payment_processing".to_string()],
            "technical" => vec!["technical_support".to_string(), "troubleshooting".to_string()],
            "sales" => vec!["sales_process".to_string(), "product_knowledge".to_string()],
            "general" => vec!["customer_service".to_string(), "communication".to_string()],
            _ => vec!["general_support".to_string()],
        };
        
        // Add specific skills based on routing reason
        let reason_lower = routing_reason.to_lowercase();
        if reason_lower.contains("refund") {
            skills.push("refund_processing".to_string());
        }
        if reason_lower.contains("technical") || reason_lower.contains("bug") {
            skills.push("technical_diagnosis".to_string());
        }
        if reason_lower.contains("urgent") || reason_lower.contains("escalat") {
            skills.push("escalation_handling".to_string());
        }
        
        skills
    }

    pub async fn register(server: &mut CustomerSupportMcpServer) -> Result<(), WorkflowError> {
        let node = Arc::new(Self::new());
        let metadata = ToolMetadata::new(
            "ticket_router".to_string(),
            "Routes ticket to appropriate department or agent based on ticket analysis".to_string(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "context_data": {
                        "type": "object",
                        "properties": {
                            "ticket_id": {"type": "string", "description": "Ticket to route"},
                            "department": {"type": "string", "enum": ["billing", "technical", "sales", "general"], "description": "Target department"},
                            "agent_id": {"type": "string", "description": "Specific agent ID (optional)"},
                            "priority": {"type": "string", "enum": ["low", "normal", "high", "urgent"], "description": "Routing priority"},
                            "routing_reason": {"type": "string", "description": "Reason for routing decision"}
                        },
                        "required": ["ticket_id", "department", "routing_reason"]
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

impl Node for TicketRouterNode {
    fn process(&self, mut task_context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Extract routing parameters from context
        let context_data = task_context.get_data::<serde_json::Value>("context_data")?;
        
        let ticket_id = context_data
            .as_ref()
            .and_then(|v| v["ticket_id"].as_str())
            .ok_or_else(|| WorkflowError::ValidationError {
                message: "Missing required field: ticket_id".to_string(),
            })?.to_string();
            
        let department = context_data
            .as_ref()
            .and_then(|v| v["department"].as_str())
            .ok_or_else(|| WorkflowError::ValidationError {
                message: "Missing required field: department".to_string(),
            })?.to_string();
            
        let routing_reason = context_data
            .as_ref()
            .and_then(|v| v["routing_reason"].as_str())
            .ok_or_else(|| WorkflowError::ValidationError {
                message: "Missing required field: routing_reason".to_string(),
            })?.to_string();
        
        let priority = context_data
            .as_ref()
            .and_then(|v| v["priority"].as_str())
            .unwrap_or("normal")
            .to_string();
            
        let agent_id = context_data
            .as_ref()
            .and_then(|v| v["agent_id"].as_str())
            .map(|s| s.to_string());

        // Implement intelligent routing logic
        let routing_decision = self.determine_routing(
            &ticket_id,
            &department,
            &priority,
            &routing_reason,
            agent_id.as_deref(),
        )?;

        // Log the routing decision
        log::info!(
            "Routing ticket {} to department {} (reason: {})",
            ticket_id,
            routing_decision.target_department,
            routing_decision.routing_reason
        );

        // Update task context with routing results
        task_context.update_node(&self.node_name(), serde_json::json!({
            "routing_decision": routing_decision,
            "ticket_id": ticket_id,
            "routed_at": chrono::Utc::now(),
            "status": "routed"
        }));
        
        // Mark ticket as routed for backward compatibility
        task_context.update_node("ticket_routed", &true);
        
        Ok(task_context)
    }
}
