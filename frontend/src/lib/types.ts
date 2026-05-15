// ============================================================
// Domain types — mirrors Rust domain crate
// ============================================================

export interface Repository {
  id: string;
  name: string;
  path: string;
  canonical_path: string;
  remote_url: string | null;
  current_branch: string | null;
  head_commit: string | null;
  dirty_state: "clean" | "modified" | "unknown";
  first_indexed_at: string;
  last_indexed_at: string;
  capability_index_status: "never_indexed" | "fresh" | "stale" | "parse_failed";
  last_capability_indexed_at: string | null;
  last_capability_error: string | null;
}

export interface RepositorySummary {
  id: string;
  name: string;
  path: string;
  branch: string | null;
  dirty_state: "clean" | "modified" | "unknown";
  capability_counts: {
    skills: number;
    mcp: number;
    hooks: number;
    rules: number;
    agents: number;
  };
  capability_index_status: "never_indexed" | "fresh" | "stale" | "parse_failed";
  last_capability_error: string | null;
  doctor_score: number | null;
  last_indexed_at: string;
}

export interface RepoDetail {
  repo: Repository;
  capabilities: CapabilityInventory;
  doctor_latest: DoctorReport | null;
}

export interface CapabilityResource {
  id: string;
  repo_id: string | null;
  pack_id: string | null;
  type: ResourceType;
  name: string;
  source_path: string | null;
  scope: "project" | "local" | "user" | "inherited" | "unknown";
  tracked_by_git: boolean;
  content_hash: string | null;
  metadata_json: string | null;
  error_message: string | null;
}

export type ResourceType =
  | "skill"
  | "mcp"
  | "hook"
  | "rule"
  | "agent"
  | "command"
  | "plugin"
  | "settings"
  | "contextDoc";

export interface CapabilityInventory {
  skills: CapabilityResource[];
  mcp: CapabilityResource[];
  hooks: CapabilityResource[];
  rules: CapabilityResource[];
  agents: CapabilityResource[];
  commands: CapabilityResource[];
  plugins: CapabilityResource[];
  settings: CapabilityResource[];
}

// ============================================================
// Query types
// ============================================================

export interface RepoFilter {
  search?: string;
  dirty_only?: boolean;
  has_capabilities?: boolean;
  sort_by?: "name" | "path" | "last_indexed_at" | "dirty_state";
  sort_order?: "asc" | "desc";
}

export interface ScanResult {
  repos_found: number;
  repos_added: number;
  repos_updated: number;
  repos_parsed: number;
  repos_parse_failed: number;
  errors: ScanError[];
}

export interface ScanError {
  path: string;
  message: string;
}

// ============================================================
// Pack types
// ============================================================

export interface PackSummary {
  id: string;
  name: string;
  version: string;
  description: string | null;
  pack_type: "project" | "blueprint" | "baseline";
  resource_count: number;
  source_repo_name: string | null;
  created_at: string;
}

export interface PackDetail {
  pack: CapabilityPack;
  resources: CapabilityResource[];
  manifest: ManifestData;
}

export interface CapabilityPack {
  id: string;
  name: string;
  version: string;
  description: string | null;
  pack_type: "project" | "blueprint" | "baseline";
  manifest_path: string;
  source_repo_id: string | null;
  source_commit: string | null;
  created_at: string;
  storage_dir: string;
}

export interface PackSelection {
  resource_ids: string[];
}

export interface PackMetadata {
  name: string;
  version: string;
  description?: string;
  pack_type: string;
}

export interface ManifestData {
  schemaVersion: string;
  name: string;
  version: string;
  description?: string;
  packType?: string;
  source?: { repo?: string; commit?: string };
  resources: ManifestResource[];
  env?: ManifestEnvVar[];
  validation?: { rules: string[] };
}

export interface ManifestResource {
  type: ResourceType;
  name: string;
  source: string;
  hash?: string;
  dependencies?: ResourceDependency[];
}

export interface ManifestEnvVar {
  name: string;
  required: boolean;
  description?: string;
}

export interface ResourceDependency {
  type: "file" | "command" | "env" | "mcp" | "skill" | "hook" | "plugin";
  name: string;
  required: boolean;
  status: "satisfied" | "missing" | "unknown";
}

export interface PackFilter {
  search?: string;
  pack_type?: string;
}

export interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
}

export interface ValidationError {
  resource_ref: string | null;
  message: string;
}

// ============================================================
// Migration types
// ============================================================

export interface MigrationPlan {
  plan_id: string;
  source_type: string;
  source_id: string;
  target_repo_id: string;
  items: MigrationPlanItem[];
  conflicts: MigrationConflict[];
  missing_dependencies: Dependency[];
}

export interface MigrationPlanItem {
  resource_id: string;
  action: "add" | "overwrite" | "skip" | "rename" | "merge" | "unresolved";
  source_path: string | null;
  target_path: string | null;
  status: "pending" | "applied" | "failed";
}

export interface MigrationConflict {
  resource_name: string;
  resource_type: ResourceType;
  reason: "same_name" | "same_path" | "incompatible_schema" | "ambiguous_merge";
  recommended_actions: string[];
}

export interface Dependency {
  type: string;
  name: string;
  required: boolean;
  status: "satisfied" | "missing" | "unknown";
}

export interface ConflictStrategy {
  resource_id: string;
  action: "skip" | "overwrite" | "rename" | "merge";
}

export interface MigrationReport {
  status: string;
  items: MigrationReportItem[];
  summary: { added: number; overwritten: number; skipped: number; failed: number };
}

export interface MigrationReportItem {
  resource_name: string;
  action: string;
  status: string;
  error: string | null;
}

export interface MigrationRunSummary {
  id: string;
  source_name: string;
  status: "planned" | "applied" | "failed" | "rolled_back";
  items_count: number;
  created_at: string;
  executed_at: string | null;
}

// ============================================================
// Doctor types
// ============================================================

export interface DoctorReport {
  id: string;
  repo_id: string;
  score: number;
  issues: DoctorIssue[];
  created_at: string;
}

export interface DoctorIssue {
  severity: "critical" | "warning" | "info";
  code: string;
  message: string;
  resource_ref: string | null;
  recommendation: string | null;
}

// ============================================================
// Compare types
// ============================================================

export interface CompareResult {
  missing: CapabilityResource[];
  extra: CapabilityResource[];
  modified: DiffItem[];
  same: CapabilityResource[];
}

export interface DiffItem {
  name: string;
  type: ResourceType;
  source_resource: CapabilityResource;
  target_resource: CapabilityResource;
}

// ============================================================
// Settings types
// ============================================================

export interface AppSettings {
  scan_roots: string[];
  scan_depth: number;
  ignore_patterns: string[];
  pack_storage_dir: string;
  file_watch_enabled: boolean;
}
