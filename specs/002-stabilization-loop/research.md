# Research: Main Process Stabilization Loop

**Date**: 2026-05-15 | **Phase**: 0

## R1: Repo Identity Strategy -- Canonical Path Lookup

### Decision 1: Canonical Path Normalization with `std::fs::canonicalize`
- **Decision**: Normalize repo paths via `std::fs::canonicalize()` at scan time. Query the `repositories` table by resolved canonical path before inserting. Reuse existing UUID if found; generate new UUID if not found.
- **Rationale**: `std::fs::canonicalize()` resolves symlinks, relative path components (`.` / `..`), and produces an absolute, unique path. This is the standard Rust approach for filesystem identity and requires zero external dependencies (part of std). The `repositories` table already has a `path` column with a UNIQUE constraint. By storing the canonical path, we naturally deduplicate repos that resolve to the same underlying directory.
- **Alternatives Considered**:
  - Abstraction crate (e.g., `path-clean`, `camino`): adds unnecessary dependency weight for a single operation that std provides natively.
  - Manual normalization (strip trailing slashes, resolve `.` / `..`): error-prone, misses symlink resolution. `canonicalize` handles all edge cases.

### Decision 2: Upsert Transaction Boundary
- **Decision**: Wrap repo upsert and capability resource replacement in a single SQLite transaction. The flow is: (1) begin transaction, (2) upsert repo by canonical path, (3) delete old resources for that `repo_id`, (4) insert new resources, (5) commit.
- **Rationale**: Atomicity ensures that partial failure never leaves the DB in an inconsistent state (e.g., repo updated but resources stale, or resources inserted but repo record missing). Per FR-006, successful parsing must atomically replace old resources.
- **Alternatives Considered**:
  - Upsert then lazy cleanup of orphans: leaves a window where stale data exists. No benefit over explicit transaction.
  - Separate transactions per operation: partial failures corrupt data. Rejected per FR-005/FR-006 requirements.

### Decision 3: Path Changed Detection
- **Decision**: No automatic path migration. If canonical path differs from stored path, treat as a new repo (generate new ID). Old repo remains in DB with old path.
- **Rationale**: The spec clarifies that repository relocation results in a new identity. The old identity is preserved for migration history references (FR-016: cascade cleanup preserves migration history with "target missing" indicator). This avoids complex path-migration logic.
- **Alternatives Considered**:
  - Update path on same ID: violates canonical path identity contract. Would break foreign key relationships silently.

---

## R2: Transaction Boundaries for Scan Pipeline

### Decision 1: Per-Repo Transaction with Rollback-On-Failure
- **Decision**: Each repository in the scan pipeline gets its own transaction: (begin) upsert repo -> delete old resources -> insert parsed resources (commit). If any step fails, rollback the entire repo's transaction but continue scanning other repos.
- **Rationale**: Per FR-004, capability parsing errors must not fail the entire scan. Per-repo transactions provide isolation: repo A's failure does not affect repo B's committed data. Wrapping the full pipeline in a single transaction would require a full rollback on any single failure, losing all previously scanned repos.
- **Alternatives Considered**:
  - Single transaction for all repos: any failure undoes all work. Unacceptable for 100-repo scans.
  - No transaction: partial writes lead to inconsistent state (repo record without resources, or resources orphaned).

### Decision 2: SQLite Transaction API in Storage Crate
- **Decision**: Add `begin_transaction()`, `commit()`, and `rollback()` methods to the `Database` struct or expose `Transaction` scope guard. Use `rusqlite::Transaction` which auto-rollbacks on drop if not committed.
- **Rationale**: The current `Database` struct wraps a `rusqlite::Connection` but exposes no transaction API. Engines like `repo-scanner` need explicit control. Using `rusqlite::Transaction` as a scope guard is idiomatic and safe -- if the guard drops without commit, it auto-rolls back.
- **Alternatives Considered**:
  - Implicit transactions (PRAGMA journal_mode): no control over boundaries. Doesn't satisfy fine-grained per-repo isolation.
  - Let callers manage `RAW: Connection::transaction()`: works but couples engines to rusqlite types. A storage-level abstraction is cleaner.

### Decision 3: SQLite Transaction Isolation -- SERIALIZABLE (Default)
- **Decision**: Use SQLite's default serializable isolation. No change needed -- rusqlite defaults to deferred transaction, which provides serializable isolation in practice for a single-writer app.
- **Rationale**: SQLite is a single-writer database. With only one `Database` instance (Tauri app state), concurrent write conflicts don't occur. The default isolation level is sufficient for all scan pipeline operations.
- **Alternatives Considered**:
  - `BEGIN IMMEDIATE`: prevents `SQLITE_BUSY` from readers, but no concurrent readers in our architecture.
  - `BEGIN EXCLUSIVE`: overkill for single-writer scenario.

