# Research: Structured Logging & Operation Tracing

**Date**: 2026-05-15

## Decisions

### Decision 1: Use `tracing` crate for structured logging

**Decision**: Adopt `tracing` + `tracing-subscriber` + `tracing-appender` for the logging framework.

**Rationale**:
- Supports structured fields (key=value pairs) out of the box, unlike `log` crate which only supports format strings
- Span-based tracing maps directly to operation_id propagation
- `tracing-subscriber` provides layered filtering (env-filter, log level)
- `tracing-appender` provides file rotation (daily/non-blocking)
- Coexists with existing `log` crate usage via `tracing`'s log compatibility layer
- Industry standard for Rust observability (used by tokio, axum, etc.)

**Alternatives considered**:
- `log` crate only: No structured fields, operation_id would need manual formatting
- `slog`: More complex API, less ecosystem support than tracing
- `fern`: File rotation only, no structured logging or span support

### Decision 2: OperationContext as a domain struct

**Decision**: Define `OperationContext` in the domain crate with operation_id, operation_type, and optional entity IDs.

**Rationale**:
- Domain layer has zero dependencies, so OpContext is usable by all crates
- Pure struct + serialization, no I/O
- Follows existing domain pattern (Repository, MigrationPlan, etc.)
- Passed through function calls from bridge → engine → store layers

### Decision 3: Redaction as a pure utility function

**Decision**: Implement `redact_sensitive(input: &str) -> String` as a pure function in the domain crate.

**Rationale**:
- No I/O or dependencies needed
- Regex-based pattern matching for tokens, API keys, bearer auth
- Can be unit tested exhaustively
- Usable in any layer before log output

### Decision 4: Operation events stored in SQLite

**Decision**: Store operation events in a new `operation_events` table in the existing SQLite database.

**Rationale**:
- No new storage infrastructure needed
- Operation events are inherently local (single user, no sync)
- SQLite querying is sufficient for activity history (low volume, ~50 events/day)
- Follows existing pattern (repositories, resources, packs all in SQLite)

**Alternatives considered**:
- Separate log file: Harder to query, no structured access for Activity page
- In-memory only: Would not survive restart, violating FR-011

### Decision 5: Debug bundle as server-side zip assembly

**Decision**: Debug bundle is assembled by the Rust backend and saved via Tauri's file dialog.

**Rationale**:
- Backend has direct file system access to logs, DB, and settings
- Tauri's dialog API provides native save dialogs
- zip crate handles compression without external dependencies
- Redaction can be applied at assembly time
