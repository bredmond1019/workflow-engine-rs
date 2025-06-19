# Contributing to AI Workflow Engine

Thank you for your interest in contributing to the AI Workflow Engine! This document provides guidelines and information for contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Making Changes](#making-changes)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Style Guidelines](#style-guidelines)
- [Community](#community)

## Code of Conduct

This project adheres to the [Contributor Covenant Code of Conduct](https://www.contributor-covenant.org/version/2/1/code_of_conduct/). By participating, you are expected to uphold this code. Please report unacceptable behavior to the project maintainers.

We are committed to providing a welcoming and inspiring community for all. Examples of behavior that contributes to a positive environment include:
- Using welcoming and inclusive language
- Being respectful of differing viewpoints and experiences
- Gracefully accepting constructive criticism
- Focusing on what is best for the community
- Showing empathy towards other community members

## Getting Started

### Prerequisites

- Rust 1.75.0 or later
- Cargo
- Git
- Docker (for integration tests)

### First Time Setup

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/workflow-engine-rs.git
   cd workflow-engine-rs
   ```
3. Add the upstream repository:
   ```bash
   git remote add upstream https://github.com/bredmond1019/workflow-engine-rs.git
   ```
4. Install dependencies and run tests:
   ```bash
   cargo build
   cargo test
   ```

## Development Setup

### Workspace Structure

The project uses a Cargo workspace with multiple crates:

```
crates/
├── workflow-engine-core/     # Core traits and execution engine
├── workflow-engine-mcp/      # MCP protocol implementation
├── workflow-engine-api/      # REST API server
├── workflow-engine-nodes/    # Built-in node implementations
└── workflow-engine-app/      # Main application binary
```

### Building the Project

```bash
# Build all crates
cargo build

# Build with all features
cargo build --all-features

# Build specific crate
cargo build -p workflow-engine-core
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p workflow-engine-core

# Run integration tests (requires Docker)
./scripts/start_test_servers.sh
cargo test -- --ignored
```

## Project Structure

- `crates/` - Workspace crates
- `services/` - Microservices
- `examples/` - Usage examples
- `docs/` - Documentation
- `scripts/` - Utility scripts
- `tests/` - Integration tests

## Making Changes

### Branch Naming

- Feature: `feature/description`
- Bug fix: `fix/description`
- Documentation: `docs/description`
- Refactor: `refactor/description`

### Commit Messages

Use conventional commits format:
- `feat:` for new features
- `fix:` for bug fixes
- `docs:` for documentation
- `refactor:` for refactoring
- `test:` for tests
- `chore:` for maintenance

Example: `feat(core): add async node execution support`

### Development Workflow

1. Create a new branch from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Make your changes following the style guidelines

3. Add tests for new functionality

4. Ensure all tests pass:
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```

5. Commit your changes:
   ```bash
   git add .
   git commit -m "feat: add new feature"
   ```

6. Push to your fork:
   ```bash
   git push origin feature/your-feature-name
   ```

7. Create a Pull Request

## Testing

### Unit Tests

Write unit tests for all public functions and methods. Place tests in the same file using `#[cfg(test)]` modules.

### Integration Tests

Integration tests are in the `tests/` directory. Use the provided test utilities:

```rust
use workflow_engine_core::testing::{mock_context, assert_node_output};

#[test]
fn test_custom_node() {
    let context = mock_context();
    let node = MyCustomNode::new();
    assert_node_output(&node, input_data, expected_output);
}
```

### Test Categories

- Unit tests: Test individual functions/methods
- Integration tests: Test component interactions
- End-to-end tests: Test complete workflows
- Performance tests: Benchmark critical paths

## Submitting Changes

### Pull Request Process

1. Ensure your PR addresses a single concern
2. Update documentation if needed
3. Add tests for new functionality
4. Ensure CI passes
5. Request review from maintainers

### PR Description Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature  
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Tests pass locally
- [ ] Added tests for new functionality
- [ ] Integration tests pass

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No breaking changes (or clearly documented)
```

## Style Guidelines

### Rust Code Style

- Use `cargo fmt` for formatting
- Follow `cargo clippy` suggestions
- Use descriptive variable names
- Add rustdoc comments for public APIs
- Prefer explicit error handling over unwrap/expect
- Use async/await for I/O operations

### Documentation

- Write clear, concise rustdoc comments
- Include examples in documentation
- Use markdown formatting
- Keep examples simple and focused

### Error Handling

- Use custom error types with `thiserror`
- Provide helpful error messages
- Include context where helpful
- Avoid panics in library code

## Community

### Getting Help

- GitHub Issues: Bug reports and feature requests
- Discussions: Questions and general discussion
- Discord: Real-time chat (if available)

### Reporting Issues

When reporting bugs, include:
- Rust version
- Operating system
- Minimal reproduction case
- Expected vs actual behavior
- Relevant logs/error messages

### Feature Requests

For feature requests, provide:
- Use case description
- Proposed API (if applicable)
- Alternative solutions considered
- Willingness to implement

## Recognition

Contributors are recognized in:
- CHANGELOG.md for significant contributions
- README.md contributors section
- Release notes for major features

Thank you for contributing to AI Workflow Engine! 🦀