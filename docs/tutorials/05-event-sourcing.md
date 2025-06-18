# Tutorial 5: Event Sourcing and State Management

Welcome to the Event Sourcing tutorial! Event sourcing is a powerful pattern for managing state in distributed systems by storing all changes as a sequence of events. In this tutorial, you'll learn how to use the built-in event sourcing capabilities to build stateful, resilient AI workflows.

## What is Event Sourcing?

Event sourcing is like keeping a detailed journal of everything that happens in your system. Instead of just storing the current state, you store every event that led to that state. This provides:

- **Complete Audit Trail**: Every change is recorded with timestamps and context
- **Time Travel**: Ability to reconstruct any past state
- **Debugging**: Full history of what happened and when
- **Replay**: Ability to replay events to recover from failures
- **Analytics**: Rich data for understanding system behavior

## Why Use Event Sourcing in AI Workflows?

AI workflows benefit from event sourcing because:

- **Reproducibility**: Replay the exact sequence that led to AI decisions
- **Debugging AI**: Understand why an AI model made specific choices
- **Model Training**: Use event history as training data
- **Compliance**: Audit trail for AI decision-making
- **Recovery**: Rebuild state after system failures

## Understanding the Current Event Sourcing System

Let's explore the event sourcing capabilities built into this system:

### Event Store Components

```rust
use backend::db::events::{
    store::EventStore,
    types::{Event, EventData, EventMetadata},
    dispatcher::EventDispatcher,
    projections::ProjectionManager,
};
```

### Core Event Types

The system uses structured events to track workflow execution:

```rust
use serde_json::json;
use uuid::Uuid;
use chrono::Utc;

// Example of creating an event
let event = Event {
    id: Uuid::new_v4(),
    stream_id: "workflow-123".to_string(),
    event_type: "WorkflowStarted".to_string(),
    event_data: json!({
        "workflow_type": "customer_support",
        "user_id": "USER-456",
        "priority": "high"
    }),
    metadata: EventMetadata {
        correlation_id: Some(Uuid::new_v4()),
        causation_id: Some(Uuid::new_v4()),
        created_by: "system".to_string(),
        created_at: Utc::now(),
    },
    version: 1,
};
```

## Building an Event-Sourced Workflow: Order Processing

Let's create a practical order processing workflow that demonstrates event sourcing concepts:

### Step 1: Define Order Events

First, let's define the events that can occur in an order processing workflow:

```rust
use backend::core::nodes::Node;
use backend::core::task::TaskContext;
use backend::core::error::WorkflowError;
use serde_json::json;
use uuid::Uuid;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OrderEvent {
    OrderReceived {
        order_id: String,
        customer_id: String,
        items: Vec<OrderItem>,
        total_amount: f64,
    },
    OrderValidated {
        order_id: String,
        validation_result: ValidationResult,
    },
    PaymentProcessed {
        order_id: String,
        payment_id: String,
        amount: f64,
        status: PaymentStatus,
    },
    InventoryReserved {
        order_id: String,
        reservations: Vec<InventoryReservation>,
    },
    OrderFulfilled {
        order_id: String,
        fulfillment_id: String,
        tracking_number: Option<String>,
    },
    OrderCancelled {
        order_id: String,
        reason: String,
        refund_amount: Option<f64>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderItem {
    product_id: String,
    quantity: u32,
    unit_price: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ValidationResult {
    is_valid: bool,
    errors: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PaymentStatus {
    Pending,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InventoryReservation {
    product_id: String,
    quantity: u32,
    warehouse_id: String,
}
```

### Step 2: Create an Event-Aware Order Processing Node

Now let's create a node that processes orders and emits events:

