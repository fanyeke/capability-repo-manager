use std::collections::HashMap;

use tauri::State;

use crate::state::{AppSettings, AppState};
use claude_parser;
use domain::{CapabilityResource, DoctorReport, OperationContext, Repository};
use repo_scanner::{RepoCatalog, ScanError, ScannerConfig};
use storage::event_store::{NewOperationEvent, OperationEvent};
use storage::{repo_store::RepositoryStore, resource_store::ResourceStore};

use super::{doctor_commands, settings_commands};

#[derive(serde::Serialize)]
pub struct RepositorySummary {
    pub id: String,
    pub name: String,
    pub path: String,
    pub branch: Option<String>,
    pub dirty_state: String,
    pub capability_counts: HashMap<String, usize>,
    pub capability_index_status: String,
    pub last_capability_error: Option<String>,
    pub doctor_score: Option<i32>,
    pub last_indexed_at: String,
}

#[derive(serde::Serialize)]
pub struct RepoDetail {
    pub repo: Repository,
    pub capabilities: CapabilityInventory,
    pub doctor_latest: Option<DoctorReport>,
}

#[derive(serde::Serialize)]
pub struct CapabilityInventory {
    pub skills: Vec<CapabilityResource>,
    pub mcp: Vec<CapabilityResource>,
    pub hooks: Vec<CapabilityResource>,
    pub rules: Vec<CapabilityResource>,
    pub agents: Vec<CapabilityResource>,
    pub commands: Vec<CapabilityResource>,
    pub plugins: Vec<CapabilityResource>,
    pub settings: Vec<CapabilityResource>,
}

#[derive(serde::Serialize)]
pub struct ScanResultOutput {
    pub repos_found: u32,
    pub repos_added: u32,
    pub repos_updated: u32,
    pub repos_parsed: u32,
    pub repos_parse_failed: u32,
    pub errors: Vec<ScanErrorOutput>,
}

#[derive(serde::Serialize)]
pub struct ScanErrorOutput {
    pub path: String,
    pub message: String,
}

impl From<ScanError> for ScanErrorOutput {
    fn from(e: ScanError) -> Self {
        ScanErrorOutput { path: e.repo_path, message: e.message }
    }
}

