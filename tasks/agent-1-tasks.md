# Agent Tasks: Infrastructure Agent

## Agent Role

**Primary Focus:** Fix critical compilation blockers and resolve security vulnerabilities to establish a stable foundation for open source publication.

## Key Responsibilities

- Implement missing constructors and core functionality to achieve compilation success
- Resolve all security vulnerabilities identified in cargo audit
- Update dependencies to secure, maintained versions
- Ensure the entire workspace compiles successfully with `cargo check --workspace`

## Assigned Tasks

### From Original Task List

- [ ] 1.0 Fix Critical Compilation Errors - (Originally task 1.0 from main list)
  - [ ] 1.1 Implement Missing JWT Authentication Methods - (Originally task 1.1 from main list)
    - [ ] 1.1.1 Add `JwtAuth::new(secret: String) -> Self` constructor in `crates/workflow-engine-api/src/auth.rs`
    - [ ] 1.1.2 Add `JwtMiddleware::new(secret: String) -> Self` constructor in `crates/workflow-engine-api/src/middleware.rs`
    - [ ] 1.1.3 Ensure both constructors properly initialize internal state and validate inputs
    - [ ] 1.1.4 Add unit tests for both constructors with valid and invalid inputs
  - [ ] 1.2 Re-enable and Fix Workflows Module - (Originally task 1.2 from main list)
    - [ ] 1.2.1 Uncomment `pub mod workflows;` in `crates/workflow-engine-api/src/lib.rs` line 53
    - [ ] 1.2.2 Uncomment workflows module re-exports in `crates/workflow-engine-api/src/lib.rs` lines 52-53
    - [ ] 1.2.3 Fix any compilation errors in the workflows module after re-enabling
    - [ ] 1.2.4 Ensure workflows module exports are properly documented
  - [ ] 1.3 Remove Unsafe Code Blocks - (Originally task 1.3 from main list)
    - [ ] 1.3.1 Replace unsafe environment variable setting in `crates/workflow-engine-app/src/main.rs` lines 16-27
    - [ ] 1.3.2 Implement proper error handling for SystemTime operations instead of unwrap()
    - [ ] 1.3.3 Use safe environment variable setting without unsafe block
    - [ ] 1.3.4 Add proper error propagation for startup configuration
  - [ ] 1.4 Fix Import Resolution Errors - (Originally task 1.4 from main list)
    - [ ] 1.4.1 Fix missing `workflows` import in `crates/workflow-engine-app/src/main.rs` line 8
    - [ ] 1.4.2 Ensure all module imports resolve correctly across workspace
    - [ ] 1.4.3 Update any broken internal crate dependencies
    - [ ] 1.4.4 Verify `cargo check --workspace` passes without errors

- [ ] 2.0 Resolve Security Vulnerabilities - (Originally task 2.0 from main list)
  - [ ] 2.1 Update Protobuf Dependency (RUSTSEC-2024-0437) - (Originally task 2.1 from main list)
    - [ ] 2.1.1 Update protobuf dependency to >=3.7.2 in workspace Cargo.toml
    - [ ] 2.1.2 Update any transitive dependencies that pull in vulnerable protobuf versions
    - [ ] 2.1.3 Test that updated protobuf version works with existing code
    - [ ] 2.1.4 Run `cargo audit` to verify vulnerability is resolved
  - [ ] 2.2 Replace Deprecated dotenv Dependency (RUSTSEC-2021-0141) - (Originally task 2.2 from main list)
    - [ ] 2.2.1 Replace `dotenv = "0.15.0"` with `dotenvy = "0.15"` in workspace dependencies
    - [ ] 2.2.2 Update all `use dotenv::` imports to `use dotenvy::`
    - [ ] 2.2.3 Update any dotenv method calls to dotenvy equivalents
    - [ ] 2.2.4 Test environment variable loading still works correctly
  - [ ] 2.3 Update proc-macro-error Chain (RUSTSEC-2024-0370) - (Originally task 2.3 from main list)
    - [ ] 2.3.1 Identify which dependencies bring in proc-macro-error (likely utoipa chain)
    - [ ] 2.3.2 Update utoipa and related dependencies to latest versions
    - [ ] 2.3.3 Verify OpenAPI documentation generation still works
    - [ ] 2.3.4 Ensure no breaking changes in updated dependencies
  - [ ] 2.4 Comprehensive Security Audit - (Originally task 2.4 from main list)
    - [ ] 2.4.1 Run `cargo audit` and ensure zero vulnerabilities reported
    - [ ] 2.4.2 Review all dependencies for maintenance status and security
    - [ ] 2.4.3 Set up automated security scanning in CI/CD pipeline
    - [ ] 2.4.4 Document security update process for future maintenance

## Relevant Files

- `crates/workflow-engine-app/src/main.rs` - Main application with compilation errors, unsafe blocks, and import issues
- `crates/workflow-engine-api/src/auth.rs` - JWT authentication implementation requiring new() constructor
- `crates/workflow-engine-api/src/middleware.rs` - JWT middleware implementation requiring new() constructor
- `crates/workflow-engine-api/src/lib.rs` - API crate exports that need workflows module re-enabled
- `crates/workflow-engine-api/src/workflows.rs` - Workflows module currently commented out
- `Cargo.toml` - Workspace dependencies requiring security updates (protobuf, dotenv, utoipa)
- `crates/*/Cargo.toml` - Individual crate dependencies that may need updates
- `Cargo.lock` - Dependency resolution file that will be updated with security fixes

## Dependencies

### Prerequisites (What this agent needs before starting)

- **None** - This agent handles the foundation and can start immediately

### Provides to Others (What this agent delivers)

- **To Code Quality Agent:** Compiling codebase free of critical errors
- **To Architecture Agent:** Working JWT authentication and workflows module for API improvements
- **To Documentation & DevOps Agent:** Security-clean dependencies for publication readiness testing

## Handoff Points

- **After Task 1.1:** Notify Architecture Agent that JWT authentication methods are implemented and ready for API improvements
- **After Task 1.2:** Notify all agents that workflows module is re-enabled and functional
- **After Task 1.4:** Notify all agents that workspace compilation is successful (`cargo check --workspace` passes)
- **After Task 2.4:** Notify Documentation & DevOps Agent that security audit is clean for CI/CD setup

## Testing Responsibilities

- Unit tests for JWT authentication constructors (tasks 1.1.4)
- Verification that `cargo check --workspace` passes without errors
- Verification that `cargo audit` reports zero vulnerabilities
- Integration testing coordination with other agents after compilation success

## Critical Success Criteria

- [ ] **Compilation Success:** `cargo check --workspace` passes without any errors
- [ ] **Security Clean:** `cargo audit` reports zero vulnerabilities
- [ ] **JWT Functionality:** Both JwtAuth::new() and JwtMiddleware::new() constructors work correctly
- [ ] **Workflows Enabled:** Workflows module is functional and properly exported
- [ ] **Safe Code:** No unnecessary unsafe blocks remain in production code

## Notes

- **Priority Order:** Complete Task 1.0 before Task 2.0 to unblock other agents quickly
- **Testing Strategy:** Test each change incrementally to avoid introducing new compilation errors
- **Security Focus:** Prioritize RUSTSEC-2024-0437 (protobuf) as it's a critical crash vulnerability
- **Coordination:** Keep other agents informed of compilation status as they depend on working code