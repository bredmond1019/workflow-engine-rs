# Publication Checklist for Crates.io

This checklist guides you through publishing the workflow-engine crates to crates.io.

## Pre-Publication Checks

### 1. Code Quality
- [ ] All tests pass: `cargo test --workspace`
- [ ] No clippy warnings: `cargo clippy --workspace -- -D warnings`
- [ ] Code is formatted: `cargo fmt --all --check`
- [ ] Documentation builds: `cargo doc --workspace --no-deps`
- [ ] No security vulnerabilities: `cargo audit`

### 2. Version Management
- [ ] Version bumped in workspace Cargo.toml
- [ ] CHANGELOG.md updated with release notes
- [ ] README files updated with new version numbers
- [ ] All example code uses the new version

### 3. Metadata Verification
- [ ] All Cargo.toml files have complete metadata:
  - [ ] description
  - [ ] documentation
  - [ ] homepage
  - [ ] repository
  - [ ] keywords (max 5)
  - [ ] categories (max 5)
  - [ ] license
  - [ ] authors

### 4. Documentation
- [ ] All public APIs have rustdoc comments
- [ ] README.md files are up to date
- [ ] Examples compile and run
- [ ] QUICK_START.md is current

## Publication Order

**IMPORTANT**: Crates must be published in dependency order!

### Stage 1: Core Crate
```bash
cd crates/workflow-engine-core
cargo publish --dry-run
# Review output, then:
cargo publish
```
Wait for crates.io to index (usually 1-2 minutes)

### Stage 2: MCP Crate
```bash
cd crates/workflow-engine-mcp
cargo publish --dry-run
# Review output, then:
cargo publish
```
Wait for crates.io to index

### Stage 3: Nodes Crate
```bash
cd crates/workflow-engine-nodes
cargo publish --dry-run
# Review output, then:
cargo publish
```
Wait for crates.io to index

### Stage 4: API Crate
```bash
cd crates/workflow-engine-api
cargo publish --dry-run
# Review output, then:
cargo publish
```
Wait for crates.io to index

### Stage 5: App Crate
```bash
cd crates/workflow-engine-app
cargo publish --dry-run
# Review output, then:
cargo publish
```

## Post-Publication

### 1. Git Tags
```bash
git tag -a v0.6.0 -m "Release v0.6.0"
git push origin v0.6.0
```

### 2. GitHub Release
1. Go to https://github.com/bredmond1019/workflow-engine-rs/releases
2. Click "Draft a new release"
3. Select the tag v0.6.0
4. Title: "v0.6.0 - Production Ready Release"
5. Copy release notes from CHANGELOG.md
6. Add installation instructions
7. Publish release

### 3. Documentation
- [ ] Verify docs.rs builds for all crates
- [ ] Update any external documentation
- [ ] Tweet/announce the release (optional)

### 4. Monitoring
- [ ] Check crates.io for successful publication
- [ ] Monitor GitHub issues for any problems
- [ ] Watch download statistics

## Rollback Procedure

If issues are discovered after publication:

1. **Yank affected versions** (doesn't delete, just prevents new downloads):
   ```bash
   cargo yank --vers 0.6.0 --crate workflow-engine-core
   ```

2. **Fix issues** in a patch release (0.6.1)

3. **Re-publish** following the checklist

## Notes

- Each crate publication is permanent (can't be deleted)
- Yanking only prevents new projects from depending on that version
- Always do dry-run first to catch issues
- Allow time between publications for crates.io indexing
- Consider publishing during low-traffic times

## Troubleshooting

### "Package not found" during publication
- Wait 2-3 minutes for previous crate to be indexed
- Check https://crates.io/crates/[crate-name] to verify

### "Version already exists"
- Bump version in Cargo.toml
- Update all dependency references

### Documentation not building on docs.rs
- Check [package.metadata.docs.rs] in Cargo.toml
- Ensure all features can be compiled together