use crate::server::ToolMetadata;
use workflow_engine_core::{error::WorkflowError, nodes::Node, task::TaskContext};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::super::server::CustomerSupportMCPServer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceProcessingResult {
    pub invoice_id: String,
    pub customer_id: String,
    pub request_type: String,
    pub status: String,
    pub action_taken: String,
    pub amount_processed: Option<f64>,
    pub resolution: String,
    pub requires_follow_up: bool,
    pub processing_notes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ProcessInvoiceNode;

impl ProcessInvoiceNode {
    pub fn new() -> Self {
        Self
    }
    
    fn validate_request_type(&self, request_type: &str) -> Result<(), WorkflowError> {
        match request_type {
            "payment" | "dispute" | "inquiry" | "correction" => Ok(()),
            _ => Err(WorkflowError::ValidationError {
                message: format!("Invalid request type: {}. Must be one of: payment, dispute, inquiry, correction", request_type),
            }),
        }
    }
    
    fn process_invoice_request(
        &self,
        invoice_id: &str,
        customer_id: &str,
        request_type: &str,
        amount: Option<f64>,
    ) -> Result<InvoiceProcessingResult, WorkflowError> {
        match request_type {
            "payment" => self.process_payment_request(invoice_id, customer_id, amount),
            "dispute" => self.process_dispute_request(invoice_id, customer_id, amount),
            "inquiry" => self.process_inquiry_request(invoice_id, customer_id),
            "correction" => self.process_correction_request(invoice_id, customer_id, amount),
            _ => Err(WorkflowError::ProcessingError {
                message: format!("Unhandled request type: {}", request_type),
            }),
        }
    }
    
    fn process_payment_request(
        &self,
        invoice_id: &str,
        customer_id: &str,
        amount: Option<f64>,
    ) -> Result<InvoiceProcessingResult, WorkflowError> {
        let mut processing_notes = Vec::new();
        
        // Validate invoice exists and is payable
        let invoice_status = self.validate_invoice_for_payment(invoice_id)?;
        processing_notes.push(format!("Invoice validation: {}", invoice_status));
        
        // Process payment
        let payment_result = if let Some(amount) = amount {
            self.process_payment_amount(invoice_id, amount)?
        } else {
            self.process_full_payment(invoice_id)?
        };
        
        processing_notes.push(format!("Payment processing: {}", payment_result.status));
        
        Ok(InvoiceProcessingResult {
            invoice_id: invoice_id.to_string(),
            customer_id: customer_id.to_string(),
            request_type: "payment".to_string(),
            status: payment_result.status,
            action_taken: payment_result.action,
            amount_processed: payment_result.amount,
            resolution: "Payment processed successfully".to_string(),
            requires_follow_up: false,
            processing_notes,
        })
    }
    
    fn process_dispute_request(
        &self,
        invoice_id: &str,
        customer_id: &str,
        amount: Option<f64>,
    ) -> Result<InvoiceProcessingResult, WorkflowError> {
        let mut processing_notes = Vec::new();
        
        // Validate dispute eligibility
        let dispute_eligibility = self.check_dispute_eligibility(invoice_id)?;
        processing_notes.push(format!("Dispute eligibility: {}", dispute_eligibility));
        
        // Create dispute case
        let dispute_case_id = self.create_dispute_case(invoice_id, customer_id, amount)?;
        processing_notes.push(format!("Dispute case created: {}", dispute_case_id));
        
        Ok(InvoiceProcessingResult {
            invoice_id: invoice_id.to_string(),
            customer_id: customer_id.to_string(),
            request_type: "dispute".to_string(),
            status: "under_review".to_string(),
            action_taken: format!("Dispute case {} created for review", dispute_case_id),
            amount_processed: amount,
            resolution: "Dispute initiated and assigned for review".to_string(),
            requires_follow_up: true,
            processing_notes,
        })
    }
    
    fn process_inquiry_request(
        &self,
        invoice_id: &str,
        customer_id: &str,
    ) -> Result<InvoiceProcessingResult, WorkflowError> {
        let mut processing_notes = Vec::new();
        
        // Retrieve invoice details
        let invoice_details = self.retrieve_invoice_details(invoice_id)?;
        processing_notes.push("Invoice details retrieved".to_string());
        
        // Generate invoice summary
        let summary = self.generate_invoice_summary(&invoice_details)?;
        processing_notes.push("Invoice summary generated".to_string());
        
        Ok(InvoiceProcessingResult {
            invoice_id: invoice_id.to_string(),
            customer_id: customer_id.to_string(),
            request_type: "inquiry".to_string(),
            status: "completed".to_string(),
            action_taken: "Invoice information provided".to_string(),
            amount_processed: None,
            resolution: format!("Invoice inquiry resolved: {}", summary),
            requires_follow_up: false,
            processing_notes,
        })
    }
    
    fn process_correction_request(
        &self,
        invoice_id: &str,
        customer_id: &str,
        amount: Option<f64>,
    ) -> Result<InvoiceProcessingResult, WorkflowError> {
        let mut processing_notes = Vec::new();
        
        // Validate correction eligibility
        let correction_eligibility = self.check_correction_eligibility(invoice_id)?;
        processing_notes.push(format!("Correction eligibility: {}", correction_eligibility));
        
        // Create correction request
        let correction_id = self.create_correction_request(invoice_id, customer_id, amount)?;
        processing_notes.push(format!("Correction request created: {}", correction_id));
        
        Ok(InvoiceProcessingResult {
            invoice_id: invoice_id.to_string(),
            customer_id: customer_id.to_string(),
            request_type: "correction".to_string(),
            status: "pending_approval".to_string(),
            action_taken: format!("Correction request {} submitted for approval", correction_id),
            amount_processed: amount,
            resolution: "Invoice correction request initiated".to_string(),
            requires_follow_up: true,
            processing_notes,
        })
    }
    
    // Helper methods for invoice operations (simulated)
    
    fn validate_invoice_for_payment(&self, invoice_id: &str) -> Result<String, WorkflowError> {
        // In a real implementation, this would check the invoice database
        log::info!("Validating invoice {} for payment", invoice_id);
        Ok("valid_for_payment".to_string())
    }
    
    fn process_payment_amount(&self, invoice_id: &str, amount: f64) -> Result<PaymentResult, WorkflowError> {
        log::info!("Processing partial payment of ${} for invoice {}", amount, invoice_id);
        Ok(PaymentResult {
            status: "partial_payment_processed".to_string(),
            action: format!("Processed payment of ${}", amount),
            amount: Some(amount),
        })
    }
    
    fn process_full_payment(&self, invoice_id: &str) -> Result<PaymentResult, WorkflowError> {
        log::info!("Processing full payment for invoice {}", invoice_id);
        Ok(PaymentResult {
            status: "payment_processed".to_string(),
            action: "Processed full payment".to_string(),
            amount: None, // Full amount would be retrieved from invoice
        })
    }
    
    fn check_dispute_eligibility(&self, invoice_id: &str) -> Result<String, WorkflowError> {
        log::info!("Checking dispute eligibility for invoice {}", invoice_id);
        Ok("eligible".to_string())
    }
    
    fn create_dispute_case(&self, invoice_id: &str, customer_id: &str, amount: Option<f64>) -> Result<String, WorkflowError> {
        let dispute_id = format!("dispute_{}_{}", invoice_id, chrono::Utc::now().timestamp());
        log::info!("Created dispute case {} for invoice {}", dispute_id, invoice_id);
        Ok(dispute_id)
    }
    
    fn retrieve_invoice_details(&self, invoice_id: &str) -> Result<InvoiceDetails, WorkflowError> {
        log::info!("Retrieving details for invoice {}", invoice_id);
        Ok(InvoiceDetails {
            id: invoice_id.to_string(),
            amount: 100.0,
            status: "issued".to_string(),
            due_date: "2024-01-31".to_string(),
        })
    }
    
    fn generate_invoice_summary(&self, details: &InvoiceDetails) -> Result<String, WorkflowError> {
        Ok(format!(
            "Invoice {}: Amount ${}, Status: {}, Due: {}",
            details.id, details.amount, details.status, details.due_date
        ))
    }
    
    fn check_correction_eligibility(&self, invoice_id: &str) -> Result<String, WorkflowError> {
        log::info!("Checking correction eligibility for invoice {}", invoice_id);
        Ok("eligible".to_string())
    }
    
    fn create_correction_request(&self, invoice_id: &str, customer_id: &str, amount: Option<f64>) -> Result<String, WorkflowError> {
        let correction_id = format!("correction_{}_{}", invoice_id, chrono::Utc::now().timestamp());
        log::info!("Created correction request {} for invoice {}", correction_id, invoice_id);
        Ok(correction_id)
    }
}

#[derive(Debug, Clone)]
struct PaymentResult {
    status: String,
    action: String,
    amount: Option<f64>,
}

#[derive(Debug, Clone)]
struct InvoiceDetails {
    id: String,
    amount: f64,
    status: String,
    due_date: String,
}

impl ProcessInvoiceNode {
    pub async fn register(server: &mut CustomerSupportMCPServer) -> Result<(), WorkflowError> {
        let node = Arc::new(Self::new());
        let metadata = ToolMetadata::new(
            "process_invoice".to_string(),
            "Processes invoice-related customer requests and billing inquiries".to_string(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "context_data": {
                        "type": "object",
                        "properties": {
                            "invoice_id": {"type": "string", "description": "Invoice identifier"},
                            "customer_id": {"type": "string", "description": "Customer identifier"},
                            "request_type": {"type": "string", "enum": ["payment", "dispute", "inquiry", "correction"]},
                            "amount": {"type": "number", "description": "Invoice amount (optional)"}
                        },
                        "required": ["invoice_id", "customer_id", "request_type"]
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

impl Node for ProcessInvoiceNode {
    fn process(&self, mut task_context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Extract invoice processing parameters from context
        let context_data = task_context.get_data::<serde_json::Value>("context_data")?;
        
        let invoice_id = context_data
            .as_ref()
            .and_then(|v| v["invoice_id"].as_str())
            .ok_or_else(|| WorkflowError::ValidationError {
                message: "Missing required field: invoice_id".to_string(),
            })?.to_string();
            
        let customer_id = context_data
            .as_ref()
            .and_then(|v| v["customer_id"].as_str())
            .ok_or_else(|| WorkflowError::ValidationError {
                message: "Missing required field: customer_id".to_string(),
            })?.to_string();
            
        let request_type = context_data
            .as_ref()
            .and_then(|v| v["request_type"].as_str())
            .ok_or_else(|| WorkflowError::ValidationError {
                message: "Missing required field: request_type".to_string(),
            })?.to_string();
        
        let amount = context_data
            .as_ref()
            .and_then(|v| v["amount"].as_f64());

        // Validate the request type
        self.validate_request_type(&request_type)?;
        
        // Process the invoice based on request type
        let processing_result = self.process_invoice_request(
            &invoice_id,
            &customer_id,
            &request_type,
            amount,
        )?;

        // Log the invoice processing
        log::info!(
            "Processed invoice {} for customer {} (Type: {}, Status: {})",
            invoice_id,
            customer_id,
            request_type,
            processing_result.status
        );

        // Update task context with processing results
        task_context.update_node(&self.node_name(), serde_json::json!({
            "processing_result": processing_result,
            "processed_at": chrono::Utc::now()
        }));
        
        // Update invoice status for backward compatibility
        task_context.update_node("invoice_status", &processing_result.status);
        
        Ok(task_context)
    }
}
