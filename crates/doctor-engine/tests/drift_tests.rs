use doctor_engine::drift;
use domain::CapabilityResource;

fn make_resource(id: &str, repo_id: &str, r#type: &str, name: &str, path: &str, hash: &str) -> CapabilityResource {
    CapabilityResource {
        id: id.to_string(),
        repo_id: Some(repo_id.to_string()),
        pack_id: None,
        r#type: r#type.to_string(),
        name: name.to_string(),
        source_path: Some(path.to_string()),
        scope: "project".to_string(),
        tracked_by_git: true,
        content_hash: Some(hash.to_string()),
        metadata_json: None,
        error_message: None,
    }
}

#[test]
fn drift_identical_sets() {
    let a = vec![make_resource("r1", "repo-a", "skill", "my-skill", "skills/my-skill/SKILL.md", "abc123")];
    let b = vec![make_resource("r2", "repo-b", "skill", "my-skill", "skills/my-skill/SKILL.md", "abc123")];

    let result = drift::compare(&a, &b);
    assert_eq!(result.same.len(), 1);
    assert!(result.missing.is_empty());
    assert!(result.extra.is_empty());
    assert!(result.modified.is_empty());
}

#[test]
fn drift_missing_in_target() {
    let a = vec![
        make_resource("r1", "repo-a", "skill", "skill-a", "skills/a/SKILL.md", "aaa"),
        make_resource("r2", "repo-a", "skill", "skill-b", "skills/b/SKILL.md", "bbb"),
    ];
    let b = vec![
        make_resource("r3", "repo-b", "skill", "skill-a", "skills/a/SKILL.md", "aaa"),
    ];

    let result = drift::compare(&a, &b);
    // a is source, b is target. "missing" = in source but not in target (target lacks it)
    assert_eq!(result.missing.len(), 1);
    assert_eq!(result.missing[0].name, "skill-b");
    assert_eq!(result.same.len(), 1);
    assert!(result.extra.is_empty());
    assert!(result.modified.is_empty());
}

#[test]
fn drift_extra_in_target() {
    let a = vec![make_resource("r1", "repo-a", "skill", "skill-a", "skills/a/SKILL.md", "aaa")];
    let b = vec![
        make_resource("r3", "repo-b", "skill", "skill-a", "skills/a/SKILL.md", "aaa"),
        make_resource("r4", "repo-b", "skill", "skill-b", "skills/b/SKILL.md", "bbb"),
    ];

    let result = drift::compare(&a, &b);
    assert_eq!(result.same.len(), 1);
    assert!(result.missing.is_empty());
    assert_eq!(result.extra.len(), 1);
    assert!(result.modified.is_empty());
}

#[test]
fn drift_modified_content() {
    let a = vec![make_resource("r1", "repo-a", "skill", "my-skill", "skills/my/SKILL.md", "aaa")];
    let b = vec![make_resource("r2", "repo-b", "skill", "my-skill", "skills/my/SKILL.md", "bbb")];

    let result = drift::compare(&a, &b);
    assert!(result.same.is_empty());
    assert!(result.missing.is_empty());
    assert!(result.extra.is_empty());
    assert_eq!(result.modified.len(), 1);
    assert_eq!(result.modified[0].name, "my-skill");
}

#[test]
fn drift_different_types_are_separate() {
    let a = vec![make_resource("r1", "repo-a", "skill", "my-skill", "skills/my/SKILL.md", "aaa")];
    let b = vec![make_resource("r2", "repo-b", "mcp", "my-skill", "mcp.json", "aaa")];

    let result = drift::compare(&a, &b);
    // Different types are treated as different resources
    assert_eq!(result.missing.len(), 1); // skill/my-skill missing in b
    assert_eq!(result.extra.len(), 1);   // mcp/my-skill extra in b
    assert!(result.same.is_empty());
    assert!(result.modified.is_empty());
}

#[test]
fn drift_empty_both() {
    let result = drift::compare(&[], &[]);
    assert!(result.same.is_empty());
    assert!(result.missing.is_empty());
    assert!(result.extra.is_empty());
    assert!(result.modified.is_empty());
}

#[test]
fn drift_mixed_scenario() {
    let a = vec![
        make_resource("r1", "repo-a", "skill", "skill-keep", "skills/keep/SKILL.md", "111"),
        make_resource("r2", "repo-a", "skill", "skill-mod", "skills/mod/SKILL.md", "old"),
        make_resource("r3", "repo-a", "skill", "skill-removed", "skills/rem/SKILL.md", "333"),
        make_resource("r4", "repo-a", "mcp", "server1", "mcp.json", "m1"),
    ];
    let b = vec![
        make_resource("r5", "repo-b", "skill", "skill-keep", "skills/keep/SKILL.md", "111"),
        make_resource("r6", "repo-b", "skill", "skill-mod", "skills/mod/SKILL.md", "new"),
        make_resource("r7", "repo-b", "skill", "skill-new", "skills/new/SKILL.md", "777"),
    ];

    let result = drift::compare(&a, &b);
    assert_eq!(result.same.len(), 1, "skill-keep should be same");
    assert_eq!(result.modified.len(), 1, "skill-mod should be modified");
    assert_eq!(result.missing.len(), 2, "skill-removed + mcp/server1 missing in b");
    assert_eq!(result.extra.len(), 1, "skill-new extra in b");
}
