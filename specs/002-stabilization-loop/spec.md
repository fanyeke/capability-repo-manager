# Feature Specification: Main Process Stabilization Loop

**Feature Branch**: `002-stabilization-loop`
**Created**: 2026-05-15
**Status**: Draft
**Input**: User description: "根据改进文档和对现有项目的审查，发现bug，结合两者编写spec — 主流程收口与闭环修复"

## User Scenarios & Testing

### User Story 1 - Stable Repository Identity Across Re-scans (Priority: P1)

A user scans their code directories, and the system discovers all Git repositories. When they scan the same directories again (e.g., after restarting the app), each repository retains the same identity. Related capability resources, doctor reports, and migration history remain correctly associated with their repository.

**Why this priority**: Every repository in the system is the root aggregate. If repo IDs change on repeat scans, all foreign key relationships (capabilities, doctor reports, migration history) break. This makes the entire product unreliable. Without stable identity, no other feature can be trusted.

**Independent Test**: Can be fully tested by scanning a directory with known Git repos, scanning the same directory again, and verifying that each repo's ID remains unchanged and that previously indexed capability resources are still associated.

**Acceptance Scenarios**:

1. **Given** a directory with 3 Git repositories, **When** the user scans it twice, **Then** each repository retains the same ID across both scans.
2. **Given** a repository that was previously scanned and has capability resources, **When** the user triggers a re-scan, **Then** all capability resources remain associated with the same repo ID.
3. **Given** a repository that was previously diagnosed by the Doctor engine, **When** a re-scan occurs, **Then** the existing doctor reports remain linked to the repository.
4. **Given** a new repository path (not previously scanned), **When** scanning, **Then** a new ID is generated and no existing repo's resources are affected.
5. **Given** repositories from two different scan roots that resolve to the same canonical path, **When** scanning completes, **Then** the repository appears only once (deduplicated by canonical path).

---

### User Story 2 - Scan Pipeline Automatically Populates Capability Inventory (Priority: P1)

A user opens the application for the first time, adds their code directory, and after scanning completes, they immediately see capability counts for each repository. They can click into a repository detail without manually triggering a refresh to see the capability inventory.

**Why this priority**: The scan is the primary entry point. If scan doesn't produce capability data, the Dashboard shows empty counts, the "has capabilities" filter is useless, and every repo requires a manual refresh. This undermines the core value proposition.

**Independent Test**: Can be tested by scanning a directory with repos that have known Claude Code configs (skills, hooks, MCP), and verifying that counts appear in the repo list immediately after scan completes, without any additional user action.

**Acceptance Scenarios**:

1. **Given** a directory with repositories that have Claude Code skills configured, **When** scanning completes, **Then** the Dashboard shows skill counts for those repositories.
2. **Given** a repository with mcp.json, hooks in settings.json, and rules in .claude/rules/, **When** scanning completes, **Then** opening the repo detail shows all capability types correctly grouped.
3. **Given** a repository with a malformed configuration file (e.g., invalid JSON in mcp.json), **When** scanning completes, **Then** the repo shows available capabilities, and the parse failure is recorded with an error indicator (not silently dropped).
4. **Given** a scan where one repo's configuration fails to parse, **When** scanning completes, **Then** other repos are unaffected and show correct capability data.
5. **Given** a scanned repository, **When** the user views the capability counts in the list, **Then** the counts match the actual number of parsed resources.

---

### User Story 3 - Refresh Error Resilience (Priority: P1)

A user has a repository with indexed capabilities. They trigger a refresh, but the capabilities can no longer be parsed (e.g., the config was temporarily corrupted). The system preserves the previously indexed capability data, updates the Git metadata, and shows an error indicator rather than silently clearing all capability information.

**Why this priority**: Without this protection, a single failed refresh destroys a repository's capability data. The user loses all context until they manually re-scan. For production use, this is unacceptable — stale-but-present data is always better than no data.

**Independent Test**: Can be tested by indexing a repo with valid configs, corrupting the config file, refreshing, and verifying that the old capability data is still visible with an error marker.

