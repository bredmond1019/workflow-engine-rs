# Task List: Production Readiness Implementation

Generated from: `/tasks/post-agent-3.md`  
Date: December 14, 2024

## Relevant Files

### Critical Production Files
- `src/core/mcp/clients/slack.rs` - Slack MCP client implementation (currently returns mock data)
- `src/core/mcp/clients/helpscout.rs` - HelpScout MCP client implementation (currently returns mock data)
- `src/core/mcp/clients/notion.rs` - Notion MCP client implementation (currently returns mock data)
- `services/knowledge_graph/src/client/dgraph.rs` - Dgraph client with unimplemented parsing methods
- `src/core/streaming/sse.rs` - SSE streaming module missing Duration import
- `src/core/ai/tokens/tests/pricing_tests.rs` - Token pricing tests missing VolumeTier import
- `src/api/events.rs` - Event processing API with type mismatches

### Test Files
- `tests/mcp_integration_tests.rs` - Integration tests for MCP clients
- `tests/knowledge_graph_tests.rs` - Knowledge graph integration tests
- `src/core/mcp/clients/tests/` - Unit tests for MCP clients
- `services/knowledge_graph/src/client/tests/` - Dgraph client unit tests
- `tests/streaming_tests.rs` - Streaming functionality tests
- `tests/token_tests.rs` - Token counting and pricing tests

### Bootstrap and Infrastructure
- `src/bootstrap/` - Bootstrap directory to be created
- `src/bootstrap/container.rs` - Dependency injection container
- `src/bootstrap/service.rs` - Service initialization logic
- `src/bootstrap/config.rs` - Configuration management
- `src/bootstrap/tests/` - Bootstrap system unit tests

### Documentation
- `README.md` - Main project documentation
- `DEVELOPMENT_SETUP.md` - Development environment setup guide
- `docs/architecture.md` - Architecture documentation
- `docs/implementation-status.md` - Implementation status tracking

### MCP Protocol and Transport
- `src/core/mcp/protocol.rs` - MCP protocol implementation (needs tests)
- `src/core/mcp/transport.rs` - Transport layer implementation (needs tests)
- `src/core/nodes/external_mcp_client.rs` - External MCP client node (needs tests)
- `src/core/nodes/registry.rs` - Node registry system (needs tests)

### Customer Support Tools
- `src/core/mcp/clients/slack/tools/` - Slack tool implementations
- `src/core/mcp/clients/helpscout/tools/` - HelpScout tool implementations
- `src/core/mcp/clients/notion/tools/` - Notion tool implementations

### Notes

- Unit tests should be placed alongside the code files they are testing
- Integration tests go in the `/tests` directory
- Use `cargo test` to run all tests, `cargo test --test <name>` for specific integration tests
- MCP server tests require test servers to be running (`./scripts/start_test_servers.sh`)
- Total of 298 tests should pass after all fixes are implemented

## Tasks

- [ ] **1.0 Fix Critical Test Compilation Errors**
  - [ ] **1.1 Fix SSE Module Import Errors**
    - [ ] 1.1.1 Open `src/core/streaming/sse.rs` and add `use std::time::Duration;` import
    - [ ] 1.1.2 Verify all other time-related imports are present
    - [ ] 1.1.3 Run `cargo check` to confirm SSE module compiles
  - [ ] **1.2 Fix Token Pricing Test Imports**
    - [ ] 1.2.1 Open `src/core/ai/tokens/tests/pricing_tests.rs`
    - [ ] 1.2.2 Add missing import: `use crate::core::ai::tokens::VolumeTier;`
    - [ ] 1.2.3 Check for any other missing type imports in pricing tests
    - [ ] 1.2.4 Run `cargo test --lib core::ai::tokens` to verify pricing tests compile
  - [ ] **1.3 Fix Event Processing Type Mismatches**
    - [ ] 1.3.1 Analyze compilation errors in `src/api/events.rs`
    - [ ] 1.3.2 Fix type mismatches between expected and actual event types
    - [ ] 1.3.3 Update event handler signatures to match expected types
    - [ ] 1.3.4 Ensure event serialization/deserialization is consistent
  - [ ] **1.4 Identify and Fix Additional Compilation Issues**
    - [ ] 1.4.1 Run `cargo test --no-run` to identify all compilation errors
    - [ ] 1.4.2 Create a list of all failing modules
    - [ ] 1.4.3 Fix each compilation error systematically
    - [ ] 1.4.4 Document any breaking changes that need attention
  - [ ] **1.5 Verify Full Test Suite Compilation**
    - [ ] 1.5.1 Run `cargo test --workspace --no-run` to check all tests compile
    - [ ] 1.5.2 Fix any remaining compilation errors in test files
    - [ ] 1.5.3 Ensure all 298 tests are discoverable by test runner
    - [ ] 1.5.4 Create CI workflow to prevent future compilation breakages

