# AI Workflow Engine - Open Source Readiness Task List

**Generated from:** `tasks/project-open-source.md`  
**Date:** 2024-12-18  
**Target:** Prepare AI Workflow Engine for open source release with focus on core workflow functionality

## Relevant Files

### Core Compilation and Dependencies
- `crates/workflow-engine-api/Cargo.toml` - API crate dependencies and feature flags
- `crates/workflow-engine-api/src/lib.rs` - Main API library entry point requiring utoipa-swagger-ui fixes
- `crates/workflow-engine-nodes/Cargo.toml` - Nodes crate dependency resolution
- `Cargo.toml` - Root workspace configuration and dependency management
- `crates/workflow-engine-core/src/lib.rs` - Core library exports and conditional compilation

### External MCP Client Removal
- `crates/workflow-engine-mcp/src/clients/slack/` - Slack MCP client implementation to remove
- `crates/workflow-engine-mcp/src/clients/notion/` - Notion MCP client implementation to remove
- `crates/workflow-engine-mcp/src/clients/helpscout/` - HelpScout MCP client implementation to remove
- `crates/workflow-engine-mcp/src/lib.rs` - MCP library exports to update after client removal
- `crates/workflow-engine-mcp/Cargo.toml` - Dependencies to clean up after client removal

### Infrastructure and Deployment
- `docker-compose.yml` - Container orchestration configuration
- `README.md` - Main documentation requiring infrastructure alignment updates
- `scripts/` - MCP server infrastructure scripts
- `Dockerfile` - Container build configuration
- `.env.example` - Environment variable template

### Pricing Engine Implementation
- `crates/workflow-engine-core/src/ai/tokens/pricing.rs` - Main pricing engine implementation
- `crates/workflow-engine-core/src/ai/tokens/mod.rs` - Token management module exports
- `crates/workflow-engine-core/src/config/` - Configuration management for API keys and pricing settings

### Test Infrastructure
- `tests/` - Integration test directory requiring compilation fixes
- `crates/*/src/*/tests/` - Unit test modules throughout crates
- `scripts/test_setup.sh` - Test environment setup scripts
- `docker-compose.test.yml` - Test-specific container configuration

### Documentation Files
- `DEVELOPMENT_SETUP.md` - Development setup guide to create
- `QUICK_START.md` - Quick start guide to create  
- `monitoring/README.md` - Monitoring documentation to create
- `CHANGELOG.md` - Version history requiring sync with Cargo.toml
- `examples/` - Code examples directory requiring import path updates

### Benchmark and Performance
- `benches/` - Benchmark directory to create
- `scripts/benchmark.sh` - Benchmark execution scripts
- `docs/performance.md` - Performance documentation

### Notes

- Use `cargo build` to test compilation fixes incrementally
- Run `cargo test` to verify test suite functionality after fixes
- Use `docker-compose up` to test infrastructure alignment
- Test documentation examples with `cargo run --example <name>`
- Integration tests require `--ignored` flag: `cargo test -- --ignored`

## Tasks

- [ ] 1.0 Fix Critical Compilation Errors
  - [ ] 1.1 Resolve utoipa-swagger-ui dependency issues in API layer
    - [ ] 1.1.1 Analyze utoipa-swagger-ui version conflicts in workflow-engine-api
    - [ ] 1.1.2 Update Cargo.toml dependencies to compatible versions
    - [ ] 1.1.3 Fix import statements and feature flag usage for utoipa
    - [ ] 1.1.4 Test API compilation with `cargo build -p workflow-engine-api`
  - [ ] 1.2 Fix type mismatches and missing imports in workflow-engine-api
    - [ ] 1.2.1 Identify and catalog all type mismatch errors
    - [ ] 1.2.2 Update import paths for new workspace structure
    - [ ] 1.2.3 Fix trait implementations and generic type parameters
    - [ ] 1.2.4 Resolve async/await compatibility issues
  - [ ] 1.3 Resolve dependency conflicts in workflow-engine-nodes package
    - [ ] 1.3.1 Analyze circular dependency issues between crates
    - [ ] 1.3.2 Update dependency versions for compatibility
    - [ ] 1.3.3 Fix module export structure in workflow-engine-nodes
    - [ ] 1.3.4 Test nodes compilation independently
  - [ ] 1.4 Fix workspace dependency configuration issues
    - [ ] 1.4.1 Verify all workspace dependencies have correct version specifications
    - [ ] 1.4.2 Fix feature flag propagation across workspace crates
    - [ ] 1.4.3 Ensure consistent Rust edition across all crates
    - [ ] 1.4.4 Test full workspace compilation with `cargo build --workspace`

