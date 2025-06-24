# Agent 2: Event Store Implementation Fix

You are Agent 2 responsible for fixing EventStore trait implementation issues in the workflow-engine-api crate. Complete these 4 tasks to fix compilation errors.

**Your focus areas:**
- EventStore trait implementations in test mocks
- EventMetadata initialization issues
- Method call fixes

**Key requirements:**
- Implement all missing trait methods
- Ensure mock implementations are suitable for testing
- Fix struct initialization with all required fields
- Maintain compatibility with existing tests

**Tasks:**

## 1. Fix MockEventStore in error_integration.rs
Location: `crates/workflow-engine-api/src/db/events/error_integration.rs:511`

### [ ] Implement missing EventStore methods
Missing methods:
- `get_events_for_aggregates`
- `cleanup_old_snapshots`
- `get_aggregate_ids_by_type`
- `optimize_storage`

Reference the EventStore trait definition to get correct signatures.

## 2. Fix MockEventStore in integration_tests.rs
Location: `crates/workflow-engine-api/src/db/events/tests/integration_tests.rs:322`

### [ ] Implement all 14 missing EventStore methods
Missing methods:
- `get_events`
- `get_events_from_version`
- `get_events_by_type`
- `get_events_by_correlation_id`
- `get_aggregate_version`
- `aggregate_exists`
- `save_snapshot`
- `get_snapshot`
- `get_current_position`
- `replay_events`
- `get_events_for_aggregates`
- `cleanup_old_snapshots`
- `get_aggregate_ids_by_type`
- `optimize_storage`

## 3. Fix EventMetadata initialization
Location: `crates/workflow-engine-api/src/api/routes/ordering.rs:518`

### [ ] Add missing fields to EventMetadata
Missing fields:
- `source`
- `tags`

Initialize with appropriate default values for tests.

## 4. Fix method call error
Location: `crates/workflow-engine-api/src/db/events/tests/integration_tests.rs:236`

### [ ] Fix unwrap() method issue
- Check what type is being unwrapped
- Ensure proper error handling or use appropriate method

**Success criteria:**
- Run `cargo test -p workflow-engine-api --no-run` and see successful compilation
- All EventStore trait implementations complete
- No missing field errors

**Dependencies:** None - you can work independently

**Testing commands:**
```bash
# Check compilation
cargo check -p workflow-engine-api

# Build tests without running
cargo test -p workflow-engine-api --no-run

# Test specific module
cargo test -p workflow-engine-api events:: --no-run
```

**Tips:**
- Look at the EventStore trait definition to understand required method signatures
- For mock implementations, simple stub implementations are fine (e.g., return Ok(vec![]) for list methods)
- Use Default::default() or reasonable defaults for missing fields
- Check existing mock implementations for patterns to follow

For each task:
- Find the trait definition to understand requirements
- Implement minimal working versions for tests
- Ensure all required fields are initialized
- Commit after each major fix