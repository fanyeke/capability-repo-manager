use std::path::PathBuf;

use tauri::State;

use crate::state::AppState;
use domain;
use migration_engine;
use pack_engine;
use storage::{migration_store::MigrationStore, repo_store::RepositoryStore, resource_store::ResourceStore};

#[derive(serde::Deserialize)]
pub struct ConflictStrategy {
    pub resource_id: String,
    pub action: String,
}

#[derive(serde::Serialize)]
pub struct RollbackResult {
    pub success: bool,
    pub restored: u32,
    pub message: String,
}

#[tauri::command]
pub fn build_migration_plan(
    pack_id: String,
    target_repo_id: String,
    state: State<AppState>,
) -> Result<domain::MigrationPlan, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let settings = state.settings.lock().map_err(|e| e.to_string())?;

    let repo_store = RepositoryStore::new(&db);
    let resource_store = ResourceStore::new(&db);

    let library = pack_engine::library::PackStore::new(PathBuf::from(&settings.pack_storage_dir));
    let pack = library
        .get_by_id(&pack_id)
        .ok_or_else(|| format!("Pack not found: {}", pack_id))?;

    let _target_repo = repo_store
        .get_by_id(&target_repo_id)
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| "Target repository not found".to_string())?;

    // Check circular reference
    migration_engine::check_circular_reference(&pack_id, &target_repo_id, pack.source_repo_id.as_deref())
        .map_err(|e| format!("{}", e))?;

    let pack_resources = resource_store
        .get_by_pack(&pack_id)
        .map_err(|e| format!("Database error: {}", e))?;

    let target_resources = resource_store
        .get_by_repo(&target_repo_id)
        .map_err(|e| format!("Database error: {}", e))?;

    let plan = migration_engine::planner::build_plan(
        &pack_resources,
        &target_resources,
        &pack_id,
        &target_repo_id,
    );

    // Store the migration run in DB via migration_store
    let plan_json = serde_json::to_string(&plan).map_err(|e| format!("Serialization error: {}", e))?;

    let run = domain::MigrationRun {
        id: plan.plan_id.clone(),
        source_type: "pack".to_string(),
        source_id: pack_id,
        target_repo_id,
        status: "planned".to_string(),
        plan_json,
        report_json: None,
        snapshot_path: None,
        created_at: chrono::Utc::now().to_rfc3339(),
        executed_at: None,
    };

    let store = MigrationStore::new(&db);
    store.insert_run(&run).map_err(|e| format!("Database error: {}", e))?;

    Ok(plan)
}

