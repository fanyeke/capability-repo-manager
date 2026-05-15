use domain::{
    CapabilityResource, MigrationPlan, MigrationReport, MigrationReportItem,
    MigrationReportSummary, SnapshotItem,
};
use std::fs;
use std::path::Path;
use uuid::Uuid;

/// Create a scoped snapshot of only the files that will be affected by migration.
///
/// Backs up each file at its target path, writing a copy to `snapshot_dir/<uuid>-backup`.
/// Returns a list of `SnapshotItem` describing what was backed up.
pub fn create_snapshot(
    items: &[MigrationPlanItem],
    target_dir: &Path,
    snapshot_dir: &Path,
) -> Result<Vec<SnapshotItem>, domain::AppError> {
    fs::create_dir_all(snapshot_dir)
        .map_err(|e| domain::AppError::Migration(format!("Failed to create snapshot dir: {}", e)))?;

    let mut snapshot_items = Vec::new();

    for item in items {
        let target_path = item.target_path.as_deref().unwrap_or("");
        if target_path.is_empty() {
            continue;
        }

        let full_path = target_dir.join(target_path);
        let existed_before = full_path.exists();

        if existed_before {
            let backup_name = format!("{}-{}", Uuid::new_v4(), target_path.replace('/', "_"));
            let backup_path = snapshot_dir.join(&backup_name);

            if let Some(parent) = backup_path.parent() {
                let _ = fs::create_dir_all(parent);
            }

            if full_path.is_dir() {
                copy_dir_recursive(&full_path, &backup_path)
                    .map_err(|e| domain::AppError::Migration(format!("Failed to backup dir: {}", e)))?;
            } else {
                if let Some(parent) = backup_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| domain::AppError::Migration(format!("Failed to create parent: {}", e)))?;
                }
                fs::copy(&full_path, &backup_path)
                    .map_err(|e| domain::AppError::Migration(format!("Failed to backup file: {}", e)))?;
            }

            snapshot_items.push(SnapshotItem {
                target_path: target_path.to_string(),
                existed_before: true,
                backup_path: backup_path.to_string_lossy().to_string(),
            });
        } else {
            snapshot_items.push(SnapshotItem {
                target_path: target_path.to_string(),
                existed_before: false,
                backup_path: String::new(),
            });
        }
    }

    // Write manifest
    let manifest = serde_json::to_string(&snapshot_items)
        .map_err(|e| domain::AppError::Migration(format!("Failed to serialize manifest: {}", e)))?;

    fs::write(snapshot_dir.join("snapshot.json"), &manifest)
        .map_err(|e| domain::AppError::Migration(format!("Failed to write manifest: {}", e)))?;

    Ok(snapshot_items)
}

pub fn execute_plan(
    plan: &MigrationPlan,
    pack_resources: &[CapabilityResource],
    pack_dir: &Path,
    target_dir: &Path,
) -> Result<MigrationReport, domain::AppError> {
    let mut report_items = Vec::new();
    let mut summary = MigrationReportSummary {
        added: 0,
        overwritten: 0,
        skipped: 0,
        failed: 0,
    };

    for item in &plan.items {
        let pack_res = pack_resources.iter().find(|r| r.id == item.resource_id);

        let result = match item.action.as_str() {
            "add" | "overwrite" | "rename" => {
                let src_path = item.source_path.as_deref().unwrap_or("");
                let dst_path = item.target_path.as_deref().unwrap_or(src_path);

                let full_src = pack_dir.join(src_path);
                let full_dst = target_dir.join(dst_path);

                copy_resource(&full_src, &full_dst)
            }
            "skip" => Ok(ExecResult::Skipped),
            _ => Ok(ExecResult::Skipped),
        };

        match result {
            Ok(ExecResult::Copied) => match item.action.as_str() {
                "add" | "rename" => {
                    summary.added += 1;
                    report_items.push(MigrationReportItem {
                        resource_name: pack_res.map(|r| r.name.clone()).unwrap_or_default(),
                        action: item.action.clone(),
                        status: "success".to_string(),
                        error: None,
                    });
                }
                "overwrite" => {
                    summary.overwritten += 1;
                    report_items.push(MigrationReportItem {
                        resource_name: pack_res.map(|r| r.name.clone()).unwrap_or_default(),
                        action: item.action.clone(),
                        status: "success".to_string(),
                        error: None,
                    });
                }
                _ => {
                    summary.added += 1;
                    report_items.push(MigrationReportItem {
                        resource_name: pack_res.map(|r| r.name.clone()).unwrap_or_default(),
                        action: item.action.clone(),
                        status: "success".to_string(),
                        error: None,
                    });
                }
            },
            Ok(ExecResult::Skipped) => {
                summary.skipped += 1;
                report_items.push(MigrationReportItem {
                    resource_name: pack_res.map(|r| r.name.clone()).unwrap_or_default(),
                    action: item.action.clone(),
                    status: "skipped".to_string(),
                    error: None,
                });
            }
            Err(e) => {
                summary.failed += 1;
                report_items.push(MigrationReportItem {
                    resource_name: pack_res.map(|r| r.name.clone()).unwrap_or_default(),
                    action: item.action.clone(),
                    status: "failed".to_string(),
                    error: Some(e),
                });
            }
        }
    }

    let status = if summary.failed == 0 {
        "success"
    } else if summary.added + summary.overwritten > 0 {
        "partial_failure"
    } else {
        "failed"
    };

    Ok(MigrationReport {
        status: status.to_string(),
        items: report_items,
        summary,
    })
}