- [ ] 2.0 Remove External MCP Client Dependencies
  - [ ] 2.1 Remove Slack MCP client implementation and dependencies
    - [ ] 2.1.1 Delete `crates/workflow-engine-mcp/src/clients/slack/` directory
    - [ ] 2.1.2 Remove Slack-related dependencies from MCP Cargo.toml
    - [ ] 2.1.3 Update MCP lib.rs exports to remove Slack client references
    - [ ] 2.1.4 Remove Slack client tests and documentation
  - [ ] 2.2 Remove Notion MCP client implementation and dependencies
    - [ ] 2.2.1 Delete `crates/workflow-engine-mcp/src/clients/notion/` directory
    - [ ] 2.2.2 Remove Notion-related dependencies from MCP Cargo.toml
    - [ ] 2.2.3 Update MCP lib.rs exports to remove Notion client references
    - [ ] 2.2.4 Remove Notion client tests and documentation
  - [ ] 2.3 Remove HelpScout MCP client implementation and dependencies
    - [ ] 2.3.1 Delete `crates/workflow-engine-mcp/src/clients/helpscout/` directory
    - [ ] 2.3.2 Remove HelpScout-related dependencies from MCP Cargo.toml
    - [ ] 2.3.3 Update MCP lib.rs exports to remove HelpScout client references
    - [ ] 2.3.4 Remove HelpScout client tests and documentation
  - [ ] 2.4 Update documentation to reflect removal of external MCP clients
    - [ ] 2.4.1 Update README.md to remove references to external service integrations
    - [ ] 2.4.2 Update API documentation to reflect available MCP capabilities
    - [ ] 2.4.3 Update example workflows to use only internal MCP features
    - [ ] 2.4.4 Update CHANGELOG.md to document breaking changes

- [ ] 3.0 Align Infrastructure and Deployment Configuration
  - [ ] 3.1 Add microservices to docker-compose.yml or update README deployment info
    - [ ] 3.1.1 Assess whether microservices should be included in docker-compose.yml
    - [ ] 3.1.2 Either add service definitions for content_processing, knowledge_graph, and realtime_communication
    - [ ] 3.1.3 Or update README to clarify microservice deployment strategy
    - [ ] 3.1.4 Ensure port mappings match documentation (8082, 3002, 8081)
  - [ ] 3.2 Create missing MCP server infrastructure or update documentation
    - [ ] 3.2.1 Assess if Python MCP servers directory structure is needed
    - [ ] 3.2.2 Either create `mcp-servers/` directory with Python implementations
    - [ ] 3.2.3 Or update README to reflect actual MCP server architecture using scripts
    - [ ] 3.2.4 Ensure MCP server startup scripts work as documented
  - [ ] 3.3 Align docker-compose services with README claims
    - [ ] 3.3.1 Verify all services mentioned in README exist in docker-compose.yml
    - [ ] 3.3.2 Check port mappings match README documentation
    - [ ] 3.3.3 Ensure environment variable configuration is consistent
    - [ ] 3.3.4 Test full docker-compose stack startup and connectivity

- [ ] 4.0 Complete Production Pricing Engine Implementation
  - [ ] 4.1 Implement live API pricing updates for OpenAI
    - [ ] 4.1.1 Create HTTP client for OpenAI pricing API endpoints
    - [ ] 4.1.2 Implement pricing data parsing and validation
    - [ ] 4.1.3 Add error handling and fallback to cached pricing
    - [ ] 4.1.4 Create scheduled update mechanism for pricing data
  - [ ] 4.2 Implement live API pricing updates for Anthropic
    - [ ] 4.2.1 Create HTTP client for Anthropic pricing API endpoints
    - [ ] 4.2.2 Implement pricing data parsing and validation for Claude models
    - [ ] 4.2.3 Add error handling and fallback mechanisms
    - [ ] 4.2.4 Integrate with existing token counting infrastructure
  - [ ] 4.3 Implement live API pricing updates for AWS Bedrock
    - [ ] 4.3.1 Create AWS SDK integration for Bedrock pricing
    - [ ] 4.3.2 Implement multi-region pricing data collection
    - [ ] 4.3.3 Add IAM role and credential management
    - [ ] 4.3.4 Create pricing aggregation for different Bedrock models
  - [ ] 4.4 Add configuration for pricing update frequency and fallback handling
    - [ ] 4.4.1 Create configuration structure for update intervals
    - [ ] 4.4.2 Implement background task scheduler for pricing updates
    - [ ] 4.4.3 Add monitoring and alerting for pricing update failures
    - [ ] 4.4.4 Create comprehensive fallback chain from API → cache → hardcoded

