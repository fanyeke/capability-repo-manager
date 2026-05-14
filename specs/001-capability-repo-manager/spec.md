# Feature Specification: Capability Repo Manager

**Feature Branch**: `001-capability-repo-manager`
**Created**: 2026-05-14
**Status**: Draft
**Input**: User description: "Claude Code 项目级能力管理器 — 以本地 Git 仓库为中心的 Claude Code 项目能力管理、迁移、复现与诊断工具"

## Clarifications

### Session 2026-05-14

- Q: 首次启动时仓库扫描是自动触发还是手动引导？ → A: 首次启动显示引导页，用户手动选择目录后触发扫描，支持「跳过」进入空状态主页。
- Q: 扫描深度与忽略规则的策略？ → A: 扫描深度可配置（默认 5 层），目录忽略规则可自定义（用户可添加/删除忽略模式），默认忽略 node_modules/.venv/vendor/.cache/build 等常见非代码目录。
- Q: Pack 的物理存储位置？ → A: 默认存储在应用数据目录（~/.capability-manager/packs/），但用户可在设置中更改存储路径。
- Q: 仓库元数据重新扫描时机？ → A: 启动时后台增量刷新 Git 元数据（branch/dirty/commit），能力资源解析保持手动触发；文件监听为可选功能。
- Q: Doctor 健康度评分的计算方式？ → A: 加权扣分制 —— 起始 100 分，每个 critical 扣 20 分，每个 warning 扣 5 分，每个 info 扣 1 分，最低 0 分。

## User Scenarios & Testing

### User Story 1 - Scan and Browse Local Git Repositories (Priority: P1)

A developer wants to see all their local Git projects in one place, understand which ones have Claude Code configurations, and quickly assess project states. They open the application, add their code directories (e.g., `~/code`, `~/work`), and the system discovers all Git repositories, displaying key metadata like branch, dirty state, and capability counts.

**Why this priority**: The repository catalog is the primary entry point for the entire product. Without it, users cannot discover or navigate to any project. All other features depend on having an indexed repository to work with.

**Independent Test**: Can be fully tested by adding a scan directory, verifying all Git repos are discovered with correct metadata displayed, and confirming the list supports search and filtering. This alone delivers value as a local repo browser with Claude Code awareness.

**Acceptance Scenarios**:

1. **Given** the app is opened for the first time with no scan directories configured, **When** the user is shown a guided setup page and adds `~/code` as a scan root, **Then** the system recursively discovers all Git repositories under that directory and displays them in a list with name, path, current branch, and dirty/clean status.
2. **Given** a list of discovered repositories, **When** the user types in the search bar, **Then** the list filters to show only repositories matching the search term by name or path.
3. **Given** a directory with no read permission that is part of the scan scope, **When** scanning encounters it, **Then** the system logs a warning and continues scanning other directories without crashing.
4. **Given** a previously scanned repository that has changed on disk, **When** the user triggers a refresh, **Then** the repository metadata and capability counts are updated incrementally.

---

### User Story 2 - View Repository Capability Inventory (Priority: P1)

A developer selects a repository and views all its Claude Code capabilities: Skills, MCP Servers, Hooks, Rules, Agents, Commands, and Settings. They can see which resources exist, their scope (project vs. local), whether they are tracked by Git, and any dependency issues.

**Why this priority**: Understanding what capabilities a repository has is the core value proposition. Users need to see "what's configured where" before they can export, migrate, or diagnose anything.

**Independent Test**: Can be tested by opening any repository that has Claude Code configurations, verifying that skills/hooks/MCP/rules are detected and displayed with correct metadata, and confirming that parse errors are shown rather than silently ignored.

**Acceptance Scenarios**:

