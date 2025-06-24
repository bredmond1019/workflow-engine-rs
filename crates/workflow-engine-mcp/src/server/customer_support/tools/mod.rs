use workflow_engine_core::error::WorkflowError;
use serde::{Deserialize, Serialize};

use super::server::CustomerSupportMcpServer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerCareEventData {
    pub ticket_id: String,
    pub customer_id: String,
    pub message: String,
    pub priority: String,
}

mod validate_ticket;
mod filter_spam;
mod determine_intent;
mod analyze_ticket;
mod generate_response;
mod escalate_ticket;
mod process_invoice;
mod close_ticket;
mod send_reply;
mod ticket_router;

pub use validate_ticket::ValidateTicketNode;
pub use filter_spam::FilterSpamNode;
pub use determine_intent::DetermineTicketIntentNode;
pub use analyze_ticket::AnalyzeTicketNode;
pub use generate_response::GenerateCustomerResponseNode;
pub use escalate_ticket::EscalateTicketNode;
pub use process_invoice::ProcessInvoiceNode;
pub use close_ticket::CloseTicketNode;
pub use send_reply::SendReplyNode;
pub use ticket_router::TicketRouterNode;

// Import register functions
use determine_intent::register_determine_intent_tool;
use analyze_ticket::register_analyze_ticket_tool;
use generate_response::register_generate_response_tool;


pub async fn register_customer_support_tools(server: &mut CustomerSupportMcpServer) -> Result<(), WorkflowError> {
  ValidateTicketNode::register(server).await?;
  FilterSpamNode::register(server).await?;
  register_determine_intent_tool(server)?;
  register_analyze_ticket_tool(server)?;
  register_generate_response_tool(server)?;
  EscalateTicketNode::register(server).await?;
  ProcessInvoiceNode::register(server).await?;
  CloseTicketNode::register(server).await?;
  SendReplyNode::register(server).await?;
  TicketRouterNode::register(server).await?;
  Ok(())
} 