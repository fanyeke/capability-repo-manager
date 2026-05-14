use migration_engine::rollback;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn setup_target_with_content(dir: &TempDir) -> PathBuf {
    let target = dir.path().join("target-repo");
    fs::create_dir_all(target.join("skills/my-skill")).unwrap();
    fs::create_dir_all(target.join(".claude")).unwrap();
    fs::write(
        target.join("skills/my-skill/SKILL.md"),
        "original content",
    )
    .unwrap();
    fs::write(
        target.join(".claude/settings.json"),
        r#"{"original": true}"#,
    )
    .unwrap();
    target
}

#[test]
fn create_snapshot_copies_all_files() {
    let tmp = tempfile::tempdir().unwrap();
    let target = setup_target_with_content(&tmp);

    let snapshot_path = rollback::create_snapshot(&target).unwrap();

    assert!(snapshot_path.contains("snapshot"));
    let snapshot = PathBuf::from(&snapshot_path);
    assert!(snapshot.exists());
    assert!(snapshot.join("skills/my-skill/SKILL.md").exists());
    assert!(snapshot.join(".claude/settings.json").exists());

    let content = fs::read_to_string(snapshot.join("skills/my-skill/SKILL.md")).unwrap();
    assert_eq!(content, "original content");
}

#[test]
fn rollback_restores_original_state() {
    let tmp = tempfile::tempdir().unwrap();
    let target = setup_target_with_content(&tmp);

    let snapshot_path = rollback::create_snapshot(&target).unwrap();

    // Modify target files
    fs::write(target.join("skills/my-skill/SKILL.md"), "modified content").unwrap();
    fs::create_dir_all(target.join("skills/new-skill")).unwrap();
    fs::write(target.join("skills/new-skill/SKILL.md"), "new file").unwrap();

    // Rollback
    rollback::rollback_to_snapshot(&snapshot_path, &target).unwrap();

    // Verify original state restored
    let content = fs::read_to_string(target.join("skills/my-skill/SKILL.md")).unwrap();
    assert_eq!(content, "original content");

    // New files should be removed
    assert!(!target.join("skills/new-skill").exists());
}

#[test]
fn rollback_removes_files_added_during_migration() {
    let tmp = tempfile::tempdir().unwrap();
    let target = setup_target_with_content(&tmp);

    let snapshot_path = rollback::create_snapshot(&target).unwrap();

    // Add new files (simulating migration)
    fs::write(target.join("skills/my-skill/SKILL.md"), "overwritten").unwrap();
    fs::create_dir_all(target.join("skills/added-skill")).unwrap();
    fs::write(target.join("skills/added-skill/SKILL.md"), "added").unwrap();
    fs::remove_file(target.join(".claude/settings.json")).unwrap();

    rollback::rollback_to_snapshot(&snapshot_path, &target).unwrap();

    // Removed file restored
    assert!(target.join(".claude/settings.json").exists());
    // Added file removed
    assert!(!target.join("skills/added-skill").exists());
    // Modified file restored
    let content = fs::read_to_string(target.join("skills/my-skill/SKILL.md")).unwrap();
    assert_eq!(content, "original content");
}

#[test]
fn snapshot_verify_detects_intact_snapshot() {
    let tmp = tempfile::tempdir().unwrap();
    let target = setup_target_with_content(&tmp);
    let snapshot_path = rollback::create_snapshot(&target).unwrap();

    assert!(rollback::verify_snapshot(&snapshot_path).is_ok());
}

#[test]
fn snapshot_verify_detects_corrupted_snapshot() {
    let tmp = tempfile::tempdir().unwrap();
    let target = setup_target_with_content(&tmp);
    let snapshot_path = rollback::create_snapshot(&target).unwrap();

    // Corrupt the snapshot by removing all its contents
    for entry in fs::read_dir(&snapshot_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(&path).unwrap();
        } else {
            fs::remove_file(&path).unwrap();
        }
    }

    assert!(rollback::verify_snapshot(&snapshot_path).is_err());
}

#[test]
fn rollback_cleans_up_snapshot_after_restore() {
    let tmp = tempfile::tempdir().unwrap();
    let target = setup_target_with_content(&tmp);
    let snapshot_path = rollback::create_snapshot(&target).unwrap();

    rollback::rollback_to_snapshot(&snapshot_path, &target).unwrap();

    // Snapshot should be cleaned up after successful rollback
    assert!(!PathBuf::from(&snapshot_path).exists());
}

#[test]
fn rollback_fails_if_snapshot_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let target = setup_target_with_content(&tmp);

    let result = rollback::rollback_to_snapshot("/nonexistent/snapshot/path", &target);
    assert!(result.is_err());
}
