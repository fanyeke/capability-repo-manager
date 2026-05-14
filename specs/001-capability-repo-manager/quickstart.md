# Quickstart: Capability Repo Manager Development

**Date**: 2026-05-14 | **Phase**: 1 - Design & Contracts

## Prerequisites

- Rust 1.75+ (`rustup default stable`)
- Node.js 20+ and pnpm
- Tauri 2 CLI (`cargo install tauri-cli --version "^2"`)
- Git 2.40+
- Linux desktop environment (MVP target)

## Setup

```bash
# Clone and enter project
cd capability-repo-manager

# Install frontend dependencies
pnpm install

# Build Rust crates (first time takes a while)
cargo build

# Run in development mode
cargo tauri dev
```

## Project Layout

```
crates/           # Rust backend (8 crates, domain-driven)
frontend/         # Svelte frontend (Tauri webview)
tests/            # Shared test fixtures, integration, E2E
specs/            # Feature specifications and plans
```

## Development Workflow

### 1. TDD Cycle (per crate)

```bash
# Write a failing test
# Edit crates/<crate>/tests/<test_file>.rs

# Run tests for that crate only
cargo test -p <crate>

# Implement until green
# Refactor, keep green

# Check coverage
cargo tarpaulin -p <crate> --out Html
```

### 2. Running Specific Tests

```bash
# Unit tests for a single crate
cargo test -p domain

# Integration tests with fixtures
cargo test -p repo-scanner --test integration

# Frontend unit tests
cd frontend && pnpm vitest run

# Frontend E2E tests
cd frontend && pnpm playwright test
```

### 3. Building for Production

```bash
# Release build
cargo tauri build

# Output: .deb package for Linux
```

## Test Fixture Repositories

The `tests/fixtures/` directory contains pre-built Git repositories:

| Fixture | Description |
|---------|-------------|
| `repo-basic` | Single skill + basic settings.json |
| `repo-full` | All resource types (skills, MCP, hooks, rules, agents) |
| `repo-broken` | Malformed JSON, missing files, broken references |
| `repo-conflict` | Overlapping resource names for migration testing |

Create fixtures by setting up real `.git` repos with `.claude/` directories.

## Architecture Principles

1. **Domain first**: Types in `crates/domain/` have no dependencies on other crates
2. **I/O at edges**: Filesystem, Git CLI, SQLite access only in `git-service/`, `storage/`, `tauri-bridge/`
3. **Each crate is testable standalone**: No circular dependencies between crates
4. **Commands are thin**: Tauri command handlers validate input and delegate to engines

## Crate Dependency Graph

```
domain           (no deps)
  ↑
  ├── repo-scanner      → domain
  ├── git-service       → domain
  ├── claude-parser     → domain
  ├── pack-engine       → domain
  ├── migration-engine  → domain
  ├── doctor-engine     → domain
  └── storage           → domain

tauri-bridge → domain, repo-scanner, claude-parser, pack-engine,
               migration-engine, doctor-engine, storage, git-service
```