1. **Given** a repository with skills, MCP configurations, hooks, and rules, **When** the user opens the repository detail, **Then** all capability resources are listed grouped by type with counts for each category.
2. **Given** a repository with a malformed configuration file, **When** the system parses capabilities, **Then** the parse error is displayed alongside the resource with an error indicator, not silently dropped.
3. **Given** a repository with both project-level and local-only configurations, **When** viewing the inventory, **Then** the scope of each resource is clearly indicated so users can distinguish shared from private configs.
4. **Given** a capability resource that references an external dependency (e.g., a hook referencing a script), **When** the user views resource details, **Then** the dependency status (satisfied, missing, unknown) is displayed.

---

### User Story 3 - Export Capability Pack from Repository (Priority: P2)

A developer has a well-configured repository and wants to capture its Claude Code setup as a reusable pack. They select which capability types to include, provide a name and description, and export a structured pack that can be applied to other repositories.

**Why this priority**: Pack export is the bridge between "seeing" capabilities and "sharing" them. Without this, users can observe but not act on their configurations.

**Independent Test**: Can be tested by selecting resources from a repository, exporting a pack, and verifying the pack manifest is structurally valid and contains all selected resources with correct metadata.

**Acceptance Scenarios**:

1. **Given** a repository with multiple capability types, **When** the user selects specific resources for export, enters a pack name and version, **Then** the system generates a pack containing a manifest file and all selected resources in the correct directory structure.
2. **Given** a selected resource that references a file that does not exist on disk, **When** exporting, **Then** the system warns the user about the missing file before completing the export.
3. **Given** exported packs, **When** the user views the Pack Library, **Then** all packs are listed with name, version, source repository, and creation date.

---

### User Story 4 - Apply Capability Pack to Target Repository (Priority: P2)

A developer wants to replicate a proven Claude Code setup from one project to another. They select a pack and a target repository, review a dry-run migration plan showing additions, overwrites, and conflicts, choose resolution strategies for each conflict, and execute the migration.

**Why this priority**: Migration is the primary action users take after understanding their capabilities. It fulfills the core promise of "copy A's setup to B without manual file operations."

**Independent Test**: Can be tested by applying a pack to a target repository, verifying the dry-run plan correctly identifies adds/conflicts/skips, resolving conflicts with a strategy, executing, and confirming all resources are correctly written to the target.

**Acceptance Scenarios**:

1. **Given** a pack and a target repository, **When** the user initiates migration, **Then** the system generates a dry-run plan showing: new resources to add, existing resources that would be overwritten, conflicts with resolution options, and missing dependencies.
2. **Given** a migration plan with name conflicts on the target, **When** the user chooses "rename" for one resource and "skip" for another, **Then** the system applies those strategies during execution.
3. **Given** an executed migration, **When** it completes, **Then** a migration report is generated listing what was added, overwritten, skipped, or failed, with next-step recommendations.
4. **Given** a migration that fails partway through, **When** execution is interrupted, **Then** the target repository is left in a consistent state (not half-migrated), and failures are clearly reported.

---

### User Story 5 - Diagnose Repository Configuration Health (Priority: P2)

A developer wants to check whether a repository's Claude Code configuration is complete, valid, and reproducible. They run a doctor diagnostic that checks skill structure, dependency integrity, hook target existence, environment variable placeholders, and overall health.

**Why this priority**: Doctor diagnostics give users confidence that their configurations work. It catches issues that manual inspection would miss, especially after migration.

**Independent Test**: Can be tested by running doctor on repositories with known issues (missing hook scripts, incomplete skill structures, unresolved env placeholders), verifying all issues are detected with appropriate severity levels and actionable recommendations.

**Acceptance Scenarios**:

1. **Given** a repository with an incomplete skill directory (missing required files), **When** doctor runs, **Then** a structural issue is reported with severity "warning" and a recommendation to add the missing files.
2. **Given** a repository where a hook references a script that does not exist, **When** doctor runs, **Then** a dependency issue is reported with severity "critical" and the specific missing file path.
3. **Given** a pack applied to a repository that has unresolved environment variable placeholders, **When** doctor runs, **Then** env issues are reported with the variable names and suggestions for setting them.
4. **Given** a repository that passes all checks, **When** doctor runs, **Then** a health score is displayed with no critical or warning issues.

