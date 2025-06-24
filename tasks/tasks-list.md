# AI Workflow Engine - Open Source Publication Task List

**Generated from:** tasks/project-prd-v3.md  
**Reference:** tasks/iteration-analysis-20241218_120000.md  
**Date:** December 18, 2024  
**Focus:** Critical blockers for crates.io publication

## Relevant Files

### Core Implementation Files
- `crates/workflow-engine-app/src/main.rs` - Main application with compilation errors (missing JwtAuth::new, JwtMiddleware::new)
- `crates/workflow-engine-api/src/lib.rs` - API crate exports (workflows module disabled, missing re-exports)
- `crates/workflow-engine-api/src/auth.rs` - JWT authentication implementation (missing constructors)
- `crates/workflow-engine-api/src/middleware.rs` - JWT middleware implementation (missing constructors)
- `crates/workflow-engine-api/src/workflows.rs` - Workflows module (currently commented out)
- `crates/workflow-engine-core/src/lib.rs` - Core crate public API surface
- `crates/workflow-engine-core/src/errors.rs` - Error types needing proper chaining and context

### Security and Dependencies
- `Cargo.toml` - Workspace dependencies (protobuf, dotenv, proc-macro-error updates needed)
- `crates/*/Cargo.toml` - Individual crate dependencies and metadata
- `Cargo.lock` - Dependency resolution (needs update for security fixes)

### Code Quality Files  
- `crates/workflow-engine-core/src/**/*.rs` - Core implementation (265+ unwrap/expect instances)
- `crates/workflow-engine-api/src/**/*.rs` - API implementation (145 clippy warnings)
- `crates/workflow-engine-mcp/src/**/*.rs` - MCP implementation (stub implementations)
- `crates/workflow-engine-nodes/src/**/*.rs` - Node implementations (disabled AI agents)

### Documentation and Community
- `SECURITY.md` - Security policy for vulnerability reporting (missing)
- `CODE_OF_CONDUCT.md` - Community code of conduct (missing)
- `.github/ISSUE_TEMPLATE/` - GitHub issue templates (need consolidation)
- `.github/PULL_REQUEST_TEMPLATE.md` - PR template (needs update)
- `README.md` - Installation and usage instructions (needs crates.io focus)

### Testing and CI/CD
- `tests/**/*.rs` - Integration tests (need compilation fixes)
- `.github/workflows/*.yml` - CI/CD pipelines (quality gates for publication)
- `benches/**/*.rs` - Performance benchmarks (path configuration issues)

### Notes

- All Rust files must compile successfully before publication
- Security vulnerabilities must be resolved (cargo audit clean)
- Clippy warnings must be fixed with `-- -D warnings` flag
- All public APIs must have documentation with examples
- Community files are required for professional open source project
- Staged publication required due to workspace dependencies

## Tasks

- [ ] 1.0 Fix Critical Compilation Errors
  - [x] 1.1 Implement Missing JWT Authentication Methods
    - [x] 1.1.1 Add `JwtAuth::new(secret: String) -> Self` constructor in `crates/workflow-engine-api/src/auth.rs`
    - [x] 1.1.2 Add `JwtMiddleware::new(secret: String) -> Self` constructor in `crates/workflow-engine-api/src/middleware.rs`
    - [x] 1.1.3 Ensure both constructors properly initialize internal state and validate inputs
    - [x] 1.1.4 Add unit tests for both constructors with valid and invalid inputs
  - [x] 1.2 Re-enable and Fix Workflows Module
    - [x] 1.2.1 Uncomment `pub mod workflows;` in `crates/workflow-engine-api/src/lib.rs` line 53
    - [x] 1.2.2 Uncomment workflows module re-exports in `crates/workflow-engine-api/src/lib.rs` lines 52-53
    - [x] 1.2.3 Fix any compilation errors in the workflows module after re-enabling
    - [x] 1.2.4 Ensure workflows module exports are properly documented
  - [ ] 1.3 Remove Unsafe Code Blocks
    - [ ] 1.3.1 Replace unsafe environment variable setting in `crates/workflow-engine-app/src/main.rs` lines 16-27
    - [ ] 1.3.2 Implement proper error handling for SystemTime operations instead of unwrap()
    - [ ] 1.3.3 Use safe environment variable setting without unsafe block
    - [ ] 1.3.4 Add proper error propagation for startup configuration
  - [ ] 1.4 Fix Import Resolution Errors
    - [ ] 1.4.1 Fix missing `workflows` import in `crates/workflow-engine-app/src/main.rs` line 8
    - [ ] 1.4.2 Ensure all module imports resolve correctly across workspace
    - [ ] 1.4.3 Update any broken internal crate dependencies
    - [ ] 1.4.4 Verify `cargo check --workspace` passes without errors

