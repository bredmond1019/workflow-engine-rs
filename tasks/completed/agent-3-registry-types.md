# Agent 3: Agent Registry & Type Fixes

You are Agent 3 responsible for fixing MockAgentRegistry implementation and type mismatch issues in the workflow-engine-api crate. Complete these tasks to resolve compilation errors.

**Your focus areas:**
- Creating MockAgentRegistry implementation
- Fixing type mismatches in registry operations
- Resolving Agent type conflicts

**Key requirements:**
- Implement MockAgentRegistry that satisfies all test needs
- Fix type conversions between different Agent types
- Ensure all imports are correct
- Maintain existing test logic

**Tasks:**

## 1. Create MockAgentRegistry Implementation
Location: Create in `crates/workflow-engine-api/src/testing/mocks.rs` or appropriate location

### [ ] Implement MockAgentRegistry
Used in these test files:
- `src/bootstrap/discovery.rs:387`
- `src/bootstrap/health.rs:491`
- `src/bootstrap/lifecycle.rs:474,506`
- `src/bootstrap/manager.rs:302,318`
- `src/bootstrap/registry.rs:485,530`

The mock should implement the `AgentRegistry` trait from `workflow_engine_core::registry::agent_registry`.

### [ ] Add proper imports
Ensure MockAgentRegistry is available where needed, either by:
- Creating it in a shared test utilities module
- Using the mockall crate if available
- Creating manual mock implementation

## 2. Fix Type Mismatches in registry.rs
Location: `crates/workflow-engine-api/src/bootstrap/registry.rs`

### [ ] Fix line 132: capabilities type
- Expected: `Vec<String>`
- Provided: `Value`
- Solution: Extract string array from Value or convert appropriately

### [ ] Fix line 133: metadata type
- Expected: `Value`
- Provided: `Option<_>`
- Solution: Unwrap or provide default Value

## 3. Resolve Agent Type Conflicts

### [ ] Identify Agent type usage
- `workflow_engine_core::registry::agent_registry::Agent` (from core)
- `crate::db::agent::Agent` (local db model)
- Ensure correct type is used in each context

### [ ] Update imports in test files
- Remove conflicting imports
- Use type aliases if needed to distinguish

### [ ] Fix service.rs test registry
Location: `crates/workflow-engine-api/src/bootstrap/service.rs:318`
- TestRegistry should return the correct Agent type
- Already partially fixed, verify it compiles

**Success criteria:**
- Run `cargo test -p workflow-engine-api --no-run` with no compilation errors
- All MockAgentRegistry usages resolve correctly
- Type mismatches are fixed
- No namespace conflicts

**Dependencies:** None - you can work independently

**Testing commands:**
```bash
# Check compilation
cargo check -p workflow-engine-api

# Build tests without running
cargo test -p workflow-engine-api --no-run

# Check specific module
cargo test -p workflow-engine-api bootstrap:: --no-run
```

**Implementation suggestions:**

For MockAgentRegistry:
```rust
use mockall::automock;
use workflow_engine_core::registry::agent_registry::{AgentRegistry, Agent, AgentRegistration, AgentRegistryError};

#[automock]
#[async_trait]
impl AgentRegistry for MockAgentRegistry {
    // Implement required methods
}
```

Or manual implementation:
```rust
struct MockAgentRegistry {
    // Internal state for testing
}

#[async_trait]
impl AgentRegistry for MockAgentRegistry {
    async fn register(&self, agent: AgentRegistration) -> Result<Agent, AgentRegistryError> {
        // Return test agent
    }
    // ... other methods
}
```

For each task:
- Understand the trait requirements first
- Implement minimal versions for testing
- Fix type conversions carefully
- Test compilation after each change
- Commit when a logical unit is complete