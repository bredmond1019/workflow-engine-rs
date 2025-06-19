# Task List for AI Workflow Engine v2 - Open Source & Crate Publication

**Generated from:** tasks/project-prd-v2.md  
**Generation Date:** 2024-12-18  
**Focus:** Open source readiness and crate.io publication preparation

## Relevant Files

- `Cargo.toml` - Root workspace configuration requiring metadata updates and restructuring
- `LICENSE` - MIT license file that needs to be created
- `CONTRIBUTING.md` - Contribution guidelines to be created
- `CODE_OF_CONDUCT.md` - Community code of conduct to be created
- `SECURITY.md` - Security vulnerability reporting guidelines to be created
- `.github/ISSUE_TEMPLATE/` - Issue templates directory to be created
- `.github/PULL_REQUEST_TEMPLATE.md` - PR template to be created
- `src/lib.rs` - Main library entry point requiring visibility and API improvements
- `src/core/nodes/mod.rs` - Node trait definition requiring async conversion
- `src/core/error/mod.rs` - Error handling improvements needed
- `.github/workflows/ci.yml` - CI/CD pipeline requiring comprehensive testing setup
- `CHANGELOG.md` - Change log to be created for version tracking

### Notes

- The workspace restructuring will affect most Cargo.toml files throughout the project
- Error handling improvements will touch many source files across the codebase
- API changes should maintain backward compatibility where possible
- All public APIs must have comprehensive rustdoc documentation

## Tasks

- [ ] 1.0 Prepare Open Source Infrastructure and Licensing
  - [ ] 1.1 Create and configure license file
    - [ ] 1.1.1 Download MIT license template
    - [ ] 1.1.2 Update copyright year and holder information
    - [ ] 1.1.3 Save as LICENSE in project root
  - [ ] 1.2 Update Cargo.toml with required metadata
    - [ ] 1.2.1 Change crate name from "backend" to "ai-workflow-engine"
    - [ ] 1.2.2 Update version to 0.5.0 to match git tags
    - [ ] 1.2.3 Change edition from "2024" to "2021"
    - [ ] 1.2.4 Add all required fields (authors, license, description, repository, homepage, documentation, keywords, categories)
    - [ ] 1.2.5 Add package.metadata.docs.rs configuration
  - [ ] 1.3 Create community files
    - [ ] 1.3.1 Write CONTRIBUTING.md with contribution guidelines
    - [ ] 1.3.2 Create CODE_OF_CONDUCT.md using Contributor Covenant
    - [ ] 1.3.3 Create SECURITY.md for vulnerability reporting
    - [ ] 1.3.4 Create CHANGELOG.md with initial version history
  - [ ] 1.4 Set up GitHub templates
    - [ ] 1.4.1 Create .github/ISSUE_TEMPLATE/bug_report.md
    - [ ] 1.4.2 Create .github/ISSUE_TEMPLATE/feature_request.md
    - [ ] 1.4.3 Create .github/PULL_REQUEST_TEMPLATE.md
  - [ ] 1.5 Update README.md
    - [ ] 1.5.1 Replace placeholder GitHub URLs with actual repository links
    - [ ] 1.5.2 Update badge URLs with correct repository information
    - [ ] 1.5.3 Add installation instructions for crate usage

- [ ] 2.0 Fix Critical Code Quality Issues
  - [ ] 2.1 Remove unwrap() and expect() from production code
    - [ ] 2.1.1 Fix unwrap() calls in src/workflows/knowledge_base_workflow.rs (8 instances)
    - [ ] 2.1.2 Fix unwrap() calls in src/monitoring/metrics.rs (13+ instances)
    - [ ] 2.1.3 Fix expect() calls in src/db/session.rs (database initialization)
    - [ ] 2.1.4 Fix expect() calls in src/api/events.rs (database operations)
    - [ ] 2.1.5 Replace remaining unwrap() calls with proper error handling
  - [ ] 2.2 Replace unsafe code with safe alternatives
    - [ ] 2.2.1 Replace unsafe global static in src/api/uptime.rs with OnceCell
    - [ ] 2.2.2 Fix unsafe env::set_var in src/main.rs
    - [ ] 2.2.3 Review and fix unsafe code in src/core/mcp/config.rs tests
  - [ ] 2.3 Remove panic!() from non-test code
    - [ ] 2.3.1 Fix panic!() in src/core/streaming/websocket.rs:506
    - [ ] 2.3.2 Replace unreachable!() in src/core/ai/tokens/counter.rs:160 with error handling
  - [ ] 2.4 Complete todo!() implementations
    - [ ] 2.4.1 Implement 4 todo!() tests in tests/event_sourcing_tests.rs
    - [ ] 2.4.2 Fix unimplemented!() in tests/service_isolation_test.rs:121
  - [ ] 2.5 Replace debug prints with proper logging
    - [ ] 2.5.1 Replace eprintln!() in src/core/nodes/external_mcp_client.rs:151 with log crate
    - [ ] 2.5.2 Remove #![allow(warnings)] from src/lib.rs

