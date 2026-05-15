# Tasks: Structured Logging & Operation Tracing

**Input**: Design documents from `specs/003-structured-logging-tracing/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Format**: `[ID] [P?] [Story?] Description with file path`

- `[P]`: Can run in parallel (different files, no dependencies)
- `[Story]`: Which user story this task belongs to

## Phase 1: Setup

**Purpose**: Ensure working baseline before any changes

**⚠️ No Setup tasks needed** — project structure, dependencies are already in place. Skip to Phase 2.

---

## Phase 2: Foundation (Blocking Prerequisites)

**Purpose**: Logging infrastructure that ALL user stories depend on

- [X] T001 Add `OperationContext` struct to `crates/domain/src/op_context.rs` — fields: operation_id, operation_type, repo_id/pack_id/migration_run_id (Option), new() builder
- [X] T002 [P] Implement `redact_sensitive(input)` in `crates/domain/src/redact.rs` — regex-based token/api_key/bearer/secret redaction
- [X] T003 [P] Add `operation_events` table to DB schema in `crates/storage/src/lib.rs` + create `crates/storage/src/event_store.rs` with `insert_event()`, `list_events()`, `get_by_operation()`
- [X] T004 Initialize tracing subscriber with console + file output in `crates/tauri-bridge/src/lib.rs` — add tracing/tracing-subscriber/tracing-appender deps, set up daily rolling file appender to `~/.capability-repo-manager/logs/`
- [X] T005 [P] Add `log_level` field to `AppSettings` in `crates/tauri-bridge/src/state.rs` (default: "info")
- [X] T006 [P] Write tests for `OperationContext` in `crates/domain/tests/op_context_tests.rs`
- [X] T007 [P] Write tests for `redact_sensitive` in `crates/domain/tests/redact_tests.rs`

**Checkpoint**: Foundation ready — tracing is initialized, operation_events table exists, redaction works

---

## Phase 3: US1 — Scan Operation Traceability (P1) ⚠️ depends on Phase 2

**Goal**: All scan operations produce structured log entries with shared operation_id

**Independent Test**: Run a scan; grep log for the operation_id shows scan_started → repo_discovered → parse_finished → scan_finished

- [ ] T008 [P] [US1] Create OperationContext at scan_repositories command entry in `crates/tauri-bridge/src/commands/repo_commands.rs` — generate UUID, pass context to all sub-steps
- [ ] T009 [P] [US1] Add event logging to scan pipeline: scan_started (INFO, roots, max_depth), repo_discovered (DEBUG, path), repo_deduplicated (DEBUG, canonical_path), git_metadata_failed (WARN), capability_parse_started/finished (INFO/DEBUG), parse_failed (WARN), scan_finished (INFO, summary counts) — all with operation_id in `crates/tauri-bridge/src/commands/repo_commands.rs`
- [ ] T010 [US1] Record OperationEvent to DB after scan completes in `crates/tauri-bridge/src/commands/repo_commands.rs` — call event_store.insert_event() with scan summary

**Checkpoint**: US1 done — scan operations are fully traceable via logs

---

## Phase 4: US2 — Migration Lifecycle Traceability (P1) ⚠️ depends on Phase 2

**Goal**: Migration apply + rollback operations produce traceable logs with shared operation_id

**Independent Test**: Apply a migration then rollback; grep for operation_id shows plan → snapshot → execute → rollback lifecycle

- [X] T011 [P] [US2] Create OperationContext at build_migration_plan entry in `crates/tauri-bridge/src/commands/migration_commands.rs` — pass through plan, apply, rollback
- [X] T012 [P] [US2] Add event logging to migration plan/apply/rollback: migration_plan_started/finished (INFO), migration_apply_started (INFO), migration_snapshot_created (INFO, snapshot_dir), migration_copy_started/failed (DEBUG/ERROR), migration_apply_finished (INFO, status+counts), rollback_started/finished (INFO), rollback_restored/removed (DEBUG) in `crates/tauri-bridge/src/commands/migration_commands.rs`
- [X] T013 [US2] Record OperationEvent after each migration phase (plan/apply/rollback) in `crates/tauri-bridge/src/commands/migration_commands.rs`

**Checkpoint**: US2 done — migration operations are fully traceable

---

## Phase 5: US3 — Debug Bundle Export (P2) ⚠️ depends on Phase 2

**Goal**: User can export a .zip debug bundle from Settings containing logs, configuration, and summaries

**Independent Test**: Click "Export Debug Bundle" → choose path → .zip created with logs, settings, summaries

- [X] T014 [P] [US3] Implement `export_debug_bundle` command in `crates/tauri-bridge/src/commands/settings_commands.rs` — collect logs (last 7 days), redacted settings, recent operation events, recent doctor reports; assemble as .zip using zip crate
- [X] T015 [P] [US3] Add `Export Debug Bundle` button to `frontend/src/lib/pages/Settings.svelte` — calls export_debug_bundle, shows save dialog
- [X] T016 [US3] Add app_start event logging in `crates/tauri-bridge/src/lib.rs` — log version, platform, settings_path, db_path, log_dir

**Checkpoint**: US3 done — debug bundle is exportable

---

## Phase 6: US4 — Activity History + Settings UI (P3) ⚠️ depends on Phase 2

**Goal**: User can view recent operation history and configure log level from Settings

**Independent Test**: After scanning, open Activity page → see scan operation listed with status and summary

- [X] T017 [P] [US4] Add `list_operation_events` Tauri command in `crates/tauri-bridge/src/commands/repo_commands.rs` or dedicated commands file — query event_store.list_events with optional type filter
- [X] T018 [P] [US4] Create `frontend/src/lib/stores/eventStore.ts` — loadEvents, events writable, loading/error state
- [X] T019 [P] [US4] Create `frontend/src/lib/pages/Activity.svelte` — reverse-chronological list of operations, status badges, summary text
- [X] T020 [US4] Add `/activity` route in `frontend/src/App.svelte` — link from nav with label "Activity"
- [X] T021 [US4] Add log level selector (INFO/DEBUG/TRACE) in `frontend/src/lib/pages/Settings.svelte` — calls update_settings with new log_level
- [X] T022 [US4] Wire log level changes to dynamic subscriber update in `crates/tauri-bridge/src/state.rs` — reload tracing filter when log_level changes
- [X] T023 [US4] Add "Open Log Directory" button to Settings in `frontend/src/lib/pages/Settings.svelte` — opens file manager via Tauri shell open API

**Checkpoint**: US4 done — activity history visible and log level configurable

---

## Phase 7: Polish & Cross-Cutting

**Purpose**: Error enhancement, Pack/Doctor logging, app_start event, integration tests

- [X] T024 [P] Add pack export/delete logging: pack_export_started/finished (INFO), pack_delete_started/finished (INFO) in `crates/tauri-bridge/src/commands/pack_commands.rs`
- [X] T025 [P] Add doctor logging: doctor_started/finished (INFO, score, issue counts) in `crates/tauri-bridge/src/commands/doctor_commands.rs`
- [X] T026 [P] Add structured error logging for failed operations: include error code, operation_id, module name, entity IDs in `crates/tauri-bridge/src/commands/migration_commands.rs` and `crates/tauri-bridge/src/commands/repo_commands.rs`
- [X] T027 [P] Write integration test for operation_id propagation across scan pipeline in `crates/storage/tests/logging_integration_tests.rs`
- [X] T028 [P] Write integration test for debug bundle export in `crates/tauri-bridge/tests/debug_bundle_tests.rs`

**Checkpoint**: All flows instrumented, error context enhanced, integration tests green

---

## Dependencies & Execution Order

### Phase Dependencies

```
Phase 2 (Foundation)
    │
    ├────────────────┬────────────────┬────────────────┐
    ▼                ▼                ▼                ▼
