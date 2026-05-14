# Data Model: Capability Repo Manager

**Date**: 2026-05-14 | **Phase**: 1 - Design & Contracts

## Entity Relationship Diagram

```
Repository (1) ──< (N) CapabilityResource
Repository (1) ──< (N) DoctorReport
Repository (1) ──< (N) MigrationRun (as target)
Repository (1) ──< (N) CapabilityPack (as source)
CapabilityPack (1) ──< (N) CapabilityResource
CapabilityPack (1) ──< (N) MigrationRun (as source)
MigrationRun (1) ──< (N) MigrationPlanItem
```

## Entities

### Repository

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| id | TEXT (UUID v4) | PK, NOT NULL | Unique identifier |
| name | TEXT | NOT NULL | Display name (derived from directory name) |
| path | TEXT | NOT NULL, UNIQUE | Absolute filesystem path to repo root |
| remote_url | TEXT | NULLABLE | Origin remote URL |
| current_branch | TEXT | NULLABLE | Current HEAD branch name |
| head_commit | TEXT | NULLABLE | Full SHA of HEAD commit |
| dirty_state | TEXT | NOT NULL, CHECK(IN('clean','modified','unknown')) | Working tree state |
| last_indexed_at | TEXT (ISO 8601) | NOT NULL | Timestamp of last scan/refresh |
| scan_depth | INTEGER | DEFAULT 5 | User-configured scan depth for this repo's parent directory |

**Identity**: `id` is the primary key. `path` is a natural unique key — two repos cannot share the same root path.

**State transitions**:
- `dirty_state`: `unknown` → `clean` or `modified` (after first index). `clean` ↔ `modified` (on refresh).
- `last_indexed_at`: Updated on every scan/refresh. Initial null → timestamp.

### CapabilityResource

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| id | TEXT (UUID v4) | PK, NOT NULL | Unique identifier |
| repo_id | TEXT | FK → Repository.id, NULLABLE (if pack-owned) | Owning repository |
| pack_id | TEXT | FK → CapabilityPack.id, NULLABLE (if repo-owned) | Owning pack |
| type | TEXT | NOT NULL, CHECK(IN('skill','mcp','hook','rule','agent','command','plugin','settings','contextDoc')) | Resource type |
| name | TEXT | NOT NULL | Human-readable name |
| source_path | TEXT | NULLABLE | Relative path from repo root to resource file/dir |
| scope | TEXT | NOT NULL, CHECK(IN('project','local','user','inherited','unknown')) | Configuration scope |
| tracked_by_git | BOOLEAN | NOT NULL, DEFAULT 0 | Whether the resource file is under Git version control |
| content_hash | TEXT | NULLABLE | SHA256 hash of resource content |
| metadata_json | TEXT | NULLABLE | Type-specific metadata as JSON blob |
| error_message | TEXT | NULLABLE | Parse error message if resource is malformed |

**Identity**: `id` is primary key. Business identity is `(repo_id OR pack_id, type, name)` — a resource name must be unique within its parent scope for a given type.

**Constraints**:
- Exactly one of `repo_id` or `pack_id` must be non-null (XOR).
- `scope` = `local` implies `tracked_by_git` = false.
- `error_message` is non-null iff the resource could not be fully parsed.

### CapabilityPack

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| id | TEXT (UUID v4) | PK, NOT NULL | Unique identifier |
| name | TEXT | NOT NULL | Pack display name |
| version | TEXT | NOT NULL | Semantic version string |
| description | TEXT | NULLABLE | Human-readable description |
| pack_type | TEXT | NOT NULL, CHECK(IN('project','blueprint','baseline')) | Pack category |
| manifest_path | TEXT | NOT NULL | Absolute path to pack.manifest.json |
| source_repo_id | TEXT | FK → Repository.id, NULLABLE | Repo this pack was exported from |
| source_commit | TEXT | NULLABLE | Commit SHA at time of export |
| created_at | TEXT (ISO 8601) | NOT NULL | Creation timestamp |
| storage_dir | TEXT | NOT NULL | Absolute path to pack root directory |

**Identity**: `id` is primary key. `(name, version)` is a natural unique key — no two packs share the same name+version combination.