#[tauri::command]
pub fn apply_migration_plan(
    plan_id: String,
    strategies: Vec<ConflictStrategy>,
    state: State<AppState>,
) -> Result<domain::MigrationReport, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let settings = state.settings.lock().map_err(|e| e.to_string())?;

    let store = MigrationStore::new(&db);

    // Fetch the stored migration run
    let run = store
        .get_run(&plan_id)
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| format!("Migration plan not found: {}", plan_id))?;

    // T038: Prevent re-apply — only "planned" can be executed
    if !migration_engine::can_execute(&run.status) {
        return Err(format!(
            "Migration plan {} has status '{}', only 'planned' can be executed",
            plan_id, run.status
        ));
    }

    // T036 step 1: Validate all ConflictStrategy actions against ConflictAction enum
    let strategy_pairs: Vec<(String, String)> = strategies
        .iter()
        .map(|s| (s.resource_id.clone(), s.action.clone()))
        .collect();
    migration_engine::validate_strategies(&strategy_pairs)
        .map_err(|e| format!("{}", e))?;

    let mut plan: domain::MigrationPlan =
        serde_json::from_str(&run.plan_json).map_err(|e| format!("Plan deserialization error: {}", e))?;

    // Apply user conflict strategies FIRST, then validate all items
    for strategy in &strategies {
        for item in &mut plan.items {
            if item.resource_id == strategy.resource_id {
                item.action = strategy.action.clone();
            }
        }
    }

    // Verify all conflicts resolved — check no item has action outside the enum
    for item in &plan.items {
        if domain::ConflictAction::from_str(&item.action).is_none() && item.action != "add" {
            return Err(format!(
                "Unresolved conflict for resource '{}': action '{}' is not valid. Resolve all conflicts before applying.",
                item.resource_id, item.action
            ));
        }
    }

    // State machine: planned → ready
    store.update_status(&plan_id, "ready").map_err(|e| format!("Database error: {}", e))?;

    // Get pack and target resources
    let library = pack_engine::library::PackStore::new(PathBuf::from(&settings.pack_storage_dir));
    let pack = library
        .get_by_id(&run.source_id)
        .ok_or_else(|| format!("Pack not found: {}", run.source_id))?;

    let resource_store = ResourceStore::new(&db);
    let repo_store = RepositoryStore::new(&db);

    let pack_resources = resource_store
        .get_by_pack(&run.source_id)
        .unwrap_or_default();

    let target_repo = repo_store
        .get_by_id(&run.target_repo_id)
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| "Target repository not found".to_string())?;

    let pack_dir = PathBuf::from(&pack.storage_dir);
    let target_dir = PathBuf::from(&target_repo.path);

    // State machine: ready → executing
    store.update_status(&plan_id, "executing").map_err(|e| format!("Database error: {}", e))?;

    // Create scoped snapshot before file writes
    let snapshot_base = dirs_or_default_snapshots(&target_repo.path);
    let snapshot_dir = snapshot_base.join(&plan_id);
    let snapshot_path = snapshot_dir.to_string_lossy().to_string();

    migration_engine::create_scoped_snapshot(&plan.items, &target_dir, &snapshot_dir)
        .map_err(|e| format!("Snapshot creation failed: {}", e))?;

    // Store snapshot path
    store.update_snapshot(&plan_id, &snapshot_path).map_err(|e| format!("Database error: {}", e))?;

    // Execute
    let report = migration_engine::executor::execute_plan(
        &plan,
        &pack_resources,
        &pack_dir,
        &target_dir,
    )
    .map_err(|e| format!("Migration execution failed: {}", e))?;

    // Determine final status from aggregate results
    let final_status = match report.status.as_str() {
        "success" => "success",
        "partial_failure" => "partial_failure",
        "failed" => "failed",
        _ => "failed",
    };

    // Update the migration run
    let report_json = serde_json::to_string(&report).map_err(|e| format!("Serialization error: {}", e))?;
    let now = chrono::Utc::now().to_rfc3339();

    store.update_status(&plan_id, final_status).map_err(|e| format!("Database error: {}", e))?;

    db.conn().execute(
        "UPDATE migration_runs SET report_json = ?1, executed_at = ?2 WHERE id = ?3",
        rusqlite::params![report_json, now, plan_id],
    ).map_err(|e| format!("Database error: {}", e))?;

    Ok(report)
}

#[tauri::command]
pub fn rollback_migration(
    run_id: String,
    state: State<AppState>,
) -> Result<RollbackResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let store = MigrationStore::new(&db);

    let run = store
        .get_run(&run_id)
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| format!("Migration run not found: {}", run_id))?;

    // T037: Only accept success or partial_failure for rollback
    if !migration_engine::can_rollback(&run.status) {
        return Err(format!(
            "Cannot rollback: migration is in '{}' state. Only 'success' or 'partial_failure' can be rolled back.",
            run.status
        ));
    }

    let snapshot_path = run
        .snapshot_path
        .ok_or_else(|| "No snapshot available for rollback".to_string())?;

    let repo_store = RepositoryStore::new(&db);
    let target_repo = repo_store
        .get_by_id(&run.target_repo_id)
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| "Target repository not found".to_string())?;

    let target_dir = PathBuf::from(&target_repo.path);
    let snapshot_dir = PathBuf::from(&snapshot_path);

    // Use scoped restore
    migration_engine::restore_scoped(&snapshot_dir, &target_dir)
        .map_err(|e| format!("Rollback failed: {}", e))?;

    // Update status to rolled_back
    store.update_status(&run_id, "rolled_back").map_err(|e| format!("Database error: {}", e))?;

    Ok(RollbackResult {
        success: true,
        restored: 0,
        message: "Migration rolled back successfully".to_string(),
    })
}

#[tauri::command]
pub fn get_migration_history(
    repo_id: String,
    state: State<AppState>,
) -> Result<Vec<domain::MigrationRun>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let store = MigrationStore::new(&db);

    store
        .list_by_repo(&repo_id)
        .map_err(|e| format!("Database error: {}", e))
}

fn dirs_or_default_snapshots(repo_path: &str) -> PathBuf {
    let base = std::env::var("HOME")
        .map(|h| PathBuf::from(h).join(".capability-repo-manager").join("snapshots"))
        .unwrap_or_else(|_| PathBuf::from("/tmp").join("capability-repo-manager-snapshots"));
    base
}