#[derive(serde::Deserialize)]
pub struct RepoFilter {
    pub search: Option<String>,
    pub dirty_only: Option<bool>,
    pub has_capabilities: Option<bool>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[tauri::command]
pub fn scan_repositories(paths: Vec<String>, state: State<AppState>) -> Result<ScanResultOutput, String> {
    let ctx = OperationContext::new("scan_repositories");
    let _span = tracing::info_span!(
        "scan_repositories",
        operation_id = %ctx.operation_id,
    )
    .entered();

    let settings = state.settings.lock().map_err(|e| e.to_string())?;

    let config = ScannerConfig {
        root_paths: paths.clone(),
        max_depth: settings.scan_depth as usize,
        ignore_dirs: settings.ignore_patterns.clone(),
    };

    tracing::info!(
        operation_id = %ctx.operation_id,
        roots = ?paths,
        max_depth = config.max_depth,
        "scan_started"
    );

    let scan_result = RepoCatalog::scan(config).map_err(|e| format!("Scan failed: {}", e))?;

    // Log each discovered repo
    for repo in &scan_result.repos {
        tracing::debug!(
            operation_id = %ctx.operation_id,
            path = %repo.path,
            canonical_path = %repo.canonical_path,
            "repo_discovered"
        );
    }

    // Log git metadata failures
    for err in &scan_result.errors {
        tracing::warn!(
            operation_id = %ctx.operation_id,
            repo_path = %err.repo_path,
            error = %err.message,
            "git_metadata_failed"
        );
    }

    let db = state.db.lock().map_err(|e| e.to_string())?;
    let repo_store = RepositoryStore::new(&db);

    // Phase 2: For each repo, use upsert_by_path which handles identity via canonical path
    let mut repos_added = 0u32;
    let mut repos_updated = 0u32;
    let mut repos_parsed = 0u32;
    let mut repos_parse_failed = 0u32;
    let mut stored_errors: Vec<ScanError> = Vec::new();

    for repo in &scan_result.repos {
        let is_new = repo_store.get_by_canonical_path(&repo.canonical_path).unwrap_or(None).is_none();

        match repo_store.upsert_by_path(repo) {
            Ok(stored) => {
                if is_new {
                    repos_added += 1;
                } else {
                    repos_updated += 1;
                }

                tracing::info!(
                    operation_id = %ctx.operation_id,
                    repo_id = %stored.id,
                    repo_path = %repo.path,
                    "capability_parse_started"
                );

                // Phase 3: Capability parsing
                match claude_parser::parse_repo(&repo.path) {
                    Ok(inventory) => {
                        let all_resources = collect_resources(&stored.id, &inventory);
                        let resource_store = ResourceStore::new(&db);
                        if let Err(e) = resource_store.replace_for_repo(&stored.id, &all_resources) {
                            stored_errors.push(ScanError {
                                repo_path: repo.path.clone(),
                                message: format!("Failed to store capabilities: {}", e),
                            });
                        }
                        if let Err(e) = repo_store.update_index_status(&stored.id, "fresh", None) {
                            stored_errors.push(ScanError {
                                repo_path: repo.path.clone(),
                                message: format!("Failed to update index status: {}", e),
                            });
                        }
                        repos_parsed += 1;

                        tracing::info!(
                            operation_id = %ctx.operation_id,
                            repo_id = %stored.id,
                            skills = inventory.skills.len(),
                            mcp = inventory.mcp.len(),
                            hooks = inventory.hooks.len(),
                            rules = inventory.rules.len(),
                            agents = inventory.agents.len(),
                            "capability_parse_finished"
                        );
                    }
                    Err(e) => {
                        let err_msg = e.to_string();
                        if let Err(db_err) = repo_store.update_index_status(&stored.id, "parse_failed", Some(&err_msg))
                        {
                            stored_errors.push(ScanError {
                                repo_path: repo.path.clone(),
                                message: format!("Failed to record parse error: {}", db_err),
                            });
                        }
                        repos_parse_failed += 1;

                        tracing::warn!(
                            operation_id = %ctx.operation_id,
                            repo_id = %stored.id,
                            error = %err_msg,
                            "parse_failed"
                        );
                    }
                }
            }
            Err(e) => {
                stored_errors
                    .push(ScanError { repo_path: repo.path.clone(), message: format!("Failed to upsert repo: {}", e) });
            }
        }
    }

    let mut all_errors = scan_result.errors;
    all_errors.extend(stored_errors);

    let errors: Vec<ScanErrorOutput> = all_errors.into_iter().map(ScanErrorOutput::from).collect();

    let repos_found = scan_result.repos.len() as u32;

    tracing::info!(
        operation_id = %ctx.operation_id,
        repos_found = repos_found,
        repos_added = repos_added,
        repos_updated = repos_updated,
        repos_parsed = repos_parsed,
        repos_parse_failed = repos_parse_failed,
        "scan_finished"
    );

    // T010: Record OperationEvent to DB
    let status = if repos_parse_failed > 0 { "partial_failure" } else { "success" };
    let summary = format!(
        "Scanned {} directories, found {} repos ({} added, {} updated, {} parsed, {} failed)",
        paths.len(),
        repos_found,
        repos_added,
        repos_updated,
        repos_parsed,
        repos_parse_failed,
    );

    let event_store = db.event_store();
    if let Err(e) = event_store.insert_event(NewOperationEvent {
        operation_id: ctx.operation_id,
        operation_type: "scan_repositories".to_string(),
        status: status.to_string(),
        repo_id: None,
        pack_id: None,
        migration_run_id: None,
        summary: Some(summary),
        detail_json: None,
    }) {
        tracing::warn!(error = %e, "failed_to_record_operation_event");
    }

    Ok(ScanResultOutput { repos_found, repos_added, repos_updated, repos_parsed, repos_parse_failed, errors })
}

#[tauri::command]
pub fn list_repositories(filter: RepoFilter, state: State<AppState>) -> Result<Vec<RepositorySummary>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let repo_store = RepositoryStore::new(&db);
    let resource_store = ResourceStore::new(&db);

    let repos = repo_store.list_all().map_err(|e| format!("Database error: {}", e))?;

    let mut summaries = Vec::new();

