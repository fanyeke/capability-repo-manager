# Tasks: Main Process Stabilization Loop

**Input**: Design documents from `specs/002-stabilization-loop/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Format**: `[ID] [P?] [Story] Description with file path`

- `[P]`: Can run in parallel (different files, no dependencies)
- `[Story]`: Which user story this task belongs to

## Phase 1: Setup

**Purpose**: Ensure working baseline before any changes

**⚠️ No Setup tasks needed** — project structure, dependencies, and CI are already in place. Skip to Phase 2.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: DB schema migration and domain type changes that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T001 Add `canonical_path`, `first_indexed_at`, `capability_index_status`, `last_capability_indexed_at`, `last_capability_error` fields to `Repository` domain struct in `crates/domain/src/repository.rs`
- [X] T002 [P] Run DB schema migration: `ALTER TABLE repositories ADD COLUMN` for all new fields + `CREATE INDEX idx_repos_canonical_path` in `crates/storage/src/lib.rs`
- [X] T003 [P] Add `begin_transaction()`, `commit()`, `rollback()` helper methods or expose `rusqlite::Transaction` scope guard in `crates/storage/src/lib.rs`

**Checkpoint**: Foundation ready — user story implementation can now begin

---

## Phase 3: US1 - Stable Repository Identity Across Re-scans (P1) ⚠️ SERIAL — blocks US2

**Goal**: Repo ID reused across scans via canonical path lookup

**Independent Test**: Scan same directory twice; verify repo IDs unchanged and resources still associated

- [X] T004 [P] [US1] Implement `get_by_canonical_path(path)` in `crates/storage/src/repo_store.rs`
- [X] T005 [P] [US1] Implement `upsert_by_path(repo)` — query by canonical_path first; if exists, update metadata and return existing ID; if not, insert with new UUID — in `crates/storage/src/repo_store.rs`
- [X] T006 [US1] Fix `RepoCatalog::scan()` in `crates/repo-scanner/src/lib.rs`: use `std::fs::canonicalize()` to resolve path, call `upsert_by_path` instead of always generating new UUID
- [X] T007 [US1] Update `scan_repositories` bridge command in `crates/tauri-bridge/src/commands/repo_commands.rs` to pass canonical path through the pipeline
- [X] T008 [P] [US1] Write/update `crates/repo-scanner/tests/scanner_tests.rs` to verify identity stability across repeat scans
- [X] T009 [P] [US1] Write/update `crates/storage/tests/repo_store_tests.rs` for `get_by_canonical_path` and `upsert_by_path`

**Checkpoint**: US1 done — repo IDs are now stable

---

## Phase 4: US2 - Scan Pipeline Auto-Populates Capability Inventory (P1) ⚠️ depends on US1

**Goal**: Scan automatically parses capabilities, stores resources in DB

**Independent Test**: Scan a directory with Claude Code config repos; verify capability counts appear immediately

- [X] T010 [P] [US2] Implement `replace_for_repo(repo_id, resources)` in `crates/storage/src/resource_store.rs` — atomic: BEGIN, DELETE old resources WHERE repo_id=?, INSERT new resources, COMMIT
- [X] T011 [P] [US2] Implement `update_index_status(id, status, error?)` in `crates/storage/src/repo_store.rs`
- [X] T012 [US2] Add capability parsing to scan pipeline in `crates/tauri-bridge/src/commands/repo_commands.rs`: after upserting repo, call `claude_parser::parse_repo()`, then `replace_for_repo()` on success, or `update_index_status(parse_failed)` on error
- [X] T013 [US2] Update `ScanResultOutput` in `crates/tauri-bridge/src/commands/repo_commands.rs` to include `repos_parsed` and `repos_parse_failed` fields
- [X] T014 [P] [US2] Write integration test in `crates/tauri-bridge/tests/` (or `tests/integration/`) for full scan -> capabilities pipeline
- [X] T015 [US2] Update `list_repositories` to show real capability_counts instead of relying on empty results, test in `crates/tauri-bridge/src/commands/repo_commands.rs`

**Checkpoint**: US2 done — initial scan now produces full capability data

---

## Phase 5: US3 - Refresh Error Resilience (P1) ⚠️ depends on US2

**Goal**: Failed refresh preserves old capability data

**Independent Test**: Index repo, corrupt config, refresh; verify old data visible with error indicator

- [X] T016 [US3] Modify `refresh_repository` in `crates/tauri-bridge/src/commands/repo_commands.rs`: on parse failure, call `update_index_status(parse_failed, error_msg)` but do NOT delete old resources; on success, call `replace_for_repo()` (which atomically replaces)
- [X] T017 [P] [US3] Add `capability_index_status` and `last_capability_error` to `RepositorySummary` output so frontend can show parse status
- [X] T018 [P] [US3] Write test in `crates/storage/tests/repo_store_tests.rs` for `update_index_status` behavior

**Checkpoint**: US3 done — refresh failures are now safe

---

## Phase 6: US4 - Reliable Resource Detail Query (P2) 🔀 PARALLEL with US5

**Goal**: Resource detail uses direct PK lookup, correct results

**Independent Test**: Click any capability resource; verify correct metadata returned

- [X] T019 [P] [US4] Implement `get_by_id(resource_id)` in `crates/storage/src/resource_store.rs`
- [X] T020 [US4] Fix `get_resource_detail` in `crates/tauri-bridge/src/commands/capability_commands.rs`: use `resource_store.get_by_id(resource_id)` instead of `get_by_repo("")` + in-memory filter
- [X] T021 [P] [US4] Write test for `get_by_id` in `crates/storage/tests/resource_store_tests.rs`

**Checkpoint**: US4 done — resource detail is now accurate

---

## Phase 7: US5 - Persistent Pack Library (P2) 🔀 PARALLEL with US4

**Goal**: Pack metadata persisted to SQLite; survives restart

**Independent Test**: Export a pack, simulate restart, verify pack still listable and detailed

- [X] T022 [P] [US5] Create `crates/storage/src/pack_store.rs` with: `insert_pack()`, `get_pack_by_id()`, `get_pack_by_name_version()`, `list_packs(filter)`, `delete_pack()`
- [X] T023 [US5] Register `pack_store` module in `crates/storage/src/lib.rs`
- [X] T024 [US5] Modify `export_capability_pack` in `crates/tauri-bridge/src/commands/pack_commands.rs`: after pack files written to disk, insert pack metadata into DB via `pack_store.insert_pack()`, and insert pack resources into `capability_resources` with `pack_id` set
- [X] T025 [US5] Modify `list_packs` in `crates/tauri-bridge/src/commands/pack_commands.rs`: query DB-backed pack_store instead of in-memory `PackStore`
- [X] T026 [US5] Modify `get_pack_detail` in `crates/tauri-bridge/src/commands/pack_commands.rs`: query DB-backed pack_store
- [X] T027 [US5] Fix `delete_pack` in `crates/tauri-bridge/src/commands/pack_commands.rs`: cascade-delete pack DB row + pack resources + disk directory
- [X] T028 [P] [US5] Write tests for `crates/storage/tests/resource_store_tests.rs`: `replace_for_pack`, `delete_by_pack`
- [X] T029 [P] [US5] Write tests for `crates/storage/src/pack_store.rs`

**Checkpoint**: US5 done — packs are now persistent across restarts

---

## Phase 8: US6 - Safe Migration with Snapshot and Rollback (P2) ⚠️ depends on US5

**Goal**: Strategy validation, scoped snapshot, state machine, per-file execution, rollback

**Independent Test**: Apply pack, verify snapshot created, rollback, verify only affected files restored

### Sub-phase 8a: Types & Store (serial internal)

- [X] T030 [P] [US6] Add `ConflictAction` enum (`Skip`, `Overwrite`, `Rename`, `Merge`) to `crates/domain/src/migration.rs` — replace free-string `action` field
- [X] T031 [P] [US6] Add `MigrationSnapshot` and `SnapshotItem` domain types to `crates/domain/src/migration.rs`
- [X] T032 [P] [US6] Create `crates/storage/src/migration_store.rs` with: `insert_run()`, `get_run()`, `update_status()`, `update_snapshot()`, `update_report()`, `list_by_repo()`
- [X] T033 [US6] Register `migration_store` module in `crates/storage/src/lib.rs`

### Sub-phase 8b: Snapshot & Execution (depends on 8a)

- [X] T034 [US6] Add `create_snapshot(affected_paths, target_dir, snapshot_dir)` in `crates/migration-engine/src/executor.rs` — backup each affected file, write JSON manifest
- [X] T035 [US6] Add `restore_snapshot(snapshot_dir, target_dir)` in `crates/migration-engine/src/rollback.rs` — restore only files in manifest; delete files that didn't previously exist
- [X] T036 [US6] Rework `apply_migration_plan` in `crates/tauri-bridge/src/commands/migration_commands.rs`:
  - Validate all `ConflictStrategy` actions against `ConflictAction` enum (reject invalid)
  - Verify all conflicts resolved (reject if unresolved)
  - State machine: `planned → ready → executing → success/partial_failure/failed`
  - Call `create_snapshot()` before any file writes
  - Per-file execution: track success/failure per item
  - Determine final status from aggregate results
- [X] T037 [US6] Fix `rollback_migration` in `crates/tauri-bridge/src/commands/migration_commands.rs`:
  - Accept status `success` or `partial_failure` only (reject others)
  - Call `restore_snapshot()` with scoped snapshot
  - Update status to `rolled_back`
- [X] T038 [US6] Prevent re-apply: check `status != 'planned'` before execution
- [X] T039 [P] [US6] Write tests in `crates/migration-engine/tests/`: snapshot_create, snapshot_restore, per_file_exec, rollback_restores_only_touched
- [X] T040 [P] [US6] Write integration test in `crates/tauri-bridge/tests/` (or `tests/integration/`): full plan→snapshot→execute→rollback cycle

**Checkpoint**: US6 done — migration is now safe and auditable

---

## Phase 9: US7 - Cascade Cleanup on Repository Removal (P2) 🔀 PARALLEL with US6

**Goal**: Removing a repo cleans up associated resources and doctor reports

**Independent Test**: Remove a repo with capabilities + doctor reports; verify all orphan data is gone

- [ ] T041 [US7] Implement `delete_cascade(id)` in `crates/storage/src/repo_store.rs`: transactional delete of repo + capability_resources + doctor_reports
- [ ] T042 [US7] Modify `remove_repository` in `crates/tauri-bridge/src/commands/repo_commands.rs`: use `delete_cascade()` instead of plain `delete()`
- [ ] T043 [P] [US7] Write test in `crates/storage/tests/repo_store_tests.rs` for cascade delete

**Checkpoint**: US7 done — repo removal no longer leaves orphans

---

## Phase 10: Settings Persistence & Chinese UI (P1) 🔀 FULLY PARALLEL — can run anytime

**Goal**: Settings survive restart; UI supports Chinese language

- [ ] T044 [P] Write settings to `~/.capability-repo-manager/settings.json` on `update_settings` call in `crates/tauri-bridge/src/commands/settings_commands.rs`
- [ ] T045 [P] Load settings from `~/.capability-repo-manager/settings.json` on `get_settings` call, falling back to defaults if file missing in `crates/tauri-bridge/src/commands/settings_commands.rs`
- [ ] T046 [P] Register `settings_commands` module in `crates/tauri-bridge/src/commands/mod.rs` and wire up in `frontend/src-tauri/src/lib.rs`
- [ ] T047 [P] Add `svelte-i18n` dependency to `frontend/package.json`
- [ ] T048 [P] Create `frontend/src/lib/i18n/en.json` and `frontend/src/lib/i18n/zh.json` with core UI strings
- [ ] T049 [P] Create `frontend/src/lib/i18n/index.ts` to initialize svelte-i18n with locale detection (navigator.language + fallback)
- [ ] T050 [P] Wrap user-facing strings in App.svelte and key pages/components with `$t('key')` calls

---

## Dependencies & Execution Order

### Phase Dependencies

```
Phase 2 (Foundational)
    │
    ▼
