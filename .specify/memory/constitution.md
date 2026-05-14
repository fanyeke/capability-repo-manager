# Capability Repo Manager Constitution

## Core Principles

### I. TDD First (NON-NEGOTIABLE)

Test-Driven Development is mandatory for all code. Red-Green-Refactor cycle strictly enforced:
- Tests written first → Tests fail (Red)
- Minimal implementation → Tests pass (Green)
- Refactor → All tests still pass
- Minimum 80% test coverage across unit, integration, and E2E tests

### II. Repo-First Architecture

Git repositories are the primary organizing boundary for all capabilities:
- Every capability resource is anchored to a repository
- Repository discovery drives the user experience entry point
- Repository metadata (branch, commit, dirty state) is core domain data

### III. Dry-Run by Default

All write operations must be previewable before execution:
- Migration plans generated and reviewed before any file writes
- Pack export previews resource selection before files are created
- Conflict resolution strategies explicitly chosen by user before execution

### IV. Domain-Driven Design

Domain logic is separated from I/O, platform, and storage concerns:
- Core domain types and business rules live in independent crates
- Parsers, scanners, engines are pure-logic abstractions
- Platform-specific code (Tauri bridge, filesystem, Git CLI) wraps domain layer
- All domain functions must be independently testable without I/O

### V. Local-First & Safe

The application operates locally with explicit user consent for actions:
- Default operation mode is read-only (scan, inventory, compare)
- Write operations (export, migrate) require explicit user trigger and confirmation
- Unknown hooks are never auto-executed; only static analysis performed
- No automatic network calls; all remote interaction is user-initiated

### VI. Cross-Platform Core with Linux Priority

Core logic is platform-agnostic; Linux is the MVP platform:
- File path handling must be cross-platform abstracted
- Git CLI interactions avoid hardcoded shell assumptions
- Platform-specific code isolated in dedicated modules
- Linux first, macOS/Windows as subsequent targets

## Quality Gates

- All code changes must pass CI: lint → typecheck → unit tests → integration tests → E2E tests
- No code merges without peer review (via code-reviewer agent)
- Security-sensitive paths (migration, hook analysis) require security reviewer sign-off
- Feature branches named with sequential prefix: `NNN-feature-name`

## Development Workflow

- 1. Spec defined → 2. Constitution verified → 3. Plan written → 4. Tasks generated → 5. TDD implementation → 6. Code review → 7. Merge
- Each milestone delivers an independently testable slice of functionality
- P1 stories are MVP-critical and must be completed before P2/P3

**Version**: 1.0.0 | **Ratified**: 2026-05-14 | **Last Amended**: 2026-05-14
