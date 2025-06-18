# Tutorial 1: Getting Started with AI Workflows

Welcome to the AI Workflow System! If you're new to workflow orchestration or wondering how to build AI-powered applications, you're in the right place. By the end of this tutorial, you'll have created your first working AI workflow using the current system APIs.

## What Are AI Workflows?

Think of an AI workflow like a smart assembly line in a factory. Just as an assembly line has different stations where workers perform specific tasks (install the engine, paint the car, add the wheels), an AI workflow has different "nodes" where each one performs a specific AI-powered task.

But here's the exciting part: instead of just following rigid instructions, AI workflows can think, make decisions, and adapt based on what they encounter. It's like having a team of intelligent workers who can solve problems and handle unexpected situations!

### Why Should You Care?

Imagine you run a customer support team and get hundreds of emails daily. Instead of reading each one manually, an AI workflow could:
- Automatically understand the customer's intent
- Route urgent issues to the right person
- Draft helpful responses using company knowledge
- Learn from patterns to get better over time

That's the power of AI workflows - they automate intelligent tasks that normally require human thinking.

## Understanding the Current Architecture

Before we build anything, let's understand how the system works with the current APIs:

### TaskContext: The Information Carrier

The `TaskContext` is like a briefcase that travels through your workflow, carrying:
- **Event Data**: The original input that started the workflow
- **Node Results**: What each step has accomplished  
- **Metadata**: Additional information about processing

```rust
use backend::core::task::TaskContext;
use serde_json::json;

// Create a task context with input data
let context = TaskContext::new(
    "customer_support".to_string(),
    json!({
        "customer_id": "CUST-123", 
        "message": "My order hasn't arrived yet",
        "priority": "high"
    })
);

println!("Event ID: {}", context.event_id);
println!("Workflow: {}", context.workflow_type);
```

### Nodes: The Workers

Nodes are individual processing units that implement the `Node` trait. Each node:
1. Receives a `TaskContext`
2. Processes the data
3. Adds its results to the context
4. Returns the updated context

```rust
use backend::core::nodes::Node;
use backend::core::task::TaskContext;
use backend::core::error::WorkflowError;
use serde_json::json;

#[derive(Debug)]
struct CustomerAnalysisNode;

impl Node for CustomerAnalysisNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Get the customer message from the input
        let input: serde_json::Value = context.get_event_data()?;
        let message = input.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        // Analyze the message (simplified logic)
        let urgency_score = if message.contains("urgent") || message.contains("asap") {
            9
        } else if message.contains("problem") || message.contains("issue") {
            7
        } else {
            5
        };
        
        let category = if message.contains("order") || message.contains("delivery") {
            "shipping"
        } else if message.contains("billing") || message.contains("payment") {
            "billing"
        } else {
            "general"
        };
        
        // Store our analysis results using the correct API
        context.update_node("customer_analysis", json!({
            "urgency_score": urgency_score,
            "category": category,
            "message_length": message.len(),
            "analysis_timestamp": chrono::Utc::now()
        }));
        
        // Add metadata for tracking
        context.set_metadata("processing_node", "customer_analysis")?;
        
        println!("✅ Analyzed message: urgency={}, category={}", urgency_score, category);
        Ok(context)
    }
}
```

## Building Your First Workflow: Simple Text Processor

Let's create a practical workflow that demonstrates the core concepts with a working example that you can actually run. We'll build a simple text processing workflow that validates input and performs basic analysis.

### Step 1: Create a Text Validation Node

First, let's create a node that validates incoming text data:

```rust
use backend::core::nodes::Node;
use backend::core::task::TaskContext;
use backend::core::error::WorkflowError;
use serde_json::json;

#[derive(Debug)]
struct TextValidationNode;

impl Node for TextValidationNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("🔍 Step 1: Validating text input...");
        
        // Extract the input data
        let input: serde_json::Value = context.get_event_data()?;
        let text = input.get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        // Perform validation checks
        let mut validation_errors = Vec::new();
        
        if text.trim().is_empty() {
            validation_errors.push("Text cannot be empty");
        }
        
        if text.len() > 1000 {
            validation_errors.push("Text too long (max 1000 characters)");
        }
        
        if text.chars().all(|c| c.is_numeric()) {
            validation_errors.push("Text cannot be only numbers");
        }
        
        let is_valid = validation_errors.is_empty();
        
        // Store validation results
        context.update_node("validation", json!({
            "is_valid": is_valid,
            "errors": validation_errors,
            "text_length": text.len(),
            "word_count": text.split_whitespace().count(),
            "processed_at": chrono::Utc::now()
        }));
        
        if is_valid {
            println!("   ✅ Text is valid and ready for analysis");
        } else {
            println!("   ❌ Validation failed: {:?}", validation_errors);
        }
        
        Ok(context)
    }
}
```

