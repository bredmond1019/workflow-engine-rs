# Basic Workflow Example

This example demonstrates the fundamental concepts of creating and executing workflows in the AI Workflow System. It shows how to build a simple text processing workflow with proper error handling using the new boxed error system.

## What You'll Learn

- ✅ How to create basic workflows
- ✅ Node registration and execution patterns
- ✅ New boxed error handling implementation
- ✅ Workflow validation and debugging
- ✅ Event sourcing basics
- ✅ Type-safe workflow construction

## Architecture Overview

```
Input Text → [Text Input Node] → [Text Processor Node] → [Text Output Node] → Result
```

This example creates a simple 3-node workflow:
1. **Text Input Node**: Receives and validates input text
2. **Text Processor Node**: Transforms text (uppercase, lowercase, reverse, etc.)
3. **Text Output Node**: Formats and returns the final result

## Files Overview

```
01_basic_workflow/
├── README.md              # This file
├── Cargo.toml            # Dependencies and example config
├── src/
│   ├── main.rs           # Main workflow example
│   ├── lib.rs            # Shared utilities and types
│   └── nodes/            # Custom node implementations
│       ├── mod.rs
│       ├── text_input.rs
│       ├── text_processor.rs
│       └── text_output.rs
├── examples/
│   ├── basic.rs          # Simple workflow execution
│   ├── advanced.rs       # Advanced features demo
│   └── error_handling.rs # Error handling patterns
└── tests/
    └── integration_test.rs
```

## Prerequisites

1. **System Running**
   ```bash
   # Start the main API server
   cd ../../..
   cargo run --bin workflow-engine
   ```

2. **Environment Setup**
   ```bash
   export JWT_SECRET="your-secure-jwt-secret"
   export DATABASE_URL="postgresql://user:pass@localhost/ai_workflow_db"
   ```

## Running the Examples

### Basic Usage

```bash
# Navigate to this directory
cd examples/01_basic_workflow

# Run the main example
cargo run

# Run specific examples
cargo run --example basic
cargo run --example advanced
cargo run --example error_handling
```

### Expected Output

```
$ cargo run

=== Basic Workflow Example ===
🏗️  Building workflow: simple_text_processor
✅ Workflow created successfully

📋 Registering nodes...
✅ Node registered: TextInputNode (id: text_input)
✅ Node registered: TextProcessorNode (id: text_processor) 
✅ Node registered: TextOutputNode (id: text_output)

🔗 Connecting workflow graph...
✅ Connected: text_input → text_processor
✅ Connected: text_processor → text_output

🔍 Validating workflow...
✅ No cycles detected
✅ All nodes reachable
✅ Workflow validation passed

🚀 Executing workflow...
Input: "Hello, Workflow System!"

Step 1/3: text_input
├─ Processing input text: "Hello, Workflow System!"
├─ Validation: ✅ length=24, non-empty=true
└─ Output: validated text data

Step 2/3: text_processor  
├─ Processing mode: uppercase
├─ Input: "Hello, Workflow System!"
├─ Transformation applied
└─ Output: "HELLO, WORKFLOW SYSTEM!"

Step 3/3: text_output
├─ Formatting result...
├─ Adding metadata: timestamp, workflow_id
└─ Final result: "HELLO, WORKFLOW SYSTEM!"

✅ Workflow completed successfully in 45ms

=== Workflow Result ===
{
  "result": "HELLO, WORKFLOW SYSTEM!",
  "metadata": {
    "workflow_id": "simple_text_processor",
    "execution_time_ms": 45,
    "timestamp": "2024-12-18T10:30:00Z",
    "nodes_processed": 3,
    "status": "completed"
  }
}
```

## Key Concepts Demonstrated

### 1. Workflow Creation

```rust
use workflow_engine_core::workflow::Workflow;
use workflow_engine_core::error::WorkflowError;

// Create a new workflow with validation
let mut workflow = Workflow::new("simple_text_processor")
    .map_err(|e| WorkflowError::validation_error(
        "Failed to create workflow",
        "workflow_name",
        "valid identifier required",
        "in workflow creation"
    ))?;
```

### 2. Node Registration

```rust
use crate::nodes::{TextInputNode, TextProcessorNode, TextOutputNode};

// Register nodes with the workflow
workflow.register_node("text_input", Box::new(TextInputNode::new()))?;
workflow.register_node("text_processor", Box::new(TextProcessorNode::new()))?; 
workflow.register_node("text_output", Box::new(TextOutputNode::new()))?;
```

### 3. Error Handling with Boxed Errors

```rust
// Use specific error constructors for different failure modes
match workflow.execute(input) {
    Ok(result) => {
        println!("✅ Workflow completed: {:?}", result);
    }
    Err(WorkflowError::ValidationError(details)) => {
        eprintln!("❌ Validation error in field '{}': {}", 
                 details.field, details.message);
    }
    Err(WorkflowError::ProcessingError(details)) => {
        eprintln!("❌ Processing error in node '{}': {}", 
                 details.node_type, details.message);
    }
    Err(WorkflowError::CycleDetected) => {
        eprintln!("❌ Workflow contains cycles - check node connections");
    }
    Err(e) => {
        eprintln!("❌ Unexpected error: {}", e);
    }
}
```

### 4. Custom Node Implementation