enum ExecResult {
    Copied,
    Skipped,
}

fn copy_resource(src: &Path, dst: &Path) -> Result<ExecResult, String> {
    if !src.exists() {
        return Err(format!("Source not found: {}", src.display()));
    }

    if src.is_dir() {
        copy_dir_recursive(src, dst)
            .map_err(|e| format!("Failed to copy directory {}: {}", src.display(), e))?;
    } else {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create parent dir: {}", e))?;
        }
        fs::copy(src, dst)
            .map_err(|e| format!("Failed to copy file {}: {}", src.display(), e))?;
    }

    Ok(ExecResult::Copied)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exec_result_values() {
        let _ = ExecResult::Copied;
        let _ = ExecResult::Skipped;
    }

    #[test]
    fn create_snapshot_backups_affected_files() {
        let tmp = tempfile::TempDir::new().unwrap();
        let target = tmp.path().join("target");
        let snap_dir = tmp.path().join("snapshots").join("run-1");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("file-a.txt"), "content a").unwrap();

        let items = vec![
            domain::MigrationPlanItem {
                resource_id: "r1".to_string(),
                action: "overwrite".to_string(),
                source_path: Some("file-a.txt".to_string()),
                target_path: Some("file-a.txt".to_string()),
                status: "pending".to_string(),
            },
            domain::MigrationPlanItem {
                resource_id: "r2".to_string(),
                action: "add".to_string(),
                source_path: Some("file-b.txt".to_string()),
                target_path: Some("file-b.txt".to_string()),
                status: "pending".to_string(),
            },
        ];

        let snapshot_items = create_snapshot(&items, &target, &snap_dir).unwrap();
        assert_eq!(snapshot_items.len(), 2);

        // file-a existed, should have backup
        let file_a = snapshot_items.iter().find(|s| s.target_path == "file-a.txt").unwrap();
        assert!(file_a.existed_before);
        assert!(!file_a.backup_path.is_empty());
        assert!(Path::new(&file_a.backup_path).exists());

        // file-b didn't exist
        let file_b = snapshot_items.iter().find(|s| s.target_path == "file-b.txt").unwrap();
        assert!(!file_b.existed_before);
        assert!(file_b.backup_path.is_empty());

        // manifest should exist
        assert!(snap_dir.join("snapshot.json").exists());
    }

    #[test]
    fn create_snapshot_empty_items() {
        let tmp = tempfile::TempDir::new().unwrap();
        let snap_dir = tmp.path().join("snapshots").join("empty");

        let items = vec![];
        let snapshot_items = create_snapshot(&items, tmp.path(), &snap_dir).unwrap();
        assert!(snapshot_items.is_empty());
        assert!(snap_dir.join("snapshot.json").exists());
    }
}
