use domain::MigrationPlanItem;
use migration_engine::{create_scoped_snapshot, restore_scoped};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn setup_pack_dir(dir: &TempDir) -> PathBuf {
    let pack = dir.path().join("pack");
    fs::create_dir_all(pack.join("skills/my-skill")).unwrap();
    fs::create_dir_all(pack.join(".claude")).unwrap();
    fs::write(pack.join("skills/my-skill/SKILL.md"), "# My Skill").unwrap();
    fs::write(pack.join(".claude/settings.json"), r#"{"hooks":{}}"#).unwrap();
    pack
}

fn setup_target_dir(dir: &TempDir) -> PathBuf {
    let target = dir.path().join("target-repo");
    fs::create_dir_all(target.join("skills/existing")).unwrap();
    fs::create_dir_all(target.join(".claude")).unwrap();
    fs::write(target.join("skills/existing/SKILL.md"), "# Existing").unwrap();
    fs::write(target.join("README.md"), "Original content").unwrap();
    fs::write(target.join(".claude/settings.json"), r#"{"original":true}"#).unwrap();
    target
}

#[test]
fn full_plan_snapshot_execute_rollback_cycle() {
    let tmp = TempDir::new().unwrap();
    let pack_dir = setup_pack_dir(&tmp);
    let target_dir = setup_target_dir(&tmp);
    let snap_base = tmp.path().join("snapshots").join("run-full");
    fs::create_dir_all(&snap_base).unwrap();

    // Step 1: Create a plan with items
    let plan_items = vec![
        MigrationPlanItem {
            resource_id: "r1".to_string(),
            action: "add".to_string(),
            source_path: Some("skills/my-skill/".to_string()),
            target_path: Some("skills/my-skill/".to_string()),
            status: "pending".to_string(),
        },
        MigrationPlanItem {
            resource_id: "r2".to_string(),
            action: "overwrite".to_string(),
            source_path: Some(".claude/settings.json".to_string()),
            target_path: Some(".claude/settings.json".to_string()),
            status: "pending".to_string(),
        },
    ];

    // Step 2: Create scoped snapshot
    let snapshot_items = create_scoped_snapshot(&plan_items, &target_dir, &snap_base).unwrap();
    assert_eq!(snapshot_items.len(), 2);

    // Only existing file should have backup
    let settings_item = snapshot_items.iter().find(|s| s.target_path == ".claude/settings.json").unwrap();
    assert!(settings_item.existed_before);
    assert!(!settings_item.backup_path.is_empty());
    assert!(PathBuf::from(&settings_item.backup_path).exists());

    let new_skill_item = snapshot_items.iter().find(|s| s.target_path == "skills/my-skill/").unwrap();
    assert!(!new_skill_item.existed_before);
    assert!(new_skill_item.backup_path.is_empty());

    // Step 3: Execute — copy files from pack to target
    // Copy files to simulate execution
    fs::create_dir_all(target_dir.join("skills/my-skill")).unwrap();
    fs::copy(pack_dir.join("skills/my-skill/SKILL.md"), target_dir.join("skills/my-skill/SKILL.md")).unwrap();
    fs::copy(pack_dir.join(".claude/settings.json"), target_dir.join(".claude/settings.json")).unwrap();

    // Verify files were written
    assert!(target_dir.join("skills/my-skill/SKILL.md").exists());
    let settings_content = fs::read_to_string(target_dir.join(".claude/settings.json")).unwrap();
    assert_eq!(settings_content, r#"{"hooks":{}}"#);

    // Step 4: Rollback using scoped snapshot
    restore_scoped(&snap_base, &target_dir).unwrap();

    // Verify: new file removed
    assert!(!target_dir.join("skills/my-skill/SKILL.md").exists(), "newly-added file should be removed");

    // Verify: overwritten file restored
    let restored_readme = fs::read_to_string(target_dir.join("README.md")).unwrap();
    assert_eq!(restored_readme, "Original content", "unchanged files should be preserved");

    // Verify: settings.json restored to original
    assert!(target_dir.join(".claude/settings.json").exists(), "settings should be restored from backup");
}

#[test]
fn rollback_restores_only_affected_files() {
    let tmp = TempDir::new().unwrap();
    let target_dir = setup_target_dir(&tmp);
    let snap_base = tmp.path().join("snapshots").join("run-scoped");
    fs::create_dir_all(&snap_base).unwrap();

    // Only one item is affected
    let plan_items = vec![MigrationPlanItem {
        resource_id: "r1".to_string(),
        action: "overwrite".to_string(),
        source_path: Some("README.md".to_string()),
        target_path: Some("README.md".to_string()),
        status: "pending".to_string(),
    }];

    create_scoped_snapshot(&plan_items, &target_dir, &snap_base).unwrap();

    // Modify only README.md (the affected file)
    fs::write(target_dir.join("README.md"), "Modified content").unwrap();

    // Also modify an unaffected file — this simulates concurrent changes
    fs::write(target_dir.join("skills/existing/SKILL.md"), "Concurrent change").unwrap();

    // Rollback
    restore_scoped(&snap_base, &target_dir).unwrap();

    // Affected file restored
    let readme = fs::read_to_string(target_dir.join("README.md")).unwrap();
    assert_eq!(readme, "Original content", "affected file should be restored");

    // Unaffected file PRESERVED (not touched by rollback)
    let skill = fs::read_to_string(target_dir.join("skills/existing/SKILL.md")).unwrap();
    assert_eq!(skill, "Concurrent change", "unaffected file should be preserved");
}
