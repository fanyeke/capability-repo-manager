# Tasks: Capability Repo Manager

**Input**: Design documents from `/specs/001-capability-repo-manager/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: TDD approach requested — all phases include test tasks written BEFORE implementation (Red → Green → Refactor).

**Organization**: Tasks grouped by user story for independent implementation and testing. Each story is an independently deliverable increment.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1-US6)
- Exact file paths included in all descriptions

---

## Phase 1: Setup (Project Initialization)

**Purpose**: Scaffold the Rust workspace, Tauri shell, Svelte frontend, and shared tooling.

- [x] T001 Create Rust workspace with Cargo.toml at repository root, listing all 9 crates (domain, repo-scanner, git-service, claude-parser, pack-engine, migration-engine, doctor-engine, storage, tauri-bridge)
- [x] T002 [P] Initialize Tauri 2 app shell with Svelte + TypeScript + Vite frontend in frontend/
- [x] T003 [P] Configure Rust tooling: rustfmt.toml (edition 2021, max_width 120), clippy.toml at repository root
- [x] T004 [P] Configure frontend tooling: .prettierrc, eslint.config.js, tsconfig.json in frontend/
- [x] T005 [P] Create tests/fixtures/ directory with README.md documenting test fixture conventions
- [x] T006 [P] Create .github/workflows/ci.yml with lint → typecheck → unit-test → integration-test → e2e-test pipeline
- [x] T007 [P] Add .gitignore entries for target/, node_modules/, dist/, .env, *.db

---

## Phase 2: Foundational (Domain + Storage + Fixtures)

**Purpose**: Core domain types and persistence layer that ALL user stories depend on.

**CRITICAL**: No user story work can begin until this phase is complete.

### Tests for Foundational Layer

- [x] T008 [P] Write unit tests for Repository entity (serialization, validation, state transitions) in crates/domain/tests/repository_tests.rs
- [x] T009 [P] Write unit tests for CapabilityResource entity (XOR constraint, uniqueness rules) in crates/domain/tests/capability_tests.rs
- [x] T010 [P] Write unit tests for CapabilityPack entity (manifest validation, version format) in crates/domain/tests/pack_tests.rs
- [x] T011 [P] Write integration tests for SQLite repository CRUD operations in crates/storage/tests/repo_store_tests.rs
- [x] T012 [P] Write integration tests for SQLite resource store round-trip in crates/storage/tests/resource_store_tests.rs

### Implementation for Foundational Layer

- [x] T013 [P] Define Repository domain type with fields (id, name, path, remote_url, current_branch, head_commit, dirty_state, last_indexed_at) in crates/domain/src/repository.rs
- [x] T014 [P] Define CapabilityResource domain type with fields (id, repo_id, pack_id, type, name, source_path, scope, tracked_by_git, content_hash, metadata_json, error_message) in crates/domain/src/capability.rs
- [x] T015 [P] Define CapabilityPack domain type with fields (id, name, version, description, pack_type, manifest_path, source_repo_id, source_commit, created_at, storage_dir) in crates/domain/src/pack.rs
- [x] T016 [P] Define MigrationPlan, MigrationRun, MigrationPlanItem, MigrationConflict domain types in crates/domain/src/migration.rs
- [x] T017 [P] Define DoctorReport, DoctorIssue domain types with scoring logic (max(0, 100 - 20*c - 5*w - i)) in crates/domain/src/doctor.rs
- [x] T018 [P] Define common error types (AppError enum with variants for Scan, Parse, Migration, Storage, Git) in crates/domain/src/error.rs
- [x] T019 Create domain crate lib.rs re-exporting all public types in crates/domain/src/lib.rs
- [x] T020 Create SQLite database initialization with schema (CREATE TABLE repositories, capability_resources, packs, migration_runs, doctor_reports) and migration framework in crates/storage/src/lib.rs
- [x] T021 [P] Implement RepositoryStore (insert, get_by_id, get_by_path, list, delete) in crates/storage/src/repo_store.rs
- [x] T022 [P] Implement ResourceStore (insert_batch, get_by_repo, get_by_pack, update) in crates/storage/src/resource_store.rs
- [x] T023 Create test fixture: repo-basic (single skill + .claude/settings.json) in tests/fixtures/repo-basic/
- [x] T024 [P] Create test fixture: repo-full (all resource types) in tests/fixtures/repo-full/
- [x] T025 [P] Create test fixture: repo-broken (malformed JSON, missing files) in tests/fixtures/repo-broken/
- [x] T026 [P] Create test fixture: repo-conflict (overlapping resource names) in tests/fixtures/repo-conflict/

**Checkpoint**: Domain types complete + storage layer ready + test fixtures available. All user stories can now begin.

---

## Phase 3: User Story 1 - Scan and Browse Local Git Repositories (Priority: P1)

**Goal**: User opens the app, adds scan directories via guided setup, discovers all local Git repos with metadata (name, path, branch, dirty state), searches/filters the list, and manually refreshes individual repos.

**Independent Test**: Launch app, add `tests/fixtures/` as scan root, verify all 4 fixture repos are discovered with correct metadata, confirm search + filter works.

### Tests for User Story 1

> Write these FIRST, ensure they FAIL before implementation.

- [x] T027 [P] [US1] Write unit tests for repo scanner (discovery with depth limit, ignore rules, permission denied handling, dedup) in crates/repo-scanner/tests/scanner_tests.rs
- [x] T028 [P] [US1] Write unit tests for Git metadata extraction (branch, commit, dirty state) in crates/git-service/tests/git_cli_tests.rs
- [x] T029 [P] [US1] Write integration test for scan → persist → list → refresh flow in tests/integration/scan_flow.rs
- [x] T030 [P] [US1] Write frontend unit tests for RepoList component (filtering, search, sorting) in frontend/src/tests/unit/RepoList.test.ts
- [x] T031 [P] [US1] Write frontend unit tests for repoStore (state transitions, refresh behavior) in frontend/src/tests/unit/repoStore.test.ts

### Implementation for User Story 1

- [x] T032 [P] [US1] Implement Git repository scanner (walkdir-based BFS, configurable depth with default 5, ignore rules with defaults for node_modules/.venv/vendor/.cache/build) in crates/repo-scanner/src/scanner.rs
- [x] T033 [P] [US1] Implement Git metadata extractor (branch name, HEAD commit SHA, dirty state detection via status porcelain, remote URL) in crates/git-service/src/git_cli.rs
- [x] T034 [US1] Implement RepoCatalog service (orchestrates scanner + git-service + storage, handles incremental refresh, permission errors) in crates/repo-scanner/src/lib.rs
- [x] T035 [US1] Implement tauri-bridge repo commands (scan_repositories, list_repositories, refresh_repository, get_repository_detail, remove_repository) in crates/tauri-bridge/src/commands/repo_commands.rs
- [x] T036 [US1] Implement Tauri app state management (AppState struct holding Storage, settings) in crates/tauri-bridge/src/state.rs
- [x] T037 [P] [US1] Define frontend TypeScript types (RepositorySummary, RepoDetail, RepoFilter, ScanResult) in frontend/src/lib/types.ts
- [x] T038 [US1] Implement repoStore (Svelte writable stores for repo list, selected repo, filter state, refresh logic via Tauri invoke) in frontend/src/lib/stores/repoStore.ts
- [x] T039 [P] [US1] Implement RepoCard component (displays name, path, branch tag, dirty indicator, capability counts) in frontend/src/lib/components/RepoCard.svelte
- [x] T040 [P] [US1] Implement RepoList component (search bar, filter chips, sort controls, RepoCard grid) in frontend/src/lib/components/RepoList.svelte
- [x] T041 [US1] Implement GuidedSetup page (first-launch: prompts to add scan directories, "Skip" option) in frontend/src/lib/pages/GuidedSetup.svelte
- [x] T042 [US1] Implement Dashboard page (main view: RepoList + summary cards for total repos, repos with config, recent scans) in frontend/src/lib/pages/Dashboard.svelte
- [x] T043 [US1] Implement Settings page (scan roots management, scan depth config, ignore patterns editor, pack storage path) in frontend/src/lib/pages/Settings.svelte
- [x] T044 [US1] Wire up Tauri invoke calls in repoStore and verify end-to-end scan → display flow works from GuidedSetup → Dashboard

**Checkpoint**: US1 independently functional — user can scan, browse, search, filter repos. MVP deliverable.

---

## Phase 4: User Story 2 - View Repository Capability Inventory (Priority: P1)

**Goal**: User opens a repo detail, sees all Claude Code capabilities (skills, MCP, hooks, rules, agents, commands, plugins, settings) grouped by type with scope indicators, dependency status, and parse error display.

**Independent Test**: Open `tests/fixtures/repo-full` detail, verify all resource types are detected with correct metadata, open `tests/fixtures/repo-broken` and verify parse errors are displayed.

### Tests for User Story 2

> Write these FIRST, ensure they FAIL before implementation.

- [x] T045 [P] [US2] Write golden file test for skill parser (fixture skill dir → expected CapabilityResource) in crates/claude-parser/tests/skill_parser_tests.rs
- [x] T046 [P] [US2] Write golden file test for MCP config parser (fixture mcp.json → expected resources) in crates/claude-parser/tests/mcp_parser_tests.rs
- [x] T047 [P] [US2] Write golden file test for hook config parser (fixture settings.json hooks → expected resources) in crates/claude-parser/tests/hook_parser_tests.rs
- [x] T048 [P] [US2] Write test for malformed config handling (broken JSON → parse error resource with error_message populated) in crates/claude-parser/tests/error_handling_tests.rs
- [x] T049 [P] [US2] Write frontend unit tests for CapabilityList component (grouping, scope badges, error indicators) in frontend/src/tests/unit/CapabilityList.test.ts
- [x] T050 [P] [US2] Write integration test for full inventory (scan repo-full → parse all types → verify grouped output) in tests/integration/capability_flow.rs

### Implementation for User Story 2

- [x] T051 [P] [US2] Implement skill directory parser (detect skills/<name>/SKILL.md, extract name + summary) in crates/claude-parser/src/skill_parser.rs
- [x] T052 [P] [US2] Implement MCP config parser (parse .claude/mcp.json array, extract serverName/command/env) in crates/claude-parser/src/mcp_parser.rs
- [x] T053 [P] [US2] Implement hook config parser (parse settings.json hooks key, extract hook type + command references) in crates/claude-parser/src/hook_parser.rs
- [x] T054 [P] [US2] Implement rule + agent parsers (detect .claude/rules/*.md, .claude/agents/*.md) in crates/claude-parser/src/rule_parser.rs and crates/claude-parser/src/agent_parser.rs
- [x] T055 [US2] Implement capability inventory orchestrator (coordinates all parsers, builds full inventory, detects scope from file locations) in crates/claude-parser/src/lib.rs
- [x] T056 [US2] Implement tauri-bridge capability commands (get_capability_inventory, get_resource_detail) in crates/tauri-bridge/src/commands/capability_commands.rs
- [x] T057 [P] [US2] Implement capabilityStore (grouped resource stores, resource detail fetching) in frontend/src/lib/stores/capabilityStore.ts
- [x] T058 [P] [US2] Implement CapabilityList component (left sidebar: type groups with counts, click to filter) in frontend/src/lib/components/CapabilityList.svelte
- [x] T059 [P] [US2] Implement ResourceDetail component (name, type, scope badge, file path, git-tracked indicator, dependencies, error display) in frontend/src/lib/components/ResourceDetail.svelte
- [x] T060 [US2] Implement RepoDetail page (tabs: Overview, Capabilities; top summary bar with repo metadata) in frontend/src/lib/pages/RepoDetail.svelte
- [x] T061 [US2] Register tauri-bridge commands in main Tauri app and wire frontend RepoDetail → capabilityStore → backend parsers

**Checkpoint**: US2 independently functional — user can view full capability inventory for any repo. Combined US1+US2 = core MVP.

---

## Phase 5: User Story 3 - Export Capability Pack from Repository (Priority: P2)

**Goal**: User selects resources from a repo, enters pack metadata, previews selection, and exports a structured pack with manifest to the pack library. Pack library shows all exported packs with metadata.

**Independent Test**: Open repo-full detail, select 3 resources, export as pack, verify pack.manifest.json is valid per schema, verify pack appears in library.

### Tests for User Story 3

> Write these FIRST, ensure they FAIL before implementation.

- [x] T062 [P] [US3] Write unit tests for manifest builder (resource selection → valid manifest JSON) in crates/pack-engine/tests/manifest_tests.rs
- [x] T063 [P] [US3] Write unit tests for pack validator (schema validation, missing file detection, hash verification) in crates/pack-engine/tests/validator_tests.rs
- [x] T064 [P] [US3] Write unit tests for pack library (CRUD operations, duplicate name+version rejection) in crates/pack-engine/tests/library_tests.rs
- [x] T065 [P] [US3] Write integration test for full export flow (repo → select resources → export pack → validate → library list) in tests/integration/pack_flow.rs
- [x] T066 [P] [US3] Write frontend unit tests for PackExport page (resource selection, metadata form validation) in frontend/src/tests/unit/PackExport.test.ts

### Implementation for User Story 3

- [x] T067 [P] [US3] Implement manifest builder (construct pack.manifest.json per JSON Schema with schemaVersion, name, version, resources array, env, validation) in crates/pack-engine/src/manifest.rs
- [x] T068 [P] [US3] Implement pack packer (copy selected resources into pack directory structure: resources/skills/, mcp/, settings/, create manifest) in crates/pack-engine/src/packer.rs
- [x] T069 [US3] Implement pack validator (schema conformance, file existence, SHA256 hash integrity, env placeholder detection) in crates/pack-engine/src/validator.rs
- [x] T070 [US3] Implement pack library (PackStore: insert/get/list/delete via storage layer, manage pack directory on disk) in crates/pack-engine/src/library.rs
- [x] T071 [US3] Implement pack-engine orchestrator (coordinates manifest → packer → validator → library, handles empty selection + missing file warnings) in crates/pack-engine/src/lib.rs
- [x] T072 [US3] Implement tauri-bridge pack commands (export_capability_pack, list_packs, get_pack_detail, delete_pack, validate_pack) in crates/tauri-bridge/src/commands/pack_commands.rs
- [x] T073 [US3] Implement packStore (pack list, selected pack, export action) in frontend/src/lib/stores/packStore.ts
- [x] T074 [P] [US3] Implement PackExport page (resource type filter, resource checkboxes, name/version/description form, preview panel, export button) in frontend/src/lib/pages/PackExport.svelte
- [x] T075 [P] [US3] Implement PackLibrary component (pack card grid, name/version/type/source badges, delete action) in frontend/src/lib/components/PackLibrary.svelte
- [x] T076 [US3] Wire export flow: RepoDetail → PackExport page → pack-engine → PackLibrary display

**Checkpoint**: US3 independently functional — user can export packs and browse pack library.

---

## Phase 6: User Story 4 - Apply Capability Pack to Target Repository (Priority: P2)

**Goal**: User selects a pack and target repo, reviews dry-run migration plan (adds/overwrites/conflicts/deps), chooses conflict resolution strategies per item, executes migration with snapshot rollback, and views migration report.

**Independent Test**: Apply a pack from repo-full to repo-basic, verify dry-run shows correct conflicts, choose "skip" for one and "rename" for another, execute, verify files written correctly, verify rollback works.

### Tests for User Story 4

> Write these FIRST, ensure they FAIL before implementation.

- [x] T077 [P] [US4] Write unit tests for migration planner (pack + target → plan with correct add/overwrite/conflict classification) in crates/migration-engine/tests/planner_tests.rs
- [x] T078 [P] [US4] Write unit tests for conflict detection (same name same path, same name diff path, diff name same path scenarios) in crates/migration-engine/tests/conflict_tests.rs
- [x] T079 [P] [US4] Write unit tests for migration executor (apply plan with strategies → verify file writes, verify report generation) in crates/migration-engine/tests/executor_tests.rs
- [x] T080 [P] [US4] Write unit tests for snapshot rollback (snapshot before → apply migration → rollback → verify original state restored) in crates/migration-engine/tests/rollback_tests.rs
- [x] T081 [P] [US4] Write integration test for full migration flow (pack → dry-run → resolve conflicts → execute → report → rollback) in tests/integration/migration_flow.rs
- [x] T082 [P] [US4] Write frontend unit tests for MigrationPlan component (plan items display, conflict badge, strategy selector) in frontend/src/tests/unit/MigrationPlan.test.ts

### Implementation for User Story 4

- [x] T083 [US4] Implement migration planner (compare pack resources vs target repo, classify each as add/overwrite/skip, detect name+path conflicts, check dependencies) in crates/migration-engine/src/planner.rs
- [x] T084 [US4] Implement conflict resolver (apply user strategies: skip/overwrite/rename/merge, generate resolved plan) in crates/migration-engine/src/conflict.rs
- [x] T085 [US4] Implement migration executor (create snapshot via copy, write files per resolved plan, handle write errors atomically, record MigrationRun) in crates/migration-engine/src/executor.rs
- [x] T086 [US4] Implement snapshot manager (create pre-migration directory snapshot via copy_tree, verify snapshot integrity, restore on rollback) in crates/migration-engine/src/rollback.rs
- [x] T087 [US4] Implement migration-engine orchestrator (plan → user-strategies → execute → report pipeline, circular reference check) in crates/migration-engine/src/lib.rs
- [x] T088 [US4] Implement tauri-bridge migration commands (build_migration_plan, apply_migration_plan, rollback_migration, get_migration_history) in crates/tauri-bridge/src/commands/migration_commands.rs
- [x] T089 [US4] Implement migrationStore (plan state, conflict strategies, execution result) in frontend/src/lib/stores/migrationStore.ts
- [x] T090 [P] [US4] Implement MigrationPlan component (dry-run results table: resource name, type, planned action, conflict indicator, strategy dropdown) in frontend/src/lib/components/MigrationPlan.svelte
- [x] T091 [P] [US4] Implement ConflictResolver component (per-item conflict display, strategy selector: skip/overwrite/rename/merge, inline preview) in frontend/src/lib/components/ConflictResolver.svelte
- [x] T092 [US4] Implement PackApply page (pack selector, target repo selector, dry-run trigger → MigrationPlan → ConflictResolver → execute → report view) in frontend/src/lib/pages/PackApply.svelte
- [x] T093 [US4] Wire migration flow: PackLibrary → PackApply page → migration-engine → report display

**Checkpoint**: US4 independently functional — user can migrate packs between repos with full conflict handling and rollback.

---

## Phase 7: User Story 5 - Diagnose Repository Configuration Health (Priority: P2)

**Goal**: User runs doctor on a repo, sees weighted health score, categorized issues (critical/warning/info) with descriptions and actionable recommendations. Doctor runs automatically after migration.

**Independent Test**: Run doctor on repo-broken fixture, verify critical issue for missing hook script, warning for incomplete skill structure, verify score computation matches formula.

### Tests for User Story 5

> Write these FIRST, ensure they FAIL before implementation.

- [x] T094 [P] [US5] Write unit tests for skill structure check (missing dir/files → warning, complete → no issue) in crates/doctor-engine/tests/checks_tests.rs
- [x] T095 [P] [US5] Write unit tests for hook target check (script exists → no issue, missing → critical) in crates/doctor-engine/tests/checks_tests.rs
- [x] T096 [P] [US5] Write unit tests for env placeholder check (unresolved ${VAR} → warning, all resolved → no issue) in crates/doctor-engine/tests/checks_tests.rs
- [x] T097 [P] [US5] Write unit tests for scoring formula (2 critical + 3 warning + 5 info → 100-40-15-5=40) in crates/doctor-engine/tests/scoring_tests.rs
- [x] T098 [P] [US5] Write integration test for full doctor flow (repo-broken → run doctor → verify all issue types + score) in tests/integration/doctor_flow.rs
- [x] T099 [P] [US5] Write frontend unit tests for DoctorReport component (score display, issue grouping, severity badges) in frontend/src/tests/unit/DoctorReport.test.ts

### Implementation for User Story 5

- [x] T100 [P] [US5] Implement skill structure checker (verify skills/<name>/ exists, contains SKILL.md or skill.md, report missing) in crates/doctor-engine/src/checks.rs
- [x] T101 [P] [US5] Implement hook target checker (extract command/script references from hooks, verify file existence on disk) in crates/doctor-engine/src/checks.rs
- [x] T102 [P] [US5] Implement env placeholder checker (scan settings/mcp for ${VAR} patterns, check against known env) in crates/doctor-engine/src/checks.rs
- [x] T103 [P] [US5] Implement MCP config integrity checker (validate JSON structure, check serverName presence) in crates/doctor-engine/src/checks.rs
- [x] T104 [US5] Implement health scorer (weighted deduction: score = max(0, 100 - 20*critical - 5*warning - info)) in crates/doctor-engine/src/scoring.rs
- [x] T105 [US5] Implement doctor-engine orchestrator (run all checks, aggregate issues, compute score, save DoctorReport to storage) in crates/doctor-engine/src/lib.rs
- [x] T106 [US5] Implement tauri-bridge doctor commands (run_doctor) in crates/tauri-bridge/src/commands/doctor_commands.rs
- [x] T107 [US5] Implement doctorStore (report state, run-trigger, auto-run after migration hook) in frontend/src/lib/stores/doctorStore.ts
- [x] T108 [P] [US5] Implement DoctorReport component (score circle with color, issue list grouped by severity, per-issue recommendation + jump-to-resource link) in frontend/src/lib/components/DoctorReport.svelte
- [x] T109 [US5] Implement Doctor page (run button, report display, historical reports list for repo) in frontend/src/lib/pages/Doctor.svelte
- [x] T110 [US5] Wire auto-doctor: after migration execution completes → automatically trigger run_doctor → display mini report in migration result

**Checkpoint**: US5 independently functional — user can diagnose any repo and get scored health report.

---

## Phase 8: User Story 6 - Compare Repositories and Detect Drift (Priority: P3)

**Goal**: User compares two repos, or a repo vs a baseline pack, and sees resources categorized as missing/extra/modified/same. Tech leads use this to check team baseline compliance.

**Independent Test**: Compare repo-full vs repo-basic, verify differences categorized correctly. Compare repo-full vs a baseline pack, verify drift detection.

### Tests for User Story 6

> Write these FIRST, ensure they FAIL before implementation.

- [x] T111 [P] [US6] Write unit tests for drift comparison (two resource sets → missing/extra/modified/same output) in crates/doctor-engine/tests/drift_tests.rs
- [x] T112 [P] [US6] Write frontend unit tests for CompareView component (diff table rendering, category filters) in frontend/src/tests/unit/CompareView.test.ts

### Implementation for User Story 6

- [x] T113 [P] [US6] Implement drift comparator (compare two CapabilityResource sets by type+name, compute content hash diff for modified detection) in crates/doctor-engine/src/drift.rs
- [x] T114 [US6] Implement tauri-bridge compare commands (compare_repo_with_pack, compare_repos) in crates/tauri-bridge/src/commands/doctor_commands.rs
- [x] T115 [US6] Implement compareStore (comparison source/target selection, result state) in frontend/src/lib/stores/compareStore.ts
- [x] T116 [P] [US6] Implement CompareView component (side-by-side diff, category tabs: Missing/Extra/Modified/Same, resource detail on click) in frontend/src/lib/components/CompareView.svelte
- [x] T117 [US6] Implement Compare page (source selector: repo or pack, target selector: repo, run comparison → CompareView) in frontend/src/lib/pages/Compare.svelte

**Checkpoint**: US6 independently functional — user can compare repos and packs, detect configuration drift.

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: E2E tests, performance validation, hardening.

- [x] T118 [P] Write E2E test for MVP user journey (guided setup → scan → open repo → view capabilities → export pack) in tests/e2e/scan-and-browse.spec.ts
- [x] T119 [P] Write E2E test for migration user journey (apply pack → review plan → resolve conflicts → execute → view report → run doctor) in tests/e2e/migrate-pack.spec.ts
- [x] T120 [P] Write E2E test for doctor user journey (open repo → run doctor → view issues → verify score) in tests/e2e/diagnose.spec.ts
- [x] T121 Performance validation: verify 500-repo scan completes within 30 seconds (SC-001)
- [x] T122 Performance validation: verify repo detail with 100+ resources renders within 10 seconds (SC-002)
- [x] T123 [P] Implement background startup refresh of Git metadata for all indexed repos (FR-005 incremental refresh)
- [x] T124 [P] Implement disk space check before pack creation and migration execution
- [x] T125 Run quickstart.md validation: verify all setup instructions work on clean Linux environment
- [x] T126 Final Constitution check: verify all 6 principles pass, code coverage ≥80% across all crates

---

## Dependencies & Execution Order

### Phase Dependencies

```
Setup (Phase 1)
    │
    ▼
