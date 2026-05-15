use std::path::PathBuf;

use tauri::State;

use crate::state::AppState;
use domain;
use domain::OperationContext;
use migration_engine;
use pack_engine;
use storage::event_store::NewOperationEvent;
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
    let ctx = OperationContext::new("build_migration_plan")
        .with_pack_id(pack_id.clone())
        .with_repo_id(target_repo_id.clone());

    tracing::info!(
        operation_id = %ctx.operation_id,
        pack_id = %pack_id,
        target_repo_id = %target_repo_id,
        "migration_plan_started"
    );

    let db = state.db.lock().map_err(|e| {
        let msg = e.to_string();
        tracing::error!(operation_id = %ctx.operation_id, error = %msg, "migration_plan_failed");
        msg
    })?;
    let settings = state.settings.lock().map_err(|e| {
        let msg = e.to_string();
        tracing::error!(operation_id = %ctx.operation_id, error = %msg, "migration_plan_failed");
        msg
    })?;

    let repo_store = RepositoryStore::new(&db);
    let resource_store = ResourceStore::new(&db);

    let library = pack_engine::library::PackStore::new(PathBuf::from(&settings.pack_storage_dir));
    let pack = library
        .get_by_id(&pack_id)
        .ok_or_else(|| {
            let msg = format!("Pack not found: {}", pack_id);
            tracing::error!(operation_id = %ctx.operation_id, error = %msg, "migration_plan_failed");
            msg
        })?;

    let _target_repo = repo_store
        .get_by_id(&target_repo_id)
        .map_err(|e| {
            let msg = format!("Database error: {}", e);
            tracing::error!(operation_id = %ctx.operation_id, error = %msg, "migration_plan_failed");
            msg
        })?
        .ok_or_else(|| {
            let msg = "Target repository not found".to_string();
            tracing::error!(operation_id = %ctx.operation_id, error = %msg, "migration_plan_failed");
            msg
        })?;

    // Check circular reference
    migration_engine::check_circular_reference(&pack_id, &target_repo_id, pack.source_repo_id.as_deref())
        .map_err(|e| {
            let msg = format!("{}", e);
            tracing::error!(operation_id = %ctx.operation_id, error = %msg, "migration_plan_failed");
            msg
        })?;

    let pack_resources = resource_store
        .get_by_pack(&pack_id)
        .map_err(|e| {
            let msg = format!("Database error: {}", e);
            tracing::error!(operation_id = %ctx.operation_id, error = %msg, "migration_plan_failed");
            msg
        })?;

    let target_resources = resource_store
        .get_by_repo(&target_repo_id)
        .map_err(|e| {
            let msg = format!("Database error: {}", e);
            tracing::error!(operation_id = %ctx.operation_id, error = %msg, "migration_plan_failed");
            msg
        })?;

    let plan = migration_engine::planner::build_plan(
        &pack_resources,
        &target_resources,
        &pack_id,
        &target_repo_id,
    );

    // Store the migration run in DB via migration_store
    let plan_json = serde_json::to_string(&plan).map_err(|e| {
        let msg = format!("Serialization error: {}", e);
        tracing::error!(operation_id = %ctx.operation_id, error = %msg, "migration_plan_failed");
        msg
    })?;

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
    store.insert_run(&run).map_err(|e| {
        let msg = format!("Database error: {}", e);
        tracing::error!(operation_id = %ctx.operation_id, error = %msg, "migration_plan_failed");
        msg
    })?;

    tracing::info!(
        operation_id = %ctx.operation_id,
        plan_id = %plan.plan_id,
        items_count = %plan.items.len(),
        "migration_plan_finished"
    );

    // Record OperationEvent
    let event_store = db.event_store();
    let _ = event_store.insert_event(NewOperationEvent {
        operation_id: ctx.operation_id,
        operation_type: "build_migration_plan".to_string(),
        status: "success".to_string(),
        repo_id: ctx.repo_id,
        pack_id: ctx.pack_id,
        migration_run_id: Some(plan.plan_id.clone()),
        summary: Some(format!("Built migration plan with {} items", plan.items.len())),
        detail_json: None,
    });

    Ok(plan)
}