- [ ] 3.0 Restructure Project as Rust Workspace
  - [ ] 3.1 Create workspace structure
    - [ ] 3.1.1 Create root Cargo.toml with [workspace] configuration
    - [ ] 3.1.2 Move core functionality to workflow-engine-core crate
    - [ ] 3.1.3 Create workflow-engine-mcp crate for MCP protocol
    - [ ] 3.1.4 Create workflow-engine-api crate for REST API
    - [ ] 3.1.5 Create workflow-engine-nodes crate for built-in nodes
  - [ ] 3.2 Organize dependencies with feature flags
    - [ ] 3.2.1 Move database dependencies behind "database" feature
    - [ ] 3.2.2 Move monitoring dependencies behind "monitoring" feature
    - [ ] 3.2.3 Move AWS SDK dependencies behind "aws" feature
    - [ ] 3.2.4 Create minimal default feature set
  - [ ] 3.3 Update module visibility
    - [ ] 3.3.1 Mark internal modules with pub(crate)
    - [ ] 3.3.2 Create clear public API surface in lib.rs
    - [ ] 3.3.3 Document which modules are part of stable API
  - [ ] 3.4 Fix binary targets
    - [ ] 3.4.1 Properly declare demo binary in Cargo.toml
    - [ ] 3.4.2 Move application code to separate binary crate
  - [ ] 3.5 Configure examples
    - [ ] 3.5.1 Add [[example]] entries for all Rust examples
    - [ ] 3.5.2 Consider moving Python examples to separate directory

- [ ] 4.0 Improve API Design and Type Safety
  - [ ] 4.1 Convert Node trait to async
    - [ ] 4.1.1 Create AsyncNode trait with async process method
    - [ ] 4.1.2 Update all built-in nodes to implement AsyncNode
    - [ ] 4.1.3 Update workflow executor to handle async nodes
    - [ ] 4.1.4 Maintain backward compatibility with sync adapter
  - [ ] 4.2 Replace TypeId with type-safe alternatives
    - [ ] 4.2.1 Create NodeId<T> phantom type wrapper
    - [ ] 4.2.2 Update NodeConfig to use type-safe node references
    - [ ] 4.2.3 Implement type-safe workflow builder methods
  - [ ] 4.3 Improve error handling design
    - [ ] 4.3.1 Split WorkflowError into specific error types
    - [ ] 4.3.2 Add error context using anyhow or similar
    - [ ] 4.3.3 Implement proper error recovery strategies
  - [ ] 4.4 Enhance builder patterns
    - [ ] 4.4.1 Add compile-time validation to WorkflowBuilder
    - [ ] 4.4.2 Implement fluent API for node connections
    - [ ] 4.4.3 Add convenience methods for common patterns
  - [ ] 4.5 Add testing utilities
    - [ ] 4.5.1 Create mock_context() helper function
    - [ ] 4.5.2 Add assert_node_output() test helper
    - [ ] 4.5.3 Implement test fixtures for common scenarios

- [ ] 5.0 Enhance Documentation and Examples
  - [ ] 5.1 Add rustdoc comments to all public APIs
    - [ ] 5.1.1 Document all public structs in src/monitoring/correlation.rs
    - [ ] 5.1.2 Document all public structs in src/monitoring/metrics.rs
    - [ ] 5.1.3 Add module-level documentation to all public modules
    - [ ] 5.1.4 Include usage examples in documentation
  - [ ] 5.2 Create comprehensive examples
    - [ ] 5.2.1 Create basic hello-world workflow example
    - [ ] 5.2.2 Create async workflow with external API example
    - [ ] 5.2.3 Create custom node implementation example
    - [ ] 5.2.4 Create error handling best practices example
  - [ ] 5.3 Update getting started guide
    - [ ] 5.3.1 Add quick start section for crate users
    - [ ] 5.3.2 Document all available feature flags
    - [ ] 5.3.3 Add troubleshooting section
  - [ ] 5.4 Set up CI/CD pipeline
    - [ ] 5.4.1 Add cargo test to CI workflow
    - [ ] 5.4.2 Add cargo clippy with deny warnings
    - [ ] 5.4.3 Add cargo fmt check
    - [ ] 5.4.4 Add cargo audit for security scanning
    - [ ] 5.4.5 Add code coverage reporting
  - [ ] 5.5 Prepare for crate publication
    - [ ] 5.5.1 Run cargo publish --dry-run to verify
    - [ ] 5.5.2 Check crate size and exclude unnecessary files
    - [ ] 5.5.3 Verify documentation builds on docs.rs
    - [ ] 5.5.4 Test installation as dependency in new project