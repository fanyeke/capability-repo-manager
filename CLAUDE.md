This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Capability Repo Manager — a Tauri 2 desktop app (Rust backend + Svelte 5 frontend) that discovers local Git repositories, catalogs Claude Code capabilities (skills, MCP servers, hooks, rules, agents), exports capability packs, migrates packs between repos with dry-run planning and conflict resolution, and diagnoses config health via a weighted scoring engine.

## Commands

### Rust (backend core)

```bash
cargo build --workspace          # Build all crates
cargo check --workspace          # Type-check without full compilation
cargo test --workspace           # Run all Rust tests (unit + integration)
cargo test -p <crate-name>       # Run tests for a single crate
cargo test -- <test_name>        # Run a specific test by name
cargo clippy --workspace -- -D warnings  # Lint (matches CI)
cargo fmt --all -- --check       # Format check (matches CI)
```

### Frontend (Svelte 5 + TypeScript)

```bash
cd frontend
pnpm install                     # Install dependencies
pnpm dev                         # Start Vite dev server (port 1420)
pnpm build                       # Production build
pnpm check                       # Type-check (svelte-check)
pnpm lint                        # Prettier + ESLint
pnpm test                        # Vitest unit tests
pnpm test:watch                  # Vitest watch mode
pnpm test:e2e                    # Playwright E2E tests
```

### Tauri desktop (full app)

```bash
cargo tauri dev                   # Launch Tauri dev with hot reload
cargo tauri build                 # Build production bundle
```

## Architecture

### Layer Structure (Rust)

The backend follows strict domain-driven layering. Each layer has its own crate.

```
crates/domain         #  [PURE TYPES] — no I/O, no dependencies on other crates
crates/claude-parser  #  [PARSER]     — parse Claude Code config files (skills, MCP, hooks, rules, agents)
crates/repo-scanner   #  [SCANNER]    — discover Git repos, extract git metadata
crates/git-service    #  [SERVICE]    — git CLI abstraction (git2 or shell outs)
crates/pack-engine    #  [ENGINE]     — pack creation, validation, manifest, library management
crates/migration-engine # [ENGINE]    — migration planning, conflict detection, execution, rollback
crates/doctor-engine  #  [ENGINE]     — diagnostic checks, health scoring, drift detection
crates/storage        #  [STORAGE]    — SQLite persistence via rusqlite (repo pattern)
crates/tauri-bridge   #  [GLUE]       — Tauri command handlers, delegates to engines
```

**Key rules:**
- `domain` has zero crate dependencies. Pure structs + serde.
- Engine crates depend on `domain`. They import domain types but never depend on `storage` or `tauri-bridge`.
- `storage` depends on `domain`. Contains `repo_store`, `resource_store`, pack store, migration run store.
- `tauri-bridge` is the outermost layer: depends on everything, owns `AppState` (Mutex-wrapped Database + Settings).
- `git-service` is standalone — no domain dependency, used by `repo-scanner`.

### Frontend Structure

```
frontend/src/
├── main.ts                 # Svelte 5 mount entry point
├── App.svelte              # Router (page switching via currentPage store)
├── lib/
│   ├── types.ts            # TypeScript types mirroring Rust domain
│   ├── stores/             # Svelte stores (repoStore, packStore, doctorStore, etc.)
│   ├── components/         # Reusable UI components (RepoList, RepoCard, CapabilityList, etc.)
│   └── pages/              # Page-level components (Dashboard, RepoDetail, PackExport, etc.)
└── tests/unit/             # Vitest unit tests
```

### Key Design Decisions

1. **TDD with 80%+ coverage** — all crates have standalone tests. CI runs Rust unit + integration tests, Vitest frontend tests, and (when enabled) Playwright E2E.
2. **Dry-run by default** — migration engine separates `planner` (generates plan) from `executor` (applies plan). Writes always require preview + user confirmation.
3. **Local-first** — no network calls for core features. All data stored in local SQLite at `~/.capability-repo-manager/data.db`.
4. **Domain/IO separation** — pure functions in `domain` and engine crates; I/O concentrated in `storage` and `tauri-bridge`.
5. **App routing via store** — `App.svelte` switches pages based on `currentPage` store value. No Svelte Router needed.

### CI Pipeline (.github/workflows/ci.yml)

| Job | Command |
|-----|---------|
| lint | `cargo fmt --all -- --check` + `cargo clippy --workspace -- -D warnings` |
| typecheck | `cargo check --workspace` |
| unit-test | `cargo test --workspace` (after typecheck) |
| integration-test | `cargo test --workspace --test '*'` (after typecheck) |
| e2e-test | disabled (`if: false`) — requires Tauri shell |

### Test Fixtures

`tests/fixtures/` contains test Git repos with known Claude Code configurations:
- `repo-basic/` — minimal config
- `repo-full/` — all capability types present
- `repo-broken/` — malformed configs (for error handling tests)
- `repo-conflict/` — configs with naming conflicts (for migration tests)

## Important Conventions

- Wire up new Tauri commands in `crates/tauri-bridge/src/commands/` then register in `frontend/src-tauri/src/lib.rs`
- Rust commands use snake_case names (e.g., `scan_repositories`); TypeScript invokes match literally
- The `CLAUDE.md` at repo root is updated by Spec Kit (`/speckit-*` skills) — do not overwrite the SPECKIT section
- Svelte 5 uses `mount()` instead of `new App()`, and `$state`/`$derived` runes in components (stores use writable/derived from `svelte/store`)

<!-- SPECKIT START -->
Current feature: specs/003-structured-logging-tracing/plan.md — Structured Logging & Operation Tracing
- Spec: specs/003-structured-logging-tracing/spec.md
- Data Model: specs/003-structured-logging-tracing/data-model.md
- Quickstart: specs/003-structured-logging-tracing/quickstart.md
- Contracts: specs/003-structured-logging-tracing/contracts/
<!-- SPECKIT END -->
