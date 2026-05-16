# Quickstart: Stabilization Loop

**Date**: 2026-05-15

## What You're Building

修复 12 个已知 Bug，打通从"仓库扫描 → 能力解析 → 资源查询 → Pack 持久化 → 安全迁移"的完整闭环。同时增加中文本地化 UI 支持。

## Development Setup

```bash
# 1. Switch to feature branch
git checkout 002-stabilization-loop

# 2. Build & verify current state
cargo check --workspace

# 3. Run existing tests as baseline
cargo test --workspace

# 4. Frontend dev
cd frontend
pnpm install
pnpm dev        # Vite dev on :1420

# Tauri dev (full app)
cargo tauri dev
```

## Implementation Order (Milestones)

### Milestone A: Stable Repository Index (P0)
**Files**: `crates/storage/src/repo_store.rs`, `crates/repo-scanner/src/lib.rs`, `crates/tauri-bridge/src/commands/repo_commands.rs`
- Task A1: Add canonical_path, index_status fields to Repository domain type & DB schema
- Task A2: Implement `upsert_by_path()`, `get_by_canonical_path()` in repo_store
- Task A3: Fix `RepoCatalog::scan()` to use canonical path lookup instead of new UUID
- Task A4: Add capability parsing to scan pipeline in repo_commands
- Task A5: Implement `replace_for_repo()` with parse error preservation
- Task A6: Implement `update_index_status()`

**Tests**: `tests/unit/repo_store_tests.rs`, `crates/repo-scanner/tests/scanner_tests.rs`

### Milestone B: Resource Detail & Pack Persistence (P0/P1)
**Files**: `crates/storage/src/resource_store.rs`, `crates/storage/src/pack_store.rs`, `crates/tauri-bridge/src/commands/capability_commands.rs`, `pack_commands.rs`
- Task B1: Add `get_by_id()` to resource_store
- Task B2: Fix `get_resource_detail` command to use PK lookup
- Task B3: Implement `storage::pack_store` module (DB-backed)
- Task B4: Modify `export_capability_pack` to persist to DB
- Task B5: Fix `delete_pack` to cascade-delete DB records
- Task B6: Fix `remove_repository` to cascade-delete resources & doctor reports

**Tests**: `tests/unit/resource_store_tests.rs`, `crates/pack-engine/tests/library_tests.rs`

### Milestone C: Safe Migration (P0)
**Files**: `crates/domain/src/migration.rs`, `crates/storage/src/migration_store.rs`, `crates/tauri-bridge/src/commands/migration_commands.rs`, `crates/migration-engine/src/executor.rs`
- Task C1: Add `ConflictAction` enum (restricted: skip/overwrite/rename/merge)
- Task C2: Add `MigrationSnapshot` / `SnapshotItem` domain types
- Task C3: Implement `storage::migration_store` module
- Task C4: Add snapshot creation to executor before writes
- Task C5: Implement per-file execution with per-item status reporting
- Task C6: Implement state machine validation (reject invalid transitions)
- Task C7: Fix rollback to use scoped snapshot (restore only touched files)
- Task C8: Validate strategies + unresolved conflicts in apply_migration_plan

**Tests**: `crates/migration-engine/tests/*`, integration tests for full apply→rollback

### Milestone D: Settings Persistence & Chinese UI (P1)
**Files**: `crates/tauri-bridge/src/commands/settings_commands.rs`, `frontend/src/lib/*`
- Task D1: Fix settings read/write to use ~/.capability-repo-manager/settings.json
- Task D2: Register settings_commands module in commands/mod.rs and lib.rs
- Task D3: Add Chinese language UI support (i18n framework)

## Test Strategy

```bash
# Run tests after each milestone
cargo test -p storage          # Storage layer tests
cargo test -p migration-engine # Migration engine tests
cargo test -p repo-scanner     # Scanner tests
cargo test --workspace         # All Rust tests

# Frontend tests
cd frontend && pnpm test
```

## Key Files to Watch

| File | Purpose |
|------|---------|
| `crates/storage/src/lib.rs` | DB schema migration (add new columns) |
| `crates/storage/src/repo_store.rs` | Repo identity + cascade delete |
| `crates/storage/src/resource_store.rs` | get_by_id, replace_for_repo/pack |
| `crates/storage/src/pack_store.rs` | **NEW**: DB-backed pack persistence |
| `crates/storage/src/migration_store.rs` | **NEW**: Migration run state machine |
| `crates/domain/src/migration.rs` | ConflictAction enum, MigrationSnapshot |
| `crates/domain/src/repository.rs` | New identity fields |
| `crates/tauri-bridge/src/commands/repo_commands.rs` | Scan pipeline overhaul |
| `crates/tauri-bridge/src/commands/migration_commands.rs` | Snapshot + state machine |
| `crates/tauri-bridge/src/commands/settings_commands.rs` | Settings persistence |
| `crates/tauri-bridge/src/commands/mod.rs` | Register settings_commands |
| `frontend/src-tauri/src/lib.rs` | Register all commands |

## Dependencies Map

```
Milestone A ──→ Milestone B ──→ Milestone C
     │                               │
     └── parallel ──→ Milestone D ───┘
                     (settings + i18n)
```

- **A1-A3** (repo identity) blocks everything — must be done first
- **A4-A6** (scan pipeline) and **B1-B2** (resource detail) can share some test setup
- **B3-B5** (pack persistence) is independent from scan changes
- **C1-C8** (migration safety) depends on no structural changes to domain::Migration types but is otherwise independent
- **D1-D2** (settings persistence) can be done in parallel with other work
- **D3** (Chinese UI) touches different layers and is least dependent
