# Getting Started - Foundation Examples

Welcome to the foundation of AI Workflow Engine! These examples will teach you the core concepts you need to build powerful workflows.

## 🎯 Learning Objectives

By completing these examples, you will:
- Understand the fundamental Node trait
- Learn how TaskContext carries data through workflows
- Create your first workflow pipeline
- Handle basic errors and data validation
- Build simple but functional workflows

## 📚 Examples in This Section

### 1. hello-world
**File**: `hello-world.rs`
**Concepts**: Basic Node implementation, TaskContext usage, simple workflow execution
**Time**: 10 minutes

Your very first workflow! Learn how to:
- Create a simple node that processes data
- Use TaskContext to pass data between nodes
- Execute a workflow with input and output

```bash
cargo run --bin hello-world
```

### 2. basic-nodes
**File**: `basic-nodes.rs`
**Concepts**: Different node types, data transformation, metadata usage
**Time**: 15 minutes

Explore different types of nodes:
- Text processing nodes
- Data validation nodes
- Transformation nodes
- Metadata handling

```bash
cargo run --bin basic-nodes
```

### 3. data-flow
**File**: `data-flow.rs`
**Concepts**: Data serialization, type-safe data access, node chaining
**Time**: 20 minutes

Learn how data flows through workflows:
- Type-safe data extraction
- Storing and retrieving node results
- Working with different data types
- Understanding the data lifecycle

```bash
cargo run --bin data-flow
```

### 4. simple-pipeline
**File**: `simple-pipeline.rs`
**Concepts**: Workflow building, node chaining, pipeline patterns
**Time**: 25 minutes

Build your first complete pipeline:
- Chain multiple nodes together
- Create a real processing pipeline
- Handle data transformation through multiple stages
- Implement a complete workflow solution

```bash
cargo run --bin simple-pipeline
```

## 🛠 Setup

1. **Navigate to this directory**:
   ```bash
   cd examples/01-getting-started
   ```

2. **Run examples individually**:
   ```bash
   cargo run --bin hello-world
   cargo run --bin basic-nodes
   cargo run --bin data-flow
   cargo run --bin simple-pipeline
   ```

3. **Run all examples**:
   ```bash
   ./run-all.sh
   ```

## 📖 Key Concepts

### The Node Trait
Every processing unit in the workflow engine implements the Node trait:

```rust
pub trait Node: Send + Sync + Debug {
    fn process(&self, task_context: TaskContext) -> Result<TaskContext, WorkflowError>;
}
```

### TaskContext
The container that carries data through your workflow:
- Contains the original input data
- Stores results from each node
- Maintains metadata about execution
- Provides type-safe data access

### Basic Workflow Pattern
1. Create nodes that implement the Node trait
2. Build a workflow that defines the execution order
3. Execute the workflow with input data
4. Extract results from the final TaskContext

## 🎓 What You'll Learn

### After hello-world:
- How to implement the Node trait
- Basic TaskContext operations
- Simple workflow execution

### After basic-nodes:
- Different node design patterns
- Data validation techniques
- Metadata usage for debugging

### After data-flow:
- Type-safe data handling
- Complex data transformations
- Error handling best practices

### After simple-pipeline:
- Building complete workflows
- Chaining multiple processing stages
- Real-world workflow patterns

## 🔄 Next Steps

Once you've completed all examples in this section:

1. **Review the code** - Make sure you understand each example
2. **Experiment** - Try modifying the examples to see what happens
3. **Move to Core Concepts** - Proceed to `../02-core-concepts/`

## 💡 Tips for Success

1. **Read the comments** - Each example has detailed inline documentation
2. **Run tests** - Use `cargo test` to verify your understanding
3. **Experiment freely** - Copy examples and modify them
4. **Ask questions** - The code is documented to answer common questions

## 🧪 Testing

Each example includes comprehensive tests:

```bash
# Test individual examples
cargo test hello_world_tests
cargo test basic_nodes_tests
cargo test data_flow_tests
cargo test simple_pipeline_tests

# Test all getting-started examples
cargo test --package getting-started
```

## 🐛 Troubleshooting

### Common Issues

1. **Compilation errors**: Make sure you're in the right directory
2. **Missing dependencies**: Run `cargo build` to install dependencies
3. **Runtime errors**: Check that input data matches expected format

### Need Help?

- Check the inline comments in each example
- Review the main project documentation
- Open an issue if you find bugs or unclear documentation

---

**Ready to start?** Begin with the [hello-world example](hello-world.rs) and work through each example in order. Each one builds on the previous concepts!