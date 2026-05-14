# Tauri Command Contracts: Capability Repo Manager

**Date**: 2026-05-14 | **Phase**: 1 - Design & Contracts

## Overview

All frontend-backend communication is via Tauri `invoke` commands. Each command has a Rust handler in `tauri-bridge/src/commands/`. Commands return `Result<T, String>` where T is a JSON-serializable type and errors are human-readable strings.

## Command Catalog

### Repo Commands

#### `scan_repositories(paths: Vec<String>) → ScanResult`
Scan one or more root directories for Git repositories.

**Input**: List of absolute directory paths to scan.
**Output**: `{ repos_found: u32, repos_added: u32, repos_updated: u32, errors: Vec<ScanError> }`
**Errors**: Permission denied, path not found, scan timeout.
**Side effects**: Inserts/updates Repository rows in SQLite. Triggers background Git metadata extraction.

#### `list_repositories(filter: RepoFilter) → Vec<RepositorySummary>`
List indexed repositories with optional filtering.

**Input**: `{ search?: string, dirty_only?: bool, has_capabilities?: bool, sort_by?: string, sort_order?: string }`
**Output**: Array of `{ id, name, path, branch, dirty_state, capability_counts: { skills, mcp, hooks, rules, agents }, doctor_score?, last_indexed_at }`
**Errors**: Database read error.

#### `refresh_repository(repo_id: String) → RepositoryDetail`
Re-scan a single repository's Git metadata and capability resources.

**Input**: Repository UUID.
**Output**: Full `RepositoryDetail` with all metadata + capability inventory.
**Errors**: Repo not found, path no longer accessible, Git error.

#### `get_repository_detail(repo_id: String) → RepositoryDetail`
Get full detail for a repository (cached data).

**Input**: Repository UUID.
**Output**: `{ repo: Repository, capabilities: Vec<CapabilityResource>, doctor_latest?: DoctorReport }`
**Errors**: Repo not found.

#### `remove_repository(repo_id: String) → ()`
Remove a repository from the index (does not delete files on disk).

**Input**: Repository UUID.
**Errors**: Repo not found.

### Capability Commands

#### `get_capability_inventory(repo_id: String) → CapabilityInventory`
Get all capability resources for a repository, grouped by type.

**Input**: Repository UUID.
**Output**: `{ skills: Vec<CapabilityResource>, mcp: Vec<CapabilityResource>, hooks: Vec<CapabilityResource>, rules: Vec<CapabilityResource>, agents: Vec<CapabilityResource>, commands: Vec<CapabilityResource>, plugins: Vec<CapabilityResource>, settings: Vec<CapabilityResource> }`
**Errors**: Repo not found, parse errors (returned per-resource not as command error).

#### `get_resource_detail(resource_id: String) → ResourceDetail`
Get full detail for a single capability resource.

**Input**: Resource UUID.
**Output**: `CapabilityResource` with all fields populated + `dependencies: Vec<DependencyStatus>`.
**Errors**: Resource not found.

### Pack Commands

#### `export_capability_pack(repo_id: String, selection: PackSelection, metadata: PackMetadata) → PackSummary`
Export selected resources as a capability pack.

**Input**:
- `repo_id`: Source repository UUID
- `selection`: `{ resource_ids: Vec<String> }` — specific resources to include
- `metadata`: `{ name: String, version: String, description?: String, pack_type: String }`

**Output**: `{ id, name, version, resource_count, storage_dir, created_at }`
**Errors**: Empty selection, repo not found, disk write error, duplicate name+version.

#### `list_packs(filter: PackFilter) → Vec<PackSummary>`
List all packs in the library.

**Input**: `{ search?: String, pack_type?: String }`  
**Output**: Array of `{ id, name, version, description, pack_type, resource_count, source_repo_name?, created_at }`
**Errors**: Database read error.

#### `get_pack_detail(pack_id: String) → PackDetail`
Get full detail for a pack including all contained resources.

**Input**: Pack UUID.
**Output**: `{ pack: CapabilityPack, resources: Vec<CapabilityResource>, manifest: ManifestData }`
**Errors**: Pack not found.

#### `delete_pack(pack_id: String) → ()`
Delete a pack from the library (removes files and database record).

**Input**: Pack UUID.
**Errors**: Pack not found, file deletion error.

#### `validate_pack(pack_id: String) → ValidationResult`
Validate pack structure and integrity.

**Input**: Pack UUID.
**Output**: `{ valid: bool, errors: Vec<ValidationError> }` where `ValidationError = { resource_ref?, message }`
**Errors**: Pack not found.

### Migration Commands

#### `build_migration_plan(pack_id: String, target_repo_id: String) → MigrationPlan`
Generate a dry-run migration plan.

**Input**: Pack UUID and target repo UUID.
**Output**: `{ plan_id, source: { type, id, name }, target: { id, name }, items: Vec<MigrationPlanItem>, conflicts: Vec<Conflict>, missing_deps: Vec<Dependency> }`
**Errors**: Pack not found, target repo not found, circular reference (pack source repo == target).

#### `apply_migration_plan(plan_id: String, strategies: Vec<ConflictStrategy>) → MigrationReport`
Execute a migration plan with user-selected conflict strategies.

**Input**:
- `plan_id`: MigrationRun UUID
- `strategies`: `[{ resource_id: String, action: String }]` where action ∈ {skip, overwrite, rename, merge}

**Output**: `{ status, items: [{ resource_name, action, status, error? }], summary: { added, overwritten, skipped, failed } }`
**Errors**: Plan not found, plan already applied, disk write error.

#### `rollback_migration(run_id: String) → RollbackResult`
Roll back a previously applied migration.

**Input**: MigrationRun UUID.
**Output**: `{ success: bool, restored: u32, message: String }`
**Errors**: Run not found, run not in 'applied' state, snapshot missing/corrupted.

#### `get_migration_history(repo_id: String) → Vec<MigrationRunSummary>`
Get migration history for a repository.

**Input**: Repository UUID.
**Output**: Array of `{ id, source_name, status, items_count, created_at, executed_at }`
**Errors**: Repo not found.

### Doctor Commands

#### `run_doctor(repo_id: String) → DoctorReport`
Run health diagnostics on a repository.

**Input**: Repository UUID.
**Output**: `{ id, repo_id, score, issues: Vec<DoctorIssue>, created_at }`
**Errors**: Repo not found, capability parse error (non-fatal).

#### `compare_repo_with_pack(repo_id: String, pack_id: String) → CompareResult`
Compare a repository's capabilities against a pack.

**Input**: Repository UUID and Pack UUID.
**Output**: `{ missing: Vec<CapabilityResource>, extra: Vec<CapabilityResource>, modified: Vec<DiffItem>, same: Vec<CapabilityResource> }`
**Errors**: Repo not found, pack not found.

#### `compare_repos(repo_id_a: String, repo_id_b: String) → CompareResult`
Compare capabilities between two repositories.

**Input**: Two repository UUIDs.
**Output**: Same structure as `compare_repo_with_pack`.
**Errors**: Either repo not found.

### Settings Commands

#### `get_settings() → AppSettings`
Get current application settings.

**Output**: `{ scan_roots: Vec<String>, scan_depth: u32, ignore_patterns: Vec<String>, pack_storage_dir: String, file_watch_enabled: bool }`

#### `update_settings(settings: AppSettings) → ()`
Update application settings.

**Input**: Full or partial settings object.
**Errors**: Invalid path, directory not writable.