```rust
use workflow_engine_core::nodes::Node;
use workflow_engine_core::task::TaskContext;
use workflow_engine_core::error::WorkflowError;

#[derive(Debug)]
pub struct TextProcessorNode {
    mode: String,
}

impl Node for TextProcessorNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Extract input with validation
        let input_text: String = context.get_input("text")
            .ok_or_else(|| WorkflowError::validation_error(
                "Missing required input 'text'",
                "text",
                "non-empty string required",
                "in text processing node"
            ))?;

        // Process the text based on mode
        let processed = match self.mode.as_str() {
            "uppercase" => input_text.to_uppercase(),
            "lowercase" => input_text.to_lowercase(), 
            "reverse" => input_text.chars().rev().collect(),
            _ => return Err(WorkflowError::processing_error(
                format!("Unknown processing mode: {}", self.mode),
                "TextProcessorNode"
            )),
        };

        // Update context with result
        context.set_output("processed_text", processed)?;
        Ok(context)
    }
}
```

## Advanced Features

### Workflow Validation

The example demonstrates comprehensive workflow validation:

```rust
// Validate workflow structure before execution
workflow.validate()
    .map_err(|e| match e {
        WorkflowError::CycleDetected => {
            WorkflowError::validation_error(
                "Workflow contains circular dependencies",
                "workflow_structure",
                "acyclic graph required",
                "in workflow validation"
            )
        }
        WorkflowError::UnreachableNodes { nodes } => {
            WorkflowError::validation_error(
                format!("Unreachable nodes found: {:?}", nodes),
                "workflow_connectivity", 
                "all nodes must be reachable",
                "in workflow validation"
            )
        }
        other => other,
    })?;
```

### Event Sourcing Integration

```rust
// Workflow execution generates events for event sourcing
let events = workflow.execute_with_events(input)?;

for event in events {
    println!("Event: {} at {}", event.event_type, event.timestamp);
    // Events are automatically stored in event store
}
```

### Debugging and Introspection

```rust
// Get workflow metadata for debugging
let metadata = workflow.get_metadata();
println!("Workflow: {}", metadata.name);
println!("Nodes: {:?}", metadata.node_ids);
println!("Connections: {:?}", metadata.connections);

// Trace execution path
let execution_trace = workflow.get_execution_trace();
for step in execution_trace {
    println!("Step {}: {} ({}ms)", step.index, step.node_id, step.duration_ms);
}
```

## Testing

### Unit Tests

```bash
# Run node tests
cargo test nodes

# Run workflow tests  
cargo test workflow

# Run integration tests
cargo test integration
```

### Integration Tests

```bash
# Start system stack
cd ../../..
./scripts/run-federation-stack.sh

# Run integration tests
cd examples/01_basic_workflow
cargo test --test integration_test -- --ignored
```

## Customization

### Adding New Node Types

1. Create a new node implementation:

```rust
// src/nodes/my_custom_node.rs
use workflow_engine_core::nodes::Node;
use workflow_engine_core::task::TaskContext;
use workflow_engine_core::error::WorkflowError;

#[derive(Debug)]
pub struct MyCustomNode {
    config: String,
}

impl Node for MyCustomNode {
    fn process(&self, context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Your custom processing logic here
        Ok(context)
    }
}
```

2. Register the node in your workflow:

```rust
workflow.register_node("my_custom", Box::new(MyCustomNode::new()))?;
```

### Modifying Processing Logic

Edit the node implementations in `src/nodes/` to change behavior:

- `text_input.rs` - Input validation and preprocessing
- `text_processor.rs` - Core text transformation logic
- `text_output.rs` - Result formatting and post-processing

### Adding Configuration

Extend nodes with configuration options:

```rust
#[derive(Debug, Clone)]
pub struct TextProcessorConfig {
    mode: String,
    preserve_case: bool,
    add_prefix: Option<String>,
}

impl TextProcessorNode {
    pub fn with_config(config: TextProcessorConfig) -> Self {
        Self { config }
    }
}
```

## Common Issues & Solutions

### Validation Errors

```
❌ Validation error in field 'text': Input cannot be empty
```

**Solution**: Ensure input text is non-empty and valid UTF-8.

### Node Not Found

```
❌ Node not found: unknown_node_type
```

**Solution**: Register all nodes before connecting them in the workflow.

### Cycle Detection

```
❌ Workflow contains cycles - check node connections
```

**Solution**: Review node connections to ensure the workflow is a directed acyclic graph (DAG).

### Processing Errors

```
❌ Processing error in node 'TextProcessorNode': Unknown processing mode: invalid_mode
```

**Solution**: Use valid processing modes: "uppercase", "lowercase", "reverse".

## Next Steps

1. **Try the Advanced Example**: Run `cargo run --example advanced` to see more complex workflows
2. **Explore Error Handling**: Run `cargo run --example error_handling` to see comprehensive error scenarios
3. **Build Custom Workflows**: Use this as a template for your own workflow implementations
4. **Integrate with MCP**: Move to the [MCP Integration example](../02_mcp_integration/) to add external service integration

## Further Reading

- **[Workflow Engine Core Documentation](../../crates/workflow-engine-core/README.md)**
- **[Node Development Guide](../../docs/NODES.md)**
- **[Error Handling Patterns](../../docs/ERROR_HANDLING.md)**
- **[Event Sourcing Integration](../04_event_sourcing/README.md)**