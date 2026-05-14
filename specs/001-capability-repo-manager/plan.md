# Implementation Plan: Capability Repo Manager

**Branch**: `001-capability-repo-manager` | **Date**: 2026-05-14 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-capability-repo-manager/spec.md`

## Summary

Build a local desktop application (Linux MVP) that discovers local Git repositories, catalogs their Claude Code capabilities (skills, MCP servers, hooks, rules, agents, commands, plugins, settings), exports capability packs, migrates packs between repositories with dry-run planning and conflict resolution, and diagnoses configuration health via a weighted scoring doctor engine. The application follows TDD methodology with 80%+ coverage targets and a domain-driven modular architecture.

## Technical Context

**Language/Version**: Rust 1.75+ (backend core), TypeScript 5.x (frontend), Svelte 4.x (UI)
**Primary Dependencies**: Tauri 2 (desktop shell), SQLite (via rusqlite), serde (serialization), notify (file watch), git2 or git CLI (Git access)
**Storage**: SQLite (local database for repo metadata, capability resources, packs, migration history, doctor reports)
**Testing**: cargo test (Rust unit + integration), Vitest (frontend unit), Playwright (E2E), fixture-based golden file tests for parsers and engines
**Target Platform**: Linux (MVP), architecture supports future macOS/Windows ports
**Project Type**: Desktop application (Tauri 2 shell: Rust backend + Svelte frontend)
**Performance Goals**: 500 repos scanned in <30s, repo detail view renders <10s, pack export <2min, migration <3min, rollback <1min
**Constraints**: Local-only operation, no network calls for core features, read-only by default, all writes require pre-execution preview and user confirmation
**Scale/Scope**: Single-user local desktop tool, 500 repos max, ~50 packs in library, packs contain up to ~100 resources each

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. TDD First (NON-NEGOTIABLE) | PASS | Spec defines user stories with acceptance scenarios → test cases. Domain/IO separation enables pure-function testing. Plan includes unit + integration + E2E layers with 80%+ target. |
| II. Repo-First Architecture | PASS | All domain entities (Repository, CapabilityResource, CapabilityPack) are repo-anchored. Repo catalog is the P1 entry point. Data model centers on Repository as root aggregate. |
| III. Dry-Run by Default | PASS | FR-013 requires dry-run plan before writes. FR-025 requires preview + confirmation. Migration engine design separates planning from execution. |
| IV. Domain-Driven Design | PASS | Architecture splits into domain/, parser/, engine/ crates (pure logic) separated from storage/, tauri-bridge/ (I/O). Each create is independently testable. |
| V. Local-First & Safe | PASS | Assumptions confirm: no cloud, no remote services, no auto-network. Spec edge cases cover permission denied, disk full. Doctor does static analysis only (no hook execution). |
| VI. Cross-Platform Core | PASS | File path abstraction in domain layer. Git CLI interaction via dedicated git-service crate. Linux MVP but architecture portable. |

**Gate Result**: ALL PASS — Proceed to Phase 0.

## Project Structure

### Documentation (this feature)

```text
specs/001-capability-repo-manager/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (Tauri command contracts)
└── tasks.md             # Phase 2 output (/speckit-tasks)
```

### Source Code (repository root)

```text
crates/
├── domain/              # Core domain types & traits (no I/O)
│   ├── src/
│   │   ├── lib.rs
│   │   ├── repository.rs
│   │   ├── capability.rs
│   │   ├── pack.rs
│   │   ├── migration.rs
│   │   └── doctor.rs
│   └── tests/
├── repo-scanner/        # Git repo discovery + metadata extraction
│   ├── src/
│   │   ├── lib.rs
│   │   ├── scanner.rs
│   │   └── metadata.rs
│   └── tests/
├── git-service/         # Git CLI abstraction layer
│   ├── src/
│   │   ├── lib.rs
│   │   └── git_cli.rs
│   └── tests/
├── claude-parser/       # Claude Code config parser (skills/MCP/hooks/rules/agents)
│   ├── src/
│   │   ├── lib.rs
│   │   ├── skill_parser.rs
│   │   ├── mcp_parser.rs
│   │   ├── hook_parser.rs
│   │   ├── rule_parser.rs
│   │   └── agent_parser.rs
│   └── tests/
├── pack-engine/         # Pack creation, validation, manifest, library
│   ├── src/
│   │   ├── lib.rs
│   │   ├── manifest.rs
│   │   ├── packer.rs
│   │   ├── validator.rs
│   │   └── library.rs
│   └── tests/
├── migration-engine/    # Migration planning, conflict detection, execution, rollback
│   ├── src/
│   │   ├── lib.rs
│   │   ├── planner.rs
│   │   ├── conflict.rs
│   │   ├── executor.rs
│   │   └── rollback.rs
│   └── tests/
├── doctor-engine/       # Diagnostic rules, health scoring, drift detection
│   ├── src/
│   │   ├── lib.rs
│   │   ├── checks.rs
│   │   ├── scoring.rs
│   │   └── drift.rs
│   └── tests/
├── storage/             # SQLite persistence layer (repository pattern)
│   ├── src/
│   │   ├── lib.rs
│   │   ├── repo_store.rs
│   │   ├── resource_store.rs
│   │   ├── pack_store.rs
│   │   └── migrations/
│   └── tests/
└── tauri-bridge/        # Tauri command handlers (glue layer)
    ├── src/
    │   ├── lib.rs
    │   ├── commands/
    │   │   ├── repo_commands.rs
    │   │   ├── capability_commands.rs
    │   │   ├── pack_commands.rs
    │   │   ├── migration_commands.rs
    │   │   └── doctor_commands.rs
    │   └── state.rs
    └── tests/

