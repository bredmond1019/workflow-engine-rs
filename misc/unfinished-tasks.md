# Unfinished Tasks and Stubbed Functions

## Overview
This document provides a comprehensive list of unfinished tasks, stubbed functions, and incomplete implementations in the workflow-engine-rs project.

## Stubbed Functions and Incomplete Implementations

### 1. Registry API Endpoints
**File**: `crates/workflow-engine-api/src/api/routes/registry.rs`

#### Stubbed Endpoints:
- `register_agent` (line 59)
  - Status: Returns HTTP 501 Not Implemented
  - Required: PostgresAgentRegistry integration
  - Impact: Service discovery functionality

- `list_agents` (line 72)
  - Status: Returns HTTP 501 Not Implemented
  - Required: Database query implementation
  - Impact: Cannot list available agents

- `discover_agents` (line 86)
  - Status: Returns HTTP 501 Not Implemented
  - Required: Service discovery logic
  - Impact: Cannot auto-discover services

- `heartbeat_agent` (line 100)
  - Status: Returns HTTP 501 Not Implemented
  - Required: Health tracking implementation
  - Impact: Cannot monitor agent health

### 2. External MCP Node (Partial Stub)
**File**: `crates/workflow-engine-nodes/src/external_mcp.rs`
- Function: `process` (line 32)
- Status: Returns context unchanged
- Note: Full implementation exists in `external_mcp_client.rs`
- Action: Remove stub or update to use actual implementation

### 3. Workflow Builder MCP Methods
**File**: `crates/workflow-engine-core/src/workflow/builder.rs`

#### Removed Methods:
- MCP client methods (line 51)
- Comment: "use workflow-engine-mcp crate directly"
- Impact: Builder pattern incomplete for MCP

#### Stubbed Template Methods:
- `with_template_workflow` (line 71)
- `add_template_node` (line 83)
- `with_template_config` (line 95)
- Status: All return self without implementation
- Impact: Template workflow building not functional

### 4. Service Discovery
**File**: `crates/workflow-engine-api/src/lib.rs`
- ServiceContainer exports disabled (lines 62-64)
- Comment: "not yet implemented"
- Impact: No automatic service discovery

### 5. WASM Plugin Support
**File**: `services/content_processing/src/lib.rs`
- Commented out: `// pub mod plugins;` (line 13)
- Status: No WASM implementation
- Documentation claims this feature exists
- Impact: Cannot use WASM plugins for content processing

### 6. OpenAI Pricing API
**File**: `crates/workflow-engine-core/src/ai/openai_pricing.rs`
- Function: `fetch_pricing` (lines 67-77)
- Status: Returns hardcoded values
- Comment: "simulate an API response with current pricing data"
- Impact: Pricing may become outdated

## Incomplete Features by Component

### workflow-engine-api (85% complete)
- [ ] Registry endpoints implementation
- [ ] Service discovery bootstrap
- [ ] Complete ServiceContainer exports
- [ ] API route tests

### workflow-engine-core (95% complete)
- [ ] Template workflow builder methods
- [ ] Live pricing API integration
- [ ] Some MCP integration points

### workflow-engine-nodes (98% complete)
- [ ] Remove or fix external_mcp.rs stub
- [ ] Complete documentation

### Services (95% complete)
- [ ] WASM plugin implementation for content_processing
- [ ] Complete service discovery integration

## TODO Comments and Notes

### Connection Pool
**File**: `crates/workflow-engine-mcp/src/connection_pool.rs`
- Line 387: Comment about tracking response times
- Status: Feature enhancement, not critical

### MCP Stub Types
**File**: `crates/workflow-engine-core/src/models/mcp_stub.rs`
- Purpose: Avoid circular dependencies
- Status: Intentional design pattern, not incomplete

## Critical Path to Completion

### Must Complete (Blocking Release)
1. Fix failing tests in workflow-engine-mcp
2. Decide on registry endpoints (implement or remove)
3. Update documentation to match reality

### Should Complete (Important)
1. Implement basic registry endpoints
2. Complete template workflow builder
3. Add critical test coverage
4. Fix or remove stub implementations

### Nice to Have (Post-Release)
1. WASM plugin support
2. Live pricing APIs
3. Advanced service discovery
4. Response time tracking in connection pool

## Implementation Effort Estimates

### Quick Fixes (< 1 day)
- Remove external_mcp.rs stub
- Update documentation
- Fix failing tests

### Medium Effort (1-3 days)
- Basic registry endpoint implementation
- Template workflow builder completion
- Add critical test coverage

### Large Effort (1+ week)
- WASM plugin system
- Full service discovery
- Live pricing API integration

## Recommendations

### For Open Source Release
1. **Document limitations**: Clearly mark unimplemented features
2. **Remove dead code**: Delete unused stubs
3. **Add "not implemented" errors**: Better than silent failures
4. **Create issues**: GitHub issues for each incomplete feature
5. **Update README**: Mark features as "planned" vs "implemented"

### For Production Use
1. **Implement registry endpoints**: Critical for service management
2. **Add comprehensive tests**: Especially for API routes
3. **Complete template system**: Or remove if not needed
4. **Monitor pricing accuracy**: If cost tracking is important

## Conclusion

The project is approximately 90-95% complete with most core functionality fully implemented. The main gaps are in auxiliary features like service registry, WASM plugins, and some convenience methods. None of the incomplete items block the core workflow engine functionality, but they should be addressed or properly documented before a production release.