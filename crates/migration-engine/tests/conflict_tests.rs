use domain::CapabilityResource;
use migration_engine::conflict;

fn make_resource(
    id: &str,
    r#type: &str,
    name: &str,
    source_path: Option<&str>,
    content_hash: &str,
) -> CapabilityResource {
    CapabilityResource {
        id: id.to_string(),
        repo_id: Some("repo-1".to_string()),
        pack_id: None,
        r#type: r#type.to_string(),
        name: name.to_string(),
        source_path: source_path.map(|s| s.to_string()),
        scope: "project".to_string(),
        tracked_by_git: true,
        content_hash: Some(content_hash.to_string()),
        metadata_json: None,
        error_message: None,
    }
}

fn make_pack_resource(
    id: &str,
    r#type: &str,
    name: &str,
    source_path: &str,
    content_hash: &str,
) -> CapabilityResource {
    CapabilityResource {
        id: id.to_string(),
        repo_id: None,
        pack_id: Some("pack-1".to_string()),
        r#type: r#type.to_string(),
        name: name.to_string(),
        source_path: Some(source_path.to_string()),
        scope: "project".to_string(),
        tracked_by_git: false,
        content_hash: Some(content_hash.to_string()),
        metadata_json: None,
        error_message: None,
    }
}

#[test]
fn detect_same_name_same_type_same_path_conflict() {
    let pack = vec![make_pack_resource(
        "r1",
        "skill",
        "my-skill",
        "skills/my-skill/",
        "hash_a",
    )];
    let target = vec![make_resource(
        "r2",
        "skill",
        "my-skill",
        Some("skills/my-skill/"),
        "hash_b",
    )];

    let conflicts = conflict::detect_conflicts(&pack, &target);

    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].resource_name, "my-skill");
    assert_eq!(conflicts[0].resource_type, "skill");
    assert_eq!(conflicts[0].reason, "overwrite");
    assert!(conflicts[0]
        .recommended_actions
        .contains(&"overwrite".to_string()));
    assert!(conflicts[0]
        .recommended_actions
        .contains(&"skip".to_string()));
}

#[test]
fn detect_same_name_same_type_different_path_conflict() {
    let pack = vec![make_pack_resource(
        "r1",
        "hook",
        "pre-commit",
        ".claude/settings.json",
        "hash_a",
    )];
    let target = vec![make_resource(
        "r2",
        "hook",
        "pre-commit",
        Some(".claude/settings.local.json"),
        "hash_b",
    )];

    let conflicts = conflict::detect_conflicts(&pack, &target);

    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].resource_name, "pre-commit");
    assert_eq!(conflicts[0].reason, "path_conflict");
    assert!(conflicts[0]
        .recommended_actions
        .contains(&"rename".to_string()));
}

#[test]
fn detect_same_name_same_type_same_content_no_conflict() {
    let pack = vec![make_pack_resource(
        "r1",
        "rule",
        "coding-style",
        ".claude/rules/coding.md",
        "same_hash",
    )];
    let target = vec![make_resource(
        "r2",
        "rule",
        "coding-style",
        Some(".claude/rules/coding.md"),
        "same_hash",
    )];

    let conflicts = conflict::detect_conflicts(&pack, &target);

    // Same content hash → idempotent, no real conflict
    assert!(conflicts.is_empty());
}

#[test]
fn detect_different_name_same_path_conflict() {
    let pack = vec![make_pack_resource(
        "r1",
        "mcp",
        "server-a",
        ".claude/mcp.json",
        "hash_a",
    )];
    let target = vec![make_resource(
        "r2",
        "mcp",
        "server-b",
        Some(".claude/mcp.json"),
        "hash_b",
    )];

    let conflicts = conflict::detect_conflicts(&pack, &target);

    // Different names share the same source path → path conflict
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].reason, "path_conflict");
}

#[test]
fn no_conflicts_when_no_overlap() {
    let pack = vec![make_pack_resource(
        "r1", "skill", "skill-a", "skills/a/", "hash_a",
    )];
    let target = vec![make_resource(
        "r2",
        "skill",
        "skill-b",
        Some("skills/b/"),
        "hash_b",
    )];

    let conflicts = conflict::detect_conflicts(&pack, &target);

    assert!(conflicts.is_empty());
}

#[test]
fn no_conflicts_with_empty_target() {
    let pack = vec![make_pack_resource(
        "r1", "skill", "skill-a", "skills/a/", "hash_a",
    )];

    let conflicts = conflict::detect_conflicts(&pack, &[]);

    assert!(conflicts.is_empty());
}

#[test]
fn multiple_conflicts_across_types() {
    let pack = vec![
        make_pack_resource("r1", "skill", "shared-name", "skills/a/", "hash1"),
        make_pack_resource("r2", "mcp", "shared-name", ".claude/mcp.json", "hash2"),
    ];
    let target = vec![
        make_resource("r3", "skill", "shared-name", Some("skills/a/"), "hash3"),
        make_resource(
            "r4",
            "mcp",
            "shared-name",
            Some(".claude/mcp.local.json"),
            "hash4",
        ),
    ];

    let conflicts = conflict::detect_conflicts(&pack, &target);

    assert_eq!(conflicts.len(), 2);
}

#[test]
fn conflict_recommended_actions_include_all_relevant_strategies() {
    let pack = vec![make_pack_resource(
        "r1",
        "skill",
        "my-skill",
        "skills/my-skill/",
        "hash_a",
    )];
    let target = vec![make_resource(
        "r2",
        "skill",
        "my-skill",
        Some("skills/my-skill/"),
        "hash_b",
    )];

    let conflicts = conflict::detect_conflicts(&pack, &target);

    // Overwrite conflict should recommend skip + overwrite + rename
    let actions = &conflicts[0].recommended_actions;
    assert!(actions.iter().any(|a| a == "skip"));
    assert!(actions.iter().any(|a| a == "overwrite"));
    assert!(actions.iter().any(|a| a == "rename"));
}
