# Pull Request

## Summary
Brief description of the changes and their purpose.

## Related Issues
Fixes #[issue number]
Related to #[issue number]

## Type of Change
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Performance improvement
- [ ] Code refactoring (no functional changes)
- [ ] Documentation update
- [ ] Dependency update
- [ ] CI/CD changes

## Changes Made

### Core Components
- [ ] Workflow Engine Core (`workflow-engine-core`)
- [ ] CLI Tools (`workflow-engine-cli`)
- [ ] Node Library (`workflow-engine-nodes`)
- [ ] MCP Integration (`workflow-engine-mcp`)
- [ ] Database Layer (`workflow-engine-db`)

### Specific Changes
- Change 1: Description
- Change 2: Description
- Change 3: Description

## Technical Details

### Architecture Impact
- [ ] No architectural changes
- [ ] Minor architectural changes
- [ ] Major architectural changes (explain below)

### Database Changes
- [ ] No database changes
- [ ] Schema migration required
- [ ] New tables/indexes added
- [ ] Data migration required

### API Changes
- [ ] No API changes
- [ ] New endpoints added
- [ ] Existing endpoints modified
- [ ] Breaking API changes (document in breaking changes section)

### Dependencies
- [ ] No dependency changes
- [ ] New dependencies added
- [ ] Dependencies updated
- [ ] Dependencies removed

## Testing

### Test Coverage
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] End-to-end tests added/updated
- [ ] MCP protocol tests added/updated
- [ ] Load tests added/updated
- [ ] Chaos tests added/updated

### Test Commands Run
```bash
# Standard tests
cargo test

# Integration tests (with MCP servers)
./scripts/start_test_servers.sh
cargo test -- --ignored

# Specific test suites
cargo test --test end_to_end_workflow_test -- --ignored
cargo test --test mcp_communication_test -- --ignored
cargo test --test workflow_external_tools_test -- --ignored

# Code quality
cargo fmt --check
cargo clippy -- -D warnings
```

### Manual Testing
- [ ] Manual testing completed
- [ ] API endpoints tested
- [ ] UI/UX tested (if applicable)
- [ ] External integrations tested
- [ ] Performance tested

## Performance Impact
- [ ] No performance impact
- [ ] Performance improvement
- [ ] Potential performance regression (explain below)

## Security Considerations
- [ ] No security impact
- [ ] Security improvement
- [ ] Potential security impact (explain below)

## Breaking Changes
If this PR contains breaking changes, list them here:
- Breaking change 1
- Breaking change 2

### Migration Guide
If breaking changes exist, provide migration steps:
1. Step 1
2. Step 2

## Documentation
- [ ] Code comments added/updated
- [ ] README updated
- [ ] API documentation updated
- [ ] CHANGELOG.md updated
- [ ] Migration guide created (if breaking changes)

## Deployment Notes
Any special deployment considerations:
- [ ] Requires database migration
- [ ] Requires environment variable changes
- [ ] Requires dependency updates
- [ ] Requires service restart
- [ ] No special deployment requirements

## Checklist
- [ ] My code follows the project's style guidelines
- [ ] I have performed a self-review of my own code
- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] I have made corresponding changes to the documentation
- [ ] My changes generate no new warnings or errors
- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] New and existing unit tests pass locally with my changes
- [ ] Any dependent changes have been merged and published

## Screenshots/Examples
If applicable, add screenshots or code examples to help explain your changes.

```rust
// Example of new API usage
```

## Additional Notes
Any additional information that reviewers should know:
- Special testing instructions
- Known limitations
- Future work planned
- Rollback procedures