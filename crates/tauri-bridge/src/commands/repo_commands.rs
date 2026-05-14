use std::collections::HashMap;

use tauri::State;

use crate::state::{AppSettings, AppState};
use claude_parser;
use domain::{CapabilityResource, DoctorReport, Repository};
use repo_scanner::{RepoCatalog, ScanError, ScannerConfig};
use storage::{repo_store::RepositoryStore, resource_store::ResourceStore};

#[derive(serde::Serialize)]
pub struct RepositorySummary {
    pub id: String,
    pub name: String,
    pub path: String,
    pub branch: Option<String>,
    pub dirty_state: String,
    pub capability_counts: HashMap<String, usize>,
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
    pub errors: Vec<ScanErrorOutput>,
}

#[derive(serde::Serialize)]
pub struct ScanErrorOutput {
    pub path: String,
    pub message: String,
}

impl From<ScanError> for ScanErrorOutput {
    fn from(e: ScanError) -> Self {
        ScanErrorOutput {
            path: e.repo_path,
            message: e.message,
        }
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
pub fn scan_repositories(
    paths: Vec<String>,
    state: State<AppState>,
) -> Result<ScanResultOutput, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?;

    let config = ScannerConfig {
        root_paths: paths,
        max_depth: settings.scan_depth as usize,
        ignore_dirs: settings.ignore_patterns.clone(),
    };

    let scan_result = RepoCatalog::scan(config).map_err(|e| format!("Scan failed: {}", e))?;

    let db = state.db.lock().map_err(|e| e.to_string())?;
    let repo_store = RepositoryStore::new(&db);

    let mut repos_added = 0u32;
    let mut repos_updated = 0u32;
    let mut stored_errors: Vec<ScanError> = Vec::new();

    for repo in &scan_result.repos {
        match repo_store.get_by_path(&repo.path) {
            Ok(Some(_existing)) => {
                repos_updated += 1;
                if let Err(e) = repo_store.update(repo) {
                    stored_errors.push(ScanError {
                        repo_path: repo.path.clone(),
                        message: format!("Failed to update repo: {}", e),
                    });
                }
            }
            Ok(None) => {
                repos_added += 1;
                if let Err(e) = repo_store.insert(repo) {
                    stored_errors.push(ScanError {
                        repo_path: repo.path.clone(),
                        message: format!("Failed to insert repo: {}", e),
                    });
                }
            }
            Err(e) => {
                stored_errors.push(ScanError {
                    repo_path: repo.path.clone(),
                    message: format!("Database error: {}", e),
                });
            }
        }
    }

    let mut all_errors = scan_result.errors;
    all_errors.extend(stored_errors);

    let errors: Vec<ScanErrorOutput> = all_errors.into_iter().map(ScanErrorOutput::from).collect();

    Ok(ScanResultOutput {
        repos_found: scan_result.repos.len() as u32,
        repos_added,
        repos_updated,
        errors,
    })
}

#[tauri::command]
pub fn list_repositories(
    filter: RepoFilter,
    state: State<AppState>,
) -> Result<Vec<RepositorySummary>, String> {
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

        let resources = resource_store
            .get_by_repo(&repo.id)
            .unwrap_or_default();

        if filter.has_capabilities.unwrap_or(false) && resources.is_empty() {
            continue;
        }

        let mut capability_counts: HashMap<String, usize> = HashMap::new();
        for r in &resources {
            *capability_counts.entry(r.r#type.clone()).or_insert(0) += 1;
        }

        summaries.push(RepositorySummary {
            id: repo.id,
            name: repo.name,
            path: repo.path,
            branch: repo.current_branch,
            dirty_state: repo.dirty_state,
            capability_counts,
            doctor_score: None,
            last_indexed_at: repo.last_indexed_at,
        });
    }

    if let Some(ref sort_by) = filter.sort_by {
        let desc = filter.sort_order.as_deref() == Some("desc");
        match sort_by.as_str() {
            "name" => summaries.sort_by(|a, b| {
                if desc { b.name.cmp(&a.name) } else { a.name.cmp(&b.name) }
            }),
            "path" => summaries.sort_by(|a, b| {
                if desc { b.path.cmp(&a.path) } else { a.path.cmp(&b.path) }
            }),
            "last_indexed_at" => summaries.sort_by(|a, b| {
                if desc { b.last_indexed_at.cmp(&a.last_indexed_at) } else { a.last_indexed_at.cmp(&b.last_indexed_at) }
            }),
            "dirty_state" => summaries.sort_by(|a, b| {
                if desc { b.dirty_state.cmp(&a.dirty_state) } else { a.dirty_state.cmp(&b.dirty_state) }
            }),
            _ => {}
        }
    }

    Ok(summaries)
}

#[tauri::command]
pub fn refresh_repository(
    repo_id: String,
    state: State<AppState>,
) -> Result<RepoDetail, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let repo_store = RepositoryStore::new(&db);

    let mut repo = repo_store
        .get_by_id(&repo_id)
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| format!("Repository not found: {}", repo_id))?;

    // Re-extract Git metadata
    git_service::git_cli::extract_metadata(&mut repo)
        .map_err(|e| format!("Git metadata extraction failed: {}", e))?;

    repo.last_indexed_at = chrono::Utc::now().to_rfc3339();
    repo_store.update(&repo).map_err(|e| format!("Failed to update repo: {}", e))?;

    // Re-parse capabilities
    let parsed = claude_parser::parse_repo(&repo.path)
        .unwrap_or_else(|_| claude_parser::CapabilityInventory {
            skills: vec![],
            mcp: vec![],
            hooks: vec![],
            rules: vec![],
            agents: vec![],
            commands: vec![],
            plugins: vec![],
            settings: vec![],
        });

    let resource_store = ResourceStore::new(&db);
    resource_store
        .delete_by_repo(&repo_id)
        .map_err(|e| format!("Failed to clear old resources: {}", e))?;

    let all_resources = collect_resources(&repo_id, &parsed);
    if !all_resources.is_empty() {
        resource_store
            .insert_batch(&all_resources)
            .map_err(|e| format!("Failed to store resources: {}", e))?;
    }

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

    Ok(RepoDetail {
        repo,
        capabilities: inventory,
        doctor_latest: None,
    })
}

#[tauri::command]
pub fn get_repository_detail(
    repo_id: String,
    state: State<AppState>,
) -> Result<RepoDetail, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let repo_store = RepositoryStore::new(&db);
    let resource_store = ResourceStore::new(&db);

    let repo = repo_store
        .get_by_id(&repo_id)
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| format!("Repository not found: {}", repo_id))?;

    let resources = resource_store
        .get_by_repo(&repo_id)
        .unwrap_or_default();

    let inventory = group_resources(resources);

    Ok(RepoDetail {
        repo,
        capabilities: inventory,
        doctor_latest: None,
    })
}

#[tauri::command]
pub fn remove_repository(
    repo_id: String,
    state: State<AppState>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let repo_store = RepositoryStore::new(&db);

    repo_store
        .get_by_id(&repo_id)
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| format!("Repository not found: {}", repo_id))?;

    repo_store
        .delete(&repo_id)
        .map_err(|e| format!("Failed to remove repository: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<AppSettings, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    Ok(settings.clone())
}

#[tauri::command]
pub fn update_settings(
    new_settings: AppSettings,
    state: State<AppState>,
) -> Result<(), String> {
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    *settings = new_settings;
    Ok(())
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

    CapabilityInventory {
        skills,
        mcp,
        hooks,
        rules,
        agents,
        commands,
        plugins,
        settings,
    }
}
