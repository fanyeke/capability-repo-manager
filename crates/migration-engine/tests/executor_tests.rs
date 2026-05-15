use domain::{CapabilityResource, MigrationPlan, MigrationPlanItem};
use migration_engine::executor;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn make_plan(
    plan_id: &str,
    items: Vec<(&str, &str, Option<&str>, Option<&str>)>,
) -> MigrationPlan {
    MigrationPlan {
        plan_id: plan_id.to_string(),
        source_type: "pack".to_string(),
        source_id: "pack-1".to_string(),
        target_repo_id: "repo-1".to_string(),
        items: items
            .into_iter()
            .map(|(rid, action, src, tgt)| MigrationPlanItem {
                resource_id: rid.to_string(),
                action: action.to_string(),
                source_path: src.map(|s| s.to_string()),
                target_path: tgt.map(|s| s.to_string()),
                status: "pending".to_string(),
            })
            .collect(),
        conflicts: vec![],
        missing_dependencies: vec![],
    }
}

fn make_resource(id: &str, r#type: &str, name: &str, source_path: &str) -> CapabilityResource {
    CapabilityResource {
        id: id.to_string(),
        repo_id: None,
        pack_id: Some("pack-1".to_string()),
        r#type: r#type.to_string(),
        name: name.to_string(),
        source_path: Some(source_path.to_string()),
        scope: "project".to_string(),
        tracked_by_git: false,
        content_hash: Some(format!("hash_{}", id)),
        metadata_json: None,
        error_message: None,
    }
}

fn setup_pack_dir(dir: &TempDir) -> PathBuf {
    let pack_dir = dir.path().join("pack-root");
    fs::create_dir_all(pack_dir.join("skills/my-skill")).unwrap();
    fs::create_dir_all(pack_dir.join(".claude")).unwrap();
    fs::write(
        pack_dir.join("skills/my-skill/SKILL.md"),
        "# My Skill\nDescription",
    )
    .unwrap();
    fs::write(
        pack_dir.join(".claude/settings.json"),
        r#"{"hooks": {"PreToolUse": [{"command": "echo hi"}]}}"#,
    )
    .unwrap();
    pack_dir
}

fn setup_target_dir(dir: &TempDir) -> PathBuf {
    let target_dir = dir.path().join("target-repo");
    fs::create_dir_all(target_dir.join("skills/existing-skill")).unwrap();
    fs::create_dir_all(target_dir.join(".claude")).unwrap();
    fs::write(
        target_dir.join("skills/existing-skill/SKILL.md"),
        "# Existing\nContent",
    )
    .unwrap();
    target_dir
}

#[test]
fn execute_add_actions_writes_files_to_target() {
    let tmp = tempfile::tempdir().unwrap();
    let pack_dir = setup_pack_dir(&tmp);
    let target_dir = setup_target_dir(&tmp);

    let plan = make_plan(
        "plan-1",
        vec![("r1", "add", Some("skills/my-skill/"), None)],
    );

    let pack_resources = vec![make_resource("r1", "skill", "my-skill", "skills/my-skill/")];

    let report = executor::execute_plan(&plan, &pack_resources, &pack_dir, &target_dir).unwrap();

    assert_eq!(report.status, "success");
    assert_eq!(report.summary.added, 1);
    assert!(target_dir.join("skills/my-skill/SKILL.md").exists());
}

#[test]
fn execute_overwrite_replaces_existing_file() {
    let tmp = tempfile::tempdir().unwrap();
    let pack_dir = setup_pack_dir(&tmp);
    let target_dir = setup_target_dir(&tmp);

    // Pre-existing file should be overwritten
    let plan = make_plan(
        "plan-2",
        vec![(
            "r1",
            "overwrite",
            Some("skills/my-skill/"),
            Some("skills/existing-skill/"),
        )],
    );

    let pack_resources = vec![make_resource("r1", "skill", "my-skill", "skills/my-skill/")];

    let report = executor::execute_plan(&plan, &pack_resources, &pack_dir, &target_dir).unwrap();

    assert_eq!(report.status, "success");
    assert_eq!(report.summary.overwritten, 1);
    // Content should now be from pack
    let content = fs::read_to_string(target_dir.join("skills/existing-skill/SKILL.md")).unwrap();
    assert!(content.contains("My Skill"));
}