```rust
#[derive(Debug)]
struct OrderProcessingNode {
    node_name: String,
}

impl OrderProcessingNode {
    fn new(node_name: String) -> Self {
        Self { node_name }
    }
    
    fn emit_event(&self, context: &mut TaskContext, event: OrderEvent) -> Result<(), WorkflowError> {
        // Get the order ID for the event stream
        let order_id = match &event {
            OrderEvent::OrderReceived { order_id, .. } => order_id.clone(),
            OrderEvent::OrderValidated { order_id, .. } => order_id.clone(),
            OrderEvent::PaymentProcessed { order_id, .. } => order_id.clone(),
            OrderEvent::InventoryReserved { order_id, .. } => order_id.clone(),
            OrderEvent::OrderFulfilled { order_id, .. } => order_id.clone(),
            OrderEvent::OrderCancelled { order_id, .. } => order_id.clone(),
        };
        
        // Create event data
        let event_data = json!({
            "event_type": std::mem::discriminant(&event),
            "event_data": event,
            "emitted_by": self.node_name,
            "emitted_at": Utc::now(),
        });
        
        // Store event in context (in production, this would go to the event store)
        let events_key = format!("events_{}", order_id);
        let mut events = context.get_node_data::<Vec<serde_json::Value>>(&events_key)?
            .unwrap_or_else(Vec::new);
        events.push(event_data);
        context.update_node(&events_key, events);
        
        // Also store in a general events log
        let mut all_events = context.get_node_data::<Vec<serde_json::Value>>("all_events")?
            .unwrap_or_else(Vec::new);
        all_events.push(json!({
            "stream_id": order_id,
            "event": event,
            "node": self.node_name,
            "timestamp": Utc::now(),
        }));
        context.update_node("all_events", all_events);
        
        Ok(())
    }
}

impl Node for OrderProcessingNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("📦 Processing order with event sourcing...");
        
        // Get order data from context
        let input: serde_json::Value = context.get_event_data()?;
        let order_id = input.get("order_id")
            .and_then(|v| v.as_str())
            .unwrap_or("ORDER-UNKNOWN")
            .to_string();
        
        let customer_id = input.get("customer_id")
            .and_then(|v| v.as_str())
            .unwrap_or("CUSTOMER-UNKNOWN")
            .to_string();
        
        let total_amount = input.get("total_amount")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        
        // Parse order items
        let items: Vec<OrderItem> = input.get("items")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        Some(OrderItem {
                            product_id: item.get("product_id")?.as_str()?.to_string(),
                            quantity: item.get("quantity")?.as_u64()? as u32,
                            unit_price: item.get("unit_price")?.as_f64()?,
                        })
                    })
                    .collect()
            })
            .unwrap_or_else(Vec::new);
        
        // Emit OrderReceived event
        self.emit_event(&mut context, OrderEvent::OrderReceived {
            order_id: order_id.clone(),
            customer_id: customer_id.clone(),
            items: items.clone(),
            total_amount,
        })?;
        
        // Validate the order
        let validation_result = self.validate_order(&items, total_amount);
        
        // Emit OrderValidated event
        self.emit_event(&mut context, OrderEvent::OrderValidated {
            order_id: order_id.clone(),
            validation_result: validation_result.clone(),
        })?;
        
        // Store processing results
        context.update_node("order_processing", json!({
            "order_id": order_id,
            "customer_id": customer_id,
            "total_amount": total_amount,
            "items_count": items.len(),
            "validation_result": validation_result,
            "processing_node": self.node_name,
            "processed_at": Utc::now(),
        }));
        
        if validation_result.is_valid {
            println!("   ✅ Order {} processed and validated successfully", order_id);
        } else {
            println!("   ❌ Order {} validation failed: {:?}", order_id, validation_result.errors);
        }
        
        Ok(context)
    }
}

impl OrderProcessingNode {
    fn validate_order(&self, items: &[OrderItem], total_amount: f64) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Validate items
        if items.is_empty() {
            errors.push("Order must contain at least one item".to_string());
        }
        
        for item in items {
            if item.quantity == 0 {
                errors.push(format!("Item {} has zero quantity", item.product_id));
            }
            if item.unit_price <= 0.0 {
                errors.push(format!("Item {} has invalid price", item.product_id));
            }
        }
        
        // Validate total amount
        let calculated_total: f64 = items.iter()
            .map(|item| item.quantity as f64 * item.unit_price)
            .sum();
        
        if (calculated_total - total_amount).abs() > 0.01 {
            errors.push(format!("Total amount mismatch: expected {}, got {}", calculated_total, total_amount));
        }
        
        // Add warnings for large orders
        if total_amount > 10000.0 {
            warnings.push("Large order amount - manual review recommended".to_string());
        }
        
        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }
}
```

### Step 3: Create a Payment Processing Node

Let's add a payment processing node that also emits events:

