pub mod conflict;
pub mod executor;
pub mod planner;
pub mod rollback;

use std::path::Path;

use domain::{CapabilityResource, MigrationPlan, MigrationReport};

/// Resolve `target_dir.join(path)` and verify the result stays within `target_dir`.
/// Returns an error if the path tries to escape via `..` or symlinks.
pub fn resolve_safe_path(target_dir: &Path, sub_path: &str) -> Result<std::path::PathBuf, domain::AppError> {
    let target_canonical = target_dir
        .canonicalize()
        .map_err(|e| domain::AppError::Migration(format!("Cannot resolve target directory: {}", e)))?;
    let joined = target_dir.join(sub_path);
    let joined_canonical = joined.canonicalize().unwrap_or_else(|_| joined.clone());
    if !joined_canonical.starts_with(&target_canonical) {
        return Err(domain::AppError::Migration(format!(
            "Path traversal detected: '{}' resolves outside target directory",
            sub_path
        )));
    }
    Ok(joined)
}

/// Minimum required disk space for migration operations (50 MB).
const MIN_DISK_SPACE: u64 = 50 * 1024 * 1024;

/// Check that the filesystem containing `path` has at least `min_bytes` available.
fn check_disk_space(path: &Path, min_bytes: u64) -> Result<(), domain::AppError> {
    match std::process::Command::new("df").arg("-k").arg(path).output() {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let last_line = stdout.lines().last().unwrap_or("");
            let fields: Vec<&str> = last_line.split_whitespace().collect();
            if fields.len() >= 4 {
                if let Ok(avail_kb) = fields[3].parse::<u64>() {
                    let avail_bytes = avail_kb * 1024;
                    if avail_bytes < min_bytes {
                        let mb = min_bytes / (1024 * 1024);
                        return Err(domain::AppError::Storage(format!(
                            "Insufficient disk space: {} MB required but only {} MB available on {}",
                            mb,
                            avail_bytes / (1024 * 1024),
                            path.display()
                        )));
                    }
                }
            }
            Ok(())
        }
        _ => Ok(()), // Can't check; proceed optimistically
    }
}

/// Build a migration plan, then execute it with the given conflict strategies.
/// This is the main orchestrator for the migration engine.
pub fn plan_and_execute(
    pack_resources: &[CapabilityResource],
    target_resources: &[CapabilityResource],
    source_id: &str,
    target_repo_id: &str,
    pack_dir: &std::path::Path,
    target_dir: &std::path::Path,
) -> Result<(MigrationPlan, MigrationReport), domain::AppError> {
    // Circular reference check: cannot migrate a pack to its own source repo
    if source_id == target_repo_id {
        return Err(domain::AppError::Migration(
            "Cannot migrate a pack to its own source repository (circular reference)".to_string(),
        ));
    }

    // Step 0: Check available disk space
    check_disk_space(target_dir, MIN_DISK_SPACE)?;

    // Step 1: Build migration plan
    let plan = planner::build_plan(pack_resources, target_resources, source_id, target_repo_id);

    // Step 2: Detect conflicts
    let _conflicts = conflict::detect_conflicts(pack_resources, target_resources);

    // Step 3: Create snapshot for rollback safety
    let snapshot_path = rollback::create_snapshot(target_dir)?;

    // Step 4: Execute the plan
    let report = executor::execute_plan(&plan, pack_resources, pack_dir, target_dir)?;

    // Step 5: If execution failed completely, rollback automatically
    if report.status == "failed" {
        let _ = rollback::rollback_to_snapshot(&snapshot_path, target_dir);
    } else {
        // Keep snapshot for user-requested rollback; store path in plan context
        // (cleanup of old snapshots is handled by storage layer)
    }

    Ok((plan, report))
}

/// Roll back a previously executed migration using the snapshot.
pub fn rollback_migration(snapshot_path: &str, target_dir: &std::path::Path) -> Result<(), domain::AppError> {
    rollback::rollback_to_snapshot(snapshot_path, target_dir)
}

/// Restore from a scoped snapshot (only affected files).
pub fn restore_scoped(snapshot_dir: &std::path::Path, target_dir: &std::path::Path) -> Result<(), domain::AppError> {
    rollback::restore_from_scoped_snapshot(snapshot_dir, target_dir)
}

/// Create a scoped snapshot of only the affected files and write the manifest.
pub fn create_scoped_snapshot(
    items: &[domain::MigrationPlanItem],
    target_dir: &std::path::Path,
    snapshot_dir: &std::path::Path,
) -> Result<Vec<domain::SnapshotItem>, domain::AppError> {
    executor::create_snapshot(items, target_dir, snapshot_dir)
}

/// Validate conflict strategies against the ConflictAction enum.
/// Returns an error if any action is not a valid ConflictAction value.
pub fn validate_strategies(strategies: &[(String, String)]) -> Result<(), domain::AppError> {
    for (resource_id, action) in strategies {
        if action.parse::<domain::ConflictAction>().is_err() {
            return Err(domain::AppError::Migration(format!(
                "Invalid conflict action '{}' for resource '{}'. Valid values: skip, overwrite",
                action, resource_id
            )));
        }
    }
    Ok(())
}

/// Check whether a migration run status allows execution.
/// Only `planned` status can be executed.
pub fn can_execute(status: &str) -> bool {
    status == "planned"
}

/// Check whether a migration run status allows rollback.
/// Only `success` or `partial_failure` can be rolled back.
pub fn can_rollback(status: &str) -> bool {
    status == "success" || status == "partial_failure"
}

/// Check for circular reference between source and target.
pub fn check_circular_reference(
    source_id: &str,
    target_repo_id: &str,
    pack_source_repo_id: Option<&str>,
) -> Result<(), domain::AppError> {
    if source_id == target_repo_id {
        return Err(domain::AppError::Migration(
            "Circular reference: cannot migrate a pack to its own source repository".to_string(),
        ));
    }

    if let Some(src_repo) = pack_source_repo_id {
        if src_repo == target_repo_id {
            return Err(domain::AppError::Migration(
                "Circular reference: pack was exported from the target repository".to_string(),
            ));
        }
    }

    Ok(())
}
