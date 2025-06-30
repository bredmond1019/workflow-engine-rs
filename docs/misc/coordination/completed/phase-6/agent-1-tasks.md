# Agent Tasks: Build & Infrastructure Agent

## Agent Role

**Primary Focus:** Critical compilation fixes, dependency management, and infrastructure alignment to unblock all other development work.

## Key Responsibilities

- Fix all compilation errors preventing the project from building
- Resolve workspace dependency configuration issues
- Align infrastructure documentation with actual deployment configuration
- Ensure Docker and deployment infrastructure works as documented
- Provide foundational stability for other agents to build upon

## Assigned Tasks

### From Original Task List

- [ ] **1.0 Fix Critical Compilation Errors** - [Originally task 1.0 from main list]
  - [ ] **1.1 Resolve utoipa-swagger-ui dependency issues in API layer** - [Originally task 1.1 from main list]
    - [ ] 1.1.1 Analyze utoipa-swagger-ui version conflicts in workflow-engine-api
    - [ ] 1.1.2 Update Cargo.toml dependencies to compatible versions
    - [ ] 1.1.3 Fix import statements and feature flag usage for utoipa
    - [ ] 1.1.4 Test API compilation with `cargo build -p workflow-engine-api`
  - [ ] **1.2 Fix type mismatches and missing imports in workflow-engine-api** - [Originally task 1.2 from main list]
    - [ ] 1.2.1 Identify and catalog all type mismatch errors
    - [ ] 1.2.2 Update import paths for new workspace structure
    - [ ] 1.2.3 Fix trait implementations and generic type parameters
    - [ ] 1.2.4 Resolve async/await compatibility issues
  - [ ] **1.3 Resolve dependency conflicts in workflow-engine-nodes package** - [Originally task 1.3 from main list]
    - [ ] 1.3.1 Analyze circular dependency issues between crates
    - [ ] 1.3.2 Update dependency versions for compatibility
    - [ ] 1.3.3 Fix module export structure in workflow-engine-nodes
    - [ ] 1.3.4 Test nodes compilation independently
  - [ ] **1.4 Fix workspace dependency configuration issues** - [Originally task 1.4 from main list]
    - [ ] 1.4.1 Verify all workspace dependencies have correct version specifications
    - [ ] 1.4.2 Fix feature flag propagation across workspace crates
    - [ ] 1.4.3 Ensure consistent Rust edition across all crates
    - [ ] 1.4.4 Test full workspace compilation with `cargo build --workspace`

- [ ] **3.0 Align Infrastructure and Deployment Configuration** - [Originally task 3.0 from main list]
  - [ ] **3.1 Add microservices to docker-compose.yml or update README deployment info** - [Originally task 3.1 from main list]
    - [ ] 3.1.1 Assess whether microservices should be included in docker-compose.yml
    - [ ] 3.1.2 Either add service definitions for content_processing, knowledge_graph, and realtime_communication
    - [ ] 3.1.3 Or update README to clarify microservice deployment strategy
    - [ ] 3.1.4 Ensure port mappings match documentation (8082, 3002, 8081)
  - [ ] **3.2 Create missing MCP server infrastructure or update documentation** - [Originally task 3.2 from main list]
    - [ ] 3.2.1 Assess if Python MCP servers directory structure is needed
    - [ ] 3.2.2 Either create `mcp-servers/` directory with Python implementations
    - [ ] 3.2.3 Or update README to reflect actual MCP server architecture using scripts
    - [ ] 3.2.4 Ensure MCP server startup scripts work as documented
  - [ ] **3.3 Align docker-compose services with README claims** - [Originally task 3.3 from main list]
    - [ ] 3.3.1 Verify all services mentioned in README exist in docker-compose.yml
    - [ ] 3.3.2 Check port mappings match README documentation
    - [ ] 3.3.3 Ensure environment variable configuration is consistent
    - [ ] 3.3.4 Test full docker-compose stack startup and connectivity

## Relevant Files

### Core Compilation and Dependencies
- `crates/workflow-engine-api/Cargo.toml` - API crate dependencies requiring utoipa-swagger-ui fixes
- `crates/workflow-engine-api/src/lib.rs` - Main API library entry point with compilation errors
- `crates/workflow-engine-nodes/Cargo.toml` - Nodes crate dependency resolution and circular dependency fixes
- `Cargo.toml` - Root workspace configuration and dependency management
- `crates/workflow-engine-core/src/lib.rs` - Core library exports and conditional compilation
- `crates/workflow-engine-mcp/Cargo.toml` - MCP crate dependencies requiring cleanup

### Infrastructure and Deployment
- `docker-compose.yml` - Container orchestration configuration requiring microservice alignment
- `README.md` - Main documentation requiring infrastructure alignment updates
- `scripts/` - MCP server infrastructure scripts requiring validation
- `Dockerfile` - Container build configuration
- `.env.example` - Environment variable template
- `docker-compose.test.yml` - Test-specific container configuration

## Dependencies

### Prerequisites (What this agent needs before starting)
- **None** - This agent has no blocking dependencies and can start immediately

### Provides to Others (What this agent delivers)
- **To Architecture Cleanup Agent:** Working compilation environment for MCP client removal
- **To Core Features Agent:** Compiled codebase for pricing engine development and testing
- **To Quality & Documentation Agent:** Working test compilation and Docker infrastructure for testing
- **To All Agents:** Stable build environment and deployment infrastructure

## Handoff Points

- **After Task 1.1:** Notify Core Features Agent that API layer compiles for pricing engine work
- **After Task 1.4:** Notify Quality & Documentation Agent that workspace builds and tests can compile
- **After Task 3.0:** Notify Quality & Documentation Agent that Docker infrastructure is ready for testing
- **After Full Completion:** Notify all agents that foundational build and infrastructure issues are resolved

## Testing Responsibilities

- Test compilation incrementally with `cargo build` after each fix
- Verify workspace builds successfully with `cargo build --workspace`
- Test Docker stack functionality with `docker-compose up --build`
- Validate test compilation readiness with `cargo test --no-run`

## Critical Success Criteria

- [ ] **All workspace crates compile without errors** (`cargo build --workspace` succeeds)
- [ ] **API layer builds successfully** (`cargo build -p workflow-engine-api` succeeds)
- [ ] **Docker stack starts without errors** (`docker-compose up` succeeds)
- [ ] **Test compilation works** (`cargo test --no-run` succeeds)
- [ ] **Infrastructure documentation matches reality** (README deployment instructions work)

## Implementation Priority Order

1. **Start with Task 1.1** - Fix utoipa-swagger-ui issues (highest impact, blocks API development)
2. **Follow with Task 1.4** - Fix workspace configuration (unblocks all crate compilation)
3. **Continue with Tasks 1.2 & 1.3** - Fix remaining compilation issues
4. **Finish with Task 3.0** - Align infrastructure (enables proper testing and deployment)

## Notes

- This agent is on the **critical path** - all other agents depend on successful completion
- Focus on getting basic compilation working before perfecting infrastructure
- Document any architectural decisions made during infrastructure alignment
- Coordinate with Architecture Cleanup Agent if MCP dependency changes affect compilation
- Use incremental testing (`cargo build -p <crate>`) to isolate and fix issues systematically