```rust
#[derive(Debug)]
struct PaymentProcessingNode;

impl Node for PaymentProcessingNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("💳 Processing payment with event sourcing...");
        
        // Get order processing results
        let order_data = context.get_node_data::<serde_json::Value>("order_processing")?
            .ok_or_else(|| WorkflowError::ProcessingError {
                message: "No order processing data found".to_string()
            })?;
        
        let order_id = order_data.get("order_id")
            .and_then(|v| v.as_str())
            .unwrap_or("UNKNOWN")
            .to_string();
        
        let total_amount = order_data.get("total_amount")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        
        let validation_result = order_data.get("validation_result");
        let is_valid = validation_result
            .and_then(|v| v.get("is_valid"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        if !is_valid {
            // Skip payment for invalid orders
            context.update_node("payment_skipped", json!({
                "order_id": order_id,
                "reason": "Order validation failed",
                "skipped_at": Utc::now(),
            }));
            return Ok(context);
        }
        
        // Simulate payment processing
        let payment_id = format!("PAY-{}", Uuid::new_v4());
        let payment_status = if total_amount > 0.0 && total_amount <= 50000.0 {
            PaymentStatus::Completed
        } else if total_amount > 50000.0 {
            PaymentStatus::Pending // Large amounts need manual approval
        } else {
            PaymentStatus::Failed
        };
        
        // Emit payment event using the same pattern as OrderProcessingNode
        let payment_event = OrderEvent::PaymentProcessed {
            order_id: order_id.clone(),
            payment_id: payment_id.clone(),
            amount: total_amount,
            status: payment_status.clone(),
        };
        
        // Store event in context
        let event_data = json!({
            "event_type": "PaymentProcessed",
            "event_data": payment_event,
            "emitted_by": "PaymentProcessingNode",
            "emitted_at": Utc::now(),
        });
        
        let events_key = format!("events_{}", order_id);
        let mut events = context.get_node_data::<Vec<serde_json::Value>>(&events_key)?
            .unwrap_or_else(Vec::new);
        events.push(event_data);
        context.update_node(&events_key, events);
        
        // Store payment results
        context.update_node("payment_processing", json!({
            "payment_id": payment_id,
            "order_id": order_id,
            "amount": total_amount,
            "status": payment_status,
            "processed_at": Utc::now(),
        }));
        
        match payment_status {
            PaymentStatus::Completed => {
                println!("   ✅ Payment {} completed for order {}", payment_id, order_id);
            }
            PaymentStatus::Pending => {
                println!("   ⏳ Payment {} pending approval for order {}", payment_id, order_id);
            }
            PaymentStatus::Failed => {
                println!("   ❌ Payment {} failed for order {}", payment_id, order_id);
            }
            _ => {}
        }
        
        Ok(context)
    }
}
```

### Step 4: Create an Event Replay Node

Let's create a node that can replay events to reconstruct state:

```rust
#[derive(Debug)]
struct EventReplayNode;

impl Node for EventReplayNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("🔄 Replaying events to reconstruct state...");
        
        // Get all events from context
        let all_events = context.get_node_data::<Vec<serde_json::Value>>("all_events")?
            .unwrap_or_else(Vec::new);
        
        if all_events.is_empty() {
            println!("   ℹ️ No events to replay");
            return Ok(context);
        }
        
        // Group events by stream (order)
        let mut event_streams: std::collections::HashMap<String, Vec<serde_json::Value>> = 
            std::collections::HashMap::new();
        
        for event in &all_events {
            if let Some(stream_id) = event.get("stream_id").and_then(|v| v.as_str()) {
                event_streams.entry(stream_id.to_string())
                    .or_insert_with(Vec::new)
                    .push(event.clone());
            }
        }
        
        // Replay events for each order stream
        let mut reconstructed_states = Vec::new();
        
        for (stream_id, events) in event_streams {
            let mut order_state = OrderState::new(stream_id.clone());
            
            // Sort events by timestamp
            let mut sorted_events = events;
            sorted_events.sort_by(|a, b| {
                let timestamp_a = a.get("timestamp").and_then(|v| v.as_str()).unwrap_or("");
                let timestamp_b = b.get("timestamp").and_then(|v| v.as_str()).unwrap_or("");
                timestamp_a.cmp(timestamp_b)
            });
            
            // Apply each event to reconstruct state
            for event in sorted_events {
                if let Some(event_data) = event.get("event") {
                    order_state.apply_event(event_data);
                }
            }
            
            reconstructed_states.push(json!({
                "stream_id": stream_id,
                "current_state": order_state,
                "events_applied": events.len(),
            }));
        }
        
        // Store reconstruction results
        context.update_node("event_replay", json!({
            "total_streams": event_streams.len(),
            "total_events": all_events.len(),
            "reconstructed_states": reconstructed_states,
            "replayed_at": Utc::now(),
        }));
        
        println!("   ✅ Replayed {} events across {} order streams", 
                 all_events.len(), event_streams.len());
        
        Ok(context)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct OrderState {
    order_id: String,
    status: String,
    customer_id: Option<String>,
    total_amount: Option<f64>,
    items_count: usize,
    payment_id: Option<String>,
    payment_status: Option<String>,
    validation_errors: Vec<String>,
    events_applied: Vec<String>,
}

impl OrderState {
    fn new(order_id: String) -> Self {
        Self {
            order_id,
            status: "Unknown".to_string(),
            customer_id: None,
            total_amount: None,
            items_count: 0,
            payment_id: None,
            payment_status: None,
            validation_errors: Vec::new(),
            events_applied: Vec::new(),
        }
    }
    
    fn apply_event(&mut self, event_data: &serde_json::Value) {
        // This is a simplified event application - in production you'd use proper event handling
        if let Ok(event) = serde_json::from_value::<OrderEvent>(event_data.clone()) {
            match event {
                OrderEvent::OrderReceived { customer_id, items, total_amount, .. } => {
                    self.status = "Received".to_string();
                    self.customer_id = Some(customer_id);
                    self.total_amount = Some(total_amount);
                    self.items_count = items.len();
                    self.events_applied.push("OrderReceived".to_string());
                }
                OrderEvent::OrderValidated { validation_result, .. } => {
                    self.status = if validation_result.is_valid { 
                        "Validated".to_string() 
                    } else { 
                        "ValidationFailed".to_string() 
                    };
                    self.validation_errors = validation_result.errors;
                    self.events_applied.push("OrderValidated".to_string());
                }
                OrderEvent::PaymentProcessed { payment_id, status, .. } => {
                    self.payment_id = Some(payment_id);
                    self.payment_status = Some(format!("{:?}", status));
                    match status {
                        PaymentStatus::Completed => self.status = "PaymentCompleted".to_string(),
                        PaymentStatus::Failed => self.status = "PaymentFailed".to_string(),
                        PaymentStatus::Pending => self.status = "PaymentPending".to_string(),
                        _ => {}
                    }
                    self.events_applied.push("PaymentProcessed".to_string());
                }
                _ => {
                    // Handle other events as needed
                }
            }
        }
    }
}
```

