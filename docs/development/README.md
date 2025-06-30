# Development Documentation

This directory contains all development-related documentation for the AI Workflow Engine project.

## Getting Started

### Setup Guides
- [Development Setup](DEVELOPMENT_SETUP.md) - Basic development environment setup
- [Development Setup Guide](DEVELOPMENT_SETUP_GUIDE.md) - Detailed step-by-step setup instructions
- [Testing Guide](TESTING.md) - Comprehensive testing documentation
- [Troubleshooting](TROUBLESHOOTING_ADDENDUM.md) - Common development issues and solutions

## Development Workflow

### Initial Setup
1. **Prerequisites**: Rust, PostgreSQL, Docker
2. **Environment**: Clone repository and set up environment variables
3. **Database**: Initialize PostgreSQL and run migrations
4. **Dependencies**: Install Rust dependencies and Python MCP servers
5. **Verification**: Run test suite to ensure everything works

### Daily Development
1. **Code Changes**: Make your changes following project conventions
2. **Testing**: Run relevant test suites
3. **Quality Checks**: Run cargo fmt, clippy, and security audits
4. **Documentation**: Update documentation as needed

### Code Quality Standards

#### Rust Code Standards
- **Formatting**: Use `cargo fmt` for consistent formatting
- **Linting**: Address all `cargo clippy` warnings
- **Error Handling**: Use the project's error types and patterns
- **Testing**: Write unit tests for all new functionality
- **Documentation**: Include rustdoc comments for public APIs

#### Project Conventions
- **Naming**: Use clear, descriptive names for functions and variables
- **Modules**: Organize code into logical modules with clear responsibilities
- **Dependencies**: Minimize external dependencies and justify new ones
- **Performance**: Consider performance implications of code changes

### Testing Strategy

#### Test Categories
1. **Unit Tests**: Test individual functions and methods
2. **Integration Tests**: Test component interactions
3. **End-to-End Tests**: Test complete workflow scenarios
4. **Load Tests**: Test system under load
5. **Chaos Tests**: Test system resilience

#### Running Tests
```bash
# All tests
cargo test

# Unit tests only
cargo test --lib

# Integration tests (requires external services)
./scripts/start_test_servers.sh
cargo test -- --ignored

# Specific test suites
cargo test --test end_to_end_workflow_test -- --ignored
cargo test --test load_test -- --ignored --nocapture
cargo test --test chaos_test -- --ignored --nocapture
```

#### Test Writing Guidelines
- **Isolation**: Tests should be independent and not rely on external state
- **Clarity**: Test names should clearly describe what is being tested
- **Coverage**: Aim for comprehensive test coverage of critical paths
- **Mocking**: Use mockall for external dependencies in unit tests
- **Data**: Use test fixtures for consistent test data

### Development Tools

#### Essential Commands
```bash
# Build the project
cargo build
cargo build --release

# Run the main server
cargo run --bin workflow-engine

# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Security audit
cargo audit

# Generate documentation
cargo doc --open
```

#### Docker Development
```bash
# Run full stack with Docker Compose
docker-compose up -d

# View logs
docker-compose logs -f ai-workflow-system

# Rebuild containers
docker-compose build --no-cache
```

#### MCP Server Development
```bash
# Start test MCP servers
./scripts/start_test_servers.sh

# Test individual MCP server
python scripts/test_mcp_server.py <server_name>

# Run MCP server tests
cd mcp-servers && python -m pytest tests/
```

### Database Development

#### Migration Management
```bash
# Create new migration
diesel migration generate <migration_name>

# Run migrations
diesel migration run

# Revert migration
diesel migration revert
```

#### Database Testing
- Use test databases for integration tests
- Clean up test data between test runs
- Use transactions for test isolation where possible

### Common Development Tasks

#### Adding a New API Endpoint
1. Define the endpoint in the appropriate API module
2. Implement the handler function
3. Add database operations if needed
4. Write unit and integration tests
5. Update OpenAPI documentation
6. Add monitoring metrics

#### Creating a New Workflow Node
1. Implement the node trait in workflow-engine-nodes
2. Add configuration structures
3. Register the node in the node registry
4. Write comprehensive tests
5. Add documentation and examples

#### Integrating External Services
1. Implement MCP client if applicable
2. Add configuration for the service
3. Implement error handling and retry logic
4. Add health checks and monitoring
5. Write integration tests with mocked services

#### Adding Database Changes
1. Create database migration
2. Update repository layer
3. Update API endpoints if needed
4. Add tests for new functionality
5. Update documentation

### Debugging and Profiling

#### Logging
- Use structured logging with correlation IDs
- Log at appropriate levels (trace, debug, info, warn, error)
- Include relevant context in log messages
- Use the monitoring/correlation module for request tracking

#### Performance Profiling
```bash
# Profile with cargo flamegraph
cargo install flamegraph
cargo flamegraph --bin workflow-engine

# Benchmark performance
cargo bench

# Memory profiling with valgrind
valgrind --tool=callgrind target/release/workflow-engine
```

#### Monitoring and Metrics
- Use Prometheus metrics for monitoring
- View dashboards in Grafana (http://localhost:3000)
- Check health endpoints for service status
- Monitor logs for errors and warnings

### Security Considerations

#### Code Security
- Validate all user inputs
- Use parameterized queries for database operations
- Avoid hardcoded secrets in code
- Follow secure coding practices for Rust

#### Dependency Security
- Regularly run `cargo audit` to check for vulnerabilities
- Keep dependencies up to date
- Review new dependencies before adding them
- Use minimal dependency versions when possible

### Contributing Guidelines

#### Before Contributing
1. Check existing issues and PRs to avoid duplication
2. Discuss major changes in an issue first
3. Ensure you understand the project architecture
4. Set up the development environment properly

#### Making Changes
1. Create a feature branch from main
2. Make atomic commits with clear messages
3. Write or update tests for your changes
4. Ensure all tests pass
5. Update documentation as needed

#### Submitting Changes
1. Run the full test suite
2. Check code quality with clippy and fmt
3. Write a clear PR description
4. Link related issues
5. Be responsive to review feedback

For component-specific development guidance, refer to the CLAUDE.md files in each crate and service directory.