Phase 3 (US1:   Phase 4 (US2:    Phase 5 (US3:    Phase 6 (US4:
 Scan Trace)    Migration       Debug Bundle     Activity+Settings)
                 Trace)                              │
    └────────────────┬────────────────────────────────┘
                     ▼
           Phase 7 (Polish & Cross-Cutting)
```

### Key Serial Chains (cannot parallelize)

| Chain | Reason |
|-------|--------|
| T001 → T008/T011/T014/T017 | OpContext must exist before flow instrumentation |
| T003 → T010/T013/T017 | event_store must exist before recording events |
| T004 → T009/T012/T024/T025 | tracing subscriber must exist before any logging |

### Parallel Opportunities

| Group | Tasks | Why |
|-------|-------|-----|
| Foundation | T002, T003, T005 | redact.rs, event_store, AppSettings touch different files |
| US1 parallel | T008, T009 | T008 creates context, T009 adds events — same file but sequential within story |
| US1+US2+US3+US4 | All US phases | Each touches different command files, can run in parallel after Phase 2 |
| Polish | T024, T025, T026 | Pack logging, Doctor logging, Error enhancement — separate command files |

### Implementation Strategy

**MVP First (Phases 2-4 only)**

1. **Phase 2**: Foundation (OpContext + redaction + event_store + tracing init)
2. **Phase 3**: US1 — Scan Traceability ✅ **first real value**
3. **Phase 4**: US2 — Migration Traceability ✅ **critical for debugging**
4. **STOP**: MVP achieved — core scan + migration observability is functional

### Full Implementation

5. **Phase 5** (parallel with Phase 6): US3 Debug Bundle + US4 Activity History
6. **Phase 7**: Polish integration tests and leftover flow logging

---

## Summary

| Phase | User Story | Priority | Tasks | Parallel |
|-------|-----------|----------|-------|----------|
| 2 | Foundation | Blocking | T001-T007 | ✅ 4 of 7 |
| 3 | US1: Scan Trace | P1 | T008-T010 | ✅ 2 of 3 |
| 4 | US2: Migration Trace | P1 | T011-T013 | ✅ 2 of 3 |
| 5 | US3: Debug Bundle | P2 | T014-T016 | ✅ 2 of 3 |
| 6 | US4: Activity + Settings | P3 | T017-T023 | ✅ 5 of 7 |
| 7 | Polish | — | T024-T028 | ✅ 4 of 5 |

**Total**: 28 tasks | **Parallelizable**: ~60% | **Serial chains**: 3 critical paths