Phase 3 (US1: Repo Identity) — SERIAL block
    │
    ▼
Phase 4 (US2: Scan Pipeline) — depends on US1
    │
    ▼
Phase 5 (US3: Refresh Resilience) — depends on US2
    │
    ├────────────────────┬────────────────────┐
    ▼                    ▼                    │
Phase 6 (US4: Detail)  Phase 7 (US5: Pack)  │  ← PARALLEL
    │                    │                    │
    │                    ▼                    │
    │              Phase 8 (US6: Migration)   │  ← depends on US5
    │                    │                    │
    └────────────────────┼────────────────────┘
                         ▼
                  Phase 9 (US7: Cascade)
                         │
                         ▼
                  Phase 10 (Settings + Chinese UI) — fully parallel, any time
```

### Key Serial Chains (cannot parallelize)

| Chain | Reason |
|-------|--------|
| T001 → T002 → T003 → T004-T009 | Domain type → DB schema → store methods → command fix |
| T010 → T012 → T016 | replace_for_repo → scan pipeline wiring → refresh fix |
| T030 → T031 → T032 → T034 → T036 → T037 | Enum types → domain types → migration store → snapshot create → apply → rollback |

### Parallel Opportunities

| Group | Tasks | Why |
|-------|-------|-----|
| Foundational | T002, T003 | Schema migration + transaction helpers touch different files |
| US1 Store | T004, T005 | get_by_canonical_path + upsert_by_path touch same file but are independent methods |
| US5 Pack | T022, T028, T029 | pack_store module + tests can be written alongside changes to pack_commands.rs |
| US4 + US5 | All US4 tasks + All US5 tasks | Resource detail fix and Pack persistence touch completely different files |

### Parallel Execution Example: Phase 6 + Phase 7

```bash
# Track A: Resource Detail Fix (US4)
Task: "Implement get_by_id() in crates/storage/src/resource_store.rs"
Task: "Fix get_resource_detail command in capability_commands.rs"