### Step 2: Create a Text Analysis Node

Now let's add a node that analyzes the text:

```rust
#[derive(Debug)]
struct TextAnalysisNode;

impl Node for TextAnalysisNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("🧠 Step 2: Analyzing text...");
        
        // Check if validation passed
        let validation_data = context.get_node_data::<serde_json::Value>("validation")?
            .ok_or_else(|| WorkflowError::ProcessingError {
                message: "No validation data found".to_string()
            })?;
        
        let is_valid = validation_data.get("is_valid")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        if !is_valid {
            // Skip analysis for invalid text
            context.update_node("analysis", json!({
                "analyzed": false,
                "reason": "Skipped due to validation failure"
            }));
            return Ok(context);
        }
        
        // Get the original text
        let input: serde_json::Value = context.get_event_data()?;
        let text = input.get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        // Simple text analysis
        let sentences = text.split('.').filter(|s| !s.trim().is_empty()).count();
        let words = text.split_whitespace().count();
        let avg_word_length = if words > 0 {
            text.chars().filter(|c| c.is_alphabetic()).count() as f64 / words as f64
        } else {
            0.0
        };
        
        // Simple sentiment analysis
        let positive_words = ["good", "great", "excellent", "love", "amazing", "helpful", "thanks"];
        let negative_words = ["bad", "terrible", "awful", "hate", "disappointed", "frustrated", "problem"];
        
        let text_lower = text.to_lowercase();
        let positive_count = positive_words.iter()
            .filter(|word| text_lower.contains(*word))
            .count();
        
        let negative_count = negative_words.iter()
            .filter(|word| text_lower.contains(*word))
            .count();
        
        let sentiment = if positive_count > negative_count {
            "positive"
        } else if negative_count > positive_count {
            "negative"
        } else {
            "neutral"
        };
        
        // Store analysis results
        context.update_node("analysis", json!({
            "analyzed": true,
            "sentences": sentences,
            "words": words,
            "avg_word_length": avg_word_length,
            "sentiment": sentiment,
            "positive_signals": positive_count,
            "negative_signals": negative_count,
            "analyzed_at": chrono::Utc::now()
        }));
        
        println!("   📊 Analysis: {} words, {} sentences, {} sentiment", 
                 words, sentences, sentiment);
        
        Ok(context)
    }
}
```

### Step 3: Create a Report Generation Node

Finally, let's create a node that generates a summary report:

```rust
#[derive(Debug)]
struct ReportGeneratorNode;

impl Node for ReportGeneratorNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("📋 Step 3: Generating report...");
        
        // Get analysis results
        let analysis_data = context.get_node_data::<serde_json::Value>("analysis")?
            .ok_or_else(|| WorkflowError::ProcessingError {
                message: "No analysis data found".to_string()
            })?;
        
        let analyzed = analysis_data.get("analyzed")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        if !analyzed {
            context.update_node("report", json!({
                "generated": false,
                "reason": "Analysis was skipped"
            }));
            return Ok(context);
        }
        
        // Get analysis details
        let words = analysis_data.get("words").and_then(|v| v.as_u64()).unwrap_or(0);
        let sentences = analysis_data.get("sentences").and_then(|v| v.as_u64()).unwrap_or(0);
        let sentiment = analysis_data.get("sentiment").and_then(|v| v.as_str()).unwrap_or("neutral");
        let avg_word_length = analysis_data.get("avg_word_length").and_then(|v| v.as_f64()).unwrap_or(0.0);
        
        // Get original text for reference
        let input: serde_json::Value = context.get_event_data()?;
        let original_text = input.get("text").and_then(|v| v.as_str()).unwrap_or("");
        
        // Generate report based on analysis
        let complexity = if avg_word_length > 6.0 {
            "complex"
        } else if avg_word_length > 4.0 {
            "moderate"
        } else {
            "simple"
        };
        
        let report_text = format!(
            "Text Analysis Report:\n\
             - Length: {} words, {} sentences\n\
             - Complexity: {} (avg word length: {:.1})\n\
             - Sentiment: {}\n\
             - Reading level: {}",
            words, 
            sentences, 
            complexity, 
            avg_word_length, 
            sentiment,
            if words > 100 { "intermediate" } else { "basic" }
        );
        
        // Store the generated report
        context.update_node("report", json!({
            "generated": true,
            "report_text": report_text,
            "summary": {
                "word_count": words,
                "sentence_count": sentences,
                "complexity": complexity,
                "sentiment": sentiment,
                "estimated_reading_time_seconds": words * 60 / 250  // ~250 WPM average
            },
            "generated_at": chrono::Utc::now()
        }));
        
        println!("   📊 Report generated: {} words, {} complexity, {} sentiment", 
                 words, complexity, sentiment);
        
        Ok(context)
    }
}
```

