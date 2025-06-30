# Agent Tasks: Architecture Agent

## Agent Role

**Primary Focus:** Complete missing implementations, ensure API consistency with Rust guidelines, and implement professional error handling and builder patterns for optimal developer experience.

## Key Responsibilities

- Remove all stub implementations and TODO comments from public APIs
- Standardize naming conventions and ensure Rust API Guidelines compliance
- Implement proper error types with chaining and context
- Add builder patterns for complex configuration objects

## Assigned Tasks

### From Original Task List

- [ ] 3.0 Complete Missing Implementations and API Polish - (Originally task 3.0 from main list)
  - [ ] 3.1 Remove Stub Implementations and TODO Comments - (Originally task 3.1 from main list)
    - [ ] 3.1.1 Audit all files for TODO, FIXME, and unimplemented!() macros
    - [ ] 3.1.2 Complete or remove stub MCP client methods in workflow builders
    - [ ] 3.1.3 Implement missing bootstrap service functionality or remove references
    - [ ] 3.1.4 Remove placeholder implementations in AI agent nodes or disable features properly
  - [ ] 3.2 Fix API Naming Consistency (Rust API Guidelines) - (Originally task 3.2 from main list)
    - [ ] 3.2.1 Standardize MCP vs Mcp naming throughout codebase (choose one pattern)
    - [ ] 3.2.2 Review all public struct and enum names for consistency with Rust conventions
    - [ ] 3.2.3 Ensure method names follow Rust naming guidelines (snake_case)
    - [ ] 3.2.4 Update documentation to reflect naming changes
  - [ ] 3.3 Implement Proper Error Types and Chaining - (Originally task 3.3 from main list)
    - [ ] 3.3.1 Replace string-only error variants with structured error types
    - [ ] 3.3.2 Add `#[source]` attributes for proper error chaining using thiserror
    - [ ] 3.3.3 Provide context-rich error messages with actionable information
    - [ ] 3.3.4 Implement Display and Debug traits properly for all error types
  - [ ] 3.4 Add Builder Patterns for Complex Configuration - (Originally task 3.4 from main list)
    - [ ] 3.4.1 Implement builder pattern for NodeConfig with proper validation
    - [ ] 3.4.2 Create builder for McpConfig with fluent interface
    - [ ] 3.4.3 Add builder for WorkflowBuilder with type-safe configuration
    - [ ] 3.4.4 Ensure all builders have proper error handling and validation

## Relevant Files

- `crates/workflow-engine-core/src/lib.rs` - Core crate public API surface and exports
- `crates/workflow-engine-core/src/errors.rs` - Error types needing proper chaining and context
- `crates/workflow-engine-core/src/workflow.rs` - Workflow builder implementation
- `crates/workflow-engine-core/src/node.rs` - Node configuration and builder patterns
- `crates/workflow-engine-mcp/src/lib.rs` - MCP crate public API and naming consistency
- `crates/workflow-engine-mcp/src/config.rs` - MCP configuration requiring builder pattern
- `crates/workflow-engine-nodes/src/lib.rs` - Node implementations with disabled AI agents
- `crates/workflow-engine-api/src/bootstrap.rs` - Bootstrap service implementation (may be incomplete)
- All public API files across crates requiring naming consistency review

## Dependencies

### Prerequisites (What this agent needs before starting)

- **From Infrastructure Agent:** Working JWT authentication methods (Task 1.1) and workflows module (Task 1.2)
- **From Infrastructure Agent:** Successful compilation to enable API testing and validation

### Provides to Others (What this agent delivers)

- **To Code Quality Agent:** Consistent error handling patterns for anti-pattern elimination
- **To Documentation & DevOps Agent:** Complete, professional APIs ready for publication
- **To All Agents:** Rust-idiomatic APIs that follow community best practices

## Handoff Points

- **Before Task 3.1:** Wait for Infrastructure Agent to complete workflows module re-enabling (Task 1.2)
- **During Task 3.3:** Coordinate with Code Quality Agent on error handling patterns (Task 4.1)
- **After Task 3.2:** Notify Code Quality Agent about naming changes for documentation updates
- **After Task 3.4:** Notify Documentation & DevOps Agent that APIs are ready for publication testing

## Testing Responsibilities

- Unit tests for all builder patterns and configuration validation
- Integration tests for complex configuration scenarios
- API usability testing with real-world examples
- Error handling tests for all new error types and chaining

## Critical Success Criteria

- [ ] **Zero Stub Implementations:** No TODO, FIXME, or unimplemented!() in public APIs
- [ ] **Naming Consistency:** All APIs follow Rust naming conventions consistently
- [ ] **Professional Error Handling:** Structured errors with proper chaining and context
- [ ] **Builder Patterns:** Fluent, type-safe builders for all complex configuration
- [ ] **API Guidelines Compliance:** Adherence to Rust API Guidelines throughout

## Detailed Implementation Strategy

### 3.1 Stub Implementation Audit:
1. **Search strategy:** Use `rg -n "TODO|FIXME|unimplemented!"` to find all instances
2. **Classification:** Categorize as complete, remove, or disable feature
3. **MCP clients:** Either implement fully or use feature flags to disable
4. **Bootstrap service:** Assess if needed for core functionality or remove

### 3.2 Naming Standardization:
1. **MCP convention:** Choose either "MCP" or "Mcp" and apply consistently
2. **Struct/enum review:** Ensure PascalCase for types, snake_case for functions
3. **Module naming:** Ensure consistency across all crate public interfaces
4. **Documentation sync:** Update all docs to reflect naming decisions

### 3.3 Error Type Architecture:
```rust
// Target error structure example
#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    #[error("Node processing failed: {message}")]
    NodeProcessing {
        node_id: String,
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    #[error("Configuration validation failed for {field}: {reason}")]
    ConfigValidation {
        field: String,
        reason: String,
        provided_value: Option<String>,
    },
}
```

### 3.4 Builder Pattern Design:
```rust
// Target builder pattern example
pub struct NodeConfigBuilder<T: Node> {
    name: Option<String>,
    timeout: Option<Duration>,
    retry_policy: Option<RetryPolicy>,
    _phantom: PhantomData<T>,
}

impl<T: Node> NodeConfigBuilder<T> {
    pub fn new() -> Self { /* ... */ }
    pub fn name(mut self, name: impl Into<String>) -> Self { /* ... */ }
    pub fn timeout(mut self, timeout: Duration) -> Self { /* ... */ }
    pub fn build(self) -> Result<NodeConfig<T>, ConfigError> { /* ... */ }
}
```

## Notes

- **API Design Philosophy:** Prioritize type safety and compile-time correctness over runtime flexibility
- **Error Context:** Every error should provide enough context for users to understand and fix the issue
- **Builder Validation:** Implement comprehensive validation in builders rather than at usage time
- **Backward Compatibility:** Consider future API evolution when designing public interfaces