# Track B: Pack Persistence (US5) — simultaneously
Task: "Create pack_store.rs module"
Task: "Modify export/delete/listing commands in pack_commands.rs"
```

### Parallel Execution Example: Phase 10

```bash
# Settings module (Rust backend)
Task: "Fix read/write settings to JSON file"
Task: "Register settings_commands module"

# Chinese UI (Frontend) — simultaneously
Task: "Add svelte-i18n dependency"
Task: "Create locale JSON files"
Task: "Wrap UI strings with $t()"
```

---

## Implementation Strategy

### MVP First (Phases 2-5 only)

1. **Phase 2**: Foundational (schema migration + domain types)
2. **Phase 3**: US1 — Stable Repo Identity ✅ **first real value**
3. **Phase 4**: US2 — Scan Pipeline Auto-Populate ✅ **scan now works end-to-end**
4. **Phase 5**: US3 — Refresh Error Resilience ✅ **data loss risk eliminated**
5. **STOP**: MVP achieved — core scan/index pipeline is stable

### Full Implementation

6. **Phase 6 + 7** (parallel): US4 Resource Detail + US5 Pack Persistence
7. **Phase 8**: US6 Safe Migration
8. **Phase 9**: US7 Cascade Cleanup
9. **Phase 10**: Settings + Chinese UI (anytime)

---

## Summary

| Phase | User Story | Priority | Tasks | Parallel |
|-------|-----------|----------|-------|----------|
| 2 | Foundational | Blocking | T001-T003 | ✅ 2 of 3 |
| 3 | US1: Stable Repo ID | P1 | T004-T009 | ✅ 2 of 6 |
| 4 | US2: Scan Pipeline | P1 | T010-T015 | ✅ 2 of 6 |
| 5 | US3: Refresh Resilience | P1 | T016-T018 | ✅ 1 of 3 |
| 6 | US4: Resource Detail | P2 | T019-T021 | ✅ 2 of 3 |
| 7 | US5: Pack Persistence | P2 | T022-T029 | ✅ 3 of 8 |
| 8 | US6: Safe Migration | P2 | T030-T040 | ❌ serial (11 tasks) |
| 9 | US7: Cascade Cleanup | P2 | T041-T043 | ✅ 1 of 3 |
| 10 | Settings + Chinese UI | P1 | T044-T050 | ✅ all 7 parallel |

**Total**: 50 tasks | **Parallelizable**: ~50% | **Serial chains**: 3 critical paths