### Step 4: Build and Run the Complete Workflow

Now let's put it all together in a runnable example:

```rust
use backend::core::task::TaskContext;
use backend::core::nodes::Node;
use backend::core::error::WorkflowError;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Text Processing Workflow");
    println!("===========================\n");
    
    // Create our processing nodes
    let validation_node = TextValidationNode;
    let analysis_node = TextAnalysisNode;
    let report_node = ReportGeneratorNode;
    
    // Test data - different types of text
    let test_cases = vec![
        json!({
            "text": "This is a wonderful example of positive text! I love how clear and helpful this explanation is. Great work on making this easy to understand."
        }),
        json!({
            "text": "This system is terrible and frustrating. Nothing works properly and the documentation is awful. I hate dealing with this problem."
        }),
        json!({
            "text": "The implementation demonstrates sophisticated algorithmic approaches to natural language processing. This methodology utilizes advanced computational linguistics techniques."
        }),
        json!({
            "text": "Hello world. This is short."
        }),
    ];
    
    // Process each test case
    for (index, test_data) in test_cases.iter().enumerate() {
        println!("🔄 Processing text sample #{}", index + 1);
        println!("─".repeat(50));
        
        let original_text = test_data.get("text").and_then(|v| v.as_str()).unwrap_or("");
        println!("📝 Original text: \"{}\"", 
                if original_text.len() > 60 { 
                    format!("{}...", &original_text[..60]) 
                } else { 
                    original_text.to_string() 
                });
        
        // Create task context
        let mut context = TaskContext::new(
            "text_processing".to_string(),
            test_data.clone()
        );
        
        // Execute the workflow pipeline
        context = validation_node.process(context)?;
        context = analysis_node.process(context)?;
        context = report_node.process(context)?;
        
        // Display results
        if let Some(report_data) = context.get_node_data::<serde_json::Value>("report")? {
            if let Some(report_text) = report_data.get("report_text").and_then(|v| v.as_str()) {
                println!("\n📊 Analysis Report:");
                for line in report_text.lines() {
                    println!("   {}", line);
                }
            }
            
            if let Some(summary) = report_data.get("summary") {
                if let Some(reading_time) = summary.get("estimated_reading_time_seconds").and_then(|v| v.as_u64()) {
                    println!("   ⏱️  Estimated reading time: {} seconds", reading_time);
                }
            }
        }
        
        println!("\n");
    }
    
    println!("✨ Workflow demonstration completed!");
    println!("\n🎯 What you learned:");
    println!("   - How to create nodes that implement the Node trait");
    println!("   - How TaskContext carries data between nodes");
    println!("   - How to store and retrieve results from nodes"); 
    println!("   - How to chain nodes together for processing");
    
    Ok(())
}
```

## Running Your Workflow

To run this workflow:

1. **Set up your environment** (following the setup from Tutorial index):
   ```bash
   cargo build
   ./scripts/quick-start.sh
   ```

2. **Create a new example file**:
   ```bash
   # Create examples/tutorial_01_text_processor.rs with the code above
   # Copy all the node implementations and main function
   ```

3. **Add it to Cargo.toml** (if needed):
   ```toml
   [[bin]]
   name = "tutorial_01_text_processor"
   path = "examples/tutorial_01_text_processor.rs"
   ```

4. **Run your workflow**:
   ```bash
   cargo run --example tutorial_01_text_processor
   ```

   Or try the existing working example:
   ```bash
   cargo run --example basic-workflow
   ```

## Understanding the Data Flow