---

### User Story 6 - Compare Repositories and Detect Drift (Priority: P3)

A tech lead wants to check whether team repositories conform to the team baseline. They compare a repository against a baseline pack and see what's missing, what's extra, what's modified, and what's consistent. For individual use, a developer compares two repositories to understand configuration differences.

**Why this priority**: Comparison and drift detection are important for team governance but are secondary to the core export-migrate-diagnose loop for individual developers.

**Independent Test**: Can be tested by comparing a repository against a baseline pack, verifying that differences are categorized as missing/extra/modified/same, and confirming that comparison results are actionable.

**Acceptance Scenarios**:

1. **Given** a repository and a baseline pack, **When** the user runs a comparison, **Then** results show resources categorized as: missing from repo, extra in repo (not in baseline), modified from baseline, and matching baseline.
2. **Given** two repositories with different capability sets, **When** comparing them, **Then** the diff clearly shows which resources exist only in one repo, exist in both, or differ in content.

---

### Edge Cases

- What happens when a scanned directory contains symbolic links or nested Git worktrees? The system must deduplicate paths and correctly identify each worktree root.
- What happens when a repository has no Claude Code configurations at all? The system must show an empty inventory with a helpful empty state message rather than an error.
- What happens when the user tries to export a pack with zero selected resources? The system must validate and prevent creating an empty pack.
- What happens when applying a pack to the same repository it was exported from? The system must detect the circular reference and warn the user.
- What happens when a file system path exceeds OS limits (e.g., deeply nested directories)? The system must handle paths gracefully without crashing.
- What happens when two discovered repositories share the same Git directory (e.g., submodules)? The system must correctly identify and de-duplicate them.
- What happens when disk space runs out during pack creation or migration? The system must fail with a clear error message and not leave corrupted partial files.

## Requirements

### Functional Requirements

- **FR-001**: System MUST allow users to add, remove, and manage multiple scan root directories for repository discovery, with a guided setup page on first launch that allows skipping to an empty state.
- **FR-002**: System MUST recursively scan directories to discover Git repositories by detecting `.git` directories, with configurable scan depth (default 5 levels) and user-customizable directory ignore rules (default ignoring common non-code directories: node_modules, .venv, vendor, .cache, build).
- **FR-003**: System MUST record repository metadata including name, absolute path, remote URL, current branch, HEAD commit hash, dirty state, and last indexed timestamp.
- **FR-004**: System MUST support searching, filtering, and sorting the repository list.
- **FR-005**: System MUST support manual refresh of individual repositories and batch refresh of all repositories. On application startup, the system MUST perform background incremental refresh of Git metadata (branch, dirty state, commit) for all indexed repositories without blocking the UI. Capability resource re-parsing remains manual-only. File system watching for automatic refresh is an optional feature.
- **FR-006**: System MUST identify and parse Claude Code capability resources: Skills, MCP Servers, Hooks, Rules, Agents, Commands, Plugins, and Settings fragments.
- **FR-007**: System MUST distinguish between project-level, local, and user-inherited configuration scopes.
- **FR-008**: System MUST display detailed information for each capability resource including name, type, file location, scope, Git tracking status, and dependencies.
- **FR-009**: System MUST handle configuration parse errors gracefully by displaying errors alongside affected resources rather than dropping them.
- **FR-010**: System MUST allow exporting selected capability resources from a repository into a structured pack with a manifest file.
- **FR-011**: System MUST validate pack structure against a schema and report missing files or invalid resources before export completes.
- **FR-012**: System MUST maintain a local pack library showing all exported packs with their metadata, with packs stored in a configurable directory (default: application data directory).
- **FR-013**: System MUST generate a dry-run migration plan before any file writes occur, showing additions, overwrites, conflicts, and missing dependencies.
- **FR-014**: System MUST support conflict resolution strategies: skip, overwrite, rename, and merge.
- **FR-015**: System MUST execute migrations based on the approved plan and user-selected conflict strategies.
- **FR-016**: System MUST generate a migration report after execution listing applied, skipped, and failed items.
- **FR-017**: System MUST create a snapshot before migration to support rollback of applied changes.
- **FR-018**: System MUST check skill directory structure completeness and report missing required files.
- **FR-019**: System MUST check hook target existence (referenced scripts/files) and report missing dependencies.
- **FR-020**: System MUST check for unresolved environment variable placeholders in configurations.
- **FR-021**: System MUST generate a health score using weighted deduction (starting from 100, each critical issue deducts 20 points, each warning deducts 5 points, each info deducts 1 point, minimum 0) and categorize issues by severity (critical, warning, info).
- **FR-022**: System MUST support comparison between two repositories, between a repository and a pack, and between two packs.
- **FR-023**: System MUST categorize comparison results as missing, extra, modified, same, or unknown.
- **FR-024**: System MUST persist repository metadata and scan results to survive application restarts.
- **FR-025**: System MUST present all write operations with a preview before execution and require explicit user confirmation.

