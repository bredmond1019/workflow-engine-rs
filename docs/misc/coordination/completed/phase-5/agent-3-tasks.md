# Agent Tasks: Architecture Agent

## Agent Role

**Primary Focus:** Restructure the project as a proper Rust workspace and improve API design for better type safety and ergonomics

## Key Responsibilities

- Transform monolithic structure into multi-crate workspace
- Improve public API design with type safety
- Implement async node execution pattern
- Establish clear module boundaries and visibility

## Assigned Tasks

### From Original Task List

- [x] 3.0 Restructure Project as Rust Workspace
  - [x] 3.1 Create workspace structure
    - [x] 3.1.1 Create root Cargo.toml with [workspace] configuration
    - [x] 3.1.2 Move core functionality to workflow-engine-core crate
    - [x] 3.1.3 Create workflow-engine-mcp crate for MCP protocol
    - [x] 3.1.4 Create workflow-engine-api crate for REST API
    - [x] 3.1.5 Create workflow-engine-nodes crate for built-in nodes
  - [ ] 3.2 Organize dependencies with feature flags
    - [ ] 3.2.1 Move database dependencies behind "database" feature
    - [ ] 3.2.2 Move monitoring dependencies behind "monitoring" feature
    - [ ] 3.2.3 Move AWS SDK dependencies behind "aws" feature
    - [ ] 3.2.4 Create minimal default feature set
  - [ ] 3.3 Update module visibility
    - [ ] 3.3.1 Mark internal modules with pub(crate)
    - [ ] 3.3.2 Create clear public API surface in lib.rs
    - [ ] 3.3.3 Document which modules are part of stable API
  - [ ] 3.4 Fix binary targets
    - [ ] 3.4.1 Properly declare demo binary in Cargo.toml
    - [ ] 3.4.2 Move application code to separate binary crate
  - [ ] 3.5 Configure examples
    - [ ] 3.5.1 Add [[example]] entries for all Rust examples
    - [ ] 3.5.2 Consider moving Python examples to separate directory

- [x] 4.0 Improve API Design and Type Safety
  - [x] 4.1 Convert Node trait to async
    - [x] 4.1.1 Create AsyncNode trait with async process method
    - [x] 4.1.2 Update all built-in nodes to implement AsyncNode
    - [x] 4.1.3 Update workflow executor to handle async nodes
    - [x] 4.1.4 Maintain backward compatibility with sync adapter
  - [x] 4.2 Replace TypeId with type-safe alternatives
    - [x] 4.2.1 Create NodeId<T> phantom type wrapper
    - [x] 4.2.2 Update NodeConfig to use type-safe node references
    - [x] 4.2.3 Implement type-safe workflow builder methods
  - [ ] 4.3 Improve error handling design
    - [ ] 4.3.1 Split WorkflowError into specific error types
    - [ ] 4.3.2 Add error context using anyhow or similar
    - [ ] 4.3.3 Implement proper error recovery strategies
  - [ ] 4.4 Enhance builder patterns
    - [ ] 4.4.1 Add compile-time validation to WorkflowBuilder
    - [ ] 4.4.2 Implement fluent API for node connections
    - [ ] 4.4.3 Add convenience methods for common patterns
  - [x] 4.5 Add testing utilities
    - [x] 4.5.1 Create mock_context() helper function
    - [x] 4.5.2 Add assert_node_output() test helper
    - [x] 4.5.3 Implement test fixtures for common scenarios

## Relevant Files

- `Cargo.toml` - Root configuration to convert to workspace
- `src/lib.rs` - Main library entry point for API surface definition
- `src/core/` - Core modules to move to workflow-engine-core
- `src/core/nodes/mod.rs` - Node trait requiring async conversion
- `src/core/mcp/` - MCP protocol code to move to separate crate
- `src/api/` - REST API code to move to workflow-engine-api
- `src/core/error/mod.rs` - Error types to be refactored
- `src/core/workflow/` - Workflow engine requiring async executor
- `src/bin/demo.rs` - Binary target to be properly configured
- `examples/` - Example files needing proper configuration

## Dependencies

### Prerequisites (What this agent needs before starting)

- **From Infrastructure Agent:** Updated Cargo.toml with new crate name (Task 1.2)
- **From Code Quality Agent:** Clean error handling patterns (Task 2.1)

### Provides to Others (What this agent delivers)

- **To Documentation & DevOps Agent:** Finalized public API for documentation
- **To Documentation & DevOps Agent:** Testing utilities for example creation
- **To All Agents:** New workspace structure for development

## Handoff Points

- **After Task 3.1:** Notify all agents about new workspace structure
- **After Task 4.1:** Notify Documentation & DevOps Agent that async API is ready
- **After Task 4.5:** Notify Documentation & DevOps Agent that testing utilities are available

## Testing Responsibilities

- Ensure workspace builds successfully with all crates
- Verify feature flags work correctly
- Test that async nodes execute properly
- Validate backward compatibility for existing code
- Test type-safe APIs compile correctly
- Ensure examples still work after restructuring

## Notes

- Maintain semantic versioning - this is a major version change
- Keep core crate minimal with few dependencies
- Use feature flags to make heavy dependencies optional
- Ensure clean separation between crates (no circular dependencies)
- Document migration path for existing users
- Consider using `async-trait` for the AsyncNode trait
- PhantomData pattern for type-safe node IDs