frontend/
├── src/
│   ├── lib/
│   │   ├── components/
│   │   │   ├── RepoList.svelte
│   │   │   ├── RepoCard.svelte
│   │   │   ├── CapabilityList.svelte
│   │   │   ├── ResourceDetail.svelte
│   │   │   ├── PackLibrary.svelte
│   │   │   ├── MigrationPlan.svelte
│   │   │   ├── ConflictResolver.svelte
│   │   │   ├── DoctorReport.svelte
│   │   │   └── CompareView.svelte
│   │   ├── pages/
│   │   │   ├── Dashboard.svelte
│   │   │   ├── RepoDetail.svelte
│   │   │   ├── PackExport.svelte
│   │   │   ├── PackApply.svelte
│   │   │   ├── Doctor.svelte
│   │   │   ├── Compare.svelte
│   │   │   └── Settings.svelte
│   │   ├── stores/
│   │   │   ├── repoStore.ts
│   │   │   ├── capabilityStore.ts
│   │   │   ├── packStore.ts
│   │   │   └── uiStore.ts
│   │   └── types.ts
│   └── tests/
│       ├── unit/
│       └── e2e/

tests/
├── fixtures/            # Test Git repos with known Claude Code configs
│   ├── repo-basic/
│   ├── repo-full/
│   ├── repo-broken/
│   └── repo-conflict/
├── integration/
│   ├── scan_flow.rs
│   ├── pack_flow.rs
│   ├── migration_flow.rs
│   └── doctor_flow.rs
└── e2e/
    ├── scan-and-browse.spec.ts
    ├── export-pack.spec.ts
    ├── migrate-pack.spec.ts
    └── diagnose.spec.ts
```

**Structure Decision**: Desktop app with Rust backend (crates/) and Svelte frontend (frontend/). Rust side follows domain-driven design with 8 independent crates — domain (pure types), 5 engine crates (scanner, parser, pack, migration, doctor), storage (SQLite), and tauri-bridge (glue). Each crate has its own tests/. Frontend follows Svelte conventions with pages/components/stores separation. Top-level tests/ holds shared fixtures, integration tests, and E2E specs.

## Complexity Tracking

> No violations to justify. All principles pass.
