// Test to verify README examples compile and work

use backend::core::task::TaskContext;
use backend::core::{nodes::Node, error::WorkflowError};
use serde_json::json;

#[derive(Debug)]
struct TestNode;

impl Node for TestNode {
    fn process(&self, context: TaskContext) -> Result<TaskContext, WorkflowError> {
        Ok(context)
    }
}

#[test]
fn test_node_trait_example_compiles() {
    let node = TestNode;
    let context = TaskContext::new("test".to_string(), json!({}));
    let result = node.process(context);
    assert!(result.is_ok());
}

#[test]
fn test_task_context_creation() {
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
    
    assert_eq!(task_context.workflow_type, "knowledge_base");
}

#[test]
fn test_customer_support_context() {
    let task_context = TaskContext::new(
        "customer_support".to_string(),
        json!({
            "ticket_id": "TICKET-12345",
            "customer_email": "user@example.com",
            "subject": "Password Reset Request",
            "description": "I forgot my password and need help resetting it.",
            "priority": "normal",
            "category": "account_access"
        })
    );
    
    assert_eq!(task_context.workflow_type, "customer_support");
}

// Note: Workflow creation tests would require the actual workflow functions to exist,
// which may not be fully implemented yet. These tests verify the basic building blocks compile.