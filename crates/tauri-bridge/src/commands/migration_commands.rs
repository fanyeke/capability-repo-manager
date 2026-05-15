use std::path::PathBuf;

use tauri::State;

use crate::state::AppState;
use domain;
use migration_engine;
use pack_engine;
use storage::{repo_store::RepositoryStore, resource_store::ResourceStore};

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
        .ok_or_else(|| format!("Target repository not found: {}", target_repo_id))?;

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

    // Store the migration run in DB
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

    db.conn().execute(
        "INSERT OR REPLACE INTO migration_runs (id, source_type, source_id, target_repo_id, status, plan_json, report_json, snapshot_path, created_at, executed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![
            run.id,
            run.source_type,
            run.source_id,
            run.target_repo_id,
            run.status,
            run.plan_json,
            run.report_json,
            run.snapshot_path,
            run.created_at,
            run.executed_at,
        ],
    ).map_err(|e| format!("Database error: {}", e))?;

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

    // Fetch the stored migration plan
    let mut stmt = db.conn().prepare(
        "SELECT id, source_type, source_id, target_repo_id, status, plan_json, snapshot_path
         FROM migration_runs WHERE id = ?1"
    ).map_err(|e| format!("Database error: {}", e))?;

    let mut rows = stmt.query_map(rusqlite::params![plan_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, Option<String>>(6)?,
        ))
    }).map_err(|e| format!("Database error: {}", e))?;

    let (_id, _source_type, source_id, target_repo_id, status, plan_json, _snapshot) =
        match rows.next() {
            Some(Ok(row)) => row,
            Some(Err(e)) => return Err(format!("Database error: {}", e)),
            None => return Err(format!("Migration plan not found: {}", plan_id)),
        };

    if status != "planned" {
        return Err(format!("Migration plan {} has already been applied or failed", plan_id));
    }

    let mut plan: domain::MigrationPlan =
        serde_json::from_str(&plan_json).map_err(|e| format!("Plan deserialization error: {}", e))?;

    // Apply user conflict strategies
    for strategy in &strategies {
        for item in &mut plan.items {
            if item.resource_id == strategy.resource_id {
                item.action = strategy.action.clone();
            }
        }
    }

    // Get the pack and target resources
    let library = pack_engine::library::PackStore::new(PathBuf::from(&settings.pack_storage_dir));
    let pack = library
        .get_by_id(&source_id)
        .ok_or_else(|| format!("Pack not found: {}", source_id))?;

    let resource_store = ResourceStore::new(&db);
    let repo_store = RepositoryStore::new(&db);

    let pack_resources = resource_store
        .get_by_pack(&source_id)
        .unwrap_or_default();

    let target_repo = repo_store
        .get_by_id(&target_repo_id)
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| format!("Target repository not found: {}", target_repo_id))?;

    let _target_resources = resource_store
        .get_by_repo(&target_repo_id)
        .unwrap_or_default();

    let pack_dir = PathBuf::from(&pack.storage_dir);
    let target_dir = PathBuf::from(&target_repo.path);

    // Execute
    let report = migration_engine::executor::execute_plan(
        &plan,
        &pack_resources,
        &pack_dir,
        &target_dir,
    )
    .map_err(|e| format!("Migration execution failed: {}", e))?;

    // Update the migration run
    let report_json = serde_json::to_string(&report).map_err(|e| format!("Serialization error: {}", e))?;
    let now = chrono::Utc::now().to_rfc3339();

    db.conn().execute(
        "UPDATE migration_runs SET status = ?1, report_json = ?2, executed_at = ?3 WHERE id = ?4",
        rusqlite::params!["applied", report_json, now, plan_id],
    ).map_err(|e| format!("Database error: {}", e))?;

    Ok(report)
}

#[tauri::command]
pub fn rollback_migration(
    run_id: String,
    state: State<AppState>,
) -> Result<RollbackResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let mut stmt = db.conn().prepare(
        "SELECT id, status, snapshot_path, target_repo_id FROM migration_runs WHERE id = ?1"
    ).map_err(|e| format!("Database error: {}", e))?;

    let mut rows = stmt.query_map(rusqlite::params![run_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, String>(3)?,
        ))
    }).map_err(|e| format!("Database error: {}", e))?;

    let (_id, status, snapshot_path, target_repo_id) = match rows.next() {
        Some(Ok(row)) => row,
        Some(Err(e)) => return Err(format!("Database error: {}", e)),
        None => return Err(format!("Migration run not found: {}", run_id)),
    };

    if status != "applied" {
        return Err(format!("Cannot rollback: migration is in '{}' state, not 'applied'", status));
    }

    let snapshot = snapshot_path.ok_or_else(|| "No snapshot available for rollback".to_string())?;

    let repo_store = RepositoryStore::new(&db);
    let target_repo = repo_store
        .get_by_id(&target_repo_id)
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| "Target repository not found".to_string())?;

    let target_dir = PathBuf::from(&target_repo.path);
    migration_engine::rollback_migration(&snapshot, &target_dir)
        .map_err(|e| format!("Rollback failed: {}", e))?;

    db.conn().execute(
        "UPDATE migration_runs SET status = 'rolled_back' WHERE id = ?1",
        rusqlite::params![run_id],
    ).map_err(|e| format!("Database error: {}", e))?;

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

    let mut stmt = db.conn().prepare(
        "SELECT id, source_type, source_id, target_repo_id, status, plan_json, report_json, snapshot_path, created_at, executed_at
         FROM migration_runs WHERE target_repo_id = ?1 ORDER BY created_at DESC"
    ).map_err(|e| format!("Database error: {}", e))?;

    let rows = stmt.query_map(rusqlite::params![repo_id], |row| {
        Ok(domain::MigrationRun {
            id: row.get(0)?,
            source_type: row.get(1)?,
            source_id: row.get(2)?,
            target_repo_id: row.get(3)?,
            status: row.get(4)?,
            plan_json: row.get(5)?,
            report_json: row.get(6)?,
            snapshot_path: row.get(7)?,
            created_at: row.get(8)?,
            executed_at: row.get(9)?,
        })
    }).map_err(|e| format!("Database error: {}", e))?;

    let mut runs = Vec::new();
    for row in rows {
        runs.push(row.map_err(|e| format!("Database error: {}", e))?);
    }

    Ok(runs)
}
