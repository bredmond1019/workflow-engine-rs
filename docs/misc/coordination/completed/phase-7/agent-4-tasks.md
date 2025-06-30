# Agent Tasks: Documentation & DevOps Agent

## Agent Role

**Primary Focus:** Prepare publication infrastructure, establish community standards, set up quality gates, and execute the staged publication to crates.io.

## Key Responsibilities

- Create missing community files for professional open source project governance
- Verify crates.io publication readiness and metadata completeness
- Set up automated quality gates and CI/CD pipelines
- Execute staged publication process following dependency order

## Assigned Tasks

### From Original Task List

- [ ] 5.0 Prepare Publication Infrastructure and Community Standards - (Originally task 5.0 from main list)
  - [ ] 5.1 Create Missing Community Files - (Originally task 5.1 from main list)
    - [ ] 5.1.1 Create SECURITY.md with vulnerability reporting process and contact information
    - [ ] 5.1.2 Add CODE_OF_CONDUCT.md using Contributor Covenant template
    - [ ] 5.1.3 Consolidate GitHub issue templates to root .github/ISSUE_TEMPLATE/ directory
    - [ ] 5.1.4 Update CONTRIBUTING.md with open source development workflow
  - [ ] 5.2 Verify Crates.io Publication Readiness - (Originally task 5.2 from main list)
    - [ ] 5.2.1 Test `cargo publish --dry-run` for workflow-engine-core (should succeed first)
    - [ ] 5.2.2 Verify all crate metadata is complete (description, keywords, categories, repository)
    - [ ] 5.2.3 Ensure README files focus on crates.io installation rather than local development
    - [ ] 5.2.4 Plan staged publication order: core → mcp → nodes → api → app
  - [ ] 5.3 Set Up Quality Gates and CI/CD - (Originally task 5.3 from main list)
    - [ ] 5.3.1 Configure CI pipeline to run `cargo clippy -- -D warnings` as quality gate
    - [ ] 5.3.2 Add automated security scanning with `cargo audit` in CI
    - [ ] 5.3.3 Ensure documentation builds without errors in CI pipeline
    - [ ] 5.3.4 Set up automated dependency updates with security monitoring
  - [ ] 5.4 Final Publication Preparation - (Originally task 5.4 from main list)
    - [ ] 5.4.1 Update all README files with crates.io installation instructions
    - [ ] 5.4.2 Prepare release notes and changelog for initial open source publication
    - [ ] 5.4.3 Create GitHub release with proper versioning and release notes
    - [ ] 5.4.4 Execute staged publication to crates.io following dependency order

## Relevant Files

- `SECURITY.md` - Security policy for vulnerability reporting (missing)
- `CODE_OF_CONDUCT.md` - Community code of conduct (missing)
- `.github/ISSUE_TEMPLATE/` - GitHub issue templates (need consolidation)
- `.github/PULL_REQUEST_TEMPLATE.md` - PR template (needs update)
- `.github/workflows/*.yml` - CI/CD pipelines requiring quality gates
- `CONTRIBUTING.md` - Development guidelines (needs open source focus)
- `README.md` - Installation and usage instructions (needs crates.io focus)
- `crates/*/README.md` - Individual crate documentation (needs publication focus)
- `CHANGELOG.md` - Version history (needs initial release notes)
- `Cargo.toml` - Workspace metadata (verify completeness)
- `crates/*/Cargo.toml` - Individual crate metadata (verify publication readiness)

## Dependencies

### Prerequisites (What this agent needs before starting)

- **From Infrastructure Agent:** Clean security audit (Task 2.4) for CI/CD setup
- **From Code Quality Agent:** Passing tests and clippy for quality gate validation (Task 4.2, 4.4)
- **From Architecture Agent:** Complete APIs for publication readiness testing (Task 3.4)

### Provides to Others (What this agent delivers)

- **To All Agents:** Professional project infrastructure and governance
- **To Community:** Open source project ready for contribution and adoption
- **To Ecosystem:** Published crates available on crates.io

## Handoff Points

- **Before Task 5.2:** Wait for Code Quality Agent to complete clippy fixes (Task 4.2) and Architecture Agent to complete APIs (Task 3.4)
- **Before Task 5.3:** Wait for Infrastructure Agent to complete security audit (Task 2.4)
- **After Task 5.1:** Notify all agents that community standards are established
- **After Task 5.4:** Announce successful publication to all agents and stakeholders

## Testing Responsibilities

- Test `cargo publish --dry-run` for all crates in dependency order
- Verify all documentation builds correctly with `cargo doc`
- Test CI/CD pipeline with all quality gates
- Validate crates.io installation process

## Critical Success Criteria

- [ ] **Community Standards:** Professional governance files following open source best practices
- [ ] **Publication Readiness:** All crates pass `cargo publish --dry-run`
- [ ] **Quality Gates:** CI/CD enforces all quality standards automatically
- [ ] **Successful Publication:** All crates published to crates.io in correct order
- [ ] **Installation Verification:** Crates can be installed and used from crates.io

## Detailed Implementation Strategy

### 5.1 Community Files:
1. **SECURITY.md:** Include vulnerability reporting process, contact email, response timeline
2. **CODE_OF_CONDUCT.md:** Use Contributor Covenant 2.1 template with appropriate contact
3. **Issue templates:** Bug report, feature request, question templates in .github/ISSUE_TEMPLATE/
4. **CONTRIBUTING.md:** Development setup, testing, PR process, coding standards

### 5.2 Publication Readiness Verification:
```bash
# Staged dry-run testing
cd crates/workflow-engine-core && cargo publish --dry-run
cd crates/workflow-engine-mcp && cargo publish --dry-run
cd crates/workflow-engine-nodes && cargo publish --dry-run
cd crates/workflow-engine-api && cargo publish --dry-run
cd crates/workflow-engine-app && cargo publish --dry-run
```

### 5.3 CI/CD Quality Gates:
```yaml
# Key quality gates to implement
- name: Check formatting
  run: cargo fmt --all -- --check
  
- name: Run clippy
  run: cargo clippy --all-targets --all-features -- -D warnings
  
- name: Security audit
  run: cargo audit
  
- name: Test all features
  run: cargo test --all-features
  
- name: Documentation
  run: cargo doc --all-features --no-deps
```

### 5.4 Publication Process:
1. **Pre-publication:** Final verification, changelog update, version tags
2. **Stage 1:** Publish workflow-engine-core
3. **Stage 2:** Update dependencies, publish workflow-engine-mcp
4. **Stage 3:** Update dependencies, publish workflow-engine-nodes
5. **Stage 4:** Update dependencies, publish workflow-engine-api
6. **Stage 5:** Update dependencies, publish workflow-engine-app
7. **Post-publication:** GitHub release, announcement, documentation updates

## Publication Order & Dependencies

```
workflow-engine-core (no dependencies)
    ↓
workflow-engine-mcp (depends on core)
    ↓
workflow-engine-nodes (depends on core, mcp)
    ↓
workflow-engine-api (depends on core, mcp)
    ↓
workflow-engine-app (depends on all others)
```

## Notes

- **Documentation Focus:** README files should prioritize crates.io users over local development
- **Version Management:** Ensure all workspace dependencies use published versions after each stage
- **Community Building:** Set up infrastructure to welcome and support contributors from day one
- **Monitoring:** Track publication success and early adoption metrics