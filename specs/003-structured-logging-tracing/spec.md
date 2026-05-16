# Feature Specification: Structured Logging & Operation Tracing

**Feature Branch**: `003-structured-logging-tracing`  
**Created**: 2026-05-15  
**Status**: Draft  
**Input**: Debugging enhancement document

## User Scenarios & Testing

### User Story 1 — Developer can trace a scan operation end-to-end (Priority: P1)

As a developer debugging a user's issue, I want to find all log entries related to a single "scan repositories" operation so that I can quickly identify which repos were found, which failed, and why capabilities are missing.

**Why this priority**: This is the foundation for all debugging. Without operation-level traceability, every issue requires reproducing manually.

**Independent Test**: Run a scan of a known directory. All log entries for that scan share the same `operation_id`. Grepping the log file for that ID returns exactly the scan's lifecycle (start, discoveries, parse results, finish).

**Acceptance Scenarios**:

1. **Given** a fresh app start, **When** a user triggers "scan repositories", **Then** all log entries produced during this scan share a common `operation_id`
2. **Given** a scan completes, **When** I search the log for that `operation_id`, **Then** I can see scan_started, repo_discovered, parse_finished, and scan_finished entries in order
3. **Given** a scan encounters a parse error, **When** the error is logged, **Then** the log entry includes the `operation_id`, the `repo_path`, and the error details

---

### User Story 2 — Developer can trace a migration lifecycle end-to-end (Priority: P1)

As a developer supporting migration failures, I want to see the full apply/rollback lifecycle of a single migration run so that I can determine which items succeeded, which failed, and whether rollback restored correctly.

**Why this priority**: Migration errors are the hardest to debug. A single operation_id spanning plan → apply → rollback is critical.

**Independent Test**: Apply a migration to a test repo, then rollback. All log entries share the same `operation_id`. The log captures snapshot creation, per-file execution results, and rollback restoration.

**Acceptance Scenarios**:

1. **Given** a migration plan is built, **When** the plan is applied, **Then** the log contains migration_apply_started, snapshot_created, per-file results, and migration_apply_finished with final status
2. **Given** a migration is rolled back, **When** rollback completes, **Then** the log contains rollback_started, restored items, deleted items, and rollback_finished
3. **Given** a migration step fails, **When** the error is logged, **Then** it includes error code, operation_id, run_id, and resource_id

---

### User Story 3 — Developer can export a debug bundle for issue reproduction (Priority: P2)

As a developer receiving a bug report, I want the user to be able to export a debug bundle containing logs, configuration, and recent operation summaries so that I can diagnose issues without asking the user to run terminal commands.

**Why this priority**: Debug bundles reduce the back-and-forth in issue triage. They are the primary tool for non-developer users to provide useful diagnostic data.

**Independent Test**: User clicks "Export Debug Bundle" in Settings. A zip file is created containing recent logs, redacted settings, migration summaries, and doctor summaries.

**Acceptance Scenarios**:

1. **Given** the debug bundle feature is available, **When** a user triggers export, **Then** a `.zip` file is saved to a chosen location
2. **Given** the bundle is created, **When** I inspect its contents, **Then** it contains app logs (last 7 days), redacted settings, migration run summaries, and doctor report summaries
3. **Given** the bundle contains path data, **When** the "redact paths" option is enabled, **Then** all `/home/username` prefixes are replaced with `~`

---

### User Story 4 — User can see operation activity history (Priority: P3)

As a user, I want to see a chronological list of recent operations (scans, exports, migrations, doctors) so that I can understand what the application has been doing and refer to past results.

**Why this priority**: Activity history improves user confidence and reduces confusion about "did my last operation succeed?"

**Independent Test**: After performing a scan and a pack export, the Activity page lists both operations with their status, timestamps, and summaries.

**Acceptance Scenarios**:

1. **Given** a user has performed several operations, **When** they open the Activity page, **Then** they see a reverse-chronological list of operations
2. **Given** an operation failed, **When** viewing the activity list, **Then** the failed operation is visually distinguished from successful ones

---

### Edge Cases

- What happens when the log directory is not writable? App should fall back to console-only logging and show a warning.
- How does the system handle extremely large log files (>100MB)? Log rotation should prevent unbounded growth.
- What happens when sensitive data (tokens, API keys) appears in error messages? The redaction layer should strip them before writing to log files.
- How does the app behave when tracing dependencies fail to initialize? The app should fall back to basic stdout logging.
- What happens when the debug bundle export directory is read-only? Show a clear error message and suggest an alternative directory.

