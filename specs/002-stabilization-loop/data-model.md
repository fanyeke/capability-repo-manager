# Data Model: Main Process Stabilization Loop

**Date**: 2026-05-15 | **Spec**: [spec.md](./spec.md)

## Entity Changes

### Repository — Enhanced

| Field | Type | Change | Description |
|-------|------|--------|-------------|
| id | UUID (PK) | — | Stable identity, reused across scans |
| name | TEXT | — | Repository directory name |
| path | TEXT (UNIQUE) | — | Canonical filesystem path (resolved symlinks) |
| canonical_path | TEXT | **NEW** | Normalized, symlink-resolved absolute path used for identity lookup |
| remote_url | TEXT? | — | Git remote origin URL |
| current_branch | TEXT? | — | Current git branch name |
| head_commit | TEXT? | — | HEAD commit SHA |
| dirty_state | TEXT | — | 'clean' / 'modified' / 'unknown' |
| first_indexed_at | TEXT | **NEW** | ISO 8601 timestamp of first scan |
| last_indexed_at | TEXT | — | ISO 8601 timestamp of last metadata refresh |
| capability_index_status | TEXT | **NEW** | 'never_indexed' / 'fresh' / 'stale' / 'parse_failed' |
| last_capability_indexed_at | TEXT? | **NEW** | ISO 8601 timestamp of last successful capability parse |
| last_capability_error | TEXT? | **NEW** | Error message from last failed capability parse |

**Migration SQL**:
```sql
ALTER TABLE repositories ADD COLUMN canonical_path TEXT;
ALTER TABLE repositories ADD COLUMN first_indexed_at TEXT;
ALTER TABLE repositories ADD COLUMN capability_index_status TEXT NOT NULL DEFAULT 'never_indexed';
ALTER TABLE repositories ADD COLUMN last_capability_indexed_at TEXT;
ALTER TABLE repositories ADD COLUMN last_capability_error TEXT;
CREATE INDEX IF NOT EXISTS idx_repos_canonical_path ON repositories(canonical_path);
```

### RepositoryStore — Enhanced Methods

| Method | Description |
|--------|-------------|
| `upsert_by_path(repo) -> Repository` | Query by canonical_path first; if exists, update metadata & return existing ID; if not, insert with new UUID |
| `get_by_canonical_path(path) -> Option<Repository>` | Direct canonical path lookup |
| `update_index_status(id, status, error?)` | Update capability_index_status, last_capability_indexed_at, last_capability_error |
| `delete_cascade(id)` | Delete repo + associated capability_resources + doctor_reports in single transaction |

### ResourceStore — Enhanced Methods

| Method | Description |
|--------|-------------|
| `get_by_id(resource_id) -> Option<CapabilityResource>` | **NEW** — Direct primary key lookup |
| `replace_for_repo(repo_id, resources) -> Result` | **NEW** — Atomic: DELETE old + INSERT new in single transaction. On parse error, skip DELETE to preserve old data. |
| `replace_for_pack(pack_id, resources) -> Result` | **NEW** — Atomic replace for pack resources |
| `delete_by_pack(pack_id) -> Result` | **NEW** — Clean up all resources associated with a pack |
| `delete_by_repo_cascade(repo_id) -> Result` | Enhanced — Used by repo cascade delete |

### PackStore — Replaced with DB-backed Storage

The in-memory `pack_engine::library::PackStore` (Vec-based) is replaced with storage crate SQLite queries.

**Existing DB table** (`packs`) already has the required schema (no migration needed for pack metadata).

**Pack resource management**: Capability resources reference a pack via `pack_id` column in `capability_resources` table. When a pack is exported, resources are inserted with the pack_id. When deleted, both the packs row and associated capability_resources rows are removed.

**New store methods** in `storage::pack_store`:
| Method | Description |
|--------|-------------|
| `insert_pack(pack) -> Result` | Insert pack metadata into DB |
| `get_pack_by_id(pack_id) -> Option<CapabilityPack>` | Query by primary key |
| `get_pack_by_name_version(name, version) -> Option<CapabilityPack>` | For duplicate detection |
| `list_packs(filter) -> Vec<CapabilityPack>` | List with optional search/type filter |
| `delete_pack(pack_id) -> Result` | Remove DB entry AND disk files AND associated resources |

### MigrationRun — Status State Machine

The `status` field in `migration_runs` table uses this state machine:

```
planned ──→ ready ──→ executing ──→ success
                                      ├── partial_failure
                                      └── failed
success ──→ rolled_back
partial_failure ──→ rolled_back
```

**No migration needed**: The existing `status TEXT` column already accommodates these values.

**Enhanced DB queries** (stored in `storage::migration_store`):
| Method | Description |
|--------|-------------|
| `insert_run(run) -> Result` | Insert migration run record |
| `get_run(run_id) -> Option<MigrationRun>` | Query by primary key |
| `update_status(run_id, status) -> Result` | Update status (state machine transition) |
| `update_snapshot(run_id, snapshot_json) -> Result` | Store snapshot manifest as JSON |
| `update_report(run_id, report_json) -> Result` | Store migration report |
| `list_by_repo(repo_id) -> Vec<MigrationRun>` | List runs for a repository |

### MigrationSnapshot — New Entity

```rust
struct MigrationSnapshot {
    id: String,             // UUID
    run_id: String,         // FK to migration_runs
    created_at: String,     // ISO 8601
    items: Vec<SnapshotItem>,
}

struct SnapshotItem {
    target_path: String,    // Relative path within target repo
    existed_before: bool,   // Whether the file existed before migration
    backup_path: String,    // Absolute path to backup copy
}
```

**Storage**: Snapshots are stored on disk under `~/.capability-repo-manager/snapshots/<run_id>/` with a `snapshot.json` manifest. Each backup file is stored with its original filename plus a UUID suffix.

### ConflictAction — Strict Enum

Replaces free-string action with:

```rust
enum ConflictAction {
    Skip,
    Overwrite,
    Rename,
    Merge,
}
```

Used in:
- `MigrationConflict.resolved_action`: populated by user strategy
- `MigrationPlanItem.action`: validated against this enum before execution

## Constraint Rules

- **Repository.canonical_path** MUST be unique in the database (enforced by unique index)
- **capability_index_status** MUST be one of: `never_indexed`, `fresh`, `stale`, `parse_failed`
- **MigrationRun.status** transitions MUST follow the defined state machine; invalid transitions MUST be rejected
- **ConflictAction** MUST reject any value not in the enum; no silent fallback to "skip"
- **Pack (name, version)** MUST remain unique (enforced by UNIQUE constraint)
- **Resource replace operations** MUST be atomic (single transaction per repo/pack)