**Acceptance Scenarios**:

1. **Given** a repository with previously indexed capabilities, **When** a refresh encounters a parse error, **Then** the old capability resources are preserved in the database.
2. **Given** a repository in a "parse failed" state, **When** the user views the repo detail, **Then** a warning indicator shows that the displayed data is from a previous successful parse.
3. **Given** a repository where refresh succeeded, **When** it completes, **Then** old resources are replaced with new ones (no stale data remains).
4. **Given** a repository where Git metadata extraction succeeds but capability parsing fails, **When** refreshing, **Then** the Git metadata (branch, dirty state) is still updated.

---

### User Story 4 - Reliable Resource Detail Query (Priority: P2)

A user browses a repository's capability list and clicks on a specific resource to view its details. The resource detail panel shows the resource's metadata, source path, content hash, scope, and any parse errors. The response is fast and accurate.

**Why this priority**: The capability list is the user's primary way to understand what's configured. If clicking a row shows incorrect data or errors, the user cannot trust the application's capability inventory.

**Independent Test**: Can be tested by viewing resource detail for a skill, an MCP config, a hook, and a rule, verifying correct metadata and content is returned for each type.

**Acceptance Scenarios**:

1. **Given** a repository with known capability resources, **When** the user clicks on a resource, **Then** the detail panel shows the correct resource ID, name, type, scope, source path, and content hash.
2. **Given** a resource that has an error_message stored, **When** viewing its detail, **Then** the error is displayed.
3. **Given** a resource with rich metadata_json, **When** viewing its detail, **Then** the metadata is returned alongside the basic resource fields.
4. **Given** a non-existent resource ID, **When** requesting its detail, **Then** a "not found" error is returned (not an empty result).

---

### User Story 5 - Persistent Pack Library (Priority: P2)

A user exports a capability pack from a repository. The pack appears immediately in the Pack Library. They can close and reopen the application, and the pack is still listed with its name, version, source, and resource count. They can delete a pack, and both the database entry and disk files are cleaned up.

**Why this priority**: Packs are the core asset that enables migration. If packs disappear on restart, the entire migration workflow is non-functional. Pack library must be as persistent as repository data.

**Independent Test**: Can be tested by exporting a pack, verifying it appears in the library immediately, simulating application restart, and verifying the pack is still present and detailed.

**Acceptance Scenarios**:

1. **Given** a repository with selected resources, **When** the user exports a pack, **Then** the pack appears in the Pack Library immediately with correct name, version, and resource count.
2. **Given** an exported pack, **When** the application is restarted (simulated by recreating the store from DB), **Then** the pack is still listed and its detail is accessible.
3. **Given** an exported pack, **When** the user views its detail, **Then** the manifest, resources, and metadata are all correct.
4. **Given** an attempt to export a pack with a name and version that already exists, **When** exporting, **Then** the user receives a clear error that a duplicate pack exists.
5. **Given** an exported pack, **When** the user deletes it, **Then** the pack entry is removed from the database and the pack directory is removed from disk.
6. **Given** a pack that is deleted, **When** verifying, **Then** associated resource entries in the DB are also removed.

---

### User Story 6 - Safe Migration with Snapshot and Rollback (Priority: P2)

A user applies a capability pack to a target repository. Before execution, the system creates a snapshot of only the files that will be affected. After migration completes, the user can roll back, restoring only the affected files to their pre-migration state, without touching any other files in the repository.

**Why this priority**: Migration writes to the user's repository. Without proper rollback capability, the risk of data loss makes the migration feature unsafe for production use. A scoped snapshot ensures minimal overhead and precise restoration.

**Independent Test**: Can be tested by applying a pack to a target repo, verifying affected files, then rolling back and confirming only those files are restored.

**Acceptance Scenarios**:

1. **Given** a migration plan for a target repository, **When** the user executes the plan, **Then** a snapshot is created containing backup copies of all files that will be overwritten or deleted.
2. **Given** a successfully executed migration, **When** the user triggers rollback, **Then** the affected files are restored to their pre-migration state and any new files are removed.
3. **Given** a migration that partially fails, **When** it completes, **Then** the report accurately reflects which items succeeded and which failed (not a blanket "applied" status).
4. **Given** a migration plan with conflicts, **When** the user provides an invalid strategy action, **Then** the system rejects it with a clear error (not silently using "skip" as default).
5. **Given** a migration plan with unresolved conflicts, **When** the user tries to execute, **Then** the system rejects execution until all conflicts are resolved.
6. **Given** a migration that was already applied, **When** attempting to apply it again, **Then** the system rejects it.

---

### User Story 7 - Cascade Cleanup on Repository Removal (Priority: P2)

A user removes a repository from the application. The system cleans up not just the repo record, but also all associated capability resources and doctor reports. Migration history may be preserved with a "target missing" indicator.

**Why this priority**: Without cascade cleanup, removing a repo leaves orphan data rows that accumulate over time, causing confusion in filters, counts, and queries.

**Independent Test**: Can be tested by removing a repo that has capabilities and doctor reports, then verifying all associated data is cleaned up.

**Acceptance Scenarios**:

1. **Given** a repository with capability resources and doctor reports, **When** the user removes it, **Then** the repo, its resources, and its doctor reports are all deleted.
2. **Given** a repository with migration history, **When** removing it, **Then** migration history is preserved but the repo reference is marked as deleted/missing in the UI.

### Edge Cases

- What happens when a scan root path is a symlink that points to an already-scanned directory? (Deduplication by canonical path)
- How does the system handle a repository that is moved to a new path between scans? (Old ID retained for old path, new ID for new path)
- What happens if pack storage directory is deleted manually? (DB entries remain but validation shows pack as missing)
- How does rollback handle files that were created externally between migration and rollback? (Rollback only affects files that were part of the snapshot)
- What happens when migration encounters a file lock or permission error? (Partial failure is reported, snapshot preserves modified files)

## Requirements

### Functional Requirements

- **FR-001**: System MUST reuse existing repo IDs when the same canonical path is detected on repeat scans, instead of generating new UUIDs.
- **FR-002**: System MUST provide a method to query for an existing repository by canonical path before inserting a new record.
- **FR-003**: Scanner MUST parse capability inventories automatically after discovering and storing repository metadata.
- **FR-004**: Scan pipeline MUST handle capability parsing errors per-repository without failing the entire scan.
- **FR-005**: When capability parsing fails during a scan/refresh, the system MUST preserve previously indexed resources and record a parse error indicator.
- **FR-006**: When capability parsing succeeds during a scan/refresh, the system MUST atomically replace old resources with new resources within a single transaction.
- **FR-007**: Resource Store MUST provide a `get_by_id(resource_id)` method for direct primary key lookup.
- **FR-008**: Resource Detail query MUST use primary key lookup (not filtering by empty repo_id).
- **FR-009**: Pack Library MUST use the SQLite database as its persistent index, not just in-memory storage.
- **FR-010**: Pack export MUST atomically write both disk artifacts and database entries, cleaning up on any step failure.
- **FR-011**: Pack deletion MUST remove both database entries and disk files, including associated capability resources.
- **FR-012**: Repository deletion MUST cascade to delete associated capability resources and doctor reports.
- **FR-013**: Migration strategies MUST be validated as a restricted enum (skip, overwrite, rename, merge), rejecting invalid values.
- **FR-014**: Migration execution MUST create a scoped snapshot of only the files that will be affected (backup of originals, record of existence status). Snapshot MUST be created per-file before any write operations begin, allowing per-file rollback in case of partial failure.
- **FR-015**: Migration execution MUST check for and prevent applying an already-applied plan.
- **FR-016**: Migration run status MUST follow the state machine: `planned` → `ready` → `executing` → `success` / `partial_failure` / `failed`. Rollback transitions `success` or `partial_failure` to `rolled_back`. The report status MUST accurately reflect the actual outcome.
- **FR-017**: Rollback MUST only restore files that were part of the migration snapshot, without affecting other files in the repository.
- **FR-018**: Migration with unresolved conflicts MUST be rejected at execution time.
- **FR-019**: Application UI MUST support Chinese language display. The display language MAY be determined by system locale or user preference in settings.