---

## R3: PackStore Persistence Strategy

### Decision 1: Replace In-Memory PackStore with SQLite-Backed Implementation
- **Decision**: Rewrite `pack-engine/src/library.rs`'s `PackStore` to use the storage crate's `Database` instead of `Vec<CapabilityPack>`. The `packs` table already exists in the schema with the required columns. The key methods (`insert`, `get_by_id`, `list`, `search`, `delete`) will query SQLite directly.
- **Rationale**: The current `PackStore` is explicitly marked "In-memory pack store -- in production, this would be backed by SQLite" (line 6-7 of library.rs). The `packs` table schema already supports all required operations (CRUD, unique name+version constraint, search via LIKE). Moving from Vec to SQLite provides persistence across app restarts (FR-009) and enables future features like history, audit logs, and cross-session pack management.
- **Alternatives Considered**:
  - Hybrid approach (memory cache + DB write-through): adds complexity without clear benefit for a desktop app with <100 packs.
  - Full file-system index (read pack manifest dirs on startup): slow startup, no query support, duplicate-name detection requires scanning all manifests.

### Decision 2: Pack Export Atomicity -- Write Disk First, Then DB
- **Decision**: Export flow: (1) create temporary directory, (2) write all pack files (manifest.json, resources), (3) validate integrity, (4) move/rename temp dir to final pack storage path, (5) insert DB record. If any step fails, clean up temp dir and abort -- no DB record is created.
- **Rationale**: FR-010 requires atomic export. Writing disk artifacts first and only inserting the DB record on success ensures no orphan DB records for failed exports. The final rename of the temp directory is an atomic filesystem operation on Linux (same filesystem).
- **Alternatives Considered**:
  - DB first then disk: disk write failure leaves a DB record referencing non-existent files.
  - Transaction across DB and disk: impossible -- filesystem and SQLite have no distributed transaction support.

### Decision 3: Pack Deletion Cascade -- DB First, Then Disk
- **Decision**: Delete flow: (1) begin transaction, (2) delete associated `capability_resources` where `pack_id = ?`, (3) delete `packs` record, (4) commit, (5) remove pack directory from disk. If disk removal fails, log warning but do not roll back the DB transaction (the pack is already logically deleted).
- **Rationale**: FR-011 requires both DB entries and disk files to be removed. Deleting DB first ensures the pack is logically gone even if disk cleanup fails (non-critical -- disk space is reclaimable manually). The cascade to `capability_resources` is explicitly required (FR-011 says "associated resource entries in the DB are also removed").
- **Alternatives Considered**:
  - Disk first then DB: if DB delete fails, disk is already removed but the pack appears in the library. Worse user experience (ghost entries).
  - Rolling back DB on disk failure: over-engineered. Stale disk files are harmless; stale DB entries are confusing.

---

## R4: Migration Snapshot Strategy

### Decision 1: Scoped Per-File Snapshot (Not Full Directory)
- **Decision**: Snapshot only the files that a migration plan will affect (add, overwrite, rename actions). Store each file's pre-migration copy in a dedicated snapshot directory. Record existence status (file existed vs. file did not exist) in a JSON manifest alongside the backups.
- **Rationale**: FR-014 requires scoped snapshots ("only the files that will be affected"). Full directory copies (current implementation in `rollback.rs`) are wasteful -- copying the entire target repo even if only 1 file changes is O(repo-size) instead of O(affected-files). For a repo with thousands of files but only 5 migration targets, scoped snapshots are ~1000x smaller.
- **Alternatives Considered**:
  - Full directory copy via `cp -r`: simple but wasteful. Current implementation does this (line 12 of rollback.rs: `copy_dir_recursive(target_dir, &snapshot_dir)`). Must be replaced.
  - git-based snapshot (`git stash`): slow, requires the target to be a git repo with clean state, adds git dependency. Not applicable for non-git-target scenarios.

### Decision 2: Snapshot Storage Location
- **Decision**: Store snapshots in `~/.capability-repo-manager/snapshots/<run-id>/`. Each snapshot directory contains a manifest file (`snapshot.json`) and a `backups/` subdirectory with the copied files preserved in relative paths.
- **Rationale**: Using a dedicated app data directory (following XDG conventions via `dirs::data_dir()`) keeps snapshots separate from user content. Grouping by migration run ID makes cleanup straightforward: when a run is rolled back or cleaned up, the entire `<run-id>/` directory is removed. The JSON manifest enables verification without scanning the filesystem.
- **Alternatives Considered**:
  - Snapshot inside target repo (`.claude/snapshots/`): pollutes user's repo, could be accidentally committed.
  - System temp dir (`/tmp/`): lost on reboot, no persistence for crash-recovery scenarios.
  - Alongside the DB file: works, but snapshot bloat is better isolated from DB.