Foundational (Phase 2) ← CRITICAL BLOCKER for all user stories
    │
    ├──► US1: Scan & Browse (Phase 3) ─── independent, no story deps
    ├──► US2: Capability Inventory (Phase 4) ─── independent, no story deps
    │         │
    │         ├──► US3: Pack Export (Phase 5) ─── depends on US2 (needs resources to select)
    │         │         │
    │         │         └──► US4: Pack Migration (Phase 6) ─── depends on US3 (needs packs)
    │         │
    │         ├──► US5: Doctor (Phase 7) ─── depends on US2 (needs resources to diagnose)
    │         │
    │         └──► US6: Compare (Phase 8) ─── depends on US2 (needs resources to compare)
    │                                              └── also depends on US3 (needs packs for repo-vs-pack)
    │
    ▼
Polish (Phase 9) ← depends on all desired user stories
```

### User Story Dependency Summary

| Story | Depends On | Can Parallel With |
|-------|-----------|-------------------|
| US1 (Scan) | Phase 2 only | US2 |
| US2 (Inventory) | Phase 2 only | US1 |
| US3 (Export) | US2 | US5 |
| US4 (Migration) | US3 | — |
| US5 (Doctor) | US2 | US3 |
| US6 (Compare) | US2, US3 | US5 |

### Within Each Phase (TDD Order)

1. **Write tests** → They FAIL (Red)
2. **Implement models/parsers/engines** → Tests PASS (Green)
3. **Implement Tauri commands** → Integration tests PASS
4. **Implement frontend components/stores/pages** → E2E assertions PASS
5. **Refactor** → All tests stay GREEN

---

## Parallel Execution Examples

### Phase 2: Foundational (after Setup complete)

```bash
# Launch all domain type tasks in parallel:
T013: "Define Repository domain type in crates/domain/src/repository.rs"
T014: "Define CapabilityResource domain type in crates/domain/src/capability.rs"
T015: "Define CapabilityPack domain type in crates/domain/src/pack.rs"
T016: "Define MigrationPlan domain types in crates/domain/src/migration.rs"
T017: "Define DoctorReport domain types in crates/domain/src/doctor.rs"
T018: "Define common error types in crates/domain/src/error.rs"

