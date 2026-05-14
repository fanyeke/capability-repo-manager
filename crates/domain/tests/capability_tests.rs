use domain::CapabilityResource;
use uuid::Uuid;

fn make_resource(repo_id: Option<&str>, pack_id: Option<&str>, rtype: &str, name: &str) -> CapabilityResource {
    CapabilityResource {
        id: Uuid::new_v4().to_string(),
        repo_id: repo_id.map(|s| s.to_string()),
        pack_id: pack_id.map(|s| s.to_string()),
        r#type: rtype.to_string(),
        name: name.to_string(),
        source_path: Some(format!("skills/{}", name)),
        scope: "project".to_string(),
        tracked_by_git: true,
        content_hash: Some("abc123".to_string()),
        metadata_json: None,
        error_message: None,
    }
}

#[test]
fn test_create_skill_resource() {
    let repo_id = Uuid::new_v4().to_string();
    let resource = make_resource(Some(&repo_id), None, "skill", "ui-code-review");

    assert_eq!(resource.r#type, "skill");
    assert_eq!(resource.name, "ui-code-review");
    assert_eq!(resource.repo_id, Some(repo_id));
    assert_eq!(resource.pack_id, None);
    assert_eq!(resource.scope, "project");
    assert!(resource.tracked_by_git);
}

#[test]
fn test_valid_resource_types() {
    let valid_types = vec![
        "skill", "mcp", "hook", "rule", "agent", "command", "plugin", "settings", "contextDoc",
    ];
    let repo_id = Uuid::new_v4().to_string();
    for rt in valid_types {
        let resource = make_resource(Some(&repo_id), None, rt, "test-resource");
        assert_eq!(resource.r#type, rt);
    }
}

#[test]
fn test_resource_with_parse_error() {
    let resource = CapabilityResource {
        id: Uuid::new_v4().to_string(),
        repo_id: Some(Uuid::new_v4().to_string()),
        pack_id: None,
        r#type: "mcp".to_string(),
        name: "broken-mcp".to_string(),
        source_path: Some(".claude/mcp.json".to_string()),
        scope: "project".to_string(),
        tracked_by_git: true,
        content_hash: None,
        metadata_json: None,
        error_message: Some("Failed to parse JSON: unexpected token at line 3".to_string()),
    };

    assert!(resource.error_message.is_some());
    assert!(resource.content_hash.is_none());
}

#[test]
fn test_resource_valid_scopes() {
    let valid_scopes = vec!["project", "local", "user", "inherited", "unknown"];
    let repo_id = Uuid::new_v4().to_string();
    for scope in valid_scopes {
        let resource = CapabilityResource {
            id: Uuid::new_v4().to_string(),
            repo_id: Some(repo_id.clone()),
            pack_id: None,
            r#type: "rule".to_string(),
            name: format!("rule-{}", scope),
            source_path: Some(".claude/rules/test.md".to_string()),
            scope: scope.to_string(),
            tracked_by_git: scope != "local",
            content_hash: Some("def456".to_string()),
            metadata_json: None,
            error_message: None,
        };
        assert_eq!(resource.scope, scope);
    }
}

#[test]
fn test_resource_xor_constraint_repo_or_pack() {
    let repo_id = Uuid::new_v4().to_string();
    let pack_id = Uuid::new_v4().to_string();

    // Repo-owned resource: repo_id set, pack_id None
    let repo_resource = make_resource(Some(&repo_id), None, "skill", "repo-skill");
    assert!(repo_resource.repo_id.is_some());
    assert!(repo_resource.pack_id.is_none());

    // Pack-owned resource: pack_id set, repo_id None
    let pack_resource = make_resource(None, Some(&pack_id), "skill", "pack-skill");
    assert!(pack_resource.repo_id.is_none());
    assert!(pack_resource.pack_id.is_some());
}

#[test]
fn test_resource_serde_roundtrip() {
    let resource = CapabilityResource {
        id: Uuid::new_v4().to_string(),
        repo_id: Some(Uuid::new_v4().to_string()),
        pack_id: None,
        r#type: "hook".to_string(),
        name: "prevent-secret-write".to_string(),
        source_path: Some(".claude/settings.json".to_string()),
        scope: "project".to_string(),
        tracked_by_git: true,
        content_hash: Some("sha256:ffee0011".to_string()),
        metadata_json: Some(r#"{"event":"PreToolUse"}"#.to_string()),
        error_message: None,
    };

    let json = serde_json::to_string(&resource).expect("serialize");
    let deserialized: CapabilityResource = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(deserialized.id, resource.id);
    assert_eq!(deserialized.name, resource.name);
    assert_eq!(deserialized.r#type, resource.r#type);
    assert_eq!(deserialized.scope, resource.scope);
    assert_eq!(deserialized.content_hash, resource.content_hash);
    assert_eq!(deserialized.metadata_json, resource.metadata_json);
}

#[test]
fn test_resource_local_scope_implies_not_tracked() {
    let repo_id = Uuid::new_v4().to_string();
    let resource = CapabilityResource {
        id: Uuid::new_v4().to_string(),
        repo_id: Some(repo_id),
        pack_id: None,
        r#type: "settings".to_string(),
        name: "local-settings".to_string(),
        source_path: Some(".claude/settings.local.json".to_string()),
        scope: "local".to_string(),
        tracked_by_git: false,
        content_hash: Some("hash123".to_string()),
        metadata_json: None,
        error_message: None,
    };

    assert_eq!(resource.scope, "local");
    assert!(!resource.tracked_by_git);
}