- [ ] 5.0 Fix and Optimize Test Suite
  - [ ] 5.1 Fix test compilation issues (utoipa-swagger-ui and dependency problems)
    - [ ] 5.1.1 Resolve test-specific dependency conflicts
    - [ ] 5.1.2 Fix test import paths for new workspace structure
    - [ ] 5.1.3 Update test configurations and feature flags
    - [ ] 5.1.4 Ensure all test modules compile with `cargo test --no-run`
  - [ ] 5.2 Create test configuration that works without external services
    - [ ] 5.2.1 Implement in-memory database alternatives for unit tests
    - [ ] 5.2.2 Create mock implementations for external service dependencies
    - [ ] 5.2.3 Add test feature flags to disable external integrations
    - [ ] 5.2.4 Configure test environments with minimal infrastructure requirements
  - [ ] 5.3 Document which tests require external infrastructure with setup instructions
    - [ ] 5.3.1 Catalog all integration tests and their infrastructure dependencies
    - [ ] 5.3.2 Create test infrastructure setup documentation
    - [ ] 5.3.3 Provide Docker-based test environment configurations
    - [ ] 5.3.4 Document test categories and execution strategies
  - [ ] 5.4 Add comprehensive API endpoint tests to fill coverage gaps
    - [ ] 5.4.1 Create integration tests for health endpoints
    - [ ] 5.4.2 Add authentication flow testing
    - [ ] 5.4.3 Implement workflow API endpoint tests
    - [ ] 5.4.4 Add metrics and monitoring endpoint tests
  - [ ] 5.5 Fix the 134 ignored tests by providing proper test infrastructure setup
    - [ ] 5.5.1 Analyze each ignored test and its infrastructure requirements
    - [ ] 5.5.2 Create test-specific infrastructure provisioning
    - [ ] 5.5.3 Implement test cleanup and isolation mechanisms
    - [ ] 5.5.4 Update test execution scripts to handle infrastructure dependencies

- [ ] 6.0 Fix Documentation Issues and Broken Links
  - [ ] 6.1 Create missing documentation files (DEVELOPMENT_SETUP.md, QUICK_START.md, monitoring/README.md)
    - [ ] 6.1.1 Create comprehensive DEVELOPMENT_SETUP.md with prerequisites and setup steps
    - [ ] 6.1.2 Write QUICK_START.md with minimal example workflows
    - [ ] 6.1.3 Create monitoring/README.md documenting metrics and observability
    - [ ] 6.1.4 Add API documentation with OpenAPI/Swagger integration
  - [ ] 6.2 Sync version numbers between Cargo.toml (0.6.0) and CHANGELOG.md (0.5.0)
    - [ ] 6.2.1 Decide on correct version number for open source release
    - [ ] 6.2.2 Update CHANGELOG.md with v0.6.0 release notes
    - [ ] 6.2.3 Ensure all workspace crates use consistent versioning
    - [ ] 6.2.4 Update release documentation and tagging procedures
  - [ ] 6.3 Update README code examples to use correct import paths for workspace structure
    - [ ] 6.3.1 Audit all code examples in README.md for accuracy
    - [ ] 6.3.2 Update import statements to reflect workspace crate structure
    - [ ] 6.3.3 Test all README examples for compilation and execution
    - [ ] 6.3.4 Add example projects in `examples/` directory
  - [ ] 6.4 Fix broken documentation links throughout README
    - [ ] 6.4.1 Audit all internal and external links in README.md
    - [ ] 6.4.2 Create missing referenced documentation files
    - [ ] 6.4.3 Update file paths to match actual project structure
    - [ ] 6.4.4 Add link validation to CI/CD pipeline

