# Research: Capability Repo Manager

**Date**: 2026-05-14 | **Phase**: 0 - Outline & Research

## R1: Tauri 2 Desktop App Architecture

**Decision**: Tauri 2 with Rust backend monolith (multiple crates) + Svelte frontend, communicating via Tauri `invoke` commands.

**Rationale**:
- Tauri 2 supports multi-crate Rust projects natively
- Svelte is lightweight, compiles to vanilla JS, ideal for a performance-focused desktop tool
- SQLite via `rusqlite` crate gives direct, zero-network persistence
- The PRD explicitly recommends this stack; it aligns with project goals (Linux desktop, offline-first, local-only)

**Alternatives considered**:
- Electron: heavier bundle, higher memory usage, overkill for this scope
- Pure CLI: limits UX for inventory browsing, comparison views, and dry-run previews
- Web app: requires server, violates local-first principle

## R2: Git Repository Discovery Performance

**Decision**: Hybrid approach — `walkdir` crate with depth-limited BFS, combined with `git2` for metadata extraction.

**Rationale**:
- `walkdir` is the fastest Rust filesystem traversal library, handles symlinks and permission errors natively
- Depth limit (default 5) drastically reduces traversal time — most repos live at depth ≤5
- Default ignore list (node_modules, .venv, vendor, .cache, build) skips large directory trees
- `git2` (libgit2 bindings) is faster than spawning `git` CLI for metadata (branch, commit, dirty state) — avoids process overhead
- Fallback to `git` CLI for operations not easily done via libgit2 (e.g., worktree detection)

**Alternatives considered**:
- Pure `git` CLI: process spawning per repo is too slow for 500 repos
- `ignore` crate: adds `.gitignore` support but slower — not needed since we skip known non-code dirs
- `jwalk` (parallel walkdir): adds complexity, benefits only on large single-directory trees; simpler to parallelize at repo level

**Performance estimate**: With walkdir depth=5 + ignore rules, scanning 500 repos ≈ 10-15 seconds for discovery + 10-15 seconds for metadata extraction = 20-30 seconds (within SC-001 target of 30s).

## R3: SQLite Schema Strategy

**Decision**: Simple flat tables with JSON blobs for flexible metadata, normalized core fields for querying.

**Rationale**:
- SQLite is the Tauri ecosystem standard for local persistence
- Core queryable fields (repo name, type, scope, status) are indexed columns
- Resource metadata that varies by type stored as JSON TEXT (serde_json) — avoids complex polymorphic joins
- Migrations managed via embedded SQL files in storage/migrations/
- Single-file database stored alongside app data

**Alternatives considered**:
- Full normalization with type-specific tables: over-engineered for single-user local tool
- sled/RocksDB: key-value not ideal for relational queries (filter by repo, type, status)
- Plain JSON files: no query capability, hard to manage cross-references

**Schema outline** (detailed in data-model.md):
- `repositories` — id, path, name, remote_url, branch, head_commit, dirty_state, last_indexed_at
- `capability_resources` — id, repo_id (FK), type, name, source_path, scope, tracked_by_git, content_hash, metadata_json
- `packs` — id, name, version, description, manifest_path, source_repo_id, source_commit, created_at
- `migration_runs` — id, source_type, source_id, target_repo_id, status, plan_json, report_json, created_at
- `doctor_reports` — id, repo_id, score, issues_json, created_at

## R4: Claude Code Config File Conventions

**Decision**: Parser supports `.claude/` directory layout conventions as of Claude Code 2025-2026.

**Rationale**:
- Skills: `skills/<name>/SKILL.md` or `skills/<name>/skill.md`
- MCP: `.claude/mcp.json` or `.claude/mcp.local.json` (JSON array of server configs)
- Hooks: `.claude/settings.json` → `hooks` key (PreToolUse, PostToolUse, Stop, etc.)
- Rules: `.claude/rules/*.md` (markdown files with rule definitions)
- Agents: `.claude/agents/*.md` (markdown files with agent definitions)
- Commands: `.claude/commands/*.md`
- Settings: `.claude/settings.json`, `.claude/settings.local.json`
- CLAUDE.md: project root level context file

**Fallback behavior**: If a config file is malformed JSON, parser returns a parse error resource with the error message (per FR-009). Unknown keys are ignored. Missing optional files produce no error.

**Alternatives considered**:
- Full Claude Code internal API: no official API exists; config files are the stable contract
- Schema-validated parsing: JSON Schema validation adds dependency weight; simpler to parse and validate structurally

## R5: Pack Manifest Format

**Decision**: JSON manifest (`pack.manifest.json`) with self-contained directory structure.

**Rationale**:
- JSON is the lingua franca of Claude Code configs (settings.json, mcp.json) — consistent
- Self-contained pack directory (resources/, mcp/, settings/, validation/ subdirs) is portable as a tarball or directory copy
- Manifest includes: schema version, name, version, description, source repo/commit reference, resource list with relative paths, env placeholders, validation rules
- Hash-based integrity: each resource has a SHA256 hash in the manifest for tamper detection

**Manifest schema** (detailed in contracts/pack-manifest.schema.json):

```json
{
  "schemaVersion": "1.0",
  "name": "string",
  "version": "string",
  "description": "string | null",
  "source": { "repo": "string | null", "commit": "string | null" },
  "resources": [{ "type": "string", "name": "string", "source": "string", "hash": "string" }],
  "env": [{ "name": "string", "required": "boolean", "description": "string | null" }],
  "validation": { "rules": ["string"] }
}
```

**Alternatives considered**:
- YAML manifest: less consistent with Claude Code ecosystem (JSON-dominant)
- TOML: Rust-native but uncommon in Claude Code tooling
- Protocol Buffers: overkill for human-readable pack format

## R6: Conflict Resolution Algorithm

**Decision**: Resource identity determined by `(type, name)` tuple. Conflicts detected by comparing target path existence and content hash.

**Rationale**:
- Same `(type, name)` on target = conflict → user chooses strategy
- Same `(type, name)` + same content hash = idempotent → auto-skip
- Different `(type, name)` on target = new resource → auto-add
- Rename strategy generates `name-1`, `name-2` pattern (configurable suffix)
- Merge strategy is marked as "manual resolve placeholder" in MVP — full merge is P2

**Alternatives considered**:
- Path-based identity: fragile if user renamed files
- Content-based dedup: expensive for large resources, misses intentional differences

## R7: Doctor Scoring Algorithm

**Decision**: Weighted deduction as clarified — start 100, critical -20, warning -5, info -1, floor 0.

**Implementation note**: Score is purely a function of issue counts. Issues are generated by rule functions (check_skill_structure, check_hook_target, check_env_placeholders, etc.). Each rule returns `Vec<DoctorIssue>` with assigned severity. The scorer aggregates and computes the final score.

This is trivially testable: `score = max(0, 100 - 20*critical - 5*warning - info)`.

## R8: TDD Strategy by Crate

**Decision**: Each crate follows a specific TDD pattern suited to its role.

| Crate | TDD Pattern | Key Test Types |
|-------|-------------|----------------|
| domain | Pure type tests | Deserialization, validation, state transitions |
| repo-scanner | Fixture-based integration | Real temp dirs with .git/, permission error simulation |
| git-service | Integration with real git | Test repos with known branch/commit/dirty states |
| claude-parser | Golden file tests | Fixture configs → expected inventory JSON |
| pack-engine | Golden file tests | Fixture repos → exported pack structure |
| migration-engine | Golden file + diff tests | Plan JSON before/after, actual file writes to temp dirs |
| doctor-engine | Rule I/O tests | Known-broken repos → expected issue list + score |
| storage | Integration with SQLite | CRUD round-trip tests, migration tests |
| tauri-bridge | Integration + E2E | Command invocation tests, Playwright for full flows |

Frontend TDD: Vitest for store logic + component tests, Playwright for E2E user journeys.