- [ ] **2.0 Implement Real Customer Support Tool Integrations**
  - [ ] **2.1 Replace Mock Slack Client Implementation**
    - [ ] 2.1.1 Open `src/core/mcp/clients/slack.rs` and identify all mock responses
    - [ ] 2.1.2 Implement real MCP protocol calls using the connection pool
    - [ ] 2.1.3 Add proper error handling for network failures
    - [ ] 2.1.4 Implement retry logic with exponential backoff
    - [ ] 2.1.5 Add logging for debugging Slack API interactions
    - [ ] 2.1.6 Update Slack tools to use real client calls
  - [ ] **2.2 Replace Mock HelpScout Client Implementation**
    - [ ] 2.2.1 Open `src/core/mcp/clients/helpscout.rs` and remove hardcoded responses
    - [ ] 2.2.2 Implement actual HelpScout API integration via MCP
    - [ ] 2.2.3 Add authentication and session management
    - [ ] 2.2.4 Implement rate limiting to respect API quotas
    - [ ] 2.2.5 Add comprehensive error handling for API responses
    - [ ] 2.2.6 Create helper functions for common HelpScout operations
  - [ ] **2.3 Replace Mock Notion Client Implementation**
    - [ ] 2.3.1 Open `src/core/mcp/clients/notion.rs` and identify stub methods
    - [ ] 2.3.2 Implement real Notion API calls through MCP protocol
    - [ ] 2.3.3 Add support for Notion's block-based content model
    - [ ] 2.3.4 Implement pagination for large result sets
    - [ ] 2.3.5 Add caching layer for frequently accessed Notion data
    - [ ] 2.3.6 Handle Notion API versioning and deprecations
  - [ ] **2.4 Create Integration Tests for Customer Support Tools**
    - [ ] 2.4.1 Create `tests/slack_integration_test.rs` with real API tests
    - [ ] 2.4.2 Create `tests/helpscout_integration_test.rs` with real API tests
    - [ ] 2.4.3 Create `tests/notion_integration_test.rs` with real API tests
    - [ ] 2.4.4 Add test fixtures and mock data for reliable testing
    - [ ] 2.4.5 Implement test cleanup to avoid polluting external services
  - [ ] **2.5 Verify MCP Connection Pooling Integration**
    - [ ] 2.5.1 Test connection reuse across multiple tool calls
    - [ ] 2.5.2 Verify connection health checks work with real endpoints
    - [ ] 2.5.3 Test failover behavior when primary connection fails
    - [ ] 2.5.4 Measure performance improvements from connection pooling
    - [ ] 2.5.5 Add metrics for connection pool utilization

- [ ] **3.0 Complete Knowledge Graph Result Parsing**
  - [ ] **3.1 Implement GraphQL Response Parsing**
    - [ ] 3.1.1 Open `services/knowledge_graph/src/client/dgraph.rs`
    - [ ] 3.1.2 Replace `unimplemented!()` in `parse_query_result` method
    - [ ] 3.1.3 Implement JSON to domain object mapping
    - [ ] 3.1.4 Handle nested GraphQL response structures
    - [ ] 3.1.5 Add support for GraphQL aliases and fragments
  - [ ] **3.2 Implement Mutation Result Parsing**
    - [ ] 3.2.1 Implement `parse_mutation_result` method
    - [ ] 3.2.2 Extract UIDs from mutation responses
    - [ ] 3.2.3 Handle bulk mutation results
    - [ ] 3.2.4 Parse error responses and conflict information
    - [ ] 3.2.5 Return structured mutation results with metadata
  - [ ] **3.3 Add Comprehensive Error Handling**
    - [ ] 3.3.1 Create custom error types for parsing failures
    - [ ] 3.3.2 Implement graceful degradation for partial results
    - [ ] 3.3.3 Add detailed error context for debugging
    - [ ] 3.3.4 Handle network timeouts and connection errors
    - [ ] 3.3.5 Implement circuit breaker for repeated failures
  - [ ] **3.4 Create Unit Tests for Response Parsing**
    - [ ] 3.4.1 Create test fixtures for various GraphQL responses
    - [ ] 3.4.2 Test parsing of successful query responses
    - [ ] 3.4.3 Test parsing of error responses
    - [ ] 3.4.4 Test edge cases (empty results, nulls, malformed JSON)
    - [ ] 3.4.5 Test parsing of complex nested structures
  - [ ] **3.5 Add Integration Tests with Dgraph**
    - [ ] 3.5.1 Set up test Dgraph instance with Docker
    - [ ] 3.5.2 Create test schema and sample data
    - [ ] 3.5.3 Test real queries against Dgraph instance
    - [ ] 3.5.4 Verify mutations work correctly
    - [ ] 3.5.5 Test transaction handling and rollbacks