### MigrationRun

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| id | TEXT (UUID v4) | PK, NOT NULL | Unique identifier |
| source_type | TEXT | NOT NULL, CHECK(IN('repo','pack')) | Type of migration source |
| source_id | TEXT | NOT NULL | FK reference (polymorphic) to Repository or CapabilityPack |
| target_repo_id | TEXT | FK → Repository.id, NOT NULL | Target repository |
| status | TEXT | NOT NULL, CHECK(IN('planned','applied','failed','rolled_back')) | Migration state |
| plan_json | TEXT | NOT NULL | Serialized MigrationPlan |
| report_json | TEXT | NULLABLE | Serialized MigrationReport after execution |
| snapshot_path | TEXT | NULLABLE | Path to pre-migration backup |
| created_at | TEXT (ISO 8601) | NOT NULL | When plan was created |
| executed_at | TEXT (ISO 8601) | NULLABLE | When migration was applied |

**State transitions**:
- `planned` → `applied` (successful execution)
- `planned` → `failed` (execution error)
- `applied` → `rolled_back` (user-initiated rollback)

### DoctorReport

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| id | TEXT (UUID v4) | PK, NOT NULL | Unique identifier |
| repo_id | TEXT | FK → Repository.id, NOT NULL | Repository diagnosed |
| score | INTEGER | NOT NULL, CHECK(0 ≤ score ≤ 100) | Health score (100 = perfect) |
| issues_json | TEXT | NOT NULL | Serialized array of DoctorIssue objects |
| created_at | TEXT (ISO 8601) | NOT NULL | When report was generated |

### DoctorIssue (embedded in issues_json)

| Field | Type | Description |
|-------|------|-------------|
| severity | TEXT (critical, warning, info) | Issue severity |
| code | TEXT | Machine-readable issue code (e.g., "SKILL_STRUCTURE_INCOMPLETE") |
| message | TEXT | Human-readable description |
| resource_ref | TEXT (nullable) | Reference to affected CapabilityResource.id |
| recommendation | TEXT (nullable) | Actionable fix suggestion |

### MigrationPlanItem (embedded in MigrationRun.plan_json)

| Field | Type | Description |
|-------|------|-------------|
| resource_id | TEXT | Reference to CapabilityResource.id |
| action | TEXT (add, overwrite, skip, rename, merge, unresolved) | Planned action |
| source_path | TEXT (nullable) | Source file path in pack |
| target_path | TEXT (nullable) | Destination file path in target repo |
| status | TEXT (pending, applied, failed) | Execution status |

## Validation Rules

### Repository
- `path` must exist and be an absolute path
- `path` must contain a `.git` directory or file (for worktrees)
- `dirty_state` defaults to `unknown` before first index

### CapabilityResource
- `(repo_id, type, name)` must be unique (or `(pack_id, type, name)`)
- `source_path` must be resolvable relative to parent's root
- XOR constraint: exactly one of `repo_id`/`pack_id` is set

### CapabilityPack
- `(name, version)` must be unique
- `manifest_path` must point to a valid existing manifest file
- Must contain at least one resource (non-empty pack)
- `storage_dir` must exist and be writable

### MigrationRun
- Cannot migrate a pack to its own source repository (circular reference check)
- `plan_json` must be non-empty (at least one planned action)
- Rollback requires `snapshot_path` to exist and be intact

### DoctorReport
- `score` must be computed as: max(0, 100 - 20*critical - 5*warning - info)
- `issues_json` must be a valid JSON array

## Indexes (SQLite)

```sql
CREATE INDEX idx_repos_path ON repositories(path);
CREATE INDEX idx_repos_name ON repositories(name);
CREATE INDEX idx_repos_dirty ON repositories(dirty_state);

CREATE INDEX idx_resources_repo ON capability_resources(repo_id);
CREATE INDEX idx_resources_pack ON capability_resources(pack_id);
CREATE INDEX idx_resources_type ON capability_resources(type);
CREATE INDEX idx_resources_scope ON capability_resources(scope);

CREATE INDEX idx_packs_name_version ON packs(name, version);
CREATE INDEX idx_packs_type ON packs(pack_type);

CREATE INDEX idx_migrations_target ON migration_runs(target_repo_id);
CREATE INDEX idx_migrations_status ON migration_runs(status);

CREATE INDEX idx_doctor_repo ON doctor_reports(repo_id);
CREATE INDEX idx_doctor_score ON doctor_reports(score);
```
