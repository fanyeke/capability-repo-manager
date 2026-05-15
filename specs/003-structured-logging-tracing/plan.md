# Implementation Plan: Structured Logging & Operation Tracing

**Branch**: `003-structured-logging-tracing` | **Date**: 2026-05-15 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `specs/003-structured-logging-tracing/spec.md`

## Summary

Add structured logging infrastructure (tracing-based), operation_id traceability, per-flow instrumentation, sensitive data redaction, configurable log levels, debug bundle export, and activity history. The feature spans Rust backend (tracing integration, OpContext, redaction, logging instrumentation) and Svelte frontend (Activity page, Settings UI, Debug Bundle export).

## Technical Context

**Language/Version**: Rust 1.75+ (backend), TypeScript 5.x (frontend), Svelte 5 (UI)
**Primary Dependencies**: tracing + tracing-subscriber + tracing-appender (structured logging), serde (serialization), rusqlite (operation_events table), chrono (timestamps)
**Storage**: Log files at `~/.capability-repo-manager/logs/` (rotating files). Operation events in SQLite `operation_events` table.
**Testing**: cargo test (Rust unit + integration), Vitest (frontend)
**Target Platform**: Linux (MVP desktop app via Tauri)
**Project Type**: Tauri 2 desktop application (Rust backend + Svelte frontend)
**Performance Goals**: Log writing <1ms overhead per operation. Activity history page loads <1s for 50 entries.
**Constraints**: Log files <100MB for 14 days. Sensitive patterns MUST be redacted from file output. No network calls for logging.
**Scale/Scope**: Single-user local desktop tool. ~50 operation events/day typical volume.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. TDD First (NON-NEGOTIABLE) | PASS | Test tasks included for: OpContext, redaction, event store, debug bundle |
| II. Repo-First Architecture | PASS | Operation events reference repo_id as primary entity. Logging is infrastructure layer, not domain. |
| III. Dry-Run by Default | PASS | No write operations introduced. Logging and tracing are observability, not mutation. |
| IV. Domain-Driven Design | PASS | OperationContext and redaction are pure functions. Logging instrumentation wraps existing engines. |
| V. Local-First & Safe | PASS | Log files local-only. Debug bundle is user-triggered export. Redaction prevents secret leakage. |
| VI. Cross-Platform Core with Linux Priority | PASS | tracing is cross-platform. File paths use std::path. Log rotation works on all platforms. |

**Gate Result**: ALL PASS — Proceed.

## Serial vs Parallel Design

### Dependency Graph

```
                    ┌──────────────────────────────────────┐
                    │  L1: Foundation (serial internal)    │
                    │  ├── tracing init + file rotation    │
                    │  ├── OperationContext + redaction    │
                    │  └── operation_events table/schema   │
                    └──────────────┬───────────────────────┘
                                   │ (L1 must complete first)
                   ┌───────────────┴───────────────────────┐
                   │                                       │
    ┌──────────────▼──────────────┐    ┌───────────────────▼───────────────┐
    │  L2a: Scan Logging          │    │  L2b: Migration Logging          │
    │  (parallel with L2b/L2c)    │    │  (parallel with L2a/L2c)         │
    └─────────────────────────────┘    └───────────────────────────────────┘
                   │                                       │
    ┌──────────────▼──────────────┐    ┌───────────────────▼───────────────┐
    │  L2c: Pack/Doctor Logging   │    │  L2d: Error Enhancement          │
    │  (parallel with L2a/L2b)    │    │  (depends on L2a-L2c)            │
    └─────────────────────────────┘    └───────────────────────────────────┘
                   │                                       │
                   └───────────────┬───────────────────────┘
                                   │ (L2 must complete first)
                    ┌──────────────┴──────────────┐
                    │                              │
    ┌───────────────▼──────────────┐  ┌────────────▼────────────────┐
    │  L3a: Debug Bundle Export   │  │  L3b: Activity History UI   │
    │  (parallel with L3b)        │  │  + Settings Log Config      │
    └──────────────────────────────┘  └─────────────────────────────┘
```

