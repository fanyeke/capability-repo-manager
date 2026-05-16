use domain::CapabilityResource;
use migration_engine::planner;

fn make_resource(
    id: &str,
    r#type: &str,
    name: &str,
    source_path: Option<&str>,
    pack_id: Option<&str>,
    repo_id: Option<&str>,
) -> CapabilityResource {
    CapabilityResource {
        id: id.to_string(),
        repo_id: repo_id.map(|s| s.to_string()),
        pack_id: pack_id.map(|s| s.to_string()),
        r#type: r#type.to_string(),
        name: name.to_string(),
        source_path: source_path.map(|s| s.to_string()),
        scope: "project".to_string(),
        tracked_by_git: true,
        content_hash: Some(format!("hash_{}", id)),
        metadata_json: None,
        error_message: None,
    }
}

fn make_pack_resource(id: &str, r#type: &str, name: &str, source_path: &str) -> CapabilityResource {
    make_resource(id, r#type, name, Some(source_path), Some("pack-1"), None)
}

fn make_repo_resource(id: &str, r#type: &str, name: &str, source_path: &str) -> CapabilityResource {
    make_resource(id, r#type, name, Some(source_path), None, Some("repo-1"))
}

#[test]
fn all_new_resources_are_classified_as_add() {
    let pack_resources = vec![
        make_pack_resource("r1", "skill", "my-skill", "skills/my-skill/"),
        make_pack_resource("r2", "mcp", "my-server", ".claude/mcp.json"),
    ];
    let target_resources = vec![];

    let plan = planner::build_plan(&pack_resources, &target_resources, "pack-1", "repo-1");

    assert_eq!(plan.items.len(), 2);
    assert!(plan.conflicts.is_empty());

    for item in &plan.items {
        assert_eq!(item.action, "add");
        assert_eq!(item.status, "pending");
    }
}

#[test]
fn matching_type_and_name_is_classified_as_overwrite() {
    let pack_resources = vec![make_pack_resource("r1", "skill", "my-skill", "skills/my-skill/")];
    let target_resources = vec![make_repo_resource("r2", "skill", "my-skill", "skills/my-skill/")];

    let plan = planner::build_plan(&pack_resources, &target_resources, "pack-1", "repo-1");

    assert_eq!(plan.items.len(), 1);
    assert_eq!(plan.items[0].action, "overwrite");
    assert_eq!(plan.items[0].resource_id, "r1");
}

#[test]
fn same_type_and_name_with_different_path_detects_conflict() {
    let pack_resources = vec![make_pack_resource("r1", "hook", "my-hook", ".claude/settings.json")];
    let target_resources = vec![make_repo_resource("r2", "hook", "my-hook", ".claude/settings.local.json")];

    let plan = planner::build_plan(&pack_resources, &target_resources, "pack-1", "repo-1");

    assert!(!plan.conflicts.is_empty());
    assert!(plan.conflicts.iter().any(|c| c.resource_name == "my-hook"));
}

#[test]
fn partial_overlap_between_pack_and_target() {
    let pack_resources = vec![
        make_pack_resource("r1", "skill", "skill-a", "skills/skill-a/"),
        make_pack_resource("r2", "skill", "skill-b", "skills/skill-b/"),
        make_pack_resource("r3", "rule", "coding-style", ".claude/rules/coding.md"),
    ];
    let target_resources = vec![
        make_repo_resource("r4", "skill", "skill-a", "skills/skill-a/"),
        make_repo_resource("r5", "rule", "security", ".claude/rules/security.md"),
    ];

    let plan = planner::build_plan(&pack_resources, &target_resources, "pack-1", "repo-1");

    // skill-a: overwrite (exists in both)
    let skill_a = plan.items.iter().find(|i| i.resource_id == "r1").unwrap();
    assert_eq!(skill_a.action, "overwrite");

    // skill-b: add (only in pack)
    let skill_b = plan.items.iter().find(|i| i.resource_id == "r2").unwrap();
    assert_eq!(skill_b.action, "add");

    // coding-style rule: add (only in pack)
    let rule = plan.items.iter().find(|i| i.resource_id == "r3").unwrap();
    assert_eq!(rule.action, "add");
}

#[test]
fn plan_has_unique_plan_id() {
    let pack = vec![make_pack_resource("r1", "skill", "s", "skills/s/")];
    let target = vec![];

    let plan1 = planner::build_plan(&pack, &target, "pack-1", "repo-1");
    let plan2 = planner::build_plan(&pack, &target, "pack-1", "repo-2");

    assert_ne!(plan1.plan_id, plan2.plan_id);
}

#[test]
fn plan_records_source_and_target_ids() {
    let pack = vec![make_pack_resource("r1", "skill", "s", "skills/s/")];
    let target = vec![];

    let plan = planner::build_plan(&pack, &target, "pack-uuid", "repo-uuid");

    assert_eq!(plan.source_type, "pack");
    assert_eq!(plan.source_id, "pack-uuid");
    assert_eq!(plan.target_repo_id, "repo-uuid");
}

#[test]
fn missing_dependencies_are_reported() {
    // Skill that references a hook that doesn't exist in pack or target
    let mut skill = make_pack_resource("r1", "skill", "dependent-skill", "skills/dep/");
    skill.metadata_json =
        Some(r#"{"dependencies":[{"type":"hook","name":"missing-hook","required":true}]}"#.to_string());

    let pack_resources = vec![skill];
    let target_resources = vec![];

    let plan = planner::build_plan(&pack_resources, &target_resources, "pack-1", "repo-1");

    assert!(!plan.missing_dependencies.is_empty());
    assert!(plan.missing_dependencies.iter().any(|d| d.name == "missing-hook"));
}

#[test]
fn new_resources_have_target_path_from_source_path() {
    let pack = vec![make_pack_resource("r1", "skill", "new-skill", "skills/new-skill/SKILL.md")];
    let target = vec![];

    let plan = planner::build_plan(&pack, &target, "pack-1", "repo-1");
    let item = &plan.items[0];
    assert_eq!(item.action, "add");
    assert_eq!(item.target_path.as_deref(), Some("skills/new-skill/SKILL.md"));
}

#[test]
fn empty_pack_produces_empty_plan() {
    let plan = planner::build_plan(&[], &[], "pack-1", "repo-1");

    assert!(plan.items.is_empty());
    assert!(plan.conflicts.is_empty());
    assert!(plan.missing_dependencies.is_empty());
}

#[test]
fn multiple_conflicts_for_same_resource_name() {
    let pack_resources = vec![
        make_pack_resource("r1", "skill", "dupe-name", "skills/a/"),
        make_pack_resource("r2", "mcp", "dupe-name", ".claude/mcp.json"),
    ];
    let target_resources = vec![
        make_repo_resource("r3", "skill", "dupe-name", "skills/b/"),
        make_repo_resource("r4", "mcp", "dupe-name", ".claude/mcp.local.json"),
    ];

    let plan = planner::build_plan(&pack_resources, &target_resources, "pack-1", "repo-1");

    // Both items should be overwrites with possible conflicts
    assert_eq!(plan.items.len(), 2);
    for item in &plan.items {
        assert_eq!(item.action, "overwrite");
    }
}
