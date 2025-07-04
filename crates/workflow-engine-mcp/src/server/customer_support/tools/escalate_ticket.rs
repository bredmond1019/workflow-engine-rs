use crate::server::ToolMetadata;
use workflow_engine_core::{error::WorkflowError, nodes::Node, task::TaskContext};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::super::server::CustomerSupportMcpServer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationResult {
    pub ticket_id: String,
    pub escalation_id: String,
    pub escalation_reason: String,
    pub target_team: String,
    pub assigned_supervisor: Option<String>,
    pub urgency_level: String,
    pub escalation_path: Vec<String>,
    pub notifications_sent: Vec<String>,
    pub estimated_response_time: String,
    pub follow_up_required: bool,
}

#[derive(Debug, Clone)]
pub struct EscalateTicketNode;

impl EscalateTicketNode {
    pub fn new() -> Self {
        Self
    }
    
    fn validate_escalation_request(
        &self,
        ticket_id: &str,
        escalation_reason: &str,
    ) -> Result<(), WorkflowError> {
        if escalation_reason.trim().is_empty() {
            return Err(WorkflowError::ValidationError {
                message: "Escalation reason cannot be empty".to_string(),
                field: "escalation_reason".to_string(),
                value: Some(escalation_reason.to_string()),
                constraint: "non-empty string".to_string(),
                context: "in escalate_ticket validation".to_string(),
            });
        }
        
        if escalation_reason.len() < 10 {
            return Err(WorkflowError::ValidationError {
                message: "Escalation reason must be at least 10 characters long".to_string(),
                field: "escalation_reason".to_string(),
                value: Some(escalation_reason.to_string()),
                constraint: "minimum 10 characters".to_string(),
                context: "in escalate_ticket validation".to_string(),
            });
        }
        
        // Check if ticket exists and is eligible for escalation
        self.check_escalation_eligibility(ticket_id)?;
        
        Ok(())
    }
    
    fn check_escalation_eligibility(&self, ticket_id: &str) -> Result<(), WorkflowError> {
        // In a real implementation, this would check the ticket database
        log::info!("Checking escalation eligibility for ticket {}", ticket_id);
        
        // Simulate eligibility check (always passes in this simulation)
        Ok(())
    }
    
    fn determine_escalation_path(
        &self,
        escalation_reason: &str,
        urgency: &str,
    ) -> Result<Vec<String>, WorkflowError> {
        let reason_lower = escalation_reason.to_lowercase();
        let mut path = Vec::new();
        
        // Determine initial escalation path based on reason and urgency
        if reason_lower.contains("legal") || reason_lower.contains("compliance") {
            path.extend_from_slice(&[
                "legal_team".to_string(),
                "compliance_officer".to_string(),
            ]);
        } else if reason_lower.contains("technical") || reason_lower.contains("bug") {
            path.extend_from_slice(&[
                "technical_lead".to_string(),
                "engineering_manager".to_string(),
            ]);
        } else if reason_lower.contains("billing") || reason_lower.contains("payment") {
            path.extend_from_slice(&[
                "billing_supervisor".to_string(),
                "finance_manager".to_string(),
            ]);
        } else if reason_lower.contains("refund") || reason_lower.contains("money") {
            path.extend_from_slice(&[
                "refund_specialist".to_string(),
                "finance_director".to_string(),
            ]);
        } else {
            path.extend_from_slice(&[
                "customer_success_manager".to_string(),
                "operations_supervisor".to_string(),
            ]);
        }
        
        // Add additional escalation levels for high urgency
        if urgency == "critical" || urgency == "high" {
            path.push("director_customer_experience".to_string());
            if urgency == "critical" {
                path.push("executive_team".to_string());
            }
        }
        
        Ok(path)
    }
    