## Requirements

### Functional Requirements

- **FR-001**: System MUST produce structured log entries with timestamp, level, event name, and operation_id for all key operations
- **FR-002**: System MUST assign a unique `operation_id` to each user-triggered high-level operation (scan, refresh, export, migrate, rollback, doctor)
- **FR-003**: System MUST propagate the `operation_id` through all sub-steps of that operation
- **FR-004**: System MUST write logs to both console (stdout) and rotating files under `~/.capability-repo-manager/logs/`
- **FR-005**: Log files MUST rotate daily and retain the most recent 14 days of logs
- **FR-006**: System MUST support configurable log levels: ERROR, WARN, INFO, DEBUG, TRACE
- **FR-007**: System MUST redact sensitive patterns (tokens, API keys, bearer auth headers) from log output before writing to files
- **FR-008**: System MUST NOT write `.env` file contents or environment variable values (variable names are OK) to logs
- **FR-009**: System MUST provide an "Export Debug Bundle" function that creates a `.zip` file containing logs, redacted settings, migration summaries, and doctor report summaries
- **FR-010**: System MUST log the following events with structured fields:
  - App startup: version, platform, settings path, DB path, log directory
  - Scan: root paths, max_depth, repos_found, repos_added, repos_updated, parse_failed
  - Pack export: pack name, version, selected resource count, output dir, manifest path, warnings
  - Pack delete: pack_id, storage_dir, safety check result
  - Migration plan: plan_id, source_id, target_repo_id, item count, conflict count
  - Migration apply: plan_id, snapshot_path, per-item results, final status
  - Migration rollback: run_id, restored items, removed items
  - Doctor: repo_id, score, issue counts by severity
- **FR-011**: System MUST persist `operation_events` to a database table for queryable activity history, with fields: operation_id, operation_type, status, repo_id, pack_id, migration_run_id, summary, created_at
- **FR-012**: Activity history (from operation_events) MUST be viewable from within the application
- **FR-013**: Error logs for failed operations MUST include: error code, operation_id, module name, and relevant entity IDs (repo_id/pack_id/run_id)
- **FR-014**: System MUST provide a log level setting in the Settings page (options: INFO, DEBUG, TRACE) that takes effect without restart
- **FR-015**: System MUST provide a "Open Log Directory" button in Settings that opens the log folder in the system file manager

### Key Entities

- **OperationEvent**: A persistent record of a user-triggered operation, with operation_id (UUID), operation_type, status, associated entity IDs, summary text, and timestamp
- **LogEntry**: A structured log record with timestamp, level, event name (e.g., scan_started), operation_id, module, and structured fields. Stored in files, not in DB.
- **DebugBundle**: An on-demand export package containing logs, redacted settings, operation summaries, and system metadata, packaged as a .zip file

## Success Criteria

### Measurable Outcomes

- **SC-001**: A developer can find all log entries for any single scan operation by grepping its `operation_id` — all entries from scan_started to scan_finished share the same ID
- **SC-002**: A developer can trace a full migration apply + rollback lifecycle from a single `operation_id` search
- **SC-003**: Users can export a debug bundle in under 30 seconds from the Settings page
- **SC-004**: Log files do not exceed 100MB total for 14 days of normal usage (daily rotation + reasonable INFO-level volume)
- **SC-005**: Sensitive patterns (token-like strings and bearer auth) are absent from all log files written to disk
- **SC-006**: Activity history page loads and displays the last 50 operations in under 1 second
- **SC-007**: Log level changes from Settings take effect within 5 seconds without restart

## Assumptions

- The existing `log` crate usage will be supplemented (not replaced) by a structured logging layer for event-based logging
- Log directory `~/.capability-repo-manager/logs/` is created on app startup if it doesn't exist
- Debug bundle export uses the system-native "Save File" dialog via Tauri
- Operation events are stored in a new `operation_events` SQLite table (not in the log files)
- The feature assumes standard desktop environment with write access to the user's home directory
- This feature does NOT include an in-app log viewer beyond the Activity History page
- Sensitive pattern redaction uses regex-based pattern matching