### Key Entities

- **Repository**: A local Git repository identified by a `.git` directory. Stores path, name, remote URL, current branch, HEAD commit, dirty state, and last-indexed timestamp.
- **CapabilityResource**: A single Claude Code configuration asset (skill, MCP server, hook, rule, agent, command, plugin, or settings fragment). Linked to a repository or pack. Stores type, name, source path, scope, Git tracking status, content hash, and dependency references.
- **CapabilityPack**: A portable collection of capability resources with a manifest file. Stores name, version, description, creation timestamp, source repository reference, and contained resources. Supports project packs (from a repo), blueprint packs (reusable templates), and baseline packs (team standards).
- **MigrationPlan**: A pre-execution plan for applying a pack to a target repository. Contains planned actions (add, overwrite, skip, rename, merge) per resource, identified conflicts with resolution recommendations, and missing dependency warnings.
- **DoctorReport**: A diagnostic result for a repository. Contains an overall health score, categorized issues with severity levels (critical, warning, info), impacted resource references, and actionable recommendations.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Users can scan a directory containing up to 500 Git repositories and see all discovered repos listed within 30 seconds.
- **SC-002**: Users can identify all Claude Code capabilities in a repository within 10 seconds of opening its detail view.
- **SC-003**: Users can export a capability pack from a well-configured repository in under 2 minutes (from selecting resources to having a valid pack).
- **SC-004**: Users can migrate a capability pack to a target repository in under 3 minutes (from selecting pack to completed migration with report).
- **SC-005**: Migration dry-run plans correctly identify 100% of path conflicts, name collisions, and missing dependencies before any files are written.
- **SC-006**: Doctor diagnostics detect at least 90% of known configuration issues (missing files, broken references, unresolved placeholders) in test repositories.
- **SC-007**: The application remains responsive and does not crash when a single repository contains unparseable or corrupted configuration files.
- **SC-008**: Users can successfully roll back a migration within 1 minute, restoring the target repository to its pre-migration state.

## Assumptions

- Target users are developers familiar with Git and Claude Code who have multiple local repositories.
- Users have a Linux desktop environment; macOS and Windows are out of scope for MVP.
- The application runs locally with no cloud synchronization or remote services.
- Claude Code configuration files follow documented conventions (`.claude/` directory, standard manifest formats).
- Git is installed and accessible on the user's system.
- Users understand basic Claude Code concepts (skills, MCP, hooks, rules, agents).
- The initial release supports a single-user local workflow; team sharing is via manual pack distribution.
- Repository discovery is limited to local filesystem paths; remote Git URLs are recorded as metadata but not cloned automatically.