#[tauri::command]
pub fn apply_migration_plan(
    plan_id: String,
    strategies: Vec<ConflictStrategy>,
    state: State<AppState>,
) -> Result<domain::MigrationReport, String> {
    let ctx = OperationContext::new("apply_migration")
        .with_migration_run_id(plan_id.clone());

    tracing::info!(
        operation_id = %ctx.operation_id,
        plan_id = %plan_id,
        "migration_apply_started"
    );

    let db = state.db.lock().map_err(|e| {
        let msg = e.to_string();
        tracing::error!(operation_id = %ctx.operation_id, error = %msg, "migration_apply_failed");
        msg
    })?;
    let settings = state.settings.lock().map_err(|e| {
        let msg = e.to_string();
        tracing::error!(operation_id = %ctx.operation_id, error = %msg, "migration_apply_failed");
        msg
    })?;

    let store = MigrationStore::new(&db);

    // Fetch the stored migration run
    let run = store
        .get_run(&plan_id)
        .map_err(|e| {
            let msg = format!("Database error: {}", e);
            tracing::error!(operation_id = %ctx.operation_id, error = %msg, "migration_apply_failed");
            msg
        })?
        .ok_or_else(|| {
            let msg = format!("Migration plan not found: {}", plan_id);
            tracing::error!(operation_id = %ctx.operation_id, error = %msg, "migration_apply_failed");
            msg
        })?;

    // T038: Prevent re-apply — only "planned" can be executed
    if !migration_engine::can_execute(&run.status) {
        let msg = format!(
            "Migration plan {} has status '{}', only 'planned' can be executed",
            plan_id, run.status
        );
        tracing::warn!(
            operation_id = %ctx.operation_id,
            plan_id = %plan_id,
            status = %run.status,
            "migration_apply_blocked"
        );
        return Err(msg);
    }

    // T036 step 1: Validate all ConflictStrategy actions against ConflictAction enum
    let strategy_pairs: Vec<(String, String)> = strategies
        .iter()
        .map(|s| (s.resource_id.clone(), s.action.clone()))
        .collect();
    if let Err(e) = migration_engine::validate_strategies(&strategy_pairs) {
        let msg = format!("{}", e);
        tracing::error!(operation_id = %ctx.operation_id, error = %msg, "migration_apply_failed");
        return Err(msg);
    }

    let mut plan: domain::MigrationPlan =
        serde_json::from_str(&run.plan_json).map_err(|e| {
            let msg = format!("Plan deserialization error: {}", e);
            tracing::error!(operation_id = %ctx.operation_id, error = %msg, "migration_apply_failed");
            msg
        })?;

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
        if item.action.parse::<domain::ConflictAction>().is_err() && item.action != "add" {
            let msg = format!(
                "Unresolved conflict for resource '{}': action '{}' is not valid. Resolve all conflicts before applying.",
                item.resource_id, item.action
            );
            tracing::error!(operation_id = %ctx.operation_id, error = %msg, resource_id = %item.resource_id, "migration_apply_failed");
            return Err(msg);
        }
    }

    // State machine: planned → ready
    store.update_status(&plan_id, "ready").map_err(|e| {
        let msg = format!("Database error: {}", e);
        tracing::error!(operation_id = %ctx.operation_id, error = %msg, "migration_apply_failed");
        msg
    })?;

    // Get pack and target resources
    let library = pack_engine::library::PackStore::new(PathBuf::from(&settings.pack_storage_dir));
    let pack = library
        .get_by_id(&run.source_id)
        .ok_or_else(|| {
            let msg = format!("Pack not found: {}", run.source_id);
            tracing::error!(operation_id = %ctx.operation_id, error = %msg, "migration_apply_failed");
            msg
        })?;

    let resource_store = ResourceStore::new(&db);
    let repo_store = RepositoryStore::new(&db);

    let pack_resources = resource_store
        .get_by_pack(&run.source_id)
        .unwrap_or_default();

    let target_repo = repo_store
        .get_by_id(&run.target_repo_id)
        .map_err(|e| {
            let msg = format!("Database error: {}", e);
            tracing::error!(operation_id = %ctx.operation_id, error = %msg, "migration_apply_failed");
            msg
        })?
        .ok_or_else(|| {
            let msg = "Target repository not found".to_string();
            tracing::error!(operation_id = %ctx.operation_id, error = %msg, "migration_apply_failed");
            msg
        })?;

    let pack_dir = PathBuf::from(&pack.storage_dir);
    let target_dir = PathBuf::from(&target_repo.path);

    // State machine: ready → executing
    store.update_status(&plan_id, "executing").map_err(|e| {
        let msg = format!("Database error: {}", e);
        tracing::error!(operation_id = %ctx.operation_id, error = %msg, "migration_apply_failed");
        msg
    })?;

    // Create scoped snapshot before file writes
    let snapshot_base = dirs_or_default_snapshots(&target_repo.path);
    let snapshot_dir = snapshot_base.join(&plan_id);
    let snapshot_path = snapshot_dir.to_string_lossy().to_string();

    migration_engine::create_scoped_snapshot(&plan.items, &target_dir, &snapshot_dir)
        .map_err(|e| {
            let msg = format!("Snapshot creation failed: {}", e);
            tracing::error!(operation_id = %ctx.operation_id, error = %msg, "migration_apply_failed");
            msg
        })?;

    tracing::info!(
        operation_id = %ctx.operation_id,
        snapshot_dir = %snapshot_path,
        "migration_snapshot_created"
    );

    // Store snapshot path
    store.update_snapshot(&plan_id, &snapshot_path).map_err(|e| {
        let msg = format!("Database error: {}", e);
        tracing::error!(operation_id = %ctx.operation_id, error = %msg, "migration_apply_failed");
        msg
    })?;

    // Execute
    tracing::debug!(
        operation_id = %ctx.operation_id,
        items_count = %plan.items.len(),
        "migration_copy_started"
    );

    let report = match migration_engine::executor::execute_plan(
        &plan,
        &pack_resources,
        &pack_dir,
        &target_dir,
    ) {
        Ok(r) => r,
        Err(e) => {
            let msg = format!("Migration execution failed: {}", e);
            tracing::error!(
                operation_id = %ctx.operation_id,
                error = %msg,
                "migration_copy_failed"
            );
            return Err(msg);
        }
    };

    // Determine final status from aggregate results
    let final_status = match report.status.as_str() {
        "success" => "success",
        "partial_failure" => "partial_failure",
        "failed" => "failed",
        _ => "failed",
    };

    // Update the migration run
    let report_json = serde_json::to_string(&report).map_err(|e| {
        let msg = format!("Serialization error: {}", e);
        tracing::error!(operation_id = %ctx.operation_id, error = %msg, "migration_apply_failed");
        msg
    })?;
    let now = chrono::Utc::now().to_rfc3339();

    store.update_status(&plan_id, final_status).map_err(|e| {
        let msg = format!("Database error: {}", e);
        tracing::error!(operation_id = %ctx.operation_id, error = %msg, "migration_apply_failed");
        msg
    })?;

    db.conn().execute(
        "UPDATE migration_runs SET report_json = ?1, executed_at = ?2 WHERE id = ?3",
        rusqlite::params![report_json, now, plan_id],
    ).map_err(|e| {
        let msg = format!("Database error: {}", e);
        tracing::error!(operation_id = %ctx.operation_id, error = %msg, "migration_apply_failed");
        msg
    })?;

    tracing::info!(
        operation_id = %ctx.operation_id,
        status = %final_status,
        items_count = %report.items.len(),
        "migration_apply_finished"
    );

    // Record OperationEvent
    let event_store = db.event_store();
    let _ = event_store.insert_event(NewOperationEvent {
        operation_id: ctx.operation_id,
        operation_type: "apply_migration".to_string(),
        status: final_status.to_string(),
        repo_id: Some(run.target_repo_id),
        pack_id: Some(run.source_id),
        migration_run_id: Some(plan_id),
        summary: Some(format!("Migration {} with {} items", final_status, report.items.len())),
        detail_json: None,
    });

    Ok(report)
}

