use std::path::PathBuf;

use tauri::State;

use crate::state::AppState;
use domain::{CapabilityResource, DoctorIssue, DoctorReport};
use storage::{repo_store::RepositoryStore, resource_store::ResourceStore};

#[derive(serde::Serialize)]
pub struct CompareResult {
    pub missing: Vec<CapabilityResource>,
    pub extra: Vec<CapabilityResource>,
    pub modified: Vec<DiffItem>,
    pub same: Vec<CapabilityResource>,
}

#[derive(serde::Serialize)]
pub struct DiffItem {
    pub name: String,
    pub resource_type: String,
    pub source_resource: CapabilityResource,
    pub target_resource: CapabilityResource,
}

#[tauri::command]
pub fn run_doctor(
    repo_id: String,
    state: State<AppState>,
) -> Result<DoctorReport, String> {
    let ctx = domain::OperationContext::new("run_doctor");
    let _span = tracing::info_span!("run_doctor", operation_id = %ctx.operation_id).entered();

    tracing::info!(
        operation_id = %ctx.operation_id,
        repo_id = %repo_id,
        "doctor_started"
    );

    let db = state.db.lock().map_err(|e| e.to_string())?;

    let repo_store = RepositoryStore::new(&db);
    let repo = repo_store
        .get_by_id(&repo_id)
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| format!("Repository not found: {}", repo_id))?;

    let repo_path = PathBuf::from(&repo.path);

    // Collect hook configs and config texts for doctor engine
    let hooks = extract_hooks(&repo_path);
    let config_texts = collect_config_texts(&repo_path);

    let mcp_json = std::fs::read_to_string(repo_path.join(".claude").join("mcp.json"))
        .ok();

    let report_id = uuid::Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().to_rfc3339();

    let report = doctor_engine::run_doctor(
        &repo_id,
        &repo_path,
        &hooks,
        &config_texts,
        mcp_json.as_deref(),
        &report_id,
        &created_at,
    );

    // Store the report
    let issues_json =
        serde_json::to_string(&report.issues).map_err(|e| format!("Serialization error: {}", e))?;

    db.conn().execute(
        "INSERT INTO doctor_reports (id, repo_id, score, issues_json, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![report.id, report.repo_id, report.score, issues_json, report.created_at],
    ).map_err(|e| format!("Database error: {}", e))?;

    let critical_count = report.issues.iter().filter(|i| i.severity == "critical").count();
    let warning_count = report.issues.iter().filter(|i| i.severity == "warning").count();
    let info_count = report.issues.iter().filter(|i| i.severity == "info").count();

    tracing::info!(
        operation_id = %ctx.operation_id,
        repo_id = %repo_id,
        score = %report.score,
        critical = %critical_count,
        warning = %warning_count,
        info = %info_count,
        "doctor_finished"
    );

    Ok(report)
}

#[tauri::command]
pub fn compare_repo_with_pack(
    repo_id: String,
    pack_id: String,
    state: State<AppState>,
) -> Result<CompareResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let resource_store = ResourceStore::new(&db);

    let repo_resources = resource_store
        .get_by_repo(&repo_id)
        .map_err(|e| format!("Database error: {}", e))?;

    let pack_resources = resource_store
        .get_by_pack(&pack_id)
        .map_err(|e| format!("Database error: {}", e))?;

    Ok(compute_diff(&repo_resources, &pack_resources))
}

#[tauri::command]
pub fn compare_repos(
    repo_id_a: String,
    repo_id_b: String,
    state: State<AppState>,
) -> Result<CompareResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let resource_store = ResourceStore::new(&db);

    let resources_a = resource_store
        .get_by_repo(&repo_id_a)
        .map_err(|e| format!("Database error: {}", e))?;

    let resources_b = resource_store
        .get_by_repo(&repo_id_b)
        .map_err(|e| format!("Database error: {}", e))?;

    Ok(compute_diff(&resources_a, &resources_b))
}

