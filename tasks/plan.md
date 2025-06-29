# AI Workflow Engine - TDD Plan for Open Source Publication

## Project Design Summary

Production-ready AI workflow orchestration platform built in Rust with GraphQL Federation. The project is **95% ready for open source publication** but needs critical code quality improvements following TDD methodology.

**Current Status**: Federation backend working, 174+ frontend tests passing, but backend has compilation warnings and code quality issues blocking crates.io publication.

**Goal**: Apply Test-Driven Development to resolve remaining code quality issues and achieve professional open source standards.

## TDD Methodology

Following Kent Beck's TDD cycle:
1. **Red**: Write failing test for specific issue
2. **Green**: Implement minimal fix to make test pass  
3. **Refactor**: Apply "Tidy First" structural improvements

**Key Principles**:
- Separate structural changes from behavioral changes
- Commit after each green phase
- Write tests that verify the fix, not just pass
- Focus on one small issue at a time

## Current State Analysis

From tasks/completed/phase-7/tasks-list.md, **Tasks 1-3 are complete** (compilation errors fixed, security vulnerabilities resolved, missing implementations added).

**Remaining blockers for publication**:
- **Task 4**: 265+ unwrap() calls, 145+ clippy warnings
- **Task 5**: Missing community files, publication readiness

## TDD Test Plan

### Phase 1: Core Code Quality (Task 4.1-4.2)

#### Test 1: Error Handling Anti-patterns ✅ COMPLETED
**Target**: Replace unwrap() calls with proper error handling
**Test Strategy**: Write integration tests that trigger error conditions

```rust
// Test case examples:
- [x] 1a. Configuration loading with invalid JWT secret ✅
- [ ] 1b. Database connection failure handling  
- [ ] 1c. MCP server connection timeouts
- [ ] 1d. Workflow execution with missing dependencies
- [ ] 1e. Template parsing with malformed input
```

**TDD CYCLE COMPLETE** for 1a: 
- **RED**: ✅ Created failing tests for JWT validation (empty secret, weak secret, missing env var)
- **GREEN**: ✅ Implemented AppConfig module with proper error handling, replaced .expect() calls
- **REFACTOR**: ✅ Applied "Tidy First" - fixed test isolation issues, improved error types

**Impact**: Eliminated 1 critical .expect() call in main.rs, added proper JWT validation with 32-char minimum, graceful startup failure

#### Test 2: Clippy Warning Resolution  
**Target**: Fix 145+ clippy warnings systematically
**Test Strategy**: Category-based approach with targeted tests

```rust
// Test categories:  
- [ ] 2a. Unused imports (91+ warnings)
- [ ] 2b. Inefficient string operations  
- [ ] 2c. Manual Default implementations
- [ ] 2d. Struct initialization patterns
- [ ] 2e. Variable naming and mutability
```

#### Test 3: Input Validation
**Target**: Add comprehensive input validation for public APIs
**Test Strategy**: Boundary testing and fuzzing approaches

```rust
// Test areas:
- [ ] 3a. JWT token validation edge cases
- [ ] 3b. Workflow configuration validation
- [ ] 3c. MCP protocol message validation  
- [ ] 3d. Node parameter type safety
- [ ] 3e. GraphQL query validation
```

### Phase 2: Documentation and API Polish (Task 4.3-4.4)

#### Test 4: Documentation Completeness
**Target**: Ensure all public APIs have working examples
**Test Strategy**: doctests that compile and run

```rust
// Documentation tests:
- [ ] 4a. Core workflow API examples
- [ ] 4b. MCP client usage examples
- [ ] 4c. Node configuration examples  
- [ ] 4d. Error handling patterns
- [ ] 4e. Integration testing examples
```

#### Test 5: Test Coverage Completeness
**Target**: All public APIs have unit tests
**Test Strategy**: Coverage-driven testing

```rust
// Coverage areas:
- [ ] 5a. workflow-engine-core public APIs
- [ ] 5b. workflow-engine-mcp client/server  
- [ ] 5c. workflow-engine-nodes execution
- [ ] 5d. workflow-engine-api endpoints
- [ ] 5e. Error condition testing
```

### Phase 3: Publication Readiness (Task 5)

#### Test 6: Community Standards
**Target**: Verify community files meet standards
**Test Strategy**: Automated policy validation

```rust  
// Community compliance:
- [ ] 6a. SECURITY.md vulnerability reporting flow
- [ ] 6b. CODE_OF_CONDUCT.md enforcement guidelines
- [ ] 6c. GitHub templates consistency  
- [ ] 6d. CONTRIBUTING.md workflow validation
- [ ] 6e. License compliance verification  
```

#### Test 7: Publication Infrastructure
**Target**: Ensure clean crates.io publication
**Test Strategy**: Dry-run publication testing

```rust
// Publication validation:
- [ ] 7a. cargo publish --dry-run for each crate
- [ ] 7b. Metadata completeness verification
- [ ] 7c. Dependency resolution testing
- [ ] 7d. README installation instructions
- [ ] 7e. Staged publication workflow
```

## Implementation Guidelines

### TDD Workflow for Each Test
1. **Write failing test** that demonstrates the specific problem
2. **Run test** to confirm it fails for the right reason  
3. **Implement minimal fix** to make test pass
4. **Run all tests** to ensure no regressions
5. **Refactor** using "Tidy First" principles if needed
6. **Commit** behavioral changes separately from structural changes

### Priority Order
Start with **Test 1** (Error Handling) as it has the highest impact on code stability and publication readiness.

### Success Criteria
- All tests passing: `cargo test --workspace`
- Clean clippy: `cargo clippy --workspace -- -D warnings`  
- Clean audit: `cargo audit`
- Clean dry-run: `cargo publish --dry-run` for each crate

## Next Steps

The plan is ready. Starting with **Test 1a: Configuration loading with invalid JWT secret** as our first failing test to demonstrate TDD methodology.

---

*This plan follows Kent Beck's TDD principles and the current project status from phase-7 completion.*