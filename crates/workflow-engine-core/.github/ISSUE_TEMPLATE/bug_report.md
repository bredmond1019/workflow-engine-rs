---
name: Bug Report
about: Create a report to help us improve the AI Workflow Engine
title: '[BUG] '
labels: ['bug']
assignees: ''
---

## Bug Description
A clear and concise description of what the bug is.

## To Reproduce
Steps to reproduce the behavior:
1. Go to '...'
2. Click on '....'
3. Scroll down to '....'
4. See error

## Expected Behavior
A clear and concise description of what you expected to happen.

## Actual Behavior
A clear and concise description of what actually happened.

## Environment Information
Please complete the following information:

**System:**
- OS: [e.g. Ubuntu 22.04, macOS 13.0, Windows 11]
- Rust Version: [e.g. 1.75.0]
- Cargo Version: [e.g. 1.75.0]

**AI Workflow Engine:**
- Version: [e.g. 0.5.0]
- Installation Method: [e.g. cargo install, docker, from source]
- Crate: [e.g. workflow-engine-core, workflow-engine-cli, workflow-engine-nodes]

**Dependencies:**
- PostgreSQL Version: [if applicable]
- Docker Version: [if using Docker]
- Python Version: [if using MCP servers]

## Configuration
Please provide relevant configuration (remove sensitive information):

```toml
# Cargo.toml workspace configuration
[workspace]
members = [...]

# Environment variables (redact sensitive values)
DATABASE_URL=postgresql://...
```

## Error Messages/Logs
If applicable, add error messages or log output:

```
[paste error messages here]
```

## Affected Components
Check all that apply:
- [ ] Workflow Engine Core
- [ ] HTTP API Server
- [ ] MCP Framework
- [ ] Database Layer
- [ ] External Service Integration
- [ ] Monitoring/Metrics
- [ ] CLI Tools
- [ ] Docker Configuration
- [ ] Documentation

## Severity
- [ ] Critical (system crash, data loss)
- [ ] High (major feature broken)
- [ ] Medium (feature partially broken)
- [ ] Low (minor issue, workaround available)

## Additional Context
Add any other context about the problem here, such as:
- Screenshots
- Related issues
- Attempted solutions
- Workarounds

## Checklist
- [ ] I have searched existing issues to ensure this is not a duplicate
- [ ] I have provided all relevant environment information
- [ ] I have included steps to reproduce the issue
- [ ] I have checked the documentation and FAQ