- [ ] 2.0 Resolve Security Vulnerabilities
  - [ ] 2.1 Update Protobuf Dependency (RUSTSEC-2024-0437)
    - [ ] 2.1.1 Update protobuf dependency to >=3.7.2 in workspace Cargo.toml
    - [ ] 2.1.2 Update any transitive dependencies that pull in vulnerable protobuf versions
    - [ ] 2.1.3 Test that updated protobuf version works with existing code
    - [ ] 2.1.4 Run `cargo audit` to verify vulnerability is resolved
  - [ ] 2.2 Replace Deprecated dotenv Dependency (RUSTSEC-2021-0141)
    - [ ] 2.2.1 Replace `dotenv = "0.15.0"` with `dotenvy = "0.15"` in workspace dependencies
    - [ ] 2.2.2 Update all `use dotenv::` imports to `use dotenvy::`
    - [ ] 2.2.3 Update any dotenv method calls to dotenvy equivalents
    - [ ] 2.2.4 Test environment variable loading still works correctly
  - [ ] 2.3 Update proc-macro-error Chain (RUSTSEC-2024-0370)
    - [ ] 2.3.1 Identify which dependencies bring in proc-macro-error (likely utoipa chain)
    - [ ] 2.3.2 Update utoipa and related dependencies to latest versions
    - [ ] 2.3.3 Verify OpenAPI documentation generation still works
    - [ ] 2.3.4 Ensure no breaking changes in updated dependencies
  - [ ] 2.4 Comprehensive Security Audit
    - [ ] 2.4.1 Run `cargo audit` and ensure zero vulnerabilities reported
    - [ ] 2.4.2 Review all dependencies for maintenance status and security
    - [ ] 2.4.3 Set up automated security scanning in CI/CD pipeline
    - [ ] 2.4.4 Document security update process for future maintenance

- [ ] 3.0 Complete Missing Implementations and API Polish
  - [ ] 3.1 Remove Stub Implementations and TODO Comments
    - [ ] 3.1.1 Audit all files for TODO, FIXME, and unimplemented!() macros
    - [ ] 3.1.2 Complete or remove stub MCP client methods in workflow builders
    - [ ] 3.1.3 Implement missing bootstrap service functionality or remove references
    - [ ] 3.1.4 Remove placeholder implementations in AI agent nodes or disable features properly
  - [ ] 3.2 Fix API Naming Consistency (Rust API Guidelines)
    - [ ] 3.2.1 Standardize MCP vs Mcp naming throughout codebase (choose one pattern)
    - [ ] 3.2.2 Review all public struct and enum names for consistency with Rust conventions
    - [ ] 3.2.3 Ensure method names follow Rust naming guidelines (snake_case)
    - [ ] 3.2.4 Update documentation to reflect naming changes
  - [ ] 3.3 Implement Proper Error Types and Chaining
    - [ ] 3.3.1 Replace string-only error variants with structured error types
    - [ ] 3.3.2 Add `#[source]` attributes for proper error chaining using thiserror
    - [ ] 3.3.3 Provide context-rich error messages with actionable information
    - [ ] 3.3.4 Implement Display and Debug traits properly for all error types
  - [ ] 3.4 Add Builder Patterns for Complex Configuration
    - [ ] 3.4.1 Implement builder pattern for NodeConfig with proper validation
    - [ ] 3.4.2 Create builder for McpConfig with fluent interface
    - [ ] 3.4.3 Add builder for WorkflowBuilder with type-safe configuration
    - [ ] 3.4.4 Ensure all builders have proper error handling and validation

