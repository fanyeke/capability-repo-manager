# Implementation Plan: Main Process Stabilization Loop

**Branch**: `002-stabilization-loop` | **Date**: 2026-05-15 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/002-stabilization-loop/spec.md`

## Summary

Fix 12 identified bugs to close the three core loops: (1) Repo Scan → Capability Inventory generation and persistence, (2) Pack Export → Persistent Pack Library → Pack Apply, (3) Migration Dry-run → Snapshot → Execute → Rollback. Adds scoped migration snapshots, complete state machine, strict enum validation, cascade cleanup, and Chinese UI support.

## Technical Context

**Language/Version**: Rust 1.75+ (backend), TypeScript 5.x (frontend), Svelte 5 (UI)
**Primary Dependencies**: Tauri 2 (desktop shell), rusqlite 0.31 (SQLite), serde 1 (serialization), uuid 1, chrono 0.4
**Storage**: SQLite at `~/.capability-repo-manager/data.db` (repo metadata, resources, packs, migrations, doctor reports). Settings: `~/.capability-repo-manager/settings.json` (JSON file). Migration snapshots: `~/.capability-repo-manager/snapshots/` (disk directory + JSON manifest).
**Testing**: cargo test (Rust unit + integration), Vitest (frontend unit). Fixture repos at `tests/fixtures/`.
**Target Platform**: Linux (MVP)
**Project Type**: Desktop application (Tauri 2 shell: Rust backend + Svelte frontend)
**Performance Goals**: 100 repos scanned with capability parsing <60s (SC-009). Resource detail lookup <500ms (SC-004).
**Constraints**: Local-only operation. Dry-run before writes. Transactional integrity for all data mutations. Scoped snapshots only (no full-repo backups).
**Scale/Scope**: Single-user local desktop tool. ~500 repos max, ~50 packs, packs with up to ~100 resources each.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. TDD First (NON-NEGOTIABLE) | PASS | Each milestone has associated test crates. No code changes without corresponding test updates. |
| II. Repo-First Architecture | PASS | All changes are repo-anchored. Identity restoration ensures repos remain root aggregates. |
| III. Dry-Run by Default | PASS | Migration safety enhancements (C1-C8) explicitly enforce dry-run → plan → snapshot → execute flow. |
| IV. Domain-Driven Design | PASS | Changes concentrated in storage layer (new store methods) and tauri-bridge (command rework). Domain types enhanced minimally. |
| V. Local-First & Safe | PASS | Settings persisted locally. Snapshots stored locally. No network changes. |
| VI. Cross-Platform Core with Linux Priority | PASS | File path handling uses std::path. Settings use home-dir convention. Linux MVP focus unchanged. |

**Gate Result**: ALL PASS — Proceed.

## Serial vs Parallel Design

This feature contains 10 tasks across 4 milestones. Here is the dependency graph:

```
                    ┌──────────────────────────────────────┐
                    │  M-A: Stable Repository Index        │
                    │  ├── A1: DB schema + domain fields   │
                    │  ├── A2: upsert_by_path()            │
                    │  ├── A3: Fix scan identity           │
                    │  ├── A4: Add parsing to pipeline     │
                    │  ├── A5: replace_for_repo()          │
                    │  └── A6: index_status fields         │
                    └──────────────┬───────────────────────┘
                                   │ (M-A must complete first)
                   ┌───────────────┴───────────────────────┐
                   │                                       │
    ┌──────────────▼──────────────┐          ┌─────────────▼──────────────┐
    │  M-B: Resource & Pack       │          │  M-C: Safe Migration      │
    │  ├── B1: get_by_id()        │          │  ├── C1: ConflictAction   │
    │  ├── B2: Fix detail query   │◄─────    │  ├── C2: Snapshot types   │
    │  ├── B3: pack_store module  │  │       │  ├── C3: migration_store  │
    │  ├── B4: Export → DB persist│  ├──────►│  ├── C4: snapshot create  │
    │  ├── B5: Fix pack delete    │  │       │  ├── C5: per-file exec    │
    │  └── B6: Cascade repo del   │  │       │  ├── C6: state machine    │
    └───────────────────────────▲──┘  │       │  ├── C7: scoped rollback │
                                 │     │       │  └── C8: strategy valid  │
                    ┌────────────┘     │       └──────────────────────────┘
                    │                  │
    ┌───────────────┴──────────────────┴──────────────────┐
    │  M-D: Settings & Chinese UI (fully parallel)        │
    │  ├── D1: Settings persistence (JSON)                │
    │  ├── D2: Register settings_commands                │
    │  └── D3: Chinese UI i18n                           │
    └────────────────────────────────────────────────────┘
```

### Parallel Execution Strategy

| Parallel Group | Tasks | Why Parallel | Coordination |
|---------------|-------|-------------|--------------|
| **Group 1** (after M-A completes) | B1-B3 + C1-C3 + D1-D2 | B (resource/pack store) and C (migration types/state) touch different files. D (settings) is fully independent. | None — no shared files between groups |
| **Group 2** (after Group 1) | B4-B6 + C4-C8 | Pack persistence wiring vs migration execution wiring — independent command handlers | B4-B6 modify pack_commands.rs; C4-C8 modify migration_commands.rs — separate files |
| **Group 3** (anytime after M-A) | D3 (Chinese UI) | Svelte i18n is fully independent of Rust backend changes | Can be worked on at any time in parallel |

### Strictly Serial Paths (cannot parallelize)

| Serial Chain | Reason |
|-------------|--------|
| A1 → A2 → A3 | Schema change → store method → command fix (direct dependency chain) |
| C1 → C2 → C3 → C4 → C5 → C6 → C7 → C8 | Enum types → snapshot domain types → store methods → executor wiring → state machine → rollback (each builds on the previous) |

## Project Structure

### Documentation (this feature)

```text
specs/002-stabilization-loop/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (8 contract files)
│   ├── README.md
│   ├── scan-repository.md
│   ├── resource-detail.md
│   ├── pack-export.md
│   ├── pack-delete.md
│   ├── repo-remove.md
│   ├── migration-plan.md
│   ├── migration-apply.md
│   ├── migration-rollback.md
│   └── settings.md
├── checklists/
│   └── requirements.md
└── tasks.md             # Phase 2 output (/speckit-tasks)
```

### Source Code Changes (by crate)

```text
crates/domain/                       # Minimal type additions
├── src/repository.rs                # +canonical_path, +index_status fields
└── src/migration.rs                 # +ConflictAction enum, +MigrationSnapshot/SnapshotItem

crates/storage/                      # Heavy: new stores + method enhancements
├── src/lib.rs                       # DB schema migration (ALTER TABLE)
├── src/repo_store.rs                # +upsert_by_path, +get_by_canonical_path, +update_index_status, +delete_cascade
├── src/resource_store.rs            # +get_by_id, +replace_for_repo, +replace_for_pack, +delete_by_pack
├── src/pack_store.rs                # NEW: DB-backed pack persistence
└── src/migration_store.rs           # NEW: migration run state machine + snapshot storage

crates/repo-scanner/                 # Minor: pass-through repo identity
├── src/lib.rs                       # Accept existing repo ID, don't generate new UUID always

crates/pack-engine/                  # Minor: library crate remains as utility
├── src/library.rs                   # PackStore becomes thin wrapper or removed

crates/migration-engine/             # Moderate: snapshot + per-file exec
├── src/executor.rs                  # +snapshot creation, +per-file execution with status
├── src/rollback.rs                  # Use scoped snapshot for precise rollback
└── src/lib.rs                       # +create_snapshot, +restore_from_snapshot

crates/tauri-bridge/                 # Heavy: command rework
├── src/commands/repo_commands.rs    # Scan pipeline overhaul
├── src/commands/capability_commands.rs  # Fix resource detail query
├── src/commands/pack_commands.rs    # DB-backed pack export/delete
├── src/commands/migration_commands.rs   # Snapshot + state machine + strategy validation
├── src/commands/settings_commands.rs    # Fix: persist to JSON file
└── src/commands/mod.rs              # Register settings_commands

crates/tauri-bridge/src/state.rs     # Settings load from ~/.capability-repo-manager/settings.json

frontend/                            # Chinese UI (i18n)
├── src/lib/i18n/                    # NEW: i18n module
└── src/*.svelte                     # Wrap strings with i18n function

frontend/src-tauri/src/lib.rs        # Register settings_commands
```

**Structure Decision**: Changes spread across existing crate boundaries. No new crates. Two new storage modules (`pack_store`, `migration_store`). One new frontend module (`i18n`).

## Complexity Tracking

> No violations to justify. All 6 principles pass without needing exceptions.

## Generated Artifacts Checklist

| Artifact | Location | Status |
|----------|----------|--------|
| plan.md | specs/002-stabilization-loop/plan.md | ✅ Done |
| research.md | specs/002-stabilization-loop/research.md | ✅ Phase 0 |
| data-model.md | specs/002-stabilization-loop/data-model.md | ✅ Phase 1 |
| contracts/ | specs/002-stabilization-loop/contracts/ | ✅ Phase 1 (8 files) |
| quickstart.md | specs/002-stabilization-loop/quickstart.md | ✅ Phase 1 |
| tasks.md | specs/002-stabilization-loop/tasks.md | 🔲 Run `/speckit-tasks` |
