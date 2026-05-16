# Quickstart: Structured Logging & Operation Tracing

**Date**: 2026-05-15

## What You're Building

Add structured logging infrastructure to all key operations — scan, pack export, migration, doctor — with operation_id traceability, sensitive data redaction, configurable log levels, debug bundle export, and activity history UI.

## Implementation Order (Milestones)

### Milestone L1: Foundation
**Files**: `crates/domain/src/op_context.rs`, `crates/domain/src/redact.rs`, `crates/storage/src/event_store.rs`, `crates/storage/src/lib.rs`
- Task L1.1: Define OperationContext struct in domain crate
- Task L1.2: Implement redact_sensitive() utility in domain crate
- Task L1.3: Add operation_events table to DB schema + event_store module
- Task L1.4: Initialize tracing subscriber with file + console output

**Tests**: op_context unit tests, redaction tests, event_store tests

### Milestone L2: Per-Flow Instrumentation (can be parallelized)
**Files**: `crates/tauri-bridge/src/commands/*.rs`
- Task L2a: Instrument scan pipeline (scan_started/finished, repo_discovered, parse_failed)
- Task L2b: Instrument pack export/delete (export_started/finished, delete log)
- Task L2c: Instrument migration plan/apply/rollback (plan_snapshot_execute_status)
- Task L2d: Instrument doctor (doctor_started/finished, issue summary)
- Task L2e: Add error enhancement (structured error codes + context)

**Tests**: Integration tests for trace_id propagation in each flow

### Milestone L3: Debug Bundle + Activity History + Settings
**Files**: `crates/tauri-bridge/src/commands/settings_commands.rs`, `frontend/src/lib/pages/Activity.svelte`, `frontend/src/lib/pages/Settings.svelte`
- Task L3a: Implement debug bundle export command
- Task L3b: Add Activity History page + eventStore
- Task L3c: Add log level, Open Log Dir, Export Debug Bundle to Settings
- Task L3d: Wire log level changes to dynamic subscriber update

**Tests**: Debug bundle integration test, Activity page load test

## Test Strategy

```bash
cargo test -p domain          # OpContext + redaction tests
cargo test -p storage          # event_store tests
cargo test --workspace         # All Rust tests

cd frontend && pnpm test       # Frontend/component tests
```

## Key Files to Watch

| File | Purpose |
|------|---------|
| `crates/domain/src/op_context.rs` | **NEW**: OperationContext struct + builder |
| `crates/domain/src/redact.rs` | **NEW**: Sensitive data redaction function |
| `crates/storage/src/event_store.rs` | **NEW**: operation_events CRUD |
| `crates/tauri-bridge/src/lib.rs` | tracing subscriber initialization |
| `crates/tauri-bridge/src/state.rs` | Add log_level to AppSettings |
| `frontend/src/lib/pages/Activity.svelte` | **NEW**: Activity history page |
| `frontend/src/lib/stores/eventStore.ts` | **NEW**: Operation event store |
| `frontend/src/lib/pages/Settings.svelte` | Add log level selector + debug bundle |