- [ ] **4.0 Create Service Bootstrap System**
  - [ ] **4.1 Create Bootstrap Directory Structure**
    - [ ] 4.1.1 Create `src/bootstrap/` directory
    - [ ] 4.1.2 Create `src/bootstrap/mod.rs` with module exports
    - [ ] 4.1.3 Create subdirectories for different bootstrap components
    - [ ] 4.1.4 Add bootstrap module to main library exports
    - [ ] 4.1.5 Update Cargo.toml if needed for new dependencies
  - [ ] **4.2 Implement Dependency Injection Container**
    - [ ] 4.2.1 Create `src/bootstrap/container.rs`
    - [ ] 4.2.2 Implement service registration mechanism
    - [ ] 4.2.3 Add service resolution with dependency graph
    - [ ] 4.2.4 Implement singleton and transient lifetimes
    - [ ] 4.2.5 Add circular dependency detection
    - [ ] 4.2.6 Create builder pattern for container configuration
  - [ ] **4.3 Add Service Initialization Logic**
    - [ ] 4.3.1 Create `src/bootstrap/service.rs`
    - [ ] 4.3.2 Implement service lifecycle management
    - [ ] 4.3.3 Add startup and shutdown hooks
    - [ ] 4.3.4 Implement health check registration
    - [ ] 4.3.5 Add graceful shutdown handling
    - [ ] 4.3.6 Create service dependency ordering
  - [ ] **4.4 Create Configuration Management System**
    - [ ] 4.4.1 Create `src/bootstrap/config.rs`
    - [ ] 4.4.2 Implement configuration loading from files
    - [ ] 4.4.3 Add environment variable overrides
    - [ ] 4.4.4 Implement configuration validation
    - [ ] 4.4.5 Add configuration hot-reloading support
    - [ ] 4.4.6 Create typed configuration structs
  - [ ] **4.5 Update Documentation**
    - [ ] 4.5.1 Update README.md to reflect actual bootstrap implementation
    - [ ] 4.5.2 Create bootstrap usage examples
    - [ ] 4.5.3 Document service registration patterns
    - [ ] 4.5.4 Add migration guide from current initialization
    - [ ] 4.5.5 Create architectural decision record (ADR)

- [ ] **5.0 Add Comprehensive Test Coverage**
  - [ ] **5.1 Add MCP Protocol Unit Tests**
    - [ ] 5.1.1 Create test module in `src/core/mcp/protocol.rs`
    - [ ] 5.1.2 Test message serialization and deserialization
    - [ ] 5.1.3 Test protocol version negotiation
    - [ ] 5.1.4 Test error message handling
    - [ ] 5.1.5 Test all protocol message types
    - [ ] 5.1.6 Add property-based tests for protocol invariants
  - [ ] **5.2 Add Transport Layer Tests**
    - [ ] 5.2.1 Create tests for HTTP transport in `src/core/mcp/transport.rs`
    - [ ] 5.2.2 Add WebSocket transport tests
    - [ ] 5.2.3 Test stdio transport implementation
    - [ ] 5.2.4 Test transport connection lifecycle
    - [ ] 5.2.5 Add tests for transport error handling
    - [ ] 5.2.6 Test message framing and buffering
  - [ ] **5.3 Create External MCP Client Node Tests**
    - [ ] 5.3.1 Add test module to `src/core/nodes/external_mcp_client.rs`
    - [ ] 5.3.2 Mock external MCP services for testing
    - [ ] 5.3.3 Test node initialization and configuration
    - [ ] 5.3.4 Test error propagation from external services
    - [ ] 5.3.5 Test timeout and retry behavior
    - [ ] 5.3.6 Add integration tests with real MCP servers
  - [ ] **5.4 Increase Customer Support Tool Coverage**
    - [ ] 5.4.1 Add unit tests for each Slack tool implementation
    - [ ] 5.4.2 Add unit tests for each HelpScout tool implementation
    - [ ] 5.4.3 Add unit tests for each Notion tool implementation
    - [ ] 5.4.4 Test tool parameter validation
    - [ ] 5.4.5 Test tool error handling scenarios
    - [ ] 5.4.6 Aim for 80%+ code coverage on all tools
  - [ ] **5.5 Add Cross-Component Integration Tests**
    - [ ] 5.5.1 Create end-to-end workflow tests
    - [ ] 5.5.2 Test MCP client to server communication
    - [ ] 5.5.3 Test workflow execution with external tools
    - [ ] 5.5.4 Test system behavior under load
    - [ ] 5.5.5 Add chaos testing for resilience verification

---

## Task Completion Tracking

**Total Tasks:** 125 (25 parent tasks + 100 sub-tasks)
**Priority Order:** Tasks 1-3 are critical blockers, Task 4-5 are high priority

## Success Metrics

- All 298 tests compile and pass
- Customer support tools return real data from external services
- Knowledge graph queries execute successfully against Dgraph
- Test coverage exceeds 80% for critical paths
- Bootstrap system provides clean dependency injection
- Documentation accurately reflects implementation state

## Timeline Estimate

- **Week 1:** Complete Task 1 (Fix compilation) and start Task 2
- **Week 2:** Complete Tasks 2-3 (Real implementations)
- **Week 3:** Complete Tasks 4-5 (Bootstrap and testing)

This task list can be executed by junior developers with clear, actionable steps for each component.