- [ ] 7.0 Implement Performance Benchmark Validation
  - [ ] 7.1 Create benchmarking framework to validate "15,000+ requests/second" claim
    - [ ] 7.1.1 Set up Criterion.rs benchmarking infrastructure
    - [ ] 7.1.2 Create HTTP API load testing scenarios
    - [ ] 7.1.3 Implement concurrent request handling benchmarks
    - [ ] 7.1.4 Document benchmark methodology and environment requirements
  - [ ] 7.2 Create benchmarks for "sub-millisecond node processing" claim
    - [ ] 7.2.1 Implement workflow node execution timing benchmarks
    - [ ] 7.2.2 Create micro-benchmarks for individual node types
    - [ ] 7.2.3 Add memory usage profiling for node operations
    - [ ] 7.2.4 Benchmark parallel vs sequential node execution performance
  - [ ] 7.3 Document benchmark setup and results in README
    - [ ] 7.3.1 Create benchmark execution documentation
    - [ ] 7.3.2 Add performance results to README with methodology notes
    - [ ] 7.3.3 Include hardware specifications for benchmark results
    - [ ] 7.3.4 Set up automated benchmark regression testing

- [ ] 8.0 Assess and Clean Up AI Features for Release
  - [ ] 8.1 Assess if WebSocket AI streaming is needed for initial release
    - [ ] 8.1.1 Evaluate WebSocket streaming feature completeness
    - [ ] 8.1.2 Determine if streaming adds significant value for v1.0
    - [ ] 8.1.3 Document streaming feature as experimental or stable
    - [ ] 8.1.4 Either complete implementation or mark as roadmap item
  - [ ] 8.2 Evaluate Gemini and Ollama provider implementations for v1.0
    - [ ] 8.2.1 Assess current implementation status of Gemini provider
    - [ ] 8.2.2 Assess current implementation status of Ollama provider
    - [ ] 8.2.3 Determine effort required to complete implementations
    - [ ] 8.2.4 Either complete or remove incomplete provider implementations
  - [ ] 8.3 Document which AI features are included vs roadmap items
    - [ ] 8.3.1 Create clear feature matrix of included AI capabilities
    - [ ] 8.3.2 Document roadmap items for future AI feature development
    - [ ] 8.3.3 Update README to clearly distinguish current vs planned features
    - [ ] 8.3.4 Add contribution guidelines for AI feature development

- [ ] 9.0 Finalize Demo Workflow Documentation
  - [ ] 9.1 Document customer support workflow as intentional demo/example
    - [ ] 9.1.1 Create clear documentation explaining demo nature of customer support workflow
    - [ ] 9.1.2 Add examples of how to extend the demo for production use
    - [ ] 9.1.3 Document the rule-based implementations as educational examples
    - [ ] 9.1.4 Provide guidance on implementing AI-powered alternatives
  - [ ] 9.2 Ensure customer support demo works reliably with rule-based implementations
    - [ ] 9.2.1 Test all customer support workflow paths end-to-end
    - [ ] 9.2.2 Verify rule-based sentiment analysis produces reasonable results
    - [ ] 9.2.3 Ensure template-based response generation works correctly
    - [ ] 9.2.4 Add comprehensive test coverage for demo workflow
  - [ ] 9.3 Add clear examples and documentation for customer support workflow
    - [ ] 9.3.1 Create step-by-step tutorial for running customer support demo
    - [ ] 9.3.2 Add example input data and expected outputs
    - [ ] 9.3.3 Document how to customize and extend the demo workflow
    - [ ] 9.3.4 Create additional demo workflows showcasing different capabilities

## Implementation Notes

### Critical Path Dependencies
1. **Task 1** must be completed before any other development work can proceed
2. **Tasks 2 and 3** can be executed in parallel with Task 1
3. **Task 5** depends on completion of Task 1 for test compilation
4. **Tasks 6-9** can be executed in parallel once foundation tasks are complete

### Success Metrics
- All workspace crates compile without errors (`cargo build --workspace`)
- 90%+ of tests pass (`cargo test`)
- Docker stack starts successfully (`docker-compose up`)
- Documentation links work and examples compile
- Performance benchmarks validate README claims

### Testing Strategy
- Run `cargo build` after each compilation fix
- Use `cargo test --no-run` to verify test compilation
- Execute `cargo test -- --ignored` for integration tests
- Test docker setup with `docker-compose up --build`

This task list provides a comprehensive roadmap for preparing the AI Workflow Engine for open source release, focusing on core functionality while maintaining architectural integrity.