Here's what happens when your workflow runs:

```
[Input Text] 
    ↓
┌─────────────────┐
│TextValidationNode│ ← Checks text quality
├─────────────────┤
│ • Check length  │
│ • Validate format│
│ • Store results │
└────────┬────────┘
         ↓
┌─────────────────┐
│ TextAnalysisNode│ ← Analyzes text content  
├─────────────────┤
│ • Count words   │
│ • Find sentiment │
│ • Calculate stats│
└────────┬────────┘
         ↓
┌─────────────────┐
│ReportGenerator  │ ← Creates summary report
├─────────────────┤
│ • Read analysis │
│ • Format report │
│ • Add metadata  │
└────────┬────────┘
         ↓
[Complete Report]
```

Each node adds its results to the TaskContext, building up a complete analysis.

## Key Concepts You've Learned

✅ **TaskContext**: The data container that flows through your workflow

✅ **Node Trait**: How to create processing units that implement specific logic

✅ **Data Flow**: How information passes and accumulates between nodes

✅ **Error Handling**: Graceful handling of validation and processing errors

✅ **Typed Data Access**: Using `get_event_data()` and `get_node_data()` for type safety

✅ **Result Storage**: Using `update_node()` to store processing results

✅ **Sequential Processing**: How nodes execute one after another to build up results

## Common Patterns You Should Know

### 1. Data Validation Pattern
```rust
let input: MyDataType = context.get_event_data()?;
if !is_valid(&input) {
    return Err(WorkflowError::ValidationError { 
        message: "Invalid input".to_string() 
    });
}
```

### 2. Conditional Processing Pattern
```rust
let previous_result = context.get_node_data::<serde_json::Value>("previous_node")?;
if let Some(data) = previous_result {
    // Process only if previous step succeeded
}
```

### 3. Error Recovery Pattern
```rust
let result = match risky_operation() {
    Ok(data) => data,
    Err(_) => {
        // Store error info and continue with fallback
        context.set_metadata("processing_error", "Used fallback")?;
        fallback_value
    }
};
```

## Next Steps

Now that you understand the basics:

1. **Experiment**: Modify the sentiment analysis logic or add new validation rules
2. **Extend**: Add more nodes like "Category Classification" or "Response Translation"
3. **Learn More**: Continue to [Tutorial 2: Understanding Nodes and Data Flow](./02-understanding-nodes.md)

## Troubleshooting

**Common Issue**: "Failed to deserialize event data"
- **Solution**: Make sure your input JSON matches your struct fields exactly

**Common Issue**: "No validation data found"  
- **Solution**: Ensure nodes are executed in the correct order

**Common Issue**: Node panics
- **Solution**: Always use `?` for error propagation instead of `unwrap()`

## Quick Reference

```rust
// Create TaskContext
let context = TaskContext::new("workflow_type".to_string(), input_data);

// Get input data (typed)
let input: serde_json::Value = context.get_event_data()?;

// Get node result (typed)
let result: Option<serde_json::Value> = context.get_node_data("node_name")?;

// Store node result
context.update_node("my_result", my_data);

// Store metadata
context.set_metadata("processing_info", "completed")?;

// Basic node implementation
impl Node for MyNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // 1. Get data
        let input: serde_json::Value = context.get_event_data()?;
        
        // 2. Process data
        let result = process_my_data(&input)?;
        
        // 3. Store results
        context.update_node("my_result", result);
        
        Ok(context)
    }
}
```

## What's Next?

Congratulations! You've built your first working workflow using the current system APIs. You now understand:

- How TaskContext carries data through workflows
- How to implement the Node trait for processing
- How to chain nodes together for complex processing
- How to handle errors gracefully

Ready for more? Continue to:

1. **[Tutorial 2: Understanding Nodes and Data Flow](./02-understanding-nodes.md)** - Learn advanced node patterns and complex workflows
2. **Try the real examples** - Run `cargo run --example basic-workflow` to see the knowledge base system in action
3. **Experiment** - Modify the text processor to add new analysis features

## Troubleshooting

**Common Issue**: "Failed to deserialize event data"
- **Solution**: Make sure your input JSON matches what your code expects

**Common Issue**: "No validation data found"  
- **Solution**: Ensure nodes are executed in the correct order

**Common Issue**: Import errors
- **Solution**: Check that you're using the correct module paths from the current codebase

You're ready to build more sophisticated AI workflows!