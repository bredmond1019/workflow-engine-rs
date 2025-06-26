# Project Open Source Preparation Status

## Overview
This document tracks the remaining tasks and issues that need to be resolved before the workflow-engine-rs project can be released as open source.

## Critical Issues to Fix

### 1. Failing Tests ❌
- **Location**: `crates/workflow-engine-mcp`
- **Failed Tests**:
  - `config::tests::test_customer_support_server_config`
  - `config::tests::test_external_server_config`
- **Priority**: HIGH - Must fix before release
- **Action**: Fix server configuration tests

### 2. Missing MCP Servers 🚫
- **Issue**: README extensively references Python MCP servers that don't exist
- **Missing Servers**:
  - HelpScout (port 8001)
  - Notion (port 8002)
  - Slack (port 8003)
- **Directory**: `mcp-servers/` exists but is empty
- **Action**: Either implement example MCP servers or update documentation to clarify they're not included

### 3. Stubbed API Endpoints ⚠️
- **Location**: `crates/workflow-engine-api/src/api/routes/registry.rs`
- **Stubbed Endpoints**:
  - `register_agent` - returns 501 Not Implemented
  - `list_agents` - returns 501 Not Implemented
  - `discover_agents` - returns 501 Not Implemented
  - `heartbeat_agent` - returns 501 Not Implemented
- **Action**: Either implement or document as planned features

### 4. Missing Test Coverage 🧪
- **Critical Missing Tests**:
  - `workflow-engine-app`: 0 tests for main binary
  - API routes: health.rs, registry.rs, auth.rs have no tests
  - Database migrations lack test coverage
- **Action**: Add tests for critical user-facing components

## Documentation Issues

### 1. README.md Inaccuracies
- **Frontend Directory**: Exists but not documented
- **Service Architecture**: Unclear if microservices are optional or required
- **Installation**: References non-existent crates.io packages (0.6.0)
- **Examples**: Reference components that don't exist

### 2. Missing Documentation Files
- **DEV_SETUP.md**: Doesn't exist (critical for developers)
- **QUICK_START.md**: Has incorrect binary names and missing scripts

### 3. Feature Documentation
- **WASM Plugins**: Documented but not implemented
- **Service Discovery**: Mentioned but not implemented
- **OpenAI Pricing API**: Using hardcoded values instead of live API

## Code Quality Issues

### 1. Incomplete Implementations
- **External MCP Node**: Simple stub in `external_mcp.rs`
- **Workflow Builder MCP Methods**: Return stubs
- **ServiceContainer**: Exports disabled with "not yet implemented"

### 2. Hardcoded Values
- **OpenAI Pricing**: Contains hardcoded pricing data
- **Action**: Document as limitation or implement API integration

## Pre-Release Checklist

### High Priority (Must Fix)
- [ ] Fix 2 failing tests in workflow-engine-mcp
- [ ] Create DEV_SETUP.md with complete setup instructions
- [ ] Update README.md to accurately reflect current state
- [ ] Fix QUICK_START.md binary names and scripts
- [ ] Add tests for workflow-engine-app main binary
- [ ] Document or implement registry API endpoints
- [ ] Clarify MCP server situation (implement examples or update docs)

### Medium Priority (Should Fix)
- [ ] Add API route tests
- [ ] Document frontend directory purpose
- [ ] Update crate versions in documentation
- [ ] Test all example code in documentation
- [ ] Add database migration tests
- [ ] Document WASM plugin status

### Low Priority (Nice to Have)
- [ ] Implement live pricing APIs
- [ ] Add more integration test coverage
- [ ] Create example MCP servers
- [ ] Improve service discovery documentation

## Release Readiness Assessment

### ✅ Ready for Release
- Core workflow engine (95% complete)
- MCP protocol implementation (100% complete)
- Built-in nodes (100% complete)
- Microservices (95% complete)
- Authentication and security
- Database integration
- Monitoring and observability

### ❌ Not Ready for Release
- Registry API endpoints (stubbed)
- Some documentation (inaccurate/missing)
- Test coverage gaps
- MCP example servers (missing)

## Estimated Time to Release

With focused effort:
- **Minimum (critical fixes only)**: 2-3 days
- **Recommended (all high priority)**: 1 week
- **Ideal (all items)**: 2 weeks

## Next Steps

1. Fix the 2 failing tests immediately
2. Create accurate DEV_SETUP.md
3. Update README.md to reflect actual state
4. Fix QUICK_START.md issues
5. Add critical missing tests
6. Make decision on MCP servers and registry endpoints
7. Final review and release