### Key Entities

- **Repository**: Core aggregate root identifies a Git repository on disk by canonical path. Has stable UUID identity across scans. Tracks indexing status (never_indexed, fresh, stale, parse_failed) and last indexing error.
- **CapabilityResource**: A single Claude Code configuration artifact (skill, MCP server, hook, rule, agent, command, plugin, or settings entry). Belongs to either a repository or a pack. Has scope, content hash, and optional parse error.
- **CapabilityPack**: A named, versioned collection of CapabilityResources exported from a repository. Physically stored as a directory on disk with manifest, and indexed in SQLite for persistence.
- **MigrationPlan**: A pre-computed list of actions (add, overwrite, skip, rename, merge) for applying a pack to a target repository. Includes conflicts and dependency checks.
- **MigrationSnapshot**: A record of pre-migration state for affected files only. Contains backup paths and existence records for rollback.
- **DoctorReport**: A diagnostic report for a repository containing a health score (0-100) and list of issues with severity (critical, warning, info), recommendations, and resource references.

## Success Criteria

### Measurable Outcomes

- **SC-001**: A repository scanned 5 times consecutively retains the same ID every time, and all associated resources remain correctly associated.
- **SC-002**: After initial scan of a directory with Claude Code-configured repos, capability counts are immediately visible in the Dashboard without any additional user action.
- **SC-003**: A refresh that encounters a parse failure preserves the previous capability data and shows a clear error indicator.
- **SC-004**: Resource detail lookup for any indexed resource returns correctly within under 500ms.
- **SC-009**: Initial scan (including capability parsing) of 100 repositories completes in under 60 seconds on typical developer hardware.
- **SC-005**: An exported pack survives application restart and remains fully queryable by ID, name, type, and search.
- **SC-006**: A migration with 5 files creates a snapshot of exactly those 5 files; rollback restores them without affecting any other files in the repository.
- **SC-007**: Deleting a pack removes 100% of its DB resources and disk files with no orphan records.
- **SC-008**: Deleting a repository cascades to all associated capability resources and doctor reports with no orphan records.

## Clarifications

### Session 2026-05-15

- Q: 仓库身份识别使用哪种策略？ → A: 方案 A — Canonical Path 查找复用。扫描时先查 DB 中是否有相同 canonical path，有则复用旧 ID，无则生成新 UUID。仓库搬家后会被识别为新仓库。
- Q: MigrationRun.status 应支持哪些状态？ → A: 完整状态机 — `planned` / `ready` / `executing` / `success` / `partial_failure` / `failed` / `rolled_back`。
- Q: 迁移写入多个文件时的原子性策略？ → A: 逐文件执行，汇总报告。每个文件独立执行，成功/失败分别记录，最终状态根据整体结果判定。
- Q: 中英双语的覆盖范围？ → A: 产品 UI 支持中文。应用程序需要增加中文本地化界面支持，显示语言根据系统环境自动切换或用户手动选择。
- Q: 首次扫描含能力解析的性能目标？ → A: 100 个仓库 <60 秒。

## Assumptions

- Repositories are identified by their canonical filesystem path (resolved symlinks, normalized). Moving a repository to a new path results in a new identity. The identity strategy is Canonical Path Lookup (方案A): scan queries DB by canonical path first, reuses existing ID if found, generates new UUID if not found.
- Scan operations are sequential per-repo; parallel scanning is out of scope for this phase.
- Pack library uses SQLite as the primary index with disk storage for file content; the library.js in-memory PackStore is replaced with DB-backed queries.
- Migration rollback is scoped to the migration-affected paths only — full-repo snapshots are not needed for this phase.
- Unresolved conflicts during migration are a hard gate — the system refuses to execute until the user provides a strategy for every conflict.
- The settings command module (`settings_commands`) is separate from repo_commands and follows the same structure as other command modules.