    for repo in repos {
        if let Some(ref search) = filter.search {
            let s = search.to_lowercase();
            if !repo.name.to_lowercase().contains(&s) && !repo.path.to_lowercase().contains(&s) {
                continue;
            }
        }

        if filter.dirty_only.unwrap_or(false) && repo.dirty_state == "clean" {
            continue;
        }

        let resources = resource_store.get_by_repo(&repo.id).unwrap_or_default();

        if filter.has_capabilities.unwrap_or(false) && resources.is_empty() {
            continue;
        }

        let mut capability_counts: HashMap<String, usize> = HashMap::new();
        for r in &resources {
            *capability_counts.entry(r.r#type.clone()).or_insert(0) += 1;
        }

        let doctor_score = doctor_commands::query_latest_report(&repo.id, &db).ok().flatten().map(|r| r.score);

        summaries.push(RepositorySummary {
            id: repo.id,
            name: repo.name,
            path: repo.path,
            branch: repo.current_branch,
            dirty_state: repo.dirty_state,
            capability_counts,
            capability_index_status: repo.capability_index_status,
            last_capability_error: repo.last_capability_error,
            doctor_score,
            last_indexed_at: repo.last_indexed_at,
        });
    }

    if let Some(ref sort_by) = filter.sort_by {
        let desc = filter.sort_order.as_deref() == Some("desc");
        match sort_by.as_str() {
            "name" => summaries.sort_by(|a, b| if desc { b.name.cmp(&a.name) } else { a.name.cmp(&b.name) }),
            "path" => summaries.sort_by(|a, b| if desc { b.path.cmp(&a.path) } else { a.path.cmp(&b.path) }),
            "last_indexed_at" => summaries.sort_by(|a, b| {
                if desc {
                    b.last_indexed_at.cmp(&a.last_indexed_at)
                } else {
                    a.last_indexed_at.cmp(&b.last_indexed_at)
                }
            }),
            "dirty_state" => summaries.sort_by(|a, b| {
                if desc {
                    b.dirty_state.cmp(&a.dirty_state)
                } else {
                    a.dirty_state.cmp(&b.dirty_state)
                }
            }),
            _ => {}
        }
    }

    Ok(summaries)
}

#[tauri::command]
pub fn refresh_repository(repo_id: String, state: State<AppState>) -> Result<RepoDetail, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let repo_store = RepositoryStore::new(&db);

    let mut repo = repo_store
        .get_by_id(&repo_id)
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| format!("Repository not found: {}", repo_id))?;

    // Re-extract Git metadata
    git_service::git_cli::extract_metadata(&mut repo).map_err(|e| format!("Git metadata extraction failed: {}", e))?;

    repo.last_indexed_at = chrono::Utc::now().to_rfc3339();
    repo_store.update(&repo).map_err(|e| format!("Failed to update repo: {}", e))?;

    // Re-parse capabilities
    let resource_store = ResourceStore::new(&db);

    match claude_parser::parse_repo(&repo.path) {
        Ok(parsed) => {
            let all_resources = collect_resources(&repo_id, &parsed);
            resource_store
                .replace_for_repo(&repo_id, &all_resources)
                .map_err(|e| format!("Failed to replace resources: {}", e))?;
            repo_store
                .update_index_status(&repo_id, "fresh", None)
                .map_err(|e| format!("Failed to update index status: {}", e))?;

            let inventory = CapabilityInventory {
                skills: parsed.skills,
                mcp: parsed.mcp,
                hooks: parsed.hooks,
                rules: parsed.rules,
                agents: parsed.agents,
                commands: parsed.commands,
                plugins: parsed.plugins,
                settings: parsed.settings,
            };

            let doctor_latest = doctor_commands::query_latest_report(&repo_id, &db).ok().flatten();

            Ok(RepoDetail { repo, capabilities: inventory, doctor_latest })
        }
        Err(e) => {
            let err_msg = e.to_string();
            repo_store
                .update_index_status(&repo_id, "parse_failed", Some(&err_msg))
                .map_err(|e| format!("Failed to record parse error: {}", e))?;

            // Re-fetch repo with updated status for response
            let updated_repo = repo_store
                .get_by_id(&repo_id)
                .map_err(|e| format!("Database error: {}", e))?
                .ok_or_else(|| format!("Repository not found after refresh: {}", repo_id))?;

            // Return existing (old) resources — NOT deleted
            let existing_resources = resource_store.get_by_repo(&repo_id).unwrap_or_default();
            let inventory = group_resources(existing_resources);
            let doctor_latest = doctor_commands::query_latest_report(&repo_id, &db).ok().flatten();

            Ok(RepoDetail { repo: updated_repo, capabilities: inventory, doctor_latest })
        }
    }
}

#[tauri::command]
pub fn refresh_all_repositories(state: State<AppState>) -> Result<u32, String> {
    let ctx = OperationContext::new("refresh_all_repositories");

    let db = state.db.lock().map_err(|e| e.to_string())?;
    let repo_store = RepositoryStore::new(&db);

    let mut repos = repo_store.list_all().map_err(|e| format!("Database error: {}", e))?;

    if repos.is_empty() {
        return Ok(0u32);
    }

    let errors = RepoCatalog::refresh_all(&mut repos);

    for repo in &repos {
        if let Err(e) = repo_store.update(repo) {
            tracing::warn!(
                operation_id = %ctx.operation_id,
                repo_name = %repo.name,
                error = %e,
                "failed_to_update_repo"
            );
        }
    }

    if !errors.is_empty() {
        for err in &errors {
            tracing::warn!(
                operation_id = %ctx.operation_id,
                repo_path = %err.repo_path,
                error = %err.message,
                "startup_refresh_warning"
            );
        }
    }

    Ok(repos.len() as u32)
}