#[test]
fn execute_skip_does_not_modify_target() {
    let tmp = tempfile::tempdir().unwrap();
    let pack_dir = setup_pack_dir(&tmp);
    let target_dir = setup_target_dir(&tmp);

    let plan = make_plan(
        "plan-3",
        vec![(
            "r1",
            "skip",
            Some("skills/my-skill/"),
            Some("skills/existing-skill/"),
        )],
    );

    let pack_resources = vec![make_resource("r1", "skill", "my-skill", "skills/my-skill/")];

    let report = executor::execute_plan(&plan, &pack_resources, &pack_dir, &target_dir).unwrap();

    assert_eq!(report.status, "success");
    assert_eq!(report.summary.skipped, 1);
    // Content should remain unchanged
    let content = fs::read_to_string(target_dir.join("skills/existing-skill/SKILL.md")).unwrap();
    assert!(content.contains("Existing"));
}

#[test]
fn execute_with_rename_strategy_creates_renamed_copy() {
    let tmp = tempfile::tempdir().unwrap();
    let pack_dir = setup_pack_dir(&tmp);
    let target_dir = setup_target_dir(&tmp);

    let plan = make_plan(
        "plan-4",
        vec![(
            "r1",
            "rename",
            Some("skills/my-skill/"),
            Some("skills/my-skill-1/"),
        )],
    );

    let pack_resources = vec![make_resource("r1", "skill", "my-skill", "skills/my-skill/")];

    let report = executor::execute_plan(&plan, &pack_resources, &pack_dir, &target_dir).unwrap();

    assert_eq!(report.status, "success");
    assert_eq!(report.summary.added, 1);
    assert!(target_dir.join("skills/my-skill-1/SKILL.md").exists());
}

#[test]
fn execute_report_records_per_item_status() {
    let tmp = tempfile::tempdir().unwrap();
    let pack_dir = setup_pack_dir(&tmp);
    let target_dir = setup_target_dir(&tmp);

    let plan = make_plan(
        "plan-5",
        vec![
            ("r1", "add", Some("skills/my-skill/"), None),
            ("r2", "skip", None, None),
        ],
    );

    let pack_resources = vec![make_resource("r1", "skill", "my-skill", "skills/my-skill/")];

    let report = executor::execute_plan(&plan, &pack_resources, &pack_dir, &target_dir).unwrap();

    assert_eq!(report.items.len(), 2);

    let add_item = report.items.iter().find(|i| i.action == "add").unwrap();
    assert_eq!(add_item.status, "success");

    let skip_item = report.items.iter().find(|i| i.action == "skip").unwrap();
    assert_eq!(skip_item.status, "skipped");
}

#[test]
fn execute_with_missing_source_file_reports_error() {
    let tmp = tempfile::tempdir().unwrap();
    let pack_dir = setup_pack_dir(&tmp);
    let target_dir = setup_target_dir(&tmp);

    let plan = make_plan(
        "plan-6",
        vec![("r1", "add", Some("skills/nonexistent/"), None)],
    );

    let pack_resources = vec![make_resource("r1", "skill", "nonexistent", "skills/nonexistent/")];

    let report = executor::execute_plan(&plan, &pack_resources, &pack_dir, &target_dir).unwrap();

    assert_eq!(report.status, "failed");
    assert_eq!(report.summary.failed, 1);

    let item = &report.items[0];
    assert_eq!(item.status, "failed");
    assert!(item.error.is_some());
}

#[test]
fn execute_mixed_results_has_correct_summary() {
    let tmp = tempfile::tempdir().unwrap();
    let pack_dir = setup_pack_dir(&tmp);
    let target_dir = setup_target_dir(&tmp);

    let plan = make_plan(
        "plan-7",
        vec![
            ("r1", "add", Some("skills/my-skill/"), None),
            ("r2", "skip", None, None),
            ("r3", "add", Some("skills/nonexistent/"), None),
        ],
    );

    let pack_resources = vec![
        make_resource("r1", "skill", "my-skill", "skills/my-skill/"),
        make_resource("r3", "skill", "nonexistent", "skills/nonexistent/"),
    ];

    let report = executor::execute_plan(&plan, &pack_resources, &pack_dir, &target_dir).unwrap();

    assert_eq!(report.status, "partial_failure");
    assert_eq!(report.summary.added, 1);
    assert_eq!(report.summary.skipped, 1);
    assert_eq!(report.summary.failed, 1);
}