- [ ] 4.0 Establish Professional Code Quality Standards
  - [ ] 4.1 Eliminate Production Code Anti-patterns
    - [ ] 4.1.1 Replace all unwrap() calls with proper error handling (265+ instances)
    - [ ] 4.1.2 Replace expect() calls with context-appropriate error handling
    - [ ] 4.1.3 Remove or properly justify any panic!() calls in production code
    - [ ] 4.1.4 Add comprehensive input validation for all public APIs
  - [ ] 4.2 Fix All Clippy Warnings with Strict Settings
    - [ ] 4.2.1 Fix unused import warnings (91+ instances)
    - [ ] 4.2.2 Replace manual string operations with strip_prefix() and similar idiomatic methods
    - [ ] 4.2.3 Use #[derive] for Default implementations where possible
    - [ ] 4.2.4 Fix inefficient struct initialization patterns
  - [ ] 4.3 Comprehensive Documentation with Examples
    - [ ] 4.3.1 Add rustdoc comments to all public APIs with practical examples
    - [ ] 4.3.2 Ensure all code examples in documentation compile and run
    - [ ] 4.3.3 Add module-level documentation explaining core concepts
    - [ ] 4.3.4 Create comprehensive API usage guide with real-world scenarios
  - [ ] 4.4 Test Coverage and Quality Assurance
    - [ ] 4.4.1 Ensure all tests pass with `cargo test --workspace`
    - [ ] 4.4.2 Add unit tests for all public APIs and error conditions
    - [ ] 4.4.3 Fix integration tests that depend on external MCP servers
    - [ ] 4.4.4 Add documentation tests to verify examples work correctly

- [ ] 5.0 Prepare Publication Infrastructure and Community Standards
  - [ ] 5.1 Create Missing Community Files
    - [ ] 5.1.1 Create SECURITY.md with vulnerability reporting process and contact information
    - [ ] 5.1.2 Add CODE_OF_CONDUCT.md using Contributor Covenant template
    - [ ] 5.1.3 Consolidate GitHub issue templates to root .github/ISSUE_TEMPLATE/ directory
    - [ ] 5.1.4 Update CONTRIBUTING.md with open source development workflow
  - [ ] 5.2 Verify Crates.io Publication Readiness
    - [ ] 5.2.1 Test `cargo publish --dry-run` for workflow-engine-core (should succeed first)
    - [ ] 5.2.2 Verify all crate metadata is complete (description, keywords, categories, repository)
    - [ ] 5.2.3 Ensure README files focus on crates.io installation rather than local development
    - [ ] 5.2.4 Plan staged publication order: core → mcp → nodes → api → app
  - [ ] 5.3 Set Up Quality Gates and CI/CD
    - [ ] 5.3.1 Configure CI pipeline to run `cargo clippy -- -D warnings` as quality gate
    - [ ] 5.3.2 Add automated security scanning with `cargo audit` in CI
    - [ ] 5.3.3 Ensure documentation builds without errors in CI pipeline
    - [ ] 5.3.4 Set up automated dependency updates with security monitoring
  - [ ] 5.4 Final Publication Preparation
    - [ ] 5.4.1 Update all README files with crates.io installation instructions
    - [ ] 5.4.2 Prepare release notes and changelog for initial open source publication
    - [ ] 5.4.3 Create GitHub release with proper versioning and release notes
    - [ ] 5.4.4 Execute staged publication to crates.io following dependency order