    fn execute_escalation(
        &self,
        ticket_id: &str,
        escalation_reason: &str,
        target_team: Option<&str>,
        urgency: &str,
        escalation_path: Vec<String>,
    ) -> Result<EscalationResult, WorkflowError> {
        let escalation_id = self.generate_escalation_id(ticket_id);
        
        // Determine target team (use provided or infer from escalation path)
        let target_team = target_team
            .map(|s| s.to_string())
            .or_else(|| escalation_path.first().cloned())
            .unwrap_or_else(|| "general_escalation".to_string());
        
        // Assign supervisor based on target team and urgency
        let assigned_supervisor = self.assign_supervisor(&target_team, urgency)?;
        
        // Send notifications to escalation path
        let notifications_sent = self.send_escalation_notifications(
            &escalation_id,
            ticket_id,
            &escalation_path,
            urgency,
        )?;
        
        // Estimate response time based on urgency and team
        let estimated_response_time = self.estimate_escalation_response_time(&target_team, urgency);
        
        // Determine if follow-up is required
        let follow_up_required = self.requires_follow_up(urgency, escalation_reason);
        
        Ok(EscalationResult {
            ticket_id: ticket_id.to_string(),
            escalation_id,
            escalation_reason: escalation_reason.to_string(),
            target_team,
            assigned_supervisor,
            urgency_level: urgency.to_string(),
            escalation_path,
            notifications_sent,
            estimated_response_time,
            follow_up_required,
        })
    }
    
    fn generate_escalation_id(&self, ticket_id: &str) -> String {
        let timestamp = chrono::Utc::now().timestamp();
        format!("esc_{}_{}", ticket_id, timestamp)
    }
    
    fn assign_supervisor(&self, target_team: &str, urgency: &str) -> Result<Option<String>, WorkflowError> {
        let supervisor = match (target_team, urgency) {
            ("legal_team", _) => Some("legal_supervisor_001".to_string()),
            ("technical_lead", "critical") => Some("cto_001".to_string()),
            ("technical_lead", _) => Some("tech_supervisor_001".to_string()),
            ("billing_supervisor", "critical") => Some("finance_director_001".to_string()),
            ("billing_supervisor", _) => Some("billing_manager_001".to_string()),
            ("refund_specialist", _) => Some("refund_manager_001".to_string()),
            ("customer_success_manager", "critical") => Some("cs_director_001".to_string()),
            ("customer_success_manager", _) => Some("cs_manager_001".to_string()),
            _ => {
                if urgency == "critical" {
                    Some("escalation_director_001".to_string())
                } else {
                    Some("general_supervisor_001".to_string())
                }
            }
        };
        
        Ok(supervisor)
    }
    
    fn send_escalation_notifications(
        &self,
        escalation_id: &str,
        ticket_id: &str,
        escalation_path: &[String],
        urgency: &str,
    ) -> Result<Vec<String>, WorkflowError> {
        let mut notifications_sent = Vec::new();
        
        for team in escalation_path {
            let notification_id = self.send_notification(escalation_id, ticket_id, team, urgency)?;
            notifications_sent.push(notification_id);
        }
        
        // Send additional notifications for critical issues
        if urgency == "critical" {
            let executive_notification = self.send_executive_notification(escalation_id, ticket_id)?;
            notifications_sent.push(executive_notification);
        }
        
        Ok(notifications_sent)
    }
    
    fn send_notification(
        &self,
        escalation_id: &str,
        ticket_id: &str,
        team: &str,
        urgency: &str,
    ) -> Result<String, WorkflowError> {
        // In a real implementation, this would send actual notifications
        let notification_id = format!("notif_{}_{}_{}", escalation_id, team, chrono::Utc::now().timestamp());
        
        log::info!(
            "Sent escalation notification {} to {} for ticket {} (urgency: {})",
            notification_id,
            team,
            ticket_id,
            urgency
        );
        
        Ok(notification_id)
    }
    
    fn send_executive_notification(
        &self,
        escalation_id: &str,
        ticket_id: &str,
    ) -> Result<String, WorkflowError> {
        let notification_id = format!("exec_notif_{}_{}", escalation_id, chrono::Utc::now().timestamp());
        
        log::warn!(
            "Sent critical escalation notification {} to executive team for ticket {}",
            notification_id,
            ticket_id
        );
        
        Ok(notification_id)
    }
    
    fn estimate_escalation_response_time(&self, target_team: &str, urgency: &str) -> String {
        match (target_team, urgency) {
            (_, "critical") => "30 minutes".to_string(),
            ("legal_team", "high") => "2 hours".to_string(),
            ("legal_team", _) => "1 day".to_string(),
            ("technical_lead", "high") => "1 hour".to_string(),
            ("technical_lead", _) => "4 hours".to_string(),
            ("billing_supervisor", "high") => "1 hour".to_string(),
            ("billing_supervisor", _) => "2 hours".to_string(),
            ("refund_specialist", _) => "2 hours".to_string(),
            ("customer_success_manager", "high") => "1 hour".to_string(),
            ("customer_success_manager", _) => "2 hours".to_string(),
            _ => "4 hours".to_string(),
        }
    }
    
    fn requires_follow_up(&self, urgency: &str, escalation_reason: &str) -> bool {
        urgency == "critical" ||
        urgency == "high" ||
        escalation_reason.to_lowercase().contains("legal") ||
        escalation_reason.to_lowercase().contains("compliance") ||
        escalation_reason.to_lowercase().contains("refund")
    }

    pub async fn register(server: &mut CustomerSupportMcpServer) -> Result<(), WorkflowError> {
        let node = Arc::new(Self::new());
        let metadata = ToolMetadata::new(
            "escalate_ticket".to_string(),
            "Escalates ticket to appropriate team or supervisor when needed".to_string(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "context_data": {
                        "type": "object",
                        "properties": {
                            "ticket_id": {"type": "string", "description": "Ticket to escalate"},
                            "escalation_reason": {"type": "string", "description": "Reason for escalation"},
                            "target_team": {"type": "string", "description": "Team to escalate to"},
                            "urgency": {"type": "string", "enum": ["low", "medium", "high", "critical"]}
                        },
                        "required": ["ticket_id", "escalation_reason"]
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

impl Node for EscalateTicketNode {
    fn process(&self, mut task_context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Extract escalation parameters from context
        let context_data = task_context.get_data::<serde_json::Value>("context_data")?;
        
        let ticket_id = context_data
            .as_ref()
            .and_then(|v| v["ticket_id"].as_str())
            .ok_or_else(|| WorkflowError::ValidationError {
                message: "Missing required field: ticket_id".to_string(),
                field: "ticket_id".to_string(),
                value: None,
                constraint: "required field".to_string(),
                context: "in escalate_ticket node".to_string(),
            })?.to_string();
            
        let escalation_reason = context_data
            .as_ref()
            .and_then(|v| v["escalation_reason"].as_str())
            .ok_or_else(|| WorkflowError::ValidationError {
                message: "Missing required field: escalation_reason".to_string(),
                field: "escalation_reason".to_string(),
                value: None,
                constraint: "required field".to_string(),
                context: "in escalate_ticket node".to_string(),
            })?.to_string();
        
        let target_team = context_data
            .as_ref()
            .and_then(|v| v["target_team"].as_str())
            .map(|s| s.to_string());
            
        let urgency = context_data
            .as_ref()
            .and_then(|v| v["urgency"].as_str())
            .unwrap_or("medium")
            .to_string();

        // Validate escalation request
        self.validate_escalation_request(&ticket_id, &escalation_reason)?;
        
        // Determine appropriate escalation path
        let escalation_path = self.determine_escalation_path(&escalation_reason, &urgency)?;
        
        // Process the escalation
        let escalation_result = self.execute_escalation(
            &ticket_id,
            &escalation_reason,
            target_team.as_deref(),
            &urgency,
            escalation_path,
        )?;

        // Log the escalation
        log::warn!(
            "Escalated ticket {} to {} (Reason: {}, Urgency: {})",
            ticket_id,
            escalation_result.target_team,
            escalation_reason,
            urgency
        );

        // Update task context with escalation results
        task_context.update_node(&self.node_name(), serde_json::json!({
            "escalation_result": escalation_result,
            "escalated_at": chrono::Utc::now()
        }));
        
        // Update ticket status for backward compatibility
        task_context.update_node("ticket_status", &"escalated");
        
        Ok(task_context)
    }
}