### Serial Chains (cannot parallelize)

| Chain | Reason |
|-------|--------|
| L1 framework → L2 instrumentation → L3 UI | Logging infrastructure must exist before flow instrumentation. Instrumentation must exist before error enhancement. |
| OpContext struct → flow instrumentation | Context type must be defined before it can be propagated through flows. |
| operation_events table → Activity History page | DB schema must exist before frontend can query it. |

### Parallel Opportunities

| Group | Tasks | Why Parallel |
|-------|-------|-------------|
| L2a + L2b + L2c | Scan logging, Migration logging, Pack/Doctor logging | Each touches different engine/command files. No shared state beyond OpContext. |
| L1 independent | OpContext + redaction + event table schema | All are separate files/domains. OpContext is Rust struct, redaction is utility fn, event table is SQL migration. |
| L3a + L3b | Debug Bundle export + Activity History UI | Bundle is backend command, Activity is frontend page. Completely independent. |
| Settings UI items | Log level selector + Open Log Dir button + Log level dynamic switch | All touch settings_commands/settings page but are independent UI additions. |

## Project Structure

### Documentation (this feature)

```text
specs/003-structured-logging-tracing/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit-tasks)
```

### Source Code Changes (by crate)

```text
crates/domain/                       # Minimal: OpContext + Redaction
├── src/op_context.rs                # NEW: OperationContext struct
├── src/redact.rs                    # NEW: sensitive data redaction
└── src/lib.rs                       # Register modules

crates/storage/                      # New: operation_events table
├── src/lib.rs                       # DB schema: CREATE TABLE operation_events
├── src/event_store.rs               # NEW: insert_event, list_events, get_by_operation

crates/tauri-bridge/                 # Heavy: logging instrumentation
├── src/commands/repo_commands.rs    # Add scan_started/finished logging, OpContext
├── src/commands/pack_commands.rs    # Add export/delete logging
├── src/commands/migration_commands.rs  # Add plan/apply/rollback logging
├── src/commands/doctor_commands.rs  # Add doctor_started/finished logging
├── src/commands/settings_commands.rs   # Add log level config
├── src/lib.rs                       # tracing init, subscriber setup
└── src/state.rs                     # Add log_level to AppSettings

frontend/                            # Activity History + Debug Bundle + Settings
├── src/lib/pages/Activity.svelte    # NEW: operation event list
├── src/lib/pages/Settings.svelte    # Add log level selector, Open Log Dir, Export Debug Bundle
├── src/lib/stores/eventStore.ts     # NEW: operation event store
├── src/lib/types.ts                 # Add OperationEvent type
└── src/App.svelte                   # Add Activity route

tests/                               # Integration tests
├── integration/
│   └── logging_tests.rs             # NEW: trace_id propagation, redaction, debug bundle
```

**Structure Decision**: Changes spread across existing crate boundaries. One new domain module (`op_context`, `redact`). One new storage module (`event_store`). One new frontend store + page (`eventStore`, `Activity`). No new crates.

## Complexity Tracking

> No violations to justify. All 6 principles pass without needing exceptions.

## Generated Artifacts Checklist

| Artifact | Location | Status |
|----------|----------|--------|
| plan.md | specs/003-structured-logging-tracing/plan.md | ✅ Done |
| research.md | specs/003-structured-logging-tracing/research.md | 🔲 Phase 0 |
| data-model.md | specs/003-structured-logging-tracing/data-model.md | 🔲 Phase 1 |
| contracts/ | specs/003-structured-logging-tracing/contracts/ | 🔲 Phase 1 |
| quickstart.md | specs/003-structured-logging-tracing/quickstart.md | 🔲 Phase 1 |
| tasks.md | specs/003-structured-logging-tracing/tasks.md | 🔲 Run `/speckit-tasks` |
