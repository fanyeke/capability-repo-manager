# Data Model: Structured Logging & Operation Tracing

**Date**: 2026-05-15 | **Spec**: [spec.md](./spec.md)

## Entity Changes

### OperationContext — New

| Field | Type | Description |
|-------|------|-------------|
| operation_id | String (UUID) | Unique identifier for the operation, generated at the Tauri command entry point |
| operation_type | String | One of: scan_repositories, refresh_repository, export_pack, delete_pack, build_migration_plan, apply_migration, rollback_migration, run_doctor |
| repo_id | Option\<String\> | Repository ID if the operation targets a repo |
| pack_id | Option\<String\> | Pack ID if the operation involves a pack |
| migration_run_id | Option\<String\> | Migration run ID if the operation is migration-related |

**Usage**: Created at the Tauri command entry, passed through to engine and store layers. All log entries produced during the operation carry the operation_id.

### LogEntry — Not stored in DB (written to log files)

| Field | Type | Description |
|-------|------|-------------|
| timestamp | String (ISO 8601) | When the event occurred |
| level | String | ERROR, WARN, INFO, DEBUG, TRACE |
| event | String | Machine-readable event name (e.g., scan_started, migration_apply_finished) |
| operation_id | String (UUID) | Links all entries in the same operation |
| module | String | Source module path (e.g., repo_scanner, storage) |
| fields | Key-value pairs | Structured context fields (repo_id, pack_id, path, count, error, etc.) |

**Format** (text output):
```
2026-05-15T10:24:02.125Z INFO scan_started operation_id=... roots=["..."] max_depth=5
```

### OperationEvent — New DB table

Stored in the existing SQLite database at `~/.capability-repo-manager/data.db`.

| Field | Type | Description |
|-------|------|-------------|
| id | TEXT (UUID, PK) | Unique event record ID |
| operation_id | TEXT (UUID, NOT NULL) | Links to the operation context |
| operation_type | TEXT (NOT NULL) | Type of operation |
| status | TEXT (NOT NULL) | success, failure, partial_failure |
| repo_id | TEXT | FK to repositories (nullable) |
| pack_id | TEXT | FK to packs (nullable) |
| migration_run_id | TEXT | FK to migration_runs (nullable) |
| summary | TEXT | Human-readable summary (e.g., "Scanned 3 directories, found 17 repos") |
| detail_json | TEXT | Optional JSON with structured details |
| created_at | TEXT (NOT NULL, ISO 8601) | When the event was created |

### RedactResult — Pure output type

| Field | Type | Description |
|-------|------|-------------|
| input | String | Original text |
| output | String | Redacted text |
| redacted_count | usize | Number of sensitive patterns found and replaced |

## DB Schema — New Table

```sql
CREATE TABLE IF NOT EXISTS operation_events (
    id TEXT PRIMARY KEY NOT NULL,
    operation_id TEXT NOT NULL,
    operation_type TEXT NOT NULL,
    status TEXT NOT NULL,
    repo_id TEXT,
    pack_id TEXT,
    migration_run_id TEXT,
    summary TEXT,
    detail_json TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_op_events_operation_id ON operation_events(operation_id);
CREATE INDEX IF NOT EXISTS idx_op_events_created_at ON operation_events(created_at);
CREATE INDEX IF NOT EXISTS idx_op_events_type ON operation_events(operation_type);
```

## AppSettings — Enhanced

Add to existing AppSettings:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| log_level | String | "info" | Log level: error, warn, info, debug, trace |

## Validation Rules

- operation_id MUST be a valid UUID v4
- operation_type MUST be one of the defined enum values
- status MUST be one of: success, failure, partial_failure
- log_level MUST be one of: error, warn, info, debug, trace
- created_at MUST be ISO 8601 format
