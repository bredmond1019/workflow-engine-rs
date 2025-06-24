# Agent 1: MCP Test Failures Fix

You are Agent 1 responsible for fixing test failures in the workflow-engine-mcp crate. Complete these 7 test fixes.

**Your focus areas:**
- Config module test failures (5 tests)
- Transport module test failures (2 tests)

**Key requirements:**
- Fix test isolation issues (especially environment variables)
- Ensure tests are deterministic
- Fix assertion errors in error message formatting
- No changes to production code, only test code

**Tasks:**

## 1. Fix Config Module Tests
Location: `crates/workflow-engine-mcp/src/config.rs`

### [ ] Fix test_basic_mcp_config
- Issue: Test expects certain default values or environment setup
- Solution: Ensure test isolation and correct assertions

### [ ] Fix test_connection_pool_config
- Issue: Configuration validation or default values
- Solution: Update test expectations to match implementation

### [ ] Fix test_customer_support_server_config
- Issue: Server configuration initialization
- Solution: Provide required configuration values

### [ ] Fix test_external_server_config
- Issue: External server configuration parsing
- Solution: Fix test data or assertions

### [ ] Fix test_mcp_config_from_env_enabled
- Issue: Environment variable test contamination
- Solution: Use test-specific env vars or mock env

## 2. Fix Transport Module Tests
Location: `crates/workflow-engine-mcp/src/transport.rs`

### [ ] Fix test_transport_type_serialization
- Issue: Serialization format mismatch
- Solution: Update expected JSON format in assertions

### [ ] Fix test_transport_error_display
- Issue: Error message format has changed
- Solution: Update expected error message strings

**Success criteria:**
- Run `cargo test -p workflow-engine-mcp` and see all tests pass
- No test flakiness or environment dependencies
- Tests remain maintainable and clear

**Dependencies:** None - you can work independently

**Testing commands:**
```bash
# Run all MCP tests
cargo test -p workflow-engine-mcp

# Run specific test
cargo test -p workflow-engine-mcp test_basic_mcp_config -- --exact

# Run with output
cargo test -p workflow-engine-mcp -- --nocapture
```

For each task:
- Analyze the test failure carefully
- Make minimal changes to fix the issue
- Ensure the test still validates the intended behavior
- Commit after each test is fixed