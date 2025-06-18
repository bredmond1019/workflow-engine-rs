mod tests {
    use serde_json::Value;

    use backend::core::task::TaskContext;
    use backend::db::event::NewEvent;
    use backend::workflows::customer_support_workflow::create_customer_care_workflow;

    #[test]
    fn test_workflow_creation() {
        let workflow = create_customer_care_workflow();
        assert!(workflow.is_ok());
    }

    #[test]
    fn test_task_context_from_event() {
        let event_data = serde_json::json!({
            "ticket_id": "TICKET-123",
            "customer_id": "CUSTOMER-456",
            "message": "I have a billing question",
            "priority": "medium"
        });

        let event = NewEvent::new(event_data, "customer_care".to_string(), Value::Null);
        let task_context = TaskContext::from_event(&event);

        assert_eq!(task_context.event_id, event.id);
        assert_eq!(task_context.workflow_type, "customer_care");
    }

    #[test]
    fn test_workflow_execution() {
        let workflow = create_customer_care_workflow().unwrap();
        let event_data = serde_json::json!({
            "ticket_id": "TICKET-123",
            "customer_id": "CUSTOMER-456",
            "message": "I have a billing question",
            "priority": "medium"
        });

        let result = workflow.run(event_data);
        assert!(result.is_ok());

        let context = result.unwrap();
        assert!(!context.nodes.is_empty());
        assert_eq!(context.workflow_type, "customer_care");
    }
}