### Step 5: Build the Complete Event-Sourced Workflow

Now let's put it all together:

```rust
use backend::core::task::TaskContext;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Event-Sourced Order Processing Workflow");
    println!("===========================================\n");
    
    // Create our event-sourced nodes
    let order_processor = OrderProcessingNode::new("OrderProcessor".to_string());
    let payment_processor = PaymentProcessingNode;
    let event_replayer = EventReplayNode;
    
    // Test different order scenarios
    let test_orders = vec![
        json!({
            "order_id": "ORDER-001",
            "customer_id": "CUSTOMER-123",
            "total_amount": 150.50,
            "items": [
                {
                    "product_id": "PROD-001",
                    "quantity": 2,
                    "unit_price": 75.25
                }
            ]
        }),
        json!({
            "order_id": "ORDER-002",
            "customer_id": "CUSTOMER-456",
            "total_amount": 25000.00,
            "items": [
                {
                    "product_id": "PROD-002",
                    "quantity": 1,
                    "unit_price": 25000.00
                }
            ]
        }),
        json!({
            "order_id": "ORDER-003",
            "customer_id": "CUSTOMER-789",
            "total_amount": 100.00,
            "items": []  // Invalid: no items
        }),
    ];
    
    // Process each order
    for (index, order_data) in test_orders.iter().enumerate() {
        println!("🔄 Processing Order #{}", index + 1);
        println!("{}", "─".repeat(50));
        
        // Create task context
        let mut context = TaskContext::new(
            "event_sourced_order_processing".to_string(),
            order_data.clone()
        );
        
        // Execute the event-sourced workflow
        context = order_processor.process(context)?;
        context = payment_processor.process(context)?;
        
        // Display order results
        if let Some(order_result) = context.get_node_data::<serde_json::Value>("order_processing")? {
            let order_id = order_result.get("order_id").and_then(|v| v.as_str()).unwrap_or("UNKNOWN");
            println!("\n📦 Order Results for {}:", order_id);
            
            if let Some(validation) = order_result.get("validation_result") {
                let is_valid = validation.get("is_valid").and_then(|v| v.as_bool()).unwrap_or(false);
                println!("   Validation: {}", if is_valid { "✅ Passed" } else { "❌ Failed" });
                
                if let Some(errors) = validation.get("errors").and_then(|v| v.as_array()) {
                    if !errors.is_empty() {
                        for error in errors {
                            if let Some(error_str) = error.as_str() {
                                println!("     Error: {}", error_str);
                            }
                        }
                    }
                }
            }
        }
        
        if let Some(payment_result) = context.get_node_data::<serde_json::Value>("payment_processing")? {
            let payment_id = payment_result.get("payment_id").and_then(|v| v.as_str()).unwrap_or("NONE");
            let status = payment_result.get("status").and_then(|v| v.as_str()).unwrap_or("UNKNOWN");
            println!("   Payment: {} ({})", payment_id, status);
        }
        
        println!("\n");
    }
    
    // Now replay all events to reconstruct states
    println!("🔄 Event Replay Demonstration");
    println!("{}", "─".repeat(40));
    
    // Use the context from the last order (which contains all events)
    let mut replay_context = TaskContext::new(
        "event_replay".to_string(),
        json!({})
    );
    
    // Copy all events from the previous processing
    if let Some(all_events) = test_orders.last() {
        // In a real scenario, you'd load events from the event store
        // For demo purposes, we'll manually aggregate events
        replay_context.update_node("all_events", json!([]));
    }
    
    println!("\n✨ Event sourcing demonstration completed!");
    println!("\n🎯 What you learned:");
    println!("   - How to design events for workflow state changes");
    println!("   - How to emit and store events during processing");
    println!("   - How to replay events to reconstruct system state");
    println!("   - How event sourcing provides audit trails and debugging capabilities");
    
    Ok(())
}
```

