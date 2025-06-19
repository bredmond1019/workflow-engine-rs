# Agent Tasks: Architecture Cleanup Agent

## Agent Role

**Primary Focus:** Remove external service dependencies and assess AI features to simplify the codebase for open source release.

## Key Responsibilities

- Remove unnecessary external MCP client implementations (Slack, Notion, HelpScout)
- Clean up dependencies and documentation related to external services
- Assess AI features for inclusion in initial open source release
- Simplify codebase architecture by removing non-essential integrations
- Update documentation to reflect architectural changes

## Assigned Tasks

### From Original Task List

- [ ] **2.0 Remove External MCP Client Dependencies** - [Originally task 2.0 from main list]
  - [ ] **2.1 Remove Slack MCP client implementation and dependencies** - [Originally task 2.1 from main list]
    - [ ] 2.1.1 Delete `crates/workflow-engine-mcp/src/clients/slack/` directory
    - [ ] 2.1.2 Remove Slack-related dependencies from MCP Cargo.toml
    - [ ] 2.1.3 Update MCP lib.rs exports to remove Slack client references
    - [ ] 2.1.4 Remove Slack client tests and documentation
  - [ ] **2.2 Remove Notion MCP client implementation and dependencies** - [Originally task 2.2 from main list]
    - [ ] 2.2.1 Delete `crates/workflow-engine-mcp/src/clients/notion/` directory
    - [ ] 2.2.2 Remove Notion-related dependencies from MCP Cargo.toml
    - [ ] 2.2.3 Update MCP lib.rs exports to remove Notion client references
    - [ ] 2.2.4 Remove Notion client tests and documentation
  - [ ] **2.3 Remove HelpScout MCP client implementation and dependencies** - [Originally task 2.3 from main list]
    - [ ] 2.3.1 Delete `crates/workflow-engine-mcp/src/clients/helpscout/` directory
    - [ ] 2.3.2 Remove HelpScout-related dependencies from MCP Cargo.toml
    - [ ] 2.3.3 Update MCP lib.rs exports to remove HelpScout client references
    - [ ] 2.3.4 Remove HelpScout client tests and documentation
  - [ ] **2.4 Update documentation to reflect removal of external MCP clients** - [Originally task 2.4 from main list]
    - [ ] 2.4.1 Update README.md to remove references to external service integrations
    - [ ] 2.4.2 Update API documentation to reflect available MCP capabilities
    - [ ] 2.4.3 Update example workflows to use only internal MCP features
    - [ ] 2.4.4 Update CHANGELOG.md to document breaking changes

- [ ] **8.0 Assess and Clean Up AI Features for Release** - [Originally task 8.0 from main list]
  - [ ] **8.1 Assess if WebSocket AI streaming is needed for initial release** - [Originally task 8.1 from main list]
    - [ ] 8.1.1 Evaluate WebSocket streaming feature completeness
    - [ ] 8.1.2 Determine if streaming adds significant value for v1.0
    - [ ] 8.1.3 Document streaming feature as experimental or stable
    - [ ] 8.1.4 Either complete implementation or mark as roadmap item
  - [ ] **8.2 Evaluate Gemini and Ollama provider implementations for v1.0** - [Originally task 8.2 from main list]
    - [ ] 8.2.1 Assess current implementation status of Gemini provider
    - [ ] 8.2.2 Assess current implementation status of Ollama provider
    - [ ] 8.2.3 Determine effort required to complete implementations
    - [ ] 8.2.4 Either complete or remove incomplete provider implementations
  - [ ] **8.3 Document which AI features are included vs roadmap items** - [Originally task 8.3 from main list]
    - [ ] 8.3.1 Create clear feature matrix of included AI capabilities
    - [ ] 8.3.2 Document roadmap items for future AI feature development
    - [ ] 8.3.3 Update README to clearly distinguish current vs planned features
    - [ ] 8.3.4 Add contribution guidelines for AI feature development

## Relevant Files

### External MCP Client Removal
- `crates/workflow-engine-mcp/src/clients/slack/` - Slack MCP client implementation to remove
- `crates/workflow-engine-mcp/src/clients/notion/` - Notion MCP client implementation to remove
- `crates/workflow-engine-mcp/src/clients/helpscout/` - HelpScout MCP client implementation to remove
- `crates/workflow-engine-mcp/src/lib.rs` - MCP library exports to update after client removal
- `crates/workflow-engine-mcp/Cargo.toml` - Dependencies to clean up after client removal

### AI Feature Assessment
- `crates/workflow-engine-core/src/ai/` - AI provider implementations and streaming features
- `crates/workflow-engine-core/src/streaming/` - WebSocket streaming implementation
- `crates/workflow-engine-core/src/nodes/agent.rs` - AI agent node implementations
- `crates/workflow-engine-core/src/ai/tokens/` - Token management and provider integrations

### Documentation Updates
- `README.md` - Main documentation requiring updates to reflect architectural changes
- `CHANGELOG.md` - Version history requiring breaking change documentation
- `examples/` - Code examples requiring updates to remove external service references
- `docs/` - API and feature documentation requiring updates

## Dependencies

### Prerequisites (What this agent needs before starting)
- **From Build & Infrastructure Agent:** Basic compilation working to safely remove dependencies
- **Optional:** Can start external client removal immediately, but should coordinate with Agent 1 on dependency changes

### Provides to Others (What this agent delivers)
- **To Quality & Documentation Agent:** Simplified codebase with clear feature boundaries for documentation
- **To Core Features Agent:** Clean MCP architecture for pricing engine integration
- **To All Agents:** Reduced complexity and clearer scope for open source release

## Handoff Points

- **After Task 2.1-2.3:** Notify Build & Infrastructure Agent of dependency changes for compilation verification
- **After Task 2.4:** Notify Quality & Documentation Agent that architectural documentation updates are complete
- **After Task 8.3:** Notify Quality & Documentation Agent that AI feature matrix is ready for final documentation
- **Before Task 8.2.4:** Coordinate with Core Features Agent if AI provider changes affect pricing engine

## Testing Responsibilities

- Verify MCP crate still compiles after client removal: `cargo build -p workflow-engine-mcp`
- Test that remaining MCP functionality works correctly
- Validate that removed external references don't break existing workflows
- Ensure AI feature assessment doesn't break core AI functionality

## Implementation Priority Order

1. **Start with Task 2.1-2.3** - Remove external clients (immediate simplification)
2. **Continue with Task 2.4** - Update documentation to reflect changes
3. **Follow with Task 8.1-8.2** - Assess AI features (requires deeper analysis)
4. **Finish with Task 8.3** - Document final AI feature decisions

## Critical Success Criteria

- [ ] **External MCP clients completely removed** (directories deleted, dependencies cleaned)
- [ ] **MCP crate still compiles and functions** after external client removal
- [ ] **Documentation accurately reflects new architecture** (no references to removed features)
- [ ] **AI feature scope clearly defined** (included vs roadmap features documented)
- [ ] **Breaking changes properly documented** in CHANGELOG.md

## Coordination Notes

- **With Build & Infrastructure Agent:** Coordinate dependency changes to avoid compilation conflicts
- **With Quality & Documentation Agent:** Provide updated architecture information for documentation
- **With Core Features Agent:** Ensure AI provider decisions don't conflict with pricing engine needs

## Notes

- Focus on **architectural simplification** - this is about making the open source release more manageable
- **Document all decisions** about what's included vs excluded for future reference
- **Preserve MCP infrastructure** while removing specific external service clients
- **Be decisive** about AI features - either complete them or move to roadmap
- **Update examples** to use only included features for better developer experience