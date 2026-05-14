use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub fn create_snapshot(target_dir: &Path) -> Result<String, domain::AppError> {
    let snapshot_name = format!("snapshot-{}", Uuid::new_v4());
    let snapshot_dir = target_dir
        .parent()
        .unwrap_or_else(|| Path::new("/tmp"))
        .join(&snapshot_name);

    copy_dir_recursive(target_dir, &snapshot_dir).map_err(|e| {
        domain::AppError::Migration(format!("Failed to create snapshot: {}", e))
    })?;

    Ok(snapshot_dir.to_string_lossy().to_string())
}

pub fn rollback_to_snapshot(snapshot_path: &str, target_dir: &Path) -> Result<(), domain::AppError> {
    let snapshot = PathBuf::from(snapshot_path);

    if !snapshot.exists() {
        return Err(domain::AppError::Migration(format!(
            "Snapshot not found: {}",
            snapshot_path
        )));
    }

    // Remove current target contents (but not the directory itself)
    if target_dir.exists() {
        remove_dir_contents(target_dir).map_err(|e| {
            domain::AppError::Migration(format!("Failed to clean target directory: {}", e))
        })?;
    }

    // Restore from snapshot
    copy_dir_recursive(&snapshot, target_dir).map_err(|e| {
        domain::AppError::Migration(format!("Failed to restore from snapshot: {}", e))
    })?;

    // Clean up snapshot after successful rollback
    let _ = fs::remove_dir_all(&snapshot);

    Ok(())
}

pub fn verify_snapshot(snapshot_path: &str) -> Result<(), domain::AppError> {
    let snapshot = PathBuf::from(snapshot_path);

    if !snapshot.exists() {
        return Err(domain::AppError::Migration(
            "Snapshot directory does not exist".to_string(),
        ));
    }

    if !snapshot.is_dir() {
        return Err(domain::AppError::Migration(
            "Snapshot path is not a directory".to_string(),
        ));
    }

    // Check that snapshot has at least some content
    let entries: Vec<_> = fs::read_dir(&snapshot)
        .map_err(|e| {
            domain::AppError::Migration(format!("Cannot read snapshot directory: {}", e))
        })?
        .collect();

    if entries.is_empty() {
        return Err(domain::AppError::Migration(
            "Snapshot directory is empty".to_string(),
        ));
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

    #[test]
    fn snapshot_name_is_unique() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("target");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("test.txt"), "hello").unwrap();

        let s1 = create_snapshot(&target).unwrap();
        let s2 = create_snapshot(&target).unwrap();

        assert_ne!(s1, s2);
        // Clean up
        let _ = fs::remove_dir_all(PathBuf::from(&s1));
        let _ = fs::remove_dir_all(PathBuf::from(&s2));
    }
}