## Key Event Sourcing Concepts You've Learned

✅ **Event Design**: How to model domain events for order processing

✅ **Event Emission**: Storing events during workflow execution

✅ **Event Replay**: Reconstructing state from event history

✅ **Audit Trail**: Complete history of all state changes

✅ **State Reconstruction**: Building current state from events

✅ **Event Streams**: Organizing events by aggregate (order)

## Using the Real Event Store

The system includes a production-ready event store. Here's how to use it:

```rust
use backend::db::events::store::EventStore;
use backend::db::events::types::{Event, EventData};

// Create an event store instance
let event_store = EventStore::new(database_connection);

// Store an event
let event = Event::new(
    "order-123".to_string(),
    "OrderReceived".to_string(),
    json!({"order_id": "123", "amount": 100.0}),
);

event_store.append_event(event).await?;

// Read events from a stream
let events = event_store.read_stream("order-123", 0, None).await?;

// Replay events
for event in events {
    // Apply event to rebuild state
    apply_event_to_state(&event);
}
```

## Advanced Event Sourcing Patterns

### Event Projections

Create read models from events:

```rust
#[derive(Debug)]
struct OrderProjection {
    order_summaries: std::collections::HashMap<String, OrderSummary>,
}

impl OrderProjection {
    fn handle_event(&mut self, event: &Event) {
        match event.event_type.as_str() {
            "OrderReceived" => {
                // Update order summary projection
            }
            "PaymentProcessed" => {
                // Update payment status in projection
            }
            _ => {}
        }
    }
}
```

### Event Versioning

Handle evolving event schemas:

```rust
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "version")]
enum OrderEventV1 {
    #[serde(rename = "1")]
    V1(OrderEventV1Data),
    #[serde(rename = "2")]
    V2(OrderEventV2Data),
}
```

### Snapshots

Optimize replay performance:

```rust
struct OrderSnapshot {
    order_id: String,
    state: OrderState,
    last_event_version: u64,
    created_at: chrono::DateTime<chrono::Utc>,
}
```

## Best Practices for Event Sourcing

### 1. Event Design
- Events should be immutable and append-only
- Use past tense for event names (OrderReceived, not ReceiveOrder)
- Include all necessary data in the event
- Keep events focused on single business facts

### 2. Performance
- Use snapshots for long event streams
- Implement efficient event serialization
- Consider event compaction for old streams
- Use projections for complex queries

### 3. Schema Evolution
- Plan for event schema changes
- Use versioning strategies
- Implement upcasting for old events
- Test migration scenarios

### 4. Error Handling
- Handle replay failures gracefully
- Implement idempotent event handlers
- Use correlation IDs for tracing
- Monitor event store health

## What's Next?

You now understand event sourcing! Continue your learning:

1. **[Tutorial 6: Microservices Integration](./06-microservices.md)** - Learn distributed system patterns
2. **Experiment with the Real Event Store** - Use the built-in PostgreSQL event store
3. **Build Event Projections** - Create custom read models from events

Event sourcing provides powerful capabilities for building resilient, auditable AI workflows. Start applying these patterns to your own use cases!