### Decision 3: Snapshot Manifest Schema
- **Decision**: JSON manifest per snapshot run:
  ```json
  {
    "runId": "uuid-of-migration-run",
    "createdAt": "2026-05-15T12:00:00Z",
    "affectedPaths": [
      {
        "relativePath": ".claude/rules/custom-rule.md",
        "existed": true,
        "backupPath": "backups/.claude/rules/custom-rule.md",
        "contentHash": "sha256-..."
      },
      {
        "relativePath": ".claude/new-file.md",
        "existed": false,
        "backupPath": null,
        "contentHash": null
      }
    ]
  }
  ```
- **Rationale**: The manifest is the single source of truth for rollback. The `existed` flag determines whether rollback should restore the file (if it existed before) or delete the file (if it was created by migration -- new files should be removed on rollback per FR-017). Content hash enables integrity verification. Keeping backup paths relative makes the snapshot portable.
- **Alternatives Considered**:
  - Filesystem-only (list dir contents diff): expensive to compute at rollback time. Manifest makes rollback O(affected-files) instead of O(repo-files).
  - SQLite-backed snapshot (store in DB): BLOBs in DB are slow for large files. Filesystem storage is standard for backups.

### Decision 4: Rollback Flow
- **Decision**: Rollback reads the snapshot manifest, then for each affected path: (a) if `existed == true`, copy backup file back to original location, (b) if `existed == false`, delete the file if it exists (created by migration). After all files restored, update migration run status to `rolled_back`.
- **Rationale**: Matches FR-017 exactly: "Rollback MUST only restore files that were part of the migration snapshot, without affecting other files in the repository." Per-file operations ensure partial rollback is possible even if some files fail.
- **Alternatives Considered**:
  - Full directory restore (copy snapshot back over target): would delete files not in the snapshot (files unrelated to migration). Dangerous and violates FR-017.

---

## R5: Chinese UI Strategy

### Decision 1: Use `svelte-i18n` for Frontend Internationalization
- **Decision**: Add `svelte-i18n` as a dependency. Create locale JSON files in `frontend/src/lib/i18n/` (en.json, zh.json). Wrap all user-facing strings in `$t('key')` calls. Default to system locale with manual override in Settings page.
- **Rationale**: `svelte-i18n` is the most mature Svelte i18n library (500+ stars, maintained). It provides reactive translations, locale switching without page reload, and pluralization support. It integrates naturally with Svelte 5's reactive `$state` system. The locale JSON approach is simple -- no build step for translations, easy to maintain.
- **Alternatives Considered**:
  - `i18next` (the most popular JS i18n framework): overkill for a desktop app with 2 languages. Large bundle size.
  - `typesafe-i18n`: requires code generation step, adds build complexity.
  - Custom Svelte store + JSON: works but duplicates effort already handled by svelte-i18n (locale detection, fallback, reactive updates).
  - Rust-side i18n (Tauri command returns translated strings): couples backend to UI concerns, makes testing harder.

### Decision 2: Locale Detection and Persistence
- **Decision**: On app launch, detect locale via Tauri's `locale` API (`@tauri-apps/plugin-os` or `navigator.language` in webview). Store user preference in `settings_commands` (new module, persisted to SQLite `settings` table or a simple JSON file). Priority: manual preference > system locale > 'en' fallback.
- **Rationale**: The spec requires "system locale with manual override in settings" (FR-019). Tauri 2 provides locale detection through its OS plugin. Persisting the preference ensures the choice survives restart. A `settings` table in SQLite is the natural persistence mechanism (already exists for scan roots, scan depth, etc.).
- **Alternatives Considered**:
  - localStorage in webview: lost on cache clear, not accessible from Rust backend.
  - Sidecar config file: unnecessary -- SQLite is already the persistence layer.

### Decision 3: Translation Coverage Scope
- **Decision**: Phase 0 covers only product UI strings (navigation, buttons, labels, error messages, empty states). CLI strings, log messages, and Rust-side error messages remain in English for this phase.
- **Rationale**: The spec clarifies "product UI supports Chinese" -- this means the Svelte frontend UI strings. Rust backend messages (log output, error enums) are developer-facing and can be translated in a future phase. Keeping Rust messages in English avoids coupling translation logic into engine crates.
- **Alternatives Considered**:
  - Full backend i18n: adds significant complexity to all 8 engine crates. Little user-facing benefit (errors are surfaced in the frontend which already translates display strings).

---

## R6: Serial vs. Parallel Task Design

### Decision 1: Dependency Graph Analysis

```
                        P0-01 (Repo ID)
                            |
                     P0-02 (Scan Pipeline)
                        /           \
                 P0-03 (Resource)  P0-04 (Pack Persistence)
                 Detail Fix             |
                                   P0-05/P0-06/P0-07 (Migration Safety)
                                            |
                                     P0-08 (Cascade Cleanup)
```

**Strictly serial dependencies**:
- **P0-01 (Repo ID)** blocks **P0-02 (Scan Pipeline)**: The canonical path lookup logic must be in place before the scan pipeline can use it.
- **P0-02 blocks P0-03 and P0-04**: Both resource detail fix and pack persistence depend on a working scan pipeline that correctly associates resources with stable repo IDs.
- **P0-04 blocks P0-05/P0-06/P0-07**: Migration safety features require persistent PackStore to be functional first.
- **P0-08 (Cascade Cleanup)** is a leaf task -- depends on everything above being stable.

**Independent (can parallelize)**:
- **P0-03 (Resource Detail Fix)** and **P0-04 (Pack Persistence)** can be implemented in parallel -- they modify different files (`resource_store.rs` vs `library.rs` + pack-engine).
- **P0-05 (Snapshot)**, **P0-06 (State Machine)**, **P0-07 (Rollback)** are internally sequential (state machine defines the framework for execution, snapshot provides the data for rollback) but are in the same crate (`migration-engine`) -- best done by one developer sequentially.
- **P0-09 (Chinese UI)** is fully independent -- it modifies only frontend files. Can run in parallel with any Rust backend work.

### Decision 2: Recommended Execution Order

```text
Phase 1 (serial):
  P0-01: Repo ID (canonical path lookup) -- repo_store.rs, repo-scanner/src/lib.rs
  P0-02: Scan Pipeline (auto capability parse, error resilience) -- repo-scanner, claude-parser integration

Phase 2 (parallel):
  [Parallel A] P0-03: Resource Detail Fix (get_by_id) -- resource_store.rs, capability_commands.rs
  [Parallel B] P0-04: Pack Persistence (DB-backed PackStore) -- pack-engine/library.rs, storage

Phase 3 (serial within parallel):
  [Branch A continues] P0-08: Cascade Cleanup (repo delete cascade) -- repo_store.rs, doctor_commands.rs
  [Branch B continues] P0-05/P0-06/P0-07: Migration Safety (snapshot, state machine, rollback) -- migration-engine/

Independent (any time):
  P0-09: Chinese UI -- frontend/src/lib/i18n/
```

- **Rationale**: This ordering minimizes blocking. Phase 1 must be serial because both downstream phases need correct repo identity and a working scan pipeline. Phase 2 splits into two parallel tracks (resource detail + pack persistence). Phase 3 completes both tracks independently. Chinese UI is a background task that can be slotted in at any point.
- **Alternatives Considered**:
  - Full serial execution: slower (9 sequential phases instead of 4), no developer parallelism.
  - Full parallel execution: impossible -- the dependency graph has strict edges (P0-01 -> P0-02 -> P0-04 -> P0-05).

### Decision 3: Per-Task File Change Analysis

| Task | Files Modified (Rust) | Files Modified (Frontend) | Estimated Complexity |
|------|----------------------|--------------------------|---------------------|
| P0-01 | `repo_store.rs`, `repo-scanner/src/lib.rs`, `domain/src/repository.rs` | none | Small (add canonical path field + lookup) |
| P0-02 | `repo-scanner/src/lib.rs`, `resource_store.rs` (transaction), `claude-parser/src/lib.rs` | `repoStore.ts` (auto-refresh) | Medium (orchestration logic) |
| P0-03 | `resource_store.rs` (add get_by_id), `capability_commands.rs` | `capabilityStore.ts` (detail fetch) | Small (single method + command) |
| P0-04 | `pack-engine/src/library.rs` (full rewrite), `pack-engine/src/lib.rs` | `packStore.ts` | Medium (rewrite from Vec to SQLite) |
| P0-05 | `migration-engine/src/rollback.rs` (rewrite), `migration-engine/src/executor.rs` | none | Medium (scoped snapshot logic) |
| P0-06 | `domain/src/migration.rs` (state machine), `migration-engine/src/lib.rs` | `migrationStore.ts` | Small (enum + validation) |
| P0-07 | `migration-engine/src/executor.rs`, `migration-engine/src/rollback.rs` | `migrationStore.ts` | Small (per-file rollback) |
| P0-08 | `repo_store.rs` (cascade delete), `doctor_commands.rs` | `repoStore.ts` | Small (SQL cascade + command) |
| P0-09 | `settings_commands.rs` (locale persistence) | i18n files, all `.svelte` files | Medium (translation wrapping) |

- **Rationale**: Understanding the file change footprint helps with parallel assignment. Tasks touching disjoint files (P0-03 vs P0-04) are safe to parallelize. Tasks touching the same files (P0-05/P0-06/P0-07 all modify `migration-engine/`) must be sequential or carefully merged.