/// Query the latest doctor report for a repository, returning `None` if none exists.
pub fn query_latest_report(
    repo_id: &str,
    db: &storage::Database,
) -> Result<Option<DoctorReport>, String> {
    let mut stmt = db.conn().prepare(
        "SELECT id, repo_id, score, issues_json, created_at
         FROM doctor_reports WHERE repo_id = ?1 ORDER BY created_at DESC LIMIT 1"
    ).map_err(|e| format!("Database error: {}", e))?;

    let mut rows = stmt.query_map(rusqlite::params![repo_id], |row| {
        let issues_json: String = row.get(3)?;
        let issues: Vec<DoctorIssue> = serde_json::from_str(&issues_json).unwrap_or_default();
        Ok(DoctorReport {
            id: row.get(0)?,
            repo_id: row.get(1)?,
            score: row.get(2)?,
            issues,
            created_at: row.get(4)?,
        })
    }).map_err(|e| format!("Database error: {}", e))?;

    match rows.next() {
        Some(Ok(report)) => Ok(Some(report)),
        Some(Err(e)) => Err(format!("Database error: {}", e)),
        None => Ok(None),
    }
}

#[tauri::command]
pub fn get_latest_doctor_report(
    repo_id: String,
    state: State<AppState>,
) -> Result<Option<DoctorReport>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    query_latest_report(&repo_id, &db)
}

fn extract_hooks(repo_path: &std::path::Path) -> Vec<doctor_engine::checks::HookConfig> {
    let settings_path = repo_path.join(".claude").join("settings.json");
    if !settings_path.exists() {
        return Vec::new();
    }

    let content = match std::fs::read_to_string(&settings_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let parsed: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let mut hooks = Vec::new();
    if let Some(hooks_obj) = parsed.get("hooks").and_then(|h| h.as_object()) {
        for (hook_type, hook_defs) in hooks_obj {
            if let Some(arr) = hook_defs.as_array() {
                for entry in arr {
                    hooks.push(doctor_engine::checks::HookConfig {
                        hook_type: hook_type.clone(),
                        name: entry
                            .get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("")
                            .to_string(),
                        command: entry
                            .get("command")
                            .and_then(|c| c.as_str())
                            .map(|s| s.to_string()),
                        script_path: entry
                            .get("script_path")
                            .and_then(|s| s.as_str())
                            .map(|s| s.to_string()),
                    });
                }
            }
        }
    }

    hooks
}

fn collect_config_texts(repo_path: &std::path::Path) -> Vec<(String, String)> {
    let mut texts = Vec::new();

    for fname in &["settings.json", "mcp.json"] {
        let path = repo_path.join(".claude").join(fname);
        if let Ok(content) = std::fs::read_to_string(&path) {
            texts.push((fname.to_string(), content));
        }
    }

    let rules_dir = repo_path.join(".claude").join("rules");
    if rules_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&rules_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("md") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            texts.push((format!("rules/{}", name), content));
                        }
                    }
                }
            }
        }
    }

    texts
}

fn compute_diff(source: &[CapabilityResource], target: &[CapabilityResource]) -> CompareResult {
    let mut missing = Vec::new();
    let mut extra = Vec::new();
    let mut modified = Vec::new();
    let mut same = Vec::new();

    let source_map: std::collections::HashMap<(&str, &str), &CapabilityResource> = source
        .iter()
        .map(|r| ((r.r#type.as_str(), r.name.as_str()), r))
        .collect();

    let target_map: std::collections::HashMap<(&str, &str), &CapabilityResource> = target
        .iter()
        .map(|r| ((r.r#type.as_str(), r.name.as_str()), r))
        .collect();

    for ((t, n), r) in &source_map {
        match target_map.get(&(t, n)) {
            Some(target_r) => {
                if r.content_hash != target_r.content_hash {
                    modified.push(DiffItem {
                        name: n.to_string(),
                        resource_type: t.to_string(),
                        source_resource: (*r).clone(),
                        target_resource: (*target_r).clone(),
                    });
                } else {
                    same.push((*r).clone());
                }
            }
            None => {
                missing.push((*r).clone());
            }
        }
    }

    for ((t, n), r) in &target_map {
        if !source_map.contains_key(&(t, n)) {
            extra.push((*r).clone());
        }
    }

    CompareResult {
        missing,
        extra,
        modified,
        same,
    }
}