#[tauri::command]
pub fn rollback_migration(
    run_id: String,
    state: State<AppState>,
) -> Result<RollbackResult, String> {
    let ctx = OperationContext::new("rollback_migration")
        .with_migration_run_id(run_id.clone());

    tracing::info!(
        operation_id = %ctx.operation_id,
        run_id = %run_id,
        "rollback_started"
    );

    let db = state.db.lock().map_err(|e| {
        let msg = e.to_string();
        tracing::error!(operation_id = %ctx.operation_id, error = %msg, "rollback_failed");
        msg
    })?;

    let store = MigrationStore::new(&db);

    let run = store
        .get_run(&run_id)
        .map_err(|e| {
            let msg = format!("Database error: {}", e);
            tracing::error!(operation_id = %ctx.operation_id, error = %msg, "rollback_failed");
            msg
        })?
        .ok_or_else(|| {
            let msg = format!("Migration run not found: {}", run_id);
            tracing::error!(operation_id = %ctx.operation_id, error = %msg, "rollback_failed");
            msg
        })?;

    // T037: Only accept success or partial_failure for rollback
    if !migration_engine::can_rollback(&run.status) {
        let msg = format!(
            "Cannot rollback: migration is in '{}' state. Only 'success' or 'partial_failure' can be rolled back.",
            run.status
        );
        tracing::warn!(
            operation_id = %ctx.operation_id,
            run_id = %run_id,
            status = %run.status,
            "rollback_blocked"
        );
        return Err(msg);
    }

    let snapshot_path = run
        .snapshot_path
        .ok_or_else(|| {
            let msg = "No snapshot available for rollback".to_string();
            tracing::error!(operation_id = %ctx.operation_id, error = %msg, "rollback_failed");
            msg
        })?;

    let repo_store = RepositoryStore::new(&db);
    let target_repo = repo_store
        .get_by_id(&run.target_repo_id)
        .map_err(|e| {
            let msg = format!("Database error: {}", e);
            tracing::error!(operation_id = %ctx.operation_id, error = %msg, "rollback_failed");
            msg
        })?
        .ok_or_else(|| {
            let msg = "Target repository not found".to_string();
            tracing::error!(operation_id = %ctx.operation_id, error = %msg, "rollback_failed");
            msg
        })?;

    let target_dir = PathBuf::from(&target_repo.path);
    let snapshot_dir = PathBuf::from(&snapshot_path);

    // Use scoped restore
    tracing::debug!(
        operation_id = %ctx.operation_id,
        snapshot_dir = %snapshot_path,
        target_dir = %target_repo.path,
        "rollback_restored"
    );

    migration_engine::restore_scoped(&snapshot_dir, &target_dir)
        .map_err(|e| {
            let msg = format!("Rollback failed: {}", e);
            tracing::error!(operation_id = %ctx.operation_id, error = %msg, "rollback_failed");
            msg
        })?;

    // Update status to rolled_back
    store.update_status(&run_id, "rolled_back").map_err(|e| {
        let msg = format!("Database error: {}", e);
        tracing::error!(operation_id = %ctx.operation_id, error = %msg, "rollback_failed");
        msg
    })?;

    tracing::debug!(
        operation_id = %ctx.operation_id,
        run_id = %run_id,
        "rollback_removed"
    );

    tracing::info!(
        operation_id = %ctx.operation_id,
        run_id = %run_id,
        "rollback_finished"
    );

    // Record OperationEvent
    let event_store = db.event_store();
    let _ = event_store.insert_event(NewOperationEvent {
        operation_id: ctx.operation_id,
        operation_type: "rollback_migration".to_string(),
        status: "success".to_string(),
        repo_id: Some(run.target_repo_id),
        pack_id: None,
        migration_run_id: Some(run_id),
        summary: Some("Migration rolled back successfully".to_string()),
        detail_json: None,
    });

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

fn dirs_or_default_snapshots(_repo_path: &str) -> PathBuf {
    std::env::var("HOME")
        .map(|h| PathBuf::from(h).join(".capability-repo-manager").join("snapshots"))
        .unwrap_or_else(|_| PathBuf::from("/tmp").join("capability-repo-manager-snapshots"))
}
