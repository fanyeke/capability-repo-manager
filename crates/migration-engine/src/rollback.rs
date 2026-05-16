use domain::SnapshotItem;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub fn create_snapshot(target_dir: &Path) -> Result<String, domain::AppError> {
    let snapshot_name = format!("snapshot-{}", Uuid::new_v4());
    let snapshot_dir = target_dir.parent().unwrap_or_else(|| Path::new("/tmp")).join(&snapshot_name);

    copy_dir_recursive(target_dir, &snapshot_dir)
        .map_err(|e| domain::AppError::Migration(format!("Failed to create snapshot: {}", e)))?;

    Ok(snapshot_dir.to_string_lossy().to_string())
}

/// Restore from a scoped snapshot manifest.
///
/// Reads `snapshot.json` from `snapshot_dir`, restores each backed-up file
/// to its original location, and removes files that didn't exist before migration.
pub fn restore_from_scoped_snapshot(snapshot_dir: &Path, target_dir: &Path) -> Result<(), domain::AppError> {
    let manifest_path = snapshot_dir.join("snapshot.json");
    if !manifest_path.exists() {
        return Err(domain::AppError::Migration("Snapshot manifest not found".to_string()));
    }

    let manifest_str = fs::read_to_string(&manifest_path)
        .map_err(|e| domain::AppError::Migration(format!("Failed to read manifest: {}", e)))?;

    let items: Vec<SnapshotItem> = serde_json::from_str(&manifest_str)
        .map_err(|e| domain::AppError::Migration(format!("Failed to parse manifest: {}", e)))?;

    for item in &items {
        let target_path = crate::resolve_safe_path(target_dir, &item.target_path)?;

        if item.existed_before {
            // Restore from backup
            let backup = PathBuf::from(&item.backup_path);
            if !backup.exists() {
                return Err(domain::AppError::Migration(format!("Backup file not found: {}", item.backup_path)));
            }

            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| domain::AppError::Migration(format!("Failed to create parent: {}", e)))?;
            }

            fs::copy(&backup, &target_path)
                .map_err(|e| domain::AppError::Migration(format!("Failed to restore {}: {}", item.target_path, e)))?;
        } else {
            // File didn't exist before — remove it
            let _ = fs::remove_file(&target_path);
            let _ = fs::remove_dir_all(&target_path);
        }
    }

    Ok(())
}

pub fn rollback_to_snapshot(snapshot_path: &str, target_dir: &Path) -> Result<(), domain::AppError> {
    let snapshot = PathBuf::from(snapshot_path);

    if !snapshot.exists() {
        return Err(domain::AppError::Migration(format!("Snapshot not found: {}", snapshot_path)));
    }

    // Remove current target contents (but not the directory itself)
    if target_dir.exists() {
        remove_dir_contents(target_dir)
            .map_err(|e| domain::AppError::Migration(format!("Failed to clean target directory: {}", e)))?;
    }

    // Restore from snapshot
    copy_dir_recursive(&snapshot, target_dir)
        .map_err(|e| domain::AppError::Migration(format!("Failed to restore from snapshot: {}", e)))?;

    // Clean up snapshot after successful rollback
    let _ = fs::remove_dir_all(&snapshot);

    Ok(())
}

pub fn verify_snapshot(snapshot_path: &str) -> Result<(), domain::AppError> {
    let snapshot = PathBuf::from(snapshot_path);

    if !snapshot.exists() {
        return Err(domain::AppError::Migration("Snapshot directory does not exist".to_string()));
    }

    if !snapshot.is_dir() {
        return Err(domain::AppError::Migration("Snapshot path is not a directory".to_string()));
    }

    // Check that snapshot has at least some content
    let entries: Vec<_> = fs::read_dir(&snapshot)
        .map_err(|e| domain::AppError::Migration(format!("Cannot read snapshot directory: {}", e)))?
        .collect();

    if entries.is_empty() {
        return Err(domain::AppError::Migration("Snapshot directory is empty".to_string()));
    }

    Ok(())
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

fn remove_dir_contents(dir: &Path) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_test_file(dir: &Path, name: &str, content: &str) {
        if let Some(parent) = dir.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(dir.join(name), content).unwrap();
    }

    #[test]
    fn snapshot_name_is_unique() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("target");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("test.txt"), "hello").unwrap();

        let s1 = create_snapshot(&target).unwrap();
        let s2 = create_snapshot(&target).unwrap();

        assert_ne!(s1, s2);
        let _ = fs::remove_dir_all(PathBuf::from(&s1));
        let _ = fs::remove_dir_all(PathBuf::from(&s2));
    }

    #[test]
    fn restore_from_scoped_snapshot_restores_existing_file() {
        let tmp = TempDir::new().unwrap();
        let snap = tmp.path().join("snap");
        let target = tmp.path().join("target");
        let backup_dir = tmp.path().join("backups");
        fs::create_dir_all(&target).unwrap();
        fs::create_dir_all(&backup_dir).unwrap();
        fs::write(target.join("file.txt"), "original").unwrap();

        // Create a backup copy
        let backup_path = backup_dir.join("file.txt.backup");
        fs::write(&backup_path, "original").unwrap();

        // Write manifest
        let items = vec![SnapshotItem {
            target_path: "file.txt".to_string(),
            existed_before: true,
            backup_path: backup_path.to_string_lossy().to_string(),
        }];
        fs::create_dir_all(&snap).unwrap();
        fs::write(snap.join("snapshot.json"), serde_json::to_string(&items).unwrap()).unwrap();

        // Modify the file
        fs::write(target.join("file.txt"), "modified").unwrap();

        // Restore
        restore_from_scoped_snapshot(&snap, &target).unwrap();

        // Verify restored
        let content = fs::read_to_string(target.join("file.txt")).unwrap();
        assert_eq!(content, "original");
    }

    #[test]
    fn restore_from_scoped_snapshot_removes_new_file() {
        let tmp = TempDir::new().unwrap();
        let snap = tmp.path().join("snap");
        let target = tmp.path().join("target");
        fs::create_dir_all(&target).unwrap();
        fs::create_dir_all(&snap).unwrap();

        // Create a file during migration (didn't exist before)
        fs::write(target.join("new-file.txt"), "new").unwrap();

        // Manifest says this file didn't exist before
        let items = vec![SnapshotItem {
            target_path: "new-file.txt".to_string(),
            existed_before: false,
            backup_path: String::new(),
        }];
        fs::write(snap.join("snapshot.json"), serde_json::to_string(&items).unwrap()).unwrap();

        // Restore
        restore_from_scoped_snapshot(&snap, &target).unwrap();

        // Verify removed
        assert!(!target.join("new-file.txt").exists());
    }

    #[test]
    fn restore_no_manifest_errors() {
        let tmp = TempDir::new().unwrap();
        let snap = tmp.path().join("snap");
        let target = tmp.path().join("target");
        fs::create_dir_all(&snap).unwrap();

        let result = restore_from_scoped_snapshot(&snap, &target);
        assert!(result.is_err());
    }
}