#[tauri::command]
pub fn get_repository_detail(repo_id: String, state: State<AppState>) -> Result<RepoDetail, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let repo_store = RepositoryStore::new(&db);
    let resource_store = ResourceStore::new(&db);

    let repo = repo_store
        .get_by_id(&repo_id)
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| format!("Repository not found: {}", repo_id))?;

    let resources = resource_store.get_by_repo(&repo_id).unwrap_or_default();

    let inventory = group_resources(resources);
    let doctor_latest = doctor_commands::query_latest_report(&repo_id, &db).ok().flatten();

    Ok(RepoDetail { repo, capabilities: inventory, doctor_latest })
}

#[tauri::command]
pub fn remove_repository(repo_id: String, state: State<AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let repo_store = RepositoryStore::new(&db);

    repo_store
        .get_by_id(&repo_id)
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| format!("Repository not found: {}", repo_id))?;

    repo_store.delete_cascade(&repo_id).map_err(|e| format!("Failed to remove repository: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<AppSettings, String> {
    // Try loading from file first, fall back to in-memory defaults
    if let Some(file_settings) = settings_commands::load_from_file() {
        let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
        *settings = file_settings.clone();
        return Ok(file_settings);
    }
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    Ok(settings.clone())
}

#[tauri::command]
pub fn update_settings(new_settings: AppSettings, state: State<AppState>) -> Result<(), String> {
    // Persist to file
    settings_commands::save_to_file(&new_settings)?;
    // Update in-memory state
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    *settings = new_settings;
    Ok(())
}

#[tauri::command]
pub fn list_operation_events(
    limit: Option<i64>,
    offset: Option<i64>,
    operation_type: Option<String>,
    state: State<AppState>,
) -> Result<Vec<OperationEvent>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let event_store = db.event_store();
    event_store
        .list_events(limit.unwrap_or(50), offset.unwrap_or(0), operation_type.as_deref())
        .map_err(|e| format!("Failed to list operation events: {}", e))
}

fn collect_resources(repo_id: &str, inv: &claude_parser::CapabilityInventory) -> Vec<CapabilityResource> {
    let mut all = Vec::new();
    for mut r in inv.skills.iter().cloned() {
        r.repo_id = Some(repo_id.to_string());
        all.push(r);
    }
    for mut r in inv.mcp.iter().cloned() {
        r.repo_id = Some(repo_id.to_string());
        all.push(r);
    }
    for mut r in inv.hooks.iter().cloned() {
        r.repo_id = Some(repo_id.to_string());
        all.push(r);
    }
    for mut r in inv.rules.iter().cloned() {
        r.repo_id = Some(repo_id.to_string());
        all.push(r);
    }
    for mut r in inv.agents.iter().cloned() {
        r.repo_id = Some(repo_id.to_string());
        all.push(r);
    }
    for mut r in inv.commands.iter().cloned() {
        r.repo_id = Some(repo_id.to_string());
        all.push(r);
    }
    for mut r in inv.plugins.iter().cloned() {
        r.repo_id = Some(repo_id.to_string());
        all.push(r);
    }
    for mut r in inv.settings.iter().cloned() {
        r.repo_id = Some(repo_id.to_string());
        all.push(r);
    }
    all
}

fn group_resources(resources: Vec<CapabilityResource>) -> CapabilityInventory {
    let mut skills = Vec::new();
    let mut mcp = Vec::new();
    let mut hooks = Vec::new();
    let mut rules = Vec::new();
    let mut agents = Vec::new();
    let mut commands = Vec::new();
    let mut plugins = Vec::new();
    let mut settings = Vec::new();

    for r in resources {
        match r.r#type.as_str() {
            "skill" => skills.push(r),
            "mcp" => mcp.push(r),
            "hook" => hooks.push(r),
            "rule" => rules.push(r),
            "agent" => agents.push(r),
            "command" => commands.push(r),
            "plugin" => plugins.push(r),
            "settings" => settings.push(r),
            _ => {}
        }
    }

    CapabilityInventory { skills, mcp, hooks, rules, agents, commands, plugins, settings }
}
