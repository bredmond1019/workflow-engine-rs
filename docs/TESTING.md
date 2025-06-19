# Testing Guide for AI Workflow Engine

This guide explains how to run tests for the AI Workflow Engine, including setup requirements and different test categories.

## Quick Start

### Running Tests Without External Dependencies

The project is configured to run most tests without requiring external services:

```bash
# Run all tests with mocked dependencies
./scripts/test_setup.sh

# Run specific test categories
./scripts/test_setup.sh -- unit      # Unit tests only
./scripts/test_setup.sh -- api       # API tests only
./scripts/test_setup.sh -- workflow  # Workflow tests only
```

### Running Tests With Real Infrastructure

Some integration tests require external services. To run these:

```bash
# Start required services and run all tests
./scripts/test_setup.sh --with-infrastructure

# Run specific integration tests
./scripts/test_setup.sh --with-infrastructure integration
```

## Test Categories

### 1. Unit Tests
- **Location**: Throughout the codebase in `src/` directories
- **Requirements**: None (uses mocks)
- **Run**: `cargo test --lib`
- **Coverage**: Core business logic, algorithms, data structures

### 2. Integration Tests
- **Location**: `tests/` directory
- **Requirements**: Some require external services (marked with `#[ignore]`)
- **Run**: `cargo test --test '*'`
- **Coverage**: Component interactions, API endpoints, database operations

### 3. MCP Protocol Tests
- **Location**: `tests/mcp_*.rs` files
- **Requirements**: MCP test servers (Python)
- **Setup**: `./scripts/start_test_servers.sh`
- **Run**: `cargo test mcp_ -- --ignored`
- **Coverage**: MCP client/server communication, protocol compliance

### 4. End-to-End Tests
- **Location**: `tests/end_to_end_*.rs`
- **Requirements**: Full infrastructure
- **Run**: `cargo test --test end_to_end_workflow_test -- --ignored`
- **Coverage**: Complete workflow scenarios

### 5. Load Tests
- **Location**: `tests/load_test.rs`
- **Requirements**: Performance testing environment
- **Run**: `cargo test --test load_test -- --ignored --nocapture`
- **Coverage**: System performance, scalability

### 6. Chaos Tests
- **Location**: `tests/chaos_test.rs`
- **Requirements**: Full infrastructure
- **Run**: `cargo test --test chaos_test -- --ignored --nocapture`
- **Coverage**: Failure scenarios, resilience

## Test Infrastructure Requirements

### Minimal Setup (Default)
- Rust toolchain
- No external dependencies
- Uses in-memory databases and mocked services

### Full Infrastructure Setup

#### 1. PostgreSQL Database
```bash
# Install PostgreSQL
# macOS: brew install postgresql
# Ubuntu: sudo apt-get install postgresql

# Create test database
createdb workflow_test_db

# Initialize schema
psql workflow_test_db < scripts/init-db.sql
```

#### 2. MCP Test Servers
```bash
# Install Python dependencies
cd mcp-servers
pip install -r requirements.txt

# Start test servers
cd ..
./scripts/start_test_servers.sh
```

#### 3. Docker Services (Optional)
```bash
# Start all services with Docker Compose
docker-compose -f docker-compose.test.yml up -d
```

## Environment Variables

### Test Configuration
```bash
# Use mocked services (default: true)
export TEST_USE_IN_MEMORY_DB=true
export TEST_USE_MOCK_MCP=true
export TEST_DISABLE_EXTERNAL_SERVICES=true

# Test database URL (when not using in-memory)
export TEST_DATABASE_URL="postgresql://test:test@localhost/workflow_test_db"

# Skip specific test categories
export SKIP_INTEGRATION_TESTS=false
export SKIP_SLOW_TESTS=false

# Enable debug logging
export RUST_LOG=debug
export RUST_BACKTRACE=1
```

### AI Provider Keys (Optional)
```bash
# Only needed for tests that use real AI providers
export OPENAI_API_KEY="your-key"
export ANTHROPIC_API_KEY="your-key"
```

## Test Configuration File

The `test-config.toml` file provides comprehensive test configuration:

```toml
[test]
use_in_memory_db = true
use_mock_mcp_servers = true
disable_external_services = true

[test.timeouts]
default_timeout_seconds = 30
integration_test_timeout_seconds = 60

[test.ai_providers]
use_deterministic_responses = true
```

## Writing Tests

### Unit Test Example
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_creation() {
        let workflow = WorkflowBuilder::new("test")
            .add_node(TestNode::new())
            .build();
        
        assert_eq!(workflow.name(), "test");
    }
}
```

### Integration Test Example
```rust
#[tokio::test]
async fn test_api_endpoint() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::get("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore] // Requires external services
async fn test_with_real_database() {
    let db = setup_test_database().await;
    // Test implementation
}
```

### Using Test Fixtures
```rust
use workflow_engine_core::testing::fixtures::*;

#[test]
fn test_with_fixture() {
    let workflow = create_test_workflow("example");
    let context = create_test_context();
    
    // Run workflow with test data
}
```

## Debugging Failed Tests

### 1. Enable Debug Logging
```bash
RUST_LOG=debug cargo test failing_test_name -- --nocapture
```

### 2. Run Single Test
```bash
cargo test test_name -- --exact --nocapture
```

### 3. Check Test Output
```bash
# Save test output to file
cargo test 2>&1 | tee test_output.log
```

### 4. Use Test Utilities
```rust
// Print debug information in tests
dbg!(&variable);
println!("Debug: {:?}", result);
```

## Continuous Integration

Tests are automatically run in CI with the following strategy:

1. **Fast Tests**: Unit tests and mocked integration tests
2. **Full Tests**: All tests including those requiring infrastructure
3. **Platform Tests**: Linux, macOS, and Windows compatibility

See `.github/workflows/test.yml` for CI configuration.

## Troubleshooting

### Common Issues

1. **"Database connection failed"**
   - Ensure PostgreSQL is running
   - Check TEST_DATABASE_URL is correct
   - Run with `TEST_USE_IN_MEMORY_DB=true`

2. **"MCP server not responding"**
   - Start test servers: `./scripts/start_test_servers.sh`
   - Check ports 8001-8003 are available
   - Use `TEST_USE_MOCK_MCP=true`

3. **"Test timeout"**
   - Increase timeout in test-config.toml
   - Check for deadlocks or infinite loops
   - Run with `--test-threads=1` for debugging

4. **"Too many open files"**
   - Increase ulimit: `ulimit -n 4096`
   - Ensure tests clean up resources
   - Run tests in smaller batches

## Best Practices

1. **Use Mocks for Unit Tests**: Keep unit tests fast and isolated
2. **Mark External Dependencies**: Use `#[ignore]` for tests requiring infrastructure
3. **Clean Up Resources**: Ensure tests clean up database records, files, etc.
4. **Use Deterministic Data**: Avoid random data that can cause flaky tests
5. **Test Error Cases**: Include tests for failure scenarios
6. **Document Requirements**: Clearly indicate what infrastructure each test needs

## Contributing

When adding new tests:

1. Place unit tests in the same file as the code being tested
2. Place integration tests in the `tests/` directory
3. Use descriptive test names that explain what is being tested
4. Add appropriate `#[ignore]` attributes for tests requiring external services
5. Update this documentation if adding new test categories or requirements