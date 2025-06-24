# Multi-Agent Test Fix Coordination

## Overview
Fixing test compilation errors across workflow-engine-mcp and workflow-engine-api crates.

- Total Errors: 37 (15 in mcp, 22 in api)
- Agents: 3
- Estimated Time: 1-2 hours parallel work

## Agent Assignment

### Agent 1: MCP Test Failures
- **Owner**: Agent-1-MCP-Tests
- **Scope**: workflow-engine-mcp test failures
- **Tasks**: 7 failing tests (config and transport modules)
- **Files**:
  - `crates/workflow-engine-mcp/src/config.rs` (tests)
  - `crates/workflow-engine-mcp/src/transport.rs` (tests)

### Agent 2: Event Store Implementation
- **Owner**: Agent-2-Event-Store
- **Scope**: workflow-engine-api EventStore trait implementations
- **Tasks**: Fix missing trait methods in mocks
- **Files**:
  - `crates/workflow-engine-api/src/db/events/error_integration.rs`
  - `crates/workflow-engine-api/src/db/events/tests/integration_tests.rs`
  - `crates/workflow-engine-api/src/api/routes/ordering.rs`

### Agent 3: Agent Registry & Types
- **Owner**: Agent-3-Registry-Types
- **Scope**: workflow-engine-api MockAgentRegistry and type fixes
- **Tasks**: Create MockAgentRegistry, fix type mismatches
- **Files**:
  - `crates/workflow-engine-api/src/bootstrap/*.rs` (test sections)
  - `crates/workflow-engine-api/src/workflows/registry.rs`

## Progress Tracking

### Agent 1 Status: [ ] Not Started
- [ ] Fix test_basic_mcp_config
- [ ] Fix test_connection_pool_config
- [ ] Fix test_customer_support_server_config
- [ ] Fix test_external_server_config
- [ ] Fix test_mcp_config_from_env_enabled
- [ ] Fix test_transport_type_serialization
- [ ] Fix test_transport_error_display

### Agent 2 Status: [ ] Not Started
- [ ] Implement missing EventStore methods in error_integration.rs
- [ ] Implement missing EventStore methods in integration_tests.rs
- [ ] Fix EventMetadata initialization
- [ ] Fix unwrap() method call issue

### Agent 3 Status: [ ] Not Started
- [ ] Create MockAgentRegistry implementation
- [ ] Fix capabilities type mismatch
- [ ] Fix metadata type mismatch
- [ ] Resolve Agent type conflicts
- [ ] Update all MockAgentRegistry usages

## Dependencies
- No blocking dependencies between agents
- All agents can work in parallel
- Final verification requires all agents to complete

## Success Criteria
1. `cargo test -p workflow-engine-mcp` - 0 failures
2. `cargo test -p workflow-engine-api --no-run` - compiles successfully
3. No new compilation errors introduced