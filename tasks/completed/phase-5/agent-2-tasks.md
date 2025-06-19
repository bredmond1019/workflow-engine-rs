# Agent Tasks: Code Quality Agent

## Agent Role

**Primary Focus:** Fix all critical code quality issues including error handling, unsafe code, and incomplete implementations

## Key Responsibilities

- Remove all unwrap() and expect() calls from production code
- Replace unsafe code with safe Rust alternatives
- Complete all todo!() and unimplemented!() macros
- Ensure proper error handling throughout the codebase

## Assigned Tasks

### From Original Task List

- [ ] 2.0 Fix Critical Code Quality Issues
  - [ ] 2.1 Remove unwrap() and expect() from production code
    - [ ] 2.1.1 Fix unwrap() calls in src/workflows/knowledge_base_workflow.rs (8 instances)
    - [ ] 2.1.2 Fix unwrap() calls in src/monitoring/metrics.rs (13+ instances)
    - [ ] 2.1.3 Fix expect() calls in src/db/session.rs (database initialization)
    - [ ] 2.1.4 Fix expect() calls in src/api/events.rs (database operations)
    - [ ] 2.1.5 Replace remaining unwrap() calls with proper error handling
  - [ ] 2.2 Replace unsafe code with safe alternatives
    - [ ] 2.2.1 Replace unsafe global static in src/api/uptime.rs with OnceCell
    - [ ] 2.2.2 Fix unsafe env::set_var in src/main.rs
    - [ ] 2.2.3 Review and fix unsafe code in src/core/mcp/config.rs tests
  - [ ] 2.3 Remove panic!() from non-test code
    - [ ] 2.3.1 Fix panic!() in src/core/streaming/websocket.rs:506
    - [ ] 2.3.2 Replace unreachable!() in src/core/ai/tokens/counter.rs:160 with error handling
  - [ ] 2.4 Complete todo!() implementations
    - [ ] 2.4.1 Implement 4 todo!() tests in tests/event_sourcing_tests.rs
    - [ ] 2.4.2 Fix unimplemented!() in tests/service_isolation_test.rs:121
  - [ ] 2.5 Replace debug prints with proper logging
    - [ ] 2.5.1 Replace eprintln!() in src/core/nodes/external_mcp_client.rs:151 with log crate
    - [ ] 2.5.2 Remove #![allow(warnings)] from src/lib.rs

## Relevant Files

- `src/workflows/knowledge_base_workflow.rs` - Contains 8 unwrap() calls to fix
- `src/monitoring/metrics.rs` - Contains 13+ unwrap() calls to fix
- `src/db/session.rs` - Database initialization with expect() calls
- `src/api/events.rs` - Database operations with expect() calls
- `src/api/uptime.rs` - Contains unsafe global static
- `src/main.rs` - Contains unsafe env::set_var calls
- `src/core/mcp/config.rs` - Test code with unsafe blocks
- `src/core/streaming/websocket.rs` - Contains panic!() call
- `src/core/ai/tokens/counter.rs` - Contains unreachable!() macro
- `src/core/nodes/external_mcp_client.rs` - Contains eprintln!() debug print
- `src/lib.rs` - Contains #![allow(warnings)] directive
- `tests/event_sourcing_tests.rs` - Contains 4 todo!() implementations
- `tests/service_isolation_test.rs` - Contains unimplemented!() macro

## Dependencies

### Prerequisites (What this agent needs before starting)

- None - This agent can start immediately

### Provides to Others (What this agent delivers)

- **To Architecture Agent:** Clean codebase without unwrap/panic for safe refactoring
- **To Documentation & DevOps Agent:** Completed tests for CI/CD pipeline setup
- **To All Agents:** Improved error types and handling patterns

## Handoff Points

- **After Task 2.1:** Notify Architecture Agent that error handling is standardized
- **After Task 2.2:** Notify Architecture Agent that unsafe code is removed
- **After Task 2.4:** Notify Documentation & DevOps Agent that all tests are complete

## Testing Responsibilities

- Ensure all error paths return proper Result types
- Verify no panics occur in production code paths
- Run existing tests to ensure no regressions
- Add tests for new error handling paths where missing
- Validate that async code properly propagates errors

## Notes

- Use `once_cell` crate for replacing unsafe global statics
- Prefer `?` operator over explicit match for error propagation
- Add context to errors using `.context()` or `.map_err()` where appropriate
- For database errors, ensure connection issues are handled gracefully
- Keep error messages helpful and actionable
- Test error paths as thoroughly as success paths