# Launch all storage tasks in parallel (after domain done):
T021: "Implement RepositoryStore in crates/storage/src/repo_store.rs"
T022: "Implement ResourceStore in crates/storage/src/resource_store.rs"

# Launch all fixture creation in parallel:
T023: "Create repo-basic fixture in tests/fixtures/repo-basic/"
T024: "Create repo-full fixture in tests/fixtures/repo-full/"
T025: "Create repo-broken fixture in tests/fixtures/repo-broken/"
T026: "Create repo-conflict fixture in tests/fixtures/repo-conflict/"
```

### Phase 3-4: US1 + US2 in Parallel (after Phase 2)

```bash
# Developer A: User Story 1 (Scan & Browse)
T032-T044: repo-scanner + git-service + tauri-bridge repo commands + frontend Dashboard

# Developer B: User Story 2 (Capability Inventory)
T051-T061: claude-parser + tauri-bridge capability commands + frontend RepoDetail
```

### Phase 5-7: US3 + US5 in Parallel (after US2)

```bash
# Developer A: User Story 3 (Pack Export)
T067-T076: pack-engine + tauri-bridge pack commands + frontend PackExport

# Developer B: User Story 5 (Doctor)
T100-T110: doctor-engine checks/scoring + tauri-bridge doctor commands + frontend Doctor
```

---

## Implementation Strategy

### MVP First (US1 Only)

1. Complete Phase 1: Setup (T001-T007)
2. Complete Phase 2: Foundational (T008-T026)
3. Complete Phase 3: User Story 1 (T027-T044)
4. **STOP and VALIDATE**: Launch app, scan fixtures, browse repos → independently valuable
5. Ship/demo as v0.1: Repository browser with Claude Code awareness

### MVP+ (US1 + US2)

1. Add Phase 4: User Story 2 (T045-T061)
2. **STOP and VALIDATE**: Open repo detail, see full capability inventory → core value delivered
3. Ship/demo as v0.2: Repo browser + capability inventory

### Full MVP (US1-US5, skip US6)

1. Add Phase 5: US3 Pack Export (T062-T076)
2. Add Phase 6: US4 Pack Migration (T077-T093)
3. Add Phase 7: US5 Doctor (T094-T110)
4. **STOP and VALIDATE**: Full loop: scan → inventory → export → migrate → diagnose
5. Ship/demo as v1.0

### Complete (including US6)

1. Add Phase 8: US6 Compare (T111-T117)
2. Add Phase 9: Polish (T118-T126)
3. Ship/demo as v1.1

---

## Notes

- [P] tasks modify different files with no shared state — can execute in parallel via background agents
- [Story] label maps each task to a user story for traceability and independent validation
- Each phase has a Checkpoint where the story must be independently functional before proceeding
- TDD: Tests listed first in each phase MUST be written and confirmed FAILING before implementation
- Commit after each logical task group (test → implement → refactor per file)
- Fixture repos in tests/fixtures/ are the canonical test data — update them when parser behavior changes
- Rust crate dependency: domain has no internal deps, all engine crates depend on domain only, tauri-bridge depends on all
- Frontend: stores.ts → components.svelte → pages.svelte (bottom